//! The persistence surface of spec §4.2: one trait, owned by this crate,
//! behind which the JSONL engine (D1) is sealed. The domain and server
//! crates never see past it — no engine type, no engine error, no engine
//! feature flag (§2).
//!
//! §4.2 names five responsibilities: open/create, read the graph, append
//! a mutation, snapshot/compact, and the format version. This trait holds
//! four of them; snapshot/compact is the later commit:
//!
//! - **open/create** are [`Store::create`] and [`Store::open`]: they take
//!   a [`crate::DataRoot`] and a repo or a slug — never a bare file path —
//!   and resolve the D4 layout (§2) themselves. They are `where Self:
//!   Sized` so the trait stays object-safe: a constructor cannot be called
//!   through a `dyn Store` (there is no `Self` to build), but that is no
//!   loss — a caller holding a `dyn Store` already has an opened store.
//!   The bound keeps them out of the vtable, so `dyn Store` still compiles
//!   and every mutation below stays dispatchable; the
//!   `_OBJECT_SAFE_IS_PINNED` const asserts that at compile time, because
//!   M3's server depends on it.
//! - **snapshot/compact** arrive with t12 (§4.1: periodic snapshot plus
//!   truncation bounds replay). The trait grows in-crate; a caller that
//!   only appends and reads is unaffected.
//!
//! Two contracts every method shares:
//!
//! - **Writes return the new state** of what they wrote (§6), so a caller
//!   never needs a read-after-write round trip: the allocated node, the
//!   epic-delete cascade, `()` for an edge.
//! - **A rejection is atomic; an io failure is session-fatal.** The domain
//!   validates first and a rejection leaves both the graph and the log
//!   untouched ([`StoreError::Domain`]). If the append itself fails after
//!   a successful apply, memory is one event ahead of disk — exactly §4.3's
//!   crash window, with the crash replaced by an io error. There is no
//!   clone-and-rollback and no failed flag: the caller must reopen the
//!   store, and replay from disk drops the unpersisted mutation. Treat an
//!   [`StoreError::Io`] from a mutation as the end of the session.
//!
//! Timestamps are caller-passed on every mutation (§8: nothing below the
//! binary layer reads a wall clock). Where the domain takes none — epics
//! and the repo node carry no timestamps (§3) — the `at` exists solely as
//! the audit instant of the event line, which is then the only record of
//! when the change happened.

use todo_domain::{
    BlocksEdge, Decision, DecisionId, Doc, DocId, Epic, EpicId, Graph, Repo, RepoSlug, Task,
    TaskId, Timestamp,
};
use todo_domain::{DecisionDraft, DocDraft, EpicDraft, TaskDraft};

use crate::error::StoreError;
use crate::event::{DecisionPatch, DocPatch, EpicPatch, TaskPatch};
use crate::layout::DataRoot;

/// The whole persistence surface a caller above this crate uses (§4.2).
pub trait Store {
    /// Creates the store for `repo` in the D4 layout under `root`
    /// (`repos/<slug>/event-log.jsonl`, §2) and returns it open and
    /// locked. The slug must not already have a log there: creation is
    /// `create_new`, so an existing store is a named error, never a
    /// truncation. This does **not** touch the registry — registering a
    /// repo (the index write under §8's registry lock, with the duplicate
    /// refusal) is [`DataRoot::register`], which builds on this.
    ///
    /// **A caller above this crate registers a new repository only through
    /// [`DataRoot::register`], never through this constructor directly.**
    /// `create` writes a log the registry does not list, which is exactly
    /// the half-created state `register` then refuses as a `duplicate`
    /// forever after (see [`DataRoot`]'s divergence note) — so calling it
    /// directly strands the slug, resolvable only by t13's inspector. It
    /// is the engine seam a fallback implementation plugs into, not the
    /// registration path.
    ///
    /// `where Self: Sized`: a constructor cannot dispatch through
    /// `dyn Store`, and does not need to — see the module docs.
    fn create(root: &DataRoot, repo: Repo) -> Result<Self, StoreError>
    where
        Self: Sized;

    /// Opens the store for `slug` in the D4 layout under `root`, replays
    /// its log into the graph it describes (§4.1), and returns it locked.
    /// The registry is not consulted: the store directory is
    /// authoritative, and a missing log is the engine's honest io error.
    ///
    /// `where Self: Sized`: a constructor cannot dispatch through
    /// `dyn Store`, and does not need to — see the module docs.
    fn open(root: &DataRoot, slug: &RepoSlug) -> Result<Self, StoreError>
    where
        Self: Sized;

    /// The in-memory graph: every read is a query over it, never a
    /// reload (§4.1 — load replays the log once, at open).
    fn graph(&self) -> &Graph;

    /// The format version this store writes and read successfully —
    /// [`crate::FORMAT_VERSION`] (§4's versioning requirement).
    fn format_version(&self) -> u32;

    /// Adds an epic and appends its `epic_add` line. `at` is the audit
    /// instant: an epic carries no timestamps (§3), so the line is the
    /// record of when it was added.
    fn add_epic(&mut self, draft: EpicDraft, at: Timestamp) -> Result<Epic, StoreError>;

