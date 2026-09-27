//! The canonical-order tests (spec §5.2, §10): the tiebreaker levels,
//! byte identity across twice-runs and shuffled insertions, the
//! epic-scoped induced subgraph, status/priority/`relates` indifference,
//! seeded validity and key-minimality against an independent naive
//! recomputation, and the degenerate shapes.
//!
//! The naive recomputation is deliberately a second, slower
//! implementation reading only the public accessors (`tasks`,
//! `blocked_by`), so agreement between the two cannot come from a
//! shared bug in the production frontier walk.

use std::collections::{BTreeMap, BTreeSet};

use crate::testcodec::to_bytes;

use super::seeded::Rng;
use super::*;

/// Fixed seeds, per the suite's rule that a determinism test never
/// reseeds from a clock.
const SEEDS: [u64; 5] = [5, 23, 0xCAFE, 0x0DD0_2026, 0xBEEF_0007];

// ---------------------------------------------------------------------
// 1. The tiebreaker levels
// ---------------------------------------------------------------------

#[test]
fn order_epic_registration_beats_task_id() {
    // Explicit-id insertion lets a later epic hold a smaller task
    // number: U9 under E1, U1 under E2. Ascending task id would say
    // [U1, U9]; D6 says epic registration order decides: [U9, U1].
    let mut graph = graph_with_epics(2);
    graph.insert_task(task_node(9, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(1, 2, Status::Open)).unwrap();
    assert_eq!(graph.canonical_order(), vec![task_id(9), task_id(1)]);

    // The same fact through the allocating API: E2's first task (U1)
    // still follows E1's later task (U2).
    let mut graph = graph_with_epics(2);
    let first = graph.add_task(task_draft(&epic_id(2)), at(0)).unwrap();
    let second = graph.add_task(task_draft(&epic_id(1)), at(1)).unwrap();
    assert_eq!(first.id, task_id(1));
    assert_eq!(second.id, task_id(2));
    assert_eq!(graph.canonical_order(), vec![task_id(2), task_id(1)]);
}

#[test]
fn order_within_one_epic_is_ascending_task_id() {
    // Inserted out of order through the replay path: the output is key
    // order, not arrival order.
    let mut graph = graph_with_epics(1);
    for n in [3, 1, 2] {
        graph.insert_task(task_node(n, 1, Status::Open)).unwrap();
    }
    assert_eq!(
        graph.canonical_order(),
        vec![task_id(1), task_id(2), task_id(3)]
    );
}

#[test]
fn order_topology_beats_the_key() {
    // U2 (E2) blocks U1 (E1): every key level prefers U1, and topology
    // overrides all of them.
    let mut graph = graph_with_epics(2);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 2, Status::Open)).unwrap();
    graph.link_blocks(task_edge(2, 1)).unwrap();
    assert_eq!(graph.canonical_order(), vec![task_id(2), task_id(1)]);
}

#[test]
fn order_diamond_has_one_exact_sequence() {
    // U4 -> {U2, U3} -> U1 with U2 under E1 and U3 under E2: after U4
    // the frontier holds both arms and the epic level picks U2.
    let mut graph = graph_with_epics(2);
    for (n, epic) in [(1, 1), (2, 1), (3, 2), (4, 1)] {
        graph.insert_task(task_node(n, epic, Status::Open)).unwrap();
    }
    for (from, to) in [(4, 2), (4, 3), (2, 1), (3, 1)] {
        graph.link_blocks(task_edge(from, to)).unwrap();
    }
    assert_eq!(
        graph.canonical_order(),
        vec![task_id(4), task_id(2), task_id(3), task_id(1)]
    );
}

#[test]
fn order_composes_all_three_tiebreaker_levels() {
    // E1: U2, U3 independent — the task-id level decides between them.
    // E2: U4 blocks U1 — topology decides inside the later epic, and the
    // epic level holds both behind E1's ready tasks.
    let mut graph = graph_with_epics(2);
    for (n, epic) in [(1, 2), (2, 1), (3, 1), (4, 2)] {
        graph.insert_task(task_node(n, epic, Status::Open)).unwrap();
    }
    graph.link_blocks(task_edge(4, 1)).unwrap();
    assert_eq!(
        graph.canonical_order(),
        vec![task_id(2), task_id(3), task_id(4), task_id(1)]
    );
}

// ---------------------------------------------------------------------
// 2. Byte identity: twice-run and insertion-order robustness
// ---------------------------------------------------------------------

/// One half of the build spec: a list of (first number, second number)
/// pairs — nodes read it as (task, epic), edges as (from, to).
type SpecPairs = Vec<(u64, u64)>;

