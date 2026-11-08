//! The selection tests (spec §5.2, §10): the two actionability halves,
//! decision gating at both levels, the epic scope's induced-order/
//! full-graph interaction, the deferred summary, the fields that must
//! never enter, identical answers across reruns and shuffled
//! insertions, and seeded agreement with an independent naive
//! recomputation.
//!
//! The naive recomputation is deliberately a second, slower
//! implementation reading only the public accessors and spelling the
//! status matches out by hand, so agreement cannot come from a shared
//! bug in the production predicates (`naive_order` in `tests/order.rs`
//! is the precedent). Judgment call (f): the builders, the invariant
//! scan, the fingerprint and the seeded generator are the parent
//! module's — the only promotion this suite needed was `run_sequence`'s
//! visibility.

use crate::{DeferredSummary, NextItems, NextTask, TaskFilter, TaskRow};

use super::seeded::{Rng, run_sequence};
use super::*;

/// Fixed seeds for the insertion shuffles, per the suite's rule that a
/// determinism test never reseeds from a clock.
const SEEDS: [u64; 5] = [9, 31, 0xD1CE, 0x5E1E_C727, 0x0C0F_FEE1];
/// Fixed seeds for the mutation-sequence graphs the agreement and
/// coverage tests run over. The coverage pin asserts per seed, so each
/// seed's final graph must on its own reach every branch the pin
/// counts — a generator change that narrows one seed fails there
/// rather than hiding behind the others.
const GRAPH_SEEDS: [u64; 5] = [11, 30, 44, 50, 60];
/// Steps per seeded graph: deep enough that the coverage pin below
/// finds every branch, shallow enough that the per-step invariant scan
/// stays cheap.
const SELECTION_STEPS: u64 = 300;

/// Asserts one scope's whole answer against expected values. The
/// expected priority is `None` because every graph this helper serves
/// is built from [`task_node`], which sets no priority — the positive
/// echo cases have their own test.
fn assert_answer(graph: &Graph, scope: Option<u64>, task: Option<u64>, total: u64, ready: u64) {
    let expected = NextTask {
        task: task.map(task_id),
        priority: None,
        deferred: DeferredSummary { total, ready },
    };
    match scope {
        None => assert_eq!(graph.next_task(), expected, "repo scope"),
        Some(epic) => assert_eq!(
            graph.next_task_of_epic(&epic_id(epic)).unwrap(),
            expected,
            "epic scope E{epic}"
        ),
    }
}

/// Closes a task through the real update API, with the evidence the
/// closure discipline demands.
fn close_task(graph: &mut Graph, n: u64, stamp: Timestamp) {
    graph
        .update_task(task_id(n), stamp, |task| {
            task.status = Status::Closed;
            task.evidence = Some("cargo test -p todo-domain -- select: all green".to_owned());
        })
        .unwrap();
}

// ---------------------------------------------------------------------
// 1./2. The actionability predicate: status half and edge half
// ---------------------------------------------------------------------

#[test]
fn select_actionable_is_the_two_live_statuses() {
    // No predecessors anywhere: the status half alone decides, so a
    // predicate that read only `is_terminal` gets four rows wrong.
    for (status, actionable) in [
        (Status::Open, true),
        (Status::InWork, true),
        (Status::Blocked, false),
        (Status::Deferred, false),
        (Status::Closed, false),
        (Status::Wontfix, false),
    ] {
        let mut graph = graph_with_epics(1);
        graph.insert_task(task_node(1, 1, status)).unwrap();
        assert_eq!(
            graph.is_actionable(&task_id(1)),
            actionable,
            "{status} actionability"
        );
        // The edge half alone is status-blind: with no predecessors it
        // holds for every row, which is what makes the split testable.
        assert!(
            graph.is_until_satisfied(&task_id(1)),
            "{status} until-satisfaction with no predecessors"
        );
        assert_eq!(
            graph.next_task().task,
            actionable.then(|| task_id(1)),
            "{status} answer"
        );
    }
}

#[test]
fn select_an_unfinished_task_predecessor_holds_it_back() {
    // U1 blocks U2; U1 walks the six statuses. The expected answer row
    // by row: a live U1 is itself the answer, a held-status U1 leaves
    // nothing, a terminal U1 releases U2.
    for (status, finished, expected) in [
        (Status::Open, false, Some(1)),
        (Status::InWork, false, Some(1)),
        (Status::Blocked, false, None),
        (Status::Deferred, false, None),
        (Status::Closed, true, Some(2)),
        (Status::Wontfix, true, Some(2)),
    ] {
        let mut graph = graph_with_epics(1);
        graph.insert_task(task_node(1, 1, status)).unwrap();
        graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
        graph.link_blocks(task_edge(1, 2)).unwrap();
        assert_eq!(
            graph.is_until_satisfied(&task_id(2)),
            finished,
            "successor of a {status} predecessor"
        );
        assert_eq!(
            graph.is_actionable(&task_id(2)),
            finished,
            "successor of a {status} predecessor"
        );
        assert_eq!(
            graph.next_task().task,
            expected.map(task_id),
            "{status} predecessor answer"
        );
    }
}

// ---------------------------------------------------------------------
// 3./4. Decision gating, at the task and at the epic
// ---------------------------------------------------------------------

