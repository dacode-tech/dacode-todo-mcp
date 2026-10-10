//! The ikvm-html dialect's suite (t47): the parser (TODO.md's walker,
//! decisions.md's three registers), the strictness and never-fired
//! error pins (duplicates, absent registers, column counts, table
//! ids, impossible dates), the four composition cross-checks, the
//! mapping (statuses, kinds, the synthetics, the edges, the
//! findings), the plan order and id reproduction at synthetic live
//! scale, the `dialect` param's wire drive, and determinism. All
//! corpora are hand-authored synthetic strings or loop-generated
//! synthetic boards in the measured dialect — the live ikvm-html
//! board is the import probe's subject (D12), never copied here: no
//! live prose, no golden pinning it.
//!
//! The wire test drives the real binary over a synthetic ikvm-html
//! tree and asserts the counts response and spot ids in-test — no
//! golden (D12's signed consequence: two of the three dialects get no
//! pinned golden) — plus the dispatch and fault pins: an unknown
//! dialect value is `invalid` naming it, and the dialect's
//! missing-file io fault names its own file's role.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{Repo, RepoSlug, Timestamp};
use todo_mcp::import::ikvm::{
    ConfidenceCell, DecisionStatusCell, IkvmError, map_ikvm, parse_decisions, parse_ikvm_board,
    parse_todo_md,
};
use todo_mcp::import::plan::{ImportOptions, PlanOp, apply_plan};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, call, drive, structured};

// =================================================================
// The corpora — a synthetic minimal ikvm-html board composed of
// section pieces, so the error-path corpora can swap one section out
// without duplicating the rest. §1 and §6 are walked past, so their
// content exists to prove exactly that.
// =================================================================

/// The intro and §1: current-state prose, walked past.
const HEAD: &str = "\
# TODO — synthetic

Intro prose, walked past.

## 1. Current state

Prose, zero checkbox items. Walked past.
";

/// §2: the milestone table — six rows, the Gated-on cells narrative.
const SECTION_2: &str = "\
## 2. Milestones

Mirrors the roadmap; the table is the register.

| | Theme | Gated on |
|---|---|---|
| **M0** | Groundwork and crate boundaries | — (D1 closed) |
| **M1** | The engine and its first adapter | — none |
| **M2** | Authentication and role mapping | D3, D4 |
| **M3** | The server and the browser shell | D13 |
| **M4** | Wider vendor coverage | D5, D18 |
| **M5** | Packaging and polish | — |
";

/// §3: the id-less blockers — two checked, two unchecked, one bold
/// run that wraps.
const SECTION_3: &str = "\
## 3. Blockers needing something other than a decision

- [x] **The reference corpus covers both wire directions.** ~~An earlier
      reading said otherwise.~~ The second capture settled it.
- [ ] **Artifacts are missing for five matrix rows** — the vendors that
      publish nothing.
- [x] **The cross-compilation target is absent**, so the portability check
      cannot run.
- [ ] **The vendored libraries are foreign-architecture binaries.** Static
      analysis only.
";

/// §4: the follow-ups register — every shape the title grammar
/// accepts: a plain checked item, a wrapped title, a nested strike
/// with a live bold pair, a backticked `~~`, and a long
/// multi-sentence title.
const SECTION_4: &str = "\
## 4. Unowned follow-ups

- [x] **U1 · Fold the late sections into the plan.** ~~The earlier text.~~
      **Resolved 2026-09-17.** The fold landed.
- [ ] **U2 · Re-examine the module count once two adapters
      exist.** Two adapters exist now.
- [ ] **U3 · A struck count ~~**111**~~ beside a live pair **222** in one
      sentence.** The body follows.
- [ ] **U4 · A title that quotes `~~` without striking.** The
      body discusses the delimiters.
- [ ] **U5 · A long multi-sentence title. The second sentence is body
      prose.** More body.
";

/// §5: the debt register — a struck fragment on an unchecked item
/// (the board's own convention: the strike is edit history).
const SECTION_5: &str = "\
## 5. Debt

- [ ] **U6 · A debt item with a struck fragment ~~412~~ and a corrected
      count.** Debt prose.
- [x] **U7 · A checked debt item.** More debt prose.
";

/// §6: the session narrative, walked past — its checkbox item is
/// body, not a register item, and its foreign-header table is walked
/// past.
const SECTION_6: &str = "\
## 6. Session log

The narrative is walked past. A checkbox here is body, not a register:

- [ ] **U99 · A §6 checkbox item is body text.** Never a register item.

| log | date | note |
|---|---|---|
| t1 | 12 | a foreign-header table, walked past |
";

/// The full synthetic ikvm-html TODO.md.
fn todo_md() -> String {
    format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}")
}

/// The synthetic ikvm-html decisions.md, aligned with the corpus's
/// milestones: five rows exercising all five confidence spellings and
/// both status spellings, five dossiers (four with banners), and the
/// four-row footer.
const DECISIONS_MD: &str = "\
# Decisions

Prose intro, walked past.

## Sign-off summary

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| ~~**D1**~~ | First decision | take the simple path | low | M0 | resolved |
| **D2** | Second decision | gather the numbers first | medium | M1 | open |
| ~~**D3**~~ | Third decision | hold | high | M2 | resolved |
| ~~**D4**~~ | Fourth decision | measure first | low-medium | M3 | resolved |
| ~~**D5**~~ | Fifth decision | defer the framework | medium-high | M4 | resolved |

## Detail

### D1 — First decision

**Gates** M0 · **Action** vote

> **Resolved 2026-09-18 — accepted as recommended: the simple path taken.**
> Applied: the spec updated.

Dossier body for D1.

### D2 — Second decision

**Gates** M1 · **Action** vote

Dossier body for D2.

### D3 — Third decision

**Gates** M2 · **Action** vote

> **Resolved 2026-09-20 — held as recommended**, and the record follows
> in the body.

Dossier body for D3.

### D4 — Fourth decision

**Gates** M3 · **Action** vote

