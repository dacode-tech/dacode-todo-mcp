//! The five node kinds of spec §3: [`Repo`], [`Epic`], [`Task`],
//! [`Decision`], [`Doc`], with exactly the §3 fields — no more, no
//! fewer.
//!
//! Serde contract (§8): derives only, field order = declaration order,
//! no renaming (the §3 field names are the wire names), collections are
//! canonically ordered ([`References`]), and no floats anywhere.
//!
//! The closure discipline (§3, §6) is validation over the field
//! combination, not a type-level constraint — a [`Task`] struct can
//! *hold* an invalid combination so that `missing_*` errors (§6.1) can
//! name it; every construction and mutation path must pass
//! [`Task::validate`] / [`Decision::validate`] before the value lands
//! (the graph and server layers enforce that; the check order below is
//! fixed, so the same invalid node always yields the same error).
//!
//! Edges, graph invariants and selection are later M1 commits; the
//! structs here are the nodes those edges will connect.

use serde::{Deserialize, Serialize};

use crate::id::{DecisionId, DocId, EpicId, RepoSlug, TaskId};
use crate::reference::References;
use crate::status::{DecisionStatus, Status, TaskKind};
use crate::time::{Date, Timestamp};
use crate::{DomainError, Priority};

/// A repository registration (spec §3, §7): the immutable slug is the
/// key; the display name and filesystem path are mutable attributes —
/// checkouts move, the tracker must not care.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repo {
    /// The immutable slug key (D3).
    pub slug: RepoSlug,
    /// The mutable display name.
    pub name: String,
    /// The mutable filesystem path of the checkout (§7).
    pub path: String,
    /// When the repository was registered; caller-passed (§8).
    pub registered_at: Timestamp,
}

/// A repo's top-level grouping (spec §3): the selection unit every list
/// query may name. An epic may be an external issue (D10) — hence the
/// references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Epic {
    /// The epic id, e.g. `camera:E2`.
    pub id: EpicId,
    /// The repo this epic belongs to (every epic has exactly one, §5.1).
    pub repo: RepoSlug,
    /// Short human title.
    pub title: String,
    /// Longer prose description.
    pub description: String,
    /// Lifecycle status; epics carry no closure-discipline rules of
    /// their own — the discipline is the task's (§3).
    pub status: Status,
    /// External references (D10), canonically sorted.
    pub references: References,
}

/// A unit of tracked work (spec §3). Every task has exactly one epic as
/// its `parent` (§5.1); `until` is pure metadata (D9) — graph targets
/// ride `blocks` edges and selection reads no calendar (§8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// The task id, e.g. `camera:U7`.
    pub id: TaskId,
    /// The repo this task belongs to.
    pub repo: RepoSlug,
    /// The parent epic id (the `parent` edge of §5.1, as a field until
    /// the graph commit lands edges).
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
    /// What was actually run and seen — required to reach `closed`
    /// (`missing_evidence` otherwise).
    pub evidence: Option<String>,
    /// Why it is stuck or postponed — required to reach `blocked` and
    /// `deferred` (`missing_note` otherwise).
    pub note: Option<String>,
    /// External references (D10), canonically sorted.
    pub references: References,
    /// Agent-set triage metadata (D11) — display-only, never an
    /// ordering input.
    pub priority: Option<Priority>,
    /// Optional until-date (D9), pure metadata.
    pub until: Option<Date>,
    /// Creation time; caller-passed (§8).
    pub created: Timestamp,
    /// Last mutation time; caller-passed (§8).
    pub updated: Timestamp,
}

impl Task {
    /// Checks the closure discipline (§3, §6) over the current field
    /// combination, in fixed order: `verify` first (required at
    /// creation), then the status-gated `evidence`/`note` requirements.
    /// The id must already be allocated — it names the error payload.
    pub fn validate(&self) -> Result<(), DomainError> {
        let id = self.id.to_string();
        if !present(self.verify.as_deref()) {
            return Err(DomainError::MissingVerify { id });
        }
        match self.status {
            Status::Closed if !present(self.evidence.as_deref()) => {
                Err(DomainError::MissingEvidence { id })
            }
            s if s.requires_note() && !present(self.note.as_deref()) => {
                Err(DomainError::MissingNote { id })
            }
            _ => Ok(()),
        }
    }
}

