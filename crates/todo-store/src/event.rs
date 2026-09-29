//! The persisted event schema of spec §4.1: every graph mutation is one
//! JSON line, and this module defines those lines.
//!
//! The domain hands the persisted shape to this crate (`graph/drafts.rs`:
//! "the persisted shape is the store's (§4.1)"), so the shapes here are
//! the format the golden files pin and that t13's inspector reads. Three
//! rules decided them:
//!
//! - **One kind per domain mutation, 1:1.** The seventeen variants of
//!   [`Event`] mirror [`Graph`]'s mutating API exactly. There is no
//!   `task_reparent` kind — [`Graph::reparent_task`] is sugar over
//!   `update_task`, and [`TaskPatch::epic`] carries it — and no
//!   single-task delete, because the domain has none: a task leaves the
//!   graph only through the epic-delete cascade.
//! - **Flat, internally tagged lines.** `{"kind":"task_add",…}` reads in
//!   an editor, which §4's human-auditability requirement asks for. Field
//!   order is declaration order (the codec's contract) and every variant
//!   carries its `ts` first.
//! - **The line is the audit record.** `ts` is the audit instant and is
//!   always caller-passed (§8: nothing below the binary layer reads a
//!   wall clock). On an add it equals the node's `created` by
//!   construction; on the mutations whose node carries no timestamp of
//!   its own — `repo_set_path`/`repo_set_name`, and every `epic_*`
//!   event, because a [`Repo`] and an [`Epic`] have no `updated` field
//!   (§3) — the event's `ts` *is* the audit trail of the change.
//!
//! Patches are DTOs, not closures. The domain's `update_*` methods take a
//! closure over the node, which can rewrite anything, `created` included;
//! the persisted surface cannot. [`TaskPatch`] and its three siblings
//! omit `id` and `repo` (identity is not patchable) and `created` and
//! `updated` (node history is not rewritable — `updated` is the event's
//! `ts`), so a log line can only change what a client meant to change.
//! [`apply_task_patch`] and its siblings are the one place a DTO becomes
//! a node mutation: the live append path and replay both go through them,
//! so the two cannot drift.
//!
//! [`BlocksEdgeDto`] mirrors [`BlocksEdge`]'s three variants for the
//! reason the domain enum has three: a forbidden topology — a task
//! blocking an epic, an epic source, a decision target — is not
//! representable in the log either, and the conversions both ways are
//! total.
//!
//! A patch field is `Option<Option<T>>` where the node's own field is
//! `Option<T>`: the outer option is "this patch does not touch it", the
//! inner one the value, so clearing a field is expressible (`Some(None)`)
//! and distinct from leaving it alone (`None`). On the wire that is RFC
//! 7386's reading — absent keeps, `null` clears, a value replaces — and it
//! needs both halves of the mechanism: `skip_serializing_if` so an
//! untouched field is absent rather than `null`, and [`double_option`] so a
//! `null` that *is* there reads back as `Some(None)`. Serde's derived
//! `Option<Option<T>>` collapses the two, and a patch that silently loses a
//! clear is a replay that ends somewhere other than the session did.
//!
//! Over the file-size target by recorded exception (`_rust.md`, board
//! §5/U13): the persisted schema, its patch application and its replay
//! are one responsibility today — a format change reads as one diff. The
//! split, when M2's query layer shows the seam, is the replay half
//! ([`replay_step`] + [`ReplayFault`]).

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use todo_domain::{
    BlocksEdge, Confidence, Date, Decision, DecisionId, DecisionStatus, Doc, DocId, DomainError,
    Epic, EpicId, Graph, Priority, References, Repo, Status, Task, TaskId, TaskKind, Timestamp,
};

use crate::FORMAT_VERSION;

/// The log's first line: the format version every reader checks before it
/// replays anything (§4's versioning requirement — a format change is a
/// migration with a version header, never an in-place edit of meaning).
///
/// `deny_unknown_fields` is f14's landing (U10's split): deserialize-only,
/// so no golden moves, and the registry and snapshot documents are the
/// precedent — an unknown field at the *current* version can only be
/// damage, while a header from a newer version is the migration question
/// [`crate::codec::VersionProbe`]'s ordering reports it as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LogHeader {
    /// The [`FORMAT_VERSION`] this log was written with.
    pub(crate) format_version: u32,
}

