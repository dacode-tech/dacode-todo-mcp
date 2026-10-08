//! The listing cluster's five read tools (§6, t35): `task_list` —
//! D16's annotated rows under the four ANDed filters — the three thin
//! listings `epic_list`, `decision_list` and `doc_list`, and
//! `graph_show`, §6.2's handover dump. Thin translations of the
//! domain's §6 listing surfaces onto the wire, each with one params
//! struct denying unknown fields (§6.1's consequence: an unrecognised
//! argument is otherwise silently dropped), a declared outputSchema
//! (D15) and read-only annotations (§6.3). No clock anywhere: all
//! five are reads over caller-built state, so every response is
//! byte-deterministic against a fixed fixture.
//!
//! # The shapes
//!
//! `task_list`'s params take `repo` as an optional `String` (the
//! committed D8: absent means the configured default repo, an
//! explicit value always wins) and `epic` as a plain `String` parsed
//! with `FromStr` — a malformed value is
//! §6.1's `invalid` through the carrier before the store is touched
//! (the registry.rs/query.rs precedent) — while `status` and `kind`
//! are typed enums: a wrong spelling is rmcp's own codeless argument
//! fault, not `invalid`, because the input schema itself enumerates
//! the closed set and a wrong enum value is an argument-shape fault
//! exactly like a wrong type (§6.1's consequence text: `invalid`
//! covers values this server parses, and the server never parses a
//! vocabulary spelling). The filters AND; an unsatisfiable
//! combination is an empty `Ok` list, and the only `Err` is
//! `unknown_id` for an unknown epic — all domain semantics, none
//! re-implemented.
//!
//! The thin listings answer id+title rows in the domain's own
//! ascending order: §6's tool table names no payload for them, and
//! the domain's module doc leaves the wire shape to this layer —
//! id+title is taken because §6's client flow needs the listing to
//! carry names an agent can show the user when a query needs an epic
//! chosen. No filters: §6 names filters for `task_list` alone.
//!
//! `graph_show`'s `repo` is optional **as §6.2 signs it** — absent
//! means every registered repo in sorted-slug order, which is not the
//! default-slug case and never becomes it. The dump is one answer
//! shape for all three scopes: per repo, the repo node, the epic
//! groups (ascending epic id, each group's tasks in that epic's
//! induced canonical order), the decisions and the docs (ascending);
//! per node, exactly the adjacency fields that can be non-empty for
//! it — every edge visible from both endpoints, no edge table. The
//! epic scope's decision set is the domain's own gate-set surface
//! (`Graph::gating_decisions_of_epic`), never re-derived here. The
//! flattened node fields serialize first, in the node's declaration
//! order, then the outer struct's own fields — D14's pin holding by
//! construction under `preserve_order`.
//!
//! # The failure arm
//!
//! Every tool's failure arm is `Err(e) => Ok(error_carrier(&e.code,
//! &e.message))` over [`crate::ToolError`] — the one error-mapping
//! layer (lib.rs), converting store, domain and value faults. The
//! repo dimension resolves through `ServerState::resolve_read` under
//! the state lock — a handles hit borrows the held graph, a miss
//! reads lock-free (D19); the success arm serializes through the
//! shared `structured` helper, whose serializer fault bypasses the
//! carrier per the registry.rs precedent.
//!
//! Over the file-size target by recorded exception (`_rust.md`, board
//! §5/U24): the cluster's dispatch pins all five tools, their params
//! and their answer DTOs — the dump's builders included — in this one
//! file. The split, if a sixth tool ever joins, is the dump half
//! (`GraphShowDto` and its builders) into its own module.

use std::str::FromStr;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use todo_domain::{
    BlockEnd, Blocker, Decision, DomainError, Epic, EpicId, Graph, RepoSlug, Status, Task,
    TaskFilter, TaskKind, TaskRow, ValueError,
};
use todo_store::Registry;

use crate::dto::{
    DecisionDto, DecisionStatusDto, DocDto, EpicDto, PriorityDto, RepoDto, StatusDto, TaskDto,
    TaskKindDto,
};
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured};

