//! §8's single writer, exercised through the layout: a second writer is
//! refused with `locked`, the lock is released by `Drop`, a failed open
//! never holds one, the registry lock contends the same way, and
//! `open_many` acquires in sorted order and is all-or-nothing.
//!
//! Contention needs no second process: two descriptors on one lock file
//! conflict even inside one process (measured M1), so a held store and a
//! refused open live in the same test.

use std::fs;

use tempfile::TempDir;
use todo_store::{JsonlStore, LockTarget, Store, StoreError};

use super::fixtures::{at, camera, data_root, epic_draft, register_and_release, repo, slug};

// =================================================================
// A second writer is refused with `locked` (required test 4).
// =================================================================

#[test]
fn a_second_writer_is_refused_with_locked_and_the_first_keeps_writing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let mut held = root.register(camera()).unwrap();

    // Opening the same slug while one store holds it is §6.1's `locked`,
    // naming the repo — never a path.
    let error = root.open(&camera_slug).unwrap_err();
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(camera_slug.clone())
        }
    );
    assert_eq!(error.code(), "locked");
    assert_eq!(error.to_string(), "store is locked: repo camera");
    assert!(!error.to_string().contains('/'), "{error} carries no path");
    assert_eq!(error.domain(), None);

    // The trait's constructor half reports the same refusal — the lock is
    // the engine's, whichever door the caller came through.
    let via_trait = <JsonlStore as Store>::open(&root, &camera_slug).unwrap_err();
    assert_eq!(via_trait, error);

    // The refusal cost the holder nothing: it still appends, and the line
    // reaches disk under its own lock.
    let epic = held
        .add_epic(
            epic_draft("Ship the sensor pipeline"),
            at("2026-09-25T08:05:00Z"),
        )
        .unwrap();
    assert_eq!(epic.id.to_string(), "camera:E1");
    drop(held);
    let bytes = fs::read(root.log_path(&camera_slug)).unwrap();
    assert!(
        String::from_utf8_lossy(&bytes).contains("\"kind\":\"epic_add\""),
        "the append landed"
    );
}

// =================================================================
// The lock is released by Drop (required test 5).
// =================================================================

#[test]
fn dropping_the_store_releases_its_lock() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");

    let first = root.register(camera()).unwrap();
    // Held: a second open is refused.
    assert!(matches!(
        root.open(&camera_slug).unwrap_err(),
        StoreError::Locked { .. }
    ));
    // This is the property the whole RAII design rests on (measured M1):
    // dropping the handle closes the descriptor, and closing is the
    // release — so the very next open succeeds.
    drop(first);
    let second = root.open(&camera_slug).unwrap();
    assert_eq!(second.graph().repo().slug, camera_slug);
    drop(second);
    // And again, so the release is not a one-shot.
    assert!(root.open(&camera_slug).is_ok());
}

// =================================================================
// A failed open holds no lock (required test 6).
// =================================================================

#[test]
fn a_failed_open_holds_no_lock() {
    // A corrupt log: opening it fails, and the guard must drop on the
    // error path so the *same* path opens to the *same* positioned error
    // a second time — never `locked`. The log suite's `expect_corrupt`
    // (two opens, one assertion) depends on exactly this (Decision 3/4).
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    register_and_release(&root, camera());
    let log = root.log_path(&camera_slug);
    let valid = fs::read(&log).unwrap();
    // Tear the last line: a committed log minus its final newline.
    fs::write(&log, &valid[..valid.len() - 1]).unwrap();

    let first = root.open(&camera_slug).unwrap_err();
    let second = root.open(&camera_slug).unwrap_err();
    assert_eq!(first, second, "a failed open is reproducible, not a lock");
    match &first {
        StoreError::CorruptStore {
            offset: Some(_),
            detail,
        } => assert!(!detail.contains('/'), "{detail} carries no path"),
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }
    assert_eq!(first.code(), "corrupt_store");

    // A missing log, twice: the same honest io NotFound both times, never
    // a stale lock from the first attempt, and the read path created
    // nothing on the way (Decisions 8 and 11).
    let missing_dir = TempDir::new().unwrap();
    let missing_root = data_root(&missing_dir);
    let absent = slug("never-registered");
    let one = missing_root.open(&absent).unwrap_err();
    let two = missing_root.open(&absent).unwrap_err();
    assert_eq!(one, two);
    assert_eq!(
        one,
        StoreError::Io {
            detail: "open the event log: NotFound".to_owned()
        }
    );
    assert!(!missing_root.store_dir(&absent).exists());
}

