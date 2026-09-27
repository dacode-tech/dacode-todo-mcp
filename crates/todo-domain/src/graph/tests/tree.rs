//! §5.1's `parent` tree: no orphans ever — reparenting moves a task, and
//! an epic is deletable only once its tasks have moved or closed.

use super::*;

/// Closes a task with the evidence `closed` requires.
fn close(graph: &mut Graph, number: u64, step: u64) {
    graph
        .update_task(task_id(number), at(step), |task| {
            task.status = Status::Closed;
            task.evidence = Some("42 passed".to_owned());
        })
        .unwrap();
}

#[test]
fn graph_orphan_rule_rejects_epic_delete_with_an_unfinished_task() {
    let mut graph = graph_with_tasks(1, 1);
    let before = graph.clone();
    assert_eq!(
        graph.delete_epic(epic_id(1)),
        Err(DomainError::OrphanNotAllowed {
            id: "camera:U1".to_owned()
        })
    );
    // Nothing moved: the epic, its task and the counters all stand.
    assert_eq!(graph, before);
    assert_invariants(&graph);

    // Every non-terminal status is an orphan in the making, including the
    // two that carry a note: `blocked` and `deferred` are not closed.
    for status in [
        Status::InWork,
        Status::Blocked,
        Status::Deferred,
        Status::Open,
    ] {
        let mut graph = graph_with_tasks(1, 1);
        graph
            .update_task(task_id(1), at(1), |task| {
                task.status = status;
                task.note = Some("held by a predecessor".to_owned());
            })
            .unwrap();
        assert_eq!(
            graph.delete_epic(epic_id(1)),
            Err(DomainError::OrphanNotAllowed {
                id: "camera:U1".to_owned()
            }),
            "{status} must block the delete"
        );
        assert!(graph.epic(&epic_id(1)).is_some());
        assert!(graph.task(&task_id(1)).is_some());
    }
}

#[test]
fn graph_orphan_rule_names_the_first_offender_in_id_order() {
    // U2 is closed, U1 and U3 are not: the error names U1, and the same
    // state names the same task on every run.
    let mut graph = graph_with_tasks(1, 3);
    close(&mut graph, 2, 1);
    for _ in 0..2 {
        assert_eq!(
            graph.delete_epic(epic_id(1)),
            Err(DomainError::OrphanNotAllowed {
                id: "camera:U1".to_owned()
            })
        );
    }
    assert_eq!(graph.tasks().count(), 3);
}

#[test]
fn graph_orphan_rule_allows_delete_once_tasks_have_moved() {
    let mut graph = graph_with_tasks(2, 1);
    // Move E1's task under E2 — "its tasks move first".
    graph.reparent_task(task_id(1), epic_id(2), at(1)).unwrap();
    assert_eq!(graph.delete_epic(epic_id(1)), Ok(Vec::new()));
    assert!(graph.epic(&epic_id(1)).is_none());
    assert_eq!(graph.tasks().count(), 2);
    for number in [1, 2] {
        assert_eq!(graph.task(&task_id(number)).unwrap().epic, epic_id(2));
    }
    // The epic number stays burned: gaps are meaningful (§3).
    assert_eq!(graph.counters().epic(), 2);
    let epic = graph.add_epic(epic_draft("the next epic")).unwrap();
    assert_eq!(epic.id.number(), 3);
    assert_invariants(&graph);
}

