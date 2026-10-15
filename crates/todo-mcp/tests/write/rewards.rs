//! The rewards surface's suite (U62, §15/D24): the closure response's
//! reward line and reflection question, `reward_reflect`, the
//! estimate, and the two fences — the disabled root's byte-identity
//! and the triggers' semantics — driven through the protocol over the
//! real binary, never by calling the handler around it (§10).
//!
//! The suite builds its **own** fixture (`rewards_root`), because the
//! shared `common` fixture must stay configless: its goldens pin the
//! inert root, and a config there would move them. The fixture holds
//! a milestone epic (the provenance description the heuristic reads)
//! and a plain one, so the milestone trigger's positive and negative
//! cases sit in one graph, and the config is a parameter — `None` is
//! the absent-config root, `Some(enabled: false)` the disabled one,
//! both pinned byte-identical to the pre-E13 closure response.
//!
//! The stamp extractors and the byte-exact success assertion are
//! suite-local copies of `write/nodes.rs`'s (that suite's own
//! provenance rule: helpers stay in the file that owns them, and
//! widening the shared surface would edit a file this suite does
//! not).
//!
//! The arithmetic the assertions expect (the domain test table's
//! numbers: m=25, high=150, task=100, no-priority ×100): a
//! no-priority M closure grants 25, a high-priority one 38
//! (37.5 rounds half up), an L closure 50.

use serde_json::json;
use tempfile::TempDir;
use todo_domain::{
    BaseCosts, KindMultipliers, Priority, PriorityMultipliers, References, Repo, RewardsConfig,
    Size, Status, TaskKind,
};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, call, drive, fixture};
use crate::reopened;

// =================================================================
// The suite-local helpers — copied from write/nodes.rs
// =================================================================

/// The length of a canonical timestamp on the wire — the 30-byte RFC
/// 3339 UTC form with the nine-digit fraction.
const TIMESTAMP_LEN: usize = 30;

/// Extracts the clock-stamped `updated` value from one response line.
fn updated_of(response: &str) -> String {
    stamp_of(response, r#""updated":""#)
}

/// The extractor behind [`updated_of`]: the 30 bytes after the
/// needle.
fn stamp_of(response: &str, needle: &str) -> String {
    let start = response
        .find(needle)
        .unwrap_or_else(|| panic!("the response carries {needle}: {response}"))
        + needle.len();
    let end = start + TIMESTAMP_LEN;
    response[start..end].to_owned()
}

/// Asserts one success response byte-exactly: the full JSON-RPC line
/// rebuilt from `expected_structured` and its text mirror. The
/// mirror escapes backslashes **before** quotes — a departure from
/// `write/nodes.rs`'s copy, whose payloads carry no backslash to
/// escape ("the only character these payloads carry that serde_json
/// escapes"); the reflection question embeds the trigger's JSON
/// spelling, so its structuredContent carries `\"` sequences the
/// mirror must double-escape exactly as serde_json does.
fn assert_success(response: &str, id: u64, expected_structured: &str) {
    let response = response.trim_end_matches('\n');
    let mirror = expected_structured
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    assert_eq!(
        response,
        format!(
            r#"{{"jsonrpc":"2.0","id":{id},"result":{{"content":[{{"type":"text","text":"{mirror}"}}],"structuredContent":{expected_structured},"isError":false}}}}"#
        )
    );
}

/// One JSON string value, quoted and escaped — the reflection
/// question carries embedded quotes (the trigger spelling it
/// teaches), so a template cannot spell it raw.
fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("a string serializes")
}

// =================================================================
// The suite's own fixture
// =================================================================

/// The enabled config the suite drives against: the domain test
/// table's numbers, the `reflect_every` threshold per test (`0` is
/// the never-fires spelling).
fn config(reflect_every: u64) -> RewardsConfig {
    RewardsConfig {
        enabled: true,
        currency: "coffee".to_owned(),
        base_costs: BaseCosts {
            s: 10,
            m: 25,
            l: 50,
            xl: 100,
        },
        default_size: Size::M,
        priority_multipliers: PriorityMultipliers {
            high: 150,
            medium: 100,
            low: 50,
        },
        kind_multipliers: KindMultipliers {
            task: 100,
            gate: 100,
            finding: 50,
            debt: 25,
        },
        reflect_every,
    }
}

