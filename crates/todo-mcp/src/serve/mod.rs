//! The `serve` subcommand's listener (t71, D20's third slice): the live
//! progress view — every route a fresh render, the port published to a
//! per-user runtime file so a second invocation attaches instead of
//! starting a second listener (U18 shape A, the Python tool's `--serve`
//! mechanics ported), plus the WebSocket push channel: one poller per
//! connection stat-watching the repo's store files, one text frame per
//! change, the page's script reloading on arrival.
//!
//! The module is four files split by responsibility — the spec's D8
//! names one file, and the split is the house pattern once the
//! listener, the HTTP layer, the WS codec and the poller together
//! cross the 600-line target: this file owns the listener (the attach
//! / takeover / publication contract of D1/D3, the accept loop, the
//! connection dispatch, the default-repo tier), `http` the
//! hand-rolled HTTP/1.1 parser and response writer (D4), `ws` the
//! RFC 6455 handshake, frame codec and connection loop (D5), `poll`
//! the stat-polling change detection.
//!
//! Thread-per-connection (D4): concurrent tabs render concurrently —
//! safe because renders are lock-free (D19), which is why the Python
//! tool's single-thread serialization (store-lock contention
//! mid-render) is obsolete here and deliberately not ported. A render
//! never runs on the accept thread; the accept loop itself polls a
//! stop flag between non-blocking accepts so the clean shutdown can
//! join it.
//!
//! The module is parameterized and env-free (D8): the root, the
//! port-file path, the startup-read cwd and the poll interval arrive
//! as values — the binary computes the default port-file path from
//! the home directory and reads the cwd once (the t68 `resolve_by_cwd`
//! precedent). No clock is read: the poller's `mtime` is file data,
//! not wall time, and no response carries a date.
//!
//! The multi-root rule is the Python's, ported: the listener serves
//! the root it was started with; to serve a different root, stop it
//! first (`--port-file` exists for isolation).

use std::fs;
use std::io::{BufReader, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use todo_domain::RepoSlug;
use todo_store::{DataRoot, Registry};

use crate::render;

mod http;
mod poll;
mod ws;

use self::http::{Request, Route};

/// The `/health` marker, byte-identical to the Python tool's (D2): the
/// attach probe's contract — a listener of ours answers exactly this,
/// and the Python driver probes it until t73 retires the Python.
pub const HEALTH_MARKER: &str = "dacode-todo-mcp progress listener";

/// The `/health` body: the marker and one newline.
const HEALTH_BODY: &[u8] = b"dacode-todo-mcp progress listener\n";

/// The failed-render body (D2): the reason is already on the
/// listener's stderr; the response says only that.
const RENDER_FAILED_BODY: &str = "render failed — see the listener's stderr\n";

/// The push poll interval (D5, judgment call (a)): two `stat()`s per
/// connected tab per tick, and sub-second push latency reads as live.
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// How long the accept loop sleeps between non-blocking accepts — the
/// stop flag's observability window, and the worst-case latency of a
/// new connection.
const ACCEPT_POLL: Duration = Duration::from_millis(50);

/// How long the attach probe waits to connect and read (the Python's
/// own bound): a foreign or wedged server reads as dead within it.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// How long a connection waits for the request head — a client that
/// connects and never sends must not hold its thread. The WS
/// connection clears it: its reader is long-lived by design.
const HEADER_TIMEOUT: Duration = Duration::from_secs(10);

/// One `serve` invocation's outcome: attached to a live listener, or
/// the listener this invocation started.
pub enum ServeOutcome {
    /// The published port answered the probe: this invocation attaches
    /// — prints the URL, exits 0 — never a second listener.
    Attached {
        /// The answering listener's URL.
        url: String,
    },
    /// A listener was bound and published; the caller owns it until it
    /// shuts down.
    Started {
        /// The new listener's URL.
        url: String,
        /// The listener: its accept loop and its clean shutdown.
        listener: Listener,
    },
}

/// A running listener: the accept thread plus the stop flag that ends
/// it. Dropping without [`Listener::shutdown`] leaves the thread
/// running — the process exit reaps it — so the binary always calls
/// [`Listener::shutdown`] on the interrupt.
pub struct Listener {
    /// The bound port, as published.
    port: u16,
    /// The accept loop's stop flag.
    stop: Arc<AtomicBool>,
    /// The accept thread.
    accept: JoinHandle<()>,
}

impl Listener {
    /// The clean shutdown (D3, SIGINT): stop accepting, join the accept
    /// thread, then remove the publication — only if it still names
    /// this port (a takeover may have replaced it). SIGTERM is the
    /// default disposition and never reaches here: the process dies,
    /// the stale publication survives, the next invocation takes over.
    pub fn shutdown(self, port_file: &Path) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.accept.join();
        if read_published_port(port_file) == Some(self.port) {
            let _ = fs::remove_file(port_file);
        }
    }
}

