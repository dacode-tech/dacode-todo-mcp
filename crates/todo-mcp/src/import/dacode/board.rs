//! The TODO.md parser (t42): the §1–§6 section walker and the four
//! imported sections' registers.
//!
//! The walker: section boundaries are `## ` at line start,
//! identified by the number in the header — never by position. §1, §3
//! and any section past §6 are walked past (not imported); §2, §4, §5
//! and §6 must all be present, and a missing one is a named error
//! naming it. A duplicated imported section is a named error.
//!
//! Strictness beyond the spec's explicit list, one judgment call
//! throughout: each imported section's *register* is required — §2's
//! decisions table, and §4's registers via `doc_register` — because an
//! absent register silently omits every entry it holds, which is the
//! failure mode the B1a rule exists to prevent. §5's milestone items
//! and §6's U items are not required (a section may legitimately carry
//! none), but every top-level checkbox that does appear must parse.
//!
//! §2 specifics (the fix round's widenings): a row carries four
//! cells, or five when the B appendix's Gates column rides inline —
//! the fifth is carried raw, never parsed as Gates. The Status cell's
//! `→`-separated segments scan right-to-left for the last one reading
//! a known spelling; a cell with none carries its final segment as
//! `SummaryStatus::Prose`.
//!
//! §5 specifics: sub-step tables are recognized by their header
//! wherever they appear in §5 (flat storage does not depend on the
//! table preceding its milestone item), and a sub-step table ends the
//! enclosing milestone item's body. A `###` line inside §5 is body
//! text — only §6 gives `###` structural meaning.
//!
//! §6 specifics: a `###` line opens a subsection; a top-level
//! checkbox before the first subsection opens the implicit leading
//! group (`title: None`) rather than erroring, because a register line
//! must parse wherever it appears.
//!
//! The checkbox-item grammar and marker extraction live in `items.rs`;
//! the wrap rules for table rows live in `table.rs`.

use std::collections::BTreeMap;

use super::ast::{MilestoneItem, SubStep, Subsection, SummaryRow, SummaryStatus, TodoMd, UItem};
use super::doc_register;
use super::error::ImportError;
use super::items::{ItemKind, PartialItem, finish_item, is_checkbox_line, parse_item_start};
use super::table::{
    ARROW, cells, collect_rows, find_table_header, is_b_id, is_header, is_substep_id, split_lines,
    strip_id_noise, strip_strikes,
};

/// §2's decisions-table header.
const DECISIONS_HEADER: [&str; 4] = ["#", "Decision", "Recommendation", "Status"];
/// §5's sub-step-table header (an empty first cell).
const SUBSTEP_HEADER: [&str; 3] = ["", "Sub-step", "Gate"];
/// §2's resolved Status-cell spelling.
const STATUS_RESOLVED: &str = "✅ resolved";
/// §2's half-applied Status-cell spelling.
const STATUS_HALF_APPLIED: &str = "⚠️ resolved, half-applied";

/// The per-section line lists: section number → its lines with 1-based
/// numbers.
type SectionMap<'a> = BTreeMap<u8, Vec<(&'a str, usize)>>;

/// Parses TODO.md into its dialect AST. Pure function of the text —
/// no I/O, no clock; file reading is the tool's (t44).
pub fn parse_todo_md(input: &str) -> Result<TodoMd, ImportError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let sections = split_sections(&numbered)?;
    let end_line = numbered.len();
    let mut todo = TodoMd {
        decisions: Vec::new(),
        b1a: String::new(),
        docs: Vec::new(),
        milestones: Vec::new(),
        substeps: Vec::new(),
        followups: Vec::new(),
    };
    parse_decisions_section(
        require_section(&sections, 2, end_line)?,
        &mut todo,
        end_line,
    )?;
    doc_register::parse_doc_register(
        require_section(&sections, 4, end_line)?,
        &mut todo,
        end_line,
    )?;
    parse_milestones_section(require_section(&sections, 5, end_line)?, &mut todo)?;
    parse_followups_section(require_section(&sections, 6, end_line)?, &mut todo)?;
    Ok(todo)
}

