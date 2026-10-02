//! The registry query side: `DataRoot::repo_list` and `DataRoot::repo_show`
//! (spec §6, §7, §8), plus the cold-open measurement t11 handed forward.
//!
//! What the two halves of a query answer are pinned separately, because
//! they come from different files: the slug set and its ascending order are
//! the registry's, and `name`/`path`/`registered_at` are each store's own
//! `Repo` node (§7's mutables live only there). A test that asserted less
//! would pass on an answer built from the index alone, which is the shape a
//! future "optimisation" would take — so the rows are compared against the
//! registered `Repo` values, and the mutations are made through the `Store`
//! trait and read back through the queries.
//!
//! The refusals are the other half. A store that will not open fails the
//! whole call with the error `DataRoot::open` itself reports, never a
//! shorter list; a slug the index does not list is `unknown_id` even with a
//! store directory behind it; and a caller holding a handle contends with
//! its own listing, because §8's contention is per descriptor.
//!
//! Corruption fixtures splice real serialized bytes — a renamed event tag,
//! a bumped `format_version` — never hand-written JSON (f8's discipline).
//! The one exception is the garbage bytes a *registry* fault test writes,
//! which `layout/registry.rs` already spells the same way.

use std::fs;

use tempfile::TempDir;
use todo_domain::{DomainError, Repo, RepoSlug};
use todo_store::{DataRoot, FORMAT_VERSION, JsonlStore, LockTarget, Registry, Store, StoreError};

use super::fixtures::{
    at, camera, data_root, dir_entries, epic_draft, register_and_release, repo, slug,
};

// =================================================================
// A fresh root (required test 1).
// =================================================================

#[test]
fn an_empty_data_root_lists_nothing_shows_nothing_and_creates_nothing() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let before = dir_entries(dir.path());

    // A missing registry loads empty rather than faulting, so a query over
    // a fresh root is an empty answer — not an error and not a creation.
    assert_eq!(root.repo_list().unwrap(), Vec::<Repo>::new());

    let error = root.repo_show(&slug("camera")).unwrap_err();
    assert_eq!(error.code(), "unknown_id");
    assert_eq!(error.to_string(), "unknown id: camera");
    assert!(!error.to_string().contains('/'), "{error} carries no path");

    assert!(!root.registry_path().exists());
    assert!(!root.repos_dir().exists());
    assert_eq!(dir_entries(dir.path()), before);
}

// =================================================================
// The order and the rows (required test 2, D3 and D4).
// =================================================================

#[test]
fn repo_list_is_complete_and_ascending_by_slug() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    // Registered out of alphabetical order, so arrival order cannot
    // satisfy the assertion — the registry's own sort is what does.
    let registrations = [
        ("shelly-ng", "2026-09-25T09:30:00Z"),
        ("camera", "2026-09-25T08:00:00Z"),
        ("ikvm-html", "2026-09-25T09:00:00.5Z"),
        ("alpha", "2026-09-25T07:00:00Z"),
    ];
    for (name, registered_at) in registrations {
        register_and_release(&root, repo(name, registered_at));
    }

    let mut expected: Vec<Repo> = registrations
        .iter()
        .map(|(name, registered_at)| repo(name, registered_at))
        .collect();
    expected.sort_by(|left, right| left.slug.cmp(&right.slug));

    let listed = root.repo_list().unwrap();
    // Whole-row equality against the registered nodes: `name` and `path`
    // differ per repo and exist nowhere in the index, so an answer built
    // from the registry alone cannot pass this.
    assert_eq!(listed, expected);
    assert!(
        listed.windows(2).all(|pair| pair[0].slug < pair[1].slug),
        "ascending by slug: {listed:?}"
    );
    // Non-vacuity for the mutable half: the four names and four paths are
    // four distinct values, not one repeated default.
    let names: Vec<&str> = listed.iter().map(|row| row.name.as_str()).collect();
    let paths: Vec<&str> = listed.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(
        names,
        [
            "alpha repository",
            "camera repository",
            "ikvm-html repository",
            "shelly-ng repository"
        ]
    );
    assert_eq!(
        paths,
        [
            "/srv/repos/alpha",
            "/srv/repos/camera",
            "/srv/repos/ikvm-html",
            "/srv/repos/shelly-ng"
        ]
    );
}

// =================================================================
// §7's mutability is visible (required test 3).
// =================================================================

