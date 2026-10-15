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
//! Every read below gates the prefix's first line — the header —
//! through the graph read's own check: a newer-format log answers
//! [`StoreError::VersionMismatch`] here exactly as the graph read
//! answers it, never a silent sum over compatible events and never a
//! `corrupt_store` misreported at the first event line (U77's note).
//! The torn seed's empty prefix has no header to gate, and the reads'
//! zero answers stand.
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
//!
//! # The rewards totals (§15, D23)
//!
//! [`DataRoot::rewards_totals`] is the third lock-free read: §2's
//! iterate-the-registry pattern — load the registry, read every listed
//! repo's committed prefix, sum the `reward_grant` events. Totals are
//! **never stored**; the events are the record, so the same
//! snapshot-blind reading as the census's applies (nothing in v1 calls
//! `compact`, §4.3, so the log is the complete record in practice). The
//! answer is [`RewardsTotals`], `BTreeMap`-keyed so the per-repo order
//! is the registry's own ascending-slug order (§8).
//!
//! # The reward events (§15's display half, U63)
//!
//! [`DataRoot::rewards_events`] is the fourth lock-free read, the
//! totals' sibling over the same walk: it **collects** the
//! `reward_grant` and `reward_reflect` events instead of summing them,
//! answering [`RewardsEvents`] — the fields verbatim, never re-derived,
//! because §15's display renders them as recorded. The order is root
//! order: the registry's ascending-slug order across repos, log
//! (append) order within one — the order the events were recorded in,
//! which "recent" is a suffix of at the display layer. The repo each
//! event came from is the read's own key (the log's slug), not the
//! event's — the events are repo-scoped by their log alone.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use todo_domain::{Graph, RepoSlug, Size, TaskId, Timestamp};

use crate::DataRoot;
use crate::codec::from_bytes;
use crate::error::{StoreError, io_error};
use crate::event::{Event, ReflectionTrigger};
use crate::log::{LineSplit, LogLine, check_header, replay};
use crate::registry::Registry;

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
    replay(path, committed_prefix(&bytes)).map(|(graph, _)| graph)
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