// =================================================================
// The params structs
// =================================================================

/// `task_list`'s parameters (§6): the repository, the optional epic
/// filter, and the three value filters — status, kind and the
/// deferred-scoped readiness — all ANDed by the domain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskListParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The optional epic filter (§6): the scope becomes that epic's tasks in their induced canonical order, e.g. `camera:E1`.
    pub epic: Option<String>,
    /// The optional status filter: one of the six lifecycle statuses, ANDed with the other filters.
    pub status: Option<StatusDto>,
    /// The optional kind filter: one of task, gate, finding, debt, ANDed with the other filters.
    pub kind: Option<TaskKindDto>,
    /// The optional deferred-scoped readiness filter (§6): `true` keeps the deferred tasks whose every predecessor is finished, `false` the held ones.
    pub ready: Option<bool>,
}

/// `epic_list`'s parameters (§6): the repository whose epics are listed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct EpicListParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
}

/// `decision_list`'s parameters (§6): the repository whose decisions are listed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionListParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
}

/// `doc_list`'s parameters (§6): the repository whose docs are listed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocListParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
}

/// `graph_show`'s parameters (§6.2): the dump's scope — neither
/// parameter dumps every registered repo, `repo` alone dumps that
/// repo, and `epic` (with or without an explicit `repo`, which is
/// authoritative when both are given) dumps that epic's subtree plus
/// the decisions gating it.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct GraphShowParams {
    /// The optional repository scope (§6.2): absent means every registered repo in sorted-slug order — the all-repos dump, not the default-slug case, which never applies to this tool.
    pub repo: Option<String>,
    /// The optional epic scope (§6.2): that epic's subtree plus its gating decisions; with `repo` also given, the explicit repo is authoritative.
    pub epic: Option<String>,
}

// =================================================================
// The output DTOs
// =================================================================

/// `task_list`'s answer (§6, D16): the scope's rows in canonical order, each carrying its blocked-by annotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct TaskListDto {
    /// The scope's rows, in §5.2 canonical order — the filtered subsequence, never re-sorted.
    pub tasks: Vec<TaskRowDto>,
}

/// One `task_list` row (§6, D16/D17): the task's id, its blocked-by annotation and its priority — the domain's `TaskRow` mirrored, nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct TaskRowDto {
    /// The task this row is about, e.g. `camera:U7`.
    pub task: String,
    /// What holds the task back, per the domain's two-run rule: its own predecessors, then its epic's decision gates — each blocker carrying its current status.
    pub blocked_by: Vec<BlockerDto>,
    /// The task's agent-set priority (D17) — high, medium, low or null; display-only, never an ordering input.
    pub priority: Option<PriorityDto>,
}

impl From<TaskRow> for TaskRowDto {
    fn from(row: TaskRow) -> Self {
        Self {
            task: row.task.to_string(),
            blocked_by: row.blocked_by.iter().map(BlockerDto::from).collect(),
            priority: row.priority.map(PriorityDto::from),
        }
    }
}

/// One blocker of a `task_list` row: the node's id and its current status, tagged by kind — the store's own tagged-union spelling (`BlocksEdgeDto`'s `shape`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub(crate) enum BlockerDto {
    /// A task predecessor, at its current status.
    Task {
        /// The predecessor's id, e.g. `camera:U7`.
        id: String,
        /// The predecessor's current lifecycle status.
        status: StatusDto,
    },
    /// A decision gate — on the task itself or on its epic — at its current status.
    Decision {
        /// The decision's id, e.g. `camera:D3`.
        id: String,
        /// The decision's current status: open, signed or superseded.
        status: DecisionStatusDto,
    },
}

impl From<&Blocker> for BlockerDto {
    fn from(blocker: &Blocker) -> Self {
        match blocker {
            Blocker::Task { id, status } => Self::Task {
                id: id.to_string(),
                status: (*status).into(),
            },
            Blocker::Decision { id, status } => Self::Decision {
                id: id.to_string(),
                status: (*status).into(),
            },
        }
    }
}