> **Resolved 2026-09-21 — measured first** (the session's own words).

### D5 — Fifth decision

**Gates** M4 · **Action** vote

> **Resolved 2026-09-22 — deferred to M3.**

## Resolved

The prose above the footer is walked past; the table is the truth.

| ID | Resolved | Outcome | Applied to |
|---|---|---|---|
| **D1** | 2026-09-18 | the simple path taken | Spec §2 |
| **D3** | 2026-09-20 | held as recommended | Spec §4 |
| **D4** | 2026-09-21 | measured first | Spec §9 |
| **D5** | 2026-09-22 | deferred to M3 | Spec §7 |
";

/// The import options the mapping tests share: a fixed slug and
/// stamp, an empty `doc_paths` (the board has no doc register).
fn opts() -> ImportOptions {
    ImportOptions {
        slug: RepoSlug::from_str("ikvm").unwrap(),
        imported_at: Timestamp::parse("2026-10-06T12:00:00Z").unwrap(),
        doc_paths: std::collections::BTreeMap::new(),
    }
}

/// The corpus board, parsed and composed.
fn corpus_board() -> todo_mcp::import::ikvm::IkvmBoard {
    parse_ikvm_board(&todo_md(), DECISIONS_MD).unwrap()
}

/// The 1-based line number of the first line containing `needle` in
/// `corpus` — derived from the corpus itself so the corpora's blank
/// lines cannot desynchronize a line assertion.
fn line_of(corpus: &str, needle: &str) -> usize {
    corpus.split('\n').position(|l| l.contains(needle)).unwrap() + 1
}

// =================================================================
// 1–2. The board parser: the registers and the walked-past prose.
// =================================================================

/// §2's census (six rows, ids, titles from Theme), §3's census (four
/// items, titles, checkbox states), §4/§5's census (ids, titles,
/// checkbox states), and §1/§6 walked past — the §6 checkbox is body
/// and the §6 table's foreign header is never a register.
#[test]
fn the_board_parser_extracts_the_registers_and_walks_the_prose_past() {
    let board = parse_todo_md(&todo_md()).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // §2: the milestone rows, in board order, titles from Theme.
    let milestones: Vec<(&str, &str)> = board
        .milestones
        .iter()
        .map(|row| (row.id.as_str(), row.title.as_str()))
        .collect();
    assert_eq!(
        milestones,
        vec![
            ("M0", "Groundwork and crate boundaries"),
            ("M1", "The engine and its first adapter"),
            ("M2", "Authentication and role mapping"),
            ("M3", "The server and the browser shell"),
            ("M4", "Wider vendor coverage"),
            ("M5", "Packaging and polish"),
        ]
    );

    // §3: the blockers — titles from the bold runs (the wrapped run
    // joined), checkbox states, no ids.
    let blockers: Vec<(&str, bool)> = board
        .blockers
        .iter()
        .map(|item| (item.title.as_str(), item.checked))
        .collect();
    assert_eq!(
        blockers,
        vec![
            ("The reference corpus covers both wire directions", true),
            ("Artifacts are missing for five matrix rows", false),
            ("The cross-compilation target is absent", true),
            (
                "The vendored libraries are foreign-architecture binaries",
                false
            ),
        ]
    );
    // The first blocker's body: everything after the closing `**`,
    // the wrap joined.
    assert_eq!(
        board.blockers[0].body,
        "~~An earlier reading said otherwise.~~ The second capture settled it."
    );

    // §4: the items in board order — ids, first-sentence titles,
    // checkbox states.
    let followups: Vec<(&str, &str, bool)> = board
        .followups
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str(), item.checked))
        .collect();
    assert_eq!(
        followups,
        vec![
            ("U1", "Fold the late sections into the plan", true),
            (
                "U2",
                "Re-examine the module count once two adapters exist",
                false
            ),
            (
                "U3",
                "A struck count  beside a live pair **222** in one sentence",
                false
            ),
            ("U4", "A title that quotes `~~` without striking", false),
            ("U5", "A long multi-sentence title", false),
        ]
    );

    // §5: the debt register — the struck fragment on an unchecked
    // item is edit history, not resolution.
    let debt: Vec<(&str, &str, bool)> = board
        .debt
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str(), item.checked))
        .collect();
    assert_eq!(
        debt,
        vec![
            (
                "U6",
                "A debt item with a struck fragment  and a corrected count",
                false
            ),
            ("U7", "A checked debt item", true),
        ]
    );

    // §1/§6 walked past: no U99 anywhere — the §6 checkbox is body.
    let ids: Vec<&str> = board
        .followups
        .iter()
        .chain(board.debt.iter())
        .map(|item| item.id.as_str())
        .collect();
    assert_eq!(ids, vec!["U1", "U2", "U3", "U4", "U5", "U6", "U7"]);
}

// =================================================================
// 3. The register census at synthetic live scale.
// =================================================================

/// The loop-generated live-scale corpus: 128 U items — 57 in §4, 71
/// in §5, the ids interleaved across the sections and each section's
/// file order rotated by one (the file order is not id order) — with
/// the measured title shapes planted at deterministic positions, 4
/// blockers, 6 milestones, 22 decisions (20 resolved, 2 open) and the
/// 20-row footer. Returns the TODO.md and decisions.md texts.
fn live_scale_corpus() -> (String, String) {
    let mut section4: Vec<u32> = (1..=128).filter(|n| n % 2 == 1 && *n <= 113).collect();
    let mut section5: Vec<u32> = (1..=128).filter(|n| n % 2 == 0 || *n > 113).collect();
    // The planted inversions: each section's last item moves to the
    // front, so neither section's file order is id order.
    if let Some(last) = section4.pop() {
        section4.insert(0, last);
    }
    if let Some(last) = section5.pop() {
        section5.insert(0, last);
    }
    let mut todo = String::from(HEAD);
    todo.push_str(SECTION_2);
    todo.push_str(SECTION_3);
    todo.push_str("## 4. Unowned follow-ups\n\nGenerated base items.\n\n");
    for n in section4 {
        todo.push_str(&register_item(n));
    }
    todo.push_str("\n## 5. Debt\n\nGenerated debt items.\n\n");
    for n in section5 {
        todo.push_str(&register_item(n));
    }
    todo.push_str(SECTION_6);
    (todo, live_scale_decisions())
}

/// One loop-generated register item for id `n`, checked per the live
/// census (57 checked), with the measured title shapes planted at
/// deterministic positions: a nested strike, a backticked `~~`, a
/// wrapped title, a long multi-sentence title.
fn register_item(n: u32) -> String {
    let marker = if n <= 57 { "- [x]" } else { "- [ ]" };
    match n % 16 {
        3 => format!("{marker} **U{n} · A count ~~**{n}00**~~ corrected in place.** Body {n}.\n"),
        7 => format!("{marker} **U{n} · About `~~` delimiters in a title.** Body {n}.\n"),
        11 => format!(
            "{marker} **U{n} · A wrapped title that continues\n      on a second line.** Body {n}.\n"
        ),
        13 => format!(
            "{marker} **U{n} · A long title with several clauses that runs on. The rest is body.** Body {n}.\n"
        ),
        _ => format!("{marker} **U{n} · Synthetic item {n} with a sentence.** Body {n}.\n"),
    }
}

