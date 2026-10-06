//! The `import_md` wire suite (t44): the importer tool through the
//! protocol — request bytes in, response bytes out, the real binary
//! over stdio (§10), never by calling the handler around it. The
//! happy path drives a temp root whose repo points at a board tree
//! derived from the parsed corpus register — parse the corpus
//! in-process, write one matching file per key (the three board
//! files at their dialect paths, the rest as stubs), plus one stray
//! extension-less file named like a key, which the walk's `.md`
//! filter must ignore — so the tree self-maintains if the corpus
//! grows. The response is clock-free by design and
//! golden-pinned; the follow-up reads (`task_list`, `doc_list`,
//! `graph_show`) are asserted structurally against the mapping's own
//! in-process plan — the domain-agreement pattern, so a wire answer
//! that is deterministic but wrong cannot pass. The fault paths pin
//! §6.1's codes through D15's carrier: the apply refusal and the
//! board's own parse/map refusals as `invalid`, a missing board file
//! as `io`, an unknown repo as `unknown_id`, and rmcp's own codeless
//! argument fault for an unknown field.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{RepoSlug, Timestamp};
use todo_mcp::import::dacode::{
    DacodeBoard, ImportOptions, ImportPlan, PlanOp, map_board, parse_board,
};
use todo_store::{DataRoot, Store};

use super::board::{A_DELTAS, B_DECISIONS, SECTION_4, todo_md};
use crate::common::{assert_carrier, assert_golden, call, drive, drive_with, structured};

// =================================================================
// The board tree — derived from the parsed corpus register
// =================================================================

/// The board directory, relative to the repo's registered checkout
/// path — the dialect's measured layout, mirrored from the tool's
/// constant so the tree the tests assemble is the tree the tool
/// walks.
const BOARD_DIR: &str = "docs/implementation";

/// The three board files' dialect paths, relative to the board
/// directory — the measured layout the tool derives from the repo's
/// registered checkout path.
const BOARD_FILES: [&str; 3] = [
    "TODO.md",
    "appendices/B-decisions.md",
    "appendices/A-design-deltas.md",
];

/// The tool's stem rule, mirrored: a key matches a file whose stem is
/// the key, or the key plus a hyphen prefix.
fn stem_matches(file: &str, key: &str) -> bool {
    let name = file.rsplit_once('/').map_or(file, |(_, name)| name);
    let stem = name.strip_suffix(".md").unwrap_or(name);
    stem == key
        || stem
            .strip_prefix(key)
            .is_some_and(|rest| rest.starts_with('-'))
}

/// Builds the corpus board's tree under a fresh temp directory: the
/// three board files at their dialect paths, one stub `.md` per
/// register key the board files do not already match (the stem rule,
/// so the tree self-maintains if the corpus grows), and one stray
/// extension-less file named like a register key — the walk's `.md`
/// filter must ignore it, or it would collide with the key and
/// refuse the board. Returns the tree's root — the repo's registered
/// checkout path.
fn board_tree() -> TempDir {
    let root = TempDir::new().unwrap();
    let board = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).unwrap();
    let dir = root.path().join(BOARD_DIR);
    for (relative, contents) in [
        ("TODO.md", todo_md()),
        ("appendices/B-decisions.md", B_DECISIONS.to_owned()),
        ("appendices/A-design-deltas.md", A_DELTAS.to_owned()),
    ] {
        let path = dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    for entry in &board.todo.docs {
        // A key the board files already match needs no stub — writing
        // one would collide (the exact and prefix arms both fire).
        if BOARD_FILES
            .iter()
            .any(|file| stem_matches(file, &entry.key))
        {
            continue;
        }
        fs::write(dir.join(format!("{}.md", entry.key)), "stub").unwrap();
    }
    // The stray: `README` beside `README.md`, extension-less — a
    // non-`.md` file the filter drops; a dropped filter would collect
    // it and collide with the `README` key.
    fs::write(dir.join("README"), "not markdown").unwrap();
    root
}

/// The doc-path map the tool must derive from the tree `board_tree`
/// builds: each register key mapped to its repo-relative file — the
/// board file that matches it, or its stub.
fn expected_doc_paths(board: &DacodeBoard) -> BTreeMap<String, String> {
    board
        .todo
        .docs
        .iter()
        .map(|entry| {
            let matched = BOARD_FILES
                .iter()
                .find(|file| stem_matches(file, &entry.key));
            let file =
                matched.map_or_else(|| format!("{}.md", entry.key), |file| (*file).to_owned());
            (entry.key.clone(), format!("{BOARD_DIR}/{file}"))
        })
        .collect()
}

