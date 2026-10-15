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
//! The module is nine files, split at U37's named point, the seams
//! t84's page rework forced, and U57's model split: this one holds
//! the surface assembly (the summary and the outputs), the
//! resolution and the shared text helpers, `model` the model types
//! with the `gather` and `build` halves that produce them (U57's
//! split, paying this file's recorded file-size debt), [`json()`] the
//! golden-pinned JSON document, [`page()`] the HTML page's skeleton
//! with the project switcher, `epic` the epic-scoped page (one
//! epic's lane as a page, the sidebar marking that epic current —
//! E12's per-epic surface), `svg` the dependency-graph SVG (extracted
//! from the page when the page half crossed the 600-line target — the
//! split the spec's D1 named, plus the seam it forced), `sidebar` the
//! left nav with the registered projects and their epics (the
//! per-repo reads and the nav tree), `panel` the activity section
//! over the event census, and `rewards` the §15 rewards section (the
//! model slice, the section's HTML, the size columns' cells and the
//! JSON twin's block — its own module because this file and `page.rs`
//! ran over the target with recorded debt when that arc landed, so
//! its display code went beside them instead of inside them). The
//! public surface is five functions over one internal model —
//! [`json()`] and [`summary`] (one surface each), [`page()`] (the
//! HTML), [`epic_page()`] (the epic-scoped HTML — serve-scoped, a
//! pure function of `(root, repo, epic)` like [`page()`] is of
//! `(root, repo)`, and never one of [`outputs`]'s written surfaces:
//! the static progress output stays one snapshot), and [`outputs`]
//! (one resolve + one census + one build, all three surfaces plus the
//! repo's registered checkout, for the subcommand's default write
//! location) — plus [`resolve_by_cwd`], the subcommand's repo tier:
//! the `--repo` flag wins (the binary owns that check), else the
//! working directory's deterministic longest-ancestor match against
//! the registered checkouts (§7, the same rule the server's default
//! tier applies). The binary reads the cwd once and passes it in;
//! this crate never touches the environment.
//!
//! The resolution mirrors `ServerState::resolve_read`'s discipline
//! exactly (the t64 hand-off); `model`'s doc carries the gather
//! ordering and the two anti-silent-omission rules ported from the
//! Python tool.

mod epic;
mod json;
mod model;
mod page;
mod panel;
mod rewards;
mod sidebar;
mod svg;

pub use self::epic::epic_page;
pub use self::json::json;
pub use self::page::{Outputs, page};

use std::path::{Path, PathBuf};

use todo_domain::{Decision, DecisionStatus, DomainError, Graph, RepoSlug, Status, Task, TaskKind};
use todo_store::{DataRoot, Registry, StoreError};

use crate::ToolError;
use crate::cwd::{MatchOutcome, match_slug};

use self::model::{KindCounts, Model, build, gather};
use self::rewards::size_cells;

/// The `progress` human summary: the Python tool's summary block minus
/// the git sha and the date — narration, not a pinned surface, but
/// derived from the same model so it can never disagree with the JSON.
pub fn summary(root: &DataRoot, repo: &RepoSlug) -> Result<String, String> {
    let (graph, census, sidebar, rewards) = gather(root, repo, None)?;
    let model = build(&graph, &census, &sidebar, rewards).map_err(|error| error.message)?;
    Ok(summary_text(&model))
}

/// Every surface from one resolve + one census + one build — the
/// subcommand's default path, so the page, the JSON twin and the
/// summary can never disagree with each other (they are one model), and
/// the binary derives its default output directory from the repo's
/// registered checkout without a second read.
pub fn outputs(root: &DataRoot, repo: &RepoSlug) -> Result<Outputs, String> {
    let (graph, census, sidebar, rewards) = gather(root, repo, None)?;
    let model = build(&graph, &census, &sidebar, rewards).map_err(|error| error.message)?;
    Ok(Outputs {
        page: page::page_text(&model)?,
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

/// One tasks-table row — the row shape both pages render (U55's hoist:
/// the epic page's task rows and the repo page's open-items rows were
/// byte-identical format strings, so the shape has one home and cannot
/// drift): the id, kind, status, title and waits-on columns, the
/// waits text from the model — plus the est/actual size cells (U63)
/// between the title and the waits, empty on every disabled root
/// (§15's fence: no columns, no bytes).
fn task_row_html(model: &Model, task: &Task) -> String {
    let waits = model.waits.get(&task.id).cloned().unwrap_or_default();
    let sizes = size_cells(model.rewards.as_ref(), task);
    format!(
        "<tr><td class=\"mono\">{}</td><td>{}</td><td class=\"st-{}\">{}</td>\
         <td>{}</td>{sizes}<td>{}</td></tr>",
        esc(&short_id(&task.id.to_string())),
        esc(task.kind.as_str()),
        esc(task.status.as_str()),
        esc(task.status.as_str()),
        esc(&short(&task.title, 96)),
        esc(&waits)
    )
}

/// One decisions-table row — the shared shape (U55's hoist): the id,
/// status, title, gates and confidence columns, the gates text from
/// the model.
fn decision_row_html(model: &Model, decision: &Decision) -> String {
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
}

/// One totals card — the shared card shape (U55's hoist): the label,
/// the percentage, the value and the note.
fn card_html(title: &str, value: &str, percent: &str, note: &str) -> String {
    format!(
        "<div class=\"card\"><i>{}</i><span class=\"p\">{}</span><b>{}</b>\
         <small>{}</small></div>",
        esc(title),
        esc(percent),
        esc(value),
        esc(note)
    )
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

    /// The label helper: whitespace collapses, truncation lands on a
    /// character boundary with the ellipsis.
    #[test]
    fn short_collapses_and_truncates_on_char_boundaries() {
        assert_eq!(short("a  b\t c", 30), "a b c");
        assert_eq!(short("abcdef", 4), "abc…");
        assert_eq!(short("日本語のタイトル", 4), "日本語…");
    }
}
