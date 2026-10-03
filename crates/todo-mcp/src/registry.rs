//! The registry cluster's first tool (§7): `repo_register`, its wire
//! DTOs and the §8-serialized store work underneath. t37 extends this
//! file with `repo_set_path`, `repo_list` and `repo_show`, answered
//! from the handles the server already holds (U20(b)).
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

use crate::{TodoServer, error_carrier, structured, time};

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

#[tool_router(router = tool_router, vis = "pub(crate)")]
impl TodoServer {
    /// Register a repository under its stable slug key: the store is created under the data root, the registry lists it, and the response is the new state.
    #[tool(
        name = "repo_register",
        output_schema = rmcp::handler::server::tool::schema_for_type::<RepoRegistered>()
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
}
