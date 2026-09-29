//! The repair suite's shared fixtures: stores built through the real API
//! under an injected data root, the byte-splicing helpers that tear a real
//! line, and the outcome assertions every case shares.
//!
//! Nothing here is a test, so this module contributes no test names of its
//! own (P-LIVE-002). Every value is caller-passed and deterministic: no
//! clock, no environment, no absolute host path (§8). The builders reuse
//! t10's timeline fixtures read-only (`super::timeline`), so the world is
//! the crate's `camera` one.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use tempfile::TempDir;
use todo_domain::{BlocksEdge, Graph, Repo, RepoSlug, TaskDraft};
use todo_store::{DataRoot, JsonlStore, RepairOutcome, Store, StoreError, TornTail};

use super::timeline::{at, epic_draft, line_offsets, lines, repo, task_draft};

/// The preview bound the crate cuts [`TornTail::line`] at, mirrored here
/// because it is private: `a_torn_line_longer_than_the_bound_is_cut_at_it`
/// asserts the count against this number, so a moved bound fails the test
/// rather than silently re-defining it.
pub(super) const PREVIEW_BOUND: usize = 200;

/// How many bytes past the LF a tear cuts, unless a case says otherwise:
/// the shape `tests/log/corrupt.rs` uses, and enough to make the torn line
/// unparseable as well as unterminated.
pub(super) const TEAR_BYTES: usize = 5;

/// The damage bytes the log suite splices with. Terminated, so a line made
/// of them is a *committed* line and never a tear.
pub(super) const GARBAGE_LINE: &[u8] = b"{{{ not json\n";

// =================================================================
// Paths and directory reads.
// =================================================================

/// A data root injected over `dir`'s path — the layout's whole input, and
/// the only way these tests reach a filesystem location. The `TempDir`
/// must outlive the returned root.
pub(super) fn data_root(dir: &TempDir) -> DataRoot {
    DataRoot::new(dir.path())
}

/// The suite's repository: t10's camera, registered at a fixed instant.
pub(super) fn camera() -> Repo {
    repo()
}

/// The camera slug every case repairs.
pub(super) fn camera_slug() -> RepoSlug {
    camera().slug
}

/// The camera store's log, in the D4 layout.
pub(super) fn log_path(root: &DataRoot) -> PathBuf {
    root.log_path(&camera_slug())
}

/// The camera store's directory — the debris scan's subject.
pub(super) fn store_dir(root: &DataRoot) -> PathBuf {
    root.store_dir(&camera_slug())
}

/// The log's bytes, as they sit on disk.
pub(super) fn log_bytes(root: &DataRoot) -> Vec<u8> {
    fs::read(log_path(root)).unwrap()
}

/// The names of `dir`'s entries, sorted — including dotfiles, so a
/// leftover temporary is caught. The debris tests' one directory reader
/// (the layout and snapshot suites' helper, repeated because integration
/// targets share nothing but `common`).
pub(super) fn dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", dir.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Asserts the store directory holds exactly the files it should: the log,
/// its lock, and — when `snapshot` — its snapshot. A `.tmp` the repair
/// failed to rename shows up here, and nothing else can.
pub(super) fn assert_store_files(root: &DataRoot, snapshot: bool) {
    let mut expected = vec![
        "event-log.jsonl".to_owned(),
        "event-log.jsonl.lock".to_owned(),
    ];
    if snapshot {
        expected.push("event-log.jsonl.snapshot".to_owned());
    }
    expected.sort();
    assert_eq!(dir_entries(&store_dir(root)), expected);
}

// =================================================================
// The fixture worlds.
// =================================================================

