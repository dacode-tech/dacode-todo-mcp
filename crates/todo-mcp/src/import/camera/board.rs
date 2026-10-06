//! The camera TODO.md parser (t46): the §1–§6 section walker, §2's
//! milestone table, and the §4/§5 checkbox registers — plus the
//! TODO.md half of the dialect AST, which lives here because the
//! camera module keeps its parser files and their types together.
//! The shared text primitives (cell splitting, the table walk) are
//! [`crate::import::helpers`]'s; `text.rs` keeps the id, checkbox and
//! register-construct shapes — the dialect's own grammar.
//!
//! The walker: section boundaries are `## ` at line start,
//! identified by the number in the header — never by position. §1
//! (the blocked-on-user mirror — decisions.md is authoritative, the
//! board's own rule), §3 (the delivery log) and §6 (the session
//! narrative) are walked past; §2, §4 and §5 must all be present, and
//! a missing one is a named error naming it. A duplicated imported
//! section is a named error.
//!
//! §2's register is its milestone table, recognized by the header row
//! (`Milestone | State | Exit criterion`) wherever it appears in the
//! section. The Milestone cell is `M<n> — <title>`; the State cell's
//! status is its prefix — `in progress`, `not started`, or
//! `done — closed <date>` — because the measured cells carry long
//! prose after the spelling. The Exit criterion cell is not imported
//! (the narrative stays in the board).
//!
//! §4 and §5 are checkbox registers, and unlike the dacode dialect
//! an *indented* checkbox is a register item here, not body text: the
//! board's own rule is that `U` ids are permanent and never reused,
//! and the suffixed oddballs (`U6a`–`U6d`, `U46a`–`U46c`) are
//! indented children that carry real register ids — walking them past
//! would silently omit work. A checkbox at any indentation must
//! therefore parse, or it is a named error.
//!
//! The item grammar: `- [ ]`/`- [x]` at any indentation, then
//! optionally the strike marker `~~`, then the bold run
//! `**<id> · <title>**`. The id is the text before the first ` · `
//! and may be `U<digits>`, `U<digits><letter>`, or `U<digits>
//! (original)` (the struck variant shape). The bold run may wrap: it
//! closes at the first `**` on the item line or a following line, the
//! trimmed pieces joining with a single space — the break replaces a
//! space — and the scan never crosses a register construct or a table
//! row. The strike is the *leading* `~~` alone: the measured board
//! closes most struck runs (`~~**…**~~`) but its U84 and U104 leave
//! it open (`~~**…**` — the t42 widening class, measured on the live
//! board), so the closing marker is typographic, never required, and
//! the struck flag is the leading marker. A struck item must be
//! checked: the strike is confirmation of a checked item, never a
//! state of its own.
//!
//! The dependency prose is scanned over the item's full raw span with
//! whitespace normalized (a line wrap replaces a space — the measured
//! board's `Held back by` phrase wraps), one dependency per kind per
//! item, the first id-bearing occurrence winning, the phrase at a word
//! boundary and an id-shaped token directly following it. The
//! recognized spellings are the ruled ones, case-sensitive: `blocked
//! on` and `Held back by`. `gates M<n>` prose is not recognized — it
//! lives in §2's cells and §6's narrative, outside the registers (the
//! D18 precedent).

use std::collections::BTreeMap;

use super::error::CameraError;
use super::text::{
    EM_DASH_SEP, is_checkbox_line, is_id_shaped, is_m_id, is_register_construct, is_u_id,
};
use crate::import::helpers::{
    cells, clean_id_word, collect_rows, find_table_header, is_day_date, split_lines, strip_id_noise,
};

