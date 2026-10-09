//! The `progress` subcommand's render (spec §13, D20): the
//! total-progress surfaces over one repository's graph — the model
//! `tools/progress.py`'s `build()` computed, ported onto the domain
//! [`Graph`] this process reads lock-free (D19) instead of a wire
//! client's `graph_show` answer. The Python tool drove a spawned
//! server because it was a client; this module is in-process, so the
//! model derives from the graph directly and every surface is a pure
//! function of `(root, repo)` — no clock, no git, no environment
//! (§8): the golden-pinned bytes, and two runs over unchanged state,
//! are byte-identical by construction.
//!
//! The module is four files, split at U37's named point and one seam
//! beyond it: this one holds the model, the resolution and the shared
//! text helpers, [`json()`] the golden-pinned JSON document, [`page()`]
//! the HTML page with the project switcher, and `svg` the
//! dependency-graph SVG (extracted from the page when the page half
//! crossed the 600-line target — the split the spec's D1 named, plus
//! the seam it forced). The public surface is four functions over one
//! internal model — [`json()`] and [`summary`] (one surface each),
//! [`page()`] (the HTML), and [`outputs`] (one resolve + one build, all
//! three surfaces plus the repo's registered checkout, for the
//! subcommand's default write location) — plus [`resolve_by_cwd`],
//! the subcommand's repo tier: the `--repo` flag wins (the binary
//! owns that check), else the working directory's deterministic
//! longest-ancestor match against the registered checkouts (§7, the
//! same rule the server's default tier applies). The binary reads the
//! cwd once and passes it in; this crate never touches the
//! environment.
//!
//! The resolution mirrors `ServerState::resolve_read`'s discipline
//! exactly (the t64 hand-off): the registry consult first —
//! an unlisted slug is `unknown_id`'s message, never a raw io, because
//! [`DataRoot::read_repo`] is path-derived and would otherwise read an
//! unregistered-but-present directory silently — then the lock-free
//! read, then the slug cross-check: a store directory holding another
//! repo's graph under the asked slug is the same `corrupt_store`
//! refusal the locked open produces, never the copied graph served as
//! the named repo.
//!
//! Two rules port verbatim from the Python tool, both because a
//! renderer that quietly shows less than everything is worse than no
//! renderer: it cannot pass by reading nothing (a repo the root does
//! not carry, or a graph with no epics, is an error — never an empty
//! document), and it cannot drop a node silently (a decision the model
//! cannot place, and every edge with an endpoint it cannot place, are
//! reported in `not_placed`, never dropped).

mod json;
mod page;
mod svg;

pub use self::json::json;
pub use self::page::{Outputs, page};

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use todo_domain::{
    BlockEnd, Decision, DecisionId, DecisionStatus, DomainError, Epic, EpicId, Graph, Repo,
    RepoSlug, Status, Task, TaskId, TaskKind,
};
use todo_store::{DataRoot, Registry, StoreError};

use crate::ToolError;
use crate::cwd::{MatchOutcome, match_slug};

/// Per-kind counts over the four task kinds (the Python tool's
/// `kind_state`, the same fixed key order) — a model type the JSON
/// document serializes and the page's cards read.
#[derive(Default, Clone, serde::Serialize)]
struct KindState {
    /// `task`-kind counts.
    task: KindCounts,
    /// `gate`-kind counts.
    gate: KindCounts,
    /// `finding`-kind counts.
    finding: KindCounts,
    /// `debt`-kind counts.
    debt: KindCounts,
}

/// One kind's counts.
#[derive(Default, Clone, serde::Serialize)]
struct KindCounts {
    /// Tasks of the kind.
    total: usize,
    /// Closed or wontfix.
    done: usize,
    /// Everything else.
    open: usize,
}

