//! The `progress --json` document (v1, golden-pinned as
//! `tests/golden/progress.v1`): the Python tool's store-derived fields
//! — its `generated` (a clock read), `head` (a git subprocess) and
//! `tracker` (the gitignored session yaml) fields are deliberately
//! absent, because none can golden-pin and D20's no-new-dependencies
//! consequence forbids the yaml read. Field order is declaration order
//! (D14's discipline); short ids (the part after `:`) wherever the
//! Python tool used them, the full epic id for a node's lane.

use serde::Serialize;

use todo_domain::{DecisionStatus, RepoSlug};
use todo_store::{DataRoot, EventCensus};

use super::{Model, build, gather};

/// The `progress --json` document: the golden-pinned v1 JSON, one
/// trailing newline — exactly the bytes the subcommand prints.
pub fn json(root: &DataRoot, repo: &RepoSlug) -> Result<String, String> {
    let (graph, census, sidebar, rewards) = gather(root, repo, None)?;
    let model = build(&graph, &census, &sidebar, rewards).map_err(|error| error.message)?;
    json_text(&model)
}

/// The document from one model — the shared half of [`json`] and
/// [`super::outputs`], so the JSON twin a default run writes beside
/// the page is byte-identical to the `--json` output.
pub(super) fn json_text(model: &Model) -> Result<String, String> {
    let mut text = serde_json::to_string_pretty(&document(model))
        .map_err(|error| format!("the progress document did not serialize: {error}"))?;
    text.push('\n');
    Ok(text)
}

/// The `progress --json` document (v1, golden-pinned as
/// `tests/golden/progress.v1`).
#[derive(Serialize)]
struct ProgressJson {
    /// The rendered repository's slug.
    repo: String,
    /// The repo node's registration instant — the staleness signal:
    /// the document renders the last import, not the board as it is.
    import_instant: String,
    /// The roll-up: every count the summary prints.
    totals: TotalsDto,
    /// The milestone epics in lane order, with the board's M id.
    milestones: Vec<MilestoneDto>,
    /// The non-done tasks in (lane, id) order.
    open_items: Vec<OpenItemDto>,
    /// Every decision, ascending id, with what it gates.
    decisions: Vec<DecisionDto>,
    /// The graph: one node per placed task or decision, the edges.
    graph: GraphDto,
    /// The store-derived activity summary — the census verbatim (t84's
    /// D6), the page's activity panel's machine-readable twin.
    activity: EventCensus,
    /// The rewards block (§15's display half, U63) — the totals, the
    /// recent grants and the reflections, the section's twin. Absent
    /// on every disabled root: the fence's byte-identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    rewards: Option<super::rewards::RewardsJson>,
}

/// The totals block.
#[derive(Serialize)]
struct TotalsDto {
    /// The lane count (one per epic).
    epics: usize,
    /// The milestone epics' done/total.
    milestones: MilestoneTotalsDto,
    /// Every task, all kinds.
    tasks: usize,
    /// Per-kind counts over the four task kinds.
    by_kind: super::KindState,
    /// The open items' short ids, the `open_items` list's order.
    open_items: Vec<String>,
    /// The decisions' signed/total.
    decisions: DecisionTotalsDto,
    /// The unplaced-decision and dropped-edge reports — never dropped.
    not_placed: Vec<String>,
}

/// The milestone totals.
#[derive(Serialize)]
struct MilestoneTotalsDto {
    /// Milestone epics whose status is closed.
    done: usize,
    /// Every milestone epic.
    total: usize,
}

/// The decisions' totals.
#[derive(Serialize)]
struct DecisionTotalsDto {
    /// Decisions whose status is signed.
    signed: usize,
    /// Every decision.
    total: usize,
}

/// One milestone row.
#[derive(Serialize)]
struct MilestoneDto {
    /// The board's M id, recovered from the epic's provenance
    /// description.
    id: String,
    /// The epic's title.
    title: String,
    /// The epic's status.
    status: String,
}

/// One open-item row.
#[derive(Serialize)]
struct OpenItemDto {
    /// The task's short id.
    id: String,
    /// The task's kind.
    kind: String,
    /// The task's status.
    status: String,
    /// The task's title.
    title: String,
    /// The planned size (§15's estimate), wire spelling. The outer
    /// `None` is the disabled root — the key is absent, the fence's
    /// byte-identity — and the inner one an enabled root's absent
    /// estimate, spelled `null` (the house document style: an absent
    /// value is `null`, never a missing key).
    #[serde(skip_serializing_if = "Option::is_none")]
    estimate: Option<Option<String>>,
    /// The actual size — the task's latest grant's size, wire
    /// spelling. The outer/inner `None` reading is
    /// [`Self::estimate`]'s.
    #[serde(skip_serializing_if = "Option::is_none")]
    actual: Option<Option<String>>,
    /// The open decisions gating the task's epic, short ids — empty
    /// when nothing gates it.
    waits_on: String,
}

