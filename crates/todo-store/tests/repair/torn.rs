//! §4.3's repairable case: an unterminated final line is shown by a dry
//! run, removed by a real one, and the log that results is byte-equal to
//! the one the crash would have left had the write completed — which is
//! what makes the store open again to its pre-tear graph. Also here: the
//! bounded preview, twice-run determinism, the debris scan, and the
//! refusal of a tear whose candidate prefix would not itself replay.

use std::fs;

use tempfile::TempDir;
use todo_store::{JsonlStore, RepairOutcome, Store};

use super::fixtures::{
    PREVIEW_BOUND, TEAR_BYTES, assert_store_files, camera_slug, data_root, dir_entries,
    expect_open_corrupt, expect_refusal, expect_torn, final_line_offset, log_bytes, log_lines,
    log_path, long_tail_graph, refusal_detail, short_store_graph, tear_append, torn, torn_fragment,
};

// =================================================================
// The torn tail: dry run, then the real thing (required test 1).
// =================================================================

#[test]
fn a_torn_tail_is_shown_by_a_dry_run_and_removed_by_a_real_one() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    let graph_before = short_store_graph(&root);
    // The pre-tear copy: what a successful repair must restore, byte for
    // byte, because the tear is an event that was never committed.
    let valid = log_bytes(&root);
    let fragment = torn_fragment(&valid, TEAR_BYTES);
    let torn_bytes = tear_append(&valid, TEAR_BYTES);
    let torn_at = valid.len();
    assert_eq!(
        final_line_offset(&torn_bytes),
        torn_at,
        "the appended fragment is the log's final line"
    );
    fs::write(log_path(&root), &torn_bytes).unwrap();

    // The open path's own report first: a positioned `corrupt_store` at the
    // torn line's start. Repair must agree with it, and does by
    // construction — it runs the same validator.
    expect_open_corrupt(&root, torn_at, "no terminating newline");

    // A dry run shows the tear and writes nothing.
    let outcome = root.repair(&slug, true).unwrap();
    let report = expect_torn(&outcome, false);
    assert_eq!(report.offset, torn_at as u64);
    assert_eq!(report.removed, fragment.len() as u64);
    // The short store's final line is under the preview bound, so the text
    // is the whole torn fragment (the bounded case has its own test below).
    assert_eq!(report.line, String::from_utf8_lossy(&fragment).into_owned());
    assert_eq!(log_bytes(&root), torn_bytes, "a dry run writes nothing");
    assert_store_files(&root, false);

    // The real run keeps exactly the promise the dry run made.
    let outcome = root.repair(&slug, false).unwrap();
    assert_eq!(expect_torn(&outcome, true), report);
    assert_eq!(log_bytes(&root), valid, "byte-equal to the pre-tear log");

    // And the store opens, to the pre-tear graph.
    let reopened = root.open(&slug).unwrap();
    assert_eq!(reopened.graph(), &graph_before);
    drop(reopened);
    assert_store_files(&root, false);
}

// =================================================================
// The bounded preview (required test 7).
// =================================================================

#[test]
fn a_torn_line_longer_than_the_bound_is_cut_at_it() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    long_tail_graph(&root);
    let valid = log_bytes(&root);
    let fragment = torn_fragment(&valid, TEAR_BYTES);
    let torn_bytes = tear_append(&valid, TEAR_BYTES);
    let torn_at = valid.len();
    fs::write(log_path(&root), &torn_bytes).unwrap();
    expect_open_corrupt(&root, torn_at, "no terminating newline");

    let outcome = root.repair(&slug, true).unwrap();
    let report = expect_torn(&outcome, false);
    let whole = String::from_utf8_lossy(&fragment);
    assert!(
        whole.chars().count() > PREVIEW_BOUND,
        "the fixture must exceed the bound, and {} characters do",
        whole.chars().count()
    );
    // Cut at the bound, on a character boundary, and marked.
    assert_eq!(report.line.chars().count(), PREVIEW_BOUND + 1);
    assert_eq!(report.line.chars().last(), Some('…'));
    let head: String = report.line.chars().take(PREVIEW_BOUND).collect();
    assert!(
        whole.starts_with(&head),
        "the preview is the line's own head"
    );
    // Only the preview is bounded: the offset and the byte count describe
    // the whole tear.
    assert_eq!(report.offset, torn_at as u64);
    assert_eq!(report.removed, fragment.len() as u64);
    assert!(report.removed > PREVIEW_BOUND as u64);
    assert_eq!(log_bytes(&root), torn_bytes, "a dry run writes nothing");
    assert_store_files(&root, false);
}

