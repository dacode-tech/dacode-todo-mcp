//! The read, write and import suites' shared halves: the golden-file
//! helper, the fixture store and the stdio drive. The write suites
//! (t36's node CRUD, t37's edges) import these helpers alongside the
//! read ones — the edges suite also drives with a configured default
//! repo — and the import suite (t44) drives its own root, leaving the
//! fixture half unreferenced in that target (allowed dead, the
//! section's own note) — so "read" in the lift's history below is
//! provenance, not scope. The protocol target joined at U66 (its own
//! local drive needed the daemon stop the shim leaves behind), and
//! the shim suite (U66) drives with the same helpers.
//!
//! The golden helper is a copy of
//! `crates/todo-store/tests/common/mod.rs`, kept verbatim per the cli
//! precedent — the workflow prescribes one shared helper *per crate*,
//! and a cross-crate test-helper dependency would be worse than the
//! copy. The only difference is this provenance paragraph. A fix to
//! one must propagate to the other.
//!
//! The fixture store is hand-built through the store's
//! public API under a temp root with fixed caller-passed timestamps —
//! fully deterministic, which is what makes the goldens honest — and
//! every store handle is dropped before the server spawns: the
//! advisory lock is per process, and the builder and the server are
//! two processes.
//!
//! The drive speaks raw JSON-RPC to the real binary over stdio, one
//! request at a time: each response line is read before the next
//! request is written, so the response order is the request order by
//! construction. rmcp dispatches each request in its own task (§6.3)
//! and the probe measured out-of-order arrivals under rapid-fire
//! writes — arrival order is a concurrency property t38 owns, not a
//! byte property these goldens pin. Since the no-subcommand path
//! became the shim (U66), every drive leaves a live daemon per root
//! — the shim spawns it — and the drive stops it after the session
//! ([`stop_daemon`]): without it the suite leaks a daemon per test
//! and the caller's own post-drive store opens contend with the
//! handles the daemon still holds.

use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::EpicId;
use todo_domain::{
    BlocksEdge, Confidence, Date, DecisionDraft, DecisionStatus, DocDraft, EpicDraft, Priority,
    Reference, ReferenceKind, References, Repo, RepoSlug, Status, TaskDraft, TaskKind, Timestamp,
};
use todo_store::{DataRoot, Store};

/// How many differing line pairs the mismatch renderer shows at most —
/// bounded so a whole-file drift cannot flood the failure output.
// The cwd target drives no goldens, so this half is unreferenced
// there: allowed dead per item, the fixture half's own remedy.
#[allow(dead_code)]
const MAX_DIFF_LINES: usize = 3;
/// How many characters of each rendered diff line are shown at most.
#[allow(dead_code)]
const MAX_LINE_CHARS: usize = 200;

/// Resolves a golden's path from its subject `name` (the part before
/// the `.golden` suffix), anchored at the crate manifest directory —
/// never the process cwd, so the tests pass from any invocation root.
#[allow(dead_code)]
pub(crate) fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{name}.golden"))
}

/// Asserts `actual` is byte-identical to the golden `name`, or — under
/// `UPDATE_GOLDEN=1` — regenerates the golden and passes.
///
/// Regeneration writes through a temporary file in the golden directory
/// plus an atomic rename, so an interrupted update cannot leave a torn
/// golden. Deliberately no `fsync` (judgment call, t9): §4.3's
/// durability discipline binds the store's own writes; a golden lost to
/// a power failure is a regenerated test fixture, never a state of
/// record, and rename-atomicity is the property this helper owes.
///
/// Failure messages per the workflow contract: a missing golden names
/// the regeneration command; a mismatch names the path, both byte
/// lengths, the first differing byte offset, and a bounded line-level
/// diff of the first differing region.
#[allow(dead_code)]
pub(crate) fn assert_golden(name: &str, actual: &[u8]) {
    let path = golden_path(name);
    if update_requested() {
        write_golden(&path, actual);
        return;
    }
    let expected = match fs::read(&path) {
        Ok(expected) => expected,
        Err(err) if err.kind() == ErrorKind::NotFound => panic!(
            "golden file {} is missing; re-run this test with \
             UPDATE_GOLDEN=1 to write it",
            path.display()
        ),
        Err(err) => panic!("golden file {} could not be read: {err}", path.display()),
    };
    if expected != actual {
        panic!("{}", mismatch_report(&path, &expected, actual));
    }
}

