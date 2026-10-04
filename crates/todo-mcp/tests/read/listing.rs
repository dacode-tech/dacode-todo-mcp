//! The listing cluster's suite (t35): the five read tools through the
//! protocol — request bytes in, response bytes out, never by calling
//! the handler around it (§10) — with one golden per tool holding the
//! tool's full JSON-RPC response lines in the request order each
//! builder below documents. The goldens pin the bytes; the domain
//! agreement test pins the semantics against the domain's own answers
//! computed in-process over the same fixture, so a wire shape that is
//! deterministic but wrong cannot pass; the determinism test sends
//! the same request twice and compares bytes (§8).

use std::str::FromStr;

use serde_json::{Value, json};
use todo_domain::{
    BlockEnd, Blocker, EpicId, Graph, RepoSlug, Status, TaskFilter, TaskId, TaskKind, TaskRow,
};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, assert_golden, call, drive, fixture, structured};

// =================================================================
// The request builders — each documents the request order its tool's
// golden pins: the success responses first, then the error responses
// (an unknown repo, an unknown epic, a malformed id or slug, and the
// unknown argument field, rmcp's own codeless argument fault, folded
// into the golden as the last request per tool).
// =================================================================

/// `task_list`'s request order: the repo scope, the two epic scopes,
/// each value filter alone, the combined and the contradictory
/// combinations, an unknown repo, an unknown epic, a malformed epic
/// id, a malformed slug, an unknown argument field.
fn task_list_requests() -> Vec<String> {
    vec![
        call(2, "task_list", r#"{"repo":"fixture"}"#),
        call(3, "task_list", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
        call(4, "task_list", r#"{"repo":"fixture","epic":"fixture:E2"}"#),
        call(5, "task_list", r#"{"repo":"fixture","status":"open"}"#),
        call(6, "task_list", r#"{"repo":"fixture","status":"deferred"}"#),
        call(7, "task_list", r#"{"repo":"fixture","ready":true}"#),
        call(8, "task_list", r#"{"repo":"fixture","ready":false}"#),
        call(
            9,
            "task_list",
            r#"{"repo":"fixture","status":"deferred","ready":true}"#,
        ),
        call(10, "task_list", r#"{"repo":"fixture","kind":"gate"}"#),
        call(
            11,
            "task_list",
            r#"{"repo":"fixture","status":"closed","ready":true}"#,
        ),
        call(12, "task_list", r#"{"repo":"nosuch"}"#),
        call(13, "task_list", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
        call(14, "task_list", r#"{"repo":"fixture","epic":"fixture:U1"}"#),
        call(15, "task_list", r#"{"repo":"Bad!"}"#),
        call(16, "task_list", r#"{"repo":"fixture","statuz":"open"}"#),
    ]
}

/// `epic_list`'s request order: the repo scope, an unknown repo, a
/// malformed slug, an unknown argument field — `epic`, the filter
/// §6 gives `task_list` alone, so the thin listing rejects it.
fn epic_list_requests() -> Vec<String> {
    vec![
        call(2, "epic_list", r#"{"repo":"fixture"}"#),
        call(3, "epic_list", r#"{"repo":"nosuch"}"#),
        call(4, "epic_list", r#"{"repo":"Bad!"}"#),
        call(5, "epic_list", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
    ]
}

/// `decision_list`'s request order: the same four shapes as
/// `epic_list`'s, over this tool.
fn decision_list_requests() -> Vec<String> {
    vec![
        call(2, "decision_list", r#"{"repo":"fixture"}"#),
        call(3, "decision_list", r#"{"repo":"nosuch"}"#),
        call(4, "decision_list", r#"{"repo":"Bad!"}"#),
        call(
            5,
            "decision_list",
            r#"{"repo":"fixture","epic":"fixture:E1"}"#,
        ),
    ]
}

/// `doc_list`'s request order: the same four shapes as
/// `epic_list`'s, over this tool.
fn doc_list_requests() -> Vec<String> {
    vec![
        call(2, "doc_list", r#"{"repo":"fixture"}"#),
        call(3, "doc_list", r#"{"repo":"nosuch"}"#),
        call(4, "doc_list", r#"{"repo":"Bad!"}"#),
        call(5, "doc_list", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
    ]
}

/// `graph_show`'s request order: the all-repos scope, the repo scope,
/// the epic scope in its derived-repo form, an unknown repo, a
/// cross-repo epic under an explicit repo, an unknown epic, a
/// malformed epic id, a malformed slug, an unknown argument field.
fn graph_show_requests() -> Vec<String> {
    vec![
        call(2, "graph_show", r#"{}"#),
        call(3, "graph_show", r#"{"repo":"fixture"}"#),
        call(4, "graph_show", r#"{"epic":"fixture:E1"}"#),
        call(5, "graph_show", r#"{"repo":"nosuch"}"#),
        call(6, "graph_show", r#"{"repo":"fixture","epic":"alpha:E1"}"#),
        call(7, "graph_show", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
        call(8, "graph_show", r#"{"epic":"fixture:U1"}"#),
        call(9, "graph_show", r#"{"repo":"Bad!"}"#),
        call(
            10,
            "graph_show",
            r#"{"repo":"fixture","epik":"fixture:E1"}"#,
        ),
    ]
}

// =================================================================
// The golden suite
// =================================================================

/// Drives the fixture and asserts the responses against the golden,
/// after checking the session ended cleanly.
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

/// `task_list`'s request→response bytes, pinned: the repo and epic
/// scopes, every filter alone and combined, the contradictory
/// combination, and the error shapes.
#[test]
fn task_list_matches_its_golden() {
    assert_tool_golden(&task_list_requests(), "task_list.v1");
}

/// `epic_list`'s request→response bytes, pinned: the repo scope and
/// the error shapes.
#[test]
fn epic_list_matches_its_golden() {
    assert_tool_golden(&epic_list_requests(), "epic_list.v1");
}

/// `decision_list`'s request→response bytes, pinned: the repo scope
/// and the error shapes.
#[test]
fn decision_list_matches_its_golden() {
    assert_tool_golden(&decision_list_requests(), "decision_list.v1");
}

/// `doc_list`'s request→response bytes, pinned: the repo scope and
/// the error shapes.
#[test]
fn doc_list_matches_its_golden() {
    assert_tool_golden(&doc_list_requests(), "doc_list.v1");
}

/// `graph_show`'s request→response bytes, pinned: the three scopes
/// and the error shapes.
#[test]
fn graph_show_matches_its_golden() {
    assert_tool_golden(&graph_show_requests(), "graph_show.v1");
}

// =================================================================
// The domain agreement
// =================================================================

/// The wire agrees with the domain, field-for-field, over the same
/// fixture: the domain's answers computed in-process (after the drive —
/// the advisory lock is per process) against the parsed wire
/// responses. The golden pins the bytes; this pins the semantics, so a
/// wire shape that is deterministic but wrong cannot pass.
#[test]
fn the_wire_agrees_with_the_domain_over_the_same_fixture() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "task_list", r#"{"repo":"fixture"}"#),
            call(3, "task_list", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
            call(4, "epic_list", r#"{"repo":"fixture"}"#),
            call(5, "decision_list", r#"{"repo":"fixture"}"#),
            call(6, "doc_list", r#"{"repo":"fixture"}"#),
            call(7, "graph_show", r#"{"repo":"fixture"}"#),
            call(8, "graph_show", r#"{"epic":"fixture:E1"}"#),
        ],
    );
    // The drive's server has exited, so its locks are released and the
    // test process may open the store itself.
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").unwrap())
        .unwrap();
    let graph = store.graph();

    // task_list: the repo scope and the epic scope, field-for-field
    // including every blocker's status and the two-run order.
    for (response, filter) in [
        (&driven.responses[0], TaskFilter::default()),
        (
            &driven.responses[1],
            TaskFilter {
                epic: Some("fixture:E1".parse().unwrap()),
                ..Default::default()
            },
        ),
    ] {
        let wire = structured(response);
        let answer = graph.task_list(&filter).unwrap();
        assert_task_rows(&wire["tasks"], &answer);
    }

    // The three thin listings: the domain's own ascending orders, the
    // titles read off the nodes.
    let wire = structured(&driven.responses[2]);
    let expected: Vec<Value> = graph
        .epic_list()
        .iter()
        .map(|id| {
            json!({
                "id": id.to_string(),
                "title": graph.epic(id).map(|node| node.title.clone()).unwrap_or_default(),
            })
        })
        .collect();
    assert_eq!(wire["epics"], json!(expected));

    let wire = structured(&driven.responses[3]);
    let expected: Vec<Value> = graph
        .decision_list()
        .iter()
        .map(|id| {
            json!({
                "id": id.to_string(),
                "title": graph.decision(id).map(|node| node.title.clone()).unwrap_or_default(),
            })
        })
        .collect();
    assert_eq!(wire["decisions"], json!(expected));

    let wire = structured(&driven.responses[4]);
    let expected: Vec<Value> = graph
        .doc_list()
        .iter()
        .map(|id| {
            json!({
                "id": id.to_string(),
                "title": graph.doc(id).map(|node| node.title.clone()).unwrap_or_default(),
            })
        })
        .collect();
    assert_eq!(wire["docs"], json!(expected));

    // graph_show, repo scope: the repo node, the epic groups, the
    // decisions with their blocks, the docs.
    let wire = structured(&driven.responses[5]);
    let repos = wire["repos"].as_array().expect("the repos array");
    assert_eq!(repos.len(), 1, "the repo scope holds one repo");
    assert_dump(&repos[0], graph);

    // graph_show, epic scope: the one group checked the same way, and
    // the decision set against the domain's gate-set surface.
    let wire = structured(&driven.responses[6]);
    let repos = wire["repos"].as_array().expect("the repos array");
    assert_eq!(repos.len(), 1, "the epic scope holds one repo");
    let dump = &repos[0];
    let groups = dump["epics"].as_array().expect("the epic groups");
    assert_eq!(groups.len(), 1, "the epic scope holds one group");
    assert_group(&groups[0], graph, &"fixture:E1".parse().unwrap());
    let gates = graph
        .gating_decisions_of_epic(&"fixture:E1".parse().unwrap())
        .unwrap();
    let wire_ids: Vec<&str> = dump["decisions"]
        .as_array()
        .expect("the gate set")
        .iter()
        .map(|node| node["id"].as_str().expect("a decision id"))
        .collect();
    let expected: Vec<String> = gates.iter().map(ToString::to_string).collect();
    assert_eq!(wire_ids, expected);
    assert_eq!(dump["docs"], json!([]));
}

// =================================================================
// The filters
// =================================================================

/// The filters AND; `ready` is deferred-scoped (a non-deferred
/// until-satisfied task is not returned); a contradictory combination
/// is an empty `Ok`, not an error; the epic filter's row order is the
/// epic's induced canonical order, read in-process.
#[test]
fn filters_and_and_ready_is_deferred_scoped_and_contradictions_are_empty() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "task_list", r#"{"repo":"fixture","status":"open"}"#),
            call(3, "task_list", r#"{"repo":"fixture","status":"deferred"}"#),
            call(4, "task_list", r#"{"repo":"fixture","kind":"gate"}"#),
            call(5, "task_list", r#"{"repo":"fixture","ready":true}"#),
            call(6, "task_list", r#"{"repo":"fixture","ready":false}"#),
            call(
                7,
                "task_list",
                r#"{"repo":"fixture","status":"deferred","ready":true}"#,
            ),
            call(
                8,
                "task_list",
                r#"{"repo":"fixture","status":"closed","ready":true}"#,
            ),
            call(9, "task_list", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
        ],
    );
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").unwrap())
        .unwrap();
    let graph = store.graph();

    // Each filter alone, and the combinations AND — against the
    // domain's own answers over the same fixture.
    for (response, filter) in [
        (
            &driven.responses[0],
            TaskFilter {
                status: Some(Status::Open),
                ..Default::default()
            },
        ),
        (
            &driven.responses[1],
            TaskFilter {
                status: Some(Status::Deferred),
                ..Default::default()
            },
        ),
        (
            &driven.responses[2],
            TaskFilter {
                kind: Some(TaskKind::Gate),
                ..Default::default()
            },
        ),
        (
            &driven.responses[3],
            TaskFilter {
                ready: Some(true),
                ..Default::default()
            },
        ),
        (
            &driven.responses[4],
            TaskFilter {
                ready: Some(false),
                ..Default::default()
            },
        ),
        (
            &driven.responses[5],
            TaskFilter {
                status: Some(Status::Deferred),
                ready: Some(true),
                ..Default::default()
            },
        ),
        (
            &driven.responses[6],
            TaskFilter {
                status: Some(Status::Closed),
                ready: Some(true),
                ..Default::default()
            },
        ),
    ] {
        let wire = structured(response);
        let answer = graph.task_list(&filter).unwrap();
        assert_task_rows(&wire["tasks"], &answer);
    }

    // The deferred semantics the ready filters exercise, pinned on the
    // wire: U8 held (its until-target is closed but the epic gate is
    // open), U20 held (its until-target is open), U21 ready — and U10,
    // open and until-satisfied, is NOT returned by `ready=true`.
    let ready_wire = structured(&driven.responses[3]);
    let ready_true = row_task_ids(&ready_wire["tasks"]);
    assert_eq!(ready_true, ["fixture:U21"]);
    let held_wire = structured(&driven.responses[4]);
    let ready_false = row_task_ids(&held_wire["tasks"]);
    assert_eq!(ready_false, ["fixture:U8", "fixture:U20"]);

    // The contradictory combination is an empty `Ok`, not an error —
    // §6.1's code list is closed and holds no code for "no matches".
    let value: Value = serde_json::from_str(&driven.responses[6]).unwrap();
    assert_eq!(value["result"]["isError"], json!(false));
    let empty_wire = structured(&driven.responses[6]);
    assert_eq!(empty_wire["tasks"], json!([]));

    // The epic filter's row order equals `canonical_order_of_epic`'s
    // answer, read in-process — the induced order, not the repo order
    // restricted to the epic's ids.
    let epic_wire = structured(&driven.responses[7]);
    let epic_rows = row_task_ids(&epic_wire["tasks"]);
    let order: Vec<String> = graph
        .canonical_order_of_epic(&"fixture:E1".parse().unwrap())
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(epic_rows, order);
}

// =================================================================
// The two-run blocked_by order
// =================================================================

/// The two-run `blocked_by` order reaches the wire: U2's row carries
/// its own edges (U1, D2) and then the epic gate (D1) — D2 before D1
/// even though D1 < D2, which a globally sorted list would not carry —
/// and U3's row, gated by the same decision both directly and through
/// its epic, lists D1 exactly once, in the own-edges run. The full
/// blocker objects are asserted — shape, id, status — not just ids.
#[test]
fn the_two_run_blocked_by_order_reaches_the_wire() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[call(2, "task_list", r#"{"repo":"fixture"}"#)],
    );
    let wire = structured(&driven.responses[0]);
    let rows = wire["tasks"].as_array().expect("the rows are an array");
    let row_of = |id: &str| {
        rows.iter()
            .find(|row| row["task"] == json!(id))
            .unwrap_or_else(|| panic!("the row for {id} is in the listing"))
    };
    assert_eq!(
        row_of("fixture:U2")["blocked_by"],
        json!([
            {"shape":"task","id":"fixture:U1","status":"open"},
            {"shape":"decision","id":"fixture:D2","status":"open"},
            {"shape":"decision","id":"fixture:D1","status":"open"},
        ]),
        "own edges first, then the epic gate — not one sorted list"
    );
    assert_eq!(
        row_of("fixture:U3")["blocked_by"],
        json!([
            {"shape":"task","id":"fixture:U1","status":"open"},
            {"shape":"decision","id":"fixture:D1","status":"open"},
        ]),
        "the decision gating both the task and its epic appears once"
    );
}

// =================================================================
// The epic scope
// =================================================================

/// The epic scope is the epic's subtree plus its gating decisions:
/// the repo node, one epic group (E1's eight tasks in induced order,
/// the epic node's `blocked_by` = `[D1]`), the gate set `[D1, D2]` —
/// D3 absent, it gates U10 in E2 — and no docs. The repo+epic form
/// answers the same payload as epic-alone (the parsed `result`
/// compared — the response lines differ only in the request id), and
/// a cross-repo epic under an explicit repo is `unknown_id`.
#[test]
fn the_epic_scope_is_the_subtree_plus_its_gating_decisions() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "graph_show", r#"{"epic":"fixture:E1"}"#),
            call(3, "graph_show", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
            call(4, "graph_show", r#"{"repo":"fixture","epic":"alpha:E1"}"#),
        ],
    );
    let wire = structured(&driven.responses[0]);
    let repos = wire["repos"].as_array().expect("the repos array");
    assert_eq!(repos.len(), 1, "the epic scope holds one repo");
    let dump = &repos[0];
    // The repo node rides in the dump.
    assert_eq!(dump["repo"]["slug"], json!("fixture"));
    // One epic group: E1, its own gate, its eight tasks.
    let groups = dump["epics"].as_array().expect("the epic groups");
    assert_eq!(groups.len(), 1, "the epic scope holds one group");
    let group = &groups[0];
    assert_eq!(group["id"], json!("fixture:E1"));
    assert_eq!(group["blocked_by"], json!(["fixture:D1"]));
    let tasks = group["tasks"].as_array().expect("the group's tasks");
    assert_eq!(tasks.len(), 8, "E1 holds eight tasks");
    // The induced order, read in-process.
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").unwrap())
        .unwrap();
    let order: Vec<String> = store
        .graph()
        .canonical_order_of_epic(&"fixture:E1".parse().unwrap())
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect();
    let wire_ids: Vec<&str> = tasks
        .iter()
        .map(|task| task["id"].as_str().expect("a task id"))
        .collect();
    assert_eq!(wire_ids, order);
    // The gate set: D1 and D2, D3 absent — it gates U10 in E2.
    let decisions = dump["decisions"].as_array().expect("the gate set");
    let decision_ids: Vec<&str> = decisions
        .iter()
        .map(|node| node["id"].as_str().expect("a decision id"))
        .collect();
    assert_eq!(decision_ids, ["fixture:D1", "fixture:D2"]);
    // A doc has no epic field: the epic scope holds none.
    assert_eq!(dump["docs"], json!([]));
    // The repo+epic form answers the same payload as epic-alone — the
    // parsed `result` compared, the lines differing only in the id.
    let alone: Value = serde_json::from_str(&driven.responses[0]).unwrap();
    let explicit: Value = serde_json::from_str(&driven.responses[1]).unwrap();
    assert_eq!(alone["result"], explicit["result"]);
    // A cross-repo epic under an explicit repo is `unknown_id` — the
    // domain's own answer for an epic the graph does not hold.
    assert_carrier(&driven.responses[2], "unknown_id", "unknown id: alpha:E1");
}

// =================================================================
// The priority echo (D17)
// =================================================================

/// `task_list`'s rows echo the agent-set priority — the dashboard
/// column, D17's stated motivation: the fixture's U1/U9/U10/U11 pin
/// high/high/medium/low beside the nulls, and an unset priority
/// serializes as `null` with the key present, never omitted (the
/// `skip_serializing_if` drift dies on the key-presence half —
/// `serde_json`'s indexing answers `Null` for a missing key too).
#[test]
fn task_list_rows_carry_their_priority() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[call(2, "task_list", r#"{"repo":"fixture"}"#)],
    );
    let wire = structured(&driven.responses[0]);
    let rows = wire["tasks"].as_array().expect("the rows are an array");
    let row_of = |id: &str| {
        rows.iter()
            .find(|row| row["task"] == json!(id))
            .unwrap_or_else(|| panic!("the row for {id} is in the listing"))
    };
    assert_eq!(row_of("fixture:U1")["priority"], json!("high"));
    assert_eq!(row_of("fixture:U9")["priority"], json!("high"));
    assert_eq!(row_of("fixture:U10")["priority"], json!("medium"));
    assert_eq!(row_of("fixture:U11")["priority"], json!("low"));
    // The null spelling: present, null.
    let plain = row_of("fixture:U12")
        .as_object()
        .expect("a row is an object")
        .clone();
    assert!(plain.contains_key("priority"), "the key is present");
    assert_eq!(plain["priority"], json!(null));
}

// =================================================================
// The all-repos scope
// =================================================================

/// The all-repos scope is sorted by slug: the no-args response's
/// `repos` array is `[alpha, fixture]` — alpha is registered after
/// `fixture` and timestamped after it too, so the pinned order is
/// the registry's slug order, distinguishable from both arrival and
/// timestamp order.
#[test]
fn the_all_repos_scope_is_sorted_by_slug() {
    let root = fixture();
    let driven = drive(root.path(), &[call(2, "graph_show", r#"{}"#)]);
    let wire = structured(&driven.responses[0]);
    let repos = wire["repos"].as_array().expect("the repos array");
    let slugs: Vec<&str> = repos
        .iter()
        .map(|dump| dump["repo"]["slug"].as_str().expect("a slug"))
        .collect();
    assert_eq!(slugs, ["alpha", "fixture"]);
}

// =================================================================
// The unknown-id codes via the carrier
// =================================================================

/// Unknown repos and epics carry `unknown_id` via D15's carrier
/// across the cluster's tools: `isError: true`,
/// `structuredContent.code`, the same JSON mirrored into the text
/// block.
#[test]
fn unknown_ids_carry_their_codes_via_the_carrier() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "task_list", r#"{"repo":"nosuch"}"#),
            call(3, "task_list", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
            call(4, "epic_list", r#"{"repo":"nosuch"}"#),
            call(5, "decision_list", r#"{"repo":"nosuch"}"#),
            call(6, "doc_list", r#"{"repo":"nosuch"}"#),
            call(7, "graph_show", r#"{"repo":"nosuch"}"#),
            call(8, "graph_show", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
            call(9, "graph_show", r#"{"repo":"fixture","epic":"alpha:E1"}"#),
        ],
    );
    let expected = [
        "unknown id: nosuch",
        "unknown id: fixture:E9",
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: fixture:E9",
        "unknown id: alpha:E1",
    ];
    for (response, message) in driven.responses.iter().zip(expected) {
        assert_carrier(response, "unknown_id", message);
    }
}

// =================================================================
// Malformed ids and slugs
// =================================================================

/// Malformed slugs and ids carry `invalid` (U9's mapping of the
/// domain's `ValueError`) before the store is touched — and a
/// malformed id outranks an unknown repo: both parameters parse
/// before any resolve, so the parse fault answers first.
#[test]
fn malformed_ids_and_slugs_carry_invalid_before_the_store_is_touched() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "task_list", r#"{"repo":"Bad!"}"#),
            call(3, "task_list", r#"{"repo":"fixture","epic":"fixture:U1"}"#),
            call(4, "epic_list", r#"{"repo":"Bad!"}"#),
            call(5, "decision_list", r#"{"repo":"Bad!"}"#),
            call(6, "doc_list", r#"{"repo":"Bad!"}"#),
            call(7, "graph_show", r#"{"repo":"Bad!"}"#),
            call(8, "graph_show", r#"{"epic":"fixture:U1"}"#),
            call(9, "task_list", r#"{"repo":"nosuch","epic":"bad"}"#),
            call(10, "graph_show", r#"{"repo":"nosuch","epic":"fixture:U1"}"#),
        ],
    );
    let expected = [
        "invalid repository slug: \"Bad!\"",
        "invalid node id: \"fixture:U1\"",
        "invalid repository slug: \"Bad!\"",
        "invalid repository slug: \"Bad!\"",
        "invalid repository slug: \"Bad!\"",
        "invalid repository slug: \"Bad!\"",
        "invalid node id: \"fixture:U1\"",
        "invalid node id: \"bad\"",
        "invalid node id: \"fixture:U1\"",
    ];
    for (response, message) in driven.responses.iter().zip(expected) {
        assert_carrier(response, "invalid", message);
    }
}

// =================================================================
// The vocabulary-fault boundary
// =================================================================

/// A bad vocabulary spelling is rmcp's own codeless argument fault,
/// not §6.1's `invalid` (the D3/J5 boundary): `invalid` covers
/// values this server parses — the `ValueError` class — while
/// `status` and `kind` are closed sets the input schema itself
/// enumerates, so a wrong enum value is an argument-shape fault
/// exactly like a wrong type. Pinned so a future leniency (a
/// `serde(other)` catch-all on the vocabulary enums) cannot move the
/// signed boundary while every test stays green.
#[test]
fn a_bad_vocabulary_spelling_is_an_argument_fault_not_invalid() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "task_list", r#"{"repo":"fixture","status":"nope"}"#),
            call(3, "task_list", r#"{"repo":"fixture","kind":"chore"}"#),
        ],
    );
    for response in &driven.responses {
        let value: Value = serde_json::from_str(response).expect("the response line is JSON");
        assert_eq!(value["result"]["isError"], json!(true), "{response}");
        assert!(
            value["result"].get("structuredContent").is_none(),
            "an argument fault carries no code: {response}"
        );
        let text = value["result"]["content"][0]["text"]
            .as_str()
            .expect("the fault text");
        assert!(
            text.contains("failed to deserialize parameters"),
            "{response}"
        );
    }
    assert!(
        driven.responses[0].contains("unknown variant `nope`"),
        "the status fault names the variant: {}",
        driven.responses[0]
    );
    assert!(
        driven.responses[1].contains("unknown variant `chore`"),
        "the kind fault names the variant: {}",
        driven.responses[1]
    );
}

// =================================================================
// Determinism
// =================================================================

/// The same request sent twice answers byte-identical responses
/// (§8) — twice within one session (the second call reads the cached
/// handle the first opened) and across two fresh spawns, for the
/// listing and the dump alike.
#[test]
fn the_same_request_twice_answers_byte_identical_responses() {
    let root = fixture();
    let task = call(2, "task_list", r#"{"repo":"fixture"}"#);
    let dump = call(3, "graph_show", r#"{"repo":"fixture"}"#);
    let driven = drive(
        root.path(),
        &[task.clone(), dump.clone(), task.clone(), dump.clone()],
    );
    assert_eq!(driven.responses[0], driven.responses[2]);
    assert_eq!(driven.responses[1], driven.responses[3]);
    let again = drive(root.path(), &[task, dump]);
    assert_eq!(driven.responses[0], again.responses[0]);
    assert_eq!(driven.responses[1], again.responses[1]);
}

// =================================================================
// The assertion helpers
// =================================================================

/// Asserts one wire row list against the domain's answer, row for
/// row, each blocker's shape, id and status included, and each row's
/// priority (D17).
fn assert_task_rows(wire: &Value, answer: &[TaskRow]) {
    let rows = wire.as_array().expect("the rows are an array");
    assert_eq!(rows.len(), answer.len());
    for (row, expected) in rows.iter().zip(answer) {
        assert_eq!(row["task"], json!(expected.task.to_string()));
        assert_eq!(row["blocked_by"], json!(blocker_dtos(&expected.blocked_by)));
        assert_eq!(
            row["priority"],
            json!(expected.priority.map(|p| p.as_str()))
        );
    }
}

/// The expected wire spelling of one row's blockers — shape, id and
/// current status, in the domain's two-run order.
fn blocker_dtos(blockers: &[Blocker]) -> Vec<Value> {
    blockers
        .iter()
        .map(|blocker| match blocker {
            Blocker::Task { id, status } => json!({
                "shape": "task",
                "id": id.to_string(),
                "status": status.as_str(),
            }),
            Blocker::Decision { id, status } => json!({
                "shape": "decision",
                "id": id.to_string(),
                "status": status.as_str(),
            }),
        })
        .collect()
}

/// The task ids of one row list, in order.
fn row_task_ids(rows: &Value) -> Vec<&str> {
    rows.as_array()
        .expect("the rows are an array")
        .iter()
        .map(|row| row["task"].as_str().expect("a task id"))
        .collect()
}

/// Asserts one wire repo dump against the domain: the repo node, the
/// epic groups, the decisions with their blocks, and the docs.
fn assert_dump(dump: &Value, graph: &Graph) {
    let repo = graph.repo();
    assert_eq!(dump["repo"]["slug"], json!(repo.slug.to_string()));
    assert_eq!(dump["repo"]["name"], json!(repo.name.as_str()));
    assert_eq!(dump["repo"]["path"], json!(repo.path.as_str()));
    assert_eq!(
        dump["repo"]["registered_at"],
        json!(repo.registered_at.as_str())
    );
    let groups = dump["epics"].as_array().expect("the epic groups");
    let epics = graph.epic_list();
    assert_eq!(groups.len(), epics.len());
    for (group, epic) in groups.iter().zip(&epics) {
        assert_group(group, graph, epic);
    }
    let decisions = dump["decisions"].as_array().expect("the decisions");
    let expected_decisions = graph.decision_list();
    assert_eq!(decisions.len(), expected_decisions.len());
    for (node, id) in decisions.iter().zip(&expected_decisions) {
        assert_eq!(node["id"], json!(id.to_string()));
        let blocks: Vec<String> = graph
            .blocks(&BlockEnd::Decision(id.clone()))
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(node["blocks"], json!(blocks));
    }
    let docs = dump["docs"].as_array().expect("the docs");
    let expected_docs = graph.doc_list();
    assert_eq!(docs.len(), expected_docs.len());
    for (node, id) in docs.iter().zip(&expected_docs) {
        assert_eq!(node["id"], json!(id.to_string()));
    }
}

/// Asserts one wire epic group against the domain: the epic id, the
/// epic's own gating decisions, and the group's tasks in the epic's
/// induced canonical order with their adjacency.
fn assert_group(group: &Value, graph: &Graph, epic: &EpicId) {
    assert_eq!(group["id"], json!(epic.to_string()));
    let epic_blocked_by: Vec<String> = graph
        .blocked_by(&BlockEnd::Epic(epic.clone()))
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(group["blocked_by"], json!(epic_blocked_by));
    let order = graph.canonical_order_of_epic(epic).unwrap();
    let tasks = group["tasks"].as_array().expect("the group's tasks");
    assert_eq!(tasks.len(), order.len());
    for (task, id) in tasks.iter().zip(&order) {
        assert_task_node(task, graph, id);
    }
}

/// Asserts one wire task node's adjacency against the domain:
/// `blocked_by` (the two-run order, statuses dropped), `blocks` and
/// `relates`.
fn assert_task_node(task: &Value, graph: &Graph, id: &TaskId) {
    assert_eq!(task["id"], json!(id.to_string()));
    let blocked_by: Vec<String> = graph
        .blockers_of(id)
        .into_iter()
        .map(|blocker| BlockEnd::from(blocker).to_string())
        .collect();
    assert_eq!(task["blocked_by"], json!(blocked_by));
    let blocks: Vec<String> = graph
        .blocks(&BlockEnd::Task(id.clone()))
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(task["blocks"], json!(blocks));
    let relates: Vec<String> = graph
        .relates_of(id)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(task["relates"], json!(relates));
}
