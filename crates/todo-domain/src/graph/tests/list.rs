//! The listing tests (spec §5.2/§6, §10): the four filters and their
//! AND, the canonical order of every answer, the deferred-scoped
//! `ready` reading, the epic filter's induced order, totality — empty
//! answers and the one `unknown_id` — the three thin listings'
//! ascending-id contract, identical answers across reruns and shuffled
//! insertions, seeded agreement with an independent naive
//! recomputation, and the listing/summary invariant.
//!
//! The naive recomputation walks the *production* canonical order —
//! production's by design, `tests/order.rs` pins it independently, and
//! `naive_select` in `tests/select.rs` documents the same choice — and
//! hand-filters by status, kind and deferred readiness with the shared
//! naive predicates from the parent module, never with production's
//! `is_until_satisfied`/`is_actionable` and never with `task_list`
//! itself.

use std::collections::BTreeMap;

use crate::testcodec::to_bytes;
use crate::{Blocker, DeferredSummary, TaskFilter, TaskRow};

use super::seeded::{Rng, run_sequence};
use super::*;

/// Fixed seeds for the insertion shuffles, per the suite's rule that a
/// determinism test never reseeds from a clock.
const SEEDS: [u64; 5] = [19, 53, 0x1F1F, 0x1157_93DF, 0xBADC_0FFE];
/// Fixed seeds for the mutation-sequence graphs the agreement and
/// coverage tests run over. The coverage pin asserts per seed, so each
/// seed's final graph must on its own reach every branch the pin
/// counts — a generator change that narrows one seed fails there
/// rather than hiding behind the others. The seed set and the step
/// depth below are this suite's own and deliberately not the selection
/// suite's: this pin counts whole-graph vocabulary — every status and
/// every kind present in one final graph, ready and held deferred
/// items, and a deferred task held by its epic's decision gate alone
/// beside them — and the gate shape in particular is rare and not
/// monotone in depth (the generator's unlink/delete churn can remove
/// it again), so the seeds are chosen by measurement to satisfy every
/// branch at this depth, and seeds that satisfy one suite's pin need
/// not satisfy another's.
const GRAPH_SEEDS: [u64; 5] = [9, 24, 36, 44, 60];
/// Steps per seeded graph: deep enough that the coverage pin below
/// finds every branch, shallow enough that the per-step invariant scan
/// stays cheap. Deeper than the selection suite's for the reason the
/// seed set above records.
const LIST_STEPS: u64 = 600;

// ---------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------

/// `task_list` unwrapped, its rows read down to their task ids: every
/// call through this helper is on a graph that holds the queried
/// scope, so the unwrap is shorthand, not a hidden expectation — the
/// one `unknown_id` case asserts on the whole `Result` instead. The
/// rows' `blocked_by` payload is `summary`'s suite to pin; this suite
/// pins the filters and the order.
fn listed(graph: &Graph, filter: &TaskFilter) -> Vec<TaskId> {
    graph
        .task_list(filter)
        .unwrap()
        .into_iter()
        .map(|row| row.task)
        .collect()
}

/// The fixed filter matrix the seeded tests walk: the default, each
/// status, each kind, the deferred readiness pair, the contradiction,
/// and status∧kind conjunctions — applied under every scope, so the
/// walk exercises epic∧status∧kind triples too.
fn filter_matrix() -> Vec<TaskFilter> {
    let mut matrix = vec![TaskFilter::default()];
    for status in [
        Status::Open,
        Status::InWork,
        Status::Blocked,
        Status::Deferred,
        Status::Closed,
        Status::Wontfix,
    ] {
        matrix.push(TaskFilter {
            status: Some(status),
            ..Default::default()
        });
    }
    for kind in [
        TaskKind::Task,
        TaskKind::Gate,
        TaskKind::Finding,
        TaskKind::Debt,
    ] {
        matrix.push(TaskFilter {
            kind: Some(kind),
            ..Default::default()
        });
    }
    matrix.push(TaskFilter {
        status: Some(Status::Deferred),
        ready: Some(true),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Deferred),
        ready: Some(false),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        ready: Some(true),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        ready: Some(false),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Open),
        ready: Some(true),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Open),
        ready: Some(false),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Open),
        kind: Some(TaskKind::Task),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Deferred),
        kind: Some(TaskKind::Gate),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Closed),
        kind: Some(TaskKind::Task),
        ..Default::default()
    });
    matrix.push(TaskFilter {
        status: Some(Status::Deferred),
        kind: Some(TaskKind::Gate),
        ready: Some(true),
        ..Default::default()
    });
    matrix
}

/// The deliberately naive second implementation, written out longhand
/// the way `naive_select` is: it walks the *production* canonical order
/// — production's by design, `tests/order.rs` pins it independently —
/// and hand-filters by status, kind and deferred readiness using the
/// shared naive predicates. It must not call `task_list`,
/// `is_until_satisfied` or `is_actionable`: an agreement that shared a
/// predicate with production would prove nothing.
fn naive_task_list(graph: &Graph, filter: &TaskFilter) -> Vec<TaskId> {
    let order = match &filter.epic {
        None => graph.canonical_order(),
        Some(epic) => graph.canonical_order_of_epic(epic).unwrap(),
    };
    let mut out = Vec::new();
    for id in &order {
        let Some(task) = graph.task(id) else { continue };
        if let Some(status) = filter.status
            && task.status != status
        {
            continue;
        }
        if let Some(kind) = filter.kind
            && task.kind != kind
        {
            continue;
        }
        if let Some(ready) = filter.ready {
            // The deferred-scoped reading, spelled out by hand: a
            // non-deferred task is never listed under `ready`, and the
            // readiness half is the naive predicate, not production's.
            if task.status != Status::Deferred || naive_until_satisfied(graph, task) != ready {
                continue;
            }
        }
        out.push(id.clone());
    }
    out
}

