//! The `repair` command, end to end: `--dry-run` prints the torn line and
//! writes nothing (byte-compare); `--yes` truncates, the log is byte-equal
//! to the pre-tear copy, and the following re-open reports clean (exit 0);
//! no `--yes` with `n` piped to stdin aborts and writes nothing (exit 1);
//! no `--yes` with `y` piped repairs (exit 0); EOF on stdin aborts;
//! `Unrepairable` exits 1 having written nothing; `Clean` exits 0 saying
//! nothing to repair; `--dry-run` on a `Diverged` pair reports and writes
//! nothing.
//!
//! Every case spawns the built binary — the prompt is a stdin/stdout
//! contract, so the process *is* the subject — and ends with the debris
//! scan (required test 6). The consent rule itself (`y`/`yes`,
//! case-insensitive, trimmed) is pure logic and goes through the lib
//! target. The prompt's bytes — including the `repo:` first line D2
//! added, so a human confirming a truncation sees which store a
//! cwd-derived resolution picked — are pinned by the piped-`y` case.

use std::fs;

use tempfile::TempDir;
use todo_mcp::inspect::repair::consent;
use todo_store::Store;

use crate::inspect::fixtures::{
    GARBAGE_LINE, assert_no_debris, at, build_short_store, camera_slug, data_root, diverged_pair,
    log_bytes, log_path, root_arg, run_cli, run_cli_cwd, run_cli_cwd_stdin, run_cli_stdin,
    short_store_bytes, torn, with_interior_garbage,
};

fn repair(dir: &TempDir, extra: &[&str]) -> crate::inspect::fixtures::CliRun {
    let mut args = vec!["repair", "--root", root_arg(dir), "--repo", "camera"];
    args.extend_from_slice(extra);
    run_cli(&args)
}

fn repair_stdin(dir: &TempDir, stdin: &str) -> crate::inspect::fixtures::CliRun {
    run_cli_stdin(
        &["repair", "--root", root_arg(dir), "--repo", "camera"],
        stdin,
    )
}

/// A short store with §4.3's torn tail on disk, and the clean bytes a
/// successful repair must restore: `(valid, torn)`.
fn torn_world(root: &todo_store::DataRoot) -> (Vec<u8>, Vec<u8>) {
    let valid = short_store_bytes(root);
    let torn_bytes = torn(&valid);
    fs::write(log_path(root), &torn_bytes).unwrap();
    (valid, torn_bytes)
}

// =================================================================
// The consent rule — pure logic, lib target.
// =================================================================

#[test]
fn consent_is_y_or_yes_trimmed_and_case_insensitive() {
    for answer in ["y", "Y", " y ", "yes", "YES", "Yes", "\tyes\n"] {
        assert!(consent(answer), "{answer:?} consents");
    }
    for answer in ["", " ", "n", "N", "no", "yeah", "yessir", "y n", "\n"] {
        assert!(!consent(answer), "{answer:?} refuses");
    }
}

// =================================================================
// `--dry-run`: the facts, and not one byte written.
// =================================================================

#[test]
fn a_dry_run_prints_the_torn_line_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let (valid, torn_bytes) = torn_world(&root);

    let run = repair(&dir, &["--dry-run"]);
    assert_eq!(run.code, 0, "a dry run did its job: it looked");
    assert_eq!(run.stdout, "", "a fault report never rides stdout");
    assert!(
        run.stderr.starts_with("classification: torn_tail\n"),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains(&format!("truncate_at: {}\n", valid.len())),
        "{:?}",
        run.stderr
    );
    let fragment = String::from_utf8_lossy(&torn_bytes[valid.len()..]);
    assert!(
        run.stderr.contains(&format!("torn_line: {fragment}\n")),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("no change written\n"),
        "{:?}",
        run.stderr
    );
    assert!(!run.stderr.contains("[y/N]"), "a dry run never prompts");
    assert_eq!(log_bytes(&root), torn_bytes, "byte-compare: nothing");
    assert_no_debris(&root);
}

#[test]
fn dry_run_wins_over_yes_and_writes_nothing() {
    // `run`'s guard order is load-bearing: both flags together classify
    // and write nothing, because `--dry-run` is the operator explicitly
    // disclaiming the write `--yes` would skip the prompt for. The
    // parse level (the lib's args tests) pins only that both flags are
    // accepted; this pins the combination against a torn store — the
    // swapped guards truncate here.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let (valid, torn_bytes) = torn_world(&root);

    // EOF stdin: a prompt, if one were reached, would abort — so exit
    // 0 with the dry-run report already proves no prompt was read.
    let run = repair(&dir, &["--dry-run", "--yes"]);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stdout, "", "a dry run reports no no-fault outcome");
    assert!(
        run.stderr.starts_with("classification: torn_tail\n"),
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
        run.stderr.contains("no change written\n"),
        "{:?}",
        run.stderr
    );
    assert!(!run.stderr.contains("[y/N]"), "a dry run never prompts");
    assert_eq!(
        log_bytes(&root),
        torn_bytes,
        "the combination writes nothing: byte-compare"
    );
    assert_no_debris(&root);
}

