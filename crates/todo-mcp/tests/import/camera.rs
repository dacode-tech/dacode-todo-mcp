//! The camera dialect's suite (t46): the parser (TODO.md's walker,
//! decisions.md's table and dossiers), the strictness and
//! never-fired-error pins (duplicates, absent registers, column
//! counts, table ids, impossible dates, wrapped rows), the mapping
//! (statuses, kinds, the synthetics, the edges, the oddball
//! references, the findings), the plan order and id reproduction at
//! synthetic scale, the `dialect` param's wire drive, and
//! determinism. All corpora are hand-authored synthetic strings in
//! the measured dialect — the live camera board is the import probe's
//! subject (D12), never copied here (D12): no live prose, no golden
//! pinning it.
//!
//! The wire test drives the real binary over a synthetic camera tree
//! and asserts the counts response and spot ids in-test — no golden
//! (D12's signed consequence: two of the three dialects get no pinned
//! golden) — plus the dispatch and fault pins: an unknown dialect
//! value is `invalid` naming it, the absent dialect takes the dacode
//! path (the t44 suite pins the default's happy path; this pins the
//! dispatch from the camera side), and each dialect's missing-file io
//! fault names its own file's role.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{Repo, RepoSlug, Timestamp};
use todo_mcp::import::camera::{
    CameraError, Dependency, DependencyKind, MilestoneState, map_camera, parse_camera_board,
    parse_decisions, parse_todo_md,
};
use todo_mcp::import::dacode::{ImportOptions, PlanOp, apply_plan};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, call, drive, structured};

// =================================================================
// The corpora — a synthetic minimal camera board composed of section
// pieces, so the error-path corpora can swap one section out without
// duplicating the rest. §1, §3 and §6 are walked past, so their
// content exists to prove exactly that.
// =================================================================

/// The intro and §1: the blocked-on-user mirror, walked past — its
/// table is not a decisions register (decisions.md is authoritative).
const HEAD: &str = "\
# TODO — synthetic

Intro prose, walked past.

## 1. Blocked on the user

A mirror of the open decisions, kept for readability only; the
decisions file is the authority. Walked past.

| ID | Decision | Gates |
|---|---|---|
| D2 | A mirror row | M1 |
";

/// §2: the milestone table, one row per State-cell spelling.
const SECTION_2: &str = "\
## 2. Milestones

| Milestone | State | Exit criterion |
|---|---|---|
| M0 — groundwork | in progress | the exit prose |
| M1 — the engine | **done — closed 2026-09-24.** Exit criterion met early | the exit prose |
| M2 — the interface | not started | — |
";

/// §3: the delivery log, walked past — its UB id is not a register
/// item.
const SECTION_3: &str = "\
## 3. Done

- **2026-09-18** — **UB1 · a delivery log entry.** The delivery log is
  walked past, and its UB id is not a register item.
";

/// §4: the follow-ups register — every shape the grammar accepts: a
/// plain item, a checked item with a trailing period, a struck item,
/// a wrapped bold run with an indented suffixed child, the struck
/// `(original)` variant, the wrapped `Held back by` phrase, and prose
/// whose id does not follow the phrase.
const SECTION_4: &str = "\
## 4. Follow-ups

Synthetic follow-ups, hand-authored for the grammar.

- [ ] **U1 · First follow-up** — body prose, no dependency.
- [x] **U2 · Second follow-up.** Checked, with a trailing period.
- [x] ~~**U3 · A struck item**~~ — struck and checked, closed prose.
- [ ] **U4 · A wrapped title that continues
  on a second line** — the bold run wraps.
  - [x] **U4a · An indented oddball** — the suffixed child.
- [x] ~~**U4 (original) · a variant item**~~ — the struck variant.
- [ ] **U5 · Held back** — prose. **Held back
  by U2**: the wrapped phrase.
- [ ] **U6 · Prose only** — this item sat blocked on the console
  (U1) for weeks, and the id does not follow the phrase.
";

/// §5: the debt register.
const SECTION_5: &str = "\
## 5. Debt

Synthetic debt entries, recorded rather than resolved.

- [ ] **U7 · A debt item** — debt prose.
- [x] **U8 · A checked debt item** — more debt prose.
";

/// §6: the session narrative, walked past — its `blocked on` prose is
/// outside the registers and never a dependency (the D18 precedent).
const SECTION_6: &str = "\
## 6. Session narrative

The narrative is walked past. Its prose names ids — this run is
blocked on U99 — and none of them becomes a dependency.
";

/// The full synthetic camera TODO.md.
fn todo_md() -> String {
    format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}")
}

/// The synthetic camera decisions.md, aligned with the corpus's
/// milestones: three rows exercising the confidence spellings and
/// both status spellings, three dossiers (one without the RESOLVED
/// marker — the table is the authority).
const DECISIONS_MD: &str = "\
# Decisions

Prose intro, walked past.

## Sign-off summary

Prose between the header and the table, walked past.

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| ~~**D1**~~ | First decision | take the simple path | low | M0 | resolved 2026-09-18 — the simple path taken |
| **D2** | Second decision | gather the numbers first | medium | M1 | open |
| ~~**D3**~~ | Third decision | hold | high | M2 | resolved 2026-09-20 — held as recommended |

## Detail

### D1 — First decision — RESOLVED 2026-09-18

Dossier body for D1.

### D2 — Second decision

Dossier body for D2.