/// `epic_list`'s answer (§6): the repo's epics as id+title rows, ascending by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct EpicListDto {
    /// The repo's epics, ascending by id — registration order.
    pub epics: Vec<EpicRowDto>,
}

/// One `epic_list` row: the epic's id and title — the name an agent shows the user when a query needs an epic chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct EpicRowDto {
    /// The epic's id, e.g. `camera:E1`.
    pub id: String,
    /// The epic's short human title.
    pub title: String,
}

/// `decision_list`'s answer (§6): the repo's decisions as id+title rows, ascending by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DecisionListDto {
    /// The repo's decisions, ascending by id — creation order.
    pub decisions: Vec<DecisionRowDto>,
}

/// One `decision_list` row: the decision's id and title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DecisionRowDto {
    /// The decision's id, e.g. `camera:D3`.
    pub id: String,
    /// The decision's short human title.
    pub title: String,
}

/// `doc_list`'s answer (§6): the repo's docs as id+title rows, ascending by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DocListDto {
    /// The repo's docs, ascending by id — registration order.
    pub docs: Vec<DocRowDto>,
}

/// One `doc_list` row: the doc's id and title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DocRowDto {
    /// The doc's id, e.g. `camera:S1`.
    pub id: String,
    /// The doc's short human title.
    pub title: String,
}

/// `graph_show`'s answer (§6.2): the dump — one answer shape for all three scopes; the scope changes the array's contents, never the schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct GraphShowDto {
    /// The dumped repositories, in sorted-slug order — one entry for the repo and epic scopes, every registered repo for the all-repos scope.
    pub repos: Vec<RepoDumpDto>,
}

/// One repo's half of the dump (§6.2): the repo node, its epic groups, its decisions and its docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct RepoDumpDto {
    /// The repo node itself — the handover context: where the checkout lives.
    pub repo: RepoDto,
    /// The repo's epic groups, ascending by epic id; the epic scope holds the one group.
    pub epics: Vec<EpicGroupDto>,
    /// The repo's decisions, ascending by id, each with the tasks and epics it gates; the epic scope holds the gate set alone.
    pub decisions: Vec<DecisionNodeDto>,
    /// The repo's docs, ascending by id; the epic scope holds none — a doc has no epic field.
    pub docs: Vec<DocDto>,
}

/// One epic group of the dump (§6.2): the epic node, its own gating decisions, and its tasks in the epic's induced canonical order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct EpicGroupDto {
    /// The epic node's every field, flattened into the group — the node's declaration order first, then the group's own fields.
    #[serde(flatten)]
    pub epic: EpicDto,
    /// The epic's own gating decisions, as decision ids, ascending — the `decision→epic` edges.
    pub blocked_by: Vec<String>,
    /// The epic's tasks, in that epic's induced canonical order (§5.2).
    pub tasks: Vec<TaskNodeDto>,
}

/// One task node of the dump (§6.2): the task's every field, flattened, plus its adjacency — every edge visible from both endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct TaskNodeDto {
    /// The task node's every field, flattened into the entry — the node's declaration order first, then the adjacency fields.
    #[serde(flatten)]
    pub task: TaskDto,
    /// What holds the task back, as ids: its own predecessors then its epic's decision gates, the two-run order preserved, statuses dropped.
    pub blocked_by: Vec<String>,
    /// The task's direct `blocks` successors, as ids, ascending — a task's successors are tasks.
    pub blocks: Vec<String>,
    /// The tasks this task carries a `relates` annotation with, as ids, ascending.
    pub relates: Vec<String>,
}

/// One decision node of the dump (§6.2): the dossier's every field, flattened, plus the tasks and epics it gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DecisionNodeDto {
    /// The decision node's every field, flattened into the entry — the node's declaration order first, then the adjacency field.
    #[serde(flatten)]
    pub decision: DecisionDto,
    /// The tasks and epics this decision gates, as ids — task targets ascending, then epic targets, the endpoint order every adjacency list in the domain is documented in (§8).
    pub blocks: Vec<String>,
}

