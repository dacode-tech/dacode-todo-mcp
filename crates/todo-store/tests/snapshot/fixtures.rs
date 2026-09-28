//! The snapshot suite's shared fixtures: the golden world (four distinct
//! counters and numbers burned above the highest live id), the f5 world
//! (a cascade-burned task counter), a test-local mirror of the snapshot
//! document that the fault fixtures mutate through, and small helpers
//! over store directories.
//!
//! Nothing here is a test, so this module contributes no test names of
//! its own (P-LIVE-002). Every value is caller-passed and deterministic:
//! no clock, no environment, no absolute host path (§8). The world is the
//! crate's `camera` fixture set — the builders reuse t10's timeline
//! fixtures read-only (`super::timeline`) — so the golden describes the
//! same world as the other subjects.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use todo_domain::{
    BlocksEdge, Confidence, Decision, DecisionDraft, DecisionStatus, Doc, DocDraft, Epic,
    References, Repo, Status, Task, TaskId,
};
use todo_store::{BlocksEdgeDto, JsonlStore, Store, StoreError, TaskPatch, from_bytes, to_bytes};

use super::timeline::{at, epic_draft, repo, short_store, task_draft, task_id};

// =================================================================
// Paths and directory reads.
// =================================================================

/// The snapshot beside a log: the protected path plus `.snapshot` —
/// the layout's derivation rule, spelled from the consumer side so the
/// suite would notice a rename. Layout callers get the same path from
/// `DataRoot::snapshot_path` (pinned in `roundtrip.rs`).
pub(super) fn snapshot_path(log: &Path) -> PathBuf {
    let mut path = log.as_os_str().to_os_string();
    path.push(".snapshot");
    PathBuf::from(path)
}

/// The snapshot document's bytes, as they sit beside `log`.
pub(super) fn snapshot_bytes(log: &Path) -> Vec<u8> {
    fs::read(snapshot_path(log)).unwrap()
}

/// The names of `dir`'s entries, sorted — including dotfiles, so a
/// leftover temporary is caught. The debris tests' one directory reader
/// (the layout suite's helper, repeated because integration targets
/// share nothing but `common`).
pub(super) fn dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", dir.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// =================================================================
// The fixture worlds.
// =================================================================

