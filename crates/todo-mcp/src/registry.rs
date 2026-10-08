//! The registry cluster (§7): `repo_register`, `repo_set_path`,
//! `repo_list` and `repo_show`, their wire DTOs and the
//! §8-serialized store work underneath. The two read tools answer
//! from the handles the server already holds on a hit (U20(b)); a
//! miss reads lock-free per call (D19) — no lock taken, no handle
//! cached — so a listing never arms and an unarmed server never
//! becomes a writer by listing: only writes arm. The registry is the
//! membership source either way; `repo_set_path`, a write, keeps the
//! arming resolve.
//!
//! The wire DTOs are server-local: named fields in declaration order
//! (D14 — `preserve_order` carries it to the wire), the params struct
//! denying unknown fields (§6.1's consequence: an unrecognised
//! argument is otherwise silently dropped), and no `schemars`
//! dependency reaching the domain (§2's boundary).

use std::str::FromStr;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use todo_domain::{Repo, RepoSlug};
use todo_store::Registry;

use crate::dto::RepoDto;
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

/// `repo_register`'s parameters (§7): the caller-chosen stable slug —
/// the key, never minted from the name — plus the mutable attributes.
/// §7's prose reads `repo_register(name, path)` minting the id; this
/// tool takes the slug as a parameter instead, because a name-derived
/// slug would be a derivation rule the domain does not host (§2) and
/// would drift when the name changes — the slugify alternative was
/// weighed and declined at dispatch. The slug arrives as a plain
/// `String` rather than a typed `RepoSlug`: a typed field would turn
/// §6.1's own `invalid` example (a bad slug) into a codeless rmcp
/// argument fault before the tool body runs, while the `String` lets
/// this crate parse it and carry `invalid` through the carrier.
///
/// The field doc comments are single-line on purpose: they become the
/// input schema's property descriptions verbatim, and a multi-line
/// comment would carry its line breaks into the wire bytes.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepoRegisterParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the caller's chosen identity, never reused once registered.
    pub slug: String,
    /// The repository's display name — a mutable attribute, unlike the slug.
    pub name: String,
    /// The filesystem path of the checkout (§7) — an attribute, not the key; `repo_set_path` moves it when a checkout moves.
    pub path: String,
}

/// `repo_register`'s success payload: the new state (§6 — writes return the new state), the repo node in declaration order (D14).
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct RepoRegistered {
    /// The repository's stable slug key.
    pub slug: String,
    /// The repository's display name.
    pub name: String,
    /// The filesystem path of the checkout.
    pub path: String,
    /// When the server registered the repository — RFC 3339 UTC with the nine-digit fraction, the one clock-stamped field.
    pub registered_at: String,
}

impl From<Repo> for RepoRegistered {
    fn from(repo: Repo) -> Self {
        Self {
            slug: repo.slug.to_string(),
            name: repo.name,
            path: repo.path,
            registered_at: repo.registered_at.as_str().to_owned(),
        }
    }
}

/// `repo_set_path`'s parameters (§7): the repo whose checkout moved, and its new path.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepoSetPathParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose checkout path moves; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The checkout's new filesystem path — an attribute, not the key (§7): the slug survives, the path follows the checkout.
    pub path: String,
}

/// `repo_list`'s parameters (§6): none — the listing is global, so the tool takes an empty object and rejects any argument.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepoListParams {}

/// `repo_show`'s parameters (§6): the repository to show.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepoShowParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo to show; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
}

/// `repo_list`'s answer (§6): every registered repo, ascending by slug.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct RepoListDto {
    /// Every registered repo, ascending by slug — the registry's own order, each row read from the repo node through the server's held handles.
    pub repos: Vec<RepoDto>,
}

// =================================================================
// The answer helpers — parse, resolve, stamp, mutate
// =================================================================

/// `repo_set_path`'s answer: parse, resolve, stamp, move. The `Repo`
/// node carries no `updated` field (§3) — the audit instant rides the
/// event line alone — and `registered_at` is fixed, so the response
/// is clock-free.
fn answer_repo_set_path(
    state: &mut ServerState,
    params: &RepoSetPathParams,
) -> Result<RepoDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let handle = state.resolve(&slug)?;
    let repo = handle.set_repo_path(params.path.clone(), time::now())?;
    Ok(RepoDto::from(&repo))
}

