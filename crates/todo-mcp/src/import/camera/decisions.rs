//! The camera decisions.md parser (t46): the sign-off summary table
//! and the decision dossiers — plus the file's half of the dialect
//! AST.
//!
//! The summary table is the register: recognized by its header row
//! (`ID | Decision | Recommendation | Confidence | Gates | Status`)
//! wherever it appears in the file, and required — its absence would
//! silently omit every decision. Each row's cells are validated as
//! the mapping's inputs: the id (strikes and bold stripped), a
//! non-empty Recommendation, a Confidence spelling, an M-shaped Gates
//! cell, and a Status cell reading `open` or
//! `resolved <date> — <outcome>`.
//!
//! Every `###` line in the file must parse as a dossier header —
//! `### D<n> — <title>` or `### D<n> — <title> — RESOLVED <date>` —
//! because walking an unknown `###` past would silently skip a
//! malformed register line. Dossier bodies run to the next `###` or
//! end of input; a `##` section inside a body stays body text. The
//! dossiers contribute the id cross-check (the composition's set
//! equality) and the Gates-finding bodies; their prose is otherwise
//! not imported — the options prose stays in the board (§9's cutover).
//!
//! The dossier header's `— RESOLVED <date>` marker is provenance
//! only: the summary table's Status cell is the authoritative status
//! source (the measured board's D5 header lacks the marker while its
//! table row records the resolution), so the two are never
//! cross-checked against each other.

use super::error::CameraError;
use super::text::{
    EM_DASH_SEP, cells, collect_rows, find_table_header, is_d_id, is_day_date, is_m_id,
    split_lines, strip_id_noise,
};

/// The decisions summary table's header.
const SUMMARY_HEADER: [&str; 6] = [
    "ID",
    "Decision",
    "Recommendation",
    "Confidence",
    "Gates",
    "Status",
];
/// The dossier header's resolved marker, spaced.
const RESOLVED_MARKER: &str = " — RESOLVED ";

/// The parsed decisions.md: the summary table and the dossiers, in
/// board order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionsFile {
    /// The summary table's rows, in board order.
    pub summary: Vec<DecisionRow>,
    /// The dossiers, in board order.
    pub dossiers: Vec<Dossier>,
}

/// One summary-table row: the mapping's inputs for one decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRow {
    /// The D id with strikes and bold stripped.
    pub id: String,
    /// The Decision cell, raw.
    pub title: String,
    /// The Recommendation cell.
    pub recommendation: String,
    /// The Confidence cell's verdict.
    pub confidence: ConfidenceCell,
    /// The Gates cell's M id.
    pub gates: String,
    /// The Status cell's verdict.
    pub status: DecisionStatusCell,
}

/// The Confidence cell's verdict, parsed — the dialect's own
/// vocabulary, mapped to the domain's enum by the mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceCell {
    /// `low`.
    Low,
    /// `medium`.
    Medium,
    /// `high`.
    High,
}

/// The Status cell's verdict, parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionStatusCell {
    /// `resolved <date> — <outcome>`.
    Resolved {
        /// The resolution date as written (`YYYY-MM-DD`).
        date: String,
        /// The outcome text, raw.
        outcome: String,
    },
    /// `open`.
    Open,
}

/// One decision dossier: `### D<n> — <title>` or
/// `### D<n> — <title> — RESOLVED <date>`, with its body running to
/// the next `###` or end of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dossier {
    /// The D id.
    pub id: String,
    /// The dossier title.
    pub title: String,
    /// The header's `— RESOLVED <date>` marker's date, when present.
    pub resolved: Option<String>,
    /// The raw body to the next `###` or end of input, joined by
    /// `\n`.
    pub body: String,
}

