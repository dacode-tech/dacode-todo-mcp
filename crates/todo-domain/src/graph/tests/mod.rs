//! The graph tests (spec §10): shared builders, the invariant scan the
//! property test re-runs after every mutation, the byte fingerprint
//! that makes "same state twice" checkable, and the naive predicates
//! the suites' independent recomputations share.
//!
//! The scan is deliberately written against the public accessors only,
//! with its own cycle detection, so it cannot share a bug with the code
//! that maintains the state it checks.

use std::collections::BTreeSet;
use std::str::FromStr;

use crate::node::{Confidence, Decision, Doc, Epic, Repo, Task};
use crate::reference::References;
use crate::status::{DecisionStatus, Priority, Status, TaskKind};
use crate::testcodec::to_bytes;
use crate::time::{Date, Timestamp};
use crate::{
    BlockEnd, BlocksEdge, DecisionDraft, DecisionId, DocDraft, DocId, DomainError, EpicDraft,
    EpicId, Graph, RepoSlug, TaskDraft, TaskId,
};

mod alloc;
mod cycles;
mod edges;
mod items;
mod list;
mod order;
mod seeded;
mod select;
mod summary;
mod tree;

/// The slug every builder below uses.
pub(super) const SLUG: &str = "camera";
/// A second slug, for nodes that must be rejected as out of scope.
pub(super) const OTHER: &str = "dacode";

pub(super) fn slug() -> RepoSlug {
    RepoSlug::from_str(SLUG).unwrap()
}

pub(super) fn other_slug() -> RepoSlug {
    RepoSlug::from_str(OTHER).unwrap()
}

/// Parses a literal canonical timestamp.
pub(super) fn ts(s: &str) -> Timestamp {
    Timestamp::parse(s).unwrap()
}

/// A caller-passed timestamp derived from a step counter, so a mutation
/// sequence records increasing times without reading a clock (§8).
pub(super) fn at(step: u64) -> Timestamp {
    ts(&format!(
        "2026-09-26T{:02}:{:02}:{:02}Z",
        (step / 3600) % 24,
        (step / 60) % 60,
        step % 60
    ))
}

pub(super) fn task_id(number: u64) -> TaskId {
    TaskId::from_str(&format!("{SLUG}:U{number}")).unwrap()
}

pub(super) fn epic_id(number: u64) -> EpicId {
    EpicId::from_str(&format!("{SLUG}:E{number}")).unwrap()
}

pub(super) fn decision_id(number: u64) -> DecisionId {
    DecisionId::from_str(&format!("{SLUG}:D{number}")).unwrap()
}

pub(super) fn doc_id(number: u64) -> DocId {
    DocId::from_str(&format!("{SLUG}:S{number}")).unwrap()
}

/// Foreign-slug ids, so the generator can attack the id half of
/// [`Graph::in_scope`] as well as the `repo`-field half (see `Forge` in
/// `seeded.rs`).
pub(super) fn other_task_id(number: u64) -> TaskId {
    TaskId::from_str(&format!("{OTHER}:U{number}")).unwrap()
}

pub(super) fn other_epic_id(number: u64) -> EpicId {
    EpicId::from_str(&format!("{OTHER}:E{number}")).unwrap()
}

pub(super) fn other_decision_id(number: u64) -> DecisionId {
    DecisionId::from_str(&format!("{OTHER}:D{number}")).unwrap()
}

pub(super) fn other_doc_id(number: u64) -> DocId {
    DocId::from_str(&format!("{OTHER}:S{number}")).unwrap()
}

/// A graph holding only its repo node.
pub(super) fn empty_graph() -> Graph {
    Graph::new(Repo {
        slug: slug(),
        name: "Camera firmware".to_owned(),
        path: "/srv/repos/camera".to_owned(),
        registered_at: ts("2026-09-25T08:00:00Z"),
    })
}

