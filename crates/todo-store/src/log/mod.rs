//! The JSONL engine (D1): an append-only event log with a versioned
//! header, replayed into an in-memory graph.
//!
//! §4.3's crash semantics, concretely: every event is one `write_all` of
//! a complete LF-terminated line to a handle opened `O_APPEND`, followed
//! by `sync_all`, so the torn window is at most the one event in flight.
//! On replay a line without its terminator is torn and is
//! [`StoreError::CorruptStore`] whatever its bytes parse to — a partial
//! write is not a committed event — and so is any other malformed line,
//! at any position: the interior of a log does not tear in a crash, so a
//! fault there means something else edited or damaged it. Nothing is
//! skipped and nothing is repaired (§4.3: "no automatic repair path
//! exists"); the error carries the offending line's **absolute** byte
//! offset, which is where [`crate::DataRoot::repair`] truncates from, and
//! the engine's position *within* that line rides in the `detail` text.
//!
//! The engine's own constructors — [`JsonlStore::create_log`] and
//! [`JsonlStore::open_log`] — take a bare log path, and every store they
//! return is **locked**: §8's single writer holds an advisory lock on
//! `<path>.lock` (the layout's one lock-path rule) for the store's whole
//! lifetime, released by its `Drop`. There is no unlocked store and no
//! opt-out. The D4 layout lives above them: [`crate::Store::create`] and
//! [`crate::Store::open`] resolve `repos/<slug>/event-log.jsonl` under a
//! [`crate::DataRoot`] and delegate here, and [`crate::DataRoot::register`]
//! adds the registry index around the same call.
//!
//! Durability note: an append that fails after the domain accepted the
//! mutation leaves memory one event ahead of disk, which is §4.3's crash
//! window with the crash replaced by an io error. That state is
//! session-fatal by design — the caller reopens, and replay from disk
//! drops the unpersisted mutation (see [`crate::Store`]).
//!
//! The engine's surface is split (U75): this module holds the mechanics
//! alone — the constructors, the append discipline, the snapshot-aware
//! replay, the header migration — and the [`crate::Store`] trait's
//! implementation (the mutations, `snapshot`/`compact`, §15's grant
//! path) lives in [`store_impl`].

mod store_impl;

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use todo_domain::{Graph, Repo, RepoSlug};

use crate::FORMAT_VERSION;
use crate::PREDECESSOR_VERSION;
use crate::codec::{VersionProbe, from_bytes, is_readable, to_bytes};
use crate::error::{LockTarget, StoreError, io_error, io_error_kind};
use crate::event::{Event, LogHeader, replay_step};
use crate::layout::{
    lock_path_for, snapshot_path_for, store_dir_missing, store_target_from_log_path,
};
use crate::lock::{CONTEXT_STORE, FileLock};
use crate::snapshot::{SnapshotDocument, load_snapshot};

/// The line terminator, and the only byte the log's line discipline
/// attaches meaning to (§4.1: every mutation is one JSON line).
const LINE_FEED: u8 = b'\n';

/// The io-fault contexts of the two bare-path constructors, pinned by
/// tests: a missing log reports the operation that missed it.
const CONTEXT_CREATE_LOG: &str = "create the event log";
const CONTEXT_OPEN_LOG: &str = "open the event log";
/// The io-fault context of the post-compaction reopen: the append handle
/// must follow the log's new inode (t11's constraint — the lock protects
/// the `.lock` inode, the handle writes the log inode).
const CONTEXT_REOPEN_LOG: &str = "reopen the event log after compaction";
/// The io-fault context of the pre-compaction read: the old log's bytes
/// are what the event count and the covered region are measured over.
const CONTEXT_READ_LOG: &str = "read the event log for compaction";
/// The io-fault contexts of the in-place header migration (§15): a v1
/// log opened for writing has its header rewritten before any append.
const CONTEXT_MIGRATE_OPEN: &str = "rewrite the event log's header";
const CONTEXT_MIGRATE_FLUSH: &str = "flush the event log's header to disk";