/// The golden's world (Decision 12): the camera repo, **four distinct
/// counters** — epic 3, task 4, decision 1, doc 2 — and numbers burned
/// *above* the highest live id: the `E3` cascade takes `U3`/`U4` with it,
/// so the task counter (4) exceeds the highest live task (U2) and the
/// epic counter (3) the highest live epic (E2). A counter transposition
/// or a restore that only re-derived counters from the live nodes cannot
/// pass a golden over this state. Both repo mutables are moved *after*
/// registration, so the document pins the **current** repo node while
/// the log's seed holds the registration (Decision 9).
pub(super) fn build_golden_store(path: &Path) -> JsonlStore {
    let mut store = JsonlStore::create_log(path, repo()).unwrap();

    let e1 = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let e2 = store
        .add_epic(epic_draft("Encode and mux"), at("2026-09-25T08:06:00Z"))
        .unwrap();
    let e3 = store
        .add_epic(
            epic_draft("Retire the legacy capture path"),
            at("2026-09-25T08:07:00Z"),
        )
        .unwrap();

    let d1 = store
        .add_decision(
            decision_draft("Which encoder profile"),
            at("2026-09-25T08:10:00Z"),
        )
        .unwrap();

    let s1 = store
        .add_doc(
            doc_draft("Sensor bring-up notes", "docs/sensor.md"),
            at("2026-09-25T08:20:00Z"),
        )
        .unwrap();
    let s2 = store
        .add_doc(
            doc_draft("Encoding plan", "docs/encoding.md"),
            at("2026-09-25T08:25:00Z"),
        )
        .unwrap();
    // Two docs, so the golden's `docs` array holds two ascending
    // entries and the doc counter is a fourth distinct value.
    assert_eq!((s1.id.number(), s2.id.number()), (1, 2));

    let u1 = store
        .add_task(
            task_draft(&e1.id, "Bring up the sensor"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    let u2 = store
        .add_task(
            task_draft(&e2.id, "Mux the elementary streams"),
            at("2026-09-25T09:10:00Z"),
        )
        .unwrap();
    let u3 = store
        .add_task(
            task_draft(&e3.id, "Delete the v4l shim"),
            at("2026-09-25T09:20:00Z"),
        )
        .unwrap();
    let u4 = store
        .add_task(
            task_draft(&e3.id, "Move the last caller off it"),
            at("2026-09-25T09:30:00Z"),
        )
        .unwrap();

    store
        .link_blocks(
            BlocksEdge::TaskToTask {
                from: u1.id.clone(),
                to: u2.id.clone(),
            },
            at("2026-09-25T09:40:00Z"),
        )
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::DecisionToTask {
                from: d1.id,
                to: u2.id.clone(),
            },
            at("2026-09-25T09:45:00Z"),
        )
        .unwrap();
    store
        .link_relates(u1.id, u2.id, at("2026-09-25T09:50:00Z"))
        .unwrap();

    // Both terminals, so the cascade may run: closed with evidence, and
    // wontfix — §5.1's delete rule.
    store
        .update_task(
            u3.id,
            at("2026-09-25T10:00:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("the shim is gone".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    store
        .update_task(
            u4.id,
            at("2026-09-25T10:05:00Z"),
            TaskPatch {
                status: Some(Status::Wontfix),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    // Burns E3, U3 and U4 (§3: counters only move up, gaps are
    // meaningful) — the state f5's counter restore exists for.
    let cascade = store
        .delete_epic(e3.id, at("2026-09-25T10:10:00Z"))
        .unwrap();
    assert_eq!(cascade, vec![task_id(3), task_id(4)]);

    store
        .set_repo_path(
            "/srv/checkouts/camera".to_owned(),
            at("2026-09-25T10:20:00Z"),
        )
        .unwrap();
    store
        .set_repo_name(
            "Camera firmware (gen2)".to_owned(),
            at("2026-09-25T10:30:00Z"),
        )
        .unwrap();

    let counters = store.graph().counters();
    assert_eq!(
        (
            counters.epic(),
            counters.task(),
            counters.decision(),
            counters.doc()
        ),
        (3, 4, 1, 2),
        "the golden fixture's counters must stay four distinct values"
    );
    store
}

/// The f5 world, end to end: `U2`/`U3` burned by the `E2` cascade while
/// `U1` stays live under `E1`, so the task counter (3) exceeds the
/// highest live task (1) — a restore that re-derived counters from the
/// nodes would re-issue `U2` on the next allocation, violating §3.
pub(super) fn build_burned_number_store(path: &Path) -> JsonlStore {
    let mut store = JsonlStore::create_log(path, repo()).unwrap();
    let e1 = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let e2 = store
        .add_epic(
            epic_draft("Retire the legacy capture path"),
            at("2026-09-25T08:06:00Z"),
        )
        .unwrap();
    store
        .add_task(
            task_draft(&e1.id, "Bring up the sensor"),
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    let u2 = store
        .add_task(
            task_draft(&e2.id, "Delete the v4l shim"),
            at("2026-09-25T09:10:00Z"),
        )
        .unwrap();
    let u3 = store
        .add_task(
            task_draft(&e2.id, "Move the last caller off it"),
            at("2026-09-25T09:20:00Z"),
        )
        .unwrap();
    store
        .update_task(
            u2.id,
            at("2026-09-25T09:30:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("the shim is gone".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    store
        .update_task(
            u3.id,
            at("2026-09-25T09:40:00Z"),
            TaskPatch {
                status: Some(Status::Wontfix),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    let cascade = store
        .delete_epic(e2.id, at("2026-09-25T10:00:00Z"))
        .unwrap();
    assert_eq!(cascade, vec![task_id(2), task_id(3)]);

    let counters = store.graph().counters();
    assert_eq!((counters.epic(), counters.task()), (2, 3));
    store
}

/// A minimal snapshotted store — t10's `short_store` (epic E1, tasks
/// U1/U2, one `blocks` edge) plus one snapshot: the base every fault
/// fixture splices from.
pub(super) fn snapshotted_short_store(path: &Path) -> JsonlStore {
    let mut store = short_store(path);
    store.snapshot().unwrap();
    store
}

// =================================================================
// The document mirror and the fault helpers.
// =================================================================

/// A test-local mirror of the snapshot document, so every fault fixture
/// is produced by the crate's own codec over **real** values — parsed
/// from a real snapshot, mutated in one field, re-serialized — never
/// hand-written JSON (t10's f8 discipline; t11's `UnsortedRegistry`
/// precedent). Drift cannot pass silently: a field this mirror loses or
/// misspells makes either the parse of the real document or the engine's
/// parse of the fixture fail loudly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct SnapshotMirror {
    pub(super) format_version: u32,
    pub(super) log_offset: u64,
    pub(super) repo: Repo,
    pub(super) counters: CountersMirror,
    pub(super) epics: Vec<Epic>,
    pub(super) tasks: Vec<Task>,
    pub(super) decisions: Vec<Decision>,
    pub(super) docs: Vec<Doc>,
    pub(super) blocks: Vec<BlocksEdgeDto>,
    /// The wire shape of a relates pair — a two-element array of id
    /// strings — as a nested `Vec` rather than the document's tuple: the
    /// JSON engine serializes tuples but cannot deserialize them, and the
    /// engine-side reader works around that the same way (see
    /// `snapshot.rs`). Same bytes on both spellings.
    pub(super) relates: Vec<Vec<TaskId>>,
}

/// The mirror's counters half: four named fields, the shape a
/// transposition cannot hide in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct CountersMirror {
    pub(super) epic: u64,
    pub(super) task: u64,
    pub(super) decision: u64,
    pub(super) doc: u64,
}

impl SnapshotMirror {
    /// The real snapshot beside `log`, parsed through the mirror.
    pub(super) fn read(log: &Path) -> Self {
        from_bytes(&snapshot_bytes(log)).unwrap()
    }

    /// Writes `self` beside `log` as the snapshot document: one compact
    /// JSON object plus exactly one trailing LF, through the crate's
    /// codec.
    pub(super) fn write(&self, log: &Path) {
        let mut bytes = to_bytes(self).unwrap();
        bytes.push(b'\n');
        fs::write(snapshot_path(log), &bytes).unwrap();
    }
}

/// Real snapshot bytes with an unknown top-level field spliced in after
/// the version — `deny_unknown_fields`' rejection case, built by string
/// surgery on a real document (t10's splice discipline).
pub(super) fn with_unknown_field(bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    text.replacen(
        "{\"format_version\":",
        "{\"surprise\":true,\"format_version\":",
        1,
    )
    .into_bytes()
}

/// Asserts that opening `log`, whose snapshot has been damaged, fails
/// with an **unpositioned** `corrupt_store` whose detail contains
/// `needle`, carries no path, and is byte-identical on a second attempt
/// (§4.3's report is deterministic). The unpositioned shape is the
/// cross-checks': a semantic disagreement between two files is not
/// located by a byte offset in either.
pub(super) fn expect_snapshot_fault(log: &Path, needle: &str) -> StoreError {
    let error = JsonlStore::open_log(log).unwrap_err();
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
    assert_eq!(error, JsonlStore::open_log(log).unwrap_err());
    error
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

fn doc_draft(title: &str, path: &str) -> DocDraft {
    DocDraft {
        title: title.to_owned(),
        path: path.to_owned(),
        version: "1.0".to_owned(),
        references: References::new(),
    }
}
