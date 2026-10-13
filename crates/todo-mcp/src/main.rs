//! The server binary (spec §2, §13): the argument vector in, an
//! [`ExitCode`] out. The runtime and the serve call live here; the
//! argument parse is the lib's [`args`] module (U38's split — the
//! parse lives in the lib so the tests drive it without spawning a
//! process) and the server, its state and the wire DTOs are the lib's
//! too — the binary translates, it does not decide: all selection and
//! ordering the domain's, all persistence the store's.
//!
//! The same binary carries the tool CLI (§13, D20): the first
//! positional selects a subcommand, and no positional at all is the
//! stdio shim — the wiring every client has is unchanged. `--daemon`
//! (D25, U65) is the no-subcommand path's extra-argument shape: the
//! shared daemon, one process per root serving every connection's
//! own session over `<root>/serve.sock` — the no-argument shim (U66)
//! spawns it and proxies MCP to it, and the lib's
//! [`daemon`](todo_mcp::daemon) module owns its logic while this arm
//! owns the exit codes. `progress` renders the total-progress
//! document and the HTML page
//! (t68's and t69's slices), writing the page and its JSON twin
//! beside the repo's checkout unless `--json`/`--no-write`/`--out`
//! say otherwise; `refresh` (t70's slice) re-imports a repo's board —
//! the registry surgery, the re-register, the census; `serve` (t71's
//! slice) is the live listener: the fresh-render page over HTTP, the
//! WebSocket push channel, the port published per-root so a second
//! invocation attaches; and the inspector the `todo-cli` crate
//! carried is absorbed (t72, D20): `dump`/`verify`/`repair` —
//! the lib's [`inspect`] module, dispatched below with the repo resolved
//! through the same flag-over-cwd tier as every subcommand. The
//! daemon owns the root's live view (D25's retirement, U66): it
//! ensures the listener once at startup, and the no-subcommand path
//! never touches it — the shim proxies and the watchdog's takeover
//! loop is gone. The globals are shared: `--root <DIR>`
//! names the store location on every path — nothing reads the
//! environment, a home directory or the cwd to find one — and
//! `--repo <SLUG>`, validated at the parse, is the shim's default
//! slug (D8, riding the socket preamble to its session) or the
//! subcommand's repo selection, resolved flag over
//! working-directory match (§7). The cwd is read once, here at
//! startup, and passed into the lib as a value (D1): the lib stays
//! env-free, and a read that fails — a deleted cwd — passes `None`
//! and derives no default. An explicit tool parameter outranks both
//! tiers.

// Tests are exempt from the panic-family lints through a mechanism, not
// a convention (the workspace's rule, stated once in the root
// Cargo.toml's lint table). The bin target does not inherit the lib's
// header, so it carries its own — kept though its tests moved to the
// lib's `args` module at t72: the bin may grow driver tests later, and
// the mechanism is stated once at the lib root regardless.
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
use std::time::Duration;

use todo_domain::RepoSlug;
use todo_mcp::args::{Command, USAGE, parse_args};
use todo_mcp::inspect;
use todo_mcp::render;
use todo_mcp::serve::{POLL_INTERVAL, ServeOutcome};
use todo_store::DataRoot;

/// The usage-fault exit code (the cli's convention).
const EXIT_USAGE: u8 = 2;

/// The subcommand-failure exit code: the repo did not resolve, the
/// store did not read or write, the page did not write, or the
/// refresh refused — environment faults, not argument faults.
const EXIT_RENDER: u8 = 1;

/// The exit teardown's bound: how long the runtime's shutdown waits
/// for in-flight work before cutting it. The bound exists for one
/// case — the shim's abnormal end, where the client task's
/// blocking-pool stdin read is still in flight when the outcome is
/// already decided, and a plain `Runtime::drop` would wait that read
/// out (the client holds its stdin open), so the exit code — the
/// thing a supervisor and U67's restart flow wait on — never
/// arrives. Every other arm, and the shim's normal ends, have
/// nothing in flight at exit, so the shutdown returns immediately
/// and the bound is never reached; a tenth of a second is far
/// beyond noticing already-stopped work.
const EXIT_SHUTDOWN_BOUND: Duration = Duration::from_millis(100);

/// Entry point. `clippy::exit` is denied workspace-wide — the binary
/// decides what is fatal by return value, never by aborting the
/// process behind the runtime's back. The runtime is explicit, not
/// `#[tokio::main]`'s implicit one, because the binary owns the exit
/// and the teardown must not veto it: the dispatch is `block_on`'d,
/// then the runtime is shut down within [`EXIT_SHUTDOWN_BOUND`] —
/// cutting any in-flight blocking op at the bound (the thread is
/// leaked; the process exit reaps it) — and only then is the outcome
/// mapped to the exit code.
fn main() -> ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("todo-mcp: the async runtime could not start: {error}");
            return ExitCode::FAILURE;
        }
    };
    let code = runtime.block_on(dispatch());
    runtime.shutdown_timeout(EXIT_SHUTDOWN_BOUND);
    code
}

