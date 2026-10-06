//! The dacode board → domain mapping: the frozen dialect AST of
//! `ast.rs` translated into an executable [`ImportPlan`], per the
//! signed mapping conventions (D18 and §9's dacode rows). The
//! id-reproduction contract is the shared `plan.rs`'s module doc; the
//! edge/finding resolution and its readings are `resolve.rs`'s; this
//! one owns the node mapping — the conventions that fill the add ops
//! — and the plan's total order, below.
//!
//! `map_board` is a pure function of `(board, opts)` — no I/O, no
//! clock, no store, `BTreeMap` throughout — so two runs over one
//! board yield equal plans (pinned by test).
//!
//! # The plan's total order
//!
//! 1. **epics** — the §5 milestones sorted by numeric id, then the
//!    synthetic `Follow-ups` epic, then the synthetic `Spec deltas`
//!    epic;
//! 2. **decisions** — the B dossiers sorted by numeric id (the
//!    appendix's file order is not id order, so the sort is
//!    load-bearing, not a no-op);
//! 3. **docs** — the §4 register's own order (the expanded list, then
//!    the milestone table's rows);
//! 4. **tasks** — the §6 U items sorted by numeric id, then the §5
//!    sub-steps sorted by `(milestone, step)`, then the A dossiers
//!    sorted by numeric id, then `B1a`, then the findings;
//! 5. **edges** — the B summary's Gates milestone edges (B-id order),
//!    then the A index's Depends-on edges (A-id order, cell order
//!    within), then the blocked-on edges (item order, marker order
//!    within).
//!
//! # The conventions (D18, operationalized)
//!
//! - **Status**: the checkbox governs closed-ness — `[x]` → `closed`
//!   with the first ✅ marker's line as evidence, else the literal
//!   `imported checked`; an unchecked item takes the first
//!   status-bearing marker in the AST's emission order (`blocked on`
//!   → `blocked`, 🧠 → `deferred`, ▶️ → `in_work`), else `open`. A ✅
//!   on an unchecked item is prose; ⚠️ never affects status. Epics
//!   take the same rule minus the evidence/note fields they do not
//!   carry. Sub-steps inherit the parent milestone's checkbox. A
//!   dossiers: `APPLIED` → `closed` (evidence `APPLIED <date>`, the
//!   outcome appended), `SIGNED OFF` → `open`, `HALF-APPLIED` →
//!   `in_work`.
//! - **Timestamps**: the B and A dossier status dates →
//!   `T00:00:00.000000000Z`; everything else → `imported_at`.
//! - **Edges and findings**: the mechanical-edge rule and its
//!   unresolvable-assertion findings — `resolve.rs`'s module doc.
//! - **B1a**: a closed task under `Follow-ups` referencing `B1`.
//! - **Not imported**: the attention-meta block and §1's narrative —
//!   already absent from the AST by the parser's design.
//!
//! # The readings this file pins
//!
//! Beyond the signed text, each veto-able as a reading and pinned by
//! a test:
//!
//! - **Titles**: the AST title cut at the first marker emoji, then
//!   trailing whitespace and one trailing `.` trimmed. A title that
//!   *begins* with a marker emoji carries a flag, not a tail — the
//!   leading flags are stripped before the cut, because the bare cut
//!   would empty the board's leading-flag titles. An A dossier's span
//!   is `<sections>: <title>`; the colon cut drops the sections
//!   (carried as `spec` references instead) and keeps the title.
//! - **A Gates doc key with no register entry** keeps the raw key as
//!   the `doc` reference's target: the domain's references may cite
//!   not-yet-registered addresses, and the citation is preserved
//!   rather than dropped.
//! - **Reference order**: the conventions name cell/span order as the
//!   emission order; the domain's `References` set stores its
//!   canonical `(kind, target, note)` order (§8), which is what a
//!   consumer observes.
//! - **Degenerate shapes** (an empty sub-step Gate cell, an empty
//!   dossier outcome) are the domain's to reject at apply — the
//!   mapping does not duplicate the closure discipline.
//!
//! Judgment calls bound here: the fixed verify literal and the
//! `APPLIED <date>` evidence joiner (`": "`).
//!
//! Over the file-size target by recorded exception (`_rust.md`, board
//! §5/U29): the node mapping's populations, reproduction index and
//! draft builders read the same sorted forms, so splitting further
//! would publish them as shared internals for a modest size win. The
//! edge/finding resolution already split out to `resolve.rs` (the
//! natural seam); the tail's pure text/status helpers moved to the
//! shared `crate::import::helpers` (the U31 lift).

