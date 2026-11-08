//! The refusals and the faults that are not refusals: a clean store is a
//! no-op; an interior fault is refused at its own offset, including when a
//! torn final line sits beside it; a fault in a *committed* final line is
//! refused as committed; a tear at byte 0 is refused rather than truncated
//! into an empty log; a fault with no byte position is refused for want of
//! a truncation point; a contended store is §6.1's `locked` with nothing
//! read; a snapshot/log pair that diverged is reported with its real
//! numbers and not written; a foreign format version and an unparseable
//! snapshot propagate as the errors `open` reports; and a store that is not
//! there reports the honest `io`/`NotFound`. Every case asserts the log's
//! bytes are unchanged: a refusal has no effect (§8's atomic rejected
//! write).

use std::fs;

use tempfile::TempDir;
use todo_store::{FORMAT_VERSION, LockTarget, RepairOutcome, Store, StoreError};

use super::fixtures::{
    GARBAGE_LINE, TEAR_BYTES, assert_store_files, build_short_store, camera_slug, data_root,
    dir_entries, expect_open_corrupt, expect_open_unpositioned, expect_refusal, expect_torn,
    foreign_snapshot, log_bytes, log_lines, log_path, short_store_graph, store_dir, torn,
    torn_fragment, with_header_version,
};
use super::timeline::line_offsets;

// =================================================================
// A clean store (required test 3).
// =================================================================

#[test]
fn a_clean_store_is_a_no_op() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    let graph = short_store_graph(&root);
    let valid = log_bytes(&root);

    for dry_run in [true, false] {
        assert_eq!(root.repair(&slug, dry_run).unwrap(), RepairOutcome::Clean);
        assert_eq!(log_bytes(&root), valid, "a clean log is not rewritten");
        assert_store_files(&root, false);
    }
    // `Clean` is the open path's verdict on the same bytes, because the
    // diagnosis *is* the open path's validator.
    let opened = root.open(&slug).unwrap();
    assert_eq!(opened.graph(), &graph);
}

// =================================================================
// Interior corruption (required test 2).
// =================================================================

#[test]
fn interior_corruption_is_refused_at_its_own_offset() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let log_lines = log_lines(&valid);
    // Line 2 is the first event after the seed: an `epic_add`.
    assert!(String::from_utf8_lossy(log_lines[2]).contains("epic_add"));

    // Garbage spliced into a middle line, from the log suite's own fixture
    // bytes: the file's final line stays terminated, so this is §4.3's
    // interior case — the interior of a log does not tear in a crash.
    let garbage: &[u8] = b"{{{ not json\n";
    let mut spliced = Vec::new();
    for (index, line) in log_lines.iter().enumerate() {
        if index == 2 {
            spliced.extend_from_slice(garbage);
        } else {
            spliced.extend_from_slice(line);
        }
    }
    fs::write(log_path(&root), &spliced).unwrap();
    let interior_at = line_offsets(&spliced)[2];
    expect_open_corrupt(&root, interior_at, "not a parseable event");

    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(
            expect_refusal(&outcome, "interior line"),
            Some(interior_at as u64)
        );
        assert_eq!(log_bytes(&root), spliced, "a refusal writes nothing");
        assert_store_files(&root, false);
    }
}

// =================================================================
// Interior damage *and* a torn final line: the state the truncation-point
// check is the only guard against.
// =================================================================

