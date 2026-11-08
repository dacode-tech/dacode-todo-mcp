//! §5.1's two edge kinds as the graph stores them: decision gating
//! (D10) and the `relates` annotation, plus the total order every
//! adjacency listing returns (§8).

use super::*;

/// A graph with two epics, three tasks (U1/U2 under E1, U3 under E2) and
/// one open decision, the fixture the edge tests share.
fn fixture() -> Graph {
    let mut graph = graph_with_tasks(2, 1);
    graph.add_task(task_draft(&epic_id(1)), at(0)).unwrap();
    graph
        .add_decision(decision_draft("which encoder"), at(0))
        .unwrap();
    graph
}

#[test]
fn graph_decision_gates_store_and_unlink_in_both_directions() {
    let mut graph = fixture();
    graph.link_blocks(gate_task(1, 1)).unwrap();
    graph.link_blocks(gate_epic(1, 2)).unwrap();

    assert_eq!(
        graph.blocks(&BlockEnd::Decision(decision_id(1))),
        vec![BlockEnd::Task(task_id(1)), BlockEnd::Epic(epic_id(2))]
    );
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(1))),
        vec![BlockEnd::Decision(decision_id(1))]
    );
    assert_eq!(
        graph.blocked_by(&BlockEnd::Epic(epic_id(2))),
        vec![BlockEnd::Decision(decision_id(1))]
    );
    assert_eq!(graph.blocks_edges(), vec![gate_task(1, 1), gate_epic(1, 2)]);
    assert_invariants(&graph);

    // Unlinking clears both indexes; the decision node is untouched.
    graph.unlink_blocks(gate_task(1, 1)).unwrap();
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(1))),
        Vec::<BlockEnd>::new()
    );
    assert_eq!(graph.blocks_edges(), vec![gate_epic(1, 2)]);
    graph.unlink_blocks(gate_epic(1, 2)).unwrap();
    assert!(graph.blocks_edges().is_empty());
    assert_eq!(graph.decisions().count(), 1);
    // An edge that was never there is not an error to remove, and both
    // indexes are back to empty — structurally equal to a graph that
    // never held them.
    assert_eq!(graph.unlink_blocks(gate_epic(1, 2)), Ok(()));
    assert_eq!(graph, fixture());
    assert_invariants(&graph);
}

#[test]
fn graph_signing_a_decision_leaves_its_edges_intact() {
    let mut graph = fixture();
    for edge in [gate_task(1, 1), gate_task(1, 3), gate_epic(1, 1)] {
        graph.link_blocks(edge).unwrap();
    }
    let edges = graph.blocks_edges();
    assert_eq!(edges.len(), 3);

    // §3/D10: signing flips the status and the edges simply stop gating —
    // no edge mutation, the history stays.
    let signed = graph
        .update_decision(decision_id(1), at(5), |decision| {
            decision.status = DecisionStatus::Signed;
            decision.resolution = Some("signed (a) 2026-09-26".to_owned());
        })
        .unwrap();
    assert!(signed.status.is_resolved());
    assert_eq!(signed.updated, at(5));
    assert_eq!(graph.blocks_edges(), edges);
    assert_eq!(
        graph.blocks(&BlockEnd::Decision(decision_id(1))),
        vec![
            BlockEnd::Task(task_id(1)),
            BlockEnd::Task(task_id(3)),
            BlockEnd::Epic(epic_id(1))
        ]
    );
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(3))),
        vec![BlockEnd::Decision(decision_id(1))]
    );

    // A signing without a resolution is rejected and changes no edge.
    let mut graph = fixture();
    graph.link_blocks(gate_task(1, 1)).unwrap();
    let before = graph.clone();
    assert_eq!(
        graph.update_decision(decision_id(1), at(6), |decision| {
            decision.status = DecisionStatus::Signed;
        }),
        Err(DomainError::MissingResolution {
            id: "camera:D1".to_owned()
        })
    );
    assert_eq!(graph, before);
    assert_invariants(&graph);
}

