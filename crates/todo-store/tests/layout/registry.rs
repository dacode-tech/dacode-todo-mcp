//! The registry document: its golden bytes, its fault reports, its byte
//! stability, and the registration refusals and debris discipline around
//! it (spec §2, §4.3; Decisions 8, 14–19).
//!
//! The golden is written through the real path — `DataRoot::register`
//! into a `tempfile` root — so the bytes on disk, not a hand-serialized
//! document, are what the fixture pins.

use std::fs;

use serde::Serialize;
use tempfile::TempDir;
use todo_domain::DomainError;
use todo_store::{
    FORMAT_VERSION, JsonlStore, Registry, RegistryEntry, Store, StoreError, to_bytes,
};

use super::fixtures::{at, camera, data_root, dir_entries, register_and_release, repo, slug};
use crate::common::assert_golden;

/// A test-local mirror of the registry document's envelope, so the
/// unsorted fixture below is produced by the crate's own codec
/// ([`to_bytes`]) over real [`RegistryEntry`] values — never hand-written
/// JSON (t10's f8 discipline). Drift cannot pass silently where it
/// matters: an extra field is rejected by `deny_unknown_fields`, a missing
/// one by serde's required-field error. Field order is the one harmless
/// difference — deserialization matches by name, and the golden comes
/// from the real `RegistryDocument`, not from this mirror.
#[derive(Serialize)]
struct UnsortedRegistry {
    format_version: u32,
    repos: Vec<RegistryEntry>,
}

/// Writes `bytes` as the registry of a fresh root and returns the load
/// fault — twice-loaded, because §4.3's report is deterministic and a
/// failed load must not hold the registry lock either.
fn faulty_registry(bytes: &[u8]) -> StoreError {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    fs::write(root.registry_path(), bytes).unwrap();
    let error = Registry::load(&root).unwrap_err();
    assert_eq!(error, Registry::load(&root).unwrap_err(), "{error}");
    error
}

// =================================================================
// The golden (required test 9, Decision 19).
// =================================================================

#[test]
fn the_registered_registry_matches_its_golden() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    // Registered in a deliberately non-sorted order with fixed
    // caller-passed timestamps, so the golden proves the slug sort, the
    // field order, the nine-digit timestamp spelling and the trailing LF
    // at once.
    for registration in [
        repo("shelly-ng", "2026-09-25T09:30:00Z"),
        repo("camera", "2026-09-25T08:00:00Z"),
        repo("ikvm-html", "2026-09-25T09:00:00.5Z"),
    ] {
        register_and_release(&root, registration);
    }
    // The golden name embeds FORMAT_VERSION, so bumping the format
    // without landing the new golden fails as "missing golden".
    assert_golden(
        &format!("registry.v{FORMAT_VERSION}"),
        &fs::read(root.registry_path()).unwrap(),
    );
}

// =================================================================
// A fresh root (required test 10, Decision 16).
// =================================================================

#[test]
fn an_absent_registry_loads_empty_and_creates_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);

    let registry = Registry::load(&root).unwrap();
    assert!(registry.entries().is_empty());
    assert!(registry.slugs().is_empty());
    assert!(!registry.contains(&slug("camera")));

    // A fresh root legitimately has no registry: loading one is not
    // corruption, not a creation, and not a silent truncation — the root
    // is exactly as empty as the `TempDir` made it.
    assert!(!root.registry_path().exists());
    assert_eq!(dir_entries(dir.path()), Vec::<String>::new());
}

// =================================================================
// The load-time sort (F5): a registry this build did not write can list
// entries out of slug order, and the sorted invariant is what makes
// `contains`/`insert`'s binary search correct.
// =================================================================

