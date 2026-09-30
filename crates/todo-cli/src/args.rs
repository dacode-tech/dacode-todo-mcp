//! The argument parse: an argument vector in, a typed [`Request`] or an
//! [`ArgsError`] out, plus the hand-written usage text that `--help`
//! prints and every failure appends.
//!
//! The parser is `lexopt` (U14, resolved 2026-09-29): pedantic — it
//! rejects what it does not understand instead of guessing — with no
//! transitive dependency and no proc macros. What lexopt does not own is
//! owned here: the usage text, the per-subcommand switch table
//! (`--dry-run`/`--yes` belong to `repair` alone and are *rejected* on
//! `dump` and `verify`, never silently ignored), and the slug validation
//! that makes a malformed `--slug` a **usage** fault (exit-code 2) rather
//! than a store fault (3).
//!
//! Parsing is total: every failure — lexopt's own, a missing required
//! option, an unknown subcommand, a malformed slug — is an [`ArgsError`]
//! value the caller prints; nothing panics (`unwrap`/`expect` are denied
//! workspace-wide). [`parse`] takes the argument vector explicitly —
//! program name excluded, the shape [`lexopt::Parser::from_args`] wants —
//! so the lib target's tests drive it without spawning a process.

use std::ffi::OsString;
use std::str::FromStr;

use todo_domain::RepoSlug;

/// The hand-written usage text (U14 accepted its cost because this cli
/// wants to own its diagnostics): printed to stdout by `--help` — top
/// level and per subcommand — and appended to stderr after every
/// [`ArgsError`] message, so a failure always carries the way to use the
/// tool correctly.
pub const USAGE: &str = "\
todo-cli — inspect and repair a dacode-todo store without an MCP client (D5).

Usage:
  todo-cli <COMMAND> --root <DIR> --slug <SLUG> [OPTIONS]
  todo-cli --help | --version            (and -h/--help per subcommand)

Commands:
  dump    Print the store's graph as one canonical JSON document
  verify  Report the store's integrity; classify a corrupt store it will not fix
  repair  Show a torn last line and truncate it on confirmation (§4.3)

Options:
      --root <DIR>    The data root holding repos/<slug>/ (required)
      --slug <SLUG>   The repository slug (§7: the slug is the key; required)
      --dry-run       repair only: classify and report, write nothing
      --yes           repair only: truncate without prompting
  -h, --help          Print this help and exit 0
  -V, --version       Print the version and exit 0

Exit codes:
  0  the command did its job — for verify that includes a clean store,
     for repair \"repaired\" and \"nothing to repair\" alike
  1  a fault reported that the command will not act on: verify on a
     faulty store; repair refused (unrepairable, diverged) or aborted
     at the prompt
  2  usage: a bad flag, a missing required option, an unknown
     subcommand, a malformed slug
  3  the store refused the operation: any store error — io, locked,
     version_mismatch, a corrupt store document

A fault's machine-readable identity is §6.1's code, printed on its own
line. dump writes canonical bytes plus one trailing LF to stdout and
nothing else, so `todo-cli dump … > graph.json` and `| jq .` both work;
reports are otherwise human-readable text on stdout and faults on stderr,
deterministic for a given store state — no clock, no map order, no host
path beyond the --root the operator supplied.
";

/// Which of the three subcommands an [`Invocation`] runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Print the store's graph as one canonical JSON document.
    Dump,
    /// Report the store's integrity.
    Verify,
    /// §4.3's explicit, confirmed repair.
    Repair,
}

impl Command {
    /// The subcommand's own name, as the surface spells it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Dump => "dump",
            Self::Verify => "verify",
            Self::Repair => "repair",
        }
    }
}

/// One fully parsed invocation: the subcommand, the two required options
/// and `repair`'s two switches. Every value is validated during the parse
/// — the slug through [`RepoSlug::from_str`], so a malformed one is an
/// [`ArgsError`] and never reaches the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    /// The subcommand to run.
    pub command: Command,
    /// The injected data root (D4/§13: the location is configuration the
    /// operator supplies; the parse validates nothing about it and the
    /// store reports a missing one as its honest `io` fault).
    pub root: OsString,
    /// The repository slug — §7's key.
    pub slug: RepoSlug,
    /// `repair` only: classify and report, write nothing.
    pub dry_run: bool,
    /// `repair` only: truncate without prompting.
    pub yes: bool,
}

/// What one argument vector asked for: a subcommand run, or the two
/// zero-exit informational requests. A request is a value like everything
/// else this parse produces — the binary maps it to streams and the
/// exit-code table, and no branch of it panics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Run [`Invocation`]'s command.
    Run(Invocation),
    /// Print [`USAGE`] to stdout, exit 0.
    Help,
    /// Print the version line to stdout, exit 0.
    Version,
}

/// A usage failure: one message for stderr, which the caller pairs with
/// [`USAGE`] and the exit-code table's `2`. The message is a diagnostic,
/// not a contract — but every case of the surface's behaviour table gets
/// its own wording here, because a usage error the operator cannot act on
/// is a bad report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgsError {
    message: String,
}