### D3 — Third decision — RESOLVED 2026-09-20

Dossier body for D3.
";

/// The import options the mapping tests share: a fixed slug and
/// stamp, an empty `doc_paths` (camera has no doc register).
fn opts() -> ImportOptions {
    ImportOptions {
        slug: RepoSlug::from_str("camera").unwrap(),
        imported_at: Timestamp::parse("2026-10-06T12:00:00Z").unwrap(),
        doc_paths: std::collections::BTreeMap::new(),
    }
}

/// The corpus board, parsed and composed.
fn corpus_board() -> todo_mcp::import::camera::CameraBoard {
    parse_camera_board(&todo_md(), DECISIONS_MD).unwrap()
}

/// The 1-based line number of the first line containing `needle` in
/// `corpus` — derived from the corpus itself so the corpora's blank
/// lines cannot desynchronize a line assertion.
fn line_of(corpus: &str, needle: &str) -> usize {
    corpus.split('\n').position(|l| l.contains(needle)).unwrap() + 1
}

// =================================================================
// 1. The board parser.
// =================================================================

/// §4/§5 register extraction (ids, titles, checkbox states, struck
/// shapes, the wrapped bold run, the wrapped dependency phrase), §2's
/// milestone rows, §1/§3/§6 walked past, UB ids not a register, and
/// the required sections all present.
#[test]
fn the_board_parser_extracts_the_registers_and_walks_the_prose_past() {
    let board = parse_todo_md(&todo_md()).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // §2: the milestone rows, in board order, with the three State
    // spellings.
    let milestones: Vec<(&str, &str, MilestoneState)> = board
        .milestones
        .iter()
        .map(|row| (row.id.as_str(), row.title.as_str(), row.status))
        .collect();
    assert_eq!(
        milestones,
        vec![
            ("M0", "groundwork", MilestoneState::InProgress),
            ("M1", "the engine", MilestoneState::DoneClosed),
            ("M2", "the interface", MilestoneState::NotStarted),
        ]
    );

    // §4: the items in board order — the indented child is a register
    // item, not body text.
    let followups: Vec<(&str, &str, bool, bool)> = board
        .followups
        .iter()
        .map(|item| {
            (
                item.id.as_str(),
                item.title.as_str(),
                item.checked,
                item.struck,
            )
        })
        .collect();
    assert_eq!(
        followups,
        vec![
            ("U1", "First follow-up", false, false),
            ("U2", "Second follow-up.", true, false),
            ("U3", "A struck item", true, true),
            (
                "U4",
                "A wrapped title that continues on a second line",
                false,
                false
            ),
            ("U4a", "An indented oddball", true, false),
            ("U4 (original)", "a variant item", true, true),
            ("U5", "Held back", false, false),
            ("U6", "Prose only", false, false),
        ]
    );

    // The raw body carries the wrap verbatim; the wrapped bold run
    // joined into one title.
    let u4 = &board.followups[3];
    assert_eq!(
        u4.body,
        "**U4 · A wrapped title that continues\n  on a second line** — the bold run wraps."
    );
    // The indented child ends the parent's span: U4's body is the
    // item line and the wrap line only.
    assert!(!u4.body.contains("An indented oddball"));

    // The wrapped `Held back by` phrase is one dependency, its text
    // the matched phrase; U6's non-following id is none.
    assert_eq!(
        board.followups[6].dependencies,
        vec![Dependency {
            kind: DependencyKind::HeldBackBy,
            id: "U2".to_owned(),
            text: "Held back by U2".to_owned(),
        }]
    );
    assert_eq!(board.followups[7].dependencies, vec![]);

    // §5: the debt register.
    let debt: Vec<(&str, bool)> = board
        .debt
        .iter()
        .map(|item| (item.id.as_str(), item.checked))
        .collect();
    assert_eq!(debt, vec![("U7", false), ("U8", true)]);

    // §1/§3/§6 walked past: no decision register in the TODO AST (the
    // mirror row is prose), no UB item, and no U99 dependency anywhere
    // — the narrative's `blocked on` never fires.
    assert_eq!(
        board
            .followups
            .iter()
            .chain(board.debt.iter())
            .flat_map(|item| item.dependencies.iter())
            .count(),
        1,
        "only U5's held-back-by dependency exists"
    );
}

// =================================================================
// 2. The strictness.
// =================================================================

/// A malformed register line and an absent §4 each a named error with
/// the 1-based line; the struck-unchecked combination, the unclosed
/// bold run and an unknown State spelling are named errors too.
#[test]
fn a_malformed_register_line_and_an_absent_section_are_named_errors() {
    // A malformed register line: the bold run's id is not U-shaped.
    let corpus = todo_md().replace(
        "- [ ] **U1 · First follow-up** — body prose, no dependency.",
        "- [ ] **X1 · Not a U id** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::MalformedLine {
            line: line_of(&corpus, "**X1 ·"),
            detail: "the bold run is not U-shaped: 'X1' is not a U id".to_owned(),
        })
    );

    // An absent §4: the named missing-section error, the line the walk
    // ended on.
    let corpus = format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_5}{SECTION_6}");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::MissingSection {
            section: "§4".to_owned(),
            line: corpus.split('\n').count(),
        })
    );

    // A struck item that is unchecked: the strike is confirmation of a
    // checked item, never a state of its own.
    let corpus = todo_md().replace(
        "- [x] ~~**U3 · A struck item**~~ — struck and checked, closed prose.",
        "- [ ] ~~**U3 · A struck item**~~ — struck but unchecked.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::StruckUnchecked {
            line: line_of(&corpus, "struck but unchecked"),
        })
    );

    // An unclosed bold run before the next item: a named error, not a
    // title polluted by the following register construct.
    let corpus = todo_md().replace(
        "- [ ] **U1 · First follow-up** — body prose, no dependency.",
        "- [ ] **U1 · Title with no close\n- [ ] **U2 · Next item.** body",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::MalformedLine {
            line: line_of(&corpus, "**U1 · Title"),
            detail: "a U item's bold run does not close".to_owned(),
        })
    );

    // An unknown State-cell spelling.
    let corpus = todo_md().replace(
        "| M0 — groundwork | in progress |",
        "| M0 — groundwork | paused |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::BadStatus {
            line: line_of(&corpus, "| M0 — groundwork | paused |"),
            text: "paused".to_owned(),
            detail:
                "the State cell must read 'in progress', 'not started', or 'done — closed <date>'"
                    .to_owned(),
        })
    );
}

