//! The node-CRUD mutation cluster's suite (t36): the nine write tools
//! through the protocol — request bytes in, response bytes out, never
//! by calling the handler around it (§10) — with the golden/in-test
//! split the cluster's clock discipline forces, stated here so t37's
//! writers (edges, `repo_set_path`) and any later tool know which
//! side of the line they fall on:
//!
//! **A golden file pins static bytes, so the property that decides is
//! clock-freeness of the response, not readness.** Epics carry no
//! timestamps (§3), so `epic_add`/`epic_update` responses are fully
//! deterministic — the `at` rides the event line alone, never the
//! response — and those two are golden-pinned (`epic_add.v1`,
//! `epic_update.v1`, the first writer goldens). Tasks, decisions and
//! docs carry `created`/`updated`, so their success responses embed
//! the server's one clock read and assert **in-test, byte-exactly
//! around the extracted stamp** — the t33 `registered_at_of` pattern
//! generalized: a suite-local extractor pulls the stamp out of the
//! response line and the assertion compares the whole line against a
//! template with the stamp interpolated (for updates, `created` is
//! the fixture's fixed value, so only `updated` is extracted). Error
//! responses are clock-free everywhere and assert through the carrier
//! helper. The alternative — a fixed-clock seam — was declined: it
//! would be a production surface for a test concern.
//!
//! Every test drives a fresh fixture (the writes mutate the store).
//! The stamped writer families — tasks, decisions, docs — end by
//! reopening the store in-process after the drive — the server has
//! exited, its locks released — asserting the mutation is on disk:
//! the wire proved the write, the reopen proves the log (the t33
//! `repo_register` precedent). The epic pair is golden-pinned instead
//! (clock-free responses), and the store's own replay suite pins the
//! epic add/update events' persistence.
//!
//! One accepted nit, recorded rather than fixed (the fixture is
//! shared and this task does not edit it): `common`'s missing-golden
//! hint names the read filter, so a missing epic golden's
//! regeneration hint is off by one command — the gates' ritual line
//! names both targets.

use serde_json::json;
use todo_domain::{Date, DecisionStatus, Priority, Status};
use todo_store::Store;

use crate::common::{assert_carrier, assert_golden, call, drive, fixture, structured};
use crate::reopened;

// =================================================================
// The suite-local helpers — the stamp extractors and the
// template-assertion arm
// =================================================================

/// The length of a canonical timestamp on the wire — the 30-byte RFC
/// 3339 UTC form with the nine-digit fraction.
const TIMESTAMP_LEN: usize = 30;

