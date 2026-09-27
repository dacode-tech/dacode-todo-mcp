//! §3's identity rule: per-repo sequential integers per kind, allocated
//! from 1, never reused, never renumbered — gaps are meaningful.

use super::*;

#[test]
fn graph_allocation_is_sequential_from_one_per_kind() {
    let mut graph = empty_graph();
    for step in 0..2 {
        let epic = graph.add_epic(epic_draft("an epic")).unwrap();
        let task = graph.add_task(task_draft(&epic.id), at(step)).unwrap();
        let decision = graph
            .add_decision(decision_draft("a decision"), at(step))
            .unwrap();
        let doc = graph.add_doc(doc_draft("a doc"), at(step)).unwrap();
        // Interleaving the kinds does not disturb any of them: each kind
        // counts its own numbers.
        assert_eq!(epic.id.number(), step + 1);
        assert_eq!(task.id.number(), step + 1);
        assert_eq!(decision.id.number(), step + 1);
        assert_eq!(doc.id.number(), step + 1);
    }
    let counters = graph.counters();
    assert_eq!(counters.epic(), 2);
    assert_eq!(counters.task(), 2);
    assert_eq!(counters.decision(), 2);
    assert_eq!(counters.doc(), 2);
    assert_eq!(graph.epics().count(), 2);
    assert_invariants(&graph);
}

#[test]
fn graph_allocation_advances_past_deleted_numbers() {
    // Delete an epic and its closed task, then allocate again: the next
    // numbers continue past the gap rather than filling it.
    let mut graph = graph_with_tasks(1, 1);
    graph
        .update_task(task_id(1), at(1), |task| {
            task.status = Status::Wontfix;
        })
        .unwrap();
    assert_eq!(graph.delete_epic(epic_id(1)), Ok(vec![task_id(1)]));

    let epic = graph.add_epic(epic_draft("after the delete")).unwrap();
    let task = graph.add_task(task_draft(&epic.id), at(2)).unwrap();
    assert_eq!(epic.id.to_string(), "camera:E2");
    assert_eq!(task.id.to_string(), "camera:U2");
    assert_eq!(graph.counters().epic(), 2);
    assert_eq!(graph.counters().task(), 2);
    assert_invariants(&graph);
}

#[test]
fn graph_explicit_insert_raises_the_counter_to_max_and_never_lowers_it() {
    let mut graph = empty_graph();
    graph.insert_epic(epic_node(1)).unwrap();
    assert_eq!(graph.counters().epic(), 1);

    // A replayed node above the counter raises it, so the next allocation
    // cannot collide with it.
    graph.insert_task(task_node(9, 1, Status::Open)).unwrap();
    assert_eq!(graph.counters().task(), 9);
    graph.insert_epic(epic_node(4)).unwrap();
    assert_eq!(graph.counters().epic(), 4);
    graph
        .insert_decision(decision_node(3, DecisionStatus::Open))
        .unwrap();
    assert_eq!(graph.counters().decision(), 3);
    graph.insert_doc(doc_node(7)).unwrap();
    assert_eq!(graph.counters().doc(), 7);

    // A replayed node below the counter leaves it alone — lowering it
    // would re-issue a live number.
    graph.insert_task(task_node(2, 1, Status::Open)).unwrap();
    graph.insert_epic(epic_node(2)).unwrap();
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.insert_doc(doc_node(1)).unwrap();
    let counters = graph.counters();
    assert_eq!(counters.task(), 9);
    assert_eq!(counters.epic(), 4);
    assert_eq!(counters.decision(), 3);
    assert_eq!(counters.doc(), 7);

    // Allocation continues above every replayed number.
    let epic = graph.add_epic(epic_draft("allocated")).unwrap();
    let task = graph.add_task(task_draft(&epic.id), at(1)).unwrap();
    let decision = graph
        .add_decision(decision_draft("allocated"), at(1))
        .unwrap();
    let doc = graph.add_doc(doc_draft("allocated"), at(1)).unwrap();
    assert_eq!(epic.id.number(), 5);
    assert_eq!(task.id.number(), 10);
    assert_eq!(decision.id.number(), 4);
    assert_eq!(doc.id.number(), 8);
    assert_invariants(&graph);
}