/// The corruption details, pinned by `tests/log/corrupt.rs`:
/// deterministic, and never carrying a host path — a §4.3 report is all a
/// repair gets, so it says enough without leaking where the store lives.
const DETAIL_EMPTY_LOG: &str = "the log is empty: its first line must be the format-version header";
const DETAIL_NO_EVENTS: &str = "the log holds no events: its first event must be repo_register";
const DETAIL_TORN_LINE: &str =
    "the last line has no terminating newline: a partial write is not a committed event (§4.3)";
const DETAIL_BAD_HEADER: &str = "the log's first line is not a format-version header";
const DETAIL_BAD_EVENT: &str = "the line is not a parseable event";
const DETAIL_BAD_SEED: &str = "the log's first event must be repo_register, found ";

/// One repository's store: its event log, the graph replayed from it, and
/// §8's advisory lock on the log.
///
/// The [`crate::Store`] trait is the whole surface a caller above this crate
/// needs; this struct is the D1 engine behind it and is only named
/// directly to create or open a log by path.
///
/// Every store is locked — there is no unlocked store and no `Option`
/// around the lock. The lock is acquired **before** the log is created or
/// read and released by this value's `Drop`, so the guard's lifetime is
/// the store's and a store cannot outlive its own write exclusivity.
#[derive(Debug)]
pub struct JsonlStore {
    /// The log's path as constructed with: the snapshot path is derived
    /// from it (the layout's suffix rule), and compaction renames and
    /// reopens exactly this path.
    path: PathBuf,
    /// The log file, kept open for appends: §8's single writer.
    file: File,
    /// §8's advisory lock, held for this store's lifetime. The leading
    /// underscore is the point: nothing reads it, dropping it is the
    /// release.
    _lock: FileLock,
    /// The graph every read answers from, kept current by every append.
    graph: Graph,
    /// The root's config document, when this store lives in the D4
    /// layout — the rewards source, read fresh at each closure (§15) so
    /// a config change takes effect without a reopen. `None` for a
    /// bare-path store: no root, no config, the inert store.
    config_path: Option<PathBuf>,
}

impl JsonlStore {
    /// Creates a log at `path` holding one repository, and returns the
    /// locked store over it.
    ///
    /// The lock on `<path>.lock` is acquired first, so two racing creates
    /// of one path cannot both proceed: the loser sees
    /// [`StoreError::Locked`], not a torn file. The file must not exist:
    /// creation is `create_new`, so an existing log is an io error and
    /// never a truncation (§4: corruption and loss are named errors, not
    /// silent ones). The header line and the `repo_register` seed are one
    /// `write_all` followed by `sync_all`, so a crash during creation
    /// leaves either a log [`JsonlStore::open_log`] rejects as corrupt —
    /// an empty or torn file — or a whole seeded one, never a
    /// half-created store that opens. The seed's `ts` is `repo`'s
    /// registration time.
    ///
    /// On any error the lock guard drops, so a failed create leaves no
    /// lock held. It may leave a zero-byte `<path>.lock` beside a log that
    /// does not exist — accepted and harmless, since a lock file's bytes
    /// mean nothing (the `lock` module documents why). A path whose store
    /// directory does not exist reports the honest `create the event log:
    /// NotFound`, never a lock fault — the directory holds no lock file to
    /// create either.
    pub fn create_log<P: AsRef<Path>>(path: P, repo: Repo) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if store_dir_missing(path) {
            return Err(io_error_kind(CONTEXT_CREATE_LOG, io::ErrorKind::NotFound));
        }
        // Lock first: the whole create runs under it.
        let lock = FileLock::acquire(&lock_path_for(path), LockTarget::Repo(repo.slug.clone()))?;

        let mut file = OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(path)
            .map_err(|error| io_error(CONTEXT_CREATE_LOG, &error))?;

        let graph = Graph::new(repo.clone());
        let event = Event::RepoRegister {
            ts: repo.registered_at.clone(),
            repo,
        };
        let mut bytes = to_bytes(&LogHeader::current())?;
        bytes.push(LINE_FEED);
        bytes.extend_from_slice(&to_bytes(&event)?);
        bytes.push(LINE_FEED);