/// The header gate the lock-free reads share (U77's note): the graph
/// read's own [`check_header`] — probe, version, strict parse — so one
/// log answers one way on every read path. A newer-format log is
/// [`StoreError::VersionMismatch`] here too, never a silent sum over
/// compatible events and never a `corrupt_store` misreported at the
/// first event line; a damaged header reports the positioned fault the
/// graph read reports. A `None` header — the torn seed's empty prefix
/// — passes: the reads' zero answers over a crashed create are their
/// own documented behavior.
fn gate_header(header: Option<LogLine<'_>>) -> Result<(), StoreError> {
    match header {
        Some(header) => check_header(&header).map(|_| ()),
        None => Ok(()),
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
    /// is gated (the graph read's own check) and skipped (not an
    /// event); the seed is counted; a terminated but
    /// unparseable line is the same positioned `corrupt_store` the
    /// graph read reports. The census is snapshot-blind: it counts what
    /// the log holds, covered region included.
    pub fn read_repo_census(&self, slug: &RepoSlug) -> Result<EventCensus, StoreError> {
        read_census(&self.log_path(slug))
    }

    /// The root's derived rewards totals (§15, D23): §2's
    /// iterate-the-registry pattern — every registered repo's log,
    /// lock-free, its `reward_grant` events summed. Totals are never
    /// stored; the events are the record, and this read is the only
    /// source of the number. Every registered repo appears in
    /// `by_repo`, zero included, so the answer's shape is the registry's
    /// own and does not move when rewards do.
    ///
    /// A store that will not read fails the whole call with the error
    /// its own read reports — the same all-or-nothing reading as
    /// [`DataRoot::repo_list`], for the same reason: a repo that
    /// quietly disappears from the totals is a silent omission. A
    /// reward sum that would overflow `u64` is a named `corrupt_store`:
    /// a log this build writes cannot produce one (each grant is
    /// checked), so it is damage, and saturating would hide it.
    pub fn rewards_totals(&self) -> Result<RewardsTotals, StoreError> {
        let registry = Registry::load(self)?;
        let mut totals = RewardsTotals::default();
        for slug in registry.slugs() {
            let repo_total = reward_total_of(&self.log_path(slug))?;
            totals.total = totals
                .total
                .checked_add(repo_total)
                .ok_or_else(overflowed)?;
            totals.by_repo.insert(slug.clone(), repo_total);
        }
        Ok(totals)
    }

    /// The root's reward events (§15's display half, U63): every
    /// `reward_grant` and `reward_reflect` event of every registered
    /// repo's committed prefix — the totals' own iterate-the-registry,
    /// lock-free walk, collecting the events instead of summing them.
    /// The answer's order is root order: the registry's
    /// ascending-slug order across repos, log (append) order within
    /// one. The fields are the events' own, verbatim — the display
    /// never re-derives them (§15).
    ///
    /// A store that will not read fails the whole call with the error
    /// its own read reports — the same all-or-nothing reading as
    /// [`DataRoot::rewards_totals`], for the same reason: a repo
    /// whose events quietly disappear from the answer is a silent
    /// omission.
    pub fn rewards_events(&self) -> Result<RewardsEvents, StoreError> {
        let registry = Registry::load(self)?;
        let mut events = RewardsEvents::default();
        for slug in registry.slugs() {
            let (mut grants, mut reflections) = reward_events_of(&self.log_path(slug), slug)?;
            events.grants.append(&mut grants);
            events.reflections.append(&mut reflections);
        }
        Ok(events)
    }
}

/// The derived rewards totals of a data root (§15): summed from the
/// `reward_grant` events of every registered repo's log — never stored.
/// A public summary type — the engine stays sealed behind it — with
/// `BTreeMap` everywhere (§8) so two reads of unchanged bytes are equal
/// and the serialized shape is canonical.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RewardsTotals {
    /// The per-root total: every `reward_grant` event's amount, summed
    /// across the registry's repos.
    pub total: u64,
    /// Per-repo totals, ascending by slug — the registry's own order.
    pub by_repo: BTreeMap<RepoSlug, u64>,
}

/// The root's reward events as the logs record them (§15's display
/// half, U63): every `reward_grant` and `reward_reflect` event of
/// every registered repo's committed prefix, in root order — the
/// registry's ascending-slug order across repos, log (append) order
/// within one. A public summary type — the engine stays sealed behind
/// it — carrying the events' fields verbatim: the display renders
/// them as recorded and never re-derives, interprets or reformats
/// them (§15).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RewardsEvents {
    /// The grants, root order.
    pub grants: Vec<GrantedReward>,
    /// The reflections, root order.
    pub reflections: Vec<RewardReflection>,
}

/// One granted reward as the log records it (§15's display half): the
/// `reward_grant` event's fields verbatim — the task, the amount, the
/// size and the multipliers the arithmetic applied — plus the repo
/// whose log holds it (the read's own key: the event is repo-scoped
/// by its log alone). Serialized in declaration order, the canonical
/// order (D14), so the progress document's twin embeds it as-is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrantedReward {
    /// The grant's audit instant — the closure's own `ts`.
    pub ts: Timestamp,
    /// The repo whose log holds the grant.
    pub repo: RepoSlug,
    /// The task whose closure granted.
    pub task: TaskId,
    /// The granted amount, in the config's currency units.
    pub amount: u64,
    /// The size whose base cost was applied — the reported actual, or
    /// the config's default bucket when the closure reported none.
    pub size: Size,
    /// The priority percent applied (100 when the task carried no
    /// priority).
    pub priority_pct: u64,
    /// The kind percent applied.
    pub kind_pct: u64,
}

