//! The batch-selection tests (spec §5.2, §10): the actionable prefix in
//! canonical order, the limit, the two scope populations and their
//! marks, the derived weight, the adjacency annotations, the deferred
//! summary's scoping, the `unknown_id` failures, the degenerate scopes,
//! seeded agreement with an independent naive recomputation, the
//! cross-pin against `next_task`, the coverage pin that keeps the
//! agreement non-vacuous, and identical answers across reruns and
//! shuffled insertions.
//!
//! The naive recomputation is deliberately a second, slower
//! implementation reading only the public accessors and spelling the
//! status matches and the dependents closure out by hand, so agreement
//! cannot come from a shared bug in the production predicates or the
//! production walk (`naive_select` in `tests/select.rs` is the
//! precedent). The canonical *order* is the production one by design,
//! for the reason the selection suite records: `tests/order.rs` pins it
//! independently, and the batch contract is a prefix of that list.

use std::collections::BTreeSet;

use crate::{DeferredSummary, NextItem, NextItems};

use super::seeded::{Rng, run_sequence};
use super::*;

/// Fixed seeds for the insertion shuffles, per the suite's rule that a
/// determinism test never reseeds from a clock.
const SEEDS: [u64; 5] = [3, 21, 0xF00D, 0x5EED_9002, 0xABCD_1234];
/// Fixed seeds for the mutation-sequence graphs the agreement and
/// coverage tests run over. The coverage pin asserts per seed, so each
/// seed's final graph must on its own reach every branch the pin
/// counts — a generator change that narrows one seed fails there
/// rather than hiding behind the others. The seed set and the step
/// depth below are this suite's own and deliberately not the selection
/// suite's: the shapes this pin counts — a scope truncated by the
/// limit, a task scope carrying both marks — are whole-batch
/// properties that need deeper graphs than the selection pin's
/// per-task shapes do, and seeds that satisfy one pin need not satisfy
/// the other.
const GRAPH_SEEDS: [u64; 5] = [13, 30, 50, 60, 128];
/// Steps per seeded graph: deep enough that the coverage pin below
/// finds every branch, shallow enough that the per-step invariant scan
/// stays cheap. Deeper than the selection suite's for the reason the
/// seed set above records.
const ITEMS_STEPS: u64 = 600;
/// The truncating limit the seeded tests use: small enough that the
/// seeded scopes really truncate, large enough that the batches carry
/// several annotated rows.
const LIMIT: u64 = 3;

// ---------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------

fn t_end(n: u64) -> BlockEnd {
    BlockEnd::Task(task_id(n))
}

fn d_end(n: u64) -> BlockEnd {
    BlockEnd::Decision(decision_id(n))
}

fn summary(total: u64, ready: u64) -> DeferredSummary {
    DeferredSummary { total, ready }
}

/// An expected row, spelled the way the field docs read.
fn row(
    task: u64,
    blocked_by: Vec<BlockEnd>,
    blocks: Vec<u64>,
    weight: u64,
    actionable: bool,
) -> NextItem {
    NextItem {
        task: task_id(task),
        blocked_by,
        blocks: blocks.into_iter().map(task_id).collect(),
        weight,
        actionable,
    }
}

fn ids(answer: &NextItems) -> Vec<TaskId> {
    answer.items.iter().map(|item| item.task.clone()).collect()
}