/// One scope's D8 equalities: the deferred listing's length is the
/// summary's `total`, and the readiness refinements' lengths are
/// `ready` and `total - ready`.
fn assert_counts(graph: &Graph, epic: Option<&EpicId>, summary: &DeferredSummary, label: &str) {
    let deferred = listed(
        graph,
        &TaskFilter {
            epic: epic.cloned(),
            status: Some(Status::Deferred),
            ..Default::default()
        },
    );
    let ready = listed(
        graph,
        &TaskFilter {
            epic: epic.cloned(),
            status: Some(Status::Deferred),
            ready: Some(true),
            ..Default::default()
        },
    );
    let held = listed(
        graph,
        &TaskFilter {
            epic: epic.cloned(),
            status: Some(Status::Deferred),
            ready: Some(false),
            ..Default::default()
        },
    );
    assert_eq!(deferred.len() as u64, summary.total, "{label}: total");
    assert_eq!(ready.len() as u64, summary.ready, "{label}: ready");
    assert_eq!(
        held.len() as u64,
        summary.total - summary.ready,
        "{label}: held"
    );
    // The two refinements partition the deferred listing.
    assert_eq!(
        ready.len() + held.len(),
        deferred.len(),
        "{label}: partition"
    );
}

/// Fisher–Yates under the seeded splitmix64: reproducible shuffles, no
/// `rand` dependency.
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        items.swap(i, rng.below(i + 1));
    }
}

// ---------------------------------------------------------------------
// 1. The unfiltered listing is the canonical order
// ---------------------------------------------------------------------

#[test]
fn list_no_filter_is_the_whole_canonical_order() {
    // D3/D5's structural claim: the unfiltered listing *is* the
    // canonical order. A graph whose canonical order is not ascending
    // id, so the pin is about order, not membership: closed U5 blocks
    // U1 — [U3, U5, U1].
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(5, 1, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(5, 1)).unwrap();

    let order = graph.canonical_order();
    let mut ascending = order.clone();
    ascending.sort();
    assert_ne!(
        order, ascending,
        "the pin needs an order that is not ascending id"
    );
    assert_eq!(listed(&graph, &TaskFilter::default()), order);

    // The epic scope is the induced order, exactly as the filter's doc
    // says — here the same list, one epic holding every task.
    let induced = graph.canonical_order_of_epic(&epic_id(1)).unwrap();
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(1)),
                ..Default::default()
            }
        ),
        induced
    );
    assert_eq!(induced, order);
}

// ---------------------------------------------------------------------
// 2./3. The status filter and the kind filter, variant by variant
// ---------------------------------------------------------------------

#[test]
fn list_the_status_filter_selects_each_of_the_six() {
    // One task per status per epic, and one cross-epic edge (U7→U1)
    // that puts E2's open task ahead of E1's in canonical order:
    // [U2, U3, U4, U5, U6, U7, U1, U8, …, U12], so the Open row pins
    // order, not only membership.
    let mut graph = graph_with_epics(2);
    for (i, status) in [
        Status::Open,
        Status::InWork,
        Status::Blocked,
        Status::Deferred,
        Status::Closed,
        Status::Wontfix,
    ]
    .iter()
    .enumerate()
    {
        graph
            .insert_task(task_node(i as u64 + 1, 1, *status))
            .unwrap();
        graph
            .insert_task(task_node(i as u64 + 7, 2, *status))
            .unwrap();
    }
    graph.link_blocks(task_edge(7, 1)).unwrap();
    assert_eq!(
        graph.canonical_order(),
        [2u64, 3, 4, 5, 6, 7, 1, 8, 9, 10, 11, 12]
            .map(task_id)
            .to_vec()
    );

    for (status, expected) in [
        (Status::Open, vec![7u64, 1]),
        (Status::InWork, vec![2, 8]),
        (Status::Blocked, vec![3, 9]),
        (Status::Deferred, vec![4, 10]),
        (Status::Closed, vec![5, 11]),
        (Status::Wontfix, vec![6, 12]),
    ] {
        assert_eq!(
            listed(
                &graph,
                &TaskFilter {
                    status: Some(status),
                    ..Default::default()
                }
            ),
            expected.into_iter().map(task_id).collect::<Vec<_>>(),
            "{status} filter"
        );
    }
    // The Open row is the order pin: ascending id would answer [U1, U7].
    assert_ne!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Open),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(7)]
    );
}

