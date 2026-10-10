//! The `verify` report, end to end: a clean store (counts, sizes,
//! `status: ok`, exit 0); a torn tail (code `corrupt_store`, the offset,
//! the `TornTail` classification with its preview, exit 1); interior
//! damage (the `Unrepairable` detail, exit 1); a diverged pair (both
//! numbers, exit 1); contended (`locked`, exit 3, and **no** repair
//! classification attempted); a missing store directory (`io`, exit 3); a
//! foreign header version (`version_mismatch`, exit 3); an unparseable
//! snapshot (exit 1, "classification: unavailable"); and the stream
//! discipline (report on stdout, faults on stderr).
//!
//! Plus §13's resolution pins, end to end through the spawned binary:
//! the cwd tier (no `--repo`, the working directory inside a registered
//! checkout — a real-path fixture, so the match has something real to
//! resolve — answers that repo's report, repair's tier included by the
//! signed sentence) and the resolution failure (no `--repo`, a cwd
//! matching nothing → exit 1 with the resolve message, the
//! progress/refresh precedent — an environment fault, not the retired
//! cli's exit-2 missing-option fault).
//!
//! Every case spawns the built binary and asserts exit code and
//! stdout/stderr; the fixtures are the shared ones (real API, fixed
//! timestamps, byte-spliced tears).

use std::fs;

use tempfile::TempDir;
use todo_store::{FORMAT_VERSION, Store};

use crate::inspect::fixtures::{
    GARBAGE_LINE, assert_no_debris, at, build_short_store, camera_slug, data_root, diverged_pair,
    log_bytes, log_path, root_arg, run_cli, run_cli_cwd, short_store_bytes, torn,
    with_header_version, with_interior_garbage,
};

fn verify(dir: &TempDir) -> crate::inspect::fixtures::CliRun {
    run_cli(&["verify", "--root", root_arg(dir), "--repo", "camera"])
}

#[test]
fn a_clean_store_reports_its_counts_sizes_and_ok() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);

    let run = verify(&dir);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stderr, "");
    // The sizes are stats, not a second reading of the format, and
    // the report is line-oriented and total — no line count (f25's
    // parked boundary), no clock, no host path.
    assert_eq!(
        run.stdout,
        format!(
            "slug: camera\n\
             epics: 1\n\
             tasks: 2\n\
             decisions: 0\n\
             docs: 0\n\
             blocks: 1\n\
             relates: 0\n\
             log_bytes: {}\n\
             snapshot_bytes: none\n\
             status: ok\n",
            valid.len()
        )
    );
    assert_no_debris(&root);
}

#[test]
fn a_clean_store_with_a_snapshot_reports_its_size() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let mut store = build_short_store(&root);
    store.snapshot().unwrap();
    let snapshot_len = fs::metadata(root.snapshot_path(&camera_slug()))
        .unwrap()
        .len();
    drop(store);

    let run = verify(&dir);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(
        run.stdout
            .contains(&format!("snapshot_bytes: {snapshot_len}\n")),
        "{:?}",
        run.stdout
    );
    assert!(run.stdout.ends_with("status: ok\n"));
    assert_no_debris(&root);
}