/// A struck item whose strike does not close — `~~**…**` with no
/// closing `~~` — still parses as struck: the live board carries two
/// such items (the measured grammar behind the widening), and the
/// strike's semantic content is the leading marker alone.
#[test]
fn an_unclosed_strike_still_parses_as_struck() {
    let corpus = todo_md().replace(
        "- [x] ~~**U3 · A struck item**~~ — struck and checked, closed prose.",
        "- [x] ~~**U3 · A struck item** — the strike never closes.",
    );
    let board = parse_todo_md(&corpus).unwrap();
    let item = &board.followups[2];
    assert_eq!(item.id, "U3");
    assert!(item.struck, "the leading marker alone is the strike");
    assert!(item.checked);
    assert!(
        item.body.starts_with("~~**U3 · A struck item**"),
        "the raw body carries the unclosed strike verbatim"
    );
}

/// A duplicated register id is a named error naming both lines — the
/// board's ids are unique, and a silent second entry would
/// double-import one item. The U namespace spans §4 and §5 (dacode's
/// twin scope is one register; camera's is two), so the duplicate is
/// named whether it lands twice in one section or once in each.
#[test]
fn a_duplicate_register_id_is_a_named_error_naming_both_lines() {
    // Twice within §4.
    let corpus = todo_md().replace("- [ ] **U6 · Prose only**", "- [ ] **U2 · A duplicate id**");
    let first = line_of(&corpus, "**U2 · Second follow-up");
    let second = line_of(&corpus, "**U2 · A duplicate id**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::DuplicateId {
            id: "U2".to_owned(),
            first,
            second,
        })
    );

    // Once in §4 and once in §5: the cross-section duplicate — the
    // id map is the walker's, shared across both registers.
    let corpus = todo_md().replace(
        "- [ ] **U7 · A debt item** — debt prose.",
        "- [ ] **U2 · A cross-section duplicate** — debt prose.",
    );
    let first = line_of(&corpus, "**U2 · Second follow-up");
    let second = line_of(&corpus, "**U2 · A cross-section duplicate**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::DuplicateId {
            id: "U2".to_owned(),
            first,
            second,
        })
    );
}

/// Absent registers, wrong column counts and malformed table ids are
/// named errors: §2 without its milestones table, the decisions file
/// without its summary table, a §2 row and a decisions row with the
/// wrong cell count, and a Milestone, ID or Gates cell whose id is not
/// the shape its register requires.
#[test]
fn absent_registers_and_malformed_table_rows_are_named_errors() {
    // §2 without its milestones table: the register is required — its
    // absence would silently omit every milestone.
    let corpus = format!(
        "{HEAD}## 2. Milestones\n\nNo table here, only prose.\n\n{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}"
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: corpus.split('\n').count(),
        })
    );

    // The decisions file without its summary table.
    let decisions = "# Decisions\n\nNo table, only prose.\n\n### D1 — First decision\n\nBody.\n";
    assert_eq!(
        parse_decisions(decisions),
        Err(CameraError::MissingRegister {
            register: "the decisions summary table".to_owned(),
            line: decisions.split('\n').count(),
        })
    );

    // A §2 row with the wrong cell count.
    let corpus = todo_md().replace(
        "| M2 — the interface | not started | — |",
        "| M2 — the interface | not started | — | extra |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::ColumnCount {
            line: line_of(&corpus, "| extra |"),
            expected: 3,
            found: 4,
        })
    );

    // A decisions row with the wrong cell count.
    let decisions = DECISIONS_MD.replace(
        "| **D2** | Second decision | gather the numbers first | medium | M1 | open |",
        "| **D2** | Second decision | gather the numbers first | medium | M1 |",
    );
    assert_eq!(
        parse_decisions(&decisions),
        Err(CameraError::ColumnCount {
            line: line_of(&decisions, "| **D2** | Second decision"),
            expected: 6,
            found: 5,
        })
    );

    // A Milestone cell whose id is not M-shaped.
    let corpus = todo_md().replace(
        "| M2 — the interface | not started | — |",
        "| X2 — the interface | not started | — |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(CameraError::BadId {
            line: line_of(&corpus, "| X2 — the interface"),
            id: "X2".to_owned(),
            detail: "not an M id".to_owned(),
        })
    );

    // A decisions ID cell that is not a D id.
    let decisions = DECISIONS_MD.replace(
        "| ~~**D1**~~ | First decision |",
        "| ~~**X1**~~ | First decision |",
    );
    assert_eq!(
        parse_decisions(&decisions),
        Err(CameraError::BadId {
            line: line_of(&decisions, "| ~~**X1**~~"),
            id: "X1".to_owned(),
            detail: "not a D id".to_owned(),
        })
    );

    // A Gates cell that is not an M id.
    let decisions = DECISIONS_MD.replace("| hold | high | M2 |", "| hold | high | D2 |");
    assert_eq!(
        parse_decisions(&decisions),
        Err(CameraError::BadId {
            line: line_of(&decisions, "| hold | high | D2 |"),
            id: "D2".to_owned(),
            detail: "not an M id".to_owned(),
        })
    );
}

