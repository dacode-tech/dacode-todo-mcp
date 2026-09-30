//! The U14 probe's behaviour table, minus `--pretty` (dropped with the
//! lexopt resolution): three happy paths, `--dry-run`/`--yes` rejected on
//! `dump` and `verify`, missing `--root`, missing `--slug`, unknown flag,
//! unknown subcommand, no arguments, stray positional, a malformed slug;
//! `--help`, `--version` and subcommand `--help` exit 0 with usage on
//! stdout. Every failure case asserts exit **2** and that stderr carries
//! the usage text.
//!
//! The parse itself is tested through the lib target — [`parse`] takes an
//! argument vector and returns values, so the behaviour table needs no
//! process; the exit-code and stream assertions are the binary's half and
//! spawn it via `env!("CARGO_BIN_EXE_todo-cli")`.

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

/// The suite, named so that every test below lists as `args::…` and the
/// target the verify line selects holds nothing else (P-LIVE-002, inline
/// wrapper per P-LIVE-004).
mod args {
    use std::ffi::OsString;

    use todo_cli::args::{Command, Invocation, Request, USAGE, parse};
    use todo_domain::RepoSlug;

    use crate::fixtures::run_cli;

    fn slug() -> RepoSlug {
        "camera".parse().unwrap()
    }

    fn invocation(command: Command, dry_run: bool, yes: bool) -> Invocation {
        Invocation {
            command,
            root: OsString::from("/data"),
            slug: slug(),
            dry_run,
            yes,
        }
    }

    // =================================================================
    // The three happy paths — lib level: the parse is total and typed.
    // =================================================================

    #[test]
    fn the_three_happy_paths_parse_into_typed_invocations() {
        assert_eq!(
            parse(["dump", "--root", "/data", "--slug", "camera"]),
            Ok(Request::Run(invocation(Command::Dump, false, false)))
        );
        assert_eq!(
            parse(["verify", "--root", "/data", "--slug", "camera"]),
            Ok(Request::Run(invocation(Command::Verify, false, false)))
        );
        assert_eq!(
            parse([
                "repair",
                "--root",
                "/data",
                "--slug",
                "camera",
                "--dry-run",
                "--yes"
            ]),
            Ok(Request::Run(invocation(Command::Repair, true, true)))
        );
    }

    #[test]
    fn the_gnu_conventions_lexopt_owns_parse_too() {
        // `--flag=value` and options ahead of the subcommand: the
        // conventions U14 chose a parser for rather than hand-rolling.
        assert_eq!(
            parse(["dump", "--root=/data", "--slug=camera"]),
            Ok(Request::Run(invocation(Command::Dump, false, false)))
        );
        assert_eq!(
            parse(["--root", "/data", "--slug", "camera", "verify"]),
            Ok(Request::Run(invocation(Command::Verify, false, false)))
        );
    }

    // =================================================================
    // The rejection table — lib level: every failure is a value with an
    // operator-actionable message.
    // =================================================================

    #[test]
    fn the_repair_switches_are_rejected_on_the_other_subcommands() {
        // The per-subcommand switch table U14 accepted the cost of: a
        // switch on the wrong subcommand is an error, never a silent
        // ignore — including before any subcommand, where it belongs to
        // no table entry at all.
        for args in [
            ["dump", "--root", "/data", "--slug", "camera", "--yes"],
            ["verify", "--root", "/data", "--slug", "camera", "--dry-run"],
            ["--dry-run", "dump", "--root", "/data", "--slug", "camera"],
        ] {
            let error = parse(args).unwrap_err();
            assert!(
                error.message().contains("belong to 'repair' only"),
                "{error}"
            );
        }
    }

    #[test]
    fn missing_required_options_are_named() {
        let error = parse(["dump", "--slug", "camera"]).unwrap_err();
        assert!(error.message().contains("--root"), "{error}");
        let error = parse(["verify", "--root", "/data"]).unwrap_err();
        assert!(error.message().contains("--slug"), "{error}");
    }

    #[test]
    fn unknown_flags_subcommands_and_stray_positionals_are_rejected() {
        let error = parse(["dump", "--root", "/data", "--slug", "camera", "--bogus"]).unwrap_err();
        assert!(error.message().contains("--bogus"), "{error}");

        let error = parse(["nosuch", "--root", "/data", "--slug", "camera"]).unwrap_err();
        assert!(
            error.message().contains("unrecognized subcommand 'nosuch'"),
            "{error}"
        );

        let error = parse(Vec::<&str>::new()).unwrap_err();
        assert!(
            error.message().contains("a subcommand is required"),
            "{error}"
        );

        let error = parse(["dump", "--root", "/data", "--slug", "camera", "extra"]).unwrap_err();
        assert!(error.message().contains("extra"), "{error}");
    }

