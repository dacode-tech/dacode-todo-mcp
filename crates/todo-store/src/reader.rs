//! D19's lock-free read (signed (a) 2026-10-08): the committed prefix of
//! a log, replayed without the writer's lock.
//!
//! §8's single writer is a *writer* discipline — one process mutates one
//! store at a time — and the engine tolerates concurrent readers of a
//! committed log. This module is that reader: it opens the log
//! read-only, never the lock path and never the append handle, and
//! writes nothing, so a session server holding the store's lock (f110's
//! measured refusal) does not contend with it and the store directory is
//! byte-identical after a read.
//!
//! # The committed prefix (§4.3's scoped reading)
//!
//! The trailing LF is the commit marker, so the prefix is cut at the
//! **last** LF: everything before it is committed, whatever the bytes
//! after it look like — the one veto-able reading D19 leaves, pinned by
//! `tests/layout/reader.rs`. An unterminated final line is an append in
//! flight, not damage: the read answers the graph the prefix describes
//! and drops the fragment. A **terminated but unparseable** final line
//! is [`StoreError::CorruptStore`] on every path — D19 weakens nothing
//! else. The locked open's semantics stay bit-identical because
//! [`crate::log::replay`] is reused verbatim: the prefix is derived
//! here, before the call, and replay's own torn-tail strictness is
//! untouched.
//!
//! No LF anywhere — a torn seed — is the empty prefix, and replay's own
//! header error answers it: a crashed create is never registered, so
//! the shape is unreachable by slug and reported, not special-cased.
//!
//! The io context strings reuse the locked open's vocabulary ("open the
//! event log", "read the event log") so a client sees one spelling per
//! fault. The one divergence is exotic: a directory parked at the log
//! path fails this path's *read* step (a read-only open tolerates a
//! directory) where the locked open fails its own open step — the
//! context names the attempt that actually refused.
//!
//! # The event census (t84's D1/D2)
//!
//! [`DataRoot::read_repo_census`] is the second lock-free read beside
//! [`DataRoot::read_repo`]: the same committed prefix, folded into
//! [`EventCensus`] instead of replayed into a graph. The header line is
//! skipped — it is format machinery, not an event — while the seed
//! (`repo_register`) is an event like any other, so `total` counts
//! every committed mutation. A terminated but unparseable line is the
//! same positioned `corrupt_store` the graph read reports, byte for
//! byte: the census weakens nothing the reader's discipline holds. It
//! is snapshot-blind by design — the log is the event record, and what
//! a compaction dropped from the log stays dropped — so a store with a
//! snapshot counts the covered lines too, which is the honest answer
//! to "what does the log hold".

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use todo_domain::{Graph, RepoSlug, Timestamp};

use crate::DataRoot;
use crate::codec::from_bytes;
use crate::error::{StoreError, io_error};
use crate::event::Event;
use crate::log::{LineSplit, replay};

/// The line terminator, and the only byte the log's line discipline
/// attaches meaning to (§4.1) — the commit marker the prefix is cut at.
const LINE_FEED: u8 = b'\n';

/// The io-fault context of the read-only open, spelled exactly as the
/// locked open's so one fault has one spelling on both paths.
const CONTEXT_OPEN_LOG: &str = "open the event log";
/// The io-fault context of the read, the locked open's own spelling.
const CONTEXT_READ_LOG: &str = "read the event log";

/// Reads the graph the log at `path` describes as of its last commit —
/// D19's lock-free read, the body behind [`crate::DataRoot::read_repo`].
///
/// Takes no lock and writes nothing: the log is opened read-only, the
/// committed prefix is derived, and [`crate::log::replay`] — reused
/// verbatim, snapshot-aware — builds the graph from it. A writer holding
/// the store's lock does not contend with this call, and an append in
/// flight (an unterminated final line) is answered as the graph without
/// it.
///
/// A snapshot written between this read's log bytes and its snapshot
/// document can surface as a transient named [`StoreError::CorruptStore`]
/// (replay's reconcile cross-checks) — loud, never a silent wrong graph;
/// a retry succeeds once the write lands.
pub(crate) fn read_graph(path: &Path) -> Result<Graph, StoreError> {
    let mut file = File::open(path).map_err(|error| io_error(CONTEXT_OPEN_LOG, &error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| io_error(CONTEXT_READ_LOG, &error))?;
    replay(path, committed_prefix(&bytes))
}

/// The committed prefix: the bytes through the last LF. The trailing LF
/// is the commit marker (§4.1's line discipline), so an unterminated
/// final line — an append in flight — and everything after the last LF
/// are dropped; a log with no LF at all is a torn seed, answered by
/// replay's own header error over the empty prefix.
fn committed_prefix(bytes: &[u8]) -> &[u8] {
    match bytes.iter().rposition(|byte| *byte == LINE_FEED) {
        Some(index) => bytes.split_at(index + 1).0,
        None => &[],
    }
}

/// The store-derived activity summary (t84's D2): what the event census
/// counts over one log's committed prefix. A public summary type — the
/// engine stays sealed behind it — with `BTreeMap` everywhere (§8) so
/// two reads of unchanged bytes are equal and the serialized shape is
/// canonical.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct EventCensus {
    /// Every committed event, the seed included.
    pub total: u64,
    /// Events per `kind` tag, alphabetical (the map's own order).
    pub by_kind: BTreeMap<String, u64>,
    /// Events per day key `YYYY-MM-DD` — the event `ts`'s date part —
    /// ascending.
    pub by_day: BTreeMap<String, u64>,
    /// The first event's audit instant, canonical — the seed's, so the
    /// registration instant. `None` over an empty prefix.
    pub first_ts: Option<String>,
    /// The last event's audit instant, canonical. `None` over an empty
    /// prefix.
    pub last_ts: Option<String>,
}