/// The live-scale decisions.md: 22 rows (D5 and D18 open, mirroring
/// the live board), the Gates cells at the live per-milestone counts
/// (8/6/2/4/2), 22 dossiers with 20 banners, and the 20-row footer
/// with matching dates.
fn live_scale_decisions() -> String {
    let confidences = ["low", "medium", "high", "low-medium", "medium-high"];
    let mut out = String::from("# Decisions\n\nProse intro.\n\n## Sign-off summary\n\n");
    out.push_str(
        "| ID | Decision | Recommendation | Confidence | Gates | Status |\n|---|---|---|---|---|---|\n",
    );
    for n in 1..=22u32 {
        let resolved = n != 5 && n != 18;
        let id_cell = if resolved {
            format!("~~**D{n}**~~")
        } else {
            format!("**D{n}**")
        };
        let status = if resolved { "resolved" } else { "open" };
        out.push_str(&format!(
            "| {id_cell} | Decision {n} | Recommendation {n} | {} | {} | {status} |\n",
            confidences[((n - 1) % 5) as usize],
            milestone_for(n),
        ));
    }
    out.push_str("\n## Detail\n\n");
    for n in 1..=22u32 {
        let resolved = n != 5 && n != 18;
        out.push_str(&format!(
            "### D{n} — Decision {n}\n\n**Gates** {} · **Action** vote\n\n",
            milestone_for(n)
        ));
        if resolved {
            out.push_str(&format!(
                "> **Resolved 2026-09-{n:02} — accepted as recommended.**\n\n"
            ));
        }
        out.push_str(&format!("Dossier body for D{n}.\n\n"));
    }
    out.push_str("## Resolved\n\n| ID | Resolved | Outcome | Applied to |\n|---|---|---|---|\n");
    for n in 1..=22u32 {
        if n != 5 && n != 18 {
            out.push_str(&format!(
                "| **D{n}** | 2026-09-{n:02} | Spec updated for {n} | Spec §{n} |\n"
            ));
        }
    }
    out
}

/// The Gates cell for decision `n` at the live per-milestone counts
/// (8×M0, 6×M1, 2×M2, 4×M3, 2×M4).
fn milestone_for(n: u32) -> &'static str {
    match n {
        1..=8 => "M0",
        9..=14 => "M1",
        15..=16 => "M2",
        17..=20 => "M3",
        _ => "M4",
    }
}

/// The §4/§5 census at synthetic live scale: 57 + 71 items, the ids
/// U1–U128 with zero gaps, the statuses at the live census (57
/// checked), and the planted shapes parsing.
#[test]
fn the_register_census_at_synthetic_live_scale() {
    let (todo, decisions) = live_scale_corpus();
    let board = parse_ikvm_board(&todo, &decisions).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(board.todo.followups.len(), 57);
    assert_eq!(board.todo.debt.len(), 71);
    let mut ids: Vec<u32> = board
        .todo
        .followups
        .iter()
        .chain(board.todo.debt.iter())
        .map(|item| item.id[1..].parse().unwrap())
        .collect();
    ids.sort_unstable();
    let expected: Vec<u32> = (1..=128).collect();
    assert_eq!(ids, expected, "the register is U1–U128 with zero gaps");
    let checked = board
        .todo
        .followups
        .iter()
        .chain(board.todo.debt.iter())
        .filter(|item| item.checked)
        .count();
    assert_eq!(checked, 57);
    // The planted shapes parse: the nested strike, the backticked
    // `~~`, the wrapped title and the long title all land.
    let all: Vec<&todo_mcp::import::ikvm::RegisterItem> = board
        .todo
        .followups
        .iter()
        .chain(board.todo.debt.iter())
        .collect();
    let find = |n: u32| {
        all.iter()
            .find(|item| item.id == format!("U{n}"))
            .unwrap_or_else(|| panic!("U{n} is in the register"))
    };
    assert_eq!(find(3).title, "A count  corrected in place");
    assert_eq!(find(7).title, "About `~~` delimiters in a title");
    assert_eq!(
        find(11).title,
        "A wrapped title that continues on a second line"
    );
    assert_eq!(
        find(13).title,
        "A long title with several clauses that runs on"
    );
    // The inversions: the first item of each section is its rotated
    // last id, not the smallest.
    assert_eq!(board.todo.followups[0].id, "U113");
    assert_eq!(board.todo.debt[0].id, "U128");
}

// =================================================================
// 4–5, 9–10. The strictness pins.
// =================================================================

/// A duplicate U id across §4 and §5 combined is a named error naming
/// both lines — one U namespace spans the sections, so the duplicate
/// check is the walker's.
#[test]
fn a_duplicate_register_id_across_the_sections_is_a_named_error() {
    let corpus = todo_md().replace(
        "- [ ] **U6 · A debt item with a struck fragment ~~412~~ and a corrected\n      count.** Debt prose.",
        "- [ ] **U2 · A cross-section duplicate** — debt prose.",
    );
    let first = line_of(&corpus, "**U2 · Re-examine");
    let second = line_of(&corpus, "**U2 · A cross-section duplicate**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::DuplicateId {
            id: "U2".to_owned(),
            first,
            second,
        })
    );
}

/// A suffixed id is a named error, not an oddball: the register is
/// measured oddball-free, so `U<digits>` exactly is the honest shape.
#[test]
fn a_suffixed_id_is_a_named_error_not_an_oddball() {
    let corpus = todo_md().replace(
        "- [ ] **U2 · Re-examine the module count once two adapters\n      exist.** Two adapters exist now.",
        "- [ ] **U2a · A suffixed id** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MalformedLine {
            line: line_of(&corpus, "**U2a ·"),
            detail: "the bold run is not U-shaped: 'U2a' is not a U id".to_owned(),
        })
    );
}

/// A duplicate M id in §2's table is a named error naming both lines
/// — the register is measured duplicate-free, so strictness is the
/// honest reading.
#[test]
fn a_duplicate_milestone_id_is_a_named_error() {
    let corpus = todo_md().replace(
        "| **M5** | Packaging and polish | — |",
        "| **M4** | Packaging and polish | — |",
    );
    let first = line_of(&corpus, "| **M4** | Wider vendor coverage");
    let second = line_of(&corpus, "| **M4** | Packaging and polish");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::DuplicateId {
            id: "M4".to_owned(),
            first,
            second,
        })
    );
}