/// The interleaved graph T1 and T2 share: E1 holds U1..U4 open with
/// U1→U2 and U3→U4, plus U5 deferred and U6 closed with U6→U5, so the
/// canonical order is [U1, U2, U3, U4, U6, U5] and interleaves the
/// actionable U1/U3 with held, terminal and deferred nodes. The summary
/// is {1, 1}: U5 is deferred and its until-target U6 is closed.
fn interleaved_graph() -> Graph {
    let mut graph = graph_with_epics(1);
    for n in 1..=4 {
        graph.insert_task(task_node(n, 1, Status::Open)).unwrap();
    }
    graph
        .insert_task(task_node(5, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(6, 1, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    graph.link_blocks(task_edge(3, 4)).unwrap();
    graph.link_blocks(task_edge(6, 5)).unwrap();
    graph
}

// ---------------------------------------------------------------------
// 1./2. The batch shape: the actionable prefix, and the limit
// ---------------------------------------------------------------------

#[test]
fn items_batch_is_the_actionable_prefix_in_canonical_order() {
    let graph = interleaved_graph();
    assert_eq!(
        graph.canonical_order(),
        vec![
            task_id(1),
            task_id(2),
            task_id(3),
            task_id(4),
            task_id(6),
            task_id(5)
        ]
    );
    // The batch skips the held U2/U4, the terminal U6 and the deferred
    // U5, and keeps the order's sequence.
    let batch = graph.next_items(u64::MAX);
    assert_eq!(ids(&batch), vec![task_id(1), task_id(3)]);
    assert_eq!(batch.deferred, summary(1, 1));
    assert_eq!(
        batch.items,
        vec![
            row(1, vec![], vec![2], 1, true),
            row(3, vec![], vec![4], 1, true),
        ]
    );
}

#[test]
fn items_limit_truncates_and_never_reorders() {
    let graph = interleaved_graph();
    let full = ids(&graph.next_items(u64::MAX));
    assert_eq!(full, vec![task_id(1), task_id(3)]);

    // `limit = 0` is an empty answer with the summary still present —
    // a value, not an error (§6.1's list is closed).
    let zero = graph.next_items(0);
    assert!(zero.items.is_empty());
    assert_eq!(zero.deferred, summary(1, 1));

    // Every finite limit is a prefix of the unlimited answer, and the
    // summary does not depend on the limit: it is over the scope, not
    // over the batch.
    for limit in [1u64, 2, 3, 100] {
        let batch = graph.next_items(limit);
        let expected: Vec<TaskId> = full.iter().take(limit as usize).cloned().collect();
        assert_eq!(ids(&batch), expected, "limit {limit}");
        assert_eq!(batch.deferred, summary(1, 1), "limit {limit}");
    }
    assert_eq!(ids(&graph.next_items(1)), vec![task_id(1)]);
}

// ---------------------------------------------------------------------
// 3./4. The marks: the redundancy invariant, and task scope's held rows
// ---------------------------------------------------------------------

#[test]
fn items_repo_and_epic_rows_are_all_actionable() {
    // E1: U1 open → U2 open. E2: U3 open, U4 closed → U5 open. Repo
    // order [U1, U2, U3, U4, U5]; the batch admits U1, U3 and U5.
    let mut graph = graph_with_epics(2);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(3, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 2, Status::Closed)).unwrap();
    graph.insert_task(task_node(5, 2, Status::Open)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    graph.link_blocks(task_edge(4, 5)).unwrap();

    let repo = graph.next_items(u64::MAX);
    assert_eq!(ids(&repo), vec![task_id(1), task_id(3), task_id(5)]);
    // D7's redundancy invariant, checked against the production
    // predicate rather than a restated status match.
    for item in &repo.items {
        assert!(item.actionable, "{} listed but not actionable", item.task);
        assert_eq!(item.actionable, graph.is_actionable(&item.task));
    }

    for (epic, expected) in [(1u64, vec![1u64]), (2, vec![3, 5])] {
        let batch = graph.next_items_of_epic(&epic_id(epic), u64::MAX).unwrap();
        assert_eq!(
            ids(&batch),
            expected.into_iter().map(task_id).collect::<Vec<_>>(),
            "epic E{epic}"
        );
        for item in &batch.items {
            assert!(item.actionable, "E{epic}: {} not actionable", item.task);
            assert_eq!(item.actionable, graph.is_actionable(&item.task));
        }
    }
}

#[test]
fn items_task_scope_returns_held_dependents_and_marks_them() {
    // D4's pin. U1 closed; U2 and U3 are its dependents, U3 also held
    // by a *different* predecessor, the open U4. Canonical order is
    // [U1, U2, U4, U3], so the task scope's list is [U2, U3].
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    graph.link_blocks(task_edge(1, 3)).unwrap();
    graph.link_blocks(task_edge(4, 3)).unwrap();
    assert_eq!(
        graph.canonical_order(),
        vec![task_id(1), task_id(2), task_id(4), task_id(3)]
    );

    let batch = graph.next_items_of_task(&task_id(1), u64::MAX).unwrap();
    // Both dependents are listed, in canonical order, each marked: an
    // actionable-only batch would drop the held U3 and could never
    // carry the "still held" mark §5.2 names.
    assert_eq!(
        batch.items,
        vec![
            row(2, vec![t_end(1)], vec![], 0, true),
            row(3, vec![t_end(1), t_end(4)], vec![], 0, false),
        ]
    );
    assert_eq!(batch.deferred, summary(0, 0));
    assert_eq!(batch.items.iter().filter(|i| !i.actionable).count(), 1);
}

// ---------------------------------------------------------------------
// 5./6. The weight: transitive dependent nodes, structural
// ---------------------------------------------------------------------

#[test]
fn items_weight_counts_dependent_nodes_not_paths() {
    // A diamond (U1 → {U2, U3} → U4), a chain (U4 → U5 → U6) and the
    // leaf U6. U4 is reachable from U1 by two paths; a walk that counts
    // paths instead of nodes answers 8 for U1 where the node count is 5.
    let mut graph = graph_with_epics(1);
    for n in 1..=6 {
        graph.insert_task(task_node(n, 1, Status::Open)).unwrap();
    }
    for (from, to) in [(1, 2), (1, 3), (2, 4), (3, 4), (4, 5), (5, 6)] {
        graph.link_blocks(task_edge(from, to)).unwrap();
    }

    // Repo scope: only U1 is actionable — everything else is held by an
    // open predecessor.
    let repo = graph.next_items(u64::MAX);
    assert_eq!(repo.items, vec![row(1, vec![], vec![2, 3], 5, true)]);

    // Task scope walks every dependent and spells each weight out.
    let batch = graph.next_items_of_task(&task_id(1), u64::MAX).unwrap();
    assert_eq!(
        batch.items,
        vec![
            row(2, vec![t_end(1)], vec![4], 3, false),
            row(3, vec![t_end(1)], vec![4], 3, false),
            row(4, vec![t_end(2), t_end(3)], vec![5], 2, false),
            row(5, vec![t_end(4)], vec![6], 1, false),
            row(6, vec![t_end(5)], vec![], 0, false),
        ]
    );
}

#[test]
fn items_weight_is_structural_and_counts_terminal_dependents() {
    // U1 open → U2 closed → U3 open. Weight never reads statuses: the
    // closed U2 still counts as U1's dependent, and the leaf U3 answers
    // 0. An implementer that "helpfully" filtered terminal dependents
    // would answer 1 for U1.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    graph.link_blocks(task_edge(2, 3)).unwrap();

    let repo = graph.next_items(u64::MAX);
    assert_eq!(
        repo.items,
        vec![
            row(1, vec![], vec![2], 2, true),
            row(3, vec![t_end(2)], vec![], 0, true),
        ]
    );

    // The task scope marks the terminal dependent itself: U2 is a row,
    // not actionable, and its own weight still counts the live U3.
    let batch = graph.next_items_of_task(&task_id(1), u64::MAX).unwrap();
    assert_eq!(
        batch.items,
        vec![
            row(2, vec![t_end(1)], vec![3], 1, false),
            row(3, vec![t_end(2)], vec![], 0, true),
        ]
    );
}

// ---------------------------------------------------------------------
// 7./8. Annotations never reorder; the weight never ranks
// ---------------------------------------------------------------------

#[test]
fn items_annotations_never_reorder() {
    // U1 open (weight 0), U2 closed → U3 open (weight 1) → U4 open.
    // Canonical order [U1, U2, U3, U4]; the batch is [U1, U3] — the
    // order's actionable prefix, asserted *from* the order list in the
    // same test that reads the annotations.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(2, 3)).unwrap();
    graph.link_blocks(task_edge(3, 4)).unwrap();

    let order = graph.canonical_order();
    assert_eq!(order, vec![task_id(1), task_id(2), task_id(3), task_id(4)]);
    let batch = graph.next_items(u64::MAX);
    let expected_prefix: Vec<TaskId> = order
        .iter()
        .filter(|id| graph.is_actionable(id))
        .take(2)
        .cloned()
        .collect();
    assert_eq!(ids(&batch), expected_prefix);
    // The annotations, read on the very batch whose order was pinned:
    // the weight is 0 on the first row and 1 on the second, and the
    // ids did not move.
    assert_eq!(
        batch.items,
        vec![
            row(1, vec![], vec![], 0, true),
            row(3, vec![t_end(2)], vec![4], 1, true),
        ]
    );

    // Epic scope: the induced order is the same list here, and the
    // batch must be its prefix too.
    let scoped_order = graph.canonical_order_of_epic(&epic_id(1)).unwrap();
    let scoped = graph.next_items_of_epic(&epic_id(1), u64::MAX).unwrap();
    assert_eq!(scoped_order, order);
    assert_eq!(ids(&scoped), expected_prefix);

    // Task scope: U2's dependents in canonical order, positions
    // strictly ascending in the repo order — the annotations of the
    // held U4 did not move it ahead of the actionable U3.
    let dependents = graph.next_items_of_task(&task_id(2), u64::MAX).unwrap();
    assert_eq!(ids(&dependents), vec![task_id(3), task_id(4)]);
    let positions: Vec<usize> = dependents
        .items
        .iter()
        .map(|item| {
            order
                .iter()
                .position(|id| *id == item.task)
                .unwrap_or(usize::MAX)
        })
        .collect();
    assert_eq!(positions, vec![2, 3]);
    assert_eq!(
        dependents.items,
        vec![
            row(3, vec![t_end(2)], vec![4], 1, true),
            row(4, vec![t_end(3)], vec![], 0, false),
        ]
    );
}

#[test]
fn items_weight_does_not_rank() {
    // Graph A: U1 open (weight 0), U2 closed → U3 open (weight 1) → U4
    // open. The first actionable task has a *lower* weight than a later
    // one — the shape a weight-ranked implementation gets wrong.
    let build = |extra: bool| {
        let mut graph = graph_with_epics(1);
        graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
        graph.insert_task(task_node(2, 1, Status::Closed)).unwrap();
        graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
        graph.insert_task(task_node(4, 1, Status::Open)).unwrap();
        graph.link_blocks(task_edge(2, 3)).unwrap();
        graph.link_blocks(task_edge(3, 4)).unwrap();
        if extra {
            // Graph B: one more edge, U1 → U5, raising U1's weight to 1
            // without moving the canonical order of U1..U4.
            graph.insert_task(task_node(5, 1, Status::Open)).unwrap();
            graph.link_blocks(task_edge(1, 5)).unwrap();
        }
        graph
    };
    let plain = build(false);
    let weighted = build(true);

    let plain_batch = plain.next_items(u64::MAX);
    let weighted_batch = weighted.next_items(u64::MAX);
    // Equal batches — same ids, same sequence — although the weights
    // differ: the edge structure moved the annotation, not the order.
    assert_eq!(ids(&plain_batch), ids(&weighted_batch));
    assert_eq!(ids(&plain_batch), vec![task_id(1), task_id(3)]);
    let plain_weights: Vec<u64> = plain_batch.items.iter().map(|i| i.weight).collect();
    let weighted_weights: Vec<u64> = weighted_batch.items.iter().map(|i| i.weight).collect();
    assert_eq!(plain_weights, vec![0, 1], "the inversion the ranker wants");
    assert_eq!(weighted_weights, vec![1, 1]);
    // The same through the epic scope.
    assert_eq!(
        ids(&plain.next_items_of_epic(&epic_id(1), u64::MAX).unwrap()),
        ids(&weighted.next_items_of_epic(&epic_id(1), u64::MAX).unwrap())
    );
}

// ---------------------------------------------------------------------
// 9./10./11. next_task is the limit=1 case; the epic scope's two pins
// ---------------------------------------------------------------------

#[test]
fn items_next_task_is_the_limit_one_case() {
    // E1: U1 open, U2 deferred, U3 closed → U2, U5 open. E2: U4 open,
    // held cross-epic by U5 (edge U5→U4). Repo order [U1, U3, U2, U5,
    // U4]; the repo batch is [U1, U5] and the summary is {1, 1}.
    let mut graph = graph_with_epics(2);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(2, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(3, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(5, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 2, Status::Open)).unwrap();
    graph.link_blocks(task_edge(3, 2)).unwrap();
    graph.link_blocks(task_edge(5, 4)).unwrap();

    // Repo scope: the limit=1 row is `next_task`'s answer, and the two
    // summaries are equal.
    let single = graph.next_items(1);
    let next = graph.next_task();
    assert_eq!(next.task, Some(task_id(1)));
    assert_eq!(ids(&single), vec![task_id(1)]);
    assert_eq!(single.deferred, next.deferred);
    assert_eq!(single.deferred, summary(1, 1));

    // Epic scope E1: the same pairing.
    let scoped_single = graph.next_items_of_epic(&epic_id(1), 1).unwrap();
    let scoped_next = graph.next_task_of_epic(&epic_id(1)).unwrap();
    assert_eq!(scoped_next.task, Some(task_id(1)));
    assert_eq!(ids(&scoped_single), vec![task_id(1)]);
    assert_eq!(scoped_single.deferred, scoped_next.deferred);

    // The None/empty pairing: E2's only task is held cross-epic, so
    // `next_task` answers None and every limit answers an empty batch,
    // summaries equal.
    let held = graph.next_items_of_epic(&epic_id(2), u64::MAX).unwrap();
    let held_next = graph.next_task_of_epic(&epic_id(2)).unwrap();
    assert_eq!(held_next.task, None);
    assert!(held.items.is_empty());
    assert_eq!(held.deferred, held_next.deferred);
    assert_eq!(held.deferred, summary(0, 0));
}

#[test]
fn items_epic_scope_honors_a_cross_epic_predecessor() {
    // E2's induced order offers U1 first, but E1's open U3 holds it
    // back cross-epic: the batch skips U1 (and the closed U5) and
    // returns both actionable tasks behind them — skip, not stop.
    let mut graph = graph_with_epics(2);
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(1, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(5, 2, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(3, 1)).unwrap();
    graph.link_blocks(task_edge(5, 4)).unwrap();

    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(2)).unwrap(),
        vec![task_id(1), task_id(2), task_id(5), task_id(4)]
    );
    let batch = graph.next_items_of_epic(&epic_id(2), u64::MAX).unwrap();
    assert_eq!(
        batch.items,
        vec![
            row(2, vec![], vec![], 0, true),
            row(4, vec![t_end(5)], vec![], 0, true),
        ]
    );
    assert_eq!(batch.deferred, summary(0, 0));
    // The limit=1 row is the scoped `next_task` answer: U2, not the
    // induced order's first id U1.
    assert_eq!(
        graph.next_task_of_epic(&epic_id(2)).unwrap().task,
        Some(task_id(2))
    );
}

#[test]
fn items_epic_scope_walks_the_induced_order() {
    // t27's T19(i) shape at the batch boundary: E2 holds U5 and U6,
    // E3's closed U7 blocks U5. The induced order of E2 is [U5, U6]
    // while the repo order filtered to E2 is [U6, U5] — so a
    // filtered-walk implementation returns the reversed batch.
    let mut graph = graph_with_epics(3);
    graph.insert_task(task_node(5, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(6, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(7, 3, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(7, 5)).unwrap();

    assert_eq!(
        graph.canonical_order(),
        vec![task_id(6), task_id(7), task_id(5)],
        "the repo-wide order the filtered walk would use"
    );
    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(2)).unwrap(),
        vec![task_id(5), task_id(6)],
        "the induced order the epic scope must walk"
    );
    let batch = graph.next_items_of_epic(&epic_id(2), u64::MAX).unwrap();
    assert_eq!(
        batch.items,
        vec![
            row(5, vec![t_end(7)], vec![], 0, true),
            row(6, vec![], vec![], 0, true),
        ],
        "the epic batch diverges from the filtered repo order"
    );
}

// ---------------------------------------------------------------------
// 12./13./14./15. The summary's scope, self-exclusion, errors, values
// ---------------------------------------------------------------------

#[test]
fn items_task_scope_summary_is_over_the_dependents() {
    // U1 open → U2 open; U4 closed → U3 deferred, outside U1's
    // dependents. D9's consequence: the task scope reports {0, 0}
    // while the repo reports the real counts.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(3, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(4, 1, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    graph.link_blocks(task_edge(4, 3)).unwrap();

    assert_eq!(graph.next_items(u64::MAX).deferred, summary(1, 1));
    assert_eq!(graph.next_task().deferred, summary(1, 1));
    let scoped = graph.next_items_of_task(&task_id(1), u64::MAX).unwrap();
    assert_eq!(scoped.deferred, summary(0, 0));
    assert_eq!(ids(&scoped), vec![task_id(2)]);

    // A deferred task *inside* the dependents moves the task scope's
    // summary: U2 deferred under its still-open predecessor U1 is
    // counted but not ready.
    graph
        .update_task(task_id(2), at(20), |task| {
            task.status = Status::Deferred;
            task.note = Some("until U1 lands".to_owned());
        })
        .unwrap();
    let scoped = graph.next_items_of_task(&task_id(1), u64::MAX).unwrap();
    assert_eq!(scoped.deferred, summary(1, 0));
    assert_eq!(graph.next_items(u64::MAX).deferred, summary(2, 1));
}

#[test]
fn items_the_queried_task_is_not_its_own_dependent() {
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();

    // The closure is irreflexive: U1's scope is [U2], never [U1, U2].
    let batch = graph.next_items_of_task(&task_id(1), u64::MAX).unwrap();
    assert_eq!(ids(&batch), vec![task_id(2)]);
    assert!(!ids(&batch).contains(&task_id(1)));
    assert_eq!(batch.items, vec![row(2, vec![t_end(1)], vec![], 0, true)]);

    // A task without dependents answers an empty list with the summary
    // — a value, not an error.
    let leaf = graph.next_items_of_task(&task_id(2), u64::MAX).unwrap();
    assert!(leaf.items.is_empty());
    assert_eq!(leaf.deferred, summary(0, 0));
}

#[test]
fn items_unknown_epic_and_task_are_unknown_id() {
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    let before = digest(&graph);

    let epic = graph.next_items_of_epic(&epic_id(9), 10);
    assert_eq!(outcome(&epic), "unknown_id");
    assert_eq!(
        epic,
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        }),
        "the payload must name the id"
    );

    let task = graph.next_items_of_task(&task_id(99), 10);
    assert_eq!(outcome(&task), "unknown_id");
    assert_eq!(
        task,
        Err(DomainError::UnknownId {
            id: "camera:U99".to_owned()
        }),
        "the payload must name the id"
    );

    assert_eq!(digest(&graph), before, "a rejected read touched state");
    // The repo-scoped answer on the same graph is unaffected.
    assert_eq!(ids(&graph.next_items(10)), vec![task_id(1)]);
}

#[test]
fn items_empty_and_degenerate_scopes_are_values() {
    // The empty graph.
    let empty = empty_graph();
    assert_eq!(
        empty.next_items(10),
        NextItems {
            items: Vec::new(),
            deferred: summary(0, 0),
        }
    );

    // An epic without tasks, both list entry points.
    let taskless = graph_with_epics(1);
    assert!(taskless.next_items(10).items.is_empty());
    assert_eq!(
        taskless.next_items_of_epic(&epic_id(1), 10).unwrap(),
        NextItems {
            items: Vec::new(),
            deferred: summary(0, 0),
        }
    );

    // A repo where every task is terminal, all three entry points. No
    // edges, so both tasks are also without dependents: the task scope
    // answers empty for the same reason the other two do. (A terminal
    // task *with* dependents would still list them, marked — task scope
    // does not filter; `items_task_scope_returns_held_dependents_and_marks_them`
    // pins that half.)
    let mut done = graph_with_epics(1);
    done.insert_task(task_node(1, 1, Status::Closed)).unwrap();
    done.insert_task(task_node(2, 1, Status::Wontfix)).unwrap();
    assert!(done.next_items(10).items.is_empty());
    assert_eq!(
        done.next_items_of_epic(&epic_id(1), 10).unwrap(),
        NextItems {
            items: Vec::new(),
            deferred: summary(0, 0),
        }
    );
    assert_eq!(
        done.next_items_of_task(&task_id(1), 10).unwrap(),
        NextItems {
            items: Vec::new(),
            deferred: summary(0, 0),
        }
    );
}

// ---------------------------------------------------------------------
// 16./17./18. Seeded graphs: agreement with an independent naive
//             recomputation, the cross-pin against next_task, and the
//             coverage pin that keeps both non-vacuous
// ---------------------------------------------------------------------

/// The three scopes as the suite enumerates them.
#[derive(Debug, Clone)]
enum Scope {
    Repo,
    Epic(EpicId),
    Task(TaskId),
}

fn scope_label(scope: &Scope) -> String {
    match scope {
        Scope::Repo => "repo".to_owned(),
        Scope::Epic(epic) => epic.to_string(),
        Scope::Task(task) => task.to_string(),
    }
}

/// Every scope the agreement test walks: the repo, each epic, and each
/// task that has dependents.
fn all_scopes(graph: &Graph) -> Vec<Scope> {
    let mut scopes = vec![Scope::Repo];
    scopes.extend(graph.epics().map(|epic| Scope::Epic(epic.id.clone())));
    scopes.extend(
        graph
            .tasks()
            .map(|task| task.id.clone())
            .filter(|id| !naive_dependents(graph, id).is_empty())
            .map(Scope::Task),
    );
    scopes
}

fn production_items(graph: &Graph, scope: &Scope, limit: u64) -> NextItems {
    match scope {
        Scope::Repo => graph.next_items(limit),
        Scope::Epic(epic) => graph.next_items_of_epic(epic, limit).unwrap(),
        Scope::Task(root) => graph.next_items_of_task(root, limit).unwrap(),
    }
}

#[test]
fn items_seeded_graphs_agree_with_an_independent_recomputation() {
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, ITEMS_STEPS);
        assert_invariants(&graph);

        for scope in all_scopes(&graph) {
            for limit in [LIMIT, u64::MAX] {
                check_scope(&graph, &scope, limit, seed);
            }
        }

        // The same seed replayed gives the same batches.
        let (_again, replayed) = run_sequence(seed, ITEMS_STEPS);
        assert_eq!(
            replayed.next_items(LIMIT),
            graph.next_items(LIMIT),
            "seed {seed} diverged on replay"
        );
    }
}