#[test]
fn list_the_kind_filter_selects_each_of_the_four() {
    // One task per kind per epic and the cross-epic edge U5→U1, so the
    // canonical order is [U2, U3, U4, U5, U1, U6, U7, U8] and the Task
    // row — [U5, U1] — pins order, not only membership.
    let mut graph = graph_with_epics(2);
    for (i, kind) in [
        TaskKind::Task,
        TaskKind::Gate,
        TaskKind::Finding,
        TaskKind::Debt,
    ]
    .iter()
    .enumerate()
    {
        let mut first = task_node(i as u64 + 1, 1, Status::Open);
        first.kind = *kind;
        graph.insert_task(first).unwrap();
        let mut second = task_node(i as u64 + 5, 2, Status::Open);
        second.kind = *kind;
        graph.insert_task(second).unwrap();
    }
    graph.link_blocks(task_edge(5, 1)).unwrap();
    assert_eq!(
        graph.canonical_order(),
        [2u64, 3, 4, 5, 1, 6, 7, 8].map(task_id).to_vec()
    );

    for (kind, expected) in [
        (TaskKind::Task, vec![5u64, 1]),
        (TaskKind::Gate, vec![2, 6]),
        (TaskKind::Finding, vec![3, 7]),
        (TaskKind::Debt, vec![4, 8]),
    ] {
        assert_eq!(
            listed(
                &graph,
                &TaskFilter {
                    kind: Some(kind),
                    ..Default::default()
                }
            ),
            expected.into_iter().map(task_id).collect::<Vec<_>>(),
            "{kind} filter"
        );
    }
    assert_ne!(
        listed(
            &graph,
            &TaskFilter {
                kind: Some(TaskKind::Task),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(5)],
        "the Task row is the order pin"
    );
}

// ---------------------------------------------------------------------
// 4. `ready` is deferred-scoped — judgment call (g), mutation M3
// ---------------------------------------------------------------------

#[test]
fn list_ready_is_deferred_scoped() {
    // Judgment call (g)'s pin (reading (ii)). U1 deferred-ready (U4
    // closed), U2 deferred-held (U5 open), U3 open and
    // until-satisfied — the non-deferred task reading (i) would
    // smuggle into ready=true.
    let mut graph = graph_with_epics(1);
    graph
        .insert_task(task_node(1, 1, Status::Deferred))
        .unwrap();
    graph
        .insert_task(task_node(2, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(5, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(4, 1)).unwrap();
    graph.link_blocks(task_edge(5, 2)).unwrap();

    // The premise, named: U3 really is until-satisfied and really is
    // not deferred, so the assertions below can fail (f47's lesson:
    // assert the state, not only the answer).
    assert_eq!(graph.task(&task_id(3)).unwrap().status, Status::Open);
    assert!(graph.is_until_satisfied(&task_id(3)));
    assert!(graph.is_until_satisfied(&task_id(1)));
    assert!(!graph.is_until_satisfied(&task_id(2)));

    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(true),
                ..Default::default()
            }
        ),
        vec![task_id(1)],
        "a non-deferred until-satisfied task is not ready"
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(false),
                ..Default::default()
            }
        ),
        vec![task_id(2)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Deferred),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(2)],
        "ready=None applies no readiness refinement"
    );
}

// ---------------------------------------------------------------------
// 4b. `ready` consults both decision-gate halves
// ---------------------------------------------------------------------

#[test]
fn list_a_deferred_task_behind_a_decision_gate_is_not_ready() {
    // The decision-gate dimension of `ready`, pinned by hand so no
    // seeded margin carries it alone: both halves of the readiness
    // predicate — a decision gating the task itself and a decision
    // gating its epic — hold a deferred task out of ready=true and
    // inside ready=false, and signing the decision moves the answer.
    // The select suite pins the same dimension for the summary
    // (`select_a_deferred_task_under_a_gated_epic_is_not_ready`); this
    // is the listing's pin.
    //
    // E1 is gated by the open D1; U1 is deferred under E1 with no own
    // predecessors, so the epic's gate is the only thing holding it —
    // the arm a task-own-edges-only readiness cannot see. E2 is
    // ungated; U2 is deferred under E2 with the open D2 gating it
    // directly.
    let mut graph = graph_with_epics(2);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph
        .insert_decision(decision_node(2, DecisionStatus::Open))
        .unwrap();
    graph.link_blocks(gate_epic(1, 1)).unwrap();
    graph
        .insert_task(task_node(1, 1, Status::Deferred))
        .unwrap();
    graph
        .insert_task(task_node(2, 2, Status::Deferred))
        .unwrap();
    graph.link_blocks(gate_task(2, 2)).unwrap();

    // The premise, named: each gate lands where the test says it does
    // — U1 has no own predecessor at all — and both decisions are open,
    // so the assertions below can fail (f47's lesson: assert the state,
    // not only the answer).
    assert!(graph.blocked_by(&BlockEnd::Task(task_id(1))).is_empty());
    assert_eq!(
        graph.blocked_by(&BlockEnd::Epic(epic_id(1))),
        vec![BlockEnd::Decision(decision_id(1))]
    );
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(2))),
        vec![BlockEnd::Decision(decision_id(2))]
    );
    assert!(graph.blocked_by(&BlockEnd::Epic(epic_id(2))).is_empty());
    assert!(
        !graph.is_until_satisfied(&task_id(1)),
        "the epic gate holds U1"
    );
    assert!(
        !graph.is_until_satisfied(&task_id(2)),
        "the task gate holds U2"
    );

    // Both deferred, neither ready, and the listing agrees with the
    // summary under the gates (D8).
    let next = graph.next_task();
    assert_eq!(next.deferred, DeferredSummary { total: 2, ready: 0 });
    assert_counts(&graph, None, &next.deferred, "both gates open");
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(true),
                ..Default::default()
            }
        ),
        Vec::<TaskId>::new()
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(false),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(2)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Deferred),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(2)],
        "the gates hold readiness, not the deferred listing itself"
    );

    // Signing the epic's gate releases U1: the answer moves.
    graph
        .update_decision(decision_id(1), at(9), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed by the user".to_owned());
        })
        .unwrap();
    assert!(graph.is_until_satisfied(&task_id(1)));
    assert!(!graph.is_until_satisfied(&task_id(2)));
    let next = graph.next_task();
    assert_eq!(next.deferred, DeferredSummary { total: 2, ready: 1 });
    assert_counts(&graph, None, &next.deferred, "the epic gate signed");
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(true),
                ..Default::default()
            }
        ),
        vec![task_id(1)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(false),
                ..Default::default()
            }
        ),
        vec![task_id(2)]
    );

    // Signing the task's own gate releases U2 too; ready=false is now
    // the empty answer, and the epic scope reads the same gates.
    graph
        .update_decision(decision_id(2), at(10), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed by the user".to_owned());
        })
        .unwrap();
    assert!(graph.is_until_satisfied(&task_id(2)));
    let next = graph.next_task();
    assert_eq!(next.deferred, DeferredSummary { total: 2, ready: 2 });
    assert_counts(&graph, None, &next.deferred, "both gates signed");
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(true),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(2)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(false),
                ..Default::default()
            }
        ),
        Vec::<TaskId>::new()
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(1)),
                ready: Some(true),
                ..Default::default()
            }
        ),
        vec![task_id(1)],
        "the epic scope consults the same gates"
    );
}

