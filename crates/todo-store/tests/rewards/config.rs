//! The per-root config document: its golden bytes, its load/save
//! contract, its faults, and the inert root (§15, D23; §4's versioning
//! and §4.3's atomic-rename discipline).
//!
//! The golden is written through the real path — `save_config` into a
//! `tempfile` root — so the bytes on disk, not a hand-serialized
//! document, are what the fixture pins.

use std::fs;

use tempfile::TempDir;
use todo_domain::Status;
use todo_store::{FORMAT_VERSION, Store, StoreError, TaskPatch};

use super::fixtures::{
    at, closing_patch, data_root, disabled_config, enabled_root_with_task, open, root_with_task,
};
use super::timeline::{camera_slug, rewards_config};
use crate::common::assert_golden;

// =================================================================
// The golden.
// =================================================================

#[test]
fn the_saved_config_matches_its_golden() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    root.save_config(&rewards_config()).unwrap();
    // The golden name embeds FORMAT_VERSION, so bumping the format
    // without landing the new golden fails as "missing golden".
    assert_golden(
        &format!("config.v{FORMAT_VERSION}"),
        &fs::read(root.config_path()).unwrap(),
    );
}

// =================================================================
// The inert root (§15's whole safety).
// =================================================================

#[test]
fn an_absent_config_leaves_the_root_inert() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let task = root_with_task(&root);

    // Absent: `None`, and the read created nothing.
    assert_eq!(root.load_config().unwrap(), None);
    assert!(!root.config_path().exists());

    // The inert root's closure is byte-identical to the raw update:
    // one `task_update` line, no reward event, no reward in the answer
    // (§15: "a root with no rewards config is byte-identical to a root
    // that never heard of the feature").
    let mut store = open(&root);
    let log_before = fs::read(root.log_path(&camera_slug())).unwrap();
    let closure = store
        .close_task(task, at("2026-09-25T10:00:00Z"), closing_patch(), None)
        .unwrap();
    assert_eq!(closure.reward, None);
    let log_after = fs::read(root.log_path(&camera_slug())).unwrap();
    let appended = &log_after[log_before.len()..];
    assert_eq!(
        String::from_utf8_lossy(appended),
        "{\"kind\":\"task_update\",\"ts\":\"2026-09-25T10:00:00.000000000Z\",\"id\":\"camera:U1\",\"patch\":{\"status\":\"closed\",\"evidence\":\"42 passed, 0 failed\"}}\n"
    );
}

#[test]
fn a_disabled_config_leaves_the_root_inert() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    root.save_config(&disabled_config()).unwrap();
    // The disabled config loads — the gate is read at the grant, not
    // refused at the load.
    assert_eq!(root.load_config().unwrap(), Some(disabled_config()));

    let mut store = open(&root);
    let closure = store
        .close_task(task, at("2026-09-25T10:00:00Z"), closing_patch(), None)
        .unwrap();
    assert_eq!(closure.reward, None);
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    assert!(!String::from_utf8_lossy(&log).contains("reward_grant"));
}

// =================================================================
// Load and save.
// =================================================================

#[test]
fn save_is_canonical_and_byte_stable() {
    let dir_a = TempDir::new().unwrap();
    let root_a = data_root(&dir_a);
    root_a.save_config(&rewards_config()).unwrap();
    let first = fs::read(root_a.config_path()).unwrap();

    // §8's serialize → load → serialize: the loaded config saves back
    // to identical bytes, and a second root's save agrees.
    let loaded = root_a.load_config().unwrap().unwrap();
    root_a.save_config(&loaded).unwrap();
    assert_eq!(fs::read(root_a.config_path()).unwrap(), first);

    let dir_b = TempDir::new().unwrap();
    let root_b = data_root(&dir_b);
    root_b.save_config(&rewards_config()).unwrap();
    assert_eq!(fs::read(root_b.config_path()).unwrap(), first);

    // One compact document, one line: exactly one trailing LF, the
    // version header leading, the rewards block in the domain's
    // declaration order.
    let text = String::from_utf8_lossy(&first);
    assert!(text.ends_with("}\n"), "the document ends in }} plus one LF");
    assert!(!text.ends_with("\n\n"));
    assert!(
        text.starts_with("{\"format_version\":2,\"rewards\":{\"enabled\":true,"),
        "{text}"
    );
}