/// Starts or attaches to the root's live listener (D1/D3): probe the
/// published port — answering means attach; dead, stale or foreign
/// means take over. The takeover binds `127.0.0.1:0` (never a chosen
/// port; the assigned one is read back from the socket — no port
/// scanning), publishes it with the store's own write discipline, and
/// re-reads the winner: a simultaneous invocation that won the publish
/// race is attached to instead (the double-overwrite interleaving
/// where both pass leaves one orphaned listener on an unadvertised
/// port — benign for a personal tool, recorded in the spec).
///
/// The publication is per-user (U18 shape A's signed wording, the
/// Python tool's path — the two interoperate until t73 retires it),
/// so the listener serves the root it was started with and a
/// different root means stopping first; the root-scoped alternative
/// (`<root>/serve.port`) would give multi-root coexistence and
/// hermetic tests by construction — a flip, not a defect.
///
/// The startup check is the registry, not a render: the listener
/// serves the root, so a root whose registry will not load or holds no
/// repository is a startup error — a wrong root, the Python's
/// fail-fast rationale. A registered repo whose store will not read
/// still starts the listener: that repo's routes answer 500 while
/// `/health` stays 200 (D2's contract, the spec's test 7).
pub fn serve(
    root: &DataRoot,
    port_file: &Path,
    cwd: Option<&Path>,
    poll_interval: Duration,
) -> Result<ServeOutcome, String> {
    if let Some(port) = read_published_port(port_file)
        && listener_answers(port)
    {
        return Ok(ServeOutcome::Attached {
            url: format!("http://127.0.0.1:{port}/"),
        });
    }
    let registry = Registry::load(root)
        .map_err(|error| format!("the data root's registry will not load: {error}"))?;
    if registry.slugs().is_empty() {
        return Err("the data root registers no repositories — nothing to serve".to_owned());
    }
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| format!("bind the listener on 127.0.0.1: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("read the listener's assigned port: {error}"))?
        .port();
    let stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(ConnState {
        root: root.clone(),
        cwd: cwd.map(PathBuf::from),
        poll_interval,
    });
    let accept = spawn_accept_thread(listener, state, stop.clone())?;
    publish_port(port_file, port)?;
    match read_published_port(port_file) {
        // The publish race: a simultaneous invocation won — it serves,
        // this one attaches (the Python's own note).
        Some(winner) if winner != port => {
            stop.store(true, Ordering::SeqCst);
            let _ = accept.join();
            Ok(ServeOutcome::Attached {
                url: format!("http://127.0.0.1:{winner}/"),
            })
        }
        // The publication vanished between write and read: re-publish.
        None => {
            publish_port(port_file, port)?;
            Ok(ServeOutcome::Started {
                url: format!("http://127.0.0.1:{port}/"),
                listener: Listener { port, stop, accept },
            })
        }
        Some(_) => Ok(ServeOutcome::Started {
            url: format!("http://127.0.0.1:{port}/"),
            listener: Listener { port, stop, accept },
        }),
    }
}

/// Spawns the accept loop (D4): non-blocking accepts with the stop
/// flag polled between them, one thread per connection so concurrent
/// tabs render concurrently.
fn spawn_accept_thread(
    listener: TcpListener,
    state: Arc<ConnState>,
    stop: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("the listener cannot poll: {error}"))?;
    thread::Builder::new()
        .name("serve-accept".to_owned())
        .spawn(move || accept_loop(listener, state, stop))
        .map_err(|error| format!("spawn the accept thread: {error}"))
}

