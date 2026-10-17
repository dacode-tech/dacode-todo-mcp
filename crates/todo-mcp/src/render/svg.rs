//! The dependency-graph SVG (the Python tool's `render_svg`
//! ported): one lane column per epic, the nodes stacked in id order
//! (tasks then decisions), the edges as bezier paths between the node
//! centers' right and left edges. Coordinates are integers except the
//! bezier midpoint, computed in f64 and formatted with Rust's shortest
//! round-trip (deterministic; the Python's `123.0` spelling becomes
//! `123` — a recorded divergence, structure-identical).

use std::collections::{BTreeMap, BTreeSet};

use super::model::{Lane, Model};
use super::{esc, short, short_id};

/// The SVG dependency graph — the Python tool's `render_svg` ported:
/// one lane column per epic, the nodes stacked in id order (tasks then
/// decisions), the edges as bezier paths between the node centers'
/// right and left edges. Coordinates are integers except the bezier
/// midpoint, computed in f64 and formatted with Rust's shortest
/// round-trip (deterministic; the Python's `123.0` spelling becomes
/// `123` — a recorded divergence, structure-identical).
pub(super) fn render_svg(model: &Model) -> String {
    const PAD: i64 = 54;
    const COLW: i64 = 214;
    const ROWH: i64 = 44;
    let mut pos: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for (i, lane) in model.lanes.iter().enumerate() {
        let x = PAD + i as i64 * COLW;
        let mut y = PAD + 34;
        for task in &lane.tasks {
            pos.insert(task.id.to_string(), (x, y));
            y += ROWH;
        }
        for decision in &lane.decisions {
            pos.insert(decision.id.to_string(), (x, y));
            y += ROWH;
        }
    }
    let width = PAD * 2 + model.lanes.len() as i64 * COLW;
    let tallest = model
        .lanes
        .iter()
        .map(|lane| (lane.tasks.len() + lane.decisions.len()) as i64)
        .max()
        .unwrap_or(0);
    let height = PAD + 60 + tallest * ROWH;
    let mut out = String::new();
    out.push_str(&format!(
        "<svg id=\"graph\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" \
         height=\"{height}\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" \
         aria-label=\"task dependency graph\">"
    ));
    out.push_str(DEFS);
    for (i, lane) in model.lanes.iter().enumerate() {
        let x = PAD + i as i64 * COLW;
        let header = match &lane.milestone {
            Some(m) => format!("{m} · {}", short(&lane.epic.title, 24)),
            None => short(&lane.epic.title, 24),
        };
        out.push_str(&format!(
            "<text id=\"lane-{}\" class=\"lane\" x=\"{x}\" y=\"{}\">{}</text>",
            lane.epic.id.number(),
            PAD - 8,
            esc(&header)
        ));
        out.push_str(&format!(
            "<line class=\"laneline\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
            x - 10,
            PAD + 4,
            x - 10,
            height - 20
        ));
    }
    for (from, to) in &model.edges {
        svg_edge(&mut out, &pos, from, to);
    }
    for lane in &model.lanes {
        for task in &lane.tasks {
            svg_node(
                &mut out,
                &pos,
                &task.id.to_string(),
                task.kind.as_str(),
                task.status.as_str(),
                &task.title,
            );
        }
        for decision in &lane.decisions {
            svg_node(
                &mut out,
                &pos,
                &decision.id.to_string(),
                "decision",
                decision.status.as_str(),
                &decision.title,
            );
        }
    }
    out.push_str("</svg>");
    out
}

/// One edge — the bezier from the source node's right edge to the
/// target's left, the control points vertical at the horizontal
/// midpoint. Both endpoints must be in `pos`; a missing one skips
/// the edge (the caller's filter decides which edges a graph draws,
/// this only draws). Returns whether the edge drew.
fn svg_edge(out: &mut String, pos: &BTreeMap<String, (i64, i64)>, from: &str, to: &str) -> bool {
    let (Some(&(x1, y1)), Some(&(x2, y2))) = (pos.get(from), pos.get(to)) else {
        return false;
    };
    let (x1e, y1e, x2e, y2e) = (x1 + 186, y1 + 15, x2, y2 + 15);
    let mx = (x1e + x2e) as f64 / 2.0;
    out.push_str(&format!(
        "<path class=\"edge\" d=\"M{x1e},{y1e} C{mx},{y1e} {mx},{y2e} {x2e},{y2e}\" \
         marker-end=\"url(#arrg)\"><title>{} gates {}</title></path>",
        esc(&short_id(from)),
        esc(&short_id(to))
    ));
    true
}

