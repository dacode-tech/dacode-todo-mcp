//! The ipmi-collector board → domain mapping (t77): the dialect AST
//! translated into the shared executable [`ImportPlan`]. The plan
//! types, the executor and its `ApplyError` are the shared plan
//! module's — one of the two code homes the dialects share, the
//! helpers module being the other — and this mapping produces that
//! shape; everything else here is ipmi-collector's own. The ipmi
//! `ImportOptions` carries an empty `doc_paths` map: the shared type
//! requires the field, and this board has no doc register to fill it
//! from — the mapping never consults it.
//!
//! `map_ipmi` is a pure function of `(board, opts)` — no I/O, no
//! clock, no store, `BTreeMap` throughout — so two runs over one
//! board yield equal plans (pinned by test).
//!
//! # The plan's total order
//!
//! 1. **epics** — the §2 milestones in board order, then the
//!    synthetic `Follow-ups` epic, then the synthetic `Debt` epic;
//! 2. **decisions** — the summary rows in table order;
//! 3. **tasks** — the §4 items in register order, then the §5 items
//!    in register order;
//! 4. **findings** — one per Gates token naming an id no register
//!    carries (decision table order, token scan order within a row);
//! 5. **edges** — the Gates edges (decision table order, token scan
//!    order within a row).
//!
//! No sorts anywhere: the register order **is** the plan's order.
//! The register is gapped (resolved items are deleted, the board's
//! own rule), so no ordering reproduces the board's numbering — the
//! store allocates fresh sequential ids, and the file's order is the
//! only order the board states. Every node imported from a register
//! carries its board id in `references` (`board:<id>`), so the
//! mapping stays traceable past the renumbering.
//!
//! # The conventions
//!
//! - **Status**: the §2 State word governs the epics (`done` →
//!   `closed`); every task is `open` by grammar (§4's register is
//!   open-only, §5's carries no checkbox); the decision's status is
//!   the Status cell (`signed` → `signed`, `deferred` → `open` — the
//!   server has no deferred decision status; not-signed is open).
//! - **Resolution**: a signed decision's resolution is the Status
//!   cell's remainder after the bold word; a deferred decision
//!   carries none (the board keeps the deferral prose).
//! - **Timestamps**: `imported_at` everywhere — the board carries no
//!   per-item dates the mapping reads (the milestone done-dates are
//!   completion prose in the State cell, not node timestamps).
//! - **Kinds**: §4's items are `task` under `Follow-ups`; §5's items
//!   are `debt` under `Debt` — the section is the declaration.
//! - **Bodies**: the item's joined prose is the task's body — the
//!   register prose is the item's description, and the board's bulk
//!   (§3/§6) is walked past, so no size concern arises.
//! - **Confidence**: `Medium` — the board's decisions table carries
//!   no Confidence column, and the middle neither overstates nor
//!   understates an unmeasured confidence.
//! - **Edges**: the Gates cell's id-bearing `U<N>`/`M<N>` tokens (the
//!   dacode A-index precedent for id-bearing prose), deduplicated,
//!   scan order; `U` → a `DecisionToTask` edge to the task the board
//!   id names, `M` → a `DecisionToEpic` edge to the milestone epic.
//!   The bare em-dash and id-free prose yield no edges.
//! - **Findings**: a Gates token naming an id no register carries
//!   becomes a `finding`-kind task under `Follow-ups` (the sibling
//!   dialects' shape for a recognized-but-unresolvable assertion)
//!   rather than a refusal — the register is open-only (resolved
//!   items are deleted), so a Gates cell referencing a resolved id
//!   is an expected future state, and the import must not hard-fail
//!   on it. No prose-phrase findings: the parser recognizes no
//!   dependency prose.
//! - **References**: every register node carries its board id as
//!   `board:<id>` (`other` kind) — the §9 oddball convention applied
//!   to the whole gapped register, uniform even where the store id
//!   coincides; the synthetic epics carry none.
//! - **Not imported**: the State/Status prose after the verdict, the
//!   dossier headers' status tails and bodies, §1/§3/§6 — already
//!   absent from the AST by the parser's design, except the dossier
//!   fields, which the map drops.

use std::collections::BTreeMap;

use todo_domain::{
    BlocksEdge, Confidence, DecisionDraft, DecisionId, DecisionStatus, EpicDraft, EpicId,
    ReferenceKind, References, RepoSlug, Status, TaskDraft, TaskId, TaskKind,
};