#[test]
fn a_mutation_through_the_store_is_what_both_queries_answer() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    let camera_slug = slug("camera");
    register_and_release(&root, camera());
    let registry_bytes = fs::read(root.registry_path()).unwrap();

    let mut store = root.open(&camera_slug).unwrap();
    let renamed = store
        .set_repo_name("Camera firmware".to_owned(), at("2026-09-26T10:00:00Z"))
        .unwrap();
    let moved = store
        .set_repo_path("/srv/mirror/camera".to_owned(), at("2026-09-26T10:05:00Z"))
        .unwrap();
    drop(store);

    assert_eq!(renamed.name, "Camera firmware");
    assert_eq!(moved.path, "/srv/mirror/camera");
    assert_ne!(moved, camera(), "the mutation moved the node");
    // The store is the source of truth for §7's mutables, so both queries
    // answer the *new* values — the index holds neither field and is not
    // what a row is built from.
    assert_eq!(root.repo_show(&camera_slug).unwrap(), moved);
    assert_eq!(root.repo_list().unwrap(), vec![moved.clone()]);
    // `registered_at` and the slug are not among the mutables.
    assert_eq!(moved.registered_at, camera().registered_at);
    assert_eq!(moved.slug, camera_slug);
    // And the index is untouched by them: one document, the immutable
    // half, no rewrite per `repo_set_*`.
    assert_eq!(fs::read(root.registry_path()).unwrap(), registry_bytes);
    let registry = Registry::load(&root).unwrap();
    assert_eq!(registry.entries().len(), 1);
    assert_eq!(registry.entries()[0].registered_at, camera().registered_at);
}

// =================================================================
// The node and the index agree (required test 4, D4's invariant).
// =================================================================

#[test]
fn every_row_agrees_with_its_registry_entry_on_the_immutable_half() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    for (name, registered_at) in [
        ("camera", "2026-09-25T08:00:00Z"),
        ("ikvm-html", "2026-09-25T09:00:00.5Z"),
        ("shelly-ng", "2026-09-25T09:30:00Z"),
    ] {
        register_and_release(&root, repo(name, registered_at));
    }
    // One store mutated, so the agreement is asserted over a node whose
    // mutable half has moved since registration.
    let mut store = root.open(&slug("ikvm-html")).unwrap();
    store
        .set_repo_path(
            "/srv/mirror/ikvm-html".to_owned(),
            at("2026-09-26T10:00:00Z"),
        )
        .unwrap();
    drop(store);

    let registry = Registry::load(&root).unwrap();
    let listed = root.repo_list().unwrap();
    assert_eq!(listed.len(), registry.entries().len());
    for (row, entry) in listed.iter().zip(registry.entries()) {
        assert_eq!(row.slug, entry.slug);
        // D4's invariant: both `registered_at` values were written from
        // one `Repo` and nothing rewrites either, and §6.1's closed list
        // has no code for a disagreement — so production does not
        // cross-check one it could not report, and the pin lives here.
        assert_eq!(row.registered_at, entry.registered_at);
        assert_eq!(
            root.repo_show(&row.slug).unwrap().registered_at,
            entry.registered_at
        );
    }
}

// =================================================================
// One record, not two (required test 5).
// =================================================================

#[test]
fn repo_show_answers_the_record_repo_list_carries() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    for (name, registered_at) in [
        ("camera", "2026-09-25T08:00:00Z"),
        ("ikvm-html", "2026-09-25T09:00:00.5Z"),
        ("shelly-ng", "2026-09-25T09:30:00Z"),
    ] {
        register_and_release(&root, repo(name, registered_at));
    }

    let listed = root.repo_list().unwrap();
    assert_eq!(listed.len(), 3);
    for row in &listed {
        assert_eq!(root.repo_show(&row.slug).unwrap(), *row);
    }
    // The refusal half agrees too: a slug neither answers for is one the
    // index does not list.
    assert_eq!(
        root.repo_show(&slug("absent")).unwrap_err().code(),
        "unknown_id"
    );
}

// =================================================================
// An unlisted slug is `unknown_id`, store directory or not (required
// test 6, D7).
// =================================================================

