//! The own dialect's suite (t54): the parser (TODO.md's walker and
//! 4-column milestone table with the measured 5-cell widening,
//! decisions.md's summary table and dossiers, the Gates cell's
//! measured em-dash widening — "gates nothing", the post-roadmap
//! shape), the strictness and
//! never-fired-error pins (duplicate U, M and D ids, absent
//! registers, column counts, table ids, the State and Status
//! vocabularies, the widened row's last-cell guard, impossible
//! dates), the mapping (statuses, kinds, the dropped bodies, the
//! synthetics, the Gates edges, the oddball references, the
//! findings), the plan order and id reproduction at synthetic scale
//! with planted inversions, the `dialect` param's wire drive, and
//! determinism. All corpora are hand-authored synthetic strings in
//! the measured dialect — this repo's own live board is the import
//! probe's subject (D4: no golden, no checked-in fixture — the board
//! moves every session), never copied here: no live prose, no golden
//! pinning it.
//!
//! The wire test drives the real binary over a synthetic own tree
//! and asserts the counts response and spot ids in-test — no golden
//! (D4) — plus the dispatch and fault pins: an unknown dialect value
//! is `invalid` naming it, the absent dialect takes the dacode path
//! (the t44 suite pins the default's happy path; this pins the
//! dispatch from the own side), and the own tree missing a file
//! faults as `io` naming the own file's role.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{Repo, RepoSlug, Timestamp};
use todo_mcp::import::dacode::ImportError;
use todo_mcp::import::own::{
    ConfidenceCell, DecisionStatusCell, MilestoneState, OwnError, map_own, parse_decisions,
    parse_own_board, parse_todo_md,
};
use todo_mcp::import::plan::{ApplyError, ImportOptions, PlanOp, apply_plan};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, call, drive, structured};

// =================================================================
// The corpora — a synthetic minimal own board composed of section
// pieces, so the error-path corpora can swap one section out without
// duplicating the rest. §1, §3 and §6 are walked past, so their
// content exists to prove exactly that — including `blocked on`
// prose in each, which this dialect's registers never read.
// =================================================================

/// The intro and §1: the current-state narrative, walked past — its
/// table is not a register and its `blocked on` prose is never a
/// dependency.
const HEAD: &str = "\
# TODO — synthetic

Intro prose, walked past.

## 1. Current state

Narrative prose, walked past. It names ids — this run is blocked on
U99 — and none of them becomes a dependency.

| Blocking | What | Action |
|---|---|---|
| — | nothing blocks | dispatch |
";

/// §2: the milestone table — the 4-column header, one row per State
/// spelling, and M3's 5-cell shape: the measured widening where the
/// closure appended the `done` verdict as a new cell, keeping the
/// stale `in progress` delivery log as a cell of its own.
const SECTION_2: &str = "\
## 2. Milestones

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | groundwork | — | done 2026-01-05 |
| M1 | the engine | **D2** gates this milestone | done 2026-02-10 — the gate ran green: every code gate |
| M2 | the interface | — | in progress — decomposed 2026-03-15: the tree |
| M3 | adoption | — (D3 signed (a) 2026-01-05) | in progress — the stale delivery log | done 2026-04-20 — the gate ran green |
";

/// §3: the delivery log, walked past — its UB id is not a register
/// item.
const SECTION_3: &str = "\
## 3. Done

- 2026-09-18 — **UB1 · a delivery log entry.** The delivery log is
  walked past, and its UB id is not a register item.
";

/// §4: the follow-ups register — every shape the grammar accepts: a
/// plain item, a checked item with a trailing period, a struck item,
/// a wrapped bold run with an indented suffixed child, the struck
/// `(original)` variant, and prose carrying a real `blocked on <id>`
/// phrase that this dialect's registers never read as a dependency.
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
- [ ] **U5 · Prose only** — this item was blocked on U1, and the
  phrase is body text here: the registers carry no dependency prose.
";

/// §5: the debt register.
const SECTION_5: &str = "\
## 5. Debt

Synthetic debt entries, recorded rather than resolved.

- [ ] **U6 · A debt item** — debt prose.
- [x] **U7 · A checked debt item** — more debt prose.
";

/// §6: the session log, walked past — its `blocked on` prose is
/// outside the registers and never a dependency.
const SECTION_6: &str = "\
## 6. Session log

The narrative is walked past. Its prose names ids — this run is
blocked on U99 — and none of them becomes a dependency.
";

/// The full synthetic own TODO.md.
fn todo_md() -> String {
    format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}")
}

/// The synthetic own decisions.md, aligned with the corpus's
/// milestones: four rows exercising the confidence spellings and the
/// Status vocabulary's measured shapes — resolved with option and
/// note, resolved without the option letter, resolved with the
/// option and no note, and the drafted (open) spelling — plus four
/// dossiers.
const DECISIONS_MD: &str = "\
# Decision record — synthetic

Prose intro, walked past.

## Sign-off summary

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| ~~**D1**~~ | First decision | take the simple path | low | M0 | resolved (a) 2026-09-18 — the simple path taken |
| **D2** | Second decision | gather the numbers first | medium | M1 | resolved 2026-09-20 — user decision, no option letter |
| ~~**D3**~~ | Third decision | hold | high | M2 | resolved (b) 2026-09-21 |
| **D4** | Fourth decision | wait for the study | medium | M2 | drafted 2026-09-22 — awaiting the user |

## Details

### D1 — First decision

Dossier body for D1.

### D2 — Second decision

Dossier body for D2.

### D3 — Third decision

Dossier body for D3.

### D4 — Fourth decision

Dossier body for D4.
";

/// The import options the mapping tests share: a fixed slug and
/// stamp, an empty `doc_paths` (the own board has no doc register).
fn opts() -> ImportOptions {
    ImportOptions {
        slug: RepoSlug::from_str("dacode-todo-mcp").unwrap(),
        imported_at: Timestamp::parse("2026-10-07T12:00:00Z").unwrap(),
        doc_paths: std::collections::BTreeMap::new(),
    }
}

