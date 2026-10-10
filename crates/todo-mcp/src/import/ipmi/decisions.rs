//! The ipmi-collector decisions.md parser (t77): the sign-off summary
//! table and the decision dossiers — plus the file's half of the
//! dialect AST.
//!
//! The summary table is the register: recognized by its header row
//! (`Id | Title | Status | Gates | Recommendation`) wherever it
//! appears in the file, and required — its absence would silently
//! omit every decision. A duplicated D id is a named error naming
//! both lines: the composition's id cross-check compares sets (which
//! dedupe), so a silent duplicate would pass it and shift the
//! mapping's decision id range past the board's numbering. Each row's
//! cells are validated as the mapping's inputs: the id (`D<digits>`,
//! the un-hyphenated family — §5's `D-N` debt ids are a different
//! register), a non-empty Recommendation, a Gates cell carried raw
//! (the mapping scans it for id-bearing tokens), and a Status cell
//! reading `**signed** <resolution>` or `**deferred** <remainder>` —
//! the leading bold word is the status; a signed row's remainder is
//! the resolution and must be non-empty (the closure discipline
//! demands a resolution — refuse rather than invent one); a deferred
//! row's remainder is carried raw and dropped by the map.
//!
//! The dossiers: every `## ` line in the file must parse as
//! `## D<n> · <title> — <tail>` — walking an unknown `##` past would
//! silently skip a malformed register line. The header's tail (the
//! status prose) is validated for presence and dropped: the summary
//! table's Status cell is the one status authority, and the measured
//! board's D2 header says `open` where its table row says `deferred`
//! — both mean not-signed, and only the table is the register. The
//! dossier bodies run to the next `## ` or end of input and are
//! carried raw; the mapping drops them (the options prose stays in
//! the board, §9's cutover).

use std::collections::BTreeMap;

use super::board::{BOLD, is_register_construct};
use super::error::IpmiError;
use crate::import::helpers::{cells, collect_rows, find_table_header, split_lines, strip_id_noise};

/// The decisions summary table's header — five columns, the measured
/// shape.
const SUMMARY_HEADER: [&str; 5] = ["Id", "Title", "Status", "Gates", "Recommendation"];
/// The dossier header's id/title separator (U+00B7 middle dot),
/// spaced.
const MIDDLE_DOT_SEP: &str = " · ";
/// The dossier header's title/tail separator: the spaced em-dash.
const EM_DASH_SEP: &str = " — ";

/// The parsed decisions.md: the summary table and the dossiers, in
/// board order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpmiDecisions {
    /// The summary table's rows, in board order.
    pub summary: Vec<DecisionRow>,
    /// The dossiers, in board order.
    pub dossiers: Vec<Dossier>,
}

/// One summary-table row: the mapping's inputs for one decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRow {
    /// The D id with noise stripped.
    pub id: String,
    /// The Title cell, raw.
    pub title: String,
    /// The Status cell's verdict.
    pub status: DecisionStatusCell,
    /// The Gates cell, raw — the mapping scans it for id-bearing
    /// tokens.
    pub gates: String,
    /// The Recommendation cell.
    pub recommendation: String,
}

/// The Status cell's verdict, parsed: the leading bold word, with the
/// remainder after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionStatusCell {
    /// `**signed** <resolution>` — the remainder is the resolution.
    Signed {
        /// The resolution text, e.g. `(b) 2026-10-08`.
        resolution: String,
    },
    /// `**deferred** <remainder>` — the remainder is carried raw and
    /// dropped by the map (the board keeps the deferral prose).
    Deferred {
        /// The remainder text, e.g. `2026-10-09`.
        remainder: String,
    },
}

/// One decision dossier: `## D<n> · <title> — <tail>`, with its body
/// running to the next `## ` or end of input. The header's tail is
/// validated for presence and dropped at parse — the summary table's
/// Status cell is the one status authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dossier {
    /// The D id.
    pub id: String,
    /// The dossier title.
    pub title: String,
    /// The raw body to the next `## ` or end of input, joined by
    /// `\n`.
    pub body: String,
}

/// Parses decisions.md into its dialect AST. Pure function of the
/// text — no I/O, no clock; file reading is the tool's.
pub fn parse_decisions(input: &str) -> Result<IpmiDecisions, IpmiError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let end_line = numbered.len();
    let mut file = IpmiDecisions {
        summary: Vec::new(),
        dossiers: Vec::new(),
    };
    parse_summary_table(&numbered, &mut file, end_line)?;
    parse_dossiers(&numbered, &mut file)?;
    Ok(file)
}

