//! §4.1's truncation half: what `compact` writes, what the re-based pair
//! replays to, the reopen-after-rename proof (amended Decision 8, step 5),
//! the crash points of the amended step order, and the
//! report the cli prints (Decision 7).

use std::fs;

use tempfile::TempDir;
use todo_domain::Graph;
use todo_store::{Event, JsonlStore, Store, StoreError, to_bytes};

use super::fixtures::{
    SnapshotMirror, build_burned_number_store, build_golden_store, dir_entries,
    expect_snapshot_fault, snapshot_bytes, snapshot_path,
};
use super::timeline::{
    at, epic_draft, epic_id, expect_corrupt, lines, log_path, repo, task_draft, task_id,
};

/// The compacted log's exact bytes: the header line plus the re-emitted
/// seed carrying `repo` — the current node, not the registration bytes
/// (Decision 9).
fn header_plus_seed(repo: &todo_domain::Repo) -> Vec<u8> {
    let mut expected = Vec::new();
    expected.extend_from_slice(br#"{"format_version":2}"#);
    expected.push(b'\n');
    expected.extend_from_slice(
        &to_bytes(&Event::RepoRegister {
            ts: repo.registered_at.clone(),
            repo: repo.clone(),
        })
        .unwrap(),
    );
    expected.push(b'\n');
    expected
}

// =================================================================
// Compaction bounds replay (required test 6).
// =================================================================

#[test]
fn compaction_rewrites_the_log_and_bounds_replay() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    let graph_before: Graph = store.graph().clone();
    let original = fs::read(&path).unwrap();
    let dropped_expected = lines(&original).len() as u64 - 2;

    let report = store.compact().unwrap();
    assert_eq!(report.log_bytes_before, original.len() as u64);
    assert_eq!(report.events_dropped, dropped_expected);
    assert!(report.log_bytes_after < report.log_bytes_before);

    // The log holds exactly the header and the re-emitted seed — asserted
    // on the bytes, and the seed carries the *current* repo node, whose
    // path and name moved after registration (Decision 9).
    let compacted = fs::read(&path).unwrap();
    assert_eq!(compacted, header_plus_seed(graph_before.repo()));
    assert_eq!(report.log_bytes_after, compacted.len() as u64);

    // The re-based snapshot's offset is the new log's length, so replay
    // over the pair has an empty tail.
    let document = SnapshotMirror::read(&path);
    assert_eq!(document.log_offset, compacted.len() as u64);
    assert_eq!(document.repo, *graph_before.repo());

    // A reopen is the snapshot alone: the same graph, burned numbers
    // included.
    drop(store);
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &graph_before);
    drop(reopened);

    // Appends after compaction land in the new log file and replay
    // through the re-based pair. This runs on a *freshly opened* handle,
    // so it pins the file-level consequence, not the in-place reopen: the
    // step-(5) proof — the test that fails when `reopen_log` is removed
    // from `compact` — is `compacting_a_fresh_store_drops_nothing`, which
    // appends on the *same* handle the compaction ran on.
    let mut reopened = JsonlStore::open_log(&path).unwrap();
    let task = reopened
        .add_task(
            task_draft(&epic_id(1), "one after the compaction"),
            at("2026-09-25T18:00:00Z"),
        )
        .unwrap();
    let mut appended_line = to_bytes(&Event::TaskAdd {
        ts: at("2026-09-25T18:00:00Z"),
        task: task.clone(),
    })
    .unwrap();
    appended_line.push(b'\n');
    let grown = fs::read(&path).unwrap();
    let mut expected = compacted.clone();
    expected.extend_from_slice(&appended_line);
    assert_eq!(grown, expected);
    let graph_after: Graph = reopened.graph().clone();
    drop(reopened);

    // And the tail replays over the snapshot to the same graph.
    let final_store = JsonlStore::open_log(&path).unwrap();
    assert_eq!(final_store.graph(), &graph_after);
    assert_eq!(final_store.graph().task(&task.id), Some(&task));
}

// =================================================================
// A compacted log is not independently replayable (required test 7,
// Decision 10).
// =================================================================

#[test]
fn a_compacted_log_without_its_snapshot_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_burned_number_store(&path);
    let report = store.compact().unwrap();
    // A post-compaction append: the tail's ids do not start at 1 (the
    // counters were restored from the snapshot), so without the document
    // t10's sequential-id check fires — the named error that makes the
    // missing snapshot loud instead of silent.
    let task = store
        .add_task(
            task_draft(&epic_id(1), "one after the compaction"),
            at("2026-09-25T11:00:00Z"),
        )
        .unwrap();
    assert_eq!(task.id, task_id(4));
    let written: Graph = store.graph().clone();
    drop(store);

    // Snapshot aside: the tail's first add carries U4 against a counter
    // that has issued nothing.
    let aside = dir.path().join("snapshot.aside");
    fs::rename(snapshot_path(&path), &aside).unwrap();
    let error = expect_corrupt(&path, report.log_bytes_after as usize, "camera:U4");
    assert!(
        error.to_string().contains("next number of its kind is 1"),
        "{error}"
    );

    // Restored: the pair opens and the graph is whole again.
    fs::rename(&aside, snapshot_path(&path)).unwrap();
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &written);
    assert_eq!(reopened.graph().counters().task(), 4);
}