/// The corpus board, parsed and composed.
fn corpus_board() -> todo_mcp::import::own::OwnBoard {
    parse_own_board(&todo_md(), DECISIONS_MD).unwrap()
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

/// §2's milestone rows (board order, the Deliverable cell as the
/// title, the State verdicts including the 5-cell row's last-cell
/// reading), §4/§5 register extraction (ids, titles, checkbox
/// states, struck shapes, the wrapped bold run), §1/§3/§6 walked
/// past, and UB ids not a register.
#[test]
fn the_board_parser_extracts_the_registers_and_walks_the_prose_past() {
    let board = parse_todo_md(&todo_md()).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // §2: the milestone rows, in board order, with the State
    // spellings — M3's 5-cell row reads its LAST cell as the State.
    let milestones: Vec<(&str, &str, MilestoneState)> = board
        .milestones
        .iter()
        .map(|row| (row.id.as_str(), row.title.as_str(), row.status))
        .collect();
    assert_eq!(
        milestones,
        vec![
            ("M0", "groundwork", MilestoneState::Done),
            ("M1", "the engine", MilestoneState::Done),
            ("M2", "the interface", MilestoneState::InProgress),
            ("M3", "adoption", MilestoneState::Done),
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
            ("U5", "Prose only", false, false),
        ]
    );

    // §5: the debt register.
    let debt: Vec<(&str, bool)> = board
        .debt
        .iter()
        .map(|item| (item.id.as_str(), item.checked))
        .collect();
    assert_eq!(debt, vec![("U6", false), ("U7", true)]);

    // §1/§3/§6 walked past: no UB item, and the AST carries no
    // dependency field at all — the registers' `blocked on` prose
    // (U5's real phrase included) is body text this dialect drops.
    assert_eq!(board.followups.len() + board.debt.len(), 9);
}

// =================================================================
// 2. The strictness.
// =================================================================

/// A malformed register line, an absent §4, the struck-unchecked
/// combination, an unclosed bold run, an unknown State spelling, a
/// done spelling without its date, and a Milestone cell carrying
/// camera's ` — ` separator — each a named error with the 1-based
/// line.
#[test]
fn a_malformed_register_line_and_an_absent_section_are_named_errors() {
    // A malformed register line: the bold run's id is not U-shaped.
    let corpus = todo_md().replace(
        "- [ ] **U1 · First follow-up** — body prose, no dependency.",
        "- [ ] **X1 · Not a U id** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::MalformedLine {
            line: line_of(&corpus, "**X1 ·"),
            detail: "the bold run is not U-shaped: 'X1' is not a U id".to_owned(),
        })
    );

    // An absent §4: the named missing-section error, the line the walk
    // ended on.
    let corpus = format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_5}{SECTION_6}");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::MissingSection {
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
        Err(OwnError::StruckUnchecked {
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
        Err(OwnError::MalformedLine {
            line: line_of(&corpus, "**U1 · Title"),
            detail: "a U item's bold run does not close".to_owned(),
        })
    );

    // An unknown State-cell spelling.
    let corpus = todo_md().replace(
        "| M2 | the interface | — | in progress — decomposed 2026-03-15: the tree |",
        "| M2 | the interface | — | paused |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::BadStatus {
            line: line_of(&corpus, "| M2 | the interface | — | paused |"),
            text: "paused".to_owned(),
            detail: "the State cell must read 'done <date>' or 'in progress'".to_owned(),
        })
    );

    // A done spelling without its date.
    let corpus = todo_md().replace(
        "| M0 | groundwork | — | done 2026-01-05 |",
        "| M0 | groundwork | — | done |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::BadStatus {
            line: line_of(&corpus, "| M0 | groundwork | — | done |"),
            text: "done".to_owned(),
            detail: "the done spelling's date is missing".to_owned(),
        })
    );

    // A Milestone cell carrying camera's ` — ` separator: the id is
    // not a bare M id — the title lives in the Deliverable cell here.
    let corpus = todo_md().replace(
        "| M0 | groundwork | — | done 2026-01-05 |",
        "| M0 — groundwork | the first deliverable | — | done 2026-01-05 |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::BadId {
            line: line_of(&corpus, "| M0 — groundwork |"),
            id: "M0 — groundwork".to_owned(),
            detail: "not a bare M id — the title lives in the Deliverable cell".to_owned(),
        })
    );
}

/// A row wider than four cells whose last cell is not a State
/// spelling is the State's named error, never a column-count error —
/// the widening reads the LAST cell as the State whatever the cell
/// count, so the guard is the spelling check, not the arithmetic.
#[test]
fn a_widened_row_with_a_non_state_last_cell_is_the_status_error() {
    // A 6-cell row: the closure appended a second verdict-shaped cell
    // after the done cell, so the row's last cell is prose.
    let corpus = todo_md().replace(
        "| M3 | adoption | — (D3 signed (a) 2026-01-05) | in progress — the stale delivery log | done 2026-04-20 — the gate ran green |",
        "| M3 | adoption | — (D3 signed (a) 2026-01-05) | in progress — the stale delivery log | done 2026-04-20 | the gate ran green |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::BadStatus {
            line: line_of(&corpus, "| M3 | adoption"),
            text: "the gate ran green".to_owned(),
            detail: "the State cell must read 'done <date>' or 'in progress'".to_owned(),
        })
    );
}

/// A duplicated register id is a named error naming both lines — the
/// board's ids are unique, and a silent second entry would
/// double-import one item. The U namespace spans §4 and §5, so the
/// duplicate is named whether it lands twice in one section or once
/// in each.
#[test]
fn a_duplicate_register_id_is_a_named_error_naming_both_lines() {
    // Twice within §4.
    let corpus = todo_md().replace(
        "- [ ] **U5 · Prose only** — this item was blocked on U1, and the",
        "- [ ] **U2 · A duplicate id** — body prose.",
    );
    let first = line_of(&corpus, "**U2 · Second follow-up");
    let second = line_of(&corpus, "**U2 · A duplicate id**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::DuplicateId {
            id: "U2".to_owned(),
            first,
            second,
        })
    );

    // Once in §4 and once in §5: the cross-section duplicate — the
    // id map is the walker's, shared across both registers.
    let corpus = todo_md().replace(
        "- [ ] **U6 · A debt item** — debt prose.",
        "- [ ] **U2 · A cross-section duplicate** — debt prose.",
    );
    let first = line_of(&corpus, "**U2 · Second follow-up");
    let second = line_of(&corpus, "**U2 · A cross-section duplicate**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::DuplicateId {
            id: "U2".to_owned(),
            first,
            second,
        })
    );
}

