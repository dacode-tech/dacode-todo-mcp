//! The `progress` HTML page (D20's second slice): the Python tool's
//! template ported to hand-built strings — the totals cards, the
//! activity panel, the milestone bar, the SVG dependency graph, the
//! open-items and decisions tables, the click-detail script (one
//! delegated listener reading the node data from an inert JSON
//! element inside the swap region — U54's staleness fix), the
//! project switcher the registry drives, the left sidebar with the
//! registered projects and their epics (t84's D4), the live-update
//! script t71's `serve` answers (the WebSocket push, view-aware since
//! U54: it re-fetches the current view through the lazy navigation's
//! fetch+swap instead of reloading the page, and its subscription
//! follows the view's repo — same-repo swaps never reconnect, a repo
//! change reconnects; inert on a static page, which is honest — the
//! static page is one repo's snapshot), and the lazy navigation
//! script itself (U54: the sidebar's epic links fetch their epic page
//! and swap the view in place — [`LAZY_SCRIPT`], carried by the epic
//! page too). Like the JSON document it is a pure
//! function of `(root, repo)` — the Python page's `date.today()`,
//! `head_sha()` and session-tracker overlay are deliberately absent
//! (none can golden-pin; D20's no-new-dependencies consequence forbids
//! the yaml read; post-g21 the store's open nodes are the live
//! picture) — so the page is golden-pinned as `tests/golden/page.v1`
//! and two runs over unchanged state are byte-identical by
//! construction.
//!
//! The stylesheet and the two scripts (the click-detail handler and
//! the lazy navigation) live in their own modules since U58's split —
//! `style` and `script` — paying the file-size debt this doc used to
//! record here; this file keeps the HTML assembly, the per-render
//! live-update init ([`live_script`]) and the static legend and
//! footer fragments.
//!
//! The switcher (the user's 2026-10-09 fold): one entry per registered
//! slug, the rendered repo marked current, every other slug linking to
//! `?repo=<slug>` — the URL shape the `serve` subcommand (t71) answers
//! with a fresh per-repo render. In the static file the links are
//! inert, which is honest: the static page is one repo's snapshot.
//! The switcher renders from the sidebar's registry walk — the page
//! pays the per-repo reads for the sidebar (D3), and the switcher
//! itself stays a registry nav. A registry read failure fails the
//! render — the switcher is part of the page, never silently omitted.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use todo_domain::{DecisionStatus, RepoSlug, Status};
use todo_store::DataRoot;

use super::model::{Model, build, gather};
use super::script::{LAZY_SCRIPT, SCRIPT};
use super::style::CSS;
use super::svg::render_svg;
use super::{card_html, decision_row_html, esc, task_row_html};

/// Every surface from one render — see [`super::outputs`].
pub struct Outputs {
    /// The HTML page (the golden-pinned `page.v1` surface).
    pub page: String,
    /// The JSON document — exactly the `--json` bytes, the page's
    /// machine-readable twin.
    pub json: String,
    /// The human summary.
    pub summary: String,
    /// The repo node's registered checkout path — the binary derives
    /// its default output directory from it (the CLI convention stays
    /// in the binary).
    pub checkout: PathBuf,
}

/// The `progress` HTML page: the Python template ported to hand-built
/// strings, with the project switcher and the left sidebar. A pure
/// function of `(root, repo)` — golden-pinned as `tests/golden/page.v1`.
pub fn page(root: &DataRoot, repo: &RepoSlug) -> Result<String, String> {
    let (graph, census, sidebar, rewards) = gather(root, repo, None)?;
    let model = build(&graph, &census, &sidebar, rewards).map_err(|error| error.message)?;
    page_text(&model)
}

