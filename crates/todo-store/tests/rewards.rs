//! §15's storage half (D23, U61): the per-root config document, the
//! closure grant path, the reflection log, and the derived totals —
//! the rewards surface the store owns, exercised end to end.
//!
//! Every root here lives under a `tempfile` directory: never a fixed
//! path, never the cwd. Timestamps are caller-passed constants, so
//! nothing reads a clock (§8) and no absolute host path reaches a
//! golden or an error detail. The fixtures continue the crate's
//! `camera` world — t10's builders are included by path, read-only —
//! so every subject describes one world.
//!
//! Every test sits under the `rewards` module and the verify line is
//! `cargo test -p todo-store --test rewards`, which selects this target
//! exactly: no substring semantics, so it can neither pass vacuously
//! nor silently widen (P-LIVE-002, P-LIVE-005). The suite is split by
//! responsibility: `rewards/fixtures.rs` holds the builders and the
//! config variants, `rewards/config.rs` the config document — its
//! golden, its faults, the inert root — `rewards/grant.rs` the
//! closure-aware update's semantics, `rewards/reflect.rs` the
//! reflection event and its round-trip, `rewards/totals.rs` the
//! derived per-root totals, and `rewards/events.rs` the reward events
//! read (§15's display half, U63).

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

/// The suite, named so that every test below lists as `rewards::…` and
/// the target the verify line selects holds nothing else (P-LIVE-002).
/// The wrapper is inline with the submodules as
/// `tests/rewards/<file>.rs` and no `mod.rs` in the directory
/// (P-LIVE-004).
mod rewards {
    /// t10's builders and byte helpers, included **read-only** by path
    /// so "reuse" means the same file, not a copy that can drift (the
    /// snapshot suite's precedent). The `allow` is for the helpers this
    /// suite does not call: the file compiles in full or not at all.
    #[allow(
        dead_code,
        reason = "t10's fixture file compiled whole; this suite calls a subset"
    )]
    #[path = "../log/fixtures.rs"]
    mod timeline;

    mod config;
    mod events;
    mod fixtures;
    mod grant;
    mod reflect;
    mod totals;
}
