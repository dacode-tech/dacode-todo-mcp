//! Replay with a snapshot: §4.3's recovery unit under open, the graph
//! equality that makes it sound, f5's burned-number restore end to end,
//! the golden, and §8's determinism instruments.

use std::fs;

use tempfile::TempDir;
use todo_domain::{BlocksEdge, Graph};
use todo_store::{DataRoot, FORMAT_VERSION, JsonlStore, Store, TaskPatch, from_bytes, to_bytes};

use super::fixtures::{
    SnapshotMirror, build_burned_number_store, build_golden_store, dir_entries, snapshot_bytes,
    snapshot_path,
};
use super::timeline::{
    at, doc_id, epic_draft, epic_id, log_path, repo, task_draft, task_id, timeline_first_half,
    timeline_second_half,
};
use crate::common::assert_golden;

// =================================================================
// Replay equality (required test 1).
// =================================================================

#[test]
fn replay_with_a_snapshot_is_graph_equal() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = JsonlStore::create_log(&path, repo()).unwrap();

    // Snapshot at two points, appending after each: the second snapshot
    // supersedes the first (§4.3's "the latest snapshot"), and the
    // append past it is the tail replay must apply.
    timeline_first_half(&mut store);
    let first_offset = store.snapshot().unwrap();
    // The returned offset is the log's byte length at the call — the
    // offset at which events the snapshot does not cover begin.
    assert_eq!(first_offset, fs::read(&path).unwrap().len() as u64);

    timeline_second_half(&mut store);
    let second_offset = store.snapshot().unwrap();
    assert!(second_offset > first_offset);
    assert_eq!(second_offset, fs::read(&path).unwrap().len() as u64);

    let extra = store
        .add_task(
            task_draft(&epic_id(3), "one beyond the snapshot"),
            at("2026-09-25T17:00:00Z"),
        )
        .unwrap();
    assert_eq!(extra.id, task_id(7));
    let written: Graph = store.graph().clone();
    drop(store);
    assert!((fs::read(&path).unwrap().len() as u64) > second_offset);

    // Reopen: restore the snapshot, apply only the tail. `Graph` derives
    // `PartialEq` including the counters, so one assertion is the whole
    // contract — burned numbers included.
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &written);

    // The same graph from a whole-log replay with no snapshot at all:
    // pre-compaction the log is still independently replayable, and the
    // snapshot path must end exactly where it does.
    drop(reopened);
    let aside = dir.path().join("snapshot.aside");
    fs::rename(snapshot_path(&path), &aside).unwrap();
    let whole_log = JsonlStore::open_log(&path).unwrap();
    assert_eq!(whole_log.graph(), &written);
    drop(whole_log);
    fs::rename(&aside, snapshot_path(&path)).unwrap();
    assert!(JsonlStore::open_log(&path).is_ok());
}

#[test]
fn the_covered_log_region_is_the_snapshots_record() {
    // Decision 5's "neither re-applied nor re-validated", pinned: a
    // length-preserving splice inside the covered region (so every offset
    // stays put) changes what a whole-log replay would see, and changes
    // nothing for a snapshot replay — the snapshot *is* the covered
    // region's record.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = JsonlStore::create_log(&path, repo()).unwrap();
    timeline_first_half(&mut store);
    let offset = store.snapshot().unwrap();
    timeline_second_half(&mut store);
    let written: Graph = store.graph().clone();
    drop(store);

    // The doc's audited path move sits in the covered region (first
    // half); the mutation keeps the byte length, so `offset` and every
    // line boundary survive it.
    let log = fs::read(&path).unwrap();
    let text = String::from_utf8(log.clone()).unwrap();
    let spliced = text.replacen("docs/bringup/sensor.md", "docs/bringup/senser.md", 1);
    assert_ne!(spliced, text, "the needle must exist exactly once");
    assert_eq!(spliced.len(), text.len());
    let splice_at = text.find("docs/bringup/sensor.md").unwrap() as u64;
    assert!(
        splice_at < offset,
        "the splice must sit in the covered region"
    );
    fs::write(&path, spliced.as_bytes()).unwrap();

    // With the snapshot: the covered mutation is invisible.
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &written);
    assert_eq!(
        reopened.graph().doc(&doc_id(1)).unwrap().path,
        "docs/bringup/sensor.md"
    );
    drop(reopened);

    // Without it: the whole-log replay reads the spliced bytes, proving
    // the splice landed and the snapshot path really skipped it.
    fs::rename(snapshot_path(&path), dir.path().join("snapshot.aside")).unwrap();
    let whole_log = JsonlStore::open_log(&path).unwrap();
    assert_eq!(
        whole_log.graph().doc(&doc_id(1)).unwrap().path,
        "docs/bringup/senser.md"
    );
}

