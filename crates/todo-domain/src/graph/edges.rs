//! The typed edges of spec §5.1 — `blocks` (a DAG over tasks plus
//! decision gating) and `relates` (an annotation) — and the mutations
//! that add and remove them.
//!
//! Two shapes make illegal edges unrepresentable instead of merely
//! rejected:
//!
//! - [`BlockEnd`] enumerates the node kinds an edge may touch at all.
//!   Docs are absent, so no edge can reach one (§5.1/D10: decisions and
//!   docs attach to their repo and carry no ordering).
//! - [`BlocksEdge`] enumerates the three endpoint *combinations* D2/D10
//!   allow — task→task, decision→task, decision→epic. A task→epic edge,
//!   an epic source or a decision target cannot be constructed, so the
//!   link API needs no error code for them (§6.1's list is closed, and
//!   `ValueError` is for malformed values, not forbidden topology).
//!   [`BlocksEdge::from_ends`] is the total function a caller with
//!   untyped endpoints — the server translating `task_link` — goes
//!   through; it answers `None` where §5.1 forbids the pair.
//!
//! `relates` is undirected and task↔task only, stored in one canonical
//! direction, so `relates(a, b)` and `relates(b, a)` are the same edge
//! and the pair set has a total order (§8). It has no ordering
//! semantics: no traversal reads it, and selection never will (§5.1).

use std::fmt;

use crate::error::DomainError;
use crate::id::{DecisionId, EpicId, TaskId};

use super::state::Graph;

/// A node that can be an endpoint of a `blocks` edge (§5.1).
///
/// The derived order — task, then epic, then decision, ascending id
/// within a kind — is the total order every adjacency list this crate
/// returns is documented in (§8). Only a task can be both source and
/// target, so a cycle walk never mixes kinds; the mixed order exists for
/// §6.2's per-node `blocks`/`blocked_by` lists.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlockEnd {
    /// A task endpoint: the only kind that is both a source and a target.
    Task(TaskId),
    /// An epic endpoint — the target of a decision's gating edge, never
    /// a source (§5.1; epic-level `blocks` is D2's named follow-up).
    Epic(EpicId),
    /// A decision endpoint — a gating source, never a target (§5.1, D10).
    Decision(DecisionId),
}

impl fmt::Display for BlockEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Task(id) => fmt::Display::fmt(id, f),
            Self::Epic(id) => fmt::Display::fmt(id, f),
            Self::Decision(id) => fmt::Display::fmt(id, f),
        }
    }
}

/// A `blocks` edge of §5.1, in one of the three shapes D2/D10 allow.
///
/// `A blocks B` means B is not actionable until A finishes — `closed`
/// or `wontfix` for a task source, `signed` or `superseded` for a
/// decision one. The derived order is (variant, source, target); the
/// graph's own edge listing sorts by source first (see
/// [`Graph::blocks_edges`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlocksEdge {
    /// A task holding back another task.
    TaskToTask {
        /// The blocking task.
        from: TaskId,
        /// The task that is not actionable until `from` finishes.
        to: TaskId,
    },
    /// A decision gating a task (D10).
    DecisionToTask {
        /// The gating decision.
        from: DecisionId,
        /// The task it holds back.
        to: TaskId,
    },
    /// A decision gating an epic, and so all work under it (D10).
    DecisionToEpic {
        /// The gating decision.
        from: DecisionId,
        /// The epic it holds back.
        to: EpicId,
    },
}

impl BlocksEdge {
    /// The endpoint the edge starts at — the blocker.
    pub fn source(&self) -> BlockEnd {
        match self {
            Self::TaskToTask { from, .. } => BlockEnd::Task(from.clone()),
            Self::DecisionToTask { from, .. } | Self::DecisionToEpic { from, .. } => {
                BlockEnd::Decision(from.clone())
            }
        }
    }

    /// The endpoint the edge points at — the node held back.
    pub fn target(&self) -> BlockEnd {
        match self {
            Self::TaskToTask { to, .. } | Self::DecisionToTask { to, .. } => {
                BlockEnd::Task(to.clone())
            }
            Self::DecisionToEpic { to, .. } => BlockEnd::Epic(to.clone()),
        }
    }