/// A duplicated D id in the decisions summary table is a named error
/// naming both lines — the composition's id cross-check compares sets
/// (which dedupe), so a silent duplicate would pass it and shift the
/// mapping's decision id range past the board's numbering.
#[test]
fn a_duplicate_decisions_summary_id_is_a_named_error() {
    let decisions = DECISIONS_MD.replace(
        "| ~~**D5**~~ | Fifth decision | defer the framework | medium-high | M4 | resolved |",
        "| ~~**D1**~~ | a duplicate row | hold | low | M0 | resolved |",
    );
    let first = line_of(&decisions, "| ~~**D1**~~ | First decision");
    let second = line_of(&decisions, "| ~~**D1**~~ | a duplicate row");
    assert_eq!(
        parse_decisions(&decisions),
        Err(IkvmError::DuplicateId {
            id: "D1".to_owned(),
            first,
            second,
        })
    );
}

/// A duplicated D id in the dossiers is a named error naming both
/// lines — the same set-equality blind spot the summary table's
/// duplicate rides.
#[test]
fn a_duplicate_dossier_header_id_is_a_named_error() {
    let decisions = DECISIONS_MD.replace("### D5 — Fifth decision", "### D1 — a duplicate dossier");
    let first = line_of(&decisions, "### D1 — First decision");
    let second = line_of(&decisions, "### D1 — a duplicate dossier");
    assert_eq!(
        parse_decisions(&decisions),
        Err(IkvmError::DuplicateId {
            id: "D1".to_owned(),
            first,
            second,
        })
    );
}

/// The never-fired strictness errors the module docs name: an absent
/// register, a wrong column count, a malformed table id, a § carried
/// twice, and an impossible banner date — each a named error, never a
/// silent skip.
#[test]
fn the_never_fired_strictness_errors_are_named() {
    // §2's table absent: MissingRegister naming it, at the input's
    // end line.
    let corpus = todo_md().replace("| | Theme | Gated on |\n|---|---|---|\n", "");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: corpus.split('\n').count(),
        })
    );

    // A milestone row with the wrong cell count.
    let corpus = todo_md().replace(
        "| **M0** | Groundwork and crate boundaries | — (D1 closed) |",
        "| **M0** | Groundwork and crate boundaries |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::ColumnCount {
            line: line_of(&corpus, "| **M0** | Groundwork"),
            expected: 3,
            found: 2,
        })
    );

    // A letter-suffixed M id: the strict shape, a named error.
    let corpus = todo_md().replace(
        "| **M0** | Groundwork and crate boundaries | — (D1 closed) |",
        "| **M0a** | Groundwork and crate boundaries | — (D1 closed) |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::BadId {
            line: line_of(&corpus, "| **M0a** | Groundwork"),
            id: "M0a".to_owned(),
            detail: "not an M id".to_owned(),
        })
    );

    // A section carried twice.
    let corpus = todo_md().replace("## 5. Debt\n", "## 4. Unowned follow-ups\n## 5. Debt\n");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MalformedLine {
            line: line_of(&corpus, "## 5. Debt") - 1,
            detail: "TODO.md carries §4 twice".to_owned(),
        })
    );

    // An impossible banner date: the parser accepts the shape (the
    // month is two digits), the mapping's calendar check refuses it.
    let decisions = DECISIONS_MD.replace("2026-09-18", "2026-13-18");
    let board = parse_ikvm_board(&todo_md(), &decisions).unwrap();
    assert_eq!(
        map_ikvm(&board, &opts()),
        Err(IkvmError::BadDate {
            date: "2026-13-18".to_owned(),
        })
    );
}

/// Malformed register lines are named errors: a checkbox with no
/// bold run, a bold run with no ` · ` separator, and a title with no
/// sentence-ending period.
#[test]
fn malformed_register_lines_are_named_errors() {
    // No bold run: the marker is followed by plain prose.
    let corpus = todo_md().replace(
        "- [ ] **U2 · Re-examine the module count once two adapters\n      exist.** Two adapters exist now.",
        "- [ ] Plain prose with no bold run.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MalformedLine {
            line: line_of(&corpus, "Plain prose with no bold run"),
            detail: "a U item's bold run is missing".to_owned(),
        })
    );

    // No ` · ` separator.
    let corpus = todo_md().replace(
        "- [ ] **U2 · Re-examine the module count once two adapters\n      exist.** Two adapters exist now.",
        "- [ ] **U2 has no separator.** Body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MalformedLine {
            line: line_of(&corpus, "**U2 has no separator"),
            detail: "the bold run is not U-shaped: no ' · ' separator".to_owned(),
        })
    );

    // No sentence-ending period: the item's span carries no `.` the
    // title rule can cut at.
    let corpus = todo_md().replace(
        "- [ ] **U2 · Re-examine the module count once two adapters\n      exist.** Two adapters exist now.",
        "- [ ] **U2 · A title with no period at all** — body prose",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MalformedLine {
            line: line_of(&corpus, "**U2 · A title with no period"),
            detail: "a U item's title has no sentence-ending period".to_owned(),
        })
    );
}

// =================================================================
// 6–7. The title grammar pins.
// =================================================================

/// The first-sentence titles: the nested strike removed, the live
/// bold pair kept, the backticked `~~` literal, the wrapped run
/// joined, the long title cut at its first period. Pinned by the
/// census test above; this pin holds the error side — a §3 bold run
/// that does not close is a named error.
#[test]
fn an_unclosed_blocker_bold_run_is_a_named_error() {
    let corpus = todo_md().replace(
        "- [ ] **The vendored libraries are foreign-architecture binaries.** Static\n      analysis only.",
        "- [ ] **A bold run that never closes\n- [ ] **The next item** body",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MalformedLine {
            line: line_of(&corpus, "**A bold run that never closes"),
            detail: "a §3 item's bold run does not close".to_owned(),
        })
    );
}

// =================================================================
// 8. Section scoping.
// =================================================================

/// §6's checkbox item is body, not a register item, and §6's table
/// with a foreign header is walked past — pinned by the census test's
/// U99 absence; this pin holds the missing-section side: an absent §3
/// is a named error naming it.
#[test]
fn an_absent_imported_section_is_a_named_error() {
    let corpus = format!("{HEAD}{SECTION_2}{SECTION_4}{SECTION_5}{SECTION_6}");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IkvmError::MissingSection {
            section: "§3".to_owned(),
            line: corpus.split('\n').count(),
        })
    );
}

// =================================================================
// 11, 13. The decisions parser.
// =================================================================

