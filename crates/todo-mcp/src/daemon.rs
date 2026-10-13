//! The daemon (D25, §13, U65): the extra-argument invocation serving
//! every connected session over one unix socket — `<root>/serve.sock`,
//! D13's per-root keying (the same directory as `serve.port`), so two
//! roots' daemons coexist by construction. The no-argument shim (U66)
//! spawns it and proxies MCP to it; the socket is a closed internal
//! surface whose only client is the shim, so the connection protocol
//! is the one-line JSON preamble §6.4 measured (rmcp has no
//! per-connection metadata channel: the handler is constructed before
//! the handshake and `RequestContext` carries only per-request state)
//! followed by a normal MCP session.
//!
//! The shape is §6.4's measured one: accept → one spawned task per
//! connection → `tokio::io::split(stream)` → `BufReader` on the read
//! half → the preamble line → the per-connection [`TodoServer`]
//! (its own session, the shared core) → `serve((reader, write_half))`
//! → `waiting()`. Bytes the client already sent — its `initialize` —
//! ride inside the `BufReader` and are the first thing rmcp reads.
//! A peer that vanishes before the handshake completes is a
//! per-connection serve error (one stderr line, the task ends, the
//! other sessions unaffected) — the stdio server's error handling,
//! per connection.
//!
//! The state split is the reason this module exists as it does:
//! per-connection [`TodoServer`]s share one core (the root and the
//! handles cache behind one lock — `DataRoot::open` opens the store
//! *locked*, so per-connection cores would fail each other with
//! `locked`) while each carries its own session, built from the
//! preamble's repo and cwd. The stdio path is byte-identical to
//! before the split — the protocol goldens pin it.
//!
//! SIGTERM is the graceful drain (the update cycle's signal, U67):
//! stop accepting, drop the listener (a new connection from then on
//! is refused), then wait for the connection tasks to end naturally,
//! bounded by [`DRAIN_TIMEOUT`] — the services are not cancelled
//! during the window, so in-flight queries that complete within it
//! get their responses delivered; at the bound the process exits and
//! the transports close with it. SIGINT keeps the default
//! disposition. On exit the socket file and the port publication are
//! left in place — the next owner's ensure handles staleness.
//!
//! The live view (D13) is ensured at startup the way the
//! no-subcommand server's first tick does it — one
//! Watchdog::ensure, no loop: the listener thread dies with the
//! process, and the watchdog's takeover duty retires in the daemon
//! shape (U66 retires it on the stdio path too).
//!
//! The module translates, never decides (§2): the preamble's repo is
//! parsed to a [`RepoSlug`] here and handed to the session; every
//! query decision stays the domain's. Nothing new reaches disk
//! except the socket file and the ensure's persistent lockfile
//! (`<root>/serve.lock`, never removed), and nothing new reaches the
//! wire except the preamble's parse.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rmcp::ServiceExt;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal::unix::{SignalKind, signal};
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout_at};

use todo_domain::RepoSlug;
use todo_store::DataRoot;

use crate::serve::{POLL_INTERVAL, Watchdog};
use crate::{CoreState, TodoServer};

/// The SIGTERM drain's bound: how long the daemon waits for its
/// connection tasks to end after SIGTERM before exiting anyway. The
/// window exists for in-flight requests — a store write or a read
/// answering — and those are local-file operations with no network
/// round-trip, so seconds of headroom is generous; the bound exists
/// for the sessions that never end on their own (an idle client
/// holds its connection open), which the process exit then cuts.
/// Five seconds keeps an update cycle's restart (the shim's retry
/// window, U67) from growing an unbounded stall.
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// How long the accept loop pauses after an accept error — the
/// listener stays up (the serve accept loop's own rule), and the
/// pause keeps a persistent error from spinning the loop.
const ACCEPT_ERROR_PAUSE: Duration = Duration::from_millis(50);

