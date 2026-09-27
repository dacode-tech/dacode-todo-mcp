//! §10's property test over the graph: pseudo-random mutation sequences
//! driven by a fixed seed never produce a `blocks` cycle, an orphan, a
//! dangling edge or a reused id — and one seed always replays to one
//! operation log and one state (§8).
//!
//! The generator is hand-rolled (Decision 10: no `rand`, no new
//! dependency) and every choice it makes reads either the seed or the
//! graph through a sorted accessor, so a run is a pure function of its
//! seed.

use std::collections::{BTreeMap, BTreeSet};

use super::*;

/// The operations one step may pick from.
const OP_COUNT: usize = 16;
/// Steps per seed: enough that every op fires dozens of times, small
/// enough that the invariant scan after each step stays cheap.
const STEPS: u64 = 300;
/// The deep run's budget. The scan after every step is linear in the
/// state, so the cost of a run grows with the square of this number:
/// 1200 steps is a few seconds, and the five seeds above already cover
/// the same operations at a shallower depth.
const LONG_STEPS: u64 = 1_200;
/// Fixed seeds. A determinism test that re-seeded from a clock would test
/// nothing, so these are literals.
const SEEDS: [u64; 5] = [1, 7, 42, 0x5EED_2026, 0xDEAD_BEEF];

/// Every number the allocator has handed out, per kind prefix: a repeat
/// is a reuse, which §3 forbids.
type Issued = BTreeMap<char, BTreeSet<u64>>;

/// splitmix64 — one `u64` of state, one output per step, no seed value
/// degenerates it (a zero state still advances). Shared with the order
/// tests, whose insertion shuffles must be just as reproducible.
pub(super) struct Rng(pub(super) u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `0..n`; `n` must be non-zero.
    pub(super) fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// A fair coin flip.
    fn coin(&mut self) -> bool {
        self.next_u64().is_multiple_of(2)
    }

    /// A clone of a random element, `None` for an empty slice.
    fn pick<T: Clone>(&mut self, items: &[T]) -> Option<T> {
        items.get(self.below(items.len().max(1))).cloned()
    }
}

#[test]
fn graph_seeded_mutations_never_yield_a_cycle_or_an_orphan() {
    let mut seen_codes: BTreeSet<String> = BTreeSet::new();
    for seed in SEEDS {
        let (log, graph) = run_sequence(seed, STEPS);
        // The sequence built a real graph, so the invariants it passed
        // are not vacuous.
        assert!(graph.tasks().count() > 0, "seed {seed} built no tasks");
        assert!(graph.epics().count() > 0, "seed {seed} built no epics");
        assert!(
            !graph.blocks_edges().is_empty(),
            "seed {seed} built no edges"
        );
        assert!(!holds_cycle(&graph), "seed {seed} ended in a cycle");
        assert_invariants(&graph);

        // Same seed, run again from scratch: identical operation log,
        // identical state, identical bytes (§8).
        let (again, replayed) = run_sequence(seed, STEPS);
        assert_eq!(log, again, "seed {seed} diverged in its operation log");
        assert_eq!(graph, replayed, "seed {seed} diverged in its state");
        assert_eq!(fingerprint(&graph), fingerprint(&replayed));
        assert_eq!(graph.counters(), replayed.counters());

        for line in &log {
            if let Some((_, code)) = line.rsplit_once(" -> ") {
                seen_codes.insert(code.to_owned());
            }
        }
    }
    // Across the seeds the sequences really did reach every rejection
    // path, so "never yields a cycle or an orphan" is a claim about
    // rejected writes too.
    for code in [
        "cycle",
        "orphan_not_allowed",
        "unknown_id",
        "duplicate",
        "missing_verify",
        "missing_note",
        "missing_evidence",
        "missing_resolution",
    ] {
        assert!(
            seen_codes.contains(code),
            "no seed produced a {code} rejection"
        );
    }
}

#[test]
fn graph_seeded_mutations_hold_for_a_long_run_too() {
    // One deep sequence: deletes, cascades and replays interacting over
    // many more steps than the fixed budget above.
    let (log, graph) = run_sequence(0x1234_5678, LONG_STEPS);
    assert_invariants(&graph);
    assert!(!holds_cycle(&graph));
    assert_eq!(log.len(), LONG_STEPS as usize);
    let (again, replayed) = run_sequence(0x1234_5678, LONG_STEPS);
    assert_eq!(log, again);
    assert_eq!(graph, replayed);
}