use std::collections::BTreeMap;

use todo_domain::{
    Confidence, DecisionDraft, DecisionId, DecisionStatus, DocDraft, DocId, EpicDraft, EpicId,
    ReferenceKind, References, RepoSlug, Status, TaskDraft, TaskId, TaskKind,
};

use super::ast::{
    ADossier, ADossierKind, AIndexRow, BDossier, BSummaryRow, DacodeBoard, DocDepth, DocEntry,
    Marker, MilestoneItem, SubStep, UItem,
};
use super::error::ImportError;
use super::resolve::{push_edge_ops, push_finding_ops};
use crate::import::helpers::{HelperError, id_sort_key, midnight, position, references};
use crate::import::plan::{ImportOptions, ImportPlan, PlanOp};

/// The fixed verify literal for every task the board gives no gate
/// of its own: the checkbox and discharge marker are the board's
/// closure evidence.
pub(super) const VERIFY_LITERAL: &str = "imported: the board's checkbox and discharge marker";
/// The evidence literal for a checked item with no ✅ marker.
const IMPORTED_CHECKED: &str = "imported checked";
/// The marker emojis, in the parser's declaration order — the title
/// cut and the leading-flag strip both scan these.
const MARKER_EMOJIS: [&str; 4] = ["✅", "▶️", "⚠️", "🧠"];
/// The synthetic `Follow-ups` epic's title.
const FOLLOWUPS_TITLE: &str = "Follow-ups";
/// The synthetic `Spec deltas` epic's title.
const DELTAS_TITLE: &str = "Spec deltas";
/// The `Follow-ups` epic's provenance description. D18 requires the
/// synthetic epics' descriptions to carry their provenance; the
/// specific wording is this mapping's own reading, pinned by the
/// draft-field test.
const FOLLOWUPS_DESCRIPTION: &str =
    "the dacode board's §6 open follow-ups, not tied to a milestone";
/// The `Spec deltas` epic's provenance description — the same
/// requirement and the same reading as
/// [`FOLLOWUPS_DESCRIPTION`]'s.
const DELTAS_DESCRIPTION: &str = "the dacode board's A-register design deltas";
/// A milestone epic's provenance description prefix; the milestone id
/// is appended.
const MILESTONE_DESCRIPTION: &str = "the dacode board's §5 milestone ";

/// Maps a parsed dacode board into an executable plan. Pure: the same
/// board and options yield an equal plan, and nothing is written
/// anywhere — the plan is executed separately by
/// [`crate::import::plan::apply_plan`].
pub fn map_board(board: &DacodeBoard, opts: &ImportOptions) -> Result<ImportPlan, ImportError> {
    let sorted = SortedBoard::build(board);
    let docs = validated_docs(&sorted.docs, opts)?;
    let index = BoardIndex::build(&sorted, &opts.slug);
    let mut ops: Vec<PlanOp> = Vec::new();
    push_epic_ops(&mut ops, &sorted, opts);
    push_decision_ops(&mut ops, &sorted, &index, opts)?;
    push_doc_ops(&mut ops, &docs, opts);
    push_u_task_ops(&mut ops, &sorted, &index, opts);
    push_substep_ops(&mut ops, &sorted, &index, opts)?;
    push_a_task_ops(&mut ops, &sorted, &index, opts)?;
    let b1a_position = sorted.u_items.len() + sorted.substeps.len() + sorted.a_dossiers.len();
    push_b1a_op(&mut ops, &board.todo.b1a, &index, opts, b1a_position)?;
    push_finding_ops(&mut ops, &sorted, &index, opts, b1a_position)?;
    push_edge_ops(&mut ops, &sorted, &index, opts);
    Ok(ImportPlan { ops })
}

