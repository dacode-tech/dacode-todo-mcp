//! The D4 layout: where a data root keeps its per-repo stores and the
//! shared registry, and the only code in this crate that knows.
//!
//! Under the **injected** root (the binary configures it — §13/D8 make
//! the location the server's configuration, never a cwd heuristic; D4's
//! default *location* is `~/.local/share/dacode-todo/`, but nothing
//! below the binary layer reads the environment, a home directory, a
//! clock or the cwd):
//!
//! ```text
//! <root>/registry.json            the shared slug index
//! <root>/registry.json.lock       its advisory lock (§8)
//! <root>/repos/<slug>/event-log.jsonl             one repository's log
//! <root>/repos/<slug>/event-log.jsonl.lock        its advisory lock (§8)
//! <root>/repos/<slug>/event-log.jsonl.snapshot    its latest snapshot (§4.3)
//! ```
//!
//! The names are part of the versioned format (§4) and exist here as
//! constants behind [`DataRoot`]'s accessors, so no other code — t13's
//! inspector and repair included — ever hardcodes a path. One rule
//! derives every file beside the log with no exception: **a derived file
//! is the protected file's path plus a suffix** — `.lock` for the
//! advisory lock, `.snapshot` for the snapshot — never a fixed
//! directory-level name, so two logs in one directory cannot silently
//! share either. The rule exists because the lock follows the inode, and
//! the protected files that *are* atomically rewritten (the registry, the
//! snapshots) must never be their own lock target (see `lock.rs`).
//!
//! `repos/<slug>/` is traversal-safe by construction: [`RepoSlug`]'s
//! charset admits no `.`, no `..`, no separator and no empty string, so
//! a slug can only ever name one directory level below `repos/` (pinned
//! by `tests/layout/paths.rs`, not assumed).
//!
//! # The total lock order (§8)
//!
//! The registry lock is the **least element**; store locks follow in
//! sorted-slug order ([`RepoSlug`]'s derived `Ord`). §8 states the
//! sorted-slug half; the registry's place is this crate's decision, and
//! what makes the whole order safe is that **the registry holds only
//! immutable fields, so no operation ever needs the registry lock while
//! holding a store lock**: [`DataRoot::register`] takes registry → the
//! new slug's store, and [`DataRoot::open_many`] takes sorted slugs
//! only. If a mutable field ever enters the registry — a `path` index,
//! say — that argument dies on the spot: `repo_set_path` would then need
//! the registry lock *while holding the store lock*, acquiring in the
//! forbidden order. Extend the registry format only together with a
//! re-derived lock order.
//!
//! # Registry ↔ store divergence is a report, not a check
//!
//! [`DataRoot::open`] does not consult the registry: the store directory
//! is authoritative for that repo's data, and a missing one is the
//! engine's honest io error. [`DataRoot::register`] writes store bytes
//! *before* the registry index, so a crash can never leave a registry
//! entry with no store — the property the ordering buys. The reverse
//! (a store the registry does not list) has two windows, told apart by
//! whether the log's first byte landed:
//!
//! - A crash **before the log exists** leaves an *empty* `repos/<slug>/`.
//!   Re-registering completes it — the directory is not a refusal, only a
//!   log is (see [`DataRoot::register`]).
//! - A crash **after the log landed but before the index was saved**
//!   leaves a store the registry does not list. `register` refuses it as
//!   `duplicate` — a half-created store is never silently adopted, that
//!   would hide the crash — and a human resolves it through t13's
//!   inspector.
//!
//! Either way the divergence is *reported* (by t13), never silently
//! repaired here. The registry query side inherits the same reading:
//! [`DataRoot::repo_list`] answers from the index, so a store the index
//! does not list is not listed, and [`DataRoot::repo_show`] refuses one as
//! `unknown_id` instead of adopting it.

use std::fs;
use std::path::{Path, PathBuf};

use todo_domain::{DomainError, Repo, RepoSlug};

use crate::error::{LockTarget, StoreError, io_error};
use crate::lock::RegistryLock;
use crate::log::JsonlStore;
use crate::registry::Registry;
use crate::store::Store;

