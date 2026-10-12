//! The ikvm-html decisions.md parser (t47): the sign-off summary
//! table, the detail dossiers and the resolved footer — plus the
//! file's half of the dialect AST.
//!
//! The file is three registers. The **summary table** is recognized
//! by its header row (`ID | Decision | Recommendation | Confidence |
//! Gates | Status`) wherever it appears, and required — its absence
//! would silently omit every decision. A duplicated D id is a named
//! error naming both lines: the composition's id cross-check compares
//! sets (which dedupe), so a silent duplicate would pass it and shift
//! the mapping's decision id range past the board's numbering. Each
//! row's cells are
//! validated as the mapping's inputs: the id (strikes and bold
//! stripped, the strike itself recorded — the resolved rows carry
//! struck ids), a non-empty Recommendation, a Confidence spelling
//! (five, strict), an M-shaped Gates cell, and a Status cell reading
//! exactly `resolved` or `open`.
//!
//! The **dossiers**: every `###` line in the file must parse as
//! `### D<n> — <title>` — walking an unknown `###` past would
//! silently skip a malformed register line. Each dossier carries a
//! `**Gates** M<n> · **Action** …` line (required — the cross-check
//! against the summary's Gates cell needs it) and, when the decision
//! is resolved, a banner: a blockquote line whose text reads
//! `**Resolved <date> — <outcome>**`, wrapping across the blockquote's
//! lines. The outcome ends at the bold run's closing `**`, whatever
//! follows it — the closings measured on the live board are `.**`,
//! `**,` and `** (` (the live probe's strictness find, widened as
//! measured grammar and pinned by test); one trailing `.` is trimmed,
//! the title discipline's own. The banner is
//! the date/outcome source (the mapping's convention); the body runs
//! to the next `###` or end of input and is the Gates-finding
//! context.
//!
//! The **footer** is recognized by its header row
//! (`ID | Resolved | Outcome | Applied to`): one row per resolved
//! decision, the date cross-checked against the banner's. The prose
//! above it is walked past — prose, not a register; the table is the
//! truth. The Applied-to column is carried raw and dropped by the
//! map.

use std::collections::BTreeMap;

use super::board::{BOLD, is_m_id, is_register_construct};
use super::error::IkvmError;
use crate::import::helpers::{
    cells, collect_rows, find_table_header, is_day_date, split_lines, strip_id_noise,
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
/// The resolved footer's header.
const FOOTER_HEADER: [&str; 4] = ["ID", "Resolved", "Outcome", "Applied to"];
/// The dossier header's id/title separator (em-dash), spaced.
const EM_DASH_SEP: &str = " — ";
/// The dossier's Gates line prefix.
const GATES_LINE_PREFIX: &str = "**Gates** ";
/// The resolution banner's opening, as the blockquote text reads it.
const BANNER_PREFIX: &str = "**Resolved ";
/// The resolution banner's date/outcome separator, spaced em-dash.
const BANNER_SEP: &str = " — ";
/// The blockquote marker, spaced.
const QUOTE_PREFIX: &str = "> ";

/// The parsed decisions.md: the summary table, the dossiers and the
/// footer, in board order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IkvmDecisions {
    /// The summary table's rows, in board order.
    pub summary: Vec<DecisionRow>,
    /// The dossiers, in board order.
    pub dossiers: Vec<Dossier>,
    /// The resolved footer's rows, in board order.
    pub footer: Vec<FooterRow>,
}

/// One summary-table row: the mapping's inputs for one decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRow {
    /// The D id with strikes and bold stripped.
    pub id: String,
    /// Whether the id cell carried a strike — the resolved-set
    /// marker the composition's three-way cross-check reads.
    pub struck: bool,
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
/// vocabulary (wider than the siblings' three-value cell), mapped to
/// the domain's enum by the mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceCell {
    /// `low`.
    Low,
    /// `medium`.
    Medium,
    /// `high`.
    High,
    /// `low-medium`.
    LowMedium,
    /// `medium-high`.
    MediumHigh,
}

/// The Status cell's verdict, parsed: exactly `resolved` or `open`.
/// The resolution's date and outcome live in the dossier's banner,
/// not the cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionStatusCell {
    /// `resolved`.
    Resolved,
    /// `open`.
    Open,
}