/// The board's populations in the plan's own order — the sorted forms
/// the mapping consumes. The AST keeps board order (it is data); the
/// plan's order is derived here, and the sorts are load-bearing: the
/// board's §6 order, B dossier order and A dossier order each carry
/// inversions against numeric id order.
pub(super) struct SortedBoard<'a> {
    /// §5's milestone items, sorted by numeric id.
    pub(super) milestones: Vec<&'a MilestoneItem>,
    /// §6's U items, flattened across subsections, sorted by numeric
    /// id.
    pub(super) u_items: Vec<&'a UItem>,
    /// §5's sub-steps, sorted by `(milestone, step)`.
    pub(super) substeps: Vec<&'a SubStep>,
    /// The B dossiers, sorted by numeric id.
    pub(super) b_dossiers: Vec<&'a BDossier>,
    /// The A dossiers, sorted by numeric id.
    pub(super) a_dossiers: Vec<&'a ADossier>,
    /// The B summary rows, sorted by numeric id — the Gates cells'
    /// home.
    pub(super) b_summary: Vec<&'a BSummaryRow>,
    /// The A index rows, sorted by numeric id — the Depends-on cells'
    /// home.
    pub(super) a_index: Vec<&'a AIndexRow>,
    /// §4's doc register, in its own order — the register order is the
    /// docs' plan order.
    pub(super) docs: Vec<&'a DocEntry>,
}

impl<'a> SortedBoard<'a> {
    /// Sorts every population into the plan's order.
    fn build(board: &'a DacodeBoard) -> Self {
        let u_items: Vec<&UItem> = {
            let mut out = board
                .todo
                .followups
                .iter()
                .flat_map(|subsection| subsection.items.iter())
                .collect::<Vec<_>>();
            out.sort_by(|a, b| id_sort_key(&a.id).cmp(&id_sort_key(&b.id)));
            out
        };
        let substeps: Vec<&SubStep> = {
            let mut out = board.todo.substeps.iter().collect::<Vec<_>>();
            out.sort_by(|a, b| substep_sort_key(&a.id).cmp(&substep_sort_key(&b.id)));
            out
        };
        Self {
            milestones: sorted(&board.todo.milestones, |item| item.id.as_str()),
            u_items,
            substeps,
            b_dossiers: sorted(&board.decisions.dossiers, |dossier| dossier.id.as_str()),
            a_dossiers: sorted(&board.deltas.dossiers, |dossier| dossier.id.as_str()),
            b_summary: sorted(&board.decisions.summary, |row| row.id.as_str()),
            a_index: sorted(&board.deltas.index, |row| row.id.as_str()),
            docs: board.todo.docs.iter().collect(),
        }
    }
}

/// A population sorted by its ids' numeric key — the plan's order for
/// every id-carrying register (the board's own order is preserved in
/// the AST; the plan's is derived).
fn sorted<T>(items: &[T], id: impl Fn(&T) -> &str) -> Vec<&T> {
    let mut out: Vec<&T> = items.iter().collect();
    out.sort_by(|a, b| id_sort_key(id(a)).cmp(&id_sort_key(id(b))));
    out
}

