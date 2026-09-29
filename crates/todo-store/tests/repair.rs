//! §4.3's repair primitive, exercised through the D4 layout: a torn last
//! line is shown by a dry run and removed by a real one, everything a
//! truncation cannot fix is refused as an outcome rather than an error, a
//! contended store is §6.1's `locked`, and a diverged snapshot/log pair is
//! reported without being written (spec §4.3, §6.1, §8;
//! `.agents/knowledge/workflows/golden-files.md` for the fixture
//! discipline this suite follows).
//!
//! Every store here lives under a `tempfile` root: never a fixed path,
//! never the cwd. Timestamps are caller-passed constants, so nothing reads
//! a clock (§8) and no absolute host path reaches a report. The fixtures
//! continue the crate's `camera` world — t10's builders are included by
//! path, read-only — so every subject describes one world.
//!
//! Torn bytes are always spliced **from real serialized lines** (a real
//! event line with its LF and its last few bytes dropped), never
//! hand-written: f8's rule, the log suite's. No golden is involved, so
//! this target does not pull in `tests/common`.
//!
//! Every test sits under the `repair` module, so both
//! `cargo test -p todo-store --test repair` and the name filter
//! `-- repair` select all of it, and neither can pass vacuously
//! (P-LIVE-002, P-LIVE-005). The suite is split by responsibility:
//! `repair/fixtures.rs` holds the builders and the byte-splicing helpers,
//! `repair/torn.rs` the repairable path (§4.3's torn tail) and
//! `repair/refusals.rs` the four refusals and the two io faults.

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

/// The suite, named so that every test below lists as `repair::…` and the
/// target the verify line selects holds nothing else (P-LIVE-002). The
/// wrapper is inline with the submodules as `tests/repair/<file>.rs` and
/// no `mod.rs` in the directory (P-LIVE-004).
mod repair {
    /// t10's builders and byte helpers, included **read-only** by path so
    /// "reuse" means the same file, not a copy that can drift — it pins
    /// `event_log.v1.golden` and must not change (the snapshot suite's
    /// precedent). The `allow` is for the helpers this suite does not
    /// call: the file compiles in full or not at all.
    #[allow(
        dead_code,
        reason = "t10's fixture file compiled whole; this suite calls a subset"
    )]
    #[path = "../log/fixtures.rs"]
    mod timeline;

    mod fixtures;
    mod refusals;
    mod torn;
}
