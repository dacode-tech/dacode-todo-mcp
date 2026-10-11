//! The sidebar (t84's D3/D4): the registered projects with their epics
//! nested under each — the left nav the repo-only switcher could not
//! be. One lock-free graph read per registered slug per render, the
//! rendered repo's read reused from the model's own graph; a repo whose
//! graph will not read is reported with the unreadable marker, never
//! dropped (the anti-silent-omission rule). The epic rows carry the
//! model's per-lane done/total task counts — not census-derived,
//! because events are node-scoped (D5) — and link to the SVG lane
//! anchors (`svg.rs`'s `id="lane-<number>"`), same-page for the
//! rendered repo, `?repo=<slug>#lane-<number>` for the others: in the
//! static file the cross-repo links are inert, which is honest — the
//! static page is one repo's snapshot.

use todo_domain::{Graph, RepoSlug};
use todo_store::{DataRoot, Registry, StoreError};

use crate::ToolError;

use super::{esc, is_done, short};

/// One repo's row in the sidebar: the slug key, the display name (the
/// `Repo` node's — the slug itself when the graph will not read), the
/// current marker, and the epic rows under it.
pub(super) struct SidebarRepo {
    /// The repo's slug — the link key and the registry's identity.
    pub(super) slug: RepoSlug,
    /// The display name: the node's `name`, or the slug when the store
    /// will not read (the name lives only in the node).
    pub(super) name: String,
    /// Whether this is the rendered repo — the marked non-link.
    pub(super) current: bool,
    /// Whether the graph read refused: the row shows the unreadable
    /// marker and no epics — reported, never dropped.
    pub(super) unreadable: bool,
    /// The repo's epics, ascending epic id (the model's lane order).
    pub(super) epics: Vec<SidebarEpic>,
}

/// One epic row: the number (the lane anchor's key), the title, and
/// the done/total task counts under the epic.
pub(super) struct SidebarEpic {
    /// The epic id's number — the `lane-<number>` anchor's key.
    pub(super) number: u64,
    /// The epic's title, shortened at render.
    pub(super) title: String,
    /// Closed or wontfix tasks under the epic.
    pub(super) done: usize,
    /// Every task under the epic.
    pub(super) total: usize,
}

/// The sidebar's per-repo reads (D3): every registered slug in the
/// registry's ascending order, one lock-free graph read each — the
/// rendered repo's read is the caller's own graph, reused, never
/// double-read. A repo whose graph will not read — or whose store
/// holds another repo's graph under the slug, the resolve cross-check
/// one level out — is reported with the unreadable marker, never
/// dropped; a registry read failure fails the render.
pub(super) fn sidebar_repos(
    root: &DataRoot,
    current: &RepoSlug,
    current_graph: &Graph,
) -> Result<Vec<SidebarRepo>, String> {
    let registry = Registry::load(root).map_err(|error| ToolError::from(error).message)?;
    let mut repos: Vec<SidebarRepo> = Vec::with_capacity(registry.slugs().len());
    for slug in registry.slugs() {
        if slug == current {
            repos.push(SidebarRepo {
                slug: slug.clone(),
                name: current_graph.repo().name.clone(),
                current: true,
                unreadable: false,
                epics: epic_rows(current_graph),
            });
            continue;
        }
        // The slug cross-check mirrors resolve's: a store directory
        // holding another repo's graph under this slug is not a graph
        // this row may display — the marker, never the foreign epics.
        let read = root.read_repo(slug);
        let read = match read {
            Ok(graph) if &graph.repo().slug != slug => Err(StoreError::CorruptStore {
                offset: None,
                detail: format!(
                    "the store opened for {slug} holds repo {}",
                    graph.repo().slug
                ),
            }),
            other => other,
        };
        match read {
            Ok(graph) => repos.push(SidebarRepo {
                slug: slug.clone(),
                name: graph.repo().name.clone(),
                current: false,
                unreadable: false,
                epics: epic_rows(&graph),
            }),
            Err(_) => repos.push(SidebarRepo {
                slug: slug.clone(),
                name: slug.as_str().to_owned(),
                current: false,
                unreadable: true,
                epics: Vec::new(),
            }),
        }
    }
    Ok(repos)
}

/// One repo's epic rows: ascending epic id (the model's lane order),
/// each with its done/total task counts.
fn epic_rows(graph: &Graph) -> Vec<SidebarEpic> {
    graph
        .epics()
        .map(|epic| {
            let tasks = graph.tasks_of_epic(&epic.id);
            let done = tasks.iter().filter(|task| is_done(task.status)).count();
            SidebarEpic {
                number: epic.id.number(),
                title: epic.title.clone(),
                done,
                total: tasks.len(),
            }
        })
        .collect()
}

/// The sidebar's nav tree (D4): one block per repo — the name linking
/// to `?repo=<slug>` for other repos, a marked non-link for the
/// rendered one (a self-link is a no-op dressed as navigation) — with
/// the epic rows under it, each linking to its lane anchor.
pub(super) fn sidebar_html(repos: &[SidebarRepo]) -> String {
    let mut out = String::from("<nav class=\"sidebar\">");
    for repo in repos {
        let name = esc(&repo.name);
        if repo.current {
            out.push_str(&format!(
                "<div class=\"srepo\"><span class=\"sname scur\">{name}</span>"
            ));
        } else {
            let slug = esc(repo.slug.as_str());
            out.push_str(&format!(
                "<div class=\"srepo\"><a class=\"sname\" href=\"?repo={slug}\">{name}</a>"
            ));
        }
        if repo.unreadable {
            out.push_str("<span class=\"unread\">unreadable</span>");
        }
        for epic in &repo.epics {
            let label = esc(&short(&epic.title, 28));
            let href = if repo.current {
                format!("#lane-{}", epic.number)
            } else {
                format!("?repo={}#lane-{}", esc(repo.slug.as_str()), epic.number)
            };
            out.push_str(&format!(
                "<div class=\"sepic\"><a href=\"{href}\">E{} · {label}</a>\
                 <span>{}/{}</span></div>",
                epic.number, epic.done, epic.total
            ));
        }
        out.push_str("</div>");
    }
    out.push_str("</nav>");
    out
}

/// The sidebar's stylesheet: the two-column layout (sidebar + main, a
/// narrow-screen media query stacking them) and the nav tree's own
/// rules.
pub(super) const SIDEBAR_CSS: &str = r#".cols { display: flex; gap: 26px; align-items: flex-start; }
.sidebar { width: 252px; flex: 0 0 252px; position: sticky; top: 20px;
           max-height: calc(100vh - 40px); overflow-y: auto; }
main { flex: 1 1 auto; min-width: 0; }
@media (max-width: 880px) { .cols { display: block; }
  .sidebar { position: static; width: auto; max-height: none; } }
.srepo { margin-bottom: 12px; }
.sname { display: block; font-weight: 600; font-size: 13px; color: #3f444c;
         text-decoration: none; margin-bottom: 3px; }
a.sname:hover { text-decoration: underline; }
.scur { color: #1f6f43; }
.sepic { display: flex; justify-content: space-between; gap: 6px; font-size: 12px;
         padding: 1px 0 1px 10px; }
.sepic a { color: #3f444c; text-decoration: none; }
.sepic a:hover { text-decoration: underline; }
.sepic span { color: #6b7280; font-variant-numeric: tabular-nums; font-size: 11px; }
.unread { color: #9b1c1c; font-size: 11px; font-weight: 700; margin-left: 6px; }
"#;
