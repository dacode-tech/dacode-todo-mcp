//! D19's lock-free read through the layout: the committed graph under a
//! held lock (f110's scenario), the committed prefix under an in-flight
//! append, and the faults the reading weakens nothing else on — plus the
//! no-debris, snapshot-aware, version, missing-store,
//! unregistered-directory and determinism pins.
//!
//! The byte-splicing helpers follow the repair suite's rule: a tear is
//! always a real serialized line cut short, never hand-written JSON.
//! The two hand-written lines anywhere here are fault-injection
//! fixtures no writer can produce — the foreign-version header and the
//! committed garbage line.

use std::fs;

use tempfile::TempDir;
use todo_domain::{EpicId, Graph, References, Status, TaskDraft, TaskKind};
use todo_store::{DataRoot, FORMAT_VERSION, JsonlStore, Store, StoreError};

use super::fixtures::{
    at, camera, data_root, dir_entries, epic_draft, register_and_release, repo, slug,
};

/// How many bytes past the LF a tear cuts: enough to make the fragment
/// unparseable as well as unterminated — the repair suite's shape.
const TEAR_BYTES: usize = 5;

/// The damage bytes spliced as a *committed* line: terminated, so it is
/// never a tear — the repair suite's shape.
const GARBAGE_LINE: &[u8] = b"{{{ not json\n";

/// A task draft — the log suite's fixture shape, so the committed world
/// holds more than epics.
fn task_draft(epic: &EpicId, title: &str) -> TaskDraft {
    TaskDraft {
        epic: epic.clone(),
        kind: TaskKind::Task,
        title: title.to_owned(),
        body: String::new(),
        status: Status::Open,
        verify: Some("cargo test -p todo-store".to_owned()),
        evidence: None,
        note: None,
        references: References::new(),
        priority: None,
        until: None,
    }
}

/// Registers camera with one epic and one task under it — the committed
/// world every read here reads back — and returns the store **still
/// holding its lock**: the f110 pin keeps the handle, every other case
/// drops it.
fn held_store(root: &DataRoot) -> JsonlStore {
    let mut store = root.register(camera()).unwrap();
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
}

/// [`held_store`], closed, plus the graph it replayed to — the committed
/// graph a read must answer with.
fn committed_graph(root: &DataRoot) -> Graph {
    let store = held_store(root);
    let graph = store.graph().clone();
    drop(store);
    graph
}

/// The torn fragment spliced after the clean bytes: the log's final real
/// event line with its LF and [`TEAR_BYTES`] more bytes dropped — what a
/// crash during that event's `write_all` leaves, spliced from a real
/// serialized line and never hand-written.
fn torn_fragment(valid: &[u8]) -> Vec<u8> {
    let last_lf = valid.iter().rposition(|byte| *byte == b'\n').unwrap();
    let start = valid[..last_lf]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |previous| previous + 1);
    let line = &valid[start..last_lf];
    line[..line.len() - TEAR_BYTES].to_vec()
}

/// The log's bytes with the torn fragment appended after them: the
/// committed log intact, plus one unterminated line — the in-flight
/// append a lock-free read must answer around.
fn tear_append(valid: &[u8]) -> Vec<u8> {
    let mut torn = valid.to_vec();
    torn.extend_from_slice(&torn_fragment(valid));
    torn
}

/// The log's bytes with a committed garbage line spliced after the seed —
/// interior damage, which no crash produces and no path forgives — plus
/// the offset the garbage landed at.
fn with_interior_garbage(valid: &[u8]) -> (Vec<u8>, usize) {
    let first_lf = valid.iter().position(|byte| *byte == b'\n').unwrap();
    let second_lf = valid[first_lf + 1..]
        .iter()
        .position(|byte| *byte == b'\n')
        .unwrap()
        + first_lf
        + 1;
    let mut damaged = valid[..=second_lf].to_vec();
    let garbage_at = second_lf + 1;
    damaged.extend_from_slice(GARBAGE_LINE);
    damaged.extend_from_slice(&valid[garbage_at..]);
    (damaged, garbage_at)
}

