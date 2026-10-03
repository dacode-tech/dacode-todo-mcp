//! The write suites (spec §10): every mutation tool driven through the
//! real binary over raw stdio — the read suites' pattern with the one
//! difference that shapes everything here: a write mutates the store,
//! so every test drives a **fresh fixture** (the read suites' per-test
//! isolation, load-bearing now — a shared fixture would make each
//! test's ids depend on every test before it). The stamped writer
//! families end by reopening the store in-process once the server has
//! exited, asserting the mutation is on disk (the t33
//! `repo_register` precedent: the wire proved the write, the reopen
//! proves the log); the clock-free epic pair is golden-pinned instead
//! — the split each suite's module doc states.
//!
//! The suite is split by cluster the way `tests/read.rs` is:
//! `write/nodes.rs` holds the node-CRUD cluster's tests, and later
//! clusters (t37's edges and registry writes) add sibling files under
//! `write/` — no `mod.rs` in the directory (P-LIVE-004). Every test
//! lists as `write::nodes::…` so the module-path filter
//! `cargo test -p todo-mcp --test write -- write::nodes` selects
//! exactly this target (P-LIVE-002/P-LIVE-005: quote the counts beside
//! the filter — the target holds one suite, so 0 filtered out).

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

mod write {
    mod nodes;
}
