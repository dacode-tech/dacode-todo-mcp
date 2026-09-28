//! The D4 structure itself: every accessor derives the signed layout from
//! the injected root and nothing else, slugs are path-safe by
//! construction, and the trait's constructor half speaks slugs, not paths
//! (spec §2, §7; Decision 2's names).

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use todo_domain::{MAX_SLUG_LEN, RepoSlug};
use todo_store::{Event, FORMAT_VERSION, JsonlStore, Registry, Store, to_bytes};

use super::fixtures::{camera, data_root, dir_entries, slug};

// =================================================================
// Path derivation (required test 1).
// =================================================================

#[test]
fn every_accessor_derives_the_d4_structure() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera = slug("camera");

    assert_eq!(root.root(), dir.path());
    assert_eq!(root.repos_dir(), dir.path().join("repos"));
    assert_eq!(
        root.store_dir(&camera),
        dir.path().join("repos").join("camera")
    );
    assert_eq!(
        root.log_path(&camera),
        dir.path()
            .join("repos")
            .join("camera")
            .join("event-log.jsonl")
    );
    assert_eq!(
        root.lock_path(&camera),
        dir.path()
            .join("repos")
            .join("camera")
            .join("event-log.jsonl.lock")
    );
    assert_eq!(root.registry_path(), dir.path().join("registry.json"));
    assert_eq!(
        root.registry_lock_path(),
        dir.path().join("registry.json.lock")
    );

    // The one lock-path rule, checked against the protected paths rather
    // than restated: a lock file is the protected file plus `.lock`, so
    // two logs in one directory could never share a lock (Decision 2).
    for protected in [root.log_path(&camera), root.registry_path()] {
        let mut expected = protected.clone().into_os_string();
        expected.push(".lock");
        let expected = PathBuf::from(expected);
        let derived = if protected == root.registry_path() {
            root.registry_lock_path()
        } else {
            root.lock_path(&camera)
        };
        assert_eq!(derived, expected, "the rule derives {protected:?}'s lock");
    }

    // Nothing absolute from the host beyond the injected root: every
    // derived path is the root plus exactly the D4 relative structure.
    for (path, tail) in [
        (root.repos_dir(), "repos"),
        (root.store_dir(&camera), "repos/camera"),
        (root.log_path(&camera), "repos/camera/event-log.jsonl"),
        (root.lock_path(&camera), "repos/camera/event-log.jsonl.lock"),
        (root.registry_path(), "registry.json"),
        (root.registry_lock_path(), "registry.json.lock"),
    ] {
        assert_eq!(
            path.strip_prefix(root.root()).unwrap(),
            Path::new(tail),
            "{path:?}"
        );
    }

    // Deriving paths touches nothing: the root is as empty as the
    // `TempDir` created it (Decision 7 — no validation, no creation).
    assert_eq!(dir_entries(dir.path()), Vec::<String>::new());
}

// =================================================================
// Slugs are path-safe by construction (required test 2).
// =================================================================

#[test]
fn slugs_are_path_safe_by_construction() {
    // The charset that makes `repos/<slug>/` traversal-safe: no empty
    // name, no dot segment, no separator, no uppercase, no leading hyphen
    // — each is refused at construction, so no such value ever reaches a
    // path join (tested, not assumed).
    let too_long = "a".repeat(MAX_SLUG_LEN + 1);
    for bad in ["", ".", "..", "a/b", "A", "-a", too_long.as_str()] {
        assert!(
            RepoSlug::new(bad.to_owned()).is_err(),
            "{bad:?} must not be a slug"
        );
    }

    // A maximum-length slug still names exactly one level below `repos/`.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let longest = slug(&"a".repeat(MAX_SLUG_LEN));
    let store_dir = root.store_dir(&longest);
    assert!(store_dir.starts_with(root.repos_dir()));
    assert_eq!(
        store_dir.file_name().unwrap().to_string_lossy(),
        "a".repeat(MAX_SLUG_LEN)
    );
    assert_eq!(
        store_dir.strip_prefix(root.repos_dir()).unwrap(),
        Path::new(&"a".repeat(MAX_SLUG_LEN))
    );
    assert!(root.log_path(&longest).starts_with(&store_dir));
    assert!(root.lock_path(&longest).starts_with(&store_dir));
    assert_eq!(dir_entries(dir.path()), Vec::<String>::new());
}

// =================================================================
// The trait's constructor half (required test 3).
// =================================================================

#[test]
fn trait_create_makes_the_d4_directory_and_open_replays_it() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    let seed = camera();

    // `Store::create` takes the root and a repo — a slug-keyed store, not
    // a path (§4.2's constructor half, §7's keying) — and makes the whole
    // D4 directory: the log and its lock at exactly the accessor paths.
    let store = <JsonlStore as Store>::create(&root, seed.clone()).unwrap();
    assert!(root.log_path(&camera_slug).is_file());
    assert!(root.lock_path(&camera_slug).is_file());
    assert_eq!(
        dir_entries(&root.store_dir(&camera_slug)),
        ["event-log.jsonl", "event-log.jsonl.lock"]
    );

    // The log's bytes are exactly the engine's header and seed — the
    // trait constructor adds layout, never content.
    let mut expected = Vec::new();
    expected.extend_from_slice(br#"{"format_version":1}"#);
    expected.push(b'\n');
    expected.extend_from_slice(
        &to_bytes(&Event::RepoRegister {
            ts: seed.registered_at.clone(),
            repo: seed.clone(),
        })
        .unwrap(),
    );
    expected.push(b'\n');
    assert_eq!(fs::read(root.log_path(&camera_slug)).unwrap(), expected);

    // The trait's create is the engine's constructor, not the registry's
    // writer: the index stays absent until `DataRoot::register` (Decision
    // 18 — registration is the whole registry mutation surface).
    assert!(!Registry::load(&root).unwrap().contains(&camera_slug));
    assert_eq!(store.graph().repo(), &seed);
    drop(store);

    // `Store::open` takes a slug, not a path, and replays the seed.
    let reopened = <JsonlStore as Store>::open(&root, &camera_slug).unwrap();
    assert_eq!(reopened.graph().repo(), &seed);
    assert_eq!(reopened.format_version(), FORMAT_VERSION);
}