/// One fixture task's fixed creation instant — 08:10:0N, distinct per
/// task, in the canonical 30-byte spelling the response carries.
fn created_at(n: u64) -> String {
    format!("2026-10-02T08:10:{n:02}.000000000Z")
}

/// Builds the rewards fixture under a fresh temp root: one repo
/// (`fixture`) with **E1**, a milestone epic (the provenance
/// description the heuristic reads) holding U1–U2, and **E2**, a
/// plain epic holding U3–U6 (U6 high-priority, the rest none) — and,
/// when `config` is `Some`, the root's rewards config saved as the
/// canonical document. Every store handle is dropped before the
/// server spawns (the advisory lock is per process).
fn rewards_root(config: Option<&RewardsConfig>) -> TempDir {
    let root = TempDir::new().expect("a fresh temp root");
    let data_root = DataRoot::new(root.path());
    let mut store = data_root
        .register(Repo {
            slug: "fixture".parse().expect("a valid slug"),
            name: "Fixture".to_owned(),
            path: "/repos/fixture".to_owned(),
            registered_at: "2026-10-02T08:00:00Z".parse().expect("a fixed timestamp"),
        })
        .expect("the fixture repo registers");
    let milestone = store
        .add_epic(
            todo_domain::EpicDraft {
                title: "Milestone epic".to_owned(),
                description: "the fixture board's §2 milestone M1".to_owned(),
                status: Status::Open,
                references: References::new(),
            },
            "2026-10-02T08:01:00Z".parse().expect("a fixed timestamp"),
        )
        .expect("E1 lands");
    let plain = store
        .add_epic(
            todo_domain::EpicDraft {
                title: "Plain epic".to_owned(),
                description: "The plain epic.".to_owned(),
                status: Status::InWork,
                references: References::new(),
            },
            "2026-10-02T08:02:00Z".parse().expect("a fixed timestamp"),
        )
        .expect("E2 lands");
    let tasks = [
        (milestone.id.clone(), "First milestone task", None),
        (milestone.id, "Last milestone task", None),
        (plain.id.clone(), "Plain task one", None),
        (plain.id.clone(), "Plain task two", None),
        (plain.id.clone(), "Plain task three", None),
        (plain.id, "High priority task", Some(Priority::High)),
    ];
    for (index, (epic, title, priority)) in tasks.iter().enumerate() {
        store
            .add_task(
                todo_domain::TaskDraft {
                    epic: epic.clone(),
                    kind: TaskKind::Task,
                    title: title.to_string(),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test -p fixture".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: *priority,
                    until: None,
                    estimate: None,
                },
                format!("2026-10-02T08:10:{:02}Z", index + 1)
                    .parse()
                    .expect("a fixed timestamp"),
            )
            .expect("the task lands");
    }
    drop(store);
    if let Some(config) = config {
        data_root.save_config(config).expect("the config saves");
    }
    root
}

/// One fixture task's expected `structuredContent` after closure: the
/// fixed fields — id and creation instant derived from the task
/// number `n` — with the closure's evidence and status substituted,
/// `updated` the extracted stamp, and the two rewards lines appended
/// when the closure produced them — the field order D14 pins, the
/// two optional lines last.
fn closed_expected(
    n: usize,
    epic: &str,
    title: &str,
    priority: Option<&str>,
    updated: &str,
    reward: Option<&str>,
    reflection: Option<&str>,
) -> String {
    let priority = match priority {
        Some(priority) => format!("\"{priority}\""),
        None => "null".to_owned(),
    };
    let mut expected = format!(
        r#"{{"id":"fixture:U{n}","repo":"fixture","epic":"{epic}","kind":"task","title":"{title}","body":"","status":"closed","verify":"cargo test -p fixture","evidence":"the closure evidence","note":null,"references":[],"priority":{priority},"until":null,"created":"{}","updated":"{updated}""#,
        created_at(n as u64)
    );
    if let Some(reward) = reward {
        expected.push_str(&format!(",\"reward\":{}", json_string(reward)));
    }
    if let Some(reflection) = reflection {
        expected.push_str(&format!(",\"reflection\":{}", json_string(reflection)));
    }
    expected.push('}');
    expected
}