/// A duplicated §2 milestone id and a duplicated decisions summary id
/// are named errors naming both lines — the composition's id
/// cross-check compares sets (which dedupe), so a silent duplicate
/// would pass it and shift the constructed id ranges, mis-mapping
/// every later row.
#[test]
fn duplicate_milestone_and_decision_ids_are_named_errors() {
    // A duplicated M id in §2: the M2 row rewritten as a second M0.
    let corpus = todo_md().replace(
        "| M2 | the interface | — | in progress — decomposed 2026-03-15: the tree |",
        "| M0 | a duplicate milestone | — | done 2026-01-05 |",
    );
    let first = line_of(&corpus, "| M0 | groundwork");
    let second = line_of(&corpus, "| M0 | a duplicate milestone");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::DuplicateId {
            id: "M0".to_owned(),
            first,
            second,
        })
    );

    // A duplicated D id in the summary table: the D4 row rewritten as
    // a second D1.
    let decisions = DECISIONS_MD.replace(
        "| **D4** | Fourth decision | wait for the study | medium | M2 | drafted 2026-09-22 — awaiting the user |",
        "| **D1** | a duplicate row | hold | low | M0 | resolved (a) 2026-09-18 — taken |",
    );
    let first = line_of(&decisions, "| ~~**D1**~~ | First decision");
    let second = line_of(&decisions, "| **D1** | a duplicate row");
    assert_eq!(
        parse_decisions(&decisions),
        Err(OwnError::DuplicateId {
            id: "D1".to_owned(),
            first,
            second,
        })
    );
}

/// Absent registers, wrong column counts and malformed table ids are
/// named errors: §2 without its milestones table, the decisions file
/// without its summary table, a §2 row with too few cells, a
/// decisions row with the wrong cell count, and a Milestone, ID or
/// Gates cell whose id is not the shape its register requires.
#[test]
fn absent_registers_and_malformed_table_rows_are_named_errors() {
    // §2 without its milestones table: the register is required — its
    // absence would silently omit every milestone.
    let corpus = format!(
        "{HEAD}## 2. Milestones\n\nNo table here, only prose.\n\n{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}"
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: corpus.split('\n').count(),
        })
    );

    // The decisions file without its summary table.
    let decisions = "# Decision record\n\nNo table, only prose.\n\n### D1 — First\n\nBody.\n";
    assert_eq!(
        parse_decisions(decisions),
        Err(OwnError::MissingRegister {
            register: "the decisions summary table".to_owned(),
            line: decisions.split('\n').count(),
        })
    );

    // A §2 row with too few cells: the column-count error names the
    // header's four.
    let corpus = todo_md().replace(
        "| M0 | groundwork | — | done 2026-01-05 |",
        "| M0 | groundwork | done 2026-01-05 |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::ColumnCount {
            line: line_of(&corpus, "| M0 | groundwork | done 2026-01-05 |"),
            expected: 4,
            found: 3,
        })
    );

    // A decisions row with the wrong cell count.
    let decisions = DECISIONS_MD.replace(
        "| **D2** | Second decision | gather the numbers first | medium | M1 | resolved 2026-09-20 — user decision, no option letter |",
        "| **D2** | Second decision | gather the numbers first | medium | M1 |",
    );
    assert_eq!(
        parse_decisions(&decisions),
        Err(OwnError::ColumnCount {
            line: line_of(&decisions, "| **D2** | Second decision"),
            expected: 6,
            found: 5,
        })
    );

    // A Milestone cell whose id is not M-shaped.
    let corpus = todo_md().replace(
        "| M0 | groundwork | — | done 2026-01-05 |",
        "| X0 | groundwork | — | done 2026-01-05 |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(OwnError::BadId {
            line: line_of(&corpus, "| X0 | groundwork"),
            id: "X0".to_owned(),
            detail: "not a bare M id — the title lives in the Deliverable cell".to_owned(),
        })
    );

    // A decisions ID cell that is not a D id.
    let decisions = DECISIONS_MD.replace(
        "| ~~**D1**~~ | First decision |",
        "| ~~**X1**~~ | First decision |",
    );
    assert_eq!(
        parse_decisions(&decisions),
        Err(OwnError::BadId {
            line: line_of(&decisions, "| ~~**X1**~~"),
            id: "X1".to_owned(),
            detail: "not a D id".to_owned(),
        })
    );

    // A Gates cell that is not an M id.
    let decisions = DECISIONS_MD.replace("| hold | high | M2 |", "| hold | high | D3 |");
    assert_eq!(
        parse_decisions(&decisions),
        Err(OwnError::BadId {
            line: line_of(&decisions, "| hold | high | D3 |"),
            id: "D3".to_owned(),
            detail: "not an M id".to_owned(),
        })
    );
}

// =================================================================
// 3. The decisions parser.
// =================================================================

