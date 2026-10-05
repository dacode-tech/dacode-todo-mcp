//! The executable import plan: the ordered op list the dacode mapping
//! produces, and its executor over the [`Store`] trait.
//!
//! # The plan's total order
//!
//! [`ImportPlan::ops`] is one fixed sequence — the order is the
//! contract that makes the store's sequential allocation reproduce
//! the board's numbering (D18's id-reproduction convention):
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
//! # Id reproduction
//!
//! Each add op carries the id the store **will** allocate, constructed
//! by the mapping from the op's position, because the store allocates
//! per-kind sequential numbers at insert and the plan's order is the
//! add order. Edges reference the constructed ids, so the plan is
//! self-contained: a consumer can inspect or report the ids without
//! touching a store.
//!
//! `apply_plan` enforces the reproduction rather than trusting it:
//! it refuses a store whose graph holds nodes **or** whose allocation
//! counters have burned numbers (a used-then-emptied store — cascade
//! deletions leave the counters raised, §3's never-reuse rule), and
//! during the walk it compares every add's returned id with the op's
//! constructed id, refusing with a named error naming both when they
//! disagree. The comparison catches a slug mismatch at the first add
//! (the store allocates under its own slug), an offset counter that
//! slipped past the up-front check, and a hand-filtered plan whose
//! ops no longer match their positions. Below the comparison, the
//! domain's own checks are the backstop: a task whose epic names an
//! id the graph does not hold is `insert_task`'s parent check —
//! §6.1's `unknown_id` naming the epic id — and an edge endpoint
//! that never landed is the same code from `link_blocks`.
//!
//! Degenerate board shapes the mapping cannot turn into valid drafts
//! (an empty sub-step Gate cell, an empty dossier outcome) are left to
//! the domain's closure discipline: they surface as the store's
//! `missing_*` errors at apply, after the earlier ops have landed.
//! The mapping does not duplicate the domain's validation — the
//! closure rules have one home.

use std::collections::BTreeMap;

use todo_domain::{
    BlocksEdge, DecisionDraft, DecisionId, DocDraft, DocId, EpicDraft, EpicId, RepoSlug, TaskDraft,
    TaskId, Timestamp,
};
use todo_store::{Store, StoreError};

use super::error::ImportError;

/// The mapping's caller-supplied parameters: the slug the plan's ids
/// are constructed under, the instant stamped on every op the board
/// carries no date for, and the doc register's file paths.
///
/// The three board files do not carry the doc filenames, so the
/// caller that knows the repo tree supplies `doc_paths`. Every
/// register key must have a path — a doc with no path is an omission,
/// not a skip — while extra keys are ignored: the register governs,
/// and the tree legitimately holds unregistered files (`TODO.md`
/// itself).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOptions {
    /// The repository slug the plan's constructed ids carry.
    pub slug: RepoSlug,
    /// The created-stamp for every op with no board date (§8: no
    /// clock below the binary layer — the caller passes the instant).
    pub imported_at: Timestamp,
    /// Each doc-register key mapped to its repo-relative path.
    pub doc_paths: BTreeMap<String, String>,
}

/// The executable plan: the ops in the module doc's total order, each
/// carrying the id the store will allocate for it.
///
/// `PartialEq` is derived so the twice-map determinism pin is a plain
/// `assert_eq!` — the same reason the dialect AST derives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPlan {
    /// The ops, in the documented order.
    pub ops: Vec<PlanOp>,
}

