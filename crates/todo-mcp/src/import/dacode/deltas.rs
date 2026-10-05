//! The A appendix parser (t42): the design-deltas index table and the
//! dossiers.
//!
//! Every `###` line in the file must parse as an A dossier header —
//! walking an unknown `###` past would silently skip a malformed
//! register line. Dossier bodies run to the next `###` or end of
//! input, so a `##` section sitting between two dossiers lands in the
//! earlier dossier's body as raw text.
//!
//! The index table is required (its absence would silently omit the
//! whole register). The index's rows are blank-line tolerant.
//!
//! The file's second table — `| # | Section | Applied | Date | Note |`,
//! the "Deltas applied" register — is a known header whose rows are
//! carried (id, sections split as the index's, the Applied, Date and
//! Note cells raw); it is optional, and its rows also ride the
//! enclosing dossier's raw body. The status marker locator accepts
//! the arrow intros ` → ✅ **` and ` → ⚠️ **` beside the em-dash
//! forms, and the status grammar accepts the via-form
//! `APPLIED via A<n>, <date>`, carrying the credited id.
//!
//! The dossier status grammar: after locating the first marker intro,
//! the marker must read `APPLIED <date>`, `SIGNED OFF <date>` or
//! `HALF-APPLIED <date>`; the pre-marker span is carried raw, and the
//! remainder after the date is carried as the raw outcome — defensive
//! symmetry with the B dossier's grammar, which does carry post-date
//! prose.
//! Dates are validated as `YYYY-MM-DD`.

use super::ast::{AAppliedRow, ADossier, ADossierKind, ADossierStatus, AIndexRow, DeltasAppendix};
use super::error::ImportError;
use super::table::{
    cells, collect_rows, find_table_header, id_tokens, is_a_id, is_day_date,
    split_at_dossier_marker, split_lines, strip_id_noise, strip_strikes,
};

/// The A index table's header.
const INDEX_HEADER: [&str; 5] = ["#", "Section", "Delta", "Depends on", "Recommend"];
/// The A applied table's header (the "Deltas applied" register).
const APPLIED_HEADER: [&str; 5] = ["#", "Section", "Applied", "Date", "Note"];

/// Parses the A appendix into its dialect AST. Pure function of the
/// text — no I/O; file reading is the tool's (t44).
pub fn parse_a_deltas(input: &str) -> Result<DeltasAppendix, ImportError> {
    let lines = split_lines(input);
    let numbered: Vec<(&str, usize)> = lines.iter().enumerate().map(|(i, t)| (*t, i + 1)).collect();
    let end_line = numbered.len();
    let mut appendix = DeltasAppendix {
        index: Vec::new(),
        applied: Vec::new(),
        dossiers: Vec::new(),
    };
    parse_index_table(&numbered, &mut appendix, end_line)?;
    parse_applied_table(&numbered, &mut appendix)?;
    parse_dossiers(&numbered, &mut appendix)?;
    Ok(appendix)
}

/// The index table: rows carry the id, the Section cell split by `,`,
/// the Delta cell raw, and the Depends-on cell's id-shaped tokens
/// (strikes are noise — the dependency exists regardless). The
/// Recommend cell is not carried (it duplicates the dossier status
/// plus prose).
fn parse_index_table(
    lines: &[(&str, usize)],
    appendix: &mut DeltasAppendix,
    end_line: usize,
) -> Result<(), ImportError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &INDEX_HEADER) else {
        return Err(ImportError::MissingRegister {
            register: "the A index table".to_owned(),
            line: end_line,
        });
    };
    let (rows, _) = collect_rows(lines, header_idx);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [id_cell, section_cell, delta_cell, depends_cell, _recommend] = row_cells.as_slice()
        else {
            return Err(ImportError::ColumnCount {
                line: number,
                expected: INDEX_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_a_id(&id) {
            return Err(ImportError::BadId {
                line: number,
                id,
                detail: "not an A id".to_owned(),
            });
        }
        let sections: Vec<String> = section_cell
            .split(',')
            .map(|section| section.trim().to_owned())
            .collect();
        let depends_on = id_tokens(&strip_strikes(depends_cell));
        appendix.index.push(AIndexRow {
            id,
            sections,
            delta: delta_cell.clone(),
            depends_on,
        });
    }
    Ok(())
}

/// The applied table: carried when its header is present; its rows
/// must parse — five
/// cells and an A id — else a named error. The Applied, Date and Note
/// cells are raw.
fn parse_applied_table(
    lines: &[(&str, usize)],
    appendix: &mut DeltasAppendix,
) -> Result<(), ImportError> {
    let Some((header_idx, _, _)) = find_table_header(lines, &APPLIED_HEADER) else {
        return Ok(());
    };
    let (rows, _) = collect_rows(lines, header_idx);
    for (text, number) in rows {
        let row_cells = cells(&text);
        let [id_cell, section_cell, applied_cell, date_cell, note_cell] = row_cells.as_slice()
        else {
            return Err(ImportError::ColumnCount {
                line: number,
                expected: APPLIED_HEADER.len(),
                found: row_cells.len(),
            });
        };
        let id = strip_id_noise(id_cell);
        if !is_a_id(&id) {
            return Err(ImportError::BadId {
                line: number,
                id,
                detail: "not an A id".to_owned(),
            });
        }
        appendix.applied.push(AAppliedRow {
            id,
            sections: section_cell
                .split(',')
                .map(|section| section.trim().to_owned())
                .collect(),
            applied: applied_cell.clone(),
            date: date_cell.clone(),
            note: note_cell.clone(),
        });
    }
    Ok(())
}

/// The dossiers: every `###` line must parse as
/// `### A<id> — <span> — ✅/⚠️ **<status>…**`; bodies run to the next
/// `###` or end of input.
fn parse_dossiers(
    lines: &[(&str, usize)],
    appendix: &mut DeltasAppendix,
) -> Result<(), ImportError> {
    let mut current: Option<PartialDossier> = None;
    for &(text, number) in lines {
        if text.starts_with("### ") {
            if let Some(dossier) = current.take() {
                appendix.dossiers.push(finish_dossier(dossier));
            }
            current = Some(parse_dossier_header(text, number)?);
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
    id: String,
    title: String,
    status: ADossierStatus,
    outcome: String,
    body: Vec<String>,
}

/// Parses one dossier header line.
fn parse_dossier_header(text: &str, line: usize) -> Result<PartialDossier, ImportError> {
    let rest = text.strip_prefix("### ").unwrap_or(text);
    let id: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
    if !is_a_id(&id) {
        return Err(ImportError::BadId {
            line,
            id,
            detail: "not an A id".to_owned(),
        });
    }
    let Some(after_id) = rest.strip_prefix(id.as_str()) else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "an A dossier header's id does not prefix its span".to_owned(),
        });
    };
    let Some(title_and_marker) = after_id.strip_prefix(" — ") else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "an A dossier header has no ' — ' after the id".to_owned(),
        });
    };
    let Some((title, marker_text)) = split_at_dossier_marker(title_and_marker) else {
        return Err(ImportError::MalformedLine {
            line,
            detail: "an A dossier header has no ' — ✅ **' or ' — ⚠️ **' status marker".to_owned(),
        });
    };
    let (status, outcome) = parse_dossier_status(marker_text, line)?;
    Ok(PartialDossier {
        id,
        title: title.to_owned(),
        status,
        outcome,
        body: Vec::new(),
    })
}