/// One decision dossier: `### D<n> — <title>`, its Gates line, its
/// resolution banner when present, and its raw body to the next
/// `###` or end of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dossier {
    /// The D id.
    pub id: String,
    /// The dossier title.
    pub title: String,
    /// The `**Gates** M<n>` line's M id.
    pub gates: String,
    /// The resolution banner, when the dossier carries one.
    pub resolved: Option<ResolvedBanner>,
    /// The raw body to the next `###` or end of input, joined by
    /// `\n`.
    pub body: String,
}

/// A resolution banner: the date and the outcome, as the banner
/// reads them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBanner {
    /// The resolution date as written (`YYYY-MM-DD`).
    pub date: String,
    /// The outcome text, raw.
    pub outcome: String,
}

/// One resolved-footer row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterRow {
    /// The D id with bold stripped.
    pub id: String,
    /// The resolution date as written (`YYYY-MM-DD`).
    pub date: String,
    /// The outcome text, raw.
    pub outcome: String,
    /// The Applied-to cell, raw — carried for the AST's completeness
    /// and dropped by the map.
    pub applied_to: String,
}

/// Parses decisions.md into its dialect AST. Pure function of the
/// text — no I/O, no clock; file reading is the tool's.
pub fn parse_decisions(input: &str) -> Result<IkvmDecisions, IkvmError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let end_line = numbered.len();
    let mut file = IkvmDecisions {
        summary: Vec::new(),
        dossiers: Vec::new(),
        footer: Vec::new(),
    };
    parse_summary_table(&numbered, &mut file, end_line)?;
    parse_dossiers(&numbered, &mut file)?;
    parse_footer_table(&numbered, &mut file, end_line)?;
    Ok(file)
}

