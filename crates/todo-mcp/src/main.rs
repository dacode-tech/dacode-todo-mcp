//! The server binary (spec §2, §13): the argument vector in, an
//! [`ExitCode`] out. The argument parse, the runtime and the serve
//! call live here and nothing else — the binary translates, it does
//! not decide: the server, its state and the wire DTOs are the lib's,
//! all selection and ordering the domain's, all persistence the
//! store's.
//!
//! The data root is explicit configuration (§7/§13/D8): `--root <DIR>`
//! names the store location; nothing reads the environment, a home
//! directory or the cwd to find one, and cwd auto-detection is M5's
//! deferred convenience, never a heuristic this binary grows.

// Tests are exempt from the panic-family lints through a mechanism, not
// a convention (the workspace's rule, stated once in the root
// Cargo.toml's lint table). The bin target does not inherit the lib's
// header, so it carries its own.
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

use std::ffi::OsString;
use std::process::ExitCode;

use rmcp::ServiceExt;
use todo_mcp::TodoServer;
use todo_store::DataRoot;

/// The usage-fault exit code (the cli's convention).
const EXIT_USAGE: u8 = 2;

/// The hand-written usage text every argument fault appends — a fault
/// always carries the way to use the tool correctly, on stderr where
/// the fault belongs.
const USAGE: &str = "\
todo-mcp — the dacode-todo MCP server over stdio (spec §13).

Usage:
  todo-mcp --root <DIR>

Options:
      --root <DIR>    The data root holding repos/<slug>/ and the shared
                      registry (required; D4's default location is
                      ~/.local/share/dacode-todo/, resolved by the caller)

Exit codes:
  0  the client closed the connection after a completed handshake
  1  the serve loop failed: the transport, the service, or a peer that
     closes before the initialize handshake completes
  2  usage: a bad flag or a missing --root
";

/// Entry point. `clippy::exit` is denied workspace-wide — the binary
/// decides what is fatal by return value, never by aborting the process
/// behind the runtime's back.
#[tokio::main]
async fn main() -> ExitCode {
    match parse_root(std::env::args_os().skip(1)) {
        Ok(root) => serve(DataRoot::new(root)).await,
        Err(message) => {
            // The message, a blank line, then the usage text (the cli's
            // convention).
            eprintln!("{message}\n");
            eprint!("{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

/// Serves over stdio until the client closes the connection. A quit
/// after a completed handshake — the transport ended, the peer went
/// away — is a stdio server's normal end (exit 0). A peer that closes
/// *before* the handshake completes is a serve failure (exit 1,
/// `connection closed: initialize request`): the server never got
/// initialized, and hiding that behind exit 0 would report a session
/// that never existed.
async fn serve(root: DataRoot) -> ExitCode {
    let server = TodoServer::new(root);
    match server.serve(rmcp::transport::stdio()).await {
        Ok(service) => match service.waiting().await {
            Ok(_) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("todo-mcp: the service quit: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("todo-mcp: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Parses `--root <DIR>` from the argument vector — program name
/// excluded, the shape `lexopt::Parser::from_args` wants. lexopt (U14)
/// is pedantic: anything but `--root` and its value is a usage fault,
/// never a guess, and the root is validated nowhere here — the store
/// reports a missing one as its honest `io` fault.
fn parse_root<I>(args: I) -> Result<OsString, String>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut root: Option<OsString> = None;
    while let Some(arg) = parser
        .next()
        .map_err(|error| format!("todo-mcp: {error}"))?
    {
        match arg {
            lexopt::Arg::Long("root") => {
                root = Some(
                    parser
                        .value()
                        .map_err(|error| format!("todo-mcp: {error}"))?,
                );
            }
            unexpected => {
                let error = unexpected.unexpected();
                return Err(format!("todo-mcp: {error}"));
            }
        }
    }
    root.ok_or_else(|| "todo-mcp: the required option '--root <DIR>' was not provided".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exit-contract coverage the usage faults had none of: the
    /// parse is total, every fault is a message the binary prints with
    /// the usage text and exit 2, and the messages are pinned exactly
    /// because they are the operator-facing contract.
    #[test]
    fn a_missing_root_is_a_usage_fault() {
        assert_eq!(
            parse_root(Vec::<OsString>::new()),
            Err("todo-mcp: the required option '--root <DIR>' was not provided".to_owned())
        );
    }

    #[test]
    fn an_unknown_flag_is_a_usage_fault() {
        assert_eq!(
            parse_root(["--bogus", "x"]),
            Err("todo-mcp: invalid option '--bogus'".to_owned())
        );
    }

    #[test]
    fn a_stray_positional_is_a_usage_fault() {
        assert_eq!(
            parse_root(["camera"]),
            Err("todo-mcp: unexpected argument \"camera\"".to_owned())
        );
    }

    #[test]
    fn the_happy_path_takes_the_root_value() {
        assert_eq!(
            parse_root(["--root", "/tmp/data"]),
            Ok(OsString::from("/tmp/data"))
        );
    }
}
