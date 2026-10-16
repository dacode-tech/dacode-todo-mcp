//! The MCP server (spec §2, §6): the rmcp wire surface over stdio.
//!
//! This crate is the binary's library half — the server, its state and
//! the wire DTOs; the argument parse lives in the [`args`] module
//! (U38's split) and `main.rs` owns the runtime and the serve call.
//! The binary translates and never decides: all
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
//! camera, ikvm-html and this repo's own board arrived as sibling
//! dialect modules (t46, t47, t54). A dialect parser is not a wire
//! surface: it decides nothing, it recognizes — a register line that
//! does not parse is a named error, never a silent skip, because an
//! importer that quietly drops an item reports less than the truth
//! while still looking complete.
//!
//! # The concurrency core (§8)
//!
//! rmcp dispatches every request in its own `tokio::spawn` (spec
//! §6.3), so concurrent tool calls reach one store handle at the same
//! time inside this process — the store's advisory lock is per file
//! descriptor and one process holds one, so §8's single-writer
//! guarantee is a *process*-boundary property the server must provide
//! itself. The state is two halves (D25, U65): the **core** — the
//! [`DataRoot`] and the open per-repo handles — behind one
//! `Arc<Mutex<…>>`, and the **session** — the default repo slug, the
//! working directory and the cwd-default memo — behind a second
//! `Mutex` on the server itself. The daemon ([`daemon`]) is the only
//! constructor since the no-subcommand path became the shim (U66):
//! it shares one `Arc` across every connection, because
//! per-connection cores could not even contend correctly —
//! `DataRoot::open` opens the store *locked*, so a second connection
//! resolving the same repo would fail with `locked`, and the
//! advisory lock is per file descriptor — while every connection's
//! session is its own: the preamble's defaults are per-connection.
//! The shim itself ([`shim`]) holds no core at all — it proxies, and
//! its failure mode answers a canned carrier without opening a
//! store. TodoServer::lock_state takes both guards at
//! once and hands the tool body a ServerState view; the guards
//! are taken and dropped **inside the sync tool body, never held
//! across an await**: the tool bodies are `async fn` but contain no
//! `.await` after acquisition — the store work is sync — so the rule
//! holds by construction. A `std::sync::Mutex`, not tokio's, is the
//! stronger discipline here: tokio's would *permit* holding the guard
//! across an await.
//!
//! Reads and writes reach a store differently (D19, signed
//! 2026-10-08): a write resolves through `ServerState::resolve`, which
//! opens the store locked and caches the handle — arming is the
//! writer contract — while a read resolves through
//! `ServerState::resolve_read`, which borrows the held graph on a
//! handles hit (U20(b)) and, on a miss, reads the committed prefix
//! lock-free through [`DataRoot::read_repo`] — no lock taken, no
//! handle cached — so a second process (a render, an inspector — the
//! f110 shape) never contends with a live session server.
//!
//! Poisoning is recovered, not fatal: the panic-family lints are
//! denied workspace-wide, so the guarded region cannot panic and
//! poison is unreachable; `PoisonError::into_inner` recovers the guard
//! rather than killing the server if that invariant is ever wrong.
//!
//! # The cwd default tier (D8, armed at t53)
//!
//! An absent `repo` parameter resolves through three tiers (§7): an
//! explicit parameter, then the `--repo` flag's configured default
//! (the daemon connection's preamble repo), then the working
//! directory's deterministic match — the registered checkout whose
//! canonicalized path is the longest component-wise ancestor of the
//! canonicalized cwd, ties refused. The binary reads the cwd once at
//! startup and passes it in (a daemon connection's preamble carries
//! the *shim's* cwd instead); this crate never touches the
//! environment (the private `cwd` module owns the mechanics,
//! `repo_of` the tier order), and the derived default is fixed per
//! session at first use (D3) — a `repo_set_path` or a later
//! `repo_register` does not move it.
//!
//! # The progress render (D20)
//!
//! The same binary carries the tool CLI (§13, D20): subcommands over
//! the store — `progress` (the total-progress document and HTML page
//! [`render`] computes from one repo's graph, read lock-free (D19) so
//! a render never contends with a live session server; both surfaces
//! golden-pinnable, which is the render's purity, not the refresh's)
//! and `refresh` ([`refresh::refresh`], the on-demand re-import: the
//! registry surgery, the re-register, the apply — the store is a
//! derived cache of the board and this rebuilds it). The render is a
//! pure function of `(root, repo)` — no clock, no git, no
//! environment — which is what makes its surfaces golden-pinnable;
//! the page carries a project switcher driven by the
//! registry (the `?repo=<slug>` URL shape the `serve` subcommand will
//! answer). The binary owns the one cwd read and the output writes;
//! the lib's [`args`] module owns the argument parse (U38's split,
//! landed with t72), and the module owns the model. The inspector
//! the `todo-cli` crate carried is absorbed here as [`inspect`] —
//! the `dump`/`verify`/`repair` subcommands (D20, t72) — formatting,
//! prompting and picking exit codes over the store's own facts,
//! every classification and byte offset the store's; the crate
//! retired with the move.
//!
//! # The live listener (D20, t71; D13, t85; D25, U66)
//!
//! `serve` ([`serve::serve`]) is the live view: a listener over
//! hand-rolled HTTP/1.1 on 127.0.0.1 — every route a fresh render,
//! the repo a URL parameter — plus the WebSocket push channel, the
//! port published per-root (`<root>/serve.port`) so a second
//! invocation attaches instead of starting a second listener. The
//! daemon owns the listener now (D25's retirement, U66): it ensures
//! the root's live view once at startup through the same
//! `serve::Watchdog::ensure` the `serve` entry backs, and the
//! listener lives exactly as long as the daemon — the no-subcommand
//! path became the shim, which never touches the listener, and the
//! watchdog's takeover loop retired with it. The module stays
//! parameterized and env-free: the default port-file path derives
//! from the root, and only the `serve` subcommand owns a SIGINT
//! shutdown — the daemon ends by SIGTERM's drain, and the next
//! owner's ensure handles the stale publication.
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
//!
//! Over the file-size target by recorded exception: the state split
//! (U65) grew the concurrency core's documentation past the target,
//! and the shim's landing (U66) grew it again — the
//! `server_config`/`merged_router` extraction the degraded failure
//! mode and the daemon's per-connection servers share. The split,
//! when the next growth arrives, is the state core — `CoreState`,
//! `SessionState`, the `ServerState` view, `resolve`/`resolve_read`
//! and `repo_of` — moving to its own module beside the cluster
//! files.

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
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    CallToolResult, ErrorData, Implementation, ProtocolVersion, ServerCapabilities, ServerConfig,
};
use rmcp::tool_handler;
use serde::Serialize;
use todo_domain::{DomainError, Graph, RepoSlug, ValueError};
use todo_store::{DataRoot, Registry, Store, StoreError};

