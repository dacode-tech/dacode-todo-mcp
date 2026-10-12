//! The epic-scoped page (E12's per-epic arc, U52): one epic's lane as
//! a page — the epic's header (status, milestone, description), its
//! counts, its tasks with statuses and waits, and its gating
//! decisions — for the serve route `/repo/<slug>/epic/<number>` (U53
//! wires the route; the render itself is a pure function reachable
//! in-process like the repo page's `page()` is, so the golden and the
//! twice-run test drive it without a server). The same [`gather`] and
//! [`build`] halves the repo page uses, so the two surfaces can never
//! disagree about the model; the requested epic's lane is selected by
//! its number within the repo, and a number the graph does not hold is
//! a named error, never an empty page. The sidebar marks the requested
//! epic current (the switcher precedent: the marked non-link) while
//! the switcher stays the repo page's, unchanged; the live-update
//! script rides every render for the same reason it rides the repo
//! page's. No click-detail block: the tables are plain, and simplicity
//! wins. No not_placed report either: a decision the model cannot
//! place gates nothing, so it sits in no epic's lane — the repo page's
//! report is its one home.

use todo_domain::{DecisionStatus, RepoSlug, Status, TaskKind};
use todo_store::DataRoot;

use super::page::{CSS, live_script, pct, switcher_html};
use super::{Lane, Model, build, counts_for, esc, gather, is_done, short, short_id};

/// The epic-scoped HTML page (E12's per-epic surface): one epic's
/// lane — the epic's header, its counts, its tasks with statuses and
/// waits, and its gating decisions — with the sidebar marking that
/// epic current and the repo page's switcher unchanged. A pure
/// function of `(root, repo, epic)` (§8): the same `gather` and
/// `build` halves the repo page uses, with the requested lane selected
/// by the epic's number within the repo — the number the serve route
/// `/repo/<slug>/epic/<number>` parses (U53). An epic number the
/// repo's graph does not hold is a named error, never an empty page;
/// an unknown repo fails through the same resolve discipline as the
/// repo page. Golden-pinned as `tests/golden/epic_page.v1`.
pub fn epic_page(root: &DataRoot, repo: &RepoSlug, epic: u64) -> Result<String, String> {
    let (graph, census, sidebar) = gather(root, repo, Some(epic))?;
    let model = build(&graph, &census, &sidebar).map_err(|error| error.message)?;
    let Some(lane) = model
        .lanes
        .iter()
        .find(|lane| lane.epic.id.number() == epic)
    else {
        return Err(format!(
            "the repo {} holds no epic number {epic} — refusing to render an empty epic page",
            model.repo.slug.as_str()
        ));
    };
    Ok(epic_page_text(&model, lane))
}

/// The epic page from one model and its selected lane — the render
/// half, infallible: every list it walks is already in the model's
/// documented order.
fn epic_page_text(model: &Model, lane: &Lane) -> String {
    let slug = model.repo.slug.as_str();
    let slug_esc = esc(slug);
    let instant_esc = esc(model.repo.registered_at.as_str());
    let number = lane.epic.id.number();
    let title_esc = esc(&short(&lane.epic.title, 96));
    let meta = meta_line(lane);
    let cards = cards_html(lane);
    let kinds = kind_line(lane);
    let rows = task_rows(model, lane);
    let decision_rows = decision_rows(model, lane);

    let mut page = String::with_capacity(8 * 1024);
    page.push_str("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\n");
    page.push_str(&format!(
        "<title>{slug_esc} · E{number} — {title_esc}</title>\n"
    ));
    page.push_str(&format!(
        "<style>{}{}{}</style></head><body>\n",
        CSS,
        super::sidebar::SIDEBAR_CSS,
        EPIC_CSS
    ));
    page.push_str(&switcher_html(model.sidebar, &model.repo.slug));
    page.push_str("\n<div class=\"cols\">\n");
    page.push_str(&super::sidebar::sidebar_html(model.sidebar));
    page.push_str("<main>\n");
    page.push_str(&format!(
        "\n<h1>{slug_esc} — E{number} · {title_esc}</h1>\n"
    ));
    page.push_str(&format!(
        "<div class=\"sub\">Rendered by <span class=\"mono\">todo-mcp</span> from \
         <span class=\"mono\">{slug_esc}</span>'s imported board — imported \
         <span class=\"mono\">{instant_esc}</span> (the repo node's registered_at — the \
         staleness signal: the page renders the last import, not the board as it is).</div>\n"
    ));
    page.push_str(&format!("\n<h2>The epic</h2>\n{meta}"));
    if !lane.epic.description.is_empty() {
        page.push_str(&format!(
            "<p class=\"legend\">{}</p>\n",
            esc(&lane.epic.description)
        ));
    }
    page.push_str(&format!(
        "\n<h2>Counts</h2>\n<div class=\"cards\">{cards}</div>\n"
    ));
    page.push_str(&kinds);
    page.push_str(&format!(
        "\n<h2>Tasks — {} rows</h2>\n\
         <table><thead><tr><th>id</th><th>kind</th><th>status</th><th>what</th><th>waits \
         on</th></tr></thead>\n<tbody>{rows}</tbody></table>\n",
        lane.tasks.len()
    ));
    page.push_str(&format!(
        "\n<h2>Decisions — {} rows</h2>\n\
         <table><thead><tr><th>id</th><th>status</th><th>what</th><th>gates</th>\
         <th>confidence</th></tr></thead>\n<tbody>{decision_rows}</tbody></table>\n",
        lane.decisions.len()
    ));
    page.push_str("</main>\n</div>\n");
    page.push_str("<script>");
    page.push_str(&live_script(slug));
    page.push_str("\n</script>\n</body></html>\n");
    page
}

