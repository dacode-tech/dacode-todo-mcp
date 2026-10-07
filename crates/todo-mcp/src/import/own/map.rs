//! The own board → domain mapping (t54): the dialect AST translated
//! into the shared executable [`ImportPlan`]. The plan types, the
//! executor and its `ApplyError` are the shared plan module's — one
//! of the two code homes the dialects share, the helpers module being
//! the other — and this mapping produces that shape; everything else
//! here is the own dialect's. The own `ImportOptions` carries an
//! empty `doc_paths` map: the shared type requires the field, and
//! this board has no doc register to fill it from — the mapping never
//! consults it.
//!
//! `map_own` is a pure function of `(board, opts)` — no I/O, no
//! clock, no store, `BTreeMap` throughout — so two runs over one
//! board yield equal plans (pinned by test).
//!
//! # The plan's total order
//!
//! 1. **epics** — the §2 milestones in **board order** (the spec's
//!    id-reproduction convention: the M ids in board order, not
//!    sorted — the measured table is already numeric, and the
//!    convention keeps a re-ordered table honest), then the synthetic
//!    `Follow-ups` epic, then the synthetic `Debt` epic;
//! 2. **decisions** — the summary rows sorted by numeric id;
//! 3. **tasks** — the base register items (§4 and §5 merged, ids
//!    `U<digits>` exactly) sorted by numeric id — the population
//!    whose sequential allocation reproduces the board's numbering —
//!    then the oddballs (suffixed and `(original)` ids) sorted by
//!    `(numeric, id)`, then the findings;
//! 4. **edges** — the Gates edges (D-id order).
//!
//! # The conventions
//!
//! - **Status**: the checkbox governs — checked → `closed` with the
//!   fixed literal `imported checked` as evidence, unchecked →
//!   `open`. The strike is redundant confirmation (the parser already
//!   refused a struck unchecked line). The epic's status is the State
//!   cell's verdict (`done <date>` → `closed`, `in progress` →
//!   `in_work`). The decision's status is the Status cell
//!   (`resolved …` → `signed`, `drafted …` → `open`).
//! - **Resolution**: a resolved decision's resolution is the note
//!   after the ` — ` when present, else the option letter when
//!   present, else the fixed literal `imported signed` — the measured
//!   rows carry the note, the option and both together, and the
//!   domain requires a resolution for every signed decision; the
//!   literal is the grammar's fallback for a row carrying neither,
//!   kept for completeness rather than because a measured row
//!   exercises it.
//! - **Timestamps**: the decisions' resolved dates →
//!   `T00:00:00.000000000Z`; everything else → `imported_at`.
//! - **Kinds**: §4's items are `task` under `Follow-ups`; §5's items
//!   are `debt` under `Debt` — the section is the declaration.
//! - **Bodies**: dropped — the empty string, every task. The board
//!   keeps the narrative per §9; the mapping carries no body prose.
//! - **The synthetics**: two, mirroring the board's own section
//!   structure, open, their descriptions carrying the provenance.
//! - **The oddballs**: each carries its original id string in
//!   `references` — §9's signed fallback for non-reproducing items.
//! - **Findings**: a Gates cell naming no §2 milestone becomes a
//!   `finding`-kind task under `Follow-ups` (D-id order, the
//!   dossier's body as the assertion's context) rather than a silent
//!   drop — the camera convention. No prose-phrase findings: the
//!   parser recognizes no dependency prose.
//! - **Titles**: trailing whitespace and one trailing `.` trimmed —
//!   the dacode mapping's title discipline, unchanged. The epic's
//!   title is the Deliverable cell; the decision's title is the
//!   Decision cell, raw (the camera mapping's decision shape).
//! - **Not imported**: the Gated-on cells (the decisions' Gates column
//!   is the edge register), the State and Status prose after the
//!   verdict, the dossier bodies (a Gates finding's context
//!   excepted), §1/§3/§6 — already absent from the AST by the
//!   parser's design.

use std::collections::BTreeMap;

use todo_domain::{
    BlocksEdge, Confidence, DecisionDraft, DecisionId, DecisionStatus, EpicDraft, EpicId,
    ReferenceKind, References, RepoSlug, Status, TaskDraft, TaskId, TaskKind,
};

use super::OwnBoard;
use super::board::{MilestoneRow, MilestoneState, RegisterItem};
use super::decisions::{ConfidenceCell, DecisionRow, DecisionStatusCell, Dossier};
use super::error::OwnError;
use crate::import::helpers::{HelperError, id_sort_key, midnight, position, references};
use crate::import::plan::{ImportOptions, ImportPlan, PlanOp};