/// A graph with `epics` epics allocated in order, so `E1..E{epics}` exist.
pub(super) fn graph_with_epics(epics: u64) -> Graph {
    let mut graph = empty_graph();
    for n in 1..=epics {
        graph.add_epic(epic_draft(&format!("epic {n}"))).unwrap();
    }
    graph
}

/// A graph with `epics` epics and `tasks` open tasks under each.
pub(super) fn graph_with_tasks(epics: u64, tasks: u64) -> Graph {
    let mut graph = graph_with_epics(epics);
    for e in 1..=epics {
        for _ in 0..tasks {
            graph.add_task(task_draft(&epic_id(e)), at(0)).unwrap();
        }
    }
    graph
}

pub(super) fn epic_draft(title: &str) -> EpicDraft {
    EpicDraft {
        title: title.to_owned(),
        description: "what it is about".to_owned(),
        status: Status::Open,
        references: References::new(),
    }
}

/// An open task draft with a `verify` method, so it validates as-is.
pub(super) fn task_draft(epic: &EpicId) -> TaskDraft {
    TaskDraft {
        epic: epic.clone(),
        kind: TaskKind::Task,
        title: "a task".to_owned(),
        body: String::new(),
        status: Status::Open,
        verify: Some("cargo test -p todo-domain -- graph".to_owned()),
        evidence: None,
        note: None,
        references: References::new(),
        priority: None,
        until: None,
        estimate: None,
    }
}

pub(super) fn decision_draft(title: &str) -> DecisionDraft {
    DecisionDraft {
        title: title.to_owned(),
        status: DecisionStatus::Open,
        options: vec!["(a) this".to_owned(), "(b) that".to_owned()],
        recommendation: Some("(a)".to_owned()),
        resolution: None,
        confidence: Confidence::Medium,
        references: References::new(),
    }
}

pub(super) fn doc_draft(title: &str) -> DocDraft {
    DocDraft {
        title: title.to_owned(),
        path: "docs/notes.md".to_owned(),
        version: "1.0".to_owned(),
        references: References::new(),
    }
}

/// A whole task node for the explicit-id (replay) path, with the fields
/// its status requires already filled, so it validates on insert.
pub(super) fn task_node(number: u64, epic: u64, status: Status) -> Task {
    let mut task = Task {
        id: task_id(number),
        repo: slug(),
        epic: epic_id(epic),
        kind: TaskKind::Task,
        title: format!("task {number}"),
        body: String::new(),
        status,
        verify: Some("cargo test".to_owned()),
        evidence: None,
        note: None,
        references: References::new(),
        priority: None,
        until: Some(Date::parse("2026-10-01").unwrap()),
        estimate: None,
        created: at(number),
        updated: at(number),
    };
    if status == Status::Closed {
        task.evidence = Some("42 passed".to_owned());
    }
    if status.requires_note() {
        task.note = Some("held by a predecessor".to_owned());
    }
    task
}

/// A whole decision node for the explicit-id path.
pub(super) fn decision_node(number: u64, status: DecisionStatus) -> Decision {
    Decision {
        id: decision_id(number),
        repo: slug(),
        title: format!("decision {number}"),
        status,
        options: vec!["(a)".to_owned(), "(b)".to_owned()],
        recommendation: None,
        resolution: status
            .is_resolved()
            .then_some("signed by the user".to_owned()),
        confidence: Confidence::High,
        references: References::new(),
        created: at(number),
        updated: at(number),
    }
}

/// A whole doc node for the explicit-id path.
pub(super) fn doc_node(number: u64) -> Doc {
    Doc {
        id: doc_id(number),
        repo: slug(),
        title: format!("doc {number}"),
        path: format!("docs/{number}.md"),
        version: "1.0".to_owned(),
        references: References::new(),
        created: at(number),
        updated: at(number),
    }
}