/// Asserts the agreement clauses for one scope and limit: the whole
/// answer equals the naive recomputation, the row positions ascend in
/// the scope's derived list, and — in the two scopes `next_task` also
/// answers — the deferred summary equals `next_task`'s (D9's invariant,
/// pinned rather than argued).
fn check_scope(graph: &Graph, scope: &Scope, limit: u64, seed: u64) {
    let label = scope_label(scope);
    let answer = production_items(graph, scope, limit);
    let expected = NextItems {
        items: naive_rows(graph, scope, limit),
        deferred: naive_summary(graph, scope),
    };
    assert_eq!(
        answer, expected,
        "seed {seed}: scope {label} limit {limit}: answers disagree"
    );

    // The ids are the scope list's prefix: every row appears in the
    // derived list, at strictly ascending positions.
    let list = naive_scope_list(graph, scope);
    let mut previous = None;
    for item in &answer.items {
        let Some(position) = list.iter().position(|id| *id == item.task) else {
            panic!(
                "seed {seed}: scope {label}: row {} left the scope",
                item.task
            );
        };
        assert!(
            previous.is_none_or(|last: usize| last < position),
            "seed {seed}: scope {label}: row {} broke the order",
            item.task
        );
        previous = Some(position);
    }

    // The summary is `next_task`'s for the same scope, wherever
    // `next_task` has one.
    match scope {
        Scope::Repo => assert_eq!(
            answer.deferred,
            graph.next_task().deferred,
            "seed {seed}: repo summaries disagree with next_task"
        ),
        Scope::Epic(epic) => assert_eq!(
            answer.deferred,
            graph.next_task_of_epic(epic).unwrap().deferred,
            "seed {seed}: scope {label} summaries disagree with next_task"
        ),
        Scope::Task(_) => {}
    }
}

