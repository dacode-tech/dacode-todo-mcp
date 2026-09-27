//! §5.1's write-time cycle rejection: a `blocks` edge that would close a
//! cycle never lands, and the `cycle` error names a deterministic path.

use super::*;

/// The path payload the spec's `[s, t, …, s]` rule produces, spelled out.
fn path(ids: &[&str]) -> DomainError {
    DomainError::Cycle {
        path: ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

#[test]
fn graph_cycle_rejected_for_a_direct_two_cycle() {
    let mut graph = graph_with_tasks(1, 2);
    graph.link_blocks(task_edge(1, 2)).unwrap();
    let linked = graph.clone();

    // U2 -> U1 closes the cycle: path = [source, target, …, source].
    assert_eq!(
        graph.link_blocks(task_edge(2, 1)),
        Err(path(&["camera:U2", "camera:U1", "camera:U2"]))
    );
    // Decision 6: the rejected write left the graph bit-for-bit as it was.
    assert_eq!(graph, linked);
    assert_eq!(fingerprint(&graph), fingerprint(&linked));
    assert_eq!(graph.blocks_edges(), vec![task_edge(1, 2)]);
    assert_invariants(&graph);
}

#[test]
fn graph_cycle_rejected_for_a_self_edge() {
    let mut graph = graph_with_tasks(1, 1);
    assert_eq!(
        graph.link_blocks(task_edge(1, 1)),
        Err(path(&["camera:U1", "camera:U1"]))
    );
    assert!(graph.blocks_edges().is_empty());
    assert_eq!(graph, graph_with_tasks(1, 1));
    assert_invariants(&graph);
}

#[test]
fn graph_cycle_rejected_for_a_transitive_n_cycle() {
    let mut graph = graph_with_tasks(1, 5);
    for n in 1..5 {
        graph.link_blocks(task_edge(n, n + 1)).unwrap();
    }
    let linked = graph.clone();

    // Closing U5 -> U1 walks the whole chain back: the path is the new
    // edge's source, then the walk from its target to that source.
    assert_eq!(
        graph.link_blocks(task_edge(5, 1)),
        Err(path(&[
            "camera:U5",
            "camera:U1",
            "camera:U2",
            "camera:U3",
            "camera:U4",
            "camera:U5",
        ]))
    );
    assert_eq!(graph, linked);
    assert_invariants(&graph);
}

#[test]
fn graph_cycle_path_is_the_sorted_first_walk_and_is_repeatable() {
    // Two ways back from U4 to U1: through U2 and through U3. The walk
    // visits successors in ascending order, so U2's branch is reported.
    let mut graph = graph_with_tasks(1, 4);
    for edge in [
        task_edge(1, 2),
        task_edge(1, 3),
        task_edge(2, 4),
        task_edge(3, 4),
    ] {
        graph.link_blocks(edge).unwrap();
    }
    let expected = Err(path(&["camera:U4", "camera:U1", "camera:U2", "camera:U4"]));
    assert_eq!(graph.link_blocks(task_edge(4, 1)), expected);
    // Same state, same attempted edge: byte-identical error, twice.
    assert_eq!(graph.link_blocks(task_edge(4, 1)), expected);
    assert_invariants(&graph);
}

#[test]
fn graph_cycle_error_uses_formatted_ids_in_edge_order() {
    // The walk is over edges, so the payload reads as the cycle does:
    // source, target, and the chain back to the source.
    let mut graph = graph_with_tasks(1, 3);
    graph.link_blocks(task_edge(2, 3)).unwrap();
    graph.link_blocks(task_edge(3, 1)).unwrap();
    assert_eq!(
        graph.link_blocks(task_edge(1, 2)),
        Err(path(&["camera:U1", "camera:U2", "camera:U3", "camera:U1"]))
    );
    assert_eq!(
        graph.link_blocks(task_edge(1, 2)).unwrap_err().to_string(),
        "cycle: camera:U1 -> camera:U2 -> camera:U3 -> camera:U1"
    );
}

#[test]
fn graph_relinking_an_existing_edge_is_not_a_cycle() {
    // The edge already in the graph is itself a path from source to
    // target, so a repeat link must be a no-op, not a false rejection.
    let mut graph = graph_with_tasks(1, 2);
    graph.link_blocks(task_edge(1, 2)).unwrap();
    let linked = graph.clone();
    assert_eq!(graph.link_blocks(task_edge(1, 2)), Ok(()));
    assert_eq!(graph, linked);
    // Unlinking frees the direction again, and the reverse stays closed.
    graph.unlink_blocks(task_edge(1, 2)).unwrap();
    assert_eq!(graph.link_blocks(task_edge(2, 1)), Ok(()));
    assert_eq!(
        graph.link_blocks(task_edge(1, 2)),
        Err(path(&["camera:U1", "camera:U2", "camera:U1"]))
    );
    assert_invariants(&graph);
}

#[test]
fn graph_a_dag_admits_every_edge_that_closes_nothing() {
    // A decision source cannot be in a cycle: nothing can point at a
    // decision, so gating edges are always admissible, however they fan
    // out over tasks and epics.
    let mut graph = graph_with_tasks(2, 2);
    graph
        .add_decision(decision_draft("which encoder"), at(1))
        .unwrap();
    graph.link_blocks(task_edge(1, 2)).unwrap();
    for edge in [
        gate_task(1, 1),
        gate_task(1, 2),
        gate_epic(1, 1),
        gate_epic(1, 2),
        task_edge(3, 4),
    ] {
        let detail = edge.to_string();
        assert_eq!(
            graph.link_blocks(edge),
            Ok(()),
            "{detail} must be admissible"
        );
    }
    assert_eq!(graph.blocks_edges().len(), 6);
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(2))),
        vec![
            BlockEnd::Task(task_id(1)),
            BlockEnd::Decision(decision_id(1))
        ]
    );
    assert_invariants(&graph);
}

#[test]
fn graph_a_rejected_cycle_burns_no_id_and_touches_no_counter() {
    let mut graph = graph_with_tasks(1, 2);
    graph.link_blocks(task_edge(1, 2)).unwrap();
    let counters = graph.counters();
    assert!(graph.link_blocks(task_edge(2, 1)).is_err());
    assert_eq!(graph.counters(), counters);
    // The next allocation still advances from where it was.
    let task = graph.add_task(task_draft(&epic_id(1)), at(2)).unwrap();
    assert_eq!(task.id.number(), 3);
}