/// The fixed verify literal for every task the board gives no gate of
/// its own: the checkbox is the board's closure evidence.
const VERIFY_LITERAL: &str = "imported: the board's checkbox";
/// The evidence literal for a checked item.
const IMPORTED_CHECKED: &str = "imported checked";
/// The resolution literal for a resolved row carrying neither a note
/// nor an option letter.
const IMPORTED_SIGNED: &str = "imported signed";
/// The synthetic `Follow-ups` epic's title.
const FOLLOWUPS_TITLE: &str = "Follow-ups";
/// The synthetic `Debt` epic's title.
const DEBT_TITLE: &str = "Debt";
/// The `Follow-ups` epic's provenance description.
const FOLLOWUPS_DESCRIPTION: &str =
    "the dacode-todo-mcp board's §4 follow-ups, not tied to a milestone";
/// The `Debt` epic's provenance description — the same requirement and
/// the same reading as [`FOLLOWUPS_DESCRIPTION`]'s; the second clause
/// is the domain's own `debt`-kind vocabulary.
const DEBT_DESCRIPTION: &str =
    "the dacode-todo-mcp board's §5 debt, recorded rather than forgotten";
/// A milestone epic's provenance description prefix; the milestone id
/// is appended.
const MILESTONE_DESCRIPTION: &str = "the dacode-todo-mcp board's §2 milestone ";

/// Maps a parsed own board into an executable plan. Pure: the same
/// board and options yield an equal plan, and nothing is written
/// anywhere — the plan is executed separately by the shared
/// `apply_plan`.
pub fn map_own(board: &OwnBoard, opts: &ImportOptions) -> Result<ImportPlan, OwnError> {
    let sorted = SortedBoard::build(board);
    let index = BoardIndex::build(&sorted, &opts.slug);
    let mut ops: Vec<PlanOp> = Vec::new();
    push_epic_ops(&mut ops, &sorted, opts);
    push_decision_ops(&mut ops, &sorted, opts)?;
    push_task_ops(&mut ops, &sorted, &index, opts)?;
    push_finding_ops(&mut ops, &sorted, &index, opts)?;
    push_edge_ops(&mut ops, &sorted, &index, opts);
    Ok(ImportPlan { ops })
}

/// The board's populations in the plan's own order — the sorted forms
/// the mapping consumes. The AST keeps board order (it is data); the
/// plan's order is derived here, and the sorts are load-bearing. The
/// milestones are the exception: they stay in board order (the
/// module doc's convention).
struct SortedBoard<'a> {
    /// §2's milestone rows, in board order.
    milestones: Vec<&'a MilestoneRow>,
    /// The summary rows, sorted by numeric id.
    decisions: Vec<&'a DecisionRow>,
    /// The dossiers, sorted by numeric id — the Gates findings'
    /// bodies.
    dossiers: Vec<&'a Dossier>,
    /// The register items in the plan's task order: the base items
    /// (numeric id order) then the oddballs ((numeric, id) order).
    tasks: Vec<SortedItem<'a>>,
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
    fn build(board: &'a OwnBoard) -> Self {
        let mut base: Vec<SortedItem<'a>> = Vec::new();
        let mut oddballs: Vec<SortedItem<'a>> = Vec::new();
        for sorted in board
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
        {
            if is_base_id(&sorted.item.id) {
                base.push(sorted);
            } else {
                oddballs.push(sorted);
            }
        }
        base.sort_by(|a, b| id_sort_key(&a.item.id).cmp(&id_sort_key(&b.item.id)));
        oddballs.sort_by(|a, b| id_sort_key(&a.item.id).cmp(&id_sort_key(&b.item.id)));
        let mut tasks = base;
        tasks.append(&mut oddballs);
        Self {
            milestones: board.todo.milestones.iter().collect(),
            decisions: sorted_refs(&board.decisions.summary, |row| row.id.as_str()),
            dossiers: sorted_refs(&board.decisions.dossiers, |dossier| dossier.id.as_str()),
            tasks,
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
/// No task index: this dialect's dependency structure is the Gates
/// column alone, so no edge or finding ever resolves a U id.
struct BoardIndex {
    /// M id → the milestone epic's constructed id (board order).
    epics: BTreeMap<String, EpicId>,
    /// The synthetic `Follow-ups` epic's constructed id.
    followups: EpicId,
    /// The synthetic `Debt` epic's constructed id.
    debt: EpicId,
}

impl BoardIndex {
    /// Builds the map from the sorted populations: a kind's i-th item
    /// gets the kind's i-th sequential number, mirroring the store's
    /// allocation. The milestones are indexed by board position —
    /// their convention, not the sort.
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
        }
    }
}

