//! The query cluster's five read tools (§6, t34): `next_task`,
//! `next_items`, `status_summary`, `task_show`, `decision_show` —
//! thin translations of the domain's §5.2/§6 query surfaces onto the
//! wire, each with one params struct denying unknown fields (§6.1's
//! consequence: an unrecognised argument is otherwise silently
//! dropped), a declared outputSchema (D15) and read-only annotations
//! (§6.3: §6's read/write split is expressible, and these five are
//! reads — no clock anywhere, every response byte-deterministic
//! against a fixed fixture).
//!
//! # The shapes
//!
//! The output DTOs are server-local mirrors of the domain's answer
//! types, named fields in the domain's declaration order (D14 — the
//! wire matches the domain's canonical spelling), with ids and
//! timestamps as `String`s in their canonical spellings and the
//! vocabulary enums mirrored variant-for-variant (`dto.rs`, shared
//! with t35/t36). §5.2's total orders reach the wire intact: arrays
//! preserve order (§6.3), and the `blocked_by` conversion walks the
//! domain's `Vec<BlockEnd>` in order — the two-run contract the field
//! doc states (t28's ruling: a schema describing that field must say
//! so).
//!
//! The params structs take `repo` as an optional `String` — the
//! committed D8's default-slug case: absent means the configured
//! default repo, an explicit value always wins ([`crate::repo_of`];
//! relaxed from required, backward-compatible for callers) — and the
//! id parameters as plain
//! `String`s parsed with `FromStr`: a malformed value is §6.1's
//! `invalid` through the carrier before the store is touched, never a
//! codeless rmcp argument fault (the registry.rs precedent — a typed
//! field would demote §6.1's own example into one). `limit` defaults
//! to 10 here — §5.2's default is the wire's, and the domain always
//! sees an explicit limit. `epic` and `task` both given on
//! `next_items` is `invalid` ("mutually exclusive scopes"): the
//! alternative — silent precedence — is the markdown-board failure
//! mode this project exists to remove.
//!
//! # The failure arm
//!
//! Every tool's failure arm is `Err(e) => Ok(error_carrier(&e.code,
//! &e.message))` over [`crate::ToolError`] — the one error-mapping
//! layer (lib.rs), converting store, domain and value faults. The repo
//! dimension resolves through `ServerState::resolve_read` under the
//! state lock — a handles hit borrows the held graph, a miss reads
//! lock-free (D19); the success arm serializes through the shared
//! `structured` helper, whose serializer fault bypasses the carrier
//! per the registry.rs precedent.

use std::str::FromStr;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use todo_domain::{
    BlockEnd, DecisionCounts, DecisionId, DomainError, EpicId, NextItem, NextItems, NextTask,
    OpenTotals, StatusCounts, StatusSummary, TaskId, ValueError,
};

use crate::dto::{DecisionDto, DeferredDto, PriorityDto, TaskDto};
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured};

/// `next_items`'s batch cap when the parameter is absent (§5.2: the
/// default is the wire's; the domain always sees an explicit limit).
const DEFAULT_LIMIT: u64 = 10;

// =================================================================
// The params structs
// =================================================================

/// `next_task`'s parameters (§6, §5.2): the repository to query and an
/// optional epic filter.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct NextTaskParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The optional epic filter (§6): the scope becomes that epic's tasks, e.g. `camera:E1`.
    pub epic: Option<String>,
}

/// `next_items`'s parameters (§5.2): the repository, an optional epic
/// or task scope (mutually exclusive), and the batch limit.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct NextItemsParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The optional epic filter (§6): the scope becomes that epic's actionable tasks; mutually exclusive with `task`.
    pub epic: Option<String>,
    /// The optional task scope (§5.2): the batch becomes that task's transitive dependents, each marked actionable or held; mutually exclusive with `epic`.
    pub task: Option<String>,
    /// How many rows the batch holds at most (§5.2); absent means 10.
    #[schemars(schema_with = "crate::schema::optional_u64")]
    pub limit: Option<u64>,
}

/// `status_summary`'s parameters (§6, D16): the repository to roll up
/// and an optional epic filter.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct StatusSummaryParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The optional epic filter (§6): the scope becomes that epic, its tasks and the decisions gating them.
    pub epic: Option<String>,
}

