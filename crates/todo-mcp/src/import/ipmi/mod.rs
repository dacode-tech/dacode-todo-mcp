//! The ipmi-collector dialect importer (t77): the two board files —
//! TODO.md and decisions.md — as a dialect AST, strictly, then the
//! AST mapped into the shared executable plan. The mapping
//! conventions are spec §9's signed mapping table
//! (`docs/todo-mcp-design.md`) applied over the ipmi-collector
//! board's measured shapes; the measured grammar is the t75 dialect
//! study (board §6's 2026-10-10 entry), measured at commit `47e0e7b`
//! — a two-file board, an open-only U register (resolved items are
//! deleted, the board's own rule), a checkbox-less `D-N` debt
//! register, a `done`-only milestone table and a five-column
//! decisions table with dossiers. This module owns the shapes below,
//! and the mapping lives in `map.rs`.
//!
//! Strictness is the load-bearing rule, the sibling dialects' own: a
//! register line that does not parse is a named error, never a silent
//! skip — a parser that skips an item reports less than the truth and
//! still looks complete. The registers are §2's milestone table, the
//! §4/§5 items, the decisions summary table and the decision
//! dossiers. Everything else is body text or walked-past prose — the
//! board is prose around registers, and tables are recognized by
//! their header row, never by position.
//!
//! The parsers are pure functions of text: no I/O, no clock, no
//! store, no new dependencies — file reading is the tool's. The AST
//! carries no line numbers on entries (board ids are the stable keys);
//! errors carry 1-based lines. The mapping (`map_ipmi` + the shared
//! `apply_plan`) is equally pure on its own half: the plan is built
//! from the AST and caller-passed parameters alone, and only the
//! executor touches a store.
//!
//! The composition (`parse_ipmi_board`) additionally cross-checks
//! ids: the decisions summary table's ids == the dossiers' ids (set
//! equality — the anti-silent-omission rule, the dacode B
//! discipline). A mismatch is a named error naming the offending id,
//! the smallest in sorted order so the report is deterministic.

mod board;
mod decisions;
mod error;
mod map;

use std::collections::BTreeSet;

pub use board::{IpmiTodo, MilestoneRow, MilestoneState, RegisterItem, parse_todo_md};
pub use decisions::{DecisionRow, DecisionStatusCell, Dossier, IpmiDecisions, parse_decisions};
pub use error::IpmiError;
pub use map::map_ipmi;

/// The composed two-file board: the two parses plus the cross-file id
/// check `parse_ipmi_board` enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpmiBoard {
    /// The parsed TODO.md.
    pub todo: IpmiTodo,
    /// The parsed decisions.md.
    pub decisions: IpmiDecisions,
}

/// Parses the two board files into the composed board, enforcing the
/// cross-file id check: the decisions summary table's ids == the
/// dossiers' ids (set equality). A mismatch is a named error naming
/// the offending id.
pub fn parse_ipmi_board(todo_md: &str, decisions: &str) -> Result<IpmiBoard, IpmiError> {
    let todo = parse_todo_md(todo_md)?;
    let file = parse_decisions(decisions)?;
    let table_ids: BTreeSet<&str> = file.summary.iter().map(|row| row.id.as_str()).collect();
    let dossier_ids: BTreeSet<&str> = file.dossiers.iter().map(|d| d.id.as_str()).collect();
    compare_id_sets(
        &table_ids,
        &dossier_ids,
        "the decisions summary table",
        "the decisions dossiers",
    )?;
    Ok(IpmiBoard {
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
) -> Result<(), IpmiError> {
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
        return Err(IpmiError::IdSetMismatch {
            id: (*id).to_owned(),
            detail,
        });
    }
    Ok(())
}