/// The log's bytes with a hand-written header declaring `version` in
/// place of the real one, every event line left as it was — the one
/// hand-written line this suite allows, because no writer can produce a
/// foreign-version header.
fn with_header_version(valid: &[u8], version: u32) -> Vec<u8> {
    let mut spliced = format!("{{\"format_version\":{version}}}\n").into_bytes();
    let first_lf = valid.iter().position(|byte| *byte == b'\n').unwrap();
    spliced.extend_from_slice(&valid[first_lf + 1..]);
    spliced
}

// =================================================================
// The f110 scenario, store-level (required test 1). THE pin.
// =================================================================

#[test]
fn a_lock_free_read_succeeds_while_the_writer_holds_the_lock() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let mut held = held_store(&root);

    // The lock is genuinely held: the locked open is §6.1's `locked` —
    // the refusal f110 measured on a live session server.
    assert!(matches!(
        root.open(&camera_slug).unwrap_err(),
        StoreError::Locked { .. }
    ));

    // The lock-free read answers anyway, with the committed graph.
    let read = root.read_repo(&camera_slug).unwrap();
    assert_eq!(&read, held.graph());

    // And the read cost the holder nothing: it still appends, and the
    // next read sees the newly committed line.
    held.add_epic(epic_draft("Harden the encoder"), at("2026-09-25T10:30:00Z"))
        .unwrap();
    assert_eq!(&root.read_repo(&camera_slug).unwrap(), held.graph());
    drop(held);
}

// =================================================================
// The committed prefix under an in-flight append (required test 2).
// =================================================================