/// §2's milestones-table header.
const MILESTONES_HEADER: [&str; 3] = ["Milestone", "State", "Exit criterion"];
/// The U item's id/title separator (U+00B7 middle dot), spaced.
const MIDDLE_DOT_SEP: &str = " · ";
/// The State cell's in-progress spelling.
const STATE_IN_PROGRESS: &str = "in progress";
/// The State cell's not-started spelling.
const STATE_NOT_STARTED: &str = "not started";
/// The State cell's done spelling prefix.
const STATE_DONE: &str = "done — closed ";
/// The strike marker.
const STRIKE: &str = "~~";
/// The recognized blocked-on phrase.
const BLOCKED_ON_PHRASE: &str = "blocked on ";
/// The recognized held-back-by phrase, as written on the board.
const HELD_BACK_PHRASE: &str = "Held back by ";

/// The parsed camera TODO.md: §2's milestone rows and the §4/§5
/// registers, in board order. Board order is the discipline — `Vec`,
/// never a map: the order is data. No line numbers on entries (board
/// ids are the stable keys); errors carry the lines. `PartialEq` is
/// derived so the twice-parse determinism pin is a plain
/// `assert_eq!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraTodo {
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
    /// The Milestone cell's prose after the `M<n> — ` prefix.
    pub title: String,
    /// The State cell's verdict.
    pub status: MilestoneState,
}

/// The State cell's verdict — D6's status mapping, parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneState {
    /// `in progress` — the epic is `in_work`.
    InProgress,
    /// `done — closed <date>` — the epic is `closed`.
    DoneClosed,
    /// `not started` — the epic is `open`.
    NotStarted,
}

/// One §4/§5 checkbox register item.
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
    /// The item's full raw span after the checkbox marker — the strike
    /// wrapper and bold run included — joined by `\n`, verbatim.
    pub body: String,
    /// The dependency assertions recognized in the body, in emission
    /// order.
    pub dependencies: Vec<Dependency>,
}

/// One recognized dependency assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// The assertion's kind.
    pub kind: DependencyKind,
    /// The target id as written.
    pub id: String,
    /// The matched phrase text (whitespace-normalized): the phrase
    /// plus the id.
    pub text: String,
}

/// The dependency assertion kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    /// `blocked on <id>`.
    BlockedOn,
    /// `Held back by <id>`.
    HeldBackBy,
}

/// The per-section line lists: section number → its lines with 1-based
/// numbers.
type SectionMap<'a> = BTreeMap<u8, Vec<(&'a str, usize)>>;

/// Parses camera TODO.md into its dialect AST. Pure function of the
/// text — no I/O, no clock; file reading is the tool's.
pub fn parse_todo_md(input: &str) -> Result<CameraTodo, CameraError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let sections = split_sections(&numbered)?;
    let end_line = numbered.len();
    let mut todo = CameraTodo {
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
) -> Result<&'a [(&'a str, usize)], CameraError> {
    sections
        .get(&number)
        .map(Vec::as_slice)
        .ok_or_else(|| CameraError::MissingSection {
            section: format!("§{number}"),
            line: end_line,
        })
}