/// The closure request over one fixture task, with the evidence the
/// closure discipline requires.
fn close(id: u64, task: &str) -> String {
    call(
        id,
        "task_update",
        &format!(
            r#"{{"repo":"fixture","task":"{task}","status":"closed","evidence":"the closure evidence"}}"#
        ),
    )
}

// =================================================================
// The reward line
// =================================================================

/// The enabled closure carries the reward line (§15's exact shape):
/// U6's high-priority M closure grants 38 (37.5 rounds half up) with
/// the root total at 38; U1's no-priority closure then grants 25 with
/// the total at 63 — the running derivation pinned — and the reopen
/// shows both grants on disk.
#[test]
fn the_enabled_closure_carries_the_reward_line() {
    let root = rewards_root(Some(&config(0)));
    let driven = drive(
        root.path(),
        &[close(2, "fixture:U6"), close(3, "fixture:U1")],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    let stamp = updated_of(&driven.responses[0]);
    assert_success(
        &driven.responses[0],
        2,
        &closed_expected(
            6,
            "fixture:E2",
            "High priority task",
            Some("high"),
            &stamp,
            Some("reward: 38 coffee (M × high). root total: 38 coffee."),
            None,
        ),
    );
    let stamp = updated_of(&driven.responses[1]);
    assert_success(
        &driven.responses[1],
        3,
        &closed_expected(
            1,
            "fixture:E1",
            "First milestone task",
            None,
            &stamp,
            Some("reward: 25 coffee (M × none). root total: 63 coffee."),
            None,
        ),
    );
    let store = reopened(root.path());
    let census = DataRoot::new(root.path())
        .read_repo_census(&"fixture".parse().expect("a valid slug"))
        .expect("the census reads");
    assert_eq!(census.by_kind.get("reward_grant"), Some(&2));
    assert_eq!(
        store
            .graph()
            .task(&"fixture:U6".parse().expect("a valid task id"))
            .expect("U6 is on disk")
            .status,
        Status::Closed
    );
}

/// The actual size rides the closure call (§15): U3 closes with
/// `actual_size: "l"` and the reward scales on the L base (50), the
/// line's pair spelling the reported bucket.
#[test]
fn the_actual_size_rides_the_closure_call() {
    let root = rewards_root(Some(&config(0)));
    let driven = drive(
        root.path(),
        &[call(
            2,
            "task_update",
            r#"{"repo":"fixture","task":"fixture:U3","status":"closed","evidence":"the closure evidence","actual_size":"l"}"#,
        )],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    let stamp = updated_of(&driven.responses[0]);
    assert_success(
        &driven.responses[0],
        2,
        &closed_expected(
            3,
            "fixture:E2",
            "Plain task one",
            None,
            &stamp,
            Some("reward: 50 coffee (L × none). root total: 50 coffee."),
            None,
        ),
    );
}

// =================================================================
// The disabled fence — byte-identity
// =================================================================

/// §15's fence: a closure on a root with no config — and one with
/// `enabled: false` — answers byte-exactly today's shape: the plain
/// task state, no reward line, no reflection question, the two
/// optional keys absent rather than `null`.
#[test]
fn inert_and_disabled_closures_are_byte_identical_to_today() {
    let expected = |response: &str, id: u64, n: usize, epic: &str, title: &str| {
        let stamp = updated_of(response);
        assert_success(
            response,
            id,
            &closed_expected(n, epic, title, None, &stamp, None, None),
        );
    };
    let absent = rewards_root(None);
    let driven = drive(absent.path(), &[close(2, "fixture:U1")]);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    expected(
        &driven.responses[0],
        2,
        1,
        "fixture:E1",
        "First milestone task",
    );
    let disabled = rewards_root(Some(&{
        let mut config = config(0);
        config.enabled = false;
        config
    }));
    let driven = drive(disabled.path(), &[close(2, "fixture:U1")]);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    expected(
        &driven.responses[0],
        2,
        1,
        "fixture:E1",
        "First milestone task",
    );
    // And nothing was logged: no grant, no event.
    let census = DataRoot::new(disabled.path())
        .read_repo_census(&"fixture".parse().expect("a valid slug"))
        .expect("the census reads");
    assert_eq!(census.by_kind.get("reward_grant"), None);
}

/// The enabled closure is byte-identical across two runs over equal
/// state (§8), modulo the one honest nondeterminism — the server's
/// clock, stamped per call.
#[test]
fn the_enabled_closure_is_byte_identical_across_two_runs() {
    let request = [close(2, "fixture:U6")];
    let root_a = rewards_root(Some(&config(0)));
    let first = drive(root_a.path(), &request);
    let root_b = rewards_root(Some(&config(0)));
    let second = drive(root_b.path(), &request);
    assert!(
        first.status.success() && second.status.success(),
        "completed handshakes end 0; stderr: {} / {}",
        first.stderr,
        second.stderr
    );
    let normalize = |response: &str| response.replace(&updated_of(response), "<stamp>");
    assert_eq!(
        normalize(&first.responses[0]),
        normalize(&second.responses[0]),
        "identical modulo the clock\n  first:  {}\n  second: {}",
        first.responses[0],
        second.responses[0]
    );
}

// =================================================================
// The milestone trigger
// =================================================================

/// The milestone trigger (§15): U1's closure under the milestone epic
/// asks nothing (U2 still open), U2's closure completes E1 and asks —
/// the question naming the epic and the trigger spelling the answer
/// echoes — a re-closing patch on the already-closed U2 asks nothing
/// a second time (the transition semantics), and completing the
/// plain epic E2 asks nothing at all (not a milestone). The answer
/// then logs through `reward_reflect` with the epic trigger.
#[test]
fn the_milestone_trigger_asks_on_the_completing_closure() {
    let root = rewards_root(Some(&config(0)));
    let driven = drive(
        root.path(),
        &[
            close(2, "fixture:U1"),
            close(3, "fixture:U2"),
            // The re-closing patch: no transition, no grant, no
            // question.
            call(
                4,
                "task_update",
                r#"{"repo":"fixture","task":"fixture:U2","status":"closed"}"#,
            ),
            // Completing the plain epic asks nothing.
            close(5, "fixture:U3"),
            close(6, "fixture:U4"),
            close(7, "fixture:U5"),
            close(8, "fixture:U6"),
            call(
                9,
                "reward_reflect",
                r#"{"repo":"fixture","answer":"The milestone closed clean; the reward feels earned.","trigger":{"trigger":"epic_closure","epic":"fixture:E1"}}"#,
            ),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    // U1's closure: the reward line alone — the epic is not yet
    // complete.
    let stamp = updated_of(&driven.responses[0]);
    assert_success(
        &driven.responses[0],
        2,
        &closed_expected(
            1,
            "fixture:E1",
            "First milestone task",
            None,
            &stamp,
            Some("reward: 25 coffee (M × none). root total: 25 coffee."),
            None,
        ),
    );
    // U2's closure: the completing moment — the reward line and the
    // milestone question.
    let stamp = updated_of(&driven.responses[1]);
    assert_success(
        &driven.responses[1],
        3,
        &closed_expected(
            2,
            "fixture:E1",
            "Last milestone task",
            None,
            &stamp,
            Some("reward: 25 coffee (M × none). root total: 50 coffee."),
            Some(
                "The milestone epic fixture:E1 closed. How does the reward feel? Answer \
                 honestly, once, through reward_reflect with trigger \
                 {\"trigger\":\"epic_closure\",\"epic\":\"fixture:E1\"}.",
            ),
        ),
    );
    // The re-closing patch: the plain state, no line, no question.
    let stamp = updated_of(&driven.responses[2]);
    assert_success(
        &driven.responses[2],
        4,
        &closed_expected(
            2,
            "fixture:E1",
            "Last milestone task",
            None,
            &stamp,
            None,
            None,
        ),
    );
    // The plain epic's completions: reward lines, never a question.
    let plain_closures = [
        (
            3usize,
            5u64,
            3usize,
            "Plain task one",
            "reward: 25 coffee (M × none). root total: 75 coffee.",
            None,
        ),
        (
            4,
            6,
            4,
            "Plain task two",
            "reward: 25 coffee (M × none). root total: 100 coffee.",
            None,
        ),
        (
            5,
            7,
            5,
            "Plain task three",
            "reward: 25 coffee (M × none). root total: 125 coffee.",
            None,
        ),
        (
            6,
            8,
            6,
            "High priority task",
            "reward: 38 coffee (M × high). root total: 163 coffee.",
            Some("high"),
        ),
    ];
    for (position, id, n, title, reward, priority) in plain_closures {
        let response = &driven.responses[position];
        let stamp = updated_of(response);
        assert_success(
            response,
            id,
            &closed_expected(n, "fixture:E2", title, priority, &stamp, Some(reward), None),
        );
    }
    // The answer: logged verbatim, the totals at the answer's moment.
    assert_success(
        &driven.responses[7],
        9,
        r#"{"answer":"The milestone closed clean; the reward feels earned.","trigger":{"trigger":"epic_closure","epic":"fixture:E1"},"root_total":163,"repo_total":163}"#,
    );
}

// =================================================================
// The threshold trigger — the from-below crossing
// =================================================================

/// When both triggers fire on one closure — U2's completion of the
/// milestone epic also lands the total exactly on the threshold —
/// the milestone question wins (the rewards module's doc: the
/// completion moment subsumes the accumulation one, and the response
/// carries one question).
#[test]
fn the_milestone_question_wins_when_both_triggers_fire() {
    let root = rewards_root(Some(&config(50)));
    let driven = drive(
        root.path(),
        &[close(2, "fixture:U1"), close(3, "fixture:U2")],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    let value: serde_json::Value =
        serde_json::from_str(&driven.responses[1]).expect("the response line is JSON");
    let reflection = value["result"]["structuredContent"]["reflection"]
        .as_str()
        .expect("the closure asked");
    assert!(
        reflection.starts_with("The milestone epic fixture:E1 closed."),
        "the milestone question, not the threshold's: {reflection}"
    );
}

/// `reflect_every`'s from-below crossing (§15, the E13 handover's
/// carried note): with the threshold at 50, U3's closure lands at 25
/// (below — no question), U4's lands exactly at 50 (the crossing —
/// the question), and U5's lands at 75 (already above — **no**
/// re-question: the pre-grant total is not below the threshold). The
/// at-threshold boundary pins the crossing as `before < threshold
/// <= after`.
#[test]
fn the_threshold_crossing_asks_once_from_below() {
    let root = rewards_root(Some(&config(50)));
    let driven = drive(
        root.path(),
        &[
            close(2, "fixture:U3"),
            close(3, "fixture:U4"),
            close(4, "fixture:U5"),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    // Below: the reward line alone.
    let stamp = updated_of(&driven.responses[0]);
    assert_success(
        &driven.responses[0],
        2,
        &closed_expected(
            3,
            "fixture:E2",
            "Plain task one",
            None,
            &stamp,
            Some("reward: 25 coffee (M × none). root total: 25 coffee."),
            None,
        ),
    );
    // The crossing: the question, the threshold and the total named.
    let stamp = updated_of(&driven.responses[1]);
    assert_success(
        &driven.responses[1],
        3,
        &closed_expected(
            4,
            "fixture:E2",
            "Plain task two",
            None,
            &stamp,
            Some("reward: 25 coffee (M × none). root total: 50 coffee."),
            Some(
                "The root's reward total crossed 50 coffee (now 50 coffee). How does the \
                 reward feel? Answer honestly, once, through reward_reflect with trigger \
                 {\"trigger\":\"threshold\"}.",
            ),
        ),
    );
    // Above: no re-question.
    let stamp = updated_of(&driven.responses[2]);
    assert_success(
        &driven.responses[2],
        4,
        &closed_expected(
            5,
            "fixture:E2",
            "Plain task three",
            None,
            &stamp,
            Some("reward: 25 coffee (M × none). root total: 75 coffee."),
            None,
        ),
    );
}

// =================================================================
// reward_reflect — the round trip and the error paths
// =================================================================

/// `reward_reflect` logs the verbatim answer as an event (§15): the
/// response carries the answer, the trigger and the derived totals,
/// and the reopen shows the event on disk — the log's own line with
/// the answer verbatim, and the census counting it.
#[test]
fn reward_reflect_logs_the_verbatim_answer() {
    let root = rewards_root(Some(&config(50)));
    let driven = drive(
        root.path(),
        &[
            close(2, "fixture:U3"),
            close(3, "fixture:U4"),
            call(
                4,
                "reward_reflect",
                r#"{"repo":"fixture","answer":"It feels earned.","trigger":{"trigger":"threshold"}}"#,
            ),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_success(
        &driven.responses[2],
        4,
        r#"{"answer":"It feels earned.","trigger":{"trigger":"threshold"},"root_total":50,"repo_total":50}"#,
    );
    // The reopen: the event is on disk, the answer verbatim.
    let _ = reopened(root.path());
    let log = std::fs::read_to_string(
        root.path()
            .join("repos")
            .join("fixture")
            .join("event-log.jsonl"),
    )
    .expect("the log reads");
    let last = log
        .lines()
        .last()
        .expect("the log holds the reflection line");
    let event: serde_json::Value = serde_json::from_str(last).expect("the line is JSON");
    assert_eq!(
        event,
        json!({
            "kind": "reward_reflect",
            "ts": event["ts"].clone(),
            "answer": "It feels earned.",
            "trigger": {"trigger": "threshold"},
            "root_total": 50,
            "repo_total": 50
        }),
        "the reflection event, verbatim: {last}"
    );
    let census = DataRoot::new(root.path())
        .read_repo_census(&"fixture".parse().expect("a valid slug"))
        .expect("the census reads");
    assert_eq!(census.by_kind.get("reward_reflect"), Some(&1));
    assert_eq!(census.by_kind.get("reward_grant"), Some(&2));
}

/// `reward_reflect`'s error paths: an inert root refuses (the
/// enabled-gate is the caller's — nothing was asked), an unknown repo
/// answers `unknown_id` before the gate, a malformed epic id in the
/// trigger answers `invalid` before the store is touched, and an
/// unknown field is rmcp's codeless argument fault.
#[test]
fn reward_reflect_refuses_where_nothing_asked() {
    let root = rewards_root(None);
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "reward_reflect",
                r#"{"repo":"fixture","answer":"x","trigger":{"trigger":"threshold"}}"#,
            ),
            call(
                3,
                "reward_reflect",
                r#"{"repo":"nosuch","answer":"x","trigger":{"trigger":"threshold"}}"#,
            ),
            call(
                4,
                "reward_reflect",
                r#"{"repo":"fixture","answer":"x","trigger":{"trigger":"epic_closure","epic":"bad"}}"#,
            ),
            call(
                5,
                "reward_reflect",
                r#"{"repo":"fixture","answer":"x","trigger":{"trigger":"threshold"},"extra":"y"}"#,
            ),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[0],
        "invalid",
        "the root's rewards are not enabled — no reflection was asked",
    );
    assert_carrier(&driven.responses[1], "unknown_id", "unknown id: nosuch");
    assert_carrier(&driven.responses[2], "invalid", "invalid node id: \"bad\"");
    let value: serde_json::Value =
        serde_json::from_str(&driven.responses[3]).expect("the response line is JSON");
    assert_eq!(value["result"]["isError"], json!(true));
    assert!(
        value["result"].get("structuredContent").is_none(),
        "an argument fault carries no code: {}",
        driven.responses[3]
    );
    let text = value["result"]["content"][0]["text"]
        .as_str()
        .expect("the fault text");
    assert!(
        text.contains("failed to deserialize parameters") && text.contains("unknown field `extra`"),
        "{text}"
    );
}

// =================================================================
// The estimate
// =================================================================

/// `task_add`'s estimate rides creation and is wire-write-only (§15's
/// signed shape): the response is today's TaskDto — no estimate key,
/// the goldens' bytes — while the reopen shows the estimate on the
/// node; `null` and absent are the same bytes to a creation call
/// (nothing to clear yet — P-LIVE-003's collapse is the correct
/// semantics here).
#[test]
fn the_estimate_rides_creation_and_is_write_only() {
    let root = rewards_root(None);
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E2","kind":"task","title":"Estimated task","body":"","status":"open","verify":"cargo test -p fixture","estimate":"m"}"#,
            ),
            call(
                3,
                "task_add",
                r#"{"repo":"fixture","epic":"fixture:E2","kind":"task","title":"Null estimate task","body":"","status":"open","verify":"cargo test -p fixture","estimate":null}"#,
            ),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    let adds = [
        (0usize, 2u64, 7usize, "Estimated task"),
        (1, 3, 8, "Null estimate task"),
    ];
    for (position, id, number, title) in adds {
        let response = &driven.responses[position];
        let stamp = updated_of(response);
        assert_success(
            response,
            id,
            &format!(
                r#"{{"id":"fixture:U{number}","repo":"fixture","epic":"fixture:E2","kind":"task","title":"{title}","body":"","status":"open","verify":"cargo test -p fixture","evidence":null,"note":null,"references":[],"priority":null,"until":null,"created":"{stamp}","updated":"{stamp}"}}"#
            ),
        );
    }
    let store = reopened(root.path());
    let graph = store.graph();
    assert_eq!(
        graph
            .task(&"fixture:U7".parse().expect("a valid task id"))
            .expect("U7 is on disk")
            .estimate,
        Some(Size::M)
    );
    assert_eq!(
        graph
            .task(&"fixture:U8".parse().expect("a valid task id"))
            .expect("U8 is on disk")
            .estimate,
        None
    );
}

// =================================================================
// The shared fixture's inertness — the goldens' premise
// =================================================================

/// The shared `common` fixture holds no config, so the write suite's
/// own closure responses (the discipline pair in `write/nodes.rs`)
/// already pin the inert closure byte-exactly; this test closes a
/// task on that fixture and asserts the response carries neither of
/// the two optional keys — the goldens' premise, stated where the
/// rewards suite can see it.
#[test]
fn the_shared_fixture_closure_carries_no_rewards_keys() {
    let root = fixture();
    let driven = drive(
        root.path(),
        &[call(
            2,
            "task_update",
            r#"{"repo":"fixture","task":"fixture:U4","status":"closed","evidence":"the samples archived twice"}"#,
        )],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    let response = &driven.responses[0];
    assert!(
        !response.contains("\"reward\"") && !response.contains("\"reflection\""),
        "the inert closure carries neither rewards key: {response}"
    );
    let value: serde_json::Value =
        serde_json::from_str(response).expect("the response line is JSON");
    assert_eq!(value["result"]["isError"], json!(false), "{response}");
    assert_eq!(
        value["result"]["structuredContent"]["status"],
        json!("closed")
    );
}
