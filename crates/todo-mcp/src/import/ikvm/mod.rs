//! The ikvm-html dialect importer (t47): the two board files —
//! TODO.md and decisions.md — as a dialect AST, strictly, then the
//! AST mapped into the shared executable plan. The mapping
//! conventions are spec §9's signed mapping table
//! (`docs/todo-mcp-design.md`) applied over the ikvm-html board's
//! measured shapes, with the measured grammar recorded in the task's
//! delivery record; this module owns the shapes below, and the
//! mapping lives in `map.rs`.
//!
//! Strictness is the load-bearing rule, the sibling dialects' own: a
//! register line that does not parse is a named error, never a silent
//! skip — a parser that skips an item reports less than the truth and
//! still looks complete. The registers are §2's milestone table, §3's
//! id-less blockers, the §4/§5 checkbox items, the decisions summary
//! table, the detail dossiers and the resolved footer. Everything
//! else is body text or walked-past prose — the board is prose around
//! registers, and tables are recognized by their header row, never by
//! position.
//!
//! The parsers are pure functions of text: no I/O, no clock, no
//! store, no new dependencies — file reading is the tool's. The AST
//! carries no line numbers on entries (board ids are the stable keys);
//! errors carry 1-based lines. The mapping (`map_ikvm` + the shared
//! `apply_plan`) is equally pure on its own half: the plan is built
//! from the AST and caller-passed parameters alone, and only the
//! executor touches a store.
//!
//! The composition (`parse_ikvm_board`) additionally cross-checks the
//! decisions file's three registers, each mismatch a named error
//! naming the smallest offending id in sorted order (the dacode B
//! discipline) so the report is deterministic:
//!
//! 1. the summary table's ids == the dossiers' ids (set equality —
//!    the anti-silent-omission rule);
//! 2. the struck summary ids == the footer ids == the bannered
//!    dossier ids (three-way — the resolved set must agree across
//!    all three places that record it);
//! 3. every resolved id's banner date == its footer date;
//! 4. every summary row's Gates cell == its dossier's Gates line.

mod board;
mod decisions;
mod error;
mod map;

use std::collections::BTreeSet;

pub use board::{BlockerItem, IkvmTodo, MilestoneRow, RegisterItem, parse_todo_md};
pub use decisions::{
    ConfidenceCell, DecisionRow, DecisionStatusCell, Dossier, FooterRow, IkvmDecisions,
    ResolvedBanner, parse_decisions,
};
pub use error::IkvmError;
pub use map::map_ikvm;

/// The composed two-file board: the two parses plus the cross-file
/// checks `parse_ikvm_board` enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IkvmBoard {
    /// The parsed TODO.md.
    pub todo: IkvmTodo,
    /// The parsed decisions.md.
    pub decisions: IkvmDecisions,
}

/// Parses the two board files into the composed board, enforcing the
/// four cross-checks the module doc lists. A mismatch is a named
/// error naming the offending id.
pub fn parse_ikvm_board(todo_md: &str, decisions: &str) -> Result<IkvmBoard, IkvmError> {
    let todo = parse_todo_md(todo_md)?;
    let file = parse_decisions(decisions)?;
    let summary_ids: BTreeSet<&str> = file.summary.iter().map(|row| row.id.as_str()).collect();
    let dossier_ids: BTreeSet<&str> = file.dossiers.iter().map(|d| d.id.as_str()).collect();
    compare_id_sets(
        &summary_ids,
        &dossier_ids,
        "the decisions summary table",
        "the decisions dossiers",
    )?;
    let struck_ids: BTreeSet<&str> = file
        .summary
        .iter()
        .filter(|row| row.struck)
        .map(|row| row.id.as_str())
        .collect();
    let footer_ids: BTreeSet<&str> = file.footer.iter().map(|row| row.id.as_str()).collect();
    let bannered_ids: BTreeSet<&str> = file
        .dossiers
        .iter()
        .filter(|dossier| dossier.resolved.is_some())
        .map(|dossier| dossier.id.as_str())
        .collect();
    three_way_resolved_set(&struck_ids, &footer_ids, &bannered_ids)?;
    check_banner_dates(&file, &footer_ids)?;
    check_gates_cells(&file)?;
    Ok(IkvmBoard {
        todo,
        decisions: file,
    })
}