    /// The edge between two endpoints, or `None` when §5.1/D2 forbid the
    /// combination — an epic or doc endpoint as a source, a decision as
    /// a target, or a task blocking an epic. Total, so an untyped caller
    /// (the server translating `task_link`) never has to guess which
    /// shapes exist.
    pub fn from_ends(from: BlockEnd, to: BlockEnd) -> Option<Self> {
        match (from, to) {
            (BlockEnd::Task(from), BlockEnd::Task(to)) => Some(Self::TaskToTask { from, to }),
            (BlockEnd::Decision(from), BlockEnd::Task(to)) => {
                Some(Self::DecisionToTask { from, to })
            }
            (BlockEnd::Decision(from), BlockEnd::Epic(to)) => {
                Some(Self::DecisionToEpic { from, to })
            }
            _ => None,
        }
    }
}

impl fmt::Display for BlocksEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.source(), self.target())
    }
}

impl Graph {
    /// Adds a `blocks` edge, rejecting one that would close a cycle
    /// before it lands (§5.1: cycle detection at write time, not repair
    /// at query time).
    ///
    /// Both endpoints must be nodes of this graph (`unknown_id`, the
    /// source checked first). An edge that is already present is left
    /// alone and returns `Ok` — it cannot create a cycle, and treating
    /// it as one would be wrong, because the existing edge is itself a
    /// path from the source to the target. A rejected write leaves the
    /// graph unchanged.
    ///
    /// The cycle error is deterministic: its `path` is
    /// `[source, target, …, source]`, where the walk from `target` back
    /// to `source` is the depth-first search that visits successors in
    /// [`BlockEnd`] order and visits each node once, so the same state
    /// and the same attempted edge always report the same path. A
    /// self-edge reports the length-one cycle `[x, x]`.
    pub fn link_blocks(&mut self, edge: BlocksEdge) -> Result<(), DomainError> {
        let from = edge.source();
        let to = edge.target();
        self.require_end(&from)?;
        self.require_end(&to)?;
        if self.has_edge(&edge) {
            return Ok(());
        }
        if let Some(path) = self.cycle_path(&from, &to) {
            return Err(DomainError::Cycle { path });
        }
        self.insert_edge(edge);
        Ok(())
    }

    /// Removes a `blocks` edge. Both endpoints must exist (`unknown_id`,
    /// the source checked first); removing an edge that is not there
    /// changes nothing and returns `Ok`.
    ///
    /// Unlinking never touches a node's status: a task that was held
    /// back simply has one fewer predecessor, and a deferred item's
    /// until-targets stop being queryable without anything reopening it
    /// (§5.1, D9).
    pub fn unlink_blocks(&mut self, edge: BlocksEdge) -> Result<(), DomainError> {
        self.require_end(&edge.source())?;
        self.require_end(&edge.target())?;
        self.remove_edge(&edge);
        Ok(())
    }

    /// Adds a `relates` annotation between two tasks, which must both
    /// exist (`unknown_id`, `a` checked before `b`).
    ///
    /// The pair is stored in canonical order, so `(a, b)` and `(b, a)`
    /// are one edge and a repeat is a no-op. A self-annotation is
    /// accepted and inert: `relates` gates nothing, so there is no
    /// cycle to reject and no §6.1 code to reject it with.
    pub fn link_relates(&mut self, a: TaskId, b: TaskId) -> Result<(), DomainError> {
        self.require_task(&a)?;
        self.require_task(&b)?;
        self.relates.insert(canonical_pair(a, b));
        Ok(())
    }

    /// Removes a `relates` annotation; both tasks must exist
    /// (`unknown_id`, `a` checked before `b`), and removing an absent
    /// annotation changes nothing.
    pub fn unlink_relates(&mut self, a: TaskId, b: TaskId) -> Result<(), DomainError> {
        self.require_task(&a)?;
        self.require_task(&b)?;
        self.relates.remove(&canonical_pair(a, b));
        Ok(())
    }

    /// The `path` payload of the `cycle` error for adding
    /// `from -> to`, or `None` when the edge is safe.
    fn cycle_path(&self, from: &BlockEnd, to: &BlockEnd) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string(), from.to_string()]);
        }
        let walk = self.walk(to, from)?;
        let mut path = Vec::with_capacity(walk.len() + 1);
        path.push(from.to_string());
        path.extend(walk.iter().map(ToString::to_string));
        Some(path)
    }
}

