//! The roll-up tests (spec §6's D16 arithmetic, §10): every count
//! field against hand-computed values, the open totals as the
//! non-terminal sums, the deferred split and its boundary cases, the
//! agreement with the selection tools, the epic scope's membership
//! and exclusion, the one `unknown_id`, determinism across reruns and
//! shuffled insertions, seeded agreement with an independent naive
//! recomputation, the coverage pin that keeps the agreement
//! non-vacuous, and the row annotation's two-run order and status
//! currency.
//!
//! The naive recomputation is deliberately a second, slower
//! implementation reading only the public accessors and spelling the
//! status matches and the two-run blocked-by builder out by hand, so
//! agreement cannot come from a shared bug in the production counting
//! or the production lift (`naive_select` in `tests/select.rs` is the
//! precedent). The canonical *order* is the production one by design,
//! for the reason the selection suite records: `tests/order.rs` pins
//! it independently, and the scope's task list is that list.

use std::collections::BTreeSet;

use crate::{
    Blocker, DecisionCounts, DeferredSummary, OpenTotals, StatusCounts, StatusSummary, TaskFilter,
    TaskRow,
};

use super::seeded::{Rng, run_sequence};
use super::*;

/// Fixed seeds for the insertion shuffles, per the suite's rule that a
/// determinism test never reseeds from a clock.
const SEEDS: [u64; 5] = [5, 47, 0xD16, 0x5EED_0D16, 0x0FF5];
/// Fixed seeds for the mutation-sequence graphs the agreement and
/// coverage tests run over. The coverage pin asserts per seed, so each
/// seed's final graph must on its own reach every branch the pin
/// counts — a generator change that narrows one seed fails there
/// rather than hiding behind the others. The seed set and the step
/// depth below are this suite's own, picked by a grid measurement so
/// that every branch — deferred items both ready and held, an epic
/// scope gated by an open decision, and a scope whose gate set gains a
/// member through a task edge alone — is reached at this depth; seeds
/// that satisfy one suite's pin need not satisfy another's, so the
/// set is not shared with the sibling suites.
const GRAPH_SEEDS: [u64; 5] = [2, 17, 23, 73, 97];
/// Steps per seeded graph: deep enough that the coverage pin below
/// finds every branch, shallow enough that the per-step invariant scan
/// stays cheap. The listing suite's depth, for the reason the seed
/// set above records.
const SUMMARY_STEPS: u64 = 600;

// ---------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------

/// A task blocker, spelled the way the field docs read.
fn b_task(n: u64, status: Status) -> Blocker {
    Blocker::Task {
        id: task_id(n),
        status,
    }
}

/// A decision blocker, spelled the way the field docs read.
fn b_decision(n: u64, status: DecisionStatus) -> Blocker {
    Blocker::Decision {
        id: decision_id(n),
        status,
    }
}

/// The id sequence of a row's blockers, for comparing against
/// `NextItem::blocked_by`'s vocabulary.
fn ends(blockers: &[Blocker]) -> Vec<BlockEnd> {
    blockers.iter().cloned().map(BlockEnd::from).collect()
}

fn zero_counts() -> StatusCounts {
    StatusCounts {
        open: 0,
        in_work: 0,
        blocked: 0,
        deferred: 0,
        closed: 0,
        wontfix: 0,
    }
}

fn zero_decisions() -> DecisionCounts {
    DecisionCounts {
        open: 0,
        signed: 0,
        superseded: 0,
    }
}

fn zero_summary() -> StatusSummary {
    StatusSummary {
        epics: zero_counts(),
        tasks: zero_counts(),
        decisions: zero_decisions(),
        docs: 0,
        open: OpenTotals {
            epics: 0,
            tasks: 0,
            decisions: 0,
        },
        deferred: DeferredSummary { total: 0, ready: 0 },
    }
}

/// The six statuses, in declaration order — the vocabulary the
/// arithmetic tests walk.
const ALL_STATUSES: [Status; 6] = [
    Status::Open,
    Status::InWork,
    Status::Blocked,
    Status::Deferred,
    Status::Closed,
    Status::Wontfix,
];

