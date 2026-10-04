//! Selection (spec §5.2): the actionability predicate, the
//! until-target ("ready") predicate, the deferred `{total, ready}`
//! summary and the two `next_task` entry points — repo scope and epic
//! scope.
//!
//! **Selection only.** The batch surface — `next_items` with its
//! annotations, derived weight and task scope — is `items`; the listing
//! reads, `task_list`'s filters and the three thin listings, are `list`.
//! Both call this module's predicates rather than restating them, and
//! `items` carries its [`DeferredSummary`] too; nothing of either is
//! pre-built here, the same way [`Graph::canonical_order`] pre-built
//! nothing of this module.
//!
//! # The rules
//!
//! A task is **actionable** when its status is `open` or `in_work` and
//! every predecessor holding it back is finished — a task predecessor
//! at `closed`/`wontfix` ([`Status::is_terminal`]), a decision one at
//! `signed`/`superseded`
//! ([`crate::DecisionStatus::is_resolved`]). `blocked` and `deferred`
//! items are never actionable whatever their edges say.
//!
//! The predecessors consulted are **two** adjacency lists: the task's
//! own incoming `blocks` edges — a deferred item's until-targets (§3) —
//! *and* its epic's incoming decision gates, which is §5.1's "an open
//! decision blocking an epic holds all work under it". That the epic
//! gate also feeds the ready count is a reading of §5.2's summary
//! sentence ("every until-target `blocks` predecessor"), taken so the
//! two predicates cannot disagree: a readiness that ignored the gate
//! would report a deferred task under a gated epic as worth revisiting
//! now, while actionability refuses it the moment it is reopened.
//!
//! The epic node's own *status* never enters: an epic has no closure
//! discipline of its own (§3) and §5.1's gating mechanism is the edge,
//! so tasks under a `closed`, `blocked` or `deferred` epic are still
//! actionable. Nor do `until` — metadata, and selection reads no
//! calendar (§3, §8) — the agent-set `priority` — display-only (D11,
//! §11), echoed on the answer (D17) and never an ordering input, since
//! promoting it would amend the signed D6 tiebreaker — or `relates`, an
//! annotation with no ordering semantics (§5.1).
//!
//! "Nothing is actionable" is `task: None`, not an error: §6.1's code
//! list is closed and holds no code for it. The deferred summary rides
//! both entry points and both scopes unconditionally, including on a
//! `None` answer. Its unit is tasks: an epic can be `deferred` (§3
//! shares [`Status`]), but the selection unit is a task and an epic's
//! deferral is visible through its own status — a reading of §5.2's
//! word "items", taken with the epic-gate one above.
//!
//! # Scope and totality
//!
//! The epic-scoped entry point walks [`Graph::canonical_order_of_epic`]
//! — the induced order — but tests each id against the **full** graph,
//! which is the contract the order documents: a cross-epic predecessor
//! contributes no in-degree to the scoped list and still holds its
//! successor back here. The consequence is deliberate: the epic-scoped
//! answer is the induced order's first actionable task, which need not
//! be the repo-scoped answer restricted to the epic. Both entry points
//! and the summary read the same derived list, so answer and summary
//! cannot disagree about what is in scope.
//!
//! Both predicates are total: an id this graph does not hold answers
//! `false` — an unknown task is never selected and never counted ready
//! — matching the crate's total-read precedent; `unknown_id` stays the
//! mutating APIs' and the scoped queries' job. The repo-scoped entry
//! point is infallible; the epic-scoped one has exactly that inherited
//! `unknown_id` failure.
//!
//! Selection is a pure function of graph state (§8): no new field, no
//! cache, no memoisation — every call recomputes, which is why nothing
//! here can go stale. One query costs one order derivation plus an
//! answer pass that stops at its first hit and a summary pass, inside
//! §4.1's scale argument.
//!
//! # Judgment calls (t27)
//!
//! - **(a) Placement and wiring.** `impl Graph` methods in this module,
//!   mirroring `order.rs`; the two answer types are re-exported through
//!   `graph/mod.rs` and the crate root. The predicates add no new
//!   crate-root name — they are [`Graph`] methods.
//! - **(b) Internal shape.** One private `finished` helper answers the
//!   per-predecessor question behind both public predicates, so
//!   "finished" has a single implementation and actionability and
//!   readiness cannot drift apart. Each entry point makes two passes
//!   over the derived list — the answer pass stops at its first hit,
//!   which a fused pass could not.
//! - **(c) The crate-root surface paragraph.** `lib.rs` named
//!   `next_task` as landed here and the rest of M2's query surface as
//!   remaining; `items` and then `list` landed beside this module and the
//!   paragraph names all three, so it now states the crate's whole query
//!   surface rather than what remains of M2's.
//! - **(d) Unclassifiable predecessors.** `finished` answers "not
//!   finished" for a [`BlockEnd::Epic`] predecessor and for a
//!   predecessor id this graph does not hold. Neither is reachable
//!   through the public API: [`crate::BlocksEdge::source`] has exactly
//!   two arms, `Task` and `Decision`, so [`Graph::blocked_by`] can
//!   never yield an epic end, and [`Graph::link_blocks`] requires both
//!   endpoints to exist while [`Graph::delete_epic`]'s cascade removes
//!   the edges with its nodes. Both fallbacks hold work back rather
//!   than releasing it; neither panics, and no test forges one — a
//!   state the public API cannot construct cannot be tested, only
//!   documented.
//! - **(e) Test layout.** `graph/tests/select.rs`, every name prefixed
//!   `select_` so the module filter selects exactly that suite.

