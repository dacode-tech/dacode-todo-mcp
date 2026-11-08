//! The ipmi-collector TODO.md parser (t77): the §-prefixed section
//! walker, §2's two-column milestone table, §4's open-only checkbox
//! register and §5's checkbox-less debt register — plus the TODO.md
//! half of the dialect AST, which lives here because the ipmi module
//! keeps its parser files and their types together. The shared text
//! primitives (cell splitting, the table walk) are
//! [`crate::import::helpers`]'s.
//!
//! The walker: section boundaries are `## ` at line start, identified
//! by the number in the `## §<n> <title>` header — the §-prefixed
//! spelling this board family measures, unlike the siblings'
//! `## <n>. <title>`. §1 (current-state prose), §3 (the landed log)
//! and §6 (the session narrative, `###` subsections included) are
//! walked past; §2, §4 and §5 must all be present, and a missing one
//! is a named error naming it. A duplicated imported section is a
//! named error.
//!
//! §2's register is its milestone table, recognized by the header row
//! (`Milestone | State`) wherever it appears in the section. The
//! Milestone cell is `M<n> — <title>`, the split at the first ` — `;
//! the State cell's leading bold word is the status — the measured
//! vocabulary is `done` alone (M0–M3), and any other word is a named
//! error, the strictness rule: the register is measured, not guessed.
//! The prose after the bold word (done-dates, commit hashes) is
//! completion narrative, dropped — the board keeps it.
//!
//! §4 is an **open-only** checkbox register: the item shape is
//! `- [ ] **<U id> · <title>** —` followed by indented prose. The
//! board's own rule is
//! that resolved items are deleted (gaps mean an item was resolved),
//! so a `- [x]` item is a named refusal, not a closed task — a
//! checked box means the convention drifted, and the import surfaces
//! it rather than guessing a closure the board never recorded. The
//! bold run may wrap (the join replaces the space); the ` —`
//! separator after the closing `**` is required; the prose — the
//! separator's remainder plus the continuation lines to the next
//! register construct, each trimmed, the empty lines skipped — joins
//! with single spaces into the item's description.
//!
//! §5 is a checkbox-less debt register: `- **<D-N id> · <title>** —`
//! followed by prose, the ids in the hyphenated `D-N` family — a
//! family distinct from the decisions file's `D<digits>` (the
//! register's own naming: debt, not decisions). A checkbox item in §5
//! is a named refusal (the register carries none), and a §5-shaped
//! bullet in §4 is a named refusal too (the register families are
//! distinct, and a misplaced item would silently lose its section's
//! semantics).
//!
//! The register-construct check — which column-zero lines begin
//! register items — is one home for the module tree: `decisions.rs`
//! consumes it too.

use std::collections::BTreeMap;

use super::error::IpmiError;
use crate::import::helpers::{cells, collect_rows, find_table_header, split_lines, strip_id_noise};

/// §2's milestones-table header — two columns, the measured shape.
const MILESTONES_HEADER: [&str; 2] = ["Milestone", "State"];
/// The U item's id/title separator (U+00B7 middle dot), spaced.
const MIDDLE_DOT_SEP: &str = " · ";
/// The milestone cell's id/title separator: the spaced em-dash.
const EM_DASH_SEP: &str = " — ";
/// The item's prose separator: the em-dash, unspaced — the tail after
/// the closing `**` reads ` —` at the measured line ends.
const PROSE_SEPARATOR: &str = "—";
/// The bold-run marker. One home for the module tree: `decisions.rs`
/// consumes this too.
pub(super) const BOLD: &str = "**";

/// The parsed ipmi-collector TODO.md: §2's milestone rows and the
/// §4/§5 registers, in board order. Board order is the discipline —
/// `Vec`, never a map: the order is data, and the mapping's id
/// reproduction reads it (register order, not numeric order — the
/// register is gapped). No line numbers on entries (board ids are the
/// stable keys); errors carry the lines. `PartialEq` is derived so
/// the twice-parse determinism pin is a plain `assert_eq!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpmiTodo {
    /// §2's milestone-table rows, in board order.
    pub milestones: Vec<MilestoneRow>,
    /// §4's register items, in board order.
    pub followups: Vec<RegisterItem>,
    /// §5's register items, in board order.
    pub debt: Vec<RegisterItem>,
}