/// The board's ids resolved to the ids the plan constructs — the
/// reproduction map. A lookup answering `None` means the board does
/// not carry that id; the caller decides whether that is a finding or
/// a refusal.
pub(super) struct BoardIndex {
    /// M id → the milestone epic's constructed id.
    pub(super) epics: BTreeMap<String, EpicId>,
    /// B id → the decision's constructed id.
    pub(super) decisions: BTreeMap<String, DecisionId>,
    /// doc key → the doc's constructed id.
    pub(super) docs: BTreeMap<String, DocId>,
    /// U id → the task's constructed id.
    pub(super) u_tasks: BTreeMap<String, TaskId>,
    /// A id → the task's constructed id.
    pub(super) a_tasks: BTreeMap<String, TaskId>,
    /// The synthetic `Follow-ups` epic's constructed id.
    pub(super) followups: EpicId,
    /// The synthetic `Spec deltas` epic's constructed id.
    pub(super) deltas: EpicId,
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
            .map(|(i, item)| {
                (
                    item.id.clone(),
                    EpicId::from_nonzero(slug.clone(), position(i)),
                )
            })
            .collect();
        let decisions: BTreeMap<String, DecisionId> = sorted
            .b_dossiers
            .iter()
            .enumerate()
            .map(|(i, dossier)| {
                (
                    dossier.id.clone(),
                    DecisionId::from_nonzero(slug.clone(), position(i)),
                )
            })
            .collect();
        let docs: BTreeMap<String, DocId> = sorted
            .docs
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                (
                    entry.key.clone(),
                    DocId::from_nonzero(slug.clone(), position(i)),
                )
            })
            .collect();
        let u_tasks: BTreeMap<String, TaskId> = sorted
            .u_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                (
                    item.id.clone(),
                    TaskId::from_nonzero(slug.clone(), position(i)),
                )
            })
            .collect();
        let a_base = sorted.u_items.len() + sorted.substeps.len();
        let a_tasks: BTreeMap<String, TaskId> = sorted
            .a_dossiers
            .iter()
            .enumerate()
            .map(|(i, dossier)| {
                (
                    dossier.id.clone(),
                    TaskId::from_nonzero(slug.clone(), position(a_base + i)),
                )
            })
            .collect();
        Self {
            epics,
            decisions,
            docs,
            u_tasks,
            a_tasks,
            followups: EpicId::from_nonzero(slug.clone(), position(sorted.milestones.len())),
            deltas: EpicId::from_nonzero(slug.clone(), position(sorted.milestones.len() + 1)),
        }
    }
}

/// The register's entries paired with their validated paths — the
/// up-front validation (a doc with no path is an omission, not a
/// skip), so the doc ops never look one up.
fn validated_docs<'a>(
    docs: &[&'a DocEntry],
    opts: &'a ImportOptions,
) -> Result<Vec<(&'a DocEntry, &'a str)>, ImportError> {
    docs.iter()
        .map(|entry| {
            opts.doc_paths
                .get(&entry.key)
                .map(|path| (*entry, path.as_str()))
                .ok_or_else(|| ImportError::MissingDocPath {
                    key: entry.key.clone(),
                })
        })
        .collect()
}

/// The epic ops: the milestones (sorted), then the two synthetics.
fn push_epic_ops(ops: &mut Vec<PlanOp>, sorted: &SortedBoard<'_>, opts: &ImportOptions) {
    for (i, item) in sorted.milestones.iter().enumerate() {
        ops.push(PlanOp::AddEpic {
            draft: EpicDraft {
                title: clean_title(&item.title),
                description: format!("{MILESTONE_DESCRIPTION}{}", item.id),
                status: checkbox_status(item.checked, &item.markers),
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
            title: DELTAS_TITLE.to_owned(),
            description: DELTAS_DESCRIPTION.to_owned(),
            status: Status::Open,
            references: References::new(),
        },
        id: EpicId::from_nonzero(opts.slug.clone(), position(sorted.milestones.len() + 1)),
        at: opts.imported_at.clone(),
    });
}

/// The decision ops: the B dossiers (sorted), each carrying its
/// summary row's Gates doc references. Both dossier kinds resolve —
/// the dossier governs, the §2 summary status is a cross-check view.
fn push_decision_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), ImportError> {
    for (i, dossier) in sorted.b_dossiers.iter().enumerate() {
        let mut entries: Vec<(ReferenceKind, String)> = Vec::new();
        if let Some(row) = sorted.b_summary.iter().find(|row| row.id == dossier.id) {
            for key in &row.gates.docs {
                // A key with no register entry keeps the raw key: the
                // domain's references may cite not-yet-registered
                // addresses, and the citation is preserved.
                let target = index
                    .docs
                    .get(key)
                    .map_or_else(|| key.clone(), |id| id.to_string());
                entries.push((ReferenceKind::Doc, target));
            }
        }
        ops.push(PlanOp::AddDecision {
            draft: DecisionDraft {
                title: dossier.title.trim().to_owned(),
                status: DecisionStatus::Signed,
                options: Vec::new(),
                recommendation: None,
                resolution: Some(dossier.outcome.clone()),
                confidence: Confidence::Medium,
                references: references(entries)?,
            },
            id: DecisionId::from_nonzero(opts.slug.clone(), position(i)),
            at: midnight(&dossier.status.date)?,
        });
    }
    Ok(())
}