/// The arithmetic graph: six epics, one per status; twelve tasks, two
/// per status under the matching epic; three decisions, one per
/// status; two docs. U7 is deferred behind the closed U9 (ready), U8
/// behind the open U1 (held).
fn arithmetic_graph() -> Graph {
    let mut graph = empty_graph();
    for (n, status) in ALL_STATUSES.iter().enumerate() {
        let mut epic = epic_node(n as u64 + 1);
        epic.status = *status;
        graph.insert_epic(epic).unwrap();
    }
    for (i, status) in ALL_STATUSES.iter().enumerate() {
        for k in 0..2 {
            let n = i as u64 * 2 + k + 1;
            graph
                .insert_task(task_node(n, i as u64 + 1, *status))
                .unwrap();
        }
    }
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph
        .insert_decision(decision_node(2, DecisionStatus::Signed))
        .unwrap();
    graph
        .insert_decision(decision_node(3, DecisionStatus::Superseded))
        .unwrap();
    graph.insert_doc(doc_node(1)).unwrap();
    graph.insert_doc(doc_node(2)).unwrap();
    graph.link_blocks(task_edge(9, 7)).unwrap();
    graph.link_blocks(task_edge(1, 8)).unwrap();
    graph
}

/// The boundary graph: E1 is gated by the open D2 and holds U4
/// deferred with no own predecessor — the gate alone holds it — plus
/// the open U6. E2, ungated, holds U1 deferred behind the wontfix U2,
/// U3 deferred behind the superseded D1, and U5 deferred behind the
/// open U7.
fn boundary_graph() -> Graph {
    let mut graph = graph_with_epics(2);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Superseded))
        .unwrap();
    graph
        .insert_decision(decision_node(2, DecisionStatus::Open))
        .unwrap();
    graph.link_blocks(gate_epic(2, 1)).unwrap();
    graph
        .insert_task(task_node(4, 1, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(6, 1, Status::Open)).unwrap();
    graph
        .insert_task(task_node(1, 2, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(2, 2, Status::Wontfix)).unwrap();
    graph
        .insert_task(task_node(3, 2, Status::Deferred))
        .unwrap();
    graph
        .insert_task(task_node(5, 2, Status::Deferred))
        .unwrap();
    graph.insert_task(task_node(7, 2, Status::Open)).unwrap();
    graph.link_blocks(task_edge(2, 1)).unwrap();
    graph.link_blocks(gate_task(1, 3)).unwrap();
    graph.link_blocks(task_edge(7, 5)).unwrap();
    graph
}

/// Fisher–Yates under the seeded splitmix64: reproducible shuffles, no
/// `rand` dependency.
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        items.swap(i, rng.below(i + 1));
    }
}

// ---------------------------------------------------------------------
// 1./2. The arithmetic, and the open totals as the non-terminal sums
// ---------------------------------------------------------------------

#[test]
fn summary_arithmetic_against_a_hand_built_graph() {
    let graph = arithmetic_graph();

    // The premises, named (f47): the deferred pair really is split by
    // its until-targets, so the deferred block below can fail.
    assert!(
        graph.is_until_satisfied(&task_id(7)),
        "U7's target is closed"
    );
    assert!(
        !graph.is_until_satisfied(&task_id(8)),
        "U8's target is open"
    );

    let summary = graph.status_summary();
    assert_eq!(
        summary.epics,
        StatusCounts {
            open: 1,
            in_work: 1,
            blocked: 1,
            deferred: 1,
            closed: 1,
            wontfix: 1,
        }
    );
    assert_eq!(
        summary.tasks,
        StatusCounts {
            open: 2,
            in_work: 2,
            blocked: 2,
            deferred: 2,
            closed: 2,
            wontfix: 2,
        }
    );
    assert_eq!(
        summary.decisions,
        DecisionCounts {
            open: 1,
            signed: 1,
            superseded: 1,
        }
    );
    assert_eq!(summary.docs, 2);
    assert_eq!(
        summary.open,
        OpenTotals {
            epics: 4,
            tasks: 8,
            decisions: 1,
        }
    );
    assert_eq!(summary.deferred, DeferredSummary { total: 2, ready: 1 });
    // The whole value at once, so a field added later cannot ship
    // unasserted.
    assert_eq!(
        summary,
        StatusSummary {
            epics: StatusCounts {
                open: 1,
                in_work: 1,
                blocked: 1,
                deferred: 1,
                closed: 1,
                wontfix: 1,
            },
            tasks: StatusCounts {
                open: 2,
                in_work: 2,
                blocked: 2,
                deferred: 2,
                closed: 2,
                wontfix: 2,
            },
            decisions: DecisionCounts {
                open: 1,
                signed: 1,
                superseded: 1,
            },
            docs: 2,
            open: OpenTotals {
                epics: 4,
                tasks: 8,
                decisions: 1,
            },
            deferred: DeferredSummary { total: 2, ready: 1 },
        }
    );
}