/// The epic's header facts: the status, and the board's M id when the
/// lane carries one — the legend shape, one span per fact.
fn meta_line(lane: &Lane) -> String {
    let status = lane.epic.status.as_str();
    let mut out =
        format!("<p class=\"legend\"><span>status <b class=\"st-{status}\">{status}</b></span>");
    if let Some(m_id) = &lane.milestone {
        out.push_str(&format!(
            "<span>milestone <b class=\"mono\">{}</b></span>",
            esc(m_id)
        ));
    }
    out.push_str("</p>\n");
    out
}

/// The epic's counts as cards — the repo page's card shape, scoped to
/// the lane: every task under the epic (all four kinds), the blocked
/// count, and the decisions gating the epic or one of its tasks (an
/// open one gates work — the waits column's source).
fn cards_html(lane: &Lane) -> String {
    let total = lane.tasks.len();
    let done = lane
        .tasks
        .iter()
        .filter(|task| is_done(task.status))
        .count();
    let blocked = lane
        .tasks
        .iter()
        .filter(|task| task.status == Status::Blocked)
        .count();
    let open = lane
        .decisions
        .iter()
        .filter(|decision| decision.status == DecisionStatus::Open)
        .count();
    let decisions_total = lane.decisions.len();
    let cards: [(&str, String, String, &str); 3] = [
        (
            "Tasks",
            format!("{done}/{total}"),
            pct(done, total),
            "every task under the epic, all four kinds",
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
        (
            "Decisions",
            format!("{open}/{decisions_total} open"),
            pct(open, decisions_total),
            "gating this epic or one of its tasks; an open one gates work",
        ),
    ];
    cards
        .iter()
        .map(|(title, value, percent, note)| {
            format!(
                "<div class=\"card\"><i>{}</i><span class=\"p\">{}</span><b>{}</b>\
                 <small>{}</small></div>",
                esc(title),
                esc(percent),
                esc(value),
                esc(note)
            )
        })
        .collect()
}

/// The per-kind counts as a legend line — the fixed kind order, the
/// summary's per-kind shape.
fn kind_line(lane: &Lane) -> String {
    let mut out = String::from("<p class=\"legend\">");
    for (label, kind) in [
        ("task", TaskKind::Task),
        ("gate", TaskKind::Gate),
        ("finding", TaskKind::Finding),
        ("debt", TaskKind::Debt),
    ] {
        let counts = counts_for(&lane.tasks, kind);
        out.push_str(&format!(
            "<span>{} {}/{} done</span>",
            label, counts.done, counts.total
        ));
    }
    out.push_str("</p>\n");
    out
}

/// The tasks table's rows — every task under the epic, the lane's
/// ascending-id order, the repo page's open-items row shape with the
/// waits column from the model.
fn task_rows(model: &Model, lane: &Lane) -> String {
    lane.tasks
        .iter()
        .map(|task| {
            let waits = model.waits.get(&task.id).cloned().unwrap_or_default();
            format!(
                "<tr><td class=\"mono\">{}</td><td>{}</td><td class=\"st-{}\">{}</td>\
                 <td>{}</td><td>{}</td></tr>",
                esc(&short_id(&task.id.to_string())),
                esc(task.kind.as_str()),
                esc(task.status.as_str()),
                esc(task.status.as_str()),
                esc(&short(&task.title, 96)),
                esc(&waits)
            )
        })
        .collect()
}

/// The decisions table's rows — the lane's decisions, decision id
/// ascending, the repo page's decision row shape.
fn decision_rows(model: &Model, lane: &Lane) -> String {
    lane.decisions
        .iter()
        .map(|decision| {
            let gates = model.gates.get(&decision.id).cloned().unwrap_or_default();
            let gates_text = if gates.is_empty() {
                "—".to_owned()
            } else {
                gates.join(", ")
            };
            format!(
                "<tr><td class=\"mono\">{}</td><td class=\"st-{}\">{}</td><td>{}</td>\
                 <td class=\"mono\">{}</td><td>{}</td></tr>",
                esc(&short_id(&decision.id.to_string())),
                esc(decision.status.as_str()),
                esc(decision.status.as_str()),
                esc(&short(&decision.title, 72)),
                esc(&gates_text),
                esc(decision.confidence.as_str())
            )
        })
        .collect()
}

/// The epic page's own stylesheet rules, appended after the shared
/// CSS. The current-epic marker needs one: `.sepic span` (the counts'
/// rule) out-specifies `.scur` alone, so without a `.sepic .scur`
/// rule the marked label renders as the counts' gray. Page-local by
/// design — the same rule in `SIDEBAR_CSS` would move the repo page's
/// pinned bytes, and U54 owns that diff.
const EPIC_CSS: &str = ".sepic .scur { color: #1f6f43; font-size: 12px; }\n";
