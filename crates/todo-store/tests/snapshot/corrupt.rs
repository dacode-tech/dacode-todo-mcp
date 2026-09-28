//! The snapshot's named-error surface (Decision 5's every fault,
//! Decision 6's two checks, and the torn document): a snapshot that
//! contradicts its log, a document this build cannot parse, and a graph
//! the domain refuses are all `corrupt_store` or `version_mismatch` —
//! never a panic, never a silently wrong graph (§4, §4.3).
//!
//! Every fixture starts from a **real** snapshot the engine wrote and is
//! mutated through the document mirror or spliced at the byte level —
//! never hand-written JSON (t10's f8 discipline).

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use tempfile::TempDir;
use todo_domain::RepoSlug;
use todo_store::{BlocksEdgeDto, FORMAT_VERSION, JsonlStore, Store, StoreError};

use super::fixtures::{
    SnapshotMirror, expect_snapshot_fault, snapshot_bytes, snapshot_path, snapshotted_short_store,
    with_unknown_field,
};
use super::timeline::{at, epic_id, line_offsets, log_path, task_draft, task_id};

/// A fresh store directory holding t10's short log — epic E1, tasks
/// U1/U2, one `blocks` edge — and one real snapshot, with the writer
/// dropped so the fault cases can reopen it (§8's single writer).
fn fault_base(dir: &TempDir) -> PathBuf {
    let path = log_path(dir);
    drop(snapshotted_short_store(&path));
    path
}

fn other_slug() -> RepoSlug {
    RepoSlug::from_str("other").unwrap()
}

// =================================================================
// Decision 5 (c): the document itself.
// =================================================================

#[test]
fn a_malformed_snapshot_is_a_positioned_named_error() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);

    // Garbage bytes: the codec's offset over a whole-file read *is*
    // absolute, so the report is positioned like the registry's.
    fs::write(snapshot_path(&path), b"{{{ not json\n").unwrap();
    let error = JsonlStore::open_log(&path).unwrap_err();
    assert_eq!(error.code(), "corrupt_store");
    match &error {
        StoreError::CorruptStore {
            offset: Some(_),
            detail,
        } => {
            assert!(
                detail.contains("the snapshot is not a parseable document"),
                "{error}"
            );
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    // Deterministic: the second open reports the identical error.
    assert_eq!(error, JsonlStore::open_log(&path).unwrap_err());

    // An empty file is corrupt too, never an absent snapshot: absence is
    // `NotFound`, an empty *document* is damage.
    fs::write(snapshot_path(&path), b"").unwrap();
    let error = JsonlStore::open_log(&path).unwrap_err();
    assert_eq!(error.code(), "corrupt_store");
    assert!(
        error
            .to_string()
            .contains("the snapshot is not a parseable document"),
        "{error}"
    );
}

#[test]
fn a_torn_snapshot_is_a_positioned_named_error() {
    // §4.3 says a snapshot is never torn *by construction* — temp-file +
    // fsync + atomic rename. A hand-damaged one is `corrupt_store` like
    // any other: the honest answer (judgment call (f)). The offset is
    // the codec's parse position within the whole-file document — the
    // only position a single-JSON document has, and absolute because the
    // read is whole-file (the registry's convention). It is *not* a
    // truncation point the way a log-line offset is: a snapshot is
    // rewritten whole or not at all, so a repair regenerates it from the
    // log tail rather than cutting at the offset.
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let valid = snapshot_bytes(&path);

    // Two cuts that both land mid-document. Dropping only the trailing
    // LF is *not* a cut — the codec accepts trailing whitespace — and is
    // pinned below as the parser-lenience fact it is.
    for cut in [valid.len() * 3 / 5, valid.len() - 2] {
        fs::write(snapshot_path(&path), &valid[..cut]).unwrap();
        let error = JsonlStore::open_log(&path).unwrap_err();
        assert_eq!(error.code(), "corrupt_store", "cut at {cut}");
        match &error {
            StoreError::CorruptStore {
                offset: Some(_),
                detail,
            } => {
                assert!(
                    detail.contains("the snapshot is not a parseable document"),
                    "{error}"
                );
            }
            other => panic!("expected a positioned corrupt_store, got {other:?}"),
        }
        assert_eq!(error, JsonlStore::open_log(&path).unwrap_err());
    }

    // The trailing LF the write path adds is the golden workflow's line
    // discipline, not a parse requirement: without it the same document
    // still opens.
    fs::write(snapshot_path(&path), &valid[..valid.len() - 1]).unwrap();
    assert!(JsonlStore::open_log(&path).is_ok());
}

#[test]
fn a_foreign_version_is_the_migration_question() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.format_version = FORMAT_VERSION + 1;
    mirror.write(&path);

    let error = JsonlStore::open_log(&path).unwrap_err();
    assert_eq!(
        error,
        StoreError::VersionMismatch {
            found: FORMAT_VERSION + 1,
            expected: FORMAT_VERSION
        }
    );
    assert_eq!(error.code(), "version_mismatch");

    // The probe runs before the strict parse: a newer build's document
    // with fields this build has never seen is still the version report,
    // not corruption (t11's registry ordering).
    let bytes = with_unknown_field(&snapshot_bytes(&path));
    fs::write(snapshot_path(&path), &bytes).unwrap();
    let error = JsonlStore::open_log(&path).unwrap_err();
    assert_eq!(error.code(), "version_mismatch");
}