/// One daemon run's outcome, the binary's exit-code mapping (the
/// `ServeOutcome` precedent: the lib owns the logic, the binary owns
/// the exit).
pub enum DaemonOutcome {
    /// A live daemon already serves this root: the caller prints one
    /// stderr line and exits 0 — never a second listener.
    AlreadyServing,
    /// SIGTERM ended the run. `severed` says the drain bound was
    /// reached with connections still live — their transports close
    /// with the process, which is the one stderr line's cause; the
    /// exit is 0 either way, the bound being the documented contract.
    Drained {
        /// Whether any connection was cut at the bound.
        severed: bool,
    },
}

/// The daemon's socket path (D13's keying): `<root>/serve.sock`, the
/// same directory as `serve.port` — one per-root slot, so two roots'
/// daemons coexist by construction.
pub fn socket_path(root: &DataRoot) -> PathBuf {
    root.root().join("serve.sock")
}

/// Runs the daemon over `root` (D25): ensure the socket, ensure the
/// live view, then accept connections until SIGTERM — each connection
/// one spawned task serving its own MCP session over the split
/// stream. SIGTERM stops the accept loop, drops the listener and
/// waits for the connection tasks within `drain_timeout`; the socket
/// file and the port publication are left in place on exit (the next
/// owner's ensure handles staleness). `cwd` is the daemon's own
/// startup-read working directory — the live view's default tier,
/// not any connection's (each connection's cwd arrives in its
/// preamble).
pub async fn run(
    root: &DataRoot,
    cwd: Option<&Path>,
    drain_timeout: Duration,
) -> Result<DaemonOutcome, String> {
    // The SIGTERM handler is installed before any work: a signal
    // that lands during startup is captured (the Signal buffers it)
    // and answered by the accept loop's select — exit 0 through the
    // drain, never the default disposition's kill.
    let mut sigterm = signal(SignalKind::terminate())
        .map_err(|error| format!("install the SIGTERM handler: {error}"))?;
    let listener = match ensure_socket(root).await? {
        EnsureSocket::AlreadyServing => return Ok(DaemonOutcome::AlreadyServing),
        EnsureSocket::Bound(listener) => listener,
    };
    // The live view (D13): the first-tick ensure, no watchdog loop.
    // The started listener's thread is detached — dropping the
    // watchdog (and its Listener handle) leaves the thread running,
    // and the process exit reaps it (the serve module's own Drop
    // discipline) — and the publication is left for the next owner.
    Watchdog::new(root, cwd, POLL_INTERVAL).ensure();
    let core = TodoServer::shared_core(root.clone());
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            _ = sigterm.recv() => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    let core = core.clone();
                    connections.spawn(serve_connection(stream, core));
                }
                Err(error) => {
                    eprintln!("todo-mcp daemon: accept failed: {error}");
                    sleep(ACCEPT_ERROR_PAUSE).await;
                }
            }
        }
    }
    // Stop accepting: the drop closes the socket, so a connection
    // attempted from here on is refused.
    drop(listener);
    // The drain: wait for the connection tasks to end naturally,
    // bounded — the services are not cancelled during the window, so
    // in-flight queries that complete within it are answered; at the
    // bound this loop leaves, the run returns, and the process exit
    // closes the transports (the JoinSet's drop is the tail of that
    // same exit, not a mid-window cancellation).
    let deadline = tokio::time::Instant::now() + drain_timeout;
    let mut severed = false;
    while !connections.is_empty() {
        if timeout_at(deadline, connections.join_next()).await.is_err() {
            severed = true;
            break;
        }
    }
    Ok(DaemonOutcome::Drained { severed })
}

