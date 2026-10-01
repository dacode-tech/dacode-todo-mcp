//! §4.3's repair primitive: the one recovery operation this crate owns.
//!
//! A crash tears at most one event — the line in flight — and §4.3 makes
//! removing it an explicit, confirmed act: "a repair operation … shows the
//! torn line and truncates it on confirmation, losing exactly that one
//! in-flight event. No automatic repair path exists." [`DataRoot::repair`]
//! is that operation. The cli prints its report and takes the
//! confirmation; the truncation itself lives here because truncating a log
//! is a *write*, §8 puts every writer under the store lock, and the fault
//! classification has to agree with [`crate::JsonlStore::open_log`]'s or an
//! inspector would report a fault the store does not have. All three come
//! from reuse rather than parallel logic: the lock is the crate's own
//! [`FileLock`], the line discipline is the log's own [`LineSplit`], and
//! the diagnosis is the log's own [`replay`] — the same validator
//! `open_log` runs, so a log this calls [`RepairOutcome::Clean`] is a log
//! `open_log` opens, by construction.
//!
//! # A refusal is an outcome, never an error
//!
//! §6.1's code list is closed and versioned, and it has no code for "not
//! repairable", so every refusal is a [`RepairOutcome`] the caller prints
//! and decides about. Only a fault repair cannot classify propagates as
//! [`StoreError`]: the filesystem's own refusal, a contended lock, a
//! format version this build does not read (a migration question, §4), and
//! a snapshot document that does not parse. For that last one the `Err` is
//! byte-identical to `open_log`'s own, which is the agreement this
//! primitive exists to keep; truncating a log cannot fix a snapshot, and
//! [`RepairOutcome::Unrepairable`]'s offset is a *log* position by
//! contract, so that fault is never reported through it.
//!
//! # The classification order is the argument
//!
//! [`replay`]'s fault is classified structurally — on the variant and the
//! offset, never on a `detail` string, which `StoreError` documents as not
//! the contract — in this order:
//!
//! 1. **Pair divergence first.** A snapshot whose `log_offset` is past the
//!    log's end is [`RepairOutcome::Diverged`], and it is checked before
//!    the torn-tail test because an out-of-band truncation can present
//!    *both* and the divergence dominates: truncating the log would not
//!    make that pair consistent, so a repair that "succeeded" would have
//!    fixed nothing. This is `compact`'s (3)–(4) window (see
//!    [`crate::Store::compact`]'s crash table), whose remedy is re-basing
//!    the snapshot — not built here, and unreachable in M1, where nothing
//!    calls `compact()`.
//! 2. **Then the truncation point.** A fault with no offset (a semantic
//!    disagreement between two files) has nothing to truncate from, and a
//!    fault at offset 0 would truncate to an empty log, which is its own
//!    `corrupt_store` — both [`RepairOutcome::Unrepairable`].
//! 3. **Then the torn tail itself**, decided by the log's own splitter: the
//!    fault's offset is the *final* line's start and that line is
//!    unterminated. Anything else is a refusal naming §4.3's backup, and
//!    the two states say so differently — a fault in an *interior* line,
//!    whose bytes a crash cannot tear, and a fault in a final line that
//!    *is* terminated, which was therefore committed and is not a tear.
//!
//! Before a single byte is written the candidate prefix is replayed, so a
//! repair never truncates a log into one that still does not open: §8's
//! atomic-rejected-write discipline, applied to a repair. A dry run runs
//! the same validation, because it makes the same promise a real run
//! keeps. The write is §4.3's — temp file in the store directory, `fsync`,
//! atomic rename — with no directory `fsync`, by the closed f16 decision
//! the `snapshot` module records. There is no append handle open during a
//! repair and a lock follows the inode while a later `open_log` opens by
//! path, so the rename orphans nothing.

use std::fs;
use std::io;
use std::path::Path;

use todo_domain::RepoSlug;