// =================================================================
// The registry lock contends the same way (required test 7).
// =================================================================

#[test]
fn the_registry_lock_refuses_a_concurrent_register() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);

    // Holding the registry's guard, a register is refused with `locked`
    // naming the registry — the guard is the whole mechanism (Decision
    // 13), shared by the contention test and the real writer.
    let guard = root.lock_registry().unwrap();
    let error = root.register(camera()).unwrap_err();
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Registry
        }
    );
    assert_eq!(error.code(), "locked");
    assert_eq!(error.to_string(), "store is locked: the registry");
    assert!(!error.to_string().contains('/'), "{error} carries no path");
    // The refused register wrote no store.
    assert!(!root.store_dir(&slug("camera")).exists());

    // Drop the guard and the same register succeeds: the release is real.
    drop(guard);
    let store = root.register(camera()).unwrap();
    assert_eq!(store.graph().repo().slug, slug("camera"));
}

// =================================================================
// open_many: sorted order, dedupe, all-or-nothing (required test 8).
// =================================================================

#[test]
fn open_many_dedupes_sorts_and_is_all_or_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let alpha = slug("alpha");
    let beta = slug("beta");
    let gamma = slug("gamma");
    for name in ["alpha", "beta", "gamma"] {
        register_and_release(&root, repo(name, "2026-09-25T08:00:00Z"));
    }

    // Dedupe plus sort: `gamma, alpha, alpha, beta` opens three stores in
    // ascending slug order. Dedupe is required, not cosmetic — two opens
    // of one path are two descriptors, and a second descriptor conflicts
    // with the first (measured M1), so a duplicate slug would self-`locked`.
    // The slice-of-references shape is D-B's: `&registry.slugs()` passes
    // straight in, and here a literal of borrows needs no clones.
    let stores = root.open_many(&[&gamma, &alpha, &alpha, &beta]).unwrap();
    assert_eq!(stores.len(), 3);
    let slugs: Vec<_> = stores
        .iter()
        .map(|store| store.graph().repo().slug.clone())
        .collect();
    assert_eq!(slugs, vec![alpha.clone(), beta.clone(), gamma.clone()]);
    drop(stores);

    // The required shape: holding the first slug in sorted order,
    // `open_many(&[beta, alpha])` fails on `alpha` at once, and `beta` —
    // never reached — is not left locked.
    let held_alpha = root.open(&alpha).unwrap();
    let error = root.open_many(&[&beta, &alpha]).unwrap_err();
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(alpha.clone())
        }
    );
    assert!(root.open(&beta).is_ok(), "beta must not be left locked");
    drop(held_alpha);

    // The rollback proof (judgment call (c)): a failure *after* a partial
    // acquisition releases everything already held. While `gamma` is held,
    // `[alpha, beta, gamma]` acquires alpha and beta first and only then
    // fails — and both are openable again immediately, before `gamma`'s
    // holder drops, so nothing the failed call took stayed taken. The
    // release is RAII (the local vector's drop), not an explicit unwind
    // (F10): this test is what pins the observable result.
    let held_gamma = root.open(&gamma).unwrap();
    let error = root.open_many(&[&alpha, &beta, &gamma]).unwrap_err();
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(gamma.clone())
        }
    );
    let rolled_back = root.open_many(&[&alpha, &beta]).unwrap();
    assert_eq!(rolled_back.len(), 2);
    drop(rolled_back);

    // `gamma` is still held throughout — the failed call rolled back its
    // own acquisitions, not the unrelated holder's — and releases on drop.
    assert!(matches!(
        root.open(&gamma).unwrap_err(),
        StoreError::Locked { .. }
    ));
    drop(held_gamma);
    assert!(root.open(&gamma).is_ok());
}

// =================================================================
// The engine's create contract mirrors open's honesty (F6).
// =================================================================