/// A structured sign-off dossier (spec §3, D10). An open decision may
/// `block` tasks and epics (§5.1); signing flips the status and the
/// edges simply stop gating — no edge mutation, the history stays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    /// The decision id, e.g. `camera:D3`.
    pub id: DecisionId,
    /// The repo this decision belongs to (decisions attach directly to
    /// their repo, §5.1).
    pub repo: RepoSlug,
    /// Short human title.
    pub title: String,
    /// Lifecycle status; `signed`/`superseded` require a resolution.
    pub status: DecisionStatus,
    /// The options the user chooses between, in authoring order — an
    /// authored sequence, not a sorted set; the bytes are as stable as
    /// the state they describe.
    pub options: Vec<String>,
    /// The proposer's recommendation among the options.
    pub recommendation: Option<String>,
    /// How it was resolved — required to reach `signed`/`superseded`
    /// (`missing_resolution` otherwise).
    pub resolution: Option<String>,
    /// Proposer confidence at drafting time.
    pub confidence: Confidence,
    /// External references (D10), canonically sorted.
    pub references: References,
    /// Creation time; caller-passed (§8).
    pub created: Timestamp,
    /// Last mutation time; caller-passed (§8).
    pub updated: Timestamp,
}

impl Decision {
    /// Checks the decision closure discipline (§3): reaching `signed`
    /// or `superseded` without a `resolution` is an error. The id must
    /// already be allocated — it names the error payload.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.status.is_resolved() && !present(self.resolution.as_deref()) {
            return Err(DomainError::MissingResolution {
                id: self.id.to_string(),
            });
        }
        Ok(())
    }
}

/// Proposer confidence in a decision's recommendation. A small enum,
/// not a float or a free string: §8 forbids floats in persisted state,
/// and §3 does not pin the type — the three-point scale mirrors the
/// dossiers in `docs/implementation/decisions.md` this design learns
/// from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// The shape is a guess; expect rework.
    Low,
    /// Reasoned, with known open edges.
    Medium,
    /// As settled as a proposal can be.
    High,
}

impl Confidence {
    /// The stable wire spelling, e.g. `"medium"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A registered document (spec §3, D10): the stable address so
/// citations survive file moves and resolve from any repo. The prose
/// body lives in git; the path is a mutable attribute like a repo's,
/// updated by an audited mutation (`doc_set_path`, §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doc {
    /// The doc id, e.g. `camera:S1`.
    pub id: DocId,
    /// The repo this doc belongs to (docs attach directly to their
    /// repo, §5.1).
    pub repo: RepoSlug,
    /// Short human title.
    pub title: String,
    /// The mutable file path of the prose body.
    pub path: String,
    /// The document's version — an opaque string (a git tag, a header
    /// version, whatever the document itself declares); §3 does not pin
    /// a scheme.
    pub version: String,
    /// External references (D10), canonically sorted.
    pub references: References,
    /// Registration time; caller-passed (§8).
    pub created: Timestamp,
    /// Last mutation time; caller-passed (§8).
    pub updated: Timestamp,
}