/// One decision row.
#[derive(Serialize)]
struct DecisionDto {
    /// The decision's short id.
    id: String,
    /// The decision's status.
    status: String,
    /// The decision's title.
    title: String,
    /// What the decision gates, short ids, the adjacency's order.
    gates: Vec<String>,
}

/// The graph block.
#[derive(Serialize)]
struct GraphDto {
    /// One node per placed task or decision, lane by lane (epic
    /// ascending), tasks then decisions within a lane, id ascending.
    nodes: Vec<NodeDto>,
    /// Every blocks edge, short ids: decision-sourced first (decision
    /// id ascending, targets in the adjacency's canonical order), then
    /// task-sourced (task id ascending).
    edges: Vec<EdgeDto>,
}

/// One graph node.
#[derive(Serialize)]
struct NodeDto {
    /// The node's short id.
    id: String,
    /// The task's kind, or `decision`.
    kind: String,
    /// The node's status.
    status: String,
    /// The lane's full epic id.
    lane: String,
}

/// One graph edge.
#[derive(Serialize)]
struct EdgeDto {
    /// The blocker's short id.
    from: String,
    /// The held-back node's short id.
    to: String,
}

/// The document from the model.
fn document(model: &Model) -> ProgressJson {
    let milestones: Vec<MilestoneDto> = model
        .milestones
        .iter()
        .map(|(epic, id)| MilestoneDto {
            id: id.clone(),
            title: epic.title.clone(),
            status: epic.status.as_str().to_owned(),
        })
        .collect();
    let open_items: Vec<OpenItemDto> = model
        .open_items
        .iter()
        .map(|task| {
            // §15's fence: the size fields ride the rows only on an
            // enabled root — a disabled root's rows are byte-identical
            // to today's.
            let (estimate, actual) = match &model.rewards {
                Some(rewards) => (
                    Some(task.estimate.map(|size| size.as_str().to_owned())),
                    Some(
                        rewards
                            .actuals
                            .get(&task.id)
                            .map(|size| size.as_str().to_owned()),
                    ),
                ),
                None => (None, None),
            };
            OpenItemDto {
                id: super::short_id(&task.id.to_string()),
                kind: task.kind.as_str().to_owned(),
                status: task.status.as_str().to_owned(),
                title: task.title.clone(),
                estimate,
                actual,
                waits_on: model.waits.get(&task.id).cloned().unwrap_or_default(),
            }
        })
        .collect();
    let decisions: Vec<DecisionDto> = model
        .decisions
        .iter()
        .map(|decision| DecisionDto {
            id: super::short_id(&decision.id.to_string()),
            status: decision.status.as_str().to_owned(),
            title: decision.title.clone(),
            gates: model.gates.get(&decision.id).cloned().unwrap_or_default(),
        })
        .collect();
    let nodes: Vec<NodeDto> = model
        .lanes
        .iter()
        .flat_map(|lane| {
            lane.tasks
                .iter()
                .map(|task| NodeDto {
                    id: super::short_id(&task.id.to_string()),
                    kind: task.kind.as_str().to_owned(),
                    status: task.status.as_str().to_owned(),
                    lane: lane.epic.id.to_string(),
                })
                .chain(lane.decisions.iter().map(|decision| NodeDto {
                    id: super::short_id(&decision.id.to_string()),
                    kind: "decision".to_owned(),
                    status: decision.status.as_str().to_owned(),
                    lane: lane.epic.id.to_string(),
                }))
        })
        .collect();
    let edges: Vec<EdgeDto> = model
        .edges
        .iter()
        .map(|(from, to)| EdgeDto {
            from: super::short_id(from),
            to: super::short_id(to),
        })
        .collect();
    ProgressJson {
        repo: model.repo.slug.as_str().to_owned(),
        import_instant: model.repo.registered_at.as_str().to_owned(),
        totals: TotalsDto {
            epics: model.lanes.len(),
            milestones: MilestoneTotalsDto {
                done: milestones
                    .iter()
                    .filter(|row| row.status == "closed")
                    .count(),
                total: milestones.len(),
            },
            tasks: model.task_count,
            by_kind: model.kind_state.clone(),
            open_items: open_items.iter().map(|row| row.id.clone()).collect(),
            decisions: DecisionTotalsDto {
                signed: model
                    .decisions
                    .iter()
                    .filter(|decision| decision.status == DecisionStatus::Signed)
                    .count(),
                total: model.decisions.len(),
            },
            not_placed: model.not_placed.clone(),
        },
        milestones,
        open_items,
        decisions,
        graph: GraphDto { nodes, edges },
        activity: model.activity.clone(),
        rewards: model.rewards.as_ref().map(super::rewards::json),
    }
}
