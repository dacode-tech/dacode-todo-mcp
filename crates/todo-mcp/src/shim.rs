//! The shim (D25, §13, U66): the no-argument invocation's body — the
//! thin stdio client of the root's daemon. The client wiring is
//! unchanged (§13: the same command every existing config entry
//! names); what changes is behind the pipe. The shim connects to
//! `<root>/serve.sock` ([`socket_path`]), spawning the daemon
//! when absent, writes the one-line JSON preamble §6.4 settled
//! (`{"repo":…,"cwd":…}` — the repo from this invocation's `--repo`
//! flag, the cwd from the binary's startup read, both optional, the
//! shim's cwd being the client's), and then proxies bytes: client
//! stdin → socket, socket → client stdout. The proxy is transparent
//! — nothing is injected between the preamble and the stream —
//! because the wire goldens pin the bytes end to end through it.
//!
//! # The connect phase
//!
//! A first connect attempt rides before any spawn (a daemon already
//! serving is attached to, never replaced). On failure the shim
//! spawns the daemon — this binary, `--root <dir> --daemon`, stdio
//! null (the daemon's diagnostics must not leak into the client's
//! pipes, and the daemon must not hold the shim's fds open after the
//! shim exits), its own process group so it survives the client's
//! process-group teardown — then retries the connect every
//! `RETRY_INTERVAL` within [`RETRY_WINDOW`]. A spawn that fails to
//! execute at all skips the window (the failure mode answers at
//! once); a connect that succeeds but dies before the preamble write
//! folds back into the window — the peer vanished between its accept
//! and the write. An exited child is reaped opportunistically inside
//! the loop: the window's retries are for whatever daemon ends up at
//! the socket, not this child in particular (U67's update cycle
//! restarting one is found the same way), and an unreaped exit would
//! hold the pid as a zombie for the window's remainder.
//!
//! # The reconnect (U67)
//!
//! The daemon→client copy's end while the client's stdin is still
//! open is the update cycle's moment — the daemon was killed (its
//! drain serves the connection to the bound, so the loss is observed
//! only at the bound's end: the drain does not count against the
//! reconnect window) or lost. The reconnect reuses the connect phase
//! verbatim, the same function with a second caller, so a daemon is
//! respawned from this binary's path — the freshly installed inode
//! after an update — within the same [`RETRY_WINDOW`]. The fresh
//! daemon-side session must be initialized before it will answer,
//! and the client cannot re-send `initialize` (its session is
//! established; it must never learn the connection broke), so the
//! shim performs the handshake-on-behalf: its own `initialize`
//! request — a string request id the client can never emit, honest
//! `clientInfo` (the shim is the client here), the protocol version
//! this server declares — then `notifications/initialized`,
//! everything consumed by the shim, nothing forwarded. The client's
//! bytes during the reconnect ride the stdin pipe: the one
//! client→daemon copy task parks on the socket slot while the
//! reconnect runs — tokio's stdin read is uncancellable, its bytes
//! dropped if the read future is dropped mid-operation, so the task
//! persists across generations rather than respawning — and after
//! the handshake the task drains the pipe, so requests sent during
//! the window are answered after the reconnect. The session bit
//! stays latched from the original session's first line; the
//! reconnect's initialize response never passes through the
//! first-line classifier (a successful handshake-on-behalf also
//! latches it, so a daemon lost before the client's handshake
//! completed still ends 0 when the reconnected session completes).
//! A reconnect whose window expires — or whose handshake is
//! refused — is the terminal: one stderr line, exit 1; there is
//! deliberately no mid-session degraded mode.
//!
//! # The failure mode (D25's no fallback, deliberately)
//!
//! After the window expires with no daemon, the shim does not exit
//! and does not serve locally: it becomes a degraded MCP server over
//! stdio — a real initialize handshake byte-identical to the daemon's
//! (the same `server_config`, so a client cannot tell the modes
//! apart from the handshake), `tools/list` answering the real tool
//! list (the router is static — no store is opened, which is the
//! no-fallback itself), and every `tools/call` answered with §6.1's
//! `io` carrier naming the daemon unreachable and the window
//! expired — the in-band error a client may retry on. The mode is
//! terminal for the shim process.
//!
//! # The exit contract
//!
//! Client-side EOF ends the session: exit 0 after a completed
//! session (proxied or degraded), exit 1 when the daemon's connection
//! ended without one establishing — the stdio server's own
//! close-before-initialize case. The daemon cannot signal its
//! per-connection serve outcome through the socket (a refused
//! connection and an ended session both just close it), so the
//! daemon→client copy classifies the daemon's first response line on
//! its way through — an `error` member (or no bytes at all) is the
//! handshake gate's refusal, anything else a running session — and
//! that bit picks the exit code. Socket-side EOF while the client's
//! stdin is still open is the reconnect's trigger (above); the
//! reconnect's failure — the window expired, the handshake refused —
//! is the abnormal end: one stderr line, exit 1. The exit is
//! delivered regardless of the client: the binary's
//! bounded runtime teardown cuts the client task's in-flight
//! blocking stdin read rather than letting the runtime's drop wait
//! it out.
//!
//! Over the file-size target by recorded exception: the reconnect
//! (U67) grew the proxy loop, the socket slot and the
//! handshake-on-behalf past the target. The split, when the next
//! growth arrives, is the proxy phase — the slot, the two copy
//! directions and the generation loop — into its own module beside
//! the connect phase.
//!
//! The module translates and proxies, never decides (§2): no store
//! is opened on any path here, and nothing new reaches the wire
//! except the preamble's line.

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use rmcp::ServerHandler;
use rmcp::ServiceExt;
use rmcp::model::{
    CacheScope, CallToolRequestParams, CallToolResponse, ErrorData, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ResultType, ServerConfig, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::UnixStream;
use tokio::sync::Notify;
use tokio::time::{Instant, sleep};

use todo_domain::RepoSlug;
use todo_store::DataRoot;

use crate::TodoServer;
use crate::daemon::socket_path;

/// How long the shim retries the connect after spawning the daemon
/// before answering degraded — and, since U67, how long a
/// mid-session reconnect retries before the terminal. The bound
/// covers spawn plus the daemon's own startup — the serialized
/// socket ensure (lock, probe, bind) and the live-view ensure — with
/// huge margin; the honest failure answers after it, never before.
/// The update cycle's drain does not count against it: the daemon
/// serves the connection to the drain's bound, so the shim observes
/// the loss only at the bound's end and the window starts there.
pub const RETRY_WINDOW: Duration = Duration::from_secs(5);

/// The connect retry cadence: the first retry lands one interval
/// after the spawn, which is the common case's whole latency (the
/// daemon binds within milliseconds of exec).
const RETRY_INTERVAL: Duration = Duration::from_millis(100);

/// One shim run's outcome, the binary's exit-code mapping (the
/// `DaemonOutcome` precedent: the lib owns the logic, the binary
/// owns the exit).
pub enum ShimOutcome {
    /// The client closed a completed session — proxied through the
    /// daemon, or served degraded after the retry window expired.
    /// Exit 0.
    Completed,
    /// The session did not complete: the daemon connection was lost
    /// mid-session, or ended without a session establishing (the
    /// client closed before the handshake — the stdio server's
    /// exit-1 case), or the degraded serve itself failed. Exit 1;
    /// the one stderr line is printed where the fault is seen.
    Incomplete,
}

/// Runs the shim over `root` (D25, U66): the connect phase (attach,
/// spawn, retry), then the byte proxy — or, when no daemon could be
/// reached within the window, the degraded failure mode. `repo` is
/// this invocation's `--repo` flag (the preamble's repo, the
/// connection's D8 tier 2) and `cwd` the binary's startup read (the
/// preamble's cwd, the client's working directory). Both ride every
/// reconnect too: each generation is a fresh connection with its own
/// preamble (U67).
pub async fn run(root: &DataRoot, repo: Option<RepoSlug>, cwd: Option<PathBuf>) -> ShimOutcome {
    match connect(root, repo.as_ref(), cwd.as_deref()).await {
        ConnectPhase::Connected(stream) => proxy(root, repo.as_ref(), cwd.as_deref(), stream).await,
        ConnectPhase::Unavailable(message) => degraded(message).await,
    }
}

/// The connect phase's answer: a connected, preamble-written socket,
/// or the message the degraded carrier answers with.
enum ConnectPhase {
    /// The socket, preamble already written — the proxy's to own.
    Connected(UnixStream),
    /// No daemon within the window (or the spawn itself failed): the
    /// failure mode's carrier message.
    Unavailable(String),
}

/// The connect phase: attach if a daemon answers, spawn one if none
/// does, retry within the window. See the module doc for the edges —
/// the fold-back and the opportunistic reap.
async fn connect(root: &DataRoot, repo: Option<&RepoSlug>, cwd: Option<&Path>) -> ConnectPhase {
    let path = socket_path(root);
    let preamble = preamble_line(repo, cwd);
    let deadline = Instant::now() + RETRY_WINDOW;
    let mut child: Option<Child> = None;
    loop {
        if let Ok(mut stream) = UnixStream::connect(&path).await {
            // A connect that succeeds is half a session: the peer can
            // die between its accept and this write, and the write
            // then fails — fold back into the window and retry.
            if stream.write_all(preamble.as_bytes()).await.is_ok() {
                return ConnectPhase::Connected(stream);
            }
        }
        if child.is_none() {
            match spawn_daemon(root) {
                Ok(spawned) => child = Some(spawned),
                // A spawn that fails to execute at all cannot become
                // a daemon by waiting: the failure mode answers now.
                Err(message) => return ConnectPhase::Unavailable(message),
            }
        }
        if Instant::now() >= deadline {
            return ConnectPhase::Unavailable(unavailable_message(&path));
        }
        sleep(RETRY_INTERVAL).await;
        if let Some(spawned) = &mut child {
            let _ = spawned.try_wait();
        }
    }
}

/// Spawns the daemon for `root`: this binary, `--root <dir>
/// --daemon`. Stdio is null on all three — the daemon's diagnostics
/// must not leak into the client's pipes, and the daemon must not
/// hold the shim's fds open after the shim exits — and the child
/// takes its own process group, so the client's process-group
/// teardown never reaches it.
fn spawn_daemon(root: &DataRoot) -> Result<Child, String> {
    let exe = std::env::current_exe()
        .map_err(|error| format!("the daemon could not be spawned: locate this binary: {error}"))?;
    let mut command = Command::new(exe);
    command.arg("--root").arg(root.root());
    command.arg("--daemon");
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command.process_group(0);
    command
        .spawn()
        .map_err(|error| format!("the daemon could not be spawned: {error}"))
}

/// The preamble line (§6.4's settlement): `{"repo":…,"cwd":…}`, both
/// fields optional, emitted in the settlement's spelling — repo then
/// cwd, the insertion order `preserve_order` keeps. The daemon's
/// parse accepts any order; the emission is deterministic anyway.
fn preamble_line(repo: Option<&RepoSlug>, cwd: Option<&Path>) -> String {
    let mut object = serde_json::Map::new();
    if let Some(slug) = repo {
        object.insert(
            "repo".to_owned(),
            serde_json::Value::String(slug.as_str().to_owned()),
        );
    }
    if let Some(dir) = cwd {
        // Display is lossy for a non-UTF-8 path; the cwd tier
        // canonicalizes what it receives, so a lossy spelling simply
        // matches nothing — the honest degradation for a path JSON
        // cannot carry.
        object.insert(
            "cwd".to_owned(),
            serde_json::Value::String(dir.display().to_string()),
        );
    }
    format!("{}\n", serde_json::Value::Object(object))
}

/// The degraded carrier's message when the window expired: the
/// socket, the window, and that the tool did not run — the facts a
/// client retrying the call needs.
fn unavailable_message(path: &Path) -> String {
    format!(
        "the daemon is unreachable: no connection to {} within the {}s retry window — \
         the tool did not run; retry the call once the daemon is back",
        path.display(),
        RETRY_WINDOW.as_secs()
    )
}

/// The reconnect handshake's request id: a string, because the
/// clients' request ids are numeric — rmcp's client-side id provider
/// is a monotonic u32 counter, and every client this server serves
/// speaks through it — so a string id can never collide with an id
/// the client emits; the daemon-side response is the shim's to match
/// alone.
const RECONNECT_ID: &str = "shim-reconnect";

/// The hand-off point between the proxy loop and the one
/// client→daemon copy task (U67): the loop owns the reconnects and
/// publishes each generation's write half — only after the
/// handshake-on-behalf, so no client byte can interleave with it —
/// while the task takes each generation as it comes and parks here
/// while the slot is empty (a reconnect in progress). The generation
/// counter is the task's staleness check: a read that completes under
/// a generation that is no longer current holds its bytes for the
/// next one instead of writing them to a dead socket.
struct SocketSlot {
    /// The current generation and its write half — the half is `None`
    /// between the task's take and the next publish (a reconnect in
    /// progress). The guard is never held across an await.
    state: Mutex<SocketState>,
    /// Wakes the task when a generation is published. `notify_one`'s
    /// stored permit is what makes the wake race-free: a publish that
    /// lands between the task's check and its wait is consumed by the
    /// next `notified` poll.
    published: Notify,
}

/// The slot's guarded state: the generation number (monotonic across
/// the session) and the published-but-untaken write half.
struct SocketState {
    generation: u64,
    half: Option<WriteHalf<UnixStream>>,
}

impl SocketSlot {
    /// The shared slot the proxy loop and the copy task hand off
    /// through.
    fn shared() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(SocketState {
                generation: 0,
                half: None,
            }),
            published: Notify::new(),
        })
    }

    /// Publishes the next generation's write half, waking the copy
    /// task. Called only after the handshake-on-behalf completed (a
    /// reconnect generation) or immediately (the first).
    fn publish(&self, half: WriteHalf<UnixStream>) {
        let mut state = self.lock();
        state.generation += 1;
        state.half = Some(half);
        self.published.notify_one();
    }

    /// The current generation number — the copy task's staleness
    /// check.
    fn generation(&self) -> u64 {
        self.lock().generation
    }

    /// Takes the live write half with its generation, waiting while a
    /// reconnect holds the slot empty.
    async fn acquire(&self) -> (u64, WriteHalf<UnixStream>) {
        loop {
            // Created before the check: a publish between the two
            // leaves a permit the wait consumes (the field doc's
            // race-freedom).
            let notified = self.published.notified();
            {
                let mut state = self.lock();
                if let Some(half) = state.half.take() {
                    return (state.generation, half);
                }
            }
            notified.await;
        }
    }

    /// Takes the published-but-untaken half, if any — never waiting.
    /// The proxy loop's EOF forwarding: when the copy task ends
    /// holding a stale half, the current generation's half is still
    /// in the slot, and the client's EOF must reach it.
    fn take_published(&self) -> Option<WriteHalf<UnixStream>> {
        self.lock().half.take()
    }

    /// The lock, poison-recovered per the lib's precedent: the
    /// guarded regions cannot panic (the panic-family lints are
    /// denied workspace-wide), so poison is unreachable;
    /// `into_inner` recovers rather than killing the shim.
    fn lock(&self) -> MutexGuard<'_, SocketState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The handshake-on-behalf's `initialize` request line (U67): the