pub mod args;
mod cwd;
pub mod daemon;
mod dto;
mod edges;
pub mod import;
mod importer;
pub mod inspect;
mod listing;
mod nodes;
mod query;
pub mod refresh;
mod registry;
pub mod render;
mod rewards;
mod schema;
pub mod serve;
pub mod shim;
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
    roll-up. When the user assigns a work stream (one epic), pass its \
    `epic` on every selection call — `next_task`, `next_items`, \
    `task_list` and `status_summary` all take it; sessions scoped to \
    different epics of one repository never cross. Closing a task requires \
    `verify` and `evidence`; `blocked` and \
    `deferred` require a `note`; signing a decision requires a \
    `resolution`. Writes return the new state. Errors carry a stable code \
    in `structuredContent.code` — branch on the code, never parse the \
    message. Tool arguments are strict JSON — every string value quoted, \
    enum spellings exactly as the schema lists them. If a call fails, read \
    the error text — argument faults carry no code, and the text is the \
    remedy: it names the missing or unknown field, and a misspelled \
    value's expected variants; a JSON parse error means the arguments you \
    emitted were not valid JSON — an unquoted value, the value itself \
    usually right — so quote it. Correct the arguments and retry once. \
    The same failure twice means stop and tell the user, quoting the tool, \
    the arguments and the error verbatim — never work around a failing \
    tracker by editing files or guessing state. Tasks carry an agent-set \
    `priority` (high, medium, low) — set \
    it when triaging, prefer higher priority when choosing among \
    actionable rows yourself; the server's order is dependency-driven and \
    ignores it by design. A task closure's response may carry a reflection \
    question — a milestone epic's completion or the root's reward total \
    crossing a threshold, asking how the reward feels. Answer it honestly, \
    once, through the reward_reflect tool; the verbatim answer is logged \
    as-is.";