// ---------------------------------------------------------------------
// 5. The epic filter walks the induced order — mutation M5
// ---------------------------------------------------------------------

#[test]
fn list_the_epic_filter_walks_the_induced_order() {
    // t27's T19(i) / f47's clause-(iii) shape: E2 holds U5 and U6
    // open, E3's closed U7 blocks U5. The induced order of E2 is
    // [U5, U6]; the repo order restricted to E2 is [U6, U5]. Both
    // orders are asserted beside the answer, so the test names which
    // order it means.
    let mut graph = graph_with_epics(3);
    graph.insert_task(task_node(5, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(6, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(7, 3, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(7, 5)).unwrap();

    let repo = graph.canonical_order();
    assert_eq!(
        repo,
        vec![task_id(6), task_id(7), task_id(5)],
        "the repo order a filtered walk would use"
    );
    let induced = graph.canonical_order_of_epic(&epic_id(2)).unwrap();
    assert_eq!(
        induced,
        vec![task_id(5), task_id(6)],
        "the induced order the epic filter must walk"
    );
    let restricted: Vec<TaskId> = repo
        .iter()
        .filter(|id| graph.task(id).unwrap().epic == epic_id(2))
        .cloned()
        .collect();
    assert_eq!(restricted, vec![task_id(6), task_id(5)]);
    assert_ne!(restricted, induced, "the pin needs the two to differ");

    // The answer is rows, in the induced order: U5's row carries its
    // closed cross-epic predecessor with its status, U6's none.
    assert_eq!(
        graph
            .task_list(&TaskFilter {
                epic: Some(epic_id(2)),
                ..Default::default()
            })
            .unwrap(),
        vec![
            TaskRow {
                task: task_id(5),
                blocked_by: vec![Blocker::Task {
                    id: task_id(7),
                    status: Status::Closed,
                }],
                priority: None,
            },
            TaskRow {
                task: task_id(6),
                blocked_by: Vec::new(),
                priority: None,
            },
        ],
        "the epic filter answers rows in the induced order"
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(2)),
                ..Default::default()
            }
        ),
        induced
    );
}

// ---------------------------------------------------------------------
// 6./7. The filters AND; contradictions and narrowings are values
// ---------------------------------------------------------------------

#[test]
fn list_filters_and_and_over_narrow_combinations_stay_values() {
    // E1: U1 deferred Gate and U3 deferred Task, both ready through
    // the closed U2; U4 open Gate. E2: U5 deferred Gate held by the
    // open U6; U7 closed. Canonical order [U2, U1, U3, U4, U6, U5, U7].
    let mut graph = graph_with_epics(2);
    let mut u1 = task_node(1, 1, Status::Deferred);
    u1.kind = TaskKind::Gate;
    graph.insert_task(u1).unwrap();
    graph.insert_task(task_node(2, 1, Status::Closed)).unwrap();
    graph
        .insert_task(task_node(3, 1, Status::Deferred))
        .unwrap();
    let mut u4 = task_node(4, 1, Status::Open);
    u4.kind = TaskKind::Gate;
    graph.insert_task(u4).unwrap();
    let mut u5 = task_node(5, 2, Status::Deferred);
    u5.kind = TaskKind::Gate;
    graph.insert_task(u5).unwrap();
    graph.insert_task(task_node(6, 2, Status::Open)).unwrap();
    graph.insert_task(task_node(7, 2, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(2, 1)).unwrap();
    graph.link_blocks(task_edge(2, 3)).unwrap();
    graph.link_blocks(task_edge(6, 5)).unwrap();
    assert_eq!(
        graph.canonical_order(),
        [2u64, 1, 3, 4, 6, 5, 7].map(task_id).to_vec()
    );

    // The four filters narrowing to a known non-empty subset.
    let narrow = TaskFilter {
        epic: Some(epic_id(1)),
        status: Some(Status::Deferred),
        kind: Some(TaskKind::Gate),
        ready: Some(true),
    };
    assert_eq!(listed(&graph, &narrow), vec![task_id(1)]);
    // The kind conjunct is the discriminating one in this conjunction:
    // dropping it widens to both deferred-ready tasks. Dropping `ready`
    // or `status` cannot widen it — `ready` already implies deferred
    // (judgment call (g)) and U1 is E1's only deferred Gate — so the
    // status filter gets its own discrimination row below.
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                kind: None,
                ..narrow.clone()
            }
        ),
        vec![task_id(1), task_id(3)],
        "the kind filter discriminates"
    );
    // The same conjunction over E2 matches nothing: an ordered empty
    // Ok, not an error.
    assert_eq!(
        graph.task_list(&TaskFilter {
            epic: Some(epic_id(2)),
            ..narrow
        }),
        Ok(vec![])
    );
    // The status filter discriminates too: E2's deferred listing is
    // U5 alone, where the unfiltered epic listing holds three tasks.
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(2)),
                status: Some(Status::Deferred),
                ..Default::default()
            }
        ),
        vec![task_id(5)]
    );
    // The empties are the narrowing, not an empty graph.
    assert_eq!(listed(&graph, &TaskFilter::default()).len(), 7);
}