/// string id, an honest `clientInfo` — the shim is the client here —
/// and the protocol version this server declares, the same one
/// [`crate::server_config`] answers the client's own handshake with.
fn reconnect_initialize_line() -> String {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": RECONNECT_ID,
        "method": "initialize",
        "params": {
            "protocolVersion": ProtocolVersion::LATEST_WITH_INITIALIZE.as_str(),
            "capabilities": {},
            "clientInfo": {
                "name": "todo-mcp-shim",
                "version": env!("CARGO_PKG_VERSION"),
            },
        },
    });
    format!("{request}\n")
}

/// The handshake-on-behalf's `notifications/initialized` line — the
/// completion notification, which produces no response (JSON-RPC
/// notifications never do).
fn reconnect_initialized_line() -> String {
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
    });
    format!("{notification}\n")
}

/// The handshake-on-behalf's response check: `None` when the line is
/// the shim's initialize response carrying a result, or the fault's
/// message — a line that is not JSON, carries an `error` member, or
/// does not echo the shim's id.
fn handshake_response_fault(line: &[u8]) -> Option<String> {
    let value: serde_json::Value = match serde_json::from_slice(line) {
        Ok(value) => value,
        Err(error) => return Some(format!("the response is not JSON: {error}")),
    };
    if value.get("error").is_some() {
        return Some(format!(
            "the daemon refused the handshake: {}",
            String::from_utf8_lossy(line)
        ));
    }
    if value.get("id").and_then(serde_json::Value::as_str) != Some(RECONNECT_ID) {
        return Some(format!(
            "the response is not the shim's: {}",
            String::from_utf8_lossy(line)
        ));
    }
    None
}

