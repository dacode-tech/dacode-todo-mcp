//! Replay, the golden, determinism and the write contract: what a log
//! means once it is on disk (spec §4.1, §8, §10).

use std::collections::BTreeMap;
use std::fs;

use tempfile::TempDir;
use todo_domain::{BlocksEdge, DecisionStatus, DomainError, Graph, Status, TaskDraft};
use todo_store::{
    DataRoot, DocPatch, Event, FORMAT_VERSION, JsonlStore, Store, StoreError, TaskPatch,
    from_bytes, to_bytes,
};

use super::fixtures::{
    EVERY_KIND, at, build_timeline, camera_slug, decision_id, doc_id, epic_id, lines,
    lines_of_kind, log_path, repo, short_store, task_draft, task_id, timeline_bytes,
    timeline_first_half, timeline_rewards_phase, timeline_root, timeline_second_half,
};
use crate::common::{assert_golden, golden_path};

// =================================================================
// Replay.
// =================================================================

#[test]
fn replay_of_the_scripted_timeline_is_graph_equal() {
    let dir = TempDir::new().unwrap();
    let root = timeline_root(&dir);
    let path = root.log_path(&camera_slug());
    let store = build_timeline(&root);
    let original = store.graph().clone();
    drop(store);

    let reopened = JsonlStore::open_log(&path).unwrap();
    // `Graph` derives `PartialEq` including the counters, so this one
    // assertion is the whole replay contract (§4.1).
    assert_eq!(reopened.graph(), &original);
    assert_eq!(reopened.format_version(), FORMAT_VERSION);

    // The cascade's gap survives replay: U3 and U4 are gone, the task
    // counter still holds 4's successor, and the later adds are U5/U6
    // — U7 beyond them, from the rewards phase.
    let counters = reopened.graph().counters();
    assert_eq!(
        (
            counters.epic(),
            counters.task(),
            counters.decision(),
            counters.doc()
        ),
        (3, 7, 1, 1)
    );
    assert_eq!(reopened.graph().task(&task_id(3)), None);
    assert_eq!(reopened.graph().task(&task_id(4)), None);
    assert!(reopened.graph().task(&task_id(5)).is_some());
    assert!(reopened.graph().task(&task_id(6)).is_some());
    // The rewards phase's task, with its estimate — the only task the
    // timeline gave one.
    let u7 = reopened.graph().task(&task_id(7)).unwrap();
    assert_eq!(u7.estimate, Some(todo_domain::Size::Xl));
    assert_eq!(u7.status, Status::Closed);

    // The reparenting landed, the relates pair and the two surviving
    // `blocks` edges are there, and the decision gate outlived its
    // signing (§3/D10).
    let graph = reopened.graph();
    assert_eq!(graph.task(&task_id(5)).unwrap().epic, epic_id(1));
    assert_eq!(graph.relates_pairs(), vec![(task_id(1), task_id(2))]);
    assert_eq!(
        graph.blocks_edges(),
        vec![
            BlocksEdge::TaskToTask {
                from: task_id(1),
                to: task_id(2)
            },
            BlocksEdge::DecisionToTask {
                from: decision_id(1),
                to: task_id(2)
            },
        ]
    );
    // A repo node carries no audit timestamp (§3), so what replays is the
    // attribute itself and the event line is the only record of when it
    // changed. The doc's audited path move and patch, and the decision's
    // signing, replay as their last lines left them.
    assert_eq!(graph.repo().path, "/srv/checkouts/camera");
    assert_eq!(graph.repo().name, "Camera firmware (gen2)");
    assert_eq!(
        graph.doc(&doc_id(1)).unwrap().path,
        "docs/bringup/sensor.md"
    );
    assert_eq!(graph.doc(&doc_id(1)).unwrap().version, "1.5");
    assert_eq!(
        graph.decision(&decision_id(1)).unwrap().status,
        DecisionStatus::Signed
    );
    assert_eq!(graph.epic(&epic_id(2)), None);
}

