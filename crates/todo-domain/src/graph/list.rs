//! The listing reads (spec §6): `task_list` — the canonical order
//! behind four ANDed filters, epic, status, kind and deferred readiness
//! — and the three thin unfiltered listings, `epic_list`,
//! `decision_list` and `doc_list`.
//!
//! **Listings only.** The answers are bare ids in their documented
//! total orders, never rows, counts or aggregates: §6's tool table
//! gives `task_list` filters and no payload beyond the listing, a
//! query result is not persisted, and the consumer owns the wire shape
//! — M3 resolves an id to its node through [`Graph::task`] and friends
//! and declares the tools' schemas, and nothing in this module
//! pre-empts them. The repo dimension of §6's `task_list(repo?, epic?,
//! …)` is not here either: a [`Graph`] is single-repo (see `state`),
//! so the epic filter is the only scope selector and the repo
//! parameter is the server's composition over one graph per repo.
//!
//! # The rules
//!
//! `task_list` walks the scope's canonical order —
//! [`Graph::canonical_order`] with no epic filter,
//! [`Graph::canonical_order_of_epic`] (the induced order, the same
//! list [`Graph::next_task_of_epic`] and [`Graph::next_items_of_epic`]
//! walk) with one — and keeps the ids passing the filters, **in that
//! order**. It never sorts and never re-derives: the output is a
//! filtered subsequence of the list §5.2 pins, which is what makes
//! "every returned list carries the documented total order of §5.2" a
//! property of the pipeline's shape rather than a promise its steps
//! keep.
//!
//! The four filters AND, and each is total — a value comparison, never
//! an error — so an unsatisfiable combination answers an empty list,
//! not an error: §6.1's code list is closed and holds no code for "no
//! matches". `ready` is deferred-scoped (judgment call (g)): it keeps
//! a task iff the task's status is `deferred` and its
//! until-satisfaction equals the filter's boolean, and it never keeps
//! a non-deferred task. The readiness predicate is
//! [`Graph::is_until_satisfied`] — the same implementation the
//! deferred summary's `ready` count uses — so the listing and the
//! summary cannot drift about what "deferred" or "ready" means: for
//! any scope, `task_list(status=deferred)`'s length equals the scope
//! summary's `total`, and its `ready=true`/`ready=false` refinements
//! equal the summary's `ready` and `total - ready`
//! (`list_the_deferred_counts_equal_the_summary` and
//! `list_seeded_graphs_agree_with_an_independent_recomputation` pin
//! the invariant).
//!
//! The three thin listings have no topological order — epics,
//! decisions and docs are not tasks in a `blocks` DAG — so their
//! documented total order is ascending id, which is
//! registration/creation order (§3's counters allocate sequentially
//! from 1 and never renumber). Each collects the corresponding `state`
//! accessor's ascending key order and adds nothing: no filter, because
//! §6 names filters for `task_list` alone, and no re-sort, because the
//! accessor's order is already the documented one.
//!
//! Listing is a pure function of graph state (§8): no new field, no
//! cache, no memoisation — every call recomputes, which is why nothing
//! here can go stale. One `task_list` call costs one order derivation
//! plus one filter pass over it, and each thin listing is one accessor
//! walk — inside §4.1's scale argument.
//!
//! # Judgment calls (t29)
//!
//! - **(a) Placement and wiring.** `impl Graph` methods in this
//!   module, mirroring `select.rs`/`items.rs`; [`TaskFilter`] is
//!   re-exported through `graph/mod.rs` and the crate root the way
//!   [`crate::NextTask`] is. `graph/mod.rs`'s layout sentence, its
//!   judgment call (e), now ends "…`items` the §5.2 batch half, `list`
//!   the §6 listing reads", and the crate-root surface paragraph names
//!   the listings as landed — the domain's M2 query surface is
//!   complete with them, and what remains of M2 is store-side and M3's
//!   wire.
//! - **(b) `TaskFilter` shape.** One struct of four named `Option`
//!   fields, with `Default` as the no-filter case: filters read by
//!   name at the call site, which four positional `Option` arguments
//!   do not, and a builder is more machinery than a four-field value
//!   needs. One method takes it — the epic is one filter among four,
//!   so the order derivation it selects happens inside the walk
//!   instead of a second entry point.
//! - **(c) The shared naive predicates.** The three naive predicates
//!   behind the suites' independent recomputations — previously one
//!   private copy per suite, byte-identical between them — live once
//!   in the parent test module, and every suite reaches them through
//!   its existing glob import. The move is test-internal: independence
//!   from *production* is the property the recomputations exist for,
//!   and sharing inside the test tree preserves it while ending the
//!   per-suite duplication. The donor suites' diffs are pure
//!   deletions, and the lift's behavior preservation is pinned by
//!   their filtered gate counts, which must not move.
//! - **(d) The thin listings.** Collect-from-accessor, ascending id,
//!   no re-sort and no filter — a decision-status or epic-status
//!   filter would pre-build a surface §6 does not name. The methods
//!   exist rather than M3 calling [`Graph::epics`] itself because the
//!   total order is a domain contract (§6: every returned list carries
//!   one): it is documented here and pinned by this crate's tests
//!   (`list_epic_list_is_ascending_id_and_history_free` and its two
//!   siblings), not left to a server layer's iteration accident.
//! - **(e) Totality and the empty answer.** Every filter is total. The
//!   status and kind comparisons read a node the order guaranteed
//!   exists — both derivations emit only ids of tasks this graph holds
//!   — and [`Graph::is_until_satisfied`] is itself total, so the one
//!   resolution seam is [`Graph::task`] over an id from the order: it
//!   answers `Some` for every id the walk produces, and its `None` arm
//!   drops the id rather than inventing a row. That is the whole
//!   state-space argument this module has. An unsatisfiable or
//!   contradictory filter is an `Ok` empty list
//!   (`list_a_contradictory_filter_is_empty_not_an_error`); the only
//!   `Err` is the epic filter naming an epic this graph does not hold.
//! - **(f) Test layout.** `graph/tests/list.rs`, every name prefixed
//!   `list_`, wired as `mod list;`. The verify filter is the module
//!   path `graph::tests::list`, not the bare word `list`: the bare
//!   word also matches the edges suite's
//!   `graph_edge_listings_are_totally_ordered_and_repeatable`, and an
//!   over-selected count cannot be read against this suite's test
//!   count.
//! - **(g) `ready` is deferred-scoped — a reading of §6's tool line,
//!   veto-able.** "For deferred items — until-satisfied (`ready`)"
//!   reads as scoping `ready` to *deferred* tasks: `Some(b)` keeps a
//!   task iff its status is `deferred` and its until-satisfaction is
//!   `b`, matching §5.2's deferred summary, where `ready` is
//!   intrinsically a sub-count of `total`. The veto-able alternative
//!   filters by until-satisfaction regardless of status — under it,
//!   `task_list(ready=true)` alone also returns non-deferred
//!   until-satisfied tasks, and the listing/summary agreement above
//!   holds only when the caller redundantly passes `status=deferred`.
//!   The reading taken makes `ready` self-contained;
//!   `list_ready_is_deferred_scoped` pins it (a non-deferred
//!   until-satisfied task is *not* returned), so a veto is a named
//!   diff to `deferred_ready`, the `ready` arm's predicate.