/// The dispatch: the argument vector in, the arm's [`ExitCode`] out.
async fn dispatch() -> ExitCode {
    match parse_args(std::env::args_os().skip(1)) {
        Ok(Command::Shim { root, default_repo }) => {
            // The cwd, read once and eagerly (D1): the one environment
            // touch this binary makes for the default tier, passed in
            // as a value so the lib stays env-free. `None` when the
            // read fails — a deleted cwd — and the lib derives no
            // default from an absent cwd. The shim's cwd is the
            // client's: it rides the socket preamble to the session.
            let cwd = std::env::current_dir().ok();
            let root = DataRoot::new(root);
            run_shim(root, default_repo, cwd).await
        }
        Ok(Command::Daemon { root }) => {
            let cwd = std::env::current_dir().ok();
            let root = DataRoot::new(root);
            run_daemon(root, cwd).await
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
        Ok(Command::Dump { root, repo }) => run_inspector(root, repo, inspect::dump::run),
        Ok(Command::Verify { root, repo }) => run_inspector(root, repo, inspect::verify::run),
        Ok(Command::Repair {
            root,
            repo,
            dry_run,
            yes,
        }) => run_repair(root, repo, dry_run, yes),
        Ok(Command::Help) => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("todo-mcp {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
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

/// The no-subcommand invocation (D25, U66): the stdio shim. The lib's
/// [`shim::run`](todo_mcp::shim::run) owns the logic — the
/// connect-or-spawn window, the preamble, the byte proxy, the
/// degraded failure mode — and this arm owns the exit codes: 0 the
/// client closed a completed session (proxied through the daemon, or
/// served degraded after the retry window), 1 the daemon connection
/// ended abnormally — lost mid-session, or ended before a session
/// established (the stdio server's close-before-initialize case).
async fn run_shim(
    root: DataRoot,
    default_repo: Option<RepoSlug>,
    cwd: Option<PathBuf>,
) -> ExitCode {
    match todo_mcp::shim::run(&root, default_repo, cwd).await {
        todo_mcp::shim::ShimOutcome::Completed => ExitCode::SUCCESS,
        todo_mcp::shim::ShimOutcome::Incomplete => ExitCode::FAILURE,
    }
}

/// The `--daemon` invocation (D25, U65): the shared server. The lib's
/// [`daemon::run`](todo_mcp::daemon::run) owns the logic — the socket
/// ensure, the live view, the accept loop, the per-connection
/// sessions, the SIGTERM drain — and this arm owns the exit codes: a
/// live daemon already serving is exit 0 with one line (never a
/// second listener — the serialized ensure makes a racing double-spawn
/// resolve the same way), a drained run is exit 0 whether or not
/// every session ended (the drain bound is the documented contract; a
/// cut connection gets its line), and a socket that could not be
/// ensured — the lock or the bind refused — is exit 1 with the
/// message.
async fn run_daemon(root: DataRoot, cwd: Option<PathBuf>) -> ExitCode {
    match todo_mcp::daemon::run(&root, cwd.as_deref(), todo_mcp::daemon::DRAIN_TIMEOUT).await {
        Ok(todo_mcp::daemon::DaemonOutcome::AlreadyServing) => {
            eprintln!("todo-mcp: a daemon already serves this root — no second one started");
            ExitCode::SUCCESS
        }
        Ok(todo_mcp::daemon::DaemonOutcome::Drained { severed }) => {
            if severed {
                eprintln!(
                    "todo-mcp: the drain bound was reached with connections live — they were cut"
                );
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            ExitCode::from(EXIT_RENDER)
        }
    }
}

/// The `serve` subcommand (t71, D20): the live listener. The default
/// port-file path is the per-root publication (D13, D1) —
/// `<root>/serve.port`, the same file the daemon's startup ensure
/// publishes, one discipline for both paths; `--port-file`
/// stays the override. The serving line prints before the await:
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
        None => todo_mcp::serve::port_file(&root),
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

/// The inspector subcommands' shared dispatch (t72, D20): the repo
/// resolves through the subcommand tier — the flag wins, else the cwd
/// match, the binary reading the cwd once here so the lib stays
/// env-free — then the lib's run does the work. A resolution failure
/// is exit 1 with the message on stderr (the progress/refresh
/// precedent): the option exists, the resolution failed — an
/// environment fault, not an argument fault.
fn run_inspector(
    root: OsString,
    repo: Option<RepoSlug>,
    run: fn(&DataRoot, &RepoSlug) -> ExitCode,
) -> ExitCode {
    let root = DataRoot::new(root);
    match resolve_repo(&root, repo) {
        Ok(slug) => run(&root, &slug),
        Err(message) => {
            eprintln!("todo-mcp: {message}");
            ExitCode::from(EXIT_RENDER)
        }
    }
}

/// `repair`'s dispatch: the same resolution, then the lib's run with
/// the two switches the parse already validated onto the subcommand.
fn run_repair(root: OsString, repo: Option<RepoSlug>, dry_run: bool, yes: bool) -> ExitCode {
    let root = DataRoot::new(root);
    match resolve_repo(&root, repo) {
        Ok(slug) => inspect::repair::run(&root, &slug, dry_run, yes),
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