// =================================================================
// f5's case, end to end (required test 2).
// =================================================================

#[test]
fn burned_numbers_survive_a_snapshot_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_burned_number_store(&path);
    // The cascade burned U2/U3 (and E2): the task counter holds 3 while
    // the highest live task is U1 — the state no node-derived restore
    // can recover (f5).
    assert_eq!(store.graph().counters().task(), 3);
    assert_eq!(store.graph().task(&task_id(2)), None);
    store.snapshot().unwrap();
    let written: Graph = store.graph().clone();
    drop(store);

    let mut reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &written);
    assert_eq!(reopened.graph().counters().task(), 3);

    // §3's never-reuse rule: the next allocation continues above the
    // burned numbers; U2 is never re-issued.
    let next = reopened
        .add_task(
            task_draft(&epic_id(1), "continue above the gap"),
            at("2026-09-25T11:00:00Z"),
        )
        .unwrap();
    assert_eq!(next.id, task_id(4));
    assert_eq!(reopened.graph().counters().task(), 4);
}

// =================================================================
// The golden (required test 3, Decision 12).
// =================================================================

#[test]
fn the_golden_fixture_snapshot_matches_its_golden() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    let offset = store.snapshot().unwrap();
    drop(store);

    let bytes = snapshot_bytes(&path);
    // The golden name embeds FORMAT_VERSION, so bumping the format
    // without landing the new golden fails as "missing golden".
    assert_golden(&format!("snapshot.v{FORMAT_VERSION}"), &bytes);

    // `log_offset` independently asserted, twice over: the returned
    // offset is the log's length, and the document carries that value.
    assert_eq!(offset, fs::read(&path).unwrap().len() as u64);
    let document = SnapshotMirror::read(&path);
    assert_eq!(document.log_offset, offset);
    assert_eq!(document.format_version, FORMAT_VERSION);
    // The four distinct counters, named per kind — a transposition of
    // `from_highest`'s positional arguments cannot pass.
    assert_eq!(
        (
            document.counters.epic,
            document.counters.task,
            document.counters.decision,
            document.counters.doc
        ),
        (3, 4, 1, 2)
    );
    // The burned gap is in the document as an absent node: U3/U4 and E3
    // are gone while the counters hold their numbers.
    assert!(document.tasks.iter().all(|task| task.id.number() <= 2));
    assert!(document.epics.iter().all(|epic| epic.id.number() <= 2));
    // The repo is the current node, not the registration (Decision 9).
    assert_eq!(document.repo.path, "/srv/checkouts/camera");
    assert_eq!(document.repo.name, "Camera firmware (gen2)");
}

// =================================================================
// The v1 pair under the v2 reader (§15's strict superset).
// =================================================================