#[test]
fn list_a_contradictory_filter_is_empty_not_an_error() {
    // D7's pin: `ready` implies deferred (judgment call (g)), so a
    // non-deferred status with `ready` is nothing — an Ok(empty),
    // never an error, because §6.1's list is closed and has no code
    // for a contradictory filter.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(2, 1, Status::Deferred))
        .unwrap();

    assert_eq!(
        graph.task_list(&TaskFilter {
            status: Some(Status::Open),
            ready: Some(true),
            ..Default::default()
        }),
        Ok(vec![])
    );
    // Every non-deferred status contradicts both readiness halves.
    for status in [
        Status::Open,
        Status::InWork,
        Status::Blocked,
        Status::Closed,
        Status::Wontfix,
    ] {
        for ready in [true, false] {
            assert_eq!(
                graph.task_list(&TaskFilter {
                    status: Some(status),
                    ready: Some(ready),
                    ..Default::default()
                }),
                Ok(vec![]),
                "{status} with ready={ready}"
            );
        }
    }
    // The empties are the contradiction, not an empty graph: looser
    // filters on the same state answer.
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Open),
                ..Default::default()
            }
        ),
        vec![task_id(1)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(true),
                ..Default::default()
            }
        ),
        vec![task_id(2)]
    );
}

// ---------------------------------------------------------------------
// 8. The one error
// ---------------------------------------------------------------------

#[test]
fn list_an_unknown_epic_is_unknown_id() {
    // D10's pin: the epic filter's unknown id is the only failure, and
    // the payload names the id — asserted whole, the way
    // `select_an_unknown_epic_is_unknown_id` does.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    let result = graph.task_list(&TaskFilter {
        epic: Some(epic_id(9)),
        ..Default::default()
    });
    assert_eq!(outcome(&result), "unknown_id");
    assert_eq!(
        result,
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        }),
        "the payload must name the id"
    );
    // Filters alongside the unknown epic change nothing: the error
    // comes from the order derivation, before any filter reads.
    assert_eq!(
        outcome(&graph.task_list(&TaskFilter {
            epic: Some(epic_id(9)),
            status: Some(Status::Open),
            ready: Some(true),
            ..Default::default()
        })),
        "unknown_id"
    );
    // The no-epic listing on the same graph is unaffected, and the
    // value filters never error.
    assert_eq!(listed(&graph, &TaskFilter::default()), vec![task_id(1)]);
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Closed),
                ..Default::default()
            }
        ),
        Vec::<TaskId>::new()
    );
}

// ---------------------------------------------------------------------
// 9. The order is canonical, not ascending id — mutation M1
// ---------------------------------------------------------------------

#[test]
fn list_the_order_is_canonical_not_ascending_id() {
    // t27's T19(ii) shape: U1 open, U3 open, U5 closed with U5
    // blocking U1 — canonical [U3, U5, U1] against ascending
    // [U1, U3, U5], so a sort-by-id implementation fails.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(5, 1, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(5, 1)).unwrap();

    let answer = listed(&graph, &TaskFilter::default());
    assert_eq!(answer, vec![task_id(3), task_id(5), task_id(1)]);
    let mut sorted = answer.clone();
    sorted.sort();
    assert_eq!(sorted, vec![task_id(1), task_id(3), task_id(5)]);
    assert_ne!(
        answer, sorted,
        "a sort-by-id implementation answers the sorted list"
    );
    // Under a filter too: the open tasks keep the order's sequence.
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Open),
                ..Default::default()
            }
        ),
        vec![task_id(3), task_id(1)]
    );
}

// ---------------------------------------------------------------------
// 10. The cross-module invariant — mutation M4
// ---------------------------------------------------------------------

