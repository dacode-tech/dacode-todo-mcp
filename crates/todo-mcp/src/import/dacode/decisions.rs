//! The B appendix parser (t42): the sign-off summary table and the
//! decision dossiers.
//!
//! Every `###` line in the file must parse as a dossier header —
//! walking an unknown `###` past would silently skip a malformed
//! register line. Dossier bodies run to the next `###` or end of
//! input; a `##` section inside a body stays body text.
//!
//! The summary table is required (its absence would silently omit the
//! whole register), and its rows are carried in board order — the
//! order is the data, not the id sequence.
//!
//! The fix round's widening: a Gates-cell doc entry is a two-digit
//! number or a single uppercase letter A–G (an appendix doc); anything
//! else stays a named error.
//!
//! The dossier status grammar: after locating the first
//! ` — ✅ **` or ` — ⚠️ **`, the marker must read `RESOLVED <date>` or
//! `SIGNED OFF <date>`, optionally followed by
//! `, APPLIED <date> via A<n>`; the remainder is the raw outcome with
//! one leading `:`/`—` separator stripped when present (the decorated
//! `SIGNED OFF <date> as a direction, amended — …` form has none and
//! is carried whole). Dates
//! are validated as `YYYY-MM-DD`.
//!
//! Both registers' ids are unique: a duplicate B id within the
//! summary table or within the dossiers is a named error — the
//! cross-file set checks in [`super::parse_board`] collapse
//! duplicates and cannot see them.

use std::collections::BTreeMap;

use super::ast::{
    AppliedVia, BDossier, BDossierKind, BDossierStatus, BSummaryRow, DecisionsAppendix, GatesCell,
};
use super::error::ImportError;
use super::table::{
    ARROW, EM_DASH, is_a_id, is_b_id, is_m_id, is_register_construct, split_at_dossier_marker,
};
use crate::import::helpers::{
    cells, collect_rows, find_table_header, is_day_date, split_lines, strip_id_noise, strip_strikes,
};

/// The B summary table's header.
const SUMMARY_HEADER: [&str; 5] = ["#", "Decision", "Recommendation", "Confidence", "Gates"];

/// Parses the B appendix into its dialect AST. Pure function of the
/// text — no I/O; file reading is the tool's (t44).
pub fn parse_b_decisions(input: &str) -> Result<DecisionsAppendix, ImportError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let end_line = numbered.len();
    let mut appendix = DecisionsAppendix {
        summary: Vec::new(),
        dossiers: Vec::new(),
    };
    parse_summary_table(&numbered, &mut appendix, end_line)?;
    parse_dossiers(&numbered, &mut appendix)?;
    Ok(appendix)
}

