//! The cli suite's shared fixtures: stores built through the real API
//! under an injected data root, the byte-splicing helpers that tear a
//! real line, the cli runner, and the debris scan every case ends with.
//!
//! The shape is the one `crates/todo-store/tests/repair/fixtures.rs`
//! established (required test 5): torn bytes are spliced **from real
//! serialized lines** — a real event line with its LF and its last few
//! bytes dropped — never hand-written; the one allowed hand-written line
//! is a foreign-version header, which no writer can produce. Timestamps
//! are caller-passed constants, ids follow the allocation order, and the
//! repo's `path` is a fixed literal — `Repo.path` is *data* (§7: the
//! tracked repository's location), so a host path in a golden would make
//! it unreproducible while a literal one is exactly what a store holds.
//!
//! Nothing here is a test, so this module contributes no test names of
//! its own (P-LIVE-002).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::str::FromStr;

use tempfile::TempDir;
use todo_domain::{
    BlocksEdge, Confidence, Date, DecisionDraft, DecisionStatus, DocDraft, EpicDraft, EpicId,
    Priority, Reference, ReferenceKind, References, Repo, RepoSlug, Status, TaskDraft, TaskId,
    TaskKind, Timestamp,
};
use todo_store::{DataRoot, JsonlStore, Store, TaskPatch};

/// How many bytes past the LF a tear cuts: the store suite's default —
/// enough to make the torn line unparseable as well as unterminated.
pub(super) const TEAR_BYTES: usize = 5;

/// The damage bytes the interior-corruption cases splice with.
/// Terminated, so a line made of them is a *committed* line and never a
/// tear.
pub(super) const GARBAGE_LINE: &[u8] = b"{{{ not json\n";

// =================================================================
// Paths, roots and the repo.
// =================================================================

/// A data root injected over `dir`'s path — the layout's whole input,
/// and the only way these tests reach a filesystem location. The
/// `TempDir` must outlive the returned root.
pub(super) fn data_root(dir: &TempDir) -> DataRoot {
    DataRoot::new(dir.path())
}

/// The suite's repository: camera, registered at a fixed instant, with a
/// fixed literal path — never a tempdir component (module docs).
pub(super) fn repo() -> Repo {
    Repo {
        slug: camera_slug(),
        name: "Camera firmware".to_owned(),
        path: "/repo/camera".to_owned(),
        registered_at: at("2026-09-25T08:00:00Z"),
    }
}

