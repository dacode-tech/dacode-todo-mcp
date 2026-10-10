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
//! `progress` renders the total-progress document and the HTML page
//! (t68's and t69's slices), writing the page and its JSON twin
//! beside the repo's checkout unless `--json`/`--no-write`/`--out`
//! say otherwise; `refresh` (t70's slice) re-imports a repo's board —
//! the registry surgery, the re-register, the census; `serve` (t71's
//! slice) is the live listener: the fresh-render page over HTTP, the
//! WebSocket push channel, the port published per-user so a second
//! invocation attaches; the later subcommands (the inspector
//! absorption) land as their own tasks. The globals are
//! shared: `--root <DIR>`
//! names the store location on every path — nothing reads the
//! environment, a home directory or the cwd to find one — and
//! `--repo <SLUG>`, validated here, is the server's default slug
//! (D8) or the subcommand's repo selection, resolved flag over
//! working-directory match (§7). The cwd is read once, here at
//! startup, and passed into the lib as a value (D1): the lib stays
//! env-free, and a read that fails — a deleted cwd — passes `None`
//! and derives no default. An explicit tool parameter outranks both
//! tiers.
//!
//! Over the 600-line target (U38): the argument layer is one file by
//! design — the spec's frozen baselines name this file as the
//! dispatch parse tests' home, so a split would move what the spec
//! pins. The split that retires the debt: the `USAGE` text and the
//! parse into an `args` module, the run_* drivers staying here.

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
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use rmcp::ServiceExt;
use todo_domain::RepoSlug;
use todo_mcp::TodoServer;
use todo_mcp::render;
use todo_mcp::serve::{POLL_INTERVAL, ServeOutcome};
use todo_store::DataRoot;

/// The usage-fault exit code (the cli's convention).
const EXIT_USAGE: u8 = 2;

/// The subcommand-failure exit code: the repo did not resolve, the
/// store did not read or write, the page did not write, or the
/// refresh refused — environment faults, not argument faults.
const EXIT_RENDER: u8 = 1;

/// The hand-written usage text every argument fault appends — a fault
/// always carries the way to use the tool correctly, on stderr where
/// the fault belongs.
const USAGE: &str = "\
todo-mcp — the dacode-todo MCP server and tool CLI (spec §13).

Usage:
  todo-mcp --root <DIR> [--repo <SLUG>]                    the MCP server on stdio
  todo-mcp --root <DIR> [--repo <SLUG>] progress [flags]   the total-progress render
  todo-mcp --root <DIR> [--repo <SLUG>] refresh            re-import the repo's board
  todo-mcp --root <DIR> serve [--port-file <PATH>] [--open]
                                                           the live progress listener

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

With `refresh`: no flags of its own — the repo resolves through the
      same --repo/cwd tier, the board's location derives from the
      repo's registered checkout, and the dialect is the slug.

Exit codes:
  0  the client closed the connection after a completed handshake,
      the listener shut down cleanly on the interrupt (or another
      invocation attached to it), or the subcommand succeeded
  1  the serve loop failed, the listener did not start (a root with no
      registry, or a home directory that cannot be found without
      --port-file), the render failed (a repo that does not resolve,
      or a store that will not read), the page did not write,
      or the refresh refused (the board is gone or broken, a live
      session holds the store, or the import failed)
  2  usage: a bad flag, a missing --root, a malformed --repo value, an
      unknown subcommand, or flags that cannot combine
";

/// The parsed command line (§13, D20): which of the binary's shapes
/// this invocation is. The globals are shared; the first positional
/// selects.
#[derive(Debug, PartialEq, Eq)]
enum Command {
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
}