/// The render model over one graph: every list in a documented total
/// order (§8). Borrowed from the graph — the model is a view, and the
/// public functions serialize or narrate from it.
struct Model<'a> {
    /// The repo node — the slug and the import instant.
    repo: &'a Repo,
    /// One lane per epic, ascending epic id.
    lanes: Vec<Lane<'a>>,
    /// Every decision, ascending id.
    decisions: Vec<&'a Decision>,
    /// Every blocks edge as `(from, to)` full ids, the documented
    /// order.
    edges: Vec<(String, String)>,
    /// What each decision gates, short ids, the adjacency's order.
    gates: BTreeMap<DecisionId, Vec<String>>,
    /// An open task under an epic gated by an open decision carries the
    /// gating decisions' short ids — the "waits on" column.
    waits: BTreeMap<TaskId, String>,
    /// The non-done tasks in (lane, id) order.
    open_items: Vec<&'a Task>,
    /// The milestone epics in lane order with the board's M id.
    milestones: Vec<(&'a Epic, String)>,
    /// The unplaced-decision and dropped-edge reports.
    not_placed: Vec<String>,
    /// Per-kind counts over the four task kinds.
    kind_state: KindState,
    /// Every task, all kinds.
    task_count: usize,
}

/// One lane: an epic, its tasks in id order, and the decisions
/// attached to it.
struct Lane<'a> {
    /// The lane's epic.
    epic: &'a Epic,
    /// The board's M id when the epic is a milestone.
    milestone: Option<String>,
    /// The epic's tasks, ascending id.
    tasks: Vec<&'a Task>,
    /// The decisions gating this epic or one of its tasks, decision id
    /// ascending.
    decisions: Vec<&'a Decision>,
}

/// The `progress` human summary: the Python tool's summary block minus
/// the git sha and the date — narration, not a pinned surface, but
/// derived from the same model so it can never disagree with the JSON.
pub fn summary(root: &DataRoot, repo: &RepoSlug) -> Result<String, String> {
    let graph = resolve(root, repo).map_err(|error| error.message)?;
    let model = build(&graph).map_err(|error| error.message)?;
    Ok(summary_text(&model))
}

/// Every surface from one resolve + one build — the subcommand's
/// default path, so the page, the JSON twin and the summary can never
/// disagree with each other (they are one model), and the binary
/// derives its default output directory from the repo's registered
/// checkout without a second read.
pub fn outputs(root: &DataRoot, repo: &RepoSlug) -> Result<Outputs, String> {
    let graph = resolve(root, repo).map_err(|error| error.message)?;
    let model = build(&graph).map_err(|error| error.message)?;
    let slugs = page::switcher_slugs(root)?;
    Ok(Outputs {
        page: page::page_text(&model, &slugs)?,
        json: json::json_text(&model)?,
        summary: summary_text(&model),
        checkout: PathBuf::from(graph.repo().path.clone()),
    })
}

/// The subcommand's repo tier (§7): the working directory's
/// deterministic longest-ancestor match against the registered
/// checkouts — the same rule the server's default tier applies, minus
/// the tool-parameter tier a subcommand does not have (the `--repo`
/// flag is checked by the binary before this is called). The walk
/// reads every registered slug lock-free with `resolve`'s
/// discipline: a store that will not read fails the walk — silently
/// skipping it would report less than the truth while still looking
/// complete.
pub fn resolve_by_cwd(root: &DataRoot, cwd: Option<&Path>) -> Result<RepoSlug, String> {
    let Some(cwd) = cwd else {
        return Err("no --repo and the working directory could not be read".to_owned());
    };
    let registry = Registry::load(root).map_err(|error| ToolError::from(error).message)?;
    let mut candidates: Vec<(RepoSlug, PathBuf)> = Vec::with_capacity(registry.slugs().len());
    for slug in registry.slugs() {
        let graph = resolve(root, slug).map_err(|error| error.message)?;
        candidates.push((slug.clone(), PathBuf::from(graph.repo().path.clone())));
    }
    match match_slug(cwd, &candidates) {
        MatchOutcome::Match(slug) => Ok(slug),
        MatchOutcome::NoMatch => {
            Err("no --repo and the working directory matches no registered checkout".to_owned())
        }
        MatchOutcome::Ambiguous(slugs) => {
            let names: Vec<&str> = slugs.iter().map(RepoSlug::as_str).collect();
            Err(format!(
                "no --repo and the working directory matches multiple repositories: {}",
                names.join(", ")
            ))
        }
    }
}