#[test]
fn select_an_open_decision_gates_a_task_and_resolving_it_releases() {
    let mut graph = graph_with_epics(1);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.link_blocks(gate_task(1, 1)).unwrap();

    assert!(!graph.is_until_satisfied(&task_id(1)));
    assert!(!graph.is_actionable(&task_id(1)));
    assert_eq!(graph.next_task().task, None);

    for status in [DecisionStatus::Signed, DecisionStatus::Superseded] {
        graph
            .update_decision(decision_id(1), at(9), |decision| {
                decision.status = status;
                decision.resolution = Some("signed by the user".to_owned());
            })
            .unwrap();
        assert!(graph.is_actionable(&task_id(1)), "{status} releases");
        assert_eq!(graph.next_task().task, Some(task_id(1)), "{status}");
    }
    // Resolving mutates no edge (§3/D10): the gate stays queryable.
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(1))),
        vec![BlockEnd::Decision(decision_id(1))]
    );
}

#[test]
fn select_an_open_decision_gating_an_epic_holds_every_task_under_it() {
    // §5.1's rule: the gate lands on the epic and holds all work under
    // it. The sharpest case in the set — a "predecessors of the task"
    // reading misses it entirely.
    let mut graph = graph_with_epics(2);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.link_blocks(gate_epic(1, 1)).unwrap();
    for (n, epic) in [(1, 1), (2, 1), (3, 2)] {
        graph.insert_task(task_node(n, epic, Status::Open)).unwrap();
    }

    assert!(!graph.is_actionable(&task_id(1)), "gated U1");
    assert!(!graph.is_actionable(&task_id(2)), "gated U2");
    assert!(graph.is_actionable(&task_id(3)), "ungated epic");
    assert_eq!(graph.next_task().task, Some(task_id(3)), "repo scope");
    assert_answer(&graph, Some(1), None, 0, 0);
    assert_answer(&graph, Some(2), Some(3), 0, 0);

    // Resolving the decision releases every task under the epic, with
    // no edge mutation.
    graph
        .update_decision(decision_id(1), at(9), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed by the user".to_owned());
        })
        .unwrap();
    assert!(graph.is_actionable(&task_id(1)));
    assert!(graph.is_actionable(&task_id(2)));
    // Canonical order is [U1, U2, U3], so the released epic leads.
    assert_eq!(graph.next_task().task, Some(task_id(1)));
    assert_answer(&graph, Some(1), Some(1), 0, 0);
}

// ---------------------------------------------------------------------
// 5. Epic scope: induced order, full-graph actionability (D9/t8)
// ---------------------------------------------------------------------

#[test]
fn select_epic_scope_honors_a_cross_epic_predecessor() {
    // E2 holds U1 and U2 with no inner edge; E1's U3 blocks U1 across
    // the epic boundary.
    let mut graph = graph_with_epics(2);
    for (n, epic) in [(1, 2), (2, 2), (3, 1)] {
        graph.insert_task(task_node(n, epic, Status::Open)).unwrap();
    }
    let induced_before = graph.canonical_order_of_epic(&epic_id(2)).unwrap();
    assert_eq!(induced_before, vec![task_id(1), task_id(2)]);
    graph.link_blocks(task_edge(3, 1)).unwrap();

    // The induced order is unchanged by the cross-epic edge — t8's
    // behaviour, re-pinned at the selection boundary.
    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(2)).unwrap(),
        induced_before
    );
    // Repo scope: E1 leads, U3 is the answer.
    assert_eq!(
        graph.canonical_order(),
        vec![task_id(3), task_id(1), task_id(2)]
    );
    assert_eq!(graph.next_task().task, Some(task_id(3)));
    // Epic scope: the induced order offers U1 first, the full-graph
    // check holds it back, so the answer is U2 — which is *not* the
    // repo-scoped answer restricted to the epic (that would be U1).
    let scoped = graph.next_task_of_epic(&epic_id(2)).unwrap();
    assert_eq!(scoped.task, Some(task_id(2)));
    let first_e2_in_repo_order = graph
        .canonical_order()
        .iter()
        .find(|id| graph.task(id).unwrap().epic == epic_id(2))
        .cloned();
    assert_eq!(first_e2_in_repo_order, Some(task_id(1)));
}

// ---------------------------------------------------------------------
// 6./7./8. Held statuses, the None answer, unknown epic
// ---------------------------------------------------------------------

#[test]
fn select_blocked_and_deferred_are_never_actionable_whatever_their_edges_say() {
    for status in [Status::Blocked, Status::Deferred] {
        // No predecessors at all.
        let mut graph = graph_with_epics(1);
        graph.insert_task(task_node(1, 1, status)).unwrap();
        assert!(!graph.is_actionable(&task_id(1)), "{status}, no edges");
        assert_eq!(graph.next_task().task, None, "{status}, no edges");
        assert_eq!(
            graph.next_task_of_epic(&epic_id(1)).unwrap().task,
            None,
            "{status}, no edges, epic scope"
        );

        // Every predecessor finished: a closed task edge and a signed
        // decision gate. The edge half says ready; the status half
        // still refuses.
        let mut graph = graph_with_epics(1);
        graph
            .insert_decision(decision_node(1, DecisionStatus::Signed))
            .unwrap();
        graph.insert_task(task_node(1, 1, Status::Closed)).unwrap();
        graph.insert_task(task_node(2, 1, status)).unwrap();
        graph.link_blocks(task_edge(1, 2)).unwrap();
        graph.link_blocks(gate_task(1, 2)).unwrap();
        assert!(
            graph.is_until_satisfied(&task_id(2)),
            "{status}, edges finished"
        );
        assert!(
            !graph.is_actionable(&task_id(2)),
            "{status}, edges finished"
        );
        assert_eq!(graph.next_task().task, None, "{status}, edges finished");
        assert_eq!(
            graph.next_task_of_epic(&epic_id(1)).unwrap().task,
            None,
            "{status}, edges finished, epic scope"
        );
    }
}

