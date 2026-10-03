//! D16's roll-up (spec §6): the `status_summary` aggregate — counts
//! by kind and status for a scope, the non-terminal "open" totals and
//! the deferred summary — and the annotated row type `task_list`
//! answers with, whose blocked-by rule is lifted here out of `items`'
//! private annotate so the contract has one home.
//!
//! **The roll-up and the row only.** The selection surfaces are
//! `select`/`items`, the listing reads `list`; this module adds the
//! aggregate D16 signed and hosts the blocked-by rule both surfaces'
//! rows carry — `items`'s `annotate` and `list`'s row walk call
//! [`Graph::blockers_of`] rather than restating it, so the two-run
//! order cannot drift between the batch and the listing.
//!
//! # The rules
//!
//! The arithmetic is §6's signed text, not this module's opinion:
//! every epic and task of the scope is counted by its [`Status`],
//! every decision by its [`DecisionStatus`], and docs — which carry
//! no status — as one total. "Open" in any total means *non-terminal*:
//! for epics and tasks `open`, `in_work`, `blocked` and `deferred`
//! (terminal: `closed`, `wontfix`); for decisions `open` (terminal:
//! `signed`, `superseded`). The open totals are carried once, at the
//! summary level, and computed from the counts — one source, so the
//! per-status fields and the totals cannot disagree.
//!
//! The deferred summary is [`DeferredSummary`], reused verbatim, and
//! counted over the scope's derived task list — the same list
//! [`Graph::next_task`] and [`Graph::next_items`] walk — with the same
//! [`Graph::is_until_satisfied`] predicate, so the summary and the
//! selection tools cannot disagree about what "ready" means; the
//! pinned agreement lives in `graph/tests/summary.rs`.
//!
//! The repo scope holds every node of the graph. An epic scope holds
//! the epic itself — counted once, at its own status — its tasks (the
//! induced order's set, the same list [`Graph::next_task_of_epic`]
//! walks), and every decision holding any of them back: both the
//! `decision→epic` edges and the `decision→task` edges into the
//! epic's tasks (judgment call (d)). Docs are `0`: a doc has no epic
//! field, so no epic scope holds one.
//!
//! The row is exactly the signed payload: the task, its `blocked_by`
//! ids, every blocker's status — and nothing else. The blocked-by
//! order is the contract [`crate::NextItem::blocked_by`] has always
//! carried: the task's own predecessors first, then its epic's
//! decision gates, each run in [`BlockEnd`] order, deduplicated at
//! the first occurrence and deliberately **not** merged into one
//! globally sorted list.
//!
//! # Scope and totality
//!
//! The repo-scoped entry point is infallible; the epic-scoped one has
//! exactly one failure, `unknown_id` for an epic this graph does not
//! hold, propagated from the order derivation. [`Graph::blockers_of`]
//! is total: a task this graph does not hold answers an empty list.
//!
//! The roll-up is a pure function of graph state (§8): no new field,
//! no cache, no memoisation — every call recomputes, which is why
//! nothing here can go stale. One query costs one order derivation,
//! one counting pass per kind and, per returned row, one blocked-by
//! walk — inside §4.1's scale argument.
//!
//! # Judgment calls (t32)
//!
//! - **(a) Two entry points, no scope enum.**
//!   [`Graph::status_summary`] answers the repo scope, infallibly;
//!   [`Graph::status_summary_of_epic`] answers one epic's scope, with
//!   `unknown_id` propagated from [`Graph::canonical_order_of_epic`]
//!   as its only failure — the t27/t28 precedent. The repo dimension
//!   of §6's `status_summary(repo?, epic?)` is M3 composing per-graph
//!   summaries over the handles it holds; a repo parameter here would
//!   be a §2 boundary defect. The answer types are re-exported
//!   through `graph/mod.rs` and the crate root the way
//!   [`crate::NextTask`] is.
//! - **(b) Counts are named-field structs, every status always
//!   carried, no maps.** Zeroes are carried, so two scopes with
//!   different status mixes still compare field-for-field; M3's
//!   `outputSchema` gets a field list, not a map schema; no
//!   map-iteration question exists. A `BTreeMap<Status, u64>` is
//!   rejected: a weaker schema, absent-zero ambiguity, and D14's
//!   declaration-order pin has nothing to pin on a map.
//! - **(c) The open totals live at the summary level, not on each
//!   counts struct — veto-able.** The alternative — an `open_total`
//!   field on each counts struct — is a named diff with identical
//!   arithmetic; the top-level block is chosen because the signed
//!   text treats "Open" as a summary-level notion and the headline a
//!   session opens with reads in one place. No `docs` field inside
//!   [`OpenTotals`]: a doc carries no status, so it has no open half
//!   to total.
//! - **(d) Epic scope membership — veto-able, the one genuinely open
//!   reading.** An epic scope holds the epic itself, its tasks, and
//!   every decision holding any of them back — both `decision→epic`
//!   edges and `decision→task` edges into the epic's tasks; docs `0`.
//!   The wider gate reading keeps the summary consistent with
//!   [`Graph::is_until_satisfied`]'s consultation set (own edges +
//!   epic gates) — the same consistency argument `items`'s judgment
//!   call (g) used for `blocked_by`; a summary reporting zero open
//!   decisions while every task in the scope is held by an open
//!   `decision→task` gate would contradict the selection machinery.
//!   The alternatives — §6.2-literal (`decision→epic` edges only) and
//!   tasks-only (epics and decisions both zero) — are each a named
//!   diff to the scope-collection walk;
//!   `summary_epic_scope_membership` pins the reading taken, so a
//!   veto is a diff, not a discovery.
//! - **(e) The row payload is exactly the signed text.** The task, its
//!   `blocked_by` ids, every blocker's status — and nothing else: no
//!   own status, no kind, no weight, no actionable mark. A field
//!   beyond the signed text is a wire change M3 would have to declare
//!   (`items`'s judgment call (h)'s class). The `Epic` arm of
//!   [`Blocker`] is unrepresentable by type — §5.1's three edge
//!   shapes make a `blocks` source a task or a decision only — the
//!   `items` judgment-call-(d) narrowing, not repeated as a runtime
//!   arm.
//! - **(f) The annotate lift.** The `blocked_by`-building half of
//!   `items`'s private `annotate` is [`Graph::blockers_of`] here — the
//!   contract's one home; `items`'s `annotate` and `list`'s row walk
//!   call it, and two homes for one contract is drift. The unknown-id
//!   arm (task lookup `None` → skip the gate half) stays, documented
//!   unreachable from the row walks — the `items`'s judgment-call-(d)
//!   discipline: a state-space argument in the doc, no forged test.
//! - **(g) No serde on any new type.** A query result is not
//!   persisted and the consumer owns its wire shape —
//!   [`crate::NextTask`], [`crate::NextItems`] and
//!   [`crate::TaskFilter`] are the precedent.
//! - **(h) Test layout.** `graph/tests/summary.rs`, every name
//!   prefixed `summary_` so the module filter selects exactly that
//!   suite; the naive recomputations are longhand in the test file and
//!   never call production, and the shared `naive_*` helpers in
//!   `tests/mod.rs` are reused through the existing glob import.