#[test]
fn graph_a_decision_is_never_a_blocks_target() {
    // The forbidden direction is not an error at link time: it has no
    // representation, so a caller cannot ask for it.
    let target = BlockEnd::Decision(decision_id(1));
    assert_eq!(
        BlocksEdge::from_ends(BlockEnd::Task(task_id(1)), target.clone()),
        None
    );
    assert_eq!(BlocksEdge::from_ends(target.clone(), target), None);
    // Nor can an epic or a doc be a source (D2's named follow-up).
    assert_eq!(
        BlocksEdge::from_ends(BlockEnd::Epic(epic_id(1)), BlockEnd::Task(task_id(1))),
        None
    );
    assert_eq!(
        BlocksEdge::from_ends(BlockEnd::Epic(epic_id(1)), BlockEnd::Epic(epic_id(2))),
        None
    );
    assert_eq!(
        BlocksEdge::from_ends(BlockEnd::Task(task_id(1)), BlockEnd::Epic(epic_id(1))),
        None
    );
}

#[test]
fn graph_relates_links_unlinks_and_deduplicates_canonically() {
    let mut graph = graph_with_tasks(1, 2);
    graph.link_relates(task_id(1), task_id(2)).unwrap();
    // The other spelling is the same annotation, not a second one.
    graph.link_relates(task_id(2), task_id(1)).unwrap();
    assert_eq!(graph.relates_pairs(), vec![(task_id(1), task_id(2))]);
    assert_eq!(graph.relates_of(&task_id(1)), vec![task_id(2)]);
    assert_eq!(graph.relates_of(&task_id(2)), vec![task_id(1)]);
    // An annotation is not an ordering edge: it appears in no adjacency.
    assert!(graph.blocks(&BlockEnd::Task(task_id(1))).is_empty());
    assert!(graph.blocked_by(&BlockEnd::Task(task_id(2))).is_empty());
    assert!(graph.blocks_edges().is_empty());
    assert_invariants(&graph);

    let linked = graph.clone();
    graph.unlink_relates(task_id(2), task_id(1)).unwrap();
    assert!(graph.relates_pairs().is_empty());
    assert!(graph.relates_of(&task_id(1)).is_empty());
    // Removing it again changes nothing.
    graph.unlink_relates(task_id(1), task_id(2)).unwrap();
    assert_eq!(graph.relates_pairs(), Vec::new());
    // And the graph is still the two tasks it started as.
    assert_eq!(graph.tasks().count(), linked.tasks().count());

    // A self-annotation is accepted and inert: nothing traverses it.
    graph.link_relates(task_id(1), task_id(1)).unwrap();
    assert_eq!(graph.relates_pairs(), vec![(task_id(1), task_id(1))]);
    assert!(graph.blocks_edges().is_empty());
    assert!(!holds_cycle(&graph));
    assert_invariants(&graph);
}

#[test]
fn graph_relates_is_removed_with_a_cascaded_task() {
    let mut graph = graph_with_tasks(2, 1);
    graph.link_relates(task_id(1), task_id(2)).unwrap();
    graph
        .update_task(task_id(1), at(1), |task| {
            task.status = Status::Wontfix;
        })
        .unwrap();
    assert_eq!(graph.delete_epic(epic_id(1)), Ok(vec![task_id(1)]));
    assert!(graph.relates_pairs().is_empty());
    assert!(graph.relates_of(&task_id(2)).is_empty());
    assert_invariants(&graph);
}

#[test]
fn graph_edge_listings_are_totally_ordered_and_repeatable() {
    let mut graph = fixture();
    // Inserted out of listing order on purpose.
    for edge in [
        gate_epic(1, 1),
        task_edge(1, 3),
        gate_task(1, 2),
        task_edge(1, 2),
    ] {
        graph.link_blocks(edge).unwrap();
    }
    // Sources ascending in BlockEnd order (tasks before decisions), and
    // within a source the edge's own order.
    let expected = vec![
        task_edge(1, 2),
        task_edge(1, 3),
        gate_task(1, 2),
        gate_epic(1, 1),
    ];
    assert_eq!(graph.blocks_edges(), expected);
    assert_eq!(graph.blocks_edges(), expected, "twice-run identical");
    assert_eq!(
        graph.blocks(&BlockEnd::Task(task_id(1))),
        vec![BlockEnd::Task(task_id(2)), BlockEnd::Task(task_id(3))]
    );
    assert_eq!(
        graph.blocks(&BlockEnd::Decision(decision_id(1))),
        vec![BlockEnd::Task(task_id(2)), BlockEnd::Epic(epic_id(1))]
    );
    let bytes = fingerprint(&graph);
    assert_eq!(fingerprint(&graph), bytes, "twice-run byte identity");
    assert_invariants(&graph);
}