/// The epic page's dependency graph (the user's 2026-10-17 request):
/// the lane's nodes in one column — the repo page's lane shape, tasks
/// then decisions in id order — beside an "outside this epic" column
/// of the external endpoints the lane's edges wire to, id-sorted (an
/// ordering the repo page's lanes do not need: the outside set is
/// mixed kinds by nature). The edges are the repo page's own drawing
/// filtered to the epic's wiring: an edge draws when at least one
/// endpoint is in the lane, and a crossing edge's external end joins
/// the outside column — an internal edge's target is the lane's own,
/// never an outside node. An edge endpointing an epic (§5.1 lets a
/// decision block one) passes the filter but draws nothing: the epic
/// is no task or decision, sits in no facts map and no lane, and
/// `svg_edge` skips it — the repo page's own degradation. Backward
/// edges (a lane node gating an outside node) render as the repo
/// page's same-lane edges do: the curve loops out and back.
pub(super) fn render_epic_svg(model: &Model, lane: &Lane) -> EpicGraph {
    const PAD: i64 = 54;
    const COLW: i64 = 214;
    const ROWH: i64 = 44;
    // The lane's membership — the edge filter.
    let mut lane_ids: BTreeSet<String> = BTreeSet::new();
    for task in &lane.tasks {
        lane_ids.insert(task.id.to_string());
    }
    for decision in &lane.decisions {
        lane_ids.insert(decision.id.to_string());
    }
    // Every node's facts, one walk of the lanes — the outside
    // endpoints' lookups. A node no lane holds is not drawn and its
    // edges skip in `svg_edge`, the repo page's own degradation.
    let mut facts: BTreeMap<String, (&str, &str, &str)> = BTreeMap::new();
    for other in &model.lanes {
        for task in &other.tasks {
            facts.insert(
                task.id.to_string(),
                (
                    task.kind.as_str(),
                    task.status.as_str(),
                    task.title.as_str(),
                ),
            );
        }
        for decision in &other.decisions {
            facts.insert(
                decision.id.to_string(),
                (
                    "decision",
                    decision.status.as_str(),
                    decision.title.as_str(),
                ),
            );
        }
    }
    let mut outside: BTreeMap<String, (&str, &str, &str)> = BTreeMap::new();
    for (from, to) in &model.edges {
        let (from_in, to_in) = (lane_ids.contains(from), lane_ids.contains(to));
        // Internal edges (both endpoints in the lane) and unrelated
        // ones (neither) contribute no outside endpoint; a crossing
        // edge contributes its external end — never its lane end.
        let other = match (from_in, to_in) {
            (true, false) => to,
            (false, true) => from,
            _ => continue,
        };
        if let Some(node) = facts.get(other) {
            outside.insert(other.clone(), *node);
        }
    }
    // The layout: the outside column at the left, the lane at the
    // right — the repo page's own geometry.
    let mut pos: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    let outside_x = PAD;
    let mut y = PAD + 34;
    for id in outside.keys() {
        pos.insert(id.clone(), (outside_x, y));
        y += ROWH;
    }
    let lane_x = PAD + COLW;
    let mut y = PAD + 34;
    for task in &lane.tasks {
        pos.insert(task.id.to_string(), (lane_x, y));
        y += ROWH;
    }
    for decision in &lane.decisions {
        pos.insert(decision.id.to_string(), (lane_x, y));
        y += ROWH;
    }
    let lane_count = (lane.tasks.len() + lane.decisions.len()) as i64;
    let height = PAD + 60 + outside.len().max(lane_count as usize) as i64 * ROWH;
    let width = PAD * 2 + 2 * COLW;
    let mut out = String::new();
    out.push_str(&format!(
        "<svg id=\"graph\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" \
         height=\"{height}\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" \
         aria-label=\"task dependency graph\">"
    ));
    out.push_str(DEFS);
    out.push_str(&format!(
        "<text id=\"lane-outside\" class=\"lane\" x=\"{outside_x}\" y=\"{}\">outside this epic</text>",
        PAD - 8
    ));
    out.push_str(&format!(
        "<line class=\"laneline\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
        outside_x - 10,
        PAD + 4,
        outside_x - 10,
        height - 20
    ));
    let header = match &lane.milestone {
        Some(m) => format!("{m} · {}", short(&lane.epic.title, 24)),
        None => short(&lane.epic.title, 24),
    };
    out.push_str(&format!(
        "<text id=\"lane-{}\" class=\"lane\" x=\"{lane_x}\" y=\"{}\">{}</text>",
        lane.epic.id.number(),
        PAD - 8,
        esc(&header)
    ));
    out.push_str(&format!(
        "<line class=\"laneline\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
        lane_x - 10,
        PAD + 4,
        lane_x - 10,
        height - 20
    ));
    let mut edges = 0;
    for (from, to) in &model.edges {
        if lane_ids.contains(from) || lane_ids.contains(to) {
            edges += svg_edge(&mut out, &pos, from, to) as usize;
        }
    }
    for (id, (kind, status, title)) in &outside {
        svg_node(&mut out, &pos, id, kind, status, title);
    }
    for task in &lane.tasks {
        svg_node(
            &mut out,
            &pos,
            &task.id.to_string(),
            task.kind.as_str(),
            task.status.as_str(),
            &task.title,
        );
    }
    for decision in &lane.decisions {
        svg_node(
            &mut out,
            &pos,
            &decision.id.to_string(),
            "decision",
            decision.status.as_str(),
            &decision.title,
        );
    }
    out.push_str("</svg>");
    EpicGraph {
        svg: out,
        nodes: outside.len() + lane.tasks.len() + lane.decisions.len(),
        edges,
    }
}