#[test]
fn a_dry_run_on_a_diverged_pair_reports_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    short_store_bytes(&root);
    let (short_log, covering, covered) = diverged_pair();
    fs::write(log_path(&root), &short_log).unwrap();
    fs::write(root.snapshot_path(&camera_slug()), &covering).unwrap();

    let run = repair(&dir, &["--dry-run"]);
    assert_eq!(run.code, 1, "a refusal is a fault not acted on");
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.starts_with("classification: diverged\n"),
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
    assert_eq!(log_bytes(&root), short_log, "nothing written");
    assert_eq!(
        fs::read(root.snapshot_path(&camera_slug())).unwrap(),
        covering,
        "nothing written"
    );
    assert_no_debris(&root);
}

// =================================================================
// The act: `--yes`, piped `y`, and what each leaves behind.
// =================================================================

#[test]
fn yes_truncates_to_the_pre_tear_bytes_and_reopens_clean() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let (valid, _) = torn_world(&root);

    let run = repair(&dir, &["--yes"]);
    assert_eq!(run.code, 0, "repaired is the did-its-job exit");
    // The post-repair re-open's verdict is stdout's good news.
    assert_eq!(run.stdout, "the store now opens clean\n");
    assert!(
        run.stderr
            .contains(&format!("truncate_at: {}\n", valid.len())),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("classification: torn_tail\n"),
        "{:?}",
        run.stderr
    );
    assert!(!run.stderr.contains("[y/N]"), "--yes skips the prompt");
    // The log is byte-equal to the pre-tear copy: the truncation
    // removed exactly the torn line and nothing else.
    assert_eq!(log_bytes(&root), valid);
    assert_no_debris(&root);
}

#[test]
fn a_piped_y_repairs_like_yes() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let (valid, torn_bytes) = torn_world(&root);

    let run = repair_stdin(&dir, "y\n");
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stdout, "the store now opens clean\n");
    // The prompt's first line names the repo (D2): a human confirming
    // a truncation sees which store the resolution picked — the one
    // deliberate prompt-bytes change of the absorption, pinned here.
    assert!(
        run.stderr.starts_with("repo: camera\n"),
        "the prompt names the repo: {:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains(&format!(
            "torn line at offset {} ({} bytes):",
            valid.len(),
            torn_bytes.len() - valid.len()
        )),
        "{:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains("truncate it, losing exactly this one in-flight event? [y/N]"),
        "{:?}",
        run.stderr
    );
    assert_eq!(log_bytes(&root), valid);
    assert_no_debris(&root);
}

#[test]
fn anything_but_consent_aborts_with_nothing_written() {
    for answer in ["n\n", "no\n", "", "\n", "maybe\n"] {
        let dir = TempDir::new().unwrap();
        let root = data_root(&dir);
        let (_, torn_bytes) = torn_world(&root);

        let run = repair_stdin(&dir, answer);
        assert_eq!(run.code, 1, "aborted: {answer:?}");
        assert_eq!(run.stdout, "");
        assert!(run.stderr.ends_with("aborted\n"), "{:?}", run.stderr);
        assert_eq!(
            log_bytes(&root),
            torn_bytes,
            "an abort writes nothing: {answer:?}"
        );
        assert_no_debris(&root);
    }
}

#[test]
fn eof_on_stdin_aborts() {
    // The default runner's stdin is null: the EOF case, and the one
    // a script that forgot to pipe an answer gets — the safe arm.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let (_, torn_bytes) = torn_world(&root);

    let run = repair(&dir, &[]);
    assert_eq!(run.code, 1);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.ends_with("aborted\n"), "{:?}", run.stderr);
    assert_eq!(log_bytes(&root), torn_bytes);
    assert_no_debris(&root);
}

// =================================================================
// The refusals and the no-op.
// =================================================================

#[test]
fn an_unrepairable_fault_exits_1_having_written_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);
    let (spliced, _) = with_interior_garbage(&valid, 2);
    fs::write(log_path(&root), &spliced).unwrap();

    // Both flag paths refuse identically — and the bare path never
    // reaches a prompt, so its EOF stdin cannot matter.
    for run in [repair(&dir, &[]), repair(&dir, &["--yes"])] {
        assert_eq!(run.code, 1);
        assert_eq!(run.stdout, "");
        assert!(
            run.stderr.starts_with("classification: unrepairable\n"),
            "{:?}",
            run.stderr
        );
        // The detail verbatim: the store's contract text (f33).
        assert!(
            run.stderr
                .contains("refusal: the fault is in an interior line"),
            "{:?}",
            run.stderr
        );
        assert!(
            run.stderr.contains("the remedy is §4.3's backup"),
            "{:?}",
            run.stderr
        );
        assert!(!run.stderr.contains("[y/N]"), "a refusal never prompts");
        assert_eq!(log_bytes(&root), spliced, "a refusal writes nothing");
    }
    assert_no_debris(&root);
}