/// The resolution mirror of `crate::ServerState::resolve_read`: the
/// registry consult, the lock-free read, the slug cross-check. The
/// refresh subcommand reuses it (D20) — the discipline has one home,
/// not a per-consumer copy.
pub(crate) fn resolve(root: &DataRoot, slug: &RepoSlug) -> Result<Graph, ToolError> {
    let registry = Registry::load(root)?;
    if !registry.contains(slug) {
        return Err(ToolError::from(DomainError::UnknownId {
            id: slug.to_string(),
        }));
    }
    let graph = root.read_repo(slug)?;
    let repo = graph.repo();
    if &repo.slug != slug {
        return Err(ToolError::from(StoreError::CorruptStore {
            offset: None,
            detail: format!("the store opened for {slug} holds repo {}", repo.slug),
        }));
    }
    Ok(graph)
}

/// The model over one graph — the Python tool's `build()` ported onto
/// the domain types.
fn build(graph: &Graph) -> Result<Model<'_>, ToolError> {
    let epics: Vec<&Epic> = graph.epics().collect();
    if epics.is_empty() {
        return Err(ToolError::invalid(
            "the graph holds no epics — refusing to render a roadmap from nothing",
        ));
    }
    let mut task_epic: BTreeMap<TaskId, EpicId> = BTreeMap::new();
    let mut lanes: Vec<Lane> = Vec::with_capacity(epics.len());
    for epic in epics.iter() {
        let tasks = graph.tasks_of_epic(&epic.id);
        for task in &tasks {
            task_epic.insert(task.id.clone(), epic.id.clone());
        }
        lanes.push(Lane {
            epic,
            milestone: milestone_of(epic),
            tasks,
            decisions: Vec::new(),
        });
    }

    // The decisions: what each gates, and the lane it attaches to —
    // the epic it gates wins, else the epic of its first gated task.
    // A decision gating nothing cannot be placed and is reported,
    // never dropped.
    let decisions: Vec<&Decision> = graph.decisions().collect();
    let mut gates: BTreeMap<DecisionId, Vec<String>> = BTreeMap::new();
    let mut not_placed: Vec<String> = Vec::new();
    let mut placed: Vec<(&Decision, EpicId)> = Vec::new();
    for decision in decisions.iter().copied() {
        let targets = graph.blocks(&BlockEnd::Decision(decision.id.clone()));
        gates.insert(
            decision.id.clone(),
            targets
                .iter()
                .map(|target| short_id(&target.to_string()))
                .collect(),
        );
        let lane_epic = targets
            .iter()
            .find_map(|target| match target {
                BlockEnd::Epic(epic) => Some(epic.clone()),
                _ => None,
            })
            .or_else(|| {
                targets.iter().find_map(|target| match target {
                    BlockEnd::Task(task) => task_epic.get(task).cloned(),
                    _ => None,
                })
            });
        match lane_epic {
            Some(epic) => placed.push((decision, epic)),
            None => not_placed.push(format!(
                "{} (a decision gating nothing)",
                short_id(&decision.id.to_string())
            )),
        }
    }
    let mut by_epic: BTreeMap<EpicId, Vec<&Decision>> = BTreeMap::new();
    for (decision, epic) in placed {
        by_epic.entry(epic).or_default().push(decision);
    }
    for lane in lanes.iter_mut() {
        if let Some(list) = by_epic.remove(&lane.epic.id) {
            lane.decisions = list;
        }
    }

    // Every blocks edge, decision-sourced first then task-sourced. An
    // endpoint the model cannot place is reported, never dropped —
    // unreachable through the domain's shapes today (a decision with a
    // target is placeable; every task and epic is), but the rule ports
    // from the Python tool and stays: a renderer that quietly shows
    // less than everything is worse than no renderer.
    let mut placed_ids: BTreeSet<String> = BTreeSet::new();
    placed_ids.extend(epics.iter().map(|epic| epic.id.to_string()));
    placed_ids.extend(graph.tasks().map(|task| task.id.to_string()));
    placed_ids.extend(lanes.iter().flat_map(|lane| {
        lane.decisions
            .iter()
            .map(|decision| decision.id.to_string())
    }));
    let mut edges: Vec<(String, String)> = Vec::new();
    for decision in decisions.iter().copied() {
        for target in graph.blocks(&BlockEnd::Decision(decision.id.clone())) {
            let from = decision.id.to_string();
            let to = target.to_string();
            if placed_ids.contains(&from) && placed_ids.contains(&to) {
                edges.push((from, to));
            } else {
                not_placed.push(format!("{} → {}", short_id(&from), short_id(&to)));
            }
        }
    }
    for task in graph.tasks() {
        for successor in graph.blocks(&BlockEnd::Task(task.id.clone())) {
            let from = task.id.to_string();
            let to = successor.to_string();
            if placed_ids.contains(&from) && placed_ids.contains(&to) {
                edges.push((from, to));
            } else {
                not_placed.push(format!("{} → {}", short_id(&from), short_id(&to)));
            }
        }
    }

    // The waits column: an open task under an epic gated by an open
    // decision waits on it.
    let open_decisions: BTreeSet<&DecisionId> = decisions
        .iter()
        .filter(|decision| decision.status == DecisionStatus::Open)
        .map(|decision| &decision.id)
        .collect();
    let mut waits: BTreeMap<TaskId, String> = BTreeMap::new();
    for lane in lanes.iter() {
        let gating: Vec<String> = graph
            .blocked_by(&BlockEnd::Epic(lane.epic.id.clone()))
            .iter()
            .filter_map(|blocker| match blocker {
                BlockEnd::Decision(id) if open_decisions.contains(id) => {
                    Some(short_id(&id.to_string()))
                }
                _ => None,
            })
            .collect();
        if gating.is_empty() {
            continue;
        }
        let text = gating.join(", ");
        for task in lane.tasks.iter().filter(|task| !is_done(task.status)) {
            waits.insert(task.id.clone(), text.clone());
        }
    }

    let mut open_items: Vec<&Task> = Vec::new();
    for lane in lanes.iter() {
        open_items.extend(
            lane.tasks
                .iter()
                .filter(|task| !is_done(task.status))
                .copied(),
        );
    }
    let milestones: Vec<(&Epic, String)> = lanes
        .iter()
        .filter_map(|lane| lane.milestone.as_ref().map(|id| (lane.epic, id.clone())))
        .collect();
    let all_tasks: Vec<&Task> = graph.tasks().collect();
    let kind_state = KindState {
        task: counts_for(&all_tasks, TaskKind::Task),
        gate: counts_for(&all_tasks, TaskKind::Gate),
        finding: counts_for(&all_tasks, TaskKind::Finding),
        debt: counts_for(&all_tasks, TaskKind::Debt),
    };
    Ok(Model {
        repo: graph.repo(),
        lanes,
        decisions,
        edges,
        gates,
        waits,
        open_items,
        milestones,
        not_placed,
        kind_state,
        task_count: all_tasks.len(),
    })
}