/// The handshake-on-behalf (U67): a reconnected daemon-side session
/// must be initialized before it will answer, and the client cannot
/// re-send `initialize` — its session is established, and it must
/// never learn the connection broke. The shim initializes on the
/// client's behalf: its own `initialize` request, then
/// `notifications/initialized`. Everything here is consumed by the
/// shim — nothing reaches the client — and the write half is
/// published (the client's pipe drained) only after, so no client
/// byte can interleave with the handshake. A failure here is the
/// terminal, not a retry: the connect phase's window already bounded
/// the wait for a daemon at the socket, and a daemon that accepts
/// the preamble but dies before answering initialize is not the
/// update cycle's shape.
async fn handshake_on_behalf(
    write_half: &mut WriteHalf<UnixStream>,
    reader: &mut BufReader<ReadHalf<UnixStream>>,
) -> Result<(), String> {
    write_half
        .write_all(reconnect_initialize_line().as_bytes())
        .await
        .map_err(|error| format!("the reconnect handshake's write failed: {error}"))?;
    let mut response = Vec::new();
    let read = reader
        .read_until(b'\n', &mut response)
        .await
        .map_err(|error| format!("the reconnect handshake's read failed: {error}"))?;
    if read == 0 {
        return Err("the daemon closed the connection during the reconnect handshake".to_owned());
    }
    let line = response.strip_suffix(b"\n").unwrap_or(&response);
    if let Some(fault) = handshake_response_fault(line) {
        return Err(fault);
    }
    // The completion notification rides only after the response —
    // rmcp's serve loop is running by then; it produces no response
    // of its own (JSON-RPC notifications never do).
    write_half
        .write_all(reconnect_initialized_line().as_bytes())
        .await
        .map_err(|error| format!("the reconnect handshake's write failed: {error}"))?;
    Ok(())
}