/// The plan the tool must reproduce, built in-process over the corpus
/// and the tree's derived doc paths — the oracle for the counts and
/// the task population. The `imported_at` instant is fixed here; the
/// tool's own read differs, and no count depends on it.
fn corpus_plan() -> ImportPlan {
    let board = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).unwrap();
    map_board(
        &board,
        &ImportOptions {
            slug: RepoSlug::from_str("dacode").unwrap(),
            imported_at: Timestamp::parse("2026-10-06T12:00:00Z").unwrap(),
            doc_paths: expected_doc_paths(&board),
        },
    )
    .unwrap()
}

/// A `repo_register` request's arguments for `slug` with its checkout
/// at `tree`.
fn register_args(slug: &str, tree: &Path) -> String {
    json!({
        "slug": slug,
        "name": slug,
        "path": tree.display().to_string(),
    })
    .to_string()
}

// =================================================================
// 1. The happy path: the golden and the follow-up reads.
// =================================================================

/// `import_md` over the corpus board: the response is golden-pinned
/// (clock-free by design), the counts agree with the mapping's own
/// plan, and the follow-up reads answer over the landed graph —
/// `task_list`'s count and first ids, `doc_list`'s register keys,
/// `graph_show`'s doc paths repo-relative and the created stamps
/// parsing, the one-read population sharing one value.
#[test]
fn import_md_imports_a_board_and_answers_the_landed_counts() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
        call(4, "task_list", r#"{"repo":"dacode"}"#),
        call(5, "doc_list", r#"{"repo":"dacode"}"#),
        call(6, "graph_show", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 5);
    // The premise (two identical failures would pass the golden
    // below): the register and the import both succeeded.
    let registered: Value = serde_json::from_str(&driven.responses[0]).unwrap();
    assert_eq!(registered["result"]["isError"], json!(false));
    let imported: Value = serde_json::from_str(&driven.responses[1]).unwrap();
    assert_eq!(
        imported["result"]["isError"],
        json!(false),
        "{}",
        driven.responses[1]
    );
    // The golden: import_md's response line, byte-pinned.
    assert_golden("import_md.v1", driven.responses[1].as_bytes());

    // The counts agree with the mapping's own plan — every op landed.
    let plan = corpus_plan();
    let answer = structured(&driven.responses[1]);
    assert_eq!(
        answer["epics"],
        json!(
            plan.ops
                .iter()
                .filter(|op| matches!(op, PlanOp::AddEpic { .. }))
                .count()
        )
    );
    assert_eq!(
        answer["decisions"],
        json!(
            plan.ops
                .iter()
                .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
                .count()
        )
    );
    assert_eq!(
        answer["docs"],
        json!(
            plan.ops
                .iter()
                .filter(|op| matches!(op, PlanOp::AddDoc { .. }))
                .count()
        )
    );
    assert_eq!(
        answer["tasks"],
        json!(
            plan.ops
                .iter()
                .filter(|op| matches!(op, PlanOp::AddTask { .. }))
                .count()
        )
    );
    assert_eq!(
        answer["blocks"],
        json!(
            plan.ops
                .iter()
                .filter(|op| matches!(op, PlanOp::LinkBlocks { .. }))
                .count()
        )
    );

    // task_list: the task count, then the first ids.
    let tasks = structured(&driven.responses[2]);
    let rows = tasks["tasks"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        plan.ops
            .iter()
            .filter(|op| matches!(op, PlanOp::AddTask { .. }))
            .count()
    );
    // The first ids: the D6 tiebreaker is visible — epic
    // registration order precedes task id, so the milestone epics'
    // substeps (U6–U8) come before the Follow-ups' U items, and U2
    // follows U1 (its blocked-on edge).
    let ids: Vec<&str> = rows.iter().map(|r| r["task"].as_str().unwrap()).collect();
    let first: Vec<&str> = ids.iter().take(5).copied().collect();
    assert_eq!(
        first,
        [
            "dacode:U6",
            "dacode:U7",
            "dacode:U8",
            "dacode:U1",
            "dacode:U2"
        ]
    );

    // doc_list: the register's keys, in register order (the docs were
    // added in it, and the listing is ascending by id).
    let board = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).unwrap();
    let docs = structured(&driven.responses[3]);
    let titles: Vec<&str> = docs["docs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| doc["title"].as_str().unwrap())
        .collect();
    let keys: Vec<&str> = board.todo.docs.iter().map(|d| d.key.as_str()).collect();
    assert_eq!(titles, keys);

    // graph_show: the doc paths repo-relative — exactly the derived
    // map, the board files' prefix arm and the stubs' exact arm both
    // visible.
    let graph = structured(&driven.responses[4]);
    let graph_docs = graph["repos"][0]["docs"].as_array().unwrap();
    let paths: BTreeMap<String, String> = graph_docs
        .iter()
        .map(|doc| {
            (
                doc["title"].as_str().unwrap().to_owned(),
                doc["path"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(paths, expected_doc_paths(&board));

    // D8's one clock read: every created stamp parses, and the
    // population the import stamped wholly from that one read — the
    // docs — shares one value. The B/A dossier nodes carry their
    // board dates (the mapping's timestamp convention), not the
    // clock, so equality is pinned over the clock-stamped population;
    // the first task in id order is a U item and carries the same
    // stamp.
    let doc_created: Vec<&str> = graph_docs
        .iter()
        .map(|doc| doc["created"].as_str().unwrap())
        .collect();
    for stamp in &doc_created {
        assert!(Timestamp::parse(stamp).is_ok(), "the stamp parses: {stamp}");
    }
    assert!(
        doc_created.iter().all(|stamp| *stamp == doc_created[0]),
        "one clock read: {doc_created:?}"
    );
    let all_tasks: Vec<&Value> = graph["repos"][0]["epics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["tasks"].as_array().unwrap())
        .collect();
    for task in &all_tasks {
        assert!(
            Timestamp::parse(task["created"].as_str().unwrap()).is_ok(),
            "the stamp parses: {}",
            task["created"]
        );
    }
    let u1 = all_tasks
        .iter()
        .find(|task| task["id"] == json!("dacode:U1"))
        .unwrap();
    assert_eq!(u1["created"].as_str().unwrap(), doc_created[0]);
}

// =================================================================
// 2–7. The fault paths, each through the carrier.
// =================================================================

/// A second import into the same repo is refused — the store is not
/// empty — and the refusal carries `invalid`: a client-fixable
/// precondition, not corruption.
#[test]
fn a_second_import_into_the_same_repo_is_refused_as_invalid() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
        call(4, "import_md", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 3);
    // The premise: the first import succeeded.
    let first: Value = serde_json::from_str(&driven.responses[1]).unwrap();
    assert_eq!(first["result"]["isError"], json!(false));
    assert_carrier(
        &driven.responses[2],
        "invalid",
        "the import into 'dacode' was refused: the store is not empty or has burned ids",
    );
}

/// An unknown repo carries `unknown_id` — the resolve path every
/// repo-scoped tool shares.
#[test]
fn an_unknown_repo_is_unknown_id() {
    let root = TempDir::new().unwrap();
    let driven = drive(root.path(), &[call(2, "import_md", r#"{"repo":"nosuch"}"#)]);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 1);
    assert_carrier(&driven.responses[0], "unknown_id", "unknown id: nosuch");
}

/// A malformed board file — a TODO.md missing §4, built from the
/// corpus sections — carries `invalid` with the parser's own Display.
#[test]
fn a_malformed_board_file_is_invalid() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    let malformed = todo_md().replace(SECTION_4, "");
    fs::write(tree.path().join(BOARD_DIR).join("TODO.md"), &malformed).unwrap();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 2);
    let line = malformed.split('\n').count();
    assert_carrier(
        &driven.responses[1],
        "invalid",
        &format!("line {line}: TODO.md is missing section §4"),
    );
}

/// A missing board file — no TODO.md under the registered path —
/// carries `io` with the attempt message naming the file's role,
/// never a path.
#[test]
fn a_missing_board_file_is_io() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    fs::remove_file(tree.path().join(BOARD_DIR).join("TODO.md")).unwrap();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 2);
    assert_carrier(
        &driven.responses[1],
        "io",
        "io: read the dacode board file TODO.md: NotFound",
    );
}

/// A doc key with no matching file — the tree lacks one register
/// key's stub — carries `invalid` with `MissingDocPath`'s Display
/// naming the key: the mapping's refusal, not the walk's.
#[test]
fn a_doc_key_with_no_matching_file_is_invalid() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    let board = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).unwrap();
    // The first key the board files do not match — its stub is the
    // file to remove.
    let key = board
        .todo
        .docs
        .iter()
        .map(|entry| entry.key.as_str())
        .find(|key| !BOARD_FILES.iter().any(|file| stem_matches(file, key)))
        .unwrap();
    fs::remove_file(tree.path().join(BOARD_DIR).join(format!("{key}.md"))).unwrap();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 2);
    assert_carrier(
        &driven.responses[1],
        "invalid",
        &format!("the doc register key '{key}' has no path in the import options"),
    );
}

/// A doc key matching more than one file — a second file beside the
/// key's stub, sharing the key's hyphen prefix — carries `invalid`
/// naming the key and the first two repo-relative paths, sorted: a
/// tree contradiction is refused, never first-wins.
#[test]
fn a_doc_key_matching_two_files_is_invalid() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    let board = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).unwrap();
    let key = board
        .todo
        .docs
        .iter()
        .map(|entry| entry.key.as_str())
        .find(|key| !BOARD_FILES.iter().any(|file| stem_matches(file, key)))
        .unwrap();
    fs::write(
        tree.path().join(BOARD_DIR).join(format!("{key}-extra.md")),
        "stub",
    )
    .unwrap();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 2);
    // The files are byte-sorted, so `{key}-extra.md` (`-` sorts
    // before `.`) precedes `{key}.md`.
    assert_carrier(
        &driven.responses[1],
        "invalid",
        &format!(
            "the doc key '{key}' matches more than one file: {BOARD_DIR}/{key}-extra.md and {BOARD_DIR}/{key}.md"
        ),
    );
}

