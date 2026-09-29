//! The snapshot document (§4.1, §4.3): the recovery unit that bounds
//! replay — "replay reads the latest snapshot, then the log tail after
//! it".
//!
//! One compact JSON document plus exactly one trailing LF, written
//! temp-file + `fsync` + atomic rename in the store directory, so a
//! snapshot is never torn (§4.3) and the previous one stays loadable
//! until the rename lands. The file is `<log path>.snapshot` — t11's
//! derivation rule extended, a pure suffix append, so two logs in one
//! directory cannot share a snapshot — and it gets **no lock file of its
//! own**: an atomically rewritten file is never its own lock target
//! (`lock.rs`), and the store's `event-log.jsonl.lock` already covers it.
//!
//! Reading follows t11's registry ordering: a lenient version probe
//! **before** the strict parse, so a document written by a newer build is
//! [`StoreError::VersionMismatch`] — a migration question (§4) — even
//! when its other fields are unknown here, and `deny_unknown_fields` at
//! the current version, where an unknown field can only be damage. A
//! malformed document is [`StoreError::CorruptStore`] carrying the
//! codec's offset, which over a whole-file read *is* absolute.
//!
//! # What a snapshot holds, and what compaction therefore drops
//!
//! The document is the whole graph — repo node, every node array in
//! ascending id order, both edge sets, and the §3 allocation counters —
//! plus the `log_offset` the covered events end at. The node arrays and
//! the `blocks` edges reuse the add-events' serde and
//! [`BlocksEdgeDto`], so a node or an edge has **one spelling in both
//! files** — and `log_offset` couples this format to `event_log.v1`'s
//! exact bytes: an event-schema change moves two goldens.
//!
//! [`crate::Store::compact`] re-emits the log as `header + seed` and
//! re-bases the snapshot onto it, which **drops the covered events**:
//! the original `repo_register` bytes and any `repo_set_path`/
//! `repo_set_name` history inside the covered region leave the live log
//! (the re-emitted seed carries the snapshot's current repo node). §4's
//! human-auditability preference is a property of the *format* — its own
//! clause says "a format a human can read in an editor keeps that
//! property" — retention is bounded by §4.1's truncation, and the
//! archive story is §4.3's backup: copy the store directory while no
//! writer holds the lock.
//!
//! §7 carries the strongest counter-text — "The server records the
//! change as a mutation like any other — the history of a repo's
//! locations is part of the audit trail" — and the reconciliation is
//! this: §7 guarantees a path move is *recorded as a mutation*, its own
//! audited `repo_set_path` line, rather than silently overwriting a
//! field, which the event log does; §4.1's truncation then bounds how
//! long the **live** log retains any event, and §4.3's backup is the
//! archive. That reading is signed, not crate-local: §4.3 (U10, resolved
//! 2026-09-29) states it as the contract — compaction drops as landed
//! and the backup is therefore the archive — and nothing in v1 calls
//! `compact` (§6 lists no tool for it and t13 ships no `compact`
//! subcommand), so no history is destroyed by the code as landed. The
//! same resolution split f14: the `deny_unknown_fields` this document
//! and the registry carry has landed on the log header too
//! (deserialize-only, so no golden moves), while the persistent
//! compaction marker stays deferred until something actually compacts —
//! its only job is making a compacted store distinguishable from a fresh
//! one, and a golden-moving format change spent with no beneficiary is
//! the wrong trade (§4.3).
//!
//! # After a compaction the pair is the unit
//!
//! The log and its snapshot are **one unit**: a compacted log's events
//! live only in the document, and once appends resume its tail's ids do
//! not start at 1, so the log alone is not replayable — opening it
//! without the snapshot is a named `corrupt_store`. Crashes cannot break
//! the pair: [`crate::Store::compact`] snapshots *before* it truncates,
//! and every crash point is a correct open or a named error — the table
//! in that method's doc is the argument. The one crash window, between
//! the log's rename and the snapshot's re-base, leaves the short log
//! beside the covering snapshot whose `log_offset` now exceeds it: reopen
//! reports "beyond the log's end" naming both numbers, and the live
//! snapshot still holds the whole graph, so t13's repair recovers by
//! re-basing the offset.
//!
//! The residual case is **not crash-reachable** and stays separate: the
//! out-of-band deletion of a compacted store's snapshot while its tail is
//! still empty replays the short log as a bare repo graph,
//! indistinguishable from a freshly created store — §4's "corruption is a
//! named error" cannot catch it, because nothing on disk claims the
//! snapshot existed. t13's inspector is the only vantage positioned to
//! flag that pair divergence; closing it needs a persistent compaction
//! marker — a log-header field (f14's decision), a signed-format change
//! moving `event_log.v1.golden` — so it is documented, not built.
//!
//! # Durability: no directory fsync (f16, closed by decision)
//!
//! Neither the registry's rename (t11) nor the renames here are followed
//! by a directory-fd `fsync`. §4.3's signed text specifies temp-file +
//! `fsync` + atomic rename and nothing more; on this host's platform a
//! directory-fd `fsync` does not actually give metadata durability (that
//! is `F_FULLFSYNC`, a heavier and platform-specific commitment this
//! project has not made); and the failure mode is bounded and named — a
//! lost rename leaves one of two states. Either the previous file
//! survives, which is a correct open: the old registry or old snapshot
//! loads, and where there was no previous file at all the document is
//! simply absent, which each reader already handles (a missing registry
//! loads empty, a missing first snapshot replays the whole log). Or the
//! rename landed and its *pair* did not — the state the amended
//! `compact()` order added, where a lost step-(4) re-base leaves the
//! *previous* covering snapshot beside the *new short* log, which is the
//! named (3)–(4) window: `log_offset` beyond the log's end, a
//! `corrupt_store` recoverable by re-basing the snapshot (see
//! [`crate::Store::compact`]'s table). Never a silent wrong graph.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use todo_domain::{
    Counters, Decision, Doc, DomainError, Epic, Graph, Repo, RepoSlug, Task, TaskId,
};