impl ArgsError {
    /// The operator-facing message, without the usage text the caller
    /// appends.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ArgsError {}

impl From<lexopt::Error> for ArgsError {
    fn from(error: lexopt::Error) -> Self {
        usage(format_args!("{error}"))
    }
}

/// A usage error from a format — the one construction site, so the
/// `todo-cli: ` prefix and the total (non-panicking) shape are stated
/// once.
fn usage(format: std::fmt::Arguments<'_>) -> ArgsError {
    ArgsError {
        message: format!("todo-cli: {format}"),
    }
}

/// Parses one argument vector — program name excluded — into the request
/// it describes.
///
/// Options may appear in any order, so `--root` before the subcommand
/// parses too. The per-subcommand switch table runs when a switch is
/// seen, which is what rejects `--dry-run`/`--yes` on `dump` and `verify`
/// (before the subcommand too — a switch with no subcommand belongs to no
/// table entry). `--help`/`--version` short-circuit where they appear,
/// clap's convention, so `dump --help` never demands the required
/// options; `--version` is the top level's alone (the surface extends
/// only `--help` to the subcommands). Required options are checked after
/// the loop, in the surface's order: subcommand, `--root`, `--slug`.
pub fn parse<I>(args: I) -> Result<Request, ArgsError>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut command: Option<Command> = None;
    let mut root: Option<OsString> = None;
    let mut slug_text: Option<OsString> = None;
    let mut dry_run = false;
    let mut yes = false;

    while let Some(arg) = parser.next().map_err(ArgsError::from)? {
        match arg {
            lexopt::Arg::Long("help") | lexopt::Arg::Short('h') => {
                reject_attached_value(&mut parser, "help")?;
                return Ok(Request::Help);
            }
            lexopt::Arg::Long("version") | lexopt::Arg::Short('V') => {
                reject_attached_value(&mut parser, "version")?;
                if let Some(command) = command {
                    return Err(usage(format_args!(
                        "unexpected argument '--version' for '{}': --version is the top level's own",
                        command.name()
                    )));
                }
                return Ok(Request::Version);
            }
            lexopt::Arg::Long("root") => {
                root = Some(parser.value().map_err(ArgsError::from)?);
            }
            lexopt::Arg::Long("slug") => {
                slug_text = Some(parser.value().map_err(ArgsError::from)?);
            }
            lexopt::Arg::Long(flag @ ("dry-run" | "yes")) => {
                // The per-subcommand switch table (U14 accepted its cost):
                // lexopt has no notion of a subcommand, so the table is
                // this check — a switch on the wrong subcommand is an
                // error, never a silent ignore.
                if command != Some(Command::Repair) {
                    let scope = match command {
                        Some(command) => format!(" for '{}'", command.name()),
                        None => String::new(),
                    };
                    return Err(usage(format_args!(
                        "unexpected argument '--{flag}'{scope}: \
                         --dry-run and --yes belong to 'repair' only"
                    )));
                }
                if flag == "dry-run" {
                    dry_run = true;
                } else {
                    yes = true;
                }
            }
            lexopt::Arg::Value(value) if command.is_none() => {
                let text = value.to_string_lossy();
                command = Some(match text.as_ref() {
                    "dump" => Command::Dump,
                    "verify" => Command::Verify,
                    "repair" => Command::Repair,
                    other => {
                        return Err(usage(format_args!(
                            "unrecognized subcommand '{other}': \
                             expected 'dump', 'verify' or 'repair'"
                        )));
                    }
                });
            }
            // A value with the subcommand already set: a second
            // subcommand or a stray positional — lexopt's own report.
            unexpected => return Err(ArgsError::from(unexpected.unexpected())),
        }
    }

    let Some(command) = command else {
        return Err(usage(format_args!(
            "a subcommand is required: 'dump', 'verify' or 'repair' (--help for the surface)"
        )));
    };
    let Some(root) = root else {
        return Err(usage(format_args!(
            "the required option '--root <DIR>' was not provided"
        )));
    };
    let Some(slug_text) = slug_text else {
        return Err(usage(format_args!(
            "the required option '--slug <SLUG>' was not provided"
        )));
    };
    // §7: the slug is the key — and a key that is not one is a usage
    // fault, never the store's (the store is not consulted).
    let slug_string = slug_text.into_string().map_err(|_| {
        usage(format_args!(
            "invalid repository slug: the value of '--slug' is not valid UTF-8"
        ))
    })?;
    let slug = RepoSlug::from_str(&slug_string).map_err(|error| usage(format_args!("{error}")))?;

    Ok(Request::Run(Invocation {
        command,
        root,
        slug,
        dry_run,
        yes,
    }))
}

/// Rejects an attached value on a flag that takes none (`--help=yes`,
/// `-hx`): lexopt hands it over instead of erroring, and pedantic parsing
/// — U14's reason for choosing lexopt — means refusing it rather than
/// dropping it on the floor.
fn reject_attached_value(parser: &mut lexopt::Parser, name: &str) -> Result<(), ArgsError> {
    match parser.optional_value() {
        None => Ok(()),
        Some(value) => {
            let text = value.to_string_lossy();
            Err(usage(format_args!(
                "the flag '--{name}' takes no value, got '{text}'"
            )))
        }
    }
}