/// The decisions census: the summary rows (ids, struck flags, titles,
/// recommendations, confidences, gates, statuses), the dossiers
/// (headers, Gates lines, banners, bodies), and the footer — plus the
/// five Confidence spellings parsing and the never-fired pins (a bad
/// Status cell, a bad Confidence spelling, an empty Recommendation).
#[test]
fn the_decisions_parser_reads_the_three_registers() {
    let file = parse_decisions(DECISIONS_MD).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // The summary rows, in board order, every cell parsed.
    assert_eq!(file.summary.len(), 5);
    assert_eq!(file.summary[0].id, "D1");
    assert!(file.summary[0].struck, "the resolved row's id is struck");
    assert_eq!(file.summary[0].title, "First decision");
    assert_eq!(file.summary[0].recommendation, "take the simple path");
    assert_eq!(file.summary[0].confidence, ConfidenceCell::Low);
    assert_eq!(file.summary[0].gates, "M0");
    assert_eq!(file.summary[0].status, DecisionStatusCell::Resolved);
    assert!(!file.summary[1].struck, "the open row's id is not struck");
    assert_eq!(file.summary[1].status, DecisionStatusCell::Open);
    assert_eq!(file.summary[1].confidence, ConfidenceCell::Medium);
    assert_eq!(file.summary[2].confidence, ConfidenceCell::High);
    assert_eq!(file.summary[3].confidence, ConfidenceCell::LowMedium);
    assert_eq!(file.summary[4].confidence, ConfidenceCell::MediumHigh);
    assert_eq!(file.summary[4].gates, "M4");

    // The dossiers: ids, titles, Gates lines, banners, bodies. The
    // banner closings are the measured grammar's three spellings:
    // `.**` (D1, D5), `**,` (D3) and `** (` (D4) — the live probe's
    // strictness find, widened and pinned.
    assert_eq!(file.dossiers.len(), 5);
    assert_eq!(file.dossiers[0].id, "D1");
    assert_eq!(file.dossiers[0].title, "First decision");
    assert_eq!(file.dossiers[0].gates, "M0");
    let banner = file.dossiers[0].resolved.as_ref().unwrap();
    assert_eq!(banner.date, "2026-09-18");
    assert_eq!(
        banner.outcome,
        "accepted as recommended: the simple path taken"
    );
    assert_eq!(
        file.dossiers[1].resolved, None,
        "the open decision has no banner"
    );
    let banner = file.dossiers[2].resolved.as_ref().unwrap();
    assert_eq!(banner.outcome, "held as recommended", "the '],' closing");
    let banner = file.dossiers[3].resolved.as_ref().unwrap();
    assert_eq!(banner.outcome, "measured first", "the ' (' closing");
    assert!(file.dossiers[0].body.contains("Dossier body for D1."));
    assert!(file.dossiers[0].body.contains("**Gates** M0"));

    // The footer: four rows, the dates matching the banners.
    assert_eq!(file.footer.len(), 4);
    assert_eq!(file.footer[0].id, "D1");
    assert_eq!(file.footer[0].date, "2026-09-18");
    assert_eq!(file.footer[0].outcome, "the simple path taken");
    assert_eq!(file.footer[0].applied_to, "Spec §2");

    // The composition's cross-checks pass on the aligned corpus.
    assert!(parse_ikvm_board(&todo_md(), DECISIONS_MD).is_ok());

    // A bad Status cell.
    let bad_status = DECISIONS_MD.replace("| medium | M1 | open |", "| medium | M1 | pending |");
    assert_eq!(
        parse_decisions(&bad_status),
        Err(IkvmError::BadStatus {
            line: line_of(&bad_status, "| medium | M1 | pending |"),
            text: "pending".to_owned(),
            detail: "the Status cell must read 'resolved' or 'open'".to_owned(),
        })
    );

    // A bad Confidence spelling.
    let bad_confidence = DECISIONS_MD.replace("| low | M0 | resolved |", "| med | M0 | resolved |");
    assert_eq!(
        parse_decisions(&bad_confidence),
        Err(IkvmError::BadStatus {
            line: line_of(&bad_confidence, "| med | M0 | resolved |"),
            text: "med".to_owned(),
            detail: "the Confidence cell must read 'low', 'medium', 'high', 'low-medium' or 'medium-high'"
                .to_owned(),
        })
    );

    // An empty Recommendation cell.
    let bad_recommendation =
        DECISIONS_MD.replacen("| take the simple path | low | M0 |", "|  | low | M0 |", 1);
    assert_eq!(
        parse_decisions(&bad_recommendation),
        Err(IkvmError::MalformedLine {
            line: line_of(&bad_recommendation, "|  | low | M0 |"),
            detail: "the Recommendation cell is empty".to_owned(),
        })
    );
}

// =================================================================
// 12. The four cross-checks.
// =================================================================

/// Each cross-check fires on a synthetic mismatch, naming the
/// smallest offending id: a dossier the table does not carry, a
/// banner the struck set does not, a banner date the footer
/// contradicts, and a Gates cell its dossier's line contradicts.
#[test]
fn the_four_cross_checks_fire_on_synthetic_mismatches() {
    // 1. A dossier the summary table does not carry.
    let extra = format!(
        "{DECISIONS_MD}### D6 — Sixth decision\n\n**Gates** M0 · **Action** vote\n\nDossier body for D6.\n"
    );
    assert_eq!(
        parse_ikvm_board(&todo_md(), &extra),
        Err(IkvmError::IdSetMismatch {
            id: "D6".to_owned(),
            detail: "D6 is in the decisions dossiers but not in the decisions summary table"
                .to_owned(),
        })
    );

    // 2. A banner the struck set does not: D2 is open (unstruck) but
    // its dossier carries a banner.
    let bannered = DECISIONS_MD.replace(
        "Dossier body for D2.",
        "> **Resolved 2026-09-19 — accepted as recommended.**\n\nDossier body for D2.",
    );
    assert_eq!(
        parse_ikvm_board(&todo_md(), &bannered),
        Err(IkvmError::IdSetMismatch {
            id: "D2".to_owned(),
            detail: "D2 is in the bannered dossiers but not in the summary table's struck ids or the resolved footer"
                .to_owned(),
        })
    );

    // 3. A banner date the footer contradicts.
    let mismatched_date = DECISIONS_MD.replace(
        "| **D1** | 2026-09-18 | the simple path taken | Spec §2 |",
        "| **D1** | 2026-09-19 | the simple path taken | Spec §2 |",
    );
    assert_eq!(
        parse_ikvm_board(&todo_md(), &mismatched_date),
        Err(IkvmError::IdSetMismatch {
            id: "D1".to_owned(),
            detail: "D1's resolution banner says 2026-09-18 but the footer says 2026-09-19"
                .to_owned(),
        })
    );

    // 4. A Gates cell its dossier's line contradicts.
    let mismatched_gates = DECISIONS_MD.replace(
        "**Gates** M0 · **Action** vote",
        "**Gates** M1 · **Action** vote",
    );
    assert_eq!(
        parse_ikvm_board(&todo_md(), &mismatched_gates),
        Err(IkvmError::IdSetMismatch {
            id: "D1".to_owned(),
            detail: "D1's Gates cell says M0 but its dossier's Gates line says M1".to_owned(),
        })
    );
}