/// The accept loop: accept, spawn, repeat — `WouldBlock` sleeps out
/// the poll interval; any other error is printed and slept through
/// (the listener stays up, D2's rule for the whole server applied to
/// the loop itself).
fn accept_loop(listener: TcpListener, state: Arc<ConnState>, stop: Arc<AtomicBool>) {
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        match listener.accept() {
            Ok((stream, _)) => {
                // macOS's accept() inherits the listener's
                // O_NONBLOCK; the connection threads want blocking
                // reads (the WS reader blocks for the connection's
                // life), so the flag is cleared explicitly — a no-op
                // where the platform hands over a blocking socket.
                if let Err(error) = stream.set_nonblocking(false) {
                    eprintln!("todo-mcp serve: the connection cannot block: {error}");
                    continue;
                }
                let state = state.clone();
                let spawned = thread::Builder::new()
                    .name("serve-conn".to_owned())
                    .spawn(move || handle_connection(stream, state));
                if spawned.is_err() {
                    eprintln!("todo-mcp serve: a connection thread could not spawn");
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(ACCEPT_POLL);
            }
            Err(error) => {
                eprintln!("todo-mcp serve: accept failed: {error}");
                thread::sleep(ACCEPT_POLL);
            }
        }
    }
}

/// The per-listener state every connection shares — all the
/// parameterized values (D8).
struct ConnState {
    /// The root this listener serves (D1: the ROOT, every repo in the
    /// registry; the repo is a URL parameter, never a second
    /// listener).
    root: DataRoot,
    /// The startup-read working directory (D2's cwd tier) — `None`
    /// when the binary's read failed.
    cwd: Option<PathBuf>,
    /// The WS poller's interval (D5).
    poll_interval: Duration,
}

/// One connection (D2/D4): parse the request, route, answer. The plain
/// routes render and close; `/ws` upgrades and persists. A failed
/// render answers 500 with the reason on stderr — the listener stays
/// up.
fn handle_connection(stream: TcpStream, state: Arc<ConnState>) {
    let _ = stream.set_read_timeout(Some(HEADER_TIMEOUT));
    let mut reader = BufReader::new(stream);
    let request = match http::read_request(&mut reader) {
        Ok(request) => request,
        Err(http::ReadHeadError::TimedOut) => {
            // An idle or speculative connection dropped at the header
            // timeout — the peer is not waiting for anything, so there
            // is nothing to answer and nothing to log.
            return;
        }
        Err(http::ReadHeadError::Fault(message)) => {
            eprintln!("todo-mcp serve: {message}");
            http::write_plain_response(
                reader.get_mut(),
                400,
                "text/plain; charset=utf-8",
                b"bad request\n",
            );
            return;
        }
    };
    match http::route(&request) {
        Route::Health => {
            http::write_plain_response(
                reader.get_mut(),
                200,
                "text/plain; charset=utf-8",
                HEALTH_BODY,
            );
        }
        Route::NotFound => {
            http::write_plain_response(
                reader.get_mut(),
                404,
                "text/plain; charset=utf-8",
                b"not found\n",
            );
        }
        Route::MethodNotAllowed => {
            // The Python's base class answers 501 for unimplemented
            // methods; 405 is the deliberate correction — the method
            // is known, the routes are GET-only.
            http::write_plain_response(
                reader.get_mut(),
                405,
                "text/plain; charset=utf-8",
                b"method not allowed\n",
            );
        }
        Route::Page => render_route(&state, &request, reader.get_mut(), Surface::Page),
        Route::Json => render_route(&state, &request, reader.get_mut(), Surface::Json),
        Route::Ws => {
            let _ = reader.get_ref().set_read_timeout(None);
            ws::handle_ws_connection(reader, &state, &request);
        }
    }
}

/// Which render surface a route asked for.
enum Surface {
    /// The HTML page.
    Page,
    /// The JSON twin.
    Json,
}

/// The render routes (D2): resolve the repo — the `?repo=` parameter,
/// else the default tier — render fresh, answer 200; any failure is
/// 500 with the reason on stderr (the listener stays up).
fn render_route(state: &ConnState, request: &Request, stream: &mut TcpStream, surface: Surface) {
    let slug = repo_for_request(state, request);
    let rendered = match surface {
        Surface::Page => slug.and_then(|slug| render::page(&state.root, &slug)),
        Surface::Json => slug.and_then(|slug| render::json(&state.root, &slug)),
    };
    match rendered {
        Ok(text) => {
            let content_type = match surface {
                Surface::Page => "text/html; charset=utf-8",
                Surface::Json => "application/json",
            };
            http::write_plain_response(stream, 200, content_type, text.as_bytes());
        }
        Err(message) => {
            eprintln!("todo-mcp serve: the render failed: {message}");
            http::write_plain_response(
                stream,
                500,
                "text/plain; charset=utf-8",
                RENDER_FAILED_BODY.as_bytes(),
            );
        }
    }
}