/// Parses decisions.md into its dialect AST. Pure function of the
/// text — no I/O, no clock; file reading is the tool's.
pub fn parse_decisions(input: &str) -> Result<DecisionsFile, CameraError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let end_line = numbered.len();
    let mut file = DecisionsFile {
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
    file: &mut DecisionsFile,
    end_line: usize,
) -> Result<(), CameraError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &SUMMARY_HEADER) else {
        return Err(CameraError::MissingRegister {
            register: "the decisions summary table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [
            id_cell,
            title_cell,
            recommendation_cell,
            confidence_cell,
            gates_cell,
            status_cell,
        ] = row_cells.as_slice()
        else {
            return Err(CameraError::ColumnCount {
                line: number,
                expected: SUMMARY_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_d_id(&id) {
            return Err(CameraError::BadId {
                line: number,
                id,
                detail: "not a D id".to_owned(),
            });
        }
        if recommendation_cell.is_empty() {
            return Err(CameraError::MalformedLine {
                line: number,
                detail: "the Recommendation cell is empty".to_owned(),
            });
        }
        let confidence = parse_confidence(confidence_cell, number)?;
        let gates = strip_id_noise(gates_cell);
        if !is_m_id(&gates) {
            return Err(CameraError::BadId {
                line: number,
                id: gates,
                detail: "not an M id".to_owned(),
            });
        }
        let status = parse_status_cell(status_cell, number)?;
        file.summary.push(DecisionRow {
            id,
            title: title_cell.clone(),
            recommendation: recommendation_cell.clone(),
            confidence,
            gates,
            status,
        });
    }
    Ok(())
}

/// The Confidence cell: `low`/`medium`/`high`, case-sensitive — any
/// other spelling is a named error.
fn parse_confidence(cell: &str, line: usize) -> Result<ConfidenceCell, CameraError> {
    match cell {
        "low" => Ok(ConfidenceCell::Low),
        "medium" => Ok(ConfidenceCell::Medium),
        "high" => Ok(ConfidenceCell::High),
        _ => Err(CameraError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Confidence cell must read 'low', 'medium', or 'high'".to_owned(),
        }),
    }
}

/// The Status cell: exactly `open`, or `resolved <date> — <outcome>`
/// with a shape-valid date and the outcome carried raw.
fn parse_status_cell(cell: &str, line: usize) -> Result<DecisionStatusCell, CameraError> {
    if cell == "open" {
        return Ok(DecisionStatusCell::Open);
    }
    let Some(after) = cell.strip_prefix("resolved ") else {
        return Err(CameraError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Status cell must read 'open' or 'resolved <date> — <outcome>'".to_owned(),
        });
    };
    let Some(date) = after.get(..10) else {
        return Err(CameraError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the resolved status's date is missing".to_owned(),
        });
    };
    if !is_day_date(date) {
        return Err(CameraError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the resolved status's date is not YYYY-MM-DD".to_owned(),
        });
    }
    let Some(rest) = after.get(10..) else {
        return Err(CameraError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the resolved status's date is missing".to_owned(),
        });
    };
    let Some(outcome) = rest.strip_prefix(EM_DASH_SEP) else {
        return Err(CameraError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the resolved status lacks ' — ' after the date".to_owned(),
        });
    };
    Ok(DecisionStatusCell::Resolved {
        date: date.to_owned(),
        outcome: outcome.to_owned(),
    })
}

/// The dossiers: every `###` line must parse as a dossier header;
/// bodies run to the next `###` or end of input.
fn parse_dossiers(lines: &[(&str, usize)], file: &mut DecisionsFile) -> Result<(), CameraError> {
    let mut current: Option<PartialDossier> = None;
    for &(text, number) in lines {
        if text.starts_with("### ") {
            if let Some(dossier) = current.take() {
                file.dossiers.push(finish_dossier(dossier));
            }
            current = Some(parse_dossier_header(text, number)?);
        } else if let Some(dossier) = current.as_mut() {
            dossier.body.push(text.to_owned());
        }
    }
    if let Some(dossier) = current.take() {
        file.dossiers.push(finish_dossier(dossier));
    }
    Ok(())
}

/// A dossier under construction: its body accumulates raw lines.
struct PartialDossier {
    id: String,
    title: String,
    resolved: Option<String>,
    body: Vec<String>,
}

/// Parses one dossier header line: `### D<n> — <title>` or
/// `### D<n> — <title> — RESOLVED <date>`.
fn parse_dossier_header(text: &str, line: usize) -> Result<PartialDossier, CameraError> {
    let rest = text.strip_prefix("### ").unwrap_or(text);
    let Some((id, after_id)) = rest.split_once(EM_DASH_SEP) else {
        return Err(CameraError::MalformedLine {
            line,
            detail: "a dossier header has no ' — ' separator after the id".to_owned(),
        });
    };
    let id = strip_id_noise(id);
    if !is_d_id(&id) {
        return Err(CameraError::BadId {
            line,
            id,
            detail: "not a D id".to_owned(),
        });
    }
    let (title, resolved) = match after_id.split_once(RESOLVED_MARKER) {
        Some((title, date)) => {
            if !is_day_date(date) {
                return Err(CameraError::BadStatus {
                    line,
                    text: text.to_owned(),
                    detail: "the RESOLVED marker's date is not YYYY-MM-DD".to_owned(),
                });
            }
            (title, Some(date.to_owned()))
        }
        None => (after_id, None),
    };
    Ok(PartialDossier {
        id,
        title: title.to_owned(),
        resolved,
        body: Vec::new(),
    })
}

/// Finishes a dossier: joins its body into the raw text.
fn finish_dossier(dossier: PartialDossier) -> Dossier {
    Dossier {
        id: dossier.id,
        title: dossier.title,
        resolved: dossier.resolved,
        body: dossier.body.join("\n"),
    }
}
