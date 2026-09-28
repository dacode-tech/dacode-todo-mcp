//! The graph value itself (spec §5.1): one repository's nodes, its
//! edges, its id-allocation counters, and the queries over all three.
//!
//! **Single-repo.** A [`Graph`] holds exactly one repository — its
//! [`Repo`] node, that repo's epics, tasks, decisions and docs, the
//! edges between them and the counters that allocate their ids. D4
//! resolved storage to per-repo directories under a shared registry
//! ("not one common database"), §8 locks per repo store and §5.2 scopes
//! ordering to a repo, so the repo is the natural unit of state and of
//! determinism. Multi-repo views (§6.2's `graph_show` with no arguments)
//! are assembled above this crate from one graph per repo; a
//! `references` target naming another repo stays an opaque string, and
//! no edge can cross graphs because every endpoint is looked up here.
//!
//! **`parent` is a field, not an edge table.** §3's signed field lists
//! give [`Task`] an `epic` and [`Epic`] a `repo`, and t6 pinned those
//! bytes; the tree of §5.1 is enforced as an invariant over the fields
//! instead — every task's `epic` names an epic of this graph, every
//! node's `repo` is this graph's slug, and reparenting is a validated
//! field update.
//!
//! Every list this module returns has a documented total order (§8):
//! nodes ascending by id, adjacency in [`BlockEnd`] order, edges by
//! source then by edge. Iteration is over [`BTreeMap`]/[`BTreeSet`]
//! throughout, so no hash order reaches a caller, disk or wire.

use std::collections::{BTreeMap, BTreeSet, btree_map};
use std::num::NonZeroU64;

use crate::error::DomainError;
use crate::id::{DecisionId, DocId, EpicId, RepoSlug, TaskId};
use crate::node::{Decision, Doc, Epic, Repo, Task};

use super::edges::{BlockEnd, BlocksEdge};

/// The per-kind id-allocation counters of §3: epics, tasks, decisions
/// and docs each get per-repo sequential integers, never reused, never
/// renumbered — gaps are meaningful.
///
/// A counter holds the **highest number issued** for its kind, `0` when
/// nothing has been issued yet, so the first number handed out is 1 and
/// an explicit-id insert can raise a counter to `max(counter, n)`
/// without ever re-issuing `n`. Counters only move up: a delete leaves
/// the number burned, which is what makes the gaps meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counters {
    epic: u64,
    task: u64,
    decision: u64,
    doc: u64,
}

/// One counter's three operations: the public read of the highest number
/// issued, the peek at the next one, and the monotonic raise. Generated
/// per kind so the four counters cannot drift apart.
macro_rules! counter {
    ($kind:literal, $field:ident, $read:ident, $next:ident, $raise:ident) => {
        #[doc = concat!(
                                    "The highest ",
                                    $kind,
                                    " number issued; `0` when none has been (§3 allocates from 1)."
                                )]
        pub fn $read(&self) -> u64 {
            self.$field
        }

        #[doc = concat!(
                                    "The next ",
                                    $kind,
                                    " number to issue, without committing it, or `None` when the ",
                                    "`u64` space is exhausted."
                                )]
        pub(super) fn $next(&self) -> Option<NonZeroU64> {
            NonZeroU64::new(self.$field.checked_add(1)?)
        }

        #[doc = concat!(
                                    "Raises the ",
                                    $kind,
                                    " counter to `max(counter, number)` — it never lowers one, so ",
                                    "an explicit-id insert cannot re-issue a live number."
                                )]
        pub(super) fn $raise(&mut self, number: u64) {
            self.$field = self.$field.max(number);
        }
    };
}

impl Counters {
    /// A fresh set: nothing issued for any kind yet.
    pub(super) fn new() -> Self {
        Self {
            epic: 0,
            task: 0,
            decision: 0,
            doc: 0,
        }
    }

