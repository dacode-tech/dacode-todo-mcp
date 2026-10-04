//! The wire suite's cross-cutting pins (t38): the proofs that live
//! above any single tool, over the surface t34–t37 landed — the
//! committed D15's error carrier, conformance-swept across every tool
//! that has a carrier-reachable error path; the full response set's
//! twice-run byte-identity (§8); the concurrency proof over one
//! connection (§8: rmcp dispatches each request in its own task, the
//! server's state mutex serializes the tool bodies, and out-of-order
//! arrivals are tolerated by matching responses to requests by
//! JSON-RPC id, never by position); and the U26 hammer over the
//! protocol suite's post-exit store-open flake. No new tools, no
//! schema changes, no goldens: the golden suite IS the byte pin,
//! and the committed D14's `preserve_order` pin is proven by
//! mutation — dropping the feature fails the goldens — not by a new
//! test duplicating what the byte comparison already provides.
//!
//! Two mechanisms are suite-local here, not in `common/` (the home
//! ruling this suite inherits): the rapid-fire drive and the stamp
//! normalization. `common/` is compiled by the read and write
//! targets, which would see wire-only helpers dead — the t37
//! `reopened` discovery, now precedent — and a single-file target's
//! root is the file itself. The handshake constants and the
//! spawn/send/receive primitives are duplicated for the same reason:
//! `common`'s are private, and widening them would edit a file this
//! task does not own.

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

// `common/` is shared by the read, write and wire targets; this
// target uses the fixture, drive and call subset, and the golden and
// carrier helpers are dead only from this target's compilation — the
// read and write targets keep them live. The allow is scoped to the
// module so this target's own helpers stay under the lint.
#[allow(dead_code)]
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{BlocksEdge, RepoSlug};
use todo_store::{DataRoot, Store};

use common::{call, drive, fixture};

// =================================================================
// The suite-local drive — the rapid-fire shape and the protocol
// suite's wait-for-exit shape
// =================================================================

/// The initialize request every drive opens with — protocol
/// 2025-11-25, the version this server declares. The clientInfo name
/// is not echoed, so it never reaches a comparison.
const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t38","version":"0"}}}"#;

/// The handshake-completion notification — no response.
const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

/// A `repo_register` call for `camera` — the flaky protocol test's own
/// request, verbatim: the U26 hammer repeats that test's exact
/// sequence.
const REGISTER_CAMERA: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"repo_register","arguments":{"slug":"camera","name":"Camera","path":"/Users/probe/camera"}}}"#;

/// What one rapid-fire drive collected.
struct Rapid {
    /// The tool responses, one line each, in arrival order — which is
    /// not necessarily the request order.
    responses: Vec<String>,
    /// The server's stderr — empty on every green path.
    stderr: String,
    /// The exit status — a completed handshake ends 0.
    status: ExitStatus,
}