#[test]
fn summary_open_totals_are_the_non_terminal_sums() {
    // The same state as the arithmetic pin, read for its totals: the
    // non-terminal sums per kind, hand-computed — four epics, eight
    // tasks, one open decision — against totals a terminal-counting
    // implementation answers differently (two, four, two).
    let graph = arithmetic_graph();
    let summary = graph.status_summary();
    assert_eq!(summary.open.epics, 4, "the non-terminal epics");
    assert_eq!(summary.open.tasks, 8, "the non-terminal tasks");
    assert_eq!(summary.open.decisions, 1, "the open decisions");

    // The sums, spelled out per status rather than trusted as fields:
    // an `open` total of a Status-counted kind is exactly the four
    // non-terminal statuses' sum, so a mutant that counts the
    // terminal pair fails here and above.
    assert_eq!(
        summary.open.epics,
        summary.epics.open + summary.epics.in_work + summary.epics.blocked + summary.epics.deferred
    );
    assert_eq!(
        summary.open.tasks,
        summary.tasks.open + summary.tasks.in_work + summary.tasks.blocked + summary.tasks.deferred
    );
    // The decision clause: a decision's non-terminal set is `open`
    // alone, so the open total is the open count — not the resolved
    // pair, which this state holds apart (one open, two resolved).
    assert_eq!(summary.open.decisions, summary.decisions.open);
    assert_ne!(
        summary.open.decisions,
        summary.decisions.signed + summary.decisions.superseded
    );
}

// ---------------------------------------------------------------------
// 3./4. The deferred split, and the agreement with the selection tools
// ---------------------------------------------------------------------

#[test]
fn summary_deferred_split_with_an_until_target_boundary() {
    let mut graph = boundary_graph();

    // The premises, named: the boundary cases really are the cases —
    // wontfix and superseded finish, the open gate and the open
    // predecessor hold.
    assert!(
        graph.is_until_satisfied(&task_id(1)),
        "a wontfix predecessor finishes"
    );
    assert!(
        graph.is_until_satisfied(&task_id(3)),
        "a superseded decision finishes"
    );
    assert!(
        !graph.is_until_satisfied(&task_id(4)),
        "the epic's open gate holds"
    );
    assert!(
        !graph.is_until_satisfied(&task_id(5)),
        "an open predecessor holds"
    );

    let summary = graph.status_summary();
    assert_eq!(summary.deferred, DeferredSummary { total: 4, ready: 2 });

    // Signing the epic's gate moves exactly U4 — the boundary the
    // signed text names ("an epic gate signed vs open").
    graph
        .update_decision(decision_id(2), at(9), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed by the user".to_owned());
        })
        .unwrap();
    assert!(graph.is_until_satisfied(&task_id(4)));
    assert_eq!(
        graph.status_summary().deferred,
        DeferredSummary { total: 4, ready: 3 }
    );
}

#[test]
fn summary_deferred_agrees_with_the_selection_tools() {
    // The signed "cannot disagree about what ready means": the
    // summary's deferred block equals `next_task`'s for the same
    // scope, repo and epic — on the boundary graph and over the seeded
    // graphs.
    let graph = boundary_graph();
    assert_eq!(graph.status_summary().deferred, graph.next_task().deferred);
    for epic in [epic_id(1), epic_id(2)] {
        assert_eq!(
            graph.status_summary_of_epic(&epic).unwrap().deferred,
            graph.next_task_of_epic(&epic).unwrap().deferred,
            "scope {epic}"
        );
    }

    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, SUMMARY_STEPS);
        assert_eq!(
            graph.status_summary().deferred,
            graph.next_task().deferred,
            "seed {seed}: repo scope"
        );
        for epic in graph
            .epics()
            .map(|epic| epic.id.clone())
            .collect::<Vec<_>>()
        {
            assert_eq!(
                graph.status_summary_of_epic(&epic).unwrap().deferred,
                graph.next_task_of_epic(&epic).unwrap().deferred,
                "seed {seed}: scope {epic}"
            );
        }
    }
}

// ---------------------------------------------------------------------
// 5./6. The epic scope's membership, and the one error
// ---------------------------------------------------------------------

