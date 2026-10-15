//! The ikvm-html board → domain mapping (t47): the dialect AST
//! translated into the shared executable [`ImportPlan`]. The plan
//! types, the executor and its `ApplyError` are the shared plan
//! module's — one of the two code homes the dialects share, the
//! helpers module being the other — and this mapping produces that
//! shape; everything else here is ikvm-html's own. The ikvm
//! `ImportOptions` carries an empty `doc_paths` map: the shared type
//! requires the field, and this board has no doc register to fill it
//! from — the mapping never consults it.
//!
//! `map_ikvm` is a pure function of `(board, opts)` — no I/O, no
//! clock, no store, `BTreeMap` throughout — so two runs over one
//! board yield equal plans (pinned by test).
//!
//! # The plan's total order
//!
//! 1. **epics** — the §2 milestones sorted by numeric id, then the
//!    synthetic `Follow-ups`, `Debt` and `Blockers` epics (nine at
//!    the live board's scale);
//! 2. **decisions** — the summary rows sorted by numeric id;
//! 3. **tasks** — the §4/§5 register merged and sorted by numeric id
//!    (the population whose sequential allocation reproduces the
//!    board's `U1`–`U128`), then the §3 blockers with generated ids
//!    after the register (board order — they carry no ids of their
//!    own), then the findings;
//! 4. **edges** — the Gates edges (D-id order).
//!
//! # The conventions
//!
//! - **Status**: the checkbox governs — checked → `closed` with the
//!   fixed literal `imported checked` as evidence, unchecked →
//!   `open`. Strikethrough is edit history on this board, not
//!   resolution (the measured convention: unchecked items carry
//!   struck title fragments), so the strike is ignored. Every task
//!   carries the fixed verify literal.
//! - **Kinds**: §4's items are `task` under `Follow-ups`; §5's items
//!   are `debt` under `Debt`; §3's blockers are `task` under the
//!   synthetic `Blockers` epic — the section is the declaration.
//! - **Epics**: all open — the milestone table carries no State
//!   column and §1's phase prose is not per-milestone, so no
//!   milestone status is derivable, and an epic's status never gates
//!   its tasks (display-only). The milestone epics' descriptions
//!   carry the M id; the synthetics' descriptions carry the
//!   provenance.
//! - **Decisions**: the Status cell governs — `resolved` → `signed`
//!   with the banner's outcome as the resolution and the banner's
//!   date as the node's stamp; `open` → `open`. The banner is the
//!   date/outcome's one home; the footer is the cross-check's pin.
//!   The title is the summary's Decision cell, trailing-trimmed. The
//!   draft mirrors the camera mapping's decision shape
//!   field-for-field: no options (the options prose stays in the
//!   board, §9's cutover), the recommendation carried, no references.
//! - **Confidence**: `low` → `Low`, `high` → `High`, `medium`,
//!   `low-medium` and `medium-high` → `Medium` — the intermediates
//!   sit between the domain's levels, and the middle neither
//!   overstates nor understates.
//! - **Timestamps**: the decisions' resolved dates → midnight;
//!   everything else → `imported_at`.
//! - **Titles**: the parser's title (first-sentence / first-`**`) is
//!   the draft's title verbatim — the parser already applied the
//!   trailing trim.
//! - **Bodies**: the task bodies are not imported — the board's
//!   bodies are its bulk (the register items' prose runs to hundreds
//!   of lines), and the graph is the product, not the prose. The
//!   AST carries them; the drafts do not.
//! - **Edges**: one `DecisionToEpic` Gates edge per summary row whose
//!   Gates cell names a §2 milestone, D-id order.
//! - **Findings**: a Gates cell naming no §2 milestone becomes a
//!   `finding`-kind task under `Follow-ups` (D-id order, the
//!   dossier's body as the assertion's context) rather than a silent
//!   drop. No prose-phrase findings — the parser recognizes no
//!   dependency prose.
//! - **Not imported**: the Gated-on cells, the dossier bodies (a
//!   finding's context excepted), the footer's Applied-to column,
//!   §1/§6+, the README — already absent from the AST by the
//!   parser's design, except the footer's column, which the map
//!   drops.

use std::collections::BTreeMap;

use todo_domain::{
    BlocksEdge, Confidence, DecisionDraft, DecisionId, DecisionStatus, EpicDraft, EpicId,
    ReferenceKind, References, RepoSlug, Status, TaskDraft, TaskId, TaskKind,
};

use super::IkvmBoard;
use super::board::{BlockerItem, MilestoneRow, RegisterItem, clean_title};
use super::decisions::{ConfidenceCell, DecisionRow, DecisionStatusCell, Dossier};
use super::error::IkvmError;
use crate::import::helpers::{HelperError, id_sort_key, midnight, position, references};
use crate::import::plan::{ImportOptions, ImportPlan, PlanOp};