/// One proxy direction's end: the source EOF'd (the direction's
/// natural end) or the copy faulted (the socket or the pipe broke
/// mid-copy).
#[derive(Debug, PartialEq, Eq)]
enum DirectionEnd {
    /// The source reached EOF — the direction copied everything it
    /// was given.
    Closed,
    /// The copy faulted before EOF.
    Fault,
}

/// Which proxy direction ended first, carrying that direction's
/// result. The `JoinError` arms fold to [`DirectionEnd::Fault`] — the
/// copy tasks hold no panic path, but the handles' `Result` demands
/// the case, and a faulted direction is the conservative reading.
enum FirstEnd {
    /// The client→daemon copy ended: `Closed` is the client's stdin
    /// EOF.
    Client(DirectionEnd),
    /// The daemon→client copy ended: the socket's EOF (or fault),
    /// with the session bit.
    Daemon(bool),
}

/// The proxy phase (U67's generation loop): both directions copied
/// concurrently until the session ends. The client→daemon copy is
/// ONE task for the whole session, parked on the socket slot between
/// generations (the module doc's uncancellable-read reasoning); each
/// generation's daemon→client copy is fresh, classifying its first
/// response line only when it is the session's first (the exit
/// contract's session bit — a reconnect generation is blind, the bit
/// already latched). The client's EOF half-closes the daemon's
/// transport — the copy task shuts down the half it holds, and the
/// loop forwards the EOF to the current generation when that half
/// was a stale one (the slot's untaken half, taken and shut down
/// here; the task is finished, so the take is race-free) — and the
/// daemon's socket EOF is then awaited, unbounded: the responses
/// still in flight must be delivered. The daemon side's end with the
/// client still open is the reconnect: the connect phase again, then
/// the loop top runs the handshake-on-behalf and the next generation.
async fn proxy(
    root: &DataRoot,
    repo: Option<&RepoSlug>,
    cwd: Option<&Path>,
    stream: UnixStream,
) -> ShimOutcome {
    let slot = SocketSlot::shared();
    let mut client_to_daemon = tokio::spawn(copy_client_to_daemon(Arc::clone(&slot)));
    let mut established = false;
    let mut stream = stream;
    let mut reconnecting = false;
    loop {
        let (read_half, write_half) = tokio::io::split(stream);
        let mut reader = BufReader::new(read_half);
        let mut write_half = write_half;
        let classify = !reconnecting;
        if reconnecting {
            if let Err(detail) = handshake_on_behalf(&mut write_half, &mut reader).await {
                eprintln!(
                    "todo-mcp: the daemon connection was lost mid-session \
                     and the reconnect's handshake failed: {detail}"
                );
                return ShimOutcome::Incomplete;
            }
            // The handshake-on-behalf's session is valid — the bit
            // latches, only ever upward: a session that already
            // established keeps it.
            established = true;
        }
        slot.publish(write_half);
        let mut daemon_to_client = tokio::spawn(copy_daemon_to_client(reader, classify));
        // Whichever direction ends first decides the shape of the end;
        // the follow-up awaits run after the select, which is what
        // releases the handles' borrows.
        let first = tokio::select! {
            joined = &mut client_to_daemon => {
                FirstEnd::Client(joined.unwrap_or(DirectionEnd::Fault))
            }
            joined = &mut daemon_to_client => {
                let (_, bit) = joined.unwrap_or((DirectionEnd::Fault, false));
                FirstEnd::Daemon(bit)
            }
        };
        match first {
            FirstEnd::Client(end) => {
                if end != DirectionEnd::Closed {
                    // With the socket's write faults waiting for
                    // reconnects, the fault here is the stdin read's
                    // own; the line names the direction, not the side.
                    eprintln!("todo-mcp: the proxy to the daemon faulted mid-session");
                    return ShimOutcome::Incomplete;
                }
                // The client's EOF must reach the CURRENT generation:
                // the copy task shut down the half it held, which can
                // be a stale one — an EOF read that completed under a
                // generation the reconnect had already replaced, or
                // before its publish — leaving the current half
                // untaken in the slot. The task is finished here, so
                // the take is race-free: nobody else will ever take
                // it.
                if let Some(mut half) = slot.take_published() {
                    let _ = half.shutdown().await;
                }
                // The half-close already told the daemon the client is
                // done; wait for its socket EOF — unbounded, because the
                // responses still in flight must be delivered and the
                // daemon ends the session when its transport ends.
                let (_, bit) = daemon_to_client
                    .await
                    .unwrap_or((DirectionEnd::Fault, false));
                if classify {
                    established |= bit;
                }
                return session_end(established);
            }
            FirstEnd::Daemon(bit) => {
                if classify {
                    established |= bit;
                }
                if client_to_daemon.is_finished() {
                    // The client closed first: the session's natural
                    // end, no reconnect (the client is gone).
                    if client_to_daemon.await.unwrap_or(DirectionEnd::Fault) == DirectionEnd::Closed
                    {
                        return session_end(established);
                    }
                    eprintln!("todo-mcp: the proxy to the daemon faulted mid-session");
                    return ShimOutcome::Incomplete;
                }
                // The reconnect: the client is still open, the daemon
                // side ended — the connect phase verbatim (attach,
                // spawn, retry within the window), then the loop top
                // runs the handshake-on-behalf and the next
                // generation.
                match connect(root, repo, cwd).await {
                    ConnectPhase::Connected(next) => {
                        stream = next;
                        reconnecting = true;
                    }
                    ConnectPhase::Unavailable(_) => {
                        eprintln!(
                            "todo-mcp: the daemon connection was lost mid-session \
                             and the reconnect window expired"
                        );
                        return ShimOutcome::Incomplete;
                    }
                }
            }
        }
    }
}