/// The summary table's cells (title, recommendation, confidence,
/// gates, the Status vocabulary's measured shapes), the dossiers'
/// headers and bodies, and the table/dossier id cross-check — a
/// mismatch a named error, an unknown confidence, an empty
/// Recommendation cell, an unrecognized Status spelling, a resolved
/// spelling without its date, a malformed option, text after the
/// date that is not a note, and a malformed dossier header each a
/// named error too.
#[test]
fn the_decisions_parser_reads_the_table_and_cross_checks_the_dossiers() {
    let file = parse_decisions(DECISIONS_MD).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // The summary rows, in board order, every cell parsed — the
    // Status vocabulary's four measured shapes.
    assert_eq!(file.summary.len(), 4);
    assert_eq!(file.summary[0].id, "D1");
    assert_eq!(file.summary[0].title, "First decision");
    assert_eq!(file.summary[0].recommendation, "take the simple path");
    assert_eq!(file.summary[0].confidence, ConfidenceCell::Low);
    assert_eq!(file.summary[0].gates.as_deref(), Some("M0"));
    assert_eq!(
        file.summary[0].status,
        DecisionStatusCell::Resolved {
            option: Some("a".to_owned()),
            date: "2026-09-18".to_owned(),
            note: Some("the simple path taken".to_owned()),
        }
    );
    assert_eq!(
        file.summary[1].status,
        DecisionStatusCell::Resolved {
            option: None,
            date: "2026-09-20".to_owned(),
            note: Some("user decision, no option letter".to_owned()),
        }
    );
    assert_eq!(
        file.summary[2].status,
        DecisionStatusCell::Resolved {
            option: Some("b".to_owned()),
            date: "2026-09-21".to_owned(),
            note: None,
        }
    );
    assert_eq!(file.summary[3].status, DecisionStatusCell::Drafted);
    assert_eq!(file.summary[3].gates.as_deref(), Some("M2"));

    // The dossiers: ids, titles, bodies.
    assert_eq!(file.dossiers.len(), 4);
    assert_eq!(file.dossiers[0].id, "D1");
    assert_eq!(file.dossiers[0].title, "First decision");
    assert!(file.dossiers[2].body.contains("Dossier body for D3."));

    // The composition's cross-check passes on the aligned corpus.
    assert!(parse_own_board(&todo_md(), DECISIONS_MD).is_ok());

    // A dossier the table does not carry: the named mismatch, the
    // smallest offending id.
    let extra = format!("{DECISIONS_MD}### D5 — Fifth decision\n\nDossier body for D5.\n");
    assert_eq!(
        parse_own_board(&todo_md(), &extra),
        Err(OwnError::IdSetMismatch {
            id: "D5".to_owned(),
            detail: "D5 is in the decisions dossiers but not in the decisions summary table"
                .to_owned(),
        })
    );

    // An unknown confidence spelling.
    let bad_confidence = DECISIONS_MD.replace("| low | M0 |", "| med | M0 |");
    assert_eq!(
        parse_decisions(&bad_confidence),
        Err(OwnError::BadStatus {
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
        Err(OwnError::MalformedLine {
            line: line_of(&bad_recommendation, "|  | low | M0 |"),
            detail: "the Recommendation cell is empty".to_owned(),
        })
    );

    // An unrecognized Status spelling.
    let bad_status =
        DECISIONS_MD.replace("resolved (a) 2026-09-18 — the simple path taken", "paused");
    assert_eq!(
        parse_decisions(&bad_status),
        Err(OwnError::BadStatus {
            line: line_of(&bad_status, "| paused |"),
            text: "paused".to_owned(),
            detail: "the Status cell must read 'resolved [<option>] <date> [— <note>]' or 'drafted <date> [— <note>]'"
                .to_owned(),
        })
    );

    // A resolved spelling without its date.
    let no_date = DECISIONS_MD.replace("resolved (b) 2026-09-21", "resolved (b)");
    assert_eq!(
        parse_decisions(&no_date),
        Err(OwnError::BadStatus {
            line: line_of(&no_date, "| resolved (b) |"),
            text: "resolved (b)".to_owned(),
            detail: "the status spelling lacks its date".to_owned(),
        })
    );

    // A malformed option: two tokens inside the parentheses.
    let bad_option = DECISIONS_MD.replace(
        "resolved (a) 2026-09-18 — the simple path taken",
        "resolved (a b) 2026-09-18 — taken",
    );
    assert_eq!(
        parse_decisions(&bad_option),
        Err(OwnError::BadStatus {
            line: line_of(&bad_option, "| resolved (a b)"),
            text: "resolved (a b) 2026-09-18 — taken".to_owned(),
            detail: "the status option must be one space-free token, e.g. '(a)'".to_owned(),
        })
    );

    // Text after the date that is not a ` — ` note.
    let bad_tail = DECISIONS_MD.replace("resolved (b) 2026-09-21", "resolved (b) 2026-09-21 extra");
    assert_eq!(
        parse_decisions(&bad_tail),
        Err(OwnError::BadStatus {
            line: line_of(&bad_tail, "| resolved (b) 2026-09-21 extra"),
            text: "resolved (b) 2026-09-21 extra".to_owned(),
            detail: "the status spelling carries text after the date that is not a ' — ' note"
                .to_owned(),
        })
    );

    // A dossier header without its separator.
    let bad_header = DECISIONS_MD.replace("### D2 — Second decision", "### D2 Second decision");
    assert_eq!(
        parse_decisions(&bad_header),
        Err(OwnError::MalformedLine {
            line: line_of(&bad_header, "### D2 Second decision"),
            detail: "a dossier header has no ' — ' separator after the id".to_owned(),
        })
    );

    // A dossier header whose id is not a D id.
    let bad_header_id = DECISIONS_MD.replace("### D2 — Second decision", "### X2 — Second");
    assert_eq!(
        parse_decisions(&bad_header_id),
        Err(OwnError::BadId {
            line: line_of(&bad_header_id, "### X2 — Second"),
            id: "X2".to_owned(),
            detail: "not a D id".to_owned(),
        })
    );
}

// =================================================================
// 4. The mapping.
// =================================================================

/// The mapping's conventions: the census on the corpus, the epic
/// statuses from the State cells (the 5-cell row's last cell
/// included), the decision statuses and resolutions (the note, the
/// option, the fixed literal), the timestamps, the task statuses
/// (checkbox governs), the debt kind for §5, the dropped bodies, the
/// two synthetics, the Gates edges — and no TaskToTask edge from the
/// registers' `blocked on` prose — the oddball references, and the
/// plan's total order.
#[test]
fn the_mapping_pins_statuses_kinds_edges_and_the_dropped_bodies() {
    let board = corpus_board();
    let plan = map_own(&board, &opts()).unwrap();
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
        (6, 4, 9, 4)
    );
    let first_edge = ops
        .iter()
        .position(|op| matches!(op, PlanOp::LinkBlocks { .. }))
        .unwrap();
    assert_eq!(first_edge, epic_count + decision_count + task_count);

    // The epic ops: the milestones' statuses from the State cells
    // (board order — E1..E4 are M0..M3), then the two synthetics.
    for (i, op) in ops.iter().enumerate() {
        let PlanOp::AddEpic { draft, id, .. } = op else {
            continue;
        };
        match id.to_string().as_str() {
            "dacode-todo-mcp:E1" => {
                assert_eq!(draft.title, "groundwork");
                assert_eq!(draft.status, todo_domain::Status::Closed);
                assert_eq!(
                    draft.description,
                    "the dacode-todo-mcp board's §2 milestone M0"
                );
            }
            "dacode-todo-mcp:E2" => {
                assert_eq!(draft.title, "the engine");
                assert_eq!(draft.status, todo_domain::Status::Closed);
            }
            "dacode-todo-mcp:E3" => {
                assert_eq!(draft.title, "the interface");
                assert_eq!(draft.status, todo_domain::Status::InWork);
            }
            "dacode-todo-mcp:E4" => {
                // The 5-cell row: the LAST cell is the State — done.
                assert_eq!(draft.title, "adoption");
                assert_eq!(draft.status, todo_domain::Status::Closed);
            }
            "dacode-todo-mcp:E5" => {
                assert_eq!(draft.title, "Follow-ups");
                assert_eq!(draft.status, todo_domain::Status::Open);
                assert_eq!(
                    draft.description,
                    "the dacode-todo-mcp board's §4 follow-ups, not tied to a milestone"
                );
            }
            "dacode-todo-mcp:E6" => {
                assert_eq!(draft.title, "Debt");
                assert_eq!(draft.status, todo_domain::Status::Open);
                assert_eq!(
                    draft.description,
                    "the dacode-todo-mcp board's §5 debt, recorded rather than forgotten"
                );
            }
            other => panic!("unexpected epic id {other} at op {i}"),
        }
    }

    // The decision ops: resolved → Signed with the resolution (the
    // note, else the option, else the fixed literal) and the date as
    // the stamp; drafted → Open at the import stamp; the
    // recommendation and confidence carried; no options.
    let decisions: Vec<&PlanOp> = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
        .collect();
    let PlanOp::AddDecision { draft, id, at } = decisions[0] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "dacode-todo-mcp:D1");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Signed);
    assert_eq!(draft.resolution.as_deref(), Some("the simple path taken"));
    assert_eq!(
        draft.recommendation.as_deref(),
        Some("take the simple path")
    );
    assert_eq!(draft.confidence, todo_domain::Confidence::Low);
    assert_eq!(draft.options, Vec::<String>::new());
    assert_eq!(at.to_string(), "2026-09-18T00:00:00.000000000Z");
    let PlanOp::AddDecision { draft, id, .. } = decisions[1] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "dacode-todo-mcp:D2");
    // No option letter: the note is the resolution.
    assert_eq!(
        draft.resolution.as_deref(),
        Some("user decision, no option letter")
    );
    let PlanOp::AddDecision { draft, id, .. } = decisions[2] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "dacode-todo-mcp:D3");
    // No note: the option letter is the resolution.
    assert_eq!(draft.resolution.as_deref(), Some("(b)"));
    let PlanOp::AddDecision { draft, id, at } = decisions[3] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "dacode-todo-mcp:D4");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Open);
    assert_eq!(draft.resolution, None);
    assert_eq!(at.to_string(), "2026-10-07T12:00:00.000000000Z");

    // The task ops: the base range reproduced (U1–U7), the oddballs
    // after it (U8, U9) carrying their original ids in references,
    // the checkbox governing status, the debt kind under Debt, and
    // every body dropped — the empty string.
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
            "dacode-todo-mcp:U1",
            "dacode-todo-mcp:U2",
            "dacode-todo-mcp:U3",
            "dacode-todo-mcp:U4",
            "dacode-todo-mcp:U5",
            "dacode-todo-mcp:U6",
            "dacode-todo-mcp:U7",
            "dacode-todo-mcp:U8",
            "dacode-todo-mcp:U9",
        ]
    );
    // U1: open, the verify literal, no evidence, no references, the
    // body dropped.
    let (_, u1) = tasks[0];
    assert_eq!(u1.title, "First follow-up");
    assert_eq!(u1.status, todo_domain::Status::Open);
    assert_eq!(u1.verify.as_deref(), Some("imported: the board's checkbox"));
    assert_eq!(u1.evidence, None);
    assert_eq!(u1.body, "");
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
    // U6: §5 → the debt kind under the Debt epic.
    let (_, u6) = tasks[5];
    assert_eq!(u6.kind, todo_domain::TaskKind::Debt);
    assert_eq!(u6.epic.to_string(), "dacode-todo-mcp:E6");
    // U1: §4 → the task kind under the Follow-ups epic.
    assert_eq!(u1.kind, todo_domain::TaskKind::Task);
    assert_eq!(u1.epic.to_string(), "dacode-todo-mcp:E5");
    // U8: the struck variant oddball — its original id as written in
    // references.
    let (_, u8) = tasks[7];
    assert_eq!(u8.title, "a variant item");
    let targets: Vec<&str> = u8.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["U4 (original)"]);
    // U9: the suffixed oddball.
    let (_, u9) = tasks[8];
    let targets: Vec<&str> = u9.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["U4a"]);

    // The edge ops: the four Gates edges (D-id order) — and no
    // TaskToTask edge anywhere: the registers' `blocked on U1` prose
    // is body text this dialect drops, so the constructed ids make
    // the plan self-contained with Gates edges alone.
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
                from: "dacode-todo-mcp:D1".parse().unwrap(),
                to: "dacode-todo-mcp:E1".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "dacode-todo-mcp:D2".parse().unwrap(),
                to: "dacode-todo-mcp:E2".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "dacode-todo-mcp:D3".parse().unwrap(),
                to: "dacode-todo-mcp:E3".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "dacode-todo-mcp:D4".parse().unwrap(),
                to: "dacode-todo-mcp:E3".parse().unwrap(),
            },
        ]
    );
}