#[test]
fn summary_epic_scope_membership() {
    // E1 (in_work) holds U1 open and U2 closed. D1 (open) gates E1
    // itself and U2 directly — both gate shapes, one node, counted
    // once. D2 (signed) gates U1. D3 (open) gates only E2's U3 — the
    // exclusion pin. S1 is a doc: no epic scope holds it.
    let mut graph = graph_with_epics(2);
    graph
        .update_epic(epic_id(1), |epic| epic.status = Status::InWork)
        .unwrap();
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Closed)).unwrap();
    graph.insert_task(task_node(3, 2, Status::Open)).unwrap();
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph
        .insert_decision(decision_node(2, DecisionStatus::Signed))
        .unwrap();
    graph
        .insert_decision(decision_node(3, DecisionStatus::Open))
        .unwrap();
    graph.insert_doc(doc_node(1)).unwrap();
    graph.link_blocks(gate_epic(1, 1)).unwrap();
    graph.link_blocks(gate_task(1, 2)).unwrap();
    graph.link_blocks(gate_task(2, 1)).unwrap();
    graph.link_blocks(gate_task(3, 3)).unwrap();

    // E1's scope: the epic at its own status, its tasks, D1 and D2 —
    // D1 once although it gates both the epic and U2 — and no docs.
    let e1 = graph.status_summary_of_epic(&epic_id(1)).unwrap();
    assert_eq!(
        e1,
        StatusSummary {
            epics: StatusCounts {
                in_work: 1,
                ..zero_counts()
            },
            tasks: StatusCounts {
                open: 1,
                closed: 1,
                ..zero_counts()
            },
            decisions: DecisionCounts {
                open: 1,
                signed: 1,
                ..zero_decisions()
            },
            docs: 0,
            open: OpenTotals {
                epics: 1,
                tasks: 1,
                decisions: 1,
            },
            deferred: DeferredSummary { total: 0, ready: 0 },
        }
    );

    // The exclusion half: D3, gating only E2's task, is absent from
    // E1's scope — pinned by the counts above (open: 1, not 2) and by
    // the contrast with the repo scope, which holds all three
    // decisions and the doc.
    let repo = graph.status_summary();
    assert_eq!(
        repo.decisions,
        DecisionCounts {
            open: 2,
            signed: 1,
            ..zero_decisions()
        }
    );
    assert_eq!(repo.docs, 1);

    // E2's scope: the open epic, its open task, and D3.
    let e2 = graph.status_summary_of_epic(&epic_id(2)).unwrap();
    assert_eq!(
        e2,
        StatusSummary {
            epics: StatusCounts {
                open: 1,
                ..zero_counts()
            },
            tasks: StatusCounts {
                open: 1,
                ..zero_counts()
            },
            decisions: DecisionCounts {
                open: 1,
                ..zero_decisions()
            },
            docs: 0,
            open: OpenTotals {
                epics: 1,
                tasks: 1,
                decisions: 1,
            },
            deferred: DeferredSummary { total: 0, ready: 0 },
        }
    );
}

#[test]
fn summary_unknown_epic_is_unknown_id() {
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    let before = digest(&graph);

    let result = graph.status_summary_of_epic(&epic_id(9));
    assert_eq!(outcome(&result), "unknown_id");
    assert_eq!(
        result,
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        }),
        "the payload must name the id"
    );
    // A rejected read touches nothing, and the repo-scoped answer on
    // the same graph is unaffected.
    assert_eq!(digest(&graph), before, "a rejected read touched state");
    assert_eq!(graph.status_summary().tasks.open, 1);
}

// ---------------------------------------------------------------------
// 7. State, not history: reruns and shuffled insertions
// ---------------------------------------------------------------------

/// One half of the build spec: (task number, epic number, status)
/// nodes.
type SpecNodes = Vec<(u64, u64, Status)>;
/// The other half: (from, to) task edges, acyclic by construction. The
/// two decisions, the doc and the three gate edges are fixed —
/// shuffling applies to the task nodes and the task edges.
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

/// Builds the sample state: three open epics, D1 open gating E2 and
/// U5 directly, D2 signed gating U4, one doc, the spec's tasks and
/// edges. U3 is deferred-ready (U2 closed), U6 deferred-held (U1
/// open). The gates land after the task inserts because
/// [`Graph::link_blocks`] requires its endpoints to exist.
fn build_spec(nodes: &[(u64, u64, Status)], edges: &[(u64, u64)]) -> Graph {
    let mut graph = graph_with_epics(3);
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph
        .insert_decision(decision_node(2, DecisionStatus::Signed))
        .unwrap();
    graph.insert_doc(doc_node(1)).unwrap();
    for (task, epic, status) in nodes {
        graph.insert_task(task_node(*task, *epic, *status)).unwrap();
    }
    graph.link_blocks(gate_epic(1, 2)).unwrap();
    graph.link_blocks(gate_task(1, 5)).unwrap();
    graph.link_blocks(gate_task(2, 4)).unwrap();
    for (from, to) in edges {
        graph.link_blocks(task_edge(*from, *to)).unwrap();
    }
    graph
}

