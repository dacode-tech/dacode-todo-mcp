//! The shared wire vocabulary: the server-local mirrors of
//! the domain's node and vocabulary types that every tool cluster
//! translates through — the vocabulary enums, [`ReferenceDto`],
//! [`TaskDto`], [`EpicDto`], [`DecisionDto`], [`DocDto`], [`RepoDto`]
//! and [`DeferredDto`] — the pieces the listing and mutation clusters
//! also consume, so the node mirrors have one home instead of
//! per-cluster copies.
//!
//! Server-local on purpose (§2): no `schemars` dependency reaches the
//! domain, so the outputSchema a client sees is generated here, from
//! mirrors. Named fields in the domain's declaration order (D14 — the
//! wire matches the domain's canonical spelling, the same vocabulary
//! the store's goldens pin); ids, timestamps and dates are `String`s
//! in their canonical spellings (`slug:U7`, the 30-byte RFC 3339 form,
//! `YYYY-MM-DD`), formats stated in the field docs; the vocabulary
//! enums mirror the domain's variant-for-variant with the same
//! `snake_case` wire spellings, so the outputSchema enumerates the
//! closed sets a client sees.
//!
//! Field docs are single-line throughout: they become the schema's
//! property descriptions verbatim (t33's rule — a multi-line comment
//! would carry its line breaks into the wire bytes). The same rule
//! holds for each struct's own doc, which the outputSchema carries as
//! its `description` (rmcp strips top-level titles and descriptions
//! from *input* schemas only, `handler/server/common.rs`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use todo_domain::{
    Confidence, Decision, DecisionStatus, DeferredSummary, Doc, Epic, Priority, Reference,
    ReferenceKind, Repo, Status, Task, TaskKind,
};

/// The wire mirror of the task/epic lifecycle status (§3, D9): `open`, `in_work`, `blocked`, `deferred`, `closed`, `wontfix`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusDto {
    /// Not started; actionable if the graph says so (§5.2).
    Open,
    /// Being worked on; actionable.
    InWork,
    /// Externally stuck; never actionable, requires a note.
    Blocked,
    /// Deliberately postponed (D9); never actionable, requires a note.
    Deferred,
    /// Finished with evidence of completed work.
    Closed,
    /// The one terminal that unblocks successors without evidence of completed work (§5.1).
    Wontfix,
}

impl From<Status> for StatusDto {
    fn from(status: Status) -> Self {
        match status {
            Status::Open => Self::Open,
            Status::InWork => Self::InWork,
            Status::Blocked => Self::Blocked,
            Status::Deferred => Self::Deferred,
            Status::Closed => Self::Closed,
            Status::Wontfix => Self::Wontfix,
        }
    }
}

/// The input direction of the vocabulary: a `task_list` filter value
/// back into the domain's enum, variant-for-variant and total.
impl From<StatusDto> for Status {
    fn from(status: StatusDto) -> Self {
        match status {
            StatusDto::Open => Self::Open,
            StatusDto::InWork => Self::InWork,
            StatusDto::Blocked => Self::Blocked,
            StatusDto::Deferred => Self::Deferred,
            StatusDto::Closed => Self::Closed,
            StatusDto::Wontfix => Self::Wontfix,
        }
    }
}

/// The wire mirror of the task-kind vocabulary (§3): `task`, `gate`, `finding`, `debt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TaskKindDto {
    /// One unit of work.
    Task,
    /// A checkpoint that must pass.
    Gate,
    /// Discovered along the way, not the point of the task.
    Finding,
    /// A deliberate shortcut, recorded rather than forgotten.
    Debt,
}

impl From<TaskKind> for TaskKindDto {
    fn from(kind: TaskKind) -> Self {
        match kind {
            TaskKind::Task => Self::Task,
            TaskKind::Gate => Self::Gate,
            TaskKind::Finding => Self::Finding,
            TaskKind::Debt => Self::Debt,
        }
    }
}

/// The input direction of the vocabulary: a `task_list` filter value
/// back into the domain's enum, variant-for-variant and total.
impl From<TaskKindDto> for TaskKind {
    fn from(kind: TaskKindDto) -> Self {
        match kind {
            TaskKindDto::Task => Self::Task,
            TaskKindDto::Gate => Self::Gate,
            TaskKindDto::Finding => Self::Finding,
            TaskKindDto::Debt => Self::Debt,
        }
    }
}