#[test]
fn list_the_deferred_counts_equal_the_summary() {
    // D8's hand-built pin: the listing and the summary are two readers
    // of one contract, so their deferred counts must be equal, per
    // scope. E1: U1 deferred-ready (U4 closed), U2 deferred-held (U5
    // open), U3 open. E2: U6 deferred-held (U7 open), U7 open.
    let mut graph = graph_with_epics(2);
    graph
        .insert_task(task_node(1, 1, Status::Deferred))
        .unwrap();
    graph
        .insert_task(task_node(2, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(4, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(5, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(6, 2, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(7, 2, Status::Open)).unwrap();
    graph.link_blocks(task_edge(4, 1)).unwrap();
    graph.link_blocks(task_edge(5, 2)).unwrap();
    graph.link_blocks(task_edge(7, 6)).unwrap();

    // The repo scope's summary, spelled out — the suite must know what
    // the state means, not only that two surfaces agree.
    let next = graph.next_task();
    assert_eq!(next.deferred, DeferredSummary { total: 3, ready: 1 });
    assert_counts(&graph, None, &next.deferred, "repo");

    // Each epic scope.
    let e1 = graph.next_task_of_epic(&epic_id(1)).unwrap();
    assert_eq!(e1.deferred, DeferredSummary { total: 2, ready: 1 });
    assert_counts(&graph, Some(&epic_id(1)), &e1.deferred, "E1");
    let e2 = graph.next_task_of_epic(&epic_id(2)).unwrap();
    assert_eq!(e2.deferred, DeferredSummary { total: 1, ready: 0 });
    assert_counts(&graph, Some(&epic_id(2)), &e2.deferred, "E2");
}

// ---------------------------------------------------------------------
// 11./12./13. The three thin listings — mutation M6
// ---------------------------------------------------------------------

/// One insertion of the listing spec.
#[derive(Clone)]
enum Insert {
    Epic(u64),
    Decision(u64, DecisionStatus),
    Doc(u64),
}

/// The listing spec: three of each kind in a deliberately non-ascending
/// sequence, so an ascending answer cannot be an accident of insertion
/// order.
fn listing_spec() -> Vec<Insert> {
    vec![
        Insert::Epic(3),
        Insert::Doc(2),
        Insert::Decision(1, DecisionStatus::Open),
        Insert::Epic(1),
        Insert::Doc(3),
        Insert::Decision(3, DecisionStatus::Signed),
        Insert::Epic(2),
        Insert::Doc(1),
        Insert::Decision(2, DecisionStatus::Superseded),
    ]
}

fn build_listings(sequence: &[Insert]) -> Graph {
    let mut graph = empty_graph();
    for insert in sequence {
        match insert {
            Insert::Epic(n) => graph.insert_epic(epic_node(*n)).unwrap(),
            Insert::Decision(n, status) => {
                graph.insert_decision(decision_node(*n, *status)).unwrap()
            }
            Insert::Doc(n) => graph.insert_doc(doc_node(*n)).unwrap(),
        }
    }
    graph
}

/// Isomorphic rebuilds of the listing spec under seeded-shuffled
/// insertion sequences, as (seed, graph) pairs.
fn shuffled_rebuilds() -> Vec<(u64, Graph)> {
    SEEDS
        .iter()
        .map(|seed| {
            let mut rng = Rng(*seed);
            let mut shuffled = listing_spec();
            shuffle(&mut rng, &mut shuffled);
            (*seed, build_listings(&shuffled))
        })
        .collect()
}

#[test]
fn list_epic_list_is_ascending_id_and_history_free() {
    // D9's named contract: ascending id, not arrival order — the spec
    // inserts E3 first — and a function of state, not of the insertion
    // sequence.
    let graph = build_listings(&listing_spec());
    assert_eq!(graph.epic_list(), vec![epic_id(1), epic_id(2), epic_id(3)]);
    for (seed, rebuilt) in shuffled_rebuilds() {
        assert_eq!(
            fingerprint(&rebuilt),
            fingerprint(&graph),
            "seed {seed} built a different state"
        );
        assert_eq!(
            rebuilt.epic_list(),
            graph.epic_list(),
            "seed {seed} listed differently"
        );
    }
}

#[test]
fn list_decision_list_is_ascending_id_and_history_free() {
    let graph = build_listings(&listing_spec());
    // The spec inserts D1, D3, D2 in that sequence.
    assert_eq!(
        graph.decision_list(),
        vec![decision_id(1), decision_id(2), decision_id(3)]
    );
    for (seed, rebuilt) in shuffled_rebuilds() {
        assert_eq!(
            fingerprint(&rebuilt),
            fingerprint(&graph),
            "seed {seed} built a different state"
        );
        assert_eq!(
            rebuilt.decision_list(),
            graph.decision_list(),
            "seed {seed} listed differently"
        );
    }
}

#[test]
fn list_doc_list_is_ascending_id_and_history_free() {
    let graph = build_listings(&listing_spec());
    // The spec inserts S2, S3, S1 in that sequence.
    assert_eq!(graph.doc_list(), vec![doc_id(1), doc_id(2), doc_id(3)]);
    for (seed, rebuilt) in shuffled_rebuilds() {
        assert_eq!(
            fingerprint(&rebuilt),
            fingerprint(&graph),
            "seed {seed} built a different state"
        );
        assert_eq!(
            rebuilt.doc_list(),
            graph.doc_list(),
            "seed {seed} listed differently"
        );
    }
}

// ---------------------------------------------------------------------
// 14. The empty graph
// ---------------------------------------------------------------------

#[test]
fn list_an_empty_graph_lists_empty() {
    let graph = empty_graph();
    assert_eq!(graph.task_list(&TaskFilter::default()), Ok(vec![]));
    for filter in filter_matrix() {
        assert_eq!(graph.task_list(&filter), Ok(vec![]), "filter {filter:?}");
    }
    assert!(graph.epic_list().is_empty());
    assert!(graph.decision_list().is_empty());
    assert!(graph.doc_list().is_empty());

    // A known epic without tasks is the same case at the epic scope:
    // an empty Ok, and the epic filter still resolves.
    let taskless = graph_with_epics(1);
    assert_eq!(
        taskless.task_list(&TaskFilter {
            epic: Some(epic_id(1)),
            ..Default::default()
        }),
        Ok(vec![])
    );
    assert_eq!(taskless.epic_list(), vec![epic_id(1)]);
}

// ---------------------------------------------------------------------
// 15. State, not history: reruns and shuffled insertions
// ---------------------------------------------------------------------

/// One half of the build spec: (task number, epic number, status,
/// kind) nodes.
type SpecNodes = Vec<(u64, u64, Status, TaskKind)>;
/// The other half: (from, to) task edges, acyclic by construction. The
/// decision and its task gate are fixed — shuffling applies to the task
/// nodes and the task edges.
type SpecPairs = Vec<(u64, u64)>;

fn sample_spec() -> (SpecNodes, SpecPairs) {
    (
        vec![
            (1, 1, Status::Open, TaskKind::Task),
            (2, 1, Status::Closed, TaskKind::Gate),
            (3, 1, Status::Deferred, TaskKind::Finding),
            (4, 2, Status::Open, TaskKind::Debt),
            (5, 2, Status::InWork, TaskKind::Task),
            (6, 2, Status::Deferred, TaskKind::Gate),
            (7, 3, Status::Blocked, TaskKind::Finding),
            (8, 3, Status::Open, TaskKind::Debt),
        ],
        vec![(2, 1), (2, 3), (1, 6), (4, 5), (7, 8)],
    )
}

/// Builds the sample state: three epics, one open decision gating U4,
/// the spec's tasks and edges. U3 is deferred-ready (U2 closed), U6
/// deferred-held (U1 open), U4 decision-gated, and the canonical order
/// is [U2, U1, U3, U4, U5, U6, U7, U8] — not ascending id. The gate
/// lands after the task inserts because [`Graph::link_blocks`] requires
/// its endpoints to exist.
fn build_spec(nodes: &[(u64, u64, Status, TaskKind)], edges: &[(u64, u64)]) -> Graph {
    let mut graph = graph_with_epics(3);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    for (task, epic, status, kind) in nodes {
        let mut node = task_node(*task, *epic, *status);
        node.kind = *kind;
        graph.insert_task(node).unwrap();
    }
    for (from, to) in edges {
        graph.link_blocks(task_edge(*from, *to)).unwrap();
    }
    graph.link_blocks(gate_task(1, 4)).unwrap();
    graph
}

/// The filters whose answers the determinism test compares: the
/// default, one status, one kind, both readiness halves and the
/// contradiction — the set §8's byte-identity instrument runs under.
fn sample_filters() -> Vec<TaskFilter> {
    vec![
        TaskFilter::default(),
        TaskFilter {
            status: Some(Status::Open),
            ..Default::default()
        },
        TaskFilter {
            status: Some(Status::Deferred),
            ..Default::default()
        },
        TaskFilter {
            kind: Some(TaskKind::Gate),
            ..Default::default()
        },
        TaskFilter {
            ready: Some(true),
            ..Default::default()
        },
        TaskFilter {
            ready: Some(false),
            ..Default::default()
        },
        TaskFilter {
            status: Some(Status::Open),
            ready: Some(true),
            ..Default::default()
        },
    ]
}

/// Every answer of the sample state: the filters above under the repo
/// scope and each epic scope.
fn all_answers(graph: &Graph) -> Vec<Vec<TaskId>> {
    let mut answers = Vec::new();
    for filter in sample_filters() {
        answers.push(listed(graph, &filter));
        for e in 1..=3 {
            answers.push(listed(
                graph,
                &TaskFilter {
                    epic: Some(epic_id(e)),
                    ..filter.clone()
                },
            ));
        }
    }
    answers
}

#[test]
fn list_is_a_function_of_state_across_reruns_and_shuffled_insertion() {
    let (nodes, edges) = sample_spec();
    let graph = build_spec(&nodes, &edges);

    // The expected answers, spelled out: the suite must know what the
    // state means, not only that two runs agree.
    let order = [2u64, 1, 3, 4, 5, 6, 7, 8].map(task_id).to_vec();
    assert_eq!(graph.canonical_order(), order);
    assert_eq!(listed(&graph, &TaskFilter::default()), order);
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Open),
                ..Default::default()
            }
        ),
        vec![task_id(1), task_id(4), task_id(8)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                status: Some(Status::Deferred),
                ..Default::default()
            }
        ),
        vec![task_id(3), task_id(6)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                kind: Some(TaskKind::Gate),
                ..Default::default()
            }
        ),
        vec![task_id(2), task_id(6)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                kind: Some(TaskKind::Finding),
                ..Default::default()
            }
        ),
        vec![task_id(3), task_id(7)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(true),
                ..Default::default()
            }
        ),
        vec![task_id(3)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                ready: Some(false),
                ..Default::default()
            }
        ),
        vec![task_id(6)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(2)),
                ..Default::default()
            }
        ),
        vec![task_id(4), task_id(5), task_id(6)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(2)),
                status: Some(Status::Open),
                ..Default::default()
            }
        ),
        vec![task_id(4)]
    );
    assert_eq!(
        listed(
            &graph,
            &TaskFilter {
                epic: Some(epic_id(1)),
                kind: Some(TaskKind::Finding),
                ..Default::default()
            }
        ),
        vec![task_id(3)]
    );
    assert_eq!(
        graph.task_list(&TaskFilter {
            status: Some(Status::Open),
            ready: Some(true),
            ..Default::default()
        }),
        Ok(vec![])
    );
    assert_eq!(graph.epic_list(), vec![epic_id(1), epic_id(2), epic_id(3)]);
    assert_eq!(graph.decision_list(), vec![decision_id(1)]);
    assert!(graph.doc_list().is_empty());

    let answers = all_answers(&graph);
    let listings = (graph.epic_list(), graph.decision_list(), graph.doc_list());
    // Twice-run identity on unchanged state (§8), down to the bytes.
    assert_eq!(all_answers(&graph), answers);
    assert_eq!(
        to_bytes(&answers).unwrap(),
        to_bytes(&all_answers(&graph)).unwrap()
    );

    // Isomorphic rebuilds whose nodes and edges land in seeded-shuffled
    // sequences: proven the same state by fingerprint, so the same
    // answers — listing reads state, never history.
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
        assert_eq!(
            to_bytes(&all_answers(&rebuilt)).unwrap(),
            to_bytes(&answers).unwrap(),
            "seed {seed} listed different bytes"
        );
        assert_eq!(
            (
                rebuilt.epic_list(),
                rebuilt.decision_list(),
                rebuilt.doc_list()
            ),
            listings,
            "seed {seed} listed a node kind differently"
        );
    }
}