use crate::FORMAT_VERSION;
use crate::codec::{VersionProbe, from_bytes, to_bytes};
use crate::error::{StoreError, io_error};
use crate::event::BlocksEdgeDto;

/// The snapshot's on-disk document: the version header, the log offset
/// the covered events end at, and the whole graph, in the field order
/// the golden pins (§4's versioning requirement; §8's canonical
/// serialization — declaration order *is* byte order, and every array is
/// in its accessor's documented total order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SnapshotDocument {
    /// The [`FORMAT_VERSION`] this document was written with; a
    /// different value is [`StoreError::VersionMismatch`], reported
    /// before the document is read strictly (module docs).
    pub(crate) format_version: u32,
    /// The log's byte length when this snapshot was written: the offset
    /// at which events the snapshot does *not* cover begin. Replay
    /// applies the log's lines from here onward and skips — without
    /// re-validating — the covered region before it.
    pub(crate) log_offset: u64,
    /// The full current repo node — as mutated, not as registered; the
    /// log's seed holds the registration and only the slug is stable
    /// between the two (§7's mutables).
    pub(crate) repo: Repo,
    /// The §3 allocation counters: the highest number issued per kind,
    /// including numbers burned above the highest live id.
    pub(crate) counters: CountersDto,
    /// Every epic, ascending by id.
    pub(crate) epics: Vec<Epic>,
    /// Every task, ascending by id.
    pub(crate) tasks: Vec<Task>,
    /// Every decision, ascending by id.
    pub(crate) decisions: Vec<Decision>,
    /// Every doc, ascending by id.
    pub(crate) docs: Vec<Doc>,
    /// Every `blocks` edge in [`Graph::blocks_edges`]' total order, in
    /// the log's own edge spelling — one DTO for both files.
    pub(crate) blocks: Vec<BlocksEdgeDto>,
    /// Every `relates` annotation in [`Graph::relates_pairs`]' ascending
    /// canonical (lower, higher) order. A pair serializes as a
    /// two-element array of id strings: `[["camera:U1","camera:U2"]]`.
    /// The read side goes through [`relates_from_arrays`] by choice, not
    /// necessity (U12): the bytes are the derived impl's, but the
    /// rejection detail is better (see that function).
    #[serde(deserialize_with = "relates_from_arrays")]
    pub(crate) relates: Vec<(TaskId, TaskId)>,
}

