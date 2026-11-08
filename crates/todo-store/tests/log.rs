//! The event log's tests: replay, corruption, determinism, and the second
//! golden (spec §4.1, §4.3, §8, §10;
//! `.agents/knowledge/workflows/golden-files.md`).
//!
//! Every log here lives under a `tempfile` directory: never a fixed path,
//! never the cwd. Timestamps are caller-passed constants, so nothing reads
//! a clock (§8) and no absolute host path reaches a golden or an error
//! detail. The fixtures continue the crate's `sample_*` set — the same
//! camera repo — so both golden subjects describe one world.
//!
//! Every test sits under the `log` module, so the tracker's verify line —
//! `cargo test -p todo-store -- log` — selects all of them by name
//! (P-LIVE-002). The suite is split by responsibility: `log/fixtures.rs`
//! holds the builders, the scripted timeline and the byte-splicing
//! helpers, `log/replay.rs` the replay, golden, determinism and
//! write-contract tests, and `log/corrupt.rs` §4.3's corruption tests.

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

mod common;

/// The suite, named so that every test below is `log::…` and the verify
/// line's name filter cannot pass vacuously (P-LIVE-002).
mod log {
    mod corrupt;
    mod fixtures;
    mod replay;
}
