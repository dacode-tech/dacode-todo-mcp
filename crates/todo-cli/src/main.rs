//! The store inspector/debugger (D5, spec §2): read and verify a repo's
//! store without an MCP client. Real commands land with the store at
//! M1/M2 (spec §14).

use std::process::ExitCode;

/// Entry point. `clippy::exit` is denied workspace-wide — only the
/// binary decides what is fatal, and it decides by return value, not by
/// `process::exit`.
fn main() -> ExitCode {
    ExitCode::SUCCESS
}
