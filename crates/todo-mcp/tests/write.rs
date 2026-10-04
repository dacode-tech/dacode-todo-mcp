//! The write suites (spec §10): every mutation tool driven through the
//! real binary over raw stdio — the read suites' pattern with the one
//! difference that shapes everything here: a write mutates the store,
//! so every test drives a **fresh fixture** (the read suites' per-test
//! isolation, load-bearing now — a shared fixture would make each
//! test's ids depend on every test before it). The stamped writer
//! families end by reopening the store in-process once the server has
//! exited, asserting the mutation is on disk (the t33
//! `repo_register` precedent: the wire proved the write, the reopen
//! proves the log); the clock-free families are golden-pinned instead
//! — the split each suite's module doc states.
//!
//! The suite is split by cluster the way `tests/read.rs` is:
//! `write/nodes.rs` holds the node-CRUD cluster's tests,
//! `write/edges.rs` the edge cluster's, and later clusters add
//! sibling files under `write/` — no `mod.rs` in the directory
//! (P-LIVE-004). Every test lists as `write::nodes::…` /
//! `write::edges::…` so the module-path filter
//! `cargo test -p todo-mcp --test write -- write::edges` selects
//! exactly one suite (P-LIVE-002/P-LIVE-005: quote the counts beside
//! the filter — the target holds two suites, so the sibling's count
//! is the filtered-out one).

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

use std::str::FromStr;

mod write {
    mod edges;
    mod nodes;
}

// The write target's own helper, shared by both suites and compiled
// by no other target: `reopened` is write-suite-only, so it cannot
// live in `common/` (the read target compiles that module and would
// see it dead). The write target's root is its one home.
/// Reopens the fixture's store in-process after the drive — the server
/// has exited, its locks released — for the persistence assertions.
pub(crate) fn reopened(root: &std::path::Path) -> todo_store::JsonlStore {
    todo_store::DataRoot::new(root)
        .open(&todo_domain::RepoSlug::from_str("fixture").unwrap())
        .unwrap()
}