/// Entry point. `clippy::exit` is denied workspace-wide — the binary
/// decides what is fatal by return value, never by aborting the process
/// behind the runtime's back.
#[tokio::main]
async fn main() -> ExitCode {
    match parse_args(std::env::args_os().skip(1)) {
        Ok(Command::McpServer { root, default_repo }) => {
            // The cwd, read once and eagerly (D1): the one environment
            // touch this binary makes for the default tier, passed in
            // as a value so the lib stays env-free. `None` when the
            // read fails — a deleted cwd — and the lib derives no
            // default from an absent cwd.
            let cwd = std::env::current_dir().ok();
            serve_stdio(DataRoot::new(root), default_repo, cwd).await
        }
        Ok(Command::Refresh { root, repo }) => run_refresh(root, repo),
        Ok(Command::Progress {
            root,
            repo,
            json,
            open,
            out,
            no_write,
        }) => run_progress(root, repo, json, open, out, no_write),
        Ok(Command::Serve {
            root,
            port_file,
            open,
        }) => run_serve(root, port_file, open).await,
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
async fn serve_stdio(
    root: DataRoot,
    default_repo: Option<RepoSlug>,
    cwd: Option<PathBuf>,
) -> ExitCode {
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

/// The `serve` subcommand (t71, D20): the live listener. The default
/// port-file path is the one environment touch this path makes — the
/// binary reads the home directory (D8: the lib stays env-free), and a
/// home that cannot be found is an environment fault naming the
/// `--port-file` remedy. The serving line prints before the await:
/// stdout is line-buffered, so a piped consumer sees the URL before
/// the listener blocks on the interrupt. SIGINT is the clean shutdown
/// (the lib's `Listener::shutdown` removes the publication if it is
/// still ours); SIGTERM keeps the default disposition — the process
/// dies, the stale publication survives, the next invocation takes
/// over.
async fn run_serve(root: OsString, port_file: Option<OsString>, open: bool) -> ExitCode {
    let root_display = root.clone();
    let root = DataRoot::new(root);
    let port_file = match port_file {
        Some(path) => PathBuf::from(path),
        None => match default_port_file() {
            Ok(path) => path,
            Err(message) => {
                eprintln!("todo-mcp: {message}");
                return ExitCode::from(EXIT_RENDER);
            }
        },
    };
    let cwd = std::env::current_dir().ok();
    match todo_mcp::serve::serve(&root, &port_file, cwd.as_deref(), POLL_INTERVAL) {
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            ExitCode::from(EXIT_RENDER)
        }
        Ok(ServeOutcome::Attached { url }) => {
            println!("already serving at {url} — attached, no second listener started");
            if open {
                open_in_browser(&url);
            }
            ExitCode::SUCCESS
        }
        Ok(ServeOutcome::Started { url, listener }) => {
            println!(
                "serving {url} (root {}; Ctrl-C to stop)",
                root_display.display()
            );
            if open {
                open_in_browser(&url);
            }
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    listener.shutdown(&port_file);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("todo-mcp: the interrupt handler failed: {error}");
                    listener.shutdown(&port_file);
                    ExitCode::from(EXIT_RENDER)
                }
            }
        }
    }
}

/// The default port-file path (D1): the per-user runtime location, the
/// Python tool's exact path so `tools/progress.py --serve` and the
/// subcommand interoperate until t73 retires the Python.
fn default_port_file() -> Result<PathBuf, String> {
    std::env::home_dir()
        .map(|home| {
            home.join(".local")
                .join("state")
                .join("dacode-todo-mcp")
                .join("serve.port")
        })
        .ok_or_else(|| "the home directory could not be read — pass --port-file <PATH>".to_owned())
}

/// The `progress` subcommand (D20): resolve the repo — the flag wins,
/// else the cwd tier, the binary reading the cwd once here so the lib
/// stays env-free — then render and print. `--json` prints the
/// document and writes nothing (the Python tool's "instead of
/// writing" contract); the default writes the page and its JSON twin
/// beside the summary, `--out` redirects the pair, `--no-write`
/// suppresses them. A failure is exit 1 with the message on stderr:
/// the repo did not resolve, the store did not read, or the page did
/// not write — environment faults, not argument faults.
fn run_progress(
    root: OsString,
    repo: Option<RepoSlug>,
    json: bool,
    open: bool,
    out: Option<OsString>,
    no_write: bool,
) -> ExitCode {
    let root = DataRoot::new(root);
    let slug = match resolve_repo(&root, repo) {
        Ok(slug) => slug,
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            return ExitCode::from(EXIT_RENDER);
        }
    };
    if json {
        return match render::json(&root, &slug) {
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
        };
    }
    let outputs = match render::outputs(&root, &slug) {
        Ok(outputs) => outputs,
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            return ExitCode::from(EXIT_RENDER);
        }
    };
    print!("{}", outputs.summary);
    // The default output location: the rendered repo's registered
    // checkout plus `.agents/cache` — the Python tool's own location,
    // generalized per-repo (the repo node's path is the one source of
    // truth for where the checkout lives, §7). The CLI convention
    // stays here in the binary.
    let out_dir = match &out {
        Some(dir) => PathBuf::from(dir),
        None => outputs.checkout.join(".agents/cache"),
    };
    if !no_write {
        if let Err(message) = write_pair(&out_dir, &outputs) {
            eprintln!("todo-mcp: {message}");
            return ExitCode::from(EXIT_RENDER);
        }
        println!(
            "  wrote {} and {}",
            out_dir.join("progress.html").display(),
            out_dir.join("progress.json").display()
        );
    }
    if open {
        open_in_browser(&out_dir.join("progress.html").display().to_string());
    }
    ExitCode::SUCCESS
}

