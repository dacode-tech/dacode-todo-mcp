//! The own TODO.md parser (t54): the §1–§6 section walker, §2's
//! milestone table, and the §4/§5 checkbox registers — plus the
//! TODO.md half of the dialect AST, which lives here because the own
//! module keeps its parser files and their types together. The
//! shared text primitives (cell splitting, the table walk) are
//! [`crate::import::helpers`]'s; `text.rs` keeps the id, checkbox and
//! register-construct shapes — camera's grammar, per the measured
//! study.
//!
//! The walker: section boundaries are `## ` at line start,
//! identified by the number in the header — never by position. §1
//! (the current-state narrative), §3 (the delivery log) and §6 (the
//! session log) are walked past; §2, §4 and §5 must all be present,
//! and a missing one is a named error naming it. A duplicated
//! imported section is a named error.
//!
//! §2's register is its milestone table, recognized by the header row
//! (`Milestone | Deliverable | Gated on | State`) wherever it appears
//! in the section — four header columns, this board's measured shape.
//! The Milestone cell is bare `M<n>` (the title lives in the Deliverable
//! cell, which is the epic title the mapping takes); the Gated-on
//! cell is narrative prose naming D-ids and is walked past — the
//! decisions file's Gates column is the edge register, and it already
//! carries the same edges (the study's judgment call: no double
//! import). The State cell's verdict is its prefix — `done <date>`
//! or `in progress` — because the measured cells carry long prose
//! after the spelling; the prose is dropped (the board keeps the
//! narrative), and an unrecognized spelling is a named error. A
//! duplicated M id is a named error naming both lines, the U
//! registers' own rule: the mapping reproduces the id range from the
//! rows, so a silent duplicate would shift every later epic's id.
//!
//! One measured widening beyond the study's four-column inventory:
//! the live M4 row carries **five** cells — its closure appended the
//! `done` verdict as a new cell, keeping the stale `in progress`
//! delivery log as a cell of its own (the board's "a status change
//! is a one-line edit" rule as actually practiced). The State is
//! therefore the row's **last** cell, and any cells between the
//! Gated-on column and the last are walked past as historical State
//! narrative. A row with fewer than four cells is a column-count
//! error; the widening is pinned by test.
//!
//! §4 and §5 are checkbox registers with camera's item grammar: `- [
//! ]`/`- [x]` at any indentation, then optionally the strike marker
//! `~~`, then the bold run `**<id> · <title>**`, which may wrap. The
//! status comes from the checkbox alone — the measured board carries
//! zero struck items, and the strike (were one to appear) is
//! confirmation of a checked item, never a state of its own. The
//! item's body prose is walked past entirely: this dialect's mapping
//! drops it (the board keeps the narrative per §9), so the AST
//! carries no body and no dependencies — the measured registers
//! carry no dependency prose (`blocked on`/`Held back by` appear
//! only in §3/§6 narrative, walked past), and the dependency
//! structure is the decisions' Gates column.

use std::collections::BTreeMap;

use super::error::OwnError;
use super::text::{is_checkbox_line, is_m_id, is_register_construct, is_u_id};
use crate::import::helpers::{
    cells, collect_rows, find_table_header, is_day_date, split_lines, strip_id_noise,
};

/// §2's milestones-table header — four columns, the measured shape.
const MILESTONES_HEADER: [&str; 4] = ["Milestone", "Deliverable", "Gated on", "State"];
/// The U item's id/title separator (U+00B7 middle dot), spaced.
const MIDDLE_DOT_SEP: &str = " · ";
/// The State cell's done spelling prefix.
const STATE_DONE: &str = "done";
/// The State cell's in-progress spelling.
const STATE_IN_PROGRESS: &str = "in progress";
/// The strike marker.
const STRIKE: &str = "~~";

/// The parsed own TODO.md: §2's milestone rows and the §4/§5
/// registers, in board order. Board order is the discipline — `Vec`,
/// never a map: the order is data. No line numbers on entries (board
/// ids are the stable keys); errors carry the lines. `PartialEq` is
/// derived so the twice-parse determinism pin is a plain
/// `assert_eq!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnTodo {
    /// §2's milestone-table rows, in board order.
    pub milestones: Vec<MilestoneRow>,
    /// §4's register items, in board order.
    pub followups: Vec<RegisterItem>,
    /// §5's register items, in board order.
    pub debt: Vec<RegisterItem>,
}

/// One §2 milestone-table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneRow {
    /// The M id, e.g. `M0`.
    pub id: String,
    /// The Deliverable cell — the epic title the mapping takes.
    pub title: String,
    /// The State cell's verdict.
    pub status: MilestoneState,
}

/// The State cell's verdict — the mapping's status input, parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneState {
    /// `done <date>` — the epic is `closed`.
    Done,
    /// `in progress` — the epic is `in_work`.
    InProgress,
}

/// One §4/§5 checkbox register item. No body and no dependencies:
/// the mapping drops the body prose (the board keeps the narrative),
/// and the measured registers carry no dependency prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterItem {
    /// The U id as written: `U<digits>`, `U<digits><letter>`, or
    /// `U<digits> (original)`.
    pub id: String,
    /// The title from the bold run.
    pub title: String,
    /// The checkbox state.
    pub checked: bool,
    /// The strike wrapper's presence.
    pub struck: bool,
}