/// A whole epic node for the explicit-id path.
pub(super) fn epic_node(number: u64) -> Epic {
    Epic {
        id: epic_id(number),
        repo: slug(),
        title: format!("epic {number}"),
        description: String::new(),
        status: Status::Open,
        references: References::new(),
    }
}

pub(super) fn task_edge(from: u64, to: u64) -> BlocksEdge {
    BlocksEdge::TaskToTask {
        from: task_id(from),
        to: task_id(to),
    }
}

pub(super) fn gate_task(decision: u64, task: u64) -> BlocksEdge {
    BlocksEdge::DecisionToTask {
        from: decision_id(decision),
        to: task_id(task),
    }
}

pub(super) fn gate_epic(decision: u64, epic: u64) -> BlocksEdge {
    BlocksEdge::DecisionToEpic {
        from: decision_id(decision),
        to: epic_id(epic),
    }
}

/// Every endpoint the graph holds, in [`BlockEnd`] order — the set the
/// invariant scan walks.
pub(super) fn all_ends(graph: &Graph) -> Vec<BlockEnd> {
    let mut ends: Vec<BlockEnd> = graph
        .tasks()
        .map(|task| BlockEnd::Task(task.id.clone()))
        .collect();
    ends.extend(graph.epics().map(|epic| BlockEnd::Epic(epic.id.clone())));
    ends.extend(
        graph
            .decisions()
            .map(|decision| BlockEnd::Decision(decision.id.clone())),
    );
    ends.sort();
    ends
}

/// Asserts the whole §5.1/§3 invariant set over a graph, from the public
/// accessors alone: scope, the parent tree, the closure discipline, edge
/// endpoints and mirror indexes, acyclicity, canonical `relates`, and
/// counters that dominate every live number.
pub(super) fn assert_invariants(graph: &Graph) {
    let slug = graph.slug();
    for epic in graph.epics() {
        assert_eq!(&epic.repo, slug, "epic {} out of scope", epic.id);
        assert_eq!(epic.id.repo(), slug, "epic id {} out of scope", epic.id);
        assert!(epic.id.number() <= graph.counters().epic());
    }
    for task in graph.tasks() {
        assert_eq!(&task.repo, slug, "task {} out of scope", task.id);
        assert_eq!(task.id.repo(), slug, "task id {} out of scope", task.id);
        assert!(
            graph.epic(&task.epic).is_some(),
            "task {} has no epic {}",
            task.id,
            task.epic
        );
        assert!(
            task.validate().is_ok(),
            "task {} breaks the closure discipline",
            task.id
        );
        assert!(
            task.id.number() <= graph.counters().task(),
            "task {} is above its counter",
            task.id
        );
    }
    for decision in graph.decisions() {
        assert_eq!(
            &decision.repo, slug,
            "decision {} out of scope",
            decision.id
        );
        assert_eq!(
            decision.id.repo(),
            slug,
            "decision id {} out of scope",
            decision.id
        );
        assert!(
            decision.validate().is_ok(),
            "decision {} breaks the closure discipline",
            decision.id
        );
        assert!(decision.id.number() <= graph.counters().decision());
    }
    for doc in graph.docs() {
        assert_eq!(&doc.repo, slug, "doc {} out of scope", doc.id);
        assert_eq!(doc.id.repo(), slug, "doc id {} out of scope", doc.id);
        assert!(doc.id.number() <= graph.counters().doc());
    }

    // Every stored edge touches existing nodes, is one of the three
    // legal shapes, is not a self-edge, and appears in both directions.
    let stored = graph.blocks_edges();
    for edge in &stored {
        let (source, target) = (edge.source(), edge.target());
        assert_end_exists(graph, &source);
        assert_end_exists(graph, &target);
        assert_ne!(source, target, "self-edge {edge}");
        assert_eq!(
            BlocksEdge::from_ends(source.clone(), target.clone()),
            Some(edge.clone()),
            "illegal endpoint pair {edge}"
        );
        assert!(
            graph.blocks(&source).contains(&target),
            "forward index lost {edge}"
        );
        assert!(
            graph.blocked_by(&target).contains(&source),
            "mirror index lost {edge}"
        );
    }
    // No duplicate edges, and both indexes account for exactly the stored
    // set — an entry that outlived its edges would show up here.
    let unique: BTreeSet<BlocksEdge> = stored.iter().cloned().collect();
    assert_eq!(unique.len(), stored.len(), "duplicate edge in the listing");
    let ends = all_ends(graph);
    let forward: usize = ends.iter().map(|end| graph.blocks(end).len()).sum();
    let backward: usize = ends.iter().map(|end| graph.blocked_by(end).len()).sum();
    assert_eq!(forward, stored.len(), "forward index disagrees");
    assert_eq!(backward, stored.len(), "mirror index disagrees");
    for end in &ends {
        assert_sorted(&graph.blocks(end), &format!("blocks({end})"));
        assert_sorted(&graph.blocked_by(end), &format!("blocked_by({end})"));
    }

    assert!(
        !holds_cycle(graph),
        "a blocks cycle survived: {:?}",
        cycle_ends(graph)
    );

    // `relates` is canonical, symmetric, task-only, and never leaks into
    // the ordering edges.
    let pairs = graph.relates_pairs();
    assert_sorted(&pairs, "relates pairs");
    for (a, b) in &pairs {
        assert!(a <= b, "uncanonical relates pair {a} {b}");
        assert!(graph.task(a).is_some(), "relates to a missing task {a}");
        assert!(graph.task(b).is_some(), "relates to a missing task {b}");
        assert!(graph.relates_of(a).contains(b), "relates_of({a}) lost {b}");
        assert!(graph.relates_of(b).contains(a), "relates_of({b}) lost {a}");
    }
    // A pair may be both related and ordered — `relates` is an
    // annotation, not an exclusion — so what is checked is that the
    // annotation created no edge of its own: the edge listing above
    // accounts for every entry in both indexes, and a `relates`-only
    // graph is asserted edge-free in `tests::edges`. What per-task
    // listing must hold is its order.
    for task in graph.tasks() {
        assert_sorted(
            &graph.relates_of(&task.id),
            &format!("relates_of({})", task.id),
        );
    }
}