#[test]
fn a_clean_store_exits_0_saying_nothing_to_repair() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);

    for extra in [Vec::new(), vec!["--dry-run"], vec!["--yes"]] {
        let run = repair(&dir, &extra);
        assert_eq!(run.code, 0, "\"nothing to repair\" did the job: {extra:?}");
        assert_eq!(run.stdout, "nothing to repair\n");
        assert_eq!(run.stderr, "");
        assert_eq!(log_bytes(&root), valid);
    }
    assert_no_debris(&root);
}

// =================================================================
// The store's own refusals of the operation: exit-code table's 3.
// =================================================================

#[test]
fn an_unparseable_snapshot_document_is_the_store_refusing() {
    // f31's Err shape: a snapshot document that does not parse is a
    // fault repair cannot classify — it propagates rather than
    // becoming an outcome — so every flag path exits the table's 3
    // with §6.1's code on its own line and nothing written.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let valid = short_store_bytes(&root);
    fs::write(root.snapshot_path(&camera_slug()), GARBAGE_LINE).unwrap();

    for extra in [Vec::new(), vec!["--dry-run"], vec!["--yes"]] {
        let run = repair(&dir, &extra);
        assert_eq!(run.code, 3, "{extra:?}");
        assert_eq!(run.stdout, "");
        assert!(
            run.stderr.starts_with("corrupt_store\n"),
            "{extra:?}: {:?}",
            run.stderr
        );
        assert!(
            run.stderr
                .contains("the snapshot is not a parseable document"),
            "{extra:?}: {:?}",
            run.stderr
        );
        assert_eq!(log_bytes(&root), valid, "nothing written: {extra:?}");
    }
    assert_no_debris(&root);
}

#[test]
fn a_missing_store_directory_is_the_honest_io_fault() {
    let dir = TempDir::new().unwrap();
    let run = repair(&dir, &["--dry-run"]);
    assert_eq!(run.code, 3);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("io\n"), "{:?}", run.stderr);
    assert!(
        run.stderr.contains("repair the event log: NotFound"),
        "{:?}",
        run.stderr
    );
    assert!(!dir.path().join("repos").exists(), "repair creates nothing");
}

// =================================================================
// §13's resolution pins on repair's own dispatch (D2, veto-able (i)
// and (iii)): the cwd tier reaches the prompt — the confirming human
// sees which store the tier picked — and the resolution failure is
// repair's exit-1 environment fault, not a usage fault. Every other
// case in this file passes `--repo` explicitly; these two drive the
// tier the duplicated resolution block in `main.rs`'s `run_repair`
// owns, so a flag-only mutant of it cannot survive.
// =================================================================

/// The cwd-tier pin for repair (§13's "the same resolution order as
/// the server", D2 — repair included): a torn store registered at a
/// real-path checkout, no `--repo`, the working directory inside it —
/// the prompt's first line names the cwd-derived repo, and consent
/// truncates THAT store. The fixture is this test's own (judgment
/// call (c)): the repo's path is a real tempdir, moved onto the built
/// store through `set_repo_path` so the builder stays the shared one.
#[test]
fn no_repo_flag_with_cwd_inside_a_checkout_prompts_and_repairs_that_repo() {
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
    // The torn tail on the cwd-resolved store: the clean prefix a
    // consented repair must restore.
    let valid = log_bytes(&root);
    let torn_bytes = torn(&valid);
    fs::write(log_path(&root), &torn_bytes).unwrap();

    let run = run_cli_cwd_stdin(&["repair", "--root", root_arg(&dir)], &checkout_at, "y\n");
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stdout, "the store now opens clean\n");
    // The prompt's first line names the repo the cwd tier picked —
    // D2's rationale end-to-end: the confirming human sees which
    // store a cwd-derived resolution chose.
    assert!(
        run.stderr.starts_with("repo: camera\n"),
        "the prompt names the cwd-derived repo: {:?}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains(&format!("torn line at offset {}", valid.len())),
        "{:?}",
        run.stderr
    );
    assert_eq!(
        log_bytes(&root),
        valid,
        "the act truncated the cwd-resolved store"
    );
    assert_no_debris(&root);
}

/// Repair's resolution-failure arm (veto-able (iii) on repair's own
/// dispatch, not just verify's): no `--repo`, a working directory
/// matching nothing — over a root whose registry holds a repo, so the
/// walk ran and the match is what failed — is exit 1 with the resolve
/// message on stderr (the progress/refresh precedent: the option
/// exists, the resolution failed — an environment fault, not a usage
/// fault).
#[test]
fn no_repo_flag_with_a_cwd_matching_nothing_is_repair_s_exit_1_resolve_message() {
    let dir = TempDir::new().unwrap();
    let elsewhere = TempDir::new().unwrap();
    let root = data_root(&dir);
    drop(build_short_store(&root));

    let run = run_cli_cwd(&["repair", "--root", root_arg(&dir)], elsewhere.path());
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