/// One §2 milestone-table row: the M id, the Milestone cell's title,
/// and the State cell's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneRow {
    /// The M id, e.g. `M0`.
    pub id: String,
    /// The Milestone cell's prose after the `M<n> — ` prefix.
    pub title: String,
    /// The State cell's verdict.
    pub status: MilestoneState,
}

/// The State cell's verdict. The measured vocabulary is `done` alone
/// — the board's milestones are all landed; a future in-flight
/// milestone widens this enum (the measured-grammar precedent:
/// refuse first, widen when measured).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneState {
    /// `**done** …` — the epic is `closed`.
    Done,
}

/// One §4/§5 register item: the id, the bold run's title, and the
/// joined prose. No checkbox field: §4's register is open-only (a
/// checked box is a named refusal at parse) and §5's carries no
/// checkbox at all — every imported item is open by grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterItem {
    /// The id as written: `U<digits>` in §4, `D-<digits>` in §5.
    pub id: String,
    /// The bold run's title, after the ` · ` separator.
    pub title: String,
    /// The prose after the ` —` separator, the continuation lines
    /// joined with single spaces — the task's description.
    pub prose: String,
}

/// The per-section line lists: section number → its lines with 1-based
/// numbers.
type SectionMap<'a> = BTreeMap<u8, Vec<(&'a str, usize)>>;

/// Parses ipmi-collector TODO.md into its dialect AST. Pure function
/// of the text — no I/O, no clock; file reading is the tool's.
pub fn parse_todo_md(input: &str) -> Result<IpmiTodo, IpmiError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let sections = split_sections(&numbered)?;
    let end_line = numbered.len();
    let mut todo = IpmiTodo {
        milestones: Vec::new(),
        followups: Vec::new(),
        debt: Vec::new(),
    };
    parse_milestones(
        require_section(&sections, 2, end_line)?,
        &mut todo,
        end_line,
    )?;
    parse_followups(require_section(&sections, 4, end_line)?, &mut todo)?;
    parse_debt(require_section(&sections, 5, end_line)?, &mut todo)?;
    Ok(todo)
}

/// The imported section's lines, or the named missing-section error.
fn require_section<'a>(
    sections: &'a SectionMap<'a>,
    number: u8,
    end_line: usize,
) -> Result<&'a [(&'a str, usize)], IpmiError> {
    sections
        .get(&number)
        .map(Vec::as_slice)
        .ok_or_else(|| IpmiError::MissingSection {
            section: format!("§{number}"),
            line: end_line,
        })
}