/// The wire mirror of the decision lifecycle status (D10): `open`, `signed`, `superseded`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DecisionStatusDto {
    /// Awaiting the user's sign-off; may block tasks and epics.
    Open,
    /// Signed off; its blocks edges simply stop gating (§5.1).
    Signed,
    /// Replaced by a later decision; stops gating like `signed`.
    Superseded,
}

impl From<DecisionStatus> for DecisionStatusDto {
    fn from(status: DecisionStatus) -> Self {
        match status {
            DecisionStatus::Open => Self::Open,
            DecisionStatus::Signed => Self::Signed,
            DecisionStatus::Superseded => Self::Superseded,
        }
    }
}

/// The input direction of the vocabulary: a `decision_add`/`decision_update` status back into the domain's enum, variant-for-variant and total.
impl From<DecisionStatusDto> for DecisionStatus {
    fn from(status: DecisionStatusDto) -> Self {
        match status {
            DecisionStatusDto::Open => Self::Open,
            DecisionStatusDto::Signed => Self::Signed,
            DecisionStatusDto::Superseded => Self::Superseded,
        }
    }
}

/// The wire mirror of the agent-set triage priority (D11): `high`, `medium`, `low` — display-only, never an ordering input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PriorityDto {
    /// Triage: look at this first.
    High,
    /// Triage: normal attention.
    Medium,
    /// Triage: look at this last.
    Low,
}

impl From<Priority> for PriorityDto {
    fn from(priority: Priority) -> Self {
        match priority {
            Priority::High => Self::High,
            Priority::Medium => Self::Medium,
            Priority::Low => Self::Low,
        }
    }
}

/// The input direction of the vocabulary: a `task_add`/`task_update` priority back into the domain's enum, variant-for-variant and total.
impl From<PriorityDto> for Priority {
    fn from(priority: PriorityDto) -> Self {
        match priority {
            PriorityDto::High => Self::High,
            PriorityDto::Medium => Self::Medium,
            PriorityDto::Low => Self::Low,
        }
    }
}

/// The wire mirror of the proposer confidence scale (D10): `low`, `medium`, `high`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConfidenceDto {
    /// The shape is a guess; expect rework.
    Low,
    /// Reasoned, with known open edges.
    Medium,
    /// As settled as a proposal can be.
    High,
}

impl From<Confidence> for ConfidenceDto {
    fn from(confidence: Confidence) -> Self {
        match confidence {
            Confidence::Low => Self::Low,
            Confidence::Medium => Self::Medium,
            Confidence::High => Self::High,
        }
    }
}

/// The input direction of the vocabulary: a `decision_add`/`decision_update` confidence back into the domain's enum, variant-for-variant and total.
impl From<ConfidenceDto> for Confidence {
    fn from(confidence: ConfidenceDto) -> Self {
        match confidence {
            ConfidenceDto::Low => Self::Low,
            ConfidenceDto::Medium => Self::Medium,
            ConfidenceDto::High => Self::High,
        }
    }
}

/// The wire mirror of the reference-kind vocabulary (§3, D10): `tracker`, `url`, `doc`, `spec`, `file`, `other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReferenceKindDto {
    /// An external issue tracker entry (e.g. a JIRA key).
    Tracker,
    /// A URL.
    Url,
    /// A registered document, optionally with a `#anchor` section suffix.
    Doc,
    /// A spec section citation.
    Spec,
    /// A file path.
    File,
    /// Anything else.
    Other,
}

impl From<ReferenceKind> for ReferenceKindDto {
    fn from(kind: ReferenceKind) -> Self {
        match kind {
            ReferenceKind::Tracker => Self::Tracker,
            ReferenceKind::Url => Self::Url,
            ReferenceKind::Doc => Self::Doc,
            ReferenceKind::Spec => Self::Spec,
            ReferenceKind::File => Self::File,
            ReferenceKind::Other => Self::Other,
        }
    }
}

/// The input direction of the vocabulary: a reference entry's kind back into the domain's enum, variant-for-variant and total.
impl From<ReferenceKindDto> for ReferenceKind {
    fn from(kind: ReferenceKindDto) -> Self {
        match kind {
            ReferenceKindDto::Tracker => Self::Tracker,
            ReferenceKindDto::Url => Self::Url,
            ReferenceKindDto::Doc => Self::Doc,
            ReferenceKindDto::Spec => Self::Spec,
            ReferenceKindDto::File => Self::File,
            ReferenceKindDto::Other => Self::Other,
        }
    }
}

