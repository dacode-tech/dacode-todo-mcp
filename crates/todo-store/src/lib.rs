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
//! Current surface (M1, plus §15's storage half): the canonical
//! serializer over the domain's serde schema — [`to_bytes`]/
//! [`from_bytes`], with the JSON engine (D1) sealed inside the crate —
//! [`StoreError`], carrying §6.1's `corrupt_store`, `version_mismatch`
//! and now `locked` ([`LockTarget`] names the contended store) with the
//! byte offset a repair needs (spec §4.3), [`FORMAT_VERSION`], the
//! single source of §4's versioning requirement, and the persistence
//! surface itself: [`Store`], the one trait of §4.2 — its constructor
//! half ([`Store::create`], [`Store::open`]) takes a [`DataRoot`] and a
//! repo or slug, not a bare path — with [`JsonlStore`] as the D1
//! implementation over [`Event`], the persisted schema. The D4 layout
//! ([`DataRoot`]), the shared registry ([`Registry`], [`RegistryEntry`])
//! and §8's advisory locks ([`RegistryLock`], and the per-store lock
//! every [`JsonlStore`] holds) are landed, as are §4.1/§4.3's snapshots
//! and log compaction ([`Store::snapshot`], [`Store::compact`] →
//! [`Compaction`]) — explicit calls with no policy and no clock behind
//! them; t13's cli and M3's server decide *when*. §15's storage half
//! (D23, U61) rides the same surface: the per-root config document
//! ([`DataRoot::load_config`]/[`DataRoot::save_config`] over
//! `<root>/config.json`), the closure-aware update that grants
//! ([`Store::close_task`] → [`TaskClosure`]) and the reflection log
//! ([`Store::log_reflection`], [`ReflectionTrigger`]), the derived
//! totals ([`DataRoot::rewards_totals`] → [`RewardsTotals`] — never
//! stored), the reward events read (§15's display half, U63:
//! [`DataRoot::rewards_events`] → [`RewardsEvents`], the grants and
//! reflections verbatim in root order), and the `estimate` on
//! task-creation events. The serialized
//! bytes are pinned by the golden files under `tests/golden/` (spec
//! §10, `.agents/knowledge/workflows/golden-files.md`): `event_log`,
//! `serialize_nodes`, `registry`, `snapshot` and `config`. The cli
//! inspector reads the store back, and §4.3's one recovery operation is
//! landed: [`DataRoot::repair`] → [`RepairOutcome`]/[`TornTail`], which
//! truncates a torn last line under §8's lock and refuses — as an
//! outcome, never as an error, §6.1's list being closed — everything a
//! truncation cannot fix. Through M1 the constructors return the
//! concrete [`JsonlStore`]; M3's server gets a boxed opener
//! (`Box<dyn Store>`) if it would otherwise have to name the engine —
//! the `dyn Store` object-safety pin keeps that available, and it is
//! deliberately not built now.

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
mod config;
mod error;
mod event;
mod layout;
mod lock;
mod log;
mod reader;
mod registry;
mod repair;
mod snapshot;
mod store;

pub use codec::{from_bytes, to_bytes};
pub use error::{LockTarget, StoreError};
pub use event::{
    BlocksEdgeDto, DecisionPatch, DocPatch, EpicPatch, Event, ReflectionTrigger, ReplayFault,
    TaskPatch, apply_decision_patch, apply_doc_patch, apply_epic_patch, apply_task_patch,
    replay_step,
};
pub use layout::DataRoot;
pub use lock::RegistryLock;
pub use log::JsonlStore;
pub use reader::{EventCensus, GrantedReward, RewardReflection, RewardsEvents, RewardsTotals};
pub use registry::{Registry, RegistryEntry};
pub use repair::{RepairOutcome, TornTail};
pub use store::{Compaction, Store, TaskClosure};

/// The store format version — spec §4: "a format version in the header;
/// a format change is a migration, never an in-place edit of meaning".
///
/// The single source every versioned surface references: golden file
/// names embed it (`<subject>.v<N>.golden`), the event-log header carries
/// it ([`JsonlStore::create_log`] writes it as the log's first line and
/// [`JsonlStore::open_log`] checks it before replaying anything), the
/// registry document carries it ([`Registry::load`] checks it), the
/// snapshot document carries it (`snapshot` checks it before the strict
/// parse), and stored bytes disagreeing with it are §6.1's
/// `version_mismatch`. A format change bumps this constant and lands a
/// migration; during one, the old and new goldens coexist
/// (`.agents/knowledge/workflows/golden-files.md`). Note that the
/// snapshot's `log_offset` couples `snapshot.v<N>` to `event_log.v<N>`'s
/// exact bytes: an event-schema change moves two goldens.
///
/// Version 2 (§15, D23) is a strict superset of 1: the `estimate` on
/// task-creation events (absent reads as `None`), the `reward_grant` and
/// `reward_reflect` event kinds, and the per-root config document. A v1
/// log, registry or snapshot replays identically under this build —
/// [`PREDECESSOR_VERSION`] names what it accepts — and a write-capable
/// open migrates a v1 log's header in place.
pub const FORMAT_VERSION: u32 = 2;

/// The predecessor format this build reads as a strict superset (§15,
/// D23): a v1 store — log, registry, snapshot — replays identically
/// under the v2 reader, and a write-capable open of a v1 log migrates
/// its header in place. Anything older is damage this build never wrote
/// and reports as [`StoreError::VersionMismatch`]; anything newer is
/// the migration question §4's version gate exists to ask.
pub const PREDECESSOR_VERSION: u32 = 1;