    /// The highest number issued per kind, as a settable value — the
    /// snapshot-restore entry point (§3's never-reuse rule: a counter
    /// whose highest-numbered node was deleted cannot be recovered from
    /// the nodes alone). The four arguments are positional in the
    /// per-kind order §3 lists — epic, task, decision, doc — and the
    /// persisted shape that carries them is the store's (§4.1), named
    /// per kind there so a transposition cannot ride this constructor
    /// into a restore.
    pub fn from_highest(epic: u64, task: u64, decision: u64, doc: u64) -> Self {
        Self {
            epic,
            task,
            decision,
            doc,
        }
    }

    counter!("epic", epic, epic, next_epic, raise_epic);
    counter!("task", task, task, next_task, raise_task);
    counter!(
        "decision",
        decision,
        decision,
        next_decision,
        raise_decision
    );
    counter!("doc", doc, doc, next_doc, raise_doc);
}

/// One repository's task graph: the nodes of §3, the edges of §5.1 and
/// the counters of §3's identity rule.
///
/// The graph is the invariant boundary of the domain: a [`Task`] or
/// [`Decision`] that violates the closure discipline, a task whose epic
/// does not exist, a node of another repo, a `blocks` cycle or an
/// orphan is unreachable through this type's public API, because every
/// mutation validates before it touches state.
///
/// Rejected writes are atomic — an `Err` return leaves the graph
/// bit-for-bit unchanged; nothing is applied and rolled back. Timestamps
/// are caller-passed on every mutation (§8: nothing below the binary
/// reads a wall clock).
///
/// No serde derive here: the persisted shape is the store's call (an
/// event log and snapshots, §4.1). `PartialEq`/`Eq` are derived so a
/// replayed graph can be compared structurally against the one that was
/// saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    /// The repo node this graph is scoped to; its slug is the key every
    /// other node must match.
    pub(super) repo: Repo,
    /// Epics by id, ascending.
    pub(super) epics: BTreeMap<EpicId, Epic>,
    /// Tasks by id, ascending.
    pub(super) tasks: BTreeMap<TaskId, Task>,
    /// Decisions by id, ascending.
    pub(super) decisions: BTreeMap<DecisionId, Decision>,
    /// Docs by id, ascending.
    pub(super) docs: BTreeMap<DocId, Doc>,
    /// `blocks` edges indexed by source. Every entry is non-empty — an
    /// emptied one is dropped, so two graphs with the same edges are
    /// `Eq` whatever their history.
    pub(super) blocks_out: BTreeMap<BlockEnd, BTreeSet<BlocksEdge>>,
    /// The same edges indexed by target — the mirror [`Graph::blocked_by`]
    /// reads, kept in step by every edge mutation.
    pub(super) blocks_in: BTreeMap<BlockEnd, BTreeSet<BlocksEdge>>,
    /// `relates` annotations as canonical (lower, higher) task pairs.
    pub(super) relates: BTreeSet<(TaskId, TaskId)>,
    /// The id-allocation counters, part of the state the store persists.
    pub(super) counters: Counters,
}

impl Graph {
    /// A graph for one registered repository: its `Repo` node, no
    /// children, no edges, and counters that will allocate from 1.
    pub fn new(repo: Repo) -> Self {
        Self {
            repo,
            epics: BTreeMap::new(),
            tasks: BTreeMap::new(),
            decisions: BTreeMap::new(),
            docs: BTreeMap::new(),
            blocks_out: BTreeMap::new(),
            blocks_in: BTreeMap::new(),
            relates: BTreeSet::new(),
            counters: Counters::new(),
        }
    }

    /// The repository slug every node of this graph carries.
    pub fn slug(&self) -> &RepoSlug {
        &self.repo.slug
    }

    /// The repo node — its name and path are mutable attributes (§7),
    /// updated by [`Graph::set_repo_name`] and [`Graph::set_repo_path`].
    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    /// The id-allocation counters, for the store to persist (§3).
    pub fn counters(&self) -> Counters {
        self.counters
    }