/// The per-section line lists: section number → its lines with 1-based
/// numbers.
type SectionMap<'a> = BTreeMap<u8, Vec<(&'a str, usize)>>;

/// Parses the own TODO.md into its dialect AST. Pure function of the
/// text — no I/O, no clock; file reading is the tool's.
pub fn parse_todo_md(input: &str) -> Result<OwnTodo, OwnError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let sections = split_sections(&numbered)?;
    let end_line = numbered.len();
    let mut todo = OwnTodo {
        milestones: Vec::new(),
        followups: Vec::new(),
        debt: Vec::new(),
    };
    parse_milestones(
        require_section(&sections, 2, end_line)?,
        &mut todo,
        end_line,
    )?;
    // One U namespace spans §4 and §5 — the duplicate check is the
    // walker's, not the section's: a U id in both sections would
    // silently double-import, and the reproduction index would resolve
    // the id to the second task only.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    parse_register(
        require_section(&sections, 4, end_line)?,
        &mut seen,
        &mut todo.followups,
    )?;
    parse_register(
        require_section(&sections, 5, end_line)?,
        &mut seen,
        &mut todo.debt,
    )?;
    Ok(todo)
}

/// The imported section's lines, or the named missing-section error.
fn require_section<'a>(
    sections: &'a SectionMap<'a>,
    number: u8,
    end_line: usize,
) -> Result<&'a [(&'a str, usize)], OwnError> {
    sections
        .get(&number)
        .map(Vec::as_slice)
        .ok_or_else(|| OwnError::MissingSection {
            section: format!("§{number}"),
            line: end_line,
        })
}