#[test]
fn repo_show_refuses_a_slug_the_registry_does_not_list() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    register_and_release(&root, camera());

    // The half-created state `register` refuses as a `duplicate` forever
    // after, built without a crash: the trait constructor writes a store
    // the registry does not list (`store.rs` documents that it touches no
    // registry). Adopting it here would be exactly the silent adoption
    // `register` refuses.
    let orphan_slug = create_unlisted(&root);
    assert!(!Registry::load(&root).unwrap().contains(&orphan_slug));

    for target in [orphan_slug.clone(), slug("never-created")] {
        let error = root.repo_show(&target).unwrap_err();
        assert_eq!(error.code(), "unknown_id");
        assert_eq!(
            error,
            StoreError::Domain(DomainError::UnknownId {
                id: target.to_string()
            })
        );
        assert_eq!(
            error.domain(),
            Some(&DomainError::UnknownId {
                id: target.to_string()
            })
        );
        assert!(error.to_string().contains(target.as_str()), "{error}");
        assert!(!error.to_string().contains('/'), "{error} carries no path");
    }
    // The refusal is the slug's, not the call's: a listed slug still
    // answers, and the unlisted store's bytes are untouched by the query.
    assert_eq!(root.repo_show(&slug("camera")).unwrap(), camera());
    assert_eq!(root.repo_list().unwrap(), vec![camera()]);
}

// =================================================================
// A store that will not open fails the whole call (required test 7, D6).
// =================================================================

#[test]
fn a_registered_store_that_will_not_open_fails_the_whole_call() {
    // (a) Interior damage spliced from real bytes: one event tag renamed,
    // so the line still parses as JSON and no longer fits the schema — an
    // interior line a crash cannot tear, which is corruption wherever it
    // sits (§4.3).
    let dir = TempDir::new().unwrap();
    let root = world(&dir, &["alpha", "camera", "zeta"]);
    let damaged = slug("camera");
    let log = root.log_path(&damaged);
    let valid = fs::read(&log).unwrap();
    let spliced = String::from_utf8(valid.clone())
        .unwrap()
        .replace("\"kind\":\"epic_add\"", "\"kind\":\"epic_added\"")
        .into_bytes();
    assert_ne!(spliced, valid, "the fixture must actually be damaged");
    fs::write(&log, &spliced).unwrap();
    let error = listing_fails_like_the_open(&root, &damaged, &["alpha", "zeta"]);
    assert_eq!(error.code(), "corrupt_store");
    assert!(
        matches!(
            &error,
            StoreError::CorruptStore {
                offset: Some(_),
                ..
            }
        ),
        "{error:?}"
    );

    // (b) A foreign `format_version` in the log header: the real header
    // line with its version bumped, so the fixture is spliced from written
    // bytes rather than hand-written.
    let dir = TempDir::new().unwrap();
    let root = world(&dir, &["alpha", "camera", "zeta"]);
    let damaged = slug("camera");
    let log = root.log_path(&damaged);
    let valid = fs::read(&log).unwrap();
    let spliced = String::from_utf8(valid.clone())
        .unwrap()
        .replacen(
            &format!("\"format_version\":{FORMAT_VERSION}"),
            &format!("\"format_version\":{}", FORMAT_VERSION + 1),
            1,
        )
        .into_bytes();
    assert_ne!(spliced, valid, "the fixture must actually be damaged");
    fs::write(&log, &spliced).unwrap();
    let error = listing_fails_like_the_open(&root, &damaged, &["alpha", "zeta"]);
    assert_eq!(
        error,
        StoreError::VersionMismatch {
            found: FORMAT_VERSION + 1,
            expected: FORMAT_VERSION
        }
    );
    assert_eq!(error.code(), "version_mismatch");

    // (c) A listed slug whose log was deleted out of band: the engine's
    // honest io fault, which `DataRoot::open` documents as its answer for
    // a store that is not there.
    let dir = TempDir::new().unwrap();
    let root = world(&dir, &["alpha", "camera", "zeta"]);
    let damaged = slug("camera");
    fs::remove_file(root.log_path(&damaged)).unwrap();
    let error = listing_fails_like_the_open(&root, &damaged, &["alpha", "zeta"]);
    assert_eq!(
        error,
        StoreError::Io {
            detail: "open the event log: NotFound".to_owned()
        }
    );
    assert_eq!(error.code(), "io");

    // (d) A contended store: §8's per-descriptor refusal, which a listing
    // meets with the same error a second writer does.
    let dir = TempDir::new().unwrap();
    let root = world(&dir, &["alpha", "camera", "zeta"]);
    let damaged = slug("camera");
    let held = root.open(&damaged).unwrap();
    let error = listing_fails_like_the_open(&root, &damaged, &["alpha", "zeta"]);
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(damaged.clone())
        }
    );
    assert_eq!(error.code(), "locked");
    drop(held);
    // Released, and the whole list answers again: the fault was the
    // store's, and the failed calls left nothing behind them.
    assert_eq!(root.repo_list().unwrap().len(), 3);
}

