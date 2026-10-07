//! The mutation cluster's nine node-CRUD tools (§6, t36): `epic_add`/
//! `epic_update`, `task_add`/`task_update`, `decision_add`/
//! `decision_update`, `doc_register`/`doc_set_path`/`doc_update` —
//! thin translations of the `Store` trait's mutations onto the wire.
//! Every mutation already exists below this crate (§2: the wire
//! converts, it does not decide), so this file holds no domain logic:
//! each tool parses its parameters, resolves the repo, converts the
//! wire shapes into the store's drafts and patches, and answers with
//! the new state (§6's frame — a client never needs a
//! read-after-write round trip).
//!
//! # The clock
//!
//! The server is the store's caller, so it stamps: **one
//! [`time::now`] read per tool call**, taken after the parse and the
//! resolve and used as the mutation's `at` — an add stamps `created`
//! and `updated` with the same instant; an update or `doc_set_path`
//! stamps `updated` alone. `time.rs` is the clock's one home and no
//! injection seam is added: a fake-clock seam would be a production
//! surface for a test concern. The consequence splits the cluster by
//! a real property — clock-freeness of the response: epics carry no
//! timestamps (§3), so `epic_add`/`epic_update` answers are fully
//! deterministic and golden-pinnable, while the seven stamped writers
//! embed the instant and assert in-test around the extracted stamp
//! (the split is stated where the next writer task reads it: the
//! write suite's module doc).
//!
//! # The shapes
//!
//! The add params mirror the domain drafts field-for-field (§6):
//! enums as the `dto.rs` mirrors, ids and slugs as plain `String`s
//! parsed with `FromStr` — a malformed value is §6.1's `invalid`
//! through the carrier before the store is touched (the
//! registry.rs/query.rs/listing.rs precedent) — `until` a `String`
//! parsed to a `Date` in the body, and `references` an optional list
//! of [`ReferenceParam`] converted per entry through the domain's
//! validating constructor (a bad target is `invalid`, never a
//! codeless deserialization fault: §6.1 names "a bad … reference
//! inside a string parameter" as `invalid`'s). `status` is required
//! on every add — the draft requires it, and a defaulted status would
//! hide which closure discipline the caller is reaching for.
//!
//! The update params mirror the store's patch DTOs (`event.rs`:
//! `TaskPatch` and its three siblings) field-for-field with wire
//! types, so the semantics a client sees are the semantics the log
//! records: RFC 7386 three-state — a field absent is untouched,
//! `null` clears an optional one, a value replaces it whole. The
//! three-state fields deserialize through a server-local
//! [`double_option`] (the store's own `event.rs` precedent is private
//! there, so this is a copy, not a lift; the semantics are pinned by
//! the tests, not by shared code). `task_update`'s `epic` is a plain
//! `Option` — a reparenting cannot be "cleared" — and maps straight
//! onto `TaskPatch::epic` (§5.1: reparenting is a validated update,
//! no separate verb, in the log or on the wire). `doc_update` carries
//! no `path`: the committed D10's separately audited verb
//! `doc_set_path` owns it.
//!
//! # The closure discipline
//!
//! `verify`, `evidence`, `note` and `resolution` are optional
//! parameters whose absence is a domain error with its §6.1 code —
//! never schema-required fields. `task_add` without `verify` answers
//! `missing_verify`; reaching `closed` without `evidence` answers
//! `missing_evidence`; reaching `blocked`/`deferred` without `note`
//! answers `missing_note`; `decision_add`/`decision_update` reaching
//! `signed`/`superseded` without `resolution` answers
//! `missing_resolution`. All four arrive as `StoreError::Domain` →
//! [`crate::ToolError`] → the committed D15's carrier, inherited from
//! lib.rs — this cluster restates none of it. A schema-required
//! `verify` would demote §6.1's own example into rmcp's codeless
//! argument fault, which is exactly what §6's frame ("the board's
//! culture becomes the API's contract") forbids.
//!
//! # The vocabulary boundary
//!
//! `status`/`kind`/`confidence`/`priority`/reference `kind` are typed
//! enums in the params: a wrong spelling is rmcp's own codeless
//! argument fault, not `invalid` — §6.1's consequence text scopes
//! `invalid` to values this server *parses* (slug, id, date,
//! reference), and the input schema itself enumerates the closed
//! sets. The t35 pin covers the mechanism; the `epic_add` golden
//! carries one instance on an add path.
//!
//! # The failure arm
//!
//! Every tool's failure arm is `Err(e) => Ok(error_carrier(&e.code,
//! &e.message))` over [`crate::ToolError`] — the one error-mapping
//! layer (lib.rs). The nine tools carry no `read_only_hint` (they are
//! writes — the annotation is absent, not `false`-asserted) and
//! `open_world_hint = false` (they touch only the local store), and
//! each declares its `output_schema` (the committed D15: the node
//! DTOs, all existing in `dto.rs`). The state lock is taken and
//! dropped inside the sync body (§8, the lib module doc's rule — no
//! `.await` after acquisition).
//!
//! Over the file-size target by recorded exception (`_rust.md`, board
//! §5/U25): the cluster's dispatch pins all nine tools, their params
//! and their draft/patch conversions in one file, the `query.rs`/
//! `listing.rs` shape this cluster's spec named; the cap holds. The
//! split, if the cluster ever grows a tenth tool, is the doc family
//! (`doc_register`/`doc_set_path`/`doc_update` and their
//! conversions) into its own module — a named diff for whoever next
//! owns the file.