/// The camera slug every case drives the cli with.
pub(super) fn camera_slug() -> RepoSlug {
    RepoSlug::from_str("camera").unwrap()
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

// =================================================================
// Value builders.
// =================================================================

pub(super) fn at(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

fn reference(kind: ReferenceKind, target: &str, note: Option<&str>) -> Reference {
    Reference::new(kind, target.to_owned(), note.map(str::to_owned)).unwrap()
}

pub(super) fn epic_draft(title: &str) -> EpicDraft {
    EpicDraft {
        title: title.to_owned(),
        description: String::new(),
        status: Status::Open,
        references: References::new(),
    }
}

pub(super) fn task_draft(epic: &EpicId, title: &str) -> TaskDraft {
    TaskDraft {
        epic: epic.clone(),
        kind: TaskKind::Task,
        title: title.to_owned(),
        body: String::new(),
        status: Status::Open,
        verify: Some("cargo test -p todo-cli".to_owned()),
        evidence: None,
        note: None,
        references: References::new(),
        priority: None,
        until: None,
    }
}

fn decision_draft(title: &str) -> DecisionDraft {
    DecisionDraft {
        title: title.to_owned(),
        status: DecisionStatus::Open,
        options: vec!["(a) baseline".to_owned(), "(b) main, 10-bit".to_owned()],
        recommendation: Some("(a)".to_owned()),
        resolution: None,
        confidence: Confidence::Medium,
        references: References::new(),
    }
}

fn doc_draft(title: &str) -> DocDraft {
    DocDraft {
        title: title.to_owned(),
        path: "docs/sensor.md".to_owned(),
        version: "1.4".to_owned(),
        references: References::new(),
    }
}

// =================================================================
// The golden world: every node kind, every blocks shape, a relates
// pair, sorted references, and counters with burned numbers in them.
// =================================================================

/// Registers camera and builds the golden world through the real API in
/// one mutation order, returning the **open, locked** store.
///
/// The world: epics E1 and E3 live, E2 deleted in a cascade that burned
/// it and task U3 with it — so the dump's counters (`epic: 3`,
/// `task: 4`) show numbers no live id reaches, which is the fact an
/// inspector needs and graph state alone cannot show. Decision D1 gates
/// both an epic and a task; doc S1 is registered; U1↔U2 carry the
/// `relates` pair and a `blocks` edge; U1 is closed with evidence; U2
/// carries references, a priority and an until-date.
pub(super) fn build_golden_store(root: &DataRoot) -> JsonlStore {
    let mut store = root.register(repo()).unwrap();
    let e1 = store
        .add_epic(golden_epic_draft(), at("2026-09-25T08:05:00Z"))
        .unwrap();
    let d1 = store
        .add_decision(
            decision_draft("Which encoder profile"),
            at("2026-09-25T08:10:00Z"),
        )
        .unwrap();
    store
        .add_doc(
            doc_draft("Sensor bring-up notes"),
            at("2026-09-25T08:20:00Z"),
        )
        .unwrap();
    let u1 = store
        .add_task(
            task_draft(&e1.id, "Bring up the sensor"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    let u2 = store
        .add_task(golden_task_draft(&e1.id), at("2026-09-25T09:30:00Z"))
        .unwrap();
    let e2 = store
        .add_epic(
            epic_draft("Retire the legacy capture path"),
            at("2026-09-25T09:40:00Z"),
        )
        .unwrap();
    let u3 = store
        .add_task(
            TaskDraft {
                kind: TaskKind::Debt,
                ..task_draft(&e2.id, "Delete the v4l shim")
            },
            at("2026-09-25T09:45:00Z"),
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::TaskToTask {
                from: u1.id.clone(),
                to: u2.id.clone(),
            },
            at("2026-09-25T10:00:00Z"),
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::DecisionToTask {
                from: d1.id.clone(),
                to: u2.id.clone(),
            },
            at("2026-09-25T10:05:00Z"),
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::DecisionToEpic {
                from: d1.id,
                to: e1.id.clone(),
            },
            at("2026-09-25T10:06:00Z"),
        )
        .unwrap();
    store
        .link_relates(u1.id.clone(), u2.id.clone(), at("2026-09-25T10:10:00Z"))
        .unwrap();
    store
        .update_task(
            u3.id,
            at("2026-09-25T12:00:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("the shim is gone".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    store
        .update_task(
            u1.id,
            at("2026-09-25T12:30:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("42 passed, 0 failed".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    // The cascade burns E2 and U3 (§3: counters only move up), so the
    // dump's ids carry the gap and its counters the burned numbers.
    assert_eq!(
        store
            .delete_epic(e2.id, at("2026-09-25T14:00:00Z"))
            .unwrap(),
        vec![task_id(3)]
    );
    let e3 = store
        .add_epic(epic_draft("Encode and mux"), at("2026-09-25T14:10:00Z"))
        .unwrap();
    store
        .add_task(
            task_draft(&e3.id, "Mux the elementary streams"),
            at("2026-09-25T15:00:00Z"),
        )
        .unwrap();
    store
}

/// The same world as [`build_golden_store`], reached through a different
/// mutation order — decision and doc before the first epic, the edges
/// and the closes interleaved differently — so the dump's byte identity
/// across the two is insertion-order robustness, the domain's own test
/// shape. Every mutation carries the same `at` value as its twin, so the
/// final nodes, edges and counters are equal.
pub(super) fn build_golden_store_reordered(root: &DataRoot) -> JsonlStore {
    let mut store = root.register(repo()).unwrap();
    let d1 = store
        .add_decision(
            decision_draft("Which encoder profile"),
            at("2026-09-25T08:10:00Z"),
        )
        .unwrap();
    let e1 = store
        .add_epic(golden_epic_draft(), at("2026-09-25T08:05:00Z"))
        .unwrap();
    store
        .add_doc(
            doc_draft("Sensor bring-up notes"),
            at("2026-09-25T08:20:00Z"),
        )
        .unwrap();
    let u1 = store
        .add_task(
            task_draft(&e1.id, "Bring up the sensor"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    let u2 = store
        .add_task(golden_task_draft(&e1.id), at("2026-09-25T09:30:00Z"))
        .unwrap();
    let e2 = store
        .add_epic(
            epic_draft("Retire the legacy capture path"),
            at("2026-09-25T09:40:00Z"),
        )
        .unwrap();
    let u3 = store
        .add_task(
            TaskDraft {
                kind: TaskKind::Debt,
                ..task_draft(&e2.id, "Delete the v4l shim")
            },
            at("2026-09-25T09:45:00Z"),
        )
        .unwrap();
    store
        .link_relates(u1.id.clone(), u2.id.clone(), at("2026-09-25T10:10:00Z"))
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::DecisionToEpic {
                from: d1.id.clone(),
                to: e1.id.clone(),
            },
            at("2026-09-25T10:06:00Z"),
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::DecisionToTask {
                from: d1.id,
                to: u2.id.clone(),
            },
            at("2026-09-25T10:05:00Z"),
        )
        .unwrap();
    store
        .update_task(
            u3.id,
            at("2026-09-25T12:00:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("the shim is gone".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::TaskToTask {
                from: u1.id.clone(),
                to: u2.id,
            },
            at("2026-09-25T10:00:00Z"),
        )
        .unwrap();
    store
        .update_task(
            u1.id,
            at("2026-09-25T12:30:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("42 passed, 0 failed".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    assert_eq!(
        store
            .delete_epic(e2.id, at("2026-09-25T14:00:00Z"))
            .unwrap(),
        vec![task_id(3)]
    );
    let e3 = store
        .add_epic(epic_draft("Encode and mux"), at("2026-09-25T14:10:00Z"))
        .unwrap();
    store
        .add_task(
            task_draft(&e3.id, "Mux the elementary streams"),
            at("2026-09-25T15:00:00Z"),
        )
        .unwrap();
    store
}

/// The golden world's epic: a description and references inserted out of
/// canonical order, so the golden pins the sorted wire order.
fn golden_epic_draft() -> EpicDraft {
    let mut references = References::new();
    references
        .insert(reference(
            ReferenceKind::Url,
            "https://sensor.example/plan",
            None,
        ))
        .unwrap();
    references
        .insert(reference(
            ReferenceKind::Tracker,
            "CAM-42",
            Some("the external issue"),
        ))
        .unwrap();
    references
        .insert(reference(ReferenceKind::Tracker, "CAM-10", None))
        .unwrap();
    EpicDraft {
        description: "Everything from capture to encoding.".to_owned(),
        references,
        ..epic_draft("Ship the sensor pipeline")
    }
}

/// The golden world's second task: a body, references out of canonical
/// order, a display priority and an until-date — every optional field of
/// §3's task shape populated.
fn golden_task_draft(epic: &EpicId) -> TaskDraft {
    let mut references = References::new();
    references
        .insert(reference(
            ReferenceKind::File,
            "src/encode.rs",
            Some("hot path"),
        ))
        .unwrap();
    references
        .insert(reference(ReferenceKind::Doc, "camera:S1#5.2", None))
        .unwrap();
    TaskDraft {
        body: "Under low light the encoder stalls.".to_owned(),
        references,
        priority: Some(Priority::High),
        until: Some(Date::parse("2026-10-01").unwrap()),
        ..task_draft(epic, "Fix the dropped frames")
    }
}

fn task_id(number: u64) -> TaskId {
    TaskId::new(camera_slug(), number).unwrap()
}

// =================================================================
// The short world: the corruption cases' subject, whose assertions are
// about byte offsets and read better over a short file.
// =================================================================

/// Registers camera and builds the short store — one epic, two tasks,
/// one `blocks` edge — through the real API, returning the **open,
/// locked** handle. The contended and diverged cases need the handle;
/// every other case wants it dropped, which [`short_store_bytes`] does.
pub(super) fn build_short_store(root: &DataRoot) -> JsonlStore {
    let mut store = root.register(repo()).unwrap();
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

/// [`build_short_store`], closed — its log bytes on disk, the clean
/// state every corruption case starts from and every refusal must leave
/// untouched.
pub(super) fn short_store_bytes(root: &DataRoot) -> Vec<u8> {
    drop(build_short_store(root));
    log_bytes(root)
}

/// The diverged snapshot/log pair — `compact`'s (3)–(4) window, built the
/// way the store suite builds it: a covering snapshot from one real run
/// beside the short log of a twin's real `compact()`. Returns the two
/// files' bytes: `(short_log, covering_snapshot, covered_offset)`.
pub(super) fn diverged_pair() -> (Vec<u8>, Vec<u8>, u64) {
    let slug = camera_slug();

    let cover_dir = TempDir::new().unwrap();
    let cover_root = data_root(&cover_dir);
    let mut cover = build_short_store(&cover_root);
    let covered = cover.snapshot().unwrap();
    let covering = fs::read(cover_root.snapshot_path(&slug)).unwrap();
    drop(cover);

    let twin_dir = TempDir::new().unwrap();
    let twin_root = data_root(&twin_dir);
    let mut twin = build_short_store(&twin_root);
    twin.compact().unwrap();
    let short_log = log_bytes(&twin_root);
    drop(twin);

    assert!(
        covered > short_log.len() as u64,
        "the window's premise: the covering offset exceeds the short log"
    );
    (short_log, covering, covered)
}

// =================================================================
// Byte-level helpers: tear a real line, never write one (module docs).
// =================================================================

/// The log's lines, terminators included, in file order.
pub(super) fn lines(bytes: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut rest = bytes;
    while let Some(index) = rest.iter().position(|byte| *byte == b'\n') {
        let (line, tail) = rest.split_at(index + 1);
        out.push(line);
        rest = tail;
    }
    if !rest.is_empty() {
        out.push(rest);
    }
    out
}

/// The absolute byte offset of each of `bytes`' lines.
pub(super) fn line_offsets(bytes: &[u8]) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut position = 0;
    for line in lines(bytes) {
        offsets.push(position);
        position += line.len();
    }
    offsets
}

/// The absolute byte offset of the log's final line — §4.3's repair
/// point, computed from the splitter above rather than a search for the
/// newline.
pub(super) fn final_line_offset(bytes: &[u8]) -> usize {
    let offsets = line_offsets(bytes);
    offsets[offsets.len() - 1]
}

/// The torn fragment: the log's final real event line with the LF and
/// `extra` more bytes dropped — spliced from a real serialized line.
pub(super) fn torn_fragment(valid: &[u8], extra: usize) -> Vec<u8> {
    let start = final_line_offset(valid);
    let line = &valid[start..valid.len() - 1];
    line[..line.len() - extra].to_vec()
}

/// The log's bytes with §4.3's torn tail appended — the committed log
/// intact plus one unterminated line, which is what a crash during that
/// event's `write_all` leaves. The clean prefix is byte-identical to
/// `valid`, so a repair truncates back to exactly `valid`.
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

/// The log's bytes with the interior line at `index` replaced by
/// [`GARBAGE_LINE`]: an interior fault, which a crash cannot produce and
/// a truncation must not "fix". Returns the spliced bytes and the
/// faulted line's offset.
pub(super) fn with_interior_garbage(valid: &[u8], index: usize) -> (Vec<u8>, usize) {
    let mut spliced = Vec::new();
    for (position, line) in lines(valid).iter().enumerate() {
        if position == index {
            spliced.extend_from_slice(GARBAGE_LINE);
        } else {
            spliced.extend_from_slice(line);
        }
    }
    let offset = line_offsets(&spliced)[index];
    (spliced, offset)
}

/// The log's bytes with a hand-written header declaring `version` in
/// place of the real one, every event line left as it was. The header is
/// the one hand-written line these fixtures allow, because no writer can
/// produce a foreign-version header.
pub(super) fn with_header_version(valid: &[u8], version: u32) -> Vec<u8> {
    let mut spliced = format!("{{\"format_version\":{version}}}\n").into_bytes();
    for line in &lines(valid)[1..] {
        spliced.extend_from_slice(line);
    }
    spliced
}

// =================================================================
// The cli runner and the debris scan.
// =================================================================

/// One cli run's observable surface: the exit code and the two streams as
/// UTF-8 text — the end-to-end cases assert exactly these, and nothing
/// else (no in-process state survives a spawn).
pub(super) struct CliRun {
    pub(super) code: i32,
    pub(super) stdout: String,
    pub(super) stderr: String,
}

/// Spawns the built binary with `args` and an empty (EOF) stdin.
pub(super) fn run_cli(args: &[&str]) -> CliRun {
    finish(
        Command::new(env!("CARGO_BIN_EXE_todo-cli"))
            .args(args)
            .output()
            .unwrap(),
    )
}

/// Spawns the built binary with `args`, piping `stdin` to it — the
/// prompt cases' `y`/`n` answers.
pub(super) fn run_cli_stdin(args: &[&str], stdin: &str) -> CliRun {
    let mut child = Command::new(env!("CARGO_BIN_EXE_todo-cli"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    finish(child.wait_with_output().unwrap())
}

fn finish(output: Output) -> CliRun {
    CliRun {
        code: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

/// The `--root` argument for a tempdir-based root, as the runner's `&str`
/// shape wants it. The `TempDir` must outlive the string's use.
pub(super) fn root_arg(dir: &TempDir) -> &str {
    dir.path().to_str().unwrap()
}

/// Asserts no temporary debris survived in the fixture: the store
/// directory and the data root hold no `.tmp` name — a repair's temp
/// file the rename failed to consume would show up here (required test
/// 6).
pub(super) fn assert_no_debris(root: &DataRoot) {
    for dir in [root.root(), &root.repos_dir(), &store_dir(root)] {
        scan_for_debris(dir);
    }
}

fn scan_for_debris(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return; // a directory that does not exist holds no debris
    };
    for entry in entries {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        assert!(
            !name.ends_with(".tmp"),
            "temporary debris '{}' in {}",
            name,
            dir.display()
        );
    }
}

/// Asserts the golden bytes carry no component of the tempdir they were
/// built in (required test 2's no-host-path rule, asserted rather than
/// documented). Components shorter than four characters are skipped: a
/// single-letter component — macOS's `T` temp root — is a substring of
/// every canonical timestamp, so matching it would test nothing.
pub(super) fn assert_no_tempdir_components(bytes: &[u8], dir: &TempDir) {
    let text = String::from_utf8_lossy(bytes);
    assert!(
        !text.contains(dir.path().to_string_lossy().as_ref()),
        "the golden embeds the tempdir path itself"
    );
    for component in dir.path().components() {
        let name = component.as_os_str().to_string_lossy();
        if name.len() < 4 {
            continue;
        }
        assert!(
            !text.contains(name.as_ref()),
            "the golden embeds the tempdir component {name:?}"
        );
    }
}
