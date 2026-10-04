//! Batch selection (spec §5.2): the `next_items` surface — up to
//! `limit` rows in three scopes (repo, epic, task), each annotated with
//! its direct blocked-by/blocks ids, a derived weight, the
//! actionable-or-held mark and its agent-set `priority` (D17), plus
//! the deferred summary [`Graph::next_task`] also carries.
//!
//! **The batch only.** The listing reads — `task_list`'s filters and
//! the three thin listings — are `list`, which follows this module's
//! shape (a sibling `impl Graph`, no new state, no serde on its answer
//! types) without calling any of it; nothing of it is pre-built here,
//! the same way [`Graph::canonical_order`] pre-built nothing of `select`
//! and `select` pre-built nothing of this module.
//!
//! # The rules
//!
//! Repo and epic scope answer with the scope's canonical order —
//! [`Graph::canonical_order`] and [`Graph::canonical_order_of_epic`],
//! the same lists [`Graph::next_task`] walks — filtered to the nodes
//! [`Graph::is_actionable`] accepts and truncated at `limit`. §6's
//! "ranked" is the canonical order's rank: annotations and the derived
//! weight never reorder the batch (D11, §11 — the weight is
//! display-only), and the batch is chosen before the first annotation
//! is attached, so they *cannot* reorder it. `next_task` is the
//! limit=1 case of these two scopes.
//!
//! Task scope answers with the queried task's transitive dependents —
//! the work hanging off a non-top-level item — in the same canonical
//! order, **every one of them**, each marked immediately actionable or
//! still held by other predecessors: this scope does not filter,
//! because the mark is the filter's replacement. The queried task is
//! never its own dependent — the closure is irreflexive — and "no
//! dependents" is an empty list, not an error (§6.1's code list is
//! closed).
//!
//! The annotation lists are the adjacency vocabulary of §6.2.
//! `blocked_by` is **structural**: every direct predecessor, finished
//! or not — the same two lists [`Graph::is_until_satisfied`] consults,
//! the task's own incoming edges plus its epic's incoming decision
//! gates — and readiness is the `actionable` field's half alone.
//! `blocks` is the task's direct successors.
//!
//! The weight is the count of the task's transitive `blocks`
//! dependents, excluding itself. It is structural — statuses never
//! enter, so a closed dependent still counts — and display-only: it
//! never orders the batch, never filters it, and no answer depends on
//! comparing one. A leaf answers 0.
//!
//! The deferred summary equals [`Graph::next_task`]'s for the same
//! scope, because both are a function of one list: the scope's derived
//! order, which in task scope *is* the dependents list — so a task
//! scope can report `{0, 0}` while the repo reports deferred items.
//! The summary pass reads the same list and the same pure predicates as
//! `select`'s; the second pass is deliberate (judgment call (i)).
//!
//! # Limit, answers and errors
//!
//! `limit` is a plain `u64`: `0` answers an empty batch with the
//! summary still present, and a limit above the scope simply returns
//! the scope — both values, not errors, because §6.1's list is closed
//! and has no code for them. There is no `truncated` or `remaining`
//! field: §5.2's response shape is the batch plus the summary, and an
//! agent that wants more asks again with a bigger limit (judgment call
//! (h)). The `limit?` default of 10 is the **wire's** (§5.2), not the
//! domain's: these functions always see an explicit limit, so the
//! number has one home and M3 need not re-derive it here.
//!
//! The repo-scoped entry point is infallible; the two scoped ones have
//! exactly one failure each, `unknown_id` for an id this graph does
//! not hold. "No items" is an empty list on every scope.
//!
//! No serde here: a query result is not persisted, and the consumer
//! owns its wire shape — M3 declares the tool's `outputSchema` and
//! D14's key order, and nothing in this module pre-empts it.
//!
//! Batch selection is a pure function of graph state (§8): no new
//! field, no cache, no memoisation — every call recomputes, which is
//! why nothing here can go stale. One query costs one order derivation,
//! one actionability pass that stops when the batch is full, one
//! dependents walk per returned row, and one summary pass — in task
//! scope plus the one closure walk that builds the scope itself —
//! inside §4.1's scale argument.
//!
//! # Judgment calls (t28)
//!
//! - **(a) Placement and wiring.** `impl Graph` methods in this module,
//!   mirroring `order.rs`/`select.rs`; the two answer types are
//!   re-exported through `graph/mod.rs` and the crate root, the way
//!   [`crate::NextTask`] is. The predicates add no new crate-root name
//!   — they are [`Graph`] methods.
//! - **(b) The dependents walk.** One depth-first walk per row over
//!   [`Graph::blocks`], not one successor map built up front: the rows
//!   are bounded by the limit, so the per-row walk allocates nothing
//!   for candidates the batch never returns, and each walk stays a
//!   local fact instead of shared state.
//! - **(c) Order, then filter, then annotate.** The batch is a prefix
//!   of the scope's derived list, chosen before any annotation exists,
//!   so "annotations never reorder" is a property of the pipeline's
//!   shape rather than a promise its steps keep — which is also what
//!   makes the order pins meaningful instead of tautological.
//! - **(d) Unreachable endpoints.** A task's successor list can only
//!   hold task ends: [`crate::BlocksEdge`]'s three variants make a task
//!   source's outgoing edge `TaskToTask` by construction, so the
//!   narrowing to [`TaskId`] drops arms that cannot occur. The
//!   annotation walk's missing-task arm is the same class: every id the
//!   batch annotates is drawn from a scope list — a canonical order or
//!   a dependents closure — and those hold only ids of tasks this graph
//!   has, so the arm where [`Graph::task`] answers `None` (which skips
//!   the epic-gate half rather than inventing one) cannot be reached
//!   from the batch or row walks. Both fallbacks filter rather than
//!   panic, and the state-space argument lives here rather than in a
//!   forged test — the public API cannot build the state a test would
//!   need (the same discipline as `select`'s judgment call (d)).
//! - **(e) Test layout.** `graph/tests/items.rs`, every name prefixed
//!   `items_` so the module filter selects exactly that suite.
//! - **(f) Two populations — a reading of §5.2, veto-able.** "Returns
//!   up to `limit` actionable nodes" and the task clause's "each marked
//!   immediately actionable or still held" are one sentence apart, and
//!   an actionable-only list could never carry the "still held" mark —
//!   under the uniform reading one of the two marks is unreachable — so
//!   repo and epic scope filter and task scope marks. Vetoing this
//!   reading changes [`Graph::next_items_of_task`] to filter its scope
//!   like the other two and empties the "held" half of the mark
//!   vocabulary.
//! - **(g) Epic gates in `blocked_by` — a second reading of §5.2,
//!   veto-able.** "Its direct blocked-by … ids" does not name the
//!   epic's decision gates; they are included because the annotation
//!   must not contradict [`Graph::is_actionable`], which consults them
//!   — a task held only by its epic's open gate would otherwise show an
//!   empty `blocked_by` beside `actionable: false` (§5.1: an open
//!   decision blocking an epic holds all work under it). Vetoing this
//!   reading drops the gate half of [`NextItem::blocked_by`] and
//!   re-introduces that disagreement.
//! - **(h) No `truncated` field.** The response is the batch plus the
//!   summary; a field the signed text does not carry is a wire change
//!   M3 would have to declare, and an agent that wants more asks again
//!   with a bigger limit.
//! - **(i) The summary is a second pass, not an extraction.** The
//!   rejected alternative — lifting `select`'s summary pass into a
//!   shared helper — would edit a reviewed file for nothing observable:
//!   both passes read the same derived list and the same pure
//!   predicates, and the agreement is pinned over the seeded graphs
//!   rather than trusted as an argument.
//! - **(j) Task scope's list derivation — a third reading of §5.2,
//!   veto-able.** "In the same canonical order" reads as the repo's:
//!   the dependents list is [`Graph::canonical_order`] restricted to
//!   the closure. The alternative reading — an order induced over the
//!   dependent subset, the way the epic scope induces over its tasks —
//!   is a different derivation this crate cannot build today: the
//!   induced walk is scoped to epics, so a veto is a named diff that
//!   adds a subset-induced seam to the order module first. The two
//!   readings part wherever a dependent's position inside the closure
//!   differs from its position in the repo order, and the suite cannot
//!   distinguish them today: the seeded agreement test's naive side
//!   reads this same contract, so its pin is circular here. Recorded so
//!   nobody assumes a pin exists.
//! - **(k) One id, once; two runs, not one sorted list.** A decision
//!   that gates a task directly *and* gates its epic reaches
//!   [`NextItem::blocked_by`] through both adjacency lists; the
//!   annotation is a list of ids, so the second occurrence is dropped —
//!   first occurrence wins. Each half keeps the [`BlockEnd`] order its
//!   accessor returns — the own edges first, then the epic's gates —
//!   and the two runs are deliberately not merged into one globally
//!   ascending list: the annotation is built from two accessor calls
//!   whose lists each arrive already sorted, and a global merge would
//!   be a second sort, re-ordering a list whose documented order the
//!   annotation's contract is to preserve. A consumer of the bytes —
//!   M3's `outputSchema` included — must describe the field as two
//!   ascending runs, not as one [`BlockEnd`]-ordered list. The rule's
//!   implementation is [`Graph::blockers_of`] — `summary`'s lift, the
//!   contract's one home — and this module's `annotate` calls it
//!   rather than restating it, so the batch's annotation and
//!   `task_list`'s rows cannot drift apart.