use std::collections::BTreeSet;

use crate::error::DomainError;
use crate::id::{DecisionId, EpicId, TaskId};
use crate::status::{DecisionStatus, Status};

use super::edges::BlockEnd;
use super::select::DeferredSummary;
use super::state::Graph;

/// The counts of one node kind by [`Status`] — the epic and the task
/// halves of a [`StatusSummary`]. Every status is carried, zeroes
/// included, so two scopes with different status mixes still compare
/// field-for-field and a consumer's schema is a field list, not a map
/// (judgment call (b)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCounts {
    /// Nodes at [`Status::Open`].
    pub open: u64,
    /// Nodes at [`Status::InWork`].
    pub in_work: u64,
    /// Nodes at [`Status::Blocked`].
    pub blocked: u64,
    /// Nodes at [`Status::Deferred`].
    pub deferred: u64,
    /// Nodes at [`Status::Closed`].
    pub closed: u64,
    /// Nodes at [`Status::Wontfix`].
    pub wontfix: u64,
}

impl StatusCounts {
    /// The zeroed counts — the seed every counting pass starts from.
    fn zeroed() -> Self {
        Self {
            open: 0,
            in_work: 0,
            blocked: 0,
            deferred: 0,
            closed: 0,
            wontfix: 0,
        }
    }

    /// Counts one node at `status`.
    fn note(&mut self, status: Status) {
        match status {
            Status::Open => self.open += 1,
            Status::InWork => self.in_work += 1,
            Status::Blocked => self.blocked += 1,
            Status::Deferred => self.deferred += 1,
            Status::Closed => self.closed += 1,
            Status::Wontfix => self.wontfix += 1,
        }
    }