#[test]
fn the_golden_timeline_exercises_every_event_kind() {
    let dir = TempDir::new().unwrap();
    let root = timeline_root(&dir);
    let bytes = timeline_bytes(&root);

    // The header is the versioned first line, exactly as written.
    let log_lines = lines(&bytes);
    assert_eq!(log_lines[0], b"{\"format_version\":2}\n");

    let mut kinds = BTreeMap::new();
    for line in &log_lines[1..] {
        let event: Event = from_bytes(line).unwrap();
        kinds.entry(event.kind()).or_insert(event);
    }
    assert_eq!(kinds.len(), EVERY_KIND.len());
    for kind in EVERY_KIND {
        assert!(kinds.contains_key(kind), "the timeline never logs {kind}");
    }
}

#[test]
fn every_event_kind_round_trips_through_the_codec() {
    let dir = TempDir::new().unwrap();
    let root = timeline_root(&dir);
    let bytes = timeline_bytes(&root);
    for line in &lines(&bytes)[1..] {
        let event: Event = from_bytes(line).unwrap();
        let serialized = to_bytes(&event).unwrap();
        // §8's byte-stability test, per event kind: serialize → load →
        // serialize is identical, and the tag on the wire is the
        // spelling `kind()` reports.
        let text = String::from_utf8(serialized.clone()).unwrap();
        assert!(
            text.starts_with(&format!("{{\"kind\":\"{}\"", event.kind())),
            "{text}"
        );
        assert_eq!(
            to_bytes(&from_bytes::<Event>(&serialized).unwrap()).unwrap(),
            serialized
        );
        assert_eq!(from_bytes::<Event>(&serialized).unwrap(), event);
        // The audit instant is on the wire in its canonical spelling,
        // and it is the line's only time source (§8).
        assert!(
            text.contains(&format!("\"ts\":\"{}\"", event.ts().as_str())),
            "{text}"
        );
    }
}

// =================================================================
// The golden.
// =================================================================

#[test]
fn the_scripted_timeline_matches_its_golden_log() {
    let dir = TempDir::new().unwrap();
    let root = timeline_root(&dir);
    // The golden name embeds FORMAT_VERSION, so bumping the format
    // without landing the new golden fails as "missing golden".
    assert_golden(
        &format!("event_log.v{FORMAT_VERSION}"),
        &timeline_bytes(&root),
    );
}

// =================================================================
// The v1 → v2 migration (§15: the strict superset).
// =================================================================

#[test]
fn a_v1_log_replays_identically_under_the_v2_reader() {
    // The frozen v1 golden — the pre-bump format's real timeline bytes,
    // kept in the tree as the migration fixture (the golden workflow's
    // coexistence rule). Under the v2 reader it replays to exactly the
    // graph the same timeline builds on an inert root: the drafts carry
    // no estimate, so the v1 lines' absent field and the v2 lines'
    // `null` agree.
    let v1 = fs::read(golden_path("event_log.v1")).unwrap();
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    fs::write(&path, &v1).unwrap();
    let store = JsonlStore::open_log(&path).unwrap();

    let inert_dir = TempDir::new().unwrap();
    let inert_root = DataRoot::new(inert_dir.path());
    let mut reference = inert_root.register(repo()).unwrap();
    timeline_first_half(&mut reference);
    timeline_second_half(&mut reference);
    assert_eq!(store.graph(), reference.graph());
    // The v1 lines carry no estimate field: every task reads as `None`.
    for task in store.graph().tasks() {
        assert_eq!(task.estimate, None);
    }

    // The header migrated in place: same length, the v2 first line, and
    // every byte after it untouched — the length-preserving rewrite
    // that keeps every snapshot `log_offset` valid.
    let migrated = fs::read(&path).unwrap();
    assert_eq!(migrated.len(), v1.len());
    assert_eq!(&migrated[..HEADER_LINE_LEN], b"{\"format_version\":2}\n");
    assert_eq!(&migrated[HEADER_LINE_LEN..], &v1[HEADER_LINE_LEN..]);
    // A second open is a plain v2 open: no migration, same graph.
    drop(store);
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), reference.graph());
}