/// D6's agreement property, asserted once per fault: the listing fails with
/// the *same* error [`DataRoot::open`] reports for that store — equal value,
/// byte-identical `Display` — and answers no list at all rather than a
/// shorter one. The healthy slugs still open and still show, so the failed
/// call dropped no row, damaged nothing and left no lock held.
fn listing_fails_like_the_open(
    root: &DataRoot,
    damaged: &RepoSlug,
    healthy: &[&str],
) -> StoreError {
    let via_open = root.open(damaged).unwrap_err();
    let via_list = root.repo_list().unwrap_err();
    assert_eq!(
        via_list, via_open,
        "the listing reports the open's own fault"
    );
    assert_eq!(via_list.to_string(), via_open.to_string(), "{via_list}");
    // Deterministic on a second call, and `repo_show` says the same thing:
    // one definition of the fault, not one per query.
    assert_eq!(root.repo_list().unwrap_err(), via_list);
    assert_eq!(root.repo_show(damaged).unwrap_err(), via_list);
    for name in healthy {
        let healthy_slug = slug(name);
        let opened = root.open(&healthy_slug).unwrap();
        assert_eq!(opened.graph().repo().slug, healthy_slug);
        drop(opened);
        assert_eq!(root.repo_show(&healthy_slug).unwrap().slug, healthy_slug);
    }
    // The index is whole: the failure removed no entry and added none.
    let registry = Registry::load(root).unwrap();
    assert_eq!(registry.slugs().len(), healthy.len() + 1);
    assert!(registry.contains(damaged));
    via_list
}

// =================================================================
// Self-contention (required test 8, D5's trap).
// =================================================================

#[test]
fn a_held_handle_makes_both_queries_contend_with_themselves() {
    let dir = TempDir::new().unwrap();
    let root = world(&dir, &["camera", "ikvm-html", "shelly-ng"]);
    let held_slug = slug("ikvm-html");

    // Holding the *middle* slug, so the listing opens `camera` first and
    // fails on `ikvm-html`: the observable half of the one-lock-at-a-time
    // discipline is that the lock it took for `camera` is released on the
    // way out.
    let held = root.open(&held_slug).unwrap();

    let error = root.repo_list().unwrap_err();
    assert_eq!(
        error,
        StoreError::Locked {
            target: LockTarget::Repo(held_slug.clone())
        }
    );
    assert_eq!(error.code(), "locked");
    assert_eq!(error.to_string(), "store is locked: repo ikvm-html");
    assert!(!error.to_string().contains('/'), "{error} carries no path");
    // `camera` was opened and released by the failed call: it opens again
    // while `ikvm-html` is still held, so the listing leaked no lock.
    assert!(root.open(&slug("camera")).is_ok());

    // `repo_show` on the held slug contends with itself the same way...
    assert_eq!(root.repo_show(&held_slug).unwrap_err(), error);
    // ...and on a slug that is not held it answers.
    assert_eq!(
        root.repo_show(&slug("camera")).unwrap(),
        repo("camera", "2026-09-25T08:00:00Z")
    );

    drop(held);
    // Released: the same call succeeds and answers every row.
    assert_eq!(root.repo_list().unwrap().len(), 3);
    assert!(root.repo_show(&held_slug).is_ok());
}

// =================================================================
// The mirror half: an unlisted directory is not listed (required test 9,
// D6).
// =================================================================

#[test]
fn an_unlisted_store_directory_is_not_listed() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    register_and_release(&root, camera());
    register_and_release(&root, repo("shelly-ng", "2026-09-25T09:30:00Z"));
    // t11's other divergence half, built without a crash: `register` writes
    // store bytes before the index, so a crash after the log landed leaves
    // a directory the registry does not list.
    let orphan_slug = create_unlisted(&root);

    let listed = root.repo_list().unwrap();
    assert_eq!(
        listed,
        vec![camera(), repo("shelly-ng", "2026-09-25T09:30:00Z")]
    );
    // The listing succeeded, so the unlisted directory is an omission the
    // index justifies — and the index itself is unchanged by it.
    assert_eq!(
        Registry::load(&root).unwrap().slugs(),
        vec![&slug("camera"), &slug("shelly-ng")]
    );
    // Resolving the divergence stays t13's inspector's job: the query side
    // reports it by omission and refusal, never by adoption, and a later
    // registration of that slug is the `duplicate` the module docs name.
    assert_eq!(
        root.repo_show(&orphan_slug).unwrap_err().code(),
        "unknown_id"
    );
    assert_eq!(root.register(orphan()).unwrap_err().code(), "duplicate");
}