    /// The non-terminal sum — §6's "Open" for a kind counted by
    /// [`Status`]: `closed` and `wontfix` stay out.
    fn open_total(&self) -> u64 {
        self.open + self.in_work + self.blocked + self.deferred
    }
}

/// The counts of decisions by [`DecisionStatus`] — the decision half
/// of a [`StatusSummary`]. Every status is carried, zeroes included
/// (see [`StatusCounts`]'s rationale).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionCounts {
    /// Decisions at [`DecisionStatus::Open`] — the kind's only
    /// non-terminal status.
    pub open: u64,
    /// Decisions at [`DecisionStatus::Signed`].
    pub signed: u64,
    /// Decisions at [`DecisionStatus::Superseded`].
    pub superseded: u64,
}

impl DecisionCounts {
    /// The zeroed counts — the seed every counting pass starts from.
    fn zeroed() -> Self {
        Self {
            open: 0,
            signed: 0,
            superseded: 0,
        }
    }

    /// Counts one decision at `status`.
    fn note(&mut self, status: DecisionStatus) {
        match status {
            DecisionStatus::Open => self.open += 1,
            DecisionStatus::Signed => self.signed += 1,
            DecisionStatus::Superseded => self.superseded += 1,
        }
    }
}

/// The non-terminal totals per kind (§6: "'Open' in any total means
/// *non-terminal*"), carried once at the summary level rather than
/// repeated per counts struct — the headline a session opens with
/// reads in one place (judgment call (c)). No `docs` field: a doc
/// carries no status, so it has no open half to total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenTotals {
    /// Non-terminal epics: `open` + `in_work` + `blocked` + `deferred`.
    pub epics: u64,
    /// Non-terminal tasks, the same four statuses.
    pub tasks: u64,
    /// Open decisions — the kind's only non-terminal status.
    pub decisions: u64,
}

/// The answer of a `status_summary` query (§6, D16): the scope's
/// counts by kind and status, the non-terminal open totals and the
/// deferred summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSummary {
    /// Every epic of the scope, by status. Repo scope: all of them;
    /// epic scope: the epic itself, once, at its own status.
    pub epics: StatusCounts,
    /// Every task of the scope, by status — the scope's derived task
    /// list, the same list the selection tools walk.
    pub tasks: StatusCounts,
    /// Every decision of the scope, by status. Repo scope: all of
    /// them; epic scope: the decisions gating the epic or its tasks
    /// (judgment call (d)).
    pub decisions: DecisionCounts,
    /// Docs of the scope, one total — a doc carries no status. Repo
    /// scope: all of them; epic scope: `0`.
    pub docs: u64,
    /// The non-terminal totals per kind, computed from the counts.
    pub open: OpenTotals,
    /// The deferred summary over the scope's task list — the same
    /// counts [`Graph::next_task`] reports for the same scope.
    pub deferred: DeferredSummary,
}

impl StatusSummary {
    /// The zeroed summary — the seed both counting passes start from.
    fn zeroed() -> Self {
        Self {
            epics: StatusCounts::zeroed(),
            tasks: StatusCounts::zeroed(),
            decisions: DecisionCounts::zeroed(),
            docs: 0,
            open: OpenTotals {
                epics: 0,
                tasks: 0,
                decisions: 0,
            },
            deferred: DeferredSummary { total: 0, ready: 0 },
        }
    }

    /// The open totals, derived from the counts — one source, so the
    /// per-status fields and the totals cannot disagree.
    fn open_totals(&self) -> OpenTotals {
        OpenTotals {
            epics: self.epics.open_total(),
            tasks: self.tasks.open_total(),
            decisions: self.decisions.open,
        }
    }
}

/// One `task_list` row (§6, D16): the task and its `blocked_by`
/// annotation. Exactly the signed payload — the task, its blocker
/// ids, every blocker's status — and nothing else: no own status, no
/// kind, no weight, no actionable mark; a field beyond the signed text
/// is a wire change M3 would have to declare (judgment call (e)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRow {
    /// The task this row is about.
    pub task: TaskId,
    /// What holds the task back, per [`Graph::blockers_of`]: its own
    /// predecessors, then its epic's decision gates — two ascending
    /// runs, deduplicated at the first occurrence — each blocker
    /// carrying its current status.
    pub blocked_by: Vec<Blocker>,
}