#[test]
fn interior_damage_beside_a_torn_tail_is_refused_at_the_interior_offset() {
    // A log that presents both: garbage in an interior line and a torn final
    // line. Replay stops at the first fault, so the offset repair is handed
    // is the *interior* line's, not the final one's — and the final line
    // really is torn, so a classification that only asked "is the last line
    // unterminated?" would truncate from the interior offset and silently
    // drop every committed event in between (§4's silent loss). The offset
    // comparison is what refuses it.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let log_lines = log_lines(&valid);
    let mut spliced = Vec::new();
    for (index, line) in log_lines.iter().enumerate() {
        if index == 2 {
            spliced.extend_from_slice(GARBAGE_LINE);
        } else {
            spliced.extend_from_slice(line);
        }
    }
    let interior_at = line_offsets(&spliced)[2];
    // The tear, spliced from one of the log's own real event lines.
    let mut both = spliced.clone();
    both.extend_from_slice(&torn_fragment(&spliced, TEAR_BYTES));
    assert_ne!(both.last(), Some(&b'\n'), "the final line is torn");
    fs::write(log_path(&root), &both).unwrap();

    // The open path reports the interior fault — replay stops at the first
    // one — and the tear is what a naive classification would have seen.
    expect_open_corrupt(&root, interior_at, "not a parseable event");
    assert!(interior_at < spliced.len(), "the two faults are distinct");

    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(
            expect_refusal(&outcome, "interior line"),
            Some(interior_at as u64),
            "the interior offset, never the torn line's at {}",
            spliced.len()
        );
        assert_eq!(log_bytes(&root), both, "a refusal writes nothing");
        assert_store_files(&root, false);
    }
}

// =================================================================
// A fault in a final line that was committed, not torn.
// =================================================================

#[test]
fn a_fault_in_a_committed_final_line_is_refused_as_committed() {
    // A terminated final line was committed (§4.1: an append is one whole
    // LF-terminated line), so whatever replay rejects in it is damage and
    // not §4.3's tear — dropping it would lose a committed event. Built by
    // re-appending the log's own seed line, a real serialized line the
    // domain rejects, which is replay's `replay_step` path rather than a
    // parse failure.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let mut committed_tail = valid.clone();
    committed_tail.extend_from_slice(log_lines(&valid)[1]);
    let tail_at = valid.len();
    fs::write(log_path(&root), &committed_tail).unwrap();
    expect_open_corrupt(&root, tail_at, "registers camera twice");

    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(
            expect_refusal(&outcome, "final line is terminated"),
            Some(tail_at as u64)
        );
        assert_eq!(log_bytes(&root), committed_tail, "a refusal writes nothing");
        assert_store_files(&root, false);
    }

    // The same verdict for a final line that does not parse: terminated, so
    // committed, so not a tear either.
    let mut garbage_tail = valid.clone();
    garbage_tail.extend_from_slice(GARBAGE_LINE);
    fs::write(log_path(&root), &garbage_tail).unwrap();
    expect_open_corrupt(&root, tail_at, "not a parseable event");
    let outcome = root.repair(&slug, false).unwrap();
    assert_eq!(
        expect_refusal(&outcome, "final line is terminated"),
        Some(tail_at as u64)
    );
    assert_eq!(log_bytes(&root), garbage_tail);
    assert_store_files(&root, false);
}

// =================================================================
// A tear at byte 0 (required test 6).
// =================================================================

#[test]
fn a_torn_header_is_refused_and_the_log_is_not_left_empty() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    // The log's sole line torn: the header with its LF dropped. Truncating
    // from byte 0 would leave an empty file, which is its own
    // `corrupt_store` — so this is refused, not repaired.
    let header = log_lines(&valid)[0];
    let torn_header = header[..header.len() - 1].to_vec();
    fs::write(log_path(&root), &torn_header).unwrap();
    expect_open_corrupt(&root, 0, "no terminating newline");

    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(expect_refusal(&outcome, "byte 0"), Some(0));
        let after = log_bytes(&root);
        assert_eq!(after, torn_header, "a refusal writes nothing");
        assert!(!after.is_empty(), "the log is not left empty");
        assert_store_files(&root, false);
    }
}

// =================================================================
// §8's single writer (required test 4).
// =================================================================