/// Splits the input into per-section line lists. A `## ` header ends
/// the current section whatever it is; a header naming an imported
/// section opens it; everything else (§1, §3, §6, sections past §6,
/// unnumbered headers) is walked past.
fn split_sections<'a>(lines: &'a [(&'a str, usize)]) -> Result<SectionMap<'a>, OwnError> {
    let mut sections: SectionMap<'a> = BTreeMap::new();
    let mut current: Option<u8> = None;
    for &(text, number) in lines {
        if text.starts_with("## ") {
            current = None;
            if let Some(section) = section_number(text)
                && is_imported(section)
            {
                if sections.contains_key(&section) {
                    return Err(OwnError::MalformedLine {
                        line: number,
                        detail: format!("TODO.md carries §{section} twice"),
                    });
                }
                sections.insert(section, Vec::new());
                current = Some(section);
            }
            continue;
        }
        if let Some(section) = current
            && let Some(section_lines) = sections.get_mut(&section)
        {
            section_lines.push((text, number));
        }
    }
    Ok(sections)
}

/// The number in a `## <n>. <title>` header, when the header carries
/// one.
fn section_number(header: &str) -> Option<u8> {
    let rest = header.strip_prefix("## ")?;
    let (number, _) = rest.split_once('.')?;
    number.parse().ok()
}

/// Whether §`n` is imported (§2, §4, §5).
fn is_imported(section: u8) -> bool {
    matches!(section, 2 | 4 | 5)
}

/// §2: the milestones table, recognized by its header row. The
/// Milestone cell is bare `M<n>`; the Deliverable cell is the title;
/// the Gated-on cell is walked past (the decisions' Gates column is
/// the edge register); the State cell is the row's **last** cell —
/// the measured M4 row carries a stale in-progress cell before its
/// done verdict, and the extra middle cells are walked past as
/// historical narrative. A duplicated M id is a named error naming
/// both lines — the mapping reproduces the id range from the rows,
/// so a silent duplicate would shift every later epic's id.
fn parse_milestones(
    lines: &[(&str, usize)],
    todo: &mut OwnTodo,
    end_line: usize,
) -> Result<(), OwnError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &MILESTONES_HEADER) else {
        return Err(OwnError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx, is_register_construct);
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [milestone_cell, deliverable_cell, _gated_on, .., state_cell] = row_cells.as_slice()
        else {
            return Err(OwnError::ColumnCount {
                line: number,
                expected: MILESTONES_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(milestone_cell);
        if !is_m_id(&id) {
            return Err(OwnError::BadId {
                line: number,
                id,
                detail: "not a bare M id — the title lives in the Deliverable cell".to_owned(),
            });
        }
        if let Some(first) = seen.get(&id) {
            return Err(OwnError::DuplicateId {
                id,
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        let status = parse_state(state_cell, number)?;
        todo.milestones.push(MilestoneRow {
            id,
            title: deliverable_cell.clone(),
            status,
        });
    }
    Ok(())
}

/// The State cell's verdict: the spelling is the cell's prefix (the
/// measured cells carry long prose after it), with the bold wrapper
/// stripped first. `done <date>` validates the date's shape (the
/// verdict alone maps — the date and prose are dropped, the board
/// keeps the narrative); `in progress` needs nothing after it.
fn parse_state(cell: &str, line: usize) -> Result<MilestoneState, OwnError> {
    let trimmed = cell.replace("**", "");
    let trimmed = trimmed.trim();
    if let Some(rest) = trimmed.strip_prefix(STATE_DONE) {
        let Some(after_space) = rest.strip_prefix(' ') else {
            return Err(OwnError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the done spelling's date is missing".to_owned(),
            });
        };
        let Some(date) = after_space.get(..10) else {
            return Err(OwnError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the done spelling's date is missing".to_owned(),
            });
        };
        if !is_day_date(date) {
            return Err(OwnError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the done spelling's date is not YYYY-MM-DD".to_owned(),
            });
        }
        return Ok(MilestoneState::Done);
    }
    if trimmed.starts_with(STATE_IN_PROGRESS) {
        return Ok(MilestoneState::InProgress);
    }
    Err(OwnError::BadStatus {
        line,
        text: cell.to_owned(),
        detail: "the State cell must read 'done <date>' or 'in progress'".to_owned(),
    })
}

/// §4/§5: the checkbox register. Every checkbox at any indentation is
/// a register item and must parse; every other line is body text,
/// walked past — the mapping drops it, so the AST carries no body.
/// `seen` is the caller's id map — one U namespace spans both
/// sections, so the duplicate check crosses them.
fn parse_register(
    lines: &[(&str, usize)],
    seen: &mut BTreeMap<String, usize>,
    items: &mut Vec<RegisterItem>,
) -> Result<(), OwnError> {
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if is_checkbox_line(text) {
            let item = parse_item(text, number, lines, idx)?;
            if let Some(first) = seen.get(&item.id) {
                return Err(OwnError::DuplicateId {
                    id: item.id.clone(),
                    first: *first,
                    second: number,
                });
            }
            seen.insert(item.id.clone(), number);
            items.push(item);
        }
    }
    Ok(())
}

/// Splits the checkbox marker, returning the state and the rest of
/// the line. A marker with nothing after it is a malformed register
/// line, not body text.
fn split_marker(text: &str, line: usize) -> Result<(bool, &str), OwnError> {
    let rest = text.trim_start_matches(' ');
    if let Some(after) = rest.strip_prefix("- [ ] ") {
        return Ok((false, after));
    }
    if let Some(after) = rest.strip_prefix("- [x] ") {
        return Ok((true, after));
    }
    Err(OwnError::MalformedLine {
        line,
        detail: "a checkbox marker with no bold run".to_owned(),
    })
}

/// Parses a checkbox item's first line: the marker, the optional
/// strike wrapper, and the bold run (which may wrap). The bold run's
/// closing `**` ends the item's head; everything after it on the line
/// and every following non-construct line is body text, walked past.
fn parse_item<'a>(
    text: &'a str,
    line: usize,
    lines: &[(&'a str, usize)],
    idx: usize,
) -> Result<RegisterItem, OwnError> {
    let (checked, rest) = split_marker(text, line)?;
    let (struck, after_strike) = match rest.strip_prefix(STRIKE) {
        Some(after) => (true, after),
        None => (false, rest),
    };
    if struck && !checked {
        return Err(OwnError::StruckUnchecked { line });
    }
    let bold = after_strike
        .strip_prefix("**")
        .ok_or_else(|| OwnError::MalformedLine {
            line,
            detail: "a U item's bold run is missing".to_owned(),
        })?;
    let inner = scan_bold_run(bold, lines, idx, line)?;
    let Some((id, title)) = inner.split_once(MIDDLE_DOT_SEP) else {
        return Err(OwnError::MalformedLine {
            line,
            detail: "the bold run is not U-shaped: no ' · ' separator".to_owned(),
        });
    };
    if !is_u_id(id) {
        return Err(OwnError::MalformedLine {
            line,
            detail: format!("the bold run is not U-shaped: '{id}' is not a U id"),
        });
    }
    Ok(RegisterItem {
        id: id.to_owned(),
        title: title.to_owned(),
        checked,
        struck,
    })
}

/// Scans the bold run from `bold` (the text after the opening `**`),
/// joining wrapped pieces with single spaces, until the closing `**`,
/// and returns the inner text. The scan never crosses a register
/// construct or a table row — an unclosed run is a named error, not a
/// title polluted by whatever follows it.
fn scan_bold_run(
    bold: &str,
    lines: &[(&str, usize)],
    idx: usize,
    line: usize,
) -> Result<String, OwnError> {
    let mut inner = String::new();
    let mut piece = bold;
    let mut offset = 0;
    loop {
        if let Some((head, _tail)) = piece.split_once("**") {
            inner.push_str(head);
            return Ok(inner);
        }
        inner.push_str(piece.trim_end());
        inner.push(' ');
        let Some(&(next, _)) = lines.get(idx + 1 + offset) else {
            return Err(OwnError::MalformedLine {
                line,
                detail: "a U item's bold run does not close".to_owned(),
            });
        };
        if is_register_construct(next) || next.starts_with('|') {
            return Err(OwnError::MalformedLine {
                line,
                detail: "a U item's bold run does not close".to_owned(),
            });
        }
        piece = next.trim();
        offset += 1;
    }
}