/// Whether this run regenerates goldens instead of comparing them.
#[allow(dead_code)]
fn update_requested() -> bool {
    std::env::var("UPDATE_GOLDEN").is_ok_and(|value| value == "1")
}

/// Writes `actual` to `path` via temp+rename in the golden directory,
/// then reports what was written.
#[allow(dead_code)]
fn write_golden(path: &Path, actual: &[u8]) {
    let directory = path
        .parent()
        .expect("a golden path always has the golden directory as its parent");
    fs::create_dir_all(directory).expect("the golden directory is creatable");
    let file_name = path
        .file_name()
        .expect("a golden path always ends in its file name")
        .to_string_lossy();
    // The pid keeps parallel test binaries from sharing a temp name.
    let temporary = directory.join(format!(".{file_name}.{}.tmp", std::process::id()));
    fs::write(&temporary, actual).expect("the golden temp file is writable");
    fs::rename(&temporary, path).expect("rename within one directory is atomic");
    println!(
        "UPDATE_GOLDEN: wrote {} bytes to {}",
        actual.len(),
        path.display()
    );
}

/// The mismatch failure message: path, both byte lengths, the first
/// differing byte offset, and the bounded line-level diff.
#[allow(dead_code)]
fn mismatch_report(path: &Path, expected: &[u8], actual: &[u8]) -> String {
    let offset = first_difference(expected, actual);
    let mut report = format!(
        "golden mismatch: {}\n  expected {} bytes, got {} bytes\n  \
         first differing byte at offset {offset}",
        path.display(),
        expected.len(),
        actual.len()
    );
    report.push_str(&line_diff(expected, actual));
    report
}

/// The offset of the first differing byte; when one side is a prefix of
/// the other, the length of the shorter.
#[allow(dead_code)]
fn first_difference(expected: &[u8], actual: &[u8]) -> usize {
    expected
        .iter()
        .zip(actual)
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| expected.len().min(actual.len()))
}

/// A line-level diff of the two byte slices: walks line pairs in index
/// order (deterministic — nothing unordered is iterated), renders at
/// most [`MAX_DIFF_LINES`] differing pairs, truncates each rendered
/// line to [`MAX_LINE_CHARS`], and counts the remainder.
#[allow(dead_code)]
fn line_diff(expected: &[u8], actual: &[u8]) -> String {
    let expected_text = String::from_utf8_lossy(expected);
    let actual_text = String::from_utf8_lossy(actual);
    let expected_lines: Vec<&str> = expected_text.lines().collect();
    let actual_lines: Vec<&str> = actual_text.lines().collect();
    let mut diff = String::new();
    let mut shown = 0usize;
    let mut differing = 0usize;
    for index in 0..expected_lines.len().max(actual_lines.len()) {
        let left = expected_lines.get(index).copied();
        let right = actual_lines.get(index).copied();
        if left == right {
            continue;
        }
        differing += 1;
        if shown < MAX_DIFF_LINES {
            shown += 1;
            diff.push_str(&format!(
                "\n  line {}:\n    expected: {}\n    actual:   {}",
                index + 1,
                bounded(left.unwrap_or("<absent>")),
                bounded(right.unwrap_or("<absent>"))
            ));
        }
    }
    if differing > shown {
        diff.push_str(&format!(
            "\n  … and {} more differing line(s)",
            differing - shown
        ));
    }
    diff
}

/// A diff line, cut to [`MAX_LINE_CHARS`] characters with an ellipsis
/// marking the cut.
#[allow(dead_code)]
fn bounded(line: &str) -> String {
    if line.chars().count() <= MAX_LINE_CHARS {
        return line.to_owned();
    }
    let mut cut: String = line.chars().take(MAX_LINE_CHARS).collect();
    cut.push('…');
    cut
}

// =================================================================
// The shared suite helpers (lifted from read/query.rs, where a second
// verbatim copy was the cost the lift ends; the write suites import
// them alongside the read ones, so the sharing is every suite's, not
// the read ones' alone)
// =================================================================

/// One `tools/call` request line with a fixed id.
// The progress target (t68) drives no JSON-RPC at all, so these three
// drive-half helpers are unreferenced there: allowed dead per item,
// the drive variants' remedy.
#[allow(dead_code)]
pub(crate) fn call(id: u64, name: &str, arguments: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{arguments}}}}}"#
    )
}

