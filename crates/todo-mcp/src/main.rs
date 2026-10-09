//! The server binary (spec §2, §13): the argument vector in, an
//! [`ExitCode`] out. The argument parse, the runtime and the serve
//! call live here and nothing else — the binary translates, it does
//! not decide: the server, its state and the wire DTOs are the lib's,
//! all selection and ordering the domain's, all persistence the
//! store's.
//!
//! The same binary carries the tool CLI (§13, D20): the first
//! positional selects a subcommand, and no positional at all is the
//! MCP server on stdio — the wiring every client has is unchanged.
//! `progress` (t68's slice) renders the total-progress document; the
//! later subcommands (`refresh`, `serve`, the inspector absorption)
//! land as their own tasks. The globals are shared: `--root <DIR>`
//! names the store location on every path — nothing reads the
//! environment, a home directory or the cwd to find one — and
//! `--repo <SLUG>`, validated here, is the server's default slug
//! (D8) or the subcommand's repo selection, resolved flag over
//! working-directory match (§7). The cwd is read once, here at
//! startup, and passed into the lib as a value (D1): the lib stays
//! env-free, and a read that fails — a deleted cwd — passes `None`
//! and derives no default. An explicit tool parameter outranks both
//! tiers.

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
use todo_mcp::render;
use todo_store::DataRoot;

/// The usage-fault exit code (the cli's convention).
const EXIT_USAGE: u8 = 2;

/// The render-failure exit code: the repo did not resolve, or the
/// store did not read — an environment fault, not an argument fault.
const EXIT_RENDER: u8 = 1;

/// The hand-written usage text every argument fault appends — a fault
/// always carries the way to use the tool correctly, on stderr where
/// the fault belongs.
const USAGE: &str = "\
todo-mcp — the dacode-todo MCP server and tool CLI (spec §13).

Usage:
  todo-mcp --root <DIR> [--repo <SLUG>]                    the MCP server on stdio
  todo-mcp --root <DIR> [--repo <SLUG>] progress [--json]  the total-progress render

Options:
      --root <DIR>    The data root holding repos/<slug>/ and the shared
                      registry (required on every path; D4's default
                      location is ~/.local/share/dacode-todo/, resolved
                      by the caller)
      --repo <SLUG>   The repository: the server's default slug (D8),
                      applied whenever a tool's repo parameter is
                      absent, or the repo the progress subcommand
                      renders — an explicit value always wins over the
                      working-directory match (§7)
      --json          With `progress`: print the JSON document instead
                      of the human summary

Exit codes:
  0  the client closed the connection after a completed handshake, or
     the subcommand succeeded
  1  the serve loop failed, or the render failed: a repo that does not
     resolve, or a store that will not read
  2  usage: a bad flag, a missing --root, a malformed --repo value, or
     an unknown subcommand
";

/// The parsed command line (§13, D20): which of the binary's shapes
/// this invocation is. The globals are shared; the first positional
/// selects.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    /// No subcommand: the MCP server on stdio, unchanged — the
    /// protocol suite and every wire golden pin its bytes.
    Serve {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The default repo slug (the committed D8).
        default_repo: Option<RepoSlug>,
    },
    /// `progress`: the total-progress render (t68's slice of D20's
    /// tool CLI).
    Progress {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The repo to render — `None` resolves through the cwd tier.
        repo: Option<RepoSlug>,
        /// Whether to print the JSON document instead of the summary.
        json: bool,
    },
}