/// The epic-scoped graph's render and its counts — the SVG plus the
/// numbers the page's section header carries.
pub(super) struct EpicGraph {
    /// The SVG text.
    pub(super) svg: String,
    /// The drawn nodes: the lane's plus the outside endpoints.
    pub(super) nodes: usize,
    /// The drawn edges: the internal plus the crossing ones.
    pub(super) edges: usize,
}

/// One SVG node group — the rect, the hover title (id, kind, status,
/// the full title on a second line), the id and label texts, the
/// status dot.
fn svg_node(
    out: &mut String,
    pos: &BTreeMap<String, (i64, i64)>,
    id: &str,
    kind: &str,
    status: &str,
    title: &str,
) {
    let Some(&(x, y)) = pos.get(id) else {
        return;
    };
    let (border, fill) = status_color(status);
    let node_short = short_id(id);
    let id_esc = esc(id);
    let title_text = format!(
        "{} · {} · {}\n{}",
        esc(&node_short),
        esc(kind),
        esc(status),
        esc(title)
    );
    let mark = esc(kind_mark(kind));
    let short_esc = esc(&node_short);
    let label_esc = esc(&short(title, 22));
    out.push_str(&format!(
        "<g class=\"node\" data-id=\"{id_esc}\" transform=\"translate({x},{y})\">\
         <rect rx=\"7\" width=\"186\" height=\"30\" fill=\"{fill}\" stroke=\"{border}\" \
         stroke-width=\"1.6\"/><title>{title_text}</title>\
         <text class=\"nid\" x=\"8\" y=\"19\" fill=\"{border}\">{mark} {short_esc}</text>\
         <text class=\"nlabel\" x=\"46\" y=\"19\">{label_esc}</text>\
         <circle class=\"dot\" cx=\"176\" cy=\"8\" r=\"4\" fill=\"{border}\"/></g>"
    ));
}

/// The status color pair `(border, fill)` — the Python tool's
/// `STATUS_COLOR` with its `node_status` fallback folded in: an
/// unknown status reads as open.
fn status_color(status: &str) -> (&'static str, &'static str) {
    match status {
        "closed" | "wontfix" | "signed" => ("#1f6f43", "#e6f4ec"),
        "in_work" | "open" | "deferred" => ("#8a5a00", "#fdf3e0"),
        "blocked" => ("#9b1c1c", "#fdeaea"),
        _ => ("#8a5a00", "#fdf3e0"),
    }
}

/// The kind's mark — the Python tool's `KIND_MARK` (its `milestone`
/// entry was unreachable: no node carries that kind, so it does not
/// port).
fn kind_mark(kind: &str) -> &'static str {
    match kind {
        "task" => "▢",
        "gate" => "▣",
        "finding" => "◆",
        "debt" => "▲",
        "decision" => "✦",
        _ => "•",
    }
}

/// The SVG's marker definitions — the Python tool's `defs` verbatim.
const DEFS: &str = "<defs><marker id=\"arr\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" \
markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0,0 L10,5 L0,10 \
z\" fill=\"#8a8f98\"/></marker><marker id=\"arrg\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" \
markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0,0 L10,5 L0,10 \
z\" fill=\"#2f6f4f\"/></marker></defs>";