#[test]
fn a_contended_store_is_refused_with_locked_having_read_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    // The holder of §8's lock: a store opened through the real API.
    let held = build_short_store(&root);
    let graph_held = held.graph().clone();
    let valid = log_bytes(&root);
    // Torn out of band, while the store is held: a repair that skipped the
    // lock would truncate here, so the byte comparison below is the proof
    // that it did not read either.
    let torn_bytes = torn(&valid);
    fs::write(log_path(&root), &torn_bytes).unwrap();

    let error = root.repair(&slug, false).unwrap_err();
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(slug.clone())
        }
    );
    assert_eq!(error.code(), "locked");
    assert_eq!(error.to_string(), "store is locked: repo camera");
    assert!(!error.to_string().contains('/'), "{error} carries no path");
    // A dry run contends identically: the lock precedes the read.
    assert_eq!(root.repair(&slug, true).unwrap_err(), error);
    assert_eq!(
        log_bytes(&root),
        torn_bytes,
        "the refusal read nothing and wrote nothing"
    );
    assert_eq!(
        held.graph(),
        &graph_held,
        "the refusal cost the holder nothing"
    );
    assert_store_files(&root, false);

    // Released, the same tear is repairable — the refusal was the lock, not
    // the fixture.
    drop(held);
    let outcome = root.repair(&slug, true).unwrap();
    assert!(matches!(
        outcome,
        RepairOutcome::TornTail {
            truncated: false,
            ..
        }
    ));
    assert_eq!(log_bytes(&root), torn_bytes);
    assert_store_files(&root, false);
}

// =================================================================
// The snapshot/log pair divergence (required test 5).
// =================================================================

#[test]
fn a_diverged_pair_is_reported_with_its_numbers_and_not_written() {
    // `compact`'s (3)–(4) window, built the way `tests/snapshot/compact.rs`
    // builds it: a covering snapshot beside a shortened log, both from real
    // runs of one deterministic fixture. The twin's real `compact()`
    // supplies the short log, so the window's bytes are the engine's own and
    // not a hand-written pair.
    let slug = camera_slug();

    let cover_dir = TempDir::new().unwrap();
    let cover_root = data_root(&cover_dir);
    let mut cover = build_short_store(&cover_root);
    let full_log = log_bytes(&cover_root);
    let covered = cover.snapshot().unwrap();
    let covering = fs::read(cover_root.snapshot_path(&slug)).unwrap();
    assert_eq!(covered, full_log.len() as u64);
    drop(cover);

    let twin_dir = TempDir::new().unwrap();
    let twin_root = data_root(&twin_dir);
    let mut twin = build_short_store(&twin_root);
    assert_eq!(
        log_bytes(&twin_root),
        full_log,
        "the twins must agree byte-for-byte before compacting"
    );
    let report = twin.compact().unwrap();
    let short_log = log_bytes(&twin_root);
    assert_eq!(report.log_bytes_after, short_log.len() as u64);
    drop(twin);

    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    // A real registered store, then the window's two files written into it.
    short_store_graph(&root);
    fs::write(log_path(&root), &short_log).unwrap();
    fs::write(root.snapshot_path(&slug), &covering).unwrap();
    assert!(
        covered > short_log.len() as u64,
        "the window's premise: the covering offset exceeds the short log"
    );

    // The open path names it, unpositioned: the disagreement is between two
    // files, not at a byte inside one.
    let error = root.open(&slug).unwrap_err();
    match &error {
        StoreError::CorruptStore {
            offset: None,
            detail,
        } => assert!(detail.contains("beyond the log's end"), "{error}"),
        other => panic!("expected an unpositioned corrupt_store, got {other:?}"),
    }

    // Repair reports the pair, with the real numbers, and writes neither
    // file. The remedy — re-basing the snapshot — is not this primitive's.
    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(
            outcome,
            RepairOutcome::Diverged {
                log_offset: covered,
                log_len: short_log.len() as u64
            }
        );
        assert_eq!(log_bytes(&root), short_log, "nothing written");
        assert_eq!(
            fs::read(root.snapshot_path(&slug)).unwrap(),
            covering,
            "nothing written"
        );
        assert_store_files(&root, true);
    }

    // And the divergence *dominates* a tear, which is why it is checked
    // first: the same pair with the short log torn as well still reports
    // `Diverged`, because truncating the log would not make it consistent.
    let torn_short = torn(&short_log);
    fs::write(log_path(&root), &torn_short).unwrap();
    assert_eq!(
        root.repair(&slug, false).unwrap(),
        RepairOutcome::Diverged {
            log_offset: covered,
            log_len: torn_short.len() as u64
        }
    );
    assert_eq!(log_bytes(&root), torn_short, "nothing written");
    assert_store_files(&root, true);
}

