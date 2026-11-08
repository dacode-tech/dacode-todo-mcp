//! The bare [`Store::snapshot`]'s refusal branches (f57): the write
//! path's two fault seams driven at the call every other pin reaches
//! only through `compact`'s step (1) (t21). The creation-refused
//! branch — the store directory refusing a new file — leaves no
//! debris at all; the rename-refused branch — a directory parked at
//! the snapshot path — leaves exactly one pid-suffixed temporary,
//! reused under the same name by this process's next attempt. Both
//! keep the promise `Store::snapshot`'s doc makes: not session-fatal
//! — the log is byte-identical, the append handle still lands writes,
//! and the store recovers the moment the fault is removed.

use std::fs;

use tempfile::TempDir;
use todo_store::{JsonlStore, Store, StoreError};

use super::fixtures::{build_golden_store, dir_entries, snapshot_path};
use super::timeline::{at, epic_id, log_path, task_draft};

/// The `.tmp` names in the store directory, sorted — the debris
/// instrument. The pid-suffixed temporary is §4.3's accepted debris of
/// a failed attempt (f13), pinned here so the "untouched" claims stay
/// precise about what they claim.
fn temporaries(dir: &TempDir) -> Vec<String> {
    dir_entries(dir.path())
        .into_iter()
        .filter(|name| name.ends_with(".tmp"))
        .collect()
}

// Gated like the layout suite's permission corners (lock.rs): the
// chmod seam is unix-only, and an ungated import would fail the whole
// target's compile elsewhere.
#[cfg(unix)]
#[test]
fn a_snapshot_refused_at_temporary_creation_is_io_with_no_debris() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    let before = fs::read(&path).unwrap();

    // The fault seam: the store directory refuses a new file, so the
    // temporary's creation itself fails — the branch no other test
    // drives (t21's seam is the rename, one step later). A snapshot
    // has no lock of its own (Decision 2), so nothing but the open
    // notices this.
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o500)).unwrap();

    let error = store.snapshot().unwrap_err();
    // §6.1's `io`; the context names which write failed. The context
    // is this crate's own contract text, so it is pinned; the
    // `ErrorKind` spelling beside it is the platform's, so it is not.
    assert_eq!(error.code(), "io");
    match &error {
        StoreError::Io { detail } => {
            assert!(
                detail.contains("write the snapshot's temporary file"),
                "the temporary's creation refused: {error}"
            );
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected an io fault, got {other:?}"),
    }

    // The creation is what refused, so nothing was ever written: no
    // temporary exists, and the log is byte-for-byte unchanged.
    assert_eq!(temporaries(&dir), Vec::<String>::new());
    assert_eq!(fs::read(&path).unwrap(), before);

    // The fault removed, the same call succeeds — the refusal was the
    // permission, not a broken store — and the store reopens from the
    // snapshot it wrote.
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o750)).unwrap();
    store.snapshot().unwrap();
    assert!(snapshot_path(&path).is_file());
    assert_eq!(temporaries(&dir), Vec::<String>::new());
    let graph = store.graph().clone();
    drop(store);
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &graph);
}

#[test]
fn a_snapshot_refused_at_rename_leaves_one_reusable_temporary() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = build_golden_store(&path);
    let before = fs::read(&path).unwrap();

    // The fault seam (t21's, driven at the bare call): a directory
    // parked at the snapshot path is a rename target `rename(2)`
    // refuses.
    fs::create_dir(snapshot_path(&path)).unwrap();

    let error = store.snapshot().unwrap_err();
    assert_eq!(error.code(), "io");
    match &error {
        StoreError::Io { detail } => {
            assert!(
                detail.contains("rename the snapshot into place"),
                "the snapshot's rename refused: {error}"
            );
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected an io fault, got {other:?}"),
    }

    // The log is byte-for-byte unchanged and the debris is exactly one
    // pid-suffixed temporary — the name this process's next attempt
    // truncates.
    assert_eq!(fs::read(&path).unwrap(), before);
    let first = temporaries(&dir);
    assert_eq!(
        first.len(),
        1,
        "the refused rename leaves exactly one temporary"
    );

    // Not session-fatal: the append handle still lands writes in this
    // log (the snapshot path never replaced the log's inode).
    let task = store
        .add_task(
            task_draft(&epic_id(1), "one after the refused snapshot"),
            at("2026-09-25T19:00:00Z"),
        )
        .unwrap();
    let grown = fs::read(&path).unwrap();
    assert!(grown.starts_with(&before), "the append landed in this log");
    assert!(grown.len() > before.len());

    // A second refusal in the same process reuses the same temporary —
    // the pid-suffixed name is per process, so the debris count stays
    // one and the name stays the same: the reuse the store's doc
    // claims, pinned by name.
    let error_again = store.snapshot().unwrap_err();
    assert_eq!(error_again, error);
    assert_eq!(
        temporaries(&dir),
        first,
        "the same process reuses the temporary"
    );

    // The fault removed: the snapshot lands, consumes the temporary,
    // and the store reopens with the grown graph — burned numbers
    // included.
    fs::remove_dir(snapshot_path(&path)).unwrap();
    store.snapshot().unwrap();
    assert!(snapshot_path(&path).is_file());
    assert_eq!(temporaries(&dir), Vec::<String>::new());
    let graph = store.graph().clone();
    drop(store);
    let reopened = JsonlStore::open_log(&path).unwrap();
    assert_eq!(reopened.graph(), &graph);
    assert_eq!(reopened.graph().task(&task.id), Some(&task));
}