impl LogHeader {
    /// The header a writer of the current format starts a log with.
    pub(crate) fn current() -> Self {
        Self {
            format_version: FORMAT_VERSION,
        }
    }
}

/// One line of the event log: a single graph mutation, in the shape the
/// golden files pin.
///
/// The wire spelling of a variant is its `kind` tag — [`Event::kind`]
/// returns it — and every variant's first field is the audit instant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// The log's seed: the repository this store holds. Always the first
    /// event; replay starts from [`Graph::new`] over it and rejects a
    /// second one.
    RepoRegister {
        /// The audit instant — `repo`'s registration time.
        ts: Timestamp,
        /// The registered repository, in full.
        repo: Repo,
    },
    /// An epic the graph allocated: the stored node, id included.
    EpicAdd {
        /// The audit instant. An epic carries no timestamps (§3), so
        /// this line is the record of when it was added.
        ts: Timestamp,
        /// The allocated epic.
        epic: Epic,
    },
    /// A task the graph allocated: the stored node, id included.
    TaskAdd {
        /// The audit instant — the task's `created` by construction.
        ts: Timestamp,
        /// The allocated task.
        task: Task,
    },
    /// A decision the graph allocated: the stored node, id included.
    DecisionAdd {
        /// The audit instant — the decision's `created` by construction.
        ts: Timestamp,
        /// The allocated decision.
        decision: Decision,
    },
    /// A document the graph registered: the stored node, id included.
    DocRegister {
        /// The audit instant — the doc's `created` by construction.
        ts: Timestamp,
        /// The registered doc.
        doc: Doc,
    },
    /// An epic patched in place.
    EpicUpdate {
        /// The audit instant. An epic has no `updated` field (§3), so
        /// this line is the only record of when it changed.
        ts: Timestamp,
        /// The epic being patched.
        id: EpicId,
        /// The fields it changes.
        patch: EpicPatch,
    },
    /// A task patched in place; the node's `updated` becomes `ts`.
    TaskUpdate {
        /// The audit instant, and the task's new `updated`.
        ts: Timestamp,
        /// The task being patched.
        id: TaskId,
        /// The fields it changes.
        patch: TaskPatch,
    },
    /// A decision patched in place; the node's `updated` becomes `ts`.
    DecisionUpdate {
        /// The audit instant, and the decision's new `updated`.
        ts: Timestamp,
        /// The decision being patched.
        id: DecisionId,
        /// The fields it changes.
        patch: DecisionPatch,
    },
    /// A doc patched in place; the node's `updated` becomes `ts`.
    ///
    /// The doc's `path` is not patchable here — it moves through
    /// [`Event::DocSetPath`], §6/D10's separately audited verb.
    DocUpdate {
        /// The audit instant, and the doc's new `updated`.
        ts: Timestamp,
        /// The doc being patched.
        id: DocId,
        /// The fields it changes.
        patch: DocPatch,
    },
    /// A doc's body moved (§6, D10): audited as its own verb, so a
    /// citation's address change is visible in the log on its own line.
    DocSetPath {
        /// The audit instant, and the doc's new `updated`.
        ts: Timestamp,
        /// The doc that moved.
        id: DocId,
        /// Its new path.
        path: String,
    },
    /// A checkout moved (§7): the slug is the key, the path an attribute.
    RepoSetPath {
        /// The audit instant. A [`Repo`] has no `updated` field (§3), so
        /// this line is the only record of when the path changed.
        ts: Timestamp,
        /// The new filesystem path of the checkout.
        path: String,
    },
    /// The repo's display name changed (§7).
    RepoSetName {
        /// The audit instant — the only record of when the name changed.
        ts: Timestamp,
        /// The new display name.
        name: String,
    },
    /// An epic deleted, and with it every task that could not survive it
    /// (§5.1: no orphans, ever). Replay re-runs the domain cascade, so
    /// the log records the intent and the domain decides the extent.
    ///
    /// The cascade burns the deleted tasks' numbers (§3: counters only
    /// move up), which is what makes the gaps in a later `task_add`
    /// meaningful — and what [`ReplayFault::NonSequential`] protects.
    EpicDelete {
        /// The audit instant.
        ts: Timestamp,
        /// The epic being deleted.
        id: EpicId,
    },
    /// A `blocks` edge added (§5.1), cycle-checked at write time.
    BlocksLink {
        /// The audit instant.
        ts: Timestamp,
        /// The edge, in one of the three shapes §5.1/D2 allow.
        edge: BlocksEdgeDto,
    },
    /// A `blocks` edge removed. Unlinking an absent edge changes nothing
    /// and is not an error, mirroring [`Graph::unlink_blocks`].
    BlocksUnlink {
        /// The audit instant.
        ts: Timestamp,
        /// The edge being removed.
        edge: BlocksEdgeDto,
    },
    /// A `relates` annotation added between two tasks (§5.1). The pair is
    /// undirected, so `(a, b)` and `(b, a)` are one annotation.
    RelatesLink {
        /// The audit instant.
        ts: Timestamp,
        /// One end of the pair.
        a: TaskId,
        /// The other end.
        b: TaskId,
    },
    /// A `relates` annotation removed. Removing an absent annotation
    /// changes nothing and is not an error, mirroring
    /// [`Graph::unlink_relates`].
    RelatesUnlink {
        /// The audit instant.
        ts: Timestamp,
        /// One end of the pair.
        a: TaskId,
        /// The other end.
        b: TaskId,
    },
}