// =================================================================
// A fault with no byte position: nothing to truncate from.
// =================================================================

#[test]
fn an_unpositioned_fault_is_refused_for_want_of_a_truncation_point() {
    // A snapshot of ANOTHER repository beside this log: `reconcile_with_log`
    // rejects it as a disagreement between two files, which has no byte
    // position in either, so there is nothing to truncate from. Real bytes
    // and no mutated document — the foreign store is registered and
    // snapshotted through the public API.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let (foreign, foreign_offset) = foreign_snapshot();
    // The premise of the case: the foreign document covers less than this
    // log, so the divergence check does not fire first and the fault reaches
    // the no-position arm.
    assert!(
        foreign_offset < valid.len() as u64,
        "the foreign offset {foreign_offset} must be below the log's {} bytes",
        valid.len()
    );
    fs::write(root.snapshot_path(&slug), &foreign).unwrap();
    expect_open_unpositioned(&root, "the snapshot holds repo encoder");

    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(expect_refusal(&outcome, "no byte position"), None);
        assert_eq!(log_bytes(&root), valid, "a refusal writes nothing");
        assert_store_files(&root, true);
    }
}

// =================================================================
// The two faults that propagate instead of being classified: a format
// this build does not read, and a snapshot document that does not parse.
// =================================================================

#[test]
fn a_foreign_format_version_is_an_error_not_a_refusal() {
    // A log written by a newer build is §4's migration question, not a
    // corruption a repair could classify, so it propagates as the very
    // `version_mismatch` `open` reports.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let newer = with_header_version(&valid, FORMAT_VERSION + 1);
    fs::write(log_path(&root), &newer).unwrap();
    let expected = StoreError::VersionMismatch {
        found: FORMAT_VERSION + 1,
        expected: FORMAT_VERSION,
    };
    assert_eq!(root.open(&slug).unwrap_err(), expected);

    for dry_run in [true, false] {
        let error = root.repair(&slug, dry_run).unwrap_err();
        assert_eq!(error, expected);
        assert_eq!(error.code(), "version_mismatch");
        assert_eq!(log_bytes(&root), newer, "nothing written");
        assert_store_files(&root, false);
    }
}

#[test]
fn an_unparseable_snapshot_propagates_the_error_open_reports() {
    // f31's deviation from the task spec's `Err` sentence, pinned rather than
    // only documented: a snapshot document that does not parse is a store
    // fault repair cannot classify, and its `Err` is **byte-identical to
    // `open`'s** — the agreement this primitive exists for. Truncating a log
    // cannot fix a snapshot, and `Unrepairable.offset` is a log position by
    // contract, so the fault is never reported through a refusal.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    fs::write(root.snapshot_path(&slug), GARBAGE_LINE).unwrap();

    let opened = root.open(&slug).unwrap_err();
    assert_eq!(opened.code(), "corrupt_store");
    assert!(
        opened.to_string().contains("snapshot"),
        "the fault names the file it is in: {opened}"
    );
    for dry_run in [true, false] {
        let error = root.repair(&slug, dry_run).unwrap_err();
        assert_eq!(error, opened);
        assert_eq!(error.to_string(), opened.to_string(), "byte-identical");
        assert_eq!(log_bytes(&root), valid, "nothing written");
        assert_store_files(&root, true);
    }
}