/// The process-wide half of the server's state (§8, D25): the
/// injected data root and the open per-repo handles — the half every
/// connection shares in the daemon, behind one lock, because the
/// store's advisory lock is per file descriptor and the handles cache
/// is the process's single-writer surface.
struct CoreState {
    /// The injected data root (D4): every store and the shared registry
    /// live under it. The location is the caller's configuration
    /// (§13/D8), never a heuristic this crate grows.
    root: DataRoot,
    /// The open, locked store per registered repo, keyed by slug and
    /// iterated in sorted-slug order (`BTreeMap`) — U20(b)'s
    /// foundation: on a handles hit, the read tools answer from these
    /// handles without cold opens; a miss reads lock-free instead of
    /// arming (D19, [`ServerState::resolve_read`]). Each is boxed
    /// into the [`Store`] trait object at insert, so no engine type
    /// is held or named (§2); the `+ Send` bound is on the trait
    /// object, not the trait — rmcp dispatches requests concurrently
    /// (§6.3), so the state must cross threads. A handle keeps its
    /// store's advisory lock, which is the point: this server is the
    /// single writer for every repo it holds (§8) — and only a write
    /// arms one.
    handles: BTreeMap<RepoSlug, Box<dyn Store + Send>>,
}

/// The per-connection half (D8, D25): the session's default repo
/// slug, its working directory and the cwd-default memo — the half
/// the daemon gives each connection its own of, constructed from the
/// socket preamble. The stdio server's is the `--repo` flag and the
/// binary's startup cwd.
struct SessionState {
    /// The configured default repo slug (the committed D8): applied by
    /// [`repo_of`] whenever a tool's `repo` parameter is absent — an
    /// explicit parameter always wins. `None` until the caller passes
    /// the flag's parsed value (or the preamble's repo); this tier is
    /// configuration, and it outranks the cwd-derived tier below (§7).
    default_repo: Option<RepoSlug>,
    /// The working directory, read once and eagerly by the binary (D1)
    /// — or carried by a daemon connection's preamble, which is the
    /// *shim's* cwd, the client's working directory — `None` when the
    /// read failed or the preamble omitted it. This crate never reads
    /// the environment: the cwd tier's derivation is a function of
    /// this value, so the lib stays testable and the binary owns the
    /// one env touch.
    cwd: Option<PathBuf>,
    /// The cwd default's per-session memo (D3): `Pending` until the
    /// first default-needing call derives it, then the derived
    /// [`cwd::MatchOutcome`] — fixed for the session, so a later
    /// `repo_set_path` or `repo_register` does not move it. An
    /// errored derivation leaves it `Pending`: the memo caches
    /// answers, not faults.
    cwd_default: cwd::CwdDefault,
}

/// The locked view every tool body works over: the core's guard and
/// the session's guard, taken together by [`TodoServer::lock_state`]
/// so a tool body's store work runs under §8's serialization point
/// while its default resolution reads this connection's own session.
/// The core guard lives as long as the view — the `resolve` borrow
/// contract — and neither guard is ever held across an await (the
/// module doc's rule).
struct ServerState<'s> {
    /// The shared core's guard: root and handles.
    core: MutexGuard<'s, CoreState>,
    /// This connection's session guard: defaults and the memo.
    session: MutexGuard<'s, SessionState>,
}