#[test]
fn summary_is_deterministic() {
    let (nodes, edges) = sample_spec();
    let graph = build_spec(&nodes, &edges);

    // The expected answers, spelled out: the suite must know what the
    // state means, not only that two runs agree.
    let repo = StatusSummary {
        epics: StatusCounts {
            open: 3,
            ..zero_counts()
        },
        tasks: StatusCounts {
            open: 3,
            in_work: 1,
            blocked: 1,
            deferred: 2,
            closed: 1,
            ..zero_counts()
        },
        decisions: DecisionCounts {
            open: 1,
            signed: 1,
            ..zero_decisions()
        },
        docs: 1,
        open: OpenTotals {
            epics: 3,
            tasks: 7,
            decisions: 1,
        },
        deferred: DeferredSummary { total: 2, ready: 1 },
    };
    assert_eq!(graph.status_summary(), repo);
    let e1 = StatusSummary {
        epics: StatusCounts {
            open: 1,
            ..zero_counts()
        },
        tasks: StatusCounts {
            open: 1,
            closed: 1,
            deferred: 1,
            ..zero_counts()
        },
        decisions: zero_decisions(),
        docs: 0,
        open: OpenTotals {
            epics: 1,
            tasks: 2,
            decisions: 0,
        },
        deferred: DeferredSummary { total: 1, ready: 1 },
    };
    assert_eq!(graph.status_summary_of_epic(&epic_id(1)).unwrap(), e1);
    let e2 = StatusSummary {
        epics: StatusCounts {
            open: 1,
            ..zero_counts()
        },
        tasks: StatusCounts {
            open: 1,
            in_work: 1,
            deferred: 1,
            ..zero_counts()
        },
        decisions: DecisionCounts {
            open: 1,
            signed: 1,
            ..zero_decisions()
        },
        docs: 0,
        open: OpenTotals {
            epics: 1,
            tasks: 3,
            decisions: 1,
        },
        deferred: DeferredSummary { total: 1, ready: 0 },
    };
    assert_eq!(graph.status_summary_of_epic(&epic_id(2)).unwrap(), e2);
    let e3 = StatusSummary {
        epics: StatusCounts {
            open: 1,
            ..zero_counts()
        },
        tasks: StatusCounts {
            open: 1,
            blocked: 1,
            ..zero_counts()
        },
        decisions: zero_decisions(),
        docs: 0,
        open: OpenTotals {
            epics: 1,
            tasks: 2,
            decisions: 0,
        },
        deferred: DeferredSummary { total: 0, ready: 0 },
    };
    assert_eq!(graph.status_summary_of_epic(&epic_id(3)).unwrap(), e3);

    // Twice-run identity on unchanged state (§8).
    assert_eq!(graph.status_summary(), repo);
    for e in 1..=3 {
        assert_eq!(
            graph.status_summary_of_epic(&epic_id(e)).unwrap(),
            graph.status_summary_of_epic(&epic_id(e)).unwrap()
        );
    }

    // Isomorphic rebuilds whose nodes and edges land in seeded-shuffled
    // sequences: proven the same state by fingerprint, so the same
    // summaries — the roll-up reads state, never history.
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
            rebuilt.status_summary(),
            repo,
            "seed {seed} summarized differently"
        );
        for (index, expected) in [&e1, &e2, &e3].iter().enumerate() {
            assert_eq!(
                rebuilt
                    .status_summary_of_epic(&epic_id(index as u64 + 1))
                    .unwrap(),
                **expected,
                "seed {seed}: scope E{}",
                index + 1
            );
        }
    }
}

// ---------------------------------------------------------------------
// 8. Seeded graphs: agreement with an independent naive recomputation
// ---------------------------------------------------------------------

#[test]
fn summary_seeded_graphs_agree_with_an_independent_recomputation() {
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, SUMMARY_STEPS);
        assert_invariants(&graph);

        // Repo scope: the whole summary, and the default listing's
        // rows.
        assert_eq!(
            graph.status_summary(),
            naive_summary(&graph, None),
            "seed {seed}: repo summary"
        );
        assert_eq!(
            graph.task_list(&TaskFilter::default()).unwrap(),
            naive_rows(&graph, None),
            "seed {seed}: repo rows"
        );
        for epic in graph
            .epics()
            .map(|epic| epic.id.clone())
            .collect::<Vec<_>>()
        {
            assert_eq!(
                graph.status_summary_of_epic(&epic).unwrap(),
                naive_summary(&graph, Some(&epic)),
                "seed {seed}: scope {epic} summary"
            );
            assert_eq!(
                graph
                    .task_list(&TaskFilter {
                        epic: Some(epic.clone()),
                        ..Default::default()
                    })
                    .unwrap(),
                naive_rows(&graph, Some(&epic)),
                "seed {seed}: scope {epic} rows"
            );
        }

        // The same seed replayed gives the same summaries.
        let (_again, replayed) = run_sequence(seed, SUMMARY_STEPS);
        assert_eq!(
            replayed.status_summary(),
            graph.status_summary(),
            "seed {seed} diverged on replay"
        );
    }
}