#[test]
fn select_nothing_actionable_is_an_answer_not_an_error() {
    // §6.1's code list is closed and has no code for "nothing
    // actionable": every shape below answers, none errs.

    // The empty graph, and the full value it answers with — the
    // summary rides the None answer (D2), and so does the priority
    // echo (D17: `None` when the answer is `None`).
    let graph = empty_graph();
    assert_eq!(
        graph.next_task(),
        NextTask {
            task: None,
            priority: None,
            deferred: DeferredSummary { total: 0, ready: 0 },
        }
    );

    // An epic without tasks, both entry points.
    let taskless = graph_with_epics(1);
    assert_answer(&taskless, None, None, 0, 0);
    assert_answer(&taskless, Some(1), None, 0, 0);

    // Every task terminal.
    let mut done = graph_with_epics(1);
    done.insert_task(task_node(1, 1, Status::Closed)).unwrap();
    done.insert_task(task_node(2, 1, Status::Wontfix)).unwrap();
    assert_answer(&done, None, None, 0, 0);
    assert_answer(&done, Some(1), None, 0, 0);

    // A repo fully gated by one open decision.
    let mut gated = graph_with_epics(2);
    gated
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    gated.link_blocks(gate_epic(1, 1)).unwrap();
    gated.link_blocks(gate_epic(1, 2)).unwrap();
    for (n, epic) in [(1, 1), (2, 2)] {
        gated.insert_task(task_node(n, epic, Status::Open)).unwrap();
    }
    assert_answer(&gated, None, None, 0, 0);
    assert_answer(&gated, Some(1), None, 0, 0);
    assert_answer(&gated, Some(2), None, 0, 0);
}

#[test]
fn select_an_unknown_epic_is_unknown_id() {
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    let result = graph.next_task_of_epic(&epic_id(9));
    assert_eq!(outcome(&result), "unknown_id");
    assert_eq!(
        result,
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        }),
        "the payload must name the id"
    );
    // The repo-scoped answer on the same graph is unaffected.
    assert_eq!(graph.next_task().task, Some(task_id(1)));
}

// ---------------------------------------------------------------------
// 9./10./11. The deferred summary
// ---------------------------------------------------------------------