/// The MCP server: the macro-generated tool router plus the §8
/// concurrency core.
pub struct TodoServer {
    /// The router `registry.rs`'s `#[tool_router]` block builds.
    tool_router: ToolRouter<Self>,
    /// The core — root and handles behind one lock (§8) — one `Arc`
    /// per process in the daemon, shared across every connection; a
    /// private one in the stdio server.
    core: Arc<Mutex<CoreState>>,
    /// The session — this connection's defaults and memo. A second
    /// `Mutex` because rmcp dispatches requests concurrently (§6.3)
    /// and the memo's first use mutates it; the lock order is always
    /// core-then-session, so no path can deadlock.
    session: Mutex<SessionState>,
}

impl TodoServer {
    /// The daemon's constructor (D25, U65): shares one core — the
    /// root and the handles cache behind one lock — across every
    /// connection, while this connection's session carries the
    /// preamble's own defaults. Per-connection cores cannot work:
    /// `DataRoot::open` opens the store *locked*, so a second
    /// connection resolving the same repo would fail with `locked`.
    /// The daemon is the only constructor since the no-subcommand
    /// path became the shim (U66): the shim proxies to this server
    /// and its failure mode answers a canned carrier, so no other
    /// path constructs one.
    pub(crate) fn with_core(
        core: Arc<Mutex<CoreState>>,
        default_repo: Option<RepoSlug>,
        cwd: Option<PathBuf>,
    ) -> Self {
        let tool_router = Self::merged_router();
        Self {
            tool_router,
            core,
            session: Mutex::new(SessionState {
                default_repo,
                cwd,
                cwd_default: cwd::CwdDefault::Pending,
            }),
        }
    }

    /// The merged, schema-flattened tool router: every cluster's
    /// `#[tool_router]` block composed, the input schemas' enum
    /// flatten (t88) applied once per tool at startup. The one
    /// construction both server shapes share — the daemon's
    /// per-connection servers, and the shim's degraded failure mode,
    /// whose `tools/list` is the router's advertisement with no
    /// store behind it.
    pub(crate) fn merged_router() -> ToolRouter<Self> {
        // The router composes across impl blocks (`ToolRouter::merge`):
        // the cluster files' routers — `registry.rs`'s `tool_router`,
        // `query.rs`'s `query_router`, `listing.rs`'s
        // `listing_router`, `nodes/mod.rs`'s `nodes_router` (the mutation
        // cluster's home file, with its family modules' `docs_router`
        // and `decisions_router` — U78's split), `edges.rs`'s
        // `edges_router`, `importer.rs`'s `import_router`,
        // `rewards.rs`'s `rewards_router` — each cluster file owning
        // its own block and no cluster editing another's. Merge order
        // is wire-invisible: the router is a name-keyed map and
        // `list_all` sorts by name (§6.3), so later clusters follow
        // the same pattern.
        let mut tool_router = Self::tool_router();
        tool_router.merge(Self::query_router());
        tool_router.merge(Self::listing_router());
        tool_router.merge(Self::nodes_router());
        tool_router.merge(Self::docs_router());
        tool_router.merge(Self::decisions_router());
        tool_router.merge(Self::edges_router());
        tool_router.merge(Self::import_router());
        tool_router.merge(Self::rewards_router());
        // The input schemas' enum flatten (t88): the `#[tool]` macro
        // generates them inside rmcp, so the merged router is the one
        // seam this crate owns over what `tools/list` advertises —
        // every route's schema is rewritten in place before the first
        // request, once per tool at startup.
        for route in tool_router.map.values_mut() {
            schema::flatten_input(&mut route.attr.input_schema);
        }
        tool_router
    }

    /// An empty shared core over `root` (D25): no handles open — only
    /// a write arms one (D19) — the one `Arc` a daemon hands every
    /// connection.
    pub(crate) fn shared_core(root: DataRoot) -> Arc<Mutex<CoreState>> {
        Arc::new(Mutex::new(CoreState {
            root,
            handles: BTreeMap::new(),
        }))
    }