/// The milestone epics follow **board order**, not numeric order: a
/// §2 table whose rows are swapped assigns E1 to the first row as
/// written, and the Gates edges follow the board-position index.
#[test]
fn the_milestone_epics_follow_board_order_not_numeric_order() {
    let corpus = todo_md().replace(
        "| M0 | groundwork | — | done 2026-01-05 |\n| M1 | the engine | **D2** gates this milestone | done 2026-02-10 — the gate ran green: every code gate |",
        "| M1 | the engine | **D2** gates this milestone | done 2026-02-10 — the gate ran green: every code gate |\n| M0 | groundwork | — | done 2026-01-05 |",
    );
    let board = parse_own_board(&corpus, DECISIONS_MD).unwrap();
    let plan = map_own(&board, &opts()).unwrap();
    let epics: Vec<(String, String)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddEpic { draft, id, .. } => Some((id.to_string(), draft.title.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        epics,
        vec![
            ("dacode-todo-mcp:E1".to_owned(), "the engine".to_owned()),
            ("dacode-todo-mcp:E2".to_owned(), "groundwork".to_owned()),
            ("dacode-todo-mcp:E3".to_owned(), "the interface".to_owned()),
            ("dacode-todo-mcp:E4".to_owned(), "adoption".to_owned()),
            ("dacode-todo-mcp:E5".to_owned(), "Follow-ups".to_owned()),
            ("dacode-todo-mcp:E6".to_owned(), "Debt".to_owned()),
        ]
    );
    // D1 gates M0, which is now the SECOND epic: the edge follows the
    // board-position index.
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(
        edges[0],
        &todo_domain::BlocksEdge::DecisionToEpic {
            from: "dacode-todo-mcp:D1".parse().unwrap(),
            to: "dacode-todo-mcp:E2".parse().unwrap(),
        }
    );
}

/// A resolved decision whose Status carries neither a note nor an
/// option letter maps to the fixed `imported signed` resolution —
/// the domain requires a resolution for a signed decision, and the
/// board's row carries nothing to say.
#[test]
fn a_bare_resolved_row_maps_to_the_fixed_resolution_literal() {
    let decisions = DECISIONS_MD.replace("resolved (b) 2026-09-21", "resolved 2026-09-21");
    let board = parse_own_board(&todo_md(), &decisions).unwrap();
    let plan = map_own(&board, &opts()).unwrap();
    let decisions: Vec<&PlanOp> = plan
        .ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
        .collect();
    let PlanOp::AddDecision { draft, id, .. } = decisions[2] else {
        unreachable!("filtered above")
    };
    assert_eq!(id.to_string(), "dacode-todo-mcp:D3");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Signed);
    assert_eq!(draft.resolution.as_deref(), Some("imported signed"));
}