#[test]
fn a_v1_log_migrates_only_through_the_write_capable_open() {
    // The lock-free read (D19) writes nothing: a v1 log read through
    // `read_repo` answers the graph and leaves the bytes exactly as
    // they were — the migration belongs to the locked open alone.
    let v1 = fs::read(golden_path("event_log.v1")).unwrap();
    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let path = root.log_path(&camera_slug());
    fs::create_dir_all(root.store_dir(&camera_slug())).unwrap();
    fs::write(&path, &v1).unwrap();

    let graph = root.read_repo(&camera_slug()).unwrap();
    assert_eq!(fs::read(&path).unwrap(), v1);
    // The read answered the v1 timeline's graph — U6 is its last task.
    assert!(graph.task(&task_id(6)).is_some());
    assert_eq!(graph.task(&task_id(7)), None);
}

/// The header line's byte length — `{"format_version":N}\n`, the same
/// for every single-digit version, which is what makes the in-place
/// migration length-preserving.
const HEADER_LINE_LEN: usize = 21;

// =================================================================
// Determinism (§8, §16 of the task's decision list).
// =================================================================

#[test]
fn the_same_timeline_twice_is_deterministic() {
    let first_dir = TempDir::new().unwrap();
    let first = timeline_bytes(&timeline_root(&first_dir));
    let second_dir = TempDir::new().unwrap();
    let second = timeline_bytes(&timeline_root(&second_dir));
    assert_eq!(first, second);
}

#[test]
fn appends_after_a_reopen_are_byte_stable() {
    let one_dir = TempDir::new().unwrap();
    let one_session = timeline_bytes(&timeline_root(&one_dir));

    // The same timeline with a session boundary in the middle: the
    // replay must leave the graph exactly where the writer had it, and
    // the appends that follow must be the same bytes.
    let dir = TempDir::new().unwrap();
    let root = timeline_root(&dir);
    let path = root.log_path(&camera_slug());
    let mut store = root.register(repo()).unwrap();
    timeline_first_half(&mut store);
    let mid_graph = store.graph().clone();
    drop(store);

    let mut reopened = root.open(&camera_slug()).unwrap();
    assert_eq!(reopened.graph(), &mid_graph);
    timeline_second_half(&mut reopened);
    timeline_rewards_phase(&mut reopened);
    drop(reopened);

    assert_eq!(fs::read(&path).unwrap(), one_session);
}

// =================================================================
// create/open discipline.
// =================================================================

#[test]
fn a_created_log_is_its_header_and_its_seed() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    drop(JsonlStore::create_log(&path, repo()).unwrap());

    let seed = repo();
    let mut expected = Vec::new();
    expected.extend_from_slice(br#"{"format_version":2}"#);
    expected.push(b'\n');
    expected.extend_from_slice(
        &to_bytes(&Event::RepoRegister {
            ts: seed.registered_at.clone(),
            repo: seed,
        })
        .unwrap(),
    );
    expected.push(b'\n');
    assert_eq!(fs::read(&path).unwrap(), expected);
}

#[test]
fn create_refuses_an_existing_log_and_open_refuses_a_missing_one() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let bytes_before = {
        drop(JsonlStore::create_log(&path, repo()).unwrap());
        fs::read(&path).unwrap()
    };

    // Creation is `create_new`: an existing log is an io error, never
    // a truncation.
    let error = JsonlStore::create_log(&path, repo()).unwrap_err();
    assert!(matches!(error, StoreError::Io { .. }), "{error}");
    assert_eq!(error.code(), "io");
    assert!(error.to_string().contains("AlreadyExists"), "{error}");
    assert_eq!(fs::read(&path).unwrap(), bytes_before);
    assert!(JsonlStore::open_log(&path).is_ok());

    let error = JsonlStore::open_log(dir.path().join("no-such-log.jsonl")).unwrap_err();
    assert!(matches!(error, StoreError::Io { .. }), "{error}");
    assert!(error.to_string().contains("NotFound"), "{error}");
    // No host path in the detail — the context and the kind are all a
    // caller gets.
    let directory = dir.path().to_string_lossy().into_owned();
    assert!(!error.to_string().contains(&directory), "{error}");
}

// =================================================================
// The write contract: atomic rejection, and the patch DTO's limits.
// =================================================================