/// The structuredContent of one response line, parsed.
#[allow(dead_code)]
pub(crate) fn structured(response: &str) -> Value {
    let value: Value = serde_json::from_str(response).expect("the response line is JSON");
    value["result"]["structuredContent"].clone()
}

/// Asserts one response is D15's carrier for `code`/`message`:
/// `isError: true`, the code in `structuredContent`, and the same JSON
/// mirrored into the text block.
#[allow(dead_code)]
pub(crate) fn assert_carrier(response: &str, code: &str, message: &str) {
    let value: Value = serde_json::from_str(response).expect("the response line is JSON");
    assert_eq!(value["result"]["isError"], json!(true), "{response}");
    assert_eq!(
        value["result"]["structuredContent"]["code"],
        json!(code),
        "{response}"
    );
    assert_eq!(
        value["result"]["structuredContent"]["message"],
        json!(message),
        "{response}"
    );
    let mirror = value["result"]["content"][0]["text"]
        .as_str()
        .expect("the text mirror");
    let mirrored: Value = serde_json::from_str(mirror).expect("the mirror is JSON");
    assert_eq!(mirrored, value["result"]["structuredContent"], "{response}");
}

// =================================================================
// The fixture store — the read and write suites' half. The import
// suite drives its own root, so every item in this section is
// unreferenced in that target: allowed dead per item here rather
// than split into a second helper file (one shared helper per crate,
// the module doc's rule).
// =================================================================

/// The fixture repo's slug — a fixture, not a product.
#[allow(dead_code)]
const FIXTURE: &str = "fixture";
/// The second repo's slug: it sorts before `fixture`, so the
/// all-repos scope's order `[alpha, fixture]` is visible and
/// non-trivial — registration order is the reverse.
#[allow(dead_code)]
const ALPHA: &str = "alpha";

/// Builds the fixture store under a fresh temp root and returns the
/// root; the `TempDir` guard keeps the directory alive for the drive,
/// and the builder drops every store handle before the server spawns
/// (the advisory lock is per process).
#[allow(dead_code)]
pub(crate) fn fixture() -> TempDir {
    let root = TempDir::new().expect("a fresh temp root");
    let data_root = DataRoot::new(root.path());
    build_fixture(&data_root);
    build_alpha(&data_root);
    root
}

/// A fixed canonical timestamp.
#[allow(dead_code)]
fn ts(value: &str) -> Timestamp {
    Timestamp::parse(value).expect("a fixed canonical timestamp")
}

/// A valid reference.
#[allow(dead_code)]
fn reference(kind: ReferenceKind, target: &str, note: Option<&str>) -> Reference {
    Reference::new(kind, target.to_owned(), note.map(str::to_owned)).expect("a valid reference")
}

