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
//! Current surface (M1): the canonical serializer over the domain's
//! serde schema — [`to_bytes`]/[`from_bytes`], with the JSON engine
//! (D1) sealed inside the crate — [`StoreError`], carrying §6.1's
//! `corrupt_store` and `version_mismatch` with the byte offset a repair
//! needs (spec §4.3), [`FORMAT_VERSION`], the single source of §4's
//! versioning requirement, and the persistence surface itself:
//! [`Store`], the one trait of §4.2, with [`JsonlStore`] — the
//! append-only event log with its versioned header, its replay into the
//! in-memory graph and §4.3's single-write append discipline — as the
//! D1 implementation, over [`Event`], the persisted schema. The
//! serialized bytes are pinned by the golden files under
//! `tests/golden/` (spec §10,
//! `.agents/knowledge/workflows/golden-files.md`). Snapshots and log
//! compaction (t12) grow the trait in-crate; the D4 layout, the
//! registry and §8's advisory locks (t11) wrap [`JsonlStore`]'s
//! constructors; §6.1's `locked` arrives with them.

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

mod codec;
mod error;
mod event;
mod log;
mod store;

pub use codec::{from_bytes, to_bytes};
pub use error::StoreError;
pub use event::{
    BlocksEdgeDto, DecisionPatch, DocPatch, EpicPatch, Event, ReplayFault, TaskPatch,
    apply_decision_patch, apply_doc_patch, apply_epic_patch, apply_task_patch, replay_step,
};
pub use log::JsonlStore;
pub use store::Store;

/// The store format version — spec §4: "a format version in the header;
/// a format change is a migration, never an in-place edit of meaning".
///
/// The single source every versioned surface references: golden file
/// names embed it (`<subject>.v<N>.golden`), the event-log header carries
/// it ([`JsonlStore::create`] writes it as the log's first line and
/// [`JsonlStore::open`] checks it before replaying anything), the snapshot
/// header (t12) will carry it, and stored bytes disagreeing with it are
/// §6.1's `version_mismatch`. A format change bumps this constant and
/// lands a migration; during one, the old and new goldens coexist
/// (`.agents/knowledge/workflows/golden-files.md`).
pub const FORMAT_VERSION: u32 = 1;
