//! The cwd tier's suite (t53): the D8-deferred default, armed — every
//! test drives the real binary spawned with a working directory, over
//! raw stdio (§10: request bytes in, response bytes out, never by
//! calling the handler around it). The fixture is built in-process
//! like common's — real temp directories registered as repos, so
//! canonicalization has something real to resolve — and every test
//! builds its own (the write suites' isolation: a shared fixture
//! would make each test's state depend on every test before it).
//!
//! The ten tests map to the spec's required list 1–10; the six
//! mandatory mutations M1–M6 are each killed by their designated test
//! (the kill sets are kept clean on purpose: only test 9 registers a
//! raw path spelling, so canonicalization's deletion fails exactly
//! the pin and not every test in the file).
//!
//! Every test lists as `cwd::…` so the module-path filter
//! `cargo test -p todo-mcp --test cwd -- cwd` selects exactly this
//! suite (P-LIVE-002/P-LIVE-005: quote the counts beside the filter).

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

mod common;

mod cwd {
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    use serde_json::json;
    use tempfile::TempDir;
    use todo_domain::{
        EpicDraft, References, Repo, RepoSlug, Status, TaskDraft, TaskKind, Timestamp,
    };
    use todo_store::{DataRoot, Store};

    use crate::common::{assert_carrier, call, drive_with_cwd, structured};

    /// The no-default message (D4): names that neither the flag nor
    /// the working directory produced a default.
    const NO_DEFAULT: &str = "the repo parameter is absent and neither the --repo flag \
         nor the working directory produced a default repo";

    /// A fixed canonical timestamp — the fixture is deterministic.
    fn ts(value: &str) -> Timestamp {
        Timestamp::parse(value).expect("a fixed canonical timestamp")
    }

    /// The checkout's canonical spelling: the physical path, what the
    /// OS hands a spawned process as its cwd regardless of the
    /// spelling it was given.
    fn canonical(dir: &TempDir) -> PathBuf {
        fs::canonicalize(dir.path()).expect("the checkout canonicalizes")
    }