use std::str::FromStr;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer};
use todo_domain::{
    Confidence, Date, DecisionDraft, DecisionId, DecisionStatus, DocDraft, DocId, EpicDraft,
    EpicId, Priority, Reference, ReferenceKind, References, Status, TaskDraft, TaskId, TaskKind,
    ValueError,
};
use todo_store::{DecisionPatch, DocPatch, EpicPatch, TaskPatch};

use crate::dto::{
    ConfidenceDto, DecisionDto, DecisionStatusDto, DocDto, EpicDto, PriorityDto, ReferenceParam,
    StatusDto, TaskDto, TaskKindDto,
};
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

// =================================================================
// The three-state deserializer
// =================================================================

/// Deserializes one three-state patch field whose node field is
/// itself optional — the wire half of RFC 7386's reading this
/// cluster's update params share with the store's patches: an absent
/// field is untouched, a `null` one clears, a value replaces. JSON
/// has a single spelling for "no value", so serde's derived
/// `Option<Option<T>>` cannot tell absent from `null` — both arrive
/// as `None` — and `#[serde(default)]` plus this function split them:
/// `null` becomes `Some(None)`, a value `Some(Some(value))`. The
/// store's `event.rs` runs the identical copy privately (a
/// cross-crate lift would widen the engine's public surface for one
/// ten-line function), so the semantics are pinned here by the write
/// suite, not by shared code.
fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

// =================================================================
// The conversion helpers
// =================================================================

/// Parses one id, slug or date parameter: a malformed value is
/// §6.1's `invalid` (U9's mapping of the domain's `ValueError`)
/// before the store is touched. The query/listing clusters' own
/// copies are private to their files (sha-pinned gate lines there),
/// so this is this cluster's — a per-cluster copy, not a
/// shared-surface change.
fn parse_value<T>(value: &str) -> Result<T, ToolError>
where
    T: FromStr<Err = ValueError>,
{
    T::from_str(value).map_err(ToolError::from)
}

/// Converts one wire reference list into the domain's canonically
/// sorted set: each entry goes through the domain's validating
/// constructor, so a bad target (an anchor on a non-`doc` kind, an
/// empty target) is §6.1's `invalid` through the carrier — never a
/// codeless deserialization fault, because the target and note are
/// string parameters this server parses (§6.1's consequence text);
/// only the *kind* is vocabulary, and a wrong spelling there is
/// rmcp's argument fault before this function runs. `None` means the
/// empty set on an add and "untouched" on an update — the caller
/// decides which.
fn references_of(list: &[ReferenceParam]) -> Result<References, ToolError> {
    let mut references = References::new();
    for entry in list {
        let reference = Reference::new(
            ReferenceKind::from(entry.kind),
            entry.target.clone(),
            entry.note.clone(),
        )
        .map_err(ToolError::from)?;
        references.insert(reference).map_err(ToolError::from)?;
    }
    Ok(references)
}

/// Parses a three-state `until` parameter's inner value: the wire
/// carries `Option<Option<String>>`, the patch wants
/// `Option<Option<Date>>`, and a malformed date is `invalid` before
/// the store is touched.
fn until_of(value: &Option<Option<String>>) -> Result<Option<Option<Date>>, ToolError> {
    value
        .as_ref()
        .map(|inner| inner.as_deref().map(Date::parse).transpose())
        .transpose()
        .map_err(ToolError::from)
}