/// The imported section's lines, or the named missing-section error.
fn require_section<'a>(
    sections: &'a BTreeMap<u8, Vec<(&'a str, usize)>>,
    number: u8,
    end_line: usize,
) -> Result<&'a [(&'a str, usize)], ImportError> {
    sections
        .get(&number)
        .map(Vec::as_slice)
        .ok_or_else(|| ImportError::MissingSection {
            section: format!("§{number}"),
            line: end_line,
        })
}

/// Splits the input into per-section line lists. A `## ` header ends
/// the current section whatever it is; a header naming an imported
/// section opens it; everything else (§1, §3, sections past §6,
/// unnumbered headers) is walked past.
fn split_sections<'a>(lines: &'a [(&'a str, usize)]) -> Result<SectionMap<'a>, ImportError> {
    let mut sections: SectionMap<'_> = BTreeMap::new();
    let mut current: Option<u8> = None;
    for &(text, number) in lines {
        if text.starts_with("## ") {
            current = None;
            if let Some(section) = section_number(text)
                && is_imported(section)
            {
                if sections.contains_key(&section) {
                    return Err(ImportError::MalformedLine {
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

/// Whether §`n` is imported (§2, §4, §5, §6).
fn is_imported(section: u8) -> bool {
    matches!(section, 2 | 4 | 5 | 6)
}

/// §2: the decisions table and the B1a paragraph.
fn parse_decisions_section(
    lines: &[(&str, usize)],
    todo: &mut TodoMd,
    end_line: usize,
) -> Result<(), ImportError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &DECISIONS_HEADER) else {
        return Err(ImportError::MissingRegister {
            register: "§2's decisions table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let (id_cell, title_cell, status_cell, extra) = match row_cells.as_slice() {
            [id, title, _recommendation, status] => (id, title, status, None),
            [id, title, _recommendation, status, extra] => (id, title, status, Some(extra)),
            _ => {
                return Err(ImportError::ColumnCount {
                    line: number,
                    expected: DECISIONS_HEADER.len(),
                    found: row_cells.len(),
                });
            }
        };
        let id = strip_id_noise(id_cell);
        if !is_b_id(&id) {
            return Err(ImportError::BadId {
                line: number,
                id,
                detail: "not a B id".to_owned(),
            });
        }
        let status = parse_summary_status(status_cell);
        todo.decisions.push(SummaryRow {
            id,
            title: title_cell.clone(),
            status,
            extra: extra.cloned(),
        });
    }
    let Some((b1a_idx, _, _)) = find_b1a_line(lines) else {
        return Err(ImportError::MissingB1a { line: end_line });
    };
    let mut paragraph: Vec<&str> = Vec::new();
    for &(text, _) in lines.iter().skip(b1a_idx) {
        if text.trim().is_empty() {
            break;
        }
        paragraph.push(text);
    }
    todo.b1a = paragraph.join("\n");
    Ok(())
}

/// The first line starting `B1a (` — §2's required sub-entry
/// paragraph.
fn find_b1a_line<'a>(lines: &'a [(&'a str, usize)]) -> Option<(usize, &'a str, usize)> {
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if text.starts_with("B1a (") {
            return Some((idx, text, number));
        }
    }
    None
}

/// §2's Status cell: strip strikes, scan the `→`-separated segments
/// right-to-left, and take the last one reading a known spelling; a
/// cell with none carries its final segment as prose.
fn parse_summary_status(cell: &str) -> SummaryStatus {
    let stripped = strip_strikes(cell);
    let segments: Vec<&str> = stripped.split(ARROW).map(str::trim).collect();
    for segment in segments.iter().rev() {
        if *segment == STATUS_RESOLVED {
            return SummaryStatus::Resolved;
        }
        if *segment == STATUS_HALF_APPLIED {
            return SummaryStatus::HalfApplied;
        }
    }
    let final_segment = segments.last().copied().unwrap_or_default();
    SummaryStatus::Prose(final_segment.to_owned())
}

/// §5: the milestone items and the sub-step tables.
fn parse_milestones_section(lines: &[(&str, usize)], todo: &mut TodoMd) -> Result<(), ImportError> {
    let mut current: Option<PartialItem> = None;
    let mut skip_past: usize = 0;
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if idx < skip_past {
            continue;
        }
        if is_header(text, &SUBSTEP_HEADER) {
            flush_milestone(&mut current, todo);
            skip_past = collect_substep_rows(lines, idx, todo)? + 1;
        } else if is_checkbox_line(text) {
            flush_milestone(&mut current, todo);
            current = Some(parse_item_start(text, number, lines, idx, ItemKind::M)?);
        } else if let Some(item) = current.as_mut() {
            item.push_line(text);
        }
    }
    flush_milestone(&mut current, todo);
    Ok(())
}

/// Collects one sub-step table's rows through the shared wrap-aware
/// collector, returning the index of the last line the table consumed.
/// Rows must carry three cells and an `M<digits>.<digits>` id, else a
/// named error.
fn collect_substep_rows(
    lines: &[(&str, usize)],
    header_idx: usize,
    todo: &mut TodoMd,
) -> Result<usize, ImportError> {
    let (rows, last) = collect_rows(lines, header_idx);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [id_cell, text_cell, gate_cell] = row_cells.as_slice() else {
            return Err(ImportError::ColumnCount {
                line: number,
                expected: SUBSTEP_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_substep_id(&id) {
            return Err(ImportError::BadId {
                line: number,
                id,
                detail: "not an M<digits>.<digits> sub-step id".to_owned(),
            });
        }
        todo.substeps.push(SubStep {
            id,
            text: text_cell.clone(),
            gate: gate_cell.clone(),
        });
    }
    Ok(last)
}

/// §6: the subsections and their U items.
fn parse_followups_section(lines: &[(&str, usize)], todo: &mut TodoMd) -> Result<(), ImportError> {
    let mut subsections: Vec<Subsection> = Vec::new();
    let mut current: Option<PartialItem> = None;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if text.starts_with("### ") {
            flush_u(&mut current, &mut subsections);
            subsections.push(Subsection {
                title: Some(text.strip_prefix("### ").unwrap_or("").to_owned()),
                items: Vec::new(),
            });
        } else if is_checkbox_line(text) {
            flush_u(&mut current, &mut subsections);
            let item = parse_item_start(text, number, lines, idx, ItemKind::U)?;
            if let Some(first) = seen.get(&item.id) {
                return Err(ImportError::DuplicateId {
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
    flush_u(&mut current, &mut subsections);
    todo.followups = subsections;
    Ok(())
}

/// Ends the current milestone item, if one is open.
fn flush_milestone(current: &mut Option<PartialItem>, todo: &mut TodoMd) {
    if let Some(item) = current.take() {
        let finished = finish_item(item);
        todo.milestones.push(MilestoneItem {
            id: finished.id,
            title: finished.title,
            checked: finished.checked,
            body: finished.body,
            markers: finished.markers,
        });
    }
}

/// Ends the current U item, if one is open, into the last subsection —
/// creating the implicit leading group when none exists, so a flushed
/// item always lands somewhere.
fn flush_u(current: &mut Option<PartialItem>, subsections: &mut Vec<Subsection>) {
    if let Some(item) = current.take() {
        let finished = finish_item(item);
        if subsections.is_empty() {
            subsections.push(Subsection {
                title: None,
                items: Vec::new(),
            });
        }
        if let Some(last) = subsections.last_mut() {
            last.items.push(UItem {
                id: finished.id,
                title: finished.title,
                checked: finished.checked,
                body: finished.body,
                markers: finished.markers,
            });
        }
    }
}