/// The deliberately naive second implementation, written out longhand
/// the way `naive_select` is: it counts every kind through the public
/// accessors with the status matches spelled out by hand, derives the
/// epic-scope membership by scanning the whole edge listing, and
/// shares no counting and no row-building with production — a
/// recomputation that called [`Graph::status_summary`] or
/// [`Graph::blockers_of`] would prove nothing. The scope's task set is
/// read off the node fields (epic equality), not off the derived
/// order, so the agreement also pins that the order derivation and
/// the parent fields name the same set.
fn naive_summary(graph: &Graph, epic: Option<&EpicId>) -> StatusSummary {
    let mut summary = zero_summary();
    match epic {
        None => {
            for node in graph.epics() {
                naive_note_epic(&mut summary, node.status);
            }
        }
        Some(id) => {
            // The epic itself, once — read off the accessor.
            if let Some(node) = graph.epic(id) {
                naive_note_epic(&mut summary, node.status);
            }
        }
    }
    for task in graph
        .tasks()
        .filter(|task| epic.is_none_or(|id| &task.epic == id))
    {
        match task.status {
            Status::Open => summary.tasks.open += 1,
            Status::InWork => summary.tasks.in_work += 1,
            Status::Blocked => summary.tasks.blocked += 1,
            Status::Deferred => summary.tasks.deferred += 1,
            Status::Closed => summary.tasks.closed += 1,
            Status::Wontfix => summary.tasks.wontfix += 1,
        }
        if task.status == Status::Deferred {
            summary.deferred.total += 1;
            summary.deferred.ready += u64::from(naive_until_satisfied(graph, task));
        }
    }
    let gating: BTreeSet<DecisionId> = match epic {
        None => graph
            .decisions()
            .map(|decision| decision.id.clone())
            .collect(),
        Some(_) => {
            let (epic_gates, task_gates) = naive_gates(graph, epic);
            epic_gates.union(&task_gates).cloned().collect()
        }
    };
    for id in &gating {
        if let Some(decision) = graph.decision(id) {
            match decision.status {
                DecisionStatus::Open => summary.decisions.open += 1,
                DecisionStatus::Signed => summary.decisions.signed += 1,
                DecisionStatus::Superseded => summary.decisions.superseded += 1,
            }
        }
    }
    summary.docs = match epic {
        None => graph.docs().count() as u64,
        Some(_) => 0,
    };
    summary.open = OpenTotals {
        epics: summary.epics.open
            + summary.epics.in_work
            + summary.epics.blocked
            + summary.epics.deferred,
        tasks: summary.tasks.open
            + summary.tasks.in_work
            + summary.tasks.blocked
            + summary.tasks.deferred,
        decisions: summary.decisions.open,
    };
    summary
}

/// The naive epic-half of the counting, spelled out.
fn naive_note_epic(summary: &mut StatusSummary, status: Status) {
    match status {
        Status::Open => summary.epics.open += 1,
        Status::InWork => summary.epics.in_work += 1,
        Status::Blocked => summary.epics.blocked += 1,
        Status::Deferred => summary.epics.deferred += 1,
        Status::Closed => summary.epics.closed += 1,
        Status::Wontfix => summary.epics.wontfix += 1,
    }
}

/// The naive gate split of one epic scope: the decisions with an edge
/// into the epic itself, and the decisions with an edge into one of
/// its tasks — read off the whole edge listing, never off the
/// production scope walk.
fn naive_gates(
    graph: &Graph,
    epic: Option<&EpicId>,
) -> (BTreeSet<DecisionId>, BTreeSet<DecisionId>) {
    let mut epic_gates = BTreeSet::new();
    let mut task_gates = BTreeSet::new();
    let Some(epic) = epic else {
        return (epic_gates, task_gates);
    };
    for edge in graph.blocks_edges() {
        match edge {
            BlocksEdge::DecisionToEpic { from, to } => {
                if &to == epic {
                    epic_gates.insert(from);
                }
            }
            BlocksEdge::DecisionToTask { from, to } => {
                if graph.task(&to).is_some_and(|task| &task.epic == epic) {
                    task_gates.insert(from);
                }
            }
            BlocksEdge::TaskToTask { .. } => {}
        }
    }
    (epic_gates, task_gates)
}