#[test]
fn a_v1_snapshot_and_log_pair_loads_under_the_v2_reader() {
    // A genuine v1 pair, built by string surgery on real v2 bytes: the
    // version numbers spelled down to 1 and the `estimate` fields
    // stripped from both files — exactly the pre-bump format's shape.
    // The snapshot's `log_offset` is re-based to the stripped log's new
    // length, so the pair stays consistent the way a real one was.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    store.snapshot().unwrap();
    let written: Graph = store.graph().clone();
    drop(store);

    let log_v2 = fs::read(&path).unwrap();
    let snapshot_v2 = snapshot_bytes(&path);
    let to_v1 = |bytes: &[u8]| -> Vec<u8> {
        String::from_utf8_lossy(bytes)
            .replace("\"format_version\":2", "\"format_version\":1")
            .replace("\"estimate\":null,", "")
            .into_bytes()
    };
    let log_v1 = to_v1(&log_v2);
    // Non-vacuity: the surgery really moved bytes.
    assert!(log_v1.len() < log_v2.len());
    assert!(!String::from_utf8_lossy(&log_v1).contains("estimate"));

    let snapshot_text = String::from_utf8_lossy(&snapshot_v2).into_owned();
    let stripped = snapshot_text
        .replace("\"format_version\":2", "\"format_version\":1")
        .replace("\"estimate\":null,", "");
    let snapshot_v1 = stripped
        .replace(
            &format!("\"log_offset\":{}", log_v2.len()),
            &format!("\"log_offset\":{}", log_v1.len()),
        )
        .into_bytes();
    assert!(snapshot_v1.len() < snapshot_v2.len());

    let v1_dir = TempDir::new().unwrap();
    let v1_path = log_path(&v1_dir);
    fs::write(&v1_path, &log_v1).unwrap();
    fs::write(snapshot_path(&v1_path), &snapshot_v1).unwrap();

    // The v2 reader restores the v1 snapshot and replays the v1 tail:
    // the same graph, every estimate `None` — the absent field's
    // reading. The header migrated in place on the write-capable open.
    let reopened = JsonlStore::open_log(&v1_path).unwrap();
    assert_eq!(reopened.graph(), &written);
    for task in reopened.graph().tasks() {
        assert_eq!(task.estimate, None);
    }
    let migrated = fs::read(&v1_path).unwrap();
    assert!(migrated.starts_with(b"{\"format_version\":2}\n"));
    assert_eq!(&migrated[21..], &log_v1[21..]);
}

// =================================================================
// Determinism (required test 4, §8).
// =================================================================

#[test]
fn two_snapshots_of_unchanged_state_are_deterministic() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);

    let first_offset = store.snapshot().unwrap();
    let first = snapshot_bytes(&path);
    let second_offset = store.snapshot().unwrap();
    let second = snapshot_bytes(&path);

    assert_eq!(first, second);
    assert_eq!(first_offset, second_offset);
    // Snapshotting appended nothing: the log is untouched.
    assert_eq!(fs::read(&path).unwrap().len() as u64, first_offset);
}

#[test]
fn two_event_orders_reaching_one_state_are_deterministic() {
    // The same state reached through two legal event orders — the edge
    // and patch mutations after the adds reordered, and `relates` linked
    // from its other spelling — gives byte-identical snapshots: node
    // arrays and edge sets serialize in their documented orders, not in
    // event order, and the same three lines give the same `log_offset`.
    let dir = TempDir::new().unwrap();
    // Fresh values per call: both orders must stamp the same instants,
    // and `Timestamp` is not `Copy`.
    let ts_link = || at("2026-09-25T10:00:00Z");
    let ts_relate = || at("2026-09-25T10:05:00Z");
    let ts_patch = || at("2026-09-25T10:10:00Z");
    let patch = || TaskPatch {
        title: Some("Bring up the sensor and hold the line".to_owned()),
        ..TaskPatch::default()
    };

    let one = dir.path().join("one.jsonl");
    let mut store = JsonlStore::create_log(&one, repo()).unwrap();
    seed_two_tasks(&mut store);
    store.link_blocks(the_edge(), ts_link()).unwrap();
    store
        .link_relates(task_id(1), task_id(2), ts_relate())
        .unwrap();
    store.update_task(task_id(1), ts_patch(), patch()).unwrap();
    store.snapshot().unwrap();
    let first = snapshot_bytes(&one);
    let first_log_len = fs::read(&one).unwrap().len();
    drop(store);

    let two = dir.path().join("two.jsonl");
    let mut store = JsonlStore::create_log(&two, repo()).unwrap();
    seed_two_tasks(&mut store);
    store.update_task(task_id(1), ts_patch(), patch()).unwrap();
    store
        .link_relates(task_id(2), task_id(1), ts_relate())
        .unwrap();
    store.link_blocks(the_edge(), ts_link()).unwrap();
    store.snapshot().unwrap();
    let second = snapshot_bytes(&two);
    drop(store);

    assert_eq!(first_log_len, fs::read(&two).unwrap().len());
    assert_eq!(first, second);
}