/// Extracts the clock-stamped `created` value from one response line —
/// the t33 `registered_at_of` pattern, generalized. The needle's plain
/// quotes cannot match the text mirror's escaped spelling, so it finds
/// the `structuredContent` occurrence.
fn created_of(response: &str) -> String {
    stamp_of(response, r#""created":""#)
}

/// Extracts the clock-stamped `updated` value from one response line —
/// [`created_of`]'s twin over the other field.
fn updated_of(response: &str) -> String {
    stamp_of(response, r#""updated":""#)
}

/// The extractor behind [`created_of`]/[`updated_of`]: the 30 bytes
/// after the needle.
fn stamp_of(response: &str, needle: &str) -> String {
    let start = response
        .find(needle)
        .unwrap_or_else(|| panic!("the response carries {needle}: {response}"))
        + needle.len();
    let end = start + TIMESTAMP_LEN;
    response[start..end].to_owned()
}

/// Asserts one success response byte-exactly: the full JSON-RPC line
/// rebuilt from `expected_structured` (the structuredContent JSON with
/// the stamp already interpolated) and its text mirror — the same JSON
/// with the quotes escaped, the only character these payloads carry
/// that serde_json escapes. The response line arrives with its
/// terminating newline; the template spells the line without it.
fn assert_success(response: &str, id: u64, expected_structured: &str) {
    let response = response.trim_end_matches('\n');
    let mirror = expected_structured.replace('"', "\\\"");
    assert_eq!(
        response,
        format!(
            r#"{{"jsonrpc":"2.0","id":{id},"result":{{"content":[{{"type":"text","text":"{mirror}"}}],"structuredContent":{expected_structured},"isError":false}}}}"#
        )
    );
}

/// The raw `structuredContent` JSON of one response line, unparsed —
/// for byte comparison, not `Value` comparison, because serde_json's
/// map equality is order-insensitive and D14's pin is an order pin.
fn structured_bytes(response: &str) -> &str {
    const START: &str = r#""structuredContent":"#;
    const END: &str = r#","isError":"#;
    let start = response
        .find(START)
        .unwrap_or_else(|| panic!("the response carries structuredContent: {response}"))
        + START.len();
    let end = start
        + response[start..]
            .find(END)
            .unwrap_or_else(|| panic!("the structuredContent ends before isError: {response}"));
    &response[start..end]
}

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

/// Asserts one response is rmcp's own codeless argument fault —
/// tool-level `isError`, unstructured text, no code — the read
/// suites' shape over this cluster's params structs.
fn assert_argument_fault(response: &str, field: &str) {
    let value: serde_json::Value =
        serde_json::from_str(response).expect("the response line is JSON");
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
    assert!(text.contains(field), "{response}");
}

// =================================================================
// The golden pair — the clock-free tools
// =================================================================

/// `epic_add`'s request order: the plain add (E3, open, no
/// references); the add with two references given in non-canonical
/// order (the response carries them canonically sorted — tracker
/// before spec, the `(kind, target, note)` order); an invalid
/// reference (a tracker-kind target carrying an `#anchor` — the
/// domain's constructor rejects it → `invalid`); an unknown repo; a
/// malformed slug; a bad vocabulary spelling (`status:"nope"` —
/// rmcp's codeless argument fault, the t35 boundary pinned on an
/// add); an unknown field.
fn epic_add_requests() -> Vec<String> {
    vec![
        call(
            2,
            "epic_add",
            r#"{"repo":"fixture","title":"Third epic","description":"The third epic's prose.","status":"open"}"#,
        ),
        call(
            3,
            "epic_add",
            r#"{"repo":"fixture","title":"Fourth epic","description":"","status":"open","references":[{"kind":"spec","target":"todo-mcp-design.md §6"},{"kind":"tracker","target":"FIX-9","note":"the issue"}]}"#,
        ),
        call(
            4,
            "epic_add",
            r#"{"repo":"fixture","title":"Fifth epic","description":"","status":"open","references":[{"kind":"tracker","target":"FIX-1#x"}]}"#,
        ),
        call(
            5,
            "epic_add",
            r#"{"repo":"nosuch","title":"x","description":"","status":"open"}"#,
        ),
        call(
            6,
            "epic_add",
            r#"{"repo":"Bad!","title":"x","description":"","status":"open"}"#,
        ),
        call(
            7,
            "epic_add",
            r#"{"repo":"fixture","title":"x","description":"","status":"nope"}"#,
        ),
        call(
            8,
            "epic_add",
            r#"{"repo":"fixture","titel":"x","description":"","status":"open"}"#,
        ),
    ]
}

/// `epic_add`'s request→response bytes, pinned: the two adds (the
/// reference sort visible), and the five error shapes.
#[test]
fn epic_add_matches_its_golden() {
    assert_tool_golden(&epic_add_requests(), "epic_add.v1");
}

/// `epic_update`'s request order: E1's title; E1's status and
/// description together; E1's references replaced (given
/// non-canonically, answered canonically sorted); the empty patch —
/// a legal no-op answering the unchanged node, the event landing
/// (pinned honestly); an unknown epic; an unknown repo; a malformed
/// epic id (a task id where an epic id is expected); an unknown
/// field.
fn epic_update_requests() -> Vec<String> {
    vec![
        call(
            2,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E1","title":"Bring-up anew"}"#,
        ),
        call(
            3,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E1","status":"in_work","description":"The amended description."}"#,
        ),
        call(
            4,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E1","references":[{"kind":"spec","target":"todo-mcp-design.md §6"},{"kind":"tracker","target":"FIX-9","note":"the issue"}]}"#,
        ),
        call(
            5,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E1"}"#,
        ),
        call(
            6,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E9","title":"x"}"#,
        ),
        call(
            7,
            "epic_update",
            r#"{"repo":"nosuch","epic":"fixture:E1","title":"x"}"#,
        ),
        call(
            8,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:U1","title":"x"}"#,
        ),
        call(
            9,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E1","titel":"x"}"#,
        ),
    ]
}

/// `epic_update`'s request→response bytes, pinned: the three patches
/// (each response carrying the accumulated state), the empty patch's
/// honest no-op, and the four error shapes.
#[test]
fn epic_update_matches_its_golden() {
    assert_tool_golden(&epic_update_requests(), "epic_update.v1");
}

// =================================================================
// task_add — the new state, the discipline trio, the persistence
// =================================================================

/// `task_add` answers the new state byte-exactly around the extracted
/// stamps (`created` and `updated` are the same instant at add), the
/// closure discipline trio carries its §6.1 codes through the carrier
/// (`missing_verify` without `verify`; `missing_evidence` at `closed`
/// without evidence; `missing_note` at `blocked` without note — the
/// domain's fixed check order means the failed adds' ids are the
/// peeked-but-uncommitted next number, U23 throughout), the id and
/// slug error paths answer before the store is touched, and the
/// reopen shows the node on disk with its allocated id and fields.
#[test]
fn task_add_answers_the_new_state_and_persists() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","title":"A new task","body":"The body.","status":"open","verify":"cargo test -p fixture","references":[{"kind":"tracker","target":"FIX-9"}],"priority":"high","until":"2026-11-01"}"#,
            ),
            call(
                3,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"open"}"#,
            ),
            call(
                4,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"closed","verify":"cargo test"}"#,
            ),
            call(
                5,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"blocked","verify":"cargo test"}"#,
            ),
            call(
                6,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E9","kind":"task","title":"x","body":"","status":"open","verify":"cargo test"}"#,
            ),
            call(
                7,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:U1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test"}"#,
            ),
            call(
                8,
                "task_add",
                r#"{"repo":"nosuch","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test"}"#,
            ),
            call(
                9,
                "task_add",
                r#"{"repo":"Bad!","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test"}"#,
            ),
            call(
                10,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","titel":"x","body":"","status":"open","verify":"cargo test"}"#,
            ),
        ],
    );
    // The new state, byte-exact around the extracted stamps — equal
    // at add, and the reference list canonically sorted.
    let stamp = created_of(&driven.responses[0]);
    assert_eq!(
        stamp,
        updated_of(&driven.responses[0]),
        "an add stamps created and updated with the same instant"
    );
    assert_success(
        &driven.responses[0],
        2,
        &format!(
            r#"{{"id":"fixture:U22","repo":"fixture","epic":"fixture:E1","kind":"task","title":"A new task","body":"The body.","status":"open","verify":"cargo test -p fixture","evidence":null,"note":null,"references":[{{"kind":"tracker","target":"FIX-9","note":null}}],"priority":"high","until":"2026-11-01","created":"{stamp}","updated":"{stamp}"}}"#
        ),
    );
    // The discipline trio, each through the carrier. The failed adds
    // peek U23 without committing it, so every error names U23 — the
    // never-reuse rule's peek side, visible in the payload.
    assert_carrier(
        &driven.responses[1],
        "missing_verify",
        "task fixture:U23 has no verify method",
    );
    assert_carrier(
        &driven.responses[2],
        "missing_evidence",
        "task fixture:U23 closed without evidence",
    );
    assert_carrier(
        &driven.responses[3],
        "missing_note",
        "task fixture:U23 blocked or deferred without a note",
    );
    // The id and slug error paths.
    assert_carrier(&driven.responses[4], "unknown_id", "unknown id: fixture:E9");
    assert_carrier(
        &driven.responses[5],
        "invalid",
        "invalid node id: \"fixture:U1\"",
    );
    assert_carrier(&driven.responses[6], "unknown_id", "unknown id: nosuch");
    assert_carrier(
        &driven.responses[7],
        "invalid",
        "invalid repository slug: \"Bad!\"",
    );
    assert_argument_fault(&driven.responses[8], "unknown field `titel`");
    // The reopen: the node is on disk with its allocated id, the
    // patched fields, and the stamps the wire answered with.
    let store = reopened(root.path());
    let task = store
        .graph()
        .task(&"fixture:U22".parse().unwrap())
        .expect("U22 is on disk");
    assert_eq!(task.title, "A new task");
    assert_eq!(task.body, "The body.");
    assert_eq!(task.status, Status::Open);
    assert_eq!(task.verify.as_deref(), Some("cargo test -p fixture"));
    assert_eq!(task.priority, Some(Priority::High));
    assert_eq!(task.until.as_ref().map(Date::as_str), Some("2026-11-01"));
    assert_eq!(task.created.as_str(), stamp.as_str());
    assert_eq!(task.updated.as_str(), stamp.as_str());
}