/// The closure discipline's presence rule: a required string is present
/// when it carries non-whitespace content — an empty or blank field is
/// as missing as `None`, so `closed` with `evidence: Some("")` is still
/// `missing_evidence`.
fn present(value: Option<&str>) -> bool {
    value.is_some_and(|v| !v.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::{Reference, ReferenceKind};

    fn slug(s: &str) -> RepoSlug {
        s.parse().unwrap()
    }

    fn ts(s: &str) -> Timestamp {
        Timestamp::parse(s).unwrap()
    }

    fn task(
        status: Status,
        verify: Option<&str>,
        evidence: Option<&str>,
        note: Option<&str>,
    ) -> Task {
        Task {
            id: "camera:U7".parse().unwrap(),
            repo: slug("camera"),
            epic: "camera:E2".parse().unwrap(),
            kind: TaskKind::Task,
            title: "a task".to_owned(),
            body: String::new(),
            status,
            verify: verify.map(str::to_owned),
            evidence: evidence.map(str::to_owned),
            note: note.map(str::to_owned),
            references: References::new(),
            priority: None,
            until: None,
            created: ts("2026-09-26T12:00:00Z"),
            updated: ts("2026-09-26T12:00:00Z"),
        }
    }

    fn decision(status: DecisionStatus, resolution: Option<&str>) -> Decision {
        Decision {
            id: "camera:D3".parse().unwrap(),
            repo: slug("camera"),
            title: "a decision".to_owned(),
            status,
            options: vec!["(a) this".to_owned(), "(b) that".to_owned()],
            recommendation: Some("(a)".to_owned()),
            resolution: resolution.map(str::to_owned),
            confidence: Confidence::Medium,
            references: References::new(),
            created: ts("2026-09-25T09:00:00Z"),
            updated: ts("2026-09-25T10:00:00Z"),
        }
    }

    fn expect_code(result: Result<(), DomainError>, code: &str, id: &str) {
        let err = result.unwrap_err();
        assert_eq!(err.code().as_str(), code);
        assert_eq!(
            err,
            match code {
                "missing_verify" => DomainError::MissingVerify { id: id.to_owned() },
                "missing_evidence" => DomainError::MissingEvidence { id: id.to_owned() },
                "missing_note" => DomainError::MissingNote { id: id.to_owned() },
                "missing_resolution" => DomainError::MissingResolution { id: id.to_owned() },
                _ => unreachable!("test input"),
            }
        );
    }

    #[test]
    fn creation_without_verify_fires_missing_verify() {
        expect_code(
            task(Status::Open, None, None, None).validate(),
            "missing_verify",
            "camera:U7",
        );
        // Blank verify is as missing as none.
        expect_code(
            task(Status::Open, Some("   "), None, None).validate(),
            "missing_verify",
            "camera:U7",
        );
        // verify wins over the status rules, in fixed order.
        expect_code(
            task(Status::Closed, None, None, None).validate(),
            "missing_verify",
            "camera:U7",
        );
    }

    #[test]
    fn closed_without_evidence_fires_missing_evidence() {
        expect_code(
            task(Status::Closed, Some("cargo test"), None, None).validate(),
            "missing_evidence",
            "camera:U7",
        );
        expect_code(
            task(Status::Closed, Some("cargo test"), Some(""), None).validate(),
            "missing_evidence",
            "camera:U7",
        );
        assert!(
            task(Status::Closed, Some("cargo test"), Some("42 passed"), None)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn blocked_and_deferred_without_note_fire_missing_note() {
        expect_code(
            task(Status::Blocked, Some("cargo test"), None, None).validate(),
            "missing_note",
            "camera:U7",
        );
        expect_code(
            task(Status::Deferred, Some("cargo test"), None, None).validate(),
            "missing_note",
            "camera:U7",
        );
        expect_code(
            task(Status::Deferred, Some("cargo test"), None, Some("  ")).validate(),
            "missing_note",
            "camera:U7",
        );
        assert!(
            task(Status::Blocked, Some("cargo test"), None, Some("needs D12"))
                .validate()
                .is_ok()
        );
        assert!(
            task(Status::Deferred, Some("cargo test"), None, Some("after t9"))
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn open_statuses_need_nothing_but_verify() {
        assert!(
            task(Status::Open, Some("cargo test"), None, None)
                .validate()
                .is_ok()
        );
        assert!(
            task(Status::InWork, Some("cargo test"), None, None)
                .validate()
                .is_ok()
        );
        // wontfix is the terminal without evidence (§5.1).
        assert!(
            task(Status::Wontfix, Some("cargo test"), None, None)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn resolved_decisions_need_a_resolution() {
        expect_code(
            decision(DecisionStatus::Signed, None).validate(),
            "missing_resolution",
            "camera:D3",
        );
        expect_code(
            decision(DecisionStatus::Superseded, Some("")).validate(),
            "missing_resolution",
            "camera:D3",
        );
        assert!(
            decision(DecisionStatus::Signed, Some("option (a), user ok"))
                .validate()
                .is_ok()
        );
        assert!(
            decision(DecisionStatus::Superseded, Some("replaced by D14"))
                .validate()
                .is_ok()
        );
        assert!(decision(DecisionStatus::Open, None).validate().is_ok());
    }

    #[test]
    fn nodes_hold_exactly_the_spec_fields_with_references() {
        let mut references = References::new();
        references
            .insert(
                Reference::new(
                    ReferenceKind::Tracker,
                    "CAM-42".to_owned(),
                    Some("the external issue".to_owned()),
                )
                .unwrap(),
            )
            .unwrap();
        let epic = Epic {
            id: "camera:E2".parse().unwrap(),
            repo: slug("camera"),
            title: "an epic".to_owned(),
            description: "what it is about".to_owned(),
            status: Status::Open,
            references: references.clone(),
        };
        assert_eq!(epic.references.len(), 1);

        let repo = Repo {
            slug: slug("camera"),
            name: "Camera".to_owned(),
            path: "/srv/repos/camera".to_owned(),
            registered_at: ts("2026-09-25T08:00:00Z"),
        };
        assert_eq!(repo.slug.as_str(), "camera");

        let mut t = task(Status::Deferred, Some("cargo test"), None, Some("until M2"));
        t.references = references.clone();
        t.priority = Some(Priority::High);
        t.until = Some(Date::parse("2026-10-01").unwrap());
        assert!(t.validate().is_ok());

        let doc = Doc {
            id: "camera:S1".parse().unwrap(),
            repo: slug("camera"),
            title: "the design".to_owned(),
            path: "docs/design.md".to_owned(),
            version: "1.4".to_owned(),
            references: references.clone(),
            created: ts("2026-09-25T08:00:00Z"),
            updated: ts("2026-09-26T08:00:00Z"),
        };
        assert_eq!(doc.version, "1.4");
        assert_eq!(doc.references.len(), 1);
        assert_eq!(Confidence::Medium.to_string(), "medium");
    }
}