use crate::error::{LockTarget, StoreError, io_error, io_error_kind};
use crate::layout::{DataRoot, store_dir_missing};
use crate::lock::FileLock;
use crate::log::{LineSplit, replay};
use crate::snapshot::{load_snapshot, rename_into_place, temp_path_for, write_temp};

/// The io-fault context of a repair whose store directory is absent: the
/// same honest `NotFound` [`crate::JsonlStore::open_log`] reports for a
/// missing store, with the operation that missed it named.
const CONTEXT_REPAIR: &str = "repair the event log";
/// The io-fault context of the log read a repair diagnoses.
const CONTEXT_READ_LOG: &str = "read the event log for repair";
/// The subject the write halves name in their io-fault contexts, so a
/// report says which write failed without carrying a path.
const SUBJECT_REPAIRED_LOG: &str = "the repaired log";

/// How many characters of a torn line [`TornTail::line`] shows at most:
/// enough for a human to recognise the event, short enough that a tear of
/// any size — a whole log's bytes with no LF in them is one line — cannot
/// flood the report. The bound counts **characters**, the cut is on a
/// character boundary and is marked with [`PREVIEW_CUT`], so a preview is
/// valid UTF-8 and deterministic for a given tear whatever bytes it holds.
const MAX_PREVIEW_CHARS: usize = 200;
/// The marker appended to a preview that [`MAX_PREVIEW_CHARS`] cut.
const PREVIEW_CUT: char = '…';

/// §4.3's backup, named in every refusal so the report carries the way
/// out, not only the verdict.
const REMEDY_BACKUP: &str = "the remedy is §4.3's backup: copy the store \
directory while no writer holds the lock, and repair the copy by hand";

/// The refusal reasons, one per case of the classification order above.
/// Each must read true of the state it is reported for, because the cli
/// prints it as the rationale a human confirms a truncation against.
const REASON_NO_POSITION: &str =
    "the fault has no byte position, so there is nothing to truncate from";
const REASON_AT_ZERO: &str = "the fault is at byte 0, and truncating from it would leave an \
empty log, which is itself a corrupt_store";
const REASON_INTERIOR: &str = "the fault is in an interior line, and the interior of a log does \
not tear in a crash (§4.3), so truncating from it would drop committed events";
const REASON_COMMITTED_TAIL: &str = "the log's final line is terminated, so it was committed: \
§4.3's tear is an unterminated line, and dropping a committed event is not a crash repair";
/// The reason of `classify`'s two total fallbacks: `replay` positions a log
/// fault at a line start inside the log, so neither arm is reachable through
/// it today, and each still has to say something true rather than borrow
/// another case's reason.
const REASON_NOT_A_LINE: &str =
    "the reported offset is not a line start in this log, so there is no line to remove";
const REASON_PREFIX_FAULTY: &str = "truncating the torn line would not leave a log that opens";

/// What a torn last line looked like, before repair removed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TornTail {
    /// Absolute byte offset of the torn line's start — §4.3's repair point.
    pub offset: u64,
    /// Bytes the repair removes: the torn line's length (it has no LF).
    pub removed: u64,
    /// The line's text for a human deciding whether to truncate: lossy
    /// UTF-8, bounded — at most this module's `MAX_PREVIEW_CHARS`
    /// characters, cut on a character boundary and marked with a trailing
    /// `…` when the line is longer, so a torn line of any size yields a
    /// bounded, deterministic report.
    pub line: String,
}