        file.write_all(&bytes)
            .map_err(|error| io_error("write the event log's header and seed", &error))?;
        file.sync_all()
            .map_err(|error| io_error("flush the event log to disk", &error))?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            _lock: lock,
            graph,
            config_path: None,
        })
    }

    /// Opens the log at `path`, replays it into the graph it describes
    /// (§4.1), and returns the locked store. The handle stays open for
    /// appends.
    ///
    /// Replay is §4.3's: when a snapshot exists beside the log
    /// (`<path>.snapshot`, the layout's suffix rule) it is the recovery
    /// unit — its graph is restored and only the log tail from its
    /// `log_offset` onward is applied; without one, the whole log is
    /// replayed line by line. The snapshot is used automatically and
    /// never written here: nothing below the binary layer decides *when*
    /// to snapshot (§8's no-clock rule; [`crate::Store::snapshot`] and
    /// [`crate::Store::compact`] are explicit calls).
    ///
    /// The lock on `<path>.lock` is acquired before the log is read, so a
    /// contended open is [`StoreError::Locked`] and never a partial read.
    /// The lock's target slug is derived from the store directory only if
    /// the open *is* contended, so a bare path outside the D4 layout — a
    /// corruption fixture's temp file — still opens, replays and reports
    /// its engine faults exactly as [`crate::Store::open`] over a real
    /// store does (the `lock` module documents the lazy derivation).
    ///
    /// A missing file is an io error, never an empty store — and a
    /// missing store *directory* reports the same honest
    /// `open the event log: NotFound` rather than a lock fault, because a
    /// directory that does not exist holds neither the log nor a lock file
    /// ([`crate::DataRoot::open`]'s answer for a slug that was never registered).
    /// A log whose header declares another format version is
    /// [`StoreError::VersionMismatch`] — except the predecessor
    /// (§15's strict superset), which replays identically and whose
    /// header this open migrates in place before any append can follow
    /// it; a log that is malformed anywhere —
    /// a torn last line, garbage in the interior, a seed out of place, an
    /// event the domain rejects, an id that is not the next of its kind —
    /// is [`StoreError::CorruptStore`] carrying the offending line's
    /// absolute byte offset. A snapshot that contradicts its log —
    /// another repo's slug, a `log_offset` beyond the log's end, before
    /// the seed's end or off a line boundary — is likewise a named error
    /// (see `replay_from_snapshot`). On any of these the lock guard
    /// drops: a failed open never leaves a stale lock, which is what lets
    /// a caller open the same corrupt path twice and get the same
    /// positioned error both times.
    pub fn open_log<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if store_dir_missing(path) {
            return Err(io_error_kind(CONTEXT_OPEN_LOG, io::ErrorKind::NotFound));
        }
        let lock_path = lock_path_for(path);
        // Lock first: the whole open runs under it. The target is derived
        // lazily, on contention only.
        let lock = FileLock::acquire_reporting(&lock_path, CONTEXT_STORE, || {
            store_target_from_log_path(path)
        })?;

        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .map_err(|error| io_error(CONTEXT_OPEN_LOG, &error))?;
        // A freshly opened handle reads from 0, and `O_APPEND` moves the
        // write position to the end on every write regardless of where a
        // read left it — so one handle serves both replay and append.
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| io_error("read the event log", &error))?;
        let (graph, declared_version) = replay(path, &bytes)?;
        // §15's in-place header migration: a v1 log opened for writing
        // becomes a v2 log before any v2 event can follow the v1 header
        // — the version gate is what protects an older binary from
        // estimate-bearing lines, so the header moves first. The write
        // is length-preserving (both spellings are
        // `{"format_version":N}\n`), so no other byte, no snapshot
        // `log_offset` and no line offset moves with it; a torn write
        // here is a named `corrupt_store` on the next open, never a
        // silent wrong graph.
        if declared_version == PREDECESSOR_VERSION {
            migrate_log_header(path)?;
        }
        Ok(Self {
            path: path.to_path_buf(),
            file,
            _lock: lock,
            graph,
            config_path: None,
        })
    }

    /// The root's config document this store reads rewards from — the
    /// layout's to set (it knows the root), never a caller above the
    /// crate's: the config is the root's, and the store's source is the
    /// layout's business. Bare-path stores keep `None`: the inert store.
    pub(crate) fn attach_config_path(&mut self, config_path: PathBuf) {
        self.config_path = Some(config_path);
    }

    /// §4.3's append discipline: one `write_all` of a complete line under
    /// `O_APPEND`, then `sync_all`. The caller has already applied the
    /// event to the graph, so a failure here is the documented
    /// session-fatal state (see the module docs).
    fn append(&mut self, event: &Event) -> Result<(), StoreError> {
        let mut line = to_bytes(event)?;
        line.push(LINE_FEED);
        self.file
            .write_all(&line)
            .map_err(|error| io_error("append to the event log", &error))?;
        self.file
            .sync_all()
            .map_err(|error| io_error("flush the event log to disk", &error))
    }

    /// The log's byte length, measured through the append handle's own
    /// descriptor (`fstat`): the size the kernel reports for the very
    /// inode every append lands on, so it cannot drift from the bytes on
    /// disk the way a length tracked in the struct could — the handle is
    /// the second source of truth this avoids, and §8's single writer
    /// means nothing else appends between the measurement and the use.
    fn log_len(&self) -> Result<u64, StoreError> {
        self.file
            .metadata()
            .map(|metadata| metadata.len())
            .map_err(|error| io_error("measure the event log", &error))
    }

    /// Re-points the append handle at the log's current inode after a
    /// rename replaced it (t11's constraint: the lock protects the
    /// `.lock` **inode** while this handle writes the **log inode**, so a
    /// handle kept across the log's rename would append to the
    /// renamed-away file forever — the orphaned inode, invisible to every
    /// later open).
    ///
    /// The old descriptor drops as the field is overwritten, which is the
    /// close; the new one opens with `open_log`'s flags. If this open
    /// fails the store is in the documented session-fatal io state with
    /// an added edge: the handle still references the orphaned inode, so
    /// any append a caller risked anyway would land in the old log, not
    /// the new one. The caller must reopen — the same contract a failed
    /// append has (`store.rs`), stated here because the orphan makes it
    /// load-bearing.
    fn reopen_log(&mut self) -> Result<(), StoreError> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| io_error(CONTEXT_REOPEN_LOG, &error))?;
        self.file = file;
        Ok(())
    }
}