/// Entry point. `clippy::exit` is denied workspace-wide — the binary
/// decides what is fatal by return value, never by aborting the process
/// behind the runtime's back.
#[tokio::main]
async fn main() -> ExitCode {
    match parse_args(std::env::args_os().skip(1)) {
        Ok(Command::Serve { root, default_repo }) => {
            // The cwd, read once and eagerly (D1): the one environment
            // touch this binary makes for the default tier, passed in
            // as a value so the lib stays env-free. `None` when the
            // read fails — a deleted cwd — and the lib derives no
            // default from an absent cwd.
            let cwd = std::env::current_dir().ok();
            serve(DataRoot::new(root), default_repo, cwd).await
        }
        Ok(Command::Progress { root, repo, json }) => run_progress(root, repo, json),
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

/// The `progress` subcommand (D20): resolve the repo — the flag wins,
/// else the cwd tier, the binary reading the cwd once here so the lib
/// stays env-free — then render and print. A failure is exit 1 with
/// the message on stderr: the repo did not resolve or the store did
/// not read, which is an environment fault, not an argument fault.
fn run_progress(root: OsString, repo: Option<RepoSlug>, json: bool) -> ExitCode {
    let root = DataRoot::new(root);
    let slug = match repo {
        Some(slug) => slug,
        None => {
            let cwd = std::env::current_dir().ok();
            match render::resolve_by_cwd(&root, cwd.as_deref()) {
                Ok(slug) => slug,
                Err(message) => {
                    eprintln!("todo-mcp: {message}");
                    return ExitCode::from(EXIT_RENDER);
                }
            }
        }
    };
    let outcome = if json {
        render::json(&root, &slug)
    } else {
        render::summary(&root, &slug)
    };
    match outcome {
        // The render's text already ends in one newline — printed
        // exactly, never doubled.
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            ExitCode::from(EXIT_RENDER)
        }
    }
}

/// Parses the command line — program name excluded, the shape
/// `lexopt::Parser::from_args` wants. The globals (`--root`, `--repo`)
/// are collected position-independently; the FIRST positional selects
/// the command (`progress` routes to the render, no positional at all
/// to the server); any other positional is a usage fault naming the
/// attempted subcommand. lexopt (U14) is pedantic: anything but the
/// known flags and their values is a usage fault, never a guess. The
/// root is validated nowhere here — the store reports a missing one as
/// its honest `io` fault — while the slug is validated at the parse:
/// configuration is validated where it enters, so a malformed
/// `--repo` value is a usage fault (exit 2), not a wire error.
fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut root: Option<OsString> = None;
    let mut repo: Option<RepoSlug> = None;
    let mut json = false;
    let mut progress = false;
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
            lexopt::Arg::Long("json") if progress => {
                json = true;
            }
            lexopt::Arg::Long("json") => {
                return Err(
                    "todo-mcp: the option '--json' belongs to the 'progress' subcommand".to_owned(),
                );
            }
            lexopt::Arg::Value(value) => {
                let text = value
                    .to_str()
                    .ok_or_else(|| "todo-mcp: the subcommand name is not valid UTF-8".to_owned())?;
                if progress {
                    return Err(format!("todo-mcp: unexpected argument {text:?}"));
                }
                match text {
                    "progress" => progress = true,
                    other => return Err(format!("todo-mcp: unknown subcommand '{other}'")),
                }
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
    Ok(if progress {
        Command::Progress { root, repo, json }
    } else {
        Command::Serve {
            root,
            default_repo: repo,
        }
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
    fn an_unknown_subcommand_is_a_usage_fault() {
        // A bare positional reads as an attempted subcommand — the
        // honest message names it, never a bare "unexpected".
        assert_eq!(
            parse_args(["camera"]),
            Err("todo-mcp: unknown subcommand 'camera'".to_owned())
        );
    }

    #[test]
    fn a_positional_after_the_subcommand_is_a_usage_fault() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress", "extra"]),
            Err("todo-mcp: unexpected argument \"extra\"".to_owned())
        );
    }

    #[test]
    fn json_without_the_subcommand_is_a_usage_fault() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--json"]),
            Err("todo-mcp: the option '--json' belongs to the 'progress' subcommand".to_owned())
        );
    }

    #[test]
    fn no_arguments_route_to_the_server() {
        assert_eq!(
            parse_args(["--root", "/tmp/data"]),
            Ok(Command::Serve {
                root: OsString::from("/tmp/data"),
                default_repo: None
            })
        );
    }

    #[test]
    fn the_progress_subcommand_routes_to_the_render() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress"]),
            Ok(Command::Progress {
                root: OsString::from("/tmp/data"),
                repo: None,
                json: false
            })
        );
    }

    #[test]
    fn the_globals_order_freely_around_the_subcommand() {
        // lexopt walks the vector, not a fixed flag order, so the
        // globals parse before or after the subcommand alike.
        let before = parse_args(["--root", "/tmp/data", "progress", "--json"]);
        let after = parse_args(["progress", "--root", "/tmp/data", "--json"]);
        assert_eq!(
            before,
            Ok(Command::Progress {
                root: OsString::from("/tmp/data"),
                repo: None,
                json: true
            })
        );
        assert_eq!(before, after);
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
            Ok(Command::Serve {
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
            Ok(Command::Serve {
                root: OsString::from("/tmp/data"),
                default_repo: Some(RepoSlug::from_str("fixture").unwrap()),
            })
        );
    }

    #[test]
    fn the_progress_subcommand_takes_the_repo_flag() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress", "--repo", "fixture"]),
            Ok(Command::Progress {
                root: OsString::from("/tmp/data"),
                repo: Some(RepoSlug::from_str("fixture").unwrap()),
                json: false
            })
        );
    }
}