/// A table row wrapped across two lines joins with a single space and
/// parses — the dacode twin's pin, over §2's milestone row and the
/// decisions summary row (the shared collector serves both tables).
#[test]
fn a_wrapped_table_row_joins_with_a_single_space() {
    let wrapped_todo = todo_md().replace(
        "| M2 — the interface | not started | — |",
        "| M2 — the interface | not\nstarted | — |",
    );
    let unwrapped = parse_todo_md(&todo_md()).unwrap();
    let wrapped = parse_todo_md(&wrapped_todo).unwrap();
    assert_eq!(wrapped.milestones, unwrapped.milestones);

    let wrapped_decisions = DECISIONS_MD.replace(
        "| **D2** | Second decision | gather the numbers first | medium | M1 | open |",
        "| **D2** | Second decision | gather the\nnumbers first | medium | M1 | open |",
    );
    let unwrapped_file = parse_decisions(DECISIONS_MD).unwrap();
    let wrapped_file = parse_decisions(&wrapped_decisions).unwrap();
    assert_eq!(wrapped_file.summary, unwrapped_file.summary);
}

// =================================================================
// 3. The decisions parser.
// =================================================================

/// The summary table's cells (title, recommendation, confidence,
/// gates, status), the dossiers' headers, and the table/dossier id
/// cross-check — a mismatch a named error, an unknown confidence and
/// an empty Recommendation cell named errors too.
#[test]
fn the_decisions_parser_reads_the_table_and_cross_checks_the_dossiers() {
    let file = parse_decisions(DECISIONS_MD).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // The summary rows, in board order, every cell parsed.
    assert_eq!(file.summary.len(), 3);
    assert_eq!(file.summary[0].id, "D1");
    assert_eq!(file.summary[0].title, "First decision");
    assert_eq!(file.summary[0].recommendation, "take the simple path");
    assert_eq!(
        file.summary[0].confidence,
        todo_mcp::import::camera::ConfidenceCell::Low
    );
    assert_eq!(file.summary[0].gates, "M0");
    assert_eq!(
        file.summary[0].status,
        todo_mcp::import::camera::DecisionStatusCell::Resolved {
            date: "2026-09-18".to_owned(),
            outcome: "the simple path taken".to_owned(),
        }
    );
    assert_eq!(
        file.summary[1].status,
        todo_mcp::import::camera::DecisionStatusCell::Open
    );
    assert_eq!(file.summary[2].gates, "M2");

    // The dossiers: ids, titles, the RESOLVED marker, bodies.
    assert_eq!(file.dossiers.len(), 3);
    assert_eq!(file.dossiers[0].id, "D1");
    assert_eq!(file.dossiers[0].title, "First decision");
    assert_eq!(file.dossiers[0].resolved.as_deref(), Some("2026-09-18"));
    assert_eq!(file.dossiers[1].resolved, None);
    assert!(file.dossiers[2].body.contains("Dossier body for D3."));

    // The composition's cross-check passes on the aligned corpus.
    assert!(parse_camera_board(&todo_md(), DECISIONS_MD).is_ok());

    // A dossier the table does not carry: the named mismatch, the
    // smallest offending id.
    let extra = format!("{DECISIONS_MD}### D4 — Fourth decision\n\nDossier body for D4.\n");
    assert_eq!(
        parse_camera_board(&todo_md(), &extra),
        Err(CameraError::IdSetMismatch {
            id: "D4".to_owned(),
            detail: "D4 is in the decisions dossiers but not in the decisions summary table"
                .to_owned(),
        })
    );

    // An unknown confidence spelling.
    let bad_confidence = DECISIONS_MD.replace("| low | M0 |", "| med | M0 |");
    assert_eq!(
        parse_decisions(&bad_confidence),
        Err(CameraError::BadStatus {
            line: line_of(&bad_confidence, "| med | M0 |"),
            text: "med".to_owned(),
            detail: "the Confidence cell must read 'low', 'medium', or 'high'".to_owned(),
        })
    );

    // An empty Recommendation cell.
    let bad_recommendation =
        DECISIONS_MD.replacen("| take the simple path | low | M0 |", "|  | low | M0 |", 1);
    assert_eq!(
        parse_decisions(&bad_recommendation),
        Err(CameraError::MalformedLine {
            line: line_of(&bad_recommendation, "|  | low | M0 |"),
            detail: "the Recommendation cell is empty".to_owned(),
        })
    );
}

// =================================================================
// 4. The mapping.
// =================================================================