    /// Takes both state locks — §8's serialization point (the core)
    /// beside this connection's session — and hands the tool body the
    /// one [`ServerState`] view. The guards must never live across an
    /// await (the module doc's rule); every current caller is a sync
    /// body.
    fn lock_state(&self) -> ServerState<'_> {
        // The panic-family lints are denied workspace-wide, so the
        // guarded region cannot panic and poison is unreachable;
        // into_inner recovers rather than killing the server.
        ServerState {
            core: self.core.lock().unwrap_or_else(PoisonError::into_inner),
            session: self.session.lock().unwrap_or_else(PoisonError::into_inner),
        }
    }
}

impl ServerState<'_> {
    /// The writer's repo dimension: resolve `slug` to an open, locked
    /// store handle — the arming resolve, every write tool's path
    /// (nodes, edges, the importer, `repo_set_path`). A handles hit
    /// returns the cached handle; a miss consults the registry — an
    /// unlisted slug is `unknown_id`, the `repo_show` pattern (t31),
    /// never a raw io — then opens the store and caches it, so the
    /// first write per repo pays one open and every later call reads
    /// the handle: U20(b)'s no-cold-opens-on-the-answer-path, the
    /// cold open amortized to once per repo per process. A registered
    /// store that will not open propagates the store's own error
    /// (`corrupt_store`/`io`) through the carrier. Read tools take
    /// [`Self::resolve_read`] instead — the same miss discipline
    /// minus the lock and the arming (D19).
    ///
    /// Called under the core lock; the registry load happens only on
    /// a miss. The returned borrow lives as long as the view — the
    /// guard is the store work's serialization.
    fn resolve(&mut self, slug: &RepoSlug) -> Result<&mut Box<dyn Store + Send>, ToolError> {
        // One deref of the guard, so the entry's mutable borrow of
        // `handles` and the registry's shared borrow of `root` split
        // as the disjoint fields they are.
        let core = &mut *self.core;
        match core.handles.entry(slug.clone()) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let registry = Registry::load(&core.root)?;
                if !registry.contains(slug) {
                    return Err(ToolError::from(DomainError::UnknownId {
                        id: slug.to_string(),
                    }));
                }
                let store = core.root.open(slug)?;
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

    /// The read tools' repo dimension (D19): resolve `slug` to a graph
    /// without taking the store's lock and without arming a handle.
    /// A handles hit borrows the held graph — U20(b) unchanged, the
    /// armed path byte-identical. A miss mirrors [`Self::resolve`]'s
    /// discipline minus the lock and the arming: the registry consult
    /// (an unlisted slug is `unknown_id`, never a raw io —
    /// [`DataRoot::read_repo`] is path-derived and would otherwise
    /// read an unregistered-but-present directory silently), then the
    /// lock-free read of the committed prefix, then the slug
    /// cross-check — a store directory copied under another slug is
    /// `corrupt_store` with the same detail the locked open produces,
    /// never the copied graph served as the named repo. Nothing is
    /// cached and nothing is written: a read never arms, so an
    /// unarmed server stays unarmed — only writes open handles.
    ///
    /// Takes `&self` because it mutates nothing; called under the
    /// core lock like every tool body.
    fn resolve_read(&self, slug: &RepoSlug) -> Result<GraphSource<'_>, ToolError> {
        let core = &*self.core;
        if let Some(handle) = core.handles.get(slug) {
            return Ok(GraphSource::Held(handle.graph()));
        }
        let registry = Registry::load(&core.root)?;
        if !registry.contains(slug) {
            return Err(ToolError::from(DomainError::UnknownId {
                id: slug.to_string(),
            }));
        }
        let graph = core.root.read_repo(slug)?;
        // The mirror of resolve's cross-check: `read_repo` is
        // path-derived and would otherwise serve a store directory
        // copied under this slug as the named repo's graph.
        let repo = graph.repo();
        if &repo.slug != slug {
            return Err(ToolError::from(StoreError::CorruptStore {
                offset: None,
                detail: format!("the store opened for {slug} holds repo {}", repo.slug),
            }));
        }
        Ok(GraphSource::Read(Box::new(graph)))
    }
}

