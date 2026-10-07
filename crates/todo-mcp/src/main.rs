//! The server binary (spec §2, §13): the argument vector in, an
//! [`ExitCode`] out. The argument parse, the runtime and the serve
//! call live here and nothing else — the binary translates, it does
//! not decide: the server, its state and the wire DTOs are the lib's,
//! all selection and ordering the domain's, all persistence the
//! store's.
//!
//! The data root is explicit configuration (§7/§13/D8): `--root <DIR>`
//! names the store location; nothing reads the environment, a home
//! directory or the cwd to find one. The default repo slug resolves
//! in three tiers (the committed D8, its cwd tier armed at M5/t53):
//! `--repo <SLUG>`, validated here — configuration is validated where
//! it enters, so a malformed value is a usage fault, not a wire error
//! the first tool call would answer — else the working directory's
//! deterministic match against the registered checkouts (§7). The cwd
//! is read once, here at startup, and passed into the lib as a value
//! (D1): the lib stays env-free, and a read that fails — a deleted
//! cwd — passes `None` and derives no default. An explicit tool
//! parameter outranks both tiers.

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
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use rmcp::ServiceExt;
use todo_domain::RepoSlug;
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
  todo-mcp --root <DIR> [--repo <SLUG>]

Options:
      --root <DIR>    The data root holding repos/<slug>/ and the shared
                      registry (required; D4's default location is
                      ~/.local/share/dacode-todo/, resolved by the caller)
      --repo <SLUG>   The default repository slug (D8): applied whenever
                      a tool's repo parameter is absent, outranking the
                      working-directory match (§7); an explicit
                      parameter always wins

Exit codes:
  0  the client closed the connection after a completed handshake
  1  the serve loop failed: the transport, the service, or a peer that
     closes before the initialize handshake completes
  2  usage: a bad flag, a missing --root, or a malformed --repo value
";

/// The parsed startup configuration: the data root and the optional
/// default repo slug.
#[derive(Debug, PartialEq, Eq)]
struct Config {
    /// The data root (§7/§13/D4): every store and the shared registry
    /// live under it, validated nowhere here — the store reports a
    /// missing one as its honest `io` fault.
    root: OsString,
    /// The default repo slug (the committed D8): applied whenever a
    /// tool's `repo` parameter is absent; an explicit parameter always
    /// wins. `None` when the flag was not given.
    default_repo: Option<RepoSlug>,
}

/// Entry point. `clippy::exit` is denied workspace-wide — the binary
/// decides what is fatal by return value, never by aborting the process
/// behind the runtime's back.
#[tokio::main]
async fn main() -> ExitCode {
    match parse_args(std::env::args_os().skip(1)) {
        Ok(config) => {
            // The cwd, read once and eagerly (D1): the one environment
            // touch this binary makes for the default tier, passed in
            // as a value so the lib stays env-free. `None` when the
            // read fails — a deleted cwd — and the lib derives no
            // default from an absent cwd.
            let cwd = std::env::current_dir().ok();
            serve(DataRoot::new(config.root), config.default_repo, cwd).await
        }
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
async fn serve(root: DataRoot, default_repo: Option<RepoSlug>, cwd: Option<PathBuf>) -> ExitCode {
    let server = TodoServer::new(root, default_repo, cwd);
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

/// Parses `--root <DIR>` and the optional `--repo <SLUG>` from the
/// argument vector — program name excluded, the shape
/// `lexopt::Parser::from_args` wants. lexopt (U14) is pedantic:
/// anything but the two flags and their values is a usage fault,
/// never a guess. The root is validated nowhere here — the store
/// reports a missing one as its honest `io` fault — while the slug is
/// validated at the parse: configuration is validated where it
/// enters, so a malformed `--repo` value is a usage fault (exit 2),
/// not a wire error.
fn parse_args<I>(args: I) -> Result<Config, String>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut root: Option<OsString> = None;
    let mut repo: Option<RepoSlug> = None;
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
            lexopt::Arg::Long("repo") => {
                let value = parser
                    .value()
                    .map_err(|error| format!("todo-mcp: {error}"))?;
                let text = value.to_str().ok_or_else(|| {
                    "todo-mcp: the value of option '--repo <SLUG>' is not valid UTF-8".to_owned()
                })?;
                let slug = RepoSlug::from_str(text)
                    .map_err(|error| format!("todo-mcp: --repo: {error}"))?;
                repo = Some(slug);
            }
            unexpected => {
                let error = unexpected.unexpected();
                return Err(format!("todo-mcp: {error}"));
            }
        }
    }
    let root = root.ok_or_else(|| {
        "todo-mcp: the required option '--root <DIR>' was not provided".to_owned()
    })?;
    Ok(Config {
        root,
        default_repo: repo,
    })
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
            parse_args(Vec::<OsString>::new()),
            Err("todo-mcp: the required option '--root <DIR>' was not provided".to_owned())
        );
    }

    #[test]
    fn an_unknown_flag_is_a_usage_fault() {
        assert_eq!(
            parse_args(["--bogus", "x"]),
            Err("todo-mcp: invalid option '--bogus'".to_owned())
        );
    }

    #[test]
    fn a_stray_positional_is_a_usage_fault() {
        assert_eq!(
            parse_args(["camera"]),
            Err("todo-mcp: unexpected argument \"camera\"".to_owned())
        );
    }

    #[test]
    fn the_happy_path_takes_the_root_value() {
        assert_eq!(
            parse_args(["--root", "/tmp/data"]),
            Ok(Config {
                root: OsString::from("/tmp/data"),
                default_repo: None
            })
        );
    }

    #[test]
    fn a_missing_repo_value_is_a_usage_fault() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--repo"]),
            Err("todo-mcp: missing argument for option '--repo'".to_owned())
        );
    }

    #[test]
    fn a_malformed_repo_slug_is_a_usage_fault() {
        // Configuration is validated where it enters: a malformed slug
        // is a usage fault (exit 2), never a wire error the first tool
        // call would answer.
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--repo", "Bad!"]),
            Err("todo-mcp: --repo: invalid repository slug: \"Bad!\"".to_owned())
        );
    }

    #[test]
    fn the_happy_path_takes_the_root_and_repo_values() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--repo", "fixture"]),
            Ok(Config {
                root: OsString::from("/tmp/data"),
                default_repo: Some(RepoSlug::from_str("fixture").unwrap()),
            })
        );
    }

    #[test]
    fn the_repo_flag_orders_freely_around_the_root_flag() {
        // lexopt walks the vector, not a fixed flag order, so either
        // order parses to the same configuration.
        assert_eq!(
            parse_args(["--repo", "fixture", "--root", "/tmp/data"]),
            Ok(Config {
                root: OsString::from("/tmp/data"),
                default_repo: Some(RepoSlug::from_str("fixture").unwrap()),
            })
        );
    }
}