// =================================================================
// 14–15. The plan census and id reproduction at synthetic live scale.
// =================================================================

/// The plan census at synthetic live scale: 9 epics, 22 decisions,
/// 132 tasks, 22 edges, 0 findings — the live board's census.
#[test]
fn the_plan_census_at_synthetic_live_scale() {
    let (todo, decisions) = live_scale_corpus();
    let board = parse_ikvm_board(&todo, &decisions).unwrap();
    let plan = map_ikvm(&board, &opts()).unwrap();
    let counts = (
        plan.ops
            .iter()
            .filter(|op| matches!(op, PlanOp::AddEpic { .. }))
            .count(),
        plan.ops
            .iter()
            .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
            .count(),
        plan.ops
            .iter()
            .filter(|op| matches!(op, PlanOp::AddTask { .. }))
            .count(),
        plan.ops
            .iter()
            .filter(|op| matches!(op, PlanOp::LinkBlocks { .. }))
            .count(),
    );
    assert_eq!(counts, (9, 22, 132, 22));
    let findings = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, .. } => Some(draft),
            _ => None,
        })
        .filter(|draft| draft.kind == todo_domain::TaskKind::Finding)
        .count();
    assert_eq!(findings, 0, "every Gates cell names a present milestone");
}

/// The id reproduction at synthetic live scale: applied to a fresh
/// temp store, the store's task ids are exactly U1–U128 (the board's
/// numbering reproduced) then U129–U132 (the blockers), the epic ids
/// E1–E9, the decision ids D1–D22 — the file order's planted
/// inversions prove the sorts are load-bearing.
#[test]
fn the_id_reproduction_at_synthetic_live_scale() {
    let (todo, decisions) = live_scale_corpus();
    let board = parse_ikvm_board(&todo, &decisions).unwrap();
    let plan = map_ikvm(&board, &opts()).unwrap();

    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str("ikvm").unwrap(),
            name: "ikvm".to_owned(),
            path: "/repos/ikvm".to_owned(),
            registered_at: Timestamp::parse("2026-10-06T00:00:00Z").unwrap(),
        })
        .unwrap();
    apply_plan(&plan, &mut store).unwrap();

    let tasks: Vec<&todo_domain::Task> = store.graph().tasks().collect();
    let ids: Vec<String> = tasks.iter().map(|task| task.id.to_string()).collect();
    let expected: Vec<String> = (1..=132).map(|n| format!("ikvm:U{n}")).collect();
    assert_eq!(ids, expected);
    // The reproduction is visible on the titles: the store's U<n> is
    // the board's U<n> for every register item.
    for n in 1..=128u32 {
        let task = tasks
            .iter()
            .find(|task| task.id.to_string() == format!("ikvm:U{n}"))
            .unwrap();
        let expected_title: String = match n % 16 {
            3 => "A count  corrected in place".to_owned(),
            7 => "About `~~` delimiters in a title".to_owned(),
            11 => "A wrapped title that continues on a second line".to_owned(),
            13 => "A long title with several clauses that runs on".to_owned(),
            _ => format!("Synthetic item {n} with a sentence"),
        };
        assert_eq!(task.title, expected_title, "U{n}");
    }
    // The blockers after the register, in board order.
    assert_eq!(
        tasks[128].title,
        "The reference corpus covers both wire directions"
    );
    assert_eq!(
        tasks[131].title,
        "The vendored libraries are foreign-architecture binaries"
    );

    let epics: Vec<String> = store
        .graph()
        .epics()
        .map(|epic| epic.id.to_string())
        .collect();
    let expected: Vec<String> = (1..=9).map(|n| format!("ikvm:E{n}")).collect();
    assert_eq!(epics, expected);

    let decisions: Vec<String> = store
        .graph()
        .decisions()
        .map(|decision| decision.id.to_string())
        .collect();
    let expected: Vec<String> = (1..=22).map(|n| format!("ikvm:D{n}")).collect();
    assert_eq!(decisions, expected);

    assert_eq!(store.graph().docs().count(), 0);
    assert_eq!(store.graph().blocks_edges().len(), 22);
}

// =================================================================
// 16–18, 20. The mapping conventions.
// =================================================================