/// One blocker of a [`TaskRow`]: the id and the **current** status of
/// a node holding the row's task back. §5.1's three edge shapes make a
/// `blocks` source a task or a decision only, so no `Epic` variant
/// exists — the narrowing is total, not a dropped runtime arm
/// (judgment call (e)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Blocker {
    /// A task predecessor, at its current status.
    Task {
        /// The predecessor's id.
        id: TaskId,
        /// The predecessor's current status.
        status: Status,
    },
    /// A decision gate — on the task itself or on its epic — at its
    /// current status.
    Decision {
        /// The decision's id.
        id: DecisionId,
        /// The decision's current status.
        status: DecisionStatus,
    },
}

/// The identity half of a [`Blocker`], for the surface that lists
/// blockers without their statuses: [`crate::NextItem::blocked_by`]
/// converts through this.
impl From<Blocker> for BlockEnd {
    fn from(blocker: Blocker) -> Self {
        match blocker {
            Blocker::Task { id, .. } => BlockEnd::Task(id),
            Blocker::Decision { id, .. } => BlockEnd::Decision(id),
        }
    }
}

impl Graph {
    /// The roll-up of this repository (§6's `status_summary`, D16):
    /// every epic and task counted by its [`Status`], every decision
    /// by its [`DecisionStatus`], docs as one total, the non-terminal
    /// open totals and the deferred summary — the arithmetic is §6's
    /// signed text (see the module docs).
    ///
    /// A total function: an empty graph answers all zeroes. Reads
    /// graph state alone (§8), so the same state always yields the
    /// same [`StatusSummary`].
    pub fn status_summary(&self) -> StatusSummary {
        let order = self.canonical_order();
        let mut summary = StatusSummary::zeroed();
        for epic in self.epics() {
            summary.epics.note(epic.status);
        }
        for task in order.iter().filter_map(|id| self.task(id)) {
            summary.tasks.note(task.status);
        }
        for decision in self.decisions() {
            summary.decisions.note(decision.status);
        }
        summary.docs = self.docs().count() as u64;
        summary.deferred = self.deferred_of(&order);
        summary.open = summary.open_totals();
        summary
    }

    /// The roll-up of one epic's scope (§6's `status_summary` with an
    /// epic, D16): the epic itself counted once at its own status, its
    /// tasks — the induced order's set, the same list
    /// [`Graph::next_task_of_epic`] walks — and every decision gating
    /// the epic or one of its tasks (judgment call (d)); docs are `0`.
    ///
    /// An epic this graph does not hold is `unknown_id`, propagated
    /// from the order derivation and the only failure this method
    /// has. Reads graph state alone (§8), so the same state always
    /// yields the same [`StatusSummary`].
    pub fn status_summary_of_epic(&self, epic: &EpicId) -> Result<StatusSummary, DomainError> {
        let order = self.canonical_order_of_epic(epic)?;
        let mut summary = StatusSummary::zeroed();
        // The None arm is unreachable: the derivation above already
        // rejected an epic this graph does not hold.
        if let Some(node) = self.epic(epic) {
            summary.epics.note(node.status);
        }
        for task in order.iter().filter_map(|id| self.task(id)) {
            summary.tasks.note(task.status);
        }
        for id in self.gating_decisions(epic, &order) {
            // Same unreachability: every id in the gate set came off an
            // adjacency list, whose edges require live endpoints.
            if let Some(decision) = self.decision(&id) {
                summary.decisions.note(decision.status);
            }
        }
        summary.deferred = self.deferred_of(&order);
        summary.open = summary.open_totals();
        Ok(summary)
    }

    /// The decisions gating one epic's scope (§6.2): every decision
    /// with an edge into the epic or into one of its tasks, ascending
    /// — the same set [`Graph::status_summary_of_epic`] counts,
    /// exposed as the ids themselves because the wire's epic-scoped
    /// dump lists exactly these nodes; one rule, one home, so the
    /// dump and the roll-up cannot drift about what gates a scope.
    ///
    /// An epic this graph does not hold is `unknown_id`, propagated
    /// from the order derivation and the only failure this method
    /// has. Reads graph state alone (§8), so the same state always
    /// yields the same vec.
    pub fn gating_decisions_of_epic(&self, epic: &EpicId) -> Result<Vec<DecisionId>, DomainError> {
        let order = self.canonical_order_of_epic(epic)?;
        Ok(self.gating_decisions(epic, &order).into_iter().collect())
    }

