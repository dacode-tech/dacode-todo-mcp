//! The edge cluster's two tools (§6, t37): `task_link`/`task_unlink`
//! — the `blocks` and `relates` edges over the wire. The endpoints
//! arrive as untyped id strings and parse into [`BlockEnd`] through
//! the domain's own parsers tried in kind order, so no id grammar is
//! duplicated here; a pair §5.1 forbids is rejected at the wire as
//! `invalid` naming both endpoints — f4's recorded criterion, the
//! domain having made the forbidden shapes unrepresentable
//! ([`BlocksEdge::from_ends`] answers `None`), and §6.1's closed list
//! carrying no code a new check could use.
//!
//! # The shapes
//!
//! One discriminator, `edge`: `blocks` or `relates`, a typed enum — a
//! wrong spelling is rmcp's codeless argument fault (the t35/t36
//! vocabulary boundary), never `invalid`. For `blocks`, `from`/`to`
//! are the source and the target; for `relates`, the two tasks in
//! either order — the annotation is undirected and the domain
//! canonicalizes the pair. The response is the echo ([`EdgeDto`]):
//! the edge acted on, both endpoints. The store returns `()` for
//! edge mutations (§4.2), so there is no node state to return; the
//! echo gives the idempotent no-ops their honest shape — the
//! alternative, an empty object, would pin nothing about the edge.
//! (The endpoints are echoed as parsed — the domain's id
//! `Display`/`FromStr` round-trips every accepted string, so there
//! is no separate canonical spelling to pin.)
//!
//! # The semantics are the domain's, mirrored exactly
//!
//! A duplicate `blocks` edge is a no-op `Ok`; a cycle closes into
//! §6.1's `cycle` with the domain's deterministic `[s, t, …, s]` path
//! in the message; `unlink` of a missing edge is a no-op `Ok` (both
//! kinds); unknown endpoints answer `unknown_id`, the source checked
//! before the target; a `relates` repeat is a no-op and a
//! self-annotation is accepted and inert. The wire restates none of
//! it — every answer below is the store's, through the carrier.
//!
//! # The clock
//!
//! One [`time::now`] read per tool call, taken after the parse and
//! the resolve — the audit instant of the event line. It never
//! reaches the response (the echo carries no timestamp), so both
//! tools are clock-free and golden-pinnable — the t36 principle: the
//! property that decides is clock-freeness of the response, not
//! readness.
//!
//! # The failure arm
//!
//! Every tool's failure arm is `Err(e) => Ok(error_carrier(&e.code,
//! &e.message))` over [`crate::ToolError`] — the one error-mapping
//! layer (lib.rs). The two tools carry no `read_only_hint` (they are
//! writes — the annotation is absent, not `false`-asserted) and
//! `open_world_hint = false` (they touch only the local store), and
//! each declares its `output_schema` (the committed D15). The state
//! lock is taken and dropped inside the sync body (§8, the lib module
//! doc's rule — no `.await` after acquisition).

use std::str::FromStr;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use todo_domain::{BlockEnd, BlocksEdge, DecisionId, EpicId, TaskId, ValueError};

use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

// =================================================================
// The DTOs
// =================================================================

/// The edge kind discriminator (§6's row names): `blocks` or `relates`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EdgeKindDto {
    /// A `blocks` edge: directed, the source holds the target back until it finishes (§5.1).
    Blocks,
    /// A `relates` annotation: undirected, task-to-task, never an ordering input (§5.1).
    Relates,
}

/// `task_link`'s parameters (§6): the edge kind, the two endpoints as untyped id strings, and the repo.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskLinkParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the edge; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The edge kind: `blocks` (directed, the source holds the target back) or `relates` (undirected annotation).
    pub edge: EdgeKindDto,
    /// The edge's first endpoint as an id string, e.g. `camera:U7` — for `blocks` the source, for `relates` either task of the pair.
    pub from: String,
    /// The edge's second endpoint as an id string — for `blocks` the target, for `relates` the other task of the pair.
    pub to: String,
}

/// `task_unlink`'s parameters (§6): the edge kind, the two endpoints as untyped id strings, and the repo.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskUnlinkParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph loses the edge; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The edge kind: `blocks` (directed, the source holds the target back) or `relates` (undirected annotation).
    pub edge: EdgeKindDto,
    /// The edge's first endpoint as an id string, e.g. `camera:U7` — for `blocks` the source, for `relates` either task of the pair.
    pub from: String,
    /// The edge's second endpoint as an id string — for `blocks` the target, for `relates` the other task of the pair.
    pub to: String,
}

/// The echo response of both edge tools: the edge acted on, both endpoints in the domain's canonical id spelling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct EdgeDto {
    /// The edge kind acted on.
    pub edge: EdgeKindDto,
    /// The first endpoint's canonical id spelling, e.g. `camera:U7`.
    pub from: String,
    /// The second endpoint's canonical id spelling, e.g. `camera:U9`.
    pub to: String,
}

// =================================================================
// The endpoint parse and the pair conversions
// =================================================================

