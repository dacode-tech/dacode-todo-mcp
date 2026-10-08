//! The own decisions.md parser (t54): the sign-off summary table and
//! the decision dossiers — plus the file's half of the dialect AST.
//!
//! The summary table is the register: recognized by its header row
//! (`ID | Decision | Recommendation | Confidence | Gates | Status`)
//! wherever it appears in the file, and required — its absence would
//! silently omit every decision. The id is column 1, struck
//! (`~~**D<n>**~~`) when resolved — the strike is typographic
//! confirmation, and the Status cell is the authoritative status
//! source, so the two are never cross-checked against each other (the
//! camera rule). A duplicated D id is a named error naming both
//! lines: the composition's id cross-check compares sets (which
//! dedupe), so a silent duplicate would pass it and shift the
//! mapping's decision id range past the board's numbering. Each row's
//! cells are validated as the mapping's
//! inputs: the id (strikes and bold stripped), a non-empty
//! Recommendation, a Confidence spelling, a Gates cell that is either
//! an M id or the bare em-dash — "gates nothing", the post-roadmap
//! shape D19 introduced — and
//! a Status cell reading `resolved [<option>] <date> [— <note>]` or
//! `drafted <date> [— <note>]` — the resolved spelling with the
//! option letter and the note each optional (the measured rows carry
//! both shapes), the drafted spelling the board's historical open
//! form.
//!
//! Every `###` line in the file must parse as a dossier header —
//! `### D<n> — <title>` — because walking an unknown `###` past
//! would silently skip a malformed register line. Dossier bodies run
//! to the next `###` or end of input; a `##` section inside a body
//! stays body text. The dossiers contribute the id cross-check (the
//! composition's set equality) and the Gates findings' context
//! bodies; their prose is otherwise not imported — the options prose
//! stays in the board (§9's cutover). The resolution banner lives in
//! the dossier *body* on this board (`**Resolved <date> — …**`), not
//! in the header, so the header carries no resolved marker and none
//! is parsed.

use std::collections::BTreeMap;

use super::error::OwnError;
use super::text::{EM_DASH_SEP, is_d_id, is_m_id, is_register_construct};
use crate::import::helpers::{
    cells, collect_rows, find_table_header, is_day_date, split_lines, strip_id_noise,
};

/// The decisions summary table's header — camera's exact six columns.
const SUMMARY_HEADER: [&str; 6] = [
    "ID",
    "Decision",
    "Recommendation",
    "Confidence",
    "Gates",
    "Status",
];
/// The Status cell's resolved spelling prefix.
const STATUS_RESOLVED: &str = "resolved";
/// The Status cell's drafted spelling prefix — the historical open
/// form; the live board holds none today, and the grammar records the
/// spelling so a future drafted row parses.
const STATUS_DRAFTED: &str = "drafted";
/// The Gates cell's "gates nothing" spelling: the bare em-dash, the
/// same character the board's tables use for an empty cell — the
/// shape a decision takes when the roadmap it would gate is already
/// closed (D19, the first post-M5 decision).
const GATES_NONE: &str = "—";

/// The parsed own decisions.md: the summary table and the dossiers,
/// in board order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnDecisions {
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
    /// The Gates cell's M id, or `None` for the em-dash spelling of
    /// "gates nothing" — the post-roadmap shape (D19).
    pub gates: Option<String>,
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
    /// `resolved [<option>] <date> [— <note>]` — the option letter and
    /// the note are each optional in the measured grammar.
    Resolved {
        /// The parenthesized option letter, e.g. `(a)`, when present.
        option: Option<String>,
        /// The resolution date as written (`YYYY-MM-DD`).
        date: String,
        /// The note after the ` — ` separator, when present.
        note: Option<String>,
    },
    /// `drafted <date> [— <note>]` — the historical open spelling.
    Drafted,
}

/// One decision dossier: `### D<n> — <title>`, with its body running
/// to the next `###` or end of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dossier {
    /// The D id.
    pub id: String,
    /// The dossier title.
    pub title: String,
    /// The raw body to the next `###` or end of input, joined by
    /// `\n` — the Gates findings' context, the camera convention.
    pub body: String,
}