/// The naive row builder: the scope's task list in the production
/// canonical order (production's by design — `tests/order.rs` pins it
/// independently), each row's blockers built by the naive two-run
/// builder below, never through [`Graph::blockers_of`] or
/// [`Graph::task_list`].
fn naive_rows(graph: &Graph, epic: Option<&EpicId>) -> Vec<TaskRow> {
    let order = match epic {
        None => graph.canonical_order(),
        Some(id) => graph.canonical_order_of_epic(id).unwrap(),
    };
    order
        .into_iter()
        .map(|id| TaskRow {
            blocked_by: naive_blockers(graph, &id),
            task: id,
        })
        .collect()
}

/// The naive two-run blocked-by builder: the task's own predecessors,
/// then its epic's gates, deduplicated at the first occurrence —
/// spelled out against the public accessors, statuses resolved by
/// hand, never through the production lift.
fn naive_blockers(graph: &Graph, task: &TaskId) -> Vec<Blocker> {
    let mut ends = graph.blocked_by(&BlockEnd::Task(task.clone()));
    if let Some(node) = graph.task(task) {
        for gate in graph.blocked_by(&BlockEnd::Epic(node.epic.clone())) {
            if !ends.contains(&gate) {
                ends.push(gate);
            }
        }
    }
    let mut blockers = Vec::new();
    for end in ends {
        match end {
            BlockEnd::Task(id) => {
                if let Some(node) = graph.task(&id) {
                    blockers.push(Blocker::Task {
                        id,
                        status: node.status,
                    });
                }
            }
            BlockEnd::Decision(id) => {
                if let Some(node) = graph.decision(&id) {
                    blockers.push(Blocker::Decision {
                        id,
                        status: node.status,
                    });
                }
            }
            BlockEnd::Epic(_) => {}
        }
    }
    blockers
}

// ---------------------------------------------------------------------
// 9. The coverage pin that keeps the agreement non-vacuous
// ---------------------------------------------------------------------

/// What the coverage pin observed on **one seed**, computed from the
/// built graphs through the naive helpers and the raw node fields —
/// never the draws and never the production predicates, so this pin
/// survives exactly the mutations it is meant to expose.
#[derive(Debug, Default)]
struct Coverage {
    deferred_ready: usize,
    deferred_held: usize,
    epic_scopes_gated_by_open: usize,
    epic_scopes_with_task_only_gates: usize,
}

impl Coverage {
    fn note(&mut self, graph: &Graph) {
        for task in graph.tasks() {
            if task.status == Status::Deferred {
                if naive_until_satisfied(graph, task) {
                    self.deferred_ready += 1;
                } else {
                    self.deferred_held += 1;
                }
            }
        }
        for epic in graph
            .epics()
            .map(|epic| epic.id.clone())
            .collect::<Vec<_>>()
        {
            let (epic_gates, task_gates) = naive_gates(graph, Some(&epic));
            if epic_gates.union(&task_gates).any(|id| {
                graph
                    .decision(id)
                    .is_some_and(|d| d.status == DecisionStatus::Open)
            }) {
                self.epic_scopes_gated_by_open += 1;
            }
            // The shape that makes the epic-scope agreement sensitive
            // to a membership walk that drops the task-edge half: a
            // decision gating one of the epic's tasks without gating
            // the epic itself.
            if task_gates.difference(&epic_gates).next().is_some() {
                self.epic_scopes_with_task_only_gates += 1;
            }
        }
    }
}

#[test]
fn summary_coverage_pin() {
    // f9's lesson: a property test whose interesting branches the
    // generator never reaches is green and vacuous. The assertions are
    // per seed: a generator change that narrows one seed's graphs
    // fails here instead of hiding behind the other four.
    for seed in GRAPH_SEEDS {
        let (_log, graph) = run_sequence(seed, SUMMARY_STEPS);
        let mut coverage = Coverage::default();
        coverage.note(&graph);
        assert!(
            coverage.deferred_ready > 0,
            "seed {seed}: no deferred task was ever ready, so the ready pins are \
             vacuous: {coverage:#?}"
        );
        assert!(
            coverage.deferred_held > 0,
            "seed {seed}: no deferred task was ever held back, so the held pins are \
             vacuous: {coverage:#?}"
        );
        assert!(
            coverage.epic_scopes_gated_by_open > 0,
            "seed {seed}: no epic scope was ever gated by an open decision, so the \
             epic-scope decision counts are vacuous: {coverage:#?}"
        );
        assert!(
            coverage.epic_scopes_with_task_only_gates > 0,
            "seed {seed}: no epic scope ever gained a gate through a task edge alone, \
             so the wider-gate reading is untested: {coverage:#?}"
        );
    }
}

