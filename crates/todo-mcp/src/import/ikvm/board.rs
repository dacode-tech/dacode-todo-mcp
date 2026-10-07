//! The ikvm-html TODO.md parser (t47): the §1–§6 section walker,
//! §2's milestone table, §3's id-less blockers and the §4/§5
//! checkbox registers — plus the TODO.md half of the dialect AST,
//! which lives here because the ikvm module keeps its parser files
//! and their types together. The shared text primitives (cell
//! splitting, the table walk, the strike regions) are
//! [`crate::import::helpers`]'s.
//!
//! The walker: section boundaries are `## ` at line start,
//! identified by the number in the header — never by position. §1
//! (current-state prose) and §6 onward (the session narrative and
//! every `## Where …` section after it) are walked past; §2, §3, §4
//! and §5 must all be present, and a missing one is a named error
//! naming it. A duplicated imported section is a named error.
//!
//! §2's register is its milestone table, recognized by the header row
//! (`| | Theme | Gated on |`) wherever it appears in the section —
//! exactly one such table and exactly six rows (the board's §14
//! roadmap shape, M0–M5), else a named error. The id cell strips to
//! an M id; a duplicate M id is a named error (the register is
//! measured duplicate-free, so strictness is the honest reading).
//! The Theme cell is the title; the Gated-on cell is living narrative
//! and is dropped, never parsed for edges.
//!
//! §3 is a population neither sibling has: checkbox items with bold
//! titles and **no ids**. The bold run closes at its first `**`
//! outside code spans and struck spans (the shared scanner shape —
//! no such marker occurs on the measured board, but the scanner does
//! not pick a side); the title is the inner text; an unclosed run is
//! a named error. No ` · ` split, no id.
//!
//! §4 and §5 are checkbox registers at column zero, and the bold
//! markers carry **no title boundary** here: the measured titles
//! contain struck spans with nested bold (`~~**111**~~`), live bold
//! pairs (`**222**`) and a literal `~~` inside backticks, and U69's
//! `**` tokens outside struck spans are all paired emphasis — so the
//! title rule is the **first sentence**: the joined bold text's first
//! `.` outside struck spans and code spans ends the title (inclusive),
//! struck spans removed, then the trailing trim. The id is the text
//! before the first ` · ` and must be `U<digits>` exactly — a
//! suffixed or `(original)` id is a named error, not an oddball (the
//! register is measured oddball-free). One U namespace spans §4 and
//! §5, so the duplicate check is the walker's, shared across both.
//! The body — everything after the title region — is captured from
//! the joined text and dropped by the map.
//!
//! The dependency prose is not scanned: the measured board carries
//! zero mechanical edges in register bodies (the `gates M<n>` and
//! `blocked on` mentions live in §6's narrative or qualified titles),
//! so the only edge source is decisions.md's Gates column (the D18
//! precedent).

use std::collections::BTreeMap;

use super::error::IkvmError;
use crate::import::helpers::{
    cells, collect_rows, find_table_header, is_header, remove_struck_spans, split_lines,
    strike_regions, strip_id_noise,
};

/// §2's milestones-table header (an empty first cell).
const MILESTONES_HEADER: [&str; 3] = ["", "Theme", "Gated on"];
/// The milestones table's row count — the board's §14 roadmap shape.
const MILESTONE_ROW_COUNT: usize = 6;
/// The U item's id/title separator (U+00B7 middle dot), spaced.
const MIDDLE_DOT_SEP: &str = " · ";
/// The bold-run marker. One home for the module tree: `decisions.rs`
/// consumes this too.
pub(super) const BOLD: &str = "**";

/// The parsed ikvm-html TODO.md: §2's milestone rows, §3's blockers
/// and the §4/§5 registers, in board order. Board order is the
/// discipline — `Vec`, never a map: the order is data. No line
/// numbers on entries (board ids are the stable keys); errors carry
/// the lines. `PartialEq` is derived so the twice-parse determinism
/// pin is a plain `assert_eq!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IkvmTodo {
    /// §2's milestone-table rows, in board order.
    pub milestones: Vec<MilestoneRow>,
    /// §3's blocker items, in board order.
    pub blockers: Vec<BlockerItem>,
    /// §4's register items, in board order.
    pub followups: Vec<RegisterItem>,
    /// §5's register items, in board order.
    pub debt: Vec<RegisterItem>,
}