/// The summary table: the register, recognized by its header row.
fn parse_summary_table(
    lines: &[(&str, usize)],
    file: &mut IkvmDecisions,
    end_line: usize,
) -> Result<(), IkvmError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &SUMMARY_HEADER) else {
        return Err(IkvmError::MissingRegister {
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
            return Err(IkvmError::ColumnCount {
                line: number,
                expected: SUMMARY_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let struck = id_cell.contains("~~");
        let id = strip_id_noise(id_cell);
        if !is_d_id(&id) {
            return Err(IkvmError::BadId {
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
            return Err(IkvmError::DuplicateId {
                id,
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        if recommendation_cell.is_empty() {
            return Err(IkvmError::MalformedLine {
                line: number,
                detail: "the Recommendation cell is empty".to_owned(),
            });
        }
        let confidence = parse_confidence(confidence_cell, number)?;
        let gates = strip_id_noise(gates_cell);
        if !is_m_id(&gates) {
            return Err(IkvmError::BadId {
                line: number,
                id: gates,
                detail: "not an M id".to_owned(),
            });
        }
        let status = parse_status_cell(status_cell, number)?;
        file.summary.push(DecisionRow {
            id,
            struck,
            title: title_cell.clone(),
            recommendation: recommendation_cell.clone(),
            confidence,
            gates,
            status,
        });
    }
    Ok(())
}

/// The Confidence cell: the five spellings, case-sensitive — any
/// other spelling is a named error.
fn parse_confidence(cell: &str, line: usize) -> Result<ConfidenceCell, IkvmError> {
    match cell {
        "low" => Ok(ConfidenceCell::Low),
        "medium" => Ok(ConfidenceCell::Medium),
        "high" => Ok(ConfidenceCell::High),
        "low-medium" => Ok(ConfidenceCell::LowMedium),
        "medium-high" => Ok(ConfidenceCell::MediumHigh),
        _ => Err(IkvmError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Confidence cell must read 'low', 'medium', 'high', 'low-medium' or 'medium-high'"
                .to_owned(),
        }),
    }
}

/// The Status cell: exactly `resolved` or `open`.
fn parse_status_cell(cell: &str, line: usize) -> Result<DecisionStatusCell, IkvmError> {
    match cell {
        "resolved" => Ok(DecisionStatusCell::Resolved),
        "open" => Ok(DecisionStatusCell::Open),
        _ => Err(IkvmError::BadStatus {
            line,
            text: cell.to_owned(),
            detail: "the Status cell must read 'resolved' or 'open'".to_owned(),
        }),
    }
}

/// The dossiers: every `###` line must parse as a dossier header;
/// bodies run to the next `###` or end of input, collecting the
/// Gates line and the banner along the way.
fn parse_dossiers(lines: &[(&str, usize)], file: &mut IkvmDecisions) -> Result<(), IkvmError> {
    let mut current: Option<PartialDossier> = None;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for &(text, number) in lines {
        if text.starts_with("### ") {
            if let Some(dossier) = current.take() {
                file.dossiers.push(finish_dossier(dossier)?);
            }
            let dossier = parse_dossier_header(text, number)?;
            // A duplicated D id is a named error naming both lines: the
            // composition's id cross-check compares sets (which
            // dedupe) and the map's resolution lookup reads the first
            // dossier per id, so a silent duplicate would pass and the
            // second dossier's banner would be ignored.
            if let Some(first) = seen.get(&dossier.id) {
                return Err(IkvmError::DuplicateId {
                    id: dossier.id,
                    first: *first,
                    second: number,
                });
            }
            seen.insert(dossier.id.clone(), number);
            current = Some(dossier);
        } else if let Some(dossier) = current.as_mut() {
            dossier.body.push((text.to_owned(), number));
        }
    }
    if let Some(dossier) = current.take() {
        file.dossiers.push(finish_dossier(dossier)?);
    }
    Ok(())
}

/// A dossier under construction: its body accumulates the raw lines
/// with their 1-based numbers (the Gates line and the banner are
/// extracted at finish, and their errors need the lines).
struct PartialDossier {
    id: String,
    title: String,
    header_line: usize,
    body: Vec<(String, usize)>,
}

/// Parses one dossier header line: `### D<n> — <title>`.
fn parse_dossier_header(text: &str, line: usize) -> Result<PartialDossier, IkvmError> {
    let rest = text.strip_prefix("### ").unwrap_or(text);
    let Some((id, title)) = rest.split_once(EM_DASH_SEP) else {
        return Err(IkvmError::MalformedLine {
            line,
            detail: "a dossier header has no ' — ' separator after the id".to_owned(),
        });
    };
    let id = strip_id_noise(id);
    if !is_d_id(&id) {
        return Err(IkvmError::BadId {
            line,
            id,
            detail: "not a D id".to_owned(),
        });
    }
    Ok(PartialDossier {
        id,
        title: title.to_owned(),
        header_line: line,
        body: Vec::new(),
    })
}

/// Finishes a dossier: joins its body into the raw text, extracts
/// the Gates line (required) and the resolution banner (when
/// present).
fn finish_dossier(dossier: PartialDossier) -> Result<Dossier, IkvmError> {
    let gates = extract_gates(&dossier)?;
    let resolved = extract_banner(&dossier)?;
    Ok(Dossier {
        id: dossier.id,
        title: dossier.title,
        gates,
        resolved,
        body: dossier
            .body
            .iter()
            .map(|(text, _)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

/// The dossier's Gates line: the first line starting `**Gates** `,
/// its M id the token up to the first whitespace. Required — the
/// composition's Gates cross-check reads it, so a dossier without
/// one is a named error, never a silent skip.
fn extract_gates(dossier: &PartialDossier) -> Result<String, IkvmError> {
    for (text, number) in &dossier.body {
        if let Some(after) = text.strip_prefix(GATES_LINE_PREFIX) {
            let id: String = after.chars().take_while(|c| !c.is_whitespace()).collect();
            if !is_m_id(&id) {
                return Err(IkvmError::BadId {
                    line: *number,
                    id,
                    detail: "the Gates line's id is not an M id".to_owned(),
                });
            }
            return Ok(id);
        }
    }
    Err(IkvmError::MalformedLine {
        line: dossier.header_line,
        detail: format!(
            "the dossier {} carries no '**Gates** M<n>' line",
            dossier.id
        ),
    })
}

/// The dossier's resolution banner: the first body line whose
/// blockquote text starts `**Resolved `, joined with the blockquote
/// lines that follow it, the outcome ending at the bold run's
/// closing `**` (the measured closings: `.**`, `**,` and `** (`).
/// `None` when the dossier carries no banner.
fn extract_banner(dossier: &PartialDossier) -> Result<Option<ResolvedBanner>, IkvmError> {
    let mut banner: Option<(&str, usize, usize)> = None;
    for (idx, (text, number)) in dossier.body.iter().enumerate() {
        if text.starts_with('>') && blockquote_text(text).starts_with(BANNER_PREFIX) {
            banner = Some((text.as_str(), *number, idx));
            break;
        }
    }
    let Some((banner_line, banner_number, start)) = banner else {
        return Ok(None);
    };
    let mut joined = String::from(blockquote_text(banner_line));
    for (text, _) in dossier.body.iter().skip(start + 1) {
        if !text.starts_with('>') {
            break;
        }
        joined.push(' ');
        joined.push_str(blockquote_text(text));
    }
    let Some(after_prefix) = joined.strip_prefix(BANNER_PREFIX) else {
        return Err(IkvmError::BadStatus {
            line: banner_number,
            text: joined.clone(),
            detail: "the resolution banner does not read '**Resolved <date> — <outcome>**'"
                .to_owned(),
        });
    };
    let Some((content, _tail)) = after_prefix.split_once(BOLD) else {
        return Err(IkvmError::BadStatus {
            line: banner_number,
            text: joined.clone(),
            detail: "the resolution banner's bold run does not close".to_owned(),
        });
    };
    let Some(date) = content.get(..10) else {
        return Err(IkvmError::BadStatus {
            line: banner_number,
            text: joined.clone(),
            detail: "the resolution banner's date is missing".to_owned(),
        });
    };
    if !is_day_date(date) {
        return Err(IkvmError::BadStatus {
            line: banner_number,
            text: joined.clone(),
            detail: "the resolution banner's date is not YYYY-MM-DD".to_owned(),
        });
    }
    let Some(outcome) = content
        .get(10..)
        .and_then(|rest| rest.strip_prefix(BANNER_SEP))
    else {
        return Err(IkvmError::BadStatus {
            line: banner_number,
            text: joined.clone(),
            detail: "the resolution banner lacks ' — ' after the date".to_owned(),
        });
    };
    let outcome = outcome.strip_suffix('.').unwrap_or(outcome);
    Ok(Some(ResolvedBanner {
        date: date.to_owned(),
        outcome: outcome.to_owned(),
    }))
}

/// A blockquote line's text: the `> ` (or bare `>`) prefix removed.
fn blockquote_text(line: &str) -> &str {
    line.strip_prefix(QUOTE_PREFIX)
        .unwrap_or(line.strip_prefix('>').unwrap_or(line))
}

/// The footer table: the resolved index, recognized by its header
/// row. Required — its absence would silently omit the cross-check's
/// second side.
fn parse_footer_table(
    lines: &[(&str, usize)],
    file: &mut IkvmDecisions,
    end_line: usize,
) -> Result<(), IkvmError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &FOOTER_HEADER) else {
        return Err(IkvmError::MissingRegister {
            register: "the resolved footer".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx, is_register_construct);
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [id_cell, date_cell, outcome_cell, applied_cell] = row_cells.as_slice() else {
            return Err(IkvmError::ColumnCount {
                line: number,
                expected: FOOTER_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_d_id(&id) {
            return Err(IkvmError::BadId {
                line: number,
                id,
                detail: "not a D id".to_owned(),
            });
        }
        // A duplicated D id is a named error naming both lines: the
        // composition's cross-checks compare sets (which dedupe) and
        // the banner-date check reads only the first row per id, so a
        // duplicate row passes silently — the footer's
        // one-row-per-resolved-decision invariant goes unenforced and
        // a divergent duplicate date escapes the pin.
        if let Some(first) = seen.get(&id) {
            return Err(IkvmError::DuplicateId {
                id,
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        if !is_day_date(date_cell) {
            return Err(IkvmError::BadStatus {
                line: number,
                text: date_cell.clone(),
                detail: "the footer's Resolved date is not YYYY-MM-DD".to_owned(),
            });
        }
        file.footer.push(FooterRow {
            id,
            date: date_cell.clone(),
            outcome: outcome_cell.clone(),
            applied_to: applied_cell.clone(),
        });
    }
    Ok(())
}

/// A D id: `D<digits>` exactly — the registers' measured shape. A
/// letter-suffixed id is a named error, not an oddball (the board's
/// D ids are plain, measured).
fn is_d_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('D') else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}