// =================================================================
// The two io faults a repair reports instead of classifying.
// =================================================================

#[test]
fn repairing_a_store_that_is_not_there_reports_the_missing_directory() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();

    for dry_run in [true, false] {
        let error = root.repair(&slug, dry_run).unwrap_err();
        // The same shape `open` reports for a store that was never
        // registered, with the repair's own context: a directory that does
        // not exist holds no lock file either, so this is never a lock fault.
        assert_eq!(
            error,
            StoreError::Io {
                detail: "repair the event log: NotFound".to_owned()
            }
        );
        assert_eq!(error.code(), "io");
        assert!(!error.to_string().contains('/'), "{error} carries no path");
    }
    // A repair creates nothing: no store directory, no lock file, no
    // registry.
    assert!(!root.repos_dir().exists());
    assert_eq!(dir_entries(root.root()), Vec::<String>::new());
}

#[test]
fn a_store_whose_log_is_missing_reports_the_read_fault() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    fs::remove_file(log_path(&root)).unwrap();

    let error = root.repair(&slug, false).unwrap_err();
    assert_eq!(
        error,
        StoreError::Io {
            detail: "read the event log for repair: NotFound".to_owned()
        }
    );
    assert_eq!(error.code(), "io");
    // §8's order is lock-then-read, so the lock file exists: the accepted
    // debris of a failed attempt, whose bytes mean nothing (the `lock`
    // module documents it). The log is not recreated.
    assert_eq!(dir_entries(&store_dir(&root)), ["event-log.jsonl.lock"]);
}

// =================================================================
// A refusal names §4.3's backup, whatever the case (the remedy half of
// every `Unrepairable` detail).
// =================================================================

#[test]
fn every_refusal_names_the_backup_remedy() {
    // Two of the refusal cases, one assertion each on the remedy text: the
    // report has to carry the way out, not only the verdict.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);

    let header = log_lines(&valid)[0];
    fs::write(log_path(&root), &header[..header.len() - 1]).unwrap();
    let outcome = root.repair(&slug, true).unwrap();
    let RepairOutcome::Unrepairable { detail, .. } = &outcome else {
        panic!("expected a refusal, got {outcome:?}");
    };
    assert!(
        detail.contains("copy the store directory while no writer holds the lock"),
        "{detail}"
    );

    let log_lines = log_lines(&valid);
    let mut spliced = Vec::new();
    for (index, line) in log_lines.iter().enumerate() {
        if index == 2 {
            spliced.extend_from_slice(b"{{{ not json\n");
        } else {
            spliced.extend_from_slice(line);
        }
    }
    fs::write(log_path(&root), &spliced).unwrap();
    let outcome = root.repair(&slug, false).unwrap();
    let RepairOutcome::Unrepairable { detail, .. } = &outcome else {
        panic!("expected a refusal, got {outcome:?}");
    };
    assert!(detail.contains("§4.3's backup"), "{detail}");
    assert_eq!(log_bytes(&root), spliced);
    assert_store_files(&root, false);
}

// =================================================================
// The store directory is the only thing a repair consults: the registry
// is not, exactly as in `DataRoot::open`.
// =================================================================

#[test]
fn a_repair_does_not_consult_the_registry() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    // The registry index removed out of band: a store the registry does not
    // list is the `layout` module's documented divergence — reported by an
    // inspector, never a refusal to repair the store that *is* there.
    fs::remove_file(root.registry_path()).unwrap();
    let valid = log_bytes(&root);
    let torn_bytes = torn(&valid);
    fs::write(log_path(&root), &torn_bytes).unwrap();

    let outcome = root.repair(&slug, false).unwrap();
    let report = expect_torn(&outcome, true);
    assert_eq!(report.offset, valid.len() as u64);
    assert_eq!(log_bytes(&root), valid);
    assert!(
        !root.registry_path().exists(),
        "a repair neither reads nor rewrites the registry"
    );
    assert_store_files(&root, false);
}