/// The mapping's conventions: the task statuses (the checkbox
/// governs, the strike ignored), the kinds and epic assignment per
/// section, the evidence and verify literals, the decision statuses
/// (resolved → Signed with the banner's outcome and date, open →
/// Open), the confidence mapping (the intermediates → Medium), and
/// the timestamps (banner dates → midnight, everything else
/// imported_at).
#[test]
fn the_mapping_pins_statuses_kinds_literals_and_timestamps() {
    let board = corpus_board();
    let plan = map_ikvm(&board, &opts()).unwrap();
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
        (9, 5, 11, 5)
    );
    let first_edge = ops
        .iter()
        .position(|op| matches!(op, PlanOp::LinkBlocks { .. }))
        .unwrap();
    assert_eq!(first_edge, epic_count + decision_count + task_count);

    // The task ops: the register items then the blockers, the
    // checkbox governing status, the kinds and epics per section.
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
            "ikvm:U1", "ikvm:U2", "ikvm:U3", "ikvm:U4", "ikvm:U5", "ikvm:U6", "ikvm:U7", "ikvm:U8",
            "ikvm:U9", "ikvm:U10", "ikvm:U11",
        ]
    );
    // Every task's stamp is the caller-passed imported_at — the
    // register items carry no board dates the conventions read.
    for op in plan
        .ops
        .iter()
        .skip(epic_count + decision_count)
        .take(task_count)
    {
        let PlanOp::AddTask { at, .. } = op else {
            unreachable!("filtered above");
        };
        assert_eq!(at.to_string(), "2026-10-06T12:00:00.000000000Z");
    }
    // U1: checked → closed with the fixed evidence literal; §4 → the
    // task kind under Follow-ups; the verify literal; the body not
    // imported.
    let (_, u1) = tasks[0];
    assert_eq!(u1.title, "Fold the late sections into the plan");
    assert_eq!(u1.status, todo_domain::Status::Closed);
    assert_eq!(u1.evidence.as_deref(), Some("imported checked"));
    assert_eq!(
        u1.verify.as_deref(),
        Some("imported: the board's checkbox alone")
    );
    assert_eq!(u1.body, "");
    assert_eq!(u1.kind, todo_domain::TaskKind::Task);
    assert_eq!(u1.epic.to_string(), "ikvm:E7");
    assert_eq!(u1.references.iter().count(), 0);
    // U2: unchecked → open, no evidence.
    let (_, u2) = tasks[1];
    assert_eq!(u2.status, todo_domain::Status::Open);
    assert_eq!(u2.evidence, None);
    // U6: §5 → the debt kind under Debt; the struck fragment on an
    // unchecked item does not close it.
    let (_, u6) = tasks[5];
    assert_eq!(u6.kind, todo_domain::TaskKind::Debt);
    assert_eq!(u6.epic.to_string(), "ikvm:E8");
    assert_eq!(
        u6.status,
        todo_domain::Status::Open,
        "the strike is edit history"
    );
    // U8: the first blocker → the task kind under Blockers, no
    // references entry (no original id).
    let (_, u8) = tasks[7];
    assert_eq!(u8.title, "The reference corpus covers both wire directions");
    assert_eq!(u8.kind, todo_domain::TaskKind::Task);
    assert_eq!(u8.epic.to_string(), "ikvm:E9");
    assert_eq!(u8.status, todo_domain::Status::Closed);
    assert_eq!(u8.references.iter().count(), 0);

    // The decision ops: resolved → Signed with the banner's outcome
    // and date; open → Open at the import stamp; the recommendation
    // and confidence carried; no options.
    let decisions: Vec<(&todo_mcp::import::plan::PlanOp, String)> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddDecision { id, .. } => Some((op, id.to_string())),
            _ => None,
        })
        .collect();
    let PlanOp::AddDecision { draft, at, .. } = decisions[0].0 else {
        unreachable!("filtered above");
    };
    assert_eq!(decisions[0].1, "ikvm:D1");
    assert_eq!(draft.title, "First decision");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Signed);
    assert_eq!(
        draft.resolution.as_deref(),
        Some("accepted as recommended: the simple path taken")
    );
    assert_eq!(
        draft.recommendation.as_deref(),
        Some("take the simple path")
    );
    assert_eq!(draft.confidence, todo_domain::Confidence::Low);
    assert_eq!(draft.options, Vec::<String>::new());
    assert_eq!(at.to_string(), "2026-09-18T00:00:00.000000000Z");
    let PlanOp::AddDecision { draft, at, .. } = decisions[1].0 else {
        unreachable!("filtered above");
    };
    assert_eq!(decisions[1].1, "ikvm:D2");
    assert_eq!(draft.status, todo_domain::DecisionStatus::Open);
    assert_eq!(draft.resolution, None);
    assert_eq!(draft.confidence, todo_domain::Confidence::Medium);
    assert_eq!(at.to_string(), "2026-10-06T12:00:00.000000000Z");
    // The confidence intermediates → Medium.
    let PlanOp::AddDecision { draft, .. } = decisions[3].0 else {
        unreachable!("filtered above");
    };
    assert_eq!(
        draft.confidence,
        todo_domain::Confidence::Medium,
        "low-medium → Medium"
    );
    let PlanOp::AddDecision { draft, .. } = decisions[4].0 else {
        unreachable!("filtered above");
    };
    assert_eq!(
        draft.confidence,
        todo_domain::Confidence::Medium,
        "medium-high → Medium"
    );

    // The edge ops: the five Gates edges (D-id order), the
    // constructed ids.
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
                from: "ikvm:D1".parse().unwrap(),
                to: "ikvm:E1".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "ikvm:D2".parse().unwrap(),
                to: "ikvm:E2".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "ikvm:D3".parse().unwrap(),
                to: "ikvm:E3".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "ikvm:D4".parse().unwrap(),
                to: "ikvm:E4".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "ikvm:D5".parse().unwrap(),
                to: "ikvm:E5".parse().unwrap(),
            },
        ]
    );
}

/// All nine epics are open — no milestone status is derivable from
/// the table (it has no State column), and the synthetics are
/// structural containers for open work.
#[test]
fn all_nine_epics_are_open() {
    let board = corpus_board();
    let plan = map_ikvm(&board, &opts()).unwrap();
    let epics: Vec<(String, &todo_domain::EpicDraft)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddEpic { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    assert_eq!(epics.len(), 9);
    for (id, draft) in &epics {
        assert_eq!(draft.status, todo_domain::Status::Open, "{id}");
    }
    // Every epic's stamp is the caller-passed imported_at — the
    // epics carry no board dates of their own.
    for op in plan.ops.iter().take(epics.len()) {
        let PlanOp::AddEpic { at, .. } = op else {
            unreachable!("filtered above");
        };
        assert_eq!(at.to_string(), "2026-10-06T12:00:00.000000000Z");
    }
    // The milestone epics' descriptions carry the M id; the
    // synthetics' descriptions carry the provenance.
    assert_eq!(
        epics[0].1.description,
        "the ikvm-html board's §2 milestone M0"
    );
    assert_eq!(
        epics[5].1.description,
        "the ikvm-html board's §2 milestone M5"
    );
    assert_eq!(epics[6].1.title, "Follow-ups");
    assert_eq!(
        epics[6].1.description,
        "the ikvm-html board's §4 follow-ups, not tied to a milestone"
    );
    assert_eq!(epics[7].1.title, "Debt");
    assert_eq!(
        epics[7].1.description,
        "the ikvm-html board's §5 debt, recorded rather than forgotten"
    );
    assert_eq!(epics[8].1.title, "Blockers");
    assert_eq!(
        epics[8].1.description,
        "the ikvm-html board's §3 blockers, needing something other than a decision"
    );
}

// =================================================================
// 19. The findings machinery.
// =================================================================

/// A Gates cell naming no §2 milestone becomes a finding task under
/// `Follow-ups`, numbered after the blockers, the dossier's body as
/// the context — never a silent drop.
#[test]
fn a_gates_cell_naming_no_milestone_becomes_a_finding() {
    let decisions = DECISIONS_MD.replace(
        "| ~~**D3**~~ | Third decision | hold | high | M2 | resolved |",
        "| ~~**D3**~~ | Third decision | hold | high | M9 | resolved |",
    );
    let decisions = decisions.replace(
        "**Gates** M2 · **Action** vote",
        "**Gates** M9 · **Action** vote",
    );
    let board = parse_ikvm_board(&todo_md(), &decisions).unwrap();
    let plan = map_ikvm(&board, &opts()).unwrap();

    // Twelve tasks: the seven register items, the four blockers, then
    // the one finding.
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    assert_eq!(tasks.len(), 12);
    let (finding_id, finding) = (&tasks[11].0, tasks[11].1);
    assert_eq!(finding_id, "ikvm:U12");
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "D3 gates M9");
    assert_eq!(finding.epic.to_string(), "ikvm:E7");
    assert_eq!(finding.status, todo_domain::Status::Open);
    assert_eq!(
        finding.verify.as_deref(),
        Some("imported: the board's checkbox alone")
    );
    // The dossier's raw body, verbatim — the blank lines around the
    // prose lines included, as the lines join.
    assert!(finding.body.contains("Dossier body for D3."));
    let targets: Vec<&str> = finding.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["D3"]);

    // D3's unresolvable Gates cell yields no edge; the four
    // resolvable Gates edges remain.
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 4);
}