use std::collections::BTreeSet;

use crate::error::DomainError;
use crate::id::{EpicId, TaskId};
use crate::status::{Priority, Status};

use super::edges::BlockEnd;
use super::select::DeferredSummary;
use super::state::Graph;

/// The answer of a `next_items` query (§5.2): the scope's batch and the
/// scope's deferred summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextItems {
    /// The batch: up to `limit` rows in the scope's canonical order —
    /// the actionable ones in repo and epic scope, every transitive
    /// dependent in task scope (see the module docs, judgment call
    /// (f)). Never reordered by an annotation.
    pub items: Vec<NextItem>,
    /// The deferred summary of the queried scope, present
    /// unconditionally — including on an empty batch — and equal to
    /// [`Graph::next_task`]'s for the same scope.
    pub deferred: DeferredSummary,
}

/// One batch row (§5.2): an id in the scope's derived order, its
/// adjacency annotations, its derived weight and its readiness mark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextItem {
    /// The task this row is about.
    pub task: TaskId,
    /// Every direct predecessor holding this task back — its own
    /// incoming `blocks` edges plus its epic's incoming decision gates,
    /// finished or not: the list is structural, and readiness is the
    /// `actionable` field's half alone. One consequence: in repo and
    /// epic scope every row is actionable, so every listed predecessor
    /// is finished and the list records what *gated* the item; a listed
    /// predecessor that still holds its row back arises only in task
    /// scope. The order is **two ascending runs**, each in [`BlockEnd`]
    /// order — the task's own edges first, then its epic's gates —
    /// deduplicated at the first occurrence, and *not* one globally
    /// sorted list (judgment call (k)); a schema describing this field
    /// must say so. The list is [`Graph::blockers_of`]'s — the rule's
    /// one home, shared with `task_list`'s rows (D16) — narrowed to the
    /// id-only vocabulary this row carries. The asymmetry with `blocks`
    /// is a consequence of §5.1's three legal edge shapes, not a
    /// choice: predecessors can be tasks or decisions.
    pub blocked_by: Vec<BlockEnd>,
    /// The task's direct `blocks` successors. Tasks only, because §5.1
    /// gives a task no outgoing edge to a decision or an epic — the
    /// narrowing is total (judgment call (d)).
    pub blocks: Vec<TaskId>,
    /// The count of this task's transitive `blocks` dependents,
    /// excluding itself; 0 for a leaf. Structural — statuses never
    /// enter — and display-only: it never orders or filters the batch
    /// (D11, §11).
    pub weight: u64,
    /// Whether the task may be picked up now — [`Graph::is_actionable`],
    /// the production predicate. Always `true` in repo and epic scope,
    /// where the batch admits actionable nodes only; the mark carries
    /// information in task scope, where held dependents stay listed.
    pub actionable: bool,
    /// The task's agent-set `priority` (D17), read off the node —
    /// display-only, never an ordering input (D11, §11): the batch is
    /// chosen before the annotation is attached, so it cannot reorder.
    pub priority: Option<Priority>,
}