#[test]
fn graph_orphan_rule_cascades_terminal_tasks_and_their_edges() {
    // E1 holds U1 and U2; E2 holds U3, the survivor the edges reach into.
    let mut graph = graph_with_tasks(1, 2);
    graph.add_epic(epic_draft("the other epic")).unwrap();
    let third = graph.add_task(task_draft(&epic_id(2)), at(1)).unwrap();
    assert_eq!(third.id.number(), 3);

    // Both terminals qualify: `closed` with evidence, `wontfix` without.
    close(&mut graph, 1, 2);
    graph
        .update_task(task_id(2), at(3), |task| {
            task.status = Status::Wontfix;
        })
        .unwrap();
    graph
        .add_decision(decision_draft("which encoder"), at(4))
        .unwrap();

    // Edges that the cascade must take with it: task edges in both
    // directions, a `relates` annotation, and decision gates on the tasks
    // and on the epic itself.
    for edge in [
        task_edge(1, 3),
        task_edge(3, 2),
        gate_task(1, 1),
        gate_task(1, 2),
        gate_epic(1, 1),
    ] {
        graph.link_blocks(edge).unwrap();
    }
    graph.link_relates(task_id(1), task_id(3)).unwrap();

    assert_eq!(
        graph.delete_epic(epic_id(1)),
        Ok(vec![task_id(1), task_id(2)])
    );
    // The nodes are gone, and so is every edge that touched them.
    assert!(graph.epic(&epic_id(1)).is_none());
    assert!(graph.task(&task_id(1)).is_none());
    assert!(graph.task(&task_id(2)).is_none());
    assert!(graph.blocks_edges().is_empty());
    assert!(graph.relates_pairs().is_empty());
    assert!(graph.blocks(&BlockEnd::Decision(decision_id(1))).is_empty());
    assert!(graph.blocked_by(&BlockEnd::Task(task_id(3))).is_empty());
    // The survivor and the gating decision are untouched.
    assert_eq!(graph.task(&task_id(3)).unwrap().epic, epic_id(2));
    assert_eq!(
        graph.decision(&decision_id(1)).unwrap().status,
        DecisionStatus::Open
    );
    // The cascade burned no numbers.
    assert_eq!(graph.counters().task(), 3);
    assert_eq!(graph.counters().epic(), 2);
    assert_invariants(&graph);
}

#[test]
fn graph_reparent_moves_a_task_and_keeps_its_history_and_edges() {
    let mut graph = graph_with_tasks(2, 1);
    graph.add_decision(decision_draft("gate"), at(1)).unwrap();
    graph.link_blocks(gate_task(1, 1)).unwrap();
    graph.link_relates(task_id(1), task_id(2)).unwrap();

    let moved = graph.reparent_task(task_id(1), epic_id(2), at(9)).unwrap();
    assert_eq!(moved.epic, epic_id(2));
    assert_eq!(moved.id, task_id(1));
    assert_eq!(moved.updated, at(9));
    assert_eq!(moved.created, at(0));
    // Edges ride along: reparenting moves the task, it does not rewrite
    // its dependencies.
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task(task_id(1))),
        vec![BlockEnd::Decision(decision_id(1))]
    );
    assert_eq!(graph.relates_of(&task_id(1)), vec![task_id(2)]);
    // E1 is now empty, so it deletes cleanly.
    assert_eq!(graph.delete_epic(epic_id(1)), Ok(Vec::new()));
    assert_invariants(&graph);
}

#[test]
fn graph_reparent_to_an_unknown_epic_is_unknown_id() {
    let mut graph = graph_with_tasks(1, 1);
    let before = graph.clone();
    assert_eq!(
        graph.reparent_task(task_id(1), epic_id(9), at(1)),
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        })
    );
    assert_eq!(graph, before);
    assert_eq!(graph.task(&task_id(1)).unwrap().epic, epic_id(1));
    // A patch that reparents through `update_task` is checked the same way.
    assert_eq!(
        graph.update_task(task_id(1), at(2), |task| task.epic = epic_id(9)),
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        })
    );
    assert_eq!(graph, before);
    assert_invariants(&graph);
}

#[test]
fn graph_explicit_insert_validates_the_tree() {
    let mut graph = graph_with_epics(1);
    // A task whose epic field names no epic of this graph cannot enter it,
    // whatever its origin — this is the replay path.
    let orphan = task_node(7, 9, Status::Open);
    assert_eq!(
        graph.insert_task(orphan.clone()),
        Err(DomainError::UnknownId {
            id: "camera:E9".to_owned()
        })
    );
    assert!(graph.task(&task_id(7)).is_none());
    // The rejected insert raised no counter: it landed nothing.
    assert_eq!(graph.counters().task(), 0);

    // With the epic present, the same node inserts.
    graph.insert_epic(epic_node(9)).unwrap();
    assert_eq!(graph.counters().epic(), 9);
    graph.insert_task(orphan.clone()).unwrap();
    assert_eq!(graph.task(&task_id(7)), Some(&orphan));
    assert_eq!(graph.counters().task(), 7);

    // The closure discipline applies on the replay path too.
    let mut invalid = task_node(8, 9, Status::Closed);
    invalid.evidence = None;
    assert_eq!(
        graph.insert_task(invalid),
        Err(DomainError::MissingEvidence {
            id: "camera:U8".to_owned()
        })
    );
    assert_eq!(graph.counters().task(), 7);
    assert_invariants(&graph);
}