/// One epic and two tasks — the shared prefix of both event orders
/// above (adds fix the ids, so they cannot be reordered).
fn seed_two_tasks(store: &mut JsonlStore) {
    let epic = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    store
        .add_task(
            task_draft(&epic.id, "Bring up the sensor"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    store
        .add_task(
            task_draft(&epic.id, "Fix the dropped frames"),
            at("2026-09-25T09:30:00Z"),
        )
        .unwrap();
}

/// The one `blocks` edge both orders link, rebuilt per call.
fn the_edge() -> BlocksEdge {
    BlocksEdge::TaskToTask {
        from: task_id(1),
        to: task_id(2),
    }
}

// =================================================================
// Byte stability (required test 5, §8 — the snapshot half; the
// compaction half lives in `compact.rs`).
// =================================================================

#[test]
fn snapshot_reopen_snapshot_is_byte_stable() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    store.snapshot().unwrap();
    let first = snapshot_bytes(&path);
    drop(store);

    // §8's serialize → load → serialize over the whole document: the
    // reopened store restores the same graph and re-snapshots to the
    // same bytes at the same offset — the log did not move.
    let mut reopened = JsonlStore::open_log(&path).unwrap();
    reopened.snapshot().unwrap();
    assert_eq!(snapshot_bytes(&path), first);
}

// =================================================================
// The D4 layout (Decision 3) and no debris (required test 11).
// =================================================================

#[test]
fn the_layout_derives_the_snapshot_path_and_open_uses_it() {
    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let camera = repo();
    let slug = camera.slug.clone();
    let mut store = root.register(camera).unwrap();
    store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    store.snapshot().unwrap();
    let written: Graph = store.graph().clone();
    drop(store);

    // The layout accessor is the engine's suffix rule: the log path plus
    // `.snapshot`, and nothing else joined the directory — the snapshot
    // has no lock file of its own (Decision 2), and no `.tmp` survived.
    assert_eq!(
        root.snapshot_path(&slug),
        snapshot_path(&root.log_path(&slug))
    );
    assert_eq!(
        dir_entries(&root.store_dir(&slug)),
        [
            "event-log.jsonl",
            "event-log.jsonl.lock",
            "event-log.jsonl.snapshot"
        ]
    );

    // `DataRoot::open` is snapshot-aware through the engine: the same
    // graph the writer held.
    let reopened = root.open(&slug).unwrap();
    assert_eq!(reopened.graph(), &written);
}

#[test]
fn snapshotting_leaves_no_debris() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    assert_eq!(
        dir_entries(dir.path()),
        ["event-log.jsonl", "event-log.jsonl.lock"]
    );

    store.snapshot().unwrap();
    // The pid-suffixed temp was consumed by the atomic rename.
    assert_eq!(
        dir_entries(dir.path()),
        [
            "event-log.jsonl",
            "event-log.jsonl.lock",
            "event-log.jsonl.snapshot"
        ]
    );
    drop(store);
    assert_eq!(
        dir_entries(dir.path()),
        [
            "event-log.jsonl",
            "event-log.jsonl.lock",
            "event-log.jsonl.snapshot"
        ]
    );
}

// =================================================================
// The document parses back through the crate's codec (Decision 4's
// field shapes), independent of the golden bytes.
// =================================================================

#[test]
fn the_document_mirror_round_trips_through_the_codec() {
    // The mirror is the fixture side's reading of Decision 4: relates
    // serialize as two-element arrays of id strings, blocks reuse the
    // log's edge DTO, and the whole document survives
    // deserialize → serialize byte-identically (§8).
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    store.snapshot().unwrap();
    drop(store);

    let bytes = snapshot_bytes(&path);
    let document: SnapshotMirror = from_bytes(&bytes).unwrap();
    assert_eq!(to_bytes_of(&document), bytes);
    assert_eq!(document.relates, vec![vec![task_id(1), task_id(2)]]);
    assert_eq!(document.blocks.len(), 2);

    let text = String::from_utf8(bytes).unwrap();
    // A relates pair is a two-element array of id strings on the wire.
    assert!(
        text.contains("\"relates\":[[\"camera:U1\",\"camera:U2\"]]"),
        "{text}"
    );
    // Exactly one trailing LF: one compact document, one line.
    assert!(text.ends_with("}\n"), "the document ends in }} plus one LF");
    assert!(!text.ends_with("\n\n"));
}

fn to_bytes_of(document: &SnapshotMirror) -> Vec<u8> {
    let mut bytes = to_bytes(document).unwrap();
    bytes.push(b'\n');
    bytes
}