/// The doc ops: the register's own order, one doc per key, the depth
/// marker as the version.
fn push_doc_ops(ops: &mut Vec<PlanOp>, docs: &[(&DocEntry, &str)], opts: &ImportOptions) {
    for (i, (entry, path)) in docs.iter().enumerate() {
        ops.push(PlanOp::AddDoc {
            draft: DocDraft {
                title: entry.key.clone(),
                path: path.to_string(),
                version: depth_marker(&entry.depth).to_owned(),
                references: References::new(),
            },
            id: DocId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
}

/// The depth marker's version string, as written on the board.
fn depth_marker(depth: &DocDepth) -> &'static str {
    match depth {
        DocDepth::Expanded => "expanded",
        DocDepth::Summary => "summary",
    }
}

/// The U task ops: the §6 items (sorted) under `Follow-ups`, no
/// references — a U item's id is the reproduced id.
fn push_u_task_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) {
    for (i, item) in sorted.u_items.iter().enumerate() {
        let (status, evidence, note) = task_closure(item.checked, &item.markers);
        ops.push(PlanOp::AddTask {
            draft: TaskDraft {
                epic: index.followups.clone(),
                kind: TaskKind::Task,
                title: clean_title(&item.title),
                body: item.body.clone(),
                status,
                verify: Some(VERIFY_LITERAL.to_owned()),
                evidence,
                note,
                references: References::new(),
                priority: None,
                until: None,
            },
            id: TaskId::from_nonzero(opts.slug.clone(), position(i)),
            at: opts.imported_at.clone(),
        });
    }
}

/// The sub-step ops: gate tasks under their milestone's epic,
/// inheriting the milestone's checkbox — the Gate cell's own marks
/// are not the signal.
fn push_substep_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), ImportError> {
    let base = sorted.u_items.len();
    for (i, step) in sorted.substeps.iter().enumerate() {
        let milestone_id = parent_milestone(&step.id);
        let Some((parent, epic)) = milestone_parent(sorted, index, milestone_id) else {
            return Err(ImportError::UnresolvedParent {
                id: step.id.clone(),
                milestone: milestone_id.to_owned(),
            });
        };
        let (status, evidence) = if parent.checked {
            (
                Status::Closed,
                Some(format!("imported: parent milestone {} complete", parent.id)),
            )
        } else {
            (Status::Open, None)
        };
        ops.push(PlanOp::AddTask {
            draft: TaskDraft {
                epic,
                kind: TaskKind::Gate,
                title: step.id.clone(),
                body: step.text.clone(),
                status,
                verify: Some(step.gate.clone()),
                evidence,
                note: None,
                references: references(vec![(ReferenceKind::Other, step.id.clone())])?,
                priority: None,
                until: None,
            },
            id: TaskId::from_nonzero(opts.slug.clone(), position(base + i)),
            at: opts.imported_at.clone(),
        });
    }
    Ok(())
}