#[test]
fn graph_duplicate_ids_are_rejected_for_every_kind() {
    let mut graph = empty_graph();
    graph.insert_epic(epic_node(1)).unwrap();
    graph.insert_task(task_node(1, 1, Status::Open)).unwrap();
    graph
        .insert_decision(decision_node(1, DecisionStatus::Open))
        .unwrap();
    graph.insert_doc(doc_node(1)).unwrap();
    let before = graph.clone();

    assert_eq!(
        graph.insert_epic(epic_node(1)),
        Err(DomainError::Duplicate {
            id: "camera:E1".to_owned()
        })
    );
    assert_eq!(
        graph.insert_task(task_node(1, 1, Status::Open)),
        Err(DomainError::Duplicate {
            id: "camera:U1".to_owned()
        })
    );
    assert_eq!(
        graph.insert_decision(decision_node(1, DecisionStatus::Open)),
        Err(DomainError::Duplicate {
            id: "camera:D1".to_owned()
        })
    );
    assert_eq!(
        graph.insert_doc(doc_node(1)),
        Err(DomainError::Duplicate {
            id: "camera:S1".to_owned()
        })
    );
    assert_eq!(graph, before);
    assert_invariants(&graph);
}

#[test]
fn graph_unknown_ids_are_rejected_by_every_mutation() {
    let mut graph = graph_with_tasks(1, 1);
    graph.add_decision(decision_draft("a gate"), at(0)).unwrap();
    let before = graph.clone();
    let missing_task = task_id(9);
    let missing_epic = epic_id(9);
    let missing_decision = decision_id(9);
    let missing_doc = doc_id(9);

    let unknown = |id: &str| DomainError::UnknownId { id: id.to_owned() };
    assert_eq!(
        graph.update_epic(missing_epic.clone(), |_| {}),
        Err(unknown("camera:E9"))
    );
    assert_eq!(
        graph.update_task(missing_task.clone(), at(1), |_| {}),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.update_decision(missing_decision, at(1), |_| {}),
        Err(unknown("camera:D9"))
    );
    assert_eq!(
        graph.update_doc(missing_doc, at(1), |_| {}),
        Err(unknown("camera:S9"))
    );
    assert_eq!(
        graph.reparent_task(missing_task.clone(), epic_id(1), at(1)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.reparent_task(task_id(1), missing_epic.clone(), at(1)),
        Err(unknown("camera:E9"))
    );

    // Both endpoints are checked, the source first: an edge from a missing
    // node reports the source even when the target is missing too.
    assert_eq!(
        graph.link_blocks(task_edge(9, 8)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.link_blocks(task_edge(1, 9)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.link_blocks(gate_task(9, 1)),
        Err(unknown("camera:D9"))
    );
    assert_eq!(
        graph.link_blocks(gate_task(1, 9)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.link_blocks(gate_epic(9, 1)),
        Err(unknown("camera:D9"))
    );
    assert_eq!(
        graph.link_blocks(gate_epic(1, 9)),
        Err(unknown("camera:E9"))
    );
    assert_eq!(
        graph.unlink_blocks(gate_epic(1, 9)),
        Err(unknown("camera:E9"))
    );
    assert_eq!(
        graph.unlink_blocks(task_edge(1, 9)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.link_relates(missing_task.clone(), task_id(1)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.link_relates(task_id(1), missing_task.clone()),
        Err(unknown("camera:U9"))
    );
    assert_eq!(
        graph.unlink_relates(missing_task, task_id(1)),
        Err(unknown("camera:U9"))
    );
    assert_eq!(graph.delete_epic(missing_epic), Err(unknown("camera:E9")));

    // Not one of them landed anything.
    assert_eq!(graph, before);
    assert_invariants(&graph);
}

#[test]
fn graph_a_rejected_add_burns_no_number() {
    let mut graph = graph_with_epics(1);
    // Missing verify, then a missing parent, then a bad closure state:
    // three rejections, and the next allocation is still U1.
    let mut draft = task_draft(&epic_id(1));
    draft.verify = None;
    assert!(graph.add_task(draft, at(1)).is_err());
    let draft = task_draft(&epic_id(9));
    assert!(graph.add_task(draft, at(1)).is_err());
    let mut draft = task_draft(&epic_id(1));
    draft.status = Status::Deferred;
    assert!(graph.add_task(draft, at(1)).is_err());
    assert_eq!(graph.counters().task(), 0);

    let task = graph.add_task(task_draft(&epic_id(1)), at(2)).unwrap();
    assert_eq!(task.id.to_string(), "camera:U1");

    // The same holds for the other kinds.
    let mut decision = decision_draft("a decision");
    decision.status = DecisionStatus::Signed;
    assert!(graph.add_decision(decision, at(3)).is_err());
    assert_eq!(graph.counters().decision(), 0);
    let decision = graph
        .add_decision(decision_draft("a decision"), at(3))
        .unwrap();
    assert_eq!(decision.id.to_string(), "camera:D1");
    assert_invariants(&graph);
}