/// Registers camera and builds the short store — one epic, two tasks, one
/// `blocks` edge — through the real API, returning the **open, locked**
/// handle. The pair-divergence case needs the handle (it snapshots and
/// compacts it); every other case wants it dropped, which
/// [`short_store_graph`] does.
pub(super) fn build_short_store(root: &DataRoot) -> JsonlStore {
    let mut store = root.register(camera()).unwrap();
    let epic = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let first = store
        .add_task(
            task_draft(&epic.id, "Bring up the sensor"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    let second = store
        .add_task(
            task_draft(&epic.id, "Fix the dropped frames"),
            at("2026-09-25T09:30:00Z"),
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::TaskToTask {
                from: first.id,
                to: second.id,
            },
            at("2026-09-25T10:00:00Z"),
        )
        .unwrap();
    store
}

/// [`build_short_store`], closed, plus the graph it replayed to — the
/// pre-tear state a successful repair must restore.
pub(super) fn short_store_graph(root: &DataRoot) -> Graph {
    let store = build_short_store(root);
    let graph: Graph = store.graph().clone();
    drop(store);
    graph
}

/// A store whose **last** line is long enough to exceed [`PREVIEW_BOUND`]:
/// camera plus one epic plus one task whose body is padding, so the final
/// `task_add` line is the bounded preview's subject.
pub(super) fn long_tail_graph(root: &DataRoot) -> Graph {
    let mut store = root.register(camera()).unwrap();
    let epic = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    store
        .add_task(
            TaskDraft {
                body: long_body(),
                ..task_draft(&epic.id, "Bring up the sensor")
            },
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    let graph: Graph = store.graph().clone();
    drop(store);
    graph
}

/// The long body: five repetitions of one sentence, well past
/// [`PREVIEW_BOUND`] once the event line's other fields join it.
fn long_body() -> String {
    "Under low light the encoder stalls and the pipeline backs up behind it. ".repeat(5)
}

/// A second repository, so a snapshot of *another* repo can be built from
/// real bytes: its slug against the log's seed is `reconcile_with_log`'s
/// unpositioned fault, which is what the no-truncation-point case needs and
/// no document mutation can produce more honestly.
pub(super) fn other() -> Repo {
    Repo {
        slug: RepoSlug::from_str("encoder").unwrap(),
        name: "Encoder library".to_owned(),
        path: "/srv/repos/encoder".to_owned(),
        registered_at: at("2026-09-25T08:00:00Z"),
    }
}

/// A real snapshot of [`other`], plus the `log_offset` it carries — its own
/// log's length. The offset is returned so a case can assert the premise it
/// needs: a foreign document covering *less* than the log under repair, so
/// the divergence check does not fire first.
pub(super) fn foreign_snapshot() -> (Vec<u8>, u64) {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = other().slug;
    let mut store = root.register(other()).unwrap();
    store.snapshot().unwrap();
    let offset = fs::read(root.log_path(&slug)).unwrap().len() as u64;
    let bytes = fs::read(root.snapshot_path(&slug)).unwrap();
    drop(store);
    (bytes, offset)
}

// =================================================================
// Byte-level helpers: tear a real line, never write one.
// =================================================================

/// The log's lines, terminators included, in file order (t10's splitter).
pub(super) fn log_lines(bytes: &[u8]) -> Vec<&[u8]> {
    lines(bytes)
}

/// The absolute byte offset of the log's final line — §4.3's repair point,
/// computed from t10's own splitter rather than a search for the newline.
pub(super) fn final_line_offset(bytes: &[u8]) -> usize {
    let offsets = line_offsets(bytes);
    offsets[offsets.len() - 1]
}

/// The torn fragment [`tear_append`] appends to `valid`: its final real
/// event line with the LF and `extra` more bytes dropped. Spliced from a
/// real serialized line, never hand-written (f8's rule).
pub(super) fn torn_fragment(valid: &[u8], extra: usize) -> Vec<u8> {
    let start = final_line_offset(valid);
    let line = &valid[start..valid.len() - 1];
    line[..line.len() - extra].to_vec()
}

/// The log's bytes with §4.3's torn tail appended — the fragment after the
/// clean bytes, which is what a crash during that event's `write_all`
/// leaves: the committed log intact, plus one unterminated line. The clean
/// prefix is byte-identical to `valid`, so a repair truncates back to
/// exactly `valid` and the store opens to the pre-tear graph.
pub(super) fn tear_append(valid: &[u8], extra: usize) -> Vec<u8> {
    let mut torn_bytes = valid.to_vec();
    torn_bytes.extend_from_slice(&torn_fragment(valid, extra));
    torn_bytes
}

/// [`tear_append`] by the default [`TEAR_BYTES`]. The torn line starts at
/// `valid.len()`.
pub(super) fn torn(valid: &[u8]) -> Vec<u8> {
    tear_append(valid, TEAR_BYTES)
}

/// The log's bytes with a hand-written header declaring `version` in place
/// of the real one, every event line left as it was. The header is the one
/// hand-written line this crate's fixtures allow — the log suite's
/// exception, because no writer can produce a foreign-version header.
pub(super) fn with_header_version(valid: &[u8], version: u32) -> Vec<u8> {
    let mut spliced = format!("{{\"format_version\":{version}}}\n").into_bytes();
    for line in &log_lines(valid)[1..] {
        spliced.extend_from_slice(line);
    }
    spliced
}

// =================================================================
// Outcome assertions.
// =================================================================

/// Asserts `outcome` is a [`RepairOutcome::TornTail`] that wrote exactly as
/// `truncated` says, and returns the torn line's report.
pub(super) fn expect_torn(outcome: &RepairOutcome, truncated: bool) -> TornTail {
    match outcome {
        RepairOutcome::TornTail {
            torn,
            truncated: wrote,
        } => {
            assert_eq!(*wrote, truncated, "{outcome:?}");
            torn.clone()
        }
        other => panic!("expected a torn tail, got {other:?}"),
    }
}

/// Asserts `outcome` is [`RepairOutcome::Unrepairable`] whose `detail`
/// names the case (`needle`), names §4.3's backup as the remedy, and
/// carries no path — the crate's rule for every detail it writes — and
/// returns the offset it reported.
pub(super) fn expect_refusal(outcome: &RepairOutcome, needle: &str) -> Option<u64> {
    match outcome {
        RepairOutcome::Unrepairable { offset, detail } => {
            assert!(detail.contains(needle), "{detail}");
            assert!(detail.contains("backup"), "the remedy is named: {detail}");
            assert!(!detail.contains('/'), "a detail carries no path: {detail}");
            *offset
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// The refusal's `detail` text, for the cases that assert on the engine
/// report riding inside it.
pub(super) fn refusal_detail(outcome: &RepairOutcome) -> String {
    match outcome {
        RepairOutcome::Unrepairable { detail, .. } => detail.clone(),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// Asserts that opening the camera store fails with an **unpositioned**
/// `corrupt_store` whose detail contains `needle` — the shape of a
/// disagreement between two files, which no byte position locates.
pub(super) fn expect_open_unpositioned(root: &DataRoot, needle: &str) {
    let error = root.open(&camera_slug()).unwrap_err();
    match &error {
        StoreError::CorruptStore {
            offset: None,
            detail,
        } => {
            assert!(detail.contains(needle), "{error}");
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected an unpositioned corrupt_store, got {other:?}"),
    }
    assert_eq!(error.code(), "corrupt_store");
}

/// Asserts that opening the camera store fails with a positioned
/// `corrupt_store` at `offset` whose detail contains `needle` — the open
/// path's own report, which a repair must agree with.
pub(super) fn expect_open_corrupt(root: &DataRoot, offset: usize, needle: &str) {
    let error = root.open(&camera_slug()).unwrap_err();
    match &error {
        StoreError::CorruptStore {
            offset: Some(found),
            detail,
        } => {
            assert_eq!(*found, offset, "{error}");
            assert!(detail.contains(needle), "{error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    assert_eq!(error.code(), "corrupt_store");
}