/// The mapping's conventions: the task statuses (checked, unchecked,
/// struck), the epic statuses from the State cells, the decision
/// statuses (resolved → Signed + resolution, open → Open), the debt
/// kind for §5, the two synthetics, the Gates edges, the
/// held-back-by edge, the oddball references, and the plan's total
/// order.
#[test]
fn the_mapping_pins_statuses_kinds_edges_and_the_oddball_references() {
    let board = corpus_board();
    let plan = map_camera(&board, &opts()).unwrap();
    let ops = &plan.ops;

    // The plan's total order: epics, decisions, tasks, edges.
    let epic_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddEpic { .. }))
        .count();
    let decision_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
        .count();
    let task_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddTask { .. }))
        .count();
    let edge_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::LinkBlocks { .. }))
        .count();
    assert_eq!(
        (epic_count, decision_count, task_count, edge_count),
        (5, 3, 10, 4)
    );
    let first_edge = ops
        .iter()
        .position(|op| matches!(op, PlanOp::LinkBlocks { .. }))
        .unwrap();
    assert_eq!(first_edge, epic_count + decision_count + task_count);

    // The epic ops: the milestones' statuses from the State cells,
    // then the two synthetics.
    for (i, op) in ops.iter().enumerate() {
        let PlanOp::AddEpic { draft, id, .. } = op else {
            continue;
        };
        match id.to_string().as_str() {
            "camera:E1" => {
                assert_eq!(draft.title, "groundwork");
                assert_eq!(draft.status, todo_domain::Status::InWork);
                assert_eq!(draft.description, "the camera board's §2 milestone M0");
            }
            "camera:E2" => {
                assert_eq!(draft.title, "the engine");
                assert_eq!(draft.status, todo_domain::Status::Closed);
            }
            "camera:E3" => {
                assert_eq!(draft.title, "the interface");
                assert_eq!(draft.status, todo_domain::Status::Open);
            }
            "camera:E4" => {
                assert_eq!(draft.title, "Follow-ups");
                assert_eq!(draft.status, todo_domain::Status::Open);
                assert_eq!(
                    draft.description,
                    "the camera board's §4 follow-ups, not tied to a milestone"
                );
            }
            "camera:E5" => {
                assert_eq!(draft.title, "Debt");
                assert_eq!(draft.status, todo_domain::Status::Open);
                assert_eq!(
                    draft.description,
                    "the camera board's §5 debt, recorded rather than forgotten"
                );
            }
            other => panic!("unexpected epic id {other} at op {i}"),
        }
    }

    // The decision ops: resolved → Signed with the outcome as the
    // resolution and the date as the stamp; open → Open at the import
    // stamp; the recommendation and confidence carried; no options.
    let decisions: Vec<&todo_mcp::import::dacode::PlanOp> = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
        .collect();
    let PlanOp::AddDecision { draft, id, at } = decisions[0] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "camera:D1");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Signed);
    assert_eq!(draft.resolution.as_deref(), Some("the simple path taken"));
    assert_eq!(
        draft.recommendation.as_deref(),
        Some("take the simple path")
    );
    assert_eq!(draft.confidence, todo_domain::Confidence::Low);
    assert_eq!(draft.options, Vec::<String>::new());
    assert_eq!(at.to_string(), "2026-09-18T00:00:00.000000000Z");
    let PlanOp::AddDecision { draft, id, at } = decisions[1] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "camera:D2");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Open);
    assert_eq!(draft.resolution, None);
    assert_eq!(at.to_string(), "2026-10-06T12:00:00.000000000Z");

    // The task ops: the base range reproduced (U1–U8), the oddballs
    // after it (U9, U10) carrying their original ids in references,
    // the checkbox governing status, the debt kind under Debt.
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    let ids: Vec<&str> = tasks.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "camera:U1",
            "camera:U2",
            "camera:U3",
            "camera:U4",
            "camera:U5",
            "camera:U6",
            "camera:U7",
            "camera:U8",
            "camera:U9",
            "camera:U10",
        ]
    );
    // U1: open, the verify literal, no evidence, no references.
    let (_, u1) = tasks[0];
    assert_eq!(u1.title, "First follow-up");
    assert_eq!(u1.status, todo_domain::Status::Open);
    assert_eq!(u1.verify.as_deref(), Some("imported: the board's checkbox"));
    assert_eq!(u1.evidence, None);
    assert_eq!(u1.references.iter().count(), 0);
    // U2: checked → closed with the fixed evidence literal; the
    // trailing period trimmed.
    let (_, u2) = tasks[1];
    assert_eq!(u2.title, "Second follow-up");
    assert_eq!(u2.status, todo_domain::Status::Closed);
    assert_eq!(u2.evidence.as_deref(), Some("imported checked"));
    // U3: struck and checked → closed, the same as checked.
    let (_, u3) = tasks[2];
    assert_eq!(u3.status, todo_domain::Status::Closed);
    // U7: §5 → the debt kind under the Debt epic.
    let (_, u7) = tasks[6];
    assert_eq!(u7.kind, todo_domain::TaskKind::Debt);
    assert_eq!(u7.epic.to_string(), "camera:E5");
    // U1: §4 → the task kind under the Follow-ups epic.
    assert_eq!(u1.kind, todo_domain::TaskKind::Task);
    assert_eq!(u1.epic.to_string(), "camera:E4");
    // U9: the struck variant oddball — its original id as written in
    // references.
    let (_, u9) = tasks[8];
    assert_eq!(u9.title, "a variant item");
    let targets: Vec<&str> = u9.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["U4 (original)"]);
    // U10: the suffixed oddball.
    let (_, u10) = tasks[9];
    let targets: Vec<&str> = u10.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["U4a"]);

    // The edge ops: the three Gates edges (D-id order), then the one
    // held-back-by edge — the constructed ids, so the plan is
    // self-contained.
    let edges: Vec<&todo_domain::BlocksEdge> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(
        edges,
        vec![
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "camera:D1".parse().unwrap(),
                to: "camera:E1".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "camera:D2".parse().unwrap(),
                to: "camera:E2".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "camera:D3".parse().unwrap(),
                to: "camera:E3".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::TaskToTask {
                from: "camera:U2".parse().unwrap(),
                to: "camera:U5".parse().unwrap(),
            },
        ]
    );
}