/// The human summary from the model.
fn summary_text(model: &Model) -> String {
    let signed = model
        .decisions
        .iter()
        .filter(|decision| decision.status == DecisionStatus::Signed)
        .count();
    let milestones_done = model
        .milestones
        .iter()
        .filter(|(epic, _)| epic.status == Status::Closed)
        .count();
    let open_ids: Vec<String> = model
        .open_items
        .iter()
        .map(|task| short_id(&task.id.to_string()))
        .collect();
    let graph_nodes: usize = model
        .lanes
        .iter()
        .map(|lane| lane.tasks.len() + lane.decisions.len())
        .sum();
    let mut out = format!(
        "progress — imported {} ({} tasks, {} decisions)\n",
        model.repo.registered_at.as_str(),
        model.task_count,
        model.decisions.len()
    );
    for (label, counts) in [
        ("task", &model.kind_state.task),
        ("gate", &model.kind_state.gate),
        ("finding", &model.kind_state.finding),
        ("debt", &model.kind_state.debt),
    ] {
        out.push_str(&format!(
            "  {label:8} {:3}/{:<3} done {:3} open\n",
            counts.done, counts.total, counts.open
        ));
    }
    out.push_str(&format!(
        "  milestones {}/{} done · decisions {}/{} signed · open items: {}\n",
        milestones_done,
        model.milestones.len(),
        signed,
        model.decisions.len(),
        if open_ids.is_empty() {
            "none".to_owned()
        } else {
            open_ids.join(", ")
        }
    ));
    out.push_str(&format!(
        "  graph: {} nodes, {} edges\n",
        graph_nodes,
        model.edges.len()
    ));
    if !model.not_placed.is_empty() {
        out.push_str(&format!(
            "  NOT PLACED (reported, never dropped): {}\n",
            model.not_placed.join(", ")
        ));
    }
    out
}

