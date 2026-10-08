//! §8's single writer: one advisory file lock per protected file, held
//! for the lifetime of the handle that owns it.
//!
//! The mechanism is `flock` under `std::fs::File::try_lock` — exclusive,
//! advisory, per inode. Three measured properties shape this module:
//!
//! - **Release is the close.** Dropping the [`File`] releases the lock;
//!   there is no explicit unlock and no half-released state. RAII is the
//!   whole mechanism, so the guards here have no release method. The
//!   kernel's flock teardown can outlive that close by a
//!   millisecond-scale lifetime (U26's measurement, recorded on the
//!   board — the ledger, never a source constant), so a `WouldBlock`ed
//!   acquire retries within a bounded window before reporting
//!   `locked`: the refusal then means a writer stayed active past the
//!   window, not merely that a release was in flight.
//! - **The lock follows the inode, not the path.** Renaming a locked file
//!   defeats the lock — a second writer opens the *new* inode under the
//!   old name and locks it fresh. So a lock file is never itself written
//!   by temp+rename, and a file that *is* atomically rewritten (the
//!   registry, t12's snapshots) is never its own lock target: each
//!   protected file gets a dedicated `.lock` beside it (the rule
//!   [`crate::DataRoot`] derives every lock path from).
//! - **A second descriptor conflicts, even in one process.** Contention
//!   is per fd, not per process, so these guards' contracts hold against
//!   a second handle in the same writer as much as against another one.
//!
//! The bounded retry the first bullet describes is this crate's one
//! production use of `std::time`: a sleep span, not a clock read. §8's
//! determinism is about answers as functions of state, and the retry's
//! *outcome* is — success or `locked`, a function of lock state; only
//! its *latency* is bounded by the window.
//!
//! A lock file's own bytes mean nothing — `flock` never reads them — so a
//! failed attempt may leave a zero-byte `.lock` beside a file that does
//! not (yet) exist. That debris is accepted and documented (see
//! [`crate::JsonlStore::open_log`]).
//!
//! Error mapping is on [`TryLockError`], not on `ErrorKind`: `WouldBlock`
//! is the §6.1 `locked` refusal, `Error(io)` is the filesystem's, reported
//! through the crate's one [`io_error`] shape. The mapping lives here and
//! nowhere else, and the context strings below are the only ones a lock
//! fault can carry — that is what keeps a host path out of every
//! `Locked`/`Io` detail.

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crate::error::{LockTarget, StoreError, io_error};

/// The context string a store-lock fault reports: what was attempted,
/// never where.
pub(crate) const CONTEXT_STORE: &str = "lock the store";

/// The context string a registry-lock fault reports.
const CONTEXT_REGISTRY: &str = "lock the registry";

/// The number of `WouldBlock`ed attempts retried inside the bounded
/// window before the final deciding attempt reports `locked` — the
/// mechanism that absorbs the kernel's deferred flock release after a
/// holder's exit (module docs). Pinned from both sides by the layout
/// suite's timing pins: a release inside the window is absorbed, a
/// genuinely held lock reports `locked` only after it.
const RETRY_ATTEMPTS: usize = 5;

/// The sleep between a `WouldBlock`ed attempt and its retry, in
/// milliseconds — the span that bounds the window at
/// `RETRY_ATTEMPTS × RETRY_SLEEP_MS`. Pinned by the same timing pins.
const RETRY_SLEEP_MS: u64 = 2;

/// An advisory exclusive lock on one file, held until this value drops.
///
/// There is no unlock method: closing the handle *is* the release —
/// dropping the [`File`] drops the `flock`. Keeping the file alive for
/// the guard's lifetime is therefore not bookkeeping but the mechanism
/// itself. The kernel's teardown can trail that close; absorbing the
/// trail is the acquiring side's business (module docs).
///
/// The lock is on the **inode**, not the path: a rename of the lock file
/// would defeat it, which is why nothing in this crate ever rewrites a
/// `.lock` file (module docs).
#[derive(Debug)]
pub(crate) struct FileLock {
    /// The open lock file. Its bytes are never read or written; `flock`
    /// state is the only thing it carries, and dropping this `File` is the
    /// release — so the field exists to be held, never to be read (the
    /// leading underscore says exactly that to the dead-code pass).
    _file: File,
}

impl FileLock {
    /// Opens (creating if absent, never truncating — a lock file's bytes
    /// mean nothing) and exclusively locks `path`, or reports the fault
    /// against `target`: [`StoreError::Locked`] when contended,
    /// [`StoreError::Io`] when the filesystem refused.
    ///
    /// A `WouldBlock`ed attempt is retried within the bounded window the
    /// module docs describe before `locked` is reported; an io fault is
    /// never retried.
    ///
    /// Nothing is written to the file beyond its creation: an existing
    /// lock file is left exactly as it was, whatever its contents.
    pub(crate) fn acquire(path: &Path, target: LockTarget) -> Result<Self, StoreError> {
        let context = context_of(&target);
        Self::acquire_reporting(path, context, move || Ok(target))
    }

