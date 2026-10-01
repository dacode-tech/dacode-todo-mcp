//! The canonical order (spec §5.2): the topological derivation over the
//! graph with the signed D6 tiebreaker — topology first, then epic
//! registration order, then task id — repo-scoped and epic-scoped.
//!
//! **Ordering only.** The surfaces that read it live beside this module:
//! `select` holds `next_task`'s actionability filtering and the deferred
//! summary, `items` the batch — `next_items`'s annotations, derived
//! weight and task-scoped dependents view — and `list` the filtered and
//! thin listings. This module builds the pure order all three read.
//! Consequently:
//!
//! - **Statuses do not filter.** A `closed`/`wontfix` task sits at its
//!   topological position; actionability is selection's question, not
//!   the order's.
//! - **Only `TaskToTask` edges constrain.** Decision gating
//!   (`DecisionToTask`/`DecisionToEpic`) is an input to the
//!   actionability check, not an order constraint, and `relates` never
//!   appears here (§5.1: "an annotation with no ordering semantics").
//! - **The agent-set `priority` never enters** — not the key, not the
//!   frontier, not the output (D11: payload metadata).
//!
//! # The derivation
//!
//! Best-first Kahn: the frontier holds the in-scope tasks whose
//! in-scope predecessors are all placed, and each step takes the
//! frontier minimum under the tiebreaker key `(epic number, task
//! number)`. Topological validity *constrains* — a task enters the
//! frontier only when its last predecessor is placed — and the key
//! *chooses* among simultaneously ready candidates, which is exactly
//! "topology first, then epic registration order, then task id". The
//! result is the lexicographically-minimal topological order under the
//! key; the tests pin that against an independent naive recomputation.
//!
//! Epic registration order is ascending [`EpicId`] number and task-id
//! order is ascending [`TaskId`] number: §3's counters allocate
//! sequentially from 1 and never renumber, so both key levels derive
//! from existing fields alone — no new state. Task numbers are never
//! reused, so the key is unique per task and the frontier minimum is a
//! function of graph state alone (§8), never of arrival or hash order:
//! every container here is a `BTreeMap`, and isomorphic graphs built by
//! shuffled insertions derive byte-identical orders.
//!
//! # Scope and totality
//!
//! The epic-scoped order is the *induced subgraph*: that epic's tasks
//! and the `TaskToTask` edges between them. A cross-epic predecessor
//! contributes no in-degree to the scoped order — no semantics is lost,
//! because the actionability check ([`Graph::is_until_satisfied`])
//! consults the full [`Graph::blocked_by`]: order answers "in what
//! sequence", selection answers "is it ready".
//!
//! [`Graph::link_blocks`] rejects cycles at write time (§5.1), so via
//! the public API the derivation always places every in-scope task:
//! [`Graph::canonical_order`] is a total function returning
//! `Vec<TaskId>` with no `Result` to interpret, and
//! [`Graph::canonical_order_of_epic`] returns `Result` only because the
//! epic id itself can be unknown. Should Kahn ever leave a remainder —
//! unreachable through the public API — the fallback documented on
//! `derive_order` appends it in key order instead of panicking or
//! fabricating a `cycle` payload the derivation cannot honor; the
//! seeded tests assert the remainder never fires.
//!
//! # Judgment calls (t8)
//!
//! - **(a) Placement.** `impl Graph` methods in this module, mirroring
//!   the `edges.rs` pattern: the order is behavior over the graph value,
//!   and its entry points sit on [`Graph`] beside the other queries.
//! - **(b) Return type.** `Vec<TaskId>`, not `Vec<&Task>`: ids keep the
//!   borrow out of the caller's way, serialize for §8's byte-identity
//!   instruments, and [`Graph::task`] resolves any of them.
//! - **(c) Key exposure.** The tiebreaker key stays module-private. The
//!   selection batch this call anticipated has landed, and the trigger
//!   did not fire: a batch is a filtered prefix of the derived order,
//!   so its positions come from the walk — never from the key, which
//!   the batch surface does not read — and the derived weight is
//!   display-only. Nothing annotates or slices by the key. The seam
//!   stays `order_key`, a pure function of two existing fields, for
//!   whoever needs it next.
//! - **(d) Indexing.** The derivation builds its own in-degree and
//!   successor maps in one pass over the tasks and stored edges; it
//!   never calls [`Graph::tasks_of_epic`], so that linear-scan accessor
//!   is neither hot here nor obsolete — it remains `delete_epic`'s and
//!   §6.2's grouping read.
//! - **(e) Ordering contracts.** This module relies on *no* accessor's
//!   iteration order: determinism comes from `BTreeMap` key order, not
//!   arrival order. The one order-adjacent fact it uses is structural,
//!   not iterative — §3's counter monotonicity (documented on
//!   `Counters`), which makes id numbers equal registration/creation
//!   order.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::DomainError;
use crate::id::{EpicId, TaskId};
use crate::node::Task;

use super::edges::BlocksEdge;
use super::state::Graph;

/// The D6 tiebreaker key: `(epic number, task number)` — §5.2's second
/// and third levels. Unique per task, because task numbers are never
/// reused (§3), so a frontier keyed by it is totally ordered.
type OrderKey = (u64, u64);

/// A task's tiebreaker key, derived from existing fields alone (§5.2).
fn order_key(task: &Task) -> OrderKey {
    (task.epic.number(), task.id.number())
}