#[test]
fn select_the_deferred_summary_separates_total_from_ready() {
    // Three deferred tasks whose until-targets differ: U3's is closed,
    // U4's is the live answer U2, U5's is the live U6. U2 open keeps
    // the scope from answering None.
    let mut graph = graph_with_epics(2);
    graph.insert_task(task_node(1, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(3, 1, Status::Deferred))
        .unwrap();
    graph
        .insert_task(task_node(4, 1, Status::Deferred))
        .unwrap();
    graph
        .insert_task(task_node(5, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(6, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(1, 3)).unwrap();
    graph.link_blocks(task_edge(2, 4)).unwrap();
    graph.link_blocks(task_edge(6, 5)).unwrap();

    // Present on both entry points; the empty E2 pins the scoping.
    assert_answer(&graph, None, Some(2), 3, 1);
    assert_answer(&graph, Some(1), Some(2), 3, 1);
    assert_answer(&graph, Some(2), None, 0, 0);

    // Flipping one predecessor to closed moves `ready` by exactly one
    // and leaves `total` alone.
    close_task(&mut graph, 6, at(20));
    assert_answer(&graph, None, Some(2), 3, 2);
    assert_answer(&graph, Some(1), Some(2), 3, 2);
}

#[test]
fn select_a_deferred_task_under_a_gated_epic_is_not_ready() {
    // D5's pin: readiness consults the epic gate too, so the summary
    // never advertises an item actionability would refuse.
    let mut graph = graph_with_epics(1);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.link_blocks(gate_epic(1, 1)).unwrap();
    graph
        .insert_task(task_node(1, 1, Status::Deferred))
        .unwrap();

    assert!(!graph.is_until_satisfied(&task_id(1)));
    assert_answer(&graph, None, None, 1, 0);
    assert_answer(&graph, Some(1), None, 1, 0);

    graph
        .update_decision(decision_id(1), at(9), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed by the user".to_owned());
        })
        .unwrap();
    assert!(graph.is_until_satisfied(&task_id(1)));
    assert_answer(&graph, None, None, 1, 1);
    // Ready, and still never actionable: the status half stands.
    assert!(!graph.is_actionable(&task_id(1)));
}

#[test]
fn select_the_deferred_summary_counts_tasks_not_epics() {
    // D10's pin: a deferred epic in scope changes neither count — the
    // selection unit is a task, and an epic's deferral is visible
    // through its own status.
    let mut graph = graph_with_epics(2);
    graph
        .update_epic(epic_id(2), |epic| epic.status = Status::Deferred)
        .unwrap();
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(2, 2, Status::Deferred))
        .unwrap();
    assert_answer(&graph, None, Some(1), 1, 1);
    assert_answer(&graph, Some(2), None, 1, 1);
}

// ---------------------------------------------------------------------
// 12./13./14. What never enters, and totality
// ---------------------------------------------------------------------

#[test]
fn select_epic_status_does_not_gate() {
    // D7's pin: the epic node's status never enters selection — the
    // gating mechanism is the edge. The second half of each round
    // proves the test cannot pass by ignoring edges too.
    for status in [Status::Closed, Status::Blocked, Status::Deferred] {
        let mut graph = graph_with_epics(2);
        graph
            .update_epic(epic_id(1), |epic| epic.status = status)
            .unwrap();
        graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
        graph.insert_task(task_node(2, 2, Status::Open)).unwrap();
        assert!(graph.is_actionable(&task_id(1)), "{status} epic gates");
        assert_answer(&graph, None, Some(1), 0, 0);
        assert_answer(&graph, Some(1), Some(1), 0, 0);

        // The same epic status with an open decision gate: held.
        let mut gated = graph_with_epics(2);
        gated
            .update_epic(epic_id(1), |epic| epic.status = status)
            .unwrap();
        gated
            .insert_decision(decision_node(1, DecisionStatus::Open))
            .unwrap();
        gated.link_blocks(gate_epic(1, 1)).unwrap();
        gated.insert_task(task_node(1, 1, Status::Open)).unwrap();
        gated.insert_task(task_node(2, 2, Status::Open)).unwrap();
        assert!(!gated.is_actionable(&task_id(1)), "{status} epic, gated");
        assert_answer(&gated, None, Some(2), 0, 0);
        assert_answer(&gated, Some(1), None, 0, 0);
    }
}

#[test]
fn select_until_priority_and_relates_never_enter() {
    // D8's three pins. Each is a silent way to be wrong: reading any
    // of them would still pass every other test in this suite.

    // `until`: two graphs identical except a past vs a future date.
    let build = |until: &str| {
        let mut graph = graph_with_epics(1);
        let mut first = task_node(1, 1, Status::Deferred);
        first.until = Some(Date::parse(until).unwrap());
        graph.insert_task(first).unwrap();
        let mut second = task_node(2, 1, Status::Open);
        second.until = Some(Date::parse(until).unwrap());
        graph.insert_task(second).unwrap();
        graph
    };
    let past = build("2020-01-01");
    let future = build("2099-12-31");
    assert_eq!(
        past.next_task(),
        future.next_task(),
        "until moved the answer"
    );
    assert_eq!(
        past.next_task_of_epic(&epic_id(1)).unwrap(),
        future.next_task_of_epic(&epic_id(1)).unwrap(),
        "until moved the scoped answer"
    );
    assert_answer(&past, None, Some(2), 1, 1);

    // `priority`: a high-priority task later in canonical order does
    // not beat the low-priority first one — and the answer carries
    // the chosen task's priority (D17's echo) while the order ignores
    // it, which is the two halves of the field's contract in one
    // assertion.
    let mut graph = graph_with_epics(1);
    let mut low = task_node(1, 1, Status::Open);
    low.priority = Some(Priority::Low);
    graph.insert_task(low).unwrap();
    let mut high = task_node(2, 1, Status::Open);
    high.priority = Some(Priority::High);
    graph.insert_task(high).unwrap();
    let expected = NextTask {
        task: Some(task_id(1)),
        priority: Some(Priority::Low),
        deferred: DeferredSummary { total: 0, ready: 0 },
    };
    assert_eq!(graph.next_task(), expected, "repo scope");
    assert_eq!(
        graph.next_task_of_epic(&epic_id(1)).unwrap(),
        expected,
        "epic scope"
    );

    // `relates`: annotating changes neither the answer nor the
    // summary. The lower-numbered end is `deferred`, not `closed` — a
    // closed end is finished, so it holds nothing under either reading
    // of the annotation and neither leak can move the answer. A deferred
    // end holds its open partner whether the stored (low, high) pair is
    // read as a directed `low blocks high` or the annotation is read as
    // symmetric, so both leaks move the answer to `None`; the symmetric
    // one holds the deferred end in turn and drops `ready`.
    let mut related = graph_with_epics(1);
    related
        .insert_task(task_node(1, 1, Status::Deferred))
        .unwrap();
    related.insert_task(task_node(2, 1, Status::Open)).unwrap();
    assert_answer(&related, None, Some(2), 1, 1);
    let before = related.next_task();
    let before_epic = related.next_task_of_epic(&epic_id(1)).unwrap();
    related.link_relates(task_id(1), task_id(2)).unwrap();
    assert_eq!(related.next_task(), before, "relates moved the answer");
    assert_eq!(
        related.next_task_of_epic(&epic_id(1)).unwrap(),
        before_epic,
        "relates moved the scoped answer"
    );
}

#[test]
fn select_the_answer_carries_the_chosen_tasks_priority() {
    // D17's echo, the positive cases: the answer's priority is the
    // chosen task's, read off the node — a High chosen task, a
    // null-priority chosen task, and a None answer (the negative half
    // lives in `select_nothing_actionable_is_an_answer_not_an_error`'s
    // full-value literal).
    let mut graph = graph_with_epics(1);
    let mut high = task_node(1, 1, Status::Open);
    high.priority = Some(Priority::High);
    graph.insert_task(high).unwrap();
    let expected = NextTask {
        task: Some(task_id(1)),
        priority: Some(Priority::High),
        deferred: DeferredSummary { total: 0, ready: 0 },
    };
    assert_eq!(graph.next_task(), expected, "repo scope");
    assert_eq!(
        graph.next_task_of_epic(&epic_id(1)).unwrap(),
        expected,
        "epic scope"
    );

    // A chosen task carrying no priority echoes `None` — the field is
    // present and null, never defaulted.
    let mut plain = graph_with_epics(1);
    plain.insert_task(task_node(1, 1, Status::Open)).unwrap();
    assert_eq!(plain.next_task().priority, None);
    assert_eq!(
        plain.next_task_of_epic(&epic_id(1)).unwrap().priority,
        None,
        "epic scope"
    );

    // A None answer echoes `None` (D3).
    let gated = graph_with_epics(1);
    assert_eq!(gated.next_task().priority, None);
}

#[test]
fn select_the_order_stays_priority_free_across_the_triage_surfaces() {
    // D17's (b)-guard, the required order-purity pin: a graph whose
    // tasks hold differing priorities answers `next_task`,
    // `next_items` and `task_list` in the SAME canonical order as its
    // priority-free twin — the twin differs only in the priority
    // fields, so any priority-reading sort moves one of the three
    // orders and fails here. The graph is shaped so it would: U1 (low)
    // leads U3 (high) in canonical order, and the task scope's rows
    // [U5 (low), U6 (high)] would flip under a high-first sort.
    let build = |priorities: [Option<Priority>; 6]| {
        let mut graph = graph_with_epics(2);
        // E1: U1–U4, no edges — the repo batch's head.
        for (n, priority) in priorities.iter().take(4).enumerate() {
            let mut task = task_node(n as u64 + 1, 1, Status::Open);
            task.priority = *priority;
            graph.insert_task(task).unwrap();
        }
        // E2: the U4→U5→U6 chain gives the task scope of U4 two rows
        // whose priorities disagree with their canonical positions.
        let mut u5 = task_node(5, 2, Status::Open);
        u5.priority = priorities[4];
        graph.insert_task(u5).unwrap();
        let mut u6 = task_node(6, 2, Status::Open);
        u6.priority = priorities[5];
        graph.insert_task(u6).unwrap();
        graph.link_blocks(task_edge(4, 5)).unwrap();
        graph.link_blocks(task_edge(5, 6)).unwrap();
        graph
    };
    let graph = build([
        Some(Priority::Low),
        None,
        Some(Priority::High),
        None,
        Some(Priority::Low),
        Some(Priority::High),
    ]);
    let twin = build([None; 6]);

    // The premise, named: the graph really carries differing
    // priorities, in the positions a priority sort would move.
    let batch = graph.next_items(u64::MAX);
    assert_eq!(batch.items[0].priority, Some(Priority::Low));
    assert_eq!(batch.items[2].priority, Some(Priority::High));
    assert_eq!(twin.next_items(u64::MAX).items[0].priority, None);

    // The three surfaces, in the same order as the twin.
    assert_eq!(
        graph.next_task().task,
        twin.next_task().task,
        "next_task moved"
    );
    let ids = |answer: &NextItems| {
        answer
            .items
            .iter()
            .map(|row| row.task.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ids(&graph.next_items(u64::MAX)),
        ids(&twin.next_items(u64::MAX)),
        "the repo batch moved"
    );
    assert_eq!(
        ids(&graph.next_items_of_epic(&epic_id(1), u64::MAX).unwrap()),
        ids(&twin.next_items_of_epic(&epic_id(1), u64::MAX).unwrap()),
        "the epic batch moved"
    );
    assert_eq!(
        ids(&graph.next_items_of_task(&task_id(4), u64::MAX).unwrap()),
        ids(&twin.next_items_of_task(&task_id(4), u64::MAX).unwrap()),
        "the task batch moved"
    );
    let row_ids = |rows: &[TaskRow]| rows.iter().map(|row| row.task.clone()).collect::<Vec<_>>();
    assert_eq!(
        row_ids(&graph.task_list(&TaskFilter::default()).unwrap()),
        row_ids(&twin.task_list(&TaskFilter::default()).unwrap()),
        "the listing moved"
    );
}

#[test]
fn select_an_id_the_graph_does_not_hold_is_neither_actionable_nor_ready() {
    // D4's totality pin: `false` is the safe direction — a `forall`
    // over an empty predecessor list would otherwise answer `true`.
    let graph = graph_with_tasks(1, 2);
    let missing = task_id(99);
    assert!(!graph.is_actionable(&missing));
    assert!(!graph.is_until_satisfied(&missing));
    // The direction does not depend on what the graph holds.
    let empty = empty_graph();
    assert!(!empty.is_actionable(&missing));
    assert!(!empty.is_until_satisfied(&missing));
    // And since `ready` is the same edge half, an unknown id is never
    // counted: the summary only sees the scope's derived list.
    assert_eq!(
        graph.next_task().deferred,
        DeferredSummary { total: 0, ready: 0 }
    );
}

// ---------------------------------------------------------------------
// 15. State, not history: reruns and shuffled insertions
// ---------------------------------------------------------------------

/// One half of the build spec: (task number, epic number, status)
/// nodes.
type SpecNodes = Vec<(u64, u64, Status)>;
/// The other half: (from, to) task edges, acyclic by construction. The
/// decision and its task gate are fixed — shuffling applies to the
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
            (6, 2, Status::Deferred),
            (7, 3, Status::Blocked),
            (8, 3, Status::Open),
        ],
        vec![(2, 3), (1, 6), (4, 5), (7, 8)],
    )
}