/// Where a read tool's graph came from: the held handle's on a hit
/// (U20(b)) or the lock-free read's on a miss (D19). The hit stays a
/// borrow — zero-cost, the armed path byte-identical — while the miss
/// owns its graph (boxed: the graph dwarfs the borrow, and the enum
/// is a per-call local, never stored), an answer holdable beside live
/// writer handles elsewhere.
enum GraphSource<'a> {
    /// The held handle's graph, borrowed: the server already holds
    /// this repo's store, so the read answers from it (U20(b)).
    Held(&'a Graph),
    /// The lock-free read's graph, owned: no handle, no lock, the
    /// committed prefix (D19).
    Read(Box<Graph>),
}

impl GraphSource<'_> {
    /// The graph to answer from, whichever arm produced it.
    fn graph(&self) -> &Graph {
        match self {
            GraphSource::Held(graph) => graph,
            GraphSource::Read(graph) => graph,
        }
    }
}

/// The repo dimension every relaxed tool shares (the committed D8,
/// armed at t53): an explicit parameter always wins — `Some` parses,
/// and a malformed slug is §6.1's `invalid` (U9's mapping of the
/// domain's `ValueError`) before the store is touched. An absent one
/// resolves through the remaining tiers: the `--repo` flag's
/// configured default, else the derived cwd default — derived and
/// cached at first use (D3), the deterministic longest-ancestor match
/// of §7 — else §6.1's `invalid`. The tool body owns the absent case
/// because the input schema is static and cannot express "required
/// iff no default": that is instance state, not a shape, so the
/// schema cannot carry the rejection and the body must.
///
/// Takes `&mut ServerState` because the cwd tier's first use derives
/// and caches the outcome in the memo (D3): the derivation itself
/// reads every registered slug's `Repo` node lock-free (D19) and arms
/// nothing, but the memo write is a mutation. Every call site already
/// held `&mut` — the writes call [`ServerState::resolve`] after, the
/// reads [`ServerState::resolve_read`].
pub(crate) fn repo_of(
    state: &mut ServerState<'_>,
    repo: Option<&str>,
) -> Result<RepoSlug, ToolError> {
    match repo {
        Some(value) => RepoSlug::from_str(value).map_err(ToolError::from),
        None => {
            // Tier 2: the --repo flag's configured default (or the
            // preamble's), fixed at the session's construction.
            if let Some(slug) = state.session.default_repo.clone() {
                return Ok(slug);
            }
            // Tier 3: the derived cwd default, fixed at first use —
            // the memo caches the outcome, and an errored derivation
            // leaves it Pending so the next call retries (D3).
            let outcome = match state.session.cwd_default.clone() {
                cwd::CwdDefault::Resolved(outcome) => outcome,
                cwd::CwdDefault::Pending => {
                    let outcome = cwd::derive(state)?;
                    state.session.cwd_default = cwd::CwdDefault::Resolved(outcome.clone());
                    outcome
                }
            };
            match outcome {
                cwd::MatchOutcome::Match(slug) => Ok(slug),
                cwd::MatchOutcome::NoMatch => Err(ToolError::invalid(
                    "the repo parameter is absent and neither the --repo flag nor the \
                     working directory produced a default repo",
                )),
                cwd::MatchOutcome::Ambiguous(slugs) => {
                    Err(ToolError::invalid(&ambiguous_message(&slugs)))
                }
            }
        }
    }
}

/// The Ambiguous refusal's message (D4): names the tied slugs in the
/// walk's slug-ascending order, so the client can see exactly what
/// the working directory matched.
fn ambiguous_message(slugs: &[RepoSlug]) -> String {
    let names: Vec<&str> = slugs.iter().map(RepoSlug::as_str).collect();
    format!(
        "the repo parameter is absent and the working directory matches multiple \
         repositories: {}",
        names.join(", ")
    )
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
        server_config()
    }
}

/// The server's identity and instructions (U16a): the one
/// construction every server shape shares — the daemon's
/// per-connection sessions and the shim's degraded failure mode
/// answer the same handshake bytes, so a client cannot tell the
/// modes apart from the initialize response.
pub(crate) fn server_config() -> ServerConfig {
    ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(Implementation::new("todo-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(INSTRUCTIONS)
        .with_protocol_version(ProtocolVersion::LATEST_WITH_INITIALIZE)
}
