//! The §4 doc register (t42): the "Already expanded" list and the
//! milestone table, split out of `board.rs` by responsibility — §4 is
//! self-contained.
//!
//! The register's rules: the expanded list is the sentence span from
//! after `Already expanded (N):` to the first terminating period (the
//! entries carry no periods, and the sentence continues as prose
//! after one); its entries are `README`, `00-overview`, two-digit
//! numbers, and the appendices phrase — whose range is pinned to A–G,
//! so a different range is a named unknown entry rather than a silent
//! guess. The milestone table's middle cells drop
//! struck spans (deepened docs) and annotations — annotations first,
//! because an annotation can carry `~~` inside itself, which would
//! corrupt a strike-first toggle scan — and a cell opening with the
//! em-dash marker or an empty cell contributes nothing. The count in `(N)` is not cross-checked: the
//! entry count is derivable, and the board's own count is prose.
//!
//! Consistency: duplicate keys within one source, and the
//! both-Expanded-and-unstruck-Summary contradiction, are named errors;
//! struck-but-not-expanded is legal and unchecked (the board's §4
//! deepens by annotation as well as by strike).

use std::collections::{BTreeMap, BTreeSet};

use super::ast::{DocDepth, DocEntry, TodoMd};
use super::error::ImportError;
use super::table::{EM_DASH, is_m_id, is_register_construct, remove_annotations};
use crate::import::helpers::{
    cells, collect_rows, find_table_header, remove_struck_spans, strip_emphasis,
};

/// §4's milestone-table header.
const MILESTONE_TABLE_HEADER: [&str; 3] = ["Milestone", "Docs still summary", "Count"];
/// The expanded list's appendices phrase, pinned to the A–G range.
const APPENDICES_SUFFIX: &str = "appendices (A–G)";

