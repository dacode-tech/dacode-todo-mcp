//! The registry surgery's tests (D20, t70): `DataRoot::unregister` —
//! the entry and the directory, under one lock, in the crash-safe
//! order. The order is the argument, and the pins follow it:
//!
//! - the success state is the **clean slate** (neither the entry nor
//!   the directory remains) — what a crash after the removal leaves;
//! - the removal's *failure* (the store directory chmod'd read-only)
//!   leaves the **orphan-dir state** — the entry already gone, the
//!   directory remaining — never the **dangling entry** the reverse
//!   order could leave, which breaks `repo_list` for every repo (the
//!   all-or-nothing rule). The failure is induced, not constructed:
//!   the real step refuses, which is the f19 discipline;
//! - a live writer (a held store handle) is refused `locked` with
//!   **nothing touched** — the entry still listed, the directory
//!   intact (§8's single-writer discipline; the surgery never
//!   orphans a live writer's store out from under it).

use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::fixtures::{camera, data_root, register_and_release, repo, slug};
use tempfile::TempDir;

/// Registers camera and a sibling, so every surgery has a survivor to
/// answer for — the dangling-entry failure mode is about `repo_list`
/// breaking for *every* repo, and an empty root cannot show that.
fn root_with_two() -> (TempDir, todo_store::DataRoot) {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    register_and_release(&root, camera());
    register_and_release(&root, repo("shelly-ng", "2026-09-25T09:00:00Z"));
    (dir, root)
}

/// The success state: the entry gone, the directory gone, the sibling
/// untouched, `repo_list` answering the survivor — the clean slate a
/// crash after the removal leaves, reached without crashing.
#[test]
fn unregister_removes_the_entry_and_the_directory() {
    let (dir, root) = root_with_two();
    root.unregister(&slug("camera")).unwrap();
    // The registry still exists (the sibling is listed); camera's
    // directory is what must be gone.
    assert!(root.registry_path().exists());
    assert!(!dir.path().join("repos").join("camera").exists());
    assert!(dir.path().join("repos").join("shelly-ng").exists());
    let listed = root.repo_list().unwrap();
    assert_eq!(
        listed
            .iter()
            .map(|repo| repo.slug.as_str())
            .collect::<Vec<_>>(),
        vec!["shelly-ng"]
    );
}

/// Idempotent: an unlisted slug with no directory is `Ok` and changes
/// nothing — a maintenance op's safer contract. The fresh-root half
/// pins the write-if-changed discipline: an unlisted slug rewrites
/// nothing, so a root with no registry still has none afterwards
/// (only the lock file, which `lock_registry` creates).
#[test]
fn unregister_is_idempotent_for_an_unlisted_slug() {
    let (dir, root) = root_with_two();
    root.unregister(&slug("elsewhere")).unwrap();
    assert!(dir.path().join("repos").join("camera").exists());
    let listed = root.repo_list().unwrap();
    assert_eq!(listed.len(), 2);

    let fresh = TempDir::new().unwrap();
    let fresh_root = data_root(&fresh);
    fresh_root.unregister(&slug("elsewhere")).unwrap();
    assert!(
        !fresh_root.registry_path().exists(),
        "an unlisted slug rewrites nothing — no registry appears"
    );
}

/// The orphan cleanup: an unlisted directory (what a crashed register
/// leaves) is removed — the "cleaned by the next refresh" property.
#[test]
fn unregister_cleans_an_orphan_directory() {
    let (dir, root) = root_with_two();
    let orphan = dir.path().join("repos").join("orphan");
    fs::create_dir_all(&orphan).unwrap();
    fs::write(orphan.join("event-log.jsonl"), b"debris\n").unwrap();
    root.unregister(&slug("orphan")).unwrap();
    assert!(!orphan.exists());
    let listed = root.repo_list().unwrap();
    assert_eq!(listed.len(), 2);
}

/// The crash-order pin, fault-induced: the removal's failure (the
/// store directory chmod'd read-only, so `remove_dir_all` cannot
/// unlink its children) leaves the **orphan state** — the entry
/// already dropped by the registry rewrite that ran first, the
/// directory remaining — and `repo_list` still answers the sibling.
/// The reverse order would leave the dangling entry (a listed repo
/// whose store is gone), which `repo_list`'s all-or-nothing rule
/// turns into a failure for every repo; this test is the pin that
/// the order prevents it.
#[test]
fn the_removal_failure_leaves_the_orphan_state_never_the_dangling_entry() {
    let (dir, root) = root_with_two();
    let store_dir = dir.path().join("repos").join("camera");
    fs::set_permissions(&store_dir, fs::Permissions::from_mode(0o550)).unwrap();
    let outcome = root.unregister(&slug("camera"));
    fs::set_permissions(&store_dir, fs::Permissions::from_mode(0o750)).unwrap();
    let error = outcome.unwrap_err();
    assert!(
        error.to_string().contains("remove the store directory"),
        "the refusal names the step: {error}"
    );
    // The orphan state: the entry is gone, the directory remains.
    assert!(store_dir.exists());
    let listed = root.repo_list().unwrap();
    assert_eq!(
        listed
            .iter()
            .map(|repo| repo.slug.as_str())
            .collect::<Vec<_>>(),
        vec!["shelly-ng"],
        "repo_list answers the sibling over the orphan state"
    );
}

/// A live writer is refused `locked` with nothing touched: the entry
/// still listed, the directory intact — the surgery takes the store's
/// own lock before it rewrites anything, so a live session's store is
/// never silently destroyed out from under it.
#[test]
fn a_live_writer_is_refused_with_nothing_touched() {
    let (dir, root) = root_with_two();
    // The live writer: a locked, open handle on camera's store.
    let held = root.open(&slug("camera")).unwrap();
    let outcome = root.unregister(&slug("camera"));
    let error = outcome.unwrap_err();
    assert!(
        error.to_string().contains("store is locked"),
        "the refusal is the locked code: {error}"
    );
    // Nothing touched: the entry still listed (read from the registry
    // directly — `repo_list` would refuse on the held lock, which is
    // its own all-or-nothing discipline, not damage), the directory
    // intact.
    let registry = todo_store::Registry::load(&root).unwrap();
    assert!(registry.contains(&slug("camera")));
    assert!(dir.path().join("repos").join("camera").exists());
    drop(held);
    // With the writer gone, the store still opens — nothing was
    // destroyed out from under it.
    drop(root.open(&slug("camera")).unwrap());
}