#[test]
fn load_sorts_an_unsorted_registry_and_a_register_rewrites_it_canonical() {
    // Build an unsorted registry.json through the codec over real
    // `RegistryEntry` values — shelly-ng first, then camera, then
    // ikvm-html — so the on-disk order is genuinely non-canonical and
    // only `Registry::load`'s sort can recover the invariant.
    let unsorted = UnsortedRegistry {
        format_version: FORMAT_VERSION,
        repos: vec![
            RegistryEntry {
                registered_at: at("2026-09-25T09:30:00Z"),
                slug: slug("shelly-ng"),
            },
            RegistryEntry {
                registered_at: at("2026-09-25T08:00:00Z"),
                slug: slug("camera"),
            },
            RegistryEntry {
                registered_at: at("2026-09-25T09:00:00.5Z"),
                slug: slug("ikvm-html"),
            },
        ],
    };
    let bytes = to_bytes(&unsorted).unwrap();
    // Non-vacuity: the serialized fixture really is out of order, so the
    // sort below has work to do (serde never reorders a `Vec`).
    let shelly_at = bytes.windows(9).position(|w| w == b"shelly-ng").unwrap();
    let camera_at = bytes.windows(6).position(|w| w == b"camera").unwrap();
    assert!(shelly_at < camera_at, "the fixture must start unsorted");

    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    fs::write(root.registry_path(), &bytes).unwrap();

    let loaded = Registry::load(&root).unwrap();
    // Load re-establishes ascending order...
    assert_eq!(
        loaded.slugs(),
        vec![&slug("camera"), &slug("ikvm-html"), &slug("shelly-ng")]
    );
    let entry_slugs: Vec<_> = loaded.entries().iter().map(|e| e.slug.clone()).collect();
    assert_eq!(
        entry_slugs,
        vec![slug("camera"), slug("ikvm-html"), slug("shelly-ng")]
    );
    // ...which is what makes `contains` — a binary search over the entries
    // — answer correctly for the slug that sat out of position on disk.
    // This is the assertion that fails if load's sort is removed.
    assert!(
        loaded.contains(&slug("shelly-ng")),
        "the out-of-position slug must still be found"
    );
    assert!(loaded.contains(&slug("camera")));
    assert!(loaded.contains(&slug("ikvm-html")));

    // A following register rewrites the whole document canonically: the
    // unsorted input never survives a write cycle. The result is
    // byte-identical to a fresh root that registered the same four repos
    // in sorted order — so the sort is not a read-time illusion but what
    // actually lands on disk.
    register_and_release(&root, repo("aaa", "2026-09-25T07:00:00Z"));
    let rewritten = fs::read(root.registry_path()).unwrap();

    let dir2 = TempDir::new().unwrap();
    let root2 = data_root(&dir2);
    for registration in [
        repo("aaa", "2026-09-25T07:00:00Z"),
        camera(),
        repo("ikvm-html", "2026-09-25T09:00:00.5Z"),
        repo("shelly-ng", "2026-09-25T09:30:00Z"),
    ] {
        register_and_release(&root2, registration);
    }
    assert_eq!(rewritten, fs::read(root2.registry_path()).unwrap());
}

// =================================================================
// The duplicate refusals (required test 11, Decision 8).
// =================================================================

#[test]
fn a_duplicate_registration_is_refused_both_ways() {
    // Way one: the registry lists the slug.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    register_and_release(&root, camera());
    let bytes_before = fs::read(root.log_path(&camera_slug)).unwrap();

    let listed = root.register(camera()).unwrap_err();
    assert_eq!(
        listed,
        StoreError::Domain(DomainError::Duplicate {
            id: "camera".to_owned()
        })
    );
    assert_eq!(listed.code(), "duplicate");
    // The refusal touched nothing.
    assert_eq!(fs::read(root.log_path(&camera_slug)).unwrap(), bytes_before);

    // Way two: the log exists while the registry does not list it — the
    // crash state Decision 8's ordering confines a half-registration to.
    // It is never silently adopted (that would hide the crash); a human
    // resolves it through t13's inspector.
    let orphan_dir = TempDir::new().unwrap();
    let orphan_root = data_root(&orphan_dir);
    drop(<JsonlStore as Store>::create(&orphan_root, camera()).unwrap());
    assert!(!orphan_root.registry_path().exists());
    let orphan_bytes = fs::read(orphan_root.log_path(&camera_slug)).unwrap();

    let orphan = orphan_root.register(camera()).unwrap_err();
    assert_eq!(
        orphan,
        StoreError::Domain(DomainError::Duplicate {
            id: "camera".to_owned()
        })
    );
    assert_eq!(orphan.code(), "duplicate");
    // Still no index entry, still the same log bytes.
    assert!(!orphan_root.registry_path().exists());
    assert_eq!(
        fs::read(orphan_root.log_path(&camera_slug)).unwrap(),
        orphan_bytes
    );
}