/// §4: the doc register — the expanded list and the milestone table.
pub(crate) fn parse_doc_register(
    lines: &[(&str, usize)],
    todo: &mut TodoMd,
    end_line: usize,
) -> Result<(), ImportError> {
    let Some((list_idx, list_text, list_number)) = find_expanded_line(lines) else {
        return Err(ImportError::MissingRegister {
            register: "§4's 'Already expanded' list".to_owned(),
            line: end_line,
        });
    };
    let expanded = parse_expanded_list(lines, list_idx, list_text, list_number)?;
    let Some((table_idx, _, _)) = find_table_header(lines, &MILESTONE_TABLE_HEADER) else {
        return Err(ImportError::MissingRegister {
            register: "§4's milestone table".to_owned(),
            line: end_line,
        });
    };
    let mut summary: Vec<(String, usize, String)> = Vec::new();
    let (rows, _) = collect_rows(lines, table_idx, is_register_construct);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [milestone_cell, middle_cell, _count] = row_cells.as_slice() else {
            return Err(ImportError::ColumnCount {
                line: number,
                expected: MILESTONE_TABLE_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let milestone = strip_emphasis(milestone_cell).trim().to_owned();
        if !is_m_id(&milestone) {
            return Err(ImportError::BadId {
                line: number,
                id: milestone,
                detail: "not an M id".to_owned(),
            });
        }
        for key in summary_keys(middle_cell, number)? {
            summary.push((key, number, milestone.clone()));
        }
    }
    check_doc_register(&expanded, &summary)?;
    for (key, _) in expanded {
        todo.docs.push(DocEntry {
            key,
            depth: DocDepth::Expanded,
            milestone: None,
        });
    }
    for (key, _, milestone) in summary {
        todo.docs.push(DocEntry {
            key,
            depth: DocDepth::Summary,
            milestone: Some(milestone),
        });
    }
    Ok(())
}

/// The first line starting `Already expanded (` — the register's
/// opening line.
fn find_expanded_line<'a>(lines: &'a [(&'a str, usize)]) -> Option<(usize, &'a str, usize)> {
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if text.starts_with("Already expanded (") {
            return Some((idx, text, number));
        }
    }
    None
}

/// The expanded list's `(key, line)` pairs. The list is the sentence
/// span: the text from after `Already expanded (N):` to the first
/// terminating period — the entries carry no periods, and the
/// sentence continues as prose after one. When no period occurs, the
/// span runs to the section's end and the entries must parse or be
/// named.
fn parse_expanded_list(
    lines: &[(&str, usize)],
    list_idx: usize,
    list_text: &str,
    list_number: usize,
) -> Result<Vec<(String, usize)>, ImportError> {
    let rest = list_text
        .strip_prefix("Already expanded (")
        .unwrap_or(list_text);
    let Some((_, list_start)) = rest.split_once("):") else {
        return Err(ImportError::MalformedLine {
            line: list_number,
            detail: "the 'Already expanded' line has no '):' after its count".to_owned(),
        });
    };
    let mut text = String::new();
    for (idx, &(line, _)) in lines.iter().enumerate().skip(list_idx) {
        let piece = if idx == list_idx { list_start } else { line };
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(piece);
        if let Some(dot) = text.find('.') {
            text.truncate(dot);
            break;
        }
    }
    let mut expanded: Vec<(String, usize)> = Vec::new();
    for raw in text.split(',') {
        let entry = raw.trim();
        if entry.is_empty() {
            continue;
        }
        let keys = expanded_keys(entry).ok_or_else(|| ImportError::BadDocEntry {
            line: list_number,
            entry: entry.to_owned(),
        })?;
        for key in keys {
            expanded.push((key, list_number));
        }
    }
    Ok(expanded)
}

/// The keys one expanded-list entry contributes: `README`,
/// `00-overview`, a two-digit number, or the appendices phrase. The
/// appendices range is pinned to A–G; a different range
/// is an unknown entry and is named.
fn expanded_keys(entry: &str) -> Option<Vec<String>> {
    let clean = strip_emphasis(entry).trim().to_owned();
    let clean = clean.strip_prefix("and ").unwrap_or(&clean);
    if clean == "README" {
        return Some(vec!["README".to_owned()]);
    }
    if clean == "00-overview" {
        return Some(vec!["00-overview".to_owned()]);
    }
    if clean.len() == 2 && clean.bytes().all(|b| b.is_ascii_digit()) {
        return Some(vec![clean.to_owned()]);
    }
    if clean.ends_with(APPENDICES_SUFFIX) {
        return Some("ABCDEFG".chars().map(|c| c.to_string()).collect());
    }
    None
}

/// The summary keys a milestone-table middle cell contributes: the
/// unstruck `<number> <name>` entries. Struck spans (deepened docs)
/// and annotations are dropped — annotations first, because an
/// annotation can carry `~~` inside itself, which would corrupt a
/// strike-first toggle scan. A cell opening with the em-dash marker
/// or an empty cell contributes nothing.
fn summary_keys(cell: &str, line: usize) -> Result<Vec<String>, ImportError> {
    let trimmed = cell.trim();
    if trimmed.is_empty() || trimmed.starts_with(EM_DASH) {
        return Ok(Vec::new());
    }
    let unstruck = remove_struck_spans(&remove_annotations(trimmed));
    let mut keys = Vec::new();
    for raw in unstruck.split(',') {
        let entry = strip_emphasis(raw).trim().to_owned();
        if entry.is_empty() {
            continue;
        }
        let Some((number, name)) = entry.split_once(' ') else {
            return Err(ImportError::BadDocEntry {
                line,
                entry: entry.clone(),
            });
        };
        if number.len() != 2 || !number.bytes().all(|b| b.is_ascii_digit()) || name.is_empty() {
            return Err(ImportError::BadDocEntry { line, entry });
        }
        keys.push(number.to_owned());
    }
    Ok(keys)
}

/// The doc register's consistency checks: duplicate keys within one
/// source, and the both-Expanded-and-unstruck-Summary contradiction.
/// Struck-but-not-expanded is legal and unchecked.
fn check_doc_register(
    expanded: &[(String, usize)],
    summary: &[(String, usize, String)],
) -> Result<(), ImportError> {
    let mut seen_expanded: BTreeMap<String, usize> = BTreeMap::new();
    for (key, line) in expanded {
        if let Some(first) = seen_expanded.get(key) {
            return Err(ImportError::DuplicateDocKey {
                key: key.clone(),
                first: *first,
                second: *line,
            });
        }
        seen_expanded.insert(key.clone(), *line);
    }
    let mut seen_summary: BTreeMap<String, usize> = BTreeMap::new();
    for (key, line, _) in summary {
        if let Some(first) = seen_summary.get(key) {
            return Err(ImportError::DuplicateDocKey {
                key: key.clone(),
                first: *first,
                second: *line,
            });
        }
        seen_summary.insert(key.clone(), *line);
    }
    let expanded_set: BTreeSet<&str> = expanded.iter().map(|(key, _)| key.as_str()).collect();
    for (key, line, _) in summary {
        if expanded_set.contains(key.as_str()) {
            return Err(ImportError::DocContradiction {
                key: key.clone(),
                line: *line,
            });
        }
    }
    Ok(())
}
