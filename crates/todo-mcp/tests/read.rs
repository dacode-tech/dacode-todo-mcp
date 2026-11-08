//! The read suites (spec §10): every read tool driven through the real
//! binary over raw stdio, its responses golden-pinned per tool — the
//! first per-tool wire goldens (t34), the discipline D14's
//! declaration-order pin and D15's carrier are proven from.
//!
//! The suite is split by cluster the way the store's `tests/layout.rs`
//! is: `read/query.rs` holds the query cluster's tests, and later
//! clusters (t35's listings) add sibling files under `read/` — no
//! `mod.rs` in the directory (P-LIVE-004). Every test lists as
//! `read::query::…` so the module-path filter
//! `cargo test -p todo-mcp -- read::query` selects exactly this suite
//! (P-LIVE-002/P-LIVE-005: quote the counts beside the filter).

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

mod read {
    mod listing;
    mod query;
}