/// The board's M id for a milestone epic, recovered from the mapping's
/// provenance description — the one contract every dialect writes:
/// `the <board> board's §<n> milestone <M-id>` (own, camera and ikvm
/// §2; dacode §5, where the board's milestones live). The synthetic
/// containers (`… §6 open follow-ups, not tied to a milestone`) do not
/// match: the section part must be digits and the milestone word must
/// be followed by exactly one token.
fn milestone_of(epic: &Epic) -> Option<String> {
    let (head, tail) = epic.description.split_once(" milestone ")?;
    if !head.starts_with("the ") || !head.contains(" board's §") {
        return None;
    }
    let section = head.rsplit(" board's §").next().unwrap_or_default();
    if section.is_empty() || !section.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut tokens = tail.split_whitespace();
    let id = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }
    Some(id.to_owned())
}

/// The Python `html.escape(quote=True)` equivalent — the same five
/// entities.
fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Whitespace-collapsed, char-boundary-safe truncation with the
/// ellipsis — the Python tool's `short()`.
fn short(label: &str, n: usize) -> String {
    let collapsed = label.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= n {
        collapsed
    } else {
        let prefix: String = collapsed.chars().take(n.saturating_sub(1)).collect();
        format!("{prefix}…")
    }
}

/// The short form of a full id — the part after the `:`.
fn short_id(full: &str) -> String {
    match full.split_once(':') {
        Some((_, short)) => short.to_owned(),
        None => full.to_owned(),
    }
}

/// Whether a task status is a done state.
fn is_done(status: Status) -> bool {
    matches!(status, Status::Closed | Status::Wontfix)
}

/// One kind's counts over the task list.
fn counts_for(tasks: &[&Task], kind: TaskKind) -> KindCounts {
    let mut counts = KindCounts::default();
    for task in tasks.iter().filter(|task| task.kind == kind) {
        counts.total += 1;
        if is_done(task.status) {
            counts.done += 1;
        } else {
            counts.open += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use todo_domain::References;

    /// An epic with `description`, for the matcher tests.
    fn epic_with(description: &str) -> Epic {
        Epic {
            id: "fixture:E1".parse().unwrap(),
            repo: "fixture".parse().unwrap(),
            title: "an epic".to_owned(),
            description: description.to_owned(),
            status: Status::Open,
            references: References::new(),
        }
    }

    /// All four dialects' provenance prefixes recover their M id —
    /// own, camera and ikvm write §2, dacode §5.
    #[test]
    fn every_dialects_milestone_prefix_recovers_the_id() {
        for (description, id) in [
            ("the dacode-todo-mcp board's §2 milestone M0", "M0"),
            ("the camera board's §2 milestone M3", "M3"),
            ("the ikvm-html board's §2 milestone M1", "M1"),
            ("the dacode board's §5 milestone M6", "M6"),
        ] {
            assert_eq!(milestone_of(&epic_with(description)), Some(id.to_owned()));
        }
    }

    /// The synthetic containers and prose descriptions do not match:
    /// the follow-ups container ends at "milestone" with no trailing
    /// id, a non-digit section fails, and a second token after the id
    /// fails.
    #[test]
    fn synthetic_containers_and_prose_do_not_match() {
        for description in [
            "the fixture board's §6 open follow-ups, not tied to a milestone",
            "the fixture board's §two milestone M0",
            "the fixture board's §2 milestone M0 and more",
            "The sensor bring-up epic.",
            "a milestone word in prose",
        ] {
            assert_eq!(milestone_of(&epic_with(description)), None);
        }
    }
}