/// An unresolvable Gates cell becomes a finding task in the
/// documented order — D-id order, under `Follow-ups`, the dossier's
/// raw body as the assertion's context — never a silent drop.
#[test]
fn an_unresolvable_gates_cell_becomes_a_finding() {
    let decisions = DECISIONS_MD.replace("| hold | high | M2 |", "| hold | high | M9 |");
    let board = parse_own_board(&todo_md(), &decisions).unwrap();
    let plan = map_own(&board, &opts()).unwrap();

    // Ten tasks: the seven base items, the two oddballs, then the
    // Gates finding.
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    assert_eq!(tasks.len(), 10);
    let (finding_id, finding) = tasks[9].clone();
    assert_eq!(finding_id, "dacode-todo-mcp:U10");
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "D3 gates M9");
    assert_eq!(finding.epic.to_string(), "dacode-todo-mcp:E5");
    // The dossier's raw body, verbatim — the blank lines around the
    // prose line included, as the lines join.
    assert_eq!(finding.body, "\nDossier body for D3.\n");
    assert_eq!(finding.status, todo_domain::Status::Open);
    let targets: Vec<&str> = finding.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["D3"]);

    // D3's unresolvable Gates cell yields no edge; the three
    // resolvable Gates edges remain.
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 3);
}

/// The em-dash Gates cell — "gates nothing", the post-roadmap shape
/// D19 introduced on the live board — parses to `None` and produces
/// neither a finding nor an edge: an absence, not an unresolvable
/// assertion. (The strictness half — a garbage cell is a named error,
/// never a silent gates-nothing — is the existing "not an M id" pin
/// above, unchanged by the widening: only the exact em-dash reads as
/// gates-nothing.)
#[test]
fn an_em_dash_gates_cell_gates_nothing() {
    // The parse: the em-dash row reads back as `None`.
    let decisions = DECISIONS_MD.replace("| hold | high | M2 |", "| hold | high | — |");
    let file = parse_decisions(&decisions).unwrap();
    assert_eq!(file.summary[2].id, "D3");
    assert_eq!(file.summary[2].gates, None);

    // The map: no finding task for D3 (nine tasks, not ten) and no
    // edge from it — D1, D2 and D4's edges remain (D4 also gates M2,
    // so the count matches the unresolvable case; the from-id is what
    // names D3's absence).
    let board = parse_own_board(&todo_md(), &decisions).unwrap();
    let plan = map_own(&board, &opts()).unwrap();
    let tasks = plan
        .ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddTask { .. }))
        .count();
    assert_eq!(tasks, 9, "no finding task for a gates-nothing row");
    let edges: Vec<String> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks {
                edge: todo_domain::BlocksEdge::DecisionToEpic { from, .. },
                ..
            } => Some(from.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 3);
    assert!(
        !edges.contains(&"dacode-todo-mcp:D3".to_owned()),
        "no edge from a gates-nothing row: {edges:?}"
    );

    // The measured boundary: a near-miss dash is NOT the em-dash —
    // the en-dash cell is the named error, never a silent
    // gates-nothing.
    let en_dash = DECISIONS_MD.replace("| hold | high | M2 |", "| hold | high | – |");
    assert_eq!(
        parse_decisions(&en_dash),
        Err(OwnError::BadId {
            line: line_of(&en_dash, "| hold | high | – |"),
            id: "–".to_owned(),
            detail: "not an M id".to_owned(),
        })
    );
}