use crate::error::DomainError;
use crate::id::{DecisionId, DocId, EpicId, TaskId};
use crate::node::Task;
use crate::status::{Status, TaskKind};

use super::state::Graph;

/// The filters of a [`Graph::task_list`] query (§6): four optional
/// predicates, ANDed. `TaskFilter::default()` narrows nothing and lists
/// the whole scope in canonical order.
///
/// No serde here: a query input is not persisted, and the wire spelling
/// — the params struct and its strictness, the schemas and D14's key
/// order — is M3's (see the module docs).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskFilter {
    /// Scope: only this epic's tasks, in the epic's *induced* canonical
    /// order ([`Graph::canonical_order_of_epic`] — the same list
    /// [`Graph::next_task_of_epic`] walks, not the repo order
    /// restricted to the epic). `None` lists the whole repository. An
    /// epic this graph does not hold is the query's only failure,
    /// `unknown_id`.
    pub epic: Option<EpicId>,
    /// Only tasks at this status. `None` applies no status narrowing —
    /// terminal tasks list like any other, because the canonical order
    /// does not drop them and this listing re-derives nothing.
    pub status: Option<Status>,
    /// Only tasks of this kind. `None` applies no kind narrowing.
    pub kind: Option<TaskKind>,
    /// Deferred-scoped readiness (module docs, judgment call (g)):
    /// `Some(true)` keeps the deferred tasks whose every predecessor —
    /// their own until-target edges and their epic's decision gates — is
    /// finished ([`Graph::is_until_satisfied`], the predicate the
    /// deferred summary's `ready` count uses), `Some(false)` the
    /// deferred tasks it holds back, and `None` applies no readiness
    /// narrowing. A non-deferred task never passes `Some(_)`, whatever
    /// its edges say, so `ready` combined with a non-deferred `status`
    /// is an empty answer, not an error.
    pub ready: Option<bool>,
}