/// Parses the own decisions.md into its dialect AST. Pure function of
/// the text — no I/O, no clock; file reading is the tool's.
pub fn parse_decisions(input: &str) -> Result<OwnDecisions, OwnError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let end_line = numbered.len();
    let mut file = OwnDecisions {
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
    file: &mut OwnDecisions,
    end_line: usize,
) -> Result<(), OwnError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &SUMMARY_HEADER) else {
        return Err(OwnError::MissingRegister {
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
            recommendation_cell,
            confidence_cell,
            gates_cell,
            status_cell,
        ] = row_cells.as_slice()
        else {
            return Err(OwnError::ColumnCount {
                line: number,
                expected: SUMMARY_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_d_id(&id) {
            return Err(OwnError::BadId {
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
            return Err(OwnError::DuplicateId {
                id,
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        if recommendation_cell.is_empty() {
            return Err(OwnError::MalformedLine {
                line: number,
                detail: "the Recommendation cell is empty".to_owned(),
            });
        }
        let confidence = parse_confidence(confidence_cell, number)?;
        let gates = strip_id_noise(gates_cell);
        // The em-dash is the board's "gates nothing" spelling — the
        // shape D19 introduced, a decision recorded after the roadmap
        // it would have gated is closed. Anything else must be an M
        // id: a garbage cell stays a named error, never a silent
        // gates-nothing.
        let gates = if gates == GATES_NONE {
            None
        } else {
            if !is_m_id(&gates) {
                return Err(OwnError::BadId {
                    line: number,
                    id: gates,
                    detail: "not an M id".to_owned(),
                });
            }
            Some(gates)
        };
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
fn parse_confidence(cell: &str, line: usize) -> Result<ConfidenceCell, OwnError> {
    match cell {
        "low" => Ok(ConfidenceCell::Low),
        "medium" => Ok(ConfidenceCell::Medium),
        "high" => Ok(ConfidenceCell::High),
        _ => Err(OwnError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Confidence cell must read 'low', 'medium', or 'high'".to_owned(),
        }),
    }
}

/// The Status cell: `resolved [<option>] <date> [— <note>]` or
/// `drafted <date> [— <note>]`. The option is a parenthesized
/// space-free token; the date is shape-checked; the note is the text
/// after the ` — ` separator. A non-empty tail that is not the
/// separator's note is a named error — the measured grammar carries
/// either nothing or ` — <note>` after the date.
fn parse_status_cell(cell: &str, line: usize) -> Result<DecisionStatusCell, OwnError> {
    if let Some(rest) = cell.strip_prefix(STATUS_RESOLVED) {
        let (option, rest) = split_option(rest, cell, line)?;
        let (date, note) = split_date_note(rest, cell, line)?;
        return Ok(DecisionStatusCell::Resolved {
            option,
            date: date.to_owned(),
            note,
        });
    }
    if let Some(rest) = cell.strip_prefix(STATUS_DRAFTED) {
        // The drafted spelling carries no option — the measured
        // historical form is `drafted <date> [— <note>]`. The date and
        // note are validated and discarded: a drafted decision maps to
        // `open` at the import stamp, and the board keeps the prose.
        let rest = require_space(rest, cell, line)?;
        split_date_note(rest, cell, line)?;
        return Ok(DecisionStatusCell::Drafted);
    }
    Err(OwnError::BadStatus {
        line,
        text: cell.to_owned(),
        detail: "the Status cell must read 'resolved [<option>] <date> [— <note>]' or 'drafted <date> [— <note>]'"
            .to_owned(),
    })
}

/// The space that must follow a status prefix before the option or
/// the date.
fn require_space<'a>(rest: &'a str, cell: &str, line: usize) -> Result<&'a str, OwnError> {
    rest.strip_prefix(' ').ok_or_else(|| OwnError::BadStatus {
        line,
        text: cell.to_owned(),
        detail: "the status spelling lacks its date".to_owned(),
    })
}

/// The optional ` (<option>)` after the resolved prefix: a space, a
/// parenthesized space-free token, a closing parenthesis, a space.
/// Returns the token (without parentheses) and the rest.
fn split_option<'a>(
    rest: &'a str,
    cell: &str,
    line: usize,
) -> Result<(Option<String>, &'a str), OwnError> {
    let after_space = require_space(rest, cell, line)?;
    if let Some(after_open) = after_space.strip_prefix('(') {
        let Some((token, after_close)) = after_open.split_once(')') else {
            return Err(OwnError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the status option's parenthesis does not close".to_owned(),
            });
        };
        if token.is_empty() || token.contains(' ') {
            return Err(OwnError::BadStatus {
                line,
                text: cell.to_owned(),
                detail: "the status option must be one space-free token, e.g. '(a)'".to_owned(),
            });
        }
        let rest = require_space(after_close, cell, line)?;
        Ok((Some(token.to_owned()), rest))
    } else {
        Ok((None, after_space))
    }
}

/// The `<date> [— <note>]` tail of a status spelling: ten
/// `YYYY-MM-DD`-shaped characters, then nothing or the separator and
/// the note text. A non-empty tail that is not the separator is a
/// named error.
fn split_date_note<'a>(
    rest: &'a str,
    cell: &str,
    line: usize,
) -> Result<(&'a str, Option<String>), OwnError> {
    let Some(date) = rest.get(..10) else {
        return Err(OwnError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the status spelling's date is missing".to_owned(),
        });
    };
    if !is_day_date(date) {
        return Err(OwnError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the status spelling's date is not YYYY-MM-DD".to_owned(),
        });
    }
    let note = match rest.get(10..) {
        None | Some("") => None,
        Some(tail) => Some(
            tail.strip_prefix(EM_DASH_SEP)
                .ok_or_else(|| OwnError::BadStatus {
                    line,
                    text: cell.to_owned(),
                    detail:
                        "the status spelling carries text after the date that is not a ' — ' note"
                            .to_owned(),
                })?
                .to_owned(),
        ),
    };
    Ok((date, note))
}

/// The dossiers: every `###` line must parse as a dossier header;
/// bodies run to the next `###` or end of input.
fn parse_dossiers(lines: &[(&str, usize)], file: &mut OwnDecisions) -> Result<(), OwnError> {
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
    body: Vec<String>,
}

/// Parses one dossier header line: `### D<n> — <title>`. This board's
/// headers carry no resolved marker — the banner lives in the body
/// prose — so a header beyond the plain shape is a named error.
fn parse_dossier_header(text: &str, line: usize) -> Result<PartialDossier, OwnError> {
    let rest = text.strip_prefix("### ").unwrap_or(text);
    let Some((id, after_id)) = rest.split_once(EM_DASH_SEP) else {
        return Err(OwnError::MalformedLine {
            line,
            detail: "a dossier header has no ' — ' separator after the id".to_owned(),
        });
    };
    let id = strip_id_noise(id);
    if !is_d_id(&id) {
        return Err(OwnError::BadId {
            line,
            id,
            detail: "not a D id".to_owned(),
        });
    }
    Ok(PartialDossier {
        id,
        title: after_id.to_owned(),
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