/// `task_show`'s parameters (§6): the repository and the task to show.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskShowParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The task's id, e.g. `camera:U7`.
    pub task: String,
}

/// `decision_show`'s parameters (§6, D10): the repository and the
/// decision to show.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionShowParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph is queried; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The decision's id, e.g. `camera:D3`.
    pub decision: String,
}

// =================================================================
// The output DTOs
// =================================================================

/// `next_task`'s answer (§5.2): the scope's first actionable task, its priority and the scope's deferred summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct NextTaskDto {
    /// The first actionable task in the queried scope's canonical order, or null when the scope holds none — an answer, not an error.
    pub task: Option<String>,
    /// The chosen task's agent-set priority (D17), echoed beside its id — high, medium, low or null; display-only, never an ordering input.
    pub priority: Option<PriorityDto>,
    /// The deferred summary of the queried scope: how many deferred tasks it holds and how many are worth revisiting now.
    pub deferred: DeferredDto,
}

impl From<NextTask> for NextTaskDto {
    fn from(answer: NextTask) -> Self {
        Self {
            task: answer.task.map(|id| id.to_string()),
            priority: answer.priority.map(PriorityDto::from),
            deferred: DeferredDto::from(answer.deferred),
        }
    }
}

/// `next_items`'s answer (§5.2): the scope's batch and the scope's deferred summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct NextItemsDto {
    /// The batch: up to `limit` rows in the scope's canonical order — the actionable ones in repo and epic scope, every transitive dependent in task scope; never reordered by an annotation.
    pub items: Vec<NextItemDto>,
    /// The deferred summary of the queried scope, present unconditionally — including on an empty batch.
    pub deferred: DeferredDto,
}

impl From<NextItems> for NextItemsDto {
    fn from(answer: NextItems) -> Self {
        Self {
            items: answer.items.into_iter().map(NextItemDto::from).collect(),
            deferred: DeferredDto::from(answer.deferred),
        }
    }
}

/// One `next_items` row (§5.2): a task in the scope's derived order, its adjacency annotations, its derived weight, its readiness mark and its priority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct NextItemDto {
    /// The task this row is about, e.g. `camera:U7`.
    pub task: String,
    /// Every direct predecessor holding this task back — two ascending runs: the task's own edges first, then its epic's decision gates — deduplicated at the first occurrence, never one globally sorted list.
    pub blocked_by: Vec<String>,
    /// The task's direct `blocks` successors, ascending.
    pub blocks: Vec<String>,
    /// The count of this task's transitive `blocks` dependents, excluding itself; display-only, never an ordering input.
    pub weight: u64,
    /// Whether the task may be picked up now: status open or in_work with every predecessor finished.
    pub actionable: bool,
    /// The task's agent-set priority (D17) — high, medium, low or null; display-only, never an ordering input.
    pub priority: Option<PriorityDto>,
}

impl From<NextItem> for NextItemDto {
    fn from(row: NextItem) -> Self {
        Self {
            task: row.task.to_string(),
            blocked_by: row.blocked_by.iter().filter_map(blocker_id).collect(),
            blocks: row.blocks.iter().map(ToString::to_string).collect(),
            weight: row.weight,
            actionable: row.actionable,
            priority: row.priority.map(PriorityDto::from),
        }
    }
}

/// One blocker's canonical id spelling. The `Epic` arm is unreachable
/// — §5.1's edge shapes make a `blocks` source a task or a decision,
/// and the domain's row conversion already dropped epic ends — so it
/// skips rather than panics.
fn blocker_id(end: &BlockEnd) -> Option<String> {
    match end {
        BlockEnd::Task(id) => Some(id.to_string()),
        BlockEnd::Decision(id) => Some(id.to_string()),
        BlockEnd::Epic(_) => None,
    }
}

/// `status_summary`'s answer (§6, D16): the scope's counts by kind and status, the non-terminal open totals and the deferred summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct StatusSummaryDto {
    /// Every epic of the scope, by status.
    pub epics: StatusCountsDto,
    /// Every task of the scope, by status.
    pub tasks: StatusCountsDto,
    /// Every decision of the scope, by status.
    pub decisions: DecisionCountsDto,
    /// Docs of the scope, one total — a doc carries no status.
    pub docs: u64,
    /// The non-terminal totals per kind.
    pub open: OpenTotalsDto,
    /// The deferred summary over the scope's task list.
    pub deferred: DeferredDto,
}