impl Event {
    /// The `kind` tag this event serializes with — the log's stable
    /// spelling of the mutation, e.g. `"task_add"`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::RepoRegister { .. } => "repo_register",
            Self::EpicAdd { .. } => "epic_add",
            Self::TaskAdd { .. } => "task_add",
            Self::DecisionAdd { .. } => "decision_add",
            Self::DocRegister { .. } => "doc_register",
            Self::EpicUpdate { .. } => "epic_update",
            Self::TaskUpdate { .. } => "task_update",
            Self::DecisionUpdate { .. } => "decision_update",
            Self::DocUpdate { .. } => "doc_update",
            Self::DocSetPath { .. } => "doc_set_path",
            Self::RepoSetPath { .. } => "repo_set_path",
            Self::RepoSetName { .. } => "repo_set_name",
            Self::EpicDelete { .. } => "epic_delete",
            Self::BlocksLink { .. } => "blocks_link",
            Self::BlocksUnlink { .. } => "blocks_unlink",
            Self::RelatesLink { .. } => "relates_link",
            Self::RelatesUnlink { .. } => "relates_unlink",
        }
    }

    /// The audit instant this event carries. Total: every variant has
    /// one, and it is the log's only time source (§8).
    pub fn ts(&self) -> &Timestamp {
        match self {
            Self::RepoRegister { ts, .. }
            | Self::EpicAdd { ts, .. }
            | Self::TaskAdd { ts, .. }
            | Self::DecisionAdd { ts, .. }
            | Self::DocRegister { ts, .. }
            | Self::EpicUpdate { ts, .. }
            | Self::TaskUpdate { ts, .. }
            | Self::DecisionUpdate { ts, .. }
            | Self::DocUpdate { ts, .. }
            | Self::DocSetPath { ts, .. }
            | Self::RepoSetPath { ts, .. }
            | Self::RepoSetName { ts, .. }
            | Self::EpicDelete { ts, .. }
            | Self::BlocksLink { ts, .. }
            | Self::BlocksUnlink { ts, .. }
            | Self::RelatesLink { ts, .. }
            | Self::RelatesUnlink { ts, .. } => ts,
        }
    }
}

/// The patchable fields of a [`Task`].
///
/// The wire convention is JSON Merge Patch's (RFC 7386), and it is the
/// same on all four DTOs: an **absent** field is untouched, a **`null`**
/// clears an optional one, a **value** replaces the field whole. Every
/// `None` is therefore skipped on serialization, so an update line reads
/// as the delta it is — §4's human-auditability requirement — and a no-op
/// patch is `"patch":{}`.
///
/// Deliberately absent: `id` and `repo` (identity is not patchable),
/// `created` and `updated` (node history is not rewritable through the
/// persisted surface — `updated` is the event's `ts`). The domain's
/// closure API can still set `created`; the log and the wire cannot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskPatch {
    /// A new title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A new body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// A new task kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<TaskKind>,
    /// A new status; the closure discipline (§3) is checked over the
    /// result, so `closed` without evidence is rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// A new `verify` method, or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub verify: Option<Option<String>>,
    /// New closure evidence, or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub evidence: Option<Option<String>>,
    /// A new blocked/deferred note, or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub note: Option<Option<String>>,
    /// A replacement reference set — the whole set, since a partial edit
    /// of a sorted set has no canonical line of its own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<References>,
    /// New display-only priority (D11), or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub priority: Option<Option<Priority>>,
    /// A new until-date (D9), pure metadata, or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub until: Option<Option<Date>>,
    /// A new parent epic — a reparenting, checked like one (§5.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic: Option<EpicId>,
}

