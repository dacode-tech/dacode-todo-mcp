//! The render dispatch (U56's split out of the listener): the routes
//! that render — `render_route` for the page and its JSON twin (the
//! `?repo=` parameter, else the default tier, resolving which repo),
//! `render_epic_route` for the epic page (the path the repo's only
//! source) — with the repo resolution they share and the
//! failed-render body. The connection dispatch that routes here, the
//! default-repo tier, and the listener itself stay in the parent
//! module; `ws` reaches the repo resolution through the parent's
//! re-import.

use std::net::TcpStream;
use std::str::FromStr;

use todo_domain::{ErrorCode, RepoSlug};

use crate::render;

use super::ConnState;
use super::default_repo;
use super::http::{self, Request};

/// The failed-render body (D2): the reason is already on the
/// listener's stderr; the response says only that.
pub(super) const RENDER_FAILED_BODY: &str = "render failed — see the listener's stderr\n";

/// Which render surface a route asked for.
pub(super) enum Surface {
    /// The HTML page.
    Page,
    /// The JSON twin.
    Json,
}

/// The render routes (D2): resolve the repo — the `?repo=` parameter,
/// else the default tier — render fresh, answer 200; any failure is
/// 500 with the reason on stderr (the listener stays up).
pub(super) fn render_route(
    state: &ConnState,
    request: &Request,
    stream: &mut TcpStream,
    surface: Surface,
) {
    let slug = repo_for_request(state, request);
    let rendered = match surface {
        Surface::Page => slug.and_then(|slug| render::page(&state.root, &slug)),
        Surface::Json => slug.and_then(|slug| render::json(&state.root, &slug)),
    };
    match rendered {
        Ok(text) => {
            let content_type = match surface {
                Surface::Page => "text/html; charset=utf-8",
                Surface::Json => "application/json",
            };
            http::write_plain_response(stream, 200, content_type, text.as_bytes());
        }
        Err(message) => {
            eprintln!("todo-mcp serve: the render failed: {message}");
            http::write_plain_response(
                stream,
                500,
                "text/plain; charset=utf-8",
                RENDER_FAILED_BODY.as_bytes(),
            );
        }
    }
}

/// The request's repo (D2): the `?repo=` parameter — a malformed value
/// is a failed render, answered 500 like an unknown one — else the
/// default tier.
pub(super) fn repo_for_request(state: &ConnState, request: &Request) -> Result<RepoSlug, String> {
    match http::repo_param(request) {
        Some(raw) => RepoSlug::from_str(raw).map_err(|error| error.to_string()),
        None => default_repo(&state.root, state.cwd.as_deref()),
    }
}

/// The epic page's route (E12, U53): `/repo/<slug>/epic/<number>` —
/// the path is the repo's only source, `?repo=` ignored entirely. The
/// pre-checks answer before any render: an unknown repo is a named
/// 404 — the resolve error's `unknown_id` code, the variant's stable
/// identity, never a message match — and an epic number the resolved
/// graph does not hold is a named 404; every other resolve failure —
/// an unreadable or corrupt store — is a server fault: 500 with the
/// reason on stderr, the `render_route` discipline. The pre-check
/// pays resolve plus an epics scan, not the render; the 200 path then
/// reads the graph a second time inside `epic_page` — the accepted
/// cost of keeping that function's pure signature.
pub(super) fn render_epic_route(
    state: &ConnState,
    slug: &RepoSlug,
    number: u64,
    stream: &mut TcpStream,
) {
    match render::resolve(&state.root, slug) {
        Ok(graph) => {
            if !graph.epics().any(|epic| epic.id.number() == number) {
                let miss = format!("{slug} holds no epic {number}");
                eprintln!("todo-mcp serve: {miss}");
                http::write_plain_response(
                    stream,
                    404,
                    "text/plain; charset=utf-8",
                    format!("{miss}\n").as_bytes(),
                );
                return;
            }
            match render::epic_page(&state.root, slug, number) {
                Ok(text) => {
                    http::write_plain_response(
                        stream,
                        200,
                        "text/html; charset=utf-8",
                        text.as_bytes(),
                    );
                }
                Err(message) => {
                    eprintln!("todo-mcp serve: the render failed: {message}");
                    http::write_plain_response(
                        stream,
                        500,
                        "text/plain; charset=utf-8",
                        RENDER_FAILED_BODY.as_bytes(),
                    );
                }
            }
        }
        Err(error) if error.code == ErrorCode::UnknownId.as_str() => {
            let miss = format!("no repo {slug} under this root");
            eprintln!("todo-mcp serve: {miss}");
            http::write_plain_response(
                stream,
                404,
                "text/plain; charset=utf-8",
                format!("{miss}\n").as_bytes(),
            );
        }
        Err(error) => {
            eprintln!(
                "todo-mcp serve: the epic page's resolve failed: {}",
                error.message
            );
            http::write_plain_response(
                stream,
                500,
                "text/plain; charset=utf-8",
                RENDER_FAILED_BODY.as_bytes(),
            );
        }
    }
}