// =================================================================
// The answer helpers — parse, resolve, query, convert
// =================================================================

/// Parses one id or slug parameter: a malformed value is §6.1's
/// `invalid` (U9's mapping of the domain's `ValueError`) before the
/// store is touched. The query cluster's own copy is private to its
/// file (a sha-pinned gate line there), so this is the listing
/// cluster's — a per-cluster copy, not a shared-surface change.
fn parse_value<T>(value: &str) -> Result<T, ToolError>
where
    T: FromStr<Err = ValueError>,
{
    T::from_str(value).map_err(ToolError::from)
}

/// `task_list`'s answer: parse, resolve, query, convert.
fn answer_task_list(
    state: &mut ServerState,
    params: &TaskListParams,
) -> Result<TaskListDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let epic = params
        .epic
        .as_deref()
        .map(parse_value::<EpicId>)
        .transpose()?;
    let filter = TaskFilter {
        epic,
        status: params.status.map(Status::from),
        kind: params.kind.map(TaskKind::from),
        ready: params.ready,
    };
    let graph = state.resolve_read(&slug)?;
    let rows = graph.graph().task_list(&filter)?;
    Ok(TaskListDto {
        tasks: rows.into_iter().map(TaskRowDto::from).collect(),
    })
}

/// `epic_list`'s answer: parse, resolve, list, convert.
fn answer_epic_list(
    state: &mut ServerState,
    params: &EpicListParams,
) -> Result<EpicListDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let graph = state.resolve_read(&slug)?;
    let graph = graph.graph();
    // The None arm is unreachable: every id came off the graph's own
    // ascending accessor, which yields live nodes only.
    let epics = graph
        .epic_list()
        .into_iter()
        .filter_map(|id| {
            graph.epic(&id).map(|node| EpicRowDto {
                id: id.to_string(),
                title: node.title.clone(),
            })
        })
        .collect();
    Ok(EpicListDto { epics })
}

/// `decision_list`'s answer: parse, resolve, list, convert.
fn answer_decision_list(
    state: &mut ServerState,
    params: &DecisionListParams,
) -> Result<DecisionListDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let graph = state.resolve_read(&slug)?;
    let graph = graph.graph();
    // The None arm is unreachable: every id came off the graph's own
    // ascending accessor, which yields live nodes only.
    let decisions = graph
        .decision_list()
        .into_iter()
        .filter_map(|id| {
            graph.decision(&id).map(|node| DecisionRowDto {
                id: id.to_string(),
                title: node.title.clone(),
            })
        })
        .collect();
    Ok(DecisionListDto { decisions })
}

/// `doc_list`'s answer: parse, resolve, list, convert.
fn answer_doc_list(
    state: &mut ServerState,
    params: &DocListParams,
) -> Result<DocListDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let graph = state.resolve_read(&slug)?;
    let graph = graph.graph();
    // The None arm is unreachable: every id came off the graph's own
    // ascending accessor, which yields live nodes only.
    let docs = graph
        .doc_list()
        .into_iter()
        .filter_map(|id| {
            graph.doc(&id).map(|node| DocRowDto {
                id: id.to_string(),
                title: node.title.clone(),
            })
        })
        .collect();
    Ok(DocListDto { docs })
}