use super::IpmiBoard;
use super::board::{MilestoneState, RegisterItem, is_m_id, is_u_id};
use super::decisions::DecisionStatusCell;
use super::error::IpmiError;
use crate::import::helpers::{HelperError, clean_id_word, position, references};
use crate::import::plan::{ImportOptions, ImportPlan, PlanOp};

/// The fixed verify literal for every task the board gives no gate of
/// its own: the register's own closure convention is deletion, so the
/// literal names it.
const VERIFY_LITERAL: &str = "imported: the board's open-only register";
/// The synthetic `Follow-ups` epic's title.
const FOLLOWUPS_TITLE: &str = "Follow-ups";
/// The synthetic `Debt` epic's title.
const DEBT_TITLE: &str = "Debt";
/// The `Follow-ups` epic's provenance description.
const FOLLOWUPS_DESCRIPTION: &str =
    "the ipmi-collector board's §4 follow-ups, not tied to a milestone";
/// The `Debt` epic's provenance description — the same requirement and
/// the same reading as [`FOLLOWUPS_DESCRIPTION`]'s; the second clause
/// is the domain's own `debt`-kind vocabulary.
const DEBT_DESCRIPTION: &str = "the ipmi-collector board's §5 debt, recorded rather than forgotten";
/// A milestone epic's provenance description prefix; the milestone id
/// is appended.
const MILESTONE_DESCRIPTION: &str = "the ipmi-collector board's §2 milestone ";

/// Maps a parsed ipmi-collector board into an executable plan. Pure:
/// the same board and options yield an equal plan, and nothing is
/// written anywhere — the plan is executed separately by the shared
/// `apply_plan`.
pub fn map_ipmi(board: &IpmiBoard, opts: &ImportOptions) -> Result<ImportPlan, IpmiError> {
    let index = BoardIndex::build(board, &opts.slug);
    let mut ops: Vec<PlanOp> = Vec::new();
    push_epic_ops(&mut ops, board, opts)?;
    push_decision_ops(&mut ops, board, opts)?;
    push_task_ops(&mut ops, board, &index, opts)?;
    push_finding_ops(&mut ops, board, &index, opts)?;
    push_edge_ops(&mut ops, board, &index, opts)?;
    Ok(ImportPlan { ops })
}

/// The board's ids resolved to the ids the plan constructs — the
/// reproduction map. A lookup answering `None` means the board does
/// not carry that id; the caller decides whether that is a refusal.
struct BoardIndex {
    /// M id → the milestone epic's constructed id (board order).
    epics: BTreeMap<String, EpicId>,
    /// Board id (U or D-N) → the task's constructed id (register
    /// order: §4 then §5).
    tasks: BTreeMap<String, TaskId>,
    /// The synthetic `Follow-ups` epic's constructed id.
    followups: EpicId,
    /// The synthetic `Debt` epic's constructed id.
    debt: EpicId,
}

impl BoardIndex {
    /// Builds the map from the board's populations: a kind's i-th
    /// item gets the kind's i-th sequential number, mirroring the
    /// store's allocation.
    fn build(board: &IpmiBoard, slug: &RepoSlug) -> Self {
        let epics: BTreeMap<String, EpicId> = board
            .todo
            .milestones
            .iter()
            .enumerate()
            .map(|(i, row)| {
                (
                    row.id.clone(),
                    EpicId::from_nonzero(slug.clone(), position(i)),
                )
            })
            .collect();
        let tasks: BTreeMap<String, TaskId> = board
            .todo
            .followups
            .iter()
            .chain(board.todo.debt.iter())
            .enumerate()
            .map(|(i, item)| {
                (
                    item.id.clone(),
                    TaskId::from_nonzero(slug.clone(), position(i)),
                )
            })
            .collect();
        Self {
            epics,
            tasks,
            followups: EpicId::from_nonzero(slug.clone(), position(board.todo.milestones.len())),
            debt: EpicId::from_nonzero(slug.clone(), position(board.todo.milestones.len() + 1)),
        }
    }
}