#[test]
fn save_leaves_no_temp_debris() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    root.save_config(&rewards_config()).unwrap();
    // The root holds the config and the registry lock file the save
    // took — the pid-suffixed temp was consumed by the atomic rename.
    let mut names: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["config.json", "registry.json.lock"]);
}

// =================================================================
// Faults: a config this build cannot parse is a named error, never a
// silent inert root.
// =================================================================

#[test]
fn config_faults_are_named_errors() {
    let faulty = |bytes: &[u8]| -> StoreError {
        let dir = TempDir::new().unwrap();
        let root = data_root(&dir);
        fs::write(root.config_path(), bytes).unwrap();
        let error = root.load_config().unwrap_err();
        assert_eq!(error, root.load_config().unwrap_err(), "{error}");
        error
    };

    // Malformed bytes: a `corrupt_store` under the fixed context, with
    // no path in the detail.
    let error = faulty(b"{{{ not json\n");
    assert_eq!(error.code(), "corrupt_store");
    match &error {
        StoreError::CorruptStore { detail, .. } => {
            assert!(
                detail.contains("the config is not a parseable document"),
                "{error}"
            );
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected a corrupt_store, got {other:?}"),
    }

    // A foreign version is the migration question — in either
    // direction: this document is born at v2 and has no predecessor
    // spelling, so a v1 declaration is as much a version this build
    // does not read as a v3 one.
    for version in [1u32, 3] {
        let error = faulty(format!("{{\"format_version\":{version},\"rewards\":{{}}}}").as_bytes());
        assert_eq!(
            error,
            StoreError::VersionMismatch {
                found: version,
                expected: FORMAT_VERSION
            },
            "version {version}"
        );
        assert_eq!(error.code(), "version_mismatch");
    }

    // `deny_unknown_fields`: an unknown field at the current version
    // can only be damage.
    let error = faulty(br#"{"format_version":2,"rewards":{"enabled":true},"surprise":true}"#);
    assert_eq!(error.code(), "corrupt_store");
}

#[test]
fn a_malformed_config_fails_the_closure_atomically() {
    // The grant path reads the config before anything is applied, so a
    // malformed document fails the whole call with nothing written —
    // not a closure that silently skipped its reward.
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    fs::write(root.config_path(), b"{{{ not json\n").unwrap();

    let mut store = open(&root);
    let log_before = fs::read(root.log_path(&camera_slug())).unwrap();
    let error = store
        .close_task(
            task.clone(),
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            None,
        )
        .unwrap_err();
    assert_eq!(error.code(), "corrupt_store");
    assert!(
        error
            .to_string()
            .contains("the config is not a parseable document"),
        "{error}"
    );
    // Atomic: nothing was applied, nothing appended.
    assert_eq!(fs::read(root.log_path(&camera_slug())).unwrap(), log_before);
    assert_eq!(store.graph().task(&task).unwrap().status, Status::Open);
}

// =================================================================
// The config is read fresh: a change takes effect without a reopen.
// =================================================================

#[test]
fn a_config_saved_after_the_open_takes_effect_on_the_next_closure() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);

    // Disable the config while the store is open: the next closure
    // reads the fresh document and grants nothing.
    root.save_config(&disabled_config()).unwrap();
    let closure = store
        .close_task(
            task.clone(),
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            None,
        )
        .unwrap();
    assert_eq!(closure.reward, None);

    // Re-open the task and re-enable: the next closure grants — the
    // store never cached the document it read at the first closure.
    root.save_config(&rewards_config()).unwrap();
    store
        .update_task(
            task.clone(),
            at("2026-09-25T10:30:00Z"),
            TaskPatch {
                status: Some(Status::Open),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    let closure = store
        .close_task(task, at("2026-09-25T11:00:00Z"), closing_patch(), None)
        .unwrap();
    assert!(closure.reward.is_some());
}