// =================================================================
// Byte stability (required test 5, the compaction half).
// =================================================================

#[test]
fn compact_reopen_compact_is_byte_stable() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    let first_report = store.compact().unwrap();
    let first_log = fs::read(&path).unwrap();
    let first_snapshot = snapshot_bytes(&path);
    drop(store);

    // The second compaction has nothing left to drop: it rewrites the
    // same bytes and changes nothing — §8's serialize → load → serialize
    // over the whole pair.
    let mut reopened = JsonlStore::open_log(&path).unwrap();
    let second_report = reopened.compact().unwrap();
    assert_eq!(second_report.log_bytes_before, first_report.log_bytes_after);
    assert_eq!(second_report.log_bytes_after, first_report.log_bytes_after);
    assert_eq!(second_report.events_dropped, 0);
    assert_eq!(fs::read(&path).unwrap(), first_log);
    assert_eq!(snapshot_bytes(&path), first_snapshot);
}

// =================================================================
// No debris (required test 11, the compaction half).
// =================================================================

#[test]
fn compaction_leaves_no_debris() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    store.compact().unwrap();

    // Both temps — the new log's and the re-based snapshot's — were
    // consumed by their renames; the snapshot has no lock file of its
    // own (Decision 2).
    assert_eq!(
        dir_entries(dir.path()),
        [
            "event-log.jsonl",
            "event-log.jsonl.lock",
            "event-log.jsonl.snapshot"
        ]
    );
    drop(store);

    // A second compaction over the reopened store leaves the same three.
    let mut reopened = JsonlStore::open_log(&path).unwrap();
    reopened.compact().unwrap();
    drop(reopened);
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
// The report's honesty on a store that never grew (Decision 7: a
// report, not a policy — compacting a fresh store is legal and inert).
// =================================================================

#[test]
fn compacting_a_fresh_store_drops_nothing() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = JsonlStore::create_log(&path, repo()).unwrap();
    let before = fs::read(&path).unwrap();

    let report = store.compact().unwrap();
    assert_eq!(report.events_dropped, 0);
    assert_eq!(report.log_bytes_before, before.len() as u64);
    // The re-emitted seed of an unmoved repo is the registration seed,
    // byte for byte.
    assert_eq!(report.log_bytes_after, before.len() as u64);
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(SnapshotMirror::read(&path).log_offset, before.len() as u64);

    // The pair still replays to the same empty graph, and appending
    // after the inert compaction allocates from 1. These appends run on
    // the *same handle* `compact` ran on — this is the step-(5) reopen
    // proof (amended Decision 8): had `compact` kept the handle on the
    // orphaned inode, the two lines below would be missing from the log
    // at `path` and the final reopen would see an empty graph.
    let epic = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let task = store
        .add_task(
            task_draft(&epic.id, "first after an inert compaction"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    assert_eq!(task.id, task_id(1));
    drop(store);
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph().counters().epic(), 1);
    assert_eq!(reopened.graph().task(&task_id(1)), Some(&task));
}

// =================================================================
// The crash points of the amended step order. The normative table
// is `Store::compact`'s doc. The post-(1)-pre-(3) point — covering
// snapshot + full log, empty tail, correct open with graph equality — is
// already pinned by
// `roundtrip::burned_numbers_survive_a_snapshot_round_trip` (snapshot →
// drop → reopen → graph equal) and
// `roundtrip::snapshot_reopen_snapshot_is_byte_stable`, so it is not
// duplicated here. What follows pins the (3)–(4) window in both of its
// shapes, built from real bytes only: every fixture byte comes from a
// real `snapshot()`/`compact()` run over the deterministic golden
// fixture, never synthesized JSON.
// =================================================================

/// Appends one task under E1 at a fixed instant — the "events since the
/// last compaction" of corner B, byte-identical across twins.
fn append_probe_task(store: &mut JsonlStore) {
    store
        .add_task(
            task_draft(&epic_id(1), "one after the first compaction"),
            at("2026-09-25T18:30:00Z"),
        )
        .unwrap();
}