// =================================================================
// A store whose graph names another repository (the check `repo_node`
// makes, which no other reader in this crate does).
// =================================================================

#[test]
fn a_store_whose_graph_names_another_repo_is_a_named_error_not_an_answer() {
    let dir = TempDir::new().unwrap();
    let root = world(&dir, &["camera", "shelly-ng"]);
    // The divergence, built without a crash and out of band — `register`
    // refuses a slug whose log already exists, so no legal call can produce
    // it: `repos/shelly-ng/` now holds camera's real log bytes, spliced
    // whole rather than hand-written (f8's rule). The store opens clean,
    // replays clean, and answers as camera.
    let camera_log = fs::read(root.log_path(&slug("camera"))).unwrap();
    fs::write(root.log_path(&slug("shelly-ng")), &camera_log).unwrap();
    assert_eq!(
        root.open(&slug("shelly-ng")).unwrap().graph().repo().slug,
        slug("camera"),
        "the fixture must actually be divergent"
    );

    let error = root.repo_show(&slug("shelly-ng")).unwrap_err();
    assert_eq!(error.code(), "corrupt_store");
    match &error {
        StoreError::CorruptStore { offset, detail } => {
            // A relation between the directory and the graph has no byte
            // position, so there is nothing for a repair to truncate at.
            assert_eq!(*offset, None, "{error:?}");
            assert!(
                detail.contains("the store opened for shelly-ng"),
                "{detail}"
            );
            assert!(detail.contains("holds repo camera"), "{detail}");
            assert!(!detail.contains('/'), "a detail carries no path: {detail}");
        }
        other => panic!("expected a corrupt_store fault, got {other:?}"),
    }

    // The listing fails the whole call rather than answering camera twice —
    // two rows for one repository, and the documented total order broken, is
    // the wrong answer this check exists to refuse.
    assert_eq!(root.repo_list().unwrap_err(), error);
    assert_eq!(root.repo_list().unwrap_err().to_string(), error.to_string());

    // The healthy repo still answers and the index is whole: the fault is the
    // copied store's, and the failed calls dropped every lock on the way out.
    assert_eq!(root.repo_show(&slug("camera")).unwrap(), camera());
    assert_eq!(Registry::load(&root).unwrap().slugs().len(), 2);
}

// =================================================================
// A read creates nothing but the lock `open` already creates (required
// test 10).
// =================================================================

#[test]
fn a_listing_creates_nothing_but_the_lock_files_open_already_creates() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    register_and_release(&root, camera());
    register_and_release(&root, repo("ikvm-html", "2026-09-25T09:00:00.5Z"));
    let camera_slug = slug("camera");
    let ikvm_slug = slug("ikvm-html");
    let listed_slugs = [camera_slug.clone(), ikvm_slug.clone()];

    // Remove both lock files so the "at most the `.lock`" half is
    // measurable: a lock file's bytes mean nothing (`lock.rs`), and an open
    // recreates it.
    for target in &listed_slugs {
        fs::remove_file(root.lock_path(target)).unwrap();
    }
    for target in &listed_slugs {
        assert_eq!(
            dir_entries(&root.store_dir(target)),
            ["event-log.jsonl".to_owned()]
        );
    }
    let root_before = dir_entries(dir.path());
    let repos_before = dir_entries(&root.repos_dir());
    let registry_before = fs::read(root.registry_path()).unwrap();

    assert_eq!(root.repo_list().unwrap().len(), 2);
    assert!(root.repo_show(&camera_slug).is_ok());

    // Exact entry lists, so a `.tmp`, a snapshot or a second lock would
    // fail here: the root and `repos/` are untouched, and each store
    // directory gained exactly the `.lock` its open creates.
    assert_eq!(dir_entries(dir.path()), root_before);
    assert_eq!(dir_entries(&root.repos_dir()), repos_before);
    for target in &listed_slugs {
        assert_eq!(
            dir_entries(&root.store_dir(target)),
            ["event-log.jsonl", "event-log.jsonl.lock"]
        );
        assert!(!root.snapshot_path(target).exists());
    }
    // And the index bytes are the same document: a query writes no format.
    assert_eq!(fs::read(root.registry_path()).unwrap(), registry_before);
}