impl Graph {
    /// The canonical order (§5.2) of every task in this repository: a
    /// topological order of the `TaskToTask` `blocks` edges in which the
    /// D6 tiebreaker decides among simultaneously ready tasks — epic
    /// registration order first, then task id (creation order). The
    /// result is the lexicographically-minimal topological order under
    /// that key.
    ///
    /// Order is derived, never stored. Statuses do not filter — a
    /// `closed` or `wontfix` task sits at its topological position,
    /// because actionability is M2's selection, not this list; decision
    /// gating edges and `relates` never reorder (§5.1); the agent-set
    /// `priority` never enters (D11).
    ///
    /// A total function: [`Graph::link_blocks`] rejects cycles at write
    /// time, so every task always places. Were that invariant ever
    /// broken, the unplaceable remainder is appended in key order, not
    /// dropped — see the `derive_order` fallback.
    pub fn canonical_order(&self) -> Vec<TaskId> {
        self.derive_order(None).0
    }

    /// The canonical order (§5.2) of one epic's tasks: the induced
    /// subgraph — that epic's tasks and the `TaskToTask` edges between
    /// them — under the same tiebreaker as [`Graph::canonical_order`].
    ///
    /// A cross-epic predecessor contributes no in-degree here: the
    /// scoped list answers "in what sequence should this epic's tasks be
    /// considered", and the actionability check
    /// ([`Graph::is_until_satisfied`]) consults the full
    /// [`Graph::blocked_by`], so the outside blocker is still honored at
    /// selection time — no semantics is lost.
    ///
    /// An epic this graph does not hold is `unknown_id` — the only
    /// failure this method has; a known epic without tasks orders empty.
    pub fn canonical_order_of_epic(&self, epic: &EpicId) -> Result<Vec<TaskId>, DomainError> {
        if !self.epics.contains_key(epic) {
            return Err(DomainError::UnknownId {
                id: epic.to_string(),
            });
        }
        Ok(self.derive_order(Some(epic)).0)
    }

    /// The shared best-first Kahn derivation behind the two public entry
    /// points, returning `(order, remainder)`.
    ///
    /// `remainder` is how many in-scope tasks Kahn could not place —
    /// tasks caught in a `blocks` cycle. That state is unreachable
    /// through the public API ([`Graph::link_blocks`] rejects cycles at
    /// write time, §5.1), and the derivation does not panic on it: the
    /// remainder is appended to `order` in tiebreaker key order, so the
    /// result stays total and deterministic even if the invariant were
    /// broken. No `cycle` error is fabricated here — §6.1's payload
    /// promises a path, and a stalled Kahn walk cannot report one. The
    /// count is the invariant seam the seeded tests assert stays zero.
    ///
    /// `scope` selects the induced subgraph: `None` orders the whole
    /// repository, `Some(epic)` that epic's tasks and the edges between
    /// them.
    pub(super) fn derive_order(&self, scope: Option<&EpicId>) -> (Vec<TaskId>, usize) {
        // The in-scope tasks and their remaining in-scope predecessor
        // counts. Keyed by reference: everything borrows from `self`,
        // and the owned output is cloned once, at the end.
        let mut in_degree: BTreeMap<&TaskId, usize> = BTreeMap::new();
        for task in self.tasks.values() {
            if scope.is_none_or(|epic| task.epic == *epic) {
                in_degree.insert(&task.id, 0);
            }
        }

        // Only `TaskToTask` edges with both ends in scope constrain the
        // order: decision gating edges are never order constraints, and
        // the epic-scoped walk is the induced subgraph.
        let mut successors: BTreeMap<&TaskId, Vec<&TaskId>> = BTreeMap::new();
        for edge in self.blocks_out.values().flatten() {
            if let BlocksEdge::TaskToTask { from, to } = edge
                && in_degree.contains_key(from)
                && in_degree.contains_key(to)
            {
                successors.entry(from).or_default().push(to);
                if let Some(degree) = in_degree.get_mut(to) {
                    *degree += 1;
                }
            }
        }

        // Best-first Kahn: the frontier is keyed by the tiebreaker, so
        // popping its first entry is taking the minimum, and every step
        // is a function of the graph state alone.
        let mut frontier: BTreeMap<OrderKey, &TaskId> = BTreeMap::new();
        for task in self.tasks.values() {
            if in_degree.get(&task.id) == Some(&0) {
                frontier.insert(order_key(task), &task.id);
            }
        }
        let total = in_degree.len();
        let mut order: Vec<TaskId> = Vec::with_capacity(total);
        while let Some((_, placed)) = frontier.pop_first() {
            order.push(placed.clone());
            if let Some(released) = successors.get(placed) {
                for next in released {
                    let Some(degree) = in_degree.get_mut(next) else {
                        continue;
                    };
                    // Each edge decrements exactly once — successors are
                    // a unique set per source — so the count cannot go
                    // below zero; saturating keeps that a local fact, not
                    // a panic path.
                    *degree = degree.saturating_sub(1);
                    if *degree == 0
                        && let Some(task) = self.tasks.get(*next)
                    {
                        frontier.insert(order_key(task), *next);
                    }
                }
            }
        }

        // The unreachable-via-public-API fallback (see the method docs):
        // append whatever Kahn could not place, in key order.
        let remainder = total.saturating_sub(order.len());
        if remainder > 0 {
            let placed: BTreeSet<&TaskId> = order.iter().collect();
            let mut rest: Vec<(OrderKey, &TaskId)> = self
                .tasks
                .values()
                .filter(|task| in_degree.contains_key(&task.id) && !placed.contains(&task.id))
                .map(|task| (order_key(task), &task.id))
                .collect();
            rest.sort_by_key(|(key, _)| *key);
            order.extend(rest.into_iter().map(|(_, id)| id.clone()));
        }
        (order, remainder)
    }
}