/// Runs `steps` operations from `seed` and returns the operation log and
/// the final state, asserting the full invariant set after every single
/// step.
fn run_sequence(seed: u64, steps: u64) -> (Vec<String>, Graph) {
    let mut rng = Rng(seed);
    let mut graph = empty_graph();
    let mut issued: Issued = BTreeMap::new();
    let mut log: Vec<String> = Vec::new();
    let mut counters = graph.counters();

    for step in 0..steps {
        // A rejected write must leave the graph bit-for-bit unchanged
        // (Decision 6), so the state is digested before the step.
        let before = digest(&graph);
        let op = rng.below(OP_COUNT);
        let line = apply(&mut graph, &mut rng, op, step, &mut issued);
        log.push(line.clone());

        assert_invariants(&graph);
        // Counters only ever move up, whatever the step did.
        let now = graph.counters();
        assert!(
            now.epic() >= counters.epic(),
            "epic counter moved back: {line}"
        );
        assert!(
            now.task() >= counters.task(),
            "task counter moved back: {line}"
        );
        assert!(
            now.decision() >= counters.decision(),
            "decision counter moved back: {line}"
        );
        assert!(
            now.doc() >= counters.doc(),
            "doc counter moved back: {line}"
        );
        counters = now;

        // Only the rejection direction is assertable: an `Ok` may be an
        // idempotent no-op (re-linking an edge that is already there).
        if !line.ends_with("-> ok") {
            assert_eq!(before, digest(&graph), "a rejected write mutated: {line}");
        }
    }
    (log, graph)
}