    /// Raises every counter to `max(current, counters)`; never lowers —
    /// the write half of [`Graph::counters`], and monotonic for the same
    /// reason: §3's allocation never re-issues a number, so a restore
    /// that lowered a counter would. A snapshot restore calls this
    /// *after* inserting its nodes, so the inserts' own raise-to-max has
    /// already run and only the numbers burned above the highest live id
    /// are left to restore.
    pub fn raise_counters_to(&mut self, counters: Counters) {
        self.counters.raise_epic(counters.epic());
        self.counters.raise_task(counters.task());
        self.counters.raise_decision(counters.decision());
        self.counters.raise_doc(counters.doc());
    }

    /// The epic under `id`, if this graph holds one.
    pub fn epic(&self, id: &EpicId) -> Option<&Epic> {
        self.epics.get(id)
    }

    /// The task under `id`, if this graph holds one.
    pub fn task(&self, id: &TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    /// The decision under `id`, if this graph holds one.
    pub fn decision(&self, id: &DecisionId) -> Option<&Decision> {
        self.decisions.get(id)
    }

    /// The doc under `id`, if this graph holds one.
    pub fn doc(&self, id: &DocId) -> Option<&Doc> {
        self.docs.get(id)
    }

    /// Every epic, ascending by id.
    pub fn epics(&self) -> btree_map::Values<'_, EpicId, Epic> {
        self.epics.values()
    }