/// Reads the `relates` wire shape: an array of two-element arrays of id
/// strings.
///
/// Once an engine workaround, now a choice (U12): `dacode-json` grew
/// tuple support at U11 (`c4aebe9`), so the derived impl reads these
/// bytes — but it rejects a 3-element entry as serde's `invalid length
/// 3, expected a tuple of size 2`, where this helper says "a relates
/// entry is a two-element array of task ids", which is what t13's
/// `repair` shows a human. Reading the same bytes as `Vec<Vec<TaskId>>`
/// and narrowing here keeps the wire shape the format specifies, an
/// inner array that is not a pair rejected as the corrupt document it
/// is, and the diagnostic actionable. The house rule (`_rust.md`:
/// named-field structs, never tuples, in a persisted schema) names
/// `relates` as the format's one positional shape, kept for byte
/// compatibility — `snapshot.v1.golden` does not move.
fn relates_from_arrays<'de, D>(deserializer: D) -> Result<Vec<(TaskId, TaskId)>, D::Error>
where
    D: Deserializer<'de>,
{
    let arrays = Vec::<Vec<TaskId>>::deserialize(deserializer)?;
    arrays
        .into_iter()
        .map(|pair| {
            let [a, b]: [TaskId; 2] = pair
                .try_into()
                .map_err(|_| D::Error::custom(DETAIL_RELATES_PAIR))?;
            Ok((a, b))
        })
        .collect()
}

/// The rejection message of a `relates` entry that is not a pair.
const DETAIL_RELATES_PAIR: &str = "a relates entry is a two-element array of task ids";

/// The §3 counters as four named fields — the store-side DTO the domain's
/// [`Counters`] deliberately is not (`Graph`'s doc: the persisted shape is
/// the store's call, and `Counters` gets no serde). Named per kind so the
/// positional `Counters::from_highest` cannot be fed a transposition that
/// rides through the document unnoticed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CountersDto {
    /// The highest epic number issued.
    pub(crate) epic: u64,
    /// The highest task number issued.
    pub(crate) task: u64,
    /// The highest decision number issued.
    pub(crate) decision: u64,
    /// The highest doc number issued.
    pub(crate) doc: u64,
}

impl From<Counters> for CountersDto {
    fn from(counters: Counters) -> Self {
        Self {
            epic: counters.epic(),
            task: counters.task(),
            decision: counters.decision(),
            doc: counters.doc(),
        }
    }
}

