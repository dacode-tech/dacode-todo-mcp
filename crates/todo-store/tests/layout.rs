//! The D4 layout's tests: path derivation, the registry document and its
//! golden, and §8's advisory locking (spec §2, §4.3, §7, §8, §10;
//! `.agents/knowledge/workflows/golden-files.md`).
//!
//! Every store here lives under a `tempfile` root: never a fixed path,
//! never the cwd. Timestamps are caller-passed constants, so nothing
//! reads a clock (§8) and no absolute host path reaches a golden or an
//! error detail. The fixtures continue the crate's `sample_*` world — the
//! camera repo among them — so every golden subject describes one world.
//!
//! Every test sits under the `layout` module and the verify line is
//! `cargo test -p todo-store --test layout`, which selects this target
//! exactly: no substring semantics, so it can neither pass vacuously nor
//! silently widen when a test elsewhere happens to carry one of its
//! tokens (P-LIVE-002, P-LIVE-005). A name filter such as `-- layout` is
//! a convenience, not the gate — it also reaches the `layout::tests` unit
//! tests inside `src/layout.rs`. The suite is split by responsibility:
//! `layout/fixtures.rs` holds the builders, `layout/paths.rs` the D4
//! derivation and the slug's path-safety, `layout/lock.rs` §8's
//! single-writer behavior, and `layout/registry.rs` the registry
//! document — its golden, its faults and its byte stability.

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

/// The suite, named so that every test below lists as `layout::…` and the
/// target the verify line selects holds nothing else (P-LIVE-002). The
/// wrapper is inline with the submodules as `tests/layout/<file>.rs` and
/// no `mod.rs` in the directory (P-LIVE-004).
mod layout {
    mod fixtures;
    mod lock;
    mod paths;
    mod registry;
}