/// `repo_list`'s answer: the registry's slugs, each resolved through
/// [`ServerState::resolve_read`] — a handles hit borrows the held
/// graph (U20(b)); a miss reads lock-free per call (D19) and arms
/// nothing, so a listing never turns an unarmed server into a
/// writer. One store is read at a time, in the registry's sorted
/// order — t31's one-lock-at-a-time discipline, now trivial: the
/// read takes no lock at all.
fn answer_repo_list(state: &mut ServerState) -> Result<RepoListDto, ToolError> {
    let registry = Registry::load(&state.root)?;
    let mut repos = Vec::new();
    for slug in registry.slugs() {
        let graph = state.resolve_read(slug)?;
        repos.push(RepoDto::from(graph.graph().repo()));
    }
    Ok(RepoListDto { repos })
}

/// `repo_show`'s answer: parse, resolve, convert — the same read
/// path `repo_list` rows and every read tool take.
fn answer_repo_show(
    state: &mut ServerState,
    params: &RepoShowParams,
) -> Result<RepoDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let graph = state.resolve_read(&slug)?;
    Ok(RepoDto::from(graph.graph().repo()))
}

#[tool_router(router = tool_router, vis = "pub(crate)")]
impl TodoServer {
    /// Register a repository under its stable slug key: the store is created under the data root, the registry lists it, and the response is the new state.
    #[tool(
        name = "repo_register",
        output_schema = crate::schema::output_for_type::<RepoRegistered>()
    )]
    async fn repo_register(
        &self,
        Parameters(params): Parameters<RepoRegisterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §6.1's `invalid` is the wire identity of the domain's
        // ValueError (U9): a malformed slug is rejected before the
        // store is touched.
        let slug = match RepoSlug::from_str(&params.slug) {
            Ok(slug) => slug,
            Err(error) => return Ok(error_carrier("invalid", &error.to_string())),
        };
        // The audit instant — the one clock read. §8's "no wall
        // clock" is about selection; the store's events carry
        // caller-passed timestamps by design, and the binary is the
        // caller here.
        let repo = Repo {
            slug: slug.clone(),
            name: params.name,
            path: params.path,
            registered_at: time::now(),
        };
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it (the lib module
        // doc's rule).
        let mut state = self.lock_state();
        match state.root.register(repo.clone()) {
            Ok(store) => {
                // Boxed into the trait object at insert: no engine type
                // is held or named in this crate (§2). The handle keeps
                // the store's advisory lock — this server is that
                // repo's single writer from here on (§8) — and the map
                // is U20(b)'s answer surface.
                state.handles.insert(slug, Box::new(store));
                // The shared success arm: its serializer-fault bypass of
                // the carrier is deliberate and documented on the helper
                // (lib.rs's `structured`).
                structured("repo_register", RepoRegistered::from(repo))
            }
            Err(error) => Ok(error_carrier(error.code(), &error.to_string())),
        }
    }

    /// Move a repository's checkout path (§7): the slug is the key, the path an attribute — the audited verb, its own event line; the response is the new state.
    #[tool(
        name = "repo_set_path",
        output_schema = crate::schema::output_for_type::<RepoDto>(),
        annotations(open_world_hint = false)
    )]
    async fn repo_set_path(
        &self,
        Parameters(params): Parameters<RepoSetPathParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it.
        let mut state = self.lock_state();
        match answer_repo_set_path(&mut state, &params) {
            Ok(output) => structured("repo_set_path", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Every registered repository, ascending by slug — the registry's membership, each row read from the repo node through the server's held handles.
    #[tool(
        name = "repo_list",
        output_schema = crate::schema::output_for_type::<RepoListDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn repo_list(
        &self,
        Parameters(_params): Parameters<RepoListParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_repo_list(&mut state) {
            Ok(output) => structured("repo_list", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// One repository's node: the slug, the display name, the checkout path and the registration instant.
    #[tool(
        name = "repo_show",
        output_schema = crate::schema::output_for_type::<RepoDto>(),
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn repo_show(
        &self,
        Parameters(params): Parameters<RepoShowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_repo_show(&mut state, &params) {
            Ok(output) => structured("repo_show", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