/// The registry document's file name (§2's shared registry file).
const REGISTRY_FILE: &str = "registry.json";
/// The registry's lock file: the protected path plus `.lock`, the one
/// rule of this module.
const REGISTRY_LOCK_FILE: &str = "registry.json.lock";
/// The per-repo store directory's name under `repos/`.
const REPOS_DIR: &str = "repos";
/// The event log's file name inside a store directory — t10's fixture
/// spelling, kept (the name is part of the versioned format, §4).
const LOG_FILE: &str = "event-log.jsonl";
/// A log's lock file: the protected path plus `.lock`.
const LOG_LOCK_FILE: &str = "event-log.jsonl.lock";
/// The lock-path rule itself, spelled once: every lock in this crate is
/// its protected file's path plus this suffix.
const LOCK_SUFFIX: &str = ".lock";
/// A log's snapshot file: the protected path plus `.snapshot` — t11's
/// derivation rule extended (a derived file is the protected file's path
/// plus a suffix), so two logs in one directory cannot share a snapshot.
/// The snapshot gets no lock file of its own: an atomically rewritten
/// file is never its own lock target (`lock.rs`), and the log's lock
/// already covers it.
const SNAPSHOT_SUFFIX: &str = ".snapshot";

/// An injected data root and the whole D4 layout under it: per-repo
/// stores in `repos/<slug>/` plus the shared registry (§2).
///
/// The root is taken as given — no validation, no canonicalization, both
/// of which would read the filesystem or the cwd, and §8 forbids the
/// clock and the environment below the binary layer. A relative root is
/// the caller's choice; the binary injects an absolute one.
///
/// Constructing a `DataRoot` touches nothing. Two paths create:
/// [`DataRoot::register`] — the store directory, the log, and the registry
/// — and [`DataRoot::lock_registry`], which makes the root directory so
/// the registry's lock file has a home. The read paths ([`Registry::load`],
/// [`DataRoot::open`], [`DataRoot::open_many`], [`DataRoot::repo_list`] and
/// [`DataRoot::repo_show`], the last two opening stores) create no store
/// and no registry, with one exception: opening a store whose directory
/// exists but whose log is missing creates a zero-byte `.lock` beside the
/// absent log before the open fails — a lock file's bytes mean nothing, so
/// the debris is harmless (see [`crate::JsonlStore::open_log`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRoot {
    /// The injected root directory.
    root: PathBuf,
}

impl DataRoot {
    /// A data root at `root`. Injected by the caller — the binary
    /// resolves D4's default location and passes an absolute path; this
    /// crate never reads the environment, home or cwd to find one.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The injected root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `<root>/repos` — the parent of every store directory.
    pub fn repos_dir(&self) -> PathBuf {
        self.root.join(REPOS_DIR)
    }

    /// `<root>/repos/<slug>` — one repository's store directory. The
    /// slug's charset makes this traversal-safe by construction (module
    /// docs).
    pub fn store_dir(&self, slug: &RepoSlug) -> PathBuf {
        self.repos_dir().join(slug.as_str())
    }

    /// `<root>/repos/<slug>/event-log.jsonl` — the store's event log.
    pub fn log_path(&self, slug: &RepoSlug) -> PathBuf {
        self.store_dir(slug).join(LOG_FILE)
    }

    /// `<root>/repos/<slug>/event-log.jsonl.lock` — the store's advisory
    /// lock file (§8).
    pub fn lock_path(&self, slug: &RepoSlug) -> PathBuf {
        self.store_dir(slug).join(LOG_LOCK_FILE)
    }

    /// `<root>/repos/<slug>/event-log.jsonl.snapshot` — the store's
    /// latest snapshot (§4.3), the log path plus the `.snapshot` suffix.
    pub fn snapshot_path(&self, slug: &RepoSlug) -> PathBuf {
        snapshot_path_for(&self.log_path(slug))
    }

    /// `<root>/registry.json` — the shared registry (§2).
    pub fn registry_path(&self) -> PathBuf {
        self.root.join(REGISTRY_FILE)
    }

    /// `<root>/registry.json.lock` — the registry's advisory lock file
    /// (§8).
    pub fn registry_lock_path(&self) -> PathBuf {
        self.root.join(REGISTRY_LOCK_FILE)
    }

    /// Acquires the registry's advisory lock (§8's one registry writer).
    /// The guard's `Drop` releases it. A contended registry is
    /// [`StoreError::Locked`] with [`LockTarget::Registry`].
    ///
    /// Creates the root directory when absent — the lock file needs a
    /// home — so this is one of the two creating paths of a `DataRoot`
    /// (see the struct docs), not a pure read.
    pub fn lock_registry(&self) -> Result<RegistryLock, StoreError> {
        fs::create_dir_all(&self.root).map_err(|error| io_error("create the data root", &error))?;
        RegistryLock::acquire(&self.registry_lock_path())
    }

