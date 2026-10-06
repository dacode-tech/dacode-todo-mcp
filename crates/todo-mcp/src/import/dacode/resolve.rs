//! The edge/finding resolution — the mechanical-edge rule's second
//! half: the dependency assertions the board's registers and markers
//! make, resolved into `blocks` edges where the §5.1 shapes allow and
//! into `finding`-kind tasks where they do not, never a silent
//! omission. The node mapping (the drafts the assertions attach to)
//! is `map.rs`'s; this module owns the resolution and its two total
//! orders.
//!
//! # The orders
//!
//! - **Findings** (tasks, numbered after `B1a`): A-depends findings
//!   (A-id order, cell order within), then Gates-milestone findings
//!   (B-id order), then the marker findings (U items then M items in
//!   numeric-id order, marker order within).
//! - **Edges** (last in the plan): Gates milestone edges (B-id
//!   order), then A-depends edges (A-id order, cell order within),
//!   then blocked-on edges (U items then M items in numeric-id order,
//!   marker order within).
//!
//! # The readings this file pins
//!
//! - **M-sourced `blocked on`**: the edge shapes presume a task
//!   source; an M item (an epic) blocked on a B decision becomes the
//!   legal decision→epic edge, and any other target becomes a finding
//!   — the epic-level `blocks` the spec defers is exactly the
//!   unresolvable case.
//! - **A Gates milestone naming no §5 milestone** becomes a finding
//!   under `Follow-ups` — the assertion is not tied to any milestone
//!   (the one it names does not exist), which is that epic's own
//!   meaning. The body is the dossier's raw text, the asserting
//!   item's own.
//! - A dossier with no summary row (unreachable through `parse_board`'s
//!   set equality, reachable in a hand-built AST) carries no Gates
//!   cell, and one with no index row (legal: dossiers may exceed the
//!   tables) carries no Depends-on cell — either way there is no
//!   assertion to resolve, so nothing is emitted.
//!
//! Judgment calls bound here: the finding title's format
//! (`<source> <verb> <target>`, the verb fixed per assertion kind —
//! `gates`, `blocked on`, `depends on`); the A-depends finding's body
//! (the index row's Delta cell — the parser carries no raw
//! Depends-on line, and the row's own cell identifies the asserting
//! row).

use todo_domain::{
    BlocksEdge, DecisionId, EpicId, ReferenceKind, Status, TaskDraft, TaskId, TaskKind,
};

use super::ast::Marker;
use super::error::ImportError;
use super::map::{BoardIndex, SortedBoard, VERIFY_LITERAL};
use crate::import::helpers::{position, references};
use crate::import::plan::{ImportOptions, PlanOp};

/// The finding ops: one `finding`-kind task per recognized-but-
/// unresolvable assertion, in the module doc's order.
pub(super) fn push_finding_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
    b1a_position: usize,
) -> Result<(), ImportError> {
    let mut number = b1a_position + 1;
    let a_base = sorted.u_items.len() + sorted.substeps.len();
    // A-depends findings: a Depends-on token resolving to no legal
    // edge. The body is the index row's Delta cell — the parser
    // carries no raw Depends-on line, and the row's own cell
    // identifies the asserting row.
    for (i, dossier) in sorted.a_dossiers.iter().enumerate() {
        let Some(row) = sorted.a_index.iter().find(|row| row.id == dossier.id) else {
            continue;
        };
        let dependent = TaskId::from_nonzero(opts.slug.clone(), position(a_base + i));
        for token in &row.depends_on {
            if dependency_edge(index, token, &dependent).is_none() {
                ops.push(finding_op(
                    opts,
                    number,
                    index.deltas.clone(),
                    &dossier.id,
                    "depends on",
                    token,
                    row.delta.clone(),
                )?);
                number += 1;
            }
        }
    }
    // Gates-milestone findings: a Gates milestone naming no §5
    // milestone — the assertion is not tied to any milestone (the one
    // it names does not exist), which is the Follow-ups epic's own
    // meaning. The body is the dossier's raw text, the asserting
    // item's own.
    for dossier in &sorted.b_dossiers {
        let Some(row) = sorted.b_summary.iter().find(|row| row.id == dossier.id) else {
            continue;
        };
        if let Some(milestone) = &row.gates.milestone
            && !index.epics.contains_key(milestone)
        {
            ops.push(finding_op(
                opts,
                number,
                index.followups.clone(),
                &dossier.id,
                "gates",
                milestone,
                dossier.body.clone(),
            )?);
            number += 1;
        }
    }
    // Marker findings: U items then M items, marker order within.
    for (i, item) in sorted.u_items.iter().enumerate() {
        let blocked = TaskId::from_nonzero(opts.slug.clone(), position(i));
        for marker in &item.markers {
            if let Some((verb, target, line)) = marker_assertion(marker)
                && dependency_edge(index, target, &blocked).is_none()
            {
                ops.push(finding_op(
                    opts,
                    number,
                    index.followups.clone(),
                    &item.id,
                    verb,
                    target,
                    line,
                )?);
                number += 1;
            }
        }
    }
    for (i, item) in sorted.milestones.iter().enumerate() {
        let epic = EpicId::from_nonzero(opts.slug.clone(), position(i));
        for marker in &item.markers {
            if let Some((verb, target, line)) = marker_assertion(marker)
                && blocked_on_epic_edge(index, target, &epic).is_none()
            {
                ops.push(finding_op(
                    opts,
                    number,
                    epic.clone(),
                    &item.id,
                    verb,
                    target,
                    line,
                )?);
                number += 1;
            }
        }
    }
    Ok(())
}