// =================================================================
// Repeatable and order-independent (required test 11, §8).
// =================================================================

#[test]
fn a_listing_is_repeatable_and_independent_of_registration_order() {
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

    let first = root_a.repo_list().unwrap();
    // Unchanged state, equal answers, twice — §8's byte-stability at the
    // query level.
    assert_eq!(root_a.repo_list().unwrap(), first);
    // Registration order never reaches the answer, on either root.
    assert_eq!(root_b.repo_list().unwrap(), first);
    assert_eq!(root_b.repo_list().unwrap(), root_b.repo_list().unwrap());
    for row in &first {
        assert_eq!(root_a.repo_show(&row.slug).unwrap(), *row);
        assert_eq!(root_b.repo_show(&row.slug).unwrap(), *row);
    }
    // The order independence is the index's, not a re-sort here: both roots
    // hold the same registry document.
    assert_eq!(
        fs::read(root_a.registry_path()).unwrap(),
        fs::read(root_b.registry_path()).unwrap()
    );
}

// =================================================================
// A registry this build cannot read fails both queries (beyond the
// required twelve: D6's principle at the index rather than the store).
// =================================================================

#[test]
fn a_registry_this_build_cannot_read_fails_both_queries() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    register_and_release(&root, camera());
    let camera_slug = slug("camera");
    let registry = fs::read(root.registry_path()).unwrap();
    let log_before = fs::read(root.log_path(&camera_slug)).unwrap();

    // A *missing* registry is an empty answer (the fresh-root test above);
    // a registry this build cannot parse is the codec's own named fault,
    // propagated whole. Substituting an empty list for it would be the
    // silent omission D6 refuses at the store, one level up.
    let foreign = String::from_utf8(registry.clone())
        .unwrap()
        .replacen(
            &format!("\"format_version\":{FORMAT_VERSION}"),
            &format!("\"format_version\":{}", FORMAT_VERSION + 1),
            1,
        )
        .into_bytes();
    assert_ne!(foreign, registry, "the fixture must actually be foreign");
    for (bytes, code) in [
        (b"{{{ not json\n".to_vec(), "corrupt_store"),
        (foreign.clone(), "version_mismatch"),
    ] {
        fs::write(root.registry_path(), &bytes).unwrap();
        let via_load = Registry::load(&root).unwrap_err();
        assert_eq!(via_load.code(), code, "{via_load}");
        assert_eq!(root.repo_list().unwrap_err(), via_load);
        assert_eq!(root.repo_show(&camera_slug).unwrap_err(), via_load);
    }
    // Neither query wrote anything on the way: the store's log is the same
    // bytes, and the index is still the damaged one it was handed.
    assert_eq!(fs::read(root.log_path(&camera_slug)).unwrap(), log_before);
    assert_eq!(fs::read(root.registry_path()).unwrap(), foreign);

    // Restored, the same queries answer again.
    fs::write(root.registry_path(), &registry).unwrap();
    assert_eq!(root.repo_list().unwrap(), vec![camera()]);
}

// =================================================================
// The cold-open measurement (required test 12, D8).
// =================================================================

/// The grid's repository counts.
const MEASURED_REPO_COUNTS: [usize; 3] = [1, 10, 50];
/// The grid's log lengths, in events appended to each store on top of its
/// seed: `0` is the log a fresh registration leaves, and the larger value
/// is the history dimension — replay cost grows with it while a registry
/// read does not, and that ratio is the decision material.
const MEASURED_EVENT_COUNTS: [usize; 2] = [0, 200];
/// Repetitions behind each reported pair of numbers: the **first** call and
/// the steady mean of the rest. At least two, because the first is reported
/// separately — it is a *lower bound* for what a just-started server pays,
/// which meets a cold page cache on top of it, and averaging the two together
/// hides exactly the number the decision needs. A mean at all, rather
/// than a single sample, because the one comparison asserted — the index floor
/// being smaller than the whole listing — is the one a single sample's noise
/// could invert.
const MEASURED_RUNS: usize = 7;