#[test]
fn a_rejected_write_leaves_the_graph_and_the_log_untouched() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = short_store(&path);
    let bytes_before = fs::read(&path).unwrap();
    let graph_before: Graph = store.graph().clone();

    // No verify method: `missing_verify`, and the number is not burned.
    let unverified = TaskDraft {
        verify: None,
        ..task_draft(&epic_id(1), "unverified")
    };
    let error = store
        .add_task(unverified, at("2026-09-25T11:00:00Z"))
        .unwrap_err();
    assert_eq!(
        error,
        StoreError::Domain(DomainError::MissingVerify {
            id: "camera:U3".to_owned()
        })
    );
    assert_eq!(error.code(), "missing_verify");

    // A link that would close a cycle.
    let error = store
        .link_blocks(
            BlocksEdge::TaskToTask {
                from: task_id(2),
                to: task_id(1),
            },
            at("2026-09-25T11:05:00Z"),
        )
        .unwrap_err();
    assert_eq!(error.code(), "cycle");

    // An epic whose tasks are not terminal.
    let error = store
        .delete_epic(epic_id(1), at("2026-09-25T11:10:00Z"))
        .unwrap_err();
    assert_eq!(error.code(), "orphan_not_allowed");

    // An unknown id.
    let error = store
        .update_task(task_id(9), at("2026-09-25T11:15:00Z"), TaskPatch::default())
        .unwrap_err();
    assert_eq!(
        error,
        StoreError::Domain(DomainError::UnknownId {
            id: "camera:U9".to_owned()
        })
    );

    // Nothing was applied, nothing was appended, and a rejection is not
    // session-fatal — only an io failure is (§4.3's window).
    assert_eq!(fs::read(&path).unwrap(), bytes_before);
    assert_eq!(store.graph(), &graph_before);
    let task = store
        .add_task(
            task_draft(&epic_id(1), "verified"),
            at("2026-09-25T11:20:00Z"),
        )
        .unwrap();
    assert_eq!(task.id, task_id(3));
    assert_eq!(store.graph().counters().task(), 3);
}

#[test]
fn unlinking_an_absent_edge_mirrors_the_domain() {
    // `Graph::unlink_blocks` and `Graph::unlink_relates` require their
    // endpoints and otherwise treat an absent edge as a no-op `Ok`. The
    // store passes that answer through and still appends the line: the
    // log records what was asked for, and replay ends where the writer
    // did because both sides agree the request changed nothing.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = short_store(&path);
    let edges_before = store.graph().blocks_edges();
    assert_eq!(edges_before.len(), 1);

    // The reverse of the linked edge was never linked.
    store
        .unlink_blocks(
            BlocksEdge::TaskToTask {
                from: task_id(2),
                to: task_id(1),
            },
            at("2026-09-25T11:00:00Z"),
        )
        .unwrap();
    assert_eq!(store.graph().blocks_edges(), edges_before);

    // A relates pair, removed, then removed again from the other
    // spelling: both are `Ok`, the second changes nothing.
    store
        .link_relates(task_id(1), task_id(2), at("2026-09-25T11:05:00Z"))
        .unwrap();
    store
        .unlink_relates(task_id(1), task_id(2), at("2026-09-25T11:06:00Z"))
        .unwrap();
    store
        .unlink_relates(task_id(2), task_id(1), at("2026-09-25T11:07:00Z"))
        .unwrap();
    assert!(store.graph().relates_pairs().is_empty());

    // An endpoint the graph does not hold is still the domain's
    // `unknown_id`.
    let error = store
        .unlink_relates(task_id(1), task_id(9), at("2026-09-25T11:10:00Z"))
        .unwrap_err();
    assert_eq!(error.code(), "unknown_id");

    // The no-op lines are in the log, and the rejected one is not.
    let bytes = fs::read(&path).unwrap();
    assert_eq!(lines_of_kind(&lines(&bytes), "blocks_unlink").len(), 1);
    assert_eq!(lines_of_kind(&lines(&bytes), "relates_unlink").len(), 2);
    drop(store);
    assert_eq!(
        JsonlStore::open_log(&path).unwrap().graph().blocks_edges(),
        edges_before
    );
}