impl SnapshotDocument {
    /// The document covering `graph` up to `log_offset` — the snapshot
    /// half of §4.1, built from the read accessors whose orders §8
    /// documents, so two runs over unchanged state are byte-identical.
    pub(crate) fn from_graph(graph: &Graph, log_offset: u64) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            log_offset,
            repo: graph.repo().clone(),
            counters: CountersDto::from(graph.counters()),
            epics: graph.epics().cloned().collect(),
            tasks: graph.tasks().cloned().collect(),
            decisions: graph.decisions().cloned().collect(),
            docs: graph.docs().cloned().collect(),
            blocks: graph.blocks_edges().into_iter().map(Into::into).collect(),
            relates: graph.relates_pairs(),
        }
    }

    /// Cross-checks this document against the log it claims to snapshot,
    /// and returns the validated `log_offset` as a `usize` — the byte the
    /// log's uncovered tail starts at (§4.3's replay: the snapshot, then
    /// the log tail after it).
    ///
    /// The checks run in a fixed order and each is a named
    /// [`StoreError::CorruptStore`]:
    ///
    /// - **slug** — against `seed_slug`, the log's `repo_register` seed.
    ///   Slugs only, never the whole repo node: the seed holds the repo
    ///   *as registered* while `repo_set_path`/`repo_set_name` mutate the
    ///   node, so a whole-node comparison false-positives on any store
    ///   that ever moved (§7).
    /// - **`log_offset <= log.len()`** — the covered region cannot exceed
    ///   the log that holds it. This is also `compact`'s one named crash
    ///   window, between the log's rename and the snapshot's re-base (the
    ///   (3)–(4) row of [`crate::Store::compact`]'s table): the short log
    ///   beside the covering snapshot, whose `log_offset` still points at
    ///   the old full log's end, lands here — named rather than silently
    ///   wrong, and recoverable because that snapshot holds the whole
    ///   graph.
    /// - **`log_offset >= seed_end`** — the seed builds the graph's scope
    ///   before the document is consulted, so it is never the snapshot's
    ///   to cover.
    /// - **line boundary** — `log[log_offset - 1]` must be an LF, for
    ///   every offset including the log's end, so an EOF that is not a
    ///   committed line's end is rejected too: a torn tail must surface
    ///   as the torn line it is, never hide behind an empty tail.
    ///
    /// All four report `offset: None`: they locate a disagreement
    /// *between* two files, not a position inside one where a repair
    /// would truncate (t9's convention — a semantic fault has no byte
    /// position), and the numbers ride in the detail instead.
    pub(crate) fn reconcile_with_log(
        &self,
        seed_slug: &RepoSlug,
        seed_end: usize,
        log_len: u64,
        log: &[u8],
    ) -> Result<usize, StoreError> {
        if &self.repo.slug != seed_slug {
            return Err(unpositioned(format!(
                "{DETAIL_SLUG}{}, but the log's seed registers {seed_slug}",
                self.repo.slug
            )));
        }
        let log_offset = self.log_offset;
        if log_offset > log_len {
            return Err(unpositioned(format!(
                "{DETAIL_BEYOND}: {log_offset} > {log_len} bytes"
            )));
        }
        // Lossless by the check above: the offset is at most the log's
        // own length.
        let offset = log_offset as usize;
        if offset < seed_end {
            return Err(unpositioned(format!(
                "{DETAIL_SEED}: {offset} < the seed's end at {seed_end}"
            )));
        }
        if log.get(offset - 1) != Some(&LINE_FEED) {
            return Err(unpositioned(format!(
                "{DETAIL_BOUNDARY}: no line ends at {offset}"
            )));
        }
        Ok(offset)
    }

    /// Restores the graph this document describes (§4.3's recovery
    /// unit).
    ///
    /// The nodes go through the domain's explicit-id inserts in document
    /// order — epics before tasks, so every task's parent exists — and
    /// **without** the log replay's sequential-id check: a snapshot
    /// legitimately holds cascade gaps, which is the whole reason the
    /// counters ride along. The counters are restored **last**, so the
    /// inserts' own raise-to-max runs first and the restore only adds the
    /// part burned *above* the highest live id (the domain's
    /// [`Graph::raise_counters_to`] never lowers).
    ///
    /// A snapshot is trusted to be internally consistent but not
    /// blindly: any domain rejection is [`StoreError::CorruptStore`]
    /// carrying the domain's message, and every live id's number must be
    /// at most its kind's counter **as the document states it** — the
    /// check reads the document's `counters`, never the graph's own,
    /// because the inserts' raise-to-max makes the graph's trivially
    /// cover every live id. Its placement before `raise_counters_to` is
    /// *not* load-bearing: both read the same document values and a
    /// raise adds no nodes, so max commutes and the verdict is the same
    /// on either side — it sits first only so a damaged document is
    /// rejected before the counters it contradicts are landed. A
    /// document whose counters do not cover its nodes is damage, named
    /// with the offending id. Both faults have no byte position (a
    /// semantic fault is not located by a parse offset), so `offset` is
    /// `None`.
    pub(crate) fn restore(self) -> Result<Graph, StoreError> {
        let mut graph = Graph::new(self.repo);
        for epic in self.epics {
            graph.insert_epic(epic).map_err(contradicted)?;
        }
        for task in self.tasks {
            graph.insert_task(task).map_err(contradicted)?;
        }
        for decision in self.decisions {
            graph.insert_decision(decision).map_err(contradicted)?;
        }
        for doc in self.docs {
            graph.insert_doc(doc).map_err(contradicted)?;
        }
        for edge in self.blocks {
            graph.link_blocks(edge.into()).map_err(contradicted)?;
        }
        for (a, b) in self.relates {
            graph.link_relates(a, b).map_err(contradicted)?;
        }
        check_counter_coverage(&graph, &self.counters)?;
        graph.raise_counters_to(Counters::from_highest(
            self.counters.epic,
            self.counters.task,
            self.counters.decision,
            self.counters.doc,
        ));
        Ok(graph)
    }
}

