//! The dependency-graph SVG (the Python tool's `render_svg`
//! ported): one lane column per epic, the nodes stacked in id order
//! (tasks then decisions), the edges as bezier paths between the node
//! centers' right and left edges. Coordinates are integers except the
//! bezier midpoint, computed in f64 and formatted with Rust's shortest
//! round-trip (deterministic; the Python's `123.0` spelling becomes
//! `123` — a recorded divergence, structure-identical).

use std::collections::BTreeMap;

use super::{Model, esc, short, short_id};

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
        let (Some(&(x1, y1)), Some(&(x2, y2))) = (pos.get(from), pos.get(to)) else {
            continue;
        };
        let (x1e, y1e, x2e, y2e) = (x1 + 186, y1 + 15, x2, y2 + 15);
        let mx = (x1e + x2e) as f64 / 2.0;
        out.push_str(&format!(
            "<path class=\"edge\" d=\"M{x1e},{y1e} C{mx},{y1e} {mx},{y2e} {x2e},{y2e}\" \
             marker-end=\"url(#arrg)\"><title>{} gates {}</title></path>",
            esc(&short_id(from)),
            esc(&short_id(to))
        ));
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
