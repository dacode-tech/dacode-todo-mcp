//! The inspector suites (t72, D20): the absorbed todo-cli suites —
//! dump, verify, repair — under the host spelling (`--repo`, the
//! subcommand first), the parse suite's binary half (the exit codes
//! and streams — the half that must spawn the process, so it lives
//! here where `CARGO_BIN_EXE_todo-mcp` is set rather than in the
//! lib's in-file parse tests, where cargo does not set it), and the
//! cwd-tier and resolution-failure pins §13's signed sentence demands
//! (the repo resolves through the same tier as the server, repair
//! included). The fixtures are the absorbed ones — real-API builders,
//! byte-spliced tears, the runner, the debris scan — with the golden
//! world's fixed literal repo path; the cwd-tier pin builds its own
//! real-path fixture (the two concerns do not mix).
//!
//! The suite is split by file the way `tests/read.rs` is: no `mod.rs`
//! in the directory (P-LIVE-004). Every test lists as
//! `inspect::<file>::…` so the module-path filter
//! `cargo test -p todo-mcp --test inspect -- inspect::dump` selects
//! exactly one file's suite (P-LIVE-002/P-LIVE-005: quote the counts
//! beside the filter).

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

// `common/` is shared by the read, write, import and wire targets;
// this target uses the golden helper alone, and the fixture and drive
// halves are dead only from this target's compilation — those targets
// keep them live. The allow is scoped to the module so this target's
// own helpers stay under the lint.
#[allow(dead_code)]
mod common;

mod inspect {
    mod dump;
    mod fixtures;
    mod repair;
    mod verify;

    /// The parse suite's binary half: the exit codes and streams the
    /// lib's in-file parse tests cannot pin — cargo sets
    /// `CARGO_BIN_EXE_<name>` for integration targets only (measured:
    /// unset in the lib's unit tests), so the two absorbed
    /// process-spawning args tests live here.
    mod args {
        use todo_mcp::args::USAGE;

        use crate::inspect::fixtures::{CliRun, run_cli};

        /// Asserts the exit-code table's 2: the message, the usage text
        /// and the `todo-mcp:` prefix on stderr, and nothing on stdout.
        fn assert_usage_fault(run: CliRun, needle: &str) {
            assert_eq!(run.code, 2, "stderr: {}", run.stderr);
            assert!(run.stderr.contains("todo-mcp:"), "{:?}", run.stderr);
            assert!(run.stderr.contains(needle), "{:?}", run.stderr);
            assert!(run.stderr.contains("Usage:"), "{:?}", run.stderr);
            assert!(run.stderr.contains(USAGE.trim_end()), "the whole usage");
            assert_eq!(run.stdout, "", "a usage fault writes nothing to stdout");
        }

        /// The absorbed behaviour table under the host wordings (D7):
        /// the repair switches on the wrong subcommand, a missing
        /// `--root`, an unknown flag, an unknown subcommand, a stray
        /// positional, a malformed `--repo` value — every case a usage
        /// fault decided before any store call (`--root` points at a
        /// directory no case may reach).
        #[test]
        fn every_failure_case_exits_2_with_the_usage_text_on_stderr() {
            assert_usage_fault(
                run_cli(&[
                    "dump",
                    "--root",
                    "/nonexistent-t72",
                    "--repo",
                    "camera",
                    "--yes",
                ]),
                "the option '--yes' belongs to the 'repair' subcommand",
            );
            assert_usage_fault(
                run_cli(&[
                    "verify",
                    "--root",
                    "/nonexistent-t72",
                    "--repo",
                    "camera",
                    "--dry-run",
                ]),
                "the option '--dry-run' belongs to the 'repair' subcommand",
            );
            assert_usage_fault(run_cli(&["dump"]), "--root");
            assert_usage_fault(
                run_cli(&[
                    "dump",
                    "--root",
                    "/nonexistent-t72",
                    "--repo",
                    "camera",
                    "--bogus",
                ]),
                "--bogus",
            );
            assert_usage_fault(
                run_cli(&["nosuch", "--root", "/nonexistent-t72", "--repo", "camera"]),
                "unknown subcommand 'nosuch'",
            );
            assert_usage_fault(
                run_cli(&[
                    "dump",
                    "--root",
                    "/nonexistent-t72",
                    "--repo",
                    "camera",
                    "extra",
                ]),
                "extra",
            );
            assert_usage_fault(
                run_cli(&["dump", "--root", "/nonexistent-t72", "--repo", "Bad Slug"]),
                "invalid repository slug",
            );
        }

        /// `--help`/`-h` exit 0 with the usage on stdout — bare and per
        /// subcommand, never demanding the required options — and
        /// `--version` exits 0 with the host's identity line.
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
                format!("todo-mcp {}\n", env!("CARGO_PKG_VERSION"))
            );
            assert_eq!(run.stderr, "");
        }
    }
}