// =================================================================
// task_update — the three-state, the discipline pair, the reparenting
// =================================================================

/// U1's expected `structuredContent` after each mutation of the
/// three-state drive: the fixture's fixed fields with the patched ones
/// substituted, `created` the fixture's fixed value, `updated` the
/// extracted stamp. The four three-state fields are spelled out so
/// each call's assertion reads against the state it expects.
fn u1_expected(
    title: &str,
    verify: &str,
    note: Option<&str>,
    priority: Option<&str>,
    until: Option<&str>,
    epic: &str,
    updated: &str,
) -> String {
    let note = quoted(note);
    let priority = quoted(priority);
    let until = quoted(until);
    format!(
        r#"{{"id":"fixture:U1","repo":"fixture","epic":"{epic}","kind":"task","title":{title:?},"body":"","status":"open","verify":{verify:?},"evidence":null,"note":{note},"references":[{{"kind":"tracker","target":"FIX-1","note":"the issue"}},{{"kind":"doc","target":"fixture:S1#2","note":null}}],"priority":{priority},"until":{until},"created":"2026-10-01T08:10:01.000000000Z","updated":"{updated}"}}"#
    )
}

/// One optional string field's JSON spelling: quoted when present,
/// `null` when absent.
fn quoted(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("{value:?}"),
        None => "null".to_owned(),
    }
}