// =================================================================
// 21. Determinism.
// =================================================================

/// The same board parsed and mapped twice yields equal ASTs and
/// equal plans — the dacode and camera pins, this dialect's own.
#[test]
fn mapping_the_same_board_twice_yields_equal_plans() {
    // The premise first: both parses and both mappings succeed, or
    // the equality below would compare two identical failures.
    let first_board = parse_ikvm_board(&todo_md(), DECISIONS_MD).unwrap();
    let second_board = parse_ikvm_board(&todo_md(), DECISIONS_MD).unwrap();
    assert_eq!(first_board, second_board);
    let first = map_ikvm(&first_board, &opts()).unwrap();
    let second = map_ikvm(&second_board, &opts()).unwrap();
    assert_eq!(first, second);
}

// =================================================================
// 22. The helpers widening.
// =================================================================

/// A backticked `~~` does not toggle a strike: the title keeps its
/// code span verbatim and the first-sentence cut passes it (the
/// census test's U4 pin), and `remove_struck_spans` keeps removing
/// real spans (the U3 and U6 pins) — this pin holds the negative
/// side, a strike that would swallow the title if the backtick were
/// not code.
#[test]
fn a_backticked_strike_does_not_toggle_a_strike() {
    // The backticked `~~` is literal: the title after it survives
    // struck-span removal — without the widening, the backticked
    // pair toggles a strike that runs to the end of the text,
    // swallowing the title's period, so the parse refuses with the
    // no-sentence-ending-period error rather than truncating.
    let corpus = todo_md().replace(
        "- [ ] **U4 · A title that quotes `~~` without striking.** The\n      body discusses the delimiters.",
        "- [ ] **U4 · Before `~~` and after the pair the title continues to its end.** Body.",
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(
        board.followups[3].title,
        "Before `~~` and after the pair the title continues to its end"
    );

    // A real struck span is still removed: the census corpus's U3
    // title carries the double space where the struck span was.
    let board = parse_todo_md(&todo_md()).unwrap();
    assert_eq!(
        board.followups[2].title,
        "A struck count  beside a live pair **222** in one sentence"
    );
}

// =================================================================
// The wire: the dialect param's dispatch.
// =================================================================

/// The board directory, relative to the repo's registered checkout
/// path — the dialect's measured layout, mirrored from the tool's
/// constant.
const BOARD_DIR: &str = "docs/implementation";

/// Builds the corpus ikvm-html tree under a fresh temp directory: the
/// two board files at their dialect paths. Returns the tree's root —
/// the repo's registered checkout path.
fn ikvm_tree() -> TempDir {
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

/// `import_md { dialect: "ikvm-html" }` against the synthetic
/// ikvm-html tree: the counts response and spot ids asserted in-test
/// (no golden, D12); an unknown dialect value is `invalid` naming all
/// four values; and a tree missing its decisions file faults as `io`
/// naming the ikvm file's role.
#[test]
fn import_md_with_dialect_ikvm_html_drives_the_ikvm_path() {
    let root = TempDir::new().unwrap();
    let tree = ikvm_tree();
    let requests = vec![
        call(2, "repo_register", &register_args("ikvm", tree.path())),
        call(3, "import_md", r#"{"repo":"ikvm","dialect":"ikvm-html"}"#),
        call(4, "graph_show", r#"{"repo":"ikvm"}"#),
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
    assert_eq!(answer["repo"], json!("ikvm"));
    assert_eq!(answer["epics"], json!(9));
    assert_eq!(answer["decisions"], json!(5));
    assert_eq!(answer["docs"], json!(0));
    assert_eq!(answer["tasks"], json!(11));
    assert_eq!(answer["blocks"], json!(5));

    // graph_show's spot checks: the reproduced base id, the debt
    // kind, and the blocker after the register.
    let graph = structured(&driven.responses[2]);
    let repo = &graph["repos"][0];
    let tasks: Vec<&Value> = repo["epics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["tasks"].as_array().unwrap())
        .collect();
    assert_eq!(tasks.len(), 11);
    let u1 = tasks
        .iter()
        .find(|task| task["id"] == json!("ikvm:U1"))
        .unwrap();
    assert_eq!(u1["title"], json!("Fold the late sections into the plan"));
    assert_eq!(u1["kind"], json!("task"));
    let u6 = tasks
        .iter()
        .find(|task| task["id"] == json!("ikvm:U6"))
        .unwrap();
    assert_eq!(u6["kind"], json!("debt"), "§5's items are debt");
    let u8 = tasks
        .iter()
        .find(|task| task["id"] == json!("ikvm:U8"))
        .unwrap();
    assert_eq!(
        u8["title"],
        json!("The reference corpus covers both wire directions"),
        "the first blocker lands after the register"
    );

    // An unknown dialect value: `invalid` naming all four values.
    let root = TempDir::new().unwrap();
    let tree = ikvm_tree();
    let driven = drive(
        root.path(),
        &[
            call(2, "repo_register", &register_args("ikvm", tree.path())),
            call(3, "import_md", r#"{"repo":"ikvm","dialect":"shelly"}"#),
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
        "the dialect 'shelly' is not known: 'dacode', 'camera', 'ikvm-html', 'dacode-todo-mcp' or 'ipmi-collector'",
    );

    // A tree missing its decisions file: the io fault names the ikvm
    // file's role, the same convention the camera path pins.
    let root = TempDir::new().unwrap();
    let tree = ikvm_tree();
    fs::remove_file(tree.path().join(BOARD_DIR).join("decisions.md")).unwrap();
    let driven = drive(
        root.path(),
        &[
            call(2, "repo_register", &register_args("ikvm", tree.path())),
            call(3, "import_md", r#"{"repo":"ikvm","dialect":"ikvm-html"}"#),
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
        "io: read the ikvm decisions file decisions.md: NotFound",
    );
}