    /// [`Self::acquire`] with the [`LockTarget`] derived **on the final
    /// refusal only** — the one point the target is ever needed is the
    /// [`StoreError::Locked`] it populates.
    ///
    /// The lazy shape exists for [`crate::JsonlStore::open_log`], whose
    /// bare path must yield the target's slug from the store directory it
    /// sits in: a D4 path always derives cleanly, and a path outside the
    /// layout (the corruption fixtures') derives [`LockTarget::Unnamed`]
    /// — §7 gives it no slug to be named by (f17, t13's decision). Since
    /// derivation runs only when the lock is contended, such a path still
    /// opens, replays and reports its engine faults exactly as before,
    /// and only a *contended* open ever produces a target — the honest
    /// `locked` refusal, naming the repo where a slug exists.
    ///
    /// A `WouldBlock`ed attempt is retried within the bounded window the
    /// module docs describe — [`RETRY_ATTEMPTS`] tries, each followed by
    /// a [`RETRY_SLEEP_MS`] sleep — and only the final attempt's
    /// `WouldBlock` reports, so the target derivation runs on that final
    /// refusal alone and a lock that frees within the window never
    /// derives one. An io fault is never retried: it is the filesystem's
    /// refusal, not contention.
    pub(crate) fn acquire_reporting(
        path: &Path,
        context: &str,
        target: impl FnOnce() -> Result<LockTarget, StoreError>,
    ) -> Result<Self, StoreError> {
        for _ in 0..RETRY_ATTEMPTS {
            match open_and_try_lock(path) {
                Ok(file) => return Ok(Self { _file: file }),
                Err(TryLockError::WouldBlock) => sleep(Duration::from_millis(RETRY_SLEEP_MS)),
                Err(TryLockError::Error(error)) => return Err(io_error(context, &error)),
            }
        }
        match open_and_try_lock(path) {
            Ok(file) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(StoreError::Locked { target: target()? }),
            Err(TryLockError::Error(error)) => Err(io_error(context, &error)),
        }
    }
}

/// The context string a lock fault reports, by target: one per protected
/// file, so an `Io` detail says which attempt failed without saying
/// where.
fn context_of(target: &LockTarget) -> &'static str {
    match target {
        // An unnamed store is still a store lock; only the registry has
        // a context of its own.
        LockTarget::Repo(_) | LockTarget::Unnamed => CONTEXT_STORE,
        LockTarget::Registry => CONTEXT_REGISTRY,
    }
}

/// The open-and-lock mechanics: open the lock file read-write (creating
/// if absent, never truncating), then `try_lock` it.
///
/// On a `PermissionDenied` open — a lock file this writer may not write
/// but must still respect — the open is retried read-only: `flock` needs
/// an open descriptor, not write access. Without the retry the fault would
/// report as `io`, degrading §8's refusal from `locked` to `io`; with it,
/// a foreign read-only lock file still yields the `locked` refusal the
/// spec promises. If the read-only open *also* fails, the **original**
/// error is reported, not the fallback's — a fallback `NotFound` must not
/// masquerade as Decision 11's "never registered" signal for what was
/// really a `PermissionDenied`. Any other open failure is reported as-is.
fn open_and_try_lock(path: &Path) -> Result<File, TryLockError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        // Never truncate: a lock file's bytes mean nothing, and
        // truncating one on every acquisition is a gratuitous write.
        .truncate(false)
        .open(path)
        .or_else(|error| {
            if error.kind() == io::ErrorKind::PermissionDenied {
                OpenOptions::new().read(true).open(path).map_err(|_| error)
            } else {
                Err(error)
            }
        })
        // An open fault is the `Error` arm's shape already; only the
        // lock call itself distinguishes refusal from fault.
        .map_err(TryLockError::Error)?;
    file.try_lock().map(|()| file)
}

/// The registry's lock guard: §8's one registry writer, public and
/// opaque.
///
/// Opaque because there is nothing to read through it — holding the value
/// *is* holding the lock, and the registry's bytes are read under it by
/// [`crate::Registry`], never through the guard. The lock is released
/// when the guard drops.
///
/// `#[must_use]` because an immediately-dropped guard releases the lock at
/// the end of the statement that produced it, which silently turns "the
/// registry is locked for this write" back into "nothing is locked".
#[must_use = "the registry lock is released when this guard drops; hold it \
              for as long as the write it protects"]
#[derive(Debug)]
pub struct RegistryLock {
    /// The held lock. Dropped with the guard, which is the release.
    _lock: FileLock,
}

impl RegistryLock {
    /// Acquires the registry lock on the file at `path`.
    pub(crate) fn acquire(path: &Path) -> Result<Self, StoreError> {
        Ok(Self {
            _lock: FileLock::acquire(path, LockTarget::Registry)?,
        })
    }
}