/// `task_update` patches the three-state fields — set, clear,
/// untouched — over one task (U1), with the middle call clearing.
/// `verify`'s clear is the one the domain refuses: the closure
/// discipline binds on the update path too, so `verify: null` answers
/// `missing_verify` through the carrier (pinned below as its own
/// assertion — the spec's "set/clear/untouched on verify" names a
/// clear the domain rejects, recorded in the task report). The
/// discipline pair (`missing_evidence` at `closed`, `missing_note` at
/// `deferred`), the reparenting (`epic` moves U1 to E2; an unknown
/// epic answers `unknown_id`), `updated` the extracted stamp with
/// `created` the fixture's fixed value, and the reopen showing the
/// final state on disk.
#[test]
fn task_update_patches_three_state_and_reparents() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            // Set: all four three-state fields take values.
            call(
                2,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","verify":"cargo test -p fixture --rerun","note":"held by the vendor","priority":"low","until":"2026-11-15"}"#,
            ),
            // Clear: the three clearable fields go null (verify's
            // clear is the domain's missing_verify, pinned after).
            call(
                3,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","note":null,"priority":null,"until":null}"#,
            ),
            // Untouched: a title-only patch leaves the four alone.
            call(
                4,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","title":"Wire the sensor anew"}"#,
            ),
            // verify's clear: the discipline on the update path.
            call(
                5,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","verify":null}"#,
            ),
            // The discipline pair.
            call(
                6,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","status":"closed"}"#,
            ),
            call(
                7,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","status":"deferred"}"#,
            ),
            // The reparenting, then its unknown-epic error.
            call(
                8,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","epic":"fixture:E2"}"#,
            ),
            call(
                9,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","epic":"fixture:E9"}"#,
            ),
        ],
    );
    // Set: the response carries all four values, created the
    // fixture's fixed instant, updated the extracted stamp.
    let set = updated_of(&driven.responses[0]);
    // The stamp is the server's clock, never the fixture's stale
    // `updated` — the real clock cannot return a past fixture
    // instant, so a mutation that left `updated` untouched fails here.
    assert_ne!(
        set, "2026-10-01T08:10:01.000000000Z",
        "updated must move off the fixture's fixed instant"
    );
    assert_success(
        &driven.responses[0],
        2,
        &u1_expected(
            "Wire the sensor",
            "cargo test -p fixture --rerun",
            Some("held by the vendor"),
            Some("low"),
            Some("2026-11-15"),
            "fixture:E1",
            &set,
        ),
    );
    // Clear: note, priority and until are null; verify keeps the set
    // value (its clear is refused below).
    let cleared = updated_of(&driven.responses[1]);
    assert_success(
        &driven.responses[1],
        3,
        &u1_expected(
            "Wire the sensor",
            "cargo test -p fixture --rerun",
            None,
            None,
            None,
            "fixture:E1",
            &cleared,
        ),
    );
    // Untouched: the title moved, the four kept the cleared state.
    let untouched = updated_of(&driven.responses[2]);
    assert_success(
        &driven.responses[2],
        4,
        &u1_expected(
            "Wire the sensor anew",
            "cargo test -p fixture --rerun",
            None,
            None,
            None,
            "fixture:E1",
            &untouched,
        ),
    );
    // verify's clear is the domain's answer: the discipline binds on
    // the update path, so the wire carries missing_verify.
    assert_carrier(
        &driven.responses[3],
        "missing_verify",
        "task fixture:U1 has no verify method",
    );
    // The discipline pair.
    assert_carrier(
        &driven.responses[4],
        "missing_evidence",
        "task fixture:U1 closed without evidence",
    );
    assert_carrier(
        &driven.responses[5],
        "missing_note",
        "task fixture:U1 blocked or deferred without a note",
    );
    // The reparenting: the response's epic field moved.
    let reparented = updated_of(&driven.responses[6]);
    assert_success(
        &driven.responses[6],
        8,
        &u1_expected(
            "Wire the sensor anew",
            "cargo test -p fixture --rerun",
            None,
            None,
            None,
            "fixture:E2",
            &reparented,
        ),
    );
    assert_carrier(&driven.responses[7], "unknown_id", "unknown id: fixture:E9");
    // The reopen: the final state is on disk.
    let store = reopened(root.path());
    let task = store
        .graph()
        .task(&"fixture:U1".parse().unwrap())
        .expect("U1 is on disk");
    assert_eq!(task.title, "Wire the sensor anew");
    assert_eq!(
        task.verify.as_deref(),
        Some("cargo test -p fixture --rerun")
    );
    assert_eq!(task.note, None);
    assert_eq!(task.priority, None);
    assert_eq!(task.until, None);
    assert_eq!(task.epic, "fixture:E2".parse().unwrap());
    assert_eq!(task.created.as_str(), "2026-10-01T08:10:01.000000000Z");
    assert_eq!(task.updated.as_str(), reparented.as_str());
}

// =================================================================
// The patch round trip
// =================================================================

