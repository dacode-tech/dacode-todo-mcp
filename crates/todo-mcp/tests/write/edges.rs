//! The edge and registry cluster's suite (t37): `task_link`/
//! `task_unlink`, `repo_set_path`, `repo_list`/`repo_show` and the
//! default-repo slug, through the protocol — request bytes in,
//! response bytes out, never by calling the handler around it (§10).
//! Every tool in the cluster is **clock-free** — the echo carries no
//! timestamp, and a `Repo` node has no `updated` field (§3) while its
//! `registered_at` is fixed — so all five are golden-pinned: the t36
//! principle applied to its first full cluster, the property that
//! decides being clock-freeness of the response, not readness. The
//! clock-stamp extractors stay in `write/nodes.rs` (the f54 rule:
//! this suite needs none of them).
//!
//! Every test drives a fresh fixture (the writes mutate the store).
//! The persistence tests reopen the store in-process after the drive
//! — the server has exited, its locks released — asserting the
//! mutation is on disk: the wire proved the write, the reopen proves
//! the log (the t33 precedent). The default-slug tests use the
//! `--repo` drive variant, and the swapped-store test builds its own
//! root (two registrations, an out-of-band log copy) to pin the
//! resolve cross-check U23 names.

use std::fs;
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{BlockEnd, Repo, RepoSlug, Timestamp};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, assert_golden, call, drive, drive_with, fixture, structured};
use crate::reopened;

// =================================================================
// The suite-local helpers
// =================================================================

