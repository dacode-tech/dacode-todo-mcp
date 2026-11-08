//! The rewards suite's shared fixtures: the config variants the grant
//! path is exercised over, and the enabled-root builders. The base
//! builders (`repo`, `task_draft`, `at`, `epic_draft`) come from the
//! log suite's fixture file, included read-only by the suite root.
//!
//! Nothing here is a test, so this module contributes no test names of
//! its own (P-LIVE-002). Every value is caller-passed and
//! deterministic: no clock, no environment, no absolute host path (§8).

use tempfile::TempDir;
use todo_domain::{
    BaseCosts, KindMultipliers, PriorityMultipliers, Repo, RewardsConfig, Status, TaskDraft,
    TaskId, Timestamp,
};
use todo_store::{DataRoot, JsonlStore, Store, TaskPatch};

use super::timeline::{camera_slug, epic_draft, epic_id, repo, rewards_config, task_draft};

/// A data root injected over `dir`'s path — the suite's whole input,
/// and the only way these tests reach a filesystem location.
pub(super) fn data_root(dir: &TempDir) -> DataRoot {
    DataRoot::new(dir.path())
}

/// A repository under `slug`, registered at the timeline's own fixed
/// instant — the multi-repo totals fixture's repos.
pub(super) fn repo_at(slug: &str) -> Repo {
    Repo {
        slug: slug.parse().unwrap(),
        name: format!("{slug} repository"),
        path: format!("/srv/repos/{slug}"),
        registered_at: at("2026-09-25T08:00:00Z"),
    }
}

/// A fixed audit instant — caller-passed, never a clock (§8).
pub(super) fn at(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

/// The closure patch that closes a task with evidence.
pub(super) fn closing_patch() -> TaskPatch {
    TaskPatch {
        status: Some(Status::Closed),
        evidence: Some(Some("42 passed, 0 failed".to_owned())),
        ..TaskPatch::default()
    }
}

/// The disabled config: §15's other inert spelling — the document is
/// there, the root still grants nothing.
pub(super) fn disabled_config() -> RewardsConfig {
    RewardsConfig {
        enabled: false,
        ..rewards_config()
    }
}

/// A config whose S base, `high` priority and `task` kind overflow on
/// the first multiplication: the domain's `None`, and the store's
/// no-event, are the pinned answers.
pub(super) fn overflowing_config() -> RewardsConfig {
    RewardsConfig {
        base_costs: BaseCosts {
            s: u64::MAX,
            ..rewards_config().base_costs
        },
        priority_multipliers: PriorityMultipliers {
            high: 2,
            ..rewards_config().priority_multipliers
        },
        kind_multipliers: KindMultipliers {
            task: 2,
            ..rewards_config().kind_multipliers
        },
        ..rewards_config()
    }
}

/// An enabled root holding one epic and one open task under it, ready
/// to close — the base world the grant tests close over. Returns the
/// root and the task's id.
pub(super) fn enabled_root_with_task(dir: &TempDir) -> (DataRoot, TaskId) {
    let root = data_root(dir);
    root.save_config(&rewards_config()).unwrap();
    let task = root_with_task(&root);
    (root, task)
}

/// A root's store holding one epic and one open task — the world
/// without the config half, for the inert-root tests.
pub(super) fn root_with_task(root: &DataRoot) -> TaskId {
    let mut store = root.register(repo()).unwrap();
    let epic = store
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    let task = store
        .add_task(
            TaskDraft {
                epic: epic.id.clone(),
                ..task_draft(&epic.id, "Bring up the sensor")
            },
            at("2026-09-25T09:00:00Z"),
        )
        .unwrap();
    drop(store);
    task.id
}

/// The root's store, opened through the layout so the config document
/// is attached as the rewards source.
pub(super) fn open(root: &DataRoot) -> JsonlStore {
    root.open(&camera_slug()).unwrap()
}

/// The first epic's id — the fixture world's only epic.
pub(super) fn first_epic() -> todo_domain::EpicId {
    epic_id(1)
}