#[test]
fn an_in_flight_append_is_read_as_the_committed_prefix() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let clean = committed_graph(&root);
    let log = root.log_path(&camera_slug);
    let valid = fs::read(&log).unwrap();
    fs::write(&log, tear_append(&valid)).unwrap();

    // The read answers the pre-tear graph — the fragment is an append in
    // flight, not damage.
    let read = root.read_repo(&camera_slug).unwrap();
    assert_eq!(&read, &clean);

    // The locked open on the same bytes still reports the torn tail: the
    // two paths disagree by design, each half pinned.
    let error = root.open(&camera_slug).unwrap_err();
    match &error {
        StoreError::CorruptStore { offset, detail } => {
            assert_eq!(*offset, Some(valid.len()), "{error}");
            assert!(detail.contains("no terminating newline"), "{error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    assert_eq!(error.code(), "corrupt_store");
}

// =================================================================
// A terminated but unparseable final line (required test 3).
// =================================================================

#[test]
fn a_terminated_but_unparseable_final_line_is_corrupt_store() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let _ = committed_graph(&root);
    let log = root.log_path(&camera_slug);
    let valid = fs::read(&log).unwrap();
    let mut damaged = valid.clone();
    damaged.extend_from_slice(GARBAGE_LINE);
    fs::write(&log, &damaged).unwrap();

    // D19 weakens nothing else: a committed line that does not parse is
    // corruption on the read path too, positioned at the line's start.
    let error = root.read_repo(&camera_slug).unwrap_err();
    match &error {
        StoreError::CorruptStore { offset, detail } => {
            assert_eq!(*offset, Some(valid.len()), "{error}");
            assert!(detail.contains("not a parseable event"), "{error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    // And byte-identical to the locked open's report.
    assert_eq!(root.open(&camera_slug).unwrap_err(), error);
}

// =================================================================
// Interior damage (required test 4).
// =================================================================

#[test]
fn interior_damage_is_corrupt_store_on_every_path() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let _ = committed_graph(&root);
    let log = root.log_path(&camera_slug);
    let valid = fs::read(&log).unwrap();
    let (damaged, garbage_at) = with_interior_garbage(&valid);
    fs::write(&log, &damaged).unwrap();

    // The interior of a log does not tear in a crash: damage there is
    // corruption on the read path exactly as on the open path.
    let error = root.read_repo(&camera_slug).unwrap_err();
    match &error {
        StoreError::CorruptStore { offset, detail } => {
            assert_eq!(*offset, Some(garbage_at), "{error}");
            assert!(detail.contains("not a parseable event"), "{error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    assert_eq!(root.open(&camera_slug).unwrap_err(), error);
}

// =================================================================
// No debris, no lock (required test 5).
// =================================================================

#[test]
fn a_lock_free_read_creates_no_lock_and_no_temp_files() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    register_and_release(&root, camera());

    // A released store's lock file removed: the read must not put it
    // back, and must leave no temporary of its own.
    fs::remove_file(root.lock_path(&camera_slug)).unwrap();
    let before = dir_entries(&root.store_dir(&camera_slug));
    assert_eq!(before, vec!["event-log.jsonl".to_owned()]);

    let read = root.read_repo(&camera_slug).unwrap();
    assert_eq!(read.repo().slug, camera_slug);
    assert_eq!(dir_entries(&root.store_dir(&camera_slug)), before);
}

// =================================================================
// Snapshot-aware (required test 6).
// =================================================================

#[test]
fn a_lock_free_read_is_snapshot_aware() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let mut store = held_store(&root);
    store.snapshot().unwrap();
    // The tail past the snapshot: an event the document does not cover.
    store
        .add_epic(epic_draft("Harden the encoder"), at("2026-09-25T10:00:00Z"))
        .unwrap();
    let written = store.graph().clone();
    drop(store);

    // The snapshot is the recovery unit on the read path too: restore,
    // then apply only the tail — and the answer equals the open path's.
    let read = root.read_repo(&camera_slug).unwrap();
    assert_eq!(&read, &written);
    let opened = root.open(&camera_slug).unwrap();
    assert_eq!(opened.graph(), &written);
}

// =================================================================
// Version mismatch (required test 7).
// =================================================================

#[test]
fn a_foreign_version_header_is_version_mismatch() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let _ = committed_graph(&root);
    let log = root.log_path(&camera_slug);
    let valid = fs::read(&log).unwrap();
    fs::write(&log, with_header_version(&valid, FORMAT_VERSION + 1)).unwrap();

    // A newer format is a migration question, not corruption — on the
    // read path exactly as on the open path.
    let error = root.read_repo(&camera_slug).unwrap_err();
    assert_eq!(
        error,
        StoreError::VersionMismatch {
            found: FORMAT_VERSION + 1,
            expected: FORMAT_VERSION
        }
    );
    assert_eq!(root.open(&camera_slug).unwrap_err(), error);
}

// =================================================================
// Missing store (required test 8).
// =================================================================

#[test]
fn a_missing_store_reports_the_honest_not_found() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let absent = slug("never-registered");

    // The same honest io NotFound the locked open reports — and the read
    // created nothing on the way.
    let error = root.read_repo(&absent).unwrap_err();
    assert_eq!(
        error,
        StoreError::Io {
            detail: "open the event log: NotFound".to_owned()
        }
    );
    assert_eq!(root.open(&absent).unwrap_err(), error);
    assert!(!root.store_dir(&absent).exists());
}

// =================================================================
// Unregistered-but-present (required test 9, judgment call (i)).
// =================================================================

#[test]
fn an_unregistered_store_directory_is_read_anyway() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    // A store the registry does not list, built through the trait's
    // constructor (which touches no registry) — the repos suite's
    // `create_unlisted` idiom, the shape a registration crashed after
    // the log landed leaves behind.
    let orphan = repo("orphan", "2026-09-25T08:00:00Z");
    let orphan_slug = orphan.slug.clone();
    let mut store = <JsonlStore as Store>::create(&root, orphan).unwrap();
    store
        .add_epic(
            epic_draft("Ship the orphan pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let written = store.graph().clone();
    drop(store);

    // The registry query side refuses the slug as `unknown_id`...
    assert_eq!(
        root.repo_show(&orphan_slug).unwrap_err().code(),
        "unknown_id"
    );
    // ...and the path-derived read reads it anyway: the store directory
    // is authoritative, and the server layer owns the refusal.
    let read = root.read_repo(&orphan_slug).unwrap();
    assert_eq!(&read, &written);
}

// =================================================================
// Determinism (required test 10).
// =================================================================

#[test]
fn two_lock_free_reads_agree() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let clean = committed_graph(&root);

    let first = root.read_repo(&camera_slug).unwrap();
    let second = root.read_repo(&camera_slug).unwrap();
    assert_eq!(first, second);
    assert_eq!(&first, &clean);
}