/// The deliberately naive second implementation, written out longhand
/// the way `naive_select` is: it resolves every predecessor, every
/// status and the dependents closure by hand through the public
/// accessors and shares no predicate and no walk with production — a
/// recomputation that called [`Graph::is_actionable`] or reused the
/// production dependents walk would prove nothing. The closure is a
/// fixpoint over the whole edge listing, a different algorithm from the
/// production depth-first walk on purpose. The *order* is the
/// production one by design: `tests/order.rs` pins it independently,
/// and the batch contract is a prefix of that list.
fn naive_rows(graph: &Graph, scope: &Scope, limit: u64) -> Vec<NextItem> {
    let list = naive_scope_list(graph, scope);
    let actionable_only = !matches!(scope, Scope::Task(_));
    let mut rows = Vec::new();
    let mut remaining = limit;
    for id in &list {
        if remaining == 0 {
            break;
        }
        let actionable = naive_actionable(graph, id);
        if actionable_only && !actionable {
            continue;
        }
        let Some(task) = graph.task(id) else { continue };
        let mut blocked_by = graph.blocked_by(&BlockEnd::Task(id.clone()));
        for gate in graph.blocked_by(&BlockEnd::Epic(task.epic.clone())) {
            if !blocked_by.contains(&gate) {
                blocked_by.push(gate);
            }
        }
        let mut blocks = Vec::new();
        for end in graph.blocks(&BlockEnd::Task(id.clone())) {
            if let BlockEnd::Task(next) = end {
                blocks.push(next);
            }
        }
        rows.push(NextItem {
            task: id.clone(),
            blocked_by,
            blocks,
            weight: naive_dependents(graph, id).len() as u64,
            actionable,
        });
        remaining -= 1;
    }
    rows
}