/// The summary table: rows carry the id, the Decision cell raw, and
/// the parsed Gates cell. The Recommendation and Confidence cells are
/// not carried (they duplicate the dossier).
fn parse_summary_table(
    lines: &[(&str, usize)],
    appendix: &mut DecisionsAppendix,
    end_line: usize,
) -> Result<(), ImportError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &SUMMARY_HEADER) else {
        return Err(ImportError::MissingRegister {
            register: "the B summary table".to_owned(),
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
            _recommendation,
            _confidence,
            gates_cell,
        ] = row_cells.as_slice()
        else {
            return Err(ImportError::ColumnCount {
                line: number,
                expected: SUMMARY_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_b_id(&id) {
            return Err(ImportError::BadId {
                line: number,
                id,
                detail: "not a B id".to_owned(),
            });
        }
        if let Some(first) = seen.get(&id) {
            return Err(ImportError::DuplicateId {
                id: id.clone(),
                first: *first,
                second: number,
            });
        }
        seen.insert(id.clone(), number);
        let gates = parse_gates_cell(gates_cell, number)?;
        appendix.summary.push(BSummaryRow {
            id,
            title: title_cell.clone(),
            gates,
        });
    }
    Ok(())
}

/// The Gates cell: strip strikes and stray `✅`, split on `→` — the
/// left half's doc entries, the right half the milestone `M<n>` when
/// present. A doc entry is a two-digit number or a single uppercase
/// letter A–G (an appendix doc); anything else is a named error.
fn parse_gates_cell(cell: &str, line: usize) -> Result<GatesCell, ImportError> {
    let stripped = strip_strikes(cell).replace('✅', "");
    let (docs_text, milestone) = match stripped.split_once(ARROW) {
        Some((left, right)) => {
            let milestone = right.trim();
            if !is_m_id(milestone) {
                return Err(ImportError::BadId {
                    line,
                    id: milestone.to_owned(),
                    detail: "not an M id".to_owned(),
                });
            }
            (left, Some(milestone.to_owned()))
        }
        None => (stripped.as_str(), None),
    };
    let mut docs = Vec::new();
    for raw in docs_text.split(',') {
        let doc = raw.trim();
        if doc.is_empty() {
            continue;
        }
        if !is_doc_entry(doc) {
            return Err(ImportError::BadId {
                line,
                id: doc.to_owned(),
                detail: "not a two-digit doc number or an A–G appendix letter".to_owned(),
            });
        }
        docs.push(doc.to_owned());
    }
    Ok(GatesCell { docs, milestone })
}

/// A Gates-cell doc entry: a two-digit number or a single uppercase
/// letter A–G.
fn is_doc_entry(token: &str) -> bool {
    if token.len() == 2 {
        return token.bytes().all(|b| b.is_ascii_digit());
    }
    token.len() == 1 && matches!(token, "A" | "B" | "C" | "D" | "E" | "F" | "G")
}

/// The dossiers: every `###` line must parse as
/// `### <seq>. [B<id>] <title> — ✅/⚠️ **<status>…**`; bodies run to
/// the next `###` or end of input.
fn parse_dossiers(
    lines: &[(&str, usize)],
    appendix: &mut DecisionsAppendix,
) -> Result<(), ImportError> {
    let mut current: Option<PartialDossier> = None;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for &(text, number) in lines {
        if text.starts_with("### ") {
            if let Some(dossier) = current.take() {
                appendix.dossiers.push(finish_dossier(dossier));
            }
            let dossier = parse_dossier_header(text, number)?;
            if let Some(first) = seen.get(&dossier.id) {
                return Err(ImportError::DuplicateId {
                    id: dossier.id.clone(),
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
        appendix.dossiers.push(finish_dossier(dossier));
    }
    Ok(())
}

/// A dossier under construction: its body accumulates raw lines.
struct PartialDossier {
    seq: u32,
    id: String,
    title: String,
    status: BDossierStatus,
    outcome: String,
    body: Vec<String>,
}

/// Parses one dossier header line.
fn parse_dossier_header(text: &str, line: usize) -> Result<PartialDossier, ImportError> {
    let rest = text.strip_prefix("### ").unwrap_or(text);
    let Some((seq_text, after_seq)) = rest.split_once(". ") else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "a B dossier header has no '<seq>. ' prefix".to_owned(),
        });
    };
    let seq: u32 = seq_text.parse().map_err(|_| ImportError::MalformedLine {
        line,
        detail: "a B dossier header's sequence number is not a number".to_owned(),
    })?;
    let Some(after_bracket) = after_seq.strip_prefix('[') else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "a B dossier header has no '[B<id>]' bracket".to_owned(),
        });
    };
    let Some((id, after_id)) = after_bracket.split_once(']') else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "a B dossier header's id bracket does not close".to_owned(),
        });
    };
    if !is_b_id(id) {
        return Err(ImportError::BadId {
            line,
            id: id.to_owned(),
            detail: "not a B id".to_owned(),
        });
    }
    let Some(title_and_marker) = after_id.strip_prefix(' ') else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "a B dossier header has no title after the id bracket".to_owned(),
        });
    };
    let Some((title, marker_text)) = split_at_dossier_marker(title_and_marker) else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "a B dossier header has no ' — ✅ **' or ' — ⚠️ **' status marker".to_owned(),
        });
    };
    let (status, outcome) = parse_dossier_status(marker_text, line)?;
    Ok(PartialDossier {
        seq,
        id: id.to_owned(),
        title: title.to_owned(),
        status,
        outcome,
        body: Vec::new(),
    })
}

