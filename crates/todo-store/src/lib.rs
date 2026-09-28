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
//! Layout (D4, per-repo stores): under an **injected** data root — D4's
//! default location is `~/.local/share/dacode-todo/`, resolved by the
//! binary and passed in, never read from the environment, home or cwd
//! below it (§13/D8: a location is configuration, not a heuristic; §7:
//! cwd auto-detection is deferred to M5), and nothing below the binary
//! reads a clock either (§8) — [`DataRoot`] lays out `repos/<slug>/`
//! holding one repository's event log and snapshots, plus a shared
//! registry file holding the slug index. Lock scope follows the layout:
//! one advisory lock per repo store plus one for the registry, acquired
//! registry-first then in sorted-slug order (spec §8), so an operation
//! touching several stores can never deadlock.
//!
//! The engine needs no async runtime; this crate stays sync and
//! runtime-agnostic (spec §2).
//!
//! Current surface (M1): the canonical serializer over the domain's
//! serde schema — [`to_bytes`]/[`from_bytes`], with the JSON engine
//! (D1) sealed inside the crate — [`StoreError`], carrying §6.1's
//! `corrupt_store`, `version_mismatch` and now `locked` ([`LockTarget`]
//! names the contended store) with the byte offset a repair needs (spec
//! §4.3), [`FORMAT_VERSION`], the single source of §4's versioning
//! requirement, and the persistence surface itself: [`Store`], the one
//! trait of §4.2 — its constructor half ([`Store::create`],
//! [`Store::open`]) takes a [`DataRoot`] and a repo or slug, not a bare
//! path — with [`JsonlStore`] as the D1 implementation over [`Event`],
//! the persisted schema. The D4 layout ([`DataRoot`]), the shared
//! registry ([`Registry`], [`RegistryEntry`]) and §8's advisory locks
//! ([`RegistryLock`], and the per-store lock every [`JsonlStore`] holds)
//! are landed. The serialized bytes are pinned by the golden files under
//! `tests/golden/` (spec §10,
//! `.agents/knowledge/workflows/golden-files.md`): `event_log`,
//! `serialize_nodes` and `registry`. Snapshots and log compaction (t12)
//! grow the trait in-crate; the cli inspector and §4.3's repair (t13)
//! read the store back. Through M1 the constructors return the concrete
//! [`JsonlStore`]; M3's server gets a boxed opener (`Box<dyn Store>`) if
//! it would otherwise have to name the engine — the `dyn Store`
//! object-safety pin keeps that available, and it is deliberately not
//! built now.

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
mod layout;
mod lock;
mod log;
mod registry;
mod store;

pub use codec::{from_bytes, to_bytes};
pub use error::{LockTarget, StoreError};
pub use event::{
    BlocksEdgeDto, DecisionPatch, DocPatch, EpicPatch, Event, ReplayFault, TaskPatch,
    apply_decision_patch, apply_doc_patch, apply_epic_patch, apply_task_patch, replay_step,
};
pub use layout::DataRoot;
pub use lock::RegistryLock;
pub use log::JsonlStore;
pub use registry::{Registry, RegistryEntry};
pub use store::Store;

/// The store format version — spec §4: "a format version in the header;
/// a format change is a migration, never an in-place edit of meaning".
///
/// The single source every versioned surface references: golden file
/// names embed it (`<subject>.v<N>.golden`), the event-log header carries
/// it ([`JsonlStore::create_log`] writes it as the log's first line and
/// [`JsonlStore::open_log`] checks it before replaying anything), the
/// registry document carries it ([`Registry::load`] checks it), the
/// snapshot header (t12) will carry it, and stored bytes disagreeing with
/// it are §6.1's `version_mismatch`. A format change bumps this constant
/// and lands a migration; during one, the old and new goldens coexist
/// (`.agents/knowledge/workflows/golden-files.md`).
pub const FORMAT_VERSION: u32 = 1;