fn assert_end_exists(graph: &Graph, end: &BlockEnd) {
    let known = match end {
        BlockEnd::Task(id) => graph.task(id).is_some(),
        BlockEnd::Epic(id) => graph.epic(id).is_some(),
        BlockEnd::Decision(id) => graph.decision(id).is_some(),
    };
    assert!(known, "edge endpoint {end} names no node");
}

fn assert_sorted<T: Ord + std::fmt::Debug>(items: &[T], what: &str) {
    assert!(
        items.windows(2).all(|pair| pair[0] < pair[1]),
        "{what} is not strictly ascending: {items:?}"
    );
}

/// Whether the `blocks` edges hold any cycle, by an independent
/// depth-first scan over every endpoint.
pub(super) fn holds_cycle(graph: &Graph) -> bool {
    let mut done: BTreeSet<BlockEnd> = BTreeSet::new();
    for start in all_ends(graph) {
        let mut path: Vec<BlockEnd> = Vec::new();
        if scan(graph, start, &mut path, &mut done) {
            return true;
        }
    }
    false
}

/// The endpoints on the cycle the scan found, for a failure message.
fn cycle_ends(graph: &Graph) -> Vec<BlockEnd> {
    let mut done: BTreeSet<BlockEnd> = BTreeSet::new();
    for start in all_ends(graph) {
        let mut path: Vec<BlockEnd> = Vec::new();
        if scan(graph, start, &mut path, &mut done) {
            return path;
        }
    }
    Vec::new()
}

