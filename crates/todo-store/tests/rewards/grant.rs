//! The closure-aware update's semantics (§15, D23): the grant fires on
//! the transition into `closed` alone, the computation is the domain's
//! single-sourced one, and every no-grant case — `wontfix`, a
//! non-closure, an already-closed task, overflow, an inert root —
//! appends no event.

use std::fs;

use tempfile::TempDir;
use todo_domain::{RewardInput, Size, Status, TaskDraft};

use todo_store::{Store, TaskPatch};

use super::fixtures::{
    at, closing_patch, data_root, disabled_config, enabled_root_with_task, first_epic, open,
    overflowing_config,
};
use super::timeline::{camera_slug, repo, rewards_config, task_draft};

/// The reward the domain computes for the same closure — the
/// single-source cross-check: the store's answer must equal it.
fn domain_reward(size: Option<Size>) -> Option<todo_domain::Reward> {
    todo_domain::reward(
        &rewards_config(),
        RewardInput {
            status: Status::Closed,
            kind: todo_domain::TaskKind::Task,
            priority: None,
            size,
        },
    )
}

#[test]
fn a_closure_transition_grants_the_domains_answer() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);

    // A reported size and the default bucket: both closures' rewards
    // equal the domain's computation over the same inputs.
    let closure = store
        .close_task(
            task.clone(),
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            Some(Size::L),
        )
        .unwrap();
    assert_eq!(closure.reward, domain_reward(Some(Size::L)));
    assert_eq!(closure.task.status, Status::Closed);

    // The event line follows the task_update line, its fields the
    // domain's Reward verbatim.
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    let text = String::from_utf8_lossy(&log);
    let expected = format!(
        "{{\"kind\":\"reward_grant\",\"ts\":\"2026-09-25T10:00:00.000000000Z\",\"task\":\"{task}\",\"amount\":{},\"size\":\"l\",\"priority_pct\":100,\"kind_pct\":110}}\n",
        domain_reward(Some(Size::L)).map(|r| r.amount).unwrap(),
    );
    assert!(text.contains(&expected), "{text}");
    // The grant follows the closure it granted on.
    let update_at = text.find("\"kind\":\"task_update\"").unwrap();
    let grant_at = text.find("\"kind\":\"reward_grant\"").unwrap();
    assert!(update_at < grant_at);
}

#[test]
fn an_absent_actual_size_falls_to_the_default_bucket() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let closure = store
        .close_task(task, at("2026-09-25T10:00:00Z"), closing_patch(), None)
        .unwrap();
    assert_eq!(closure.reward, domain_reward(None));
    assert_eq!(closure.reward.map(|r| r.size), Some(Size::M));
}

#[test]
fn wontfix_grants_nothing() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let closure = store
        .close_task(
            task,
            at("2026-09-25T10:00:00Z"),
            TaskPatch {
                status: Some(Status::Wontfix),
                ..TaskPatch::default()
            },
            Some(Size::Xl),
        )
        .unwrap();
    assert_eq!(closure.reward, None);
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    assert!(!String::from_utf8_lossy(&log).contains("reward_grant"));
}

#[test]
fn a_non_closure_patch_grants_nothing() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    // A patch that does not close: no transition, no grant — the call
    // is still a valid update.
    let closure = store
        .close_task(
            task,
            at("2026-09-25T10:00:00Z"),
            TaskPatch {
                title: Some("Bring up the sensor (renamed)".to_owned()),
                ..TaskPatch::default()
            },
            Some(Size::L),
        )
        .unwrap();
    assert_eq!(closure.reward, None);
    assert_eq!(closure.task.title, "Bring up the sensor (renamed)");
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    assert!(!String::from_utf8_lossy(&log).contains("reward_grant"));
}

