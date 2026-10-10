//! The argument parse (§13, D20; U38's split): the argument vector in,
//! a typed [`Command`] or a usage-fault message out, plus the
//! hand-written usage text that `--help` prints and every fault
//! appends.
//!
//! The parse lives in the lib (the todo-cli precedent) so the tests
//! drive it without spawning a process; the binary imports it and
//! owns only the dispatch. The parser is `lexopt` (U14, resolved
//! 2026-09-29): pedantic — it rejects what it does not understand
//! instead of guessing, with no transitive dependency and no proc
//! macros. What lexopt does not own is owned here: the usage text,
//! the per-subcommand switch table (`--json`/`--out`/`--no-write`
//! belong to `progress`, `--port-file` to `serve`,
//! `--dry-run`/`--yes` to `repair` — each rejected everywhere else,
//! never silently ignored), and the slug validation that makes a
//! malformed `--repo` a **usage** fault (exit 2) rather than a store
//! fault.
//!
//! Parsing is total: every failure — lexopt's own, a missing required
//! option, an unknown subcommand, a malformed slug — is a message the
//! caller prints with the usage text; nothing panics. The globals
//! (`--root`, `--repo`) are collected position-independently; the
//! FIRST positional selects the command; `--help`/`-h` short-circuit
//! wherever they appear (per-subcommand help never demands the
//! required options), and `--version`/`-V` is the top level's alone.

use std::ffi::OsString;
use std::str::FromStr;

use todo_domain::RepoSlug;

/// The hand-written usage text every argument fault appends — a fault
/// always carries the way to use the tool correctly, on stderr where
/// the fault belongs — and `--help` prints to stdout. One text, one
/// exit-code table (D5): the server paths' clauses and the
/// inspector's, 0/1/2/3.
pub const USAGE: &str = "\
todo-mcp — the dacode-todo MCP server and tool CLI (spec §13).

Usage:
  todo-mcp --root <DIR> [--repo <SLUG>]                    the MCP server on stdio
  todo-mcp --root <DIR> [--repo <SLUG>] progress [flags]   the total-progress render
  todo-mcp --root <DIR> [--repo <SLUG>] refresh            re-import the repo's board
  todo-mcp --root <DIR> serve [--port-file <PATH>] [--open]
                                                           the live progress listener
  todo-mcp --root <DIR> [--repo <SLUG>] dump               print the store's graph as one canonical JSON document
  todo-mcp --root <DIR> [--repo <SLUG>] verify             report the store's integrity
  todo-mcp --root <DIR> [--repo <SLUG>] repair [--dry-run] [--yes]
                                                           show a torn last line and truncate it on confirmation (§4.3)
  todo-mcp --help | --version                              (and -h/--help per subcommand)

Options:
      --root <DIR>    The data root holding repos/<slug>/ and the shared
                      registry (required on every path; D4's default
                      location is ~/.local/share/dacode-todo/, resolved
                      by the caller)
      --repo <SLUG>   The repository: the server's default slug (D8),
                      applied whenever a tool's repo parameter is
                      absent, or the repo the progress subcommand
                      renders — an explicit value always wins over the
                      working-directory match (§7). Cannot combine
                      with serve: the listener serves every repo in
                      the root, the repo is a URL parameter
      --json          With `progress`: print the JSON document to stdout
                      instead of the summary; writes nothing
      --out <DIR>     With `progress`: write progress.html and
                      progress.json there instead of the rendered
                      repo's .agents/cache
      --no-write      With `progress`: print the summary only, write
                      nothing
      --open          With `progress`: open the written page in a
                      browser; with `serve`: open the served page
      --port-file <PATH>
                      With `serve`: where the listener's port is
                      published (default ~/.local/state/dacode-todo-mcp/
                      serve.port) — a second invocation attaches to a
                      live listener instead of starting another
      --dry-run       With `repair`: classify and report, write nothing
      --yes           With `repair`: truncate without prompting
  -h, --help          Print this help and exit 0
  -V, --version       Print the version and exit 0

With `refresh`: no flags of its own — the repo resolves through the
      same --repo/cwd tier, the board's location derives from the
      repo's registered checkout, and the dialect is the slug.

