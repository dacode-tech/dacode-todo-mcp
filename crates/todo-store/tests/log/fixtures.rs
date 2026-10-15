//! The log suite's shared fixtures: the value builders, the scripted
//! timeline that exercises all seventeen event kinds (and that the
//! `event_log` golden pins), and the byte-level helpers the corruption
//! fixtures splice with.
//!
//! The tests themselves live in the sibling [`super::replay`] and
//! [`super::corrupt`] modules; nothing here is a test, so this module
//! contributes no test names of its own.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use tempfile::TempDir;
use todo_domain::{
    BaseCosts, BlocksEdge, Confidence, Date, DecisionDraft, DecisionId, DecisionStatus, DocDraft,
    DocId, EpicDraft, EpicId, KindMultipliers, Priority, PriorityMultipliers, Reference,
    ReferenceKind, References, Repo, RepoSlug, RewardsConfig, Size, Status, TaskDraft, TaskId,
    TaskKind, Timestamp,
};
use todo_store::{
    DataRoot, DecisionPatch, DocPatch, EpicPatch, JsonlStore, ReflectionTrigger, RewardsTotals,
    Store, StoreError, TaskPatch,
};

/// The nineteen `kind` tags of §4.1's schema (v2: §15's two rewards
/// kinds join), in the order the timeline first exercises them.
pub(super) const EVERY_KIND: [&str; 19] = [
    "repo_register",
    "epic_add",
    "task_add",
    "decision_add",
    "doc_register",
    "epic_update",
    "task_update",
    "decision_update",
    "doc_update",
    "doc_set_path",
    "repo_set_path",
    "repo_set_name",
    "epic_delete",
    "blocks_link",
    "blocks_unlink",
    "relates_link",
    "relates_unlink",
    "reward_grant",
    "reward_reflect",
];

// =================================================================
// Fixtures.
// =================================================================

