//! The deterministic core of the task graph (spec §2): nodes, typed
//! edges, graph invariants and the selection rules over them.
//!
//! Boundary: no I/O, no async, no workspace dependencies. Persistence
//! lives in `todo-store`, the wire in `todo-mcp`; this crate is pure
//! computation over in-memory graph state, and a `use` crossing that
//! line is a defect.
//!
//! Determinism lives here and nowhere else (spec §8): canonical ordering
//! and `next_task`/`next_items` selection are pure functions of graph
//! state — topology first, then epic registration order, then task id
//! (spec §5.2). Same store bytes in, same answer out, on every run and
//! every platform; no wall clock, no randomness, no hash-iteration
//! order.
//!
//! The domain model lands at M1 (spec §14); an empty, documented crate
//! is the correct M0 state.

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