/// The startup socket ensure (D25), serialized: the whole sequence —
/// probe, remove, bind — runs under an exclusive advisory lock on
/// `<root>/serve.lock`, a persistent lockfile the daemon owns and
/// never removes. Unserialized, the ensure is a TOCTOU a racing
/// double-spawn wins twice: the loser's remove can land after the
/// winner's bind, unlinking the winner's socket and letting the
/// loser bind its own — two live daemons on one root, the winner's
/// socket inode orphaned until it is signaled. Under the lock the
/// waiter blocks for the milliseconds of another spawn's ensure,
/// then probes the winner's live socket and exits 0 — the connect
/// inside the lock is the liveness arbiter. A bind that still
/// refuses (a foreign holder of the path) is the honest error.
async fn ensure_socket(root: &DataRoot) -> Result<EnsureSocket, String> {
    let path = socket_path(root);
    // Held for the ensure's duration only — the guard's drop is the
    // release.
    let _ensure_lock = lock_ensure(root)?;
    if UnixStream::connect(&path).await.is_ok() {
        return Ok(EnsureSocket::AlreadyServing);
    }
    // The failed connect left a stale file (the owner died without
    // cleanup) or none; a removal failure other than "absent" is an
    // honest error — a socket path that cannot be cleared cannot be
    // bound.
    if let Err(error) = std::fs::remove_file(&path)
        && error.kind() != io::ErrorKind::NotFound
    {
        return Err(format!(
            "remove the stale socket {}: {error}",
            path.display()
        ));
    }
    let listener = UnixListener::bind(&path)
        .map_err(|error| format!("bind the daemon socket {}: {error}", path.display()))?;
    Ok(EnsureSocket::Bound(listener))
}

/// The ensure's serialization: an exclusive `flock` on
/// `<root>/serve.lock`, acquired blocking. The wait it can sit out is
/// another spawn's ensure — milliseconds — and a SIGTERM that lands
/// during it still drains: the handler is installed before this
/// point, so the signal is captured and answered by the accept loop's
/// select once the ensure completes, never by the default
/// disposition's kill. The lockfile is persistent and never removed:
/// its bytes mean nothing (`flock` never reads them), and its
/// *existence* is what serializes — removing and recreating it would
/// hand two racers fresh inodes whose locks cannot conflict.
fn lock_ensure(root: &DataRoot) -> Result<File, String> {
    let path = root.root().join("serve.lock");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        // Never truncate: a lock file's bytes mean nothing, and
        // truncating one on every acquisition is a gratuitous write.
        .truncate(false)
        .open(&path)
        .map_err(|error| format!("open the ensure lock {}: {error}", path.display()))?;
    file.lock()
        .map_err(|error| format!("lock the ensure lock {}: {error}", path.display()))?;
    Ok(file)
}

/// The socket ensure's answer: a live daemon found, or the listener
/// this invocation bound and owns.
enum EnsureSocket {
    /// Another daemon answers the socket — attach, never a second
    /// listener.
    AlreadyServing,
    /// The freshly bound listener.
    Bound(UnixListener),
}

/// One connection (§6.4's measured shape): split the stream, read the
/// preamble line off the `BufReader`, construct the per-connection
/// server (its own session from the preamble, the shared core), and
/// serve the MCP session over `(reader, write_half)`. Every fault on
/// this path ends the task with one stderr line — the other sessions
/// are unaffected.
async fn serve_connection(stream: UnixStream, core: Arc<Mutex<CoreState>>) {
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    let read = match reader.read_line(&mut line).await {
        Ok(read) => read,
        Err(error) => {
            eprintln!("todo-mcp daemon: the preamble read failed: {error}");
            return;
        }
    };
    if read == 0 {
        // A peer that connects and closes without sending a byte —
        // the ensure probe's own shape; nothing was asked, so there
        // is nothing to answer and nothing to log (the header-timeout
        // precedent).
        return;
    }
    let session = match parse_preamble(&line) {
        Ok(session) => session,
        Err(message) => {
            eprintln!("todo-mcp daemon: {message}");
            return;
        }
    };
    let server = TodoServer::with_core(core, session.repo, session.cwd);
    match server.serve((reader, write_half)).await {
        Ok(service) => {
            if let Err(error) = service.waiting().await {
                eprintln!("todo-mcp daemon: the session quit: {error}");
            }
        }
        Err(error) => {
            eprintln!("todo-mcp daemon: the session failed: {error}");
        }
    }
}