/// The natural end's classification: the client closed after a
/// completed session — or after none established, the stdio server's
/// exit-1 case (a peer that closes before the handshake).
fn session_end(established: bool) -> ShimOutcome {
    if established {
        ShimOutcome::Completed
    } else {
        eprintln!("todo-mcp: the daemon's connection ended without an established session");
        ShimOutcome::Incomplete
    }
}

/// The client→daemon copy (U67's shape): ONE task for the whole
/// session, reading stdin and writing to whichever socket generation
/// the slot publishes. The persistence is the point: tokio's stdin
/// read is uncancellable — the blocking-pool `read` cannot be
/// cancelled, and its bytes are dropped if the read future is dropped
/// mid-operation (the handle's own docs) — so a fresh copy task per
/// generation would lose whatever an in-flight read returns, eating a
/// request sent during the reconnect window. The task instead parks
/// on the slot while a reconnect runs, and the OS pipe buffers the
/// client's bytes — a client writing past the pipe's capacity
/// blocks, it does not fail; the bytes an in-flight read returned are
/// held here, bounded by one chunk. After the handshake-on-behalf the
/// slot publishes the new write half and the task drains the pipe, so
/// requests sent during the window are answered after the reconnect.
/// A write that faults holds its bytes and waits for the next
/// generation (the socket died); a fault after a partial write can
/// duplicate a prefix onto the next session, and the daemon's line
/// parser ignores unparseable lines — the honest bound of the
/// mechanism.
async fn copy_client_to_daemon(slot: Arc<SocketSlot>) -> DirectionEnd {
    let mut stdin = tokio::io::stdin();
    let mut chunk = [0u8; 4096];
    let mut held: Vec<u8> = Vec::new();
    // The half is held across reads — the slot holds only the
    // unpublished generations, so a re-acquire happens exactly when
    // the held one was dropped: it went stale under a read, or its
    // write faulted.
    let (mut generation, mut half) = slot.acquire().await;
    loop {
        if !held.is_empty() {
            if half.write_all(&held).await.is_err() {
                // The socket died again mid-write: the bytes stay
                // held; the next generation's acquire waits for a
                // live half.
                (generation, half) = slot.acquire().await;
                continue;
            }
            held.clear();
        }
        match stdin.read(&mut chunk).await {
            Err(_) => return DirectionEnd::Fault,
            Ok(0) => {
                // The client's stdin EOF becomes the daemon's
                // transport EOF, which ends its session naturally
                // (the stdio server's own contract, forwarded). The
                // half this task holds can be a stale generation's —
                // the proxy loop forwards the EOF to the current one.
                let _ = half.shutdown().await;
                return DirectionEnd::Closed;
            }
            Ok(read) => {
                // `read` is at most the chunk's length by `Read`'s
                // contract, so the `get` cannot miss; the default arm
                // is the unreachable no-op, never a panic.
                let bytes = chunk.get(..read).unwrap_or_default();
                if slot.generation() != generation {
                    // The socket this read started under is gone: hold
                    // the bytes, drop the stale half — the slot
                    // publishes the next one only after the
                    // handshake-on-behalf.
                    held.extend_from_slice(bytes);
                    (generation, half) = slot.acquire().await;
                    continue;
                }
                if half.write_all(bytes).await.is_err() {
                    held.extend_from_slice(bytes);
                    (generation, half) = slot.acquire().await;
                    continue;
                }
            }
        }
    }
}