#[test]
fn a_torn_tail_is_reported_with_its_offset_and_classification() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);
    let torn_bytes = torn(&valid);
    fs::write(log_path(&root), &torn_bytes).unwrap();

    let run = verify(&dir);
    assert_eq!(run.code, 1, "a faulty store is the found-a-fault exit");
    assert_eq!(run.stdout, "", "faults go to stderr");
    // §6.1's code on its own line, first.
    assert!(
        run.stderr.starts_with("corrupt_store\n"),
        "{:?}",
        run.stderr
    );
    // The offset, with the plain statement of what an offset can
    // mean (a snapshot-document offset is not a log byte — f31).
    assert!(
        run.stderr.contains(&format!("offset: {}\n", valid.len())),
        "{:?}",
        run.stderr
    );
    assert!(run.stderr.contains("not a log byte"), "{:?}", run.stderr);
    // The engine's own detail, then the store's classification via
    // the dry run: the TornTail with its offset, removed bytes and
    // bounded preview line.
    assert!(
        run.stderr.contains("no terminating newline"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("classification: torn_tail\n"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains(&format!("truncate_at: {}\n", valid.len())),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains(&format!(
            "bytes_removed: {}\n",
            torn_bytes.len() - valid.len()
        )),
        "{:?}",
        run.stderr
    );
    let fragment = String::from_utf8_lossy(&torn_bytes[valid.len()..]);
    assert!(
        run.stderr.contains(&format!("torn_line: {fragment}\n")),
        "{:?}",
        run.stderr
    );
    assert_eq!(log_bytes(&root), torn_bytes, "verify writes nothing");
    assert_no_debris(&root);
}

#[test]
fn interior_damage_is_reported_unrepairable_with_the_backup_remedy() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);
    // Line 2 is the first event after the seed: an `epic_add`.
    let (spliced, interior_at) = with_interior_garbage(&valid, 2);
    fs::write(log_path(&root), &spliced).unwrap();

    let run = verify(&dir);
    assert_eq!(run.code, 1);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.starts_with("corrupt_store\n"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains(&format!("offset: {interior_at}\n")),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("classification: unrepairable\n"),
        "{:?}",
        run.stderr
    );
    // The refusal detail is the store's contract text, printed
    // verbatim: the case, the engine's report and §4.3's backup.
    assert!(run.stderr.contains("interior line"), "{:?}", run.stderr);
    assert!(
        run.stderr.contains(
            "the remedy is §4.3's backup: copy the store directory \
             while no writer holds the lock"
        ),
        "{:?}",
        run.stderr
    );
    assert_eq!(log_bytes(&root), spliced, "verify writes nothing");
    assert_no_debris(&root);
}

#[test]
fn a_diverged_pair_is_reported_with_both_numbers() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    short_store_bytes(&root);
    let (short_log, covering, covered) = diverged_pair();
    fs::write(log_path(&root), &short_log).unwrap();
    fs::write(root.snapshot_path(&camera_slug()), &covering).unwrap();

    let run = verify(&dir);
    assert_eq!(run.code, 1);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.starts_with("corrupt_store\n"),
        "{:?}",
        run.stderr
    );
    // The open's own fault is unpositioned — a disagreement between
    // two files has no byte position in either — so no offset line
    // (and no snapshot-offset caveat) between the code and detail.
    assert!(
        run.stderr.starts_with("corrupt_store\ndetail: "),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("beyond the log's end"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("classification: diverged\n"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains(&format!("snapshot_log_offset: {covered}\n")),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains(&format!("log_bytes: {}\n", short_log.len())),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("re-base the snapshot"),
        "{:?}",
        run.stderr
    );
    assert_eq!(log_bytes(&root), short_log, "verify writes nothing");
    assert_no_debris(&root);
}

#[test]
fn a_contended_store_is_locked_with_no_classification_attempted() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    // §8's single writer, held in-process: the spawned binary's open
    // must refuse with `locked` — and must not then attempt a repair
    // dry run, which would contend on the same lock.
    let held = build_short_store(&root);

    let run = verify(&dir);
    assert_eq!(run.code, 3, "the store refused the operation");
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("locked\n"), "{:?}", run.stderr);
    assert!(
        run.stderr.contains("store is locked: repo camera"),
        "{:?}",
        run.stderr
    );
    assert!(
        !run.stderr.contains('/'),
        "the locked report carries no path: {:?}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("classification"),
        "no repair dry run against a contended lock: {:?}",
        run.stderr
    );

    drop(held);
    assert_no_debris(&root);
}

#[test]
fn a_missing_store_directory_is_the_honest_io_fault() {
    let dir = TempDir::new().unwrap();
    // Nothing registered: the engine's NotFound, never a panic and
    // never a guess (spec verify step 5).
    let run = verify(&dir);
    assert_eq!(run.code, 3);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("io\n"), "{:?}", run.stderr);
    assert!(
        run.stderr.contains("open the event log: NotFound"),
        "{:?}",
        run.stderr
    );
    assert!(!dir.path().join("repos").exists(), "verify creates nothing");
}