/// The fixture graph, hand-built through the store's public API with
/// fixed caller-passed timestamps:
///
/// - **E1 "Bring-up"** (open) is the rich epic, gated by the open
///   decision D1: the status vocabulary (U1 open, U4 in_work, U5
///   blocked, U6 closed, U7 wontfix, U8 deferred), the two-run
///   `blocked_by` cases (U2: own edges plus an epic gate; U3: the same
///   decision gating both the task and its epic, the dedup case), and
///   a deferred task whose finished until-target still loses to the
///   epic gate (U8: not ready).
/// - **E2 "Runway"** (in_work) is ungated: eleven actionable tasks
///   (U9–U19) so the wire's default limit of ten is visible as a cap,
///   plus the deferred pair — U20 held (its until-target U9 is open),
///   U21 ready (its until-target U6 is closed). U9/U10/U11 carry
///   high/medium/low (D17) so the triage surfaces' priority echo pins
///   three distinct values beside the nulls.
/// - **D3** is signed and gates U10 — a finished decision predecessor
///   a listed row records; **S1** is a doc; U1↔U6 is a `relates`
///   annotation.
#[allow(dead_code)]
fn build_fixture(root: &DataRoot) {
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str(FIXTURE).expect("a valid slug"),
            name: "Fixture".to_owned(),
            path: "/repos/fixture".to_owned(),
            registered_at: ts("2026-10-01T08:00:00Z"),
        })
        .expect("the fixture repo registers");

    let bring_up = store
        .add_epic(
            EpicDraft {
                title: "Bring-up".to_owned(),
                description: "The sensor bring-up epic.".to_owned(),
                status: Status::Open,
                references: References::new(),
            },
            ts("2026-10-01T08:01:00Z"),
        )
        .expect("E1 lands");
    let runway = store
        .add_epic(
            EpicDraft {
                title: "Runway".to_owned(),
                description: "The actionable runway epic.".to_owned(),
                status: Status::InWork,
                references: References::new(),
            },
            ts("2026-10-01T08:02:00Z"),
        )
        .expect("E2 lands");

    // D1 gates the epic AND U3 directly — the dedup case: a task gated
    // by the same decision both ways lists it once, in the own-edges
    // run. D2 gates U2, whose epic D1 also gates — the two-run order:
    // the own-edges run lists D2 before the epic-gate run's D1 even
    // though D1 < D2, which a globally sorted list would not carry.
    let gate = store
        .add_decision(
            DecisionDraft {
                title: "Gate the bring-up epic".to_owned(),
                status: DecisionStatus::Open,
                options: vec!["(a) proceed".to_owned(), "(b) hold".to_owned()],
                recommendation: Some("(a)".to_owned()),
                resolution: None,
                confidence: Confidence::Medium,
                references: References::new(),
            },
            ts("2026-10-01T08:03:00Z"),
        )
        .expect("D1 lands");
    let second_gate = store
        .add_decision(
            DecisionDraft {
                title: "Second gate".to_owned(),
                status: DecisionStatus::Open,
                options: vec!["(a) ship".to_owned(), "(b) wait".to_owned()],
                recommendation: Some("(b)".to_owned()),
                resolution: None,
                confidence: Confidence::Low,
                references: References::new(),
            },
            ts("2026-10-01T08:04:00Z"),
        )
        .expect("D2 lands");
    let mut spec = References::new();
    spec.insert(reference(
        ReferenceKind::Spec,
        "todo-mcp-design.md §6",
        None,
    ))
    .expect("the spec reference lands");
    let signed = store
        .add_decision(
            DecisionDraft {
                title: "The encoder profile".to_owned(),
                status: DecisionStatus::Signed,
                options: vec!["(a) baseline".to_owned(), "(b) main".to_owned()],
                recommendation: Some("(a)".to_owned()),
                resolution: Some("signed (a) 2026-10-01".to_owned()),
                confidence: Confidence::High,
                references: spec,
            },
            ts("2026-10-01T08:05:00Z"),
        )
        .expect("D3 lands");

    store
        .add_doc(
            DocDraft {
                title: "Bring-up notes".to_owned(),
                path: "docs/bring-up.md".to_owned(),
                version: "1.0".to_owned(),
                references: References::new(),
            },
            ts("2026-10-01T08:06:00Z"),
        )
        .expect("S1 lands");

    // The add instant of task number `n`: 08:10:00 plus `n` seconds —
    // fixed, caller-passed, distinct per task.
    let at = |n: u64| ts(&format!("2026-10-01T08:10:{n:02}Z"));
    let draft = |epic: &EpicId, title: &str, kind: TaskKind, status: Status| TaskDraft {
        epic: epic.clone(),
        kind,
        title: title.to_owned(),
        body: String::new(),
        status,
        verify: Some("cargo test -p fixture".to_owned()),
        evidence: None,
        note: None,
        references: References::new(),
        priority: None,
        until: None,
    };

    let mut u1_references = References::new();
    u1_references
        .insert(reference(
            ReferenceKind::Tracker,
            "FIX-1",
            Some("the issue"),
        ))
        .expect("the tracker reference lands");
    u1_references
        .insert(reference(ReferenceKind::Doc, "fixture:S1#2", None))
        .expect("the doc reference lands");
    let u1 = store
        .add_task(
            TaskDraft {
                references: u1_references,
                priority: Some(Priority::High),
                until: Some(Date::parse("2026-10-15").expect("a valid date")),
                ..draft(
                    &bring_up.id,
                    "Wire the sensor",
                    TaskKind::Task,
                    Status::Open,
                )
            },
            at(1),
        )
        .expect("U1 lands");
    let u2 = store
        .add_task(
            draft(
                &bring_up.id,
                "Calibrate the rig",
                TaskKind::Gate,
                Status::Open,
            ),
            at(2),
        )
        .expect("U2 lands");
    let u3 = store
        .add_task(
            draft(
                &bring_up.id,
                "Validate the gate",
                TaskKind::Task,
                Status::Open,
            ),
            at(3),
        )
        .expect("U3 lands");
    store
        .add_task(
            draft(
                &bring_up.id,
                "Characterize the noise",
                TaskKind::Task,
                Status::InWork,
            ),
            at(4),
        )
        .expect("U4 lands");
    store
        .add_task(
            TaskDraft {
                note: Some("waiting on the vendor".to_owned()),
                ..draft(&bring_up.id, "Vendor fix", TaskKind::Task, Status::Blocked)
            },
            at(5),
        )
        .expect("U5 lands");
    let u6 = store
        .add_task(
            TaskDraft {
                evidence: Some("42 samples archived".to_owned()),
                ..draft(
                    &bring_up.id,
                    "Archive the samples",
                    TaskKind::Task,
                    Status::Closed,
                )
            },
            at(6),
        )
        .expect("U6 lands");
    store
        .add_task(
            draft(
                &bring_up.id,
                "Skip the telemetry pass",
                TaskKind::Task,
                Status::Wontfix,
            ),
            at(7),
        )
        .expect("U7 lands");
    let u8 = store
        .add_task(
            TaskDraft {
                note: Some("until the samples land".to_owned()),
                ..draft(
                    &bring_up.id,
                    "Revisit the archive format",
                    TaskKind::Task,
                    Status::Deferred,
                )
            },
            at(8),
        )
        .expect("U8 lands");

    let u9 = store
        .add_task(
            TaskDraft {
                priority: Some(Priority::High),
                ..draft(&runway.id, "Land the runway", TaskKind::Debt, Status::Open)
            },
            at(9),
        )
        .expect("U9 lands");
    let u10 = store
        .add_task(
            TaskDraft {
                priority: Some(Priority::Medium),
                ..draft(
                    &runway.id,
                    "Wire the encoder",
                    TaskKind::Finding,
                    Status::Open,
                )
            },
            at(10),
        )
        .expect("U10 lands");
    for step in 11..=19 {
        store
            .add_task(
                TaskDraft {
                    // U11's low completes the fixture's three distinct
                    // values (D17); the rest carry none.
                    priority: (step == 11).then_some(Priority::Low),
                    ..draft(
                        &runway.id,
                        &format!("Runway step {step}"),
                        TaskKind::Task,
                        Status::Open,
                    )
                },
                at(step),
            )
            .expect("the runway task lands");
    }
    let u20 = store
        .add_task(
            TaskDraft {
                note: Some("until the runway lands".to_owned()),
                ..draft(
                    &runway.id,
                    "Revisit after U9",
                    TaskKind::Task,
                    Status::Deferred,
                )
            },
            at(20),
        )
        .expect("U20 lands");
    let u21 = store
        .add_task(
            TaskDraft {
                note: Some("once the samples are archived".to_owned()),
                ..draft(
                    &runway.id,
                    "Revisit the archive",
                    TaskKind::Task,
                    Status::Deferred,
                )
            },
            at(21),
        )
        .expect("U21 lands");

    // Task edges: U1 holds back U2 and U3 (the task-scope batch's
    // rows); the deferred pair's until-targets — U6→U8 (finished, but
    // the epic gate holds U8 back), U9→U20 (open: U20 not ready),
    // U6→U21 (finished and ungated: U21 ready).
    for edge in [
        BlocksEdge::TaskToTask {
            from: u1.id.clone(),
            to: u2.id.clone(),
        },
        BlocksEdge::TaskToTask {
            from: u1.id.clone(),
            to: u3.id.clone(),
        },
        BlocksEdge::TaskToTask {
            from: u6.id.clone(),
            to: u8.id.clone(),
        },
        BlocksEdge::TaskToTask {
            from: u9.id.clone(),
            to: u20.id.clone(),
        },
        BlocksEdge::TaskToTask {
            from: u6.id.clone(),
            to: u21.id.clone(),
        },
    ] {
        store
            .link_blocks(edge, ts("2026-10-01T08:20:00Z"))
            .expect("the edge lands");
    }
    // Decision gates: D1 gates the epic and U3 (the dedup case), D2
    // gates U2 (the two-run case), D3 — signed — gates U10.
    for edge in [
        BlocksEdge::DecisionToEpic {
            from: gate.id.clone(),
            to: bring_up.id.clone(),
        },
        BlocksEdge::DecisionToTask {
            from: gate.id.clone(),
            to: u3.id.clone(),
        },
        BlocksEdge::DecisionToTask {
            from: second_gate.id.clone(),
            to: u2.id.clone(),
        },
        BlocksEdge::DecisionToTask {
            from: signed.id.clone(),
            to: u10.id.clone(),
        },
    ] {
        store
            .link_blocks(edge, ts("2026-10-01T08:21:00Z"))
            .expect("the gate lands");
    }
    // A relates annotation — never an ordering input (§5.1), present
    // so the fixture holds one.
    store
        .link_relates(u1.id, u6.id, ts("2026-10-01T08:22:00Z"))
        .expect("the annotation lands");

    // The builder's last act: drop the handle, releasing the advisory
    // lock before the server spawns.
    drop(store);
}