#[test]
fn a_patch_keeps_untouched_cleared_and_replaced_apart() {
    // RFC 7386's three states on the wire, pinned because serde's derived
    // `Option<Option<T>>` collapses the first two: an absent field is
    // untouched, `null` clears, a value replaces. Losing the distinction
    // is a replay that ends somewhere other than the session did.
    let untouched = TaskPatch::default();
    assert_eq!(to_bytes(&untouched).unwrap(), b"{}");
    assert_eq!(from_bytes::<TaskPatch>(b"{}").unwrap(), untouched);

    let clearing = TaskPatch {
        note: Some(None),
        ..TaskPatch::default()
    };
    assert_eq!(to_bytes(&clearing).unwrap(), br#"{"note":null}"#);
    assert_eq!(
        from_bytes::<TaskPatch>(br#"{"note":null}"#).unwrap(),
        clearing
    );
    assert_ne!(clearing, untouched);

    let replacing = TaskPatch {
        note: Some(Some("waiting on D12".to_owned())),
        ..TaskPatch::default()
    };
    let bytes = to_bytes(&replacing).unwrap();
    assert_eq!(bytes, br#"{"note":"waiting on D12"}"#);
    // §8's byte-stability test over the patch DTO.
    assert_eq!(from_bytes::<TaskPatch>(&bytes).unwrap(), replacing);
    assert_eq!(
        to_bytes(&from_bytes::<TaskPatch>(&bytes).unwrap()).unwrap(),
        bytes
    );

    // Through the store: a no-op patch moves only `updated`, and a clearing
    // patch clears — both after a reopen, so the line and the session
    // agree.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = short_store(&path);
    let before = store.graph().task(&task_id(1)).unwrap().clone();
    let after = store
        .update_task(task_id(1), at("2026-09-25T12:00:00Z"), TaskPatch::default())
        .unwrap();
    assert_eq!(after.created, before.created);
    assert_eq!(after.updated, at("2026-09-25T12:00:00Z"));
    assert_eq!(after.title, before.title);
    assert_eq!(after.status, before.status);
    assert_eq!(after.references, before.references);

    store
        .update_task(
            task_id(2),
            at("2026-09-25T12:10:00Z"),
            TaskPatch {
                status: Some(Status::Blocked),
                note: Some(Some("waiting on the sensor".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    let cleared = store
        .update_task(
            task_id(2),
            at("2026-09-25T12:20:00Z"),
            TaskPatch {
                status: Some(Status::InWork),
                note: Some(None),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    assert_eq!(cleared.note, None);
    drop(store);

    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(
        reopened.graph().task(&task_id(1)),
        Some(&after),
        "a no-op patch must survive replay"
    );
    assert_eq!(reopened.graph().task(&task_id(2)), Some(&cleared));
    assert_eq!(reopened.graph().task(&task_id(2)).unwrap().note, None);
}

#[test]
fn a_patch_cannot_rewrite_node_history() {
    // `created` and `updated` are not patch fields — `updated` is the
    // event's `ts` — and `deny_unknown_fields` refuses a line that
    // smuggles one in, along with the identity fields. The domain's raw
    // closure API can still rewrite them; the persisted surface, which is
    // what a client reaches, cannot.
    for smuggled in [
        r#"{"title":"x","created":"2026-09-25T08:00:00.000000000Z"}"#,
        r#"{"updated":"2026-09-25T08:00:00.000000000Z"}"#,
        r#"{"id":"camera:U1"}"#,
        r#"{"repo":"camera"}"#,
        r#"{"path":"docs/x.md"}"#,
    ] {
        assert!(
            from_bytes::<TaskPatch>(smuggled.as_bytes()).is_err(),
            "{smuggled} must be rejected"
        );
    }
    // A doc patch has no `path` either: that moves through its own
    // audited verb (§6, D10).
    assert!(from_bytes::<DocPatch>(br#"{"path":"docs/x.md"}"#).is_err());
    assert!(from_bytes::<TaskPatch>(br#"{"title":"x"}"#).is_ok());
}