/// The request's repo (D2): the `?repo=` parameter — a malformed value
/// is a failed render, answered 500 like an unknown one — else the
/// default tier.
fn repo_for_request(state: &ConnState, request: &Request) -> Result<RepoSlug, String> {
    match http::repo_param(request) {
        Some(raw) => RepoSlug::from_str(raw).map_err(|error| error.to_string()),
        None => default_repo(&state.root, state.cwd.as_deref()),
    }
}

/// The default repo for `/` and `/ws` (D2, judgment call (f)): the cwd
/// tier, and — for every way it can fail to answer (no cwd, no match,
/// ambiguity, a store that will not read) — the registry's first slug:
/// a listener must not 500 on `/` because it was spawned from `/tmp`.
fn default_repo(root: &DataRoot, cwd: Option<&Path>) -> Result<RepoSlug, String> {
    if let Ok(slug) = render::resolve_by_cwd(root, cwd) {
        return Ok(slug);
    }
    let registry = Registry::load(root).map_err(|error| error.to_string())?;
    registry
        .slugs()
        .first()
        .map(|slug| (*slug).clone())
        .ok_or_else(|| "the registry holds no repositories".to_owned())
}

/// Reads the published port (D3): digits only — anything else
/// (absent, empty, foreign bytes, a number too large for a port)
/// reads as not published, which is the takeover path.
fn read_published_port(path: &Path) -> Option<u16> {
    let text = fs::read_to_string(path).ok()?;
    let text = text.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// Publishes the port with the store's own write discipline (D3): a
/// pid-suffixed temporary in the same directory, fsync, then the
/// atomic rename — a torn publish is a stale file the next probe
/// takes over, never a wrong port.
fn publish_port(path: &Path, port: u16) -> Result<(), String> {
    let directory = path
        .parent()
        .ok_or_else(|| "the port file has no parent directory".to_owned())?;
    fs::create_dir_all(directory)
        .map_err(|error| format!("create the port file's directory: {error}"))?;
    let name = path
        .file_name()
        .ok_or_else(|| "the port file has no file name".to_owned())?;
    let temporary = directory.join(format!("{}.{}.tmp", name.to_string_lossy(), process::id()));
    let mut file =
        fs::File::create(&temporary).map_err(|error| format!("create the port file: {error}"))?;
    writeln!(file, "{port}").map_err(|error| format!("write the port file: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("fsync the port file: {error}"))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("rename the port file into place: {error}"))
}

/// Probes the published port (D3): `/health` must answer 200 with the
/// marker. Anything else — refused, timed out, unparseable, foreign —
/// reads as dead; a foreign server is never killed, only replaced in
/// the publication.
fn listener_answers(port: u16) -> bool {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, PROBE_TIMEOUT) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(PROBE_TIMEOUT));
    let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut response = Vec::new();
    if stream.read_to_end(&mut response).is_err() {
        return false;
    }
    let text = String::from_utf8_lossy(&response);
    // The Python listener answers HTTP/1.0 (http.server's default
    // protocol); ours answers 1.1 — the probe accepts both so the two
    // interoperate until t73 retires the Python.
    (text.starts_with("HTTP/1.1 200") || text.starts_with("HTTP/1.0 200"))
        && text.contains(HEALTH_MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The publication round-trip: publish, read back; absent,
    /// non-digit and out-of-range files read as not published.
    #[test]
    fn the_publication_round_trips() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("serve.port");
        assert_eq!(read_published_port(&path), None);
        publish_port(&path, 41234).expect("the publish lands");
        assert_eq!(read_published_port(&path), Some(41234));
        std::fs::write(&path, "not-a-port\n").expect("the foreign bytes land");
        assert_eq!(read_published_port(&path), None);
        std::fs::write(&path, "99999\n").expect("the out-of-range port lands");
        assert_eq!(read_published_port(&path), None);
    }

    /// The default tier's fallback (judgment call (f)): no cwd to
    /// match falls to the registry's first slug, ascending.
    #[test]
    fn the_default_repo_falls_to_the_first_slug() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let root = DataRoot::new(dir.path());
        for (slug, instant) in [
            ("fixture", "2026-10-02T08:00:00Z"),
            ("alpha", "2026-10-02T09:00:00Z"),
        ] {
            root.register(todo_domain::Repo {
                slug: RepoSlug::from_str(slug).expect("a valid slug"),
                name: slug.to_owned(),
                path: format!("/repos/{slug}"),
                registered_at: todo_domain::Timestamp::parse(instant).expect("a timestamp"),
            })
            .expect("the repo registers");
        }
        assert_eq!(
            default_repo(&root, None),
            Ok(RepoSlug::from_str("alpha").expect("a valid slug"))
        );
    }
}