/// Splits the input into per-section line lists. A `## ` header ends
/// the current section whatever it is; a `## §<n> ` header naming an
/// imported section opens it; everything else (§1, §3, §6, unnumbered
/// or sibling-shaped headers) is walked past.
fn split_sections<'a>(lines: &'a [(&str, usize)]) -> Result<SectionMap<'a>, IpmiError> {
    let mut sections: SectionMap<'a> = BTreeMap::new();
    let mut current: Option<u8> = None;
    for &(text, number) in lines {
        if text.starts_with("## ") {
            current = None;
            if let Some(section) = section_number(text)
                && is_imported(section)
            {
                if sections.contains_key(&section) {
                    return Err(IpmiError::MalformedLine {
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

/// The number in a `## §<n> <title>` header, when the header carries
/// one — the §-prefixed spelling this board family measures.
fn section_number(header: &str) -> Option<u8> {
    let rest = header.strip_prefix("## §")?;
    let (number, _) = rest.split_once(' ')?;
    number.parse().ok()
}

/// Whether §`n` is imported (§2, §4, §5).
fn is_imported(section: u8) -> bool {
    matches!(section, 2 | 4 | 5)
}

/// §2: the milestones table, recognized by its header row — no
/// duplicate M ids (a duplicated row would silently double-import an
/// epic).
fn parse_milestones(
    lines: &[(&str, usize)],
    todo: &mut IpmiTodo,
    end_line: usize,
) -> Result<(), IpmiError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &MILESTONES_HEADER) else {
        return Err(IpmiError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx, is_register_construct);
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [milestone_cell, state_cell] = row_cells.as_slice() else {
            return Err(IpmiError::ColumnCount {
                line: number,
                expected: MILESTONES_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let Some((id, title)) = milestone_cell.split_once(EM_DASH_SEP) else {
            return Err(IpmiError::MalformedLine {
                line: number,
                detail: "the Milestone cell has no ' — ' separator after the id".to_owned(),
            });
        };
        let id = strip_id_noise(id);
        if !is_m_id(&id) {
            return Err(IpmiError::BadId {
                line: number,
                id,
                detail: "not an M id".to_owned(),
            });
        }
        if let Some(first) = seen.get(&id) {
            return Err(IpmiError::DuplicateId {
                id: id.clone(),
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        let status = parse_state(state_cell, number)?;
        todo.milestones.push(MilestoneRow {
            id,
            title: title.to_owned(),
            status,
        });
    }
    Ok(())
}

/// The State cell's verdict: the leading bold word. The measured
/// vocabulary is `done` alone; the prose after the word (dates,
/// commit hashes) is completion narrative, dropped.
fn parse_state(cell: &str, line: usize) -> Result<MilestoneState, IpmiError> {
    let Some(after_open) = cell.strip_prefix(BOLD) else {
        return Err(IpmiError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the State cell's bold word is missing".to_owned(),
        });
    };
    let Some((word, _rest)) = after_open.split_once(BOLD) else {
        return Err(IpmiError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the State cell's bold word does not close".to_owned(),
        });
    };
    match word.trim() {
        "done" => Ok(MilestoneState::Done),
        _ => Err(IpmiError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the State cell's bold word must read 'done'".to_owned(),
        }),
    }
}

/// §4: the open-only checkbox register. Every column-zero `- ` line
/// is a register line and must parse as a checkbox item — a checked
/// one is the open-only refusal, a §5-shaped bullet is a malformed
/// line. Every other line is the current item's continuation, already
/// captured in the item's prose join.
fn parse_followups(lines: &[(&str, usize)], todo: &mut IpmiTodo) -> Result<(), IpmiError> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (idx, &(text, number)) in lines.iter().enumerate() {
        let Some(rest) = register_line_rest(text) else {
            continue;
        };
        if rest.starts_with("[x] ") {
            return Err(IpmiError::CheckedItem { line: number });
        }
        let Some(after) = rest.strip_prefix("[ ] ") else {
            return Err(IpmiError::MalformedLine {
                line: number,
                detail: "a §4 register line is not a checkbox item".to_owned(),
            });
        };
        let item = parse_item(after, number, lines, idx)?;
        if !is_u_id(&item.id) {
            return Err(IpmiError::BadId {
                line: number,
                id: item.id,
                detail: "not a U id".to_owned(),
            });
        }
        if let Some(first) = seen.get(&item.id) {
            return Err(IpmiError::DuplicateId {
                id: item.id.clone(),
                first: *first,
                second: number,
            });
        }
        seen.insert(item.id.clone(), number);
        todo.followups.push(item);
    }
    Ok(())
}

/// §5: the checkbox-less debt register. Every column-zero `- ` line
/// is a register line and must parse as a `D-N` bullet — a checkbox
/// item is a named refusal (the register carries none). Every other
/// line is the current item's continuation, already captured in the
/// item's prose join.
fn parse_debt(lines: &[(&str, usize)], todo: &mut IpmiTodo) -> Result<(), IpmiError> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (idx, &(text, number)) in lines.iter().enumerate() {
        let Some(rest) = register_line_rest(text) else {
            continue;
        };
        if rest.starts_with("[ ] ") || rest.starts_with("[x] ") {
            return Err(IpmiError::MalformedLine {
                line: number,
                detail: "a §5 register line is a checkbox item — the register is checkbox-less"
                    .to_owned(),
            });
        }
        let item = parse_item(rest, number, lines, idx)?;
        if !is_debt_id(&item.id) {
            return Err(IpmiError::BadId {
                line: number,
                id: item.id,
                detail: "not a D-N id".to_owned(),
            });
        }
        if let Some(first) = seen.get(&item.id) {
            return Err(IpmiError::DuplicateId {
                id: item.id.clone(),
                first: *first,
                second: number,
            });
        }
        seen.insert(item.id.clone(), number);
        todo.debt.push(item);
    }
    Ok(())
}

/// The text after a column-zero register line's `- ` prefix, when the
/// line is one. The continuation prose is indented, so the
/// column-zero `- ` prefix is the unambiguous register marker.
fn register_line_rest(text: &str) -> Option<&str> {
    text.strip_prefix("- ")
}

/// Parses one register item's fields from the text after its marker:
/// the bold run (which may wrap), the ` · ` id/title split, the ` —`
/// separator, and the joined prose. The id's family is the caller's
/// to validate — §4 checks `U<digits>`, §5 checks `D-<digits>`.
fn parse_item(
    rest: &str,
    line: usize,
    lines: &[(&str, usize)],
    idx: usize,
) -> Result<RegisterItem, IpmiError> {
    let after_bold = rest
        .strip_prefix(BOLD)
        .ok_or_else(|| IpmiError::MalformedLine {
            line,
            detail: "a register item's bold run is missing".to_owned(),
        })?;
    let (inner, tail, consumed) = scan_bold_run(after_bold, lines, idx, line)?;
    let Some((id, title)) = inner.split_once(MIDDLE_DOT_SEP) else {
        return Err(IpmiError::MalformedLine {
            line,
            detail: "the bold run has no ' · ' separator".to_owned(),
        });
    };
    let prose_first =
        tail.trim()
            .strip_prefix(PROSE_SEPARATOR)
            .ok_or_else(|| IpmiError::MalformedLine {
                line,
                detail: "the item's ' —' separator after the bold run is missing".to_owned(),
            })?;
    Ok(RegisterItem {
        id: id.trim().to_owned(),
        title: title.trim().to_owned(),
        prose: join_prose(prose_first, lines, idx + 1 + consumed),
    })
}

/// Scans the bold run from `bold` (the text after the opening `**`),
/// joining wrapped pieces with single spaces, until the closing
/// `**`. Returns the inner text, the remainder of the line the
/// closing `**` sits on, and the number of continuation lines the
/// scan consumed (0 when the run closes on the item line). The scan
/// never crosses a register construct or a table row — an unclosed
/// run is a named error, not a title polluted by whatever follows it.
fn scan_bold_run<'a>(
    bold: &'a str,
    lines: &[(&'a str, usize)],
    idx: usize,
    line: usize,
) -> Result<(String, &'a str, usize), IpmiError> {
    let mut inner = String::new();
    let mut piece = bold;
    let mut offset = 0;
    loop {
        if let Some((head, tail)) = piece.split_once(BOLD) {
            inner.push_str(head);
            return Ok((inner, tail, offset));
        }
        inner.push_str(piece.trim_end());
        inner.push(' ');
        let Some(next) = lines.get(idx + 1 + offset).map(|&(text, _)| text) else {
            return Err(IpmiError::MalformedLine {
                line,
                detail: "a register item's bold run does not close".to_owned(),
            });
        };
        if is_register_construct(next) || next.starts_with('|') {
            return Err(IpmiError::MalformedLine {
                line,
                detail: "a register item's bold run does not close".to_owned(),
            });
        }
        piece = next.trim();
        offset += 1;
    }
}

/// The item's prose: the separator's remainder plus the continuation
/// lines to the next register construct or table row, each trimmed,
/// the empty lines skipped, joined with single spaces — the break
/// replaces a space.
fn join_prose(first: &str, lines: &[(&str, usize)], start: usize) -> String {
    let mut pieces: Vec<&str> = Vec::new();
    let first = first.trim();
    if !first.is_empty() {
        pieces.push(first);
    }
    for &(text, _) in lines.iter().skip(start) {
        if is_span_end(text) {
            break;
        }
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            pieces.push(trimmed);
        }
    }
    pieces.join(" ")
}

/// A line that begins a register construct: a column-zero register
/// line, a subsection header or a section header — the strict shape
/// this board measures. One home for the module tree: `decisions.rs`
/// consumes this too.
pub(super) fn is_register_construct(text: &str) -> bool {
    register_line_rest(text).is_some() || text.starts_with("### ") || text.starts_with("## ")
}

/// A line that ends the current item's prose span: a register
/// construct or a table row — the join never absorbs either.
fn is_span_end(text: &str) -> bool {
    is_register_construct(text) || text.starts_with('|')
}

/// A U id: `U<digits>` exactly — §4's measured shape. One home for
/// the module tree: `map.rs` consumes this too (the Gates scan).
pub(super) fn is_u_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('U') else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

/// An M id: `M<digits>` exactly — §2's measured shape. One home for
/// the module tree: `map.rs` consumes this too (the Gates scan).
pub(super) fn is_m_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('M') else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

/// A debt id: `D-<digits>` exactly — §5's hyphenated family, distinct
/// from the decisions file's `D<digits>`.
fn is_debt_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix("D-") else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}
