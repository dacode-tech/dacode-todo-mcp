//! The refresh subcommand's suite (spec §13, D20, t70): the
//! round-trip (a changed board re-imported, the store current), the
//! two refusals that leave the store untouched (the gone board — a
//! cut-over repo's store is the record; the broken board — the plan
//! builds before the surgery), the name/path preservation, the
//! resolution discipline, and the binary's exit contract — driven
//! in-process where the surface is a library function and through
//! the real binary where the surface is the process (the
//! tests/progress.rs pattern).
//!
//! The fixture repo's slug is `dacode-todo-mcp` — a dialect value,
//! because the refresh resolves the dialect as the slug (the
//! observed coincidence every current repo carries; the spec's
//! veto-able reading (i), and this suite's cost). The board is the
//! own dialect's synthetic corpus (borrowed from `tests/import/own.rs`
//! — hand-authored for the grammar, never live prose), written to a
//! temp checkout at the dialect's measured layout.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use tempfile::TempDir;
use todo_domain::{Repo, RepoSlug};
use todo_mcp::refresh;
use todo_store::DataRoot;

/// The fixture repo's slug — a dialect value (the dialect=slug rule).
const SLUG: &str = "dacode-todo-mcp";

/// A fixed canonical timestamp.
fn ts(value: &str) -> todo_domain::Timestamp {
    todo_domain::Timestamp::parse(value).expect("a fixed canonical timestamp")
}

/// The fixture's slug, parsed.
fn slug() -> RepoSlug {
    RepoSlug::from_str(SLUG).expect("a valid slug")
}

const HEAD: &str = "\
# TODO — synthetic

Intro prose, walked past.

## 1. Current state

Narrative prose, walked past.

| Blocking | What | Action |
|---|---|---|
| — | nothing blocks | dispatch |
";

const SECTION_2: &str = "\
## 2. Milestones

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | groundwork | — | done 2026-01-05 |
| M1 | the engine | **D2** gates this milestone | done 2026-02-10 — the gate ran green |
| M2 | the interface | — | in progress — decomposed 2026-03-15: the tree |
";

const SECTION_3: &str = "\
## 3. Done

- 2026-09-18 — **UB1 · a delivery log entry.** The delivery log is
  walked past, and its UB id is not a register item.
";

const SECTION_4: &str = "\
## 4. Follow-ups

Synthetic follow-ups, hand-authored for the grammar.

- [ ] **U1 · First follow-up** — body prose, no dependency.
- [x] **U2 · Second follow-up.** Checked, with a trailing period.
- [x] ~~**U3 · A struck item**~~ — struck and checked, closed prose.
- [ ] **U5 · Prose only** — body text.
";

const SECTION_5: &str = "\
## 5. Debt

Synthetic debt entries, recorded rather than resolved.

- [ ] **U6 · A debt item** — debt prose.
- [x] **U7 · A checked debt item** — more debt prose.
";

const SECTION_6: &str = "\
## 6. Session log

The narrative is walked past.
";

const DECISIONS_MD: &str = "\
# Decision record — synthetic

Prose intro, walked past.

## Sign-off summary

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| ~~**D1**~~ | First decision | take the simple path | low | M0 | resolved (a) 2026-09-18 — the simple path taken |
| **D2** | Second decision | gather the numbers first | medium | M1 | resolved 2026-09-20 — user decision, no option letter |
| **D4** | Fourth decision | wait for the study | medium | M2 | drafted 2026-09-22 — awaiting the user |
| **D5** | Fifth decision | gates nothing | low | — | resolved (a) 2026-09-23 — the gates-nothing row |

## Details

### D1 — First decision

Dossier body for D1.

### D2 — Second decision

Dossier body for D2.

### D4 — Fourth decision

Dossier body for D4.

### D5 — Fifth decision

Dossier body for D5.
";

/// Writes the corpus board into a fresh checkout, answering the
/// checkout's root — the repo's registered path.
fn checkout_with_board() -> TempDir {
    let checkout = TempDir::new().expect("a fresh checkout");
    write_board(checkout.path(), &board_text());
    checkout
}

/// Writes the two board files at the dialect's measured layout.
fn write_board(checkout: &Path, todo_md: &str) {
    let dir = checkout.join("docs").join("implementation");
    fs::create_dir_all(&dir).expect("the board directory is created");
    fs::write(dir.join("TODO.md"), todo_md).expect("TODO.md is written");
    fs::write(dir.join("decisions.md"), DECISIONS_MD).expect("decisions.md is written");
}

/// The corpus TODO.md.
fn board_text() -> String {
    format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}")
}

/// Registers the fixture repo at `checkout` with `name`, releasing
/// the store handle — the refresh refuses a live writer by design.
fn register(root: &DataRoot, checkout: &Path, name: &str) {
    drop(
        root.register(Repo {
            slug: slug(),
            name: name.to_owned(),
            path: checkout.display().to_string(),
            registered_at: ts("2026-10-02T08:00:00Z"),
        })
        .expect("the fixture repo registers"),
    );
}

/// One refresh over the fixture, in-process.
fn refresh_fixture(root: &DataRoot) -> Result<todo_mcp::refresh::ImportedDto, String> {
    refresh::refresh(root, &slug())
}

/// The store's task count, read lock-free.
fn task_count(root: &DataRoot) -> usize {
    root.read_repo(&slug())
        .expect("the store reads")
        .tasks()
        .count()
}

