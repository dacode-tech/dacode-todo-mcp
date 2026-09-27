//! The task graph of spec §5.1: the `parent` tree, the `blocks` DAG with
//! write-time cycle rejection, `relates` annotations, decision gating,
//! the no-orphans rule and the per-repo id counters of §3.
//!
//! [`Graph`] is the whole value — one repository's nodes, edges and
//! counters — and it is the domain's invariant boundary: nothing that
//! violates §5.1 or the §3 closure discipline is reachable through its
//! public API, and a rejected write leaves it unchanged. Two callers are
//! served, and both are visible in the API: the server, which allocates
//! identity ([`Graph::add_task`] and friends take a draft and return the
//! stored node), and the store's replay, which supplies identity
//! ([`Graph::insert_task`] and friends take a whole node and raise the
//! counter to `max(counter, n)`).
//!
//! # Where the invariants live
//!
//! - **`parent` is a field, not an edge table.** §3 gives [`crate::Task`] an
//!   `epic` and [`crate::Epic`] a `repo`, and those bytes are already pinned;
//!   the tree is enforced over the fields — a task whose epic is not in
//!   the graph cannot enter it, and reparenting is
//!   [`Graph::reparent_task`].
//! - **`blocks` is a DAG.** [`Graph::link_blocks`] rejects a cycle before
//!   it lands, with a deterministic path, and the three legal
//!   endpoint shapes are [`crate::BlocksEdge`]'s three variants.
//! - **`relates` is an annotation.** Undirected, task↔task, stored in one
//!   canonical direction, never traversed, never gating (§5.1).
//! - **No orphans, ever.** [`Graph::delete_epic`] is the only deletion in
//!   the public API: §6 has no `task_delete` verb, so a task leaves the
//!   graph only in its epic's cascade.
//!
//! # Judgment calls this module settles
//!
//! The spec pins the rules; these are the shapes it leaves open, decided
//! here and binding on the callers above:
//!
//! - **(a) Epic delete cascade.** "Its tasks move first or be closed"
//!   reads *closed* as both terminals, `closed` and `wontfix` (§3 calls
//!   `wontfix` a terminal). Qualifying tasks cannot survive their parent
//!   — no orphans — so they are deleted with the epic, edges included,
//!   and their ids are returned. One non-terminal task rejects the whole
//!   write as `orphan_not_allowed` naming the first offender in
//!   ascending id order. Incoming decision→epic edges go with the epic;
//!   the decision node stays.
//! - **(b) `relates` shape.** Undirected, task↔task only, canonical
//!   low-id-first, so `(a, b)` and `(b, a)` are one edge. Forbidden
//!   `blocks` endpoint combinations need no error code at all:
//!   [`crate::BlocksEdge`] has exactly three variants, so the illegal
//!   ones are unrepresentable, and [`crate::BlocksEdge::from_ends`] is
//!   the total translation for an untyped caller.
//! - **(c) No public task deletion.** v1 exposes none; §6 has no verb for
//!   it and every deletion path would have to re-derive the orphan rule.
//! - **(d) Errors reuse the closed §6.1 list.** A node of another
//!   repository is `orphan_not_allowed` (it has no parent here); a
//!   missing endpoint, parent or node is `unknown_id`; a taken id is
//!   `duplicate`; an exhausted per-kind number space is reported as
//!   `duplicate` because §6.1 has no code of its own for it and at
//!   `u64::MAX` every id of that kind is taken.
//! - **(e) Layout.** `state` holds the value and its queries, `edges` the
//!   edge types and their mutations, `drafts` the add-API input shapes,
//!   `nodes` the node mutations, `order` the §5.2 canonical-order
//!   derivation. Every mutation validates on a local before touching
//!   `self`, which is what makes rejection atomic without a rollback
//!   path.

mod drafts;
mod edges;
mod nodes;
mod order;
mod state;

#[cfg(test)]
mod tests;

pub use drafts::{DecisionDraft, DocDraft, EpicDraft, TaskDraft};
pub use edges::{BlockEnd, BlocksEdge};
pub use state::{Counters, Graph};