/// The page from one model — the shared half of [`page`] and
/// [`super::outputs`], so a written page and a served page can never
/// disagree in shape.
pub(super) fn page_text(model: &Model) -> Result<String, String> {
    let slug = model.repo.slug.as_str();
    let slug_esc = esc(slug);
    let instant_esc = esc(model.repo.registered_at.as_str());
    let card_html = cards_html(model);
    let ms_bar = ms_bar_html(model);
    let svg = render_svg(model);
    let graph_nodes: usize = model
        .lanes
        .iter()
        .map(|lane| lane.tasks.len() + lane.decisions.len())
        .sum();
    let rows = open_item_rows(model);
    let decision_rows = decision_rows(model);
    let data = detail_data(model)?;
    // §15's fence: the rewards CSS rides the page only when the root's
    // config enables rewards — a disabled root's style block is
    // byte-identical to today's.
    let rewards_css = if model.rewards.is_some() {
        super::rewards::REWARDS_CSS
    } else {
        ""
    };
    // The est/actual column heads (§15): present only on an enabled
    // root, between the title and the waits-on heads — the row cells'
    // own order.
    let size_heads = if model.rewards.is_some() {
        super::rewards::SIZE_HEADS
    } else {
        ""
    };

    let mut page = String::with_capacity(12 * 1024);
    page.push_str("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\n");
    page.push_str(&format!("<title>{slug_esc} · progress</title>\n"));
    page.push_str(&format!(
        "<style>{}{}{}{}</style></head><body>\n",
        CSS,
        super::sidebar::SIDEBAR_CSS,
        super::panel::PANEL_CSS,
        rewards_css
    ));
    page.push_str(&switcher_html(model.sidebar, &model.repo.slug));
    page.push_str("\n<div class=\"cols\">\n");
    page.push_str(&super::sidebar::sidebar_html(model.sidebar));
    page.push_str("<main>\n");
    page.push_str(&format!("\n<h1>{slug_esc} — total progress</h1>\n"));
    page.push_str(&format!(
        "<div class=\"sub\">Rendered by <span class=\"mono\">todo-mcp progress</span> from \
         <span class=\"mono\">{slug_esc}</span>'s imported board — imported \
         <span class=\"mono\">{instant_esc}</span> (the repo node's registered_at — the \
         staleness signal: the page renders the last import, not the board as it is). The \
         markdown board remains the tracker of record; if this page disagrees with it, the \
         board wins. Machine-readable twin: <span class=\"mono\">progress.json</span>.</div>\n"
    ));
    page.push_str(&format!(
        "\n<h2>Totals</h2>\n<div class=\"cards\">{card_html}</div>\n"
    ));
    page.push_str(&super::panel::panel_html(model.activity));
    // The rewards section (§15, U63): after the activity panel — both
    // are event-derived — and only on an enabled root.
    if let Some(rewards) = &model.rewards {
        page.push_str(&super::rewards::section_html(rewards));
    }
    page.push_str(&format!(
        "\n<h2>Milestones (the board's roadmap via the server's epics)</h2>\n\
         <div class=\"msrow\">{ms_bar}</div>\n"
    ));
    page.push_str(&format!(
        "\n<h2>Dependency graph — {graph_nodes} nodes, {} edges</h2>\n\
         <div class=\"scroll\">{svg}</div>\n",
        model.edges.len()
    ));
    page.push_str(LEGEND);
    if !model.not_placed.is_empty() {
        page.push_str(&format!(
            "<p class=\"legend\"><b>Not placed:</b> {} — nodes the renderer cannot place are \
             reported, never dropped.</p>\n",
            esc(&model.not_placed.join(", "))
        ));
    }
    page.push_str("<div id=\"detail\"></div>\n");
    // The node data rides an inert element inside main — inside the
    // lazy swap region — so a swapped-in view always serves its own
    // fresh fields. A non-JS-typed script element never executes, on
    // load or on insertion, and the resident handler parses it per
    // click; keeping the data out of the executable script is also
    // what lets that script run exactly once per full load (a second
    // top-level `const DATA` would throw).
    page.push_str(&format!(
        "<script type=\"application/json\" id=\"detail-data\">{data}</script>\n"
    ));
    page.push_str(&format!(
        "\n<h2>Open items — {} rows</h2>\n\
         <table><thead><tr><th>id</th><th>kind</th><th>status</th><th>what</th>{size_heads}\
         <th>waits on</th></tr></thead>\n<tbody>{rows}</tbody></table>\n",
        model.open_items.len()
    ));
    page.push_str(&format!(
        "\n<h2>Decisions — {} rows</h2>\n\
         <table><thead><tr><th>id</th><th>status</th><th>what</th><th>gates</th>\
         <th>confidence</th></tr></thead>\n<tbody>{decision_rows}</tbody></table>\n",
        model.decisions.len()
    ));
    page.push_str(FOOTER);
    page.push_str("</main>\n</div>\n");
    page.push_str("<script>\n");
    page.push_str(SCRIPT);
    page.push_str(&live_script(slug));
    page.push_str(LAZY_SCRIPT);
    page.push_str("\n</script>\n</body></html>\n");
    Ok(page)
}