#[test]
fn a_patch_on_an_already_closed_task_grants_nothing_twice() {
    // The transition rule: a task already closed is not closing again,
    // so a later patch through the closure-aware path cannot double a
    // grant — one closure, one reward event.
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let first = store
        .close_task(
            task.clone(),
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            None,
        )
        .unwrap();
    assert!(first.reward.is_some());
    let second = store
        .close_task(
            task,
            at("2026-09-25T10:30:00Z"),
            TaskPatch {
                title: Some("Bring up the sensor (final)".to_owned()),
                ..TaskPatch::default()
            },
            None,
        )
        .unwrap();
    assert_eq!(second.reward, None);
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    let text = String::from_utf8_lossy(&log);
    let grants = text.matches("\"kind\":\"reward_grant\"").count();
    assert_eq!(grants, 1);
}

#[test]
fn an_overflowing_config_grants_nothing_and_appends_no_event() {
    // The domain's `None` on intermediate overflow — base × p
    // overflowing with k = 0 grants nothing even though the exact
    // triple product is 0 — and the event emission follows the `None`.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    root.save_config(&overflowing_config()).unwrap();
    let mut store = root.register(repo()).unwrap();
    let epic = store
        .add_epic(
            super::timeline::epic_draft("one"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let task = store
        .add_task(task_draft(&epic.id, "one"), at("2026-09-25T09:00:00Z"))
        .unwrap();
    let closure = store
        .close_task(
            task.id,
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            Some(Size::S),
        )
        .unwrap();
    assert_eq!(closure.reward, None);
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    assert!(!String::from_utf8_lossy(&log).contains("reward_grant"));
}

#[test]
fn the_raw_update_path_never_grants() {
    // `update_task` is the raw patch path: the importer's historical
    // closures ride it, and they grant nothing — rewards are for work
    // closed under the tracker's watch, not for imported history.
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let task = store
        .update_task(task, at("2026-09-25T10:00:00Z"), closing_patch())
        .unwrap();
    assert_eq!(task.status, Status::Closed);
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    assert!(!String::from_utf8_lossy(&log).contains("reward_grant"));
}

#[test]
fn the_estimate_rides_the_creation_event_and_survives_replay() {
    // The honesty anchor's first half: the guess is recorded at
    // creation, explicit-null when absent, and replays back.
    let dir = TempDir::new().unwrap();
    let (root, _task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let estimated = store
        .add_task(
            TaskDraft {
                estimate: Some(Size::Xl),
                ..task_draft(&first_epic(), "the pre-flight guess")
            },
            at("2026-09-25T09:30:00Z"),
        )
        .unwrap();
    assert_eq!(estimated.estimate, Some(Size::Xl));
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    let text = String::from_utf8_lossy(&log);
    // The present spelling, between `until` and `created`.
    assert!(
        text.contains("\"until\":null,\"estimate\":\"xl\",\"created\""),
        "{text}"
    );
    // The absent spelling: the fixture world's other task carries the
    // explicit null.
    assert!(
        text.contains("\"until\":null,\"estimate\":null,\"created\""),
        "{text}"
    );
    drop(store);
    let reopened = open(&root);
    assert_eq!(
        reopened.graph().task(&estimated.id).unwrap().estimate,
        Some(Size::Xl)
    );
}

#[test]
fn a_rejected_closure_grants_nothing() {
    // The domain's closure discipline runs first: `closed` without
    // evidence is `missing_evidence`, the write is refused whole, and
    // no reward event exists to append.
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let log_before = fs::read(root.log_path(&camera_slug())).unwrap();
    let error = store
        .close_task(
            task.clone(),
            at("2026-09-25T10:00:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                ..TaskPatch::default()
            },
            None,
        )
        .unwrap_err();
    assert_eq!(error.code(), "missing_evidence");
    assert_eq!(fs::read(root.log_path(&camera_slug())).unwrap(), log_before);
}

#[test]
fn the_disabled_gate_is_read_at_the_grant_not_the_load() {
    // The domain's contract: `reward` computes regardless of `enabled`
    // — the gate is the caller's. The store reads it: a disabled
    // config's closure grants nothing while the config itself loads.
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    root.save_config(&disabled_config()).unwrap();
    let mut store = open(&root);
    let closure = store
        .close_task(task, at("2026-09-25T10:00:00Z"), closing_patch(), None)
        .unwrap();
    assert_eq!(closure.reward, None);
}