fn naive_summary(graph: &Graph, scope: &Scope) -> DeferredSummary {
    let mut deferred = DeferredSummary { total: 0, ready: 0 };
    for id in naive_scope_list(graph, scope) {
        let Some(task) = graph.task(&id) else {
            continue;
        };
        if task.status != Status::Deferred {
            continue;
        }
        deferred.total += 1;
        deferred.ready += u64::from(naive_until_satisfied(graph, task));
    }
    deferred
}

/// The scope's derived id list, as the module docs define it: the
/// canonical order for the repo, the induced order for an epic, and the
/// repo order restricted to the irreflexive dependents closure for a
/// task scope.
fn naive_scope_list(graph: &Graph, scope: &Scope) -> Vec<TaskId> {
    match scope {
        Scope::Repo => graph.canonical_order(),
        Scope::Epic(epic) => graph.canonical_order_of_epic(epic).unwrap(),
        Scope::Task(root) => {
            let dependents = naive_dependents(graph, root);
            graph
                .canonical_order()
                .into_iter()
                .filter(|id| dependents.contains(id))
                .collect()
        }
    }
}

/// The transitive dependents of `root`, excluding `root`, by fixpoint
/// over the whole edge listing — deliberately not the production walk.
fn naive_dependents(graph: &Graph, root: &TaskId) -> BTreeSet<TaskId> {
    let edges = graph.blocks_edges();
    let mut set: BTreeSet<TaskId> = BTreeSet::new();
    loop {
        let before = set.len();
        for edge in &edges {
            if let BlocksEdge::TaskToTask { from, to } = edge
                && (from == root || set.contains(from))
                && to != root
            {
                set.insert(to.clone());
            }
        }
        if set.len() == before {
            break;
        }
    }
    set
}