impl Graph {
    /// The next actionable batch of this repository (§5.2): the first
    /// `limit` nodes of [`Graph::canonical_order`] that
    /// [`Graph::is_actionable`] accepts, annotated, plus the deferred
    /// summary of the whole repo scope. [`Graph::next_task`] is the
    /// `limit = 1` case.
    ///
    /// A total function: an empty batch is an answer, not an error, and
    /// the summary rides it unconditionally. Selection reads graph state
    /// alone (§8), so the same state always yields the same
    /// [`NextItems`].
    pub fn next_items(&self, limit: u64) -> NextItems {
        let scope = self.canonical_order();
        let items = self.batch(&scope, limit, true);
        NextItems {
            deferred: self.summarize(&scope),
            items,
        }
    }

    /// The next actionable batch of one epic (§5.2): the first `limit`
    /// nodes of [`Graph::canonical_order_of_epic`] — the induced order —
    /// that [`Graph::is_actionable`] accepts against the **full** graph,
    /// annotated, plus the deferred summary of that epic's scope.
    /// [`Graph::next_task_of_epic`] is the `limit = 1` case.
    ///
    /// The batch skips a held node and keeps walking: a cross-epic
    /// predecessor contributes no in-degree to the induced list and
    /// still holds its successor back here. An epic this graph does not
    /// hold is `unknown_id`, propagated from the order derivation and
    /// the only failure this method has.
    pub fn next_items_of_epic(&self, epic: &EpicId, limit: u64) -> Result<NextItems, DomainError> {
        let scope = self.canonical_order_of_epic(epic)?;
        Ok(NextItems {
            items: self.batch(&scope, limit, true),
            deferred: self.summarize(&scope),
        })
    }