#[test]
fn a_crash_between_the_renames_is_named_without_a_prior_snapshot() {
    // Corner A: the FIRST-EVER compact(), crashed between step (3) (the
    // log's rename) and step (4) (the snapshot's re-base).
    //
    // Equivalence, established from real runs of the one deterministic
    // fixture: the "cover" store's `snapshot()` bytes are byte-identical
    // to what compact()'s step (1) writes — same graph, same log length,
    // `SnapshotDocument::from_graph` is deterministic — and the twin's
    // real `compact()` leaves exactly the log step (3) renames into
    // place. The twins are asserted byte-identical before either
    // operation, so pairing the twin's short log with the cover store's
    // covering snapshot *is* the crashed compact's on-disk state.
    //
    // Under the pre-amendment truncate-then-re-base order this window
    // left NO snapshot at all beside the short log — the covering
    // document was still its un-renamed `.tmp` — and the open succeeded
    // silently as a bare repo graph, every event lost. The amended
    // order's covering snapshot turns it into a named error.
    let cover_dir = TempDir::new().unwrap();
    let cover_path = log_path(&cover_dir);
    let mut cover = build_golden_store(&cover_path);
    let full_log = fs::read(&cover_path).unwrap();
    let covered = cover.snapshot().unwrap();
    let covering = snapshot_bytes(&cover_path);
    assert_eq!(covered, full_log.len() as u64);
    let full_graph = cover.graph().clone();
    drop(cover);

    let twin_dir = TempDir::new().unwrap();
    let twin_path = log_path(&twin_dir);
    let mut twin = build_golden_store(&twin_path);
    assert_eq!(
        fs::read(&twin_path).unwrap(),
        full_log,
        "the twins must agree byte-for-byte before compacting"
    );
    let report = twin.compact().unwrap();
    let short_log = fs::read(&twin_path).unwrap();
    assert_eq!(report.log_bytes_after, short_log.len() as u64);
    drop(twin);

    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    fs::write(&path, &short_log).unwrap();
    fs::write(snapshot_path(&path), &covering).unwrap();
    assert!(
        covered > short_log.len() as u64,
        "the window's premise: the covering offset exceeds the short log"
    );

    // Named, both numbers in the detail, no path, deterministic
    // (`expect_snapshot_fault` pins all four, and its `unwrap_err` is
    // the assertion that this is NOT a clean bare-repo open).
    let error = expect_snapshot_fault(&path, "beyond the log's end");
    let detail = error.to_string();
    assert!(detail.contains(&covered.to_string()), "{error}");
    assert!(detail.contains(&short_log.len().to_string()), "{error}");

    // The window is recoverable, which is why it is acceptable: the
    // live snapshot still holds the whole graph, so t13's repair — here
    // spelled as re-basing the offset onto the short log — reopens to
    // the full pre-compaction state, burned numbers included.
    let mut mirror = SnapshotMirror::read(&path);
    mirror.log_offset = short_log.len() as u64;
    mirror.write(&path);
    let repaired = JsonlStore::open_log(&path).unwrap();
    assert_eq!(repaired.graph(), &full_graph);
    assert_eq!(repaired.graph().counters(), full_graph.counters());
}

#[test]
fn a_crash_between_the_renames_is_named_with_a_prior_snapshot_and_appends() {
    // Corner B: a REPEAT compact() over a store holding the prior
    // compaction's re-based snapshot, with events appended since. Same
    // window, same twin construction and equivalence argument as
    // corner A, one compaction of history behind it.
    //
    // The regression this exists to prevent: under the pre-amendment
    // order the crash left the PRIOR re-based snapshot in place, whose
    // offset equalled the previous short log's length — and since the
    // re-emitted seed is byte-stable for an unmoved repo, that also
    // equalled the NEW short log's length, so all of Decision 5's
    // offset checks passed and the open silently restored the graph as
    // of the previous compaction, dropping the appended events. Under
    // the amended order step (1) has already replaced that snapshot
    // with the covering one (offset = the grown length), so the same
    // crash is a named error. The load-bearing inequality is asserted
    // below.
    let cover_dir = TempDir::new().unwrap();
    let cover_path = log_path(&cover_dir);
    let mut cover = build_golden_store(&cover_path);
    cover.compact().unwrap();
    append_probe_task(&mut cover);
    let grown = fs::read(&cover_path).unwrap();
    let covered = cover.snapshot().unwrap();
    let covering = snapshot_bytes(&cover_path);
    assert_eq!(covered, grown.len() as u64);
    let graph = cover.graph().clone();
    drop(cover);

    let twin_dir = TempDir::new().unwrap();
    let twin_path = log_path(&twin_dir);
    let mut twin = build_golden_store(&twin_path);
    twin.compact().unwrap();
    append_probe_task(&mut twin);
    assert_eq!(
        fs::read(&twin_path).unwrap(),
        grown,
        "the twins must agree byte-for-byte before the second compaction"
    );
    let report = twin.compact().unwrap();
    let short_log = fs::read(&twin_path).unwrap();
    assert_eq!(report.log_bytes_after, short_log.len() as u64);
    drop(twin);

    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    fs::write(&path, &short_log).unwrap();
    fs::write(snapshot_path(&path), &covering).unwrap();
    // The load-bearing inequality: the covering offset strictly exceeds
    // the short log. The pre-amendment order left a snapshot whose
    // offset EQUALLED it — that equality is what opened clean and stale.
    assert!(covered > short_log.len() as u64);

    // Named error — specifically NOT a clean open to the previous
    // compaction's stale graph (`expect_snapshot_fault`'s `unwrap_err`
    // is that assertion, spelled out once more for the record):
    assert!(JsonlStore::open_log(&path).is_err());
    let error = expect_snapshot_fault(&path, "beyond the log's end");
    let detail = error.to_string();
    assert!(detail.contains(&covered.to_string()), "{error}");
    assert!(detail.contains(&short_log.len().to_string()), "{error}");

    // And the recovery re-bases to the FULL graph — the appended task
    // included, the state the stale open would have dropped.
    let mut mirror = SnapshotMirror::read(&path);
    mirror.log_offset = short_log.len() as u64;
    mirror.write(&path);
    let repaired = JsonlStore::open_log(&path).unwrap();
    assert_eq!(repaired.graph(), &graph);
    assert_eq!(repaired.graph().counters().task(), 5);
}