/// What a repair attempt found, and whether it wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairOutcome {
    /// The log replays clean. Nothing to repair, nothing written.
    Clean,
    /// §4.3's repairable case: an unterminated final line. `truncated`
    /// says whether this call wrote — `false` for a dry run, which reads
    /// and validates exactly what a real run would and writes nothing.
    TornTail {
        /// The torn line the repair removes.
        torn: TornTail,
        /// Whether the log was rewritten to exclude it.
        truncated: bool,
    },
    /// A fault truncation cannot fix — one with no byte position, an
    /// interior line, a final line that was committed rather than torn, a
    /// tear whose truncation point is 0, or one whose candidate prefix would
    /// not itself replay. `detail` says which, and names §4.3's backup as
    /// the remedy. Nothing written.
    Unrepairable {
        /// The log offset the fault was reported at, when it had one. This
        /// is a position in the log, never in a snapshot.
        offset: Option<u64>,
        /// Which refusal case this is, the engine's own report, and the
        /// remedy. Deterministic and path-free, like every detail this
        /// crate writes.
        detail: String,
    },
    /// The snapshot/log pair diverged: the snapshot's `log_offset` is past
    /// the log's end (compact's (3)–(4) window). The remedy is re-basing
    /// the snapshot, which this does not do and nothing in M1 can produce
    /// — reported, not written.
    Diverged {
        /// The offset the snapshot claims to cover up to.
        log_offset: u64,
        /// The log's actual byte length.
        log_len: u64,
    },
}

impl DataRoot {
    /// Attempts §4.3's torn-tail repair on one repo's store, under §8's
    /// lock.
    ///
    /// The lock is taken before the log is read and held until this
    /// returns, so a contended store is [`StoreError::Locked`] with
    /// nothing read and nothing written; the store directory is checked
    /// before the lock, because a directory that does not exist holds no
    /// lock file either and the honest report is the log's own `NotFound`.
    /// The registry is not consulted — the store directory is
    /// authoritative for that repo's data, exactly as in
    /// [`DataRoot::open`].
    ///
    /// With `dry_run`, nothing is written and the outcome is the one a real
    /// run would have produced *and kept*: the same validation runs, so a
    /// dry run never promises a truncation the real run would refuse.
    ///
    /// A refusal is `Ok(RepairOutcome)`, never an `Err` (module docs). The
    /// `Err` cases are faults repair cannot classify: [`StoreError::Io`],
    /// [`StoreError::Locked`], [`StoreError::VersionMismatch`], and a
    /// snapshot document that does not parse.
    ///
    /// This never re-bases a snapshot and never compacts: a
    /// [`RepairOutcome::Diverged`] pair is reported for a human to resolve.
    pub fn repair(&self, slug: &RepoSlug, dry_run: bool) -> Result<RepairOutcome, StoreError> {
        let log_path = self.log_path(slug);
        let lock_path = self.lock_path(slug);
        let snapshot_path = self.snapshot_path(slug);

        if store_dir_missing(&log_path) {
            return Err(io_error_kind(CONTEXT_REPAIR, io::ErrorKind::NotFound));
        }
        // §8's single writer, taken first and held to the end. The layout
        // always derives a slug, so `LockTarget::Unnamed` cannot occur
        // here.
        let _lock = FileLock::acquire(&lock_path, LockTarget::Repo(slug.clone()))?;

        let bytes = fs::read(&log_path).map_err(|error| io_error(CONTEXT_READ_LOG, &error))?;
        // The open path's own validator, so a `Clean` here and a successful
        // `open_log` there cannot disagree.
        let fault = match replay(&log_path, &bytes) {
            Ok(_) => return Ok(RepairOutcome::Clean),
            Err(fault) => fault,
        };
        classify(fault, &bytes, &log_path, &snapshot_path, dry_run)
    }
}