// =================================================================
// Twice-run determinism (required test 8).
// =================================================================

#[test]
fn the_same_tear_twice_is_deterministic_and_a_second_repair_is_clean() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let torn_bytes = torn(&valid);
    fs::write(log_path(&root), &torn_bytes).unwrap();

    // The same fixture repaired twice, dry both times: equal values, and
    // still nothing written.
    let first = root.repair(&slug, true).unwrap();
    let second = root.repair(&slug, true).unwrap();
    assert_eq!(first, second);
    assert_eq!(log_bytes(&root), torn_bytes);

    // A real repair, then a second call: the log now replays clean, so the
    // answer is `Clean` — under a real run and a dry one alike.
    let truncated = root.repair(&slug, false).unwrap();
    assert!(matches!(
        truncated,
        RepairOutcome::TornTail {
            truncated: true,
            ..
        }
    ));
    assert_eq!(log_bytes(&root), valid);
    assert_eq!(root.repair(&slug, false).unwrap(), RepairOutcome::Clean);
    assert_eq!(root.repair(&slug, true).unwrap(), RepairOutcome::Clean);
    assert_store_files(&root, false);
}

// =================================================================
// No debris (required test 9, the repair half; every other case runs the
// same scan through `assert_store_files`).
// =================================================================

#[test]
fn a_repair_leaves_no_temp_debris() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    fs::write(log_path(&root), torn(&valid)).unwrap();

    assert_store_files(&root, false);
    root.repair(&slug, true).unwrap();
    // A dry run stages no temp file at all.
    assert_store_files(&root, false);
    root.repair(&slug, false).unwrap();
    // The real run's temp was consumed by its rename.
    assert_store_files(&root, false);
    root.repair(&slug, false).unwrap();
    // A clean no-op neither.
    assert_store_files(&root, false);
    // Nothing beside the store directory either: the registry's own two
    // files and `repos/`.
    assert_eq!(
        dir_entries(root.root()),
        ["registry.json", "registry.json.lock", "repos"]
    );
}

// =================================================================
// A tear whose candidate prefix would not open (the case the prefix
// validation of §8's atomic-rejected-write discipline exists for).
// =================================================================

#[test]
fn a_tear_whose_prefix_would_not_open_is_refused() {
    // The seed line torn: the fault is positioned at the seed, the seed *is*
    // the final line, and it is unterminated — so the shape is a torn tail.
    // But the candidate prefix is the header alone, which is its own
    // `corrupt_store` ("the log holds no events"), so truncating would leave
    // a log that does not open. The prefix is validated before anything is
    // written, and a dry run runs the same validation: both refuse, and the
    // log is untouched.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    short_store_graph(&root);
    let valid = log_bytes(&root);
    let log_lines = log_lines(&valid);
    let seed_at = log_lines[0].len();
    let mut torn_seed = log_lines[0].to_vec();
    torn_seed.extend_from_slice(&log_lines[1][..log_lines[1].len() - 1 - TEAR_BYTES]);
    fs::write(log_path(&root), &torn_seed).unwrap();
    expect_open_corrupt(&root, seed_at, "no terminating newline");

    for dry_run in [true, false] {
        let outcome = root.repair(&slug, dry_run).unwrap();
        assert_eq!(
            expect_refusal(&outcome, "would not leave a log that opens"),
            Some(seed_at as u64)
        );
        // The honest reason rides in the report: the prefix's own fault.
        assert!(
            refusal_detail(&outcome).contains("holds no events"),
            "{outcome:?}"
        );
        assert_eq!(log_bytes(&root), torn_seed, "a refusal writes nothing");
        assert_store_files(&root, false);
    }

    // And the prefix a truncation would have left — the header alone — is a
    // log `open` rejects, which is what makes the refusal the honest answer
    // rather than a policy choice. Opened by bare path in its own directory,
    // the log suite's shape, so the store directory stays as asserted.
    let candidate_dir = TempDir::new().unwrap();
    let candidate = candidate_dir.path().join("candidate.jsonl");
    fs::write(&candidate, &torn_seed[..seed_at]).unwrap();
    let error = JsonlStore::open_log(&candidate).unwrap_err();
    assert_eq!(error.code(), "corrupt_store");
    assert!(error.to_string().contains("holds no events"), "{error}");
}