// =================================================================
// The add params — the domain drafts, wire-typed
// =================================================================

/// `epic_add`'s parameters (§6): the epic's every field except the identity the graph allocates — an epic carries no timestamps (§3).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct EpicAddParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the epic; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// Short human title.
    pub title: String,
    /// Longer prose description.
    pub description: String,
    /// Lifecycle status: open, in_work, blocked, deferred, closed or wontfix.
    pub status: StatusDto,
    /// External references (D10); absent means the empty set.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `task_add`'s parameters (§6): the task's every field except the identity and the timestamps — `created`/`updated` are the server's one clock read.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskAddParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the task; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The parent epic's id, e.g. `camera:E1` — the tree edge (§5.1); an epic the graph does not hold is `unknown_id`.
    pub epic: String,
    /// What kind of work this records: task, gate, finding or debt.
    pub kind: TaskKindDto,
    /// Short human title.
    pub title: String,
    /// Longer prose body.
    pub body: String,
    /// Lifecycle status: open, in_work, blocked, deferred, closed or wontfix — the closure discipline is checked over the result.
    pub status: StatusDto,
    /// The method that will prove it done — required at creation (`missing_verify` otherwise).
    pub verify: Option<String>,
    /// What was actually run and seen — required to reach `closed` (`missing_evidence` otherwise).
    pub evidence: Option<String>,
    /// Why it is stuck or postponed — required to reach `blocked` and `deferred` (`missing_note` otherwise).
    pub note: Option<String>,
    /// External references (D10); absent means the empty set.
    pub references: Option<Vec<ReferenceParam>>,
    /// Agent-set triage priority (D11) — display-only, never an ordering input.
    pub priority: Option<PriorityDto>,
    /// Optional until-date (D9), pure metadata — `YYYY-MM-DD`.
    pub until: Option<String>,
}

/// `decision_add`'s parameters (§6, D10): the dossier's every field except the identity and the timestamps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionAddParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the decision; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// Short human title.
    pub title: String,
    /// Lifecycle status: open, signed or superseded — `signed`/`superseded` require a resolution.
    pub status: DecisionStatusDto,
    /// The options the user chooses between, in authoring order.
    pub options: Vec<String>,
    /// The proposer's recommendation among the options.
    pub recommendation: Option<String>,
    /// How it was resolved — required to reach `signed`/`superseded` (`missing_resolution` otherwise).
    pub resolution: Option<String>,
    /// Proposer confidence at drafting time: low, medium or high.
    pub confidence: ConfidenceDto,
    /// External references (D10); absent means the empty set.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `doc_register`'s parameters (§6, D10): the doc's every field except the identity and the timestamps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocRegisterParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the doc; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// Short human title.
    pub title: String,
    /// The mutable file path of the prose body — an attribute, moved later by `doc_set_path`.
    pub path: String,
    /// The document's version — an opaque string the document itself declares.
    pub version: String,
    /// External references (D10); absent means the empty set.
    pub references: Option<Vec<ReferenceParam>>,
}

// =================================================================
// The update params — the store's patches, wire-typed
// =================================================================

/// `epic_update`'s parameters (§6): the patchable epic fields — an absent field is untouched, a value replaces it whole; an epic's strings cannot be cleared, only replaced.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct EpicUpdateParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the epic; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The epic's id, e.g. `camera:E1`.
    pub epic: String,
    /// A new title; absent leaves the field untouched.
    pub title: Option<String>,
    /// A new description; absent leaves the field untouched.
    pub description: Option<String>,
    /// A new lifecycle status; absent leaves the field untouched.
    pub status: Option<StatusDto>,
    /// A replacement reference set — the whole set, canonically sorted; absent leaves the field untouched.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `task_update`'s parameters (§6): the patchable task fields under RFC 7386's three-state reading — absent is untouched, `null` clears an optional field, a value replaces it whole.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskUpdateParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the task; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The task's id, e.g. `camera:U7`.
    pub task: String,
    /// A new title; absent leaves the field untouched.
    pub title: Option<String>,
    /// A new body; absent leaves the field untouched.
    pub body: Option<String>,
    /// A new task kind; absent leaves the field untouched.
    pub kind: Option<TaskKindDto>,
    /// A new lifecycle status; the closure discipline is checked over the result.
    pub status: Option<StatusDto>,
    /// A new verify method; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub verify: Option<Option<String>>,
    /// New closure evidence; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub evidence: Option<Option<String>>,
    /// A new blocked/deferred note; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub note: Option<Option<String>>,
    /// A replacement reference set — the whole set, canonically sorted; absent leaves the field untouched.
    pub references: Option<Vec<ReferenceParam>>,
    /// New display-only priority (D11); absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub priority: Option<Option<PriorityDto>>,
    /// A new until-date (D9); absent leaves the field untouched, `null` clears it, a value replaces it — `YYYY-MM-DD`.
    #[serde(default, deserialize_with = "double_option")]
    pub until: Option<Option<String>>,
    /// A new parent epic — a reparenting, checked like one (§5.1); absent leaves the task where it is.
    pub epic: Option<String>,
}