// ---------------------------------------------------------------------
// 16. Seeded graphs: agreement with an independent naive
//     recomputation, and the listing/summary invariant
// ---------------------------------------------------------------------

#[test]
fn list_seeded_graphs_agree_with_an_independent_recomputation() {
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, LIST_STEPS);
        assert_invariants(&graph);

        check_scope(&graph, None, seed);
        let epics: Vec<EpicId> = graph.epics().map(|epic| epic.id.clone()).collect();
        for epic in &epics {
            check_scope(&graph, Some(epic), seed);
        }

        // The same seed replayed gives the same listings.
        let (_again, replayed) = run_sequence(seed, LIST_STEPS);
        for filter in filter_matrix() {
            assert_eq!(
                replayed.task_list(&filter).unwrap(),
                graph.task_list(&filter).unwrap(),
                "seed {seed} diverged on replay for filter {filter:?}"
            );
        }
        assert_eq!(
            replayed.epic_list(),
            graph.epic_list(),
            "seed {seed} diverged on replay"
        );
        assert_eq!(
            replayed.decision_list(),
            graph.decision_list(),
            "seed {seed} diverged on replay"
        );
        assert_eq!(
            replayed.doc_list(),
            graph.doc_list(),
            "seed {seed} diverged on replay"
        );
    }
}