/// The fixture's second repo, **alpha** — minimal: one epic, one
/// task, no edges, decisions or docs. It exists to pin the
/// all-repos scope's membership and order: registered *after*
/// `fixture` and timestamped after it too, but sorted before it by
/// slug — so the pinned `[alpha, fixture]` order is the registry's
/// slug order, distinguishable from both arrival and timestamp
/// order. Every repo-scoped request is blind to it.
#[allow(dead_code)]
fn build_alpha(root: &DataRoot) {
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str(ALPHA).expect("a valid slug"),
            name: "Alpha".to_owned(),
            path: "/repos/alpha".to_owned(),
            registered_at: ts("2026-10-01T09:00:00Z"),
        })
        .expect("the alpha repo registers");
    let epic = store
        .add_epic(
            EpicDraft {
                title: "Alpha epic".to_owned(),
                description: String::new(),
                status: Status::Open,
                references: References::new(),
            },
            ts("2026-10-01T09:01:00Z"),
        )
        .expect("alpha E1 lands");
    store
        .add_task(
            TaskDraft {
                epic: epic.id,
                kind: TaskKind::Task,
                title: "Alpha task".to_owned(),
                body: String::new(),
                status: Status::Open,
                verify: Some("cargo test -p alpha".to_owned()),
                evidence: None,
                note: None,
                references: References::new(),
                priority: None,
                until: None,
            },
            ts("2026-10-01T09:02:00Z"),
        )
        .expect("alpha U1 lands");
    drop(store);
}