/// The input mirror of one `{kind, target, note?}` reference entry (§3, D10) — the shape an add or update's `references` parameter takes, converted per entry through the domain's validating constructor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReferenceParam {
    /// The reference kind: tracker, url, doc, spec, file or other.
    pub kind: ReferenceKindDto,
    /// The target string, including any `#anchor` suffix on doc-kind references.
    pub target: String,
    /// The optional human note.
    pub note: Option<String>,
}

/// The wire mirror of one `{kind, target, note?}` reference entry (§3, D10), in the domain's field order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct ReferenceDto {
    /// The reference kind: tracker, url, doc, spec, file or other.
    pub kind: ReferenceKindDto,
    /// The target string, including any `#anchor` suffix on doc-kind references.
    pub target: String,
    /// The optional human note.
    pub note: Option<String>,
}

impl From<&Reference> for ReferenceDto {
    fn from(reference: &Reference) -> Self {
        Self {
            kind: reference.kind().into(),
            target: reference.target().to_owned(),
            note: reference.note().map(str::to_owned),
        }
    }
}

/// The wire mirror of a task node (§3): every field, in the domain's declaration order, ids and timestamps in their canonical spellings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct TaskDto {
    /// The task's id, e.g. `camera:U7`.
    pub id: String,
    /// The slug of the repository this task belongs to.
    pub repo: String,
    /// The parent epic's id — the parent edge of the tree (§5.1).
    pub epic: String,
    /// What kind of work this records: task, gate, finding or debt.
    pub kind: TaskKindDto,
    /// Short human title.
    pub title: String,
    /// Longer prose body.
    pub body: String,
    /// Lifecycle status: open, in_work, blocked, deferred, closed or wontfix.
    pub status: StatusDto,
    /// The method that will prove it done — required at creation.
    pub verify: Option<String>,
    /// What was actually run and seen — required to reach closed.
    pub evidence: Option<String>,
    /// Why it is stuck or postponed — required to reach blocked and deferred.
    pub note: Option<String>,
    /// External references (D10), canonically sorted by (kind, target, note).
    pub references: Vec<ReferenceDto>,
    /// Agent-set triage metadata (D11) — display-only, never an ordering input.
    pub priority: Option<PriorityDto>,
    /// Optional until-date (D9), pure metadata — `YYYY-MM-DD`.
    pub until: Option<String>,
    /// Creation time — RFC 3339 UTC with the nine-digit fraction, e.g. `2026-10-01T08:10:00.000000000Z`.
    pub created: String,
    /// Last mutation time — RFC 3339 UTC with the nine-digit fraction.
    pub updated: String,
}

impl From<&Task> for TaskDto {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id.to_string(),
            repo: task.repo.to_string(),
            epic: task.epic.to_string(),
            kind: task.kind.into(),
            title: task.title.clone(),
            body: task.body.clone(),
            status: task.status.into(),
            verify: task.verify.clone(),
            evidence: task.evidence.clone(),
            note: task.note.clone(),
            references: task.references.iter().map(ReferenceDto::from).collect(),
            priority: task.priority.map(PriorityDto::from),
            until: task.until.as_ref().map(|date| date.as_str().to_owned()),
            created: task.created.as_str().to_owned(),
            updated: task.updated.as_str().to_owned(),
        }
    }
}

/// The wire mirror of an epic node (§3): every field, in the domain's declaration order, ids as canonical `String`s.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct EpicDto {
    /// The epic's id, e.g. `camera:E1`.
    pub id: String,
    /// The slug of the repository this epic belongs to.
    pub repo: String,
    /// Short human title.
    pub title: String,
    /// Longer prose description.
    pub description: String,
    /// Lifecycle status: open, in_work, blocked, deferred, closed or wontfix.
    pub status: StatusDto,
    /// External references (D10), canonically sorted by (kind, target, note).
    pub references: Vec<ReferenceDto>,
}

impl From<&Epic> for EpicDto {
    fn from(epic: &Epic) -> Self {
        Self {
            id: epic.id.to_string(),
            repo: epic.repo.to_string(),
            title: epic.title.clone(),
            description: epic.description.clone(),
            status: epic.status.into(),
            references: epic.references.iter().map(ReferenceDto::from).collect(),
        }
    }
}