/// A real `blocked on <id>` phrase creates the TaskToTask edge, and
/// the occurrences around it do not disturb the choice: a
/// boundary-violating `unblocked on <id>` and a non-id `blocked on
/// the console` are both skipped, and a *later* id-bearing occurrence
/// loses to the first — the first id-bearing occurrence wins. The
/// positive pin the live registers cannot cover — the census's one
/// `blocked on` lives in §6, walked past.
#[test]
fn a_blocked_on_phrase_creates_the_edge_past_skipped_occurrences() {
    let corpus = todo_md().replace(
        "- [ ] **U6 · Prose only** — this item sat blocked on the console\n  (U1) for weeks, and the id does not follow the phrase.",
        "- [ ] **U6 · Prose then a real block** — this item was unblocked\n  on U3, sat blocked on the console (U1) for weeks, and in the end\n  was blocked on U2, then blocked on U3.",
    );
    // The parse: only the first id-bearing occurrence is a dependency
    // — the boundary-violating and the non-id occurrences before it
    // are skipped, not consumed, and the later id-bearing U3 loses to
    // the first U2.
    let board = parse_camera_board(&corpus, DECISIONS_MD).unwrap();
    assert_eq!(
        board.todo.followups[7].dependencies,
        vec![Dependency {
            kind: DependencyKind::BlockedOn,
            id: "U2".to_owned(),
            text: "blocked on U2".to_owned(),
        }]
    );

    // The mapping: the blocked-on edge lands after the held-back-by
    // edge (task order), both referencing the constructed ids — the
    // edge goes to U2, not U3.
    let plan = map_camera(&board, &opts()).unwrap();
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(
        edges,
        vec![
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "camera:D1".parse().unwrap(),
                to: "camera:E1".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "camera:D2".parse().unwrap(),
                to: "camera:E2".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "camera:D3".parse().unwrap(),
                to: "camera:E3".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::TaskToTask {
                from: "camera:U2".parse().unwrap(),
                to: "camera:U5".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::TaskToTask {
                from: "camera:U2".parse().unwrap(),
                to: "camera:U6".parse().unwrap(),
            },
        ]
    );
}

/// Unresolvable assertions become finding tasks in the documented
/// order: a Gates cell naming a milestone §2 does not carry, and a
/// dependency phrase naming no register task — Gates findings first
/// (D-id order), then prose findings (task order), numbered after the
/// oddballs, never a silent drop.
#[test]
fn unresolvable_assertions_become_findings_in_the_documented_order() {
    let corpus = todo_md().replace(
        "- [ ] **U6 · Prose only** — this item sat blocked on the console\n  (U1) for weeks, and the id does not follow the phrase.",
        "- [ ] **U6 · An unresolvable block** — this item is blocked on\n  U99, which the register does not carry.",
    );
    let decisions = DECISIONS_MD.replace("| hold | high | M2 |", "| hold | high | M9 |");
    let board = parse_camera_board(&corpus, &decisions).unwrap();
    let plan = map_camera(&board, &opts()).unwrap();

    // Twelve tasks: the eight base items, the two oddballs, then the
    // two findings — the Gates finding first, the prose finding after.
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    assert_eq!(tasks.len(), 12);
    let (gates_id, gates) = tasks[10].clone();
    assert_eq!(gates_id, "camera:U11");
    assert_eq!(gates.kind, todo_domain::TaskKind::Finding);
    assert_eq!(gates.title, "D3 gates M9");
    assert_eq!(gates.epic.to_string(), "camera:E4");
    // The dossier's raw body, verbatim — the blank lines around the
    // prose line included, as the lines join.
    assert_eq!(gates.body, "\nDossier body for D3.\n");
    assert_eq!(gates.status, todo_domain::Status::Open);
    let gates_targets: Vec<&str> = gates.references.iter().map(|r| r.target()).collect();
    assert_eq!(gates_targets, vec!["D3"]);
    let (prose_id, prose) = tasks[11].clone();
    assert_eq!(prose_id, "camera:U12");
    assert_eq!(prose.kind, todo_domain::TaskKind::Finding);
    assert_eq!(prose.title, "U6 blocked on U99");
    assert_eq!(prose.epic.to_string(), "camera:E4");
    assert_eq!(prose.body, "blocked on U99");
    let prose_targets: Vec<&str> = prose.references.iter().map(|r| r.target()).collect();
    assert_eq!(prose_targets, vec!["U6"]);

    // D3's unresolvable Gates cell yields no edge; the two resolvable
    // Gates edges and U5's held-back-by edge remain.
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 3);
    assert_eq!(
        edges[0],
        &todo_domain::BlocksEdge::DecisionToEpic {
            from: "camera:D1".parse().unwrap(),
            to: "camera:E1".parse().unwrap(),
        }
    );
    assert_eq!(
        edges[2],
        &todo_domain::BlocksEdge::TaskToTask {
            from: "camera:U2".parse().unwrap(),
            to: "camera:U5".parse().unwrap(),
        }
    );
}