/// A fixed isomorphic-build spec: (task number, epic number) nodes and
/// (from, to) task edges, acyclic by construction.
fn sample_spec() -> (SpecPairs, SpecPairs) {
    (
        vec![
            (1, 1),
            (2, 1),
            (3, 2),
            (4, 2),
            (5, 3),
            (6, 3),
            (7, 1),
            (8, 2),
            (9, 3),
            (10, 1),
            (11, 2),
            (12, 3),
        ],
        vec![
            (1, 2),
            (2, 7),
            (3, 4),
            (4, 8),
            (5, 6),
            (6, 9),
            (10, 5),
            (7, 10),
            (8, 12),
            (9, 12),
            (3, 11),
            (11, 12),
        ],
    )
}

fn build_spec(nodes: &[(u64, u64)], edges: &[(u64, u64)]) -> Graph {
    let mut graph = graph_with_epics(3);
    for (task, epic) in nodes {
        graph
            .insert_task(task_node(*task, *epic, Status::Open))
            .unwrap();
    }
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

#[test]
fn order_bytes_survive_reruns_and_shuffled_insertion() {
    let (nodes, edges) = sample_spec();
    let graph = build_spec(&nodes, &edges);
    let bytes = to_bytes(&graph.canonical_order()).unwrap();
    // Twice-run byte identity on unchanged state (§8).
    assert_eq!(to_bytes(&graph.canonical_order()).unwrap(), bytes);

    // Isomorphic rebuilds whose nodes and edges land in seeded-shuffled
    // sequences: same state, so same order bytes — the derivation reads
    // state, never history.
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
            to_bytes(&rebuilt.canonical_order()).unwrap(),
            bytes,
            "seed {seed} derived a different order"
        );
    }
}

// ---------------------------------------------------------------------
// 3. Epic scope: induced subgraph, unknown id
// ---------------------------------------------------------------------

#[test]
fn order_epic_scope_is_the_induced_subgraph() {
    // E1: U1 -> U2 -> U3; E2: U5 -> U4; cross-epic U3 -> U5.
    let mut graph = graph_with_epics(2);
    for (n, epic) in [(1, 1), (2, 1), (3, 1), (4, 2), (5, 2)] {
        graph.insert_task(task_node(n, epic, Status::Open)).unwrap();
    }
    for (from, to) in [(1, 2), (2, 3), (3, 5), (5, 4)] {
        graph.link_blocks(task_edge(from, to)).unwrap();
    }
    assert_eq!(
        graph.canonical_order(),
        vec![task_id(1), task_id(2), task_id(3), task_id(5), task_id(4)]
    );
    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(1)).unwrap(),
        vec![task_id(1), task_id(2), task_id(3)]
    );
    // E2's scoped order ignores the cross-epic predecessor U3 — U5
    // leads even though the full order holds it behind U3 ...
    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(2)).unwrap(),
        vec![task_id(5), task_id(4)]
    );
    // ... and the blocker stays visible through the adjacency M2's
    // actionability check will consult.
    assert!(
        graph
            .blocked_by(&BlockEnd::Task(task_id(5)))
            .contains(&BlockEnd::Task(task_id(3)))
    );
}

#[test]
fn order_epic_scope_rejects_an_unknown_epic_and_changes_nothing() {
    let graph = graph_with_tasks(2, 2);
    let before = digest(&graph);
    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(9)),
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        })
    );
    assert_eq!(digest(&graph), before, "a rejected read touched state");
}

#[test]
fn order_epic_scope_of_a_taskless_epic_is_empty() {
    let mut graph = graph_with_tasks(1, 2);
    let fresh = graph.add_epic(epic_draft("empty")).unwrap();
    assert!(graph.canonical_order_of_epic(&fresh.id).unwrap().is_empty());
    // The repo-scoped order still holds the other epic's tasks.
    assert_eq!(graph.canonical_order(), vec![task_id(1), task_id(2)]);
}

#[test]
fn order_decision_gates_do_not_constrain() {
    // An open decision gating U2 and E1 is M2's actionability input;
    // the order sees only `TaskToTask` edges (Decision 1).
    let mut graph = graph_with_epics(1);
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    let decision = graph
        .add_decision(decision_draft("which encoder"), at(0))
        .unwrap();
    graph
        .link_blocks(gate_task(decision.id.number(), 2))
        .unwrap();
    graph
        .link_blocks(gate_epic(decision.id.number(), 1))
        .unwrap();
    assert_eq!(graph.canonical_order(), vec![task_id(1), task_id(2)]);
    assert_eq!(
        graph.canonical_order_of_epic(&epic_id(1)).unwrap(),
        vec![task_id(1), task_id(2)]
    );
}

// ---------------------------------------------------------------------
// 4./5. Status, priority and relates never move the order
// ---------------------------------------------------------------------

