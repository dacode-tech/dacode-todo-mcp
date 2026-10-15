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
//! stdin is still open is the reconnect's trigger (the `proxy` module
//! doc); the reconnect's failure — the window expired, the handshake
//! refused — is the abnormal end: one stderr line, exit 1. The exit is
//! delivered regardless of the client: the binary's
//! bounded runtime teardown cuts the client task's in-flight
//! blocking stdin read rather than letting the runtime's drop wait
//! it out.
//!
//! The proxy phase — the socket slot, the two copy directions and
//! the generation loop, with the reconnect and its
//! handshake-on-behalf — lives in `proxy` (U70's split); this
//! module keeps the entry, the connect phase and the failure mode.
//!
//! The module translates and proxies, never decides (§2): no store
//! is opened on any path here, and nothing new reaches the wire
//! except the preamble's line.

mod proxy;

use self::proxy::proxy;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use rmcp::ServerHandler;
use rmcp::ServiceExt;
use rmcp::model::{
    CacheScope, CallToolRequestParams, CallToolResponse, ErrorData, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ResultType, ServerConfig, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
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
}