/// Runs the binary with `args` in `cwd`, answering its exit code,
/// stdout and stderr.
fn run(cwd: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("the binary spawns");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The round-trip: the first refresh imports the board, a changed
/// board re-imports — the census counts the new item and the store's
/// graph carries it. The sync's whole point.
#[test]
fn the_refresh_round_trips() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), SLUG);

    let first = refresh_fixture(&root).expect("the first refresh imports");
    let before = task_count(&root);
    assert_eq!(first.tasks as usize, before);

    // The board moves: a new follow-up lands.
    let changed = board_text().replace(
        "- [ ] **U5 · Prose only** — body text.\n",
        "- [ ] **U5 · Prose only** — body text.\n- [ ] **U8 · Added by the test** — the moved board.\n",
    );
    assert!(changed.contains("Added by the test"));
    write_board(checkout.path(), &changed);

    let second = refresh_fixture(&root).expect("the second refresh re-imports");
    assert_eq!(second.tasks as usize, before + 1);
    let graph = root.read_repo(&slug()).expect("the store reads");
    assert!(
        graph
            .tasks()
            .any(|task| task.title.contains("Added by the test")),
        "the new item is in the store"
    );
}

/// The gone board: a checkout with no board directory is the graceful
/// refusal — a cut-over repo's store is the record — and **nothing
/// was touched**: the entry still listed, the store still answering
/// the old graph.
#[test]
fn a_gone_board_refuses_with_nothing_touched() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), SLUG);
    let before = refresh_fixture(&root).expect("the first refresh imports");
    let tasks_before = task_count(&root);

    fs::remove_dir_all(checkout.path().join("docs").join("implementation"))
        .expect("the board is gone");
    let message = refresh_fixture(&root).expect_err("the gone board is refused");
    assert!(
        message.contains("no board to import") && message.contains("nothing was touched"),
        "the refusal names the case: {message}"
    );
    let registry = todo_store::Registry::load(&root).expect("the registry reads");
    assert!(registry.contains(&slug()), "the entry is untouched");
    assert_eq!(task_count(&root), tasks_before, "the store is untouched");
    let _ = before;
}

/// The broken board: a parse failure refuses before the surgery —
/// the plan is built first, so the store is still intact when the
/// board's grammar breaks.
#[test]
fn a_broken_board_refuses_with_nothing_touched() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), SLUG);
    refresh_fixture(&root).expect("the first refresh imports");
    let tasks_before = task_count(&root);

    fs::write(
        checkout
            .path()
            .join("docs")
            .join("implementation")
            .join("TODO.md"),
        "not a board at all\n",
    )
    .expect("the board is broken");
    let message = refresh_fixture(&root).expect_err("the broken board is refused");
    assert!(
        !message.contains("no board to import"),
        "the refusal is the parse error, not the gone-board case: {message}"
    );
    let registry = todo_store::Registry::load(&root).expect("the registry reads");
    assert!(registry.contains(&slug()), "the entry is untouched");
    assert_eq!(task_count(&root), tasks_before, "the store is untouched");
}

/// The re-register preserves the node's name and path (the Python
/// hardcoded `name = slug` — the recorded divergence), and stamps a
/// fresh registration instant — the staleness signal the page shows.
#[test]
fn the_refresh_preserves_the_name_and_path() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), "The Fixture Name");

    refresh_fixture(&root).expect("the refresh lands");
    let repo = root
        .read_repo(&slug())
        .expect("the store reads")
        .repo()
        .clone();
    assert_eq!(repo.name, "The Fixture Name");
    assert_eq!(repo.path, checkout.path().display().to_string());
    assert_ne!(
        repo.registered_at,
        ts("2026-10-02T08:00:00Z"),
        "the registration instant moved"
    );
}

/// The resolution discipline: an unregistered slug is `unknown_id`'s
/// message, never a raw io.
#[test]
fn an_unregistered_repo_is_refused() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), SLUG);
    let elsewhere = RepoSlug::from_str("elsewhere").expect("a valid slug");
    let message = refresh::refresh(&root, &elsewhere).expect_err("the unlisted slug is refused");
    assert_eq!(message, "unknown id: elsewhere");
}

/// The binary's exit contract, the happy path: run from the checkout
/// (the cwd tier resolves the repo), exit 0 with the census on
/// stdout.
#[test]
fn the_binary_refresh_prints_the_census() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), SLUG);

    // The in-process refresh first, so the census line is pinned
    // exactly — the field order and the wording, not just a prefix.
    let census = refresh_fixture(&root).expect("the in-process refresh");
    let (code, stdout, stderr) = run(
        checkout.path(),
        &[
            "refresh",
            "--root",
            dir.path().to_str().expect("a UTF-8 root"),
        ],
    );
    assert_eq!(code, Some(0), "stderr: {stderr}");
    assert_eq!(
        stdout,
        format!(
            "refresh: imported {} epics, {} decisions, {} docs, {} tasks, {} blocks edges into {}\n",
            census.epics,
            census.decisions,
            census.docs,
            census.tasks,
            census.blocks,
            dir.path().display()
        ),
        "the census line is the exact report"
    );
}

/// The binary's exit contract, the refusal: a gone board is exit 1
/// with the message on stderr and nothing on stdout.
#[test]
fn the_binary_gone_board_is_exit_1() {
    let checkout = checkout_with_board();
    let dir = TempDir::new().expect("a fresh root");
    let root = DataRoot::new(dir.path());
    register(&root, checkout.path(), SLUG);
    fs::remove_dir_all(checkout.path().join("docs").join("implementation"))
        .expect("the board is gone");

    let (code, stdout, stderr) = run(
        checkout.path(),
        &[
            "refresh",
            "--root",
            dir.path().to_str().expect("a UTF-8 root"),
        ],
    );
    assert_eq!(code, Some(1));
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("no board to import"), "stderr: {stderr}");
}