#[test]
fn order_does_not_filter_on_status() {
    let mut graph = graph_with_epics(1);
    for n in 1..=4 {
        graph.insert_task(task_node(n, 1, Status::Open)).unwrap();
    }
    for (from, to) in [(1, 2), (2, 3)] {
        graph.link_blocks(task_edge(from, to)).unwrap();
    }
    let before = to_bytes(&graph.canonical_order()).unwrap();

    // Close U2 through the real update API, with the evidence the
    // closure discipline demands, and wontfix U4: neither moves nor
    // drops — order is topology, actionability is M2.
    graph
        .update_task(task_id(2), at(10), |task| {
            task.status = Status::Closed;
            task.evidence = Some("cargo test -p todo-domain -- order: all green".to_owned());
        })
        .unwrap();
    graph
        .update_task(task_id(4), at(11), |task| {
            task.status = Status::Wontfix;
        })
        .unwrap();

    let after = graph.canonical_order();
    assert_eq!(to_bytes(&after).unwrap(), before);
    assert_eq!(after, vec![task_id(1), task_id(2), task_id(3), task_id(4)]);
    assert_eq!(graph.task(&task_id(2)).unwrap().status, Status::Closed);
    assert_eq!(graph.task(&task_id(4)).unwrap().status, Status::Wontfix);
}

#[test]
fn order_ignores_priority_and_relates() {
    let (nodes, edges) = sample_spec();
    let mut plain = build_spec(&nodes, &edges);

    // The same graph with every task carrying an agent-set priority
    // (D11: payload metadata — it never enters the tiebreaker).
    let mut decorated = graph_with_epics(3);
    for (i, (task, epic)) in nodes.iter().enumerate() {
        let mut node = task_node(*task, *epic, Status::Open);
        node.priority = Some(match i % 3 {
            0 => Priority::High,
            1 => Priority::Low,
            _ => Priority::Medium,
        });
        decorated.insert_task(node).unwrap();
    }
    for (from, to) in &edges {
        decorated.link_blocks(task_edge(*from, *to)).unwrap();
    }
    assert_eq!(
        to_bytes(&decorated.canonical_order()).unwrap(),
        to_bytes(&plain.canonical_order()).unwrap(),
        "priority reordered the derivation"
    );

    // `relates` is an annotation with no ordering semantics (§5.1):
    // linking pairs — even against the blocks direction — changes
    // neither scope of the order.
    let repo_before = to_bytes(&plain.canonical_order()).unwrap();
    let epic_before = to_bytes(&plain.canonical_order_of_epic(&epic_id(3)).unwrap()).unwrap();
    plain.link_relates(task_id(12), task_id(1)).unwrap();
    plain.link_relates(task_id(4), task_id(3)).unwrap();
    assert_eq!(to_bytes(&plain.canonical_order()).unwrap(), repo_before);
    assert_eq!(
        to_bytes(&plain.canonical_order_of_epic(&epic_id(3)).unwrap()).unwrap(),
        epic_before
    );
}

// ---------------------------------------------------------------------
// 6. Seeded DAGs: validity, minimality, remainder never fires
// ---------------------------------------------------------------------

#[test]
fn order_seeded_dags_are_valid_and_key_minimal() {
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let mut graph = empty_graph();
        let epic_count = 2 + rng.below(3);
        for e in 1..=epic_count {
            graph.add_epic(epic_draft(&format!("epic {e}"))).unwrap();
        }
        let task_count = 12 + rng.below(24);
        let mut ids: Vec<TaskId> = Vec::new();
        for _ in 0..task_count {
            let epic = epic_id(1 + rng.below(epic_count) as u64);
            let task = graph.add_task(task_draft(&epic), at(0)).unwrap();
            ids.push(task.id);
        }
        // Random edges; a cycle rejection is deterministic and simply
        // skips the candidate, so the result is always a real DAG.
        for _ in 0..2 * task_count {
            let from = ids[rng.below(ids.len())].clone();
            let to = ids[rng.below(ids.len())].clone();
            let _ = graph.link_blocks(BlocksEdge::TaskToTask { from, to });
        }
        assert_invariants(&graph);

        check_order(&graph, None, &ids, seed);
        for e in 1..=epic_count {
            let epic = epic_id(e as u64);
            let scoped: Vec<TaskId> = ids
                .iter()
                .filter(|id| graph.task(id).unwrap().epic == epic)
                .cloned()
                .collect();
            check_order(&graph, Some(&epic), &scoped, seed);
        }
    }
}