fn naive_actionable(graph: &Graph, id: &TaskId) -> bool {
    let Some(task) = graph.task(id) else {
        return false;
    };
    let live = task.status == Status::Open || task.status == Status::InWork;
    live && naive_until_satisfied(graph, task)
}

fn naive_until_satisfied(graph: &Graph, task: &Task) -> bool {
    for end in graph.blocked_by(&BlockEnd::Task(task.id.clone())) {
        if !naive_end_finished(graph, &end) {
            return false;
        }
    }
    // §5.1's epic gate, consulted by hand.
    for end in graph.blocked_by(&BlockEnd::Epic(task.epic.clone())) {
        if !naive_end_finished(graph, &end) {
            return false;
        }
    }
    true
}

fn naive_end_finished(graph: &Graph, end: &BlockEnd) -> bool {
    match end {
        BlockEnd::Task(id) => match graph.task(id) {
            Some(predecessor) => {
                predecessor.status == Status::Closed || predecessor.status == Status::Wontfix
            }
            None => false,
        },
        BlockEnd::Decision(id) => match graph.decision(id) {
            Some(decision) => {
                decision.status == DecisionStatus::Signed
                    || decision.status == DecisionStatus::Superseded
            }
            None => false,
        },
        BlockEnd::Epic(_) => false,
    }
}

#[test]
fn items_next_task_agrees_with_the_first_row() {
    // The hand-built half, stated so it can fail: the first actionable
    // task is *not* the heaviest one — U1 (weight 0) leads, U3 (weight
    // 1, through U4) follows — so a weight-ranked batch answers U3 and
    // breaks the pairing. U2 closed → U3 open → U4 open.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(2, 3)).unwrap();
    graph.link_blocks(task_edge(3, 4)).unwrap();

    let batch = graph.next_items(u64::MAX);
    assert_eq!(batch.items.len(), 2);
    assert_eq!(batch.items[0].weight, 0);
    assert_eq!(batch.items[1].weight, 1);
    assert_eq!(
        graph.next_task().task,
        batch.items.first().map(|item| item.task.clone())
    );
    assert_eq!(graph.next_task().task, Some(task_id(1)));
    assert_eq!(
        graph
            .next_items_of_epic(&epic_id(1), u64::MAX)
            .unwrap()
            .items,
        batch.items
    );

    // The seeded half: in the two scopes whose populations coincide,
    // `next_task`'s answer is the first row whenever either is
    // non-empty — asserted as an equality of `Option`s, so the
    // None/empty pairing is the same assertion's other half.
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, ITEMS_STEPS);
        let next = graph.next_task();
        let batch = graph.next_items(u64::MAX);
        assert_eq!(
            next.task,
            batch.items.first().map(|item| item.task.clone()),
            "seed {seed}: repo scope"
        );
        for epic in graph
            .epics()
            .map(|epic| epic.id.clone())
            .collect::<Vec<_>>()
        {
            let next = graph.next_task_of_epic(&epic).unwrap();
            let batch = graph.next_items_of_epic(&epic, u64::MAX).unwrap();
            assert_eq!(
                next.task,
                batch.items.first().map(|item| item.task.clone()),
                "seed {seed}: epic scope {epic}"
            );
        }
    }
}

