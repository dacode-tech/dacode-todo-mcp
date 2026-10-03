//! The MCP server (spec §2, §6): the rmcp wire surface over stdio.
//!
//! This crate is the binary's library half — the server, its state and
//! the wire DTOs; `main.rs` owns the argument parse, the runtime and
//! the serve call. The binary translates and never decides: all
//! selection, ordering and query logic lives in `todo-domain`, all
//! persistence in `todo-store`, and this crate maps tool calls onto
//! those. The storage engine stays sealed inside `todo-store` (§2):
//! its constructors return the concrete engine, and this crate boxes
//! each store into the [`Store`] trait object at the boundary — the
//! shape `todo-store`'s own lib.rs provisioned for M3's server — so no
//! engine type, error or feature flag is held or named here.
//!
//! # The concurrency core (§8)
//!
//! rmcp dispatches every request in its own `tokio::spawn` (spec
//! §6.3), so concurrent tool calls reach one store handle at the same
//! time inside this process — the store's advisory lock is per file
//! descriptor and one process holds one, so §8's single-writer
//! guarantee is a *process*-boundary property the server must provide
//! itself. [`TodoServer`] holds its `ServerState` — the [`DataRoot`] and
//! the open per-repo handles — behind one `Arc<Mutex<…>>`, and every
//! tool body takes that lock. The guard is taken and dropped **inside
//! the sync tool body, never held across an await**: the tool bodies
//! are `async fn` but contain no `.await` after acquisition — the
//! store work is sync — so the rule holds by construction. A
//! `std::sync::Mutex`, not tokio's, is the stronger discipline here:
//! tokio's would *permit* holding the guard across an await.
//!
//! Poisoning is recovered, not fatal: the panic-family lints are
//! denied workspace-wide, so the guarded region cannot panic and
//! poison is unreachable; `PoisonError::into_inner` recovers the guard
//! rather than killing the server if that invariant is ever wrong.
//!
//! # The error carrier (D15)
//!
//! Every tool failure travels through the `error_carrier` helper as
//! [`CallToolResult::structured_error`] with
//! `structuredContent = { "code", "message" }` and the same JSON
//! mirrored into the text block (§6.1): a client branches on the code,
//! a model still reads the message. Codes are §6.1's closed list —
//! answered by `StoreError::code()` for store faults, `"invalid"` for
//! the domain's `ValueError` (U9's mapping). Argument faults — a wrong
//! type, a missing or unknown field — are rmcp's own unstructured text
//! errors and never reach the tool body (§6.3), so they arrive
//! codeless by design and no code is invented for them.

// Tests are exempt from the panic-family lints through a mechanism, not
// a convention (the workspace's rule, stated once in the root
// Cargo.toml's lint table).
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    CallToolResult, Implementation, ProtocolVersion, ServerCapabilities, ServerConfig,
};
use rmcp::tool_handler;
use todo_domain::RepoSlug;
use todo_store::{DataRoot, Store};

mod registry;
mod time;

/// U16(a)'s base instructions: the one text an agent that has never
/// seen this repository reads before any tool description. It
/// describes §6's signed surface completely even though the tools land
/// task by task — no client exists before the smoke test, and the text
/// has this one home rather than growing per task. §6 signs the tool
/// list, not this text: it is an add-deliverable the user may amend.
const INSTRUCTIONS: &str = "You are connected to a deterministic task-graph server. \
    Work is tracked as typed nodes: an epic groups work, a task is one unit \
    (kinds: task, gate, finding, debt), a decision is a dossier that blocks \
    work until the user signs it, and a doc is a registered document cited \
    by stable id. Ids are stable and never reused. Ask `next_task` for what \
    to do now, `next_items` for the batch, `status_summary` for the \
    roll-up. Closing a task requires `verify` and `evidence`; `blocked` and \
    `deferred` require a `note`; signing a decision requires a \
    `resolution`. Writes return the new state. Errors carry a stable code \
    in `structuredContent.code` — branch on the code, never parse the \
    message.";

/// The server's mutable state, held behind one lock (§8): the injected
/// data root and the open per-repo handles.
struct ServerState {
    /// The injected data root (D4): every store and the shared registry
    /// live under it. The location is the caller's configuration
    /// (§13/D8), never a heuristic this crate grows.
    root: DataRoot,
    /// The open, locked store per registered repo, keyed by slug and
    /// iterated in sorted-slug order (`BTreeMap`) — U20(b)'s
    /// foundation: the registry queries answer from these handles
    /// without cold opens. Each is boxed into the [`Store`] trait
    /// object at insert, so no engine type is held or named (§2); the
    /// `+ Send` bound is on the trait object, not the trait — rmcp
    /// dispatches requests concurrently (§6.3), so the state must
    /// cross threads. A handle keeps its store's advisory lock, which
    /// is the point: this server is the single writer for every repo
    /// it holds (§8).
    handles: BTreeMap<RepoSlug, Box<dyn Store + Send>>,
}

/// The MCP server: the macro-generated tool router plus the §8
/// concurrency core.
pub struct TodoServer {
    /// The router `registry.rs`'s `#[tool_router]` block builds.
    tool_router: ToolRouter<Self>,
    /// The one lock every tool body takes (the module doc's rule).
    state: Arc<Mutex<ServerState>>,
}

impl TodoServer {
    /// A server serving every store under `root`.
    pub fn new(root: DataRoot) -> Self {
        Self {
            tool_router: Self::tool_router(),
            state: Arc::new(Mutex::new(ServerState {
                root,
                handles: BTreeMap::new(),
            })),
        }
    }

    /// Takes the state lock — §8's serialization point, and the one
    /// entry every tool body shares. The guard must never live across
    /// an await (the module doc's rule); every current caller is a
    /// sync body.
    fn lock_state(&self) -> MutexGuard<'_, ServerState> {
        // The panic-family lints are denied workspace-wide, so the
        // guarded region cannot panic and poison is unreachable;
        // into_inner recovers rather than killing the server.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// D15's error carrier: a §6.1 code and its message as a tool-level
/// error result — `isError: true`, `structuredContent.code` for a
/// client to branch on, the same JSON mirrored into the text block for
/// a model to read. Every tool failure goes through this one helper;
/// t34+ inherit it.
pub(crate) fn error_carrier(code: &str, message: &str) -> CallToolResult {
    CallToolResult::structured_error(serde_json::json!({
        "code": code,
        "message": message,
    }))
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for TodoServer {
    /// The server's identity and instructions (U16a): our own
    /// `serverInfo` — unset, every client sees rmcp's own
    /// `rmcp/3.5.0` (§6.3's measured default) — the base instructions,
    /// and the handshake version this server implements,
    /// `LATEST_WITH_INITIALIZE` (2025-11-25). The 2026-07-28 discover
    /// flow is served by the same `serve` call: rmcp dispatches it from
    /// the handler's defaults, no extra surface of ours.
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("todo-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
            .with_protocol_version(ProtocolVersion::LATEST_WITH_INITIALIZE)
    }
}