/// Asserts the agreement clauses for one scope: every filter of the
/// fixed matrix answers exactly the naive recomputation, and the D8
/// deferred counts equal `next_task`'s summary for the same scope.
fn check_scope(graph: &Graph, epic: Option<&EpicId>, seed: u64) {
    let label = match epic {
        None => "repo".to_owned(),
        Some(id) => id.to_string(),
    };
    for filter in filter_matrix() {
        let scoped = TaskFilter {
            epic: epic.cloned(),
            ..filter
        };
        let rows = graph.task_list(&scoped).unwrap();
        let ids: Vec<TaskId> = rows.iter().map(|row| row.task.clone()).collect();
        assert_eq!(
            ids,
            naive_task_list(graph, &scoped),
            "seed {seed}: scope {label}: filter {scoped:?}: listings disagree"
        );
    }
    let summary = match epic {
        None => graph.next_task().deferred,
        Some(id) => graph.next_task_of_epic(id).unwrap().deferred,
    };
    assert_counts(graph, epic, &summary, &format!("seed {seed}: {label}"));
}

// ---------------------------------------------------------------------
// 17. The coverage pin that keeps the agreement non-vacuous
// ---------------------------------------------------------------------

/// What the coverage pin observed on **one seed**, computed from the
/// built graphs through the naive helpers and the raw node fields —
/// never the draws and never the production predicates, so this pin
/// survives exactly the mutations it is meant to expose.
#[derive(Debug, Default)]
struct Coverage {
    statuses: BTreeMap<Status, usize>,
    kinds: BTreeMap<TaskKind, usize>,
    deferred_ready: usize,
    deferred_held: usize,
    deferred_held_by_epic_gate_alone: usize,
    epics_with_tasks: usize,
    epics_without_tasks: usize,
    empty_on_nonempty: usize,
}

impl Coverage {
    fn note(&mut self, graph: &Graph) {
        for task in graph.tasks() {
            *self.statuses.entry(task.status).or_default() += 1;
            *self.kinds.entry(task.kind).or_default() += 1;
            if task.status == Status::Deferred {
                if naive_until_satisfied(graph, task) {
                    self.deferred_ready += 1;
                } else {
                    self.deferred_held += 1;
                }
                // The shape that makes the agreement test sensitive to
                // a readiness that ignores the epic's decision gates:
                // every own predecessor finished while the epic's gate
                // still holds, so the listing's ready answer flips
                // exactly when the gate half is dropped. A deferred task
                // an unfinished own predecessor also holds answers the
                // same with or without the gate half, so it does not
                // count here.
                let own_held = graph
                    .blocked_by(&BlockEnd::Task(task.id.clone()))
                    .iter()
                    .any(|end| !naive_end_finished(graph, end));
                let gate_held = graph
                    .blocked_by(&BlockEnd::Epic(task.epic.clone()))
                    .iter()
                    .any(|end| !naive_end_finished(graph, end));
                if !own_held && gate_held {
                    self.deferred_held_by_epic_gate_alone += 1;
                }
            }
        }
        let epics: Vec<EpicId> = graph.epics().map(|epic| epic.id.clone()).collect();
        for epic in &epics {
            if graph.tasks().any(|task| &task.epic == epic) {
                self.epics_with_tasks += 1;
            } else {
                self.epics_without_tasks += 1;
            }
        }
        // Empty answers on a graph that holds tasks: the epic filter
        // over a taskless epic and the contradicting combinations both
        // land here, so the empty-Ok path is exercised by name.
        if graph.tasks().next().is_some() {
            let mut scopes: Vec<Option<EpicId>> = vec![None];
            scopes.extend(epics.iter().cloned().map(Some));
            for scope in &scopes {
                for filter in filter_matrix() {
                    let scoped = TaskFilter {
                        epic: scope.clone(),
                        ..filter
                    };
                    if naive_task_list(graph, &scoped).is_empty() {
                        self.empty_on_nonempty += 1;
                    }
                }
            }
        }
    }
}

#[test]
fn list_seeded_coverage() {
    // f9's lesson: a property test whose interesting branches the
    // generator never reaches is green and vacuous. The assertions are
    // per seed: a generator change that narrows one seed's graphs
    // fails here instead of hiding behind the other four.
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, LIST_STEPS);
        let mut coverage = Coverage::default();
        coverage.note(&graph);
        for status in [
            Status::Open,
            Status::InWork,
            Status::Blocked,
            Status::Deferred,
            Status::Closed,
            Status::Wontfix,
        ] {
            assert!(
                coverage.statuses.get(&status).copied().unwrap_or(0) > 0,
                "seed {seed}: no task ever held status {status}: {coverage:#?}"
            );
        }
        for kind in [
            TaskKind::Task,
            TaskKind::Gate,
            TaskKind::Finding,
            TaskKind::Debt,
        ] {
            assert!(
                coverage.kinds.get(&kind).copied().unwrap_or(0) > 0,
                "seed {seed}: no task ever had kind {kind}: {coverage:#?}"
            );
        }
        assert!(
            coverage.deferred_ready > 0,
            "seed {seed}: no deferred task was ever ready, so the ready=true \
             pins are vacuous: {coverage:#?}"
        );
        assert!(
            coverage.deferred_held > 0,
            "seed {seed}: no deferred task was ever held back, so the ready=false \
             pins are vacuous: {coverage:#?}"
        );
        assert!(
            coverage.deferred_held_by_epic_gate_alone > 0,
            "seed {seed}: no deferred task was ever held by its epic's decision gate \
             alone, so the seeded agreement cannot expose a readiness that ignores \
             the gate: {coverage:#?}"
        );
        assert!(
            coverage.epics_with_tasks > 0,
            "seed {seed}: no epic ever held a task, so the epic filter never saw \
             a non-empty scope: {coverage:#?}"
        );
        assert!(
            coverage.epics_without_tasks > 0,
            "seed {seed}: every epic held tasks, so the epic filter never answered \
             empty from a known epic: {coverage:#?}"
        );
        assert!(
            coverage.empty_on_nonempty > 0,
            "seed {seed}: no filter combination ever answered empty on a non-empty \
             graph: {coverage:#?}"
        );
    }
}
