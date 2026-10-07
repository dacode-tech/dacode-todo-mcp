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
//! # The importer (§9, D12)
//!
//! The crate also hosts the board importer: [`import::dacode`] is the
//! dacode dialect's parser, a pure function of text with no I/O, no
//! clock and no store — file reading belongs to the `import_md` wire
//! tool (`importer.rs`), the mapping to domain types to t43, and
//! camera and ikvm-html arrived as sibling dialect modules (t46,
//! t47). A
//! dialect parser is not a wire surface: it decides nothing, it
//! recognizes — a register line that does not parse is a named error,
//! never a silent skip, because an importer that quietly drops an
//! item reports less than the truth while still looking complete.
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
//! a model still reads the message. Codes are §6.1's closed list,
//! mapped once by `ToolError` — answered by `StoreError::code()` for
//! store faults, `DomainError::code()` for domain faults, `"invalid"`
//! for the domain's `ValueError` (U9's mapping) — so a tool's failure
//! arm is one line and t35+ inherit the mapping rather than restating
//! it. Argument faults — a wrong type, a missing or unknown field —
//! are rmcp's own unstructured text errors and never reach the tool
//! body (§6.3), so they arrive codeless by design and no code is
//! invented for them.
//!
//! # The advertised schemas' format discipline (U32)
//!
//! schemars emits OpenAPI integer formats (`uint64` and kin) that JSON
//! Schema validators do not know — opencode's Ajv warns on every
//! `tools/list` — so every schema this server advertises strips them;
//! the `schema` module owns the strip and its wire pin.

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
use std::collections::btree_map::Entry;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    CallToolResult, ErrorData, Implementation, ProtocolVersion, ServerCapabilities, ServerConfig,
};
use rmcp::tool_handler;
use serde::Serialize;
use todo_domain::{DomainError, RepoSlug, ValueError};
use todo_store::{DataRoot, Registry, Store, StoreError};

mod dto;
mod edges;
pub mod import;
mod importer;
mod listing;
mod nodes;
mod query;
mod registry;
mod schema;
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
    message. Tasks carry an agent-set `priority` (high, medium, low) — set \
    it when triaging, prefer higher priority when choosing among \
    actionable rows yourself; the server's order is dependency-driven and \
    ignores it by design.";