/// `decision_update`'s parameters (§6, D10): the patchable dossier fields under RFC 7386's three-state reading — absent is untouched, `null` clears an optional field, a value replaces it whole.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionUpdateParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the decision; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The decision's id, e.g. `camera:D3`.
    pub decision: String,
    /// A new title; absent leaves the field untouched.
    pub title: Option<String>,
    /// A new lifecycle status; `signed`/`superseded` require a resolution (§3).
    pub status: Option<DecisionStatusDto>,
    /// A replacement option list, in authoring order; absent leaves the field untouched.
    pub options: Option<Vec<String>>,
    /// A new recommendation; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub recommendation: Option<Option<String>>,
    /// A new resolution; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub resolution: Option<Option<String>>,
    /// A new proposer confidence; absent leaves the field untouched.
    pub confidence: Option<ConfidenceDto>,
    /// A replacement reference set — the whole set, canonically sorted; absent leaves the field untouched.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `doc_update`'s parameters (§6, D10): the patchable doc fields — no `path`, which the separately audited verb `doc_set_path` owns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocUpdateParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the doc; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The doc's id, e.g. `camera:S1`.
    pub doc: String,
    /// A new title; absent leaves the field untouched.
    pub title: Option<String>,
    /// A new version string; absent leaves the field untouched.
    pub version: Option<String>,
    /// A replacement reference set — the whole set, canonically sorted; absent leaves the field untouched.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `doc_set_path`'s parameters (§6, D10): the audited move — a citation's address change is its own verb, visible in the log on its own line.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocSetPathParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the doc; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The doc's id, e.g. `camera:S1`.
    pub doc: String,
    /// The doc's new filesystem path of the prose body.
    pub path: String,
}

// =================================================================
// The answer helpers — parse, resolve, convert, mutate
// =================================================================

/// `epic_add`'s answer: parse, resolve, convert, stamp, add.
fn answer_epic_add(state: &mut ServerState, params: &EpicAddParams) -> Result<EpicDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let references = references_of(params.references.as_deref().unwrap_or(&[]))?;
    let draft = EpicDraft {
        title: params.title.clone(),
        description: params.description.clone(),
        status: Status::from(params.status),
        references,
    };
    let handle = state.resolve(&slug)?;
    let epic = handle.add_epic(draft, time::now())?;
    Ok(EpicDto::from(&epic))
}

/// `epic_update`'s answer: parse, resolve, convert, stamp, patch.
fn answer_epic_update(
    state: &mut ServerState,
    params: &EpicUpdateParams,
) -> Result<EpicDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let epic = parse_value::<EpicId>(&params.epic)?;
    let references = params
        .references
        .as_deref()
        .map(references_of)
        .transpose()?;
    let patch = EpicPatch {
        title: params.title.clone(),
        description: params.description.clone(),
        status: params.status.map(Status::from),
        references,
    };
    let handle = state.resolve(&slug)?;
    let epic = handle.update_epic(epic, time::now(), patch)?;
    Ok(EpicDto::from(&epic))
}

/// `task_add`'s answer: parse, resolve, convert, stamp, add.
fn answer_task_add(state: &mut ServerState, params: &TaskAddParams) -> Result<TaskDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let epic = parse_value::<EpicId>(&params.epic)?;
    let references = references_of(params.references.as_deref().unwrap_or(&[]))?;
    let until = params
        .until
        .as_deref()
        .map(Date::parse)
        .transpose()
        .map_err(ToolError::from)?;
    let draft = TaskDraft {
        epic,
        kind: TaskKind::from(params.kind),
        title: params.title.clone(),
        body: params.body.clone(),
        status: Status::from(params.status),
        verify: params.verify.clone(),
        evidence: params.evidence.clone(),
        note: params.note.clone(),
        references,
        priority: params.priority.map(Priority::from),
        until,
    };
    let handle = state.resolve(&slug)?;
    let task = handle.add_task(draft, time::now())?;
    Ok(TaskDto::from(&task))
}

