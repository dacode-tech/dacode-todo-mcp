//! The proxy phase (U70's split out of `shim`): the generation loop
//! that copies both directions across the daemon's socket — the
//! socket slot, the two copy tasks, the handshake-on-behalf and the
//! reconnect that drives them. The connect phase it reuses, the entry
//! that dispatches to this loop, and the degraded failure mode stay
//! in the parent module.
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
//! after an update — within the same [`crate::shim::RETRY_WINDOW`].
//! The fresh daemon-side session must be initialized before it will
//! answer, and the client cannot re-send `initialize` (its session is
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

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rmcp::model::ProtocolVersion;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::UnixStream;
use tokio::sync::Notify;

use todo_domain::RepoSlug;
use todo_store::DataRoot;

use super::{ConnectPhase, ShimOutcome, connect};

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
pub(super) async fn proxy(
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

#[cfg(test)]
mod tests {
    use super::*;

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