/// The daemon→client copy: the socket to stdout. The first response
/// line is classified on its way through — but only for the session's
/// first generation (`classify`): a reconnect generation is blind
/// from byte one, the session bit already latched, and the
/// handshake-on-behalf's response never passes through here at all
/// (the handshake consumes it; this copy's bytes begin with the
/// client's own drained requests). The returned bit is meaningful
/// only when `classify` — a blind copy returns `false` and the caller
/// ignores it.
async fn copy_daemon_to_client(
    mut reader: BufReader<ReadHalf<UnixStream>>,
    classify: bool,
) -> (DirectionEnd, bool) {
    let mut stdout = tokio::io::stdout();
    if !classify {
        return match tokio::io::copy(&mut reader, &mut stdout).await {
            Ok(_) => (DirectionEnd::Closed, false),
            Err(_) => (DirectionEnd::Fault, false),
        };
    }
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut end = DirectionEnd::Closed;
    let established = loop {
        let read = match reader.read(&mut chunk).await {
            Ok(0) => break classify_first_line(&buffer),
            Ok(read) => read,
            Err(_) => {
                end = DirectionEnd::Fault;
                break classify_first_line(&buffer);
            }
        };
        // `read` is at most the chunk's length by `Read`'s contract,
        // so the `get` cannot miss; the default arm is the
        // unreachable no-op, never a panic.
        buffer.extend_from_slice(chunk.get(..read).unwrap_or_default());
        // `position` is the buffer's own index, so the `get` cannot
        // miss either — same contract, same no-op default.
        if let Some(position) = buffer.iter().position(|&byte| byte == b'\n') {
            break classify_first_line(buffer.get(..position).unwrap_or_default());
        }
    };
    if stdout.write_all(&buffer).await.is_err() {
        return (DirectionEnd::Fault, established);
    }
    match tokio::io::copy(&mut reader, &mut stdout).await {
        Ok(_) => (end, established),
        Err(_) => (DirectionEnd::Fault, established),
    }
}