/// The server's mutable state, held behind one lock (§8): the injected
/// data root, the configured default repo slug and the open per-repo
/// handles.
struct ServerState {
    /// The injected data root (D4): every store and the shared registry
    /// live under it. The location is the caller's configuration
    /// (§13/D8), never a heuristic this crate grows.
    root: DataRoot,
    /// The configured default repo slug (the committed D8): applied by
    /// [`repo_of`] whenever a tool's `repo` parameter is absent — an
    /// explicit parameter always wins. `None` until the binary passes
    /// the `--repo` flag's parsed value; the default is configuration,
    /// never a cwd heuristic.
    default_repo: Option<RepoSlug>,
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
    /// A server serving every store under `root`, with an optional
    /// default repo slug (the committed D8): the `--repo` startup
    /// flag's parsed value, applied whenever a tool's `repo` parameter
    /// is absent — an explicit parameter always wins.
    pub fn new(root: DataRoot, default_repo: Option<RepoSlug>) -> Self {
        // The router composes across impl blocks (`ToolRouter::merge`):
        // the cluster files' routers — `registry.rs`'s `tool_router`,
        // `query.rs`'s `query_router`, `listing.rs`'s
        // `listing_router`, `nodes.rs`'s `nodes_router`, `edges.rs`'s
        // `edges_router`, `importer.rs`'s `import_router` — each
        // cluster file owning its own block and no cluster editing
        // another's. Merge order is wire-invisible: the router is a
        // name-keyed map and `list_all` sorts by name (§6.3), so
        // later clusters follow the same pattern.
        let mut tool_router = Self::tool_router();
        tool_router.merge(Self::query_router());
        tool_router.merge(Self::listing_router());
        tool_router.merge(Self::nodes_router());
        tool_router.merge(Self::edges_router());
        tool_router.merge(Self::import_router());
        Self {
            tool_router,
            state: Arc::new(Mutex::new(ServerState {
                root,
                default_repo,
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

impl ServerState {
    /// Every tool's repo dimension: resolve `slug` to an open store
    /// handle. A handles hit returns the cached handle; a miss consults
    /// the registry — an unlisted slug is `unknown_id`, the
    /// `repo_show` pattern (t31), never a raw io — then opens the
    /// store and caches it, so the first query per repo pays one open
    /// and every later query reads the handle: U20(b)'s
    /// no-cold-opens-on-the-query-path, the cold open amortized to
    /// once per repo per process. A registered store that will not
    /// open propagates the store's own error (`corrupt_store`/`io`)
    /// through the carrier.
    ///
    /// Called under the state lock; the registry load happens only on
    /// a miss.
    fn resolve(&mut self, slug: &RepoSlug) -> Result<&mut Box<dyn Store + Send>, ToolError> {
        match self.handles.entry(slug.clone()) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let registry = Registry::load(&self.root)?;
                if !registry.contains(slug) {
                    return Err(ToolError::from(DomainError::UnknownId {
                        id: slug.to_string(),
                    }));
                }
                let store = self.root.open(slug)?;
                // The mirror of the store's own cross-check
                // (`DataRoot::repo_node`, layout.rs): a store directory
                // copied or moved under another slug is `corrupt_store`
                // — the same error, the same detail — never served as
                // the named repo's graph. Runs only on a handles miss:
                // a cached handle was checked at its own open.
                let repo = store.graph().repo();
                if &repo.slug != slug {
                    return Err(ToolError::from(StoreError::CorruptStore {
                        offset: None,
                        detail: format!("the store opened for {slug} holds repo {}", repo.slug),
                    }));
                }
                Ok(entry.insert(Box::new(store)))
            }
        }
    }
}

/// The repo dimension every relaxed tool shares (the committed D8):
/// an explicit parameter always wins — `Some` parses, and a malformed
/// slug is §6.1's `invalid` (U9's mapping of the domain's
/// `ValueError`) before the store is touched — while an absent one
/// answers the configured default, or `invalid` when no default
/// exists. The tool body owns the absent case because the input
/// schema is static and cannot express "required iff no default":
/// that is instance state, not a shape, so the schema cannot carry
/// the rejection and the body must.
pub(crate) fn repo_of(state: &ServerState, repo: Option<&str>) -> Result<RepoSlug, ToolError> {
    match repo {
        Some(value) => RepoSlug::from_str(value).map_err(ToolError::from),
        None => state.default_repo.clone().ok_or_else(|| {
            ToolError::invalid("the repo parameter is absent and no default repo is configured")
        }),
    }
}

/// The one error-mapping layer between the crates below and D15's
/// carrier: a §6.1 code plus its message, converted from
/// [`StoreError`], [`DomainError`] and [`ValueError`] (U9's `invalid`
/// mapping). Every tool's failure arm is
/// `Err(e) => Ok(error_carrier(&e.code, &e.message))` — one home, so
/// t35+ inherit the mapping rather than restating it per cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolError {
    /// The stable §6.1 code a client branches on.
    pub code: String,
    /// The human-readable message, mirrored into the carrier's text
    /// block; never the contract.
    pub message: String,
}

impl ToolError {
    /// §6.1's `invalid` for a rejection this crate makes itself (not a
    /// domain `ValueError`): t34's mutually exclusive `next_items`
    /// scopes.
    pub(crate) fn invalid(message: &str) -> Self {
        Self {
            code: "invalid".to_owned(),
            message: message.to_owned(),
        }
    }
}

impl From<StoreError> for ToolError {
    fn from(error: StoreError) -> Self {
        Self {
            code: error.code().to_owned(),
            message: error.to_string(),
        }
    }
}

impl From<DomainError> for ToolError {
    fn from(error: DomainError) -> Self {
        Self {
            code: error.code().as_str().to_owned(),
            message: error.to_string(),
        }
    }
}

impl From<ValueError> for ToolError {
    fn from(error: ValueError) -> Self {
        Self {
            code: "invalid".to_owned(),
            message: error.to_string(),
        }
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

/// The shared success arm: the DTO serialized into the structured
/// result. The one failure path that bypasses the carrier, on purpose:
/// no §6.1 code names a serializer fault, the path is unreachable for
/// DTOs of strings, integers and booleans, and a protocol-level
/// internal error is the honest carrier for it. One home beside
/// [`error_carrier`] so every cluster — registry, query, and the
/// clusters t35+ land — shares the mechanism instead of copying it.
pub(crate) fn structured<T: Serialize>(tool: &str, output: T) -> Result<CallToolResult, ErrorData> {
    let value = serde_json::to_value(output).map_err(|error| {
        ErrorData::internal_error(
            format!("failed to serialize {tool}'s output: {error}"),
            None,
        )
    })?;
    Ok(CallToolResult::structured(value))
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