/// One undirected `relates` pair, stored low-id-first so both spellings
/// of the same annotation collapse to one entry.
fn canonical_pair(a: TaskId, b: TaskId) -> (TaskId, TaskId) {
    if a <= b { (a, b) } else { (b, a) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn task(n: u64) -> TaskId {
        TaskId::from_str(&format!("camera:U{n}")).unwrap()
    }

    fn epic(n: u64) -> EpicId {
        EpicId::from_str(&format!("camera:E{n}")).unwrap()
    }

    fn decision(n: u64) -> DecisionId {
        DecisionId::from_str(&format!("camera:D{n}")).unwrap()
    }

    #[test]
    fn edge_endpoints_render_and_convert() {
        let edge = BlocksEdge::DecisionToTask {
            from: decision(1),
            to: task(2),
        };
        assert_eq!(edge.source(), BlockEnd::Decision(decision(1)));
        assert_eq!(edge.target(), BlockEnd::Task(task(2)));
        assert_eq!(edge.to_string(), "camera:D1 -> camera:U2");
        assert_eq!(BlockEnd::Epic(epic(3)).to_string(), "camera:E3");

        // Round trip through the untyped endpoints keeps the shape.
        assert_eq!(
            BlocksEdge::from_ends(edge.source(), edge.target()),
            Some(edge)
        );
        let epic_gate = BlocksEdge::DecisionToEpic {
            from: decision(1),
            to: epic(3),
        };
        assert_eq!(
            BlocksEdge::from_ends(epic_gate.source(), epic_gate.target()),
            Some(epic_gate)
        );
        assert_eq!(
            BlocksEdge::from_ends(BlockEnd::Task(task(1)), BlockEnd::Task(task(2))),
            Some(BlocksEdge::TaskToTask {
                from: task(1),
                to: task(2)
            })
        );
    }

    #[test]
    fn edge_forbidden_endpoint_pairs_have_no_edge() {
        // A task or epic source into an epic target, and any decision
        // target, are the shapes D2/D10 leave out.
        let forbidden = [
            (BlockEnd::Task(task(1)), BlockEnd::Epic(epic(2))),
            (BlockEnd::Epic(epic(1)), BlockEnd::Task(task(2))),
            (BlockEnd::Epic(epic(1)), BlockEnd::Epic(epic(2))),
            (BlockEnd::Task(task(1)), BlockEnd::Decision(decision(2))),
            (
                BlockEnd::Decision(decision(1)),
                BlockEnd::Decision(decision(2)),
            ),
            (BlockEnd::Epic(epic(1)), BlockEnd::Decision(decision(2))),
        ];
        for (from, to) in forbidden {
            assert_eq!(
                BlocksEdge::from_ends(from.clone(), to.clone()),
                None,
                "{from} -> {to}"
            );
        }
    }

    #[test]
    fn edge_orders_are_total() {
        // BlockEnd: kind first, then id.
        assert!(BlockEnd::Task(task(9)) < BlockEnd::Epic(epic(1)));
        assert!(BlockEnd::Epic(epic(9)) < BlockEnd::Decision(decision(1)));
        assert!(BlockEnd::Task(task(2)) < BlockEnd::Task(task(10)));
        // BlocksEdge: variant, then source, then target.
        assert!(
            BlocksEdge::TaskToTask {
                from: task(1),
                to: task(9)
            } < BlocksEdge::DecisionToTask {
                from: decision(1),
                to: task(2)
            }
        );
        assert!(
            BlocksEdge::DecisionToTask {
                from: decision(1),
                to: task(2)
            } < BlocksEdge::DecisionToEpic {
                from: decision(1),
                to: epic(2)
            }
        );
    }

    #[test]
    fn relates_pair_is_canonical_in_both_directions() {
        assert_eq!(
            canonical_pair(task(2), task(1)),
            canonical_pair(task(1), task(2))
        );
        assert_eq!(canonical_pair(task(1), task(2)).0, task(1));
        assert_eq!(canonical_pair(task(7), task(7)).0, task(7));
    }
}