/// Reads and parses the snapshot at `path`: `None` when the file is
/// absent (a store without a snapshot replays its whole log — §4.3's
/// "the latest snapshot" is simply not there), the document when it is
/// there and parses, and a named error otherwise. A read fault other
/// than absence is the crate's io shape.
pub(crate) fn load_snapshot(path: &Path) -> Result<Option<SnapshotDocument>, StoreError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(CONTEXT_READ, &error)),
    };
    // The version probe runs on the lenient shape first (module docs).
    let probe: VersionProbe = from_bytes(&bytes).map_err(|error| corrupt(CONTEXT_PARSE, &error))?;
    if probe.format_version != FORMAT_VERSION {
        return Err(StoreError::VersionMismatch {
            found: probe.format_version,
            expected: FORMAT_VERSION,
        });
    }
    let document: SnapshotDocument =
        from_bytes(&bytes).map_err(|error| corrupt(CONTEXT_PARSE, &error))?;
    Ok(Some(document))
}

/// Writes `document` to `path` under §4.3's atomic-rename discipline: a
/// pid-suffixed temp file in the same directory (same filesystem, so the
/// rename is atomic; the pid keeps parallel writers from sharing a temp
/// name — the store lock already serializes them, the pid guards against
/// debris), `fsync`, then the rename. No directory `fsync` follows the
/// rename, by decision — the module docs record why (f16).
pub(crate) fn write_snapshot_atomic(
    path: &Path,
    document: &SnapshotDocument,
) -> Result<(), StoreError> {
    let mut bytes = to_bytes(document)?;
    bytes.push(LINE_FEED);
    let temporary = temp_path_for(path);
    write_temp(&temporary, &bytes, "the snapshot")?;
    rename_into_place(&temporary, path, "the snapshot")
}

/// Writes `bytes` to the temp file `temporary` and fsyncs it — the first
/// half of §4.3's discipline, kept apart from the rename because a
/// compaction interleaves the two per file (stage the new log's temp,
/// rename it, then stage the re-based snapshot's temp, rename it — the
/// order whose crash points [`crate::Store::compact`]'s table
/// enumerates), and every rename is preceded by its own temp's `fsync`,
/// so no reader ever meets a half-written file. `subject` names the file
/// in the io-fault contexts, so a report says which write failed without
/// carrying a path.
pub(crate) fn write_temp(temporary: &Path, bytes: &[u8], subject: &str) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temporary)
        .map_err(|error| io_error(&format!("write {subject}'s temporary file"), &error))?;
    file.write_all(bytes)
        .map_err(|error| io_error(&format!("write {subject}'s temporary file"), &error))?;
    file.sync_all()
        .map_err(|error| io_error(&format!("flush {subject} to disk"), &error))
}

/// The rename half of §4.3's discipline: `temporary` into `path`, atomic
/// within one directory. The previous file stays readable until this
/// lands.
pub(crate) fn rename_into_place(
    temporary: &Path,
    path: &Path,
    subject: &str,
) -> Result<(), StoreError> {
    fs::rename(temporary, path)
        .map_err(|error| io_error(&format!("rename {subject} into place"), &error))
}

/// The pid-suffixed temp path beside `path`: `<file name>.<pid>.tmp` in
/// the same directory — the spelling the layout decision pins for the
/// snapshot and the compacted log, and the registry's scheme without the
/// leading dot (both are consumed by the rename; the debris test pins
/// that neither survives).
pub(crate) fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "event-log.jsonl".to_owned());
    let temporary = format!("{file_name}.{}.tmp", std::process::id());
    match path.parent() {
        Some(parent) => parent.join(temporary),
        None => PathBuf::from(temporary),
    }
}