// =================================================================
// The step order itself, *induced* rather than constructed (f19).
//
// The two corner tests above build the (3)–(4) window's on-disk state
// from real bytes, so they pin what the window leaves and cannot see the
// order that produced it: reversing the steps in `compact` keeps both
// green. This one makes step (1) fail instead — a directory parked at
// `<log>.snapshot` is a rename target `rename(2)` refuses — and asserts
// the file-level consequences of the covering snapshot being written
// first: the log was never shortened and the append handle was never
// orphaned. Under the pre-amendment truncate-then-re-base order the
// log's rename had already landed, so both assertions fail.
// =================================================================

#[test]
fn a_refused_covering_snapshot_leaves_the_log_untouched() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    let before = fs::read(&path).unwrap();

    // The fault seam. A snapshot has no lock of its own (Decision 2), so
    // nothing but the rename itself notices this.
    fs::create_dir(snapshot_path(&path)).unwrap();

    let error = store.compact().unwrap_err();
    // §6.1's `io`, and the context says which rename refused — the
    // snapshot's, not step (3)'s "the compacted log". The context is this
    // crate's own contract text, so it is pinned; the `ErrorKind` spelling
    // beside it is the platform's, so it is not. Steps (1) and (4) rename
    // the same subject and are therefore indistinguishable here: what
    // places this refusal before step (3) is the untouched log below.
    assert_eq!(error.code(), "io");
    match &error {
        StoreError::Io { detail } => {
            assert!(
                detail.contains("rename the snapshot into place"),
                "the snapshot's rename refused, not the log's: {error}"
            );
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected an io fault, got {other:?}"),
    }

    // Step (3) never ran: the log is byte-for-byte the pre-compaction one
    // and the parked directory is still a directory, so nothing landed in
    // place. What the refused rename does leave is §4.3's temp file —
    // accepted debris of a failed attempt, pinned here so this test's
    // "untouched" is precise about what it claims. The pid-suffixed name
    // is what this process's next `write_temp` truncates.
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(snapshot_path(&path).is_dir());
    assert_eq!(
        dir_entries(dir.path())
            .iter()
            .filter(|name| name.ends_with(".tmp"))
            .count(),
        1,
        "the refused rename leaves exactly one temporary"
    );

    // Nor is the failed compaction session-fatal, which is the promise
    // `Store::snapshot`'s doc makes and step (1) is a snapshot: the log's
    // inode has not been replaced underneath the append handle, so this
    // append lands in the log at `path`. No other test covers an append
    // after a refused compaction.
    let task = store
        .add_task(
            task_draft(&epic_id(1), "one after the refused compaction"),
            at("2026-09-25T19:00:00Z"),
        )
        .unwrap();
    let grown = fs::read(&path).unwrap();
    assert!(grown.starts_with(&before), "the append landed in this log");
    assert!(grown.len() > before.len());
    let graph_after = store.graph().clone();
    drop(store);

    // The parked directory is a named fault on the read side too, not a
    // silent "no snapshot": reading a directory fails, so replay reports
    // `io` rather than opening the log as an un-snapshotted store.
    let with_fault = JsonlStore::open_log(&path).unwrap_err();
    assert_eq!(with_fault.code(), "io");

    // With the injected fault removed the pair is the pre-compaction one
    // plus the append, burned numbers included: §4's "never silent
    // truncation", observable from the files alone.
    fs::remove_dir(snapshot_path(&path)).unwrap();
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &graph_after);
    assert_eq!(reopened.graph().counters(), graph_after.counters());
    assert_eq!(reopened.graph().task(&task.id), Some(&task));
}