/// A shape-valid but impossible calendar date is a named error — the
/// camera twin's pin: `2026-02-30` parses as a shape and fails as a
/// day, at the mapping's midnight construction.
#[test]
fn a_shape_valid_but_impossible_date_is_a_named_error() {
    let decisions = DECISIONS_MD.replace(
        "resolved (a) 2026-09-18 — the simple path taken",
        "resolved (a) 2026-02-30 — the simple path taken",
    );
    // The parse succeeds — the date is shape-valid — and the mapping
    // refuses it.
    let board = parse_own_board(&todo_md(), &decisions).unwrap();
    assert_eq!(
        map_own(&board, &opts()),
        Err(OwnError::BadDate {
            date: "2026-02-30".to_owned(),
        })
    );
}

/// The apply refusal: a plan applied to a store that already holds
/// nodes is refused by name — the id reproduction would be corrupted
/// by a used store's burned counters.
#[test]
fn applying_to_a_non_empty_graph_is_refused_by_name() {
    let board = corpus_board();
    let plan = map_own(&board, &opts()).unwrap();
    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str("dacode-todo-mcp").unwrap(),
            name: "dacode-todo-mcp".to_owned(),
            path: "/repos/dacode-todo-mcp".to_owned(),
            registered_at: Timestamp::parse("2026-10-07T00:00:00Z").unwrap(),
        })
        .unwrap();
    apply_plan(&plan, &mut store).unwrap();
    let refusal = apply_plan(&plan, &mut store).unwrap_err();
    assert_eq!(
        refusal,
        ApplyError::Refused(ImportError::StoreNotEmpty {
            slug: "dacode-todo-mcp".to_owned(),
        })
    );
}

// =================================================================
// 5. The plan order and the id reproduction at synthetic scale.
// =================================================================

/// A generated corpus with a contiguous base range whose board order
/// carries planted inversions (the live board's own shape: U26–U28
/// sit before U22, U13 after it), plus seventeen decisions with
/// seventeen Gates edges, applied to a fresh temp store: the store's
/// task ids are exactly the base range reproduced — the sort, not
/// the board order, governs — and the edge count is the pin.
#[test]
fn the_base_range_reproduces_past_planted_inversions_and_the_edges_pin_at_seventeen() {
    // §4 carries U1–U5 in order, then the planted inversion U8, U6,
    // U7; §5 carries U9, U11, U10 — another inversion. Eleven base
    // ids, contiguous U1–U11, board order with inversions.
    let mut section4 = String::from("## 4. Follow-ups\n\nGenerated base items.\n\n");
    for n in 1..=5 {
        section4.push_str(&format!("- [ ] **U{n} · Base item {n}** — body {n}.\n"));
    }
    for n in [8, 6, 7] {
        section4.push_str(&format!("- [x] **U{n} · Base item {n}** — body {n}.\n"));
    }
    let mut section5 = String::from("## 5. Debt\n\nGenerated debt items.\n\n");
    for n in [9, 11, 10] {
        section5.push_str(&format!("- [ ] **U{n} · Debt item {n}** — body {n}.\n"));
    }
    let generated = format!("{HEAD}{SECTION_2}{SECTION_3}{section4}{section5}{SECTION_6}");
    // Seventeen decisions, gates cycling M0–M3, all resolved — the
    // edge-count pin's population.
    let mut decisions = String::from(
        "# Decision record — synthetic\n\n## Sign-off summary\n\n| ID | Decision | Recommendation | Confidence | Gates | Status |\n|---|---|---|---|---|---|\n",
    );
    for n in 1..=17 {
        let gates = format!("M{}", (n - 1) % 4);
        decisions.push_str(&format!(
            "| ~~**D{n}**~~ | Decision {n} | proceed | medium | {gates} | resolved (a) 2026-09-18 — done |\n"
        ));
    }
    decisions.push_str("\n## Details\n\n");
    for n in 1..=17 {
        decisions.push_str(&format!("### D{n} — Decision {n}\n\nDossier body {n}.\n\n"));
    }
    let board = parse_own_board(&generated, &decisions).unwrap();
    let plan = map_own(&board, &opts()).unwrap();

    // The census: 6 epics, 17 decisions, 11 tasks, 17 edges.
    let count = |kind: &str| {
        plan.ops
            .iter()
            .filter(|op| {
                matches!(
                    (op, kind),
                    (PlanOp::AddEpic { .. }, "epic")
                        | (PlanOp::AddDecision { .. }, "decision")
                        | (PlanOp::AddTask { .. }, "task")
                        | (PlanOp::LinkBlocks { .. }, "edge")
                )
            })
            .count()
    };
    assert_eq!(
        (
            count("epic"),
            count("decision"),
            count("task"),
            count("edge")
        ),
        (6, 17, 11, 17)
    );

    // A fresh temp store, registered and applied.
    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str("dacode-todo-mcp").unwrap(),
            name: "dacode-todo-mcp".to_owned(),
            path: "/repos/dacode-todo-mcp".to_owned(),
            registered_at: Timestamp::parse("2026-10-07T00:00:00Z").unwrap(),
        })
        .unwrap();
    apply_plan(&plan, &mut store).unwrap();

    // The store's task ids: the base range reproduced exactly — the
    // store's U<n> is the board's U<n> — past the planted inversions.
    let tasks: Vec<&todo_domain::Task> = store.graph().tasks().collect();
    let ids: Vec<String> = tasks.iter().map(|task| task.id.to_string()).collect();
    let expected: Vec<String> = (1..=11).map(|n| format!("dacode-todo-mcp:U{n}")).collect();
    assert_eq!(ids, expected);

    // The reproduction is visible on the titles: the store's U<n> is
    // the board's U<n> for every base item — the inversion (U8 before
    // U6 in board order) is corrected by the sort, not preserved.
    for n in 1..=11 {
        let task = tasks
            .iter()
            .find(|task| task.id.to_string() == format!("dacode-todo-mcp:U{n}"))
            .unwrap();
        let expected_title = if n <= 8 {
            format!("Base item {n}")
        } else {
            format!("Debt item {n}")
        };
        assert_eq!(task.title, expected_title, "U{n}");
    }

    // The epic and decision populations landed too, with the edge
    // count pinned at seventeen.
    assert_eq!(store.graph().epics().count(), 6);
    assert_eq!(store.graph().decisions().count(), 17);
    assert_eq!(store.graph().docs().count(), 0);
    assert_eq!(store.graph().blocks_edges().len(), 17);
}