/// The fixed verify literal for every task the board gives no gate of
/// its own: the checkbox alone is the board's closure evidence.
const VERIFY_LITERAL: &str = "imported: the board's checkbox alone";
/// The evidence literal for a checked item.
const IMPORTED_CHECKED: &str = "imported checked";
/// The synthetic `Follow-ups` epic's title.
const FOLLOWUPS_TITLE: &str = "Follow-ups";
/// The synthetic `Debt` epic's title.
const DEBT_TITLE: &str = "Debt";
/// The synthetic `Blockers` epic's title.
const BLOCKERS_TITLE: &str = "Blockers";
/// The `Follow-ups` epic's provenance description.
const FOLLOWUPS_DESCRIPTION: &str = "the ikvm-html board's §4 follow-ups, not tied to a milestone";
/// The `Debt` epic's provenance description — the same requirement and
/// the same reading as [`FOLLOWUPS_DESCRIPTION`]'s; the second clause
/// is the domain's own `debt`-kind vocabulary.
const DEBT_DESCRIPTION: &str = "the ikvm-html board's §5 debt, recorded rather than forgotten";
/// The `Blockers` epic's provenance description — §3's own title is
/// the description's second half.
const BLOCKERS_DESCRIPTION: &str =
    "the ikvm-html board's §3 blockers, needing something other than a decision";
/// A milestone epic's provenance description prefix; the milestone id
/// is appended.
const MILESTONE_DESCRIPTION: &str = "the ikvm-html board's §2 milestone ";

/// Maps a parsed ikvm-html board into an executable plan. Pure: the
/// same board and options yield an equal plan, and nothing is written
/// anywhere — the plan is executed separately by the shared
/// `apply_plan`.
pub fn map_ikvm(board: &IkvmBoard, opts: &ImportOptions) -> Result<ImportPlan, IkvmError> {
    let sorted = SortedBoard::build(board);
    let index = BoardIndex::build(&sorted, &opts.slug);
    let mut ops: Vec<PlanOp> = Vec::new();
    push_epic_ops(&mut ops, &sorted, opts);
    push_decision_ops(&mut ops, &sorted, opts)?;
    push_task_ops(&mut ops, &sorted, &index, opts);
    push_finding_ops(&mut ops, &sorted, &index, opts)?;
    push_edge_ops(&mut ops, &sorted, &index, opts);
    Ok(ImportPlan { ops })
}

/// The board's populations in the plan's own order — the sorted forms
/// the mapping consumes. The AST keeps board order (it is data); the
/// plan's order is derived here, and the sorts are load-bearing: the
/// §4/§5 file order carries inversions against numeric id order.
struct SortedBoard<'a> {
    /// §2's milestone rows, sorted by numeric id.
    milestones: Vec<&'a MilestoneRow>,
    /// The summary rows, sorted by numeric id.
    decisions: Vec<&'a DecisionRow>,
    /// The dossiers, sorted by numeric id — the Gates findings'
    /// bodies.
    dossiers: Vec<&'a Dossier>,
    /// The register items in the plan's task order: §4 and §5 merged,
    /// sorted by numeric id.
    tasks: Vec<SortedItem<'a>>,
    /// §3's blockers, in board order — they carry no ids, so board
    /// order is the only order they have.
    blockers: Vec<&'a BlockerItem>,
}

/// One register item paired with the section it came from — the epic
/// and kind the mapping assigns.
struct SortedItem<'a> {
    /// The item.
    item: &'a RegisterItem,
    /// Whether the item came from §5 (Debt) rather than §4.
    debt: bool,
}

impl<'a> SortedBoard<'a> {
    /// Sorts every population into the plan's order.
    fn build(board: &'a IkvmBoard) -> Self {
        let mut tasks: Vec<SortedItem<'a>> = board
            .todo
            .followups
            .iter()
            .map(|item| SortedItem { item, debt: false })
            .chain(
                board
                    .todo
                    .debt
                    .iter()
                    .map(|item| SortedItem { item, debt: true }),
            )
            .collect();
        tasks.sort_by(|a, b| id_sort_key(&a.item.id).cmp(&id_sort_key(&b.item.id)));
        Self {
            milestones: sorted_refs(&board.todo.milestones, |row| row.id.as_str()),
            decisions: sorted_refs(&board.decisions.summary, |row| row.id.as_str()),
            dossiers: sorted_refs(&board.decisions.dossiers, |dossier| dossier.id.as_str()),
            tasks,
            blockers: board.todo.blockers.iter().collect(),
        }
    }
}

/// A population sorted by its ids' numeric key.
fn sorted_refs<T>(items: &[T], id: impl Fn(&T) -> &str) -> Vec<&T> {
    let mut out: Vec<&T> = items.iter().collect();
    out.sort_by(|a, b| id_sort_key(id(a)).cmp(&id_sort_key(id(b))));
    out
}