/// One §2 milestone-table row: the M id and the Theme cell. The
/// Gated-on cell is not carried (the parser drops it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneRow {
    /// The M id, e.g. `M0`.
    pub id: String,
    /// The Theme cell, raw.
    pub title: String,
}

/// One §3 checkbox item: a bold title with no id. Strikethrough is
/// edit history here, not resolution, so no `struck` field exists —
/// the checkbox alone is the status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockerItem {
    /// The bold run's inner text, trailing-trimmed.
    pub title: String,
    /// The checkbox state.
    pub checked: bool,
    /// Everything after the bold run, from the joined text.
    pub body: String,
}

/// One §4/§5 checkbox register item. No `struck` field (edit history
/// is not parsed) and no dependencies field (the register bodies
/// carry no mechanical edges).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterItem {
    /// The U id as written: `U<digits>` exactly.
    pub id: String,
    /// The first-sentence title, struck spans removed,
    /// trailing-trimmed.
    pub title: String,
    /// The checkbox state.
    pub checked: bool,
    /// Everything after the title region, from the joined text.
    pub body: String,
}

/// The per-section line lists: section number → its lines with 1-based
/// numbers.
type SectionMap<'a> = BTreeMap<u8, Vec<(&'a str, usize)>>;

/// Parses ikvm-html TODO.md into its dialect AST. Pure function of the
/// text — no I/O, no clock; file reading is the tool's.
pub fn parse_todo_md(input: &str) -> Result<IkvmTodo, IkvmError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let sections = split_sections(&numbered)?;
    let end_line = numbered.len();
    let mut todo = IkvmTodo {
        milestones: Vec::new(),
        blockers: Vec::new(),
        followups: Vec::new(),
        debt: Vec::new(),
    };
    parse_milestones(
        require_section(&sections, 2, end_line)?,
        &mut todo,
        end_line,
    )?;
    parse_blockers(require_section(&sections, 3, end_line)?, &mut todo)?;
    // One U namespace spans §4 and §5 — the duplicate check is the
    // walker's, not the section's: a U id in both sections would
    // silently double-import, and the reproduction index would
    // resolve the id to the second task only.
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
) -> Result<&'a [(&'a str, usize)], IkvmError> {
    sections
        .get(&number)
        .map(Vec::as_slice)
        .ok_or_else(|| IkvmError::MissingSection {
            section: format!("§{number}"),
            line: end_line,
        })
}