/// The milestone item and its epic's constructed id — a sub-step's
/// parent, when §5 carries the milestone.
fn milestone_parent<'a>(
    sorted: &'a SortedBoard<'a>,
    index: &BoardIndex,
    milestone_id: &str,
) -> Option<(&'a MilestoneItem, EpicId)> {
    let item = sorted
        .milestones
        .iter()
        .copied()
        .find(|item| item.id == milestone_id)?;
    let epic = index.epics.get(milestone_id).cloned()?;
    Some((item, epic))
}

/// The A-delta task ops: the dossiers (sorted) under `Spec deltas`,
/// each carrying its original id and its section list as references.
fn push_a_task_ops(
    ops: &mut Vec<PlanOp>,
    sorted: &SortedBoard<'_>,
    index: &BoardIndex,
    opts: &ImportOptions,
) -> Result<(), ImportError> {
    let base = sorted.u_items.len() + sorted.substeps.len();
    for (i, dossier) in sorted.a_dossiers.iter().enumerate() {
        let (status, evidence) = a_closure(dossier);
        let mut entries = vec![(ReferenceKind::Other, dossier.id.clone())];
        for section in a_sections(&dossier.title) {
            entries.push((ReferenceKind::Spec, section));
        }
        ops.push(PlanOp::AddTask {
            draft: TaskDraft {
                epic: index.deltas.clone(),
                kind: TaskKind::Task,
                title: a_title(&dossier.title),
                body: dossier.body.clone(),
                status,
                verify: Some(VERIFY_LITERAL.to_owned()),
                evidence,
                note: None,
                references: references(entries)?,
                priority: None,
                until: None,
            },
            id: TaskId::from_nonzero(opts.slug.clone(), position(base + i)),
            at: midnight(&dossier.status.date)?,
        });
    }
    Ok(())
}

/// The A dossier's status and evidence: `APPLIED` → closed with the
/// application record (the outcome appended when non-empty), `SIGNED
/// OFF` → open, `HALF-APPLIED` → in work.
fn a_closure(dossier: &ADossier) -> (Status, Option<String>) {
    match dossier.status.kind {
        ADossierKind::Applied => {
            let mut evidence = format!("APPLIED {}", dossier.status.date);
            if !dossier.outcome.is_empty() {
                evidence.push_str(": ");
                evidence.push_str(&dossier.outcome);
            }
            (Status::Closed, Some(evidence))
        }
        ADossierKind::SignedOff => (Status::Open, None),
        ADossierKind::HalfApplied => (Status::InWork, None),
    }
}

/// The B1a op: a closed task under `Follow-ups`, its evidence the
/// paragraph itself, referencing `B1`.
fn push_b1a_op(
    ops: &mut Vec<PlanOp>,
    b1a: &str,
    index: &BoardIndex,
    opts: &ImportOptions,
    position_index: usize,
) -> Result<(), ImportError> {
    ops.push(PlanOp::AddTask {
        draft: TaskDraft {
            epic: index.followups.clone(),
            kind: TaskKind::Task,
            title: "B1a".to_owned(),
            body: b1a.to_owned(),
            status: Status::Closed,
            verify: Some(VERIFY_LITERAL.to_owned()),
            evidence: Some(b1a.to_owned()),
            note: None,
            references: references(vec![(ReferenceKind::Other, "B1".to_owned())])?,
            priority: None,
            until: None,
        },
        id: TaskId::from_nonzero(opts.slug.clone(), position(position_index)),
        at: opts.imported_at.clone(),
    });
    Ok(())
}

/// The status of a checkbox item: the checkbox governs; the first
/// status-bearing marker in emission order refines an open state.
/// Epics take this directly (they carry no evidence or note); tasks
/// add the closure fields through [`task_closure`].
fn checkbox_status(checked: bool, markers: &[Marker]) -> Status {
    if checked {
        return Status::Closed;
    }
    match deciding_marker(markers) {
        Some(Marker::BlockedOn { .. }) => Status::Blocked,
        Some(Marker::Brainstorm { .. }) => Status::Deferred,
        Some(Marker::InWork { .. }) => Status::InWork,
        _ => Status::Open,
    }
}