/// `task_update` then `task_show` through the wire: the write's answer
/// is what a read returns — §6's "a client never needs a
/// read-after-write round trip", and this proves the round trip would
/// agree anyway. The comparison is over the raw structuredContent
/// bytes, not parsed values, because map equality is order-insensitive
/// and D14's pin is an order pin.
#[test]
fn the_patch_round_trip() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U2","title":"Calibrate the rig anew","status":"in_work"}"#,
            ),
            call(3, "task_show", r#"{"repo":"fixture","task":"fixture:U2"}"#),
        ],
    );
    assert_eq!(
        structured_bytes(&driven.responses[0]),
        structured_bytes(&driven.responses[1]),
        "the write's answer is what the read returns"
    );
    // And the patch landed: the read shows the new title and status.
    let wire = structured(&driven.responses[1]);
    assert_eq!(wire["title"], json!("Calibrate the rig anew"));
    assert_eq!(wire["status"], json!("in_work"));
}

// =================================================================
// decision_add / decision_update
// =================================================================

/// `decision_add` answers the new state byte-exactly around the
/// extracted stamps, `missing_resolution` fires at `signed` and at
/// `superseded` (the failed adds peek D5 without committing it), the
/// repo error paths answer before the store is touched, and the
/// reopen shows the dossier on disk.
#[test]
fn decision_add_answers_the_new_state_and_persists() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "decision_add",
                r#"{"repo":"fixture","title":"A new decision","status":"open","options":["(a) proceed","(b) hold"],"recommendation":"(a)","confidence":"medium","references":[{"kind":"spec","target":"todo-mcp-design.md §6"}]}"#,
            ),
            call(
                3,
                "decision_add",
                r#"{"repo":"fixture","title":"x","status":"signed","options":["(a)"],"confidence":"low"}"#,
            ),
            call(
                4,
                "decision_add",
                r#"{"repo":"fixture","title":"x","status":"superseded","options":["(a)"],"confidence":"low"}"#,
            ),
            call(
                5,
                "decision_add",
                r#"{"repo":"nosuch","title":"x","status":"open","options":["(a)"],"confidence":"low"}"#,
            ),
            call(
                6,
                "decision_add",
                r#"{"repo":"Bad!","title":"x","status":"open","options":["(a)"],"confidence":"low"}"#,
            ),
            call(
                7,
                "decision_add",
                r#"{"repo":"fixture","titel":"x","status":"open","options":["(a)"],"confidence":"low"}"#,
            ),
        ],
    );
    let stamp = created_of(&driven.responses[0]);
    assert_eq!(stamp, updated_of(&driven.responses[0]));
    assert_success(
        &driven.responses[0],
        2,
        &format!(
            r#"{{"id":"fixture:D4","repo":"fixture","title":"A new decision","status":"open","options":["(a) proceed","(b) hold"],"recommendation":"(a)","resolution":null,"confidence":"medium","references":[{{"kind":"spec","target":"todo-mcp-design.md §6","note":null}}],"created":"{stamp}","updated":"{stamp}"}}"#
        ),
    );
    assert_carrier(
        &driven.responses[1],
        "missing_resolution",
        "decision fixture:D5 signed or superseded without a resolution",
    );
    assert_carrier(
        &driven.responses[2],
        "missing_resolution",
        "decision fixture:D5 signed or superseded without a resolution",
    );
    assert_carrier(&driven.responses[3], "unknown_id", "unknown id: nosuch");
    assert_carrier(
        &driven.responses[4],
        "invalid",
        "invalid repository slug: \"Bad!\"",
    );
    assert_argument_fault(&driven.responses[5], "unknown field `titel`");
    let store = reopened(root.path());
    let decision = store
        .graph()
        .decision(&"fixture:D4".parse().unwrap())
        .expect("D4 is on disk");
    assert_eq!(decision.title, "A new decision");
    assert_eq!(decision.recommendation.as_deref(), Some("(a)"));
    assert_eq!(decision.resolution, None);
    assert_eq!(decision.created.as_str(), stamp.as_str());
    assert_eq!(decision.updated.as_str(), stamp.as_str());
}