impl From<StatusSummary> for StatusSummaryDto {
    fn from(answer: StatusSummary) -> Self {
        Self {
            epics: StatusCountsDto::from(answer.epics),
            tasks: StatusCountsDto::from(answer.tasks),
            decisions: DecisionCountsDto::from(answer.decisions),
            docs: answer.docs,
            open: OpenTotalsDto::from(answer.open),
            deferred: DeferredDto::from(answer.deferred),
        }
    }
}

/// The counts of one node kind by status (D16) — the epic and the task halves of a summary; every status is carried, zeroes included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct StatusCountsDto {
    /// The count at status open.
    pub open: u64,
    /// The count at status in_work.
    pub in_work: u64,
    /// The count at status blocked.
    pub blocked: u64,
    /// The count at status deferred.
    pub deferred: u64,
    /// The count at status closed.
    pub closed: u64,
    /// The count at status wontfix.
    pub wontfix: u64,
}

impl From<StatusCounts> for StatusCountsDto {
    fn from(counts: StatusCounts) -> Self {
        Self {
            open: counts.open,
            in_work: counts.in_work,
            blocked: counts.blocked,
            deferred: counts.deferred,
            closed: counts.closed,
            wontfix: counts.wontfix,
        }
    }
}

/// The counts of decisions by decision status (D16) — the decision half of a summary; every status is carried, zeroes included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DecisionCountsDto {
    /// The count at decision status open.
    pub open: u64,
    /// The count at decision status signed.
    pub signed: u64,
    /// The count at decision status superseded.
    pub superseded: u64,
}

impl From<DecisionCounts> for DecisionCountsDto {
    fn from(counts: DecisionCounts) -> Self {
        Self {
            open: counts.open,
            signed: counts.signed,
            superseded: counts.superseded,
        }
    }
}

/// The non-terminal totals per kind (§6: "Open" in any total means non-terminal), carried once at the summary level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct OpenTotalsDto {
    /// Non-terminal epics: open + in_work + blocked + deferred.
    pub epics: u64,
    /// Non-terminal tasks, the same four statuses.
    pub tasks: u64,
    /// Open decisions — the kind's only non-terminal status.
    pub decisions: u64,
}

impl From<OpenTotals> for OpenTotalsDto {
    fn from(totals: OpenTotals) -> Self {
        Self {
            epics: totals.epics,
            tasks: totals.tasks,
            decisions: totals.decisions,
        }
    }
}

// =================================================================
// The answer helpers — parse, resolve, query, convert
// =================================================================

/// Parses one id or slug parameter: a malformed value is §6.1's
/// `invalid` (U9's mapping of the domain's `ValueError`) before the
/// store is touched.
fn parse_value<T>(value: &str) -> Result<T, ToolError>
where
    T: FromStr<Err = ValueError>,
{
    T::from_str(value).map_err(ToolError::from)
}

/// `next_task`'s answer: parse, resolve, query, convert.
fn answer_next_task(
    state: &mut ServerState,
    params: &NextTaskParams,
) -> Result<NextTaskDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let epic = params
        .epic
        .as_deref()
        .map(parse_value::<EpicId>)
        .transpose()?;
    let graph = state.resolve_read(&slug)?;
    let answer = match epic.as_ref() {
        Some(epic) => graph.graph().next_task_of_epic(epic)?,
        None => graph.graph().next_task(),
    };
    Ok(NextTaskDto::from(answer))
}

/// `next_items`'s answer: parse, reject the contradictory scopes,
/// resolve, query, convert.
fn answer_next_items(
    state: &mut ServerState,
    params: &NextItemsParams,
) -> Result<NextItemsDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let epic = params
        .epic
        .as_deref()
        .map(parse_value::<EpicId>)
        .transpose()?;
    let task = params
        .task
        .as_deref()
        .map(parse_value::<TaskId>)
        .transpose()?;
    if epic.is_some() && task.is_some() {
        return Err(ToolError::invalid(
            "epic and task are mutually exclusive scopes",
        ));
    }
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    let graph = state.resolve_read(&slug)?;
    let answer = if let Some(epic) = epic.as_ref() {
        graph.graph().next_items_of_epic(epic, limit)?
    } else if let Some(task) = task.as_ref() {
        graph.graph().next_items_of_task(task, limit)?
    } else {
        graph.graph().next_items(limit)
    };
    Ok(NextItemsDto::from(answer))
}