// =================================================================
// The drive
// =================================================================

/// The initialize request every drive opens with — protocol
/// 2025-11-25, the version this server declares. The clientInfo name
/// is not echoed, so it never reaches a golden.
const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t34","version":"0"}}}"#;

/// The handshake-completion notification — no response.
const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

/// How long [`stop_daemon`] waits for the SIGTERM'd daemon to be
/// gone: the drain bound (no connections live by the time the stop
/// runs — the shim's session ended with it) plus generous slack, so
/// a slow host does not flake.
const STOP_TIMEOUT: Duration = Duration::from_secs(15);

/// The pids of the daemons serving `root`: every process whose
/// command line carries the root path *and* `--daemon`. The temp
/// root is unique per test, so the root path discriminates this
/// test's daemon from every other's; the `--daemon` token keeps the
/// pattern from matching the shim itself (the shim's command line
/// has no `--daemon`). A zombie shows as `<defunct>` — its command
/// line is gone — so an exited-but-unreaped daemon reads as stopped,
/// which is the parent's (launchd's, or the test's own Child) reap
/// to finish.
pub(crate) fn daemon_pids(root: &Path) -> Vec<u32> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,command="])
        .output()
        .expect("ps runs");
    let text = String::from_utf8_lossy(&output.stdout);
    let root_text = root.as_os_str().to_string_lossy();
    text.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let (pid, command) = line.split_once(char::is_whitespace)?;
            if command.contains(root_text.as_ref()) && command.contains("--daemon") {
                pid.parse().ok()
            } else {
                None
            }
        })
        .collect()
}