/// The board's ids resolved to the ids the plan constructs — the
/// reproduction map. A lookup answering `None` means the board does
/// not carry that id; the caller decides whether that is a finding.
struct BoardIndex {
    /// M id → the milestone epic's constructed id.
    epics: BTreeMap<String, EpicId>,
    /// The synthetic `Follow-ups` epic's constructed id.
    followups: EpicId,
    /// The synthetic `Debt` epic's constructed id.
    debt: EpicId,
    /// The synthetic `Blockers` epic's constructed id.
    blockers: EpicId,
}

impl BoardIndex {
    /// Builds the map from the sorted populations: a kind's i-th item
    /// gets the kind's i-th sequential number, mirroring the store's
    /// allocation.
    fn build(sorted: &SortedBoard<'_>, slug: &RepoSlug) -> Self {
        let epics: BTreeMap<String, EpicId> = sorted
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
        Self {
            epics,
            followups: EpicId::from_nonzero(slug.clone(), position(sorted.milestones.len())),
            debt: EpicId::from_nonzero(slug.clone(), position(sorted.milestones.len() + 1)),
            blockers: EpicId::from_nonzero(slug.clone(), position(sorted.milestones.len() + 2)),
        }
    }
}

/// The epic ops: the milestones (sorted), then the three synthetics —
/// all open (no milestone status is derivable from the table, and an
/// epic's status never gates its tasks).
fn push_epic_ops(ops: &mut Vec<PlanOp>, sorted: &SortedBoard<'_>, opts: &ImportOptions) {
    for (i, row) in sorted.milestones.iter().enumerate() {
        ops.push(PlanOp::AddEpic {
            draft: EpicDraft {
                title: clean_title(&row.title),
                description: format!("{MILESTONE_DESCRIPTION}{}", row.id),
                status: Status::Open,
                references: References::new(),
            },
            id: EpicId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
    // The synthetics are structural containers for open work — open.
    ops.push(PlanOp::AddEpic {
        draft: EpicDraft {
            title: FOLLOWUPS_TITLE.to_owned(),
            description: FOLLOWUPS_DESCRIPTION.to_owned(),
            status: Status::Open,
            references: References::new(),
        },
        id: EpicId::from_nonzero(opts.slug.clone(), position(sorted.milestones.len())),
        at: opts.imported_at.clone(),
    });
    ops.push(PlanOp::AddEpic {
        draft: EpicDraft {
            title: DEBT_TITLE.to_owned(),
            description: DEBT_DESCRIPTION.to_owned(),
            status: Status::Open,
            references: References::new(),
        },
        id: EpicId::from_nonzero(opts.slug.clone(), position(sorted.milestones.len() + 1)),
        at: opts.imported_at.clone(),
    });
    ops.push(PlanOp::AddEpic {
        draft: EpicDraft {
            title: BLOCKERS_TITLE.to_owned(),
            description: BLOCKERS_DESCRIPTION.to_owned(),
            status: Status::Open,
            references: References::new(),
        },
        id: EpicId::from_nonzero(opts.slug.clone(), position(sorted.milestones.len() + 2)),
        at: opts.imported_at.clone(),
    });
}

/// The decision ops: the summary rows (sorted), the Status cell
/// governing status, the banner the date/outcome source, the resolved
/// date as the node's stamp.
fn push_decision_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    opts: &ImportOptions,
) -> Result<(), IkvmError> {
    for (i, row) in sorted.decisions.iter().enumerate() {
        let (status, resolution, at) = match &row.status {
            DecisionStatusCell::Resolved => {
                // The banner is the date/outcome's one home; a
                // resolved row whose dossier carries none is a board
                // inconsistency the cross-checks do not cover, and
                // the mapping refuses it rather than guessing a date.
                let banner = sorted
                    .dossiers
                    .iter()
                    .find(|dossier| dossier.id == row.id)
                    .and_then(|dossier| dossier.resolved.as_ref())
                    .ok_or_else(|| IkvmError::IdSetMismatch {
                        id: row.id.clone(),
                        detail: format!(
                            "{} is resolved in the summary table but its dossier carries no resolution banner",
                            row.id
                        ),
                    })?;
                (
                    DecisionStatus::Signed,
                    Some(banner.outcome.clone()),
                    midnight(&banner.date)?,
                )
            }
            DecisionStatusCell::Open => (DecisionStatus::Open, None, opts.imported_at.clone()),
        };
        ops.push(PlanOp::AddDecision {
            draft: DecisionDraft {
                title: clean_title(&row.title),
                status,
                // The options prose lives in the dossiers and stays in
                // the board (§9's cutover).
                options: Vec::new(),
                recommendation: Some(row.recommendation.clone()),
                resolution,
                confidence: confidence(row.confidence),
                references: References::new(),
            },
            id: DecisionId::from_nonzero(opts.slug.clone(), position(i)),
            at,
        });
    }
    Ok(())
}

/// The Confidence cell's verdict as the domain's enum — the
/// intermediates map to the middle.
fn confidence(cell: ConfidenceCell) -> Confidence {
    match cell {
        ConfidenceCell::Low => Confidence::Low,
        ConfidenceCell::Medium | ConfidenceCell::LowMedium | ConfidenceCell::MediumHigh => {
            Confidence::Medium
        }
        ConfidenceCell::High => Confidence::High,
    }
}

/// The task ops: the register items in the plan's task order, then
/// the §3 blockers with generated ids after the register. The
/// checkbox governs status; the bodies are not imported.
fn push_task_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) {
    for (i, sorted_item) in sorted.tasks.iter().enumerate() {
        let item = sorted_item.item;
        let (status, evidence) = if item.checked {
            (Status::Closed, Some(IMPORTED_CHECKED.to_owned()))
        } else {
            (Status::Open, None)
        };
        ops.push(PlanOp::AddTask {
            draft: TaskDraft {
                epic: if sorted_item.debt {
                    index.debt.clone()
                } else {
                    index.followups.clone()
                },
                kind: if sorted_item.debt {
                    TaskKind::Debt
                } else {
                    TaskKind::Task
                },
                title: item.title.clone(),
                body: String::new(),
                status,
                verify: Some(VERIFY_LITERAL.to_owned()),
                evidence,
                note: None,
                references: References::new(),
                priority: None,
                until: None,
                estimate: None,
            },
            id: TaskId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
    // The blockers carry no original id, so no references entry; their
    // ids are generated after the register's, continuing the store's
    // sequential allocation.
    let base = sorted.tasks.len();
    for (i, blocker) in sorted.blockers.iter().enumerate() {
        let (status, evidence) = if blocker.checked {
            (Status::Closed, Some(IMPORTED_CHECKED.to_owned()))
        } else {
            (Status::Open, None)
        };
        ops.push(PlanOp::AddTask {
            draft: TaskDraft {
                epic: index.blockers.clone(),
                kind: TaskKind::Task,
                title: blocker.title.clone(),
                body: String::new(),
                status,
                verify: Some(VERIFY_LITERAL.to_owned()),
                evidence,
                note: None,
                references: References::new(),
                priority: None,
                until: None,
                estimate: None,
            },
            id: TaskId::from_nonzero(opts.slug.clone(), position(base + i)),
            at: opts.imported_at.clone(),
        });
    }
}

/// The finding ops: one `finding`-kind task per Gates cell naming no
/// §2 milestone, numbered after the blockers — D-id order, under
/// `Follow-ups`, the dossier's body as the assertion's context.
fn push_finding_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), IkvmError> {
    let mut number = sorted.tasks.len() + sorted.blockers.len();
    for row in &sorted.decisions {
        if !index.epics.contains_key(&row.gates) {
            let body = sorted
                .dossiers
                .iter()
                .find(|dossier| dossier.id == row.id)
                .map_or_else(String::new, |dossier| dossier.body.clone());
            ops.push(finding_op(
                opts,
                number,
                index.followups.clone(),
                &row.id,
                "gates",
                &row.gates,
                body,
            )?);
            number += 1;
        }
    }
    Ok(())
}

/// One finding task: parent = `Follow-ups`, title =
/// `<source> gates <target>`, body = the dossier's raw text, open,
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
) -> Result<PlanOp, IkvmError> {
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
            estimate: None,
        },
        id: TaskId::from_nonzero(opts.slug.clone(), position(number)),
        at: opts.imported_at.clone(),
    })
}

/// The edge ops: the Gates edges (D-id order), one per summary row
/// whose Gates cell names a §2 milestone.
fn push_edge_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) {
    for (i, row) in sorted.decisions.iter().enumerate() {
        let from = DecisionId::from_nonzero(opts.slug.clone(), position(i));
        if let Some(to) = index.epics.get(&row.gates) {
            ops.push(PlanOp::LinkBlocks {
                edge: BlocksEdge::DecisionToEpic {
                    from,
                    to: to.clone(),
                },
                at: opts.imported_at.clone(),
            });
        }
    }
}

/// The shared helpers' neutral error into this dialect's own,
/// variant-for-variant — the seam that lets `midnight` and
/// `references` live in the shared module while every message and
/// field stays the dialect's.
impl From<HelperError> for IkvmError {
    fn from(error: HelperError) -> Self {
        match error {
            HelperError::BadDate { date } => IkvmError::BadDate { date },
            HelperError::BadReference { target, detail } => {
                IkvmError::BadReference { target, detail }
            }
        }
    }
}
