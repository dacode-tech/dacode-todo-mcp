//! The query cluster's suite (t34): the five read tools through the
//! protocol — request bytes in, response bytes out, never by calling
//! the handler around it (§10) — with one golden per tool holding the
//! tool's full JSON-RPC response lines in the request order each
//! builder below documents. The goldens pin the bytes; the domain
//! agreement test pins the semantics against the domain's own answers
//! computed in-process over the same fixture, so a wire shape that is
//! deterministic but wrong cannot pass; the determinism test sends the
//! same request twice and compares bytes (§8).

use std::str::FromStr;

use serde_json::{Value, json};
use todo_domain::{RepoSlug, StatusCounts};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, assert_golden, call, drive, fixture, structured};

// =================================================================
// The request builders — each documents the request order its tool's
// golden pins: the success responses first, then the error responses
// (an unknown repo, an unknown epic/task/decision, a malformed id or
// slug, and — for `next_task`, the representative — an unknown
// argument field, rmcp's own codeless argument fault).
// =================================================================

/// `next_task`'s request order: the repo scope, the two epic scopes,
/// an unknown repo, an unknown epic, a malformed epic id, a malformed
/// slug, an unknown argument field.
fn next_task_requests() -> Vec<String> {
    vec![
        call(2, "next_task", r#"{"repo":"fixture"}"#),
        call(3, "next_task", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
        call(4, "next_task", r#"{"repo":"fixture","epic":"fixture:E2"}"#),
        call(5, "next_task", r#"{"repo":"nosuch"}"#),
        call(6, "next_task", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
        call(7, "next_task", r#"{"repo":"fixture","epic":"fixture:U1"}"#),
        call(8, "next_task", r#"{"repo":"Bad!"}"#),
        call(9, "next_task", r#"{"repo":"fixture","epik":"fixture:E1"}"#),
    ]
}

/// `next_items`'s request order: the repo scope at the default limit
/// and an explicit one, the epic scope both ways, the task scope both
/// ways, the contradictory scopes, an unknown repo, an unknown epic,
/// an unknown task, a malformed task id, a malformed slug.
fn next_items_requests() -> Vec<String> {
    vec![
        call(2, "next_items", r#"{"repo":"fixture"}"#),
        call(3, "next_items", r#"{"repo":"fixture","limit":2}"#),
        call(4, "next_items", r#"{"repo":"fixture","epic":"fixture:E2"}"#),
        call(
            5,
            "next_items",
            r#"{"repo":"fixture","epic":"fixture:E2","limit":3}"#,
        ),
        call(6, "next_items", r#"{"repo":"fixture","task":"fixture:U1"}"#),
        call(
            7,
            "next_items",
            r#"{"repo":"fixture","task":"fixture:U1","limit":1}"#,
        ),
        call(
            8,
            "next_items",
            r#"{"repo":"fixture","epic":"fixture:E1","task":"fixture:U1"}"#,
        ),
        call(9, "next_items", r#"{"repo":"nosuch"}"#),
        call(
            10,
            "next_items",
            r#"{"repo":"fixture","epic":"fixture:E9"}"#,
        ),
        call(
            11,
            "next_items",
            r#"{"repo":"fixture","task":"fixture:U99"}"#,
        ),
        call(
            12,
            "next_items",
            r#"{"repo":"fixture","task":"fixture:E1"}"#,
        ),
        call(13, "next_items", r#"{"repo":"Bad!"}"#),
    ]
}

/// `status_summary`'s request order: the repo scope, the two epic
/// scopes, an unknown repo, an unknown epic, a malformed epic id, a
/// malformed slug.
fn status_summary_requests() -> Vec<String> {
    vec![
        call(2, "status_summary", r#"{"repo":"fixture"}"#),
        call(
            3,
            "status_summary",
            r#"{"repo":"fixture","epic":"fixture:E1"}"#,
        ),
        call(
            4,
            "status_summary",
            r#"{"repo":"fixture","epic":"fixture:E2"}"#,
        ),
        call(5, "status_summary", r#"{"repo":"nosuch"}"#),
        call(
            6,
            "status_summary",
            r#"{"repo":"fixture","epic":"fixture:E9"}"#,
        ),
        call(
            7,
            "status_summary",
            r#"{"repo":"fixture","epic":"fixture:U1"}"#,
        ),
        call(8, "status_summary", r#"{"repo":"Bad!"}"#),
    ]
}

/// `task_show`'s request order: the rich task (references, priority,
/// until), the gate-kind task, an unknown repo, an unknown task, a
/// malformed task id, a malformed slug.
fn task_show_requests() -> Vec<String> {
    vec![
        call(2, "task_show", r#"{"repo":"fixture","task":"fixture:U1"}"#),
        call(3, "task_show", r#"{"repo":"fixture","task":"fixture:U2"}"#),
        call(4, "task_show", r#"{"repo":"nosuch","task":"fixture:U1"}"#),
        call(5, "task_show", r#"{"repo":"fixture","task":"fixture:U99"}"#),
        call(6, "task_show", r#"{"repo":"fixture","task":"fixture:D1"}"#),
        call(7, "task_show", r#"{"repo":"Bad!","task":"fixture:U1"}"#),
    ]
}

/// `decision_show`'s request order: the signed decision (resolution,
/// references), the open one, an unknown repo, an unknown decision, a
/// malformed decision id, a malformed slug.
fn decision_show_requests() -> Vec<String> {
    vec![
        call(
            2,
            "decision_show",
            r#"{"repo":"fixture","decision":"fixture:D3"}"#,
        ),
        call(
            3,
            "decision_show",
            r#"{"repo":"fixture","decision":"fixture:D1"}"#,
        ),
        call(
            4,
            "decision_show",
            r#"{"repo":"nosuch","decision":"fixture:D1"}"#,
        ),
        call(
            5,
            "decision_show",
            r#"{"repo":"fixture","decision":"fixture:D9"}"#,
        ),
        call(
            6,
            "decision_show",
            r#"{"repo":"fixture","decision":"fixture:U1"}"#,
        ),
        call(
            7,
            "decision_show",
            r#"{"repo":"Bad!","decision":"fixture:D1"}"#,
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

/// `next_task`'s request→response bytes, pinned: the repo scope,
/// the two epic scopes, and the error shapes.
#[test]
fn next_task_matches_its_golden() {
    assert_tool_golden(&next_task_requests(), "next_task.v1");
}

/// `next_items`'s request→response bytes, pinned: the three
/// scopes at the default and explicit limits, the contradictory
/// scopes, and the error shapes.
#[test]
fn next_items_matches_its_golden() {
    assert_tool_golden(&next_items_requests(), "next_items.v1");
}

/// `status_summary`'s request→response bytes, pinned: the repo
/// scope, the two epic scopes, and the error shapes.
#[test]
fn status_summary_matches_its_golden() {
    assert_tool_golden(&status_summary_requests(), "status_summary.v1");
}

/// `task_show`'s request→response bytes, pinned: the rich task,
/// the gate-kind task, and the error shapes.
#[test]
fn task_show_matches_its_golden() {
    assert_tool_golden(&task_show_requests(), "task_show.v1");
}

/// `decision_show`'s request→response bytes, pinned: the signed
/// decision, the open one, and the error shapes.
#[test]
fn decision_show_matches_its_golden() {
    assert_tool_golden(&decision_show_requests(), "decision_show.v1");
}

// =================================================================
// tools/list — the advertised schemas
// =================================================================

/// `tools/list` advertises every landed tool with a declared
/// outputSchema (D15: every tool declares one explicitly) and no
/// description anywhere — the tool's, a property's, a schema's, a
/// variant's — carries a line break: the doc comments become the
/// schema descriptions verbatim, so a multi-line doc would put its
/// line breaks into the wire bytes. The goldens hold `tools/call`
/// responses, so this is where the schema discipline is pinned.
#[test]
fn tools_list_advertises_output_schemas_with_single_line_descriptions() {
    let root = fixture();
    let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#.to_owned();
    let driven = drive(root.path(), &[request]);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    let value: Value = serde_json::from_str(&driven.responses[0]).expect("tools/list answers JSON");
    let tools = value["result"]["tools"].as_array().expect("the tool list");
    let names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("a tool name"))
        .collect();
    assert_eq!(
        names,
        [
            "decision_add",
            "decision_list",
            "decision_show",
            "decision_update",
            "doc_list",
            "doc_register",
            "doc_set_path",
            "doc_update",
            "epic_add",
            "epic_list",
            "epic_update",
            "graph_show",
            "next_items",
            "next_task",
            "repo_list",
            "repo_register",
            "repo_set_path",
            "repo_show",
            "status_summary",
            "task_add",
            "task_link",
            "task_list",
            "task_show",
            "task_unlink",
            "task_update"
        ],
        "all twenty-five landed tools, sorted by name"
    );
    for tool in tools {
        assert!(
            tool.get("outputSchema")
                .is_some_and(|schema| !schema.is_null()),
            "{} advertises an outputSchema",
            tool["name"]
        );
    }
    assert_no_newline_in_descriptions(&value, "$");
}

// =================================================================
// The unknown-id codes via the carrier
// =================================================================

/// Unknown repos, epics, tasks and decisions carry `unknown_id` via
/// D15's carrier: `isError: true`, `structuredContent.code`, the same
/// JSON mirrored into the text block.
#[test]
fn unknown_ids_carry_their_codes_via_the_carrier() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "next_task", r#"{"repo":"nosuch"}"#),
            call(3, "next_items", r#"{"repo":"nosuch"}"#),
            call(4, "status_summary", r#"{"repo":"nosuch"}"#),
            call(5, "task_show", r#"{"repo":"nosuch","task":"fixture:U1"}"#),
            call(
                6,
                "decision_show",
                r#"{"repo":"nosuch","decision":"fixture:D1"}"#,
            ),
            call(7, "next_task", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
            call(8, "next_items", r#"{"repo":"fixture","epic":"fixture:E9"}"#),
            call(
                9,
                "status_summary",
                r#"{"repo":"fixture","epic":"fixture:E9"}"#,
            ),
            call(
                10,
                "task_show",
                r#"{"repo":"fixture","task":"fixture:U99"}"#,
            ),
            call(
                11,
                "decision_show",
                r#"{"repo":"fixture","decision":"fixture:D9"}"#,
            ),
        ],
    );
    let expected = [
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: nosuch",
        "unknown id: fixture:E9",
        "unknown id: fixture:E9",
        "unknown id: fixture:E9",
        "unknown id: fixture:U99",
        "unknown id: fixture:D9",
    ];
    for (response, message) in driven.responses.iter().zip(expected) {
        assert_carrier(response, "unknown_id", message);
    }
}

// =================================================================
// Malformed ids and slugs
// =================================================================

/// Malformed slugs and ids carry `invalid` (U9's mapping of the
/// domain's `ValueError`) — and a malformed id outranks an unknown
/// repo: the parse happens before the store is touched, so the last
/// call answers `invalid`, not `unknown_id`.
#[test]
fn malformed_ids_and_slugs_carry_invalid_before_the_store_is_touched() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "next_task", r#"{"repo":"Bad!"}"#),
            call(3, "next_task", r#"{"repo":"fixture","epic":"fixture:U1"}"#),
            call(4, "next_items", r#"{"repo":"fixture","task":"fixture:E1"}"#),
            call(5, "task_show", r#"{"repo":"fixture","task":"fixture:D1"}"#),
            call(
                6,
                "decision_show",
                r#"{"repo":"fixture","decision":"fixture:U1"}"#,
            ),
            call(7, "next_task", r#"{"repo":"nosuch","epic":"bad"}"#),
        ],
    );
    let expected = [
        "invalid repository slug: \"Bad!\"",
        "invalid node id: \"fixture:U1\"",
        "invalid node id: \"fixture:E1\"",
        "invalid node id: \"fixture:D1\"",
        "invalid node id: \"fixture:U1\"",
        "invalid node id: \"bad\"",
    ];
    for (response, message) in driven.responses.iter().zip(expected) {
        assert_carrier(response, "invalid", message);
    }
}

// =================================================================
// The unknown argument field
// =================================================================

/// An unknown argument field is rmcp's own argument fault — tool-level
/// `isError`, unstructured text, no code — because
/// `deny_unknown_fields` turned the silent drop into a rejection
/// before the tool body ran. One representative tool; the mechanism is
/// identical per params struct.
#[test]
fn an_unknown_argument_field_is_rejected_before_the_tool_body() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[call(
            2,
            "next_task",
            r#"{"repo":"fixture","epik":"fixture:E1"}"#,
        )],
    );
    let response = &driven.responses[0];
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
    assert!(text.contains("unknown field `epik`"), "{response}");
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
            call(2, "next_task", r#"{"repo":"fixture"}"#),
            call(3, "next_task", r#"{"repo":"fixture","epic":"fixture:E1"}"#),
            call(4, "next_items", r#"{"repo":"fixture"}"#),
            call(5, "next_items", r#"{"repo":"fixture","task":"fixture:U1"}"#),
            call(6, "status_summary", r#"{"repo":"fixture"}"#),
            call(
                7,
                "status_summary",
                r#"{"repo":"fixture","epic":"fixture:E1"}"#,
            ),
            call(
                8,
                "status_summary",
                r#"{"repo":"fixture","epic":"fixture:E2"}"#,
            ),
        ],
    );
    // The drive's server has exited, so its locks are released and the
    // test process may open the store itself.
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").unwrap())
        .unwrap();
    let graph = store.graph();

    // next_task: the repo scope and the gated epic's.
    let answer = graph.next_task();
    let wire = structured(&driven.responses[0]);
    assert_eq!(wire["task"], json!(answer.task.map(|id| id.to_string())));
    assert_eq!(wire["deferred"]["total"], json!(answer.deferred.total));
    assert_eq!(wire["deferred"]["ready"], json!(answer.deferred.ready));

    let answer = graph
        .next_task_of_epic(&"fixture:E1".parse().unwrap())
        .unwrap();
    let wire = structured(&driven.responses[1]);
    assert_eq!(wire["task"], json!(answer.task.map(|id| id.to_string())));
    assert_eq!(wire["deferred"]["total"], json!(answer.deferred.total));
    assert_eq!(wire["deferred"]["ready"], json!(answer.deferred.ready));

    // next_items: the repo batch and the task-scoped batch.
    let answer = graph.next_items(10);
    let wire = structured(&driven.responses[2]);
    assert_eq!(wire["deferred"]["total"], json!(answer.deferred.total));
    assert_eq!(wire["deferred"]["ready"], json!(answer.deferred.ready));
    assert_next_items(&wire, &answer);

    let answer = graph
        .next_items_of_task(&"fixture:U1".parse().unwrap(), 10)
        .unwrap();
    let wire = structured(&driven.responses[3]);
    assert_eq!(wire["deferred"]["total"], json!(answer.deferred.total));
    assert_eq!(wire["deferred"]["ready"], json!(answer.deferred.ready));
    assert_next_items(&wire, &answer);

    // status_summary: the repo scope and both epic scopes.
    for (response, answer) in [
        (&driven.responses[4], graph.status_summary()),
        (
            &driven.responses[5],
            graph
                .status_summary_of_epic(&"fixture:E1".parse().unwrap())
                .unwrap(),
        ),
        (
            &driven.responses[6],
            graph
                .status_summary_of_epic(&"fixture:E2".parse().unwrap())
                .unwrap(),
        ),
    ] {
        let wire = structured(response);
        assert_status_counts(&wire["epics"], &answer.epics);
        assert_status_counts(&wire["tasks"], &answer.tasks);
        assert_eq!(wire["decisions"]["open"], json!(answer.decisions.open));
        assert_eq!(wire["decisions"]["signed"], json!(answer.decisions.signed));
        assert_eq!(
            wire["decisions"]["superseded"],
            json!(answer.decisions.superseded)
        );
        assert_eq!(wire["docs"], json!(answer.docs));
        assert_eq!(wire["open"]["epics"], json!(answer.open.epics));
        assert_eq!(wire["open"]["tasks"], json!(answer.open.tasks));
        assert_eq!(wire["open"]["decisions"], json!(answer.open.decisions));
        assert_eq!(wire["deferred"]["total"], json!(answer.deferred.total));
        assert_eq!(wire["deferred"]["ready"], json!(answer.deferred.ready));
    }
}

// =================================================================
// The limit
// =================================================================

/// `limit`: absent means 10 — the fixture holds eleven actionable
/// tasks in the repo scope, so the cap is visible as a row count —
/// explicit limits are honoured, and the epic and task scopes are both
/// exercised.
#[test]
fn the_limit_default_is_ten_and_explicit_limits_are_honoured() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(2, "next_items", r#"{"repo":"fixture"}"#),
            call(3, "next_items", r#"{"repo":"fixture","limit":2}"#),
            call(4, "next_items", r#"{"repo":"fixture","epic":"fixture:E2"}"#),
            call(
                5,
                "next_items",
                r#"{"repo":"fixture","epic":"fixture:E2","limit":3}"#,
            ),
            call(6, "next_items", r#"{"repo":"fixture","task":"fixture:U1"}"#),
            call(
                7,
                "next_items",
                r#"{"repo":"fixture","task":"fixture:U1","limit":1}"#,
            ),
        ],
    );
    let rows = |index: usize| {
        structured(&driven.responses[index])["items"]
            .as_array()
            .expect("the batch is an array")
            .len()
    };
    assert_eq!(rows(0), 10, "the absent limit caps the batch at ten");
    assert_eq!(rows(1), 2, "an explicit limit is honoured");
    assert_eq!(rows(2), 10, "the epic scope's default caps at ten");
    assert_eq!(rows(3), 3, "the epic scope honours an explicit limit");
    assert_eq!(rows(4), 2, "the task scope lists every dependent");
    assert_eq!(rows(5), 1, "the task scope honours an explicit limit");
    // The cap is a cap: the repo scope holds eleven actionable tasks,
    // so ten rows means the limit bit, not the scope ran out.
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").unwrap())
        .unwrap();
    assert_eq!(
        store.graph().next_items(u64::MAX).items.len(),
        11,
        "the fixture holds more actionable tasks than the default limit"
    );
}

// =================================================================
// Determinism
// =================================================================

/// The same request sent twice answers byte-identical responses
/// (§8) — twice within one session (the second call reads the cached
/// handle the first opened) and across two fresh spawns.
#[test]
fn the_same_request_twice_answers_byte_identical_responses() {
    let root = fixture();
    let request = call(2, "next_task", r#"{"repo":"fixture"}"#);
    let driven = drive(root.path(), &[request.clone(), request.clone()]);
    assert_eq!(driven.responses[0], driven.responses[1]);
    let again = drive(root.path(), &[request]);
    assert_eq!(driven.responses[0], again.responses[0]);
}

// =================================================================
// The two-run blocked_by order
// =================================================================

/// The two-run `blocked_by` order reaches the wire: U2's row carries
/// its own edges (U1, D2) and then the epic gate (D1) — D2 before D1
/// even though D1 < D2, which a globally sorted list would not carry —
/// and U3's row, gated by the same decision both directly and through
/// its epic, lists D1 exactly once, in the own-edges run. Pinned in the
/// golden and checked here against the domain's own `NextItem`.
#[test]
fn the_two_run_blocked_by_order_reaches_the_wire() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[call(
            6,
            "next_items",
            r#"{"repo":"fixture","task":"fixture:U1"}"#,
        )],
    );
    let wire = structured(&driven.responses[0]);
    let rows = wire["items"].as_array().expect("the batch is an array");
    assert_eq!(rows.len(), 2, "U1's dependents are U2 and U3");
    assert_eq!(rows[0]["task"], json!("fixture:U2"));
    assert_eq!(
        rows[0]["blocked_by"],
        json!(["fixture:U1", "fixture:D2", "fixture:D1"]),
        "own edges first, then the epic gate — not one sorted list"
    );
    assert_eq!(rows[1]["task"], json!("fixture:U3"));
    assert_eq!(
        rows[1]["blocked_by"],
        json!(["fixture:U1", "fixture:D1"]),
        "the decision gating both the task and its epic appears once"
    );
    // And against the domain's own answer, element-wise in order.
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").unwrap())
        .unwrap();
    let answer = store
        .graph()
        .next_items_of_task(&"fixture:U1".parse().unwrap(), 10)
        .unwrap();
    for (row, expected) in rows.iter().zip(&answer.items) {
        let blocked_by: Vec<String> = expected
            .blocked_by
            .iter()
            .map(|end| end.to_string())
            .collect();
        assert_eq!(row["blocked_by"], json!(blocked_by));
    }
}

// =================================================================
// The assertion helpers
// =================================================================

/// Walks a parsed response and asserts no `description` string
/// anywhere carries a line break — the single-line rule for every doc
/// comment that becomes a schema description. A `description` key
/// whose value is an object is a *property* of that name (an output
/// DTO field, e.g. the epic's prose field), not a doc string; the
/// recursion still checks the schema object's own inner descriptions.
fn assert_no_newline_in_descriptions(value: &Value, path: &str) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                if key == "description"
                    && let Some(text) = inner.as_str()
                {
                    assert!(
                        !text.contains('\n'),
                        "a description carries a line break at {path}: {text:?}"
                    );
                }
                assert_no_newline_in_descriptions(inner, &format!("{path}.{key}"));
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                assert_no_newline_in_descriptions(item, &format!("{path}[{index}]"));
            }
        }
        _ => {}
    }
}

/// Asserts one wire batch against the domain's answer, row for row.
fn assert_next_items(wire: &Value, answer: &todo_domain::NextItems) {
    let rows = wire["items"].as_array().expect("the batch is an array");
    assert_eq!(rows.len(), answer.items.len());
    for (row, expected) in rows.iter().zip(&answer.items) {
        assert_eq!(row["task"], json!(expected.task.to_string()));
        let blocked_by: Vec<String> = expected
            .blocked_by
            .iter()
            .map(|end| end.to_string())
            .collect();
        assert_eq!(row["blocked_by"], json!(blocked_by));
        let blocks: Vec<String> = expected.blocks.iter().map(|id| id.to_string()).collect();
        assert_eq!(row["blocks"], json!(blocks));
        assert_eq!(row["weight"], json!(expected.weight));
        assert_eq!(row["actionable"], json!(expected.actionable));
    }
}

/// Asserts one wire counts block against the domain's counts.
fn assert_status_counts(wire: &Value, counts: &StatusCounts) {
    for (name, count) in [
        ("open", counts.open),
        ("in_work", counts.in_work),
        ("blocked", counts.blocked),
        ("deferred", counts.deferred),
        ("closed", counts.closed),
        ("wontfix", counts.wontfix),
    ] {
        assert_eq!(wire[name], json!(count), "count {name}");
    }
}