/// The epic ops: the milestones (board order), then the two
/// synthetics.
fn push_epic_ops(ops: &mut Vec<PlanOp>, sorted: &SortedBoard<'_>, opts: &ImportOptions) {
    for (i, row) in sorted.milestones.iter().enumerate() {
        ops.push(PlanOp::AddEpic {
            draft: EpicDraft {
                title: clean_title(&row.title),
                description: format!("{MILESTONE_DESCRIPTION}{}", row.id),
                status: epic_status(row.status),
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
}

/// The State cell's verdict as the epic's status (the mapping's
/// convention).
fn epic_status(state: MilestoneState) -> Status {
    match state {
        MilestoneState::Done => Status::Closed,
        MilestoneState::InProgress => Status::InWork,
    }
}

/// The decision ops: the summary rows (sorted), the Status cell
/// governing status and resolution, the resolved date as the node's
/// stamp.
fn push_decision_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    opts: &ImportOptions,
) -> Result<(), OwnError> {
    for (i, row) in sorted.decisions.iter().enumerate() {
        let (status, resolution, at) = match &row.status {
            DecisionStatusCell::Resolved { option, date, note } => (
                DecisionStatus::Signed,
                Some(resolution_text(option.as_deref(), note.as_deref())),
                midnight(date)?,
            ),
            DecisionStatusCell::Drafted => (DecisionStatus::Open, None, opts.imported_at.clone()),
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
                confidence: confidence(row.confidence),
                references: References::new(),
            },
            id: DecisionId::from_nonzero(opts.slug.clone(), position(i)),
            at,
        });
    }
    Ok(())
}

/// A resolved row's resolution: the note when present, else the
/// option letter, else the fixed literal — the domain requires a
/// resolution for every signed decision, and the literal is the
/// grammar's fallback for a row carrying neither (no measured row
/// does; the grammar admits the shape).
fn resolution_text(option: Option<&str>, note: Option<&str>) -> String {
    if let Some(note) = note {
        return note.to_owned();
    }
    if let Some(option) = option {
        return format!("({option})");
    }
    IMPORTED_SIGNED.to_owned()
}

/// The Confidence cell's verdict as the domain's enum.
fn confidence(cell: ConfidenceCell) -> Confidence {
    match cell {
        ConfidenceCell::Low => Confidence::Low,
        ConfidenceCell::Medium => Confidence::Medium,
        ConfidenceCell::High => Confidence::High,
    }
}

/// The task ops: the register items in the plan's task order — base
/// then oddballs — the checkbox governing status, the body dropped,
/// the oddballs carrying their original ids.
fn push_task_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), OwnError> {
    for (i, sorted_item) in sorted.tasks.iter().enumerate() {
        let item = sorted_item.item;
        let (status, evidence) = if item.checked {
            (Status::Closed, Some(IMPORTED_CHECKED.to_owned()))
        } else {
            (Status::Open, None)
        };
        let entries = if is_base_id(&item.id) {
            Vec::new()
        } else {
            vec![(ReferenceKind::Other, item.id.clone())]
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
                title: clean_title(&item.title),
                body: String::new(),
                status,
                verify: Some(VERIFY_LITERAL.to_owned()),
                evidence,
                note: None,
                references: references(entries)?,
                priority: None,
                until: None,
            },
            id: TaskId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
    Ok(())
}

/// The finding ops: one `finding`-kind task per Gates cell naming no
/// §2 milestone, numbered after the oddballs, D-id order — the camera
/// convention, never a silent drop.
fn push_finding_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), OwnError> {
    let mut number = sorted.tasks.len();
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
                &row.gates,
                body,
            )?);
            number += 1;
        }
    }
    Ok(())
}

/// One finding task: parent = `Follow-ups`, title =
/// `<source> gates <target>`, body = the assertion's context, open,
/// the fixed verify literal, one `other` reference targeting the
/// source id.
fn finding_op(
    opts: &ImportOptions,
    number: usize,
    epic: EpicId,
    source: &str,
    target: &str,
    body: String,
) -> Result<PlanOp, OwnError> {
    Ok(PlanOp::AddTask {
        draft: TaskDraft {
            epic,
            kind: TaskKind::Finding,
            title: format!("{source} gates {target}"),
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

/// The edge ops, in the module doc's order: the Gates edges (D-id
/// order).
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

/// The task or epic title from the board's raw title: trailing
/// whitespace and one trailing `.` trimmed — the dacode mapping's
/// title discipline, unchanged.
fn clean_title(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed.strip_suffix('.').unwrap_or(trimmed).to_owned()
}

/// The shared helpers' neutral error into this dialect's own,
/// variant-for-variant — the seam that lets `midnight` and
/// `references` live in the shared module while every message and
/// field stays the dialect's.
impl From<HelperError> for OwnError {
    fn from(error: HelperError) -> Self {
        match error {
            HelperError::BadDate { date } => OwnError::BadDate { date },
            HelperError::BadReference { target, detail } => {
                OwnError::BadReference { target, detail }
            }
        }
    }
}

/// Whether the register id is a base id — `U<digits>` exactly, the
/// shape that reproduces the board's numbering; everything else
/// (suffixed, `(original)`) is an oddball.
fn is_base_id(id: &str) -> bool {
    let rest = id.strip_prefix('U').unwrap_or("");
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}