/// Builds the sample state: three epics, one open decision gating U4,
/// the spec's tasks and edges. U3 is deferred-ready (U2 closed), U6
/// deferred-held (U1 open), U4 decision-held, U5 held by U4, U8 held
/// by the blocked U7 — the repo answer is U1.
fn build_spec(nodes: &[(u64, u64, Status)], edges: &[(u64, u64)]) -> Graph {
    let mut graph = graph_with_epics(3);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    for (task, epic, status) in nodes {
        graph.insert_task(task_node(*task, *epic, *status)).unwrap();
    }
    for (from, to) in edges {
        graph.link_blocks(task_edge(*from, *to)).unwrap();
    }
    graph.link_blocks(gate_task(1, 4)).unwrap();
    graph
}

/// Fisher–Yates under the seeded splitmix64: reproducible shuffles, no
/// `rand` dependency.
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        items.swap(i, rng.below(i + 1));
    }
}

#[test]
fn select_is_a_function_of_state_across_reruns_and_shuffled_insertion() {
    let (nodes, edges) = sample_spec();
    let graph = build_spec(&nodes, &edges);

    // The expected answers, spelled out: the suite must know what the
    // state means, not only that two runs agree.
    assert_answer(&graph, None, Some(1), 2, 1);
    assert_answer(&graph, Some(1), Some(1), 1, 1);
    assert_answer(&graph, Some(2), None, 1, 0);
    assert_answer(&graph, Some(3), None, 0, 0);

    let repo = graph.next_task();
    let scoped: Vec<NextTask> = (1..=3)
        .map(|e| graph.next_task_of_epic(&epic_id(e)).unwrap())
        .collect();
    // Twice-run identity on unchanged state (§8).
    assert_eq!(graph.next_task(), repo);
    assert_eq!(
        (1..=3)
            .map(|e| graph.next_task_of_epic(&epic_id(e)).unwrap())
            .collect::<Vec<_>>(),
        scoped
    );

    // Isomorphic rebuilds whose nodes and edges land in seeded-shuffled
    // sequences: proven the same state by fingerprint, so the same
    // answers — selection reads state, never history.
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
            rebuilt.next_task(),
            repo,
            "seed {seed} answered differently"
        );
        for (index, expected) in scoped.iter().enumerate() {
            assert_eq!(
                rebuilt
                    .next_task_of_epic(&epic_id(index as u64 + 1))
                    .unwrap(),
                *expected,
                "seed {seed} answered differently for epic E{}",
                index + 1
            );
        }
    }
}