    /// Registers `slug` under `root` at `checkout` — the path string
    /// is the caller's spelling, canonical or raw, the pin's choice —
    /// with one epic and one open task, so `next_task` answers
    /// `<slug>:U1` and the match has something real to resolve. The
    /// store handle is dropped before the server spawns (the advisory
    /// lock is per process).
    fn register_at(root: &DataRoot, slug: &str, checkout: &Path) {
        let mut store = root
            .register(Repo {
                slug: RepoSlug::from_str(slug).expect("a valid slug"),
                name: format!("The {slug} repository"),
                path: checkout.display().to_string(),
                registered_at: ts("2026-10-01T08:00:00Z"),
            })
            .expect("the repo registers");
        let epic = store
            .add_epic(
                EpicDraft {
                    title: format!("The {slug} epic"),
                    description: String::new(),
                    status: Status::Open,
                    references: References::new(),
                },
                ts("2026-10-01T08:01:00Z"),
            )
            .expect("the epic lands");
        store
            .add_task(
                TaskDraft {
                    epic: epic.id,
                    kind: TaskKind::Task,
                    title: format!("The {slug} task"),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test -p fixture".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                ts("2026-10-01T08:02:00Z"),
            )
            .expect("the task lands");
        drop(store);
    }

    /// Asserts a `next_task` response answers `<slug>:U1` — the
    /// fixture's one open task.
    fn answers_task(response: &str, slug: &str) {
        assert_eq!(
            structured(response)["task"],
            json!(format!("{slug}:U1")),
            "{response}"
        );
    }

    // =================================================================
    // The ten required tests
    // =================================================================

    /// Test 1 — the cwd equals a registered checkout → `next_task` with no
    /// repo answers that repo's task — the honest oracle: identical
    /// to what the explicit parameter answers. Registered and spawned
    /// at the canonical spelling, so this test pins the behavior, not
    /// the spelling (test 9 owns the spelling pin).
    #[test]
    fn the_cwd_equal_to_a_checkout_answers_that_repo() {
        let root = TempDir::new().expect("a fresh root");
        let checkout = TempDir::new().expect("a fresh checkout");
        let at = canonical(&checkout);
        register_at(&DataRoot::new(root.path()), "fixture", &at);
        let driven = drive_with_cwd(
            root.path(),
            None,
            &at,
            &[
                call(2, "next_task", r#"{}"#),
                call(3, "next_task", r#"{"repo":"fixture"}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        answers_task(&driven.responses[0], "fixture");
        assert_eq!(
            structured(&driven.responses[0]),
            structured(&driven.responses[1]),
            "the derived default answers exactly what the explicit parameter answers"
        );
    }

    /// Test 2 — the cwd under a SUBDIRECTORY of a registered checkout →
    /// still that repo: the match is ancestor-or-equal, component-wise.
    #[test]
    fn the_cwd_under_a_subdirectory_still_answers_that_repo() {
        let root = TempDir::new().expect("a fresh root");
        let checkout = TempDir::new().expect("a fresh checkout");
        let sub = canonical(&checkout).join("src").join("deep");
        fs::create_dir_all(&sub).expect("the subdirectory lands");
        register_at(
            &DataRoot::new(root.path()),
            "fixture",
            &canonical(&checkout),
        );
        let driven = drive_with_cwd(
            root.path(),
            None,
            &sub,
            &[
                call(2, "next_task", r#"{}"#),
                call(3, "next_task", r#"{"repo":"fixture"}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        answers_task(&driven.responses[0], "fixture");
        assert_eq!(
            structured(&driven.responses[0]),
            structured(&driven.responses[1])
        );
    }

    /// Test 3 — Nested checkouts — `base` at the parent, `nested` deeper —
    /// → the longer match wins. `base` sorts first in the walk, so a
    /// first-candidate-wins bug would answer it (M1's kill).
    #[test]
    fn the_longest_match_wins_between_nested_checkouts() {
        let root = TempDir::new().expect("a fresh root");
        let parent = TempDir::new().expect("a parent checkout");
        let nested = canonical(&parent).join("nested");
        fs::create_dir_all(&nested).expect("the nested checkout lands");
        let cwd = nested.join("work");
        fs::create_dir_all(&cwd).expect("the working directory lands");
        let data_root = DataRoot::new(root.path());
        register_at(&data_root, "base", &canonical(&parent));
        register_at(&data_root, "nested", &nested);
        let driven = drive_with_cwd(
            root.path(),
            None,
            &cwd,
            &[
                call(2, "next_task", r#"{}"#),
                call(3, "next_task", r#"{"repo":"nested"}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        answers_task(&driven.responses[0], "nested");
        assert_eq!(
            structured(&driven.responses[0]),
            structured(&driven.responses[1])
        );
    }

    /// Test 4 — the cwd matching nothing → `invalid` through the carrier,
    /// the message naming the absence (D4's no-default message).
    #[test]
    fn a_cwd_matching_nothing_is_invalid_naming_the_absence() {
        let root = TempDir::new().expect("a fresh root");
        let elsewhere = TempDir::new().expect("an unregistered directory");
        let checkout = TempDir::new().expect("a fresh checkout");
        register_at(
            &DataRoot::new(root.path()),
            "fixture",
            &canonical(&checkout),
        );
        let driven = drive_with_cwd(
            root.path(),
            None,
            elsewhere.path(),
            &[call(2, "next_task", r#"{}"#)],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        assert_carrier(&driven.responses[0], "invalid", NO_DEFAULT);
    }

    /// Test 5 — the `--repo` flag plus a matching cwd → the flag wins: with
    /// `--repo zzz` configured and the cwd inside `aaa`'s checkout,
    /// the absent parameter answers zzz (M5's kill — a cwd-overrides
    /// bug would answer aaa).
    #[test]
    fn the_flag_wins_over_a_matching_cwd() {
        let root = TempDir::new().expect("a fresh root");
        let checkout_a = TempDir::new().expect("aaa's checkout");
        let checkout_z = TempDir::new().expect("zzz's checkout");
        let data_root = DataRoot::new(root.path());
        register_at(&data_root, "aaa", &canonical(&checkout_a));
        register_at(&data_root, "zzz", &canonical(&checkout_z));
        let flag = RepoSlug::from_str("zzz").expect("a valid slug");
        let driven = drive_with_cwd(
            root.path(),
            Some(&flag),
            &canonical(&checkout_a),
            &[
                call(2, "next_task", r#"{}"#),
                call(3, "next_task", r#"{"repo":"zzz"}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        answers_task(&driven.responses[0], "zzz");
        assert_eq!(
            structured(&driven.responses[0]),
            structured(&driven.responses[1])
        );
    }

    /// Test 6 — An explicit repo parameter plus a matching cwd → the
    /// parameter wins: the cwd sits inside `aaa`'s checkout, the
    /// parameter names `zzz`, and the answer is zzz's — with the
    /// absent-parameter call beside it answering aaa's, the contrast
    /// is pinned in one drive.
    #[test]
    fn an_explicit_parameter_wins_over_a_matching_cwd() {
        let root = TempDir::new().expect("a fresh root");
        let checkout_a = TempDir::new().expect("aaa's checkout");
        let checkout_z = TempDir::new().expect("zzz's checkout");
        let data_root = DataRoot::new(root.path());
        register_at(&data_root, "aaa", &canonical(&checkout_a));
        register_at(&data_root, "zzz", &canonical(&checkout_z));
        let driven = drive_with_cwd(
            root.path(),
            None,
            &canonical(&checkout_a),
            &[
                call(2, "next_task", r#"{"repo":"zzz"}"#),
                call(3, "next_task", r#"{}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        answers_task(&driven.responses[0], "zzz");
        answers_task(&driven.responses[1], "aaa");
    }

    /// Test 7 — Two repos at the same canonical path → no default, the
    /// message names both slugs in the walk's order (M3's kill — a
    /// first-slug-wins bug would answer aaa's task instead of the
    /// refusal).
    #[test]
    fn two_repos_at_one_path_refuse_naming_both() {
        let root = TempDir::new().expect("a fresh root");
        let checkout = TempDir::new().expect("a shared checkout");
        let at = canonical(&checkout);
        let data_root = DataRoot::new(root.path());
        register_at(&data_root, "aaa", &at);
        register_at(&data_root, "zzz", &at);
        let driven = drive_with_cwd(root.path(), None, &at, &[call(2, "next_task", r#"{}"#)]);
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        assert_carrier(
            &driven.responses[0],
            "invalid",
            "the repo parameter is absent and the working directory matches multiple \
             repositories: aaa, zzz",
        );
    }

    /// Test 8 — `repo_set_path` after the first derivation → the cached
    /// answer is unchanged (D3's per-process fixity; M4's kill — a
    /// re-derive-per-call bug would find the moved path and answer
    /// the no-default refusal instead).
    #[test]
    fn repo_set_path_after_the_first_derivation_keeps_the_cached_default() {
        let root = TempDir::new().expect("a fresh root");
        let checkout = TempDir::new().expect("a fresh checkout");
        let at = canonical(&checkout);
        register_at(&DataRoot::new(root.path()), "fixture", &at);
        let driven = drive_with_cwd(
            root.path(),
            None,
            &at,
            &[
                call(2, "next_task", r#"{}"#),
                call(
                    3,
                    "repo_set_path",
                    r#"{"repo":"fixture","path":"/repos/moved"}"#,
                ),
                call(4, "next_task", r#"{}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        // The first call derived and cached fixture's default.
        answers_task(&driven.responses[0], "fixture");
        // The move landed (the response carries the new path)…
        assert_eq!(
            structured(&driven.responses[1])["path"],
            json!("/repos/moved")
        );
        // …and the cached default did not move with it.
        answers_task(&driven.responses[2], "fixture");
    }

    /// Test 9 — the canonicalization pin (J3, corrected): the fixture
    /// registers the RAW spelling — `/var/folders/…`, a symlink to
    /// `/private/var/folders/…` on this host — and the OS hands the
    /// spawned binary the PHYSICAL spelling, so the two sides differ
    /// at match time and only canonicalization reconciles them (M2's
    /// kill). The spec's J3 as written (register canonical, spawn
    /// raw) pins nothing on macOS: `current_dir` resolves the spawn
    /// spelling before the binary sees it, so both sides would arrive
    /// physical and the match would survive canonicalization's
    /// deletion. On a host whose temp root is already canonical this
    /// test still passes but pins nothing; the mutation proof runs
    /// here.
    #[test]
    fn a_raw_registered_spelling_matches_through_canonicalization() {
        let root = TempDir::new().expect("a fresh root");
        let checkout = TempDir::new().expect("a fresh checkout");
        // The raw spelling — NOT canonicalized — is what registers.
        register_at(&DataRoot::new(root.path()), "fixture", checkout.path());
        let driven = drive_with_cwd(
            root.path(),
            None,
            checkout.path(),
            &[
                call(2, "next_task", r#"{}"#),
                call(3, "next_task", r#"{"repo":"fixture"}"#),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        answers_task(&driven.responses[0], "fixture");
        assert_eq!(
            structured(&driven.responses[0]),
            structured(&driven.responses[1])
        );
    }

    /// Test 10 — A sibling directory sharing a STRING prefix with the
    /// checkout (`proj1` vs `proj11`) → no match: the comparison is
    /// component-wise, never bytes (M6's kill — a string-prefix bug
    /// would match proj1 and answer its task).
    #[test]
    fn a_string_prefix_sibling_does_not_match() {
        let root = TempDir::new().expect("a fresh root");
        let parent = TempDir::new().expect("a fresh parent");
        let proj1 = canonical(&parent).join("proj1");
        let proj11 = canonical(&parent).join("proj11");
        fs::create_dir_all(&proj1).expect("proj1 lands");
        fs::create_dir_all(&proj11).expect("proj11 lands");
        register_at(&DataRoot::new(root.path()), "proj1", &proj1);
        let driven = drive_with_cwd(root.path(), None, &proj11, &[call(2, "next_task", r#"{}"#)]);
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        assert_carrier(&driven.responses[0], "invalid", NO_DEFAULT);
    }

    /// Test 11 — a registered store that will not open fails the
    /// derivation with its own error, even when the cwd matches a
    /// healthy repo: the walk reports what it finds or names the
    /// fault, never a silent subset (the `repo_list` precedent, t31 —
    /// and skipping the broken store would answer the healthy match
    /// as though the registry held one repo fewer than it does). The
    /// memo stays `Pending` on an errored derivation, so the second
    /// call re-derives and fails identically — the retry is
    /// deterministic, never a cached fault.
    #[test]
    fn a_store_that_will_not_open_fails_the_derivation() {
        let root = TempDir::new().expect("a fresh root");
        let checkout = TempDir::new().expect("a fresh checkout");
        let broken_checkout = TempDir::new().expect("a broken checkout");
        let at = canonical(&checkout);
        let data_root = DataRoot::new(root.path());
        register_at(&data_root, "fixture", &at);
        register_at(&data_root, "broken", &canonical(&broken_checkout));
        // Corrupt the second store's log after its handle is dropped:
        // a garbage line makes the open fail with `corrupt_store`.
        let log = root
            .path()
            .join("repos")
            .join("broken")
            .join("event-log.jsonl");
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&log)
            .expect("the log opens for the corruption");
        file.write_all(b"not a serialized event\n")
            .expect("the garbage lands");
        drop(file);
        let requests = vec![call(2, "next_task", r#"{}"#), call(3, "next_task", r#"{}"#)];
        let driven = drive_with_cwd(root.path(), None, &at, &requests);
        assert!(
            driven.status.success(),
            "a completed handshake ends 0; stderr: {}",
            driven.stderr
        );
        assert_eq!(driven.responses.len(), 2);
        let value: serde_json::Value =
            serde_json::from_str(&driven.responses[0]).expect("the response line is JSON");
        assert_eq!(
            value["result"]["structuredContent"]["code"],
            json!("corrupt_store"),
            "the derivation names the store's own fault: {}",
            driven.responses[0]
        );
        assert_eq!(
            structured(&driven.responses[0]),
            structured(&driven.responses[1]),
            "the errored derivation left the memo Pending — the retry re-derived and \
             re-failed identically (the lines differ only in the JSON-RPC id)"
        );
    }
}
