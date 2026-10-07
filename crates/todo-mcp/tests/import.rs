//! The import suites (t42's parser, t43's mapping, t44's wire tool,
//! t45's fixture, t46's camera dialect, t47's ikvm-html dialect): the
//! dacode dialect's behavior over hand-authored synthetic corpora,
//! plus the real board itself — t45 checks in the three sibling files
//! as the fixture (the only sibling content this repository holds,
//! D12/§10) and pins the graph they import to — and the camera and
//! ikvm-html dialects over their own hand-authored and loop-generated
//! corpora (never the live boards' prose, D12).
//!
//! The suite is split by file the way `tests/read.rs` is:
//! `import/board.rs` holds the TODO.md walker's tests plus the
//! composition, determinism and order suites, `import/decisions.rs`
//! the B appendix's, `import/deltas.rs` the A appendix's,
//! `import/map.rs` the mapping's (the D18 conventions over
//! directly-built ASTs, plus the parse→map→apply pipeline),
//! `import/wire.rs` the `import_md` tool's (the importer through the
//! protocol, over a board tree derived from the parsed corpus
//! register), `import/e2e.rs` the real board end to end (the
//! fixture's dump golden, its identity pin, the wire composition),
//! `import/camera.rs` the camera dialect's (the parser, the
//! mapping, the `dialect` param's wire drive, the id reproduction at
//! synthetic scale, determinism), and `import/ikvm.rs` the ikvm-html
//! dialect's (the parser, the four cross-checks, the mapping, the
//! plan census and id reproduction at synthetic live scale, the wire
//! drive, determinism) — no `mod.rs` in the directory (P-LIVE-004).
//! Every test lists as `import::<file>::…` so the module-path filter
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

mod common;

mod import {
    mod board;
    mod camera;
    mod decisions;
    mod deltas;
    mod e2e;
    mod ikvm;
    mod map;
    mod wire;
}