// ---------------------------------------------------------------------
// 16./17. Seeded graphs: agreement with a naive recomputation, and
//         the coverage pin that keeps the agreement non-vacuous
// ---------------------------------------------------------------------

#[test]
fn select_seeded_graphs_agree_with_an_independent_recomputation() {
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, SELECTION_STEPS);
        assert_invariants(&graph);

        check_selection(&graph, None, seed);
        let epics: Vec<EpicId> = graph.epics().map(|epic| epic.id.clone()).collect();
        for epic in &epics {
            check_selection(&graph, Some(epic), seed);
        }

        // (iv) The same seed replayed gives the same answer.
        let (_again, replayed) = run_sequence(seed, SELECTION_STEPS);
        assert_eq!(
            replayed.next_task(),
            graph.next_task(),
            "seed {seed} diverged on replay"
        );
    }
}

/// Asserts the agreement clauses for one scope: (i) the returned task
/// is actionable per the naive predicate, (ii) no task earlier in the
/// same canonical order is, (iii) both summary counts equal the naive
/// counts.
fn check_selection(graph: &Graph, scope: Option<&EpicId>, seed: u64) {
    let answer = match scope {
        None => graph.next_task(),
        Some(epic) => graph.next_task_of_epic(epic).unwrap(),
    };
    let naive = naive_select(graph, scope);
    let order = match scope {
        None => graph.canonical_order(),
        Some(epic) => graph.canonical_order_of_epic(epic).unwrap(),
    };
    match &answer.task {
        Some(id) => {
            assert!(
                naive_actionable(graph, id),
                "seed {seed}: the answer {id} is not naive-actionable"
            );
            for earlier in order.iter().take_while(|candidate| *candidate != id) {
                assert!(
                    !naive_actionable(graph, earlier),
                    "seed {seed}: {earlier} precedes the answer and is naive-actionable"
                );
            }
        }
        None => {
            for candidate in &order {
                assert!(
                    !naive_actionable(graph, candidate),
                    "seed {seed}: {candidate} is naive-actionable but nothing was returned"
                );
            }
        }
    }
    assert_eq!(
        answer.deferred, naive.deferred,
        "seed {seed}: summaries disagree"
    );
    // (i)+(ii) over a total order name a single task, so the two
    // implementations must agree on it outright.
    assert_eq!(answer.task, naive.task, "seed {seed}: answers disagree");
}

