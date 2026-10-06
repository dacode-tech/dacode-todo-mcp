//! The dacode dialect importer (t42's parser, t43's mapping): the
//! three board files — TODO.md, the B decisions appendix, the A
//! design-deltas appendix — as a dialect AST, strictly, then the AST
//! mapped into an executable plan of domain drafts and edges. The
//! dialect inventory and the mapping conventions are the
//! dacode-dialect decision's (`docs/implementation/decisions.md`);
//! this module owns the shapes below, and the mapping lives in
//! `map.rs`/`plan.rs`.
//!
//! Strictness is the load-bearing rule: a register line that
//! does not parse is a named error, never a silent skip — a parser
//! that skips an item reports less than the truth and still looks
//! complete. The registers are §2's decisions table and B1a line,
//! §4's doc register, §5's milestones and sub-step tables, §6's U
//! items, the B summary and dossiers, the A index and dossiers.
//! Everything else is body text or walked-past prose — the board is
//! prose around registers, and tables are recognized by their header
//! row, never by position.
//!
//! The parsers are pure functions of text: no I/O, no clock, no
//! store, no new dependencies — file reading is the tool's
//! (t44). The AST carries no line numbers on entries (board ids are
//! the stable keys); errors carry 1-based lines. The mapping
//! (`map_board` + `apply_plan`) is equally pure on its own half: the
//! plan is built from the AST and caller-passed parameters alone, and
//! only the executor touches a store.
//!
//! Judgment calls (the task spec's (a)–(e)), bound where they apply:
//!
//! - (a) AST field granularity: `ast.rs`'s module doc.
//! - (b) Table-row cell splitting: `table.rs`'s module doc — escaped
//!   pipes are absent from the dialect, code-span pipes are present,
//!   so the splitter is code-span aware.
//! - (c) Line endings: `table.rs`'s `split_lines` — split on `\n`,
//!   trailing `\r` trimmed.
//! - (d) Type and field naming: `ast.rs`'s module doc.
//! - (e) `board.rs` is split by responsibility whenever the file
//!   budget would otherwise be exceeded: `doc_register.rs` holds §4,
//!   `items.rs` holds the checkbox-item grammar and marker
//!   extraction, and `board.rs` keeps the walker. The budget is the
//!   awk measure in `.agents/roles/_rust.md`'s, not a comment's.
//!
//! The composition (`parse_board`) additionally cross-checks ids:
//! TODO.md §2 ids == B summary ids == B dossier ids (set equality),
//! while the A side is one-directional — every id in the A index or
//! the A applied table must have a dossier, and dossiers may exceed
//! the tables. A mismatch is a named error naming the offending id,
//! the smallest in sorted order so the report is deterministic.

mod ast;
mod board;
mod decisions;
mod deltas;
mod doc_register;
mod error;
mod items;
mod map;
mod plan;
mod resolve;
mod table;

use std::collections::BTreeSet;

pub use ast::{
    AAppliedRow, ADossier, ADossierKind, ADossierStatus, AIndexRow, AppliedVia, BDossier,
    BDossierKind, BDossierStatus, BSummaryRow, DacodeBoard, DecisionsAppendix, DeltasAppendix,
    DocDepth, DocEntry, GatesCell, Marker, MilestoneItem, SubStep, Subsection, SummaryRow,
    SummaryStatus, TodoMd, UItem,
};
pub use error::ImportError;
pub use map::map_board;
pub use plan::{ApplyError, ImportOptions, ImportPlan, PlanOp, apply_plan};

pub use board::parse_todo_md;
pub use decisions::parse_b_decisions;
pub use deltas::parse_a_deltas;

/// Parses the three board files into the composed board, enforcing
/// the cross-file id checks: set equality on the B side (TODO.md §2
/// ids == B summary ids == B dossier ids), and on the A side that
/// every id in the A index or the A applied table has a dossier while
/// dossiers may exceed the tables. A mismatch is a named error naming
/// the offending id.
pub fn parse_board(
    todo_md: &str,
    b_decisions: &str,
    a_deltas: &str,
) -> Result<DacodeBoard, ImportError> {
    let todo = parse_todo_md(todo_md)?;
    let decisions = parse_b_decisions(b_decisions)?;
    let deltas = parse_a_deltas(a_deltas)?;
    let todo_ids: BTreeSet<&str> = todo.decisions.iter().map(|row| row.id.as_str()).collect();
    let summary_ids: BTreeSet<&str> = decisions
        .summary
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    let dossier_ids: BTreeSet<&str> = decisions.dossiers.iter().map(|d| d.id.as_str()).collect();
    compare_id_sets(&todo_ids, &summary_ids, "TODO.md §2", "the B summary")?;
    compare_id_sets(&todo_ids, &dossier_ids, "TODO.md §2", "the B dossiers")?;
    let index_ids: BTreeSet<&str> = deltas.index.iter().map(|row| row.id.as_str()).collect();
    let applied_ids: BTreeSet<&str> = deltas.applied.iter().map(|row| row.id.as_str()).collect();
    let a_dossier_ids: BTreeSet<&str> = deltas.dossiers.iter().map(|d| d.id.as_str()).collect();
    // The A cross-check is one-directional (the fix round's
    // relaxation): every id in either table must have a dossier — a
    // dangling table row is a named error — while dossiers may exceed
    // the tables.
    require_dossiers(&index_ids, &a_dossier_ids, "the A index")?;
    require_dossiers(&applied_ids, &a_dossier_ids, "the A applied table")?;
    Ok(DacodeBoard {
        todo,
        decisions,
        deltas,
    })
}

/// Every id in `table` must have a dossier; dossiers may exceed the
/// table. A dangling table row names the smallest missing id.
fn require_dossiers(
    table: &BTreeSet<&str>,
    dossiers: &BTreeSet<&str>,
    table_name: &str,
) -> Result<(), ImportError> {
    if let Some(id) = table.difference(dossiers).min() {
        return Err(ImportError::IdSetMismatch {
            id: (*id).to_owned(),
            detail: format!("{id} is in {table_name} but has no dossier"),
        });
    }
    Ok(())
}

/// Set equality of two id sets, naming the smallest mismatching id in
/// sorted order when they differ.
fn compare_id_sets(
    left: &BTreeSet<&str>,
    right: &BTreeSet<&str>,
    left_name: &str,
    right_name: &str,
) -> Result<(), ImportError> {
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
        return Err(ImportError::IdSetMismatch {
            id: (*id).to_owned(),
            detail,
        });
    }
    Ok(())
}
