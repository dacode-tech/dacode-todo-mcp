//! The render model (U57's split out of `mod.rs`, paying the
//! file-size debt that file's module doc recorded): the types every
//! progress surface renders from — [`Model`] with its [`Lane`]s and
//! the per-kind counts — and the two halves that produce it,
//! [`gather`] (the reads) and [`build`] (the construction, the
//! Python tool's `build()` ported onto the domain types). Every
//! surface renders from one model built by one call, so the JSON
//! document, the HTML pages and the summary can never disagree with
//! each other.
//!
//! The resolution inside [`gather`] mirrors
//! `ServerState::resolve_read`'s discipline exactly (the t64
//! hand-off): the registry consult first — an unlisted slug is
//! `unknown_id`'s message, never a raw io, because
//! [`DataRoot::read_repo`] is path-derived and would otherwise read
//! an unregistered-but-present directory silently — then the
//! lock-free read, then the slug cross-check: a store directory
//! holding another repo's graph under the asked slug is the same
//! `corrupt_store` refusal the locked open produces, never the
//! copied graph served as the named repo. The census read joins the
//! resolve in [`gather`] (t84's D8): after it, over the same slug,
//! so an unregistered slug is refused before it — and a census
//! failure fails the render, the panel being part of the page, never
//! silently omitted. The rewards read joins last (U63): the config
//! first — absent or disabled answers `None` and every later read is
//! skipped, §15's fence — and a malformed config fails the render
//! the same way, never a silent inert root.
//!
//! Two rules port verbatim from the Python tool, both because a
//! renderer that quietly shows less than everything is worse than no
//! renderer: it cannot pass by reading nothing (a repo the root does
//! not carry, or a graph with no epics, is an error — never an empty
//! document), and it cannot drop a node silently (a decision the
//! model cannot place, and every edge with an endpoint it cannot
//! place, are reported in `not_placed`, never dropped).

use std::collections::{BTreeMap, BTreeSet};

use todo_domain::{
    BlockEnd, Decision, DecisionId, DecisionStatus, Epic, EpicId, Graph, Repo, RepoSlug, Task,
    TaskId, TaskKind,
};
use todo_store::{DataRoot, EventCensus};

use crate::ToolError;
use crate::rewards::milestone_of;

use super::rewards::{RewardsModel, gather_rewards};
use super::sidebar::{SidebarRepo, sidebar_repos};
use super::{counts_for, is_done, resolve, short_id};

/// Per-kind counts over the four task kinds (the Python tool's
/// `kind_state`, the same fixed key order) — a model type the JSON
/// document serializes and the page's cards read.
#[derive(Default, Clone, serde::Serialize)]
pub(super) struct KindState {
    /// `task`-kind counts.
    pub(super) task: KindCounts,
    /// `gate`-kind counts.
    pub(super) gate: KindCounts,
    /// `finding`-kind counts.
    pub(super) finding: KindCounts,
    /// `debt`-kind counts.
    pub(super) debt: KindCounts,
}

/// One kind's counts.
#[derive(Default, Clone, serde::Serialize)]
pub(super) struct KindCounts {
    /// Tasks of the kind.
    pub(super) total: usize,
    /// Closed or wontfix.
    pub(super) done: usize,
    /// Everything else.
    pub(super) open: usize,
}

/// The render model over one graph: every list in a documented total
/// order (§8). Borrowed from the graph — the model is a view, and the
/// public functions serialize or narrate from it.
pub(super) struct Model<'a> {
    /// The repo node — the slug and the import instant.
    pub(super) repo: &'a Repo,
    /// One lane per epic, ascending epic id.
    pub(super) lanes: Vec<Lane<'a>>,
    /// Every decision, ascending id.
    pub(super) decisions: Vec<&'a Decision>,
    /// Every blocks edge as `(from, to)` full ids, the documented
    /// order.
    pub(super) edges: Vec<(String, String)>,
    /// What each decision gates, short ids, the adjacency's order.
    pub(super) gates: BTreeMap<DecisionId, Vec<String>>,
    /// An open task under an epic gated by an open decision carries the
    /// gating decisions' short ids — the "waits on" column.
    pub(super) waits: BTreeMap<TaskId, String>,
    /// The non-done tasks in (lane, id) order.
    pub(super) open_items: Vec<&'a Task>,
    /// The milestone epics in lane order with the board's M id.
    pub(super) milestones: Vec<(&'a Epic, String)>,
    /// The unplaced-decision and dropped-edge reports.
    pub(super) not_placed: Vec<String>,
    /// Per-kind counts over the four task kinds.
    pub(super) kind_state: KindState,
    /// Every task, all kinds.
    pub(super) task_count: usize,
    /// The event census — the activity panel's and the JSON twin's
    /// source (t84's D8).
    pub(super) activity: &'a EventCensus,
    /// The sidebar's per-repo reads: every registered repo with its
    /// epics, the rendered repo marked, an unreadable one reported.
    pub(super) sidebar: &'a [SidebarRepo],
    /// The rewards model slice (§15's display half, U63) — `None` on
    /// every disabled root, the fence: no section, no columns, no
    /// bytes that differ.
    pub(super) rewards: Option<RewardsModel>,
}

/// One lane: an epic, its tasks in id order, and the decisions
/// attached to it.
pub(super) struct Lane<'a> {
    /// The lane's epic.
    pub(super) epic: &'a Epic,
    /// The board's M id when the epic is a milestone.
    pub(super) milestone: Option<String>,
    /// The epic's tasks, ascending id.
    pub(super) tasks: Vec<&'a Task>,
    /// The decisions gating this epic or one of its tasks, decision id
    /// ascending.
    pub(super) decisions: Vec<&'a Decision>,
}

/// One resolve, one census read, the sidebar's per-repo reads and the
/// rewards read — the shared orchestration of every surface (t84's
/// D8; U63's join). The census happens after the resolve, over the
/// same slug, so an unregistered slug is refused before it; a census
/// failure fails the render (the panel is part of the page, never
/// silently omitted); the sidebar reads every registered repo
/// lock-free, reporting an unreadable repo rather than dropping it
/// (D3); the rewards read is the config first — absent or disabled
/// answers `None` and the totals/events reads are skipped (§15's
/// fence) — and a malformed config fails the render. The
/// `current_epic` number is the sidebar's current-epic marker —
/// `None` on every repo-scoped surface, the epic page's scoped
/// number.
pub(super) fn gather(
    root: &DataRoot,
    slug: &RepoSlug,
    current_epic: Option<u64>,
) -> Result<(Graph, EventCensus, Vec<SidebarRepo>, Option<RewardsModel>), String> {
    let graph = resolve(root, slug).map_err(|error| error.message)?;
    let census = root
        .read_repo_census(slug)
        .map_err(|error| ToolError::from(error).message)?;
    let sidebar = sidebar_repos(root, slug, &graph, current_epic)?;
    let rewards = gather_rewards(root)?;
    Ok((graph, census, sidebar, rewards))
}

/// The model over one graph, its census, the sidebar's reads and the
/// rewards slice — the Python tool's `build()` ported onto the domain
/// types, grown by t84's D8 and U63's join.
pub(super) fn build<'a>(
    graph: &'a Graph,
    census: &'a EventCensus,
    sidebar: &'a [SidebarRepo],
    rewards: Option<RewardsModel>,
) -> Result<Model<'a>, ToolError> {
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
        activity: census,
        sidebar,
        rewards,
    })
}
