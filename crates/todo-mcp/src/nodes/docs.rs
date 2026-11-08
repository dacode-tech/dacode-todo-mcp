//! The doc family (§6, D10 — U78's split out of the mutation
//! cluster's home file): `doc_register`/`doc_set_path`/`doc_update`,
//! the registered documents' three verbs. A doc is a citation
//! target: the prose body stays in git, the graph holds the
//! registration — the id, the title, the version, the mutable path,
//! the references — and the path moves only through `doc_set_path`,
//! the separately audited verb (the committed D10: a citation's
//! address change is its own event, visible in the log on its own
//! line); `doc_update` carries no `path` parameter for that reason.
//! The params mirror the domain's `DocDraft` and the store's
//! `DocPatch` field-for-field (§6) — the update params' `references`
//! a whole-set replacement, the rest plain strings — and the shared
//! conversion helpers live in the cluster's home (`super`). Each
//! tool stamps one `time::now` read: `created` and `updated` on
//! register, `updated` alone on the two updates. The failure arm and
//! the annotations are the cluster's own (the home file's doc):
//! `error_carrier` over [`crate::ToolError`], `open_world_hint =
//! false`, the `DocDto` output schema.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use todo_domain::{DocDraft, DocId};

use todo_store::DocPatch;

use crate::dto::{DocDto, ReferenceParam};
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

use super::{parse_value, references_of};

/// `doc_register`'s parameters (§6, D10): the doc's every field except the identity and the timestamps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocRegisterParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the doc; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// Short human title.
    pub title: String,
    /// The mutable file path of the prose body — an attribute, moved later by `doc_set_path`.
    pub path: String,
    /// The document's version — an opaque string the document itself declares.
    pub version: String,
    /// External references (D10); absent means the empty set.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `doc_update`'s parameters (§6, D10): the patchable doc fields — no `path`, which the separately audited verb `doc_set_path` owns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocUpdateParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the doc; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The doc's id, e.g. `camera:S1`.
    pub doc: String,
    /// A new title; absent leaves the field untouched.
    pub title: Option<String>,
    /// A new version string; absent leaves the field untouched.
    pub version: Option<String>,
    /// A replacement reference set — the whole set, canonically sorted; absent leaves the field untouched.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `doc_set_path`'s parameters (§6, D10): the audited move — a citation's address change is its own verb, visible in the log on its own line.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocSetPathParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the doc; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The doc's id, e.g. `camera:S1`.
    pub doc: String,
    /// The doc's new filesystem path of the prose body.
    pub path: String,
}

/// `doc_register`'s answer: parse, resolve, convert, stamp, add.
fn answer_doc_register(
    state: &mut ServerState<'_>,
    params: &DocRegisterParams,
) -> Result<DocDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let references = references_of(params.references.as_deref().unwrap_or(&[]))?;
    let draft = DocDraft {
        title: params.title.clone(),
        path: params.path.clone(),
        version: params.version.clone(),
        references,
    };
    let handle = state.resolve(&slug)?;
    let doc = handle.add_doc(draft, time::now())?;
    Ok(DocDto::from(&doc))
}

/// `doc_set_path`'s answer: parse, resolve, stamp, move.
fn answer_doc_set_path(
    state: &mut ServerState<'_>,
    params: &DocSetPathParams,
) -> Result<DocDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let doc = parse_value::<DocId>(&params.doc)?;
    let handle = state.resolve(&slug)?;
    let doc = handle.set_doc_path(doc, params.path.clone(), time::now())?;
    Ok(DocDto::from(&doc))
}

/// `doc_update`'s answer: parse, resolve, convert, stamp, patch.
fn answer_doc_update(
    state: &mut ServerState<'_>,
    params: &DocUpdateParams,
) -> Result<DocDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let doc = parse_value::<DocId>(&params.doc)?;
    let references = params
        .references
        .as_deref()
        .map(references_of)
        .transpose()?;
    let patch = DocPatch {
        title: params.title.clone(),
        version: params.version.clone(),
        references,
    };
    let handle = state.resolve(&slug)?;
    let doc = handle.update_doc(doc, time::now(), patch)?;
    Ok(DocDto::from(&doc))
}

#[tool_router(router = docs_router, vis = "pub(crate)")]
impl TodoServer {
    /// Register a document for stable citation: the graph allocates its id, and the response is the new state with its clock-stamped `created`/`updated`.
    #[tool(
        name = "doc_register",
        output_schema = crate::schema::output_for_type::<DocDto>(),
        annotations(open_world_hint = false)
    )]
    async fn doc_register(
        &self,
        Parameters(params): Parameters<DocRegisterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_register(&mut state, &params) {
            Ok(output) => structured("doc_register", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Move a doc's body to a new path — the audited verb (D10): a citation's address change is its own event — and answer with the moved doc's new state, `updated` clock-stamped.
    #[tool(
        name = "doc_set_path",
        output_schema = crate::schema::output_for_type::<DocDto>(),
        annotations(open_world_hint = false)
    )]
    async fn doc_set_path(
        &self,
        Parameters(params): Parameters<DocSetPathParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_set_path(&mut state, &params) {
            Ok(output) => structured("doc_set_path", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Patch a doc in place — absent fields untouched, given fields replaced whole; the path moves only through `doc_set_path` — and answer with the new state, `updated` clock-stamped.
    #[tool(
        name = "doc_update",
        output_schema = crate::schema::output_for_type::<DocDto>(),
        annotations(open_world_hint = false)
    )]
    async fn doc_update(
        &self,
        Parameters(params): Parameters<DocUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_doc_update(&mut state, &params) {
            Ok(output) => structured("doc_update", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