/// Set equality of two id sets, naming the smallest mismatching id in
/// sorted order when they differ.
fn compare_id_sets(
    left: &BTreeSet<&str>,
    right: &BTreeSet<&str>,
    left_name: &str,
    right_name: &str,
) -> Result<(), IkvmError> {
    let mut mismatches: BTreeSet<&str> = BTreeSet::new();
    for id in left.difference(right) {
        mismatches.insert(id);
    }
    for id in right.difference(left) {
        mismatches.insert(id);
    }
    if let Some(id) = mismatches.first() {
        let detail = if left.contains(*id) {
            format!("{id} is in {left_name} but not in {right_name}")
        } else {
            format!("{id} is in {right_name} but not in {left_name}")
        };
        return Err(IkvmError::IdSetMismatch {
            id: (*id).to_owned(),
            detail,
        });
    }
    Ok(())
}

/// The three-way resolved-set equality: the struck summary ids, the
/// footer ids and the bannered dossier ids must agree — the resolved
/// set is recorded in three places, and any disagreement is a named
/// error naming the smallest id whose membership differs.
fn three_way_resolved_set(
    struck: &BTreeSet<&str>,
    footer: &BTreeSet<&str>,
    bannered: &BTreeSet<&str>,
) -> Result<(), IkvmError> {
    let mut union: BTreeSet<&str> = BTreeSet::new();
    for id in struck.iter().chain(footer.iter()).chain(bannered.iter()) {
        union.insert(*id);
    }
    for id in union {
        let in_struck = struck.contains(id);
        let in_footer = footer.contains(id);
        let in_bannered = bannered.contains(id);
        if in_struck == in_footer && in_footer == in_bannered {
            continue;
        }
        let places: [(bool, &str); 3] = [
            (in_struck, "the summary table's struck ids"),
            (in_footer, "the resolved footer"),
            (in_bannered, "the bannered dossiers"),
        ];
        let carries: Vec<&str> = places
            .iter()
            .filter(|(present, _)| *present)
            .map(|(_, name)| *name)
            .collect();
        let misses: Vec<&str> = places
            .iter()
            .filter(|(present, _)| !*present)
            .map(|(_, name)| *name)
            .collect();
        return Err(IkvmError::IdSetMismatch {
            id: id.to_owned(),
            detail: format!(
                "{id} is in {} but not in {}",
                carries.join(" and "),
                misses.join(" or ")
            ),
        });
    }
    Ok(())
}

/// Cross-check 3: every resolved id's banner date == its footer date.
/// The banner is the date's one home (the mapping reads it); the
/// footer is the pin.
fn check_banner_dates(
    file: &IkvmDecisions,
    resolved_ids: &BTreeSet<&str>,
) -> Result<(), IkvmError> {
    for id in resolved_ids {
        let banner_date = file
            .dossiers
            .iter()
            .find(|dossier| dossier.id == *id)
            .and_then(|dossier| dossier.resolved.as_ref())
            .map(|banner| banner.date.as_str());
        let footer_date = file
            .footer
            .iter()
            .find(|row| row.id == *id)
            .map(|row| row.date.as_str());
        if let (Some(banner), Some(footer)) = (banner_date, footer_date)
            && banner != footer
        {
            return Err(IkvmError::IdSetMismatch {
                id: (*id).to_owned(),
                detail: format!(
                    "{id}'s resolution banner says {banner} but the footer says {footer}"
                ),
            });
        }
    }
    Ok(())
}

/// Cross-check 4: every summary row's Gates cell == its dossier's
/// Gates line. The smallest mismatching id in sorted order is named.
fn check_gates_cells(file: &IkvmDecisions) -> Result<(), IkvmError> {
    let mut offenders: Vec<(&str, &str, &str)> = Vec::new();
    for row in &file.summary {
        let Some(dossier) = file.dossiers.iter().find(|d| d.id == row.id) else {
            continue;
        };
        if row.gates != dossier.gates {
            offenders.push((row.id.as_str(), row.gates.as_str(), dossier.gates.as_str()));
        }
    }
    offenders.sort_by_key(|(id, _, _)| *id);
    if let Some((id, cell, line)) = offenders.first() {
        return Err(IkvmError::IdSetMismatch {
            id: (*id).to_owned(),
            detail: format!(
                "{id}'s Gates cell says {cell} but its dossier's Gates line says {line}"
            ),
        });
    }
    Ok(())
}