/// Splits the input into per-section line lists. A `## ` header ends
/// the current section whatever it is; a header naming an imported
/// section opens it; everything else (§1, §6, the `## Where …`
/// sections, unnumbered headers) is walked past.
fn split_sections<'a>(lines: &'a [(&str, usize)]) -> Result<SectionMap<'a>, IkvmError> {
    let mut sections: SectionMap<'a> = BTreeMap::new();
    let mut current: Option<u8> = None;
    for &(text, number) in lines {
        if text.starts_with("## ") {
            current = None;
            if let Some(section) = section_number(text)
                && is_imported(section)
            {
                if sections.contains_key(&section) {
                    return Err(IkvmError::MalformedLine {
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

/// Whether §`n` is imported (§2, §3, §4, §5).
fn is_imported(section: u8) -> bool {
    matches!(section, 2..=5)
}

/// §2: the milestones table, recognized by its header row — exactly
/// one such table, exactly six rows, no duplicate M ids.
fn parse_milestones(
    lines: &[(&str, usize)],
    todo: &mut IkvmTodo,
    end_line: usize,
) -> Result<(), IkvmError> {
    let Some((header_idx, _, header_number)) = find_table_header(lines, &MILESTONES_HEADER) else {
        return Err(IkvmError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: end_line,
        });
    };
    let header_count = lines
        .iter()
        .filter(|(text, _)| is_header(text, &MILESTONES_HEADER))
        .count();
    if header_count > 1 {
        return Err(IkvmError::MalformedLine {
            line: header_number,
            detail: "§2 carries the milestones-table header more than once".to_owned(),
        });
    }
    let (rows, _) = collect_rows(lines, header_idx, is_register_construct);
    if rows.len() != MILESTONE_ROW_COUNT {
        return Err(IkvmError::MalformedLine {
            line: header_number,
            detail: format!(
                "§2's milestones table has {} rows, exactly {MILESTONE_ROW_COUNT} required",
                rows.len()
            ),
        });
    }
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [id_cell, theme_cell, _gated] = row_cells.as_slice() else {
            return Err(IkvmError::ColumnCount {
                line: number,
                expected: MILESTONES_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_m_id(&id) {
            return Err(IkvmError::BadId {
                line: number,
                id,
                detail: "not an M id".to_owned(),
            });
        }
        if let Some(first) = seen.get(&id) {
            return Err(IkvmError::DuplicateId {
                id: id.clone(),
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        todo.milestones.push(MilestoneRow {
            id,
            title: theme_cell.clone(),
        });
    }
    Ok(())
}

/// §3: the id-less blockers. Every checkbox at column zero is a
/// register item and must parse; every other line is the current
/// item's continuation, already captured in the item's join.
fn parse_blockers(lines: &[(&str, usize)], todo: &mut IkvmTodo) -> Result<(), IkvmError> {
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if is_checkbox_line(text) {
            todo.blockers
                .push(parse_blocker_item(text, number, lines, idx)?);
        }
    }
    Ok(())
}

/// §4/§5: the checkbox register. Every checkbox at column zero is a
/// register item and must parse; every other line is the current
/// item's continuation, already captured in the item's join. `seen`
/// is the caller's id map — one U namespace spans both sections, so
/// the duplicate check crosses them.
fn parse_register(
    lines: &[(&str, usize)],
    seen: &mut BTreeMap<String, usize>,
    items: &mut Vec<RegisterItem>,
) -> Result<(), IkvmError> {
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if is_checkbox_line(text) {
            let item = parse_register_item(text, number, lines, idx)?;
            if let Some(first) = seen.get(&item.id) {
                return Err(IkvmError::DuplicateId {
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
fn split_marker(text: &str, line: usize) -> Result<(bool, &str), IkvmError> {
    if let Some(rest) = text.strip_prefix("- [ ] ") {
        return Ok((false, rest));
    }
    if let Some(rest) = text.strip_prefix("- [x] ") {
        return Ok((true, rest));
    }
    Err(IkvmError::MalformedLine {
        line,
        detail: "a checkbox marker with no bold run".to_owned(),
    })
}

/// Parses one §3 blocker item: the checkbox marker, the bold run
/// (which may wrap), the title as the run's inner text, and the body
/// as everything after the closing `**`.
fn parse_blocker_item(
    text: &str,
    line: usize,
    lines: &[(&str, usize)],
    idx: usize,
) -> Result<BlockerItem, IkvmError> {
    let (checked, rest) = split_marker(text, line)?;
    let after_bold = rest
        .strip_prefix(BOLD)
        .ok_or_else(|| IkvmError::MalformedLine {
            line,
            detail: "a §3 item's bold run is missing".to_owned(),
        })?;
    let joined = join_span(after_bold, lines, idx);
    let Some((inner, body)) = split_outside_spans(&joined, BOLD) else {
        return Err(IkvmError::MalformedLine {
            line,
            detail: "a §3 item's bold run does not close".to_owned(),
        });
    };
    Ok(BlockerItem {
        title: clean_title(inner),
        checked,
        body: body.trim().to_owned(),
    })
}

/// Parses one §4/§5 register item: the checkbox marker, the joined
/// bold text, the id before the first ` · `, and the first-sentence
/// title.
fn parse_register_item(
    text: &str,
    line: usize,
    lines: &[(&str, usize)],
    idx: usize,
) -> Result<RegisterItem, IkvmError> {
    let (checked, rest) = split_marker(text, line)?;
    let after_bold = rest
        .strip_prefix(BOLD)
        .ok_or_else(|| IkvmError::MalformedLine {
            line,
            detail: "a U item's bold run is missing".to_owned(),
        })?;
    let joined = join_span(after_bold, lines, idx);
    let Some((id, tail)) = joined.split_once(MIDDLE_DOT_SEP) else {
        return Err(IkvmError::MalformedLine {
            line,
            detail: "the bold run is not U-shaped: no ' · ' separator".to_owned(),
        });
    };
    if !is_strict_u_id(id) {
        return Err(IkvmError::MalformedLine {
            line,
            detail: format!("the bold run is not U-shaped: '{id}' is not a U id"),
        });
    }
    let Some((title_region, body)) = split_outside_spans(tail, ".") else {
        return Err(IkvmError::MalformedLine {
            line,
            detail: "a U item's title has no sentence-ending period".to_owned(),
        });
    };
    Ok(RegisterItem {
        id: id.to_owned(),
        title: clean_title(&remove_struck_spans(title_region)),
        checked,
        body: body.trim().to_owned(),
    })
}

/// The item's raw span joined across its wraps: each line trimmed,
/// the pieces joined with single spaces — the break replaces a space.
/// The join ends at the next register construct or table row, never
/// absorbing either.
fn join_span(first: &str, lines: &[(&str, usize)], idx: usize) -> String {
    let mut joined = String::from(first.trim());
    for &(text, _) in lines.iter().skip(idx + 1) {
        if is_span_end(text) {
            break;
        }
        joined.push(' ');
        joined.push_str(text.trim());
    }
    joined
}

/// Splits `text` at the first `needle` occurrence outside code spans
/// and struck spans, returning the text before it and the text after
/// it (the needle itself dropped). `None` when no such occurrence
/// exists. The walk mirrors [`strike_regions`]'s own — backticks
/// toggle the code state, a `~~` pair advances two bytes — so the
/// two agree on where the spans sit.
fn split_outside_spans<'a>(text: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let struck = strike_regions(text);
    let mut in_code = false;
    let mut region = 0;
    let mut offset = 0;
    let mut rest = text;
    while !rest.is_empty() {
        while struck.get(region).is_some_and(|&(_, end)| offset >= end) {
            region += 1;
        }
        let in_struck = struck
            .get(region)
            .is_some_and(|&(start, _)| offset >= start);
        if !in_code && !in_struck && rest.starts_with(needle) {
            // `rest` is a suffix of `text` by construction (only
            // `strip_prefix` advances it), so the head is the prefix
            // it removes.
            let head = text.strip_suffix(rest)?;
            return Some((head, rest.strip_prefix(needle)?));
        }
        if rest.starts_with("~~") {
            rest = rest.strip_prefix("~~")?;
            offset += 2;
        } else {
            let ch = rest.chars().next()?;
            if ch == '`' {
                in_code = !in_code;
            }
            rest = rest.strip_prefix(ch)?;
            offset += ch.len_utf8();
        }
    }
    None
}

/// The task or epic title from the board's raw title: trailing
/// whitespace and one trailing `.` trimmed — the dacode mapping's
/// title discipline, unchanged.
pub(super) fn clean_title(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed.strip_suffix('.').unwrap_or(trimmed).to_owned()
}

/// A top-level checkbox line: `- [ ]` or `- [x]` at column zero.
/// Indented checkboxes are body text — this board's registers are
/// measured at column zero.
fn is_checkbox_line(text: &str) -> bool {
    text.starts_with("- [ ]") || text.starts_with("- [x]")
}

/// A line that begins a register construct: a column-0 checkbox item,
/// a subsection header or a section header — the strict shape this
/// board measures. One home for the module tree: `decisions.rs`
/// consumes this too.
pub(super) fn is_register_construct(text: &str) -> bool {
    is_checkbox_line(text) || text.starts_with("### ") || text.starts_with("## ")
}

/// A line that ends the current item's span: a register construct or
/// a table row — the wrap join never absorbs either.
fn is_span_end(text: &str) -> bool {
    is_register_construct(text) || text.starts_with('|')
}

/// A register id as written: `U<digits>` exactly — the shape that
/// reproduces the board's numbering. A suffixed or `(original)` id is
/// a named error here, not an oddball: the register is measured
/// oddball-free, so strictness is the honest reading.
fn is_strict_u_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('U') else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

/// An M id: `M<digits>` exactly — the table registers' measured
/// shape. A letter-suffixed id is a named error, not an oddball
/// (the board's M ids are plain, measured). One home for the module
/// tree: `decisions.rs` consumes this too.
pub(super) fn is_m_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('M') else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}