/// Classifies `replay`'s fault and, when it is §4.3's repairable one,
/// performs or rehearses the truncation (module docs for the order and the
/// argument).
fn classify(
    fault: StoreError,
    bytes: &[u8],
    log_path: &Path,
    snapshot_path: &Path,
    dry_run: bool,
) -> Result<RepairOutcome, StoreError> {
    // `replay` takes no lock and translates every domain rejection into a
    // positioned `CorruptStore`, so no other variant reaches here from it;
    // propagation keeps the arm total without asserting that, which a
    // future change could falsify.
    let StoreError::CorruptStore { offset, detail } = fault else {
        return Err(fault);
    };
    let log_len = bytes.len() as u64;

    // (1) Divergence dominates: checked before anything about the log's own
    // lines, because an out-of-band truncation can present both and
    // truncating would not fix the pair. The offset is read through the
    // crate's own loader, so this cannot drift from the parse `replay`
    // does; a snapshot that does not parse is propagated, never guessed at.
    if let Some(document) = load_snapshot(snapshot_path)?
        && document.log_offset > log_len
    {
        return Ok(RepairOutcome::Diverged {
            log_offset: document.log_offset,
            log_len,
        });
    }

    // (2) The truncation point.
    let Some(torn_at) = offset else {
        return Ok(refuse(None, REASON_NO_POSITION, &detail));
    };
    if torn_at == 0 {
        return Ok(refuse(Some(0), REASON_AT_ZERO, &detail));
    }

    // (3) The torn tail, by the log's own notion of a line: the fault's
    // offset is the final line's start and that line has no terminator. Two
    // separate refusals, because they are two different facts about the log
    // and the report is what a human confirms against.
    let offset = Some(torn_at as u64);
    let Some(line) = LineSplit::new(bytes).last() else {
        return Ok(refuse(offset, REASON_NOT_A_LINE, &detail));
    };
    // The truncation point is not the final line, so every committed event
    // between the two would be dropped.
    if line.offset != torn_at {
        return Ok(refuse(offset, REASON_INTERIOR, &detail));
    }
    // It is the final line, but a terminated one was committed and §4.3's
    // tear is unterminated: this is damage, not a crash window.
    if line.terminated {
        return Ok(refuse(offset, REASON_COMMITTED_TAIL, &detail));
    }
    // `replay` positions a log fault at a line start, so the offset is
    // within the buffer; `get` keeps that a fact rather than an assumption.
    let Some(prefix) = bytes.get(..torn_at) else {
        return Ok(refuse(offset, REASON_NOT_A_LINE, &detail));
    };

    // §8's atomic rejected write, applied to a repair: validate the
    // candidate before writing anything, dry run included.
    if let Err(prefix_fault) = replay(log_path, prefix) {
        return Ok(refuse(
            offset,
            REASON_PREFIX_FAULTY,
            &prefix_fault.to_string(),
        ));
    }

    let torn = TornTail {
        offset: torn_at as u64,
        removed: line.bytes.len() as u64,
        line: preview(line.bytes),
    };
    if dry_run {
        return Ok(RepairOutcome::TornTail {
            torn,
            truncated: false,
        });
    }
    let temporary = temp_path_for(log_path);
    write_temp(&temporary, prefix, SUBJECT_REPAIRED_LOG)?;
    rename_into_place(&temporary, log_path, SUBJECT_REPAIRED_LOG)?;
    Ok(RepairOutcome::TornTail {
        torn,
        truncated: true,
    })
}

/// A refusal: which case it is, the engine's own report (composed for the
/// human, never read back — the classification above is structural), and
/// §4.3's backup as the remedy.
fn refuse(offset: Option<u64>, reason: &str, underlying: &str) -> RepairOutcome {
    RepairOutcome::Unrepairable {
        offset,
        detail: format!("{reason}: {underlying}. {REMEDY_BACKUP}"),
    }
}

/// The bounded human-readable preview of a torn line: lossy UTF-8, cut at
/// [`MAX_PREVIEW_CHARS`] characters on a character boundary and marked
/// with [`PREVIEW_CUT`].
fn preview(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.chars().count() <= MAX_PREVIEW_CHARS {
        return text.into_owned();
    }
    let mut cut: String = text.chars().take(MAX_PREVIEW_CHARS).collect();
    cut.push(PREVIEW_CUT);
    cut
}