/// The live-update state (t71's D6, view-aware since U54): the
/// connection's holder and the repo it serves, initialized
/// server-side — the rendered page's slug is always the correct
/// initial subscription — with the connection itself opened by
/// [`LAZY_SCRIPT`]'s `connectWs()` (a function declaration hoists
/// within the one script element, so calling it above its source is
/// sound). The subscription follows the view: same-repo swaps never
/// reconnect, and a lazy navigation that crossed repos closes the old
/// connection before opening the new one — `followView()` in
/// [`LAZY_SCRIPT`], after every fetch+swap. The try/catch inside
/// `connectWs` degrades silently on a static `file://` page — an
/// empty `location.host` makes the URL invalid, the constructor
/// throws, the catch swallows it. The script rides every render — the
/// repo page's and the epic page's alike: a static page simply never
/// receives a push. The slug's charset needs no JS string escaping.
pub(super) fn live_script(slug: &str) -> String {
    format!("\nlet liveWs = null, liveRepo = '{slug}';\nconnectWs();")
}

/// The switcher: one entry per registered slug, the rendered repo as
/// the marked non-link (a self-link is a no-op dressed as navigation),
/// every other slug linking to `?repo=<slug>` — the URL shape the
/// `serve` subcommand answers. Renders from the sidebar's registry
/// walk, so the switcher and the sidebar can never disagree about
/// membership or order. The epic page reuses it unchanged: the
/// switcher marks the current repo, never the current epic.
pub(super) fn switcher_html(repos: &[super::sidebar::SidebarRepo], current: &RepoSlug) -> String {
    let mut out = String::from("<nav class=\"switch\">");
    for repo in repos {
        let name = esc(repo.slug.as_str());
        if repo.slug == *current {
            out.push_str(&format!("<span class=\"cur\">{name}</span>"));
        } else {
            out.push_str(&format!("<a href=\"?repo={name}\">{name}</a>"));
        }
    }
    out.push_str("</nav>");
    out
}