#[test]
fn graph_insert_rejects_nodes_of_another_repository() {
    let mut graph = graph_with_epics(1);
    // A node keyed by another repo's id has no parent here (§5.1's tree is
    // rooted at one repo), which is what `orphan_not_allowed` says.
    let foreign = Task {
        id: other_task_id(1),
        epic: epic_id(1),
        ..task_node(1, 1, Status::Open)
    };
    assert_eq!(
        graph.insert_task(foreign),
        Err(DomainError::OrphanNotAllowed {
            id: "dacode:U1".to_owned()
        })
    );
    // So does a node whose id is in scope but whose `repo` field is not.
    let inconsistent = Task {
        repo: other_slug(),
        ..task_node(2, 1, Status::Open)
    };
    assert_eq!(
        graph.insert_task(inconsistent),
        Err(DomainError::OrphanNotAllowed {
            id: "camera:U2".to_owned()
        })
    );
    let foreign_epic = Epic {
        repo: other_slug(),
        ..epic_node(2)
    };
    assert_eq!(
        graph.insert_epic(foreign_epic),
        Err(DomainError::OrphanNotAllowed {
            id: "camera:E2".to_owned()
        })
    );
    let foreign_decision = Decision {
        repo: other_slug(),
        ..decision_node(1, DecisionStatus::Open)
    };
    assert_eq!(
        graph.insert_decision(foreign_decision),
        Err(DomainError::OrphanNotAllowed {
            id: "camera:D1".to_owned()
        })
    );
    let foreign_doc = Doc {
        repo: other_slug(),
        ..doc_node(1)
    };
    assert_eq!(
        graph.insert_doc(foreign_doc),
        Err(DomainError::OrphanNotAllowed {
            id: "camera:S1".to_owned()
        })
    );
    assert_eq!(graph.tasks().count(), 0);
    assert_eq!(graph.epics().count(), 1);
    // A rejected insert lands nothing, not even a counter move: the one
    // epic the graph was built with is all it has issued.
    let counters = graph.counters();
    assert_eq!(counters.epic(), 1);
    assert_eq!(counters.task(), 0);
    assert_eq!(counters.decision(), 0);
    assert_eq!(counters.doc(), 0);
    assert_invariants(&graph);
}

#[test]
fn graph_delete_of_an_unknown_epic_is_unknown_id() {
    let mut graph = graph_with_epics(1);
    assert_eq!(
        graph.delete_epic(epic_id(2)),
        Err(DomainError::UnknownId {
            id: "camera:E2".to_owned()
        })
    );
    assert_eq!(graph, graph_with_epics(1));
    assert_invariants(&graph);
}

#[test]
fn graph_an_epic_holding_only_gates_deletes_and_takes_them() {
    // An epic with no tasks of its own, gated by a decision: deleting it
    // removes the gate edge and leaves the decision node alone (§5.1/D10 —
    // the history stays).
    let mut graph = graph_with_epics(1);
    graph.add_decision(decision_draft("gating"), at(1)).unwrap();
    graph.link_blocks(gate_epic(1, 1)).unwrap();
    assert_eq!(
        graph.blocks(&BlockEnd::Decision(decision_id(1))),
        vec![BlockEnd::Epic(epic_id(1))]
    );

    assert_eq!(graph.delete_epic(epic_id(1)), Ok(Vec::new()));
    assert!(graph.blocks_edges().is_empty());
    assert!(graph.decision(&decision_id(1)).is_some());
    assert_invariants(&graph);
}