/// **Why this test is `#[ignore]`d**: it is a measurement, not a property.
/// It asserts structure only — the row count, ascending order, the rows
/// being node-backed, and the registry floor being smaller than the whole
/// listing — and never a wall-clock threshold, which would be flaky on a
/// loaded host. Run it with:
///
/// ```text
/// cargo test -p todo-store --test layout -- --ignored --nocapture
/// ```
///
/// It prints one row per grid cell: the repository count, the events per
/// store, one store's log bytes, the fixture build time, the steady
/// [`Registry::load`] floor, the **first** [`DataRoot::repo_list`], the
/// **steady** mean of the remaining runs, and the per-repo delta between the
/// steady pair. Those numbers belong in this run's output and the tracker
/// node's evidence, never in a source comment — a figure in a comment is a
/// claim nobody re-measures.
///
/// **"Cold" here means cold at the process level, and only that**: a fresh
/// [`DataRoot`], no handle held anywhere, every store opened and replayed
/// from its bytes. The page cache is **warm** — `measured_world` wrote every
/// one of those bytes in this process, right before the timed calls, and the
/// `build ms` column says how long that took — so these are open-plus-replay
/// costs, not first-touch I/O, and a first run on a truly cold cache would be
/// slower. Both terms are warm, so they are comparable — but a cache that was
/// not would not touch them alike: the whole listing reads every log while the
/// floor reads one small document, so cold inflates the whole far more than the
/// floor. These numbers are therefore the **optimistic** case for opening every
/// store, which is the direction the registry decision turns on.
///
/// `std::time::Instant` appears in this file alone: §8 bars clocks below the
/// binary layer as a *determinism* rule, and a measurement harness
/// produces no bytes, touches no golden and asserts no duration.
#[test]
#[ignore = "a measurement, not a property; run with --ignored --nocapture"]
fn cold_repo_list_open_cost() {
    use std::time::Instant;

    println!();
    println!(
        "DataRoot::repo_list over the D4 layout: the first call, then the steady \
         mean of the other {} runs",
        MEASURED_RUNS - 1
    );
    println!(
        "cold at the PROCESS level only — a fresh DataRoot, no handle held, every store \
         opened and replayed; the page cache is warm, so these are open+replay costs"
    );
    println!("fixture: repos named repo-000.., each with N epics appended, no snapshot");
    println!(
        "{:>6} {:>7} {:>10} {:>10} {:>10} {:>11} {:>11} {:>12}",
        "repos",
        "events",
        "log bytes",
        "build ms",
        "floor us",
        "first us",
        "steady us",
        "per-repo us"
    );
    for repos in MEASURED_REPO_COUNTS {
        for events in MEASURED_EVENT_COUNTS {
            // Fixture construction, timed separately and never folded into
            // a measured number.
            let built = Instant::now();
            let dir = TempDir::new().unwrap();
            let root = measured_world(&dir, repos, events);
            let build = built.elapsed();

            let registry = Registry::load(&root).unwrap();
            let slugs: Vec<RepoSlug> = registry.slugs().into_iter().cloned().collect();
            drop(registry);
            assert_eq!(slugs.len(), repos, "the fixture registered every repo");
            let first = slugs.first().unwrap();
            let log_bytes = fs::metadata(root.log_path(first)).unwrap().len();

            // The floor: the index read alone, with a fresh `DataRoot`.
            let floor = timed_micros(MEASURED_RUNS, || {
                Registry::load(&DataRoot::new(root.root()))
                    .unwrap()
                    .entries()
                    .len()
            });
            // The whole: a process-cold listing with a fresh `DataRoot` and
            // no handle held anywhere — `measured_world` released every one.
            let whole = timed_micros(MEASURED_RUNS, || {
                DataRoot::new(root.root()).repo_list().unwrap().len()
            });

            // Structure only. No duration is asserted, because a threshold
            // would be a flaky test rather than a measurement.
            assert!(
                floor.steady_us < whole.steady_us,
                "the floor is smaller than the whole: {} >= {}",
                floor.steady_us,
                whole.steady_us
            );
            let listed = DataRoot::new(root.root()).repo_list().unwrap();
            assert_eq!(listed.len(), repos);
            assert!(
                listed.windows(2).all(|pair| pair[0].slug < pair[1].slug),
                "ascending by slug"
            );
            for (row, expected) in listed.iter().zip(&slugs) {
                // Each row is that store's node, so the measured call is the
                // real query and not a shortcut over the index.
                assert_eq!(&row.slug, expected);
                assert_eq!(
                    row,
                    &repo(expected.as_str(), "2026-09-25T08:00:00Z"),
                    "the row is the node"
                );
                assert_eq!(
                    DataRoot::new(root.root()).repo_show(expected).unwrap(),
                    *row
                );
            }

            let per_repo = (whole.steady_us - floor.steady_us) / u128::try_from(repos).unwrap();
            println!(
                "{repos:>6} {events:>7} {log_bytes:>10} {:>10} {:>10} {:>11} {:>11} \
                 {per_repo:>12}",
                build.as_millis(),
                floor.steady_us,
                whole.first_us,
                whole.steady_us
            );
        }
    }
}