/// `graph_show`'s answer: parse both parameters (repo then epic, both
/// before any resolve — malformed values are `invalid` first), then
/// the three scopes of §6.2 — the epic scope (an explicit repo is
/// authoritative; alone, the repo is derived from the epic id), the
/// repo scope, and the all-repos scope.
fn answer_graph_show(
    state: &mut ServerState,
    params: &GraphShowParams,
) -> Result<GraphShowDto, ToolError> {
    let repo = params
        .repo
        .as_deref()
        .map(parse_value::<RepoSlug>)
        .transpose()?;
    let epic = params
        .epic
        .as_deref()
        .map(parse_value::<EpicId>)
        .transpose()?;
    if let Some(epic) = epic.as_ref() {
        let slug = match repo.as_ref() {
            Some(slug) => slug.clone(),
            None => epic.repo().clone(),
        };
        let graph = state.resolve_read(&slug)?;
        let dump = epic_scope_dump(graph.graph(), epic)?;
        Ok(GraphShowDto { repos: vec![dump] })
    } else if let Some(slug) = repo.as_ref() {
        let graph = state.resolve_read(slug)?;
        Ok(GraphShowDto {
            repos: vec![repo_scope_dump(graph.graph())?],
        })
    } else {
        // The all-repos scope: every registered repo, sorted by slug —
        // the registry's set (§6.2), read one by one lock-free on a
        // handles miss (D19), never a cold open and never an arming.
        let registry = Registry::load(&state.root)?;
        let mut repos = Vec::new();
        for slug in registry.slugs() {
            let graph = state.resolve_read(slug)?;
            repos.push(repo_scope_dump(graph.graph())?);
        }
        Ok(GraphShowDto { repos })
    }
}

/// The repo-scoped dump (§6.2): the repo node, every epic group
/// (ascending epic id, each group's tasks in that epic's induced
/// canonical order), every decision and every doc (ascending).
fn repo_scope_dump(graph: &Graph) -> Result<RepoDumpDto, ToolError> {
    let epics = graph
        .epics()
        .map(|node| epic_group(graph, node))
        .collect::<Result<Vec<_>, _>>()?;
    let decisions = graph
        .decisions()
        .map(|node| decision_node(graph, node))
        .collect();
    let docs = graph.docs().map(DocDto::from).collect();
    Ok(RepoDumpDto {
        repo: RepoDto::from(graph.repo()),
        epics,
        decisions,
        docs,
    })
}

/// The epic-scoped dump (§6.2): the repo node, the one epic group,
/// the gate set as decision nodes, and no docs — a doc has no epic
/// field.
fn epic_scope_dump(graph: &Graph, epic: &EpicId) -> Result<RepoDumpDto, ToolError> {
    // The gate set first: its derivation rejects an epic this graph
    // does not hold with the domain's own `unknown_id` — the epic
    // scope's membership check.
    let gates = graph.gating_decisions_of_epic(epic)?;
    // Unreachable in practice — the derivation above rejected an
    // unheld epic — but the error is the domain's own answer for the
    // same state, not a forged one.
    let node = graph.epic(epic).ok_or_else(|| {
        ToolError::from(DomainError::UnknownId {
            id: epic.to_string(),
        })
    })?;
    let group = epic_group(graph, node)?;
    // Every gate id came off an adjacency list whose edges require
    // live endpoints, so the lookup cannot miss; a miss drops the node
    // rather than inventing one (the domain's own discipline).
    let decisions = gates
        .iter()
        .filter_map(|id| graph.decision(id))
        .map(|node| decision_node(graph, node))
        .collect();
    Ok(RepoDumpDto {
        repo: RepoDto::from(graph.repo()),
        epics: vec![group],
        decisions,
        docs: Vec::new(),
    })
}

/// One epic group of the dump (§6.2): the epic node, its own gating
/// decisions, and its tasks in the epic's induced canonical order.
/// The order derivation cannot fail for an epic the graph holds —
/// both callers hold it — but the propagated error is the domain's
/// own answer, not a forged one.
fn epic_group(graph: &Graph, node: &Epic) -> Result<EpicGroupDto, ToolError> {
    let order = graph.canonical_order_of_epic(&node.id)?;
    Ok(EpicGroupDto {
        epic: EpicDto::from(node),
        blocked_by: end_ids(graph.blocked_by(&BlockEnd::Epic(node.id.clone()))),
        // The order emits only ids of tasks this graph holds (the
        // domain's own state-space argument), so the lookup cannot
        // miss; a miss drops the task rather than inventing a node.
        tasks: order
            .iter()
            .filter_map(|id| graph.task(id))
            .map(|task| task_node(graph, task))
            .collect(),
    })
}