/// One finding task: parent = the asserting item's epic, title =
/// `<source> <verb> <target>`, body = the assertion's raw text, open,
/// the fixed verify literal, one `other` reference targeting the
/// source id.
fn finding_op(
    opts: &ImportOptions,
    number: usize,
    epic: EpicId,
    source: &str,
    verb: &str,
    target: &str,
    body: String,
) -> Result<PlanOp, ImportError> {
    Ok(PlanOp::AddTask {
        draft: TaskDraft {
            epic,
            kind: TaskKind::Finding,
            title: format!("{source} {verb} {target}"),
            body,
            status: Status::Open,
            verify: Some(VERIFY_LITERAL.to_owned()),
            evidence: None,
            note: None,
            references: references(vec![(ReferenceKind::Other, source.to_owned())])?,
            priority: None,
            until: None,
        },
        id: TaskId::from_nonzero(opts.slug.clone(), position(number)),
        at: opts.imported_at.clone(),
    })
}

/// The edge ops, in the module doc's order.
pub(super) fn push_edge_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) {
    for (i, dossier) in sorted.b_dossiers.iter().enumerate() {
        let from = DecisionId::from_nonzero(opts.slug.clone(), position(i));
        let Some(row) = sorted.b_summary.iter().find(|row| row.id == dossier.id) else {
            continue;
        };
        if let Some(milestone) = &row.gates.milestone
            && let Some(to) = index.epics.get(milestone)
        {
            ops.push(PlanOp::LinkBlocks {
                edge: BlocksEdge::DecisionToEpic {
                    from,
                    to: to.clone(),
                },
                at: opts.imported_at.clone(),
            });
        }
    }
    let a_base = sorted.u_items.len() + sorted.substeps.len();
    for (i, dossier) in sorted.a_dossiers.iter().enumerate() {
        let dependent = TaskId::from_nonzero(opts.slug.clone(), position(a_base + i));
        let Some(row) = sorted.a_index.iter().find(|row| row.id == dossier.id) else {
            continue;
        };
        for token in &row.depends_on {
            if let Some(edge) = dependency_edge(index, token, &dependent) {
                ops.push(PlanOp::LinkBlocks {
                    edge,
                    at: opts.imported_at.clone(),
                });
            }
        }
    }
    for (i, item) in sorted.u_items.iter().enumerate() {
        let blocked = TaskId::from_nonzero(opts.slug.clone(), position(i));
        for marker in &item.markers {
            if let Marker::BlockedOn { id, .. } = marker
                && let Some(edge) = dependency_edge(index, id, &blocked)
            {
                ops.push(PlanOp::LinkBlocks {
                    edge,
                    at: opts.imported_at.clone(),
                });
            }
        }
    }
    for (i, item) in sorted.milestones.iter().enumerate() {
        let epic = EpicId::from_nonzero(opts.slug.clone(), position(i));
        for marker in &item.markers {
            if let Marker::BlockedOn { id, .. } = marker
                && let Some(edge) = blocked_on_epic_edge(index, id, &epic)
            {
                ops.push(PlanOp::LinkBlocks {
                    edge,
                    at: opts.imported_at.clone(),
                });
            }
        }
    }
}

/// The edge a dependency token names with `dependent` held back: a B
/// id gates it (decision→task), a U or A id blocks it (task→task). M
/// ids and unknown tokens resolve to no legal edge — the finding
/// case.
fn dependency_edge(index: &BoardIndex, token: &str, dependent: &TaskId) -> Option<BlocksEdge> {
    if let Some(from) = index.decisions.get(token) {
        return Some(BlocksEdge::DecisionToTask {
            from: from.clone(),
            to: dependent.clone(),
        });
    }
    let from = index
        .u_tasks
        .get(token)
        .or_else(|| index.a_tasks.get(token))?;
    Some(BlocksEdge::TaskToTask {
        from: from.clone(),
        to: dependent.clone(),
    })
}

/// The edge an M item's `blocked on` marker names: only a B target
/// resolves — the decision gating the epic. Any other target has no
/// legal shape (an epic is never a blocks source; epic-level `blocks`
/// is the spec's deferred follow-up), the finding case.
fn blocked_on_epic_edge(index: &BoardIndex, token: &str, epic: &EpicId) -> Option<BlocksEdge> {
    index
        .decisions
        .get(token)
        .map(|from| BlocksEdge::DecisionToEpic {
            from: from.clone(),
            to: epic.clone(),
        })
}

/// A marker's assertion, when it is one the mapping resolves: the
/// verb, the target id, and the raw line. `Check`, `InWork`, `Warning`
/// and `Brainstorm` are not dependency assertions.
fn marker_assertion(marker: &Marker) -> Option<(&'static str, &str, String)> {
    match marker {
        Marker::BlockedOn { id, line } => Some(("blocked on", id.as_str(), line.clone())),
        Marker::MilestoneAssertion { milestone, line } => {
            Some(("gates", milestone.as_str(), line.clone()))
        }
        _ => None,
    }
}