/// Rewrites a predecessor log's header line in place (§4, §15): the v1
/// and v2 spellings are both `{"format_version":N}\n` — the same byte
/// length — so the write moves no other byte, and every snapshot
/// `log_offset` and every line offset stays valid. The write-mode handle
/// starts at offset 0 (no `O_APPEND`), writes the one line, and syncs;
/// the append handle is untouched and still points at the same inode.
/// A torn write is the one crash window, and it is a named
/// `corrupt_store` on the next open — never a silent wrong graph.
fn migrate_log_header(path: &Path) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|error| io_error(CONTEXT_MIGRATE_OPEN, &error))?;
    let mut line = to_bytes(&LogHeader::current())?;
    line.push(LINE_FEED);
    file.write_all(&line)
        .map_err(|error| io_error(CONTEXT_MIGRATE_OPEN, &error))?;
    file.sync_all()
        .map_err(|error| io_error(CONTEXT_MIGRATE_FLUSH, &error))
}

/// Replays log bytes into the graph they describe (§4.1): the header, the
/// seed, then — when a snapshot sits beside the log — the restored graph
/// plus only the tail after the snapshot's `log_offset` (§4.3: "replay
/// reads the latest snapshot, then the log tail after it"); without one,
/// every event in order. Returns the graph and the header's **declared**
/// version, so a write-capable caller can migrate a predecessor header
/// (§15) while the lock-free readers simply ignore it.
///
/// The whole log is read into one buffer and split once: at §4.1's scale —
/// 10^3 to 10^5 nodes — the buffer is the cheaper machinery, and holding
/// every byte in one place is what lets a line's offset be absolute rather
/// than relative to whatever a streaming reader had buffered. A snapshot
/// narrows what has to be *applied*, not what has to be read: the offset
/// checks below need the covered bytes to prove the boundary.
///
/// The order of the header's own checks is deliberate — probe, version,
/// strict parse, terminator (see [`VersionProbe`], and `check_header`) —
/// so a log written by a newer build reports
/// [`StoreError::VersionMismatch`] even when its header line is the only
/// thing in the file or carries fields this build has never seen: that
/// is a migration question (§4), not corruption. The predecessor version
/// is the one accepted reading (§15's strict superset); anything older
/// is damage this build never wrote and reports as the mismatch. The
/// log's checks all run before the snapshot is consulted, so a log that
/// is itself malformed reports its own fault at its own offset even when
/// a snapshot exists.
pub(crate) fn replay(path: &Path, bytes: &[u8]) -> Result<(Graph, u32), StoreError> {
    let mut lines = LineSplit::new(bytes);

    let Some(header) = lines.next() else {
        return Err(corrupt(0, DETAIL_EMPTY_LOG));
    };
    let declared_version = check_header(&header)?;

    let Some(seed) = lines.next() else {
        return Err(corrupt(0, DETAIL_NO_EVENTS));
    };
    let event = parse_line(&seed)?;
    let Event::RepoRegister { repo, .. } = &event else {
        return Err(corrupt(
            seed.offset,
            format!("{DETAIL_BAD_SEED}{}", event.kind()),
        ));
    };
    // `parse_line` committed the seed, so it is terminated: the seed's
    // end is the first byte a snapshot may cover, and the first byte a
    // tail may start at.
    let seed_end = seed.offset + seed.bytes.len() + 1;

    let graph = match load_snapshot(&snapshot_path_for(path))? {
        None => {
            let mut graph = Graph::new(repo.clone());
            for line in lines {
                let event = parse_line(&line)?;
                replay_step(&mut graph, &event)
                    .map_err(|fault| corrupt(line.offset, fault.to_string()))?;
            }
            graph
        }
        Some(document) => replay_from_snapshot(document, &repo.slug, seed_end, bytes, lines)?,
    };
    Ok((graph, declared_version))
}