    /// Adds a task and appends its `task_add` line; `at` stamps the task's
    /// `created` and `updated` (§3's closure discipline applies at
    /// creation, so a draft without `verify` is rejected and nothing is
    /// written).
    fn add_task(&mut self, draft: TaskDraft, at: Timestamp) -> Result<Task, StoreError>;

    /// Adds a decision and appends its `decision_add` line; `at` stamps
    /// both of the node's timestamps.
    fn add_decision(&mut self, draft: DecisionDraft, at: Timestamp)
    -> Result<Decision, StoreError>;

    /// Registers a doc and appends its `doc_register` line; `at` stamps
    /// both of the node's timestamps.
    fn add_doc(&mut self, draft: DocDraft, at: Timestamp) -> Result<Doc, StoreError>;

    /// Patches an epic and appends its `epic_update` line. `at` is the
    /// audit instant alone — an epic has no `updated` field (§3).
    fn update_epic(
        &mut self,
        id: EpicId,
        at: Timestamp,
        patch: EpicPatch,
    ) -> Result<Epic, StoreError>;

    /// Patches a task and appends its `task_update` line; `at` becomes the
    /// task's `updated`. Setting [`TaskPatch::epic`] is a reparenting and
    /// is checked like one (§5.1) — there is no separate verb for it, in
    /// the log or here.
    fn update_task(
        &mut self,
        id: TaskId,
        at: Timestamp,
        patch: TaskPatch,
    ) -> Result<Task, StoreError>;

    /// Patches a decision and appends its `decision_update` line; `at`
    /// becomes the node's `updated`. Signing it changes no edge: its
    /// `blocks` edges simply stop gating (§3/D10).
    fn update_decision(
        &mut self,
        id: DecisionId,
        at: Timestamp,
        patch: DecisionPatch,
    ) -> Result<Decision, StoreError>;

    /// Patches a doc and appends its `doc_update` line; `at` becomes the
    /// node's `updated`. The path is not patchable here — see
    /// [`Store::set_doc_path`].
    fn update_doc(&mut self, id: DocId, at: Timestamp, patch: DocPatch) -> Result<Doc, StoreError>;

    /// Moves a doc's body and appends its own `doc_set_path` line (§6,
    /// D10: an audited verb of its own, so a citation's address change is
    /// visible in the log); `at` becomes the node's `updated`.
    fn set_doc_path(&mut self, id: DocId, path: String, at: Timestamp) -> Result<Doc, StoreError>;

    /// Updates the checkout path and appends its `repo_set_path` line
    /// (§7: the slug is the key, the path an attribute). `at` is the audit
    /// instant — a [`Repo`] has no `updated` field (§3).
    fn set_repo_path(&mut self, path: String, at: Timestamp) -> Result<Repo, StoreError>;

    /// Updates the display name and appends its `repo_set_name` line; `at`
    /// is the audit instant.
    fn set_repo_name(&mut self, name: String, at: Timestamp) -> Result<Repo, StoreError>;

    /// Deletes an epic and appends its `epic_delete` line, returning the
    /// task ids the cascade took with it, ascending (§5.1: no orphans,
    /// ever — every task must be closed or `wontfix` first, or the whole
    /// write is rejected). `at` is the audit instant.
    ///
    /// The log records the intent, not the extent: replay re-runs the
    /// domain cascade, so one rule decides both. The deleted tasks'
    /// numbers stay burned (§3), which is what makes the gaps in a later
    /// allocation meaningful.
    fn delete_epic(&mut self, id: EpicId, at: Timestamp) -> Result<Vec<TaskId>, StoreError>;

    /// Adds a `blocks` edge and appends its `blocks_link` line, rejecting
    /// one that would close a cycle before anything is written (§5.1).
    /// `at` is the audit instant.
    fn link_blocks(&mut self, edge: BlocksEdge, at: Timestamp) -> Result<(), StoreError>;

    /// Removes a `blocks` edge and appends its `blocks_unlink` line; `at`
    /// is the audit instant. Both endpoints must exist, but an edge that
    /// is not there is not an error — the domain's answer, mirrored
    /// ([`Graph::unlink_blocks`]).
    fn unlink_blocks(&mut self, edge: BlocksEdge, at: Timestamp) -> Result<(), StoreError>;

    /// Adds a `relates` annotation and appends its `relates_link` line;
    /// `at` is the audit instant. The pair is undirected, so both
    /// spellings are one annotation and a repeat is a no-op.
    fn link_relates(&mut self, a: TaskId, b: TaskId, at: Timestamp) -> Result<(), StoreError>;

    /// Removes a `relates` annotation and appends its `relates_unlink`
    /// line; `at` is the audit instant. An absent annotation is not an
    /// error ([`Graph::unlink_relates`]).
    fn unlink_relates(&mut self, a: TaskId, b: TaskId, at: Timestamp) -> Result<(), StoreError>;
}

/// Compile-time pin: `dyn Store` must keep compiling, because M3's server
/// holds opened stores as trait objects behind the §2 boundary. The
/// `where Self: Sized` constructors are what keep the trait object-safe;
/// this const fails the build the moment a change (a new non-`Sized`
/// method, a `Self` return, a generic) makes `Store` non-object-safe, so
/// the property cannot silently regress. The binding is never called —
/// its type is the assertion.
#[allow(dead_code, reason = "a compile-time assertion; the value is its type")]
const _OBJECT_SAFE_IS_PINNED: fn(&dyn Store) = |_| {};
