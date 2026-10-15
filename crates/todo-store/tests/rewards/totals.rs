//! The derived per-root totals (§15, D23): §2's iterate-the-registry
//! pattern over the reward events — never stored, lock-free, and
//! ordered by the registry's own ascending-slug order.

use std::fs;

use tempfile::TempDir;
use todo_domain::{RepoSlug, Size, Status, TaskDraft};

use todo_store::{DataRoot, Store, TaskPatch};

use super::fixtures::{at, closing_patch, data_root, open, repo_at};
use super::timeline::{camera_slug, epic_draft, rewards_config, task_draft};

/// Registers `slug` under `root` with one epic and one open task, and
/// returns the task's id — the unit the multi-repo totals sum over.
fn repo_with_task(root: &DataRoot, slug: &str) -> todo_domain::TaskId {
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

#[test]
fn the_totals_sum_reward_events_across_the_registry() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    root.save_config(&rewards_config()).unwrap();

    // Three repos, registered out of slug order: the answer's order is
    // the registry's own, not the registration's.
    let camera_task = repo_with_task(&root, "camera");
    let shelly_task = repo_with_task(&root, "shelly-ng");
    let ikvm_task = repo_with_task(&root, "ikvm-html");

    // camera grants twice (28 + 28 = 56: task × no-priority × the
    // default M, 25 × 100 × 110 / 10000 → 27.5 → 28), shelly once
    // (28), ikvm never — its task stays open, and a repo with no
    // rewards still appears, zero included.
    let mut camera = open(&root);
    let first = camera
        .close_task(
            camera_task,
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            None,
        )
        .unwrap();
    let second_task = camera
        .add_task(
            TaskDraft {
                estimate: Some(Size::S),
                ..task_draft(&first.task.epic, "two")
            },
            at("2026-09-25T10:30:00Z"),
        )
        .unwrap();
    camera
        .close_task(
            second_task.id,
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
    let shelly_slug: RepoSlug = "shelly-ng".parse().unwrap();
    let mut shelly = root.open(&shelly_slug).unwrap();
    shelly
        .close_task(
            shelly_task,
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            None,
        )
        .unwrap();
    drop(shelly);
    let ikvm_slug: RepoSlug = "ikvm-html".parse().unwrap();
    assert!(
        root.open(&ikvm_slug)
            .unwrap()
            .graph()
            .task(&ikvm_task)
            .is_some()
    );

    let camera = camera_slug();
    let shelly: RepoSlug = "shelly-ng".parse().unwrap();
    let totals = root.rewards_totals().unwrap();
    assert_eq!(totals.total, 84);
    let expected: Vec<(&RepoSlug, u64)> = vec![(&camera, 56), (&ikvm_slug, 0), (&shelly, 28)];
    assert_eq!(
        totals
            .by_repo
            .iter()
            .map(|(slug, total)| (slug, *total))
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn the_totals_are_never_stored_and_write_nothing() {
    // The events are the record: the query reads the logs and the
    // registry, and the root's bytes are exactly what they were.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    root.save_config(&rewards_config()).unwrap();
    let task = repo_with_task(&root, "camera");
    let mut store = open(&root);
    store
        .close_task(task, at("2026-09-25T10:00:00Z"), closing_patch(), None)
        .unwrap();
    drop(store);

    let before: Vec<(std::path::PathBuf, Vec<u8>)> = vec![
        (root.config_path(), fs::read(root.config_path()).unwrap()),
        (
            root.registry_path(),
            fs::read(root.registry_path()).unwrap(),
        ),
        (
            root.log_path(&camera_slug()),
            fs::read(root.log_path(&camera_slug())).unwrap(),
        ),
    ];
    let totals = root.rewards_totals().unwrap();
    assert_eq!(totals.total, 28);
    for (path, bytes) in &before {
        assert_eq!(
            &fs::read(path).unwrap(),
            bytes,
            "{} is untouched",
            path.display()
        );
    }
}

#[test]
fn an_empty_root_totals_zero() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let totals = root.rewards_totals().unwrap();
    assert_eq!(totals.total, 0);
    assert!(totals.by_repo.is_empty());
}

#[test]
fn a_v1_log_totals_zero() {
    // The frozen v1 golden's bytes — a pre-E13 store, no reward events
    // — total zero under the same query: the strict superset's read
    // side, over the totals.
    let v1 = fs::read(crate::common::golden_path("event_log.v1")).unwrap();
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let slug = camera_slug();
    drop(root.register(repo_at("camera")).unwrap());
    // Replace the freshly created v2 log with the v1 timeline's bytes —
    // the same camera repo, the pre-bump format.
    fs::write(root.log_path(&slug), &v1).unwrap();

    let totals = root.rewards_totals().unwrap();
    assert_eq!(totals.total, 0);
    assert_eq!(totals.by_repo.get(&slug), Some(&0));
}