#[test]
fn an_unknown_field_at_the_current_version_is_rejected() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let bytes = with_unknown_field(&snapshot_bytes(&path));
    fs::write(snapshot_path(&path), &bytes).unwrap();

    let error = JsonlStore::open_log(&path).unwrap_err();
    assert_eq!(error.code(), "corrupt_store");
    assert!(error.to_string().contains("surprise"), "{error}");
    assert!(
        error
            .to_string()
            .contains("the snapshot is not a parseable document"),
        "{error}"
    );
}

#[test]
fn a_relates_entry_that_is_not_a_pair_is_rejected() {
    // The reject side of `relates_from_arrays`' narrowing (snapshot.rs):
    // the reader-side answer to the JSON engine's tuple gap reads the
    // wire shape as nested seqs, and an inner array that is not exactly
    // a pair is a corrupt document — pinned so a silent-drop regression
    // cannot pass the suite. Fixtures go through the mirror: real
    // values, the crate's codec, never hand-written JSON. The accept
    // side over real golden bytes is
    // `roundtrip::the_document_mirror_round_trips_through_the_codec`;
    // the tail of this test replays it through the same mechanism.
    for relates in [
        vec![vec![task_id(1)]],
        vec![vec![task_id(1), task_id(2), task_id(3)]],
    ] {
        let dir = TempDir::new().unwrap();
        let path = fault_base(&dir);
        let mut mirror = SnapshotMirror::read(&path);
        mirror.relates = relates;
        mirror.write(&path);

        let error = JsonlStore::open_log(&path).unwrap_err();
        assert_eq!(error.code(), "corrupt_store");
        match &error {
            StoreError::CorruptStore { detail, .. } => {
                assert!(
                    detail.contains("a relates entry is a two-element array of task ids"),
                    "{error}"
                );
                assert!(!detail.contains('/'), "a detail carries no path: {error}");
            }
            other => panic!("expected a corrupt_store, got {other:?}"),
        }
        assert_eq!(error, JsonlStore::open_log(&path).unwrap_err());
    }

    // The valid two-element case still round-trips.
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.relates = vec![vec![task_id(1), task_id(2)]];
    mirror.write(&path);
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(
        reopened.graph().relates_pairs(),
        vec![(task_id(1), task_id(2))]
    );
}

// =================================================================
// Decision 5 (d)/(e): the snapshot↔log cross-checks.
// =================================================================

#[test]
fn a_snapshot_of_another_repo_is_a_named_error() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.repo.slug = other_slug();
    mirror.write(&path);

    // Slug only, and both slugs named: the report a repair needs.
    expect_snapshot_fault(
        &path,
        "the snapshot holds repo other, but the log's seed registers camera",
    );
}

#[test]
fn a_log_offset_beyond_the_log_is_a_named_error() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let log_len = fs::read(&path).unwrap().len() as u64;
    let mut mirror = SnapshotMirror::read(&path);
    mirror.log_offset = log_len + 100;
    mirror.write(&path);

    let error = expect_snapshot_fault(&path, "beyond the log's end");
    // Both numbers ride in the detail — the claimed offset and the log's
    // length — which is what makes `offset: None` informative here.
    assert!(
        error
            .to_string()
            .contains(&format!("{} > {log_len}", log_len + 100)),
        "{error}"
    );
}

#[test]
fn a_log_offset_inside_a_line_is_a_named_error() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let log = fs::read(&path).unwrap();
    let offsets = line_offsets(&log);
    // Three bytes into the first event line: a position no line starts
    // at, however the bytes around it parse.
    let mut mirror = SnapshotMirror::read(&path);
    mirror.log_offset = (offsets[2] + 3) as u64;
    mirror.write(&path);

    expect_snapshot_fault(&path, "does not land on a line boundary");
}

#[test]
fn a_log_offset_before_the_seed_is_a_named_error() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let log = fs::read(&path).unwrap();
    let offsets = line_offsets(&log);
    // The seed line's *start* is a real line boundary, but the seed is
    // not the snapshot's to cover: replay builds the graph's scope from
    // the seed before consulting the document.
    let mut mirror = SnapshotMirror::read(&path);
    mirror.log_offset = offsets[1] as u64;
    mirror.write(&path);

    expect_snapshot_fault(&path, "does not cover the log's seed");

    // Zero is refused by the same check.
    mirror.log_offset = 0;
    mirror.write(&path);
    expect_snapshot_fault(&path, "does not cover the log's seed");
}

// =================================================================
// Decision 6: the restore's own checks.
// =================================================================

