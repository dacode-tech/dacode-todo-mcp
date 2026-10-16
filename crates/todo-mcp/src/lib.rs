//! The MCP server (spec §2, §6): the rmcp wire surface over stdio.
//!
//! This crate is the binary's library half — the server, its state and
//! the wire DTOs; the argument parse lives in the [`args`] module
//! (U38's split), the server and its state core in the `state` module
//! (U68's split), and `main.rs` owns the runtime and the serve call.
//! The binary translates and never decides: all
//! selection, ordering and query logic lives in `todo-domain`, all
//! persistence in `todo-store`, and this crate maps tool calls onto
//! those. The storage engine stays sealed inside `todo-store` (§2):
//! its constructors return the concrete engine, and this crate boxes
//! each store into the [`todo_store::Store`] trait object at the
//! boundary — the shape `todo-store`'s own lib.rs provisioned for
//! M3's server — so no engine type, error or feature flag is held or
//! named here.
//!
//! # The importer (§9, D12)
//!
//! The crate also hosts the board importer: [`import::dacode`] is the
//! dacode dialect's parser, a pure function of text with no I/O, no
//! clock and no store — file reading belongs to the `import_md` wire
//! tool (`importer.rs`), the mapping to domain types to t43, and
//! camera, ikvm-html and this repo's own board arrived as sibling
//! dialect modules (t46, t47, t54). A dialect parser is not a wire
//! surface: it decides nothing, it recognizes — a register line that
//! does not parse is a named error, never a silent skip, because an
//! importer that quietly drops an item reports less than the truth
//! while still looking complete.
//!
//! # The state core (§8, D25; D19; D8)
//!
//! The server's state is two halves behind two locks — the shared
//! core (the data root and the open per-repo handles) and the
//! per-connection session (the default repo, the working directory,
//! the cwd-default memo) — and the repo dimension resolves an absent
//! parameter through the flag's configured default, else the cwd's
//! deterministic longest-ancestor match. The full contracts — the
//! lock discipline, the read/write resolve split (D19), the tier
//! order (D8) — live in the `state` module with the code they
//! describe.
//!
//! # The progress render (D20)
//!
//! The same binary carries the tool CLI (§13, D20): subcommands over
//! the store — `progress` (the total-progress document and HTML page
//! [`render`] computes from one repo's graph, read lock-free (D19) so
//! a render never contends with a live session server; both surfaces
//! golden-pinnable, which is the render's purity, not the refresh's)
//! and `refresh` ([`refresh::refresh`], the on-demand re-import: the
//! registry surgery, the re-register, the apply — the store is a
//! derived cache of the board and this rebuilds it). The render is a
//! pure function of `(root, repo)` — no clock, no git, no
//! environment — which is what makes its surfaces golden-pinnable;
//! the page carries a project switcher driven by the
//! registry (the `?repo=<slug>` URL shape the `serve` subcommand will
//! answer). The binary owns the one cwd read and the output writes;
//! the lib's [`args`] module owns the argument parse (U38's split,
//! landed with t72), and the module owns the model. The inspector
//! the `todo-cli` crate carried is absorbed here as [`inspect`] —
//! the `dump`/`verify`/`repair` subcommands (D20, t72) — formatting,
//! prompting and picking exit codes over the store's own facts,
//! every classification and byte offset the store's; the crate
//! retired with the move.
//!
//! # The live listener (D20, t71; D13, t85; D25, U66)
//!
//! `serve` ([`serve::serve`]) is the live view: a listener over
//! hand-rolled HTTP/1.1 on 127.0.0.1 — every route a fresh render,
//! the repo a URL parameter — plus the WebSocket push channel, the
//! port published per-root (`<root>/serve.port`) so a second
//! invocation attaches instead of starting a second listener. The
//! daemon owns the listener now (D25's retirement, U66): it ensures
//! the root's live view once at startup through the same
//! `serve::Watchdog::ensure` the `serve` entry backs, and the
//! listener lives exactly as long as the daemon — the no-subcommand
//! path became the shim, which never touches the listener, and the
//! watchdog's takeover loop retired with it. The module stays
//! parameterized and env-free: the default port-file path derives
//! from the root, and only the `serve` subcommand owns a SIGINT
//! shutdown — the daemon ends by SIGTERM's drain, and the next
//! owner's ensure handles the stale publication.
//!
//! # The error carrier (D15)
//!
//! Every tool failure travels through the `error_carrier` helper as
//! [`CallToolResult::structured_error`] with
//! `structuredContent = { "code", "message" }` and the same JSON
//! mirrored into the text block (§6.1): a client branches on the code,
//! a model still reads the message. Codes are §6.1's closed list,
//! mapped once by `ToolError` — answered by `StoreError::code()` for
//! store faults, `DomainError::code()` for domain faults, `"invalid"`
//! for the domain's `ValueError` (U9's mapping) — so a tool's failure
//! arm is one line and t35+ inherit the mapping rather than restating
//! it. Argument faults — a wrong type, a missing or unknown field —
//! are rmcp's own unstructured text errors and never reach the tool
//! body (§6.3), so they arrive codeless by design and no code is
//! invented for them.
//!
//! # The advertised schemas' format discipline (U32)
//!
//! schemars emits OpenAPI integer formats (`uint64` and kin) that JSON
//! Schema validators do not know — opencode's Ajv warns on every
//! `tools/list` — so every schema this server advertises strips them;
//! the `schema` module owns the strip and its wire pin.