/// What the coverage pin observed on **one seed**, computed from the
/// built graphs through the naive helpers. Every counter counts a shape
/// the agreement test's assertions can observe: rows come from the
/// unlimited naive batch, which the agreement test compares against at
/// both limits, and truncation is exactly the difference between the
/// two.
#[derive(Debug, Default)]
struct Coverage {
    weight_zero_rows: usize,
    weight_heavy_rows: usize,
    task_scopes_mixed: usize,
    rows_with_task_end: usize,
    rows_with_decision_end: usize,
    rows_with_epic_gate: usize,
    truncated_scopes: usize,
    untruncated_scopes: usize,
    empty_from_nonempty: usize,
}

impl Coverage {
    fn note(&mut self, graph: &Graph, scope: &Scope, limit: u64) {
        let list = naive_scope_list(graph, scope);
        let rows = naive_rows(graph, scope, u64::MAX);
        for row in &rows {
            if row.weight == 0 {
                self.weight_zero_rows += 1;
            }
            if row.weight > 1 {
                self.weight_heavy_rows += 1;
            }
            if row
                .blocked_by
                .iter()
                .any(|end| matches!(end, BlockEnd::Task(_)))
            {
                self.rows_with_task_end += 1;
            }
            if row
                .blocked_by
                .iter()
                .any(|end| matches!(end, BlockEnd::Decision(_)))
            {
                self.rows_with_decision_end += 1;
            }
            // The shape that makes the agreement test sensitive to
            // mutation M5: a predecessor end that reached the row
            // *only* through the epic's gate list. A decision that also
            // gates the task directly would survive the mutation on the
            // row's own half, so it does not count here.
            if let Some(task) = graph.task(&row.task) {
                let own = graph.blocked_by(&BlockEnd::Task(row.task.clone()));
                let gates = graph.blocked_by(&BlockEnd::Epic(task.epic.clone()));
                if gates
                    .iter()
                    .any(|gate| !own.contains(gate) && row.blocked_by.contains(gate))
                {
                    self.rows_with_epic_gate += 1;
                }
            }
        }
        if matches!(scope, Scope::Task(_))
            && rows.iter().any(|row| row.actionable)
            && rows.iter().any(|row| !row.actionable)
        {
            self.task_scopes_mixed += 1;
        }
        let eligible = rows.len() as u64;
        if eligible > limit {
            self.truncated_scopes += 1;
        }
        if eligible > 0 && eligible <= limit {
            self.untruncated_scopes += 1;
        }
        if !list.is_empty() && rows.is_empty() {
            self.empty_from_nonempty += 1;
        }
    }
}

#[test]
fn items_seeded_coverage() {
    // f9's lesson, and the order suite's item (2): a property test
    // whose interesting branches the generator never reaches is green
    // and vacuous. Every count is computed from the built graphs
    // through the naive helpers, never from the draws and never through
    // the production predicates — so this pin survives exactly the
    // mutations it is meant to expose. The assertions are per seed: a
    // generator change that narrows one seed's graphs fails here
    // instead of hiding behind the other four.
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, ITEMS_STEPS);
        let mut coverage = Coverage::default();
        for scope in all_scopes(&graph) {
            coverage.note(&graph, &scope, LIMIT);
        }
        assert!(
            coverage.weight_zero_rows > 0,
            "seed {seed}: no returned row ever had weight 0: {coverage:#?}"
        );
        assert!(
            coverage.weight_heavy_rows > 0,
            "seed {seed}: no returned row ever had weight above 1, so the node-vs-path \
             pin only sees trivial closures: {coverage:#?}"
        );
        assert!(
            coverage.task_scopes_mixed > 0,
            "seed {seed}: no task scope ever held both an actionable and a held \
             dependent, so D4's marking is untested: {coverage:#?}"
        );
        assert!(
            coverage.rows_with_task_end > 0,
            "seed {seed}: no returned row ever listed a task predecessor: {coverage:#?}"
        );
        assert!(
            coverage.rows_with_decision_end > 0,
            "seed {seed}: no returned row ever listed a decision gate: {coverage:#?}"
        );
        assert!(
            coverage.rows_with_epic_gate > 0,
            "seed {seed}: no returned row ever listed a predecessor that came only \
             through its epic's gate, so D6's second reading is untested: {coverage:#?}"
        );
        assert!(
            coverage.truncated_scopes > 0,
            "seed {seed}: no scope was ever truncated by the limit: {coverage:#?}"
        );
        assert!(
            coverage.untruncated_scopes > 0,
            "seed {seed}: no non-empty scope ever fit under the limit: {coverage:#?}"
        );
        assert!(
            coverage.empty_from_nonempty > 0,
            "seed {seed}: no non-empty scope ever answered an empty batch: {coverage:#?}"
        );
    }
}

// ---------------------------------------------------------------------
// 19. State, not history: reruns and shuffled insertions
// ---------------------------------------------------------------------

/// One half of the build spec: (task number, epic number, status)
/// nodes.
type SpecNodes = Vec<(u64, u64, Status)>;
/// The other half: (from, to) task edges, acyclic by construction. The
/// decision and its two gate edges are fixed — shuffling applies to the
/// task nodes and the task edges.
type SpecPairs = Vec<(u64, u64)>;