/// §4.3's replay over a snapshot: the document reconciles itself against
/// the log (slug and the three offset checks, all in `snapshot.rs`), is
/// restored to a graph, and only the uncovered tail is applied. The
/// covered region `[seed_end, log_offset)` is skipped without being
/// parsed: the snapshot *is* its record, and what a compaction dropped
/// from the log stays dropped (§4.1's truncation bounds both replay and
/// retention — the `snapshot` module docs state the reading).
fn replay_from_snapshot(
    document: SnapshotDocument,
    seed_slug: &RepoSlug,
    seed_end: usize,
    bytes: &[u8],
    lines: LineSplit<'_>,
) -> Result<Graph, StoreError> {
    let offset = document.reconcile_with_log(seed_slug, seed_end, bytes.len() as u64, bytes)?;
    let mut graph = document.restore()?;
    for line in lines {
        if line.offset < offset {
            continue;
        }
        let event = parse_line(&line)?;
        replay_step(&mut graph, &event).map_err(|fault| corrupt(line.offset, fault.to_string()))?;
    }
    Ok(graph)
}

/// Checks the header line: probe, version, strict parse, committed
/// (terminated) — [`VersionProbe`]'s ordering, the registry's and the
/// snapshot's own. Returns the declared version. The lenient probe is
/// what lets a header from a *newer* format report
/// [`StoreError::VersionMismatch`] even when it carries fields this
/// build has never seen: that is a migration question (§4), not
/// corruption. The predecessor version is the one other accepted
/// reading (§15's strict superset — a v1 log replays identically, and
/// the write-capable open migrates its header in place); anything older
/// is a version this build never wrote and reports as the mismatch. The
/// strict [`LogHeader`] parse then rejects an unknown field at any
/// readable version, which is f14's damage — `deny_unknown_fields`
/// with the registry and snapshot documents as the precedent.
fn check_header(line: &LogLine<'_>) -> Result<u32, StoreError> {
    let probe: VersionProbe = from_bytes(line.bytes)
        .map_err(|error| corrupt(line.offset, prefixed(DETAIL_BAD_HEADER, &error)))?;
    if !is_readable(probe.format_version) {
        return Err(StoreError::VersionMismatch {
            found: probe.format_version,
            expected: FORMAT_VERSION,
        });
    }
    let _: LogHeader = from_bytes(line.bytes)
        .map_err(|error| corrupt(line.offset, prefixed(DETAIL_BAD_HEADER, &error)))?;
    if !line.terminated {
        return Err(corrupt(line.offset, DETAIL_TORN_LINE));
    }
    Ok(probe.format_version)
}