/// The patchable fields of an [`Epic`], under [`TaskPatch`]'s wire
/// convention. An epic has no timestamps (§3), so its audit instant is the
/// event's `ts` alone.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpicPatch {
    /// A new title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A new description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A new status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// A replacement reference set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<References>,
}

/// The patchable fields of a [`Decision`], under [`TaskPatch`]'s wire
/// convention. Signing one changes no edge — its `blocks` edges simply stop
/// gating (§3/D10).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionPatch {
    /// A new title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A new status; `signed`/`superseded` require a resolution (§3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DecisionStatus>,
    /// A replacement option list, in authoring order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    /// A new recommendation, or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub recommendation: Option<Option<String>>,
    /// A new resolution, or `null` to clear it.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub resolution: Option<Option<String>>,
    /// A new proposer confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    /// A replacement reference set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<References>,
}

/// The patchable fields of a [`Doc`], under [`TaskPatch`]'s wire
/// convention. The `path` is absent: it moves through
/// [`Event::DocSetPath`], §6/D10's separately audited verb.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocPatch {
    /// A new title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A new version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// A replacement reference set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<References>,
}

/// A `blocks` edge as the log spells it: the three endpoint combinations
/// §5.1/D2 allow, so a forbidden topology cannot appear in a log any more
/// than it can be constructed in memory. Conversions to and from
/// [`BlocksEdge`] are total in both directions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum BlocksEdgeDto {
    /// A task holding back another task.
    TaskToTask {
        /// The blocking task.
        from: TaskId,
        /// The task held back.
        to: TaskId,
    },
    /// A decision gating a task (D10).
    DecisionToTask {
        /// The gating decision.
        from: DecisionId,
        /// The task it holds back.
        to: TaskId,
    },
    /// A decision gating an epic, and so all work under it (D10).
    DecisionToEpic {
        /// The gating decision.
        from: DecisionId,
        /// The epic it holds back.
        to: EpicId,
    },
}

impl From<BlocksEdge> for BlocksEdgeDto {
    fn from(edge: BlocksEdge) -> Self {
        match edge {
            BlocksEdge::TaskToTask { from, to } => Self::TaskToTask { from, to },
            BlocksEdge::DecisionToTask { from, to } => Self::DecisionToTask { from, to },
            BlocksEdge::DecisionToEpic { from, to } => Self::DecisionToEpic { from, to },
        }
    }
}

impl From<BlocksEdgeDto> for BlocksEdge {
    fn from(edge: BlocksEdgeDto) -> Self {
        match edge {
            BlocksEdgeDto::TaskToTask { from, to } => Self::TaskToTask { from, to },
            BlocksEdgeDto::DecisionToTask { from, to } => Self::DecisionToTask { from, to },
            BlocksEdgeDto::DecisionToEpic { from, to } => Self::DecisionToEpic { from, to },
        }
    }
}

/// Deserializes one patch field whose node field is itself optional.
///
/// JSON has a single spelling for "no value", so serde's derived
/// `Option<Option<T>>` cannot tell an absent field (leave it alone) from a
/// `null` one (clear it) — both arrive as `None`. `#[serde(default)]` fills
/// the absent case, and this function the present one, wrapping whatever
/// the inner option read: `null` becomes `Some(None)`, a value
/// `Some(Some(value))`. Without it a clearing patch would replay as a
/// no-op and the replayed graph would not equal the one that was written.
fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Writes a [`TaskPatch`] onto a task: every `Some` replaces its field
/// whole, every `None` leaves it. The one place a patch DTO becomes a
/// node mutation, shared by the live append path and by replay.
pub fn apply_task_patch(task: &mut Task, patch: &TaskPatch) {
    if let Some(title) = &patch.title {
        task.title = title.clone();
    }
    if let Some(body) = &patch.body {
        task.body = body.clone();
    }
    if let Some(kind) = patch.kind {
        task.kind = kind;
    }
    if let Some(status) = patch.status {
        task.status = status;
    }
    if let Some(verify) = &patch.verify {
        task.verify = verify.clone();
    }
    if let Some(evidence) = &patch.evidence {
        task.evidence = evidence.clone();
    }
    if let Some(note) = &patch.note {
        task.note = note.clone();
    }
    if let Some(references) = &patch.references {
        task.references = references.clone();
    }
    if let Some(priority) = &patch.priority {
        task.priority = *priority;
    }
    if let Some(until) = &patch.until {
        task.until = until.clone();
    }
    if let Some(epic) = &patch.epic {
        task.epic = epic.clone();
    }
}

