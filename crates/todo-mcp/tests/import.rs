//! The import suites (t42's parser, t43's mapping): the dacode
//! dialect's behavior over hand-authored synthetic corpora — the
//! live board is t45's checked-in fixture and its prose is never
//! copied into this repo.
//!
//! The suite is split by file the way `tests/read.rs` is:
//! `import/board.rs` holds the TODO.md walker's tests plus the
//! composition, determinism and order suites, `import/decisions.rs`
//! the B appendix's, `import/deltas.rs` the A appendix's, and
//! `import/map.rs` the mapping's (the D18 conventions over
//! directly-built ASTs, plus the parse→map→apply pipeline) — no
//! `mod.rs` in the directory (P-LIVE-004). Every test lists as
//! `import::<file>::…` so the module-path filter
//! `cargo test -p todo-mcp --test import -- import::board` selects
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

mod import {
    mod board;
    mod decisions;
    mod deltas;
    mod map;
}