// =================================================================
// 6. The wire: the dialect param's dispatch.
// =================================================================

/// The board directory, relative to the repo's registered checkout
/// path — the dialect's measured layout, mirrored from the tool's
/// constant.
const BOARD_DIR: &str = "docs/implementation";

/// Builds the corpus own tree under a fresh temp directory: the two
/// board files at their dialect paths. Returns the tree's root — the
/// repo's registered checkout path.
fn own_tree() -> TempDir {
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

/// `import_md { dialect: "dacode-todo-mcp" }` against the synthetic
/// own tree: the counts response and spot ids asserted in-test (no
/// golden, D4); an unknown dialect value is `invalid` naming it; the
/// absent dialect takes the dacode path — the io fault names the
/// dacode file the own tree does not carry; and an own tree missing
/// its decisions file faults as `io` naming the own file's role.
#[test]
fn import_md_with_dialect_own_drives_the_own_path() {
    let root = TempDir::new().unwrap();
    let tree = own_tree();
    let requests = vec![
        call(
            2,
            "repo_register",
            &register_args("dacode-todo-mcp", tree.path()),
        ),
        call(
            3,
            "import_md",
            r#"{"repo":"dacode-todo-mcp","dialect":"dacode-todo-mcp"}"#,
        ),
        call(4, "graph_show", r#"{"repo":"dacode-todo-mcp"}"#),
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
    assert_eq!(answer["repo"], json!("dacode-todo-mcp"));
    assert_eq!(answer["epics"], json!(6));
    assert_eq!(answer["decisions"], json!(4));
    assert_eq!(answer["docs"], json!(0));
    assert_eq!(answer["tasks"], json!(9));
    assert_eq!(answer["blocks"], json!(4));

    // graph_show's spot checks: the reproduced base id, the debt
    // kind, the dropped body, and the oddball after the base carrying
    // its original id.
    let graph = structured(&driven.responses[2]);
    let repo = &graph["repos"][0];
    let tasks: Vec<&Value> = repo["epics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["tasks"].as_array().unwrap())
        .collect();
    assert_eq!(tasks.len(), 9);
    let u1 = tasks
        .iter()
        .find(|task| task["id"] == json!("dacode-todo-mcp:U1"))
        .unwrap();
    assert_eq!(u1["title"], json!("First follow-up"));
    assert_eq!(u1["kind"], json!("task"));
    assert_eq!(u1["body"], json!(""), "the body prose is dropped");
    let u6 = tasks
        .iter()
        .find(|task| task["id"] == json!("dacode-todo-mcp:U6"))
        .unwrap();
    assert_eq!(u6["kind"], json!("debt"), "§5's items are debt");
    let u8 = tasks
        .iter()
        .find(|task| task["id"] == json!("dacode-todo-mcp:U8"))
        .unwrap();
    assert_eq!(u8["title"], json!("a variant item"));
    assert_eq!(
        u8["references"][0]["target"],
        json!("U4 (original)"),
        "the oddball carries its original id"
    );

    // An unknown dialect value: `invalid` naming it.
    let root = TempDir::new().unwrap();
    let tree = own_tree();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "repo_register",
                &register_args("dacode-todo-mcp", tree.path()),
            ),
            call(
                3,
                "import_md",
                r#"{"repo":"dacode-todo-mcp","dialect":"shelly"}"#,
            ),
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
        "the dialect 'shelly' is not known: 'dacode', 'camera', 'ikvm-html' or 'dacode-todo-mcp'",
    );

    // The absent dialect: the dacode path — the own tree lacks the
    // dacode layout, so the io fault names the dacode file.
    let root = TempDir::new().unwrap();
    let tree = own_tree();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "repo_register",
                &register_args("dacode-todo-mcp", tree.path()),
            ),
            call(3, "import_md", r#"{"repo":"dacode-todo-mcp"}"#),
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

    // An own tree missing its decisions file: the io fault names the
    // own file's role, the same convention the sibling paths pin.
    let root = TempDir::new().unwrap();
    let tree = own_tree();
    fs::remove_file(tree.path().join(BOARD_DIR).join("decisions.md")).unwrap();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "repo_register",
                &register_args("dacode-todo-mcp", tree.path()),
            ),
            call(
                3,
                "import_md",
                r#"{"repo":"dacode-todo-mcp","dialect":"dacode-todo-mcp"}"#,
            ),
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
        "io: read the own decisions file decisions.md: NotFound",
    );
}

// =================================================================
// 7. Determinism.
// =================================================================

/// The same board parsed and mapped twice yields equal ASTs and
/// equal plans — the sibling dialects' pin, this dialect's own.
#[test]
fn mapping_the_same_board_twice_yields_equal_plans() {
    // The premise first: both parses and both mappings succeed, or
    // the equality below would compare two identical failures.
    let first_board = parse_own_board(&todo_md(), DECISIONS_MD).unwrap();
    let second_board = parse_own_board(&todo_md(), DECISIONS_MD).unwrap();
    assert_eq!(first_board, second_board);
    let first = map_own(&first_board, &opts()).unwrap();
    let second = map_own(&second_board, &opts()).unwrap();
    assert_eq!(first, second);
}