/// The totals cards — the Python tool's seven, each carrying its own
/// denominator because "progress" depends entirely on what is counted.
fn cards_html(model: &Model) -> String {
    let ms_done = model
        .milestones
        .iter()
        .filter(|(epic, _)| epic.status == Status::Closed)
        .count();
    let ms_total = model.milestones.len();
    let blocked = model
        .open_items
        .iter()
        .filter(|task| task.status == Status::Blocked)
        .count();
    let signed = model
        .decisions
        .iter()
        .filter(|decision| decision.status == DecisionStatus::Signed)
        .count();
    let decisions_total = model.decisions.len();
    let kind = &model.kind_state;
    let cards: [(&str, String, String, &str); 7] = [
        (
            "Milestones",
            format!("{ms_done}/{ms_total}"),
            pct(ms_done, ms_total),
            "the board's milestone table via the server's epics — the roadmap measure",
        ),
        (
            "Tasks",
            format!("{}/{}", kind.task.done, kind.task.total),
            pct(kind.task.done, kind.task.total),
            "the board's §4 follow-ups, one unit each",
        ),
        (
            "Gates",
            format!("{}/{}", kind.gate.done, kind.gate.total),
            pct(kind.gate.done, kind.gate.total),
            "milestone exit gates",
        ),
        (
            "Findings",
            format!("{}/{}", kind.finding.done, kind.finding.total),
            pct(kind.finding.done, kind.finding.total),
            "an open finding is a fact written down, not a defect",
        ),
        (
            "Debt",
            format!("{}/{} closed", kind.debt.done, kind.debt.total),
            pct(kind.debt.done, kind.debt.total),
            "the board's §5, acknowledged deferrals",
        ),
        (
            "Decisions",
            format!("{signed}/{decisions_total} signed"),
            pct(signed, decisions_total),
            "the decisions file's dossiers; an open one gates work",
        ),
        (
            "Blocked",
            format!("{blocked}"),
            if blocked == 0 {
                "0%".to_owned()
            } else {
                "—".to_owned()
            },
            "tasks whose status is blocked",
        ),
    ];
    cards
        .iter()
        .map(|(title, value, percent, note)| card_html(title, value, percent, note))
        .collect()
}

/// The milestone bar: one chip per milestone epic, done chips marked.
fn ms_bar_html(model: &Model) -> String {
    let mut out = String::new();
    for (epic, m_id) in &model.milestones {
        let done = epic.status == Status::Closed;
        let label = if done { "done" } else { epic.status.as_str() };
        out.push_str(&format!(
            "<div class=\"ms{}\" title=\"{} — {}\"><b>{}</b><span>{}</span></div>",
            if done { " msdone" } else { "" },
            esc(&epic.title),
            esc(epic.status.as_str()),
            esc(m_id),
            esc(label)
        ));
    }
    out
}

/// The open-items table's rows — the shared row shape (U55's hoist).
fn open_item_rows(model: &Model) -> String {
    model
        .open_items
        .iter()
        .map(|task| task_row_html(model, task))
        .collect()
}

/// The decisions table's rows — the shared row shape (U55's hoist).
fn decision_rows(model: &Model) -> String {
    model
        .decisions
        .iter()
        .map(|decision| decision_row_html(model, decision))
        .collect()
}

/// The click-detail block the resident handler parses per click: one
/// node per placed task or decision (keyed by full id — the SVG's
/// `data-id`), the edges as full-id pairs. Serialized compactly, then
/// guarded: every `</` in the text becomes `<\/` — a valid JSON
/// escape that cannot close the inert element the block rides (the
/// HTML parser ends a script element at the first `</script` sequence
/// whatever the element's type, and `JSON.parse` reads the escaped
/// form back as `/`).
fn detail_data(model: &Model) -> Result<String, String> {
    let mut nodes: BTreeMap<String, DetailNodeDto> = BTreeMap::new();
    for lane in &model.lanes {
        for task in &lane.tasks {
            nodes.insert(
                task.id.to_string(),
                DetailNodeDto {
                    id: task.id.to_string(),
                    kind: task.kind.as_str().to_owned(),
                    status: task.status.as_str().to_owned(),
                    title: task.title.clone(),
                    verify: task.verify.clone(),
                    evidence: task.evidence.clone(),
                    note: task.note.clone(),
                    resolution: None,
                },
            );
        }
        for decision in &lane.decisions {
            nodes.insert(
                decision.id.to_string(),
                DetailNodeDto {
                    id: decision.id.to_string(),
                    kind: "decision".to_owned(),
                    status: decision.status.as_str().to_owned(),
                    title: decision.title.clone(),
                    verify: None,
                    evidence: None,
                    note: None,
                    resolution: decision.resolution.clone(),
                },
            );
        }
    }
    let edges: Vec<DetailEdgeDto> = model
        .edges
        .iter()
        .map(|(from, to)| DetailEdgeDto {
            from: from.clone(),
            to: to.clone(),
        })
        .collect();
    let data = serde_json::to_string(&DetailData { nodes, edges })
        .map_err(|error| format!("the detail block did not serialize: {error}"))?;
    Ok(data.replace("</", "<\\/"))
}