/// Stops the daemon the shim left serving `root` (U66): SIGTERM —
/// the drain signal, `Child::kill` sends SIGKILL — every matching
/// pid, then wait for the process to be gone. Without this the suite
/// leaks a daemon per driven test, and the daemon's held store
/// handles contend with any post-drive store open the caller makes.
pub(crate) fn stop_daemon(root: &Path) {
    let deadline = Instant::now() + STOP_TIMEOUT;
    loop {
        let pids = daemon_pids(root);
        if pids.is_empty() {
            return;
        }
        for pid in &pids {
            let _ = Command::new("/bin/kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .status();
        }
        assert!(
            Instant::now() < deadline,
            "the daemon for {} did not exit within {:?}",
            root.display(),
            STOP_TIMEOUT
        );
        thread::sleep(Duration::from_millis(50));
    }
}

/// What one drive collected: the tool responses in request order, the
/// server's stderr and its exit status.
// The progress target (t68) drives nothing, so the collected fields
// are unread there: allowed dead, the drive half's remedy.
#[allow(dead_code)]
pub(crate) struct Driven {
    /// The tool responses, one line each, in request order.
    pub responses: Vec<String>,
    /// The server's stderr — empty on every green path: the shim
    /// prints nothing on a proxied session (the daemon's own
    /// diagnostics go to its null stdio, and the live-view URL line
    /// with them).
    pub stderr: String,
    /// The exit status — a completed session ends 0.
    pub status: ExitStatus,
}

/// Drives the real binary over raw stdio: the handshake, then one
/// request at a time — each response line is read before the next
/// request is written, so the response order is the request order by
/// construction (see the module docs for why that matters).
// The drive variants are each referenced by some subset of the four
// targets that compile this module (read, write, import, cwd):
// `drive_with_cwd` only by cwd, `drive`/`drive_with` by every one
// but cwd. Allowed dead per item, the fixture half's remedy.
#[allow(dead_code)]
pub(crate) fn drive(root: &Path, requests: &[String]) -> Driven {
    drive_with(root, None, requests)
}

/// [`drive`]'s variant with a default repo slug: the binary is
/// spawned with `--repo <slug>` (the committed D8's startup flag), so
/// the driven session sees the configured default — the tool calls
/// themselves decide whether to rely on it or pass an explicit repo,
/// which always wins.
#[allow(dead_code)]
pub(crate) fn drive_with(root: &Path, repo: Option<&RepoSlug>, requests: &[String]) -> Driven {
    drive_inner(root, repo, None, requests)
}

/// [`drive`]'s variant with a default repo slug and a working
/// directory (t53's third tier): the binary is spawned with
/// `--repo <slug>` when given and its cwd set to `cwd`, so the driven
/// session sees exactly what a client launched inside that checkout
/// sees — the flag outranks the cwd, an explicit parameter outranks
/// both, and the cwd's deterministic match (§7) supplies the default
/// when no flag is given.
// Only the cwd target drives with a working directory; the read,
// write and import targets compile this module without referencing
// this variant (the fixture half's allowed-dead precedent).
#[allow(dead_code)]
pub(crate) fn drive_with_cwd(
    root: &Path,
    repo: Option<&RepoSlug>,
    cwd: &Path,
    requests: &[String],
) -> Driven {
    drive_inner(root, repo, Some(cwd), requests)
}

/// The one drive body every variant shares: spawn, handshake, one
/// request at a time, collect. The variants differ only in what they
/// pass to [`spawn`].
fn drive_inner(
    root: &Path,
    repo: Option<&RepoSlug>,
    cwd: Option<&Path>,
    requests: &[String],
) -> Driven {
    let mut child = spawn(root, repo, cwd);
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

    let mut responses = Vec::with_capacity(requests.len());
    for request in requests {
        send(&mut stdin, request);
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
    // The shim leaves a live daemon per root (U66): stop it after the
    // session, so the suite leaks nothing and the caller's own
    // post-drive store opens find the locks released.
    stop_daemon(root);
    Driven {
        responses,
        stderr: stderr_text,
        status,
    }
}

/// Spawns the real binary with `--root`, the optional `--repo` and
/// the optional working directory — `None` inherits this process's
/// cwd, the pre-t53 behavior.
fn spawn(root: &Path, repo: Option<&RepoSlug>, cwd: Option<&Path>) -> Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_todo-mcp"));
    command.arg("--root").arg(root);
    if let Some(slug) = repo {
        command.arg("--repo").arg(slug.as_str());
    }
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    command
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