/// Drives the real binary over raw stdio in the rapid-fire shape: the
/// handshake, then **all** request lines written before any response
/// is read, then exactly `requests.len()` response lines read. rmcp
/// dispatches each request in its own task (§6.3), so arrivals can
/// reorder under this shape — the interactive drive in `common/`
/// cannot exercise that (each response is read before the next
/// request is written, so the response order is the request order by
/// construction). A burst's response bytes stay well under the pipe
/// buffer, so writing everything before reading anything cannot
/// deadlock.
fn rapid_fire(root: &Path, requests: &[String]) -> Rapid {
    let mut child = spawn(root);
    let mut stdin = child.stdin.take().expect("the child's stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("the child's stdout"));
    let mut stderr = child.stderr.take().expect("the child's stderr");

    send(&mut stdin, INITIALIZE);
    let handshake = receive(&mut stdout);
    assert!(
        handshake
            .starts_with(r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25""#),
        "the handshake answered: {handshake}"
    );
    send(&mut stdin, INITIALIZED);

    for request in requests {
        send(&mut stdin, request);
    }
    let mut responses = Vec::with_capacity(requests.len());
    for _ in 0..requests.len() {
        responses.push(receive(&mut stdout));
    }

    drop(stdin);
    let mut trailing = String::new();
    stdout
        .read_to_string(&mut trailing)
        .expect("read the remaining stdout");
    assert!(
        trailing.is_empty(),
        "no response bytes exist beyond the driven ones: {trailing}"
    );
    let mut stderr_text = String::new();
    stderr
        .read_to_string(&mut stderr_text)
        .expect("read the server's stderr");
    let status = child.wait().expect("wait for the server");
    Rapid {
        responses,
        stderr: stderr_text,
        status,
    }
}

/// The protocol suite's drive shape (t33's), suite-local for the U26
/// hammer: write every request line, close stdin — a stdio server's
/// natural end — and wait for exit with the output collected. The
/// flake's window lives between this reap and the caller's next open,
/// so the hammer rides the exact shape the flaky test rode.
fn drive_to_exit(root: &Path, requests: &str) -> (String, String, ExitStatus) {
    let mut child = spawn(root);
    {
        let mut stdin = child.stdin.take().expect("the child's stdin");
        stdin
            .write_all(requests.as_bytes())
            .expect("write the requests");
    }
    let output = child.wait_with_output().expect("wait for the server");
    (
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        output.status,
    )
}

/// Spawns the real binary with `--root` — `common`'s spawn is private
/// and this task does not edit that file, so the suite carries its
/// own copy.
fn spawn(root: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
        .arg("--root")
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn todo-mcp")
}

/// Writes one request line.
fn send(stdin: &mut ChildStdin, line: &str) {
    stdin.write_all(line.as_bytes()).expect("write the request");
    stdin.write_all(b"\n").expect("terminate the request line");
}

/// Reads one response line, newline included.
fn receive(stdout: &mut BufReader<ChildStdout>) -> String {
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read a response line");
    assert!(
        !line.is_empty(),
        "the server closed its stdout before answering"
    );
    line
}

/// Matches rapid-fire responses to their requests by JSON-RPC id —
/// never by position, which is the out-of-order tolerance itself.
/// Every id must answer exactly once.
fn by_id(responses: &[String]) -> BTreeMap<u64, Value> {
    let mut matched = BTreeMap::new();
    for response in responses {
        let value: Value = serde_json::from_str(response).expect("the response line is JSON");
        let id = value["id"]
            .as_u64()
            .unwrap_or_else(|| panic!("the response carries a numeric id: {response}"));
        assert!(
            matched.insert(id, value).is_none(),
            "the id {id} answered exactly once: {response}"
        );
    }
    matched
}

// =================================================================
// The suite-local stamp normalization
// =================================================================

/// The length of a canonical timestamp on the wire — the 30-byte RFC
/// 3339 UTC form with the nine-digit fraction. The fixed width is
/// what makes a stamp value a bounded needle.
const TIMESTAMP_LEN: usize = 30;

/// The placeholder a normalized stamp becomes.
const STAMP: &str = "<stamp>";

/// The stamp-key needles, both spellings: the structuredContent's
/// plain quotes and the text mirror's escaped ones. `registered_at`
/// joins `created`/`updated` because `repo_register`'s happy response
/// carries the clock in it — the one field of a repo writer a
/// twice-run comparison cannot take byte-exactly (the protocol
/// suite's own extraction precedent).
const NEEDLES: [&str; 6] = [
    r#""created":""#,
    r#""updated":""#,
    r#""registered_at":""#,
    r#"\"created\":\""#,
    r#"\"updated\":\""#,
    r#"\"registered_at\":\""#,
];

/// Erases the clock-stamped values from one response line — every
/// 30-byte value following a stamp key, in both spellings — so two
/// runs of a stamped writer compare byte-exactly modulo the one
/// honest nondeterminism: the server's clock, stamped per call. §8's
/// property for a clock-carrying server is identity modulo it. This
/// is normalization, not extraction: the write suite's
/// `created_of`/`updated_of` pull a stamp out for template
/// assertions, this erases one for comparison, and the target
/// structure keeps the two apart (`common/` is compiled by the read
/// target, which would see a wire-only helper dead).
fn normalize_stamps(response: &str) -> String {
    let mut normalized = response.to_owned();
    for needle in NEEDLES {
        normalized = erase_after(&normalized, needle);
    }
    normalized
}

/// Replaces the 30 bytes after every occurrence of `needle` with the
/// placeholder.
fn erase_after(text: &str, needle: &str) -> String {
    let mut erased = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find(needle) {
        let value_start = position + needle.len();
        let value_end = value_start + TIMESTAMP_LEN;
        erased.push_str(&rest[..value_start]);
        erased.push_str(STAMP);
        rest = &rest[value_end..];
    }
    erased.push_str(rest);
    erased
}

// =================================================================
// The carrier shape assertion
// =================================================================

/// Asserts one response carries the committed D15's error carrier,
/// shape-level: `isError: true`, a non-empty `code` and `message` in
/// `structuredContent`, and the text block's JSON mirroring the
/// structuredContent exactly — parsed and compared, the t34
/// `assert_carrier` assertions applied uniformly. The expected code
/// is asserted because the sweep drives one uniform error per row;
/// the message is checked non-empty only — each tool's code and
/// message are pinned byte-exactly in that tool's own suite, and the
/// sweep's point is the shape's uniformity, not the codes.
fn assert_carrier_shape(response: &str, expected_code: &str) {
    let value: Value = serde_json::from_str(response).expect("the response line is JSON");
    assert_eq!(value["result"]["isError"], json!(true), "{response}");
    let code = value["result"]["structuredContent"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("the code is a string: {response}"));
    assert!(!code.is_empty(), "the code is non-empty: {response}");
    assert_eq!(code, expected_code, "{response}");
    let message = value["result"]["structuredContent"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("the message is a string: {response}"));
    assert!(!message.is_empty(), "the message is non-empty: {response}");
    let mirror = value["result"]["content"][0]["text"]
        .as_str()
        .expect("the text mirror");
    let mirrored: Value = serde_json::from_str(mirror).expect("the mirror is JSON");
    assert_eq!(
        mirrored, value["result"]["structuredContent"],
        "the text mirror is the structuredContent: {response}"
    );
}

// =================================================================
// The carrier conformance sweep
// =================================================================

/// One sweep row: the tool, its otherwise-valid arguments — so the
/// unknown repo is the row's only fault — and the uniform code the
/// row asserts. Alphabetical by tool, the `tools/list` order.
const SWEEP: [(&str, &str, &str); 24] = [
    (
        "decision_add",
        r#"{"repo":"nosuch","title":"x","status":"open","options":["(a)"],"confidence":"low"}"#,
        "unknown_id",
    ),
    ("decision_list", r#"{"repo":"nosuch"}"#, "unknown_id"),
    (
        "decision_show",
        r#"{"repo":"nosuch","decision":"fixture:D1"}"#,
        "unknown_id",
    ),
    (
        "decision_update",
        r#"{"repo":"nosuch","decision":"fixture:D1","title":"x"}"#,
        "unknown_id",
    ),
    ("doc_list", r#"{"repo":"nosuch"}"#, "unknown_id"),
    (
        "doc_register",
        r#"{"repo":"nosuch","title":"x","path":"docs/x.md","version":"1"}"#,
        "unknown_id",
    ),
    (
        "doc_set_path",
        r#"{"repo":"nosuch","doc":"fixture:S1","path":"docs/x.md"}"#,
        "unknown_id",
    ),
    (
        "doc_update",
        r#"{"repo":"nosuch","doc":"fixture:S1","title":"x"}"#,
        "unknown_id",
    ),
    (
        "epic_add",
        r#"{"repo":"nosuch","title":"x","description":"x","status":"open"}"#,
        "unknown_id",
    ),
    ("epic_list", r#"{"repo":"nosuch"}"#, "unknown_id"),
    (
        "epic_update",
        r#"{"repo":"nosuch","epic":"fixture:E1","title":"x"}"#,
        "unknown_id",
    ),
    ("graph_show", r#"{"repo":"nosuch"}"#, "unknown_id"),
    ("next_items", r#"{"repo":"nosuch"}"#, "unknown_id"),
    ("next_task", r#"{"repo":"nosuch"}"#, "unknown_id"),
    (
        "repo_register",
        r#"{"slug":"fixture","name":"x","path":"/p"}"#,
        "duplicate",
    ),
    (
        "repo_set_path",
        r#"{"repo":"nosuch","path":"/repos/nosuch"}"#,
        "unknown_id",
    ),
    ("repo_show", r#"{"repo":"nosuch"}"#, "unknown_id"),
    ("status_summary", r#"{"repo":"nosuch"}"#, "unknown_id"),
    (
        "task_add",
        r#"{"repo":"nosuch","epic":"fixture:E1","kind":"task","title":"x","body":"","status":"open","verify":"cargo test"}"#,
        "unknown_id",
    ),
    (
        "task_link",
        r#"{"repo":"nosuch","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        "unknown_id",
    ),
    ("task_list", r#"{"repo":"nosuch"}"#, "unknown_id"),
    (
        "task_show",
        r#"{"repo":"nosuch","task":"fixture:U1"}"#,
        "unknown_id",
    ),
    (
        "task_unlink",
        r#"{"repo":"nosuch","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        "unknown_id",
    ),
    (
        "task_update",
        r#"{"repo":"nosuch","task":"fixture:U1","title":"x"}"#,
        "unknown_id",
    ),
];

/// The carrier conformance sweep: one error path through every tool
/// that has a carrier-reachable one — the unknown repo (`"nosuch"` →
/// `unknown_id`) for the twenty-three repo-scoped tools, the
/// duplicate (the fixture's existing slug) for `repo_register` —
/// every response asserted to carry the committed D15's carrier in
/// the same shape. `repo_list` is excluded, and the exclusion is
/// deliberate: its cheap fault is the unknown-argument fault, which
/// is codeless *by design* (§6.1's consequence text — the schema
/// rejects an unknown field before the tool body runs), and its
/// carrier paths need a corrupt registry — a fixture this sweep does
/// not build. Twenty-four tools carry the pin.
#[test]
fn every_tools_error_carrier_is_shaped_identically() {
    let root = fixture();
    let requests: Vec<String> = SWEEP
        .iter()
        .enumerate()
        .map(|(index, (tool, arguments, _))| call(index as u64 + 2, tool, arguments))
        .collect();
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), SWEEP.len());
    for (index, (_, _, code)) in SWEEP.iter().enumerate() {
        // The interactive drive answers in request order.
        assert_carrier_shape(&driven.responses[index], code);
    }
}

// =================================================================
// The twice-run byte-identity
// =================================================================

/// One twice-run row: the tool, its representative happy-path
/// request, and whether the response is clock-stamped (compared with
/// the stamps normalized) or clock-free (compared byte-exactly). The
/// writes mutate the fresh fixture in this fixed sequence, so both
/// runs see the same states and allocate the same ids.
const TWICE_RUN: [(&str, &str, bool); 25] = [
    // The reads, over the pristine fixture — clock-free.
    ("next_task", r#"{"repo":"fixture"}"#, false),
    ("next_items", r#"{"repo":"fixture"}"#, false),
    ("status_summary", r#"{"repo":"fixture"}"#, false),
    (
        "task_show",
        r#"{"repo":"fixture","task":"fixture:U1"}"#,
        false,
    ),
    (
        "decision_show",
        r#"{"repo":"fixture","decision":"fixture:D1"}"#,
        false,
    ),
    ("task_list", r#"{"repo":"fixture"}"#, false),
    ("epic_list", r#"{"repo":"fixture"}"#, false),
    ("decision_list", r#"{"repo":"fixture"}"#, false),
    ("doc_list", r#"{"repo":"fixture"}"#, false),
    ("graph_show", r#"{}"#, false),
    // The repo tools: three clock-free, `repo_register` stamped in
    // `registered_at`.
    ("repo_list", r#"{}"#, false),
    ("repo_show", r#"{"repo":"fixture"}"#, false),
    (
        "repo_register",
        r#"{"slug":"gamma","name":"Gamma","path":"/repos/gamma"}"#,
        true,
    ),
    (
        "repo_set_path",
        r#"{"repo":"fixture","path":"/repos/fixture-moved"}"#,
        false,
    ),
    // The epic writers — an epic carries no timestamps (§3).
    (
        "epic_add",
        r#"{"repo":"fixture","title":"Third epic","description":"The third epic.","status":"open"}"#,
        false,
    ),
    (
        "epic_update",
        r#"{"repo":"fixture","epic":"fixture:E3","title":"Third epic retitled"}"#,
        false,
    ),
    // The edge writers — the echo carries no timestamp.
    (
        "task_link",
        r#"{"repo":"fixture","edge":"blocks","from":"fixture:U4","to":"fixture:U9"}"#,
        false,
    ),
    (
        "task_unlink",
        r#"{"repo":"fixture","edge":"blocks","from":"fixture:U1","to":"fixture:U2"}"#,
        false,
    ),
    // The stamped writers: tasks, decisions, docs.
    (
        "task_add",
        r#"{"repo":"fixture","epic":"fixture:E3","kind":"task","title":"Third epic task","body":"","status":"open","verify":"cargo test -p fixture"}"#,
        true,
    ),
    (
        "task_update",
        r#"{"repo":"fixture","task":"fixture:U1","title":"Wire the sensor retitled"}"#,
        true,
    ),
    (
        "decision_add",
        r#"{"repo":"fixture","title":"Fourth decision","status":"open","options":["(a)","(b)"],"confidence":"medium"}"#,
        true,
    ),
    (
        "decision_update",
        r#"{"repo":"fixture","decision":"fixture:D4","title":"Fourth decision retitled"}"#,
        true,
    ),
    (
        "doc_register",
        r#"{"repo":"fixture","title":"Second doc","path":"docs/second.md","version":"1"}"#,
        true,
    ),
    (
        "doc_set_path",
        r#"{"repo":"fixture","doc":"fixture:S2","path":"docs/second-moved.md"}"#,
        true,
    ),
    (
        "doc_update",
        r#"{"repo":"fixture","doc":"fixture:S2","title":"Second doc retitled"}"#,
        true,
    ),
];

/// The full response set's twice-run identity (§8): one representative
/// happy-path request per tool — all twenty-five — driven twice
/// across two fresh spawns over two fresh fixtures. The rows flagged
/// stamped — the task/decision/doc writers, plus `repo_register`,
/// whose happy response carries the clock in `registered_at` —
/// compare with the stamped values normalized; every other row
/// compares byte-exactly. The one honest nondeterminism is the
/// server's clock, stamped per call; §8's property for a
/// clock-carrying server is identity modulo it.
#[test]
fn the_full_response_set_is_byte_identical_across_two_runs() {
    let requests: Vec<String> = TWICE_RUN
        .iter()
        .enumerate()
        .map(|(index, (tool, arguments, _))| call(index as u64 + 2, tool, arguments))
        .collect();
    let root_a = fixture();
    let first = drive(root_a.path(), &requests);
    let root_b = fixture();
    let second = drive(root_b.path(), &requests);
    assert!(
        first.status.success() && second.status.success(),
        "completed handshakes end 0; stderr: {} / {}",
        first.stderr,
        second.stderr
    );
    assert_eq!(first.responses.len(), TWICE_RUN.len());
    assert_eq!(second.responses.len(), TWICE_RUN.len());
    // The rows are happy paths, and the comparison below would pass
    // vacuously if a uniformly-broken server answered every request
    // with the same error — two identical failures are still
    // byte-identical. The success assertion holds the premise.
    for (run, driven) in [("first", &first), ("second", &second)] {
        for (index, (tool, _, _)) in TWICE_RUN.iter().enumerate() {
            let response = &driven.responses[index];
            let value: Value = serde_json::from_str(response).expect("the response line is JSON");
            assert_eq!(
                value["result"]["isError"],
                json!(false),
                "{run} run, {tool}: {response}"
            );
        }
    }
    for (index, (tool, _, stamped)) in TWICE_RUN.iter().enumerate() {
        let left = &first.responses[index];
        let right = &second.responses[index];
        if *stamped {
            assert_eq!(
                normalize_stamps(left),
                normalize_stamps(right),
                "{tool}: identical modulo the clock\n  first:  {left}\n  second: {right}"
            );
        } else {
            assert_eq!(
                left, right,
                "{tool}: byte-identical\n  first:  {left}\n  second: {right}"
            );
        }
    }
}

// =================================================================
// The concurrency proof
// =================================================================

/// The consistency burst (§8): a mixed rapid-fire burst over one
/// connection — three `task_add`s, an `epic_add`, a `task_link`, and
/// the reads `task_list`, `next_task`, `status_summary` — with every
/// response matched to its request by id. The dispatch order under
/// the state mutex is nondeterministic, so the three adds' allocated
/// ids are asserted **as a set** (the per-request mapping is not
/// assertable); every response is a valid success for its request id;
/// and the reopen after the drive shows every write on disk with the
/// graph holding exactly the expected nodes and edges — the store
/// consistent, nothing torn.
#[test]
fn concurrent_calls_over_one_connection_leave_the_store_consistent() {
    let root = fixture();
    let requests = vec![
        call(
            2,
            "task_add",
            r#"{"repo":"fixture","epic":"fixture:E2","kind":"task","title":"Concurrent one","body":"","status":"open","verify":"cargo test -p fixture"}"#,
        ),
        call(
            3,
            "task_add",
            r#"{"repo":"fixture","epic":"fixture:E2","kind":"task","title":"Concurrent two","body":"","status":"open","verify":"cargo test -p fixture"}"#,
        ),
        call(
            4,
            "task_add",
            r#"{"repo":"fixture","epic":"fixture:E2","kind":"task","title":"Concurrent three","body":"","status":"open","verify":"cargo test -p fixture"}"#,
        ),
        call(
            5,
            "epic_add",
            r#"{"repo":"fixture","title":"Concurrent epic","description":"The concurrent epic.","status":"open"}"#,
        ),
        call(
            6,
            "task_link",
            r#"{"repo":"fixture","edge":"blocks","from":"fixture:U4","to":"fixture:U9"}"#,
        ),
        call(7, "task_list", r#"{"repo":"fixture"}"#),
        call(8, "next_task", r#"{"repo":"fixture"}"#),
        call(9, "status_summary", r#"{"repo":"fixture"}"#),
    ];
    let rapid = rapid_fire(root.path(), &requests);
    assert!(
        rapid.status.success(),
        "a completed handshake ends 0; stderr: {}",
        rapid.stderr
    );
    let matched = by_id(&rapid.responses);
    assert_eq!(matched.len(), requests.len(), "every request answered");
    // Every response is a valid success for its request id. The
    // reads' contents are deliberately not pinned: they race with the
    // writes, and which state they see is the dispatch order's
    // business, not this test's.
    for (id, response) in &matched {
        assert_eq!(
            response["result"]["isError"],
            json!(false),
            "id {id}: {response}"
        );
    }
    // The three adds' allocated ids, as a set: which request got
    // which id is the dispatch order's, nondeterministic under the
    // mutex.
    let mut allocated = BTreeSet::new();
    for id in [2u64, 3, 4] {
        let response = &matched[&id];
        allocated.insert(
            response["result"]["structuredContent"]["id"]
                .as_str()
                .unwrap_or_else(|| panic!("id {id} answers a task id: {response}"))
                .to_owned(),
        );
    }
    assert_eq!(
        allocated,
        BTreeSet::from([
            "fixture:U22".to_owned(),
            "fixture:U23".to_owned(),
            "fixture:U24".to_owned()
        ]),
        "the burst allocated exactly the next three task ids"
    );
    // The epic and the link are single writes: their answers are
    // deterministic.
    assert_eq!(
        matched[&5]["result"]["structuredContent"]["id"],
        json!("fixture:E3"),
        "the epic add allocated the next epic id"
    );
    assert_eq!(
        matched[&6]["result"]["structuredContent"],
        json!({"edge": "blocks", "from": "fixture:U4", "to": "fixture:U9"}),
        "the link answers the echo"
    );
    // The reopen: every write on disk, the graph exactly the expected
    // nodes and edges — the server has exited, its locks released.
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("fixture").expect("a valid slug"))
        .expect("the fixture store reopens");
    let graph = store.graph();
    let epic_ids: Vec<String> = graph.epic_list().iter().map(ToString::to_string).collect();
    assert_eq!(
        epic_ids,
        ["fixture:E1", "fixture:E2", "fixture:E3"],
        "the fixture's epics plus the burst's"
    );
    assert_eq!(
        graph.tasks().count(),
        24,
        "the fixture's 21 tasks plus the burst's 3"
    );
    let decision_ids: Vec<String> = graph
        .decision_list()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        decision_ids,
        ["fixture:D1", "fixture:D2", "fixture:D3"],
        "the burst added no decisions"
    );
    let doc_ids: Vec<String> = graph.doc_list().iter().map(ToString::to_string).collect();
    assert_eq!(doc_ids, ["fixture:S1"], "the burst added no docs");
    let mut titles = BTreeSet::new();
    for id in ["fixture:U22", "fixture:U23", "fixture:U24"] {
        let task = graph
            .task(&id.parse().expect("a valid task id"))
            .unwrap_or_else(|| panic!("{id} is on disk"));
        assert_eq!(task.epic.to_string(), "fixture:E2", "{id} sits under E2");
        titles.insert(task.title.clone());
    }
    assert_eq!(
        titles,
        BTreeSet::from([
            "Concurrent one".to_owned(),
            "Concurrent two".to_owned(),
            "Concurrent three".to_owned()
        ]),
        "the three adds' titles, as a set"
    );
    // The edges: the fixture's nine blocks edges plus the burst's
    // U4→U9, nothing more, nothing torn — compared as a sorted set of
    // (source, target) id pairs.
    let mut edges: Vec<(String, String)> = graph
        .blocks_edges()
        .iter()
        .map(|edge| match edge {
            BlocksEdge::TaskToTask { from, to } => (from.to_string(), to.to_string()),
            BlocksEdge::DecisionToTask { from, to } => (from.to_string(), to.to_string()),
            BlocksEdge::DecisionToEpic { from, to } => (from.to_string(), to.to_string()),
        })
        .collect();
    edges.sort();
    assert_eq!(
        edges,
        vec![
            ("fixture:D1".to_owned(), "fixture:E1".to_owned()),
            ("fixture:D1".to_owned(), "fixture:U3".to_owned()),
            ("fixture:D2".to_owned(), "fixture:U2".to_owned()),
            ("fixture:D3".to_owned(), "fixture:U10".to_owned()),
            ("fixture:U1".to_owned(), "fixture:U2".to_owned()),
            ("fixture:U1".to_owned(), "fixture:U3".to_owned()),
            ("fixture:U4".to_owned(), "fixture:U9".to_owned()),
            ("fixture:U6".to_owned(), "fixture:U21".to_owned()),
            ("fixture:U6".to_owned(), "fixture:U8".to_owned()),
            ("fixture:U9".to_owned(), "fixture:U20".to_owned()),
        ],
        "the fixture's edges plus the burst's, exactly"
    );
    let relates: Vec<(String, String)> = graph
        .relates_pairs()
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    assert_eq!(
        relates,
        vec![("fixture:U1".to_owned(), "fixture:U6".to_owned())],
        "the fixture's annotation is intact"
    );
}

/// U1's full wire record with `title` substituted — every field the
/// fixture gives it, from the task_show golden's own bytes.
fn u1_record(title: &str) -> Value {
    json!({
        "id": "fixture:U1",
        "repo": "fixture",
        "epic": "fixture:E1",
        "kind": "task",
        "title": title,
        "body": "",
        "status": "open",
        "verify": "cargo test -p fixture",
        "evidence": null,
        "note": null,
        "references": [
            {"kind": "tracker", "target": "FIX-1", "note": "the issue"},
            {"kind": "doc", "target": "fixture:S1#2", "note": null}
        ],
        "priority": "high",
        "until": "2026-10-15",
        "created": "2026-10-01T08:10:01.000000000Z",
        "updated": "2026-10-01T08:10:01.000000000Z"
    })
}

/// The write-then-read race (§8): a rapid-fire burst of
/// `[task_update(U1's title), task_show(U1)]` — the write and the
/// read dispatched concurrently, the state mutex serializing their
/// bodies. The read's response must be the pre-state or the
/// post-state — a whole, valid task record either way, never a torn
/// one. The post-state is pinned exactly: it is the record the
/// write's own response answered, `updated` stamp included, so a read
/// mixing the two states — the new title with the old stamp, or any
/// field pair straddling the write — matches neither and fails.
#[test]
fn a_concurrent_read_sees_a_whole_state_never_a_torn_one() {
    let root = fixture();
    let requests = vec![
        call(
            2,
            "task_update",
            r#"{"repo":"fixture","task":"fixture:U1","title":"Wire the sensor retitled"}"#,
        ),
        call(3, "task_show", r#"{"repo":"fixture","task":"fixture:U1"}"#),
    ];
    let rapid = rapid_fire(root.path(), &requests);
    assert!(
        rapid.status.success(),
        "a completed handshake ends 0; stderr: {}",
        rapid.stderr
    );
    let matched = by_id(&rapid.responses);
    assert_eq!(matched.len(), 2, "both requests answered");
    // The write's own response: the post-state, new title and all.
    let update = &matched[&2];
    assert_eq!(update["result"]["isError"], json!(false), "{update}");
    assert_eq!(
        update["result"]["structuredContent"]["title"],
        json!("Wire the sensor retitled"),
        "{update}"
    );
    // The read's response: the pre-state or the write's own
    // post-state — the disjunction is the whole-record comparison,
    // updated stamp included.
    let read = &matched[&3];
    assert_eq!(read["result"]["isError"], json!(false), "{read}");
    let read_state = read["result"]["structuredContent"].clone();
    let pre_state = u1_record("Wire the sensor");
    let post_state = update["result"]["structuredContent"].clone();
    assert!(
        read_state == pre_state || read_state == post_state,
        "the read is the pre-state or the write's post-state, whole either way:\n  {read}"
    );
}

// =================================================================
// The U26 hammer
// =================================================================

/// The hammer's round count — a working number, not a proof: the
/// disposition records whatever the runs show, and an unreproduced
/// hammer is recorded as such, never as a closure. Raised from the
/// spec's working fifty after the first runs were clean and the
/// target's runtime stayed in family with the read and write suites'.
const ROUNDS: usize = 150;

/// The U26 hammer: the protocol suite's post-exit flake —
/// `repo_register_creates_a_real_store_and_returns_the_new_state`
/// failing on "the store opens after the server exits: store is
/// locked: repo camera" (~3/83 runs during t37's review, green on
/// every immediate re-run) — reproduced or closed under this
/// machinery. Each round repeats the flaky test's exact sequence:
/// fresh root, `repo_register` over the wire, the server exits, the
/// in-process `repo_list()`, the final open — asserted to succeed
/// every round, with the panic carrying the `StoreError` so a
/// recurrence is captured, not silent. The disposition when this
/// suite landed: **unreproduced** — the hammer and the repetition
/// campaign recorded in the tracker's t38 evidence all ran clean, so
/// U26 stays open with those numbers and the flake remains the smoke
/// session's watch item; the hammer stays in the suite as the
/// standing detector, and a reproduction isolates its cause by the
/// two named suspects — a variant without the `repo_list()` call
/// (self-contention), a variant with a bounded sleep before the open
/// (the flock outliving the reap).
#[test]
fn the_u26_sequence_hammers_clean_or_reproduces() {
    for round in 0..ROUNDS {
        let root = TempDir::new().expect("a fresh temp root");
        let (stdout, stderr, status) = drive_to_exit(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{REGISTER_CAMERA}\n"),
        );
        assert!(
            status.success(),
            "round {round}: a completed handshake ends 0; stderr: {stderr}"
        );
        assert_eq!(
            stdout.lines().count(),
            2,
            "round {round}: the handshake and the register answered"
        );
        let data_root = DataRoot::new(root.path());
        let repos = data_root
            .repo_list()
            .unwrap_or_else(|error| panic!("round {round}: the registry lists: {error}"));
        assert_eq!(
            repos.len(),
            1,
            "round {round}: the registry lists exactly camera"
        );
        match data_root.open(&repos[0].slug) {
            Ok(_) => {}
            Err(error) => panic!("round {round}: the store opens after the server exits: {error}"),
        }
    }
}