// ---------------------------------------------------------------------
// 10./11. The rows: the two-run order, and status currency
// ---------------------------------------------------------------------

#[test]
fn summary_rows_carry_the_two_run_blocked_by_order() {
    // E1 is gated by D2 and D3. U1's own predecessors are U2, D3 and
    // D5 — D3 gates both the task and its epic, so it appears once, in
    // the own-edges run. The own run is [U2, D3, D5] (BlockEnd order:
    // tasks before decisions), the gate run after dedup is [D2], and
    // the answer is the concatenation — [U2, D3, D5, D2] — which a
    // globally sorted list is not.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    for n in 1..=5 {
        graph
            .insert_decision(decision_node(n, DecisionStatus::Open))
            .unwrap();
    }
    graph.link_blocks(task_edge(2, 1)).unwrap();
    graph.link_blocks(gate_task(3, 1)).unwrap();
    graph.link_blocks(gate_task(5, 1)).unwrap();
    graph.link_blocks(gate_epic(2, 1)).unwrap();
    graph.link_blocks(gate_epic(3, 1)).unwrap();

    let rows = graph.task_list(&TaskFilter::default()).unwrap();
    let row = rows
        .iter()
        .find(|row| row.task == task_id(1))
        .expect("U1 is in the listing");
    assert_eq!(
        row.blocked_by,
        vec![
            b_task(2, Status::Open),
            b_decision(3, DecisionStatus::Open),
            b_decision(5, DecisionStatus::Open),
            b_decision(2, DecisionStatus::Open),
        ]
    );
    // The whole is not globally sorted: the pin against a merge.
    let sequence = ends(&row.blocked_by);
    let mut sorted = sequence.clone();
    sorted.sort();
    assert_ne!(
        sequence, sorted,
        "a globally merged implementation answers the sorted list"
    );

    // The agreement pin: the id sequence equals `NextItem::blocked_by`'s
    // for the same task — U1 is U2's dependent, so the task scope
    // carries its row.
    let batch = graph.next_items_of_task(&task_id(2), u64::MAX).unwrap();
    let item = batch
        .items
        .iter()
        .find(|item| item.task == task_id(1))
        .expect("U1 is U2's dependent");
    assert_eq!(sequence, item.blocked_by);

    // The correctness pin: an independent naive two-run builder.
    assert_eq!(row.blocked_by, naive_blockers(&graph, &task_id(1)));
}

#[test]
fn summary_rows_carry_every_blockers_current_status() {
    // U1 open, held by the open U2 and the open D1; E1 ungated, so
    // the row's blockers are exactly its own predecessors.
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.link_blocks(task_edge(2, 1)).unwrap();
    graph.link_blocks(gate_task(1, 1)).unwrap();

    let row_of = |graph: &Graph| {
        graph
            .task_list(&TaskFilter::default())
            .unwrap()
            .into_iter()
            .find(|row| row.task == task_id(1))
            .expect("U1 is in the listing")
    };
    // The premises, named: both blockers really are open, so the first
    // assertion can fail.
    assert_eq!(graph.task(&task_id(2)).unwrap().status, Status::Open);
    assert_eq!(
        graph.decision(&decision_id(1)).unwrap().status,
        DecisionStatus::Open
    );
    assert_eq!(
        row_of(&graph).blocked_by,
        vec![b_task(2, Status::Open), b_decision(1, DecisionStatus::Open)]
    );

    // A mutation moves the row: closing U2 and signing D1 re-reads both
    // statuses at query time — a frozen or dropped status cannot
    // follow.
    graph
        .update_task(task_id(2), at(9), |task| {
            task.status = Status::Closed;
            task.evidence = Some("cargo test -p todo-domain -- summary: all green".to_owned());
        })
        .unwrap();
    graph
        .update_decision(decision_id(1), at(10), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed by the user".to_owned());
        })
        .unwrap();
    assert_eq!(
        row_of(&graph).blocked_by,
        vec![
            b_task(2, Status::Closed),
            b_decision(1, DecisionStatus::Signed)
        ]
    );
}