/// Drives the fixture and asserts the responses against the golden,
/// after checking the session ended cleanly — the read suites' helper,
/// over this target's requests.
fn assert_tool_golden(requests: &[String], name: &str) {
    let root = fixture();
    let driven = drive(root.path(), requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_golden(name, driven.responses.concat().as_bytes());
}

/// The fixture repo's slug, parsed once for the `--repo` drives.
fn fixture_slug() -> RepoSlug {
    RepoSlug::from_str("fixture").unwrap()
}

// =================================================================
// The request builders — each documents the request order its tool's
// golden pins: the success responses first, then the error responses.
// =================================================================

/// `task_link`'s request order: the three new `blocks` edges
/// (task→task, decision→task, decision→epic); a new `relates`
/// annotation; the `relates` repeat (the fixture's U1↔U6 → the no-op
/// `Ok` echo); the three forbidden shapes (task→epic, epic source,
/// decision target → `invalid` naming the pair); the cycle (U2→U1
/// against the fixture's U1→U2); an unknown endpoint; a malformed id;
/// an unknown repo; a malformed slug; a bad edge spelling (rmcp's
/// codeless argument fault); an unknown field.
fn task_link_requests() -> Vec<String> {
    vec![
        call(
            2,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U2","to":"fixture:U3"}"#,
        ),
        call(
            3,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:D2","to":"fixture:U4"}"#,
        ),
        call(
            4,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:D1","to":"fixture:E2"}"#,
        ),
        call(
            5,
            "task_link",
            r#"{"repo":"fixture","edge":"relates","from":"fixture:U2","to":"fixture:U4"}"#,
        ),
        call(
            6,
            "task_link",
            r#"{"repo":"fixture","edge":"relates","from":"fixture:U1","to":"fixture:U6"}"#,
        ),
        call(
            7,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:E1"}"#,
        ),
        call(
            8,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:E1","to":"fixture:U1"}"#,
        ),
        call(
            9,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:D1"}"#,
        ),
        call(
            10,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U2","to":"fixture:U1"}"#,
        ),
        call(
            11,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U99","to":"fixture:U2"}"#,
        ),
        call(
            12,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"bad","to":"fixture:U2"}"#,
        ),
        call(
            13,
            "task_link",
            r#"{"repo":"nosuch","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        ),
        call(
            14,
            "task_link",
            r#"{"repo":"Bad!","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        ),
        call(
            15,
            "task_link",
            r#"{"repo":"fixture","edge":"chain","from":"fixture:U1","to":"fixture:U2"}"#,
        ),
        call(
            16,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:U2","wieght":"x"}"#,
        ),
    ]
}

/// `task_unlink`'s request order: the existing `blocks` edge (U1→U2);
/// a missing `blocks` edge (U2→U3 → the no-op `Ok`); the existing
/// `relates` annotation (U1↔U6); an absent one (U2↔U4 → the no-op
/// `Ok`); an unknown endpoint; a malformed id; an unknown repo; an
/// unknown field.
fn task_unlink_requests() -> Vec<String> {
    vec![
        call(
            2,
            "task_unlink",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        ),
        call(
            3,
            "task_unlink",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U2","to":"fixture:U3"}"#,
        ),
        call(
            4,
            "task_unlink",
            r#"{"repo":"fixture","edge":"relates","from":"fixture:U1","to":"fixture:U6"}"#,
        ),
        call(
            5,
            "task_unlink",
            r#"{"repo":"fixture","edge":"relates","from":"fixture:U2","to":"fixture:U4"}"#,
        ),
        call(
            6,
            "task_unlink",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U99","to":"fixture:U2"}"#,
        ),
        call(
            7,
            "task_unlink",
            r#"{"repo":"fixture","edge":"blocks","from":"bad","to":"fixture:U2"}"#,
        ),
        call(
            8,
            "task_unlink",
            r#"{"repo":"nosuch","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        ),
        call(
            9,
            "task_unlink",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:U2","wieght":"x"}"#,
        ),
    ]
}

/// `repo_set_path`'s request order: the move; an unknown repo; a
/// malformed slug; an unknown field.
fn repo_set_path_requests() -> Vec<String> {
    vec![
        call(
            2,
            "repo_set_path",
            r#"{"repo":"fixture","path":"/repos/fixture-v2"}"#,
        ),
        call(3, "repo_set_path", r#"{"repo":"nosuch","path":"/x"}"#),
        call(4, "repo_set_path", r#"{"repo":"Bad!","path":"/x"}"#),
        call(
            5,
            "repo_set_path",
            r#"{"repo":"fixture","path":"/x","paht":"/y"}"#,
        ),
    ]
}

/// `repo_list`'s request order: the listing (the fixture's
/// `[alpha, fixture]` rows); an unknown field — the tool takes no
/// argument at all, so any one is a fault.
fn repo_list_requests() -> Vec<String> {
    vec![
        call(2, "repo_list", r#"{}"#),
        call(3, "repo_list", r#"{"repo":"fixture"}"#),
    ]
}

/// `repo_show`'s request order: the fixture's `RepoDto`; an unknown
/// repo; a malformed slug; an unknown field.
fn repo_show_requests() -> Vec<String> {
    vec![
        call(2, "repo_show", r#"{"repo":"fixture"}"#),
        call(3, "repo_show", r#"{"repo":"nosuch"}"#),
        call(4, "repo_show", r#"{"repo":"Bad!"}"#),
        call(5, "repo_show", r#"{"repo":"fixture","paht":"/x"}"#),
    ]
}

// =================================================================
// The golden suite — all five tools are clock-free
// =================================================================

/// `task_link`'s request→response bytes, pinned: the three new blocks
/// edges, the relates pair and its repeat, the three forbidden
/// shapes, the cycle, and the error shapes.
#[test]
fn task_link_matches_its_golden() {
    assert_tool_golden(&task_link_requests(), "task_link.v1");
}

/// `task_unlink`'s request→response bytes, pinned: the two removals,
/// the two no-ops, and the error shapes.
#[test]
fn task_unlink_matches_its_golden() {
    assert_tool_golden(&task_unlink_requests(), "task_unlink.v1");
}

/// `repo_set_path`'s request→response bytes, pinned — plus the reopen
/// persistence assertion: the moved path is on disk (the t36 pattern:
/// the wire proved the write, the reopen proves the log).
#[test]
fn repo_set_path_matches_its_golden() {
    let root = fixture();
    let driven = drive(root.path(), &repo_set_path_requests());
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_golden("repo_set_path.v1", driven.responses.concat().as_bytes());
    let store = reopened(root.path());
    assert_eq!(store.graph().repo().path, "/repos/fixture-v2");
}

/// `repo_list`'s request→response bytes, pinned: the registry's two
/// rows in slug order, and the any-argument fault.
#[test]
fn repo_list_matches_its_golden() {
    assert_tool_golden(&repo_list_requests(), "repo_list.v1");
}

/// `repo_show`'s request→response bytes, pinned: the fixture's repo
/// node, and the error shapes.
#[test]
fn repo_show_matches_its_golden() {
    assert_tool_golden(&repo_show_requests(), "repo_show.v1");
}

// =================================================================
// The persistence — the edges land on disk
// =================================================================

/// Link and unlink land on disk: the reopen shows `blocked_by`/
/// `relates_of` reflecting the driven mutations — the wire proved the
/// writes, the reopen proves the log.
#[test]
fn the_edges_persist() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "task_link",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:U2","to":"fixture:U3"}"#,
            ),
            call(
                3,
                "task_link",
                r#"{"repo":"fixture","edge":"relates","from":"fixture:U2","to":"fixture:U4"}"#,
            ),
            call(
                4,
                "task_unlink",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
            ),
            call(
                5,
                "task_unlink",
                r#"{"repo":"fixture","edge":"relates","from":"fixture:U1","to":"fixture:U6"}"#,
            ),
            // The duplicate blocks edge: U1→U3 exists in the fixture,
            // and the domain's answer is the no-op `Ok` — not
            // `duplicate`, not an error. A wire adding its own
            // rejection here would fail the success-echo loop below.
            call(
                6,
                "task_link",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:U3"}"#,
            ),
            // The relates self-annotation: accepted and stored (the
            // domain's contract — `relates` gates nothing, so there
            // is no cycle and no code to reject it with; "inert"
            // means it gates nothing, not that it is absent). The
            // reopen below shows `relates_of(U1)` as the
            // self-annotation alone: U1↔U6 is unlinked and the
            // self-pair is listed once.
            call(
                7,
                "task_link",
                r#"{"repo":"fixture","edge":"relates","from":"fixture:U1","to":"fixture:U1"}"#,
            ),
        ],
    );
    // Every response is the success echo.
    for response in &driven.responses {
        let value: Value = serde_json::from_str(response).expect("the response line is JSON");
        assert_eq!(value["result"]["isError"], json!(false), "{response}");
    }
    let store = reopened(root.path());
    let graph = store.graph();
    // U2→U3 landed: U3's predecessors are U1 and U2 (own edges,
    // ascending) then D1 — the fixture's direct gate, the two-run
    // order the domain documents.
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task("fixture:U3".parse().unwrap())),
        vec![
            BlockEnd::Task("fixture:U1".parse().unwrap()),
            BlockEnd::Task("fixture:U2".parse().unwrap()),
            BlockEnd::Decision("fixture:D1".parse().unwrap()),
        ]
    );
    // U1→U2 unlinked: U2's predecessors are D2 alone.
    assert_eq!(
        graph.blocked_by(&BlockEnd::Task("fixture:U2".parse().unwrap())),
        vec![BlockEnd::Decision("fixture:D2".parse().unwrap())]
    );
    // U2↔U4 landed; U1↔U6 unlinked.
    assert_eq!(
        graph.relates_of(&"fixture:U2".parse().unwrap()),
        vec!["fixture:U4".parse().unwrap()]
    );
    // U1's annotations after the drive: U1↔U6 unlinked, the
    // self-annotation stored and listed once.
    assert_eq!(
        graph.relates_of(&"fixture:U1".parse().unwrap()),
        vec!["fixture:U1".parse().unwrap()]
    );
}

// =================================================================
// f4's criterion — the forbidden endpoint pairs
// =================================================================

/// f4's named criterion as its own test: the three forbidden shapes
/// each answer `invalid` via the carrier, the message naming both
/// endpoints (the golden pins the bytes; this pins the criterion's
/// visibility).
#[test]
fn the_forbidden_endpoint_pairs_are_rejected_at_the_wire() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "task_link",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:E1"}"#,
            ),
            call(
                3,
                "task_link",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:E1","to":"fixture:U1"}"#,
            ),
            call(
                4,
                "task_link",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:D1"}"#,
            ),
        ],
    );
    let pairs = [
        ("fixture:U1", "fixture:E1"),
        ("fixture:E1", "fixture:U1"),
        ("fixture:U1", "fixture:D1"),
    ];
    for (response, (from, to)) in driven.responses.iter().zip(pairs) {
        let value: Value = serde_json::from_str(response).expect("the response line is JSON");
        assert_eq!(
            value["result"]["structuredContent"]["code"],
            json!("invalid"),
            "{response}"
        );
        let message = value["result"]["structuredContent"]["message"]
            .as_str()
            .expect("the message");
        assert!(
            message.contains(from) && message.contains(to),
            "the message names both endpoints: {message}"
        );
    }
}

// =================================================================
// The cycle
// =================================================================

/// The cycle rejection carries §6.1's `cycle` code and the domain's
/// deterministic `[s, t, …, s]` path in the message — the tracker
/// verify line's named case.
#[test]
fn the_cycle_rejection_carries_the_cycle_code() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[call(
            2,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U2","to":"fixture:U1"}"#,
        )],
    );
    assert_carrier(
        &driven.responses[0],
        "cycle",
        "cycle: fixture:U2 -> fixture:U1 -> fixture:U2",
    );
}