/// The closure fields a checkbox item's status requires: the evidence
/// of a closed item (the first ✅ marker's line, else the literal) and
/// the note of a blocked or deferred one (the deciding marker's
/// line).
fn task_closure(checked: bool, markers: &[Marker]) -> (Status, Option<String>, Option<String>) {
    if checked {
        let evidence = markers
            .iter()
            .find_map(|marker| match marker {
                Marker::Check { line } => Some(line.clone()),
                _ => None,
            })
            .unwrap_or_else(|| IMPORTED_CHECKED.to_owned());
        return (Status::Closed, Some(evidence), None);
    }
    match deciding_marker(markers) {
        Some(Marker::BlockedOn { line, .. }) => (Status::Blocked, None, Some(line.clone())),
        Some(Marker::Brainstorm { line }) => (Status::Deferred, None, Some(line.clone())),
        Some(Marker::InWork { .. }) => (Status::InWork, None, None),
        _ => (Status::Open, None, None),
    }
}

/// The first status-bearing marker in emission order — the one that
/// decides an unchecked item's open state.
fn deciding_marker(markers: &[Marker]) -> Option<&Marker> {
    markers.iter().find(|marker| {
        matches!(
            marker,
            Marker::BlockedOn { .. } | Marker::Brainstorm { .. } | Marker::InWork { .. }
        )
    })
}

/// The task or epic title from a bold run: leading marker emojis are
/// flags (stripped, with their whitespace), the first remaining
/// marker emoji ends the title, then trailing whitespace and one
/// trailing `.` are trimmed. The bare first-emoji cut would empty the
/// board's leading-flag titles, so the flags are stripped first.
fn clean_title(raw: &str) -> String {
    let mut rest = raw.trim_start();
    while let Some(after) = MARKER_EMOJIS
        .iter()
        .find_map(|emoji| rest.strip_prefix(*emoji))
    {
        rest = after.trim_start();
    }
    let cut = MARKER_EMOJIS
        .iter()
        .filter_map(|emoji| rest.find(emoji))
        .min();
    let title = cut.map_or(rest, |at| &rest[..at]);
    let trimmed = title.trim_end();
    let stripped = trimmed.strip_suffix('.').unwrap_or(trimmed);
    stripped.trim_end().to_owned()
}

/// The A dossier's task title: the span after its first colon — the
/// span is `<sections>: <title>`, and the colon cut drops the sections
/// (carried as `spec` references instead). A span with no colon is
/// all title.
fn a_title(span: &str) -> String {
    let title = span.split_once(':').map_or(span, |(_, after)| after);
    let trimmed = title.trim();
    let stripped = trimmed.strip_suffix('.').unwrap_or(trimmed);
    stripped.trim_end().to_owned()
}

/// The A dossier's section list: the span before its first colon,
/// split on `,`, in span order. A span with no colon contributes
/// none.
fn a_sections(span: &str) -> Vec<String> {
    let before = span.split_once(':').map_or("", |(head, _)| head);
    before
        .split(',')
        .map(str::trim)
        .filter(|section| !section.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The shared helpers' neutral error into this dialect's own,
/// variant-for-variant — the seam that lets `midnight` and
/// `references` live in the shared module while every message and
/// field stays the dialect's.
impl From<HelperError> for ImportError {
    fn from(error: HelperError) -> Self {
        match error {
            HelperError::BadDate { date } => ImportError::BadDate { date },
            HelperError::BadReference { target, detail } => {
                ImportError::BadReference { target, detail }
            }
        }
    }
}

/// The sub-step sort key: `(milestone, step)` — the milestone's
/// numeric id, then the step number, then the id itself.
fn substep_sort_key(id: &str) -> ((u64, &str), u64, &str) {
    let (major, minor) = id.split_once('.').unwrap_or((id, ""));
    (id_sort_key(major), minor.parse().unwrap_or(0), id)
}

/// The milestone a sub-step id names — its `M<n>` major part.
fn parent_milestone(id: &str) -> &str {
    id.split_once('.').map_or(id, |(major, _)| major)
}