    /// Registers `repo` and returns its open, locked store — §6's
    /// `repo_register` underneath (§7: the slug is the key, chosen at
    /// registration and immutable ever after).
    ///
    /// The order is the crash-consistency order (module docs): registry
    /// lock → load the registry → refuse a duplicate → create the store
    /// → write the registry → return the store still holding its own
    /// lock. Both refusals are checked under the registry lock and are
    /// the same `duplicate` code: the registry lists the slug, **or**
    /// `log_path(slug)` already exists — a half-created store directory
    /// is never silently adopted, a human resolves it through t13's
    /// inspector. An *empty* store directory with no log is not a
    /// refusal: creating the store completes what a crashed earlier
    /// attempt left behind.
    pub fn register(&self, repo: Repo) -> Result<JsonlStore, StoreError> {
        let registry_lock = self.lock_registry()?;
        let mut registry = Registry::load(self)?;
        if registry.contains(&repo.slug) || self.log_path(&repo.slug).exists() {
            return Err(duplicate(&repo.slug));
        }

        // Store bytes land before the registry index, so a crash can
        // never leave an entry with no store. The reverse window is the
        // module docs' second case: the log is down, so a
        // re-registration is refused as a duplicate and t13's inspector
        // resolves it.
        let store = self.create_store(&repo)?;

        registry.insert(&repo);
        registry.save(self)?;
        drop(registry_lock);
        Ok(store)
    }

    /// Creates a repository's store directory and log under this root,
    /// returning the locked store — [`crate::Store::create`]'s body and
    /// the store-creation half of [`DataRoot::register`].
    ///
    /// This is the one write path that creates the per-repo directory:
    /// [`JsonlStore::create_log`] keeps the engine's bare-path semantics
    /// (`create_new` on the log, no parent creation), so the D4 directory
    /// is made here, where the layout is known. It touches **no**
    /// registry — a store created this way is on disk but unlisted, which
    /// is exactly what a registration crashed after this point leaves
    /// behind, and what a later [`DataRoot::register`] then refuses as a
    /// `duplicate` (module docs).
    pub(crate) fn create_store(&self, repo: &Repo) -> Result<JsonlStore, StoreError> {
        fs::create_dir_all(self.store_dir(&repo.slug))
            .map_err(|error| io_error("create the store directory", &error))?;
        JsonlStore::create_log(self.log_path(&repo.slug), repo.clone())
    }

    /// Opens the store registered under `slug` and returns it locked.
    ///
    /// The registry is **not** consulted: the store directory is
    /// authoritative for that repo's data, and a missing log is the
    /// engine's honest [`StoreError::Io`] (`open the event log:
    /// NotFound`). Registry↔store divergence is a report, and t13's
    /// inspector owns it (module docs). Creates nothing.
    pub fn open(&self, slug: &RepoSlug) -> Result<JsonlStore, StoreError> {
        JsonlStore::open_log(self.log_path(slug))
    }

    /// Opens several stores for §2's cross-repo queries: "iterate the
    /// registry and open per-repo stores". The slice-of-references shape
    /// lets a caller pass [`Registry::slugs`] straight in —
    /// `open_many(&registry.slugs())` — with no clone-collect between the
    /// two halves of that sentence.
    ///
    /// The slugs are deduplicated and sorted ascending, and the locks are
    /// acquired in exactly that order — §8's sorted-slug rule, and the
    /// order that can never deadlock against another `open_many`. Dedupe
    /// is required, not cosmetic: a lock is per descriptor, so one slug
    /// twice is two fds on one lock file, and the second conflicts with
    /// the first.
    ///
    /// All-or-nothing: on any failure the stores already opened are
    /// dropped as the local vector goes out of scope, and each store's
    /// `Drop` releases its advisory lock, so the error is returned with
    /// nothing left held. RAII provides this — there is no explicit
    /// unwind, and the rollback test in `tests/layout/lock.rs` pins the
    /// observable result. The success case returns the stores in
    /// sorted-slug order.
    pub fn open_many(&self, slugs: &[&RepoSlug]) -> Result<Vec<JsonlStore>, StoreError> {
        let mut sorted: Vec<&RepoSlug> = slugs.to_vec();
        sorted.sort();
        sorted.dedup();

        let mut stores: Vec<JsonlStore> = Vec::with_capacity(sorted.len());
        for slug in sorted {
            // `?` returns early on any failure; `stores` then drops and
            // releases every lock already taken (see the doc above).
            stores.push(self.open(slug)?);
        }
        Ok(stores)
    }