use crate::error::DomainError;
use crate::id::{EpicId, TaskId};
use crate::status::{Priority, Status};

use super::edges::BlockEnd;
use super::state::Graph;

/// The answer of a `next_task` query (§5.2): the scope's first
/// actionable task, and the scope's deferred summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextTask {
    /// The first actionable task in the queried scope's canonical
    /// order, or `None` when the scope holds none — an answer, not an
    /// error (§6.1's code list is closed and has no code for it).
    pub task: Option<TaskId>,
    /// The chosen task's agent-set `priority` (D17), read off the node
    /// and echoed beside its id — display-only, never an ordering
    /// input (D11, §11). `None` when the answer is `None` or the task
    /// carries no priority.
    pub priority: Option<Priority>,
    /// The deferred summary of the queried scope, present
    /// unconditionally — including on a `None` answer.
    pub deferred: DeferredSummary,
}

/// The deferred summary of a queried scope (§5.2's D9 amendment): how
/// many deferred tasks the scope holds and how many are worth revisiting
/// now. The summary informs; nothing reopens automatically — reopening a
/// ready item is an explicit update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredSummary {
    /// Deferred tasks in the scope. The unit is tasks: a `deferred`
    /// epic is not counted (see the module docs).
    pub total: u64,
    /// Of those, the tasks whose every predecessor — their own
    /// until-target edges and their epic's decision gates — is
    /// finished.
    pub ready: u64,
}

impl Graph {
    /// The next actionable task of this repository (§5.2): the first
    /// node in [`Graph::canonical_order`] that
    /// [`Graph::is_actionable`] accepts, plus the deferred summary of
    /// the whole repo scope.
    ///
    /// A total function: "nothing is actionable" is `task: None`, and
    /// the summary rides the answer unconditionally. Selection reads
    /// graph state alone (§8) — no clock, no stored order, no cache —
    /// so the same state always yields the same [`NextTask`].
    pub fn next_task(&self) -> NextTask {
        self.select_from(&self.canonical_order())
    }