// =================================================================
// 8–9. The default repo and the argument fault.
// =================================================================

/// The default repo (the committed D8): an absent `repo` parameter
/// imports into the configured default; an explicit parameter wins
/// over it (the default's store is already imported, so the default
/// losing would refuse); and with no default configured the absent
/// parameter is `invalid` — `repo_of`'s own arm.
#[test]
fn the_default_repo_imports_when_the_param_is_absent() {
    let root = TempDir::new().unwrap();
    let tree_dacode = board_tree();
    let tree_other = board_tree();
    let default = RepoSlug::from_str("dacode").unwrap();
    let driven = drive_with(
        root.path(),
        Some(&default),
        &[
            call(
                2,
                "repo_register",
                &register_args("dacode", tree_dacode.path()),
            ),
            call(3, "import_md", "{}"),
            call(
                4,
                "repo_register",
                &register_args("other", tree_other.path()),
            ),
            call(5, "import_md", r#"{"repo":"other"}"#),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 4);
    // The absent parameter imported into the default.
    let answer = structured(&driven.responses[1]);
    assert_eq!(answer["repo"], json!("dacode"));
    // The explicit parameter won over the configured default — the
    // default's store is non-empty now, so the default losing would
    // have refused.
    let answer = structured(&driven.responses[3]);
    assert_eq!(answer["repo"], json!("other"));

    // No default configured: the absent parameter is `invalid`.
    let bare = TempDir::new().unwrap();
    let driven = drive(bare.path(), &[call(2, "import_md", "{}")]);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 1);
    assert_carrier(
        &driven.responses[0],
        "invalid",
        "the repo parameter is absent and no default repo is configured",
    );
}

/// An unknown argument field is rmcp's own argument fault —
/// tool-level `isError`, unstructured text, no code — because
/// `deny_unknown_fields` turned the silent drop into a rejection
/// before the tool body ran; and nothing was imported.
#[test]
fn an_unknown_argument_field_is_a_codeless_argument_fault() {
    let root = TempDir::new().unwrap();
    let tree = board_tree();
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode","path":"/nowhere"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 2);
    let response = &driven.responses[1];
    assert!(response.contains(r#""isError":true"#), "{response}");
    assert!(
        !response.contains("structuredContent"),
        "an argument fault carries no code: {response}"
    );
    assert!(
        response.contains("failed to deserialize parameters"),
        "{response}"
    );
    assert!(response.contains("unknown field `path`"), "{response}");
    // The rejection happened before the tool body: the registered
    // store is still empty (the server has exited, its locks
    // released).
    let store = DataRoot::new(root.path())
        .open(&RepoSlug::from_str("dacode").unwrap())
        .unwrap();
    let graph = store.graph();
    assert_eq!(
        graph.epics().count()
            + graph.tasks().count()
            + graph.decisions().count()
            + graph.docs().count(),
        0,
        "nothing was imported"
    );
}