/// The wire mirror of a decision dossier (§3, D10): every field, in the domain's declaration order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DecisionDto {
    /// The decision's id, e.g. `camera:D3`.
    pub id: String,
    /// The slug of the repository this decision belongs to.
    pub repo: String,
    /// Short human title.
    pub title: String,
    /// Lifecycle status: open, signed or superseded.
    pub status: DecisionStatusDto,
    /// The options the user chooses between, in authoring order.
    pub options: Vec<String>,
    /// The proposer's recommendation among the options.
    pub recommendation: Option<String>,
    /// How it was resolved — required to reach signed or superseded.
    pub resolution: Option<String>,
    /// Proposer confidence at drafting time: low, medium or high.
    pub confidence: ConfidenceDto,
    /// External references (D10), canonically sorted by (kind, target, note).
    pub references: Vec<ReferenceDto>,
    /// Creation time — RFC 3339 UTC with the nine-digit fraction.
    pub created: String,
    /// Last mutation time — RFC 3339 UTC with the nine-digit fraction.
    pub updated: String,
}

impl From<&Decision> for DecisionDto {
    fn from(decision: &Decision) -> Self {
        Self {
            id: decision.id.to_string(),
            repo: decision.repo.to_string(),
            title: decision.title.clone(),
            status: decision.status.into(),
            options: decision.options.clone(),
            recommendation: decision.recommendation.clone(),
            resolution: decision.resolution.clone(),
            confidence: decision.confidence.into(),
            references: decision.references.iter().map(ReferenceDto::from).collect(),
            created: decision.created.as_str().to_owned(),
            updated: decision.updated.as_str().to_owned(),
        }
    }
}

/// The wire mirror of a doc node (§3, D10): every field, in the domain's declaration order, ids and timestamps in their canonical spellings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DocDto {
    /// The doc's id, e.g. `camera:S1`.
    pub id: String,
    /// The slug of the repository this doc belongs to.
    pub repo: String,
    /// Short human title.
    pub title: String,
    /// The mutable file path of the prose body.
    pub path: String,
    /// The document's version — an opaque string the document itself declares.
    pub version: String,
    /// External references (D10), canonically sorted by (kind, target, note).
    pub references: Vec<ReferenceDto>,
    /// Registration time — RFC 3339 UTC with the nine-digit fraction.
    pub created: String,
    /// Last mutation time — RFC 3339 UTC with the nine-digit fraction.
    pub updated: String,
}

impl From<&Doc> for DocDto {
    fn from(doc: &Doc) -> Self {
        Self {
            id: doc.id.to_string(),
            repo: doc.repo.to_string(),
            title: doc.title.clone(),
            path: doc.path.clone(),
            version: doc.version.clone(),
            references: doc.references.iter().map(ReferenceDto::from).collect(),
            created: doc.created.as_str().to_owned(),
            updated: doc.updated.as_str().to_owned(),
        }
    }
}

/// The wire mirror of a repo node (§7) — field-for-field `repo_register`'s answer shape (`RepoRegistered`, registry.rs), an acknowledged duplication: the write tool's answer stays in its cluster file, and a consolidation, if one ever happens, is a later task's call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct RepoDto {
    /// The repository's stable slug key.
    pub slug: String,
    /// The repository's display name.
    pub name: String,
    /// The filesystem path of the checkout.
    pub path: String,
    /// When the repository was registered — RFC 3339 UTC with the nine-digit fraction.
    pub registered_at: String,
}

impl From<&Repo> for RepoDto {
    fn from(repo: &Repo) -> Self {
        Self {
            slug: repo.slug.to_string(),
            name: repo.name.clone(),
            path: repo.path.clone(),
            registered_at: repo.registered_at.as_str().to_owned(),
        }
    }
}

/// The wire mirror of the deferred summary (§5.2's D9 amendment): how many deferred tasks the queried scope holds and how many are worth revisiting now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct DeferredDto {
    /// Deferred tasks in the queried scope.
    pub total: u64,
    /// Of those, the tasks whose every predecessor — their own until-target edges and their epic's decision gates — is finished.
    pub ready: u64,
}

impl From<DeferredSummary> for DeferredDto {
    fn from(deferred: DeferredSummary) -> Self {
        Self {
            total: deferred.total,
            ready: deferred.ready,
        }
    }
}