/// Parses the status marker text that follows ` — ✅ **` or
/// ` — ⚠️ **`: the kind, the date, the optional APPLIED clause, and
/// the raw outcome (the remainder, one leading separator stripped,
/// the closing `**` stripped).
fn parse_dossier_status(text: &str, line: usize) -> Result<(BDossierStatus, String), ImportError> {
    let (kind, after_kind) = if let Some(rest) = text.strip_prefix("RESOLVED ") {
        (BDossierKind::Resolved, rest)
    } else if let Some(rest) = text.strip_prefix("SIGNED OFF ") {
        (BDossierKind::SignedOff, rest)
    } else {
        return Err(ImportError::BadStatus {
            line,
            text: text.trim().to_owned(),
            detail: "the marker must read 'RESOLVED <date>' or 'SIGNED OFF <date>'".to_owned(),
        });
    };
    let Some(date) = after_kind.get(..10) else {
        return Err(ImportError::BadStatus {
            line,
            text: text.trim().to_owned(),
            detail: "the status date is missing".to_owned(),
        });
    };
    if !is_day_date(date) {
        return Err(ImportError::BadStatus {
            line,
            text: text.trim().to_owned(),
            detail: "the status date is not YYYY-MM-DD".to_owned(),
        });
    }
    let Some(mut rest) = after_kind.get(10..) else {
        return Err(ImportError::BadStatus {
            line,
            text: text.trim().to_owned(),
            detail: "the status date is missing".to_owned(),
        });
    };
    let mut applied = None;
    if let Some(after_applied) = rest.strip_prefix(", APPLIED ") {
        let Some(applied_date) = after_applied.get(..10) else {
            return Err(ImportError::BadStatus {
                line,
                text: text.trim().to_owned(),
                detail: "the APPLIED clause's date is missing".to_owned(),
            });
        };
        if !is_day_date(applied_date) {
            return Err(ImportError::BadStatus {
                line,
                text: text.trim().to_owned(),
                detail: "the APPLIED clause's date is not YYYY-MM-DD".to_owned(),
            });
        }
        let Some(after_date) = after_applied.get(10..) else {
            return Err(ImportError::BadStatus {
                line,
                text: text.trim().to_owned(),
                detail: "the APPLIED clause's date is missing".to_owned(),
            });
        };
        let Some(after_via) = after_date.strip_prefix(" via ") else {
            return Err(ImportError::BadStatus {
                line,
                text: text.trim().to_owned(),
                detail: "the APPLIED clause lacks ' via A<n>'".to_owned(),
            });
        };
        let via: String = after_via
            .chars()
            .take_while(|c| c.is_alphanumeric())
            .collect();
        if !is_a_id(&via) {
            return Err(ImportError::BadStatus {
                line,
                text: text.trim().to_owned(),
                detail: "the APPLIED clause's credit is not an A id".to_owned(),
            });
        }
        rest = after_via.strip_prefix(via.as_str()).unwrap_or("");
        applied = Some(AppliedVia {
            date: applied_date.to_owned(),
            via,
        });
    }
    let outcome = strip_outcome(rest);
    Ok((
        BDossierStatus {
            kind,
            date: date.to_owned(),
            applied,
        },
        outcome,
    ))
}

/// The raw outcome from the marker's remainder: the closing `**`
/// stripped, then one leading `:`/`—` separator stripped when present,
/// then trimmed.
fn strip_outcome(rest: &str) -> String {
    let mut outcome = rest.strip_suffix("**").unwrap_or(rest).trim();
    if let Some(stripped) = outcome.strip_prefix(':') {
        outcome = stripped.trim();
    } else if let Some(stripped) = outcome.strip_prefix(EM_DASH) {
        outcome = stripped.trim();
    }
    outcome.to_owned()
}

/// Finishes a dossier: joins its body into the raw text.
fn finish_dossier(dossier: PartialDossier) -> BDossier {
    BDossier {
        seq: dossier.seq,
        id: dossier.id,
        title: dossier.title,
        status: dossier.status,
        outcome: dossier.outcome,
        body: dossier.body.join("\n"),
    }
}