    #[test]
    fn a_malformed_slug_is_a_usage_fault() {
        // §7: the slug is the key — and a key that is not one never
        // reaches the store, so it can never be mistaken for one.
        for bad in ["Bad Slug", "-x", "", "camera-"] {
            let error = parse(["dump", "--root", "/data", "--slug", bad]).unwrap_err();
            assert!(
                error.message().contains("invalid repository slug"),
                "{error}"
            );
        }
    }

    #[test]
    fn help_and_version_are_requests_not_errors() {
        assert_eq!(parse(["--help"]), Ok(Request::Help));
        assert_eq!(parse(["-h"]), Ok(Request::Help));
        assert_eq!(parse(["dump", "--help"]), Ok(Request::Help));
        assert_eq!(
            parse(["repair", "--root", "/data", "-h"]),
            Ok(Request::Help)
        );
        assert_eq!(parse(["--version"]), Ok(Request::Version));
        assert_eq!(parse(["-V"]), Ok(Request::Version));
        // The surface extends only --help to the subcommands.
        let error = parse(["dump", "--version"]).unwrap_err();
        assert!(error.message().contains("--version"), "{error}");
        // Pedantic: a flag that takes none refuses an attached value.
        let error = parse(["--help=yes"]).unwrap_err();
        assert!(error.message().contains("takes no value"), "{error}");
    }

    // =================================================================
    // The binary's half: exit codes and streams (required test 1's
    // "every failure case asserts exit 2 and stderr carries the usage
    // text"). `--root` points at a directory no case may reach — a usage
    // fault is decided before any store call.
    // =================================================================

    /// Asserts the exit-code table's 2: the code, the usage text and the
    /// `todo-cli:` message prefix on stderr, and nothing on stdout.
    fn assert_usage_fault(run: crate::fixtures::CliRun, needle: &str) {
        assert_eq!(run.code, 2, "stderr: {}", run.stderr);
        assert!(run.stderr.contains("todo-cli:"), "{:?}", run.stderr);
        assert!(run.stderr.contains(needle), "{:?}", run.stderr);
        assert!(run.stderr.contains("Usage:"), "{:?}", run.stderr);
        assert!(run.stderr.contains(USAGE.trim_end()), "the whole usage");
        assert_eq!(run.stdout, "", "a usage fault writes nothing to stdout");
    }

    #[test]
    fn every_failure_case_exits_2_with_the_usage_text_on_stderr() {
        assert_usage_fault(
            run_cli(&[
                "dump",
                "--root",
                "/nonexistent-t20",
                "--slug",
                "camera",
                "--yes",
            ]),
            "--yes",
        );
        assert_usage_fault(
            run_cli(&[
                "verify",
                "--root",
                "/nonexistent-t20",
                "--slug",
                "camera",
                "--dry-run",
            ]),
            "--dry-run",
        );
        assert_usage_fault(run_cli(&["dump", "--slug", "camera"]), "--root");
        assert_usage_fault(run_cli(&["verify", "--root", "/nonexistent-t20"]), "--slug");
        assert_usage_fault(
            run_cli(&[
                "dump",
                "--root",
                "/nonexistent-t20",
                "--slug",
                "camera",
                "--bogus",
            ]),
            "--bogus",
        );
        assert_usage_fault(
            run_cli(&["nosuch", "--root", "/nonexistent-t20", "--slug", "camera"]),
            "unrecognized subcommand",
        );
        assert_usage_fault(run_cli(&[]), "a subcommand is required");
        assert_usage_fault(
            run_cli(&[
                "dump",
                "--root",
                "/nonexistent-t20",
                "--slug",
                "camera",
                "extra",
            ]),
            "extra",
        );
        assert_usage_fault(
            run_cli(&["dump", "--root", "/nonexistent-t20", "--slug", "Bad Slug"]),
            "invalid repository slug",
        );
    }

    #[test]
    fn help_and_version_exit_0_with_their_text_on_stdout() {
        for args in [
            vec!["--help"],
            vec!["-h"],
            vec!["dump", "--help"],
            vec!["verify", "-h"],
            vec!["repair", "--help"],
        ] {
            let run = run_cli(&args);
            assert_eq!(run.code, 0, "stderr: {}", run.stderr);
            assert!(run.stdout.contains("Usage:"), "{:?}", run.stdout);
            assert_eq!(run.stderr, "");
        }
        let run = run_cli(&["--version"]);
        assert_eq!(run.code, 0);
        assert_eq!(
            run.stdout,
            format!("todo-cli {}\n", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(run.stderr, "");
    }
}

#[allow(
    dead_code,
    reason = "the shared fixture file compiles whole; this target uses a subset"
)]
mod fixtures;