// =================================================================
// The default slug (the committed D8)
// =================================================================

/// The default slug is applied when the repo parameter is absent and
/// overridden when present (the committed D8): with `--repo fixture`
/// configured, `epic_list` with no repo answers the default's epics,
/// `next_task` with an epic and no repo answers the default's, and
/// `epic_list` with `repo: "alpha"` answers alpha's — the explicit
/// wins. With no `--repo` configured, an absent repo answers
/// `invalid` naming both default tiers' failure (the message the
/// assertion below pins).
#[test]
fn the_default_slug_is_applied_when_absent_and_overridden_when_present() {
    let root = fixture();
    let driven = drive_with(
        root.path(),
        Some(&fixture_slug()),
        &[
            call(2, "epic_list", r#"{}"#),
            call(3, "next_task", r#"{"epic":"fixture:E1"}"#),
            call(4, "epic_list", r#"{"repo":"alpha"}"#),
        ],
    );
    // The default applied: fixture's epics, not alpha's.
    assert_eq!(
        structured(&driven.responses[0])["epics"],
        json!([
            {"id": "fixture:E1", "title": "Bring-up"},
            {"id": "fixture:E2", "title": "Runway"},
        ])
    );
    // The default applied with an epic filter: E1's answer — gated,
    // null, with the epic's deferred summary (the next_task.v1 golden
    // pins the same bytes).
    assert_eq!(
        structured(&driven.responses[1]),
        json!({"task": null, "priority": null, "deferred": {"total": 1, "ready": 0}})
    );
    // The explicit wins: alpha's epics.
    assert_eq!(
        structured(&driven.responses[2])["epics"],
        json!([{"id": "alpha:E1", "title": "Alpha epic"}])
    );
    // No default configured: an absent repo is `invalid` from the
    // tool body, not a codeless argument fault. The fixture's
    // registered paths (/repos/*) do not exist, so the cwd tier
    // derives no default either — the message names both tiers'
    // failure (t53's D4).
    let bare = fixture();
    let driven_bare = drive(bare.path(), &[call(2, "epic_list", r#"{}"#)]);
    assert_carrier(
        &driven_bare.responses[0],
        "invalid",
        "the repo parameter is absent and neither the --repo flag nor the working \
         directory produced a default repo",
    );
}

// =================================================================
// §6's client flow
// =================================================================

/// §6's signed client flow end-to-end over the wire with the default
/// configured: `repo_list` → `epic_list` (no repo — the default) →
/// `next_task` (the epic, no repo).
#[test]
fn the_client_flow_repo_list_epic_list_next_task() {
    let root = fixture();
    let driven = drive_with(
        root.path(),
        Some(&fixture_slug()),
        &[
            call(2, "repo_list", r#"{}"#),
            call(3, "epic_list", r#"{}"#),
            call(4, "next_task", r#"{"epic":"fixture:E2"}"#),
        ],
    );
    // repo_list: the full registry set, ascending by slug.
    assert_eq!(
        structured(&driven.responses[0])["repos"],
        json!([
            {
                "slug": "alpha",
                "name": "Alpha",
                "path": "/repos/alpha",
                "registered_at": "2026-10-01T09:00:00.000000000Z",
            },
            {
                "slug": "fixture",
                "name": "Fixture",
                "path": "/repos/fixture",
                "registered_at": "2026-10-01T08:00:00.000000000Z",
            },
        ])
    );
    // epic_list: the default's epics.
    assert_eq!(
        structured(&driven.responses[1])["epics"],
        json!([
            {"id": "fixture:E1", "title": "Bring-up"},
            {"id": "fixture:E2", "title": "Runway"},
        ])
    );
    // next_task: the chosen epic's answer — E2's first actionable
    // task with its priority (D17) and the deferred summary.
    assert_eq!(
        structured(&driven.responses[2]),
        json!({"task": "fixture:U9", "priority": "high", "deferred": {"total": 2, "ready": 1}})
    );
}

// =================================================================
// U20(b) — repo_list reads the held handles
// =================================================================

/// `repo_list` after a query reads the held handles (U20(b)): a query
/// first — opening fixture's handle and holding its advisory lock —
/// then `repo_list` answers the full registry set. A re-opening
/// implementation would contend with its own held lock and answer
/// `locked` (t31's measured self-contention trap); the resolve path
/// answers the set.
#[test]
fn repo_list_after_a_query_reads_the_held_handles() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "next_task", r#"{"repo":"fixture"}"#),
            call(3, "repo_list", r#"{}"#),
        ],
    );
    assert_eq!(
        structured(&driven.responses[1])["repos"],
        json!([
            {
                "slug": "alpha",
                "name": "Alpha",
                "path": "/repos/alpha",
                "registered_at": "2026-10-01T09:00:00.000000000Z",
            },
            {
                "slug": "fixture",
                "name": "Fixture",
                "path": "/repos/fixture",
                "registered_at": "2026-10-01T08:00:00.000000000Z",
            },
        ]),
        "the full registry set, not a locked fault"
    );
}

// =================================================================
// The parse-before-resolve order
// =================================================================

/// Malformed ids and slugs carry `invalid` before the store is
/// touched — and a malformed id outranks an unknown repo: the parse
/// happens before the resolve, so the calls naming an unknown repo
/// still answer `invalid`. A doc id (`S`-prefixed) is accepted by no
/// endpoint parser and answers the same `invalid node id`; a
/// non-task endpoint on a `relates` link is the wire's own
/// forbidden-pair rejection.
#[test]
fn malformed_ids_and_slugs_carry_invalid_before_the_store_is_touched() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "task_link",
                r#"{"repo":"nosuch","edge":"blocks","from":"bad","to":"fixture:U2"}"#,
            ),
            call(
                3,
                "task_unlink",
                r#"{"repo":"nosuch","edge":"relates","from":"bad","to":"fixture:U1"}"#,
            ),
            call(
                4,
                "task_link",
                r#"{"repo":"Bad!","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
            ),
            call(
                5,
                "task_unlink",
                r#"{"repo":"Bad!","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
            ),
            call(6, "repo_set_path", r#"{"repo":"Bad!","path":"/x"}"#),
            call(7, "repo_show", r#"{"repo":"Bad!"}"#),
            call(
                8,
                "task_link",
                r#"{"repo":"fixture","edge":"blocks","from":"fixture:S1","to":"fixture:U2"}"#,
            ),
            call(
                9,
                "task_link",
                r#"{"repo":"fixture","edge":"relates","from":"fixture:E1","to":"fixture:U1"}"#,
            ),
        ],
    );
    let expected = [
        "invalid node id: \"bad\"",
        "invalid node id: \"bad\"",
        "invalid repository slug: \"Bad!\"",
        "invalid repository slug: \"Bad!\"",
        "invalid repository slug: \"Bad!\"",
        "invalid repository slug: \"Bad!\"",
        "invalid node id: \"fixture:S1\"",
        "the relates pair fixture:E1 <-> fixture:U1 is forbidden: relates is task-to-task",
    ];
    for (response, message) in driven.responses.iter().zip(expected) {
        assert_carrier(response, "invalid", message);
    }
}

// =================================================================
// Determinism — the clock-free tools
// =================================================================

/// The clock-free tools answer byte-identically across two fresh
/// spawns (§8 — two runs over equal state return identical bytes):
/// `task_link`'s echo and `repo_list`'s rows carry no clock-read
/// byte, so nothing the clock could leak reaches the response. Two
/// fresh fixtures hold the equal state — a writer's second run over
/// the *same* root would see the first's mutation, which is a
/// different question.
#[test]
fn the_clock_free_tools_answer_byte_identically_twice() {
    let requests = vec![
        call(
            2,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U2","to":"fixture:U3"}"#,
        ),
        call(3, "repo_list", r#"{}"#),
    ];
    let root_a = fixture();
    let driven_a = drive(root_a.path(), &requests);
    let root_b = fixture();
    let driven_b = drive(root_b.path(), &requests);
    assert_eq!(
        driven_a.responses, driven_b.responses,
        "two runs over equal state answer byte-identical bytes"
    );
}

// =================================================================
// U23 — the swapped store
// =================================================================

/// A swapped store is refused, not adopted (U23): a test-local root
/// registers `aaa` and `bbb` (two registrations, handles dropped),
/// `aaa`'s event log is copied over `bbb`'s out of band, and
/// `repo_show` on `bbb` answers `corrupt_store` naming both slugs —
/// the resolve cross-check (live since t34, pinned here) never
/// serves `bbb` the graph `aaa` built.
#[test]
fn a_swapped_store_is_refused_not_adopted() {
    let root = TempDir::new().expect("a fresh temp root");
    let data_root = DataRoot::new(root.path());
    let repo = |slug: &str, at: &str| Repo {
        slug: RepoSlug::from_str(slug).expect("a valid slug"),
        name: slug.to_owned(),
        path: format!("/repos/{slug}"),
        registered_at: Timestamp::parse(at).expect("a fixed canonical timestamp"),
    };
    drop(
        data_root
            .register(repo("aaa", "2026-10-01T10:00:00.000000000Z"))
            .expect("aaa registers"),
    );
    drop(
        data_root
            .register(repo("bbb", "2026-10-01T10:01:00.000000000Z"))
            .expect("bbb registers"),
    );
    // The out-of-band swap: aaa's log over bbb's.
    fs::copy(
        root.path().join("repos/aaa/event-log.jsonl"),
        root.path().join("repos/bbb/event-log.jsonl"),
    )
    .expect("the swap lands");
    let driven = drive(root.path(), &[call(2, "repo_show", r#"{"repo":"bbb"}"#)]);
    assert_carrier(
        &driven.responses[0],
        "corrupt_store",
        "corrupt store: the store opened for bbb holds repo aaa",
    );
}