/// The measurement's fixture: `repos` registered stores named `repo-000`…,
/// each with `events` epics appended over its seed. Every handle is
/// released before the return, so the timed calls hold no lock and open
/// every store from nothing.
fn measured_world(dir: &TempDir, repos: usize, events: usize) -> DataRoot {
    let root = data_root(dir);
    for index in 0..repos {
        let name = format!("repo-{index:03}");
        let mut store = root.register(repo(&name, "2026-09-25T08:00:00Z")).unwrap();
        for event in 0..events {
            store
                .add_epic(
                    epic_draft(&format!("Epic {event}")),
                    at("2026-09-25T09:00:00Z"),
                )
                .unwrap();
        }
        drop(store);
    }
    root
}

/// One cell's two numbers: the **first** call's wall time and the **steady**
/// mean of the remaining ones. The first is the open-and-replay path's
/// first-touch cost in an already-warm process — the fixture build has the
/// allocator, the codec, the filesystem layer and the page cache warm by the
/// time it runs — and it is in practice the slower of the two, though that is
/// an observation about this host rather than a guarantee. It is a lower bound
/// for what a just-started server pays, which meets a cold page cache as well.
/// Reporting only a mean over all runs would average the two together and hide
/// the number the cold-path decision actually turns on.
struct Timing {
    first_us: u128,
    steady_us: u128,
}

/// Times `body` once, then `runs - 1` more times inside a single span, so the
/// steady mean pays the clock's own overhead once rather than per call.
/// `runs` is at least two ([`MEASURED_RUNS`]). Integer microseconds: a
/// measurement printed for a human needs no float, and §8 keeps floats out of
/// persisted state.
fn timed_micros<F, T>(runs: usize, mut body: F) -> Timing
where
    F: FnMut() -> T,
{
    use std::time::Instant;

    let started = Instant::now();
    body();
    let first_us = started.elapsed().as_micros();

    let steady = Instant::now();
    for _ in 1..runs {
        body();
    }
    Timing {
        first_us,
        steady_us: steady.elapsed().as_micros() / u128::try_from(runs - 1).unwrap(),
    }
}

// =================================================================
// Shared builders for the fault tests.
// =================================================================

/// The unlisted-store repo: §7's divergence fixture, whose slug the index
/// never holds.
fn orphan() -> Repo {
    repo("orphan", "2026-09-25T07:30:00Z")
}

/// Writes the orphan's store without an index entry — through the trait
/// constructor `store.rs` documents as not touching the registry — releases
/// the handle, and returns the slug it stranded. The state is on disk, so
/// every caller reads it back through a query.
fn create_unlisted(root: &DataRoot) -> RepoSlug {
    let unlisted = orphan();
    let stranded = unlisted.slug.clone();
    drop(<JsonlStore as Store>::create(root, unlisted).unwrap());
    assert!(root.log_path(&stranded).is_file());
    stranded
}

/// A world of `names` registered repositories, each with one epic appended
/// so its log has an interior event line a fixture can damage. Every handle
/// is released before the return; the registrations are stamped in
/// ascending seconds, so no two rows share a `registered_at`.
fn world(dir: &TempDir, names: &[&str]) -> DataRoot {
    let root = data_root(dir);
    for (index, name) in names.iter().enumerate() {
        let registered_at = format!("2026-09-25T08:00:{index:02}Z");
        let mut store = root.register(repo(name, &registered_at)).unwrap();
        store
            .add_epic(
                epic_draft("Ship the sensor pipeline"),
                at("2026-09-25T08:05:00Z"),
            )
            .unwrap();
        drop(store);
    }
    root
}