/// Every live id's number must be at most its kind's counter **as the
/// document states it**: the internal-consistency check of a restore.
/// The comparison is against the document's values, never the graph's —
/// the inserts' raise-to-max already makes the graph's counters cover
/// every live id, so only the document's own numbers can be falsified
/// (see [`SnapshotDocument::restore`]; the placement relative to
/// `raise_counters_to` is semantically neutral, max commutes). The kinds
/// are checked in §3's order and each in ascending id order, so the
/// first violation a damaged document can hold is always the one
/// reported.
fn check_counter_coverage(graph: &Graph, counters: &CountersDto) -> Result<(), StoreError> {
    for epic in graph.epics() {
        if epic.id.number() > counters.epic {
            return Err(uncovered("epic", counters.epic, &epic.id));
        }
    }
    for task in graph.tasks() {
        if task.id.number() > counters.task {
            return Err(uncovered("task", counters.task, &task.id));
        }
    }
    for decision in graph.decisions() {
        if decision.id.number() > counters.decision {
            return Err(uncovered("decision", counters.decision, &decision.id));
        }
    }
    for doc in graph.docs() {
        if doc.id.number() > counters.doc {
            return Err(uncovered("doc", counters.doc, &doc.id));
        }
    }
    Ok(())
}

/// A counter-coverage fault: deterministic, informative, path-free, and
/// naming the id (Decision 6's check).
fn uncovered(kind: &str, counter: u64, id: &impl std::fmt::Display) -> StoreError {
    unpositioned(format!(
        "the snapshot's {kind} counter holds {counter}, below its live id {id}"
    ))
}

/// A domain rejection of a snapshot value, as this crate reports it: the
/// semantic fault has no byte position (t9's convention), and the
/// domain's own message rides in the detail so a repair tool can say why
/// the document is wrong.
fn contradicted(error: DomainError) -> StoreError {
    unpositioned(format!("{DETAIL_CONTRADICTED}: {error}"))
}

/// An unpositioned [`StoreError::CorruptStore`]: the cross-checks and the
/// restore faults locate a fact, not a byte a repair would cut at.
fn unpositioned(detail: impl Into<String>) -> StoreError {
    StoreError::CorruptStore {
        offset: None,
        detail: detail.into(),
    }
}

/// The fixed detail prefix of a domain-rejected restore.
const DETAIL_CONTRADICTED: &str = "the snapshot contradicts itself";

/// The snapshot↔log cross-check details, pinned by
/// `tests/snapshot/corrupt.rs`: deterministic, informative, and never
/// carrying a host path — a slug is an id, not a path (§7).
const DETAIL_SLUG: &str = "the snapshot holds repo ";
const DETAIL_BEYOND: &str = "the snapshot's log_offset is beyond the log's end";
const DETAIL_SEED: &str = "the snapshot's log_offset does not cover the log's seed";
const DETAIL_BOUNDARY: &str = "the snapshot's log_offset does not land on a line boundary";

/// The fault contexts, pinned by tests: deterministic, and never
/// carrying a host path.
const CONTEXT_READ: &str = "read the snapshot";
const CONTEXT_PARSE: &str = "the snapshot is not a parseable document";

/// The line terminator the snapshot document ends with — the log's line
/// discipline, and the golden workflow's "exactly one trailing newline".
const LINE_FEED: u8 = b'\n';

/// A codec rejection as this crate reports it: the context prefix plus
/// the codec's own message, carrying the codec's offset through — over a
/// whole-file read that offset is absolute (§4.3). Validation faults
/// (an id or timestamp the newtypes reject) have no position and keep
/// the codec's `None`. The registry's helper, for the same document
/// shape and the same reason.
fn corrupt(context: &str, error: &StoreError) -> StoreError {
    match error {
        StoreError::CorruptStore { offset, detail } => StoreError::CorruptStore {
            offset: *offset,
            detail: format!("{context}: {detail}"),
        },
        // `from_bytes` only ever fails as `CorruptStore`; the arm keeps
        // the mapping total rather than asserting that.
        other => other.clone(),
    }
}