    /// The next actionable task of one epic (§5.2): the first node in
    /// [`Graph::canonical_order_of_epic`] that
    /// [`Graph::is_actionable`] accepts, plus the deferred summary of
    /// that epic's scope.
    ///
    /// The induced order, tested against the full graph — the contract
    /// the scoped order documents: a cross-epic predecessor contributes
    /// no in-degree to the list and still holds its successor back
    /// here. An epic this graph does not hold is `unknown_id`,
    /// propagated from the order derivation and the only failure this
    /// method has.
    pub fn next_task_of_epic(&self, epic: &EpicId) -> Result<NextTask, DomainError> {
        Ok(self.select_from(&self.canonical_order_of_epic(epic)?))
    }

    /// Whether a task may be picked up now (§5.2): status `open` or
    /// `in_work`, and [`Graph::is_until_satisfied`]. `blocked` and
    /// `deferred` tasks are never actionable whatever their edges say.
    ///
    /// Total: an id this graph does not hold answers `false` — an
    /// unknown task is never selected.
    pub fn is_actionable(&self, task: &TaskId) -> bool {
        let Some(task) = self.task(task) else {
            return false;
        };
        matches!(task.status, Status::Open | Status::InWork) && self.is_until_satisfied(&task.id)
    }

    /// Whether every predecessor holding `task` back is finished: its
    /// own incoming `blocks` edges — the until-targets of a deferred
    /// item (§3) — and its epic's incoming decision gates (§5.1: an
    /// open decision blocking an epic holds all work under it).
    ///
    /// A task predecessor is finished at `closed`/`wontfix`, a decision
    /// one at `signed`/`superseded`; any endpoint that cannot be
    /// classified as finished holds the task back (see `finished`).
    /// This is the edge half of actionability and the summary's `ready`
    /// test at once — one implementation, so the two cannot disagree
    /// about what "finished" means.
    ///
    /// Total: an id this graph does not hold answers `false` — an
    /// unknown task is never counted ready.
    pub fn is_until_satisfied(&self, task: &TaskId) -> bool {
        let Some(task) = self.task(task) else {
            return false;
        };
        self.ends_finished(&self.blocked_by(&BlockEnd::Task(task.id.clone())))
            && self.ends_finished(&self.blocked_by(&BlockEnd::Epic(task.epic.clone())))
    }

    /// The shared walk of one scope's derived list: the first
    /// actionable id, and the summary counted over the same list.
    fn select_from(&self, order: &[TaskId]) -> NextTask {
        let mut task = None;
        let mut priority = None;
        for id in order {
            if self.is_actionable(id) {
                task = Some(id.clone());
                // `is_actionable` accepted this id, so the lookup
                // cannot miss; a miss would leave the echo unset
                // rather than panicking.
                priority = self.task(id).and_then(|node| node.priority);
                break;
            }
        }
        let mut deferred = DeferredSummary { total: 0, ready: 0 };
        for candidate in order.iter().filter_map(|id| self.task(id)) {
            if candidate.status != Status::Deferred {
                continue;
            }
            deferred.total += 1;
            deferred.ready += u64::from(self.is_until_satisfied(&candidate.id));
        }
        NextTask {
            task,
            priority,
            deferred,
        }
    }

    /// Whether every endpoint in one adjacency list is finished.
    fn ends_finished(&self, ends: &[BlockEnd]) -> bool {
        ends.iter().all(|end| self.finished(end))
    }

    /// Whether one predecessor endpoint is finished, by kind.
    ///
    /// The two `false` fallbacks are unreachable through the public API
    /// and hold work back rather than releasing it — the state-space
    /// argument is in the module docs, judgment call (d).
    fn finished(&self, end: &BlockEnd) -> bool {
        match end {
            BlockEnd::Task(id) => self.task(id).is_some_and(|task| task.status.is_terminal()),
            BlockEnd::Decision(id) => self
                .decision(id)
                .is_some_and(|decision| decision.status.is_resolved()),
            BlockEnd::Epic(_) => false,
        }
    }
}