/// `decision_update` signs D1 with a resolution — the response carries
/// both, the stamps assert (`created` the fixture's fixed value,
/// `updated` the extracted stamp), and the reopen shows the signed
/// node; the three-state runs on `recommendation` (set, clear,
/// untouched); signing without a resolution (D2, whose resolution is
/// null) answers `missing_resolution`; an unknown decision and a
/// malformed id answer their codes.
#[test]
fn decision_update_signs_with_resolution() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            // Sign D1.
            call(
                2,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:D1","status":"signed","resolution":"signed (a) 2026-10-04"}"#,
            ),
            // The three-state on recommendation: set, clear, untouched.
            call(
                3,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:D1","recommendation":"(b)"}"#,
            ),
            call(
                4,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:D1","recommendation":null}"#,
            ),
            call(
                5,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:D1","title":"Gate the bring-up epic anew"}"#,
            ),
            // Signing without a resolution: D2's resolution is null.
            call(
                6,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:D2","status":"signed"}"#,
            ),
            // The id error paths.
            call(
                7,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:D9","title":"x"}"#,
            ),
            call(
                8,
                "decision_update",
                r#"{"repo":"fixture","decision":"fixture:U1","title":"x"}"#,
            ),
        ],
    );
    // The signing response: status and resolution both carried,
    // created the fixture's fixed instant, updated the stamp.
    let signed = updated_of(&driven.responses[0]);
    // The stamp is the server's clock, never the fixture's stale
    // `updated` — a mutation that left `updated` untouched fails here.
    assert_ne!(
        signed, "2026-10-01T08:03:00.000000000Z",
        "updated must move off the fixture's fixed instant"
    );
    assert_success(
        &driven.responses[0],
        2,
        &format!(
            r#"{{"id":"fixture:D1","repo":"fixture","title":"Gate the bring-up epic","status":"signed","options":["(a) proceed","(b) hold"],"recommendation":"(a)","resolution":"signed (a) 2026-10-04","confidence":"medium","references":[],"created":"2026-10-01T08:03:00.000000000Z","updated":"{signed}"}}"#
        ),
    );
    // The three-state on recommendation: set, then clear, then
    // untouched (a title-only patch leaves the cleared null).
    let set = updated_of(&driven.responses[1]);
    assert_success(
        &driven.responses[1],
        3,
        &format!(
            r#"{{"id":"fixture:D1","repo":"fixture","title":"Gate the bring-up epic","status":"signed","options":["(a) proceed","(b) hold"],"recommendation":"(b)","resolution":"signed (a) 2026-10-04","confidence":"medium","references":[],"created":"2026-10-01T08:03:00.000000000Z","updated":"{set}"}}"#
        ),
    );
    let cleared = updated_of(&driven.responses[2]);
    assert_success(
        &driven.responses[2],
        4,
        &format!(
            r#"{{"id":"fixture:D1","repo":"fixture","title":"Gate the bring-up epic","status":"signed","options":["(a) proceed","(b) hold"],"recommendation":null,"resolution":"signed (a) 2026-10-04","confidence":"medium","references":[],"created":"2026-10-01T08:03:00.000000000Z","updated":"{cleared}"}}"#
        ),
    );
    let untouched = updated_of(&driven.responses[3]);
    assert_success(
        &driven.responses[3],
        5,
        &format!(
            r#"{{"id":"fixture:D1","repo":"fixture","title":"Gate the bring-up epic anew","status":"signed","options":["(a) proceed","(b) hold"],"recommendation":null,"resolution":"signed (a) 2026-10-04","confidence":"medium","references":[],"created":"2026-10-01T08:03:00.000000000Z","updated":"{untouched}"}}"#
        ),
    );
    // Signing without a resolution, the unknown decision, the
    // malformed id.
    assert_carrier(
        &driven.responses[4],
        "missing_resolution",
        "decision fixture:D2 signed or superseded without a resolution",
    );
    assert_carrier(&driven.responses[5], "unknown_id", "unknown id: fixture:D9");
    assert_carrier(
        &driven.responses[6],
        "invalid",
        "invalid node id: \"fixture:U1\"",
    );
    // The reopen: the signed node is on disk, recommendation cleared,
    // title replaced.
    let store = reopened(root.path());
    let decision = store
        .graph()
        .decision(&"fixture:D1".parse().unwrap())
        .expect("D1 is on disk");
    assert_eq!(decision.status, DecisionStatus::Signed);
    assert_eq!(
        decision.resolution.as_deref(),
        Some("signed (a) 2026-10-04")
    );
    assert_eq!(decision.recommendation, None);
    assert_eq!(decision.title, "Gate the bring-up epic anew");
    assert_eq!(decision.created.as_str(), "2026-10-01T08:03:00.000000000Z");
    assert_eq!(decision.updated.as_str(), untouched.as_str());
}

// =================================================================
// doc_register / doc_set_path / doc_update
// =================================================================

/// `doc_register` answers the new state byte-exactly around the
/// extracted stamps, the repo error paths answer before the store is
/// touched, and the reopen shows the doc on disk.
#[test]
fn doc_register_answers_the_new_state_and_persists() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "doc_register",
                r#"{"repo":"fixture","title":"Runway notes","path":"docs/runway.md","version":"1.0","references":[{"kind":"file","target":"docs/runway.md"}]}"#,
            ),
            call(
                3,
                "doc_register",
                r#"{"repo":"nosuch","title":"x","path":"x","version":"1"}"#,
            ),
            call(
                4,
                "doc_register",
                r#"{"repo":"Bad!","title":"x","path":"x","version":"1"}"#,
            ),
            call(
                5,
                "doc_register",
                r#"{"repo":"fixture","titel":"x","path":"x","version":"1"}"#,
            ),
        ],
    );
    let stamp = created_of(&driven.responses[0]);
    assert_eq!(stamp, updated_of(&driven.responses[0]));
    assert_success(
        &driven.responses[0],
        2,
        &format!(
            r#"{{"id":"fixture:S2","repo":"fixture","title":"Runway notes","path":"docs/runway.md","version":"1.0","references":[{{"kind":"file","target":"docs/runway.md","note":null}}],"created":"{stamp}","updated":"{stamp}"}}"#
        ),
    );
    assert_carrier(&driven.responses[1], "unknown_id", "unknown id: nosuch");
    assert_carrier(
        &driven.responses[2],
        "invalid",
        "invalid repository slug: \"Bad!\"",
    );
    assert_argument_fault(&driven.responses[3], "unknown field `titel`");
    let store = reopened(root.path());
    let doc = store
        .graph()
        .doc(&"fixture:S2".parse().unwrap())
        .expect("S2 is on disk");
    assert_eq!(doc.title, "Runway notes");
    assert_eq!(doc.path, "docs/runway.md");
    assert_eq!(doc.version, "1.0");
    assert_eq!(doc.created.as_str(), stamp.as_str());
}