    /// The work hanging off one task (§5.2): its transitive `blocks`
    /// dependents — never the queried task itself — in
    /// [`Graph::canonical_order`]'s sequence, up to `limit`, **each
    /// marked** immediately actionable or still held by other
    /// predecessors, plus the deferred summary of that dependents list.
    ///
    /// This scope does not filter to actionable nodes: the mark is the
    /// filter's replacement (module docs, judgment call (f)). A task
    /// this graph does not hold is `unknown_id`, the only failure this
    /// method has; a task without dependents is an empty batch, not an
    /// error.
    pub fn next_items_of_task(&self, task: &TaskId, limit: u64) -> Result<NextItems, DomainError> {
        self.require_task(task)?;
        let dependents = self.dependents_of(task);
        let scope: Vec<TaskId> = self
            .canonical_order()
            .into_iter()
            .filter(|id| dependents.contains(id))
            .collect();
        Ok(NextItems {
            items: self.batch(&scope, limit, false),
            deferred: self.summarize(&scope),
        })
    }

    /// The shared row walk: the first `limit` ids of the scope's derived
    /// list — only the actionable ones when `actionable_only` — each
    /// annotated after the batch is chosen (judgment call (c)).
    fn batch(&self, scope: &[TaskId], limit: u64, actionable_only: bool) -> Vec<NextItem> {
        let mut items: Vec<NextItem> = Vec::new();
        let mut remaining = limit;
        for id in scope {
            if remaining == 0 {
                break;
            }
            let actionable = self.is_actionable(id);
            if actionable_only && !actionable {
                continue;
            }
            items.push(self.annotate(id, actionable));
            remaining -= 1;
        }
        items
    }

    /// One row's annotations, computed for returned rows only — after
    /// the limit is applied. `actionable` is passed in because the
    /// caller's walk has just asked the same question. The
    /// `blocked_by` half is [`Graph::blockers_of`]'s — the rule's one
    /// home (judgment call (k)) — converted to the id-only vocabulary
    /// this row carries. An id this graph does not hold — unreachable,
    /// because every scope list derives from the graph's own tasks
    /// (judgment call (d)) — skips the epic-gate half rather than
    /// inventing one.
    fn annotate(&self, id: &TaskId, actionable: bool) -> NextItem {
        let blocked_by = self
            .blockers_of(id)
            .into_iter()
            .map(BlockEnd::from)
            .collect();
        let blocks = self.task_successors(id);
        let weight = self.dependents_of(id).len() as u64;
        // The echo (D17): read off the node, never derived. The
        // missing-node arm is unreachable for the reason above and
        // answers `None` rather than inventing a priority.
        let priority = self.task(id).and_then(|node| node.priority);
        NextItem {
            task: id.clone(),
            blocked_by,
            blocks,
            weight,
            actionable,
            priority,
        }
    }

    /// The deferred summary over one scope's derived list — the same
    /// list the batch walk reads, so scope has one definition. The
    /// second pass is judgment call (i); the pinned agreement with
    /// [`Graph::next_task`] lives in `graph/tests/items.rs`.
    fn summarize(&self, scope: &[TaskId]) -> DeferredSummary {
        let mut deferred = DeferredSummary { total: 0, ready: 0 };
        for candidate in scope.iter().filter_map(|id| self.task(id)) {
            if candidate.status != Status::Deferred {
                continue;
            }
            deferred.total += 1;
            deferred.ready += u64::from(self.is_until_satisfied(&candidate.id));
        }
        deferred
    }

    /// The transitive `blocks` dependents of `task`, excluding `task`
    /// itself — an irreflexive closure, so a self-edge (unreachable
    /// through the public API, which rejects cycles at write time) could
    /// not smuggle the queried task into its own scope. Only
    /// `TaskToTask` edges leave a task, so the walk never reaches a
    /// decision or an epic, and the parent tree is not a dependency
    /// relation — it is a field.
    fn dependents_of(&self, task: &TaskId) -> BTreeSet<TaskId> {
        let mut seen: BTreeSet<TaskId> = BTreeSet::new();
        let mut stack = vec![task.clone()];
        while let Some(current) = stack.pop() {
            for next in self.task_successors(&current) {
                if next != *task && seen.insert(next.clone()) {
                    stack.push(next);
                }
            }
        }
        seen
    }

    /// The direct `blocks` successors of a task, narrowed to task ids.
    /// The dropped arm cannot occur (judgment call (d)).
    fn task_successors(&self, task: &TaskId) -> Vec<TaskId> {
        self.blocks(&BlockEnd::Task(task.clone()))
            .into_iter()
            .filter_map(|end| match end {
                BlockEnd::Task(id) => Some(id),
                BlockEnd::Epic(_) | BlockEnd::Decision(_) => None,
            })
            .collect()
    }
}