/// Writes an [`EpicPatch`] onto an epic — [`apply_task_patch`]'s
/// contract over the epic's fields.
pub fn apply_epic_patch(epic: &mut Epic, patch: &EpicPatch) {
    if let Some(title) = &patch.title {
        epic.title = title.clone();
    }
    if let Some(description) = &patch.description {
        epic.description = description.clone();
    }
    if let Some(status) = patch.status {
        epic.status = status;
    }
    if let Some(references) = &patch.references {
        epic.references = references.clone();
    }
}

/// Writes a [`DecisionPatch`] onto a decision — [`apply_task_patch`]'s
/// contract over the decision's fields.
pub fn apply_decision_patch(decision: &mut Decision, patch: &DecisionPatch) {
    if let Some(title) = &patch.title {
        decision.title = title.clone();
    }
    if let Some(status) = patch.status {
        decision.status = status;
    }
    if let Some(options) = &patch.options {
        decision.options = options.clone();
    }
    if let Some(recommendation) = &patch.recommendation {
        decision.recommendation = recommendation.clone();
    }
    if let Some(resolution) = &patch.resolution {
        decision.resolution = resolution.clone();
    }
    if let Some(confidence) = patch.confidence {
        decision.confidence = confidence;
    }
    if let Some(references) = &patch.references {
        decision.references = references.clone();
    }
}

/// Writes a [`DocPatch`] onto a doc — [`apply_task_patch`]'s contract
/// over the doc's fields. The path is not patchable here.
pub fn apply_doc_patch(doc: &mut Doc, patch: &DocPatch) {
    if let Some(title) = &patch.title {
        doc.title = title.clone();
    }
    if let Some(version) = &patch.version {
        doc.version = version.clone();
    }
    if let Some(references) = &patch.references {
        doc.references = references.clone();
    }
}

/// Why an event could not be replayed onto a graph.
///
/// Every case means the log contradicts itself: a live append allocates
/// ids through the graph and validates before it writes, so no log this
/// crate wrote can hold one. A reader turns a fault into §6.1's
/// `corrupt_store` carrying the offending line's byte offset — §4.3
/// leaves no automatic repair path, so the report is all there is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayFault {
    /// The domain rejected the event: a failed validation, a duplicate
    /// id, an unknown parent, a `blocks` cycle.
    Domain(DomainError),
    /// An add-event's id is not the next number of its kind. A lower
    /// number was already burned — §3's never-reuse rule survives an
    /// epic-delete cascade, which drops nodes but never lowers a counter
    /// — and a higher one skipped allocation.
    NonSequential {
        /// The id the event carries.
        id: String,
        /// The number the graph would have issued next.
        expected: u64,
    },
    /// A `repo_register` event that is not the log's first: the seed
    /// appears exactly once, because it is what the graph is built from.
    DuplicateSeed {
        /// The slug being registered a second time.
        slug: String,
    },
}

impl fmt::Display for ReplayFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(error) => write!(f, "the log contradicts itself: {error}"),
            Self::NonSequential { id, expected } => write!(
                f,
                "the log adds {id}, but the next number of its kind is {expected}: \
                 numbers are never reissued and never skipped (§3)"
            ),
            Self::DuplicateSeed { slug } => write!(
                f,
                "the log registers {slug} twice: repo_register is the seed and appears once, \
                 as the first event"
            ),
        }
    }
}

impl std::error::Error for ReplayFault {}

