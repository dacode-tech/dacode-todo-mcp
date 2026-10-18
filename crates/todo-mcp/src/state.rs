//! The server's state core (U68's split out of the crate root): the
//! two-halves state, the locked view every tool body works over, the
//! repo dimension's resolves and default tiers, and the server itself
//! — its constructor, its merged router and its handshake. The
//! cluster files, the daemon and the shim reach these through the
//! crate root's re-exports, so no `use` outside this module changed
//! with the split.
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
//! `Mutex` on the server itself. The daemon ([`crate::daemon`]) is the only
//! constructor since the no-subcommand path became the shim (U66):
//! it shares one `Arc` across every connection, because
//! per-connection cores could not even contend correctly —
//! `DataRoot::open` opens the store *locked*, so a second connection
//! resolving the same repo would fail with `locked`, and the
//! advisory lock is per file descriptor — while every connection's
//! session is its own: the preamble's defaults are per-connection.
//! The shim itself ([`crate::shim`]) holds no core at all — it proxies, and
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

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerConfig};
use rmcp::tool_handler;
use todo_domain::{DomainError, Graph, RepoSlug};
use todo_store::{DataRoot, Registry, Store, StoreError};

use crate::ToolError;
use crate::{cwd, schema};

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
    different epics of one repository never cross. One epic's subtree is \
    `graph_show` with the `epic` parameter — there is no `epic_show` \
    tool; the per-node shows are `repo_show`, `task_show` and \
    `decision_show`. Closing a task requires \
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
pub(crate) struct CoreState {
    /// The injected data root (D4): every store and the shared registry
    /// live under it. The location is the caller's configuration
    /// (§13/D8), never a heuristic this crate grows.
    pub(crate) root: DataRoot,
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
    pub(crate) handles: BTreeMap<RepoSlug, Box<dyn Store + Send>>,
}

/// The per-connection half (D8, D25): the session's default repo
/// slug, its working directory and the cwd-default memo — the half
/// the daemon gives each connection its own of, constructed from the
/// socket preamble. The stdio server's is the `--repo` flag and the
/// binary's startup cwd.
pub(crate) struct SessionState {
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
    pub(crate) cwd: Option<PathBuf>,
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
pub(crate) struct ServerState<'s> {
    /// The shared core's guard: root and handles.
    pub(crate) core: MutexGuard<'s, CoreState>,
    /// This connection's session guard: defaults and the memo.
    pub(crate) session: MutexGuard<'s, SessionState>,
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
    pub(crate) fn lock_state(&self) -> ServerState<'_> {
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
    pub(crate) fn resolve(
        &mut self,
        slug: &RepoSlug,
    ) -> Result<&mut Box<dyn Store + Send>, ToolError> {
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
    pub(crate) fn resolve_read(&self, slug: &RepoSlug) -> Result<GraphSource<'_>, ToolError> {
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
pub(crate) enum GraphSource<'a> {
    /// The held handle's graph, borrowed: the server already holds
    /// this repo's store, so the read answers from it (U20(b)).
    Held(&'a Graph),
    /// The lock-free read's graph, owned: no handle, no lock, the
    /// committed prefix (D19).
    Read(Box<Graph>),
}

impl GraphSource<'_> {
    /// The graph to answer from, whichever arm produced it.
    pub(crate) fn graph(&self) -> &Graph {
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