/// The session bit from the daemon's first response line: an `error`
/// member — or no bytes at all — is the handshake gate's refusal (no
/// session ever ran); anything else (a `result`, a notification, an
/// unparseable line) reads as a running session. The bytes flow
/// regardless; the bit only picks the exit code at the end.
fn classify_first_line(line: &[u8]) -> bool {
    if line.is_empty() {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) else {
        return true;
    };
    value.get("error").is_none()
}

/// The failure mode's server (D25's no fallback): a degraded MCP
/// server over stdio. The handshake is the daemon's — the same
/// `server_config`, so a client cannot tell the modes apart —
/// `tools/list` answers the real (static) tool list, and every
/// `tools/call` answers §6.1's `io` carrier: the daemon is
/// unreachable, the window expired, the tool did not run. No store
/// is opened; that is the no-fallback itself.
struct DegradedServer {
    /// The carrier's message: why the daemon is unreachable.
    message: String,
    /// The real tool list — the merged router's advertisement, no
    /// store behind it.
    tools: Vec<Tool>,
}

impl DegradedServer {
    /// The degraded server for one unreachable cause.
    fn new(message: String) -> Self {
        Self {
            message,
            tools: TodoServer::merged_router().list_all(),
        }
    }
}

/// The degraded session: the failure mode served over stdio until the
/// client closes — the stdio server's own end contract (a completed
/// handshake ends 0; a peer that closes before it ends 1).
async fn degraded(message: String) -> ShimOutcome {
    let server = DegradedServer::new(message);
    match server.serve(rmcp::transport::stdio()).await {
        Ok(service) => match service.waiting().await {
            Ok(_) => ShimOutcome::Completed,
            Err(error) => {
                eprintln!("todo-mcp: the degraded session quit: {error}");
                ShimOutcome::Incomplete
            }
        },
        Err(error) => {
            eprintln!("todo-mcp: {error}");
            ShimOutcome::Incomplete
        }
    }
}

impl ServerHandler for DegradedServer {
    /// The daemon's own identity: the same construction, so the
    /// handshake bytes cannot differ between the modes.
    fn get_info(&self) -> ServerConfig {
        crate::server_config()
    }

    /// Every tool call answers the io carrier — the daemon is
    /// unreachable and the tool did not run. Argument validation is
    /// moot (every call answers, unknown tools included); a client
    /// may retry on the code.
    async fn call_tool(
        &self,
        _request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        Ok(crate::error_carrier("io", &self.message).into())
    }

