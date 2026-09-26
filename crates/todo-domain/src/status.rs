//! Status and kind vocabularies (spec §3, D9/D10/D11) and the
//! predicates the closure discipline is built from.
//!
//! Serde spelling is `snake_case` for every enum here — the wire
//! vocabulary of §3 (`in_work`, never `InWork` or `in-work`). The
//! closure-discipline rules themselves live on the node types
//! ([`crate::Task::validate`], [`crate::Decision::validate`]) because
//! they need node fields; the predicates below are what those checks
//! read, so the rule table has one home.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The lifecycle status of an epic or task (D9): renamed from the
/// `tools/tasks.py` set the culture comes from (pending→open,
/// active→in_work, done→closed) plus `deferred`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Not started; actionable if the graph says so (§5.2).
    Open,
    /// Being worked on; actionable.
    InWork,
    /// Externally stuck; never actionable, requires a note.
    Blocked,
    /// Deliberately postponed (D9); never actionable, requires a note.
    /// Nothing reactivates it by calendar or by an edge completing —
    /// reopening is an explicit update.
    Deferred,
    /// Finished with evidence of completed work.
    Closed,
    /// The one terminal that unblocks successors without evidence of
    /// completed work (§5.1).
    Wontfix,
}

impl Status {
    /// The stable wire spelling, e.g. `"in_work"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InWork => "in_work",
            Self::Blocked => "blocked",
            Self::Deferred => "deferred",
            Self::Closed => "closed",
            Self::Wontfix => "wontfix",
        }
    }

    /// Whether this status finishes a `blocks` predecessor (§5.1):
    /// `closed` or `wontfix`.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Closed | Self::Wontfix)
    }

    /// Whether reaching this status requires a `note` (§3's closure
    /// discipline): `blocked` or `deferred`.
    pub fn requires_note(self) -> bool {
        matches!(self, Self::Blocked | Self::Deferred)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What kind of work a task node records (§3): the sibling projects'
/// vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// One unit of work.
    Task,
    /// A checkpoint that must pass.
    Gate,
    /// Discovered along the way, not the point of the task.
    Finding,
    /// A deliberate shortcut, recorded rather than forgotten.
    Debt,
}

impl TaskKind {
    /// The stable wire spelling, e.g. `"finding"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Gate => "gate",
            Self::Finding => "finding",
            Self::Debt => "debt",
        }
    }
}

impl fmt::Display for TaskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The lifecycle status of a decision dossier (D10). Transitions mirror
/// task closure: reaching `signed` or `superseded` without a
/// `resolution` is a `missing_resolution` error (§6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    /// Awaiting the user's sign-off; may `block` tasks and epics.
    Open,
    /// Signed off; its `blocks` edges simply stop gating (§5.1).
    Signed,
    /// Replaced by a later decision; stops gating like `signed`.
    Superseded,
}

impl DecisionStatus {
    /// The stable wire spelling, e.g. `"signed"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Signed => "signed",
            Self::Superseded => "superseded",
        }
    }

    /// Whether this status finishes a decision `blocks` predecessor
    /// (§5.1): `signed` or `superseded`.
    pub fn is_resolved(self) -> bool {
        matches!(self, Self::Signed | Self::Superseded)
    }
}

impl fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Agent-set triage metadata on tasks (D11), echoed in payloads.
/// Display-only: priority never enters ordering or selection (§5.2,
/// §11) — promoting it into the tiebreaker would be a new decision
/// amending D6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Triage: look at this first.
    High,
    /// Triage: normal attention.
    Medium,
    /// Triage: look at this last.
    Low,
}

impl Priority {
    /// The stable wire spelling, e.g. `"high"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_spellings() {
        assert_eq!(Status::Open.as_str(), "open");
        assert_eq!(Status::InWork.as_str(), "in_work");
        assert_eq!(Status::Blocked.as_str(), "blocked");
        assert_eq!(Status::Deferred.as_str(), "deferred");
        assert_eq!(Status::Closed.as_str(), "closed");
        assert_eq!(Status::Wontfix.as_str(), "wontfix");
        assert_eq!(Status::InWork.to_string(), "in_work");
        assert_eq!(TaskKind::Task.as_str(), "task");
        assert_eq!(TaskKind::Gate.as_str(), "gate");
        assert_eq!(TaskKind::Finding.as_str(), "finding");
        assert_eq!(TaskKind::Debt.as_str(), "debt");
        assert_eq!(DecisionStatus::Open.as_str(), "open");
        assert_eq!(DecisionStatus::Signed.as_str(), "signed");
        assert_eq!(DecisionStatus::Superseded.as_str(), "superseded");
        assert_eq!(Priority::High.as_str(), "high");
        assert_eq!(Priority::Medium.as_str(), "medium");
        assert_eq!(Priority::Low.as_str(), "low");
    }

    #[test]
    fn terminal_and_actionability_predicates() {
        // §5.1: a predecessor is finished at closed/wontfix.
        assert!(Status::Closed.is_terminal());
        assert!(Status::Wontfix.is_terminal());
        for s in [
            Status::Open,
            Status::InWork,
            Status::Blocked,
            Status::Deferred,
        ] {
            assert!(!s.is_terminal(), "{s} is not terminal");
        }
    }

    #[test]
    fn note_requirements() {
        assert!(Status::Blocked.requires_note());
        assert!(Status::Deferred.requires_note());
        for s in [
            Status::Open,
            Status::InWork,
            Status::Closed,
            Status::Wontfix,
        ] {
            assert!(!s.requires_note(), "{s} requires no note");
        }
    }

    #[test]
    fn decision_resolution_predicates() {
        assert!(DecisionStatus::Signed.is_resolved());
        assert!(DecisionStatus::Superseded.is_resolved());
        assert!(!DecisionStatus::Open.is_resolved());
    }
}