/// `status_summary`'s answer: parse, resolve, query, convert.
fn answer_status_summary(
    state: &mut ServerState,
    params: &StatusSummaryParams,
) -> Result<StatusSummaryDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let epic = params
        .epic
        .as_deref()
        .map(parse_value::<EpicId>)
        .transpose()?;
    let graph = state.resolve_read(&slug)?;
    let answer = match epic.as_ref() {
        Some(epic) => graph.graph().status_summary_of_epic(epic)?,
        None => graph.graph().status_summary(),
    };
    Ok(StatusSummaryDto::from(answer))
}

/// `task_show`'s answer: parse, resolve, look up, convert.
fn answer_task_show(
    state: &mut ServerState,
    params: &TaskShowParams,
) -> Result<TaskDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let task = parse_value::<TaskId>(&params.task)?;
    let graph = state.resolve_read(&slug)?;
    let node = graph.graph().task(&task).ok_or_else(|| {
        ToolError::from(DomainError::UnknownId {
            id: task.to_string(),
        })
    })?;
    Ok(TaskDto::from(node))
}

/// `decision_show`'s answer: parse, resolve, look up, convert.
fn answer_decision_show(
    state: &mut ServerState,
    params: &DecisionShowParams,
) -> Result<DecisionDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let decision = parse_value::<DecisionId>(&params.decision)?;
    let graph = state.resolve_read(&slug)?;
    let node = graph.graph().decision(&decision).ok_or_else(|| {
        ToolError::from(DomainError::UnknownId {
            id: decision.to_string(),
        })
    })?;
    Ok(DecisionDto::from(node))
}

// =================================================================
// The tools
// =================================================================

#[tool_router(router = query_router, vis = "pub(crate)")]
impl TodoServer {
    /// The question the system exists for: the first actionable task of a repository (optionally one epic), in canonical order, with the scope's deferred summary.
    #[tool(
        name = "next_task",
        output_schema = crate::schema::output_for_type::<NextTaskDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn next_task(
        &self,
        Parameters(params): Parameters<NextTaskParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it.
        let mut state = self.lock_state();
        match answer_next_task(&mut state, &params) {
            Ok(output) => structured("next_task", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// The ranked actionable batch: up to `limit` (default 10) tasks of a repository, one epic, or one task's transitive dependents, each annotated with what blocks it, its direct successors, a derived weight and the actionable mark, plus the deferred summary.
    #[tool(
        name = "next_items",
        output_schema = crate::schema::output_for_type::<NextItemsDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn next_items(
        &self,
        Parameters(params): Parameters<NextItemsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_next_items(&mut state, &params) {
            Ok(output) => structured("next_items", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// The roll-up of a repository (optionally one epic): counts of epics and tasks by status, decisions by decision status, docs as one total, the non-terminal open totals and the deferred summary.
    #[tool(
        name = "status_summary",
        output_schema = crate::schema::output_for_type::<StatusSummaryDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn status_summary(
        &self,
        Parameters(params): Parameters<StatusSummaryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_status_summary(&mut state, &params) {
            Ok(output) => structured("status_summary", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// One task's full record: every field of the node, ids and timestamps in their canonical spellings.
    #[tool(
        name = "task_show",
        output_schema = crate::schema::output_for_type::<TaskDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn task_show(
        &self,
        Parameters(params): Parameters<TaskShowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_task_show(&mut state, &params) {
            Ok(output) => structured("task_show", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// One decision dossier's full record: every field of the node, ids and timestamps in their canonical spellings.
    #[tool(
        name = "decision_show",
        output_schema = crate::schema::output_for_type::<DecisionDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn decision_show(
        &self,
        Parameters(params): Parameters<DecisionShowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_decision_show(&mut state, &params) {
            Ok(output) => structured("decision_show", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
