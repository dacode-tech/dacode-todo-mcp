//! The snapshot and compaction tests: §4.1's "periodic snapshot +
//! truncation bounds replay" and §4.3's "snapshots are the recovery unit
//! … replay reads the latest snapshot, then the log tail after it"
//! (spec §4.1, §4.3, §8, §10;
//! `.agents/knowledge/workflows/golden-files.md`).
//!
//! Every store here lives under a `tempfile` directory: never a fixed
//! path, never the cwd. Timestamps are caller-passed constants, so
//! nothing reads a clock (§8) and no absolute host path reaches a golden
//! or an error detail. The fixtures continue the crate's `camera` world,
//! so every golden subject describes one world.
//!
//! Every test sits under the `snapshot` module and the verify line is
//! `cargo test -p todo-store --test snapshot`, which selects this target
//! exactly: no substring semantics, so it can neither pass vacuously nor
//! silently widen (P-LIVE-002, P-LIVE-005). The suite is split by
//! responsibility: `snapshot/fixtures.rs` holds the builders and the
//! document mirror the fault fixtures splice through,
//! `snapshot/roundtrip.rs` the replay equality, the burned-number
//! restore, the golden and the determinism instruments,
//! `snapshot/compact.rs` the truncation contract, and
//! `snapshot/corrupt.rs` the named-error surface.

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

/// The suite, named so that every test below lists as `snapshot::…` and
/// the target the verify line selects holds nothing else (P-LIVE-002).
/// The wrapper is inline with the submodules as `tests/snapshot/<file>.rs`
/// and no `mod.rs` in the directory (P-LIVE-004).
mod snapshot {
    /// t10's scripted timeline and byte helpers, included **read-only**
    /// by path so "reuse" means the same file, not a copy that can drift
    /// — it pins `event_log.v1.golden` and must not change. The `allow`
    /// is for the helpers this suite does not call: the file compiles in
    /// full or not at all.
    #[allow(
        dead_code,
        reason = "t10's fixture file compiled whole; this suite calls a subset"
    )]
    #[path = "../log/fixtures.rs"]
    mod timeline;

    mod compact;
    mod corrupt;
    mod fixtures;
    mod roundtrip;
}