/// `doc_set_path` moves the audited path: the response shows the moved
/// doc with `updated` the extracted stamp and `created` the fixture's
/// fixed value, the reopen shows the moved path on disk, and the id
/// error paths answer their codes.
#[test]
fn doc_set_path_moves_the_audited_path() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "doc_set_path",
                r#"{"repo":"fixture","doc":"fixture:S1","path":"docs/bring-up-v2.md"}"#,
            ),
            call(
                3,
                "doc_set_path",
                r#"{"repo":"fixture","doc":"fixture:S9","path":"x"}"#,
            ),
            call(
                4,
                "doc_set_path",
                r#"{"repo":"fixture","doc":"fixture:U1","path":"x"}"#,
            ),
            call(
                5,
                "doc_set_path",
                r#"{"repo":"fixture","doc":"fixture:S1","path":"x","titel":"y"}"#,
            ),
        ],
    );
    let stamp = updated_of(&driven.responses[0]);
    assert_success(
        &driven.responses[0],
        2,
        &format!(
            r#"{{"id":"fixture:S1","repo":"fixture","title":"Bring-up notes","path":"docs/bring-up-v2.md","version":"1.0","references":[],"created":"2026-10-01T08:06:00.000000000Z","updated":"{stamp}"}}"#
        ),
    );
    assert_carrier(&driven.responses[1], "unknown_id", "unknown id: fixture:S9");
    assert_carrier(
        &driven.responses[2],
        "invalid",
        "invalid node id: \"fixture:U1\"",
    );
    assert_argument_fault(&driven.responses[3], "unknown field `titel`");
    let store = reopened(root.path());
    let doc = store
        .graph()
        .doc(&"fixture:S1".parse().unwrap())
        .expect("S1 is on disk");
    assert_eq!(doc.path, "docs/bring-up-v2.md");
    assert_eq!(doc.created.as_str(), "2026-10-01T08:06:00.000000000Z");
    assert_eq!(doc.updated.as_str(), stamp.as_str());
}

/// `doc_update` patches title, version and references — each a
/// whole-field replacement under the patch convention, the path
/// untouched (the audited verb owns it) — with the stamps asserting
/// and the reopen showing the patched doc; the empty patch answers
/// the unchanged node; the error paths answer their codes.
#[test]
fn doc_update_patches_three_state() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "doc_update",
                r#"{"repo":"fixture","doc":"fixture:S1","title":"Bring-up notes v2","version":"1.1","references":[{"kind":"spec","target":"todo-mcp-design.md §6"}]}"#,
            ),
            call(3, "doc_update", r#"{"repo":"fixture","doc":"fixture:S1"}"#),
            call(
                4,
                "doc_update",
                r#"{"repo":"fixture","doc":"fixture:S9","title":"x"}"#,
            ),
            call(
                5,
                "doc_update",
                r#"{"repo":"fixture","doc":"fixture:U1","title":"x"}"#,
            ),
            call(
                6,
                "doc_update",
                r#"{"repo":"fixture","doc":"fixture:S1","titel":"x"}"#,
            ),
        ],
    );
    // The replacement: title, version and references moved, the path
    // untouched, created the fixture's fixed value, updated the stamp.
    let stamp = updated_of(&driven.responses[0]);
    assert_success(
        &driven.responses[0],
        2,
        &format!(
            r#"{{"id":"fixture:S1","repo":"fixture","title":"Bring-up notes v2","path":"docs/bring-up.md","version":"1.1","references":[{{"kind":"spec","target":"todo-mcp-design.md §6","note":null}}],"created":"2026-10-01T08:06:00.000000000Z","updated":"{stamp}"}}"#
        ),
    );
    // The empty patch: the unchanged node, a legal no-op whose event
    // lands (the epic_update golden pins the same honestly).
    let noop = updated_of(&driven.responses[1]);
    assert_success(
        &driven.responses[1],
        3,
        &format!(
            r#"{{"id":"fixture:S1","repo":"fixture","title":"Bring-up notes v2","path":"docs/bring-up.md","version":"1.1","references":[{{"kind":"spec","target":"todo-mcp-design.md §6","note":null}}],"created":"2026-10-01T08:06:00.000000000Z","updated":"{noop}"}}"#
        ),
    );
    assert_carrier(&driven.responses[2], "unknown_id", "unknown id: fixture:S9");
    assert_carrier(
        &driven.responses[3],
        "invalid",
        "invalid node id: \"fixture:U1\"",
    );
    assert_argument_fault(&driven.responses[4], "unknown field `titel`");
    let store = reopened(root.path());
    let doc = store
        .graph()
        .doc(&"fixture:S1".parse().unwrap())
        .expect("S1 is on disk");
    assert_eq!(doc.title, "Bring-up notes v2");
    assert_eq!(doc.path, "docs/bring-up.md");
    assert_eq!(doc.version, "1.1");
    assert_eq!(doc.references.len(), 1);
}