#[test]
fn a_live_id_above_its_counter_is_a_named_error() {
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    // U2 is live but the document claims the task counter never passed
    // 1: the internal-consistency check names the id.
    let mut mirror = SnapshotMirror::read(&path);
    assert_eq!(mirror.counters.task, 2);
    mirror.counters.task = 1;
    mirror.write(&path);

    expect_snapshot_fault(
        &path,
        "the snapshot's task counter holds 1, below its live id camera:U2",
    );
}

#[test]
fn a_snapshot_the_domain_rejects_is_a_named_error() {
    // Four damaged documents, one contract each: `corrupt_store` with
    // the domain's own message in the detail and `offset: None` — a
    // semantic fault has no byte position (t9's convention).
    let dir = TempDir::new().unwrap();

    // A task whose epic the document does not hold.
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.tasks[0].epic = epic_id(9);
    mirror.write(&path);
    expect_snapshot_fault(&path, "the snapshot contradicts itself");
    expect_snapshot_fault(&path, "unknown id: camera:E9");

    // A node of another repository.
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.tasks[0].repo = other_slug();
    mirror.write(&path);
    expect_snapshot_fault(&path, "the snapshot contradicts itself");
    expect_snapshot_fault(&path, "node camera:U1 would be orphaned");

    // A duplicate id.
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.tasks.push(mirror.tasks[0].clone());
    mirror.write(&path);
    expect_snapshot_fault(&path, "the snapshot contradicts itself");
    expect_snapshot_fault(&path, "duplicate id: camera:U1");

    // A `blocks` cycle the write path could never have produced.
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let mut mirror = SnapshotMirror::read(&path);
    mirror.blocks.push(BlocksEdgeDto::TaskToTask {
        from: task_id(2),
        to: task_id(1),
    });
    mirror.write(&path);
    expect_snapshot_fault(&path, "the snapshot contradicts itself");
    expect_snapshot_fault(&path, "cycle");
}

// =================================================================
// The log's own faults keep priority (Decision 5's order: the log is
// checked before the snapshot is consulted).
// =================================================================

#[test]
fn a_malformed_log_outranks_its_snapshot() {
    // The header and seed are checked in `replay` before the snapshot is
    // even read, so a damaged log reports its own §4.3 fault at its own
    // offset whatever the healthy snapshot beside it holds.
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    let valid = fs::read(&path).unwrap();

    // Corrupt the header line's bytes in place, length-preserving so the
    // seed's offset (and every line boundary) is untouched: the header no
    // longer parses.
    let mut damaged = valid.clone();
    damaged[1] = b'#';
    fs::write(&path, &damaged).unwrap();

    let error = JsonlStore::open_log(&path).unwrap_err();
    match &error {
        StoreError::CorruptStore {
            offset: Some(0),
            detail,
        } => {
            assert!(detail.contains("not a format-version header"), "{error}");
        }
        other => panic!("expected the log's header report at offset 0, got {other:?}"),
    }
}

#[test]
fn a_torn_tail_after_the_snapshot_is_the_logs_own_error() {
    // §4.3's real crash shape: a snapshot at offset O, more events
    // appended past it, then a crash mid-append tears the last line. On
    // reopen the snapshot loads (O is inside the log and on a boundary)
    // and the tail from O is parsed, so the torn last line is caught as
    // the log's own positioned `corrupt_store` — exactly the no-snapshot
    // report, at the same offset.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = snapshotted_short_store(&path);
    let covered = store.snapshot().unwrap();
    // Append one more task past the snapshot: a tail line.
    store
        .add_task(
            task_draft(&epic_id(1), "one in the tail"),
            at("2026-09-25T11:00:00Z"),
        )
        .unwrap();
    drop(store);

    let valid = fs::read(&path).unwrap();
    assert!((valid.len() as u64) > covered);
    // Tear the last line: drop its terminating LF.
    fs::write(&path, &valid[..valid.len() - 1]).unwrap();

    let offsets = line_offsets(&valid);
    let last = offsets[offsets.len() - 1];
    let error = JsonlStore::open_log(&path).unwrap_err();
    match &error {
        StoreError::CorruptStore {
            offset: Some(offset),
            detail,
        } => {
            assert_eq!(*offset, last);
            assert!(detail.contains("no terminating newline"), "{error}");
        }
        other => panic!("expected the log's torn-line report, got {other:?}"),
    }
}

#[test]
fn a_missing_snapshot_is_not_a_fault() {
    // Absence is the no-snapshot replay path, not corruption: remove the
    // document and the short log opens on its own (pre-compaction logs
    // stay independently replayable — Decision 10's other half).
    let dir = TempDir::new().unwrap();
    let path = fault_base(&dir);
    fs::remove_file(snapshot_path(&path)).unwrap();
    assert!(JsonlStore::open_log(&path).is_ok());
    assert!(
        !snapshot_path(&path).exists(),
        "a read path creates nothing"
    );
}