/// The epic ops: the milestones (board order, the State word
/// governing status), then the two synthetics — open, structural
/// containers for open work.
fn push_epic_ops(
    ops: &mut Vec<PlanOp>,
    board: &IpmiBoard,
    opts: &ImportOptions,
) -> Result<(), IpmiError> {
    for (i, row) in board.todo.milestones.iter().enumerate() {
        ops.push(PlanOp::AddEpic {
            draft: EpicDraft {
                title: row.title.clone(),
                description: format!("{MILESTONE_DESCRIPTION}{}", row.id),
                status: epic_status(row.status),
                references: board_reference(&row.id)?,
            },
            id: EpicId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
    ops.push(PlanOp::AddEpic {
        draft: EpicDraft {
            title: FOLLOWUPS_TITLE.to_owned(),
            description: FOLLOWUPS_DESCRIPTION.to_owned(),
            status: Status::Open,
            references: References::new(),
        },
        id: EpicId::from_nonzero(opts.slug.clone(), position(board.todo.milestones.len())),
        at: opts.imported_at.clone(),
    });
    ops.push(PlanOp::AddEpic {
        draft: EpicDraft {
            title: DEBT_TITLE.to_owned(),
            description: DEBT_DESCRIPTION.to_owned(),
            status: Status::Open,
            references: References::new(),
        },
        id: EpicId::from_nonzero(opts.slug.clone(), position(board.todo.milestones.len() + 1)),
        at: opts.imported_at.clone(),
    });
    Ok(())
}

/// The State cell's verdict as the epic's status.
fn epic_status(state: MilestoneState) -> Status {
    match state {
        MilestoneState::Done => Status::Closed,
    }
}

/// The decision ops: the summary rows (table order), the Status cell
/// governing status and resolution, the recommendation carried, the
/// confidence the fixed middle.
fn push_decision_ops(
    ops: &mut Vec<PlanOp>,
    board: &IpmiBoard,
    opts: &ImportOptions,
) -> Result<(), IpmiError> {
    for (i, row) in board.decisions.summary.iter().enumerate() {
        let (status, resolution) = match &row.status {
            DecisionStatusCell::Signed { resolution } => {
                (DecisionStatus::Signed, Some(resolution.clone()))
            }
            DecisionStatusCell::Deferred { .. } => (DecisionStatus::Open, None),
        };
        ops.push(PlanOp::AddDecision {
            draft: DecisionDraft {
                title: row.title.clone(),
                status,
                // The options prose lives in the dossiers and stays in
                // the board (§9's cutover).
                options: Vec::new(),
                recommendation: Some(row.recommendation.clone()),
                resolution,
                confidence: Confidence::Medium,
                references: board_reference(&row.id)?,
            },
            id: DecisionId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
    Ok(())
}

/// The task ops: the §4 items (register order), then the §5 items
/// (register order) — every task open by grammar, the prose the body.
fn push_task_ops(
    ops: &mut Vec<PlanOp>,
    board: &IpmiBoard,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), IpmiError> {
    let mut number = 0usize;
    for item in &board.todo.followups {
        ops.push(task_op(
            opts,
            number,
            &index.followups,
            TaskKind::Task,
            item,
        )?);
        number += 1;
    }
    for item in &board.todo.debt {
        ops.push(task_op(opts, number, &index.debt, TaskKind::Debt, item)?);
        number += 1;
    }
    Ok(())
}

/// One task op: parent and kind per the section, open, the fixed
/// verify literal, the prose as the body, the board-id reference.
fn task_op(
    opts: &ImportOptions,
    number: usize,
    epic: &EpicId,
    kind: TaskKind,
    item: &RegisterItem,
) -> Result<PlanOp, IpmiError> {
    Ok(PlanOp::AddTask {
        draft: TaskDraft {
            epic: epic.clone(),
            kind,
            title: item.title.clone(),
            body: item.prose.clone(),
            status: Status::Open,
            verify: Some(VERIFY_LITERAL.to_owned()),
            evidence: None,
            note: None,
            references: board_reference(&item.id)?,
            priority: None,
            until: None,
        },
        id: TaskId::from_nonzero(opts.slug.clone(), position(number)),
        at: opts.imported_at.clone(),
    })
}

/// The finding ops: one `finding`-kind task per Gates token naming an
/// id no register carries, numbered after the register tasks —
/// decision table order, token scan order within a row. The register
/// is open-only (resolved items are deleted), so a Gates cell
/// referencing a resolved id is an expected future state: the finding
/// records it rather than refusing the import.
fn push_finding_ops(
    ops: &mut Vec<PlanOp>,
    board: &IpmiBoard,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), IpmiError> {
    let mut number = board.todo.followups.len() + board.todo.debt.len();
    for row in &board.decisions.summary {
        for token in gates_tokens(&row.gates) {
            if !resolves(index, &token) {
                ops.push(finding_op(
                    opts,
                    number,
                    &index.followups,
                    &row.id,
                    &token,
                    &row.gates,
                )?);
                number += 1;
            }
        }
    }
    Ok(())
}

/// One finding task: parent = `Follow-ups`, title =
/// `<D-id> gates <token>`, body = the Gates cell text, open, the
/// fixed verify literal, one `other` reference targeting the board
/// id of the asserting decision.
fn finding_op(
    opts: &ImportOptions,
    number: usize,
    epic: &EpicId,
    source: &str,
    token: &str,
    body: &str,
) -> Result<PlanOp, IpmiError> {
    Ok(PlanOp::AddTask {
        draft: TaskDraft {
            epic: epic.clone(),
            kind: TaskKind::Finding,
            title: format!("{source} gates {token}"),
            body: body.to_owned(),
            status: Status::Open,
            verify: Some(VERIFY_LITERAL.to_owned()),
            evidence: None,
            note: None,
            references: board_reference(source)?,
            priority: None,
            until: None,
        },
        id: TaskId::from_nonzero(opts.slug.clone(), position(number)),
        at: opts.imported_at.clone(),
    })
}

/// The edge ops: the Gates edges — for each decision row (table
/// order), the cell's id-bearing tokens (deduplicated, scan order),
/// `U` → a `DecisionToTask` edge, `M` → a `DecisionToEpic` edge. A
/// token naming an id no register carries yields no edge here — the
/// finding ops record it.
fn push_edge_ops(
    ops: &mut Vec<PlanOp>,
    board: &IpmiBoard,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), IpmiError> {
    for (i, row) in board.decisions.summary.iter().enumerate() {
        let from = DecisionId::from_nonzero(opts.slug.clone(), position(i));
        for token in gates_tokens(&row.gates) {
            let edge = if is_u_id(&token) {
                let Some(to) = index.tasks.get(&token) else {
                    continue;
                };
                BlocksEdge::DecisionToTask {
                    from: from.clone(),
                    to: to.clone(),
                }
            } else {
                let Some(to) = index.epics.get(&token) else {
                    continue;
                };
                BlocksEdge::DecisionToEpic {
                    from: from.clone(),
                    to: to.clone(),
                }
            };
            ops.push(PlanOp::LinkBlocks {
                edge,
                at: opts.imported_at.clone(),
            });
        }
    }
    Ok(())
}

/// Whether a Gates token resolves against the board's registers — a
/// `U` token against the task index, an `M` token against the epic
/// index.
fn resolves(index: &BoardIndex, token: &str) -> bool {
    if is_u_id(token) {
        index.tasks.contains_key(token)
    } else {
        index.epics.contains_key(token)
    }
}

/// The id-bearing tokens of a Gates cell: whitespace-split words,
/// punctuation-stripped at the edges and a trailing `'s` removed (the
/// camera dependency scanner's word rule), kept when `U<digits>` or
/// `M<digits>` exactly — the families the mapping resolves.
/// Deduplicated, first-occurrence order. The bare em-dash and
/// id-free prose yield none.
fn gates_tokens(cell: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in cell.split_whitespace() {
        let token = clean_id_word(word);
        if (is_u_id(token) || is_m_id(token)) && !out.iter().any(|seen| seen == token) {
            out.push(token.to_owned());
        }
    }
    out
}

/// The board-id reference every register node carries: `board:<id>`,
/// the `other` kind — the §9 oddball convention applied to the whole
/// gapped register.
fn board_reference(id: &str) -> Result<References, IpmiError> {
    Ok(references(vec![(
        ReferenceKind::Other,
        format!("board:{id}"),
    )])?)
}

/// The shared helpers' neutral error into this dialect's own,
/// variant-for-variant — the seam that lets `references` live in the
/// shared module while every message and field stays the dialect's.
/// The `BadDate` arm is unreachable: this mapping reads no board
/// dates (every stamp is the caller-passed `imported_at`).
impl From<HelperError> for IpmiError {
    fn from(error: HelperError) -> Self {
        match error {
            HelperError::BadDate { date } => IpmiError::BadDate { date },
            HelperError::BadReference { target, detail } => {
                IpmiError::BadReference { target, detail }
            }
        }
    }
}