/// The preamble envelope (§6.4, U65): one JSON line read from the
/// stream *before* `serve` sees it — `{"repo": <slug>, "cwd":
/// <path>}`, both fields optional, unknown fields denied. The
/// envelope is same-binary paired with the shim (U66), so it carries
/// no version. The cwd is the *shim's* — the client's working
/// directory — never the daemon's own.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Preamble {
    /// The connection's default repo slug (this session's D8 tier 2).
    repo: Option<String>,
    /// The shim's working directory (this session's D8 tier 3 input).
    cwd: Option<String>,
}

/// The parsed preamble: the repo validated to a [`RepoSlug`], the cwd
/// as a path — the values TodoServer::with_core takes.
#[derive(Debug)]
struct ParsedPreamble {
    /// The validated default repo slug, when the preamble carried one.
    repo: Option<RepoSlug>,
    /// The shim's working directory, when the preamble carried one.
    cwd: Option<PathBuf>,
}

/// Parses and validates one preamble line: JSON, the two known
/// fields, a well-formed slug. Any fault is a message for the one
/// stderr line — pre-protocol, so no MCP error is answered (the
/// connection is dropped before any MCP byte is interpreted).
fn parse_preamble(line: &str) -> Result<ParsedPreamble, String> {
    let preamble: Preamble = serde_json::from_str(line.trim_end())
        .map_err(|error| format!("the socket preamble is not valid: {error}"))?;
    let repo = match &preamble.repo {
        Some(value) => Some(
            RepoSlug::from_str(value)
                .map_err(|error| format!("the socket preamble's repo is not a slug: {error}"))?,
        ),
        None => None,
    };
    Ok(ParsedPreamble {
        repo,
        cwd: preamble.cwd.map(PathBuf::from),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The envelope's accepted shapes: both fields, either alone, and
    /// the empty object (no defaults — the cwd tier answers no match
    /// on a relaxed call). Field order is free.
    #[test]
    fn the_preamble_parses_its_accepted_shapes() {
        let both = parse_preamble(r#"{"repo":"fixture","cwd":"/repos/fixture"}"#).unwrap();
        assert_eq!(both.repo, Some(RepoSlug::from_str("fixture").unwrap()));
        assert_eq!(both.cwd, Some(PathBuf::from("/repos/fixture")));
        let repo_only = parse_preamble(r#"{"repo":"fixture"}"#).unwrap();
        assert_eq!(repo_only.repo, Some(RepoSlug::from_str("fixture").unwrap()));
        assert_eq!(repo_only.cwd, None);
        let cwd_only = parse_preamble(r#"{"cwd":"/repos/fixture"}"#).unwrap();
        assert_eq!(cwd_only.repo, None);
        assert_eq!(cwd_only.cwd, Some(PathBuf::from("/repos/fixture")));
        let empty = parse_preamble("{}").unwrap();
        assert_eq!(empty.repo, None);
        assert_eq!(empty.cwd, None);
        // The trailing newline is the read's, not the envelope's.
        let trailing = parse_preamble("{\"repo\":\"fixture\"}\n").unwrap();
        assert_eq!(trailing.repo, Some(RepoSlug::from_str("fixture").unwrap()));
        // Field order is free.
        let flipped = parse_preamble(r#"{"cwd":"/p","repo":"fixture"}"#).unwrap();
        assert_eq!(flipped.repo, Some(RepoSlug::from_str("fixture").unwrap()));
        assert_eq!(flipped.cwd, Some(PathBuf::from("/p")));
    }

    /// The fault path: a malformed line, an unknown field, a
    /// malformed slug — each a message, never a panic, never a
    /// half-parsed session.
    #[test]
    fn the_preamble_refuses_its_fault_shapes() {
        for (line, needle) in [
            ("not json", "the socket preamble is not valid"),
            ("", "the socket preamble is not valid"),
            ("{\"repo\":\"fixture\",\"extra\":1}", "unknown field"),
            ("{\"repo\":\"Bad!\"}", "is not a slug"),
        ] {
            let error = parse_preamble(line).unwrap_err();
            assert!(error.contains(needle), "{line}: {error}");
        }
    }
}