/// One logged reflection as the log records it (§15's display half):
/// the `reward_reflect` event's fields verbatim — the answer exactly
/// as given, the trigger that asked, and the derived totals at the
/// moment the answer landed — plus the repo whose log holds it. The
/// answer is never interpreted here or above it (§15: the answers are
/// the experiment's data, not the server's).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RewardReflection {
    /// The reflection's audit instant — the answer's.
    pub ts: Timestamp,
    /// The repo whose log holds the reflection.
    pub repo: RepoSlug,
    /// The verbatim answer, as given.
    pub answer: String,
    /// Why the question was asked.
    pub trigger: ReflectionTrigger,
    /// The root's accumulated total at the moment.
    pub root_total: u64,
    /// This repo's accumulated total at the moment.
    pub repo_total: u64,
}

/// The overflow fault of a totals sum: unpositioned, because it locates
/// an arithmetic fact across a whole root, not a byte a repair would
/// cut at.
fn overflowed() -> StoreError {
    StoreError::CorruptStore {
        offset: None,
        detail: "the root's summed rewards overflow u64".to_owned(),
    }
}

/// One repo's reward total: the `reward_grant` events of its committed
/// prefix, summed. The same lock-free read, the same header gate and
/// the same parse-fault spelling as the census's, so one damaged line
/// — or one foreign-version header — reports identically on both
/// paths.
fn reward_total_of(path: &Path) -> Result<u64, StoreError> {
    let mut file = File::open(path).map_err(|error| io_error(CONTEXT_OPEN_LOG, &error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| io_error(CONTEXT_READ_LOG, &error))?;
    let mut lines = LineSplit::new(committed_prefix(&bytes));
    gate_header(lines.next())?;
    let mut total: u64 = 0;
    for line in lines {
        let event: Event =
            from_bytes(line.bytes).map_err(|error| census_parse_fault(line.offset, &error))?;
        if let Event::RewardGrant { amount, .. } = event {
            total = total.checked_add(amount).ok_or_else(overflowed)?;
        }
    }
    Ok(total)
}

/// One repo's reward events: the `reward_grant` and `reward_reflect`
/// events of its committed prefix, in log order. The same lock-free
/// read, the same header gate and the same parse-fault spelling as the
/// census's and the totals', so one damaged line — or one
/// foreign-version header — reports identically on every path.
fn reward_events_of(
    path: &Path,
    slug: &RepoSlug,
) -> Result<(Vec<GrantedReward>, Vec<RewardReflection>), StoreError> {
    let mut file = File::open(path).map_err(|error| io_error(CONTEXT_OPEN_LOG, &error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| io_error(CONTEXT_READ_LOG, &error))?;
    let mut lines = LineSplit::new(committed_prefix(&bytes));
    gate_header(lines.next())?;
    let mut grants = Vec::new();
    let mut reflections = Vec::new();
    for line in lines {
        let event: Event =
            from_bytes(line.bytes).map_err(|error| census_parse_fault(line.offset, &error))?;
        match event {
            Event::RewardGrant {
                ts,
                task,
                amount,
                size,
                priority_pct,
                kind_pct,
            } => grants.push(GrantedReward {
                ts,
                repo: slug.clone(),
                task,
                amount,
                size,
                priority_pct,
                kind_pct,
            }),
            Event::RewardReflect {
                ts,
                answer,
                trigger,
                root_total,
                repo_total,
            } => reflections.push(RewardReflection {
                ts,
                repo: slug.clone(),
                answer,
                trigger,
                root_total,
                repo_total,
            }),
            _ => {}
        }
    }
    Ok((grants, reflections))
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
    // The header is gated (the graph read's own check) then skipped —
    // format machinery, not an event, never counted. An empty prefix
    // yields no header and no events — the zero census, the honest
    // count of a torn seed.
    gate_header(lines.next())?;
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
/// offset — duplicated here (the log module's `DETAIL_BAD_EVENT` is
/// private to it) so a damaged line reports byte-identically on both
/// read paths; `tests/layout/reader.rs` pins that equality and catches
/// any drift.
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
