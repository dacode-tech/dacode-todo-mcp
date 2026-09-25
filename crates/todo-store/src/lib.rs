//! Persistence for the task graph, behind a single `Store` trait
//! (spec §2).
//!
//! The storage engine — an append-only JSONL event log plus snapshots
//! (D1) — never leaks above the trait: no engine type, no engine error,
//! no engine feature flag in `todo-domain` or `todo-mcp`. Corruption is
//! a named error, never a panic; the on-disk format is pinned by golden
//! files, and a format change is a migration with a version header
//! (spec §4, §10).
//!
//! Layout (D4, per-repo stores): under `~/.local/share/dacode-todo/`,
//! `repos/<slug>/` holds one repository's event log and snapshots, and a
//! shared registry file holds repo metadata and the slug→store mapping.
//! Lock scope follows the layout: one advisory lock per repo store plus
//! one for the registry, acquired in sorted-slug order (spec §8).
//!
//! The engine needs no async runtime; this crate stays sync and
//! runtime-agnostic (spec §2).
//!
//! The `Store` trait and the event log land at M1 (spec §14); an empty,
//! documented crate is the correct M0 state.

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