    /// The real tool list — the same shape the macro-generated
    /// handler answers (the cache hints keyed to the negotiated
    /// version, the result type rmcp strips for legacy peers), so
    /// the bytes cannot differ between the modes.
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let supports_cache_hints = context
            .protocol_version()
            .is_some_and(|version| version >= ProtocolVersion::V_2026_07_28);
        Ok(ListToolsResult {
            result_type: Some(ResultType::COMPLETE),
            tools: self.tools.clone(),
            meta: None,
            next_cursor: None,
            ttl_ms: supports_cache_hints.then_some(0),
            cache_scope: supports_cache_hints.then_some(CacheScope::Public),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// The preamble's emission: the settlement's spelling (repo then
    /// cwd), either field alone, and the empty object — the shapes
    /// the daemon's parse accepts, pinned on the emitting side.
    #[test]
    fn the_preamble_emits_the_settlement_shapes() {
        let repo = RepoSlug::from_str("fixture").expect("a valid slug");
        assert_eq!(
            preamble_line(Some(&repo), Some(Path::new("/repos/fixture"))),
            "{\"repo\":\"fixture\",\"cwd\":\"/repos/fixture\"}\n"
        );
        assert_eq!(preamble_line(Some(&repo), None), "{\"repo\":\"fixture\"}\n");
        assert_eq!(
            preamble_line(None, Some(Path::new("/repos/fixture"))),
            "{\"cwd\":\"/repos/fixture\"}\n"
        );
        assert_eq!(preamble_line(None, None), "{}\n");
    }

    /// The session bit's classification: a result line establishes,
    /// an error line or no bytes refuses, anything else fails open.
    #[test]
    fn the_first_line_classification() {
        assert!(classify_first_line(
            br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25"}}"#
        ));
        assert!(!classify_first_line(
            br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"x"}}"#
        ));
        assert!(!classify_first_line(b""));
        // A notification or an unparseable line fails open: the bytes
        // flow regardless, and the exit code only matters at the end.
        assert!(classify_first_line(
            br#"{"jsonrpc":"2.0","method":"notifications/tools/list_changed"}"#
        ));
        assert!(classify_first_line(b"not json"));
    }

    /// The degraded carrier's message names the socket, the window
    /// and the not-run tool — the facts a retrying client needs.
    #[test]
    fn the_unavailable_message_names_the_facts() {
        let message = unavailable_message(Path::new("/tmp/root/serve.sock"));
        assert!(message.contains("/tmp/root/serve.sock"), "{message}");
        assert!(
            message.contains(&format!("{}s retry window", RETRY_WINDOW.as_secs())),
            "{message}"
        );
        assert!(message.contains("the tool did not run"), "{message}");
    }

    /// The handshake-on-behalf's request line: the string id the
    /// client can never emit, the declared protocol version, and an
    /// honest clientInfo naming the shim — the wire shape the
    /// reconnect sends on the client's behalf.
    #[test]
    fn the_reconnect_initialize_line_names_the_shim() {
        let line = reconnect_initialize_line();
        assert!(line.ends_with('\n'), "the line is newline-terminated");
        let value: serde_json::Value =
            serde_json::from_str(line.trim_end()).expect("the line is JSON");
        assert_eq!(value["id"], RECONNECT_ID, "{line}");
        assert_eq!(value["method"], "initialize", "{line}");
        assert_eq!(
            value["params"]["protocolVersion"],
            ProtocolVersion::LATEST_WITH_INITIALIZE.as_str(),
            "{line}"
        );
        assert_eq!(
            value["params"]["clientInfo"]["name"], "todo-mcp-shim",
            "{line}"
        );
        assert_eq!(
            value["params"]["clientInfo"]["version"],
            env!("CARGO_PKG_VERSION"),
            "{line}"
        );
        // The completion notification: a notification, no id, no
        // response to consume.
        let notification = reconnect_initialized_line();
        let value: serde_json::Value =
            serde_json::from_str(notification.trim_end()).expect("the notification is JSON");
        assert_eq!(
            value["method"], "notifications/initialized",
            "{notification}"
        );
        assert!(value.get("id").is_none(), "{notification}");
    }

    /// The handshake-on-behalf's response check: the shim's id with a
    /// result passes; an error member, a foreign id, or a line that
    /// is not JSON is the fault each message names.
    #[test]
    fn the_handshake_response_check() {
        let ok = format!(
            r#"{{"jsonrpc":"2.0","id":"{RECONNECT_ID}","result":{{"protocolVersion":"2025-11-25"}}}}"#
        );
        assert!(handshake_response_fault(ok.as_bytes()).is_none());
        let refused = format!(
            r#"{{"jsonrpc":"2.0","id":"{RECONNECT_ID}","error":{{"code":-32603,"message":"x"}}}}"#
        );
        let fault = handshake_response_fault(refused.as_bytes()).expect("the error is a fault");
        assert!(fault.contains("refused the handshake"), "{fault}");
        let foreign = br#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        let fault = handshake_response_fault(foreign).expect("a foreign id is a fault");
        assert!(fault.contains("not the shim's"), "{fault}");
        let fault = handshake_response_fault(b"not json").expect("garbage is a fault");
        assert!(fault.contains("not JSON"), "{fault}");
    }
}