/// `task_update`'s answer: parse, resolve, convert, stamp, patch.
fn answer_task_update(
    state: &mut ServerState,
    params: &TaskUpdateParams,
) -> Result<TaskDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let task = parse_value::<TaskId>(&params.task)?;
    let references = params
        .references
        .as_deref()
        .map(references_of)
        .transpose()?;
    let until = until_of(&params.until)?;
    let epic = params
        .epic
        .as_deref()
        .map(parse_value::<EpicId>)
        .transpose()?;
    let patch = TaskPatch {
        title: params.title.clone(),
        body: params.body.clone(),
        kind: params.kind.map(TaskKind::from),
        status: params.status.map(Status::from),
        verify: params.verify.clone(),
        evidence: params.evidence.clone(),
        note: params.note.clone(),
        references,
        priority: params.priority.map(|inner| inner.map(Priority::from)),
        until,
        epic,
    };
    let handle = state.resolve(&slug)?;
    let task = handle.update_task(task, time::now(), patch)?;
    Ok(TaskDto::from(&task))
}

/// `decision_add`'s answer: parse, resolve, convert, stamp, add.
fn answer_decision_add(
    state: &mut ServerState,
    params: &DecisionAddParams,
) -> Result<DecisionDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let references = references_of(params.references.as_deref().unwrap_or(&[]))?;
    let draft = DecisionDraft {
        title: params.title.clone(),
        status: DecisionStatus::from(params.status),
        options: params.options.clone(),
        recommendation: params.recommendation.clone(),
        resolution: params.resolution.clone(),
        confidence: Confidence::from(params.confidence),
        references,
    };
    let handle = state.resolve(&slug)?;
    let decision = handle.add_decision(draft, time::now())?;
    Ok(DecisionDto::from(&decision))
}

/// `decision_update`'s answer: parse, resolve, convert, stamp, patch.
fn answer_decision_update(
    state: &mut ServerState,
    params: &DecisionUpdateParams,
) -> Result<DecisionDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let decision = parse_value::<DecisionId>(&params.decision)?;
    let references = params
        .references
        .as_deref()
        .map(references_of)
        .transpose()?;
    let patch = DecisionPatch {
        title: params.title.clone(),
        status: params.status.map(DecisionStatus::from),
        options: params.options.clone(),
        recommendation: params.recommendation.clone(),
        resolution: params.resolution.clone(),
        confidence: params.confidence.map(Confidence::from),
        references,
    };
    let handle = state.resolve(&slug)?;
    let decision = handle.update_decision(decision, time::now(), patch)?;
    Ok(DecisionDto::from(&decision))
}

/// `doc_register`'s answer: parse, resolve, convert, stamp, add.
fn answer_doc_register(
    state: &mut ServerState,
    params: &DocRegisterParams,
) -> Result<DocDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let references = references_of(params.references.as_deref().unwrap_or(&[]))?;
    let draft = DocDraft {
        title: params.title.clone(),
        path: params.path.clone(),
        version: params.version.clone(),
        references,
    };
    let handle = state.resolve(&slug)?;
    let doc = handle.add_doc(draft, time::now())?;
    Ok(DocDto::from(&doc))
}

/// `doc_set_path`'s answer: parse, resolve, stamp, move.
fn answer_doc_set_path(
    state: &mut ServerState,
    params: &DocSetPathParams,
) -> Result<DocDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let doc = parse_value::<DocId>(&params.doc)?;
    let handle = state.resolve(&slug)?;
    let doc = handle.set_doc_path(doc, params.path.clone(), time::now())?;
    Ok(DocDto::from(&doc))
}

/// `doc_update`'s answer: parse, resolve, convert, stamp, patch.
fn answer_doc_update(
    state: &mut ServerState,
    params: &DocUpdateParams,
) -> Result<DocDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let doc = parse_value::<DocId>(&params.doc)?;
    let references = params
        .references
        .as_deref()
        .map(references_of)
        .transpose()?;
    let patch = DocPatch {
        title: params.title.clone(),
        version: params.version.clone(),
        references,
    };
    let handle = state.resolve(&slug)?;
    let doc = handle.update_doc(doc, time::now(), patch)?;
    Ok(DocDto::from(&doc))
}