    /// Lists every registered repository — §6's `repo_list` underneath.
    ///
    /// **The total order is ascending slug**, and it is the registry's own
    /// rather than one this method imposes: [`Registry::load`] keeps its
    /// entries sorted and [`Registry::slugs`] hands them back that way, so
    /// registration order never reaches the answer (§8's total orders
    /// everywhere). Sorted, though, and **not deduplicated**: an index this
    /// crate wrote cannot hold one slug twice, because
    /// [`DataRoot::register`] refuses a duplicate, but a hand-spliced repeat
    /// parses and is listed twice — this method answers what the index says.
    ///
    /// Each row is the listed store's own [`Repo`] node, because §7's
    /// mutable `name` and `path` live only there — the index holds the
    /// immutable half alone (see [`crate::Registry`]) — so answering costs
    /// one store open per listed repo. `registered_at` is the node's too.
    /// It equals the registry entry's by construction: [`DataRoot::register`]
    /// writes both from the same [`Repo`], nothing rewrites an entry (§6
    /// has no `repo_delete`, D3 never reuses a slug) and no `repo_set_*`
    /// mutation touches the field. §6.1's closed list has no code for "the
    /// index and the node disagree", so production does not cross-check a
    /// contradiction it could not report; `tests/layout/repos.rs` pins the
    /// equality where it is observable.
    ///
    /// # One lock at a time, in sorted order
    ///
    /// The registry read takes no lock — only its writers do — and each
    /// store is opened, read and dropped before the next opens, so **two
    /// locks are never held at once**: §8's sorted-slug rule is satisfied
    /// trivially, because no acquisition order exists to deadlock. That is
    /// [`DataRoot::open_many`] deliberately *not* used. A listing protects
    /// no cross-store invariant, holding every descriptor at once has a
    /// per-process ceiling, and one lock at a time is the shorter
    /// contention window.
    ///
    /// **The trap that follows: this call takes every listed store's lock in
    /// turn, so anything else holding one fails the whole call with
    /// `locked`.** Three ways, all §8's per-descriptor contention (see
    /// `lock.rs`): a caller already holding a handle for a listed repo
    /// contends with *itself*, since two handles in one process conflict; a
    /// second listing contends with the first; and [`DataRoot::register`]
    /// returns its new store still holding that store's lock, so a listing
    /// that reads the index after the save but before the registrant drops
    /// the handle fails on the repo just registered. M3 answers `repo_list`
    /// from the handles it already holds, or before it opens any. The failed
    /// call leaves nothing held: every store it opened dropped on the way
    /// out, and a store's `Drop` is its release.
    ///
    /// # A store that will not open fails the whole call
    ///
    /// The first [`DataRoot::open`] fault propagates and no list is
    /// returned at all — a corrupt log, a foreign format version, a listed
    /// slug whose log is gone and a contended store each fail the call with
    /// the error [`DataRoot::open`] itself reports, byte-identical. §6.1's
    /// list is closed and has no code for "partially listed", and a row
    /// that quietly disappears is a silent omission. The mirror half: a
    /// store directory the registry does **not** list — what a registration
    /// that crashed after the log landed leaves behind — is simply not
    /// listed, and resolving that stays t13's inspector's job (module
    /// docs).
    ///
    /// What the per-repo open costs, and so whether the registry should
    /// ever cache `name`/`path` at the price of a golden-visible format
    /// change, is measured — by `tests/layout/repos.rs`'s ignored
    /// `cold_repo_list_open_cost`, whose numbers live in that run's output
    /// and the tracker node's evidence, never in a comment here.
    pub fn repo_list(&self) -> Result<Vec<Repo>, StoreError> {
        let registry = Registry::load(self)?;
        let mut repos: Vec<Repo> = Vec::with_capacity(registry.entries().len());
        for slug in registry.slugs() {
            repos.push(self.repo_node(slug)?);
        }
        Ok(repos)
    }