/// Performs one pseudo-random operation and returns its log line.
fn apply(graph: &mut Graph, rng: &mut Rng, op: usize, step: u64, issued: &mut Issued) -> String {
    let stamp = at(step);
    match op {
        0 => {
            let title = format!("epic {step}");
            let result = graph.add_epic(epic_draft(&title));
            record(issued, &result, EpicId::PREFIX, |epic| epic.id.number());
            line("add_epic", &title, &result)
        }
        1 => {
            let epic = some_epic(graph, rng);
            let mut draft = task_draft(&epic);
            draft.status = random_status(rng);
            draft.kind = random_kind(rng);
            draft.evidence = rng.coin().then(|| format!("evidence {step}"));
            draft.note = rng.coin().then(|| format!("note {step}"));
            draft.priority = rng.coin().then(|| random_priority(rng));
            if rng.below(8) == 0 {
                draft.verify = None;
            }
            let detail = format!("{epic} {}", draft.status);
            let result = graph.add_task(draft, stamp);
            record(issued, &result, TaskId::PREFIX, |task| task.id.number());
            line("add_task", &detail, &result)
        }
        2 => {
            let mut draft = decision_draft(&format!("decision {step}"));
            draft.status = random_decision_status(rng);
            draft.resolution = rng.coin().then(|| format!("resolution {step}"));
            let detail = draft.status.as_str().to_owned();
            let result = graph.add_decision(draft, stamp);
            record(issued, &result, DecisionId::PREFIX, |d| d.id.number());
            line("add_decision", &detail, &result)
        }
        3 => {
            let draft = doc_draft(&format!("doc {step}"));
            let result = graph.add_doc(draft, stamp);
            record(issued, &result, DocId::PREFIX, |doc| doc.id.number());
            line("add_doc", &format!("doc {step}"), &result)
        }
        4 => {
            let from = some_task(graph, rng);
            let to = some_task(graph, rng);
            let edge = task_edge(from.number(), to.number());
            let detail = edge.to_string();
            let result = graph.link_blocks(edge);
            line("link_blocks", &detail, &result)
        }
        5 => {
            let from = some_decision(graph, rng);
            let edge = if rng.coin() {
                gate_task(from.number(), some_task(graph, rng).number())
            } else {
                gate_epic(from.number(), some_epic(graph, rng).number())
            };
            let detail = edge.to_string();
            let result = graph.link_blocks(edge);
            line("link_gate", &detail, &result)
        }
        6 => {
            let edge = match rng.pick(&graph.blocks_edges()) {
                Some(edge) => edge,
                None => task_edge(
                    some_task(graph, rng).number(),
                    some_task(graph, rng).number(),
                ),
            };
            let detail = edge.to_string();
            let result = graph.unlink_blocks(edge);
            line("unlink_blocks", &detail, &result)
        }
        7 => {
            let a = some_task(graph, rng);
            let b = some_task(graph, rng);
            let result = graph.link_relates(a.clone(), b.clone());
            line("link_relates", &format!("{a} {b}"), &result)
        }
        8 => {
            let a = some_task(graph, rng);
            let b = some_task(graph, rng);
            let result = graph.unlink_relates(a.clone(), b.clone());
            line("unlink_relates", &format!("{a} {b}"), &result)
        }
        9 => {
            let id = some_task(graph, rng);
            let status = random_status(rng);
            let evidence = rng.coin().then(|| format!("evidence {step}"));
            let note = rng.coin().then(|| format!("note {step}"));
            let result = graph
                .update_task(id.clone(), stamp, move |task| {
                    task.status = status;
                    task.evidence = evidence;
                    task.note = note;
                    task.title = format!("task {step}");
                })
                .map(|task| task.id);
            line("update_task", &format!("{id} {status}"), &result)
        }
        10 => {
            let id = some_decision(graph, rng);
            let status = random_decision_status(rng);
            let resolution = rng.coin().then(|| format!("resolution {step}"));
            // A status change mutates no edge (§3/D10: the history stays).
            let edges = graph.blocks_edges();
            let result = graph.update_decision(id.clone(), stamp, move |decision| {
                decision.status = status;
                decision.resolution = resolution;
            });
            assert_eq!(graph.blocks_edges(), edges, "a status change moved an edge");
            line(
                "update_decision",
                &format!("{id} {status}"),
                &result.map(|d| d.id),
            )
        }
        11 => {
            let id = some_epic(graph, rng);
            let status = random_status(rng);
            let result = graph.update_epic(id.clone(), move |epic| {
                epic.status = status;
                epic.title = format!("epic {step}");
            });
            line(
                "update_epic",
                &format!("{id} {status}"),
                &result.map(|e| e.id),
            )
        }
        12 => {
            let id = some_doc(graph, rng);
            let result = graph.update_doc(id.clone(), stamp, move |doc| {
                doc.path = format!("docs/{step}.md");
                doc.version = format!("1.{step}");
            });
            line("update_doc", &id.to_string(), &result.map(|d| d.id))
        }
        13 => {
            let id = some_task(graph, rng);
            let epic = some_epic(graph, rng);
            let result = graph.reparent_task(id.clone(), epic.clone(), stamp);
            line(
                "reparent_task",
                &format!("{id} {epic}"),
                &result.map(|t| t.id),
            )
        }
        14 => {
            let id = some_epic(graph, rng);
            let result = graph.delete_epic(id.clone());
            let detail = match &result {
                Ok(tasks) => format!(
                    "{id} cascade={}",
                    tasks
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                Err(_) => id.to_string(),
            };
            line("delete_epic", &detail, &result)
        }
        _ => explicit_insert(graph, rng, step),
    }
}

/// The replay path: an explicit-id insert of a random kind, with a number
/// below or above the counter, sometimes in scope and sometimes not,
/// sometimes validating and sometimes not.
fn explicit_insert(graph: &mut Graph, rng: &mut Rng, step: u64) -> String {
    // Reach above the counters often enough that the raise path runs.
    let number = 1 + rng.below(graph.counters().task().max(4) as usize + 3) as u64;
    let foreign = rng.below(10) == 0;
    match rng.below(4) {
        0 => {
            let mut epic = epic_node(number);
            if foreign {
                epic.repo = other_slug();
            }
            let result = graph.insert_epic(epic);
            line(
                "insert_epic",
                &insert_detail(number, foreign, step),
                &result,
            )
        }
        1 => {
            let epic = some_epic(graph, rng).number();
            let status = random_status(rng);
            let mut task = task_node(number, epic, status);
            if foreign {
                task.repo = other_slug();
            }
            // Half the time the closure fields are stripped, so the
            // replay path exercises the discipline too.
            if rng.coin() {
                task.evidence = None;
                task.note = None;
            }
            let result = graph.insert_task(task);
            line(
                "insert_task",
                &insert_detail(number, foreign, epic),
                &result,
            )
        }
        2 => {
            let status = random_decision_status(rng);
            let mut decision = decision_node(number, status);
            if foreign {
                decision.repo = other_slug();
            }
            if rng.coin() {
                decision.resolution = None;
            }
            let result = graph.insert_decision(decision);
            line(
                "insert_decision",
                &insert_detail(number, foreign, step),
                &result,
            )
        }
        _ => {
            let mut doc = doc_node(number);
            if foreign {
                doc.repo = other_slug();
            }
            let result = graph.insert_doc(doc);
            line("insert_doc", &insert_detail(number, foreign, step), &result)
        }
    }
}

/// A task id to aim an operation at: a live one when the graph holds any,
/// otherwise a small number that is usually missing, so the sequences
/// exercise `unknown_id` as well as the happy path.
fn some_task(graph: &Graph, rng: &mut Rng) -> TaskId {
    let live: Vec<TaskId> = graph.tasks().map(|task| task.id.clone()).collect();
    rng.pick(&live)
        .unwrap_or_else(|| task_id(1 + rng.below(3) as u64))
}

fn some_epic(graph: &Graph, rng: &mut Rng) -> EpicId {
    let live: Vec<EpicId> = graph.epics().map(|epic| epic.id.clone()).collect();
    rng.pick(&live)
        .unwrap_or_else(|| epic_id(1 + rng.below(3) as u64))
}

fn some_decision(graph: &Graph, rng: &mut Rng) -> DecisionId {
    let live: Vec<DecisionId> = graph
        .decisions()
        .map(|decision| decision.id.clone())
        .collect();
    rng.pick(&live)
        .unwrap_or_else(|| decision_id(1 + rng.below(3) as u64))
}

fn some_doc(graph: &Graph, rng: &mut Rng) -> DocId {
    let live: Vec<DocId> = graph.docs().map(|doc| doc.id.clone()).collect();
    rng.pick(&live)
        .unwrap_or_else(|| doc_id(1 + rng.below(3) as u64))
}

fn random_status(rng: &mut Rng) -> Status {
    match rng.below(6) {
        0 => Status::Open,
        1 => Status::InWork,
        2 => Status::Blocked,
        3 => Status::Deferred,
        4 => Status::Closed,
        _ => Status::Wontfix,
    }
}

fn random_decision_status(rng: &mut Rng) -> DecisionStatus {
    match rng.below(3) {
        0 => DecisionStatus::Open,
        1 => DecisionStatus::Signed,
        _ => DecisionStatus::Superseded,
    }
}

fn random_kind(rng: &mut Rng) -> TaskKind {
    match rng.below(4) {
        0 => TaskKind::Task,
        1 => TaskKind::Gate,
        2 => TaskKind::Finding,
        _ => TaskKind::Debt,
    }
}

fn random_priority(rng: &mut Rng) -> Priority {
    match rng.below(3) {
        0 => Priority::High,
        1 => Priority::Medium,
        _ => Priority::Low,
    }
}

/// One log line: the operation, the arguments it was given, and what the
/// graph answered.
fn line<T>(name: &str, detail: &str, result: &Result<T, DomainError>) -> String {
    format!("{name} {detail} -> {}", outcome(result))
}

fn insert_detail(number: u64, foreign: bool, extra: u64) -> String {
    format!("{number} foreign={foreign} {extra}")
}

/// Notes an allocated number, and fails the test if the allocator ever
/// hands the same one out twice (§3: never reused).
fn record<T>(
    issued: &mut Issued,
    result: &Result<T, DomainError>,
    prefix: char,
    number: impl Fn(&T) -> u64,
) {
    let Ok(node) = result else { return };
    let fresh = issued.entry(prefix).or_default().insert(number(node));
    assert!(fresh, "the allocator reissued {prefix}{}", number(node));
}