// =================================================================
// The parse-before-resolve order, across the cluster
// =================================================================

/// Malformed ids, slugs and dates carry `invalid` before the store is
/// touched — and a malformed id outranks an unknown repo: the parse
/// happens before the resolve, so every call below names an unknown
/// repo yet answers `invalid` (the t34/t35 convention, pinned across
/// the cluster's tools). The malformed `until` and the invalid
/// reference are the same rule over the values this server parses.
#[test]
fn malformed_ids_and_slugs_carry_invalid_before_the_store_is_touched() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "epic_update",
                r#"{"repo":"nosuch","epic":"bad","title":"x"}"#,
            ),
            call(
                3,
                "task_add",
                r#"{"repo":"nosuch","epic":"fixture:U1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test"}"#,
            ),
            call(
                4,
                "task_update",
                r#"{"repo":"nosuch","task":"fixture:E1","title":"x"}"#,
            ),
            call(
                5,
                "decision_update",
                r#"{"repo":"nosuch","decision":"fixture:U1","title":"x"}"#,
            ),
            call(
                6,
                "doc_set_path",
                r#"{"repo":"nosuch","doc":"fixture:U1","path":"x"}"#,
            ),
            call(
                7,
                "doc_update",
                r#"{"repo":"nosuch","doc":"fixture:U1","title":"x"}"#,
            ),
            call(
                8,
                "epic_add",
                r#"{"repo":"Bad!","title":"x","description":"","status":"open"}"#,
            ),
            call(
                9,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test","until":"2026-13-01"}"#,
            ),
            call(
                10,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U1","until":"not-a-date"}"#,
            ),
            call(
                11,
                "epic_add",
                r#"{"repo":"fixture","title":"x","description":"","status":"open","references":[{"kind":"tracker","target":"FIX-1#x"}]}"#,
            ),
            // A nested unknown field is rejected like a top-level
            // one: `deny_unknown_fields` on ReferenceParam turns the
            // silent drop into rmcp's codeless argument fault.
            call(
                12,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test","references":[{"kind":"tracker","target":"FIX-9","notes":"y"}]}"#,
            ),
        ],
    );
    let expected = [
        "invalid node id: \"bad\"",
        "invalid node id: \"fixture:U1\"",
        "invalid node id: \"fixture:E1\"",
        "invalid node id: \"fixture:U1\"",
        "invalid node id: \"fixture:U1\"",
        "invalid node id: \"fixture:U1\"",
        "invalid repository slug: \"Bad!\"",
        "invalid YYYY-MM-DD date: \"2026-13-01\"",
        "invalid YYYY-MM-DD date: \"not-a-date\"",
        "invalid reference target \"FIX-1#x\": anchors are only valid on doc-kind references",
    ];
    for (response, message) in driven.responses.iter().zip(expected) {
        assert_carrier(response, "invalid", message);
    }
    // A nested unknown field is rejected like a top-level one:
    // `deny_unknown_fields` on ReferenceParam turns the silent drop
    // into rmcp's codeless argument fault, the field named.
    assert_argument_fault(&driven.responses[10], "unknown field `notes`");
}

// =================================================================
// Determinism — the clock-free pair
// =================================================================

/// The clock-free tools answer byte-identically across two fresh
/// spawns (§8 — two runs over equal state return identical bytes):
/// `epic_add` and `epic_update` carry no timestamps, so nothing the
/// clock could leak reaches the response. Two fresh fixtures hold the
/// equal state — a writer's second run over the *same* root would see
/// the first's mutation, which is a different question; the stamped
/// writers cannot pin even this, which is exactly the split the
/// module doc states.
#[test]
fn the_clock_free_tools_answer_byte_identically_twice() {
    let requests = vec![
        call(
            2,
            "epic_add",
            r#"{"repo":"fixture","title":"Third epic","description":"","status":"open"}"#,
        ),
        call(
            3,
            "epic_update",
            r#"{"repo":"fixture","epic":"fixture:E1","title":"Bring-up anew"}"#,
        ),
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