/// Parses one edge endpoint: an untyped id string into a [`BlockEnd`]
/// through the domain's own parsers, tried in kind order — task, then
/// decision, then epic. The dispatch is unambiguous by construction:
/// each id type's `FromStr` validates its own per-kind prefix, so at
/// most one parser accepts any string (`fixture:D1` fails the task
/// parser and succeeds the decision one; a doc id fails all three).
/// All three failing answers the domain's uniform `invalid node id`
/// — no id grammar is duplicated in the wire, which only tries the
/// domain's public parsers. The alternative, a domain rider
/// exposing a generic node-id parse, was declined: a new public
/// type with one consumer is more machinery than this dispatch.
fn block_end(value: &str) -> Result<BlockEnd, ToolError> {
    if let Ok(id) = TaskId::from_str(value) {
        return Ok(BlockEnd::Task(id));
    }
    if let Ok(id) = DecisionId::from_str(value) {
        return Ok(BlockEnd::Decision(id));
    }
    if let Ok(id) = EpicId::from_str(value) {
        return Ok(BlockEnd::Epic(id));
    }
    Err(ToolError::from(ValueError::InvalidId {
        input: value.to_owned(),
    }))
}

/// The `blocks` edge between two parsed endpoints, or `invalid` naming
/// both when §5.1 forbids the pair — f4's criterion at the wire: the
/// domain made the forbidden shapes unrepresentable
/// ([`BlocksEdge::from_ends`] answers `None` for an epic source, a
/// decision target, a task blocking an epic), and §6.1's closed list
/// has no code for a forbidden topology, so this crate's own
/// rejection class (`invalid`, the t34 mutually-exclusive-scopes
/// precedent) is the only fit.
fn blocks_edge(from: &BlockEnd, to: &BlockEnd) -> Result<BlocksEdge, ToolError> {
    BlocksEdge::from_ends(from.clone(), to.clone()).ok_or_else(|| {
        ToolError::invalid(&format!(
            "the blocks pair {from} -> {to} is forbidden: a blocks edge runs task to \
             task, decision to task, or decision to epic"
        ))
    })
}

/// The two tasks of a `relates` annotation, or `invalid` naming both
/// when an endpoint is not a task — the annotation is task-to-task
/// (§5.1) and the domain's mutation takes two [`TaskId`]s, so a
/// non-task endpoint is this crate's own rejection, the same class
/// and shape as the blocks forbidden pair.
fn relates_tasks(from: &BlockEnd, to: &BlockEnd) -> Result<(TaskId, TaskId), ToolError> {
    match (from, to) {
        (BlockEnd::Task(a), BlockEnd::Task(b)) => Ok((a.clone(), b.clone())),
        _ => Err(ToolError::invalid(&format!(
            "the relates pair {from} <-> {to} is forbidden: relates is task-to-task"
        ))),
    }
}

// =================================================================
// The answer helpers — parse, resolve, convert, stamp, mutate
// =================================================================

/// `task_link`'s answer: parse, resolve, convert, stamp, link.
fn answer_task_link(
    state: &mut ServerState,
    params: &TaskLinkParams,
) -> Result<EdgeDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let from = block_end(&params.from)?;
    let to = block_end(&params.to)?;
    let handle = state.resolve(&slug)?;
    match params.edge {
        EdgeKindDto::Blocks => {
            let edge = blocks_edge(&from, &to)?;
            handle.link_blocks(edge, time::now())?;
        }
        EdgeKindDto::Relates => {
            let (a, b) = relates_tasks(&from, &to)?;
            handle.link_relates(a, b, time::now())?;
        }
    }
    Ok(EdgeDto {
        edge: params.edge,
        from: from.to_string(),
        to: to.to_string(),
    })
}

/// `task_unlink`'s answer: parse, resolve, convert, stamp, unlink.
fn answer_task_unlink(
    state: &mut ServerState,
    params: &TaskUnlinkParams,
) -> Result<EdgeDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let from = block_end(&params.from)?;
    let to = block_end(&params.to)?;
    let handle = state.resolve(&slug)?;
    match params.edge {
        EdgeKindDto::Blocks => {
            let edge = blocks_edge(&from, &to)?;
            handle.unlink_blocks(edge, time::now())?;
        }
        EdgeKindDto::Relates => {
            let (a, b) = relates_tasks(&from, &to)?;
            handle.unlink_relates(a, b, time::now())?;
        }
    }
    Ok(EdgeDto {
        edge: params.edge,
        from: from.to_string(),
        to: to.to_string(),
    })
}

// =================================================================
// The tools
// =================================================================

#[tool_router(router = edges_router, vis = "pub(crate)")]
impl TodoServer {
    /// Link two nodes: a `blocks` edge (the source holds the target back until it finishes) or a `relates` annotation (undirected, task-to-task); a duplicate is a no-op, and the response is the echo naming the edge acted on.
    #[tool(
        name = "task_link",
        output_schema = rmcp::handler::server::tool::schema_for_type::<EdgeDto>(),
        annotations(open_world_hint = false)
    )]
    async fn task_link(
        &self,
        Parameters(params): Parameters<TaskLinkParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it.
        let mut state = self.lock_state();
        match answer_task_link(&mut state, &params) {
            Ok(output) => structured("task_link", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Unlink two nodes: remove a `blocks` edge or a `relates` annotation; a missing edge is a no-op, and the response is the echo naming the edge acted on.
    #[tool(
        name = "task_unlink",
        output_schema = rmcp::handler::server::tool::schema_for_type::<EdgeDto>(),
        annotations(open_world_hint = false)
    )]
    async fn task_unlink(
        &self,
        Parameters(params): Parameters<TaskUnlinkParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_task_unlink(&mut state, &params) {
            Ok(output) => structured("task_unlink", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