/// Asserts the derivation contract for one scope: the Kahn remainder
/// never fires, the output is exactly a permutation of the in-scope
/// tasks, every in-scope `TaskToTask` edge has `from` before `to`, and
/// the order equals the independent naive recomputation — i.e. it is the
/// lexicographically-minimal topological order under the key.
fn check_order(graph: &Graph, scope: Option<&EpicId>, in_scope: &[TaskId], seed: u64) {
    let order = match scope {
        None => graph.canonical_order(),
        Some(epic) => graph.canonical_order_of_epic(epic).unwrap(),
    };
    let (_, remainder) = graph.derive_order(scope);
    assert_eq!(remainder, 0, "seed {seed}: Kahn left a remainder");

    let mut sorted = order.clone();
    sorted.sort();
    let mut expected = in_scope.to_vec();
    expected.sort();
    assert_eq!(sorted, expected, "seed {seed}: not a permutation");
    let unique: BTreeSet<&TaskId> = order.iter().collect();
    assert_eq!(unique.len(), order.len(), "seed {seed}: duplicate placed");

    let position: BTreeMap<&TaskId, usize> =
        order.iter().enumerate().map(|(i, id)| (id, i)).collect();
    for edge in graph.blocks_edges() {
        if let BlocksEdge::TaskToTask { from, to } = &edge
            && position.contains_key(from)
            && position.contains_key(to)
        {
            assert!(
                position[from] < position[to],
                "seed {seed}: {edge} inverted"
            );
        }
    }

    assert_eq!(order, naive_order(graph, scope), "seed {seed}: not minimal");
}

/// The deliberately naive second implementation: repeatedly scan every
/// unplaced in-scope task and take the key-minimum one whose in-scope
/// task predecessors are all placed. O(n²) and reading only the public
/// accessors, so it shares no structure with the production walk.
fn naive_order(graph: &Graph, scope: Option<&EpicId>) -> Vec<TaskId> {
    let candidates: Vec<&Task> = graph
        .tasks()
        .filter(|task| scope.is_none_or(|epic| task.epic == *epic))
        .collect();
    let in_scope = |id: &TaskId| candidates.iter().any(|task| &task.id == id);
    let mut placed: BTreeSet<TaskId> = BTreeSet::new();
    let mut order: Vec<TaskId> = Vec::new();
    while order.len() < candidates.len() {
        let mut best: Option<&Task> = None;
        for task in &candidates {
            if placed.contains(&task.id) {
                continue;
            }
            let ready = graph
                .blocked_by(&BlockEnd::Task(task.id.clone()))
                .iter()
                .all(|end| match end {
                    // A decision gate is no order constraint, and an
                    // out-of-scope predecessor does not count (induced
                    // subgraph).
                    BlockEnd::Task(predecessor) => {
                        !in_scope(predecessor) || placed.contains(predecessor)
                    }
                    BlockEnd::Epic(_) | BlockEnd::Decision(_) => true,
                });
            if ready && best.is_none_or(|current| key(task) < key(current)) {
                best = Some(task);
            }
        }
        let Some(task) = best else { break };
        placed.insert(task.id.clone());
        order.push(task.id.clone());
    }
    order
}

/// The tiebreaker key as the tests understand it: (epic number, task
/// number) — written out here independently of the production `order_key`.
fn key(task: &Task) -> (u64, u64) {
    (task.epic.number(), task.id.number())
}

// ---------------------------------------------------------------------
// 7. Degenerate shapes
// ---------------------------------------------------------------------

#[test]
fn order_degenerate_shapes() {
    // The empty graph.
    let graph = empty_graph();
    assert!(graph.canonical_order().is_empty());

    // A single task, both scopes.
    let mut single = graph_with_epics(1);
    single.insert_task(task_node(1, 1, Status::Open)).unwrap();
    assert_eq!(single.canonical_order(), vec![task_id(1)]);
    assert_eq!(
        single.canonical_order_of_epic(&epic_id(1)).unwrap(),
        vec![task_id(1)]
    );

    // No edges: pure key order, which interleaves differently from
    // ascending id once epics alternate.
    let mut flat = graph_with_epics(2);
    for e in [1, 2, 1, 2, 1, 2] {
        flat.add_task(task_draft(&epic_id(e)), at(0)).unwrap();
    }
    assert_eq!(
        flat.canonical_order(),
        vec![
            task_id(1),
            task_id(3),
            task_id(5),
            task_id(2),
            task_id(4),
            task_id(6)
        ]
    );

    // A long chain whose topology fully reverses the key order.
    const CHAIN: u64 = 100;
    let mut chain = graph_with_epics(1);
    for _ in 0..CHAIN {
        chain.add_task(task_draft(&epic_id(1)), at(0)).unwrap();
    }
    for n in (1..CHAIN).rev() {
        chain.link_blocks(task_edge(n + 1, n)).unwrap();
    }
    let expected: Vec<TaskId> = (1..=CHAIN).rev().map(task_id).collect();
    assert_eq!(chain.canonical_order(), expected);
}