/// One executable step of the plan. The `id` fields are the ids the
/// store will allocate — the reproduction contract, not an input the
/// executor passes (the store allocates its own); edges reference
/// them, and a consumer can read them off the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanOp {
    /// Adds an epic. `at` is the audit instant — an epic carries no
    /// timestamps of its own.
    AddEpic {
        /// The epic's fields.
        draft: EpicDraft,
        /// The id the store will allocate.
        id: EpicId,
        /// The audit instant.
        at: Timestamp,
    },
    /// Adds a decision; `at` stamps both of the node's timestamps.
    AddDecision {
        /// The decision's fields.
        draft: DecisionDraft,
        /// The id the store will allocate.
        id: DecisionId,
        /// The node's created/updated stamp.
        at: Timestamp,
    },
    /// Registers a doc; `at` stamps both of the node's timestamps.
    AddDoc {
        /// The doc's fields.
        draft: DocDraft,
        /// The id the store will allocate.
        id: DocId,
        /// The node's created/updated stamp.
        at: Timestamp,
    },
    /// Adds a task; `at` stamps both of the node's timestamps.
    AddTask {
        /// The task's fields.
        draft: TaskDraft,
        /// The id the store will allocate.
        id: TaskId,
        /// The node's created/updated stamp.
        at: Timestamp,
    },
    /// Adds a `blocks` edge; `at` is the audit instant.
    LinkBlocks {
        /// The edge, referencing the plan's constructed ids.
        edge: BlocksEdge,
        /// The audit instant.
        at: Timestamp,
    },
}

/// Walks the plan over a store, op by op, in the plan's order.
///
/// Refuses up front — before anything is written, because apply is
/// not transactional — a store whose graph holds any node (epic,
/// task, decision or doc) **or** whose allocation counters have burned
/// numbers: importing into a used store would corrupt the id
/// reproduction, because the store's allocation would continue past
/// the numbers the plan's ids predict.
///
/// During the walk, every add's returned id is compared with the op's
/// constructed id; a disagreement is a named error naming both (the
/// module doc's list of what that catches). Store errors otherwise
/// propagate unchanged: a domain rejection (the closure discipline,
/// an unknown parent, a cycle) is atomic per op but not per plan —
/// the ops before the failing one have landed, and the operator
/// resolves the half-imported store by removing its directory and
/// fixing the board. There is no transaction across the [`Store`]
/// trait and none is invented here.
pub fn apply_plan(plan: &ImportPlan, store: &mut dyn Store) -> Result<(), StoreError> {
    let graph = store.graph();
    let counters = graph.counters();
    let held = graph.epics().count()
        + graph.tasks().count()
        + graph.decisions().count()
        + graph.docs().count();
    if held > 0
        || counters.epic() > 0
        || counters.task() > 0
        || counters.decision() > 0
        || counters.doc() > 0
    {
        return Err(refusal(ImportError::StoreNotEmpty {
            slug: graph.slug().to_string(),
        }));
    }
    for op in &plan.ops {
        match op {
            PlanOp::AddEpic { draft, id, at } => {
                let added = store.add_epic(draft.clone(), at.clone())?;
                check_id(&added.id, id)?;
            }
            PlanOp::AddDecision { draft, id, at } => {
                let added = store.add_decision(draft.clone(), at.clone())?;
                check_id(&added.id, id)?;
            }
            PlanOp::AddDoc { draft, id, at } => {
                let added = store.add_doc(draft.clone(), at.clone())?;
                check_id(&added.id, id)?;
            }
            PlanOp::AddTask { draft, id, at } => {
                let added = store.add_task(draft.clone(), at.clone())?;
                check_id(&added.id, id)?;
            }
            PlanOp::LinkBlocks { edge, at } => {
                store.link_blocks(edge.clone(), at.clone())?;
            }
        }
    }
    Ok(())
}

/// An apply-time refusal as the store's `corrupt_store`: §4 makes
/// corruption one named error whether the fault is in the bytes or in
/// what they contradict, and a store contradicting the import's
/// preconditions is that family. Built here, at the refusal sites —
/// the parse variants never travel this path, and which §6.1 code a
/// refusal should reach a client under is the import tool's mapping
/// to settle.
fn refusal(error: ImportError) -> StoreError {
    StoreError::CorruptStore {
        offset: None,
        detail: error.to_string(),
    }
}

/// The reproduction check: the id the store allocated for one add
/// must equal the id the plan constructed for it. The four id types
/// compare and render the same way, so one check serves all four.
fn check_id<T: PartialEq + std::fmt::Display>(found: &T, expected: &T) -> Result<(), StoreError> {
    if found == expected {
        return Ok(());
    }
    Err(refusal(ImportError::IdMismatch {
        expected: expected.to_string(),
        found: found.to_string(),
    }))
}