/// Parses the status marker text that follows ` — ✅ **` or
/// ` — ⚠️ **`: the kind, the date, and the raw outcome (the remainder
/// after the date, the closing `**` stripped).
fn parse_dossier_status(text: &str, line: usize) -> Result<(ADossierStatus, String), ImportError> {
    let (kind, after_kind) = if let Some(rest) = text.strip_prefix("APPLIED ") {
        (ADossierKind::Applied, rest)
    } else if let Some(rest) = text.strip_prefix("SIGNED OFF ") {
        (ADossierKind::SignedOff, rest)
    } else if let Some(rest) = text.strip_prefix("HALF-APPLIED ") {
        (ADossierKind::HalfApplied, rest)
    } else {
        return Err(ImportError::BadStatus {
            line,
            text: text.trim().to_owned(),
            detail: "the marker must read 'APPLIED <date>', 'SIGNED OFF <date>' or \
                     'HALF-APPLIED <date>'"
                .to_owned(),
        });
    };
    // The via-form: `APPLIED via A<n>, <date>`
    // — carry the credited id, the date follows the comma.
    let mut via = None;
    let date_source = if kind == ADossierKind::Applied {
        match after_kind.strip_prefix("via ") {
            Some(after_via_word) => {
                let via_id: String = after_via_word
                    .chars()
                    .take_while(|c| c.is_alphanumeric())
                    .collect();
                if !is_a_id(&via_id) {
                    return Err(ImportError::BadStatus {
                        line,
                        text: text.trim().to_owned(),
                        detail: "the via-form's credit is not an A id".to_owned(),
                    });
                }
                let Some(after_id) = after_via_word.strip_prefix(via_id.as_str()) else {
                    return Err(ImportError::BadStatus {
                        line,
                        text: text.trim().to_owned(),
                        detail: "the via-form's credit is malformed".to_owned(),
                    });
                };
                let Some(after_comma) = after_id.strip_prefix(", ") else {
                    return Err(ImportError::BadStatus {
                        line,
                        text: text.trim().to_owned(),
                        detail: "the via-form lacks ', <date>' after its credit".to_owned(),
                    });
                };
                via = Some(via_id);
                after_comma
            }
            None => after_kind,
        }
    } else {
        after_kind
    };
    let Some(date) = date_source.get(..10) else {
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
    let Some(rest) = date_source.get(10..) else {
        return Err(ImportError::BadStatus {
            line,
            text: text.trim().to_owned(),
            detail: "the status date is missing".to_owned(),
        });
    };
    let outcome = rest.strip_suffix("**").unwrap_or(rest).trim().to_owned();
    Ok((
        ADossierStatus {
            kind,
            date: date.to_owned(),
            via,
        },
        outcome,
    ))
}

/// Finishes a dossier: joins its body into the raw text.
fn finish_dossier(dossier: PartialDossier) -> ADossier {
    ADossier {
        id: dossier.id,
        title: dossier.title,
        status: dossier.status,
        outcome: dossier.outcome,
        body: dossier.body.join("\n"),
    }
}