    /// Shows one registered repository — §6's `repo_show` underneath, and
    /// the same record [`DataRoot::repo_list`] carries, for one slug.
    ///
    /// The registry **is** consulted here, unlike [`DataRoot::open`]: a
    /// slug the index does not list is §6.1's `unknown_id` even when a
    /// store directory sits at `repos/<slug>/`. `repo_show` is a registry
    /// query (§6 groups all three registry tools under §7), the registry is
    /// the index of *which repositories exist*, and answering for an
    /// unlisted directory would silently adopt the half-created state
    /// [`DataRoot::register`] refuses as a `duplicate` forever after. A
    /// listed slug then opens its store, so a listed store that will not
    /// open is [`DataRoot::repo_list`]'s named error, reported identically,
    /// and one lock is held at a time by the same argument.
    ///
    /// **That mapping is a reading, flagged so a veto is a named diff.**
    /// §6.1 lists `unknown_id` and glosses nothing; the gloss that creates
    /// the tension is the domain's own — [`DomainError::UnknownId`] is "no
    /// node exists under `id`", and a registry is not the graph, so a slug
    /// it does not list is not literally an unknown *node*. The rejected
    /// alternative is [`DataRoot::open`]'s behaviour: no registry consult,
    /// and §6.1's `io` for a slug with no store — which would answer a
    /// different code for "this server has no such repository" depending on
    /// whether a directory happens to sit at that slug.
    pub fn repo_show(&self, slug: &RepoSlug) -> Result<Repo, StoreError> {
        if !Registry::load(self)?.contains(slug) {
            return Err(unknown_slug(slug));
        }
        self.repo_node(slug)
    }

    /// One store's [`Repo`] node: open, check that the store this slug names
    /// really holds that repo's graph, clone, and let the store drop —
    /// releasing its lock before returning, which is the one-lock-at-a-time
    /// discipline both queries above document. **Every *registry* query that
    /// answers a [`Repo`] goes through this method, and a new one must too**:
    /// the check below is the only thing between a caller and another
    /// repository's record, and a method that opened the store directly would
    /// lose it silently.
    ///
    /// The check is the snapshot reconcile's slug cross-check one level out:
    /// a node whose slug is not the one this store was opened under means the
    /// directory and its contents disagree, which no reader can resolve and
    /// no answer may paper over — a listing would otherwise carry the same
    /// repo twice under two slugs and break the total order it documents. So
    /// it is §6.1's `corrupt_store`, unpositioned, because the fault is a
    /// relation between two facts rather than a byte in one file, and its
    /// detail names both slugs — an id, not a path (§7). Unreachable through
    /// this crate's own writes ([`DataRoot::register`] creates the store from
    /// the same [`Repo`] it indexes and D3 never reuses a slug); reachable by
    /// a store directory copied or moved out of band. [`DataRoot::open`]
    /// itself does not check this, so the inspector's `dump`/`verify` still
    /// report a copied store under the slug they were asked for.
    fn repo_node(&self, slug: &RepoSlug) -> Result<Repo, StoreError> {
        let store = self.open(slug)?;
        let repo = store.graph().repo();
        if &repo.slug != slug {
            return Err(StoreError::CorruptStore {
                offset: None,
                detail: format!("{DETAIL_FOREIGN_STORE}{slug} holds repo {}", repo.slug),
            });
        }
        Ok(repo.clone())
    }
}

/// The detail of a store whose graph names another repository than the slug
/// it was opened under — caller-facing contract text like every detail this
/// crate writes, so it is deterministic, path-free (both halves are slugs,
/// and a slug is an id, §7) and pinned by a needle in
/// `tests/layout/repos.rs`.
const DETAIL_FOREIGN_STORE: &str = "the store opened for ";

/// The refused re-registration: a repo's id *is* its slug (D3), so the
/// taken id is the slug's own string — the domain's `duplicate` code,
/// because that is what happened (§6.1).
fn duplicate(slug: &RepoSlug) -> StoreError {
    StoreError::Domain(DomainError::Duplicate {
        id: slug.to_string(),
    })
}

/// A slug the registry does not list, as [`DataRoot::repo_show`] refuses
/// it: §6.1's `unknown_id`, spelled from the slug's own string the way
/// [`duplicate`] spells the taken id, because a repo's id *is* its slug
/// (D3) and the registry is the index of which repos exist.
fn unknown_slug(slug: &RepoSlug) -> StoreError {
    StoreError::Domain(DomainError::UnknownId {
        id: slug.to_string(),
    })
}

/// The one lock-path rule of this module: the protected file's path plus
/// `.lock`. [`crate::JsonlStore`]'s bare-path constructors use this so a
/// log's lock is derived, never spelled twice.
pub(crate) fn lock_path_for(path: &Path) -> PathBuf {
    suffixed(path, LOCK_SUFFIX)
}