With `dump`/`verify`/`repair`: the repo resolves through the same
      --repo/cwd tier as every subcommand (§13). dump writes canonical
      bytes plus one trailing LF to stdout and nothing else, so
      `todo-mcp dump … > graph.json` and `| jq .` both work; a fault's
      machine-readable identity is §6.1's code, printed on its own
      line; reports are otherwise human-readable text on stdout and
      faults on stderr, deterministic for a given store state.

Exit codes:
  0  the client closed the connection after a completed handshake,
      the listener shut down cleanly on the interrupt (or another
      invocation attached to it), or the subcommand succeeded
  1  the serve loop failed, the listener did not start (a root with no
      registry, or a home directory that cannot be found without
      --port-file), the render failed (a repo that does not resolve,
      or a store that will not read), the page did not write,
      or the refresh refused (the board is gone or broken, a live
      session holds the store, or the import failed); verify found a
      fault it will not act on; repair refused (unrepairable,
      diverged) or aborted at the prompt
  2  usage: a bad flag, a missing --root, a malformed --repo value, an
      unknown subcommand, or flags that cannot combine
  3  the store refused the operation: any store error — io, locked,
      version_mismatch, a corrupt store document
";

/// The parsed command line (§13, D20): which of the binary's shapes
/// this invocation is. The globals are shared; the first positional
/// selects.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// No subcommand: the MCP server on stdio, unchanged — the
    /// protocol suite and every wire golden pin its bytes. Named for
    /// what it is now that `serve` names the listener (t71's D7).
    McpServer {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The default repo slug (the committed D8).
        default_repo: Option<RepoSlug>,
    },
    /// `refresh`: the on-demand re-import of the repo's board (t70's
    /// slice of D20's tool CLI) — the registry surgery, the
    /// re-register, the apply, the census.
    Refresh {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The repo to refresh — `None` resolves through the cwd tier.
        repo: Option<RepoSlug>,
    },
    /// `progress`: the total-progress render (t68's slice of D20's
    /// tool CLI; t69's the page and the writes).
    Progress {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The repo to render — `None` resolves through the cwd tier.
        repo: Option<RepoSlug>,
        /// Whether to print the JSON document instead of the summary.
        json: bool,
        /// Whether to open the written page in a browser.
        open: bool,
        /// Where to write the pair — `None` is the rendered repo's
        /// `.agents/cache`.
        out: Option<OsString>,
        /// Whether to write nothing.
        no_write: bool,
    },
    /// `serve`: the live progress listener (t71's slice of D20's tool
    /// CLI) — the fresh-render page over HTTP, the WebSocket push
    /// channel, the per-user port publication with attach / takeover
    /// / SIGINT-cleanup.
    Serve {
        /// The data root (§7/§13/D4) — the listener serves every repo
        /// in it; the repo is a URL parameter.
        root: OsString,
        /// Where the port is published — `None` is the per-user
        /// default (D1).
        port_file: Option<OsString>,
        /// Whether to open the served page in a browser.
        open: bool,
    },
    /// `dump`: the store's graph as one canonical JSON document (the
    /// inspector absorbed at t72, D20) — the lib's
    /// [`inspect::dump`](crate::inspect::dump) run.
    Dump {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The repo to dump — `None` resolves through the cwd tier.
        repo: Option<RepoSlug>,
    },
    /// `verify`: the store's integrity report (the inspector absorbed
    /// at t72, D20).
    Verify {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The repo to verify — `None` resolves through the cwd tier.
        repo: Option<RepoSlug>,
    },
    /// `repair`: §4.3's explicit, confirmed repair (the inspector
    /// absorbed at t72, D20) — `--dry-run` classifies and writes
    /// nothing, `--yes` truncates without prompting, and the bare
    /// form prompts.
    Repair {
        /// The data root (§7/§13/D4).
        root: OsString,
        /// The repo to repair — `None` resolves through the cwd tier.
        repo: Option<RepoSlug>,
        /// Whether to classify and report, writing nothing.
        dry_run: bool,
        /// Whether to truncate without prompting.
        yes: bool,
    },
    /// `--help`/`-h`: print [`USAGE`] to stdout and exit 0,
    /// short-circuiting wherever it appears — per-subcommand help
    /// never demands the required options.
    Help,
    /// `--version`/`-V`: print the version line to stdout and exit 0
    /// — the top level's alone, a usage fault after a subcommand.
    Version,
}