/// The deliberately naive second implementation, written out longhand
/// the way `naive_order` is: it walks the scope's canonical order,
/// resolves every predecessor by hand through the public accessors and
/// shares no predicate with production — a recomputation that called
/// [`Graph::is_actionable`] would prove nothing. The *order* is the
/// production one by design: t8's suite pins it independently, and the
/// spec's clause (ii) walks "the same canonical order".
fn naive_select(graph: &Graph, scope: Option<&EpicId>) -> NextTask {
    let order = match scope {
        None => graph.canonical_order(),
        Some(epic) => graph.canonical_order_of_epic(epic).unwrap(),
    };
    let mut answer = None;
    for id in &order {
        if answer.is_none() && naive_actionable(graph, id) {
            answer = Some(id.clone());
        }
    }
    let mut deferred = DeferredSummary { total: 0, ready: 0 };
    for id in &order {
        let Some(task) = graph.task(id) else { continue };
        if task.status != Status::Deferred {
            continue;
        }
        deferred.total += 1;
        deferred.ready += u64::from(naive_until_satisfied(graph, task));
    }
    // The echo, recomputed by hand: the chosen task's priority read
    // off the node through the public accessor.
    let priority = answer
        .as_ref()
        .and_then(|id| graph.task(id).and_then(|node| node.priority));
    NextTask {
        task: answer,
        priority,
        deferred,
    }
}

/// What the coverage pin observed on **one seed**, computed from the
/// built graphs.
#[derive(Debug, Default)]
struct Coverage {
    held_by_task: usize,
    held_by_decision: usize,
    held_by_epic_gate_alone: usize,
    deferred_ready: usize,
    deferred_held: usize,
    none_scopes: usize,
    scopes_with_deferred: usize,
}

impl Coverage {
    fn note(&mut self, graph: &Graph, scope: Option<&EpicId>) {
        let naive = naive_select(graph, scope);
        if naive.task.is_none() {
            self.none_scopes += 1;
        }
        if naive.deferred.total > 0 {
            self.scopes_with_deferred += 1;
        }
        for task in graph
            .tasks()
            .filter(|task| scope.is_none_or(|epic| task.epic == *epic))
        {
            let own = graph.blocked_by(&BlockEnd::Task(task.id.clone()));
            let gates = graph.blocked_by(&BlockEnd::Epic(task.epic.clone()));
            let own_held = own.iter().any(|end| !naive_end_finished(graph, end));
            let gate_held = gates.iter().any(|end| !naive_end_finished(graph, end));
            if own
                .iter()
                .any(|end| matches!(end, BlockEnd::Task(_)) && !naive_end_finished(graph, end))
            {
                self.held_by_task += 1;
            }
            if own
                .iter()
                .any(|end| matches!(end, BlockEnd::Decision(_)) && !naive_end_finished(graph, end))
            {
                self.held_by_decision += 1;
            }
            // The shape that makes the agreement test sensitive to
            // mutation M1: the epic's gate holds and nothing else does —
            // a task an unfinished own predecessor also holds answers
            // the same with or without the epic-gating half. The status
            // filter is part of that shape: the agreement test sees the
            // gate only through a scope's answer or its `ready` counts, so
            // a sole-hold that can be neither — a terminal task, or a
            // `blocked` one, which is never actionable whatever its edges
            // say — would inflate this count without making that test
            // sensitive to anything.
            if gate_held
                && !own_held
                && matches!(
                    task.status,
                    Status::Open | Status::InWork | Status::Deferred
                )
            {
                self.held_by_epic_gate_alone += 1;
            }
            if task.status == Status::Deferred {
                if own_held || gate_held {
                    self.deferred_held += 1;
                } else {
                    self.deferred_ready += 1;
                }
            }
        }
    }
}

#[test]
fn select_seeded_coverage() {
    // f9's lesson: a property test whose interesting branches the
    // generator never reaches is green and vacuous. Every count is
    // computed from the built graphs through the naive helpers, never
    // from the draws and never through the production predicates — so
    // this pin survives exactly the mutations it is meant to expose.
    // The assertions are per seed: a generator change that narrows one
    // seed's graphs fails here instead of hiding behind the other four.
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, SELECTION_STEPS);
        let mut coverage = Coverage::default();
        coverage.note(&graph, None);
        let epics: Vec<EpicId> = graph.epics().map(|epic| epic.id.clone()).collect();
        for epic in &epics {
            coverage.note(&graph, Some(epic));
        }
        assert!(
            coverage.held_by_task > 0,
            "seed {seed}: no task was ever held by an unfinished task predecessor: {coverage:#?}"
        );
        assert!(
            coverage.held_by_decision > 0,
            "seed {seed}: no task was ever held by an open decision gate: {coverage:#?}"
        );
        assert!(
            coverage.held_by_epic_gate_alone > 0,
            "seed {seed}: no task was ever held by its epic's gate alone — an open \
             epic gate with every own predecessor finished: {coverage:#?}"
        );
        assert!(
            coverage.deferred_ready > 0,
            "seed {seed}: no deferred task was ever ready: {coverage:#?}"
        );
        assert!(
            coverage.deferred_held > 0,
            "seed {seed}: no deferred task was ever held back: {coverage:#?}"
        );
        assert!(
            coverage.none_scopes > 0,
            "seed {seed}: every scope always had an answer: {coverage:#?}"
        );
        assert!(
            coverage.scopes_with_deferred > 0,
            "seed {seed}: no scope ever held a deferred task, so the ready/held \
             pins are vacuous: {coverage:#?}"
        );
    }
}