/// A shape-valid but impossible calendar date is a named error — the
/// dacode twin's pin: `2026-02-30` parses as a shape and fails as a
/// day, at the mapping's midnight construction.
#[test]
fn a_shape_valid_but_impossible_date_is_a_named_error() {
    let decisions = DECISIONS_MD.replace(
        "resolved 2026-09-18 — the simple path taken",
        "resolved 2026-02-30 — the simple path taken",
    );
    // The parse succeeds — the date is shape-valid — and the mapping
    // refuses it.
    let board = parse_camera_board(&todo_md(), &decisions).unwrap();
    assert_eq!(
        map_camera(&board, &opts()),
        Err(CameraError::BadDate {
            date: "2026-02-30".to_owned(),
        })
    );
}

// =================================================================
// 5. The plan order and the id reproduction at synthetic scale.
// =================================================================

/// A corpus with a contiguous base range plus planted oddballs,
/// applied to a fresh temp store: the store's task ids are exactly
/// the base range reproduced, the oddballs after it, each carrying
/// its original id in references.
#[test]
fn the_base_range_reproduces_and_the_oddballs_land_after_it() {
    // The generated corpus: §4 carries U1–U9 with U3's indented child
    // U3a and the struck U7 (original) variant; §5 carries U10–U12
    // with U12's indented child U12a. Twelve base items, three
    // oddballs.
    let mut section4 = String::from("## 4. Follow-ups\n\nGenerated base items.\n\n");
    for n in 1..=9 {
        section4.push_str(&format!("- [ ] **U{n} · Base item {n}** — body {n}.\n"));
        if n == 3 {
            section4.push_str("  - [x] **U3a · The first oddball** — child prose.\n");
        }
        if n == 7 {
            section4
                .push_str("- [x] ~~**U7 (original) · The second oddball**~~ — variant prose.\n");
        }
    }
    let mut section5 = String::from("## 5. Debt\n\nGenerated debt items.\n\n");
    for n in 10..=12 {
        section5.push_str(&format!("- [ ] **U{n} · Debt item {n}** — body {n}.\n"));
    }
    section5.push_str("  - [ ] **U12a · The third oddball** — child prose.\n");
    let generated = format!("{HEAD}{SECTION_2}{SECTION_3}{section4}{section5}{SECTION_6}");
    let decisions = "\
# Decisions

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| ~~**D1**~~ | The one decision | proceed | low | M0 | resolved 2026-09-18 — proceeded |

### D1 — The one decision — RESOLVED 2026-09-18

Dossier body.
";
    let board = parse_camera_board(&generated, decisions).unwrap();
    let plan = map_camera(&board, &opts()).unwrap();

    // A fresh temp store, registered and applied.
    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str("camera").unwrap(),
            name: "camera".to_owned(),
            path: "/repos/camera".to_owned(),
            registered_at: Timestamp::parse("2026-10-06T00:00:00Z").unwrap(),
        })
        .unwrap();
    apply_plan(&plan, &mut store).unwrap();

    // The store's task ids: the base range reproduced exactly — the
    // store's U<n> is the board's U<n> — then the oddballs after it.
    let tasks: Vec<&todo_domain::Task> = store.graph().tasks().collect();
    let ids: Vec<String> = tasks.iter().map(|task| task.id.to_string()).collect();
    let mut expected: Vec<String> = (1..=12).map(|n| format!("camera:U{n}")).collect();
    expected.push("camera:U13".to_owned());
    expected.push("camera:U14".to_owned());
    expected.push("camera:U15".to_owned());
    assert_eq!(ids, expected);

    // The reproduction is visible on the titles: the store's U<n> is
    // the board's U<n> for every base item.
    for n in 1..=12 {
        let task = tasks
            .iter()
            .find(|task| task.id.to_string() == format!("camera:U{n}"))
            .unwrap();
        assert_eq!(
            task.title,
            format!("Base item {n}").replace("Base", if n >= 10 { "Debt" } else { "Base" }),
            "U{n}"
        );
    }

    // The oddballs after the base, each carrying its original id in
    // references, in (numeric, id) order.
    let oddballs: Vec<&todo_domain::Task> = tasks
        .iter()
        .copied()
        .filter(|task| task.id.number() > 12)
        .collect();
    let targets: Vec<Vec<&str>> = oddballs
        .iter()
        .map(|task| task.references.iter().map(|r| r.target()).collect())
        .collect();
    assert_eq!(
        targets,
        vec![vec!["U3a"], vec!["U7 (original)"], vec!["U12a"],]
    );

    // The epic and decision populations landed too.
    assert_eq!(store.graph().epics().count(), 5);
    assert_eq!(store.graph().decisions().count(), 1);
    assert_eq!(store.graph().docs().count(), 0);
    assert_eq!(store.graph().blocks_edges().len(), 1);
}

// =================================================================
// 6. The wire: the dialect param's dispatch.
// =================================================================

/// The board directory, relative to the repo's registered checkout
/// path — the dialect's measured layout, mirrored from the tool's
/// constant.
const BOARD_DIR: &str = "docs/implementation";