fn sample_spec() -> (SpecNodes, SpecPairs) {
    (
        vec![
            (1, 1, Status::Open),
            (2, 1, Status::Closed),
            (3, 1, Status::Deferred),
            (4, 2, Status::Open),
            (5, 2, Status::InWork),
            (6, 3, Status::Open),
            (7, 3, Status::Open),
        ],
        vec![(2, 3), (4, 5), (6, 7)],
    )
}

/// Builds the sample state: three epics, one open decision gating E2
/// and, directly, U5 — so U5 is gated twice, once through each adjacency
/// list — the spec's tasks and edges. U3 is deferred-ready (U2 closed),
/// U4 is held by the epic gate alone, U5 by U4 and both gates, U7 by U6
/// — the repo batch is [U1, U6]. The direct gate lands after the task
/// inserts because [`Graph::link_blocks`] requires its endpoints to
/// exist.
fn build_spec(nodes: &[(u64, u64, Status)], edges: &[(u64, u64)]) -> Graph {
    let mut graph = graph_with_epics(3);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.link_blocks(gate_epic(1, 2)).unwrap();
    for (task, epic, status) in nodes {
        graph.insert_task(task_node(*task, *epic, *status)).unwrap();
    }
    graph.link_blocks(gate_task(1, 5)).unwrap();
    for (from, to) in edges {
        graph.link_blocks(task_edge(*from, *to)).unwrap();
    }
    graph
}

/// Fisher–Yates under the seeded splitmix64: reproducible shuffles, no
/// `rand` dependency.
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        items.swap(i, rng.below(i + 1));
    }
}

/// Every answer of the sample state, for the twice-run and shuffled-
/// rebuild comparisons.
fn all_answers(graph: &Graph) -> Vec<NextItems> {
    let mut answers = vec![graph.next_items(LIMIT), graph.next_items(u64::MAX)];
    for epic in 1..=3 {
        answers.push(graph.next_items_of_epic(&epic_id(epic), u64::MAX).unwrap());
    }
    for task in 1..=7 {
        answers.push(graph.next_items_of_task(&task_id(task), u64::MAX).unwrap());
    }
    answers
}

#[test]
fn items_is_a_function_of_state_across_reruns_and_shuffled_insertion() {
    let (nodes, edges) = sample_spec();
    let graph = build_spec(&nodes, &edges);

    // The expected answers, spelled out: the suite must know what the
    // state means, not only that two runs agree.
    assert_eq!(
        graph.canonical_order(),
        vec![
            task_id(1),
            task_id(2),
            task_id(3),
            task_id(4),
            task_id(5),
            task_id(6),
            task_id(7)
        ]
    );
    assert_eq!(
        graph.next_items(u64::MAX),
        NextItems {
            items: vec![
                row(1, vec![], vec![], 0, true),
                row(6, vec![], vec![7], 1, true),
            ],
            deferred: summary(1, 1),
        },
        "repo scope"
    );
    assert_eq!(
        graph.next_items_of_epic(&epic_id(1), u64::MAX).unwrap(),
        NextItems {
            items: vec![row(1, vec![], vec![], 0, true)],
            deferred: summary(1, 1),
        },
        "E1"
    );
    assert_eq!(
        graph.next_items_of_epic(&epic_id(2), u64::MAX).unwrap(),
        NextItems {
            items: Vec::new(),
            deferred: summary(0, 0),
        },
        "E2: the open gate holds every task under it"
    );
    assert_eq!(
        graph.next_items_of_epic(&epic_id(3), u64::MAX).unwrap(),
        NextItems {
            items: vec![row(6, vec![], vec![7], 1, true)],
            deferred: summary(0, 0),
        },
        "E3"
    );
    // U2's dependents: the deferred U3, held-marked, ready in the
    // summary (U2 is closed).
    assert_eq!(
        graph.next_items_of_task(&task_id(2), u64::MAX).unwrap(),
        NextItems {
            items: vec![row(3, vec![t_end(2)], vec![], 0, false)],
            deferred: summary(1, 1),
        },
        "task scope U2"
    );
    // U4's dependents: U5, whose annotation carries the epic's gate
    // beside its own predecessor — D6's second reading, hand-pinned.
    // D1 gates U5 twice (directly and through E2) and is listed once:
    // the first-occurrence dedup of judgment call (k), hand-pinned.
    assert_eq!(
        graph.next_items_of_task(&task_id(4), u64::MAX).unwrap(),
        NextItems {
            items: vec![row(5, vec![t_end(4), d_end(1)], vec![], 0, false)],
            deferred: summary(0, 0),
        },
        "task scope U4"
    );
    assert_eq!(
        graph.next_items_of_task(&task_id(6), u64::MAX).unwrap(),
        NextItems {
            items: vec![row(7, vec![t_end(6)], vec![], 0, false)],
            deferred: summary(0, 0),
        },
        "task scope U6"
    );
    // U1 has no dependents: an empty scope, not an error.
    assert!(
        graph
            .next_items_of_task(&task_id(1), u64::MAX)
            .unwrap()
            .items
            .is_empty()
    );

    let answers = all_answers(&graph);
    // Twice-run identity on unchanged state (§8).
    assert_eq!(all_answers(&graph), answers);

    // Isomorphic rebuilds whose nodes and edges land in seeded-shuffled
    // sequences: proven the same state by fingerprint, so the same
    // answers — the batch reads state, never history.
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let mut shuffled_nodes = nodes.clone();
        let mut shuffled_edges = edges.clone();
        shuffle(&mut rng, &mut shuffled_nodes);
        shuffle(&mut rng, &mut shuffled_edges);
        let rebuilt = build_spec(&shuffled_nodes, &shuffled_edges);
        assert_eq!(
            fingerprint(&rebuilt),
            fingerprint(&graph),
            "seed {seed} built a different state"
        );
        assert_eq!(
            all_answers(&rebuilt),
            answers,
            "seed {seed} answered differently"
        );
    }
}