    /// Every task, ascending by id — creation order, which is §5.2's
    /// last tiebreaker.
    pub fn tasks(&self) -> btree_map::Values<'_, TaskId, Task> {
        self.tasks.values()
    }

    /// Every decision, ascending by id.
    pub fn decisions(&self) -> btree_map::Values<'_, DecisionId, Decision> {
        self.decisions.values()
    }

    /// Every doc, ascending by id.
    pub fn docs(&self) -> btree_map::Values<'_, DocId, Doc> {
        self.docs.values()
    }

    /// The tasks whose parent is `epic`, ascending by id — §6.2 groups
    /// the dump by epic, and §5.1's delete rule reads this set. A linear
    /// scan: the parent is a field on the task, so no index is kept.
    pub fn tasks_of_epic(&self, epic: &EpicId) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| &task.epic == epic)
            .collect()
    }

    /// The `blocks` successors of `node` — what it holds back — in
    /// [`BlockEnd`] order (§6.2's per-node `blocks` list). Empty for a
    /// node with no outgoing edge; a node this graph does not hold is
    /// the caller's error, reported by the mutating APIs as `unknown_id`.
    pub fn blocks(&self, node: &BlockEnd) -> Vec<BlockEnd> {
        match self.blocks_out.get(node) {
            Some(edges) => edges.iter().map(BlocksEdge::target).collect(),
            None => Vec::new(),
        }
    }

    /// The `blocks` predecessors of `node` — what holds it back — in
    /// [`BlockEnd`] order (§6.2's per-node `blocked_by` list). Whether
    /// any of them is still unfinished is a status question, answered by
    /// selection (§5.2), not by this list.
    pub fn blocked_by(&self, node: &BlockEnd) -> Vec<BlockEnd> {
        match self.blocks_in.get(node) {
            Some(edges) => edges.iter().map(BlocksEdge::source).collect(),
            None => Vec::new(),
        }
    }

    /// Every `blocks` edge, ascending by source endpoint and then by the
    /// edge's own order within that source — a total order, since a
    /// source's edges are a sorted set.
    pub fn blocks_edges(&self) -> Vec<BlocksEdge> {
        self.blocks_out.values().flatten().cloned().collect()
    }

    /// Every `relates` annotation as its canonical (lower, higher) task
    /// pair, ascending. The order is §8's, not a semantics: `relates`
    /// never orders anything (§5.1).
    pub fn relates_pairs(&self) -> Vec<(TaskId, TaskId)> {
        self.relates.iter().cloned().collect()
    }

    /// The tasks `task` carries a `relates` annotation with, ascending.
    /// Empty for a task this graph does not hold.
    pub fn relates_of(&self, task: &TaskId) -> Vec<TaskId> {
        self.relates
            .iter()
            .filter(|(a, b)| a == task || b == task)
            .map(|(a, b)| if a == task { b.clone() } else { a.clone() })
            .collect()
    }

    /// Checks that an endpoint names a node of this graph — the
    /// `unknown_id` of §6.1 every edge mutation starts from.
    pub(super) fn require_end(&self, end: &BlockEnd) -> Result<(), DomainError> {
        let known = match end {
            BlockEnd::Task(id) => self.tasks.contains_key(id),
            BlockEnd::Epic(id) => self.epics.contains_key(id),
            BlockEnd::Decision(id) => self.decisions.contains_key(id),
        };
        if known {
            Ok(())
        } else {
            Err(DomainError::UnknownId {
                id: end.to_string(),
            })
        }
    }

    /// [`Graph::require_end`] for the one kind that is never an epic or
    /// a decision endpoint: a task.
    pub(super) fn require_task(&self, id: &TaskId) -> Result<(), DomainError> {
        if self.tasks.contains_key(id) {
            Ok(())
        } else {
            Err(DomainError::UnknownId { id: id.to_string() })
        }
    }

    /// Whether a node of this graph is in scope for a write: §5.1's tree
    /// is rooted at one repo, so a node whose `repo` field — or whose id
    /// carries another slug — belongs to another repository's graph and
    /// would have no parent here.
    pub(super) fn in_scope(&self, repo: &RepoSlug, id_repo: &RepoSlug) -> bool {
        repo == self.slug() && id_repo == self.slug()
    }

    /// Whether this exact `blocks` edge is already stored.
    pub(super) fn has_edge(&self, edge: &BlocksEdge) -> bool {
        let source = edge.source();
        self.blocks_out
            .get(&source)
            .is_some_and(|edges| edges.contains(edge))
    }

    /// Stores an edge in both indexes.
    pub(super) fn insert_edge(&mut self, edge: BlocksEdge) {
        let (source, target) = (edge.source(), edge.target());
        self.blocks_out
            .entry(source)
            .or_default()
            .insert(edge.clone());
        self.blocks_in.entry(target).or_default().insert(edge);
    }

    /// Removes an edge from both indexes, dropping an entry that becomes
    /// empty so no index ever holds an empty set.
    pub(super) fn remove_edge(&mut self, edge: &BlocksEdge) {
        let (source, target) = (edge.source(), edge.target());
        drop_edge(&mut self.blocks_out, &source, edge);
        drop_edge(&mut self.blocks_in, &target, edge);
    }

    /// Removes every edge touching `node`, in either direction — what a
    /// node deletion cascades to.
    pub(super) fn remove_node_edges(&mut self, node: &BlockEnd) {
        for edge in self.touching(node) {
            self.remove_edge(&edge);
        }
    }

    /// The `blocks` path from `from` to `to`, if the existing edges hold
    /// one: a depth-first walk that visits successors in [`BlockEnd`]
    /// order and each node once, so one state and one pair always yield
    /// the same path. `from == to` is not itself a path — the caller
    /// treats a self-edge separately.
    pub(super) fn walk(&self, from: &BlockEnd, to: &BlockEnd) -> Option<Vec<BlockEnd>> {
        let mut path = vec![from.clone()];
        let mut visited = BTreeSet::from([from.clone()]);
        // Frames hold unvisited successors in descending order, so
        // popping one yields the ascending order the walk documents.
        let mut stack: Vec<Vec<BlockEnd>> = vec![self.successors_descending(from)];
        loop {
            let next = {
                let frame = stack.last_mut()?;
                frame.pop()
            };
            let Some(next) = next else {
                stack.pop();
                path.pop();
                continue;
            };
            if &next == to {
                path.push(next);
                return Some(path);
            }
            if visited.insert(next.clone()) {
                path.push(next.clone());
                stack.push(self.successors_descending(&next));
            }
        }
    }

    /// The successors of `node` in descending [`BlockEnd`] order.
    fn successors_descending(&self, node: &BlockEnd) -> Vec<BlockEnd> {
        let mut out = match self.blocks_out.get(node) {
            Some(edges) => edges.iter().map(BlocksEdge::target).collect::<Vec<_>>(),
            None => Vec::new(),
        };
        out.reverse();
        out
    }

    /// Every edge touching `node`, as one owned list so the caller can
    /// mutate while iterating it.
    fn touching(&self, node: &BlockEnd) -> Vec<BlocksEdge> {
        let mut out = match self.blocks_out.get(node) {
            Some(edges) => edges.iter().cloned().collect::<Vec<_>>(),
            None => Vec::new(),
        };
        if let Some(edges) = self.blocks_in.get(node) {
            out.extend(edges.iter().cloned());
        }
        out
    }
}