/// Builds the corpus camera tree under a fresh temp directory: the
/// two board files at their dialect paths. Returns the tree's root —
/// the repo's registered checkout path.
fn camera_tree() -> TempDir {
    let root = TempDir::new().unwrap();
    let dir = root.path().join(BOARD_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("TODO.md"), todo_md()).unwrap();
    fs::write(dir.join("decisions.md"), DECISIONS_MD).unwrap();
    root
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

/// `import_md { dialect: "camera" }` against the synthetic camera
/// tree: the counts response and spot ids asserted in-test (no
/// golden, D12); an unknown dialect value is `invalid` naming it; the
/// absent dialect takes the dacode path — the io fault names the
/// dacode file the camera tree does not carry; and a camera tree
/// missing its decisions file faults as `io` naming the camera file's
/// role.
#[test]
fn import_md_with_dialect_camera_drives_the_camera_path() {
    let root = TempDir::new().unwrap();
    let tree = camera_tree();
    let requests = vec![
        call(2, "repo_register", &register_args("camera", tree.path())),
        call(3, "import_md", r#"{"repo":"camera","dialect":"camera"}"#),
        call(4, "graph_show", r#"{"repo":"camera"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 3);
    // The premise: the register and the import both succeeded.
    let registered: Value = serde_json::from_str(&driven.responses[0]).unwrap();
    assert_eq!(registered["result"]["isError"], json!(false));
    let imported: Value = serde_json::from_str(&driven.responses[1]).unwrap();
    assert_eq!(
        imported["result"]["isError"],
        json!(false),
        "{}",
        driven.responses[1]
    );

    // The counts response: the corpus's census, read off the landed
    // graph.
    let answer = structured(&driven.responses[1]);
    assert_eq!(answer["repo"], json!("camera"));
    assert_eq!(answer["epics"], json!(5));
    assert_eq!(answer["decisions"], json!(3));
    assert_eq!(answer["docs"], json!(0));
    assert_eq!(answer["tasks"], json!(10));
    assert_eq!(answer["blocks"], json!(4));

    // graph_show's spot checks: the reproduced base id, the debt kind,
    // and the oddball after the base carrying its original id.
    let graph = structured(&driven.responses[2]);
    let repo = &graph["repos"][0];
    let tasks: Vec<&Value> = repo["epics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["tasks"].as_array().unwrap())
        .collect();
    assert_eq!(tasks.len(), 10);
    let u1 = tasks
        .iter()
        .find(|task| task["id"] == json!("camera:U1"))
        .unwrap();
    assert_eq!(u1["title"], json!("First follow-up"));
    assert_eq!(u1["kind"], json!("task"));
    let u7 = tasks
        .iter()
        .find(|task| task["id"] == json!("camera:U7"))
        .unwrap();
    assert_eq!(u7["kind"], json!("debt"), "§5's items are debt");
    let u9 = tasks
        .iter()
        .find(|task| task["id"] == json!("camera:U9"))
        .unwrap();
    assert_eq!(u9["title"], json!("a variant item"));
    assert_eq!(
        u9["references"][0]["target"],
        json!("U4 (original)"),
        "the oddball carries its original id"
    );

    // An unknown dialect value: `invalid` naming it.
    let root = TempDir::new().unwrap();
    let tree = camera_tree();
    let driven = drive(
        root.path(),
        &[
            call(2, "repo_register", &register_args("camera", tree.path())),
            call(3, "import_md", r#"{"repo":"camera","dialect":"ikvm"}"#),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[1],
        "invalid",
        "the dialect 'ikvm' is not known: 'dacode' or 'camera'",
    );

    // The absent dialect: the dacode path — the camera tree lacks the
    // dacode layout, so the io fault names the dacode file.
    let root = TempDir::new().unwrap();
    let tree = camera_tree();
    let driven = drive(
        root.path(),
        &[
            call(2, "repo_register", &register_args("camera", tree.path())),
            call(3, "import_md", r#"{"repo":"camera"}"#),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[1],
        "io",
        "io: read the dacode board file appendices/B-decisions.md: NotFound",
    );

    // A camera tree missing its decisions file: the io fault names the
    // camera file's role, the same convention the dacode path pins.
    let root = TempDir::new().unwrap();
    let tree = camera_tree();
    fs::remove_file(tree.path().join(BOARD_DIR).join("decisions.md")).unwrap();
    let driven = drive(
        root.path(),
        &[
            call(2, "repo_register", &register_args("camera", tree.path())),
            call(3, "import_md", r#"{"repo":"camera","dialect":"camera"}"#),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[1],
        "io",
        "io: read the camera decisions file decisions.md: NotFound",
    );
}

// =================================================================
// 7. Determinism.
// =================================================================

/// The same board parsed and mapped twice yields equal ASTs and
/// equal plans — the dacode pin, camera's own.
#[test]
fn mapping_the_same_board_twice_yields_equal_plans() {
    // The premise first: both parses and both mappings succeed, or
    // the equality below would compare two identical failures.
    let first_board = parse_camera_board(&todo_md(), DECISIONS_MD).unwrap();
    let second_board = parse_camera_board(&todo_md(), DECISIONS_MD).unwrap();
    assert_eq!(first_board, second_board);
    let first = map_camera(&first_board, &opts()).unwrap();
    let second = map_camera(&second_board, &opts()).unwrap();
    assert_eq!(first, second);
}
