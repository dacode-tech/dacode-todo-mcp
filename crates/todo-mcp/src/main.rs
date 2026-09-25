//! The MCP server binary (spec §2).
//!
//! The rmcp wire surface over stdio lands at M3 (spec §6, §14). This
//! binary translates and never decides: all selection, ordering and
//! query logic lives in `todo-domain`, all persistence in `todo-store`.
//! The binary may adopt tokio for rmcp; the libraries must not require
//! it (spec §2).

use std::process::ExitCode;

/// Entry point. `clippy::exit` is denied workspace-wide — only the
/// binary decides what is fatal, and it decides by return value, not by
/// `process::exit`.
fn main() -> ExitCode {
    ExitCode::SUCCESS
}