// =================================================================
// The tools
// =================================================================

#[tool_router(router = nodes_router, vis = "pub(crate)")]
impl TodoServer {
    /// Add an epic: the graph allocates its id, and the response is the new state — an epic carries no timestamps, so the answer is fully deterministic.
    #[tool(
        name = "epic_add",
        output_schema = crate::schema::output_for_type::<EpicDto>(),
        annotations(open_world_hint = false)
    )]
    async fn epic_add(
        &self,
        Parameters(params): Parameters<EpicAddParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it.
        let mut state = self.lock_state();
        match answer_epic_add(&mut state, &params) {
            Ok(output) => structured("epic_add", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Patch an epic in place — absent fields untouched, given fields replaced whole — and answer with the new state.
    #[tool(
        name = "epic_update",
        output_schema = crate::schema::output_for_type::<EpicDto>(),
        annotations(open_world_hint = false)
    )]
    async fn epic_update(
        &self,
        Parameters(params): Parameters<EpicUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_epic_update(&mut state, &params) {
            Ok(output) => structured("epic_update", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Add a task under an epic: the graph allocates its id, the closure discipline is checked over the draft, and the response is the new state with its clock-stamped `created`/`updated`.
    #[tool(
        name = "task_add",
        output_schema = crate::schema::output_for_type::<TaskDto>(),
        annotations(open_world_hint = false)
    )]
    async fn task_add(
        &self,
        Parameters(params): Parameters<TaskAddParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_task_add(&mut state, &params) {
            Ok(output) => structured("task_add", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Patch a task in place — absent fields untouched, `null` clears an optional one, a value replaces it whole; `epic` reparents, validated like one — and answer with the new state, `updated` clock-stamped.
    #[tool(
        name = "task_update",
        output_schema = crate::schema::output_for_type::<TaskDto>(),
        annotations(open_world_hint = false)
    )]
    async fn task_update(
        &self,
        Parameters(params): Parameters<TaskUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_task_update(&mut state, &params) {
            Ok(output) => structured("task_update", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Add a decision dossier: the graph allocates its id, `signed`/`superseded` require a resolution, and the response is the new state with its clock-stamped `created`/`updated`.
    #[tool(
        name = "decision_add",
        output_schema = crate::schema::output_for_type::<DecisionDto>(),
        annotations(open_world_hint = false)
    )]
    async fn decision_add(
        &self,
        Parameters(params): Parameters<DecisionAddParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_decision_add(&mut state, &params) {
            Ok(output) => structured("decision_add", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Patch a decision dossier in place — absent fields untouched, `null` clears an optional one, a value replaces it whole; signing requires a resolution — and answer with the new state, `updated` clock-stamped.
    #[tool(
        name = "decision_update",
        output_schema = crate::schema::output_for_type::<DecisionDto>(),
        annotations(open_world_hint = false)
    )]
    async fn decision_update(
        &self,
        Parameters(params): Parameters<DecisionUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_decision_update(&mut state, &params) {
            Ok(output) => structured("decision_update", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Register a document for stable citation: the graph allocates its id, and the response is the new state with its clock-stamped `created`/`updated`.
    #[tool(
        name = "doc_register",
        output_schema = crate::schema::output_for_type::<DocDto>(),
        annotations(open_world_hint = false)
    )]
    async fn doc_register(
        &self,
        Parameters(params): Parameters<DocRegisterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_register(&mut state, &params) {
            Ok(output) => structured("doc_register", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Move a doc's body to a new path — the audited verb (D10): a citation's address change is its own event — and answer with the moved doc's new state, `updated` clock-stamped.
    #[tool(
        name = "doc_set_path",
        output_schema = crate::schema::output_for_type::<DocDto>(),
        annotations(open_world_hint = false)
    )]
    async fn doc_set_path(
        &self,
        Parameters(params): Parameters<DocSetPathParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_set_path(&mut state, &params) {
            Ok(output) => structured("doc_set_path", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Patch a doc in place — absent fields untouched, given fields replaced whole; the path moves only through `doc_set_path` — and answer with the new state, `updated` clock-stamped.
    #[tool(
        name = "doc_update",
        output_schema = crate::schema::output_for_type::<DocDto>(),
        annotations(open_world_hint = false)
    )]
    async fn doc_update(
        &self,
        Parameters(params): Parameters<DocUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_update(&mut state, &params) {
            Ok(output) => structured("doc_update", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