/// Applies one replayed event to a graph whose seed has already been
/// read (§4.1: load replays the log into the graph).
///
/// This is the same path [`crate::JsonlStore::open_log`] walks, exposed for a
/// reader that works over events rather than over a store handle — t13's
/// inspector dumping or verifying a log line by line. The seed event
/// itself is not applied here: it builds the graph the caller passes in,
/// and a second one is [`ReplayFault::DuplicateSeed`].
///
/// Adds go through the domain's explicit-id inserts and are checked
/// against the counters first ([`ReplayFault::NonSequential`]); every
/// other event is the domain mutation the live append path made, so a
/// replayed log and the session that wrote it end in equal graphs.
pub fn replay_step(graph: &mut Graph, event: &Event) -> Result<(), ReplayFault> {
    match event {
        Event::RepoRegister { repo, .. } => Err(ReplayFault::DuplicateSeed {
            slug: repo.slug.to_string(),
        }),
        Event::EpicAdd { epic, .. } => {
            expect_sequential(graph.counters().epic(), &epic.id, epic.id.number())?;
            graph.insert_epic(epic.clone()).map_err(ReplayFault::Domain)
        }
        Event::TaskAdd { task, .. } => {
            expect_sequential(graph.counters().task(), &task.id, task.id.number())?;
            graph.insert_task(task.clone()).map_err(ReplayFault::Domain)
        }
        Event::DecisionAdd { decision, .. } => {
            expect_sequential(
                graph.counters().decision(),
                &decision.id,
                decision.id.number(),
            )?;
            graph
                .insert_decision(decision.clone())
                .map_err(ReplayFault::Domain)
        }
        Event::DocRegister { doc, .. } => {
            expect_sequential(graph.counters().doc(), &doc.id, doc.id.number())?;
            graph.insert_doc(doc.clone()).map_err(ReplayFault::Domain)
        }
        // An epic carries no timestamps (§3), so `update_epic` takes no
        // `at` and the event's `ts` is the audit record alone.
        Event::EpicUpdate { id, patch, .. } => graph
            .update_epic(id.clone(), |epic| apply_epic_patch(epic, patch))
            .map_err(ReplayFault::Domain)
            .map(|_| ()),
        Event::TaskUpdate { ts, id, patch } => graph
            .update_task(id.clone(), ts.clone(), |task| apply_task_patch(task, patch))
            .map_err(ReplayFault::Domain)
            .map(|_| ()),
        Event::DecisionUpdate { ts, id, patch } => graph
            .update_decision(id.clone(), ts.clone(), |decision| {
                apply_decision_patch(decision, patch)
            })
            .map_err(ReplayFault::Domain)
            .map(|_| ()),
        Event::DocUpdate { ts, id, patch } => graph
            .update_doc(id.clone(), ts.clone(), |doc| apply_doc_patch(doc, patch))
            .map_err(ReplayFault::Domain)
            .map(|_| ()),
        Event::DocSetPath { ts, id, path } => graph
            .update_doc(id.clone(), ts.clone(), |doc| doc.path = path.clone())
            .map_err(ReplayFault::Domain)
            .map(|_| ()),
        Event::RepoSetPath { path, .. } => {
            graph.set_repo_path(path.clone());
            Ok(())
        }
        Event::RepoSetName { name, .. } => {
            graph.set_repo_name(name.clone());
            Ok(())
        }
        Event::EpicDelete { id, .. } => graph
            .delete_epic(id.clone())
            .map_err(ReplayFault::Domain)
            .map(|_| ()),
        Event::BlocksLink { edge, .. } => graph
            .link_blocks(edge.clone().into())
            .map_err(ReplayFault::Domain),
        Event::BlocksUnlink { edge, .. } => graph
            .unlink_blocks(edge.clone().into())
            .map_err(ReplayFault::Domain),
        Event::RelatesLink { a, b, .. } => graph
            .link_relates(a.clone(), b.clone())
            .map_err(ReplayFault::Domain),
        Event::RelatesUnlink { a, b, .. } => graph
            .unlink_relates(a.clone(), b.clone())
            .map_err(ReplayFault::Domain),
    }
}

/// §3's never-reuse rule on the replay path: an add-event's number must
/// be the next one of its kind, because the session that wrote it
/// allocated it through the graph. `counter` is the highest number issued
/// so far, `0` when none has been.
fn expect_sequential(counter: u64, id: &impl fmt::Display, number: u64) -> Result<(), ReplayFault> {
    let expected = counter.saturating_add(1);
    if number == expected {
        return Ok(());
    }
    Err(ReplayFault::NonSequential {
        id: id.to_string(),
        expected,
    })
}