/// Reads one event line: the torn-tail check first (§4.3 — an
/// unterminated last line was never committed, whatever its bytes parse
/// to), then the parse.
fn parse_line(line: &LogLine<'_>) -> Result<Event, StoreError> {
    if !line.terminated {
        return Err(corrupt(line.offset, DETAIL_TORN_LINE));
    }
    from_bytes(line.bytes).map_err(|error| corrupt(line.offset, prefixed(DETAIL_BAD_EVENT, &error)))
}

/// One line of a log: its absolute byte offset, its bytes without the
/// terminator, and whether the terminator was there.
pub(crate) struct LogLine<'a> {
    /// Where the line starts in the file — the offset a repair truncates
    /// from (§4.3).
    pub(crate) offset: usize,
    /// The line's own bytes, terminator excluded.
    pub(crate) bytes: &'a [u8],
    /// Whether an LF followed them. `false` means the write was torn.
    pub(crate) terminated: bool,
}

/// Splits log bytes into [`LogLine`]s in file order.
///
/// The offset arithmetic lives here and nowhere else: a line reports the
/// position the splitter itself had consumed when it reached it, so the
/// offsets in §4.3's errors cannot drift from the split that produced
/// them.
pub(crate) struct LineSplit<'a> {
    /// The bytes not yet split.
    rest: &'a [u8],
    /// How many bytes of the input the yielded lines accounted for.
    consumed: usize,
}

impl<'a> LineSplit<'a> {
    /// A splitter over a whole log.
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            rest: bytes,
            consumed: 0,
        }
    }
}

impl<'a> Iterator for LineSplit<'a> {
    type Item = LogLine<'a>;

    fn next(&mut self) -> Option<LogLine<'a>> {
        if self.rest.is_empty() {
            return None;
        }
        let (bytes, terminated) = match self.rest.iter().position(|byte| *byte == LINE_FEED) {
            Some(index) => {
                let (line, tail) = self.rest.split_at(index);
                // `tail` begins at the newline the position just found, so
                // the prefix is there by construction; `unwrap_or_default`
                // keeps the drop total rather than asserting it.
                self.rest = tail.strip_prefix(&[LINE_FEED]).unwrap_or_default();
                (line, true)
            }
            None => {
                let line = self.rest;
                self.rest = &[];
                (line, false)
            }
        };
        let line = LogLine {
            offset: self.consumed,
            bytes,
            terminated,
        };
        self.consumed += bytes.len() + usize::from(terminated);
        Some(line)
    }
}

/// A positioned [`StoreError::CorruptStore`] — the shape every §4.3
/// report takes.
fn corrupt(offset: usize, detail: impl Into<String>) -> StoreError {
    StoreError::CorruptStore {
        offset: Some(offset),
        detail: detail.into(),
    }
}

/// A context prefix plus the engine's own message, which carries its
/// line-relative position ("… at byte N") when it has one.
fn prefixed(context: &str, error: &StoreError) -> String {
    match error {
        StoreError::CorruptStore { detail, .. } => format!("{context}: {detail}"),
        other => format!("{context}: {other}"),
    }
}
