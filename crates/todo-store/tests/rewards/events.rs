//! The reward events read (§15's display half, U63): the grants and
//! reflections collected from every registered repo's committed
//! prefix — root order (the registry's ascending-slug order across
//! repos, log order within one), the fields verbatim, lock-free, and
//! all-or-nothing across the registry like the totals' own read.

use std::fs;

use tempfile::TempDir;
use todo_domain::{RepoSlug, Size, Status, TaskDraft, TaskId};
use todo_store::{DataRoot, ReflectionTrigger, RewardsTotals, Store, TaskPatch};

use super::fixtures::{at, closing_patch, data_root, repo_at};
use super::timeline::{camera_slug, epic_draft, rewards_config, task_draft};

/// The second repo's slug — registered after camera, sorted before
/// it, so the answer's cross-repo order is the registry's own, not
/// the registration's and not the timestamps'.
const ALPHA: &str = "alpha";

/// The alpha slug, parsed.
pub(super) fn alpha_slug() -> RepoSlug {
    ALPHA.parse().unwrap()
}

/// Registers `slug` under `root` with one epic and one open task, and
/// returns the task's id — the unit the events fixture closes.
fn repo_with_task(root: &DataRoot, slug: &str) -> TaskId {
    let mut store = root.register(repo_at(slug)).unwrap();
    let epic = store
        .add_epic(epic_draft("one"), at("2026-09-25T08:05:00Z"))
        .unwrap();
    let task = store
        .add_task(task_draft(&epic.id, "one"), at("2026-09-25T09:00:00Z"))
        .unwrap();
    drop(store);
    task.id
}

/// The events fixture: an enabled root where **camera** (registered
/// first, sorted second) holds two grants and one reflection, and
/// **alpha** (registered second, sorted first) holds one grant and
/// one reflection — so the root order is visible, non-trivial and
/// disagreeing with both the registration order and the timestamps'
/// (alpha's grant is the latest by `ts` yet comes first).
pub(super) fn events_root(dir: &TempDir) -> DataRoot {
    let root = data_root(dir);
    root.save_config(&rewards_config()).unwrap();
    let camera_task = repo_with_task(&root, "camera");
    let alpha_task = repo_with_task(&root, ALPHA);

    // camera: two closures — the first with a reported actual (S),
    // the second without (the default M) — then a reflection.
    let mut camera = root.open(&camera_slug()).unwrap();
    let first = camera
        .close_task(
            camera_task,
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            Some(Size::S),
        )
        .unwrap();
    let second = camera
        .add_task(
            TaskDraft {
                estimate: Some(Size::L),
                ..task_draft(&first.task.epic, "two")
            },
            at("2026-09-25T10:30:00Z"),
        )
        .unwrap();
    camera
        .close_task(
            second.id,
            at("2026-09-25T11:00:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("ran it".to_owned())),
                ..TaskPatch::default()
            },
            None,
        )
        .unwrap();
    drop(camera);
    // alpha: one closure, latest by `ts` — first in root order.
    let mut alpha = root.open(&alpha_slug()).unwrap();
    alpha
        .close_task(
            alpha_task,
            at("2026-09-25T12:00:00Z"),
            closing_patch(),
            None,
        )
        .unwrap();
    drop(alpha);
    root
}