pub(super) fn at(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

fn slug(value: &str) -> RepoSlug {
    RepoSlug::from_str(value).unwrap()
}

fn reference(kind: ReferenceKind, target: &str, note: Option<&str>) -> Reference {
    Reference::new(kind, target.to_owned(), note.map(str::to_owned)).unwrap()
}

// The timeline's allocation order fixes every id, so the second half
// can name nodes the first half created without being handed them —
// which is what lets a session reopen mid-timeline and carry on.
pub(super) fn task_id(number: u64) -> TaskId {
    TaskId::new(slug("camera"), number).unwrap()
}

pub(super) fn epic_id(number: u64) -> EpicId {
    EpicId::new(slug("camera"), number).unwrap()
}

pub(super) fn decision_id(number: u64) -> DecisionId {
    DecisionId::new(slug("camera"), number).unwrap()
}

pub(super) fn doc_id(number: u64) -> DocId {
    DocId::new(slug("camera"), number).unwrap()
}

pub(super) fn repo() -> Repo {
    Repo {
        slug: slug("camera"),
        name: "Camera firmware".to_owned(),
        path: "/srv/repos/camera".to_owned(),
        registered_at: at("2026-09-25T08:00:00Z"),
    }
}

/// The timeline's camera slug — the store's path under a root.
pub(super) fn camera_slug() -> RepoSlug {
    slug("camera")
}

/// The timeline's rewards config (§15): every number distinct so the
/// golden's reward lines pin which multiplier produced which amount.
/// `reflect_every` sits above U1's grant and below the running total
/// after U3's, so the timeline's threshold crossing is real.
pub(super) fn rewards_config() -> RewardsConfig {
    RewardsConfig {
        enabled: true,
        currency: "coffee".to_owned(),
        base_costs: BaseCosts {
            s: 10,
            m: 25,
            l: 50,
            xl: 100,
        },
        default_size: Size::M,
        priority_multipliers: PriorityMultipliers {
            high: 150,
            medium: 100,
            low: 50,
        },
        kind_multipliers: KindMultipliers {
            task: 110,
            gate: 130,
            finding: 70,
            debt: 40,
        },
        reflect_every: 100,
    }
}

/// The golden timeline's root: a data root with the rewards config
/// saved, so the timeline's closures grant (§15's enabled root). An
/// inert root — no config — is simply `DataRoot::new(dir.path())`.
pub(super) fn timeline_root(dir: &TempDir) -> DataRoot {
    let root = DataRoot::new(dir.path());
    root.save_config(&rewards_config()).unwrap();
    root
}

pub(super) fn log_path(dir: &TempDir) -> PathBuf {
    dir.path().join("event-log.jsonl")
}

pub(super) fn epic_draft(title: &str) -> EpicDraft {
    EpicDraft {
        title: title.to_owned(),
        description: String::new(),
        status: Status::Open,
        references: References::new(),
    }
}

/// The first epic's draft, with references inserted out of canonical
/// order so the golden pins the sorted wire order.
fn sensor_epic_draft() -> EpicDraft {
    EpicDraft {
        description: "Everything from capture to encoding.".to_owned(),
        references: tracker_references(),
        ..epic_draft("Ship the sensor pipeline")
    }
}

pub(super) fn task_draft(epic: &EpicId, title: &str) -> TaskDraft {
    TaskDraft {
        epic: epic.clone(),
        kind: TaskKind::Task,
        title: title.to_owned(),
        body: String::new(),
        status: Status::Open,
        verify: Some("cargo test -p todo-store".to_owned()),
        evidence: None,
        note: None,
        references: References::new(),
        priority: None,
        until: None,
        estimate: None,
    }
}

fn kinded_task_draft(epic: &EpicId, title: &str, kind: TaskKind) -> TaskDraft {
    TaskDraft {
        kind,
        ..task_draft(epic, title)
    }
}

/// The first task's draft: references out of order, a priority and an
/// until-date, so the add line carries every optional field.
fn sensor_task_draft(epic: &EpicId) -> TaskDraft {
    TaskDraft {
        body: "Under low light the encoder stalls.".to_owned(),
        references: hot_path_references(),
        priority: Some(Priority::High),
        until: Some(Date::parse("2026-10-01").unwrap()),
        ..task_draft(epic, "Bring up the sensor")
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

/// Three references inserted out of canonical order: url, tracker with
/// a note, tracker without.
fn tracker_references() -> References {
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
    references
}

/// Two references out of order: file with a note, then doc.
fn hot_path_references() -> References {
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
    references
}

fn spec_references() -> References {
    let mut references = References::new();
    references
        .insert(reference(
            ReferenceKind::Spec,
            "todo-mcp-design.md §4",
            None,
        ))
        .unwrap();
    references
        .insert(reference(
            ReferenceKind::Url,
            "https://sensor.example",
            None,
        ))
        .unwrap();
    references
}

// =================================================================
// The scripted timeline (all 19 event kinds).
// =================================================================

/// Creates the camera store under the enabled `root` and runs the whole
/// scripted timeline over it, returning the open store.
///
/// The timeline is root-based because its closures grant: the config
/// document is the root's, and only a store the layout opened reads it.
/// On an inert root the same halves run grant-free — every `close_task`
/// degrades to its raw update — which is the byte-neutrality §15 pins.
pub(super) fn build_timeline(root: &DataRoot) -> JsonlStore {
    let mut store = root.register(repo()).unwrap();
    timeline_first_half(&mut store);
    timeline_second_half(&mut store);
    timeline_rewards_phase(&mut store);
    store
}

/// The timeline's first half: three epics, an open decision, a
/// registered doc with its audited path move, four tasks across two
/// epics, all three `blocks` shapes and both unlinks.
pub(super) fn timeline_first_half(store: &mut JsonlStore) {
    let e1 = store
        .add_epic(sensor_epic_draft(), at("2026-09-25T08:05:00Z"))
        .unwrap();
    let e2 = store
        .add_epic(
            epic_draft("Retire the legacy capture path"),
            at("2026-09-25T08:06:30Z"),
        )
        .unwrap();
    // The third epic holds no task until the second half, after the
    // cascade below has burned two numbers.
    store
        .add_epic(epic_draft("Encode and mux"), at("2026-09-25T08:07:00.25Z"))
        .unwrap();

    let d1 = store
        .add_decision(
            decision_draft("Which encoder profile"),
            at("2026-09-25T08:10:00Z"),
        )
        .unwrap();

    let s1 = store
        .add_doc(
            doc_draft("Sensor bring-up notes"),
            at("2026-09-25T08:20:00Z"),
        )
        .unwrap();
    store
        .set_doc_path(
            s1.id.clone(),
            "docs/bringup/sensor.md".to_owned(),
            at("2026-09-25T08:25:00Z"),
        )
        .unwrap();
    store
        .update_doc(
            s1.id,
            at("2026-09-25T08:30:00.5Z"),
            DocPatch {
                title: Some("Sensor bring-up".to_owned()),
                version: Some("1.5".to_owned()),
                references: Some(spec_references()),
            },
        )
        .unwrap();

    let u1 = store
        .add_task(sensor_task_draft(&e1.id), at("2026-09-25T09:00:00Z"))
        .unwrap();
    let u2 = store
        .add_task(
            task_draft(&e1.id, "Fix the dropped frames"),
            at("2026-09-25T09:30:00Z"),
        )
        .unwrap();
    let u3 = store
        .add_task(
            kinded_task_draft(&e2.id, "Delete the v4l shim", TaskKind::Debt),
            at("2026-09-25T09:45:00Z"),
        )
        .unwrap();
    let u4 = store
        .add_task(
            kinded_task_draft(&e2.id, "Move the last caller off it", TaskKind::Finding),
            at("2026-09-25T09:50:00Z"),
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
                to: e2.id,
            },
            at("2026-09-25T10:06:00Z"),
        )
        .unwrap();
    store
        .link_relates(u1.id.clone(), u2.id.clone(), at("2026-09-25T10:10:00Z"))
        .unwrap();
    store
        .link_relates(u3.id.clone(), u4.id.clone(), at("2026-09-25T10:15:00Z"))
        .unwrap();
    store
        .unlink_relates(u3.id.clone(), u4.id.clone(), at("2026-09-25T10:20:00Z"))
        .unwrap();
    store
        .link_blocks(
            BlocksEdge::TaskToTask {
                from: u1.id.clone(),
                to: u4.id.clone(),
            },
            at("2026-09-25T10:25:00Z"),
        )
        .unwrap();
    store
        .unlink_blocks(
            BlocksEdge::TaskToTask {
                from: u1.id,
                to: u4.id,
            },
            at("2026-09-25T10:30:00Z"),
        )
        .unwrap();
}

/// The timeline's second half: updates through every patch DTO with
/// every field `Some` at least once, the epic-delete cascade that
/// burns two task numbers, the repo attribute updates whose audit
/// instant rides the event line, and the adds that show the gap. The
/// three terminal updates go through the closure-aware path: on the
/// enabled root U1 and U3 grant (a reported size and the default
/// bucket), U4's `wontfix` grants nothing; on an inert root all three
/// are byte-identical to the raw update.
pub(super) fn timeline_second_half(store: &mut JsonlStore) {
    store
        .update_task(
            task_id(2),
            at("2026-09-25T11:00:00Z"),
            TaskPatch {
                status: Some(Status::Blocked),
                note: Some(Some("waiting on the sensor bring-up".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    // `Some(None)` clears a field, which `None` cannot express.
    store
        .update_task(
            task_id(2),
            at("2026-09-25T11:30:00.75Z"),
            TaskPatch {
                status: Some(Status::InWork),
                note: Some(None),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    let u1_closure = store
        .close_task(
            task_id(1),
            at("2026-09-25T12:00:00Z"),
            TaskPatch {
                title: Some("Bring up the sensor and hold the line".to_owned()),
                body: Some("Low light stalls the encoder; the gate holds the pipeline.".to_owned()),
                kind: Some(TaskKind::Gate),
                status: Some(Status::Closed),
                verify: Some(Some("cargo test -p encoder frames".to_owned())),
                evidence: Some(Some("42 passed, 0 failed".to_owned())),
                note: Some(None),
                references: Some(hot_path_references()),
                priority: Some(Some(Priority::High)),
                until: Some(Some(Date::parse("2026-10-01").unwrap())),
                ..TaskPatch::default()
            },
            // The actual size reported at the closure call: L, against
            // no estimate — the pre-E13 task's shape. On the enabled
            // root this grants 50 × 150 × 130 / 10000 → 98 coffee (the
            // gate kind the patch set, the high priority, the reported
            // L); the amount is pinned by the golden's bytes and the
            // rewards suite's own assertions, which run where the root
            // is known to be enabled — these halves run on inert roots
            // too, where the same call grants nothing.
            Some(Size::L),
        )
        .unwrap();
    drop(u1_closure);
    // Both terminal statuses, so the cascade below may run.
    let u3_closure = store
        .close_task(
            task_id(3),
            at("2026-09-25T12:30:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("the shim is gone".to_owned())),
                ..TaskPatch::default()
            },
            // No size reported: the default bucket (M) applies. On the
            // enabled root: 25 × 100 × 40 / 10000 → 10 coffee.
            None,
        )
        .unwrap();
    drop(u3_closure);
    let u4_closure = store
        .close_task(
            task_id(4),
            at("2026-09-25T12:45:00Z"),
            TaskPatch {
                status: Some(Status::Wontfix),
                ..TaskPatch::default()
            },
            Some(Size::S),
        )
        .unwrap();
    // `wontfix` grants nothing on any root — declined work is not
    // rewarded work.
    assert_eq!(u4_closure.reward, None);

    store
        .update_epic(
            epic_id(1),
            at("2026-09-25T13:00:00Z"),
            EpicPatch {
                title: Some("Ship the sensor pipeline (gen2)".to_owned()),
                description: Some("Capture to encoding on the gen2 SoC.".to_owned()),
                status: Some(Status::InWork),
                references: Some(tracker_references()),
            },
        )
        .unwrap();
    store
        .update_decision(
            decision_id(1),
            at("2026-09-25T13:15:00Z"),
            DecisionPatch {
                title: Some("Which encoder profile (gen2)".to_owned()),
                options: Some(vec![
                    "(a) baseline".to_owned(),
                    "(b) main, 10-bit".to_owned(),
                ]),
                recommendation: Some(Some("(a) — the SoC tops out there".to_owned())),
                confidence: Some(Confidence::High),
                references: Some(spec_references()),
                ..DecisionPatch::default()
            },
        )
        .unwrap();
    // Signing changes no edge (§3/D10): the gating lines stay in the
    // log and in the graph.
    store
        .update_decision(
            decision_id(1),
            at("2026-09-25T13:30:00Z"),
            DecisionPatch {
                status: Some(DecisionStatus::Signed),
                resolution: Some(Some("signed (a) 2026-09-25".to_owned())),
                ..DecisionPatch::default()
            },
        )
        .unwrap();

    // The cascade burns camera:U3 and camera:U4 (§3: counters only
    // move up), so the next allocation is U5 and the gap is visible in
    // the ids that follow.
    let cascade = store
        .delete_epic(epic_id(2), at("2026-09-25T14:00:00Z"))
        .unwrap();
    assert_eq!(cascade, vec![task_id(3), task_id(4)]);

    // A `Repo` has no `updated` field (§3), so these two lines' `ts` is
    // the only record of when the attribute changed.
    store
        .set_repo_path(
            "/srv/checkouts/camera".to_owned(),
            at("2026-09-25T14:30:00Z"),
        )
        .unwrap();
    store
        .set_repo_name(
            "Camera firmware (gen2)".to_owned(),
            at("2026-09-25T14:45:00Z"),
        )
        .unwrap();

    let u5 = store
        .add_task(
            task_draft(&epic_id(3), "Mux the elementary streams"),
            at("2026-09-25T15:00:00Z"),
        )
        .unwrap();
    assert_eq!(u5.id, task_id(5));
    // A reparenting rides the patch DTO's `epic` field: there is no
    // `task_reparent` kind, because the domain method is sugar.
    store
        .update_task(
            u5.id,
            at("2026-09-25T15:30:00Z"),
            TaskPatch {
                epic: Some(epic_id(1)),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    store
        .add_task(
            task_draft(&epic_id(3), "Profile the muxer"),
            at("2026-09-25T16:00:00Z"),
        )
        .unwrap();
}

/// The timeline's rewards phase (§15): an estimated task closed through
/// the grant path — the Xl estimate is the guess, the reported L the
/// fact, so the honesty anchor's delta is in the data — and the
/// reflection the threshold crossing asked for. Runs on the enabled
/// root alone; the inert timelines never call it.
pub(super) fn timeline_rewards_phase(store: &mut JsonlStore) {
    let u7 = store
        .add_task(
            TaskDraft {
                estimate: Some(Size::Xl),
                ..task_draft(&epic_id(3), "Ship the muxer profile")
            },
            at("2026-09-25T16:30:00Z"),
        )
        .unwrap();
    assert_eq!(u7.estimate, Some(Size::Xl));
    let closure = store
        .close_task(
            u7.id.clone(),
            at("2026-09-25T17:00:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("the muxer is profiled".to_owned())),
                ..TaskPatch::default()
            },
            Some(Size::L),
        )
        .unwrap();
    // On the enabled root: 50 × 100 × 110 / 10000 → 55 coffee (the
    // task kind, no priority, the reported L against the Xl estimate).
    drop(closure);

    // The running total crossed `reflect_every` (100) at U3's grant
    // (98 + 10); the answer arrives later, after U7's — the totals at
    // the moment are the answer's, 98 + 10 + 55 = 163.
    let totals = RewardsTotals {
        total: 163,
        by_repo: [(camera_slug(), 163)].into_iter().collect(),
    };
    store
        .log_reflection(
            "the closure went clean and the estimate held.".to_owned(),
            ReflectionTrigger::Threshold,
            totals,
            at("2026-09-25T17:05:00Z"),
        )
        .unwrap();
}

/// A minimal valid log — the seed, one epic, two tasks and one
/// `blocks` edge between them — for the corruption fixtures, whose
/// assertions are about byte offsets and read better over a short file.
pub(super) fn short_store(path: &Path) -> JsonlStore {
    let mut store = JsonlStore::create_log(path, repo()).unwrap();
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

/// [`short_store`], closed — the fixtures that read a log's bytes back.
pub(super) fn short_log(path: &Path) {
    drop(short_store(path));
}

// =================================================================
// Byte-level helpers for the corruption fixtures.
// =================================================================

/// The log's lines, terminators included, in file order — the unit the
/// fixtures splice.
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

/// The lines whose `kind` tag is `kind`.
pub(super) fn lines_of_kind<'a>(lines: &[&'a [u8]], kind: &str) -> Vec<&'a [u8]> {
    let needle = format!("\"kind\":\"{kind}\"");
    lines
        .iter()
        .filter(|line| String::from_utf8_lossy(line).contains(&needle))
        .copied()
        .collect()
}

/// A real event line with its `ts` field cut out: parseable JSON that
/// no longer fits the schema.
pub(super) fn without_ts(line: &[u8]) -> Vec<u8> {
    let text = String::from_utf8(line.to_vec()).unwrap();
    let start = text.find("\"ts\":\"").unwrap();
    let end = start + text[start..].find("\",").unwrap() + 2;
    format!("{}{}", &text[..start], &text[end..]).into_bytes()
}

/// Writes `bytes` to a fresh log path under `dir` and returns it.
pub(super) fn write_fixture(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    path
}

/// Asserts that opening `path` fails with a positioned `corrupt_store`
/// at `offset` whose detail contains `needle`, carries no path, and is
/// byte-identical on a second attempt (§4.3's report is deterministic).
pub(super) fn expect_corrupt(path: &Path, offset: usize, needle: &str) -> StoreError {
    let error = JsonlStore::open_log(path).unwrap_err();
    match &error {
        StoreError::CorruptStore {
            offset: Some(found),
            detail,
        } => {
            assert_eq!(*found, offset, "{error}");
            assert!(detail.contains(needle), "{error}");
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    assert_eq!(error.code(), "corrupt_store");
    assert_eq!(error, JsonlStore::open_log(path).unwrap_err());
    error
}

/// The timeline's log bytes, on disk under `root`.
pub(super) fn timeline_bytes(root: &DataRoot) -> Vec<u8> {
    drop(build_timeline(root));
    fs::read(root.log_path(&camera_slug())).unwrap()
}