/// Parses the command line — program name excluded, the shape
/// `lexopt::Parser::from_args` wants. The globals (`--root`, `--repo`)
/// are collected position-independently; the FIRST positional selects
/// the command (`progress` routes to the render, `serve` to the
/// listener, `dump`/`verify`/`repair` to the inspector, no positional
/// at all to the server); any other positional is a usage fault naming
/// the attempted subcommand. lexopt (U14) is pedantic: anything but
/// the known flags and their values is a usage fault, never a guess.
/// The root is validated nowhere here — the store reports a missing
/// one as its honest `io` fault — while the slug is validated at the
/// parse: configuration is validated where it enters, so a malformed
/// `--repo` value is a usage fault (exit 2), not a wire error.
pub fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut root: Option<OsString> = None;
    let mut repo: Option<RepoSlug> = None;
    let mut refresh = false;
    let mut json = false;
    let mut open = false;
    let mut out: Option<OsString> = None;
    let mut no_write = false;
    let mut progress = false;
    let mut serve = false;
    let mut dump = false;
    let mut verify = false;
    let mut repair = false;
    let mut dry_run = false;
    let mut yes = false;
    let mut port_file: Option<OsString> = None;
    while let Some(arg) = parser
        .next()
        .map_err(|error| format!("todo-mcp: {error}"))?
    {
        match arg {
            lexopt::Arg::Long("help") | lexopt::Arg::Short('h') => {
                reject_attached_value(&mut parser, "help")?;
                return Ok(Command::Help);
            }
            lexopt::Arg::Long("version") | lexopt::Arg::Short('V') => {
                reject_attached_value(&mut parser, "version")?;
                if let Some(subcommand) = [
                    (progress, "progress"),
                    (refresh, "refresh"),
                    (serve, "serve"),
                    (dump, "dump"),
                    (verify, "verify"),
                    (repair, "repair"),
                ]
                .into_iter()
                .find_map(|(seen, name)| seen.then_some(name))
                {
                    return Err(format!(
                        "todo-mcp: unexpected argument '--version' for '{subcommand}': \
                         --version is the top level's own"
                    ));
                }
                return Ok(Command::Version);
            }
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
            lexopt::Arg::Long("open") if progress || serve => {
                open = true;
            }
            lexopt::Arg::Long("open") => {
                return Err(
                    "todo-mcp: the option '--open' belongs to the 'progress' or 'serve' subcommand"
                        .to_owned(),
                );
            }
            lexopt::Arg::Long("no-write") if progress => {
                no_write = true;
            }
            lexopt::Arg::Long("no-write") => {
                return Err(
                    "todo-mcp: the option '--no-write' belongs to the 'progress' subcommand"
                        .to_owned(),
                );
            }
            lexopt::Arg::Long("out") if progress => {
                out = Some(
                    parser
                        .value()
                        .map_err(|error| format!("todo-mcp: {error}"))?,
                );
            }
            lexopt::Arg::Long("out") => {
                return Err(
                    "todo-mcp: the option '--out <DIR>' belongs to the 'progress' subcommand"
                        .to_owned(),
                );
            }
            lexopt::Arg::Long("port-file") if serve => {
                port_file = Some(
                    parser
                        .value()
                        .map_err(|error| format!("todo-mcp: {error}"))?,
                );
            }
            lexopt::Arg::Long("port-file") => {
                return Err(
                    "todo-mcp: the option '--port-file <PATH>' belongs to the 'serve' subcommand"
                        .to_owned(),
                );
            }
            lexopt::Arg::Long("dry-run") if repair => {
                dry_run = true;
            }
            lexopt::Arg::Long("dry-run") => {
                return Err(
                    "todo-mcp: the option '--dry-run' belongs to the 'repair' subcommand"
                        .to_owned(),
                );
            }
            lexopt::Arg::Long("yes") if repair => {
                yes = true;
            }
            lexopt::Arg::Long("yes") => {
                return Err(
                    "todo-mcp: the option '--yes' belongs to the 'repair' subcommand".to_owned(),
                );
            }
            lexopt::Arg::Value(value) => {
                let text = value
                    .to_str()
                    .ok_or_else(|| "todo-mcp: the subcommand name is not valid UTF-8".to_owned())?;
                if progress || refresh || serve || dump || verify || repair {
                    return Err(format!("todo-mcp: unexpected argument {text:?}"));
                }
                match text {
                    "progress" => progress = true,
                    "refresh" => refresh = true,
                    "serve" => serve = true,
                    "dump" => dump = true,
                    "verify" => verify = true,
                    "repair" => repair = true,
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
    if serve && repo.is_some() {
        // The listener serves every repo in the root; a `--repo` flag
        // would be silently ignored — a usage fault instead, never a
        // no-op dressed as configuration.
        return Err(
            "todo-mcp: '--repo' cannot combine with 'serve' (the listener serves every repo \
             in the root; the repo is a URL parameter)"
                .to_owned(),
        );
    }
    if progress {
        // The write-shaping flags cannot combine with each other or
        // with `--json` — a flag that would be silently ignored is a
        // usage fault, never a no-op dressed as configuration (the
        // Python tool's `--serve` exclusivity, applied to the page
        // writes).
        if json && out.is_some() {
            return Err(
                "todo-mcp: '--json' cannot combine with '--out' (it prints the document to \
                 stdout and writes nothing)"
                    .to_owned(),
            );
        }
        if json && open {
            return Err(
                "todo-mcp: '--json' cannot combine with '--open' (nothing is written to open)"
                    .to_owned(),
            );
        }
        if no_write && out.is_some() {
            return Err(
                "todo-mcp: '--no-write' cannot combine with '--out' (nothing is written)"
                    .to_owned(),
            );
        }
        if no_write && open {
            return Err(
                "todo-mcp: '--no-write' cannot combine with '--open' (nothing is written to \
                 open)"
                    .to_owned(),
            );
        }
    }
    Ok(if refresh {
        Command::Refresh { root, repo }
    } else if progress {
        Command::Progress {
            root,
            repo,
            json,
            open,
            out,
            no_write,
        }
    } else if serve {
        Command::Serve {
            root,
            port_file,
            open,
        }
    } else if dump {
        Command::Dump { root, repo }
    } else if verify {
        Command::Verify { root, repo }
    } else if repair {
        Command::Repair {
            root,
            repo,
            dry_run,
            yes,
        }
    } else {
        Command::McpServer {
            root,
            default_repo: repo,
        }
    })
}

/// Rejects an attached value on a flag that takes none (`--help=yes`,
/// `-hx`): lexopt hands it over instead of erroring, and pedantic
/// parsing — U14's reason for choosing lexopt — means refusing it
/// rather than dropping it on the floor.
fn reject_attached_value(parser: &mut lexopt::Parser, name: &str) -> Result<(), String> {
    match parser.optional_value() {
        None => Ok(()),
        Some(value) => {
            let text = value.to_string_lossy();
            Err(format!(
                "todo-mcp: the flag '--{name}' takes no value, got '{text}'"
            ))
        }
    }
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
            Ok(Command::McpServer {
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
                json: false,
                open: false,
                out: None,
                no_write: false
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
                json: true,
                open: false,
                out: None,
                no_write: false
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
            Ok(Command::McpServer {
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
            Ok(Command::McpServer {
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
                json: false,
                open: false,
                out: None,
                no_write: false
            })
        );
    }

    #[test]
    fn the_write_flags_parse_onto_the_subcommand() {
        assert_eq!(
            parse_args([
                "--root",
                "/tmp/data",
                "progress",
                "--out",
                "/tmp/page",
                "--open"
            ]),
            Ok(Command::Progress {
                root: OsString::from("/tmp/data"),
                repo: None,
                json: false,
                open: true,
                out: Some(OsString::from("/tmp/page")),
                no_write: false
            })
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress", "--no-write"]),
            Ok(Command::Progress {
                root: OsString::from("/tmp/data"),
                repo: None,
                json: false,
                open: false,
                out: None,
                no_write: true
            })
        );
    }

    #[test]
    fn the_write_flags_belong_to_the_subcommand() {
        // A write flag without `progress` is a usage fault naming the
        // flag — never a silently ignored option. `--open` is shared
        // with `serve`, so its fault names both.
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--open"]),
            Err(
                "todo-mcp: the option '--open' belongs to the 'progress' or 'serve' subcommand"
                    .to_owned()
            )
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--no-write"]),
            Err(
                "todo-mcp: the option '--no-write' belongs to the 'progress' subcommand".to_owned()
            )
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--out", "/tmp/page"]),
            Err(
                "todo-mcp: the option '--out <DIR>' belongs to the 'progress' subcommand"
                    .to_owned()
            )
        );
    }

    #[test]
    fn json_cannot_combine_with_the_write_flags() {
        // `--json` prints the document and writes nothing, so the
        // write-shaping flags would be silently ignored beside it — a
        // usage fault instead.
        assert_eq!(
            parse_args([
                "--root",
                "/tmp/data",
                "progress",
                "--json",
                "--out",
                "/tmp/page"
            ]),
            Err(
                "todo-mcp: '--json' cannot combine with '--out' (it prints the document to \
                 stdout and writes nothing)"
                    .to_owned()
            )
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress", "--json", "--open"]),
            Err(
                "todo-mcp: '--json' cannot combine with '--open' (nothing is written to open)"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_refresh_subcommand_routes() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "refresh"]),
            Ok(Command::Refresh {
                root: OsString::from("/tmp/data"),
                repo: None
            })
        );
    }

    #[test]
    fn the_refresh_subcommand_takes_the_repo_flag() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "refresh", "--repo", "fixture"]),
            Ok(Command::Refresh {
                root: OsString::from("/tmp/data"),
                repo: Some(RepoSlug::from_str("fixture").unwrap())
            })
        );
    }

    #[test]
    fn the_usage_text_names_the_refresh_subcommand() {
        // The operator-facing contract: the usage every fault prints
        // carries the refresh line and its exit-1 clause.
        assert!(USAGE.contains("refresh            re-import the repo's board"));
        assert!(USAGE.contains("or the refresh refused"));
    }

    #[test]
    fn no_write_cannot_combine_with_the_write_flags() {
        // `--no-write` writes nothing, so `--out` and `--open` would
        // be silently ignored beside it — usage faults instead.
        assert_eq!(
            parse_args([
                "--root",
                "/tmp/data",
                "progress",
                "--no-write",
                "--out",
                "/tmp/page"
            ]),
            Err(
                "todo-mcp: '--no-write' cannot combine with '--out' (nothing is written)"
                    .to_owned()
            )
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress", "--no-write", "--open"]),
            Err(
                "todo-mcp: '--no-write' cannot combine with '--open' (nothing is written to \
                 open)"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_serve_subcommand_routes() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "serve"]),
            Ok(Command::Serve {
                root: OsString::from("/tmp/data"),
                port_file: None,
                open: false
            })
        );
    }

    #[test]
    fn the_serve_flags_parse_onto_the_subcommand() {
        // The globals order freely around the subcommand, and the
        // serve flags land on it.
        assert_eq!(
            parse_args([
                "--root",
                "/tmp/data",
                "serve",
                "--port-file",
                "/tmp/serve.port",
                "--open"
            ]),
            Ok(Command::Serve {
                root: OsString::from("/tmp/data"),
                port_file: Some(OsString::from("/tmp/serve.port")),
                open: true
            })
        );
    }

    #[test]
    fn the_port_file_flag_belongs_to_serve() {
        assert_eq!(
            parse_args(["--root", "/tmp/data", "--port-file", "/tmp/serve.port"]),
            Err(
                "todo-mcp: the option '--port-file <PATH>' belongs to the 'serve' subcommand"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_repo_flag_cannot_combine_with_serve() {
        // The listener serves every repo in the root; a `--repo` flag
        // would be silently ignored — a usage fault instead.
        assert_eq!(
            parse_args(["--root", "/tmp/data", "serve", "--repo", "fixture"]),
            Err(
                "todo-mcp: '--repo' cannot combine with 'serve' (the listener serves every \
                 repo in the root; the repo is a URL parameter)"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_progress_flags_do_not_combine_with_serve() {
        // The progress-only flags beside `serve` are usage faults
        // naming their subcommand — never silently ignored options.
        assert_eq!(
            parse_args(["--root", "/tmp/data", "serve", "--json"]),
            Err("todo-mcp: the option '--json' belongs to the 'progress' subcommand".to_owned())
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "serve", "--out", "/tmp/page"]),
            Err(
                "todo-mcp: the option '--out <DIR>' belongs to the 'progress' subcommand"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_usage_text_names_the_serve_subcommand() {
        // The operator-facing contract: the usage every fault prints
        // carries the serve line, its flags and its exit-code clauses.
        assert!(USAGE.contains("serve [--port-file <PATH>] [--open]"));
        assert!(USAGE.contains("--port-file <PATH>"));
        assert!(USAGE.contains("the listener shut down cleanly on the interrupt"));
        assert!(USAGE.contains("the listener did not start"));
    }

    // =================================================================
    // The inspector's parse coverage (t72, D20): the absorbed args
    // tests adapted to the host wordings (D7), plus the surface the
    // absorption adds — the help/version requests and the
    // per-subcommand switch table's repair rows. The two
    // process-spawning absorbed tests (the exit-code and stream half)
    // live in the `inspect` integration target, where cargo sets
    // `CARGO_BIN_EXE_todo-mcp`.
    // =================================================================

    #[test]
    fn the_inspector_happy_paths_parse_into_typed_commands() {
        // The absorbed three happy paths, adapted: `--repo` is the
        // host's name for the repo dimension, and `repair` carries its
        // two switches.
        assert_eq!(
            parse_args(["dump", "--root", "/data", "--repo", "camera"]),
            Ok(Command::Dump {
                root: OsString::from("/data"),
                repo: Some(RepoSlug::from_str("camera").unwrap())
            })
        );
        assert_eq!(
            parse_args(["verify", "--root", "/data", "--repo", "camera"]),
            Ok(Command::Verify {
                root: OsString::from("/data"),
                repo: Some(RepoSlug::from_str("camera").unwrap())
            })
        );
        assert_eq!(
            parse_args([
                "repair",
                "--root",
                "/data",
                "--repo",
                "camera",
                "--dry-run",
                "--yes"
            ]),
            Ok(Command::Repair {
                root: OsString::from("/data"),
                repo: Some(RepoSlug::from_str("camera").unwrap()),
                dry_run: true,
                yes: true
            })
        );
    }

    #[test]
    fn the_gnu_conventions_lexopt_owns_parse_too() {
        // `--flag=value` and options ahead of the subcommand: the
        // conventions U14 chose a parser for rather than hand-rolling.
        assert_eq!(
            parse_args(["dump", "--root=/data", "--repo=camera"]),
            Ok(Command::Dump {
                root: OsString::from("/data"),
                repo: Some(RepoSlug::from_str("camera").unwrap())
            })
        );
        assert_eq!(
            parse_args(["--root", "/data", "--repo", "camera", "verify"]),
            Ok(Command::Verify {
                root: OsString::from("/data"),
                repo: Some(RepoSlug::from_str("camera").unwrap())
            })
        );
    }

    #[test]
    fn the_repair_switches_are_rejected_on_the_other_subcommands() {
        // The per-subcommand switch table U14 accepted the cost of: a
        // switch on the wrong subcommand is an error, never a silent
        // ignore — including before any subcommand, where it belongs
        // to no table entry at all.
        for args in [
            ["dump", "--root", "/data", "--repo", "camera", "--yes"],
            ["verify", "--root", "/data", "--repo", "camera", "--dry-run"],
            ["--dry-run", "dump", "--root", "/data", "--repo", "camera"],
        ] {
            let error = parse_args(args).unwrap_err();
            assert!(
                error.contains("belongs to the 'repair' subcommand"),
                "{error}"
            );
        }
    }

    #[test]
    fn missing_required_options_are_named() {
        // The host's one required option: a subcommand without `--root`
        // names it. (`--repo` is optional everywhere — it resolves
        // through the cwd tier — so the absorbed `--slug` case has no
        // host twin.)
        let error = parse_args(["dump"]).unwrap_err();
        assert!(error.contains("--root"), "{error}");
    }

    #[test]
    fn unknown_flags_subcommands_and_stray_positionals_are_rejected() {
        let error =
            parse_args(["dump", "--root", "/data", "--repo", "camera", "--bogus"]).unwrap_err();
        assert!(error.contains("--bogus"), "{error}");

        let error = parse_args(["nosuch", "--root", "/data"]).unwrap_err();
        assert!(error.contains("unknown subcommand 'nosuch'"), "{error}");

        let error =
            parse_args(["dump", "--root", "/data", "--repo", "camera", "extra"]).unwrap_err();
        assert!(error.contains("extra"), "{error}");
    }

    #[test]
    fn a_malformed_repo_value_is_a_usage_fault() {
        // §7: the slug is the key — and a key that is not one never
        // reaches the store, so it can never be mistaken for one.
        for bad in ["Bad Slug", "-x", "", "camera-"] {
            let error = parse_args(["dump", "--root", "/data", "--repo", bad]).unwrap_err();
            assert!(error.contains("invalid repository slug"), "{error}");
        }
    }

    #[test]
    fn help_and_version_are_requests_not_errors() {
        assert_eq!(parse_args(["--help"]), Ok(Command::Help));
        assert_eq!(parse_args(["-h"]), Ok(Command::Help));
        assert_eq!(parse_args(["dump", "--help"]), Ok(Command::Help));
        assert_eq!(
            parse_args(["repair", "--root", "/data", "-h"]),
            Ok(Command::Help)
        );
        assert_eq!(parse_args(["--version"]), Ok(Command::Version));
        assert_eq!(parse_args(["-V"]), Ok(Command::Version));
        // The surface extends only --help to the subcommands: a
        // --version after one is a usage fault naming it, in the
        // absorbed wording under the host prefix (D6). Pinned for
        // every subcommand so a dropped row in the check's array
        // cannot survive.
        for subcommand in ["progress", "refresh", "serve", "dump", "verify", "repair"] {
            assert_eq!(
                parse_args([subcommand, "--version"]),
                Err(format!(
                    "todo-mcp: unexpected argument '--version' for '{subcommand}': \
                     --version is the top level's own"
                ))
            );
        }
        // Pedantic: a flag that takes none refuses an attached value.
        let error = parse_args(["--help=yes"]).unwrap_err();
        assert!(error.contains("takes no value"), "{error}");
    }

    #[test]
    fn the_inspector_globals_order_freely_around_the_subcommand() {
        // The globals parse before or after the inspector's
        // subcommand alike — the existing rule, pinned for the
        // absorbed commands.
        let before = parse_args(["--root", "/data", "--repo", "camera", "dump"]);
        let after = parse_args(["dump", "--repo", "camera", "--root", "/data"]);
        assert_eq!(
            before,
            Ok(Command::Dump {
                root: OsString::from("/data"),
                repo: Some(RepoSlug::from_str("camera").unwrap())
            })
        );
        assert_eq!(before, after);
    }

    #[test]
    fn the_inspector_flags_are_rejected_beside_serve_and_progress() {
        // The inspector's switches beside the other subcommands are
        // the same class of fault as the progress flags beside serve.
        assert_eq!(
            parse_args(["--root", "/tmp/data", "serve", "--dry-run"]),
            Err("todo-mcp: the option '--dry-run' belongs to the 'repair' subcommand".to_owned())
        );
        assert_eq!(
            parse_args(["--root", "/tmp/data", "progress", "--yes"]),
            Err("todo-mcp: the option '--yes' belongs to the 'repair' subcommand".to_owned())
        );
    }

    #[test]
    fn the_usage_text_names_the_inspector_subcommands() {
        // The operator-facing contract: the usage every fault prints
        // carries the three subcommands, repair's switches, the
        // help/version lines and the exit-3 clause (D5's one table).
        assert!(USAGE.contains("dump               print the store's graph"));
        assert!(USAGE.contains("verify             report the store's integrity"));
        assert!(USAGE.contains("repair [--dry-run] [--yes]"));
        assert!(USAGE.contains("      --dry-run       With `repair`"));
        assert!(USAGE.contains("      --yes           With `repair`"));
        assert!(USAGE.contains("  -h, --help          Print this help and exit 0"));
        assert!(USAGE.contains("  -V, --version       Print the version and exit 0"));
        assert!(
            USAGE.contains("3  the store refused the operation: any store error — io, locked,")
        );
    }
}