/// The snapshot-path rule: the log's path plus `.snapshot` — a pure
/// suffix append on the full file name, never a directory-level name, so
/// two logs in one directory cannot share a snapshot. The engine derives
/// its snapshot path with this; [`DataRoot::snapshot_path`] returns the
/// same path for layout callers.
pub(crate) fn snapshot_path_for(path: &Path) -> PathBuf {
    suffixed(path, SNAPSHOT_SUFFIX)
}

/// One path plus one suffix — the derivation rule both helpers share.
fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let mut derived = path.as_os_str().to_os_string();
    derived.push(suffix);
    PathBuf::from(derived)
}

/// Whether the store directory that would hold the log at `path` is
/// absent — the honest "no store here" that the engine's bare-path
/// constructors report as the log's own io `NotFound`, never as a lock
/// fault.
///
/// The lock file sits beside the log (one rule, above), so a directory
/// that does not exist can hold neither: `create(true)` on the lock file
/// would fail `NotFound` there, and Decision 11 pins the missing-store
/// report to the engine's `open the event log: NotFound`, not to the lock
/// machinery. Checking the directory up front keeps §8's lock-first order
/// for every store that *is* there while never surfacing a lock detail for
/// one that is not. A bare file name (empty parent → cwd) or a root path
/// is not "missing" here; the lock attempt handles those.
pub(crate) fn store_dir_missing(path: &Path) -> bool {
    match path.parent() {
        Some(parent) => !parent.as_os_str().is_empty() && !parent.exists(),
        None => false,
    }
}

/// The [`LockTarget`] of a log living at `path`: inside the D4 layout,
/// the store directory's name *is* the slug (`repos/<slug>/…`), so a
/// contended bare-path lock reports which repo it refused without ever
/// carrying a path.
///
/// A path outside the layout — a corruption fixture's temp file — has no
/// directory that parses as a slug, and none may be invented (§7: the
/// slug is the key, and this store was not opened by one). The target is
/// [`LockTarget::Unnamed`] (f17, t13's decision), so the contended
/// report is §6.1's `locked` — honest, because nothing was read or
/// written — rather than the `io` this arm used to degrade to. §6.1's
/// code list is untouched: `locked` was already on it. The case is only
/// reachable when such a path is *contended* (see [`crate::lock`]'s
/// lazy-target note): every other fault on a bare path keeps t10's exact
/// report.
///
/// The derivation trusts the parent directory's name, so a contended
/// bare-path open of a log whose parent happens to be a valid slug names
/// *that* slug — which need not be a repo involved in the call. Through
/// [`DataRoot`] this cannot happen (the parent always is the store's own
/// slug), so [`LockTarget::Unnamed`] is unreachable for layout callers —
/// including t13's inspector, which opens by slug. The non-layout callers
/// are the bare-path [`JsonlStore::open_log`] uses, which are all test
/// fixtures today.
///
/// The `Result` is vestigial: both arms now produce a target, and the
/// shape survives only because the lock layer's `acquire_reporting` takes
/// a fallible closure.
pub(crate) fn store_target_from_log_path(path: &Path) -> Result<LockTarget, StoreError> {
    let dir_name = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned());
    match dir_name
        .as_deref()
        .and_then(|name| RepoSlug::new(name.to_owned()).ok())
    {
        Some(slug) => Ok(LockTarget::Repo(slug)),
        None => Ok(LockTarget::Unnamed),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use todo_domain::RepoSlug;

    use super::store_target_from_log_path;
    use crate::error::{LockTarget, StoreError};

    #[test]
    fn a_non_layout_path_names_no_slug() {
        // A log whose parent directory is not a valid slug — the shape a
        // corruption fixture's temp file has — cannot name a repo (§7), so
        // the target is `Unnamed`: the contended report is §6.1's
        // `locked`, never a slug invented from the path and never the
        // `io` this arm used to degrade to (f17).
        let path = Path::new("/tmp/Not A Slug/event-log.jsonl");
        let target = store_target_from_log_path(path).unwrap();
        assert_eq!(target, LockTarget::Unnamed);
        let error = StoreError::Locked { target };
        assert_eq!(error.code(), "locked");
        assert!(!error.to_string().contains('/'), "{error} carries no path");
    }

    #[test]
    fn a_layout_path_names_its_own_slug() {
        // The happy path the derivation exists for: `repos/<slug>/…`
        // yields that slug, so a contended D4 open names the right repo.
        let path = Path::new("/data/repos/camera/event-log.jsonl");
        let target = store_target_from_log_path(path).unwrap();
        assert_eq!(
            target,
            LockTarget::Repo(RepoSlug::new("camera".to_owned()).unwrap())
        );
    }
}