#[test]
fn create_under_a_missing_directory_reports_not_found_and_holds_no_lock() {
    // The create twin of `a_failed_open_holds_no_lock`: a log path whose
    // store directory does not exist reports the engine's own NotFound
    // (t10's exact string), never a lock fault — the directory holds no
    // lock file to create either — and leaves nothing held, so a later
    // create with the directory present acquires cleanly.
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("no-such-dir").join("event-log.jsonl");

    let error = JsonlStore::create_log(&missing, camera()).unwrap_err();
    assert_eq!(
        error,
        StoreError::Io {
            detail: "create the event log: NotFound".to_owned()
        }
    );
    assert_eq!(error.code(), "io");
    // Reproducible, and it created nothing on the way.
    assert_eq!(
        JsonlStore::create_log(&missing, camera()).unwrap_err(),
        error
    );
    assert!(!missing.parent().unwrap().exists());

    // No stale lock was left: with the directory present, the same
    // create_log acquires its lock and succeeds.
    fs::create_dir_all(missing.parent().unwrap()).unwrap();
    let store = JsonlStore::create_log(&missing, camera()).unwrap();
    assert_eq!(store.graph().repo().slug, slug("camera"));
}

// =================================================================
// The read-only-fallback corner (F4): §8's refusal survives a lock file
// this writer may not write, and F3 keeps the original cause when the
// fallback also fails. A root-equivalent uid overrides file modes, so
// each test detects that and prints a note rather than passing silently
// on a corner it never reached.
// =================================================================

#[cfg(unix)]
#[test]
fn a_read_only_lock_file_still_refuses_a_second_writer_with_locked() {
    use std::fs::OpenOptions;
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let held = root.register(camera()).unwrap();

    // Make the lock file read-only: a second writer cannot open it
    // read-write, so without the read-only fallback it would degrade §8's
    // `locked` refusal to `io`. With it, the flock still sees the held
    // lock and reports `locked`.
    let lock = root.lock_path(&camera_slug);
    let original = fs::metadata(&lock).unwrap().permissions();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o444)).unwrap();

    // Does this uid enforce 0444? If a read-write open succeeds, it does
    // not (root), and the fallback path is not what produces the result.
    let enforced = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock)
        .is_err();
    if !enforced {
        eprintln!(
            "NOTE: a 0444 lock file is not enforced under this uid (root?); \
             the read-only fallback is not exercised by this run"
        );
    }

    let error = root.open(&camera_slug).unwrap_err();
    // Restore before asserting, so the temp dir cleans up regardless.
    fs::set_permissions(&lock, original).unwrap();

    // Locked either way — under a non-root uid, specifically because the
    // read-only fallback reached the flock instead of erroring on the mode.
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(camera_slug.clone())
        },
        "a read-only lock file must still refuse a second writer with `locked`"
    );
    drop(held);
}

#[cfg(unix)]
#[test]
fn a_read_only_store_directory_reports_permission_denied_not_not_found() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let store_dir = root.store_dir(&camera_slug);
    fs::create_dir_all(&store_dir).unwrap();

    // A read-only store directory with no lock file in it: creating the
    // lock file is denied. This is F3's corner — the read-write open fails
    // `PermissionDenied`, the read-only fallback then finds no lock file
    // (its own `NotFound`), and the report must keep the *original*
    // `PermissionDenied`. A leaked fallback `NotFound` would wear Decision
    // 11's "never registered" signal and invite a client to register over
    // a permissions problem.
    let original = fs::metadata(&store_dir).unwrap().permissions();
    fs::set_permissions(&store_dir, fs::Permissions::from_mode(0o555)).unwrap();
    let enforced = fs::write(store_dir.join(".probe"), b"x").is_err();
    let _ = fs::remove_file(store_dir.join(".probe"));

    let error = root.open(&camera_slug).unwrap_err();
    // Restore before asserting, so the temp dir cleans up regardless.
    fs::set_permissions(&store_dir, original).unwrap();

    if enforced {
        match &error {
            StoreError::Io { detail } => {
                assert!(
                    detail.contains("PermissionDenied"),
                    "expected PermissionDenied, got {detail}"
                );
                assert!(
                    !detail.contains("NotFound"),
                    "the fallback's NotFound must not mask the cause: {detail}"
                );
            }
            other => panic!("expected an io fault, got {other:?}"),
        }
    } else {
        // Root-equivalent uid: the mode is overridden, the lock file is
        // created, and the open reports the honest missing-log NotFound.
        // The PermissionDenied corner is not exercised under this uid.
        eprintln!(
            "NOTE: a 0555 store directory is not enforced under this uid (root?); \
             the PermissionDenied corner is not exercised by this run"
        );
        assert!(
            matches!(&error, StoreError::Io { detail } if detail.contains("NotFound")),
            "{error}"
        );
    }
}