    /// What holds `task` back (§6's D16 annotation, the same contract
    /// [`crate::NextItem::blocked_by`] carries): the task's own
    /// predecessors, then its epic's decision gates — each run in
    /// [`BlockEnd`] order, deduplicated at the first occurrence, and
    /// deliberately **not** merged into one globally sorted list; a
    /// decision gating both the task and its epic appears once, in the
    /// own-edges run. Each blocker carries its current status, so a
    /// mutation moves the row.
    ///
    /// This is the rule's one home (judgment call (f)): `items`'s row
    /// walk and `list`'s row conversion both call it, and a consumer
    /// of the bytes — M3's `outputSchema` included — must describe the
    /// field as two ascending runs, not one sorted list.
    ///
    /// Total: a task this graph does not hold answers an empty list.
    /// The missing-task arm below — which skips the epic-gate half
    /// rather than inventing one — is unreachable from the row walks
    /// for the reason `items`'s judgment call (d) records: every
    /// id a row walk annotates comes from a scope list that holds only
    /// ids of tasks this graph has.
    pub fn blockers_of(&self, task: &TaskId) -> Vec<Blocker> {
        let mut ends = self.blocked_by(&BlockEnd::Task(task.clone()));
        if let Some(node) = self.task(task) {
            for gate in self.blocked_by(&BlockEnd::Epic(node.epic.clone())) {
                if !ends.contains(&gate) {
                    ends.push(gate);
                }
            }
        }
        ends.into_iter()
            .filter_map(|end| self.blocker(end))
            .collect()
    }

    /// One predecessor endpoint as a [`Blocker`], its status resolved
    /// from the graph. The `Epic` arm cannot occur — a `blocks` source
    /// is a task or a decision only (§5.1) — and a missing node cannot
    /// occur either: [`Graph::link_blocks`] requires both endpoints to
    /// exist and every deletion cascades its edges away. Both
    /// fallbacks drop the entry rather than inventing a status.
    fn blocker(&self, end: BlockEnd) -> Option<Blocker> {
        match end {
            BlockEnd::Task(id) => self.task(&id).map(|node| Blocker::Task {
                id,
                status: node.status,
            }),
            BlockEnd::Decision(id) => self.decision(&id).map(|node| Blocker::Decision {
                id,
                status: node.status,
            }),
            BlockEnd::Epic(_) => None,
        }
    }

    /// The decisions gating one epic scope: every decision with an
    /// edge into the epic or into one of its tasks, deduplicated — a
    /// decision gating both the epic and a task is one node. The epic
    /// arm of the first narrowing cannot occur (only decisions target
    /// an epic, §5.1); the task arm of the second is the point — a
    /// task predecessor of one of the epic's tasks is not a gate of
    /// the scope.
    fn gating_decisions(&self, epic: &EpicId, tasks: &[TaskId]) -> BTreeSet<DecisionId> {
        let mut gating = BTreeSet::new();
        for end in self.blocked_by(&BlockEnd::Epic(epic.clone())) {
            if let BlockEnd::Decision(id) = end {
                gating.insert(id);
            }
        }
        for id in tasks {
            for end in self.blocked_by(&BlockEnd::Task(id.clone())) {
                if let BlockEnd::Decision(id) = end {
                    gating.insert(id);
                }
            }
        }
        gating
    }

    /// The deferred summary over one scope's derived list — the same
    /// list the selection tools walk, so scope has one definition and
    /// the agreement with [`Graph::next_task`] is pinnable by type
    /// equality. The readiness half is [`Graph::is_until_satisfied`],
    /// the same predicate the selection tools apply; the pass itself
    /// is this module's own, the way `items`'s judgment call (i)
    /// records — the agreement is pinned over the seeded graphs, not
    /// trusted as an argument.
    fn deferred_of(&self, scope: &[TaskId]) -> DeferredSummary {
        let mut deferred = DeferredSummary { total: 0, ready: 0 };
        for task in scope.iter().filter_map(|id| self.task(id)) {
            if task.status != Status::Deferred {
                continue;
            }
            deferred.total += 1;
            deferred.ready += u64::from(self.is_until_satisfied(&task.id));
        }
        deferred
    }
}