// =================================================================
// The half-created store directory (required test 12, judgment (f)).
// =================================================================

#[test]
fn a_half_created_store_directory_completes_registration() {
    // Decision 8 refuses only when the *log* exists: an empty
    // `repos/<slug>/` is the debris of a crash before any byte was
    // written, and re-registering completes it rather than refusing.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    fs::create_dir_all(root.store_dir(&camera_slug)).unwrap();

    let store = root.register(camera()).unwrap();
    assert_eq!(store.graph().repo().slug, camera_slug);
    assert!(root.log_path(&camera_slug).is_file());
    assert!(Registry::load(&root).unwrap().contains(&camera_slug));
    drop(store);
    // The completed store opens like any registered one.
    assert!(root.open(&camera_slug).is_ok());
}

// =================================================================
// Registry faults (required test 13, Decisions 14 and 16).
// =================================================================

#[test]
fn registry_faults_are_named_errors() {
    // Malformed bytes: a positioned `corrupt_store` under the fixed
    // context, with no path in the detail.
    let error = faulty_registry(b"{{{ not json\n");
    assert_eq!(error.code(), "corrupt_store");
    match &error {
        StoreError::CorruptStore {
            offset: Some(_),
            detail,
        } => {
            assert!(
                detail.contains("the registry is not a parseable document"),
                "{error}"
            );
            assert!(!detail.contains('/'), "a detail carries no path: {error}");
        }
        other => panic!("expected a positioned corrupt_store, got {other:?}"),
    }

    // A foreign version is the migration question it is (§4), reported
    // before the document is read strictly — a newer build's extra fields
    // cannot masquerade as damage.
    let error = faulty_registry(br#"{"format_version":2,"repos":[]}"#);
    assert_eq!(
        error,
        StoreError::VersionMismatch {
            found: 2,
            expected: FORMAT_VERSION
        }
    );
    assert_eq!(error.code(), "version_mismatch");
    let error = faulty_registry(br#"{"format_version":2,"repos":[],"future":"field"}"#);
    assert_eq!(error.code(), "version_mismatch");

    // `deny_unknown_fields` at the current version, on the document and on
    // an entry: an unknown field can only be damage (Decision 16).
    let error = faulty_registry(br#"{"format_version":1,"repos":[],"surprise":true}"#);
    assert_eq!(error.code(), "corrupt_store");
    let error = faulty_registry(
        br#"{"format_version":1,"repos":[{"registered_at":"2026-09-25T08:00:00.000000000Z","slug":"camera","name":"x"}]}"#,
    );
    assert_eq!(error.code(), "corrupt_store");

    // A slug or a timestamp that fails its validated newtype arrives as
    // the codec's rejection: a registry this build cannot parse is not a
    // registry.
    let error = faulty_registry(
        br#"{"format_version":1,"repos":[{"registered_at":"2026-09-25T08:00:00.000000000Z","slug":"CAMERA"}]}"#,
    );
    assert_eq!(error.code(), "corrupt_store");
    assert!(
        error.to_string().contains("invalid repository slug"),
        "{error}"
    );
    let error = faulty_registry(
        br#"{"format_version":1,"repos":[{"registered_at":"yesterday","slug":"camera"}]}"#,
    );
    assert_eq!(error.code(), "corrupt_store");
    assert!(error.to_string().contains("timestamp"), "{error}");
}

// =================================================================
// Byte stability (required test 14, §8).
// =================================================================

#[test]
fn registering_the_same_repos_in_any_order_is_byte_stable() {
    let camera = || repo("camera", "2026-09-25T08:00:00Z");
    let ikvm = || repo("ikvm-html", "2026-09-25T09:00:00.5Z");
    let shelly = || repo("shelly-ng", "2026-09-25T09:30:00Z");

    let dir_a = TempDir::new().unwrap();
    let root_a = data_root(&dir_a);
    for registration in [camera(), ikvm(), shelly()] {
        register_and_release(&root_a, registration);
    }
    let dir_b = TempDir::new().unwrap();
    let root_b = data_root(&dir_b);
    for registration in [shelly(), camera(), ikvm()] {
        register_and_release(&root_b, registration);
    }

    // Registration order never reaches the bytes: every save re-sorts,
    // and each of the three saves per root *is* a load → insert → write
    // cycle, so these two roots ran §8's serialize → load → serialize
    // stability three times each and landed on identical bytes.
    let bytes_a = fs::read(root_a.registry_path()).unwrap();
    let bytes_b = fs::read(root_b.registry_path()).unwrap();
    assert_eq!(bytes_a, bytes_b);

    // Load is a pure read of those bytes: equal registries, equal slugs in
    // canonical order, and nothing on disk moved.
    let registry_a = Registry::load(&root_a).unwrap();
    let registry_b = Registry::load(&root_b).unwrap();
    assert_eq!(registry_a, registry_b);
    assert_eq!(
        registry_a.slugs(),
        vec![&slug("camera"), &slug("ikvm-html"), &slug("shelly-ng")]
    );
    assert_eq!(
        registry_a
            .entries()
            .iter()
            .map(|entry| entry.registered_at.clone())
            .collect::<Vec<_>>(),
        vec![
            camera().registered_at,
            ikvm().registered_at,
            shelly().registered_at
        ]
    );

    // §2's documented composition, exercised rather than merely described
    // in `open_many`'s doc: the registry's slugs feed it with no
    // clone-collect between the two halves of "iterate the registry and
    // open per-repo stores", and the stores come back in the registry's
    // own canonical order.
    let stores = root_a.open_many(&registry_a.slugs()).unwrap();
    let opened: Vec<_> = stores
        .iter()
        .map(|store| store.graph().slug().clone())
        .collect();
    assert_eq!(
        opened,
        vec![slug("camera"), slug("ikvm-html"), slug("shelly-ng")]
    );
    // Opening every store the registry lists writes nothing either: the
    // bytes above are still the bytes on disk.
    assert_eq!(fs::read(root_a.registry_path()).unwrap(), bytes_a);
    assert_eq!(fs::read(root_b.registry_path()).unwrap(), bytes_b);
}

// =================================================================
// No temporary debris (required test 15, Decision 17).
// =================================================================

#[test]
fn registration_leaves_no_temp_debris() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let first = root.register(camera()).unwrap();
    let second = root
        .register(repo("ikvm-html", "2026-09-25T09:00:00.5Z"))
        .unwrap();

    // The root holds the registry document, its lock, and `repos/` — the
    // pid-suffixed temp file was consumed by §4.3's atomic rename, twice.
    assert_eq!(
        dir_entries(dir.path()),
        ["registry.json", "registry.json.lock", "repos"]
    );
    assert_eq!(dir_entries(&root.repos_dir()), ["camera", "ikvm-html"]);
    // A store directory holds exactly the log and its lock — while the
    // stores are open…
    for repo_slug in [slug("camera"), slug("ikvm-html")] {
        assert_eq!(
            dir_entries(&root.store_dir(&repo_slug)),
            ["event-log.jsonl", "event-log.jsonl.lock"]
        );
    }
    drop(first);
    drop(second);
    // …and after they close: releasing a lock deletes nothing.
    assert_eq!(
        dir_entries(dir.path()),
        ["registry.json", "registry.json.lock", "repos"]
    );
    for repo_slug in [slug("camera"), slug("ikvm-html")] {
        assert_eq!(
            dir_entries(&root.store_dir(&repo_slug)),
            ["event-log.jsonl", "event-log.jsonl.lock"]
        );
    }
}