/// Removes `edge` from one adjacency index, dropping the entry when it
/// becomes empty.
fn drop_edge(
    index: &mut BTreeMap<BlockEnd, BTreeSet<BlocksEdge>>,
    key: &BlockEnd,
    edge: &BlocksEdge,
) {
    let emptied = match index.get_mut(key) {
        Some(edges) => {
            edges.remove(edge);
            edges.is_empty()
        }
        None => return,
    };
    if emptied {
        index.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::drafts::{EpicDraft, TaskDraft};
    use crate::reference::References;
    use crate::status::{Status, TaskKind};
    use crate::time::Timestamp;
    use std::str::FromStr;

    fn slug(s: &str) -> RepoSlug {
        RepoSlug::from_str(s).unwrap()
    }

    fn repo() -> Repo {
        Repo {
            slug: slug("camera"),
            name: "Camera firmware".to_owned(),
            path: "/srv/repos/camera".to_owned(),
            registered_at: Timestamp::parse("2026-09-25T08:00:00Z").unwrap(),
        }
    }

    fn task(n: u64) -> TaskId {
        TaskId::from_str(&format!("camera:U{n}")).unwrap()
    }

    #[test]
    fn graph_starts_empty_scoped_to_its_repo() {
        let graph = Graph::new(repo());
        assert_eq!(graph.slug().as_str(), "camera");
        assert_eq!(graph.repo().path, "/srv/repos/camera");
        assert_eq!(graph.epics().count(), 0);
        assert_eq!(graph.tasks().count(), 0);
        assert_eq!(graph.decisions().count(), 0);
        assert_eq!(graph.docs().count(), 0);
        assert!(graph.blocks_edges().is_empty());
        assert!(graph.relates_pairs().is_empty());
        // Queries on a node the graph does not hold are empty, not a
        // panic: existence is the mutating APIs' business.
        assert!(graph.blocks(&BlockEnd::Task(task(1))).is_empty());
        assert!(graph.blocked_by(&BlockEnd::Task(task(1))).is_empty());
        assert!(graph.relates_of(&task(1)).is_empty());
        assert_eq!(graph.task(&task(1)), None);
    }

    #[test]
    fn graph_counters_start_at_zero_and_report_the_highest_issued() {
        let counters = Graph::new(repo()).counters();
        assert_eq!(counters.epic(), 0);
        assert_eq!(counters.task(), 0);
        assert_eq!(counters.decision(), 0);
        assert_eq!(counters.doc(), 0);
    }

    #[test]
    fn graph_counters_raise_to_max_and_never_lower() {
        let mut counters = Counters::new();
        assert_eq!(counters.next_task(), NonZeroU64::new(1));
        counters.raise_task(7);
        assert_eq!(counters.task(), 7);
        assert_eq!(counters.next_task(), NonZeroU64::new(8));
        // A replayed node below the counter does not move it, so its
        // number is never issued twice.
        counters.raise_task(3);
        assert_eq!(counters.task(), 7);
        // Exhaustion is a None, not a wrap: `u64::MAX` has no successor.
        counters.raise_task(u64::MAX);
        assert_eq!(counters.next_task(), None);
    }

    #[test]
    fn counters_from_highest_pins_each_kind_independently() {
        // Four positional arguments, four distinct values: a
        // transposition cannot pass this test.
        let counters = Counters::from_highest(1, 2, 3, 4);
        assert_eq!(counters.epic(), 1);
        assert_eq!(counters.task(), 2);
        assert_eq!(counters.decision(), 3);
        assert_eq!(counters.doc(), 4);
        // The restore lands as if those numbers had been issued: the next
        // of each kind continues above its restored highest.
        assert_eq!(counters.next_epic(), NonZeroU64::new(2));
        assert_eq!(counters.next_task(), NonZeroU64::new(3));
        assert_eq!(counters.next_decision(), NonZeroU64::new(4));
        assert_eq!(counters.next_doc(), NonZeroU64::new(5));
    }

    #[test]
    fn raise_counters_to_is_monotonic_per_kind() {
        let mut graph = Graph::new(repo());
        graph.raise_counters_to(Counters::from_highest(5, 6, 7, 8));
        assert_eq!(
            (
                graph.counters().epic(),
                graph.counters().task(),
                graph.counters().decision(),
                graph.counters().doc()
            ),
            (5, 6, 7, 8)
        );

        // A smaller set changes nothing — per kind, not all-or-nothing.
        graph.raise_counters_to(Counters::from_highest(9, 1, 7, 0));
        assert_eq!(
            (
                graph.counters().epic(),
                graph.counters().task(),
                graph.counters().decision(),
                graph.counters().doc()
            ),
            (9, 6, 7, 8)
        );
    }

    #[test]
    fn a_burned_number_survives_a_counter_restore() {
        // f5's case at the domain level: an epic cascade burns a task
        // number whose node is gone, so no insert can restore the
        // counter — only the persisted highest can.
        let mut graph = Graph::new(repo());
        let at = Timestamp::parse("2026-09-26T09:00:00Z").unwrap();
        let keep = graph
            .add_epic(EpicDraft {
                title: "keep".to_owned(),
                description: String::new(),
                status: Status::Open,
                references: References::new(),
            })
            .unwrap();
        let doomed = graph
            .add_epic(EpicDraft {
                title: "doomed".to_owned(),
                description: String::new(),
                status: Status::Open,
                references: References::new(),
            })
            .unwrap();
        graph
            .add_task(
                TaskDraft {
                    epic: keep.id.clone(),
                    kind: TaskKind::Task,
                    title: "first".to_owned(),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                at.clone(),
            )
            .unwrap();
        let burned = graph
            .add_task(
                TaskDraft {
                    epic: doomed.id.clone(),
                    kind: TaskKind::Task,
                    title: "second".to_owned(),
                    body: String::new(),
                    status: Status::Closed,
                    verify: Some("cargo test".to_owned()),
                    evidence: Some("ran it".to_owned()),
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                at,
            )
            .unwrap();
        assert_eq!(burned.id, task(2));
        assert_eq!(graph.delete_epic(doomed.id).unwrap(), vec![task(2)]);

        // The state a snapshot would persist: one live task, counters
        // holding both burned numbers.
        let persisted = graph.counters();
        assert_eq!((persisted.epic(), persisted.task()), (2, 2));

        // Restoring the live nodes alone cannot recover the burned
        // number: the raise-to-max of the inserts stops at the highest
        // *live* id…
        let mut restored = Graph::new(graph.repo().clone());
        restored
            .insert_epic(graph.epic(&keep.id).unwrap().clone())
            .unwrap();
        restored
            .insert_task(graph.task(&task(1)).unwrap().clone())
            .unwrap();
        assert_eq!(restored.counters().task(), 1);

        // …so the counter restore is what keeps §3's never-reuse rule,
        // and the next allocation continues above the burned number.
        restored.raise_counters_to(persisted);
        assert_eq!(restored.counters(), graph.counters());
        let next = restored
            .add_task(
                TaskDraft {
                    epic: keep.id,
                    kind: TaskKind::Task,
                    title: "third".to_owned(),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                Timestamp::parse("2026-09-26T10:00:00Z").unwrap(),
            )
            .unwrap();
        assert_eq!(next.id, task(3), "the burned number is never reissued");
    }

    #[test]
    fn graph_walk_finds_the_sorted_first_path() {
        // camera:U1 -> camera:U3, camera:U1 -> camera:U2 -> camera:U3.
        // Both are paths from U1 to U3; the walk visits successors in
        // ascending order, so U2 comes first.
        let mut graph = Graph::new(repo());
        for edge in [
            BlocksEdge::TaskToTask {
                from: task(1),
                to: task(2),
            },
            BlocksEdge::TaskToTask {
                from: task(2),
                to: task(3),
            },
            BlocksEdge::TaskToTask {
                from: task(1),
                to: task(3),
            },
        ] {
            // The nodes do not exist yet, so link through the indexes
            // directly: this test is about the walk, not the link API.
            graph.insert_edge(edge);
        }
        let walk = graph
            .walk(&BlockEnd::Task(task(1)), &BlockEnd::Task(task(3)))
            .unwrap();
        assert_eq!(
            walk,
            vec![
                BlockEnd::Task(task(1)),
                BlockEnd::Task(task(2)),
                BlockEnd::Task(task(3))
            ]
        );
        // The same state walks identically twice.
        assert_eq!(
            graph.walk(&BlockEnd::Task(task(1)), &BlockEnd::Task(task(3))),
            Some(walk)
        );
        // No path the other way, and none to an unconnected node.
        assert_eq!(
            graph.walk(&BlockEnd::Task(task(3)), &BlockEnd::Task(task(1))),
            None
        );
        assert_eq!(
            graph.walk(&BlockEnd::Task(task(1)), &BlockEnd::Task(task(9))),
            None
        );
    }

    #[test]
    fn graph_edge_indexes_stay_mirrored_and_drop_empty_entries() {
        let edge = BlocksEdge::TaskToTask {
            from: task(1),
            to: task(2),
        };
        let mut graph = Graph::new(repo());
        graph.insert_edge(edge.clone());
        assert!(graph.has_edge(&edge));
        assert_eq!(
            graph.blocks(&BlockEnd::Task(task(1))),
            vec![BlockEnd::Task(task(2))]
        );
        assert_eq!(
            graph.blocked_by(&BlockEnd::Task(task(2))),
            vec![BlockEnd::Task(task(1))]
        );

        graph.remove_edge(&edge);
        assert!(!graph.has_edge(&edge));
        // Both indexes are empty again — structurally equal to a graph
        // that never held the edge, which the replay comparison needs.
        assert!(graph.blocks_out.is_empty());
        assert!(graph.blocks_in.is_empty());
        assert_eq!(graph, Graph::new(repo()));
    }

    #[test]
    fn graph_remove_node_edges_clears_both_directions() {
        let mut graph = Graph::new(repo());
        for edge in [
            BlocksEdge::TaskToTask {
                from: task(1),
                to: task(2),
            },
            BlocksEdge::TaskToTask {
                from: task(2),
                to: task(3),
            },
        ] {
            graph.insert_edge(edge);
        }
        graph.remove_node_edges(&BlockEnd::Task(task(2)));
        assert!(graph.blocks_edges().is_empty());
        assert!(graph.blocks_out.is_empty());
        assert!(graph.blocks_in.is_empty());
    }
}