#[test]
fn an_unparseable_snapshot_document_is_reported_with_no_classification() {
    // t22's f31 deviation: a snapshot document that does not parse is
    // a store fault repair cannot classify — truncating a log cannot
    // fix a snapshot — so verify reports the faulty store (exit 1,
    // its numbered step 4's `Err(CorruptStore)` arm) and says plainly
    // that the classification is unavailable instead of guessing. The
    // fault's offset is a position in the snapshot's own bytes:
    // exactly what the note line under it states.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);
    fs::write(root.snapshot_path(&camera_slug()), GARBAGE_LINE).unwrap();

    let run = verify(&dir);
    assert_eq!(run.code, 1, "a faulty store is the found-a-fault exit");
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.starts_with("corrupt_store\n"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains("the snapshot is not a parseable document"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("classification: unavailable\n"),
        "{:?}",
        run.stderr
    );
    assert_eq!(log_bytes(&root), valid, "verify writes nothing");
    assert_no_debris(&root);
}

#[test]
fn a_foreign_header_version_is_the_migration_question() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);
    let newer = with_header_version(&valid, FORMAT_VERSION + 1);
    fs::write(log_path(&root), &newer).unwrap();

    let run = verify(&dir);
    assert_eq!(run.code, 3);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.starts_with("version_mismatch\n"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains(&format!(
            "store format version {}, this build reads version {}",
            FORMAT_VERSION + 1,
            FORMAT_VERSION
        )),
        "{:?}",
        run.stderr
    );
    assert_eq!(log_bytes(&root), newer, "verify writes nothing");
    assert_no_debris(&root);
}

// =================================================================
// §13's resolution pins: the cwd tier and the resolution failure.
// =================================================================

/// The cwd-tier pin (§13's "the same resolution order as the server",
/// D2 — repair included): an inspector run with no `--repo`, spawned
/// with its working directory inside a registered checkout, answers
/// that repo's report. The fixture is this test's own (judgment call
/// (c)): the repo's path is a real tempdir — the golden world's fixed
/// literal path never mixes with a resolution fixture — moved onto the
/// built store through the store's own `set_repo_path` so the builder
/// stays the shared one.
#[test]
fn no_repo_flag_with_cwd_inside_a_checkout_answers_that_repo() {
    let dir = TempDir::new().unwrap();
    let checkout = TempDir::new().unwrap();
    let root = data_root(&dir);
    // The physical spelling: what the OS hands the spawned binary as
    // its cwd, and what the match canonicalizes both sides to.
    let checkout_at = fs::canonicalize(checkout.path()).unwrap();
    let mut store = build_short_store(&root);
    store
        .set_repo_path(
            checkout_at.display().to_string(),
            at("2026-09-25T16:00:00Z"),
        )
        .unwrap();
    drop(store);

    let run = run_cli_cwd(&["verify", "--root", root_arg(&dir)], &checkout_at);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stderr, "");
    assert!(
        run.stdout.starts_with("slug: camera\n"),
        "the cwd tier answered camera: {:?}",
        run.stdout
    );
    assert!(run.stdout.contains("epics: 1\n"));
    assert!(run.stdout.contains("tasks: 2\n"));
    assert!(run.stdout.ends_with("status: ok\n"));
    assert_no_debris(&root);
}

/// The resolution-failure pin (veto-able iii): no `--repo`, a working
/// directory matching nothing — over a root whose registry holds a
/// repo, so the walk ran and the match is what failed — is exit 1 with
/// the resolve message on stderr (the progress/refresh precedent: the
/// option exists, the resolution failed — an environment fault, not
/// the retired cli's exit-2 missing-option fault).
#[test]
fn no_repo_flag_with_a_cwd_matching_nothing_is_exit_1_with_the_resolve_message() {
    let dir = TempDir::new().unwrap();
    let elsewhere = TempDir::new().unwrap();
    let root = data_root(&dir);
    drop(build_short_store(&root));

    let run = run_cli_cwd(&["verify", "--root", root_arg(&dir)], elsewhere.path());
    assert_eq!(run.code, 1, "an environment fault, not a usage fault");
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains(
            "todo-mcp: no --repo and the working directory matches no registered checkout"
        ),
        "{:?}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("Usage:"),
        "a resolution failure is not a usage fault: {:?}",
        run.stderr
    );
    assert_no_debris(&root);
}