impl Graph {
    /// The tasks of this repository matching `filter` (§6's
    /// `task_list`), as ids in §5.2 canonical order:
    /// [`Graph::canonical_order`] with no epic filter,
    /// [`Graph::canonical_order_of_epic`] — the induced order — with
    /// one. The answer is the scope's canonical order filtered, never
    /// re-sorted.
    ///
    /// The filters AND and each is total; a combination that matches
    /// nothing — a contradictory one like a non-deferred `status` with
    /// `ready` included — is an empty list, not an error (§6.1's code
    /// list is closed). The only failure is `unknown_id` for an epic
    /// this graph does not hold, propagated from the order derivation.
    ///
    /// Reads graph state alone (§8), so the same state and the same
    /// filter always yield the same list.
    pub fn task_list(&self, filter: &TaskFilter) -> Result<Vec<TaskId>, DomainError> {
        let order = match &filter.epic {
            Some(epic) => self.canonical_order_of_epic(epic)?,
            None => self.canonical_order(),
        };
        Ok(order
            .into_iter()
            .filter(|id| self.matches_filter(id, filter))
            .collect())
    }

    /// Every epic id, ascending — registration order (§3's counters
    /// allocate sequentially and never renumber), and this listing's
    /// documented total order: epics are not tasks in a `blocks` DAG
    /// and have no topological one. Unfiltered — §6 names filters for
    /// [`Graph::task_list`] alone — and infallible.
    pub fn epic_list(&self) -> Vec<EpicId> {
        self.epics().map(|epic| epic.id.clone()).collect()
    }

    /// Every decision id, ascending — creation order, and this
    /// listing's documented total order ([`Graph::epic_list`]'s
    /// reasoning). Unfiltered: a decision-status filter would be a
    /// surface §6 does not name. Infallible.
    pub fn decision_list(&self) -> Vec<DecisionId> {
        self.decisions()
            .map(|decision| decision.id.clone())
            .collect()
    }

    /// Every doc id, ascending — registration order, and this listing's
    /// documented total order ([`Graph::epic_list`]'s reasoning).
    /// Unfiltered and infallible.
    pub fn doc_list(&self) -> Vec<DocId> {
        self.docs().map(|doc| doc.id.clone()).collect()
    }

    /// One id of the scope's order against the three value filters; the
    /// epic half is the order derivation's own scope, not a predicate
    /// here. The `None` arm is unreachable — both derivations emit only
    /// ids of tasks this graph holds (judgment call (e)) — and drops
    /// the id rather than inventing a row.
    fn matches_filter(&self, id: &TaskId, filter: &TaskFilter) -> bool {
        self.task(id).is_some_and(|task| {
            filter.status.is_none_or(|status| task.status == status)
                && filter.kind.is_none_or(|kind| task.kind == kind)
                && filter
                    .ready
                    .is_none_or(|ready| self.deferred_ready(task, ready))
        })
    }

    /// The `ready` predicate against one filter value: the task must be
    /// deferred, and its until-satisfaction must equal `ready` —
    /// reading (ii), judgment call (g). A non-deferred task answers
    /// `false` under either filter value, whatever its edges say, so
    /// `ready=Some(false)` lists the deferred-*held* tasks and never
    /// the whole rest of the graph. The readiness half is
    /// [`Graph::is_until_satisfied`], the deferred summary's own
    /// predicate.
    fn deferred_ready(&self, task: &Task, ready: bool) -> bool {
        task.status == Status::Deferred && self.is_until_satisfied(&task.id) == ready
    }
}