/// One branch of the scan: `path` is the current branch, `done` the set
/// of endpoints already proved acyclic. A node met again on its own
/// branch is a back edge, which in a DAG cannot exist.
fn scan(
    graph: &Graph,
    node: BlockEnd,
    path: &mut Vec<BlockEnd>,
    done: &mut BTreeSet<BlockEnd>,
) -> bool {
    if path.contains(&node) {
        path.push(node);
        return true;
    }
    if done.contains(&node) {
        return false;
    }
    path.push(node.clone());
    for next in graph.blocks(&node) {
        if scan(graph, next, path, done) {
            return true;
        }
    }
    path.pop();
    done.insert(node);
    false
}

/// The graph's whole state as bytes: nodes in id order, edges in listing
/// order, `relates` pairs, counters. Two graphs with equal fingerprints
/// are the same state as far as anything a caller can read goes, which is
/// how the seeded test asserts twice-run equality (§8).
pub(super) fn fingerprint(graph: &Graph) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(to_bytes(graph.repo()).unwrap());
    out.extend(to_bytes(&graph.epics().cloned().collect::<Vec<_>>()).unwrap());
    out.extend(to_bytes(&graph.tasks().cloned().collect::<Vec<_>>()).unwrap());
    out.extend(to_bytes(&graph.decisions().cloned().collect::<Vec<_>>()).unwrap());
    out.extend(to_bytes(&graph.docs().cloned().collect::<Vec<_>>()).unwrap());
    let edges: Vec<String> = graph
        .blocks_edges()
        .iter()
        .map(ToString::to_string)
        .collect();
    out.extend(to_bytes(&edges).unwrap());
    let relates: Vec<String> = graph
        .relates_pairs()
        .iter()
        .map(|(a, b)| format!("{a} {b}"))
        .collect();
    out.extend(to_bytes(&relates).unwrap());
    let counters = graph.counters();
    out.extend(
        to_bytes(&vec![
            counters.epic(),
            counters.task(),
            counters.decision(),
            counters.doc(),
        ])
        .unwrap(),
    );
    out
}

/// The error code a result carried, for operation logs.
pub(super) fn outcome<T>(result: &Result<T, DomainError>) -> String {
    match result {
        Ok(_) => "ok".to_owned(),
        Err(error) => error.code().as_str().to_owned(),
    }
}

/// The naive predicates behind the suites' independent recomputations.
/// Deliberately a second, slower implementation reading only the public
/// accessors and spelling the status matches out by hand, so agreement
/// with production cannot come from a shared bug — independence from
/// *production* is the property that matters, and one shared copy
/// inside the test tree preserves it without per-suite duplication.
pub(super) fn naive_actionable(graph: &Graph, id: &TaskId) -> bool {
    let Some(task) = graph.task(id) else {
        return false;
    };
    let live = task.status == Status::Open || task.status == Status::InWork;
    live && naive_until_satisfied(graph, task)
}

pub(super) fn naive_until_satisfied(graph: &Graph, task: &Task) -> bool {
    for end in graph.blocked_by(&BlockEnd::Task(task.id.clone())) {
        if !naive_end_finished(graph, &end) {
            return false;
        }
    }
    // §5.1's epic gate, consulted by hand.
    for end in graph.blocked_by(&BlockEnd::Epic(task.epic.clone())) {
        if !naive_end_finished(graph, &end) {
            return false;
        }
    }
    true
}

pub(super) fn naive_end_finished(graph: &Graph, end: &BlockEnd) -> bool {
    match end {
        BlockEnd::Task(id) => match graph.task(id) {
            Some(predecessor) => {
                predecessor.status == Status::Closed || predecessor.status == Status::Wontfix
            }
            None => false,
        },
        BlockEnd::Decision(id) => match graph.decision(id) {
            Some(decision) => {
                decision.status == DecisionStatus::Signed
                    || decision.status == DecisionStatus::Superseded
            }
            None => false,
        },
        BlockEnd::Epic(_) => false,
    }
}