/// One task node of the dump: the task's every field plus its
/// adjacency — `blocked_by` from the domain's two-run rule with the
/// statuses dropped, `blocks` and `relates` from the graph's own
/// adjacency surfaces.
fn task_node(graph: &Graph, task: &Task) -> TaskNodeDto {
    TaskNodeDto {
        task: TaskDto::from(task),
        blocked_by: graph
            .blockers_of(&task.id)
            .into_iter()
            .map(|blocker| BlockEnd::from(blocker).to_string())
            .collect(),
        blocks: task_successor_ids(graph.blocks(&BlockEnd::Task(task.id.clone()))),
        relates: graph
            .relates_of(&task.id)
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

/// One decision node of the dump: the dossier plus the tasks and
/// epics it gates.
fn decision_node(graph: &Graph, node: &Decision) -> DecisionNodeDto {
    DecisionNodeDto {
        decision: DecisionDto::from(node),
        blocks: end_ids(graph.blocks(&BlockEnd::Decision(node.id.clone()))),
    }
}

/// `BlockEnd`s as id strings — the epic's `blocked_by` (only
/// decisions target an epic) and the decision's `blocks` (tasks and
/// epics), where every arm that can occur maps.
fn end_ids(ends: Vec<BlockEnd>) -> Vec<String> {
    ends.into_iter().map(|end| end.to_string()).collect()
}

/// A task's `blocks` successors as id strings — a task's out-edges
/// are `TaskToTask` only (§5.1), so the `Epic`/`Decision` arms are
/// unreachable and skip rather than map (query.rs's `blocker_id`
/// discipline).
fn task_successor_ids(ends: Vec<BlockEnd>) -> Vec<String> {
    ends.into_iter()
        .filter_map(|end| match end {
            BlockEnd::Task(id) => Some(id.to_string()),
            BlockEnd::Decision(_) | BlockEnd::Epic(_) => None,
        })
        .collect()
}

// =================================================================
// The tools
// =================================================================

#[tool_router(router = listing_router, vis = "pub(crate)")]
impl TodoServer {
    /// The task listing (§6): the scope's tasks in canonical order under the ANDed filters — epic, status, kind, deferred readiness — each row carrying its blocked-by annotation with every blocker's current status.
    #[tool(
        name = "task_list",
        output_schema = crate::schema::output_for_type::<TaskListDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn task_list(
        &self,
        Parameters(params): Parameters<TaskListParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it.
        let mut state = self.lock_state();
        match answer_task_list(&mut state, &params) {
            Ok(output) => structured("task_list", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// The repo's epics as id+title rows, ascending by id — the listing an agent shows the user when a query needs an epic chosen.
    #[tool(
        name = "epic_list",
        output_schema = crate::schema::output_for_type::<EpicListDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn epic_list(
        &self,
        Parameters(params): Parameters<EpicListParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_epic_list(&mut state, &params) {
            Ok(output) => structured("epic_list", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// The repo's decisions as id+title rows, ascending by id.
    #[tool(
        name = "decision_list",
        output_schema = crate::schema::output_for_type::<DecisionListDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn decision_list(
        &self,
        Parameters(params): Parameters<DecisionListParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_decision_list(&mut state, &params) {
            Ok(output) => structured("decision_list", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// The repo's docs as id+title rows, ascending by id.
    #[tool(
        name = "doc_list",
        output_schema = crate::schema::output_for_type::<DocListDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn doc_list(
        &self,
        Parameters(params): Parameters<DocListParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_list(&mut state, &params) {
            Ok(output) => structured("doc_list", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// The handover dump (§6.2): every registered repo, one repo, or one epic's subtree plus its gating decisions — nodes grouped by epic in canonical order, each carrying its blocked-by/blocks adjacency.
    #[tool(
        name = "graph_show",
        output_schema = crate::schema::output_for_type::<GraphShowDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn graph_show(
        &self,
        Parameters(params): Parameters<GraphShowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_graph_show(&mut state, &params) {
            Ok(output) => structured("graph_show", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