/// The subcommands' shared repo tier (§7): the `--repo` flag wins,
/// else the cwd match — the binary reading the cwd once here so the
/// lib stays env-free. The message is the caller's to print.
fn resolve_repo(root: &DataRoot, repo: Option<RepoSlug>) -> Result<RepoSlug, String> {
    match repo {
        Some(slug) => Ok(slug),
        None => {
            let cwd = std::env::current_dir().ok();
            render::resolve_by_cwd(root, cwd.as_deref())
        }
    }
}

/// The `refresh` subcommand (D20): resolve the repo, then re-import
/// the board and print the census. A refusal is exit 1 with the
/// message on stderr: the board is gone or broken, a live session
/// holds the store, or the import failed — environment faults, not
/// argument faults. The refusals before the surgery leave the store
/// exactly as it was (an import failure after it leaves a fresh,
/// empty store — the next refresh recovers).
fn run_refresh(root: OsString, repo: Option<RepoSlug>) -> ExitCode {
    let root_display = root.clone();
    let root = DataRoot::new(root);
    let slug = match resolve_repo(&root, repo) {
        Ok(slug) => slug,
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            return ExitCode::from(EXIT_RENDER);
        }
    };
    match todo_mcp::refresh::refresh(&root, &slug) {
        Ok(census) => {
            println!(
                "refresh: imported {} epics, {} decisions, {} docs, {} tasks, {} blocks edges into {}",
                census.epics,
                census.decisions,
                census.docs,
                census.tasks,
                census.blocks,
                root_display.display()
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            ExitCode::from(EXIT_RENDER)
        }
    }
}

/// Writes the page and its JSON twin — plain writes, the Python
/// tool's own discipline: the pair is a derived cache, not store
/// data, and a torn write is a stale page the next run replaces
/// (§4.3's temp+fsync+rename is the store's contract, not the
/// renderer's).
fn write_pair(out_dir: &Path, outputs: &render::Outputs) -> Result<(), String> {
    std::fs::create_dir_all(out_dir)
        .map_err(|error| format!("create the output directory: {error}"))?;
    std::fs::write(out_dir.join("progress.html"), outputs.page.as_bytes())
        .map_err(|error| format!("write progress.html: {error}"))?;
    std::fs::write(out_dir.join("progress.json"), outputs.json.as_bytes())
        .map_err(|error| format!("write progress.json: {error}"))?;
    Ok(())
}

/// Opens a target — the written page's path, or the served page's URL
/// — with the platform opener, never fatal: a spawn failure or a
/// non-zero opener status prints the target instead (the Python
/// tool's `check=False`).
fn open_in_browser(target: &str) {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        println!("  open it yourself: {target}");
        return;
    };
    if std::process::Command::new(program)
        .arg(target)
        .status()
        .is_err()
    {
        println!("  open it yourself: {target}");
    }
}

/// Parses the command line — program name excluded, the shape
/// `lexopt::Parser::from_args` wants. The globals (`--root`, `--repo`)
/// are collected position-independently; the FIRST positional selects
/// the command (`progress` routes to the render, `serve` to the
/// listener, no positional at all to the server); any other positional
/// is a usage fault naming the attempted subcommand. lexopt (U14) is
/// pedantic: anything but the known flags and their values is a usage
/// fault, never a guess. The root is validated nowhere here — the
/// store reports a missing one as its honest `io` fault — while the
/// slug is validated at the parse: configuration is validated where it
/// enters, so a malformed `--repo` value is a usage fault (exit 2),
/// not a wire error.
fn parse_args<I>(args: I) -> Result<Command, String>
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
    let mut port_file: Option<OsString> = None;
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
            lexopt::Arg::Value(value) => {
                let text = value
                    .to_str()
                    .ok_or_else(|| "todo-mcp: the subcommand name is not valid UTF-8".to_owned())?;
                if progress || refresh || serve {
                    return Err(format!("todo-mcp: unexpected argument {text:?}"));
                }
                match text {
                    "progress" => progress = true,
                    "refresh" => refresh = true,
                    "serve" => serve = true,
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
    } else {
        Command::McpServer {
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
}