/// FNV-1a over bytes: the cheap half of the state comparison. No
/// allocation, so it can run after every step of a mutation sequence;
/// [`fingerprint`] is the stronger byte form, used where a whole run is
/// compared against itself.
struct Digest(u64);

impl Digest {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn bytes(&mut self, data: &[u8]) {
        for byte in data {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn text(&mut self, text: &str) {
        self.bytes(text.as_bytes());
    }

    fn number(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    fn optional(&mut self, value: Option<&str>) {
        match value {
            Some(text) => {
                self.number(1);
                self.text(text);
            }
            None => self.number(0),
        }
    }

    fn references(&mut self, references: &References) {
        self.number(references.len() as u64);
        for reference in references.iter() {
            self.text(reference.kind().as_str());
            self.text(reference.target());
            self.optional(reference.note());
        }
    }
}

/// A structural digest of the whole graph state: every node field, every
/// edge, every annotation and the counters. Equal digests over the same
/// accessor order mean the same state, which is what makes "a rejected
/// write changed nothing" checkable after every step.
pub(super) fn digest(graph: &Graph) -> u64 {
    let mut hash = Digest::new();
    let repo = graph.repo();
    hash.text(repo.slug.as_str());
    hash.text(&repo.name);
    hash.text(&repo.path);
    hash.text(repo.registered_at.as_str());
    for epic in graph.epics() {
        hash.number(epic.id.number());
        hash.text(epic.repo.as_str());
        hash.text(&epic.title);
        hash.text(&epic.description);
        hash.text(epic.status.as_str());
        hash.references(&epic.references);
    }
    for task in graph.tasks() {
        hash.number(task.id.number());
        hash.text(task.repo.as_str());
        hash.number(task.epic.number());
        hash.text(task.kind.as_str());
        hash.text(&task.title);
        hash.text(&task.body);
        hash.text(task.status.as_str());
        hash.optional(task.verify.as_deref());
        hash.optional(task.evidence.as_deref());
        hash.optional(task.note.as_deref());
        hash.references(&task.references);
        hash.optional(task.priority.map(|priority| priority.as_str()));
        hash.optional(task.until.as_ref().map(Date::as_str));
        hash.text(task.created.as_str());
        hash.text(task.updated.as_str());
    }
    for decision in graph.decisions() {
        hash.number(decision.id.number());
        hash.text(decision.repo.as_str());
        hash.text(&decision.title);
        hash.text(decision.status.as_str());
        hash.number(decision.options.len() as u64);
        for option in &decision.options {
            hash.text(option);
        }
        hash.optional(decision.recommendation.as_deref());
        hash.optional(decision.resolution.as_deref());
        hash.text(decision.confidence.as_str());
        hash.references(&decision.references);
        hash.text(decision.created.as_str());
        hash.text(decision.updated.as_str());
    }
    for doc in graph.docs() {
        hash.number(doc.id.number());
        hash.text(doc.repo.as_str());
        hash.text(&doc.title);
        hash.text(&doc.path);
        hash.text(&doc.version);
        hash.references(&doc.references);
        hash.text(doc.created.as_str());
        hash.text(doc.updated.as_str());
    }
    for edge in graph.blocks_edges() {
        hash_end(&mut hash, &edge.source());
        hash_end(&mut hash, &edge.target());
    }
    for (a, b) in graph.relates_pairs() {
        hash.number(a.number());
        hash.number(b.number());
    }
    let counters = graph.counters();
    hash.number(counters.epic());
    hash.number(counters.task());
    hash.number(counters.decision());
    hash.number(counters.doc());
    hash.0
}

fn hash_end(hash: &mut Digest, end: &BlockEnd) {
    match end {
        BlockEnd::Task(id) => {
            hash.text("U");
            hash.number(id.number());
        }
        BlockEnd::Epic(id) => {
            hash.text("E");
            hash.number(id.number());
        }
        BlockEnd::Decision(id) => {
            hash.text("D");
            hash.number(id.number());
        }
    }
}