/// Splits the input into per-section line lists. A `## ` header ends
/// the current section whatever it is; a header naming an imported
/// section opens it; everything else (§1, §3, §6, sections past §6,
/// unnumbered headers) is walked past.
fn split_sections<'a>(lines: &'a [(&'a str, usize)]) -> Result<SectionMap<'a>, CameraError> {
    let mut sections: SectionMap<'a> = BTreeMap::new();
    let mut current: Option<u8> = None;
    for &(text, number) in lines {
        if text.starts_with("## ") {
            current = None;
            if let Some(section) = section_number(text)
                && is_imported(section)
            {
                if sections.contains_key(&section) {
                    return Err(CameraError::MalformedLine {
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

/// §2: the milestones table, recognized by its header row.
fn parse_milestones(
    lines: &[(&str, usize)],
    todo: &mut CameraTodo,
    end_line: usize,
) -> Result<(), CameraError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &MILESTONES_HEADER) else {
        return Err(CameraError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx, is_register_construct);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [milestone_cell, state_cell, _exit] = row_cells.as_slice() else {
            return Err(CameraError::ColumnCount {
                line: number,
                expected: MILESTONES_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let Some((id, title)) = milestone_cell.split_once(EM_DASH_SEP) else {
            return Err(CameraError::MalformedLine {
                line: number,
                detail: "the Milestone cell has no ' — ' separator after the id".to_owned(),
            });
        };
        let id = strip_id_noise(id);
        if !is_m_id(&id) {
            return Err(CameraError::BadId {
                line: number,
                id: id.to_owned(),
                detail: "not an M id".to_owned(),
            });
        }
        let status = parse_state(state_cell, number)?;
        todo.milestones.push(MilestoneRow {
            id: id.to_owned(),
            title: title.to_owned(),
            status,
        });
    }
    Ok(())
}

/// The State cell's verdict: the spelling is the cell's prefix (the
/// measured cells carry long prose after it), with the bold wrapper
/// stripped first.
fn parse_state(cell: &str, line: usize) -> Result<MilestoneState, CameraError> {
    let trimmed = cell.replace("**", "");
    let trimmed = trimmed.trim();
    if let Some(rest) = trimmed.strip_prefix(STATE_DONE) {
        let Some(date) = rest.get(..10) else {
            return Err(CameraError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the done spelling's date is missing".to_owned(),
            });
        };
        if !is_day_date(date) {
            return Err(CameraError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the done spelling's date is not YYYY-MM-DD".to_owned(),
            });
        }
        return Ok(MilestoneState::DoneClosed);
    }
    if trimmed.starts_with(STATE_IN_PROGRESS) {
        return Ok(MilestoneState::InProgress);
    }
    if trimmed.starts_with(STATE_NOT_STARTED) {
        return Ok(MilestoneState::NotStarted);
    }
    Err(CameraError::BadStatus {
        line,
        text: cell.to_owned(),
        detail: "the State cell must read 'in progress', 'not started', or 'done — closed <date>'"
            .to_owned(),
    })
}

/// §4/§5: the checkbox register. Every checkbox at any indentation is
/// a register item and must parse; every other line is the current
/// item's body text. `seen` is the caller's id map — one U namespace
/// spans both sections, so the duplicate check crosses them.
fn parse_register(
    lines: &[(&str, usize)],
    seen: &mut BTreeMap<String, usize>,
    items: &mut Vec<RegisterItem>,
) -> Result<(), CameraError> {
    let mut current: Option<PartialItem> = None;
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if is_checkbox_line(text) {
            flush(&mut current, items);
            let item = parse_item(text, number, lines, idx)?;
            if let Some(first) = seen.get(&item.id) {
                return Err(CameraError::DuplicateId {
                    id: item.id.clone(),
                    first: *first,
                    second: number,
                });
            }
            seen.insert(item.id.clone(), number);
            current = Some(item);
        } else if let Some(item) = current.as_mut() {
            item.push_line(text);
        }
    }
    flush(&mut current, items);
    Ok(())
}

/// Splits the checkbox marker, returning the state and the rest of
/// the line. A marker with nothing after it is a malformed register
/// line, not body text.
fn split_marker(text: &str, line: usize) -> Result<(bool, &str), CameraError> {
    let rest = text.trim_start_matches(' ');
    if let Some(after) = rest.strip_prefix("- [ ] ") {
        return Ok((false, after));
    }
    if let Some(after) = rest.strip_prefix("- [x] ") {
        return Ok((true, after));
    }
    Err(CameraError::MalformedLine {
        line,
        detail: "a checkbox marker with no bold run".to_owned(),
    })
}

/// A register item under construction: its span accumulates the raw
/// lines from the item line (after the checkbox marker) through its
/// continuations.
struct PartialItem {
    checked: bool,
    struck: bool,
    /// The item's id, readable for the duplicate check.
    id: String,
    title: String,
    span: Vec<String>,
}

impl PartialItem {
    /// Accumulates one raw span line.
    fn push_line(&mut self, line: &str) {
        self.span.push(line.to_owned());
    }
}

/// Parses a checkbox item's first line: the marker, the optional
/// strike wrapper, and the bold run (which may wrap). The item's raw
/// body span starts at the text after the checkbox marker, verbatim.
fn parse_item<'a>(
    text: &'a str,
    line: usize,
    lines: &[(&'a str, usize)],
    idx: usize,
) -> Result<PartialItem, CameraError> {
    let (checked, rest) = split_marker(text, line)?;
    let (struck, after_strike) = match rest.strip_prefix(STRIKE) {
        Some(after) => (true, after),
        None => (false, rest),
    };
    if struck && !checked {
        return Err(CameraError::StruckUnchecked { line });
    }
    let bold = after_strike
        .strip_prefix("**")
        .ok_or_else(|| CameraError::MalformedLine {
            line,
            detail: "a U item's bold run is missing".to_owned(),
        })?;
    let inner = scan_bold_run(bold, lines, idx, line)?;
    let Some((id, title)) = inner.split_once(MIDDLE_DOT_SEP) else {
        return Err(CameraError::MalformedLine {
            line,
            detail: "the bold run is not U-shaped: no ' · ' separator".to_owned(),
        });
    };
    if !is_u_id(id) {
        return Err(CameraError::MalformedLine {
            line,
            detail: format!("the bold run is not U-shaped: '{id}' is not a U id"),
        });
    }
    Ok(PartialItem {
        checked,
        struck,
        id: id.to_owned(),
        title: title.to_owned(),
        span: vec![rest.to_owned()],
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
) -> Result<String, CameraError> {
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
            return Err(CameraError::MalformedLine {
                line,
                detail: "a U item's bold run does not close".to_owned(),
            });
        };
        if is_register_construct(next) || next.starts_with('|') {
            return Err(CameraError::MalformedLine {
                line,
                detail: "a U item's bold run does not close".to_owned(),
            });
        }
        piece = next.trim();
        offset += 1;
    }
}

/// Ends the current item, if one is open, extracting its dependencies.
fn flush(current: &mut Option<PartialItem>, items: &mut Vec<RegisterItem>) {
    if let Some(item) = current.take() {
        let body = item.span.join("\n");
        let dependencies = extract_dependencies(&body);
        items.push(RegisterItem {
            id: item.id,
            title: item.title,
            checked: item.checked,
            struck: item.struck,
            body,
            dependencies,
        });
    }
}

/// Extracts the dependency assertions from an item's full raw span:
/// the span is whitespace-normalized (a line wrap replaces a space),
/// then each recognized phrase is matched once — the first id-bearing
/// occurrence per kind wins, in declaration order — with the phrase at
/// a word boundary and an id-shaped token directly following it. A
/// non-id or boundary-violating occurrence does not consume the scan:
/// prose like `blocked on the console` before a real `blocked on U2`
/// leaves the real one recognized.
fn extract_dependencies(body: &str) -> Vec<Dependency> {
    let normalized: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = Vec::new();
    for (kind, phrase) in [
        (DependencyKind::BlockedOn, BLOCKED_ON_PHRASE),
        (DependencyKind::HeldBackBy, HELD_BACK_PHRASE),
    ] {
        if let Some(id) = phrase_id(&normalized, phrase) {
            out.push(Dependency {
                kind,
                text: format!("{phrase}{id}"),
                id,
            });
        }
    }
    out
}

/// The id-shaped token following the first id-bearing occurrence of
/// `phrase` in `text`. Each occurrence must start at a word boundary —
/// the preceding character, if any, is not alphanumeric, so
/// `unblocked on` does not match — and an occurrence whose following
/// token is not id-shaped is skipped, not consumed: the scan continues
/// to the next occurrence.
fn phrase_id(text: &str, phrase: &str) -> Option<String> {
    let mut rest = text;
    while let Some((before, after)) = rest.split_once(phrase) {
        if !before
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric())
            && let Some(word) = after.split_whitespace().next()
            && let token = clean_id_word(word)
            && is_id_shaped(token)
        {
            return Some(token.to_owned());
        }
        rest = after;
    }
    None
}