/// The detail block's node entry — the click box's fields only;
/// created/updated are deliberately absent (the Python tool's own
/// rule: they are the import instant, and embedding them would move
/// page bytes the box never shows).
#[derive(Serialize)]
struct DetailNodeDto {
    /// The node's full id.
    id: String,
    /// The task's kind, or `decision`.
    kind: String,
    /// The node's status.
    status: String,
    /// The node's title.
    title: String,
    /// The task's verify method, when set.
    verify: Option<String>,
    /// The task's closure evidence, when set.
    evidence: Option<String>,
    /// The task's blocked/deferred note, when set.
    note: Option<String>,
    /// The decision's resolution, when set.
    resolution: Option<String>,
}

/// The detail block's edge entry.
#[derive(Serialize)]
struct DetailEdgeDto {
    /// The blocker's full id.
    from: String,
    /// The held-back node's full id.
    to: String,
}

/// The detail block.
#[derive(Serialize)]
struct DetailData {
    /// The nodes, keyed by full id.
    nodes: BTreeMap<String, DetailNodeDto>,
    /// The edges.
    edges: Vec<DetailEdgeDto>,
}

/// The percentage with its denominator applied: integer arithmetic,
/// round-half-up — no floating point in a rendered percentage (the
/// Python tool's `round()` is banker's rounding; a recorded
/// divergence, 1/8 shows 13% here and 12% there).
pub(super) fn pct(a: usize, b: usize) -> String {
    let Some(value) = (100 * a + b / 2).checked_div(b) else {
        return "—".to_owned();
    };
    format!("{value}%")
}

/// The graph legend — static.
const LEGEND: &str = "<p class=\"legend\">\n<span><i class=\"sw\" \
style=\"background:#1f6f43\"></i>done / signed</span>\n<span><i class=\"sw\" \
style=\"background:#8a5a00\"></i>open / in work</span>\n<span><i class=\"sw\" \
style=\"background:#9b1c1c\"></i>blocked</span>\n<span>— the lanes are the epics; a decision \
sits in the lane of the epic it gates</span>\n<span>▢ task ▣ gate ◆ finding ▲ debt ✦ \
decision</span>\n<span>click a node for its fields and edges</span>\n</p>\n";

/// The footer — the percentages-reading paragraph and the staleness
/// signal (the Python tool's tracker-citations sentence drops with the
/// overlay).
const FOOTER: &str = "<footer>\nReading the percentages: each card states its own \
denominator, because \"progress\" depends\nentirely on what is counted. <b>Milestones</b> \
measure the roadmap. <b>Tasks</b> measure the\nwork decomposed so far, which is <em>not</em> \
the whole project. <b>Findings</b> and\n<b>debt</b> are records rather than deliverables. The \
import instant above is the staleness\nsignal: the page renders the last import, not the board \
as it is now.\n</footer>\n";

#[cfg(test)]
mod tests {
    use super::*;

    /// The percentage helper: integer round-half-up, `—` on a zero
    /// denominator (the Python tool's banker's rounding is the
    /// recorded divergence — 1/8 shows 13% here).
    #[test]
    fn pct_rounds_half_up_and_dashes_on_zero() {
        assert_eq!(pct(1, 8), "13%");
        assert_eq!(pct(0, 0), "—");
        assert_eq!(pct(3, 3), "100%");
        assert_eq!(pct(0, 4), "0%");
    }

    /// The five entities, and nothing else — the Python
    /// `html.escape(quote=True)` equivalent.
    #[test]
    fn esc_replaces_the_five_entities() {
        assert_eq!(esc("<b>&\"'"), "&lt;b&gt;&amp;&quot;&#x27;");
        assert_eq!(esc("plain text · §2"), "plain text · §2");
    }
}
