//! The decision family (§6, D10 — U78's split out of the mutation
//! cluster's home file): `decision_add`/`decision_update`, the
//! dossier pair. A decision is a structured choice that gates work
//! until the user signs it: the params mirror the domain's
//! `DecisionDraft` and the store's `DecisionPatch` field-for-field
//! (§6), the update params under RFC 7386's three-state reading —
//! absent is untouched, `null` clears an optional field
//! (recommendation, resolution), a value replaces it whole — through
//! the cluster's shared [`super::double_option`]. `signed`/
//! `superseded` require a resolution (`missing_resolution`
//! otherwise — the closure discipline, the home file's doc). The
//! shared conversion helpers live in the cluster's home (`super`),
//! and each tool stamps one `time::now` read: `created`/`updated` on
//! add, `updated` alone on update. The failure arm and the
//! annotations are the cluster's own: `error_carrier` over
//! [`crate::ToolError`], `open_world_hint = false`, the
//! `DecisionDto` output schema.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use todo_domain::{Confidence, DecisionDraft, DecisionId, DecisionStatus};

use todo_store::DecisionPatch;

use crate::dto::{ConfidenceDto, DecisionDto, DecisionStatusDto, ReferenceParam};
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

use super::{double_option, parse_value, references_of};

/// `decision_add`'s parameters (§6, D10): the dossier's every field except the identity and the timestamps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionAddParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph gains the decision; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// Short human title.
    pub title: String,
    /// Lifecycle status: open, signed or superseded — `signed`/`superseded` require a resolution.
    pub status: DecisionStatusDto,
    /// The options the user chooses between, in authoring order.
    pub options: Vec<String>,
    /// The proposer's recommendation among the options.
    pub recommendation: Option<String>,
    /// How it was resolved — required to reach `signed`/`superseded` (`missing_resolution` otherwise).
    pub resolution: Option<String>,
    /// Proposer confidence at drafting time: low, medium or high.
    pub confidence: ConfidenceDto,
    /// External references (D10); absent means the empty set.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `decision_update`'s parameters (§6, D10): the patchable dossier fields under RFC 7386's three-state reading — absent is untouched, `null` clears an optional field, a value replaces it whole.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionUpdateParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph holds the decision; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The decision's id, e.g. `camera:D3`.
    pub decision: String,
    /// A new title; absent leaves the field untouched.
    pub title: Option<String>,
    /// A new lifecycle status; `signed`/`superseded` require a resolution (§3).
    pub status: Option<DecisionStatusDto>,
    /// A replacement option list, in authoring order; absent leaves the field untouched.
    pub options: Option<Vec<String>>,
    /// A new recommendation; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub recommendation: Option<Option<String>>,
    /// A new resolution; absent leaves the field untouched, `null` clears it, a value replaces it.
    #[serde(default, deserialize_with = "double_option")]
    pub resolution: Option<Option<String>>,
    /// A new proposer confidence; absent leaves the field untouched.
    pub confidence: Option<ConfidenceDto>,
    /// A replacement reference set — the whole set, canonically sorted; absent leaves the field untouched.
    pub references: Option<Vec<ReferenceParam>>,
}

/// `decision_add`'s answer: parse, resolve, convert, stamp, add.
fn answer_decision_add(
    state: &mut ServerState<'_>,
    params: &DecisionAddParams,
) -> Result<DecisionDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let references = references_of(params.references.as_deref().unwrap_or(&[]))?;
    let draft = DecisionDraft {
        title: params.title.clone(),
        status: DecisionStatus::from(params.status),
        options: params.options.clone(),
        recommendation: params.recommendation.clone(),
        resolution: params.resolution.clone(),
        confidence: Confidence::from(params.confidence),
        references,
    };
    let handle = state.resolve(&slug)?;
    let decision = handle.add_decision(draft, time::now())?;
    Ok(DecisionDto::from(&decision))
}

/// `decision_update`'s answer: parse, resolve, convert, stamp, patch.
fn answer_decision_update(
    state: &mut ServerState<'_>,
    params: &DecisionUpdateParams,
) -> Result<DecisionDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let decision = parse_value::<DecisionId>(&params.decision)?;
    let references = params
        .references
        .as_deref()
        .map(references_of)
        .transpose()?;
    let patch = DecisionPatch {
        title: params.title.clone(),
        status: params.status.map(DecisionStatus::from),
        options: params.options.clone(),
        recommendation: params.recommendation.clone(),
        resolution: params.resolution.clone(),
        confidence: params.confidence.map(Confidence::from),
        references,
    };
    let handle = state.resolve(&slug)?;
    let decision = handle.update_decision(decision, time::now(), patch)?;
    Ok(DecisionDto::from(&decision))
}

#[tool_router(router = decisions_router, vis = "pub(crate)")]
impl TodoServer {
    /// Add a decision dossier: the graph allocates its id, `signed`/`superseded` require a resolution, and the response is the new state with its clock-stamped `created`/`updated`.
    #[tool(
        name = "decision_add",
        output_schema = crate::schema::output_for_type::<DecisionDto>(),
        annotations(open_world_hint = false)
    )]
    async fn decision_add(
        &self,
        Parameters(params): Parameters<DecisionAddParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_decision_add(&mut state, &params) {
            Ok(output) => structured("decision_add", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }

    /// Patch a decision dossier in place — absent fields untouched, `null` clears an optional one, a value replaces it whole; signing requires a resolution — and answer with the new state, `updated` clock-stamped.
    #[tool(
        name = "decision_update",
        output_schema = crate::schema::output_for_type::<DecisionDto>(),
        annotations(open_world_hint = false)
    )]
    async fn decision_update(
        &self,
        Parameters(params): Parameters<DecisionUpdateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut state = self.lock_state();
        match answer_decision_update(&mut state, &params) {
            Ok(output) => structured("decision_update", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}