/// The events collect in root order with the events' own fields
/// verbatim: alpha's grant first (the registry's order, not the
/// registration's and not the timestamps'), camera's two after it in
/// log order, and the reflections likewise. Two reads over unchanged
/// bytes are equal (§8).
#[test]
fn the_events_collect_in_root_order_with_verbatim_fields() {
    let dir = TempDir::new().unwrap();
    let root = events_root(&dir);
    // One reflection lands after the grants, below.
    let mut camera = root.open(&camera_slug()).unwrap();
    camera
        .log_reflection(
            "The milestone closed clean.".to_owned(),
            ReflectionTrigger::EpicClosure {
                epic: super::timeline::epic_id(1),
            },
            RewardsTotals::default(),
            at("2026-09-25T13:00:00Z"),
        )
        .unwrap();
    drop(camera);

    let events = root.rewards_events().unwrap();
    let grants: Vec<&TaskId> = events.grants.iter().map(|grant| &grant.task).collect();
    let alpha_task: TaskId = "alpha:U1".parse().unwrap();
    let camera_one: TaskId = "camera:U1".parse().unwrap();
    let camera_two: TaskId = "camera:U2".parse().unwrap();
    assert_eq!(
        grants,
        vec![&alpha_task, &camera_one, &camera_two],
        "root order: alpha first (the registry's order), camera's in log order"
    );
    // The fields verbatim: the reported actual, the default bucket,
    // the multipliers, the audit instants, the repo of each log.
    let alpha_grant = &events.grants[0];
    assert_eq!(alpha_grant.repo, alpha_slug());
    assert_eq!(alpha_grant.amount, 28);
    assert_eq!(alpha_grant.size, Size::M);
    assert_eq!(alpha_grant.priority_pct, 100);
    assert_eq!(alpha_grant.kind_pct, 110);
    assert_eq!(alpha_grant.ts, at("2026-09-25T12:00:00Z"));
    let camera_first = &events.grants[1];
    assert_eq!(camera_first.repo, camera_slug());
    assert_eq!(camera_first.size, Size::S);
    assert_eq!(camera_first.amount, 11);
    assert_eq!(events.grants[2].size, Size::M);
    // The reflection: the answer verbatim, the trigger, the totals as
    // handed, the repo of its log.
    assert_eq!(events.reflections.len(), 1);
    let reflection = &events.reflections[0];
    assert_eq!(reflection.repo, camera_slug());
    assert_eq!(reflection.answer, "The milestone closed clean.");
    assert_eq!(
        reflection.trigger,
        ReflectionTrigger::EpicClosure {
            epic: super::timeline::epic_id(1)
        }
    );
    assert_eq!(reflection.root_total, 0);
    assert_eq!(reflection.repo_total, 0);
    // Determinism (§8): two reads over unchanged bytes are equal.
    assert_eq!(events, root.rewards_events().unwrap());
}

/// The read is lock-free and writes nothing (D19): it succeeds beside
/// a held store lock, and the root's bytes are exactly what they
/// were — the events are the record, and reading them records
/// nothing.
#[test]
fn the_events_read_is_lock_free_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let root = events_root(&dir);
    let _holder = root.open(&camera_slug()).unwrap();
    let before = fs::read(root.log_path(&camera_slug())).unwrap();
    let events = root.rewards_events().unwrap();
    assert_eq!(events.grants.len(), 3);
    assert_eq!(
        fs::read(root.log_path(&camera_slug())).unwrap(),
        before,
        "the log is untouched"
    );
}

/// A repo whose log will not parse fails the whole read — the
/// all-or-nothing reading the totals' own read pins, for the same
/// reason: a repo whose events quietly disappear is a silent
/// omission. The fault is the same positioned `corrupt_store` every
/// read path reports.
#[test]
fn a_damaged_log_fails_the_whole_read() {
    let dir = TempDir::new().unwrap();
    let root = events_root(&dir);
    let alpha_log = root.log_path(&alpha_slug());
    let valid = fs::read(&alpha_log).unwrap();
    let mut damaged = valid.clone();
    damaged.extend_from_slice(b"{{{ not json\n");
    fs::write(&alpha_log, damaged).unwrap();
    let error = root.rewards_events().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("the line is not a parseable event"),
        "the positioned parse fault: {error}"
    );
}

/// An empty root reads empty, and a v1 log — a pre-E13 store, no
/// reward events — reads empty under the same query: the strict
/// superset's read side, over the events.
#[test]
fn an_empty_root_and_a_v1_log_read_empty() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let events = root.rewards_events().unwrap();
    assert!(events.grants.is_empty() && events.reflections.is_empty());

    let v1 = fs::read(crate::common::golden_path("event_log.v1")).unwrap();
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    drop(root.register(repo_at("camera")).unwrap());
    // Replace the freshly created v2 log with the v1 timeline's bytes —
    // the same camera repo, the pre-bump format.
    fs::write(root.log_path(&slug), &v1).unwrap();
    let events = root.rewards_events().unwrap();
    assert!(events.grants.is_empty() && events.reflections.is_empty());
}

/// The events read never consults the config: it is a read over the
/// logs' own bytes, so an inert root's events (hand-logged, nothing
/// asked) still read back — the store records what it was handed
/// (§15), and the enabled gate is the caller's.
#[test]
fn the_events_read_ignores_the_config() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let task = repo_with_task(&root, "camera");
    let mut store = root.open(&camera_slug()).unwrap();
    store
        .log_reflection(
            "Nothing was asked; this is damage's honest record.".to_owned(),
            ReflectionTrigger::Threshold,
            RewardsTotals::default(),
            at("2026-09-25T10:00:00Z"),
        )
        .unwrap();
    drop(store);
    let _ = task;
    let events = root.rewards_events().unwrap();
    assert_eq!(events.reflections.len(), 1);
    assert!(events.grants.is_empty());
}
