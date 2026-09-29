//! The input shapes of the allocating adds (§6): a draft is a node's
//! fields without its identity, because the graph allocates the id and
//! stamps the repo. Keeping identity out of the type is what makes an
//! allocating add unable to collide with the replay path's explicit-id
//! inserts — a draft cannot carry a number, so it cannot carry a wrong
//! one.
//!
//! No serde derive here: the wire shape of a tool call is the server's
//! business (§6, M3), and the persisted shape is the store's (§4.1).

use crate::id::EpicId;
use crate::node::Confidence;
use crate::reference::References;
use crate::status::{DecisionStatus, Priority, Status, TaskKind};
use crate::time::Date;

/// The fields an `epic_add` supplies (§6): everything [`crate::Epic`] holds
/// except the identity the graph allocates. An epic carries no
/// timestamps (§3), so adding one takes none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpicDraft {
    /// Short human title.
    pub title: String,
    /// Longer prose description.
    pub description: String,
    /// Lifecycle status.
    pub status: Status,
    /// External references (D10).
    pub references: References,
}

/// The fields a `task_add` supplies (§6): everything [`crate::Task`] holds
/// except the identity the graph allocates and the timestamps the caller
/// passes separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDraft {
    /// The parent epic — the `parent` edge of §5.1, which must name an
    /// epic of this graph (`unknown_id` otherwise).
    pub epic: EpicId,
    /// What kind of work this records.
    pub kind: TaskKind,
    /// Short human title.
    pub title: String,
    /// Longer prose body.
    pub body: String,
    /// Lifecycle status (D9).
    pub status: Status,
    /// The method that will prove it done — required at creation
    /// (`missing_verify` otherwise).
    pub verify: Option<String>,
    /// What was actually run and seen — required to reach `closed`.
    pub evidence: Option<String>,
    /// Why it is stuck or postponed — required to reach `blocked` and
    /// `deferred`.
    pub note: Option<String>,
    /// External references (D10).
    pub references: References,
    /// Agent-set triage metadata (D11), never an ordering input.
    pub priority: Option<Priority>,
    /// Optional until-date (D9), pure metadata.
    pub until: Option<Date>,
}

/// The fields a `decision_add` supplies (§6, D10): everything a
/// [`crate::Decision`] holds except the identity the graph allocates and the
/// timestamps the caller passes separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionDraft {
    /// Short human title.
    pub title: String,
    /// Lifecycle status; `signed`/`superseded` require a resolution.
    pub status: DecisionStatus,
    /// The options the user chooses between, in authoring order.
    pub options: Vec<String>,
    /// The proposer's recommendation among the options.
    pub recommendation: Option<String>,
    /// How it was resolved.
    pub resolution: Option<String>,
    /// Proposer confidence at drafting time.
    pub confidence: Confidence,
    /// External references (D10).
    pub references: References,
}

/// The fields a `doc_register` supplies (§6, D10): everything [`crate::Doc`]
/// holds except the identity the graph allocates and the timestamps the
/// caller passes separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocDraft {
    /// Short human title.
    pub title: String,
    /// The mutable file path of the prose body.
    pub path: String,
    /// The document's version, an opaque string.
    pub version: String,
    /// External references (D10).
    pub references: References,
}