impl DataRoot {
    /// Reads the store under `slug` and counts its committed events —
    /// the census read beside [`DataRoot::read_repo`] (t84's D1): the
    /// same lock-free discipline over the same committed prefix, folded
    /// into counts instead of replayed into a graph.
    ///
    /// Path-derived like [`DataRoot::read_repo`]: the registry is not
    /// consulted, no lock is taken, nothing is created. The header line
    /// is skipped (not an event); the seed is counted; a terminated but
    /// unparseable line is the same positioned `corrupt_store` the
    /// graph read reports. The census is snapshot-blind: it counts what
    /// the log holds, covered region included.
    pub fn read_repo_census(&self, slug: &RepoSlug) -> Result<EventCensus, StoreError> {
        read_census(&self.log_path(slug))
    }
}

/// Counts the events the log at `path` holds as of its last commit —
/// the census read's body behind [`DataRoot::read_repo_census`].
///
/// The committed prefix is derived exactly as [`read_graph`] derives
/// it, so the two reads answer the same world: an unterminated final
/// line is an append in flight and is not counted. Every line after the
/// header must parse — the one fault the census reports, spelled
/// byte-identically to the graph read's, so one damaged line has one
/// report on both paths.
pub(crate) fn read_census(path: &Path) -> Result<EventCensus, StoreError> {
    let mut file = File::open(path).map_err(|error| io_error(CONTEXT_OPEN_LOG, &error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| io_error(CONTEXT_READ_LOG, &error))?;
    census_of(committed_prefix(&bytes))
}

/// The census over one committed prefix: skip the header line, then
/// fold every event into the counts. The prefix ends at the last LF, so
/// every line within it is terminated — the torn-tail cut has already
/// happened, and no unterminated line can reach this loop.
fn census_of(prefix: &[u8]) -> Result<EventCensus, StoreError> {
    let mut lines = LineSplit::new(prefix);
    // The header is format machinery, not an event: skipped, never
    // counted. An empty prefix yields no header and no events — the
    // zero census, the honest count of a torn seed.
    let _header = lines.next();
    let mut census = EventCensus::default();
    for line in lines {
        let event: Event =
            from_bytes(line.bytes).map_err(|error| census_parse_fault(line.offset, &error))?;
        *census.by_kind.entry(event.kind().to_owned()).or_insert(0) += 1;
        *census.by_day.entry(day_key(event.ts())).or_insert(0) += 1;
        census.total += 1;
        if census.first_ts.is_none() {
            census.first_ts = Some(event.ts().as_str().to_owned());
        }
        census.last_ts = Some(event.ts().as_str().to_owned());
    }
    Ok(census)
}

/// The day key of an event's audit instant: the canonical spelling's
/// first ten bytes, `YYYY-MM-DD` — the width and shape
/// [`Timestamp`]'s validation guarantees, taken as chars so no slice
/// indexing appears on the path.
fn day_key(ts: &Timestamp) -> String {
    ts.as_str().chars().take(10).collect()
}

/// The census's parse fault: the graph read's own spelling —
/// `the line is not a parseable event: …` at the line's absolute
/// offset — duplicated here (log.rs's `DETAIL_BAD_EVENT` is private to
/// it) so a damaged line reports byte-identically on both read paths;
/// `tests/layout/reader.rs` pins that equality and catches any drift.
fn census_parse_fault(offset: usize, error: &StoreError) -> StoreError {
    let inner = match error {
        StoreError::CorruptStore { detail, .. } => detail.clone(),
        other => other.to_string(),
    };
    StoreError::CorruptStore {
        offset: Some(offset),
        detail: format!("the line is not a parseable event: {inner}"),
    }
}