// Tests are exempt from the panic-family lints through a mechanism, not
// a convention (the workspace's rule, stated once in the root
// Cargo.toml's lint table).
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

use rmcp::model::{CallToolResult, ErrorData};
use serde::Serialize;
use todo_domain::{DomainError, ValueError};
use todo_store::StoreError;

pub mod args;
mod cwd;
pub mod daemon;
mod dto;
mod edges;
pub mod import;
mod importer;
pub mod inspect;
mod listing;
mod nodes;
mod query;
pub mod refresh;
mod registry;
pub mod render;
mod rewards;
mod schema;
pub mod serve;
pub mod shim;
mod state;
mod time;

// The state core's re-exports (U68's split): the cluster files, the
// daemon and the shim keep their `crate::` paths unchanged.
pub use state::TodoServer;
pub(crate) use state::{CoreState, ServerState, repo_of, server_config};

/// The one error-mapping layer between the crates below and D15's
/// carrier: a §6.1 code plus its message, converted from
/// [`StoreError`], [`DomainError`] and [`ValueError`] (U9's `invalid`
/// mapping). Every tool's failure arm is
/// `Err(e) => Ok(error_carrier(&e.code, &e.message))` — one home, so
/// t35+ inherit the mapping rather than restating it per cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolError {
    /// The stable §6.1 code a client branches on.
    pub code: String,
    /// The human-readable message, mirrored into the carrier's text
    /// block; never the contract.
    pub message: String,
}

impl ToolError {
    /// §6.1's `invalid` for a rejection this crate makes itself (not a
    /// domain `ValueError`): t34's mutually exclusive `next_items`
    /// scopes.
    pub(crate) fn invalid(message: &str) -> Self {
        Self {
            code: "invalid".to_owned(),
            message: message.to_owned(),
        }
    }
}

impl From<StoreError> for ToolError {
    fn from(error: StoreError) -> Self {
        Self {
            code: error.code().to_owned(),
            message: error.to_string(),
        }
    }
}

impl From<DomainError> for ToolError {
    fn from(error: DomainError) -> Self {
        Self {
            code: error.code().as_str().to_owned(),
            message: error.to_string(),
        }
    }
}

impl From<ValueError> for ToolError {
    fn from(error: ValueError) -> Self {
        Self {
            code: "invalid".to_owned(),
            message: error.to_string(),
        }
    }
}

/// D15's error carrier: a §6.1 code and its message as a tool-level
/// error result — `isError: true`, `structuredContent.code` for a
/// client to branch on, the same JSON mirrored into the text block for
/// a model to read. Every tool failure goes through this one helper;
/// t34+ inherit it.
pub(crate) fn error_carrier(code: &str, message: &str) -> CallToolResult {
    CallToolResult::structured_error(serde_json::json!({
        "code": code,
        "message": message,
    }))
}

/// The shared success arm: the DTO serialized into the structured
/// result. The one failure path that bypasses the carrier, on purpose:
/// no §6.1 code names a serializer fault, the path is unreachable for
/// DTOs of strings, integers and booleans, and a protocol-level
/// internal error is the honest carrier for it. One home beside
/// [`error_carrier`] so every cluster — registry, query, and the
/// clusters t35+ land — shares the mechanism instead of copying it.
pub(crate) fn structured<T: Serialize>(tool: &str, output: T) -> Result<CallToolResult, ErrorData> {
    let value = serde_json::to_value(output).map_err(|error| {
        ErrorData::internal_error(
            format!("failed to serialize {tool}'s output: {error}"),
            None,
        )
    })?;
    Ok(CallToolResult::structured(value))
}