/// The summary table: the register, recognized by its header row.
fn parse_summary_table(
    lines: &[(&str, usize)],
    file: &mut IpmiDecisions,
    end_line: usize,
) -> Result<(), IpmiError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &SUMMARY_HEADER) else {
        return Err(IpmiError::MissingRegister {
            register: "the decisions summary table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx, is_register_construct);
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [
            id_cell,
            title_cell,
            status_cell,
            gates_cell,
            recommendation_cell,
        ] = row_cells.as_slice()
        else {
            return Err(IpmiError::ColumnCount {
                line: number,
                expected: SUMMARY_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_d_id(&id) {
            return Err(IpmiError::BadId {
                line: number,
                id,
                detail: "not a D id".to_owned(),
            });
        }
        // A duplicated D id is a named error naming both lines: the
        // composition's id cross-check compares sets (which dedupe),
        // so a silent duplicate would pass it and shift the mapping's
        // decision id range past the board's numbering.
        if let Some(first) = seen.get(&id) {
            return Err(IpmiError::DuplicateId {
                id,
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        if recommendation_cell.is_empty() {
            return Err(IpmiError::MalformedLine {
                line: number,
                detail: "the Recommendation cell is empty".to_owned(),
            });
        }
        let status = parse_status_cell(status_cell, number)?;
        file.summary.push(DecisionRow {
            id,
            title: title_cell.clone(),
            status,
            gates: gates_cell.clone(),
            recommendation: recommendation_cell.clone(),
        });
    }
    Ok(())
}

/// The Status cell: `**signed** <resolution>` or
/// `**deferred** <remainder>`. The leading bold word is the status; the signed
/// remainder is the resolution and must be non-empty (the closure
/// discipline demands a resolution — refuse rather than invent); the
/// deferred remainder is carried raw.
fn parse_status_cell(cell: &str, line: usize) -> Result<DecisionStatusCell, IpmiError> {
    let Some(after_open) = cell.strip_prefix(BOLD) else {
        return Err(IpmiError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Status cell's bold word is missing".to_owned(),
        });
    };
    let Some((word, rest)) = after_open.split_once(BOLD) else {
        return Err(IpmiError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Status cell's bold word does not close".to_owned(),
        });
    };
    let rest = rest.trim();
    match word.trim() {
        "signed" => {
            if rest.is_empty() {
                return Err(IpmiError::BadStatus {
                    line,
                    text: cell.to_owned(),
                    detail: "a signed decision's Status cell carries no resolution".to_owned(),
                });
            }
            Ok(DecisionStatusCell::Signed {
                resolution: rest.to_owned(),
            })
        }
        "deferred" => Ok(DecisionStatusCell::Deferred {
            remainder: rest.to_owned(),
        }),
        _ => Err(IpmiError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Status cell's bold word must read 'signed' or 'deferred'".to_owned(),
        }),
    }
}

/// The dossiers: every `## ` line must parse as a dossier header;
/// bodies run to the next `## ` or end of input.
fn parse_dossiers(lines: &[(&str, usize)], file: &mut IpmiDecisions) -> Result<(), IpmiError> {
    let mut current: Option<PartialDossier> = None;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for &(text, number) in lines {
        if text.starts_with("## ") {
            if let Some(dossier) = current.take() {
                file.dossiers.push(finish_dossier(dossier));
            }
            let dossier = parse_dossier_header(text, number)?;
            // A duplicated D id is a named error naming both lines: the
            // composition's id cross-check compares sets (which dedupe),
            // so a silent duplicate would pass it and shift the mapping's
            // decision id range past the board's numbering.
            if let Some(first) = seen.get(&dossier.id) {
                return Err(IpmiError::DuplicateId {
                    id: dossier.id,
                    first: *first,
                    second: number,
                });
            }
            seen.insert(dossier.id.clone(), number);
            current = Some(dossier);
        } else if let Some(dossier) = current.as_mut() {
            dossier.body.push(text.to_owned());
        }
    }
    if let Some(dossier) = current.take() {
        file.dossiers.push(finish_dossier(dossier));
    }
    Ok(())
}

/// A dossier under construction: its body accumulates the raw lines.
struct PartialDossier {
    id: String,
    title: String,
    body: Vec<String>,
}

/// Parses one dossier header line: `## D<n> · <title> — <tail>`. The
/// tail is validated for presence and dropped — the summary table's
/// Status cell is the one status authority (the measured D2 header
/// says `open` where its row says `deferred`).
fn parse_dossier_header(text: &str, line: usize) -> Result<PartialDossier, IpmiError> {
    let rest = text.strip_prefix("## ").unwrap_or(text);
    let Some((id, after_id)) = rest.split_once(MIDDLE_DOT_SEP) else {
        return Err(IpmiError::MalformedLine {
            line,
            detail: "a dossier header has no ' · ' separator after the id".to_owned(),
        });
    };
    let Some((title, _tail)) = after_id.split_once(EM_DASH_SEP) else {
        return Err(IpmiError::MalformedLine {
            line,
            detail: "a dossier header has no ' — ' separator after the title".to_owned(),
        });
    };
    let id = strip_id_noise(id);
    if !is_d_id(&id) {
        return Err(IpmiError::BadId {
            line,
            id,
            detail: "not a D id".to_owned(),
        });
    }
    Ok(PartialDossier {
        id,
        title: title.to_owned(),
        body: Vec::new(),
    })
}

/// Finishes a dossier: joins its body into the raw text.
fn finish_dossier(dossier: PartialDossier) -> Dossier {
    Dossier {
        id: dossier.id,
        title: dossier.title,
        body: dossier.body.join("\n"),
    }
}

/// A D id: `D<digits>` exactly — the decisions registers' measured
/// shape, distinct from §5's hyphenated `D-N` family.
fn is_d_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('D') else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}