// ---------------------------------------------------------------------
// 18. A deep chain: the first actionable, not the last
// ---------------------------------------------------------------------

#[test]
fn select_a_deep_chain_returns_the_first_actionable_not_the_last() {
    // A→B→C→D, all open: the answer is the *first* actionable node in
    // the order, not the last. U5 rides along as an independent
    // actionable task later in canonical order: the chain alone holds
    // exactly one actionable task at every step, so without a second
    // candidate "first" and "last" are indistinguishable and a
    // last-hit implementation would pass. Direct-vs-transitive
    // actionability is *not* this test's pin — a transitive-closure
    // mutant answers the same on this chain; the seeded agreement test
    // carries that property.
    let mut graph = graph_with_epics(1);
    for n in 1..=5 {
        graph.insert_task(task_node(n, 1, Status::Open)).unwrap();
    }
    for (from, to) in [(1, 2), (2, 3), (3, 4)] {
        graph.link_blocks(task_edge(from, to)).unwrap();
    }
    assert_eq!(graph.next_task().task, Some(task_id(1)));

    close_task(&mut graph, 1, at(20));
    assert_eq!(graph.next_task().task, Some(task_id(2)));
    close_task(&mut graph, 2, at(21));
    assert_eq!(graph.next_task().task, Some(task_id(3)));
    // The epic scope walks the same chain.
    assert_eq!(
        graph.next_task_of_epic(&epic_id(1)).unwrap().task,
        Some(task_id(3))
    );
}

// ---------------------------------------------------------------------
// 19. Epic scope walks the induced order — not the filtered repo order,
//     not ascending task id, not a chain-following greedy walk
// ---------------------------------------------------------------------

#[test]
fn select_epic_scope_walks_the_induced_order() {
    // (i) Not the repo-wide order filtered to the epic. E2 holds U5
    // and U6, both open; E3 holds U7 closed; U7 blocks U5. The induced
    // order of E2 is [U5, U6] — no edge between them, the key decides —
    // while the repo-wide order is [U6, U7, U5], so the filtered order
    // is [U6, U5]. U5 is actionable (its only predecessor is closed),
    // so the epic answer is U5 and a filter-the-repo-order walk answers
    // U6.
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
    assert_eq!(graph.next_task().task, Some(task_id(6)), "repo scope");
    assert_eq!(
        graph.next_task_of_epic(&epic_id(2)).unwrap().task,
        Some(task_id(5)),
        "the epic scope diverges from the filtered repo order"
    );

    // (ii) Not ascending task id. One epic holds U1 open, U3 open and
    // U5 closed, with U5 blocking U1. The induced order is
    // [U3, U5, U1] — U3 and U5 start ready and the key picks U3; U5
    // releases U1 — while ascending id is [U1, U3, U5]. U1 is
    // actionable (U5 is closed, so terminal) and so is U3, so the
    // answer is U3 and an ascending-id walk answers U1.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(3, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(5, 1, Status::Closed)).unwrap();
    graph.link_blocks(task_edge(5, 1)).unwrap();

    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(1)).unwrap(),
        vec![task_id(3), task_id(5), task_id(1)],
        "the induced order the epic scope must walk"
    );
    // Single epic: the repo scope walks the same list and answers the
    // same, which is the point — the divergence in (ii) is against the
    // ascending-id walk, not between the scopes.
    assert_eq!(graph.next_task().task, Some(task_id(3)), "repo scope");
    assert_eq!(
        graph.next_task_of_epic(&epic_id(1)).unwrap().task,
        Some(task_id(3)),
        "the epic scope is not an ascending-id walk"
    );

    // (iii) Not a chain-following greedy walk either: draining a
    // just-released successor before re-taking the frontier minimum still
    // yields a valid topological order of the same induced subgraph, so
    // the walk is not wrong about sequence — only about the tiebreaker.
    // The two part wherever a just-released successor is taken ahead of
    // a smaller-keyed task the frontier still holds. No seeded epic scope
    // reaches that shape, so it is built here: statuses do not filter the
    // order, the closed U1 leads both walks, and the walks diverge at the
    // second position, where the frontier minimum is U2 but the greedy
    // walk takes the U3 that placing U1 just released — induced
    // [U1, U2, U3, U4] against greedy [U1, U3, U2, U4], so the first
    // actionable task is U2 under the induced order and U3 under the
    // greedy one.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Closed)).unwrap();
    for n in 2..=4 {
        graph.insert_task(task_node(n, 1, Status::Open)).unwrap();
    }
    graph.link_blocks(task_edge(1, 3)).unwrap();
    graph.link_blocks(task_edge(2, 4)).unwrap();

    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(1)).unwrap(),
        vec![task_id(1), task_id(2), task_id(3), task_id(4)],
        "the induced order the epic scope must walk"
    );
    // Single epic, so the repo scope walks the same list: the divergence
    // is against the greedy walk, not between the scopes. A greedy
    // rewrite of the repo-scoped entry point alone is a different
    // residual, and this is the assertion that catches it.
    assert_eq!(graph.next_task().task, Some(task_id(2)), "repo scope");
    assert_eq!(
        graph.next_task_of_epic(&epic_id(1)).unwrap().task,
        Some(task_id(2)),
        "the epic scope is not a chain-following greedy walk"
    );
}
