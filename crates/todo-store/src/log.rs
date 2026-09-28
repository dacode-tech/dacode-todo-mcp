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
//! offset, which is where t13's repair truncates from, and the engine's
//! position *within* that line rides in the `detail` text.
//!
//! The layout is a bare path today: [`JsonlStore::create`] and
//! [`JsonlStore::open`] take the log file itself. t11 wraps them in the D4
//! layout — `repos/<slug>/` under an injected data root, plus the shared
//! registry — and puts §8's advisory lock on the handle this struct keeps
//! open (single writer, one process per store).
//!
//! Durability note: an append that fails after the domain accepted the
//! mutation leaves memory one event ahead of disk, which is §4.3's crash
//! window with the crash replaced by an io error. That state is
//! session-fatal by design — the caller reopens, and replay from disk
//! drops the unpersisted mutation (see [`crate::Store`]).

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use todo_domain::{
    BlocksEdge, Decision, DecisionId, Doc, DocId, Epic, EpicId, Graph, Repo, Task, TaskId,
    Timestamp,
};
use todo_domain::{DecisionDraft, DocDraft, EpicDraft, TaskDraft};

use crate::FORMAT_VERSION;
use crate::codec::{from_bytes, to_bytes};
use crate::error::StoreError;
use crate::event::{
    BlocksEdgeDto, DecisionPatch, DocPatch, EpicPatch, Event, LogHeader, TaskPatch,
    apply_decision_patch, apply_doc_patch, apply_epic_patch, apply_task_patch, replay_step,
};
use crate::store::Store;

/// The line terminator, and the only byte the log's line discipline
/// attaches meaning to (§4.1: every mutation is one JSON line).
const LINE_FEED: u8 = b'\n';

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

/// One repository's store: its event log and the graph replayed from it.
///
/// The [`Store`] trait is the whole surface a caller above this crate
/// needs; this struct is the D1 engine behind it and is only named
/// directly to create or open a log.
#[derive(Debug)]
pub struct JsonlStore {
    /// The log file, kept open for appends: §8's single writer, and the
    /// handle t11's advisory lock goes on.
    file: File,
    /// The graph every read answers from, kept current by every append.
    graph: Graph,
}

impl JsonlStore {
    /// Creates a log at `path` holding one repository, and returns the
    /// store over it.
    ///
    /// The file must not exist: creation is `create_new`, so an existing
    /// log is an io error and never a truncation (§4: corruption and loss
    /// are named errors, not silent ones). The header line and the
    /// `repo_register` seed are one `write_all` followed by `sync_all`, so
    /// a crash during creation leaves either a log [`JsonlStore::open`]
    /// rejects as corrupt — an empty or torn file — or a whole seeded one,
    /// never a half-created store that opens. The seed's `ts` is `repo`'s
    /// registration time.
    pub fn create<P: AsRef<Path>>(path: P, repo: Repo) -> Result<Self, StoreError> {
        let mut file = OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(path.as_ref())
            .map_err(|error| io_error("create the event log", &error))?;

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
        Ok(Self { file, graph })
    }

    /// Opens the log at `path` and replays it into the graph it describes
    /// (§4.1). The handle stays open for appends.
    ///
    /// A missing file is an io error, never an empty store. A log whose
    /// header declares another format version is
    /// [`StoreError::VersionMismatch`]; a log that is malformed anywhere —
    /// a torn last line, garbage in the interior, a seed out of place, an
    /// event the domain rejects, an id that is not the next of its kind —
    /// is [`StoreError::CorruptStore`] carrying the offending line's
    /// absolute byte offset.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(path.as_ref())
            .map_err(|error| io_error("open the event log", &error))?;
        // A freshly opened handle reads from 0, and `O_APPEND` moves the
        // write position to the end on every write regardless of where a
        // read left it — so one handle serves both replay and append.
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| io_error("read the event log", &error))?;
        let graph = replay(&bytes)?;
        Ok(Self { file, graph })
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
}

impl Store for JsonlStore {
    fn graph(&self) -> &Graph {
        &self.graph
    }

    fn format_version(&self) -> u32 {
        FORMAT_VERSION
    }

    fn add_epic(&mut self, draft: EpicDraft, at: Timestamp) -> Result<Epic, StoreError> {
        let epic = self.graph.add_epic(draft).map_err(StoreError::Domain)?;
        self.append(&Event::EpicAdd {
            ts: at,
            epic: epic.clone(),
        })?;
        Ok(epic)
    }

    fn add_task(&mut self, draft: TaskDraft, at: Timestamp) -> Result<Task, StoreError> {
        let task = self
            .graph
            .add_task(draft, at.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::TaskAdd {
            ts: at,
            task: task.clone(),
        })?;
        Ok(task)
    }

    fn add_decision(
        &mut self,
        draft: DecisionDraft,
        at: Timestamp,
    ) -> Result<Decision, StoreError> {
        let decision = self
            .graph
            .add_decision(draft, at.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::DecisionAdd {
            ts: at,
            decision: decision.clone(),
        })?;
        Ok(decision)
    }

    fn add_doc(&mut self, draft: DocDraft, at: Timestamp) -> Result<Doc, StoreError> {
        let doc = self
            .graph
            .add_doc(draft, at.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::DocRegister {
            ts: at,
            doc: doc.clone(),
        })?;
        Ok(doc)
    }

    fn update_epic(
        &mut self,
        id: EpicId,
        at: Timestamp,
        patch: EpicPatch,
    ) -> Result<Epic, StoreError> {
        let epic = self
            .graph
            .update_epic(id.clone(), |epic| apply_epic_patch(epic, &patch))
            .map_err(StoreError::Domain)?;
        self.append(&Event::EpicUpdate { ts: at, id, patch })?;
        Ok(epic)
    }

    fn update_task(
        &mut self,
        id: TaskId,
        at: Timestamp,
        patch: TaskPatch,
    ) -> Result<Task, StoreError> {
        let event_id = id.clone();
        let task = self
            .graph
            .update_task(id, at.clone(), |task| apply_task_patch(task, &patch))
            .map_err(StoreError::Domain)?;
        self.append(&Event::TaskUpdate {
            ts: at,
            id: event_id,
            patch,
        })?;
        Ok(task)
    }

    fn update_decision(
        &mut self,
        id: DecisionId,
        at: Timestamp,
        patch: DecisionPatch,
    ) -> Result<Decision, StoreError> {
        let event_id = id.clone();
        let decision = self
            .graph
            .update_decision(id, at.clone(), |decision| {
                apply_decision_patch(decision, &patch)
            })
            .map_err(StoreError::Domain)?;
        self.append(&Event::DecisionUpdate {
            ts: at,
            id: event_id,
            patch,
        })?;
        Ok(decision)
    }

    fn update_doc(&mut self, id: DocId, at: Timestamp, patch: DocPatch) -> Result<Doc, StoreError> {
        let event_id = id.clone();
        let doc = self
            .graph
            .update_doc(id, at.clone(), |doc| apply_doc_patch(doc, &patch))
            .map_err(StoreError::Domain)?;
        self.append(&Event::DocUpdate {
            ts: at,
            id: event_id,
            patch,
        })?;
        Ok(doc)
    }

    fn set_doc_path(&mut self, id: DocId, path: String, at: Timestamp) -> Result<Doc, StoreError> {
        let event_id = id.clone();
        let doc = self
            .graph
            .update_doc(id, at.clone(), |doc| doc.path = path.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::DocSetPath {
            ts: at,
            id: event_id,
            path,
        })?;
        Ok(doc)
    }

    fn set_repo_path(&mut self, path: String, at: Timestamp) -> Result<Repo, StoreError> {
        let repo = self.graph.set_repo_path(path.clone());
        self.append(&Event::RepoSetPath { ts: at, path })?;
        Ok(repo)
    }

    fn set_repo_name(&mut self, name: String, at: Timestamp) -> Result<Repo, StoreError> {
        let repo = self.graph.set_repo_name(name.clone());
        self.append(&Event::RepoSetName { ts: at, name })?;
        Ok(repo)
    }

    fn delete_epic(&mut self, id: EpicId, at: Timestamp) -> Result<Vec<TaskId>, StoreError> {
        let cascade = self
            .graph
            .delete_epic(id.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::EpicDelete { ts: at, id })?;
        Ok(cascade)
    }

    fn link_blocks(&mut self, edge: BlocksEdge, at: Timestamp) -> Result<(), StoreError> {
        let line = BlocksEdgeDto::from(edge.clone());
        self.graph.link_blocks(edge).map_err(StoreError::Domain)?;
        self.append(&Event::BlocksLink { ts: at, edge: line })
    }

    fn unlink_blocks(&mut self, edge: BlocksEdge, at: Timestamp) -> Result<(), StoreError> {
        let line = BlocksEdgeDto::from(edge.clone());
        self.graph.unlink_blocks(edge).map_err(StoreError::Domain)?;
        self.append(&Event::BlocksUnlink { ts: at, edge: line })
    }

    fn link_relates(&mut self, a: TaskId, b: TaskId, at: Timestamp) -> Result<(), StoreError> {
        self.graph
            .link_relates(a.clone(), b.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::RelatesLink { ts: at, a, b })
    }

    fn unlink_relates(&mut self, a: TaskId, b: TaskId, at: Timestamp) -> Result<(), StoreError> {
        self.graph
            .unlink_relates(a.clone(), b.clone())
            .map_err(StoreError::Domain)?;
        self.append(&Event::RelatesUnlink { ts: at, a, b })
    }
}

/// Replays log bytes into the graph they describe (§4.1): the header, the
/// seed, then every event in order.
///
/// The whole log is read into one buffer and split once: at §4.1's scale —
/// 10^3 to 10^5 nodes — the buffer is the cheaper machinery, and holding
/// every byte in one place is what lets a line's offset be absolute rather
/// than relative to whatever a streaming reader had buffered. t12's
/// snapshot + log-tail replay narrows what has to be read, not how.
///
/// The order of the header's own checks is deliberate — parse, version,
/// terminator — so a log written by a newer build reports
/// [`StoreError::VersionMismatch`] even when its header line is the only
/// thing in the file: that is a migration question (§4), not corruption.
fn replay(bytes: &[u8]) -> Result<Graph, StoreError> {
    let mut lines = LineSplit::new(bytes);

    let Some(header) = lines.next() else {
        return Err(corrupt(0, DETAIL_EMPTY_LOG));
    };
    check_header(&header)?;

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
    let mut graph = Graph::new(repo.clone());

    for line in lines {
        let event = parse_line(&line)?;
        replay_step(&mut graph, &event).map_err(|fault| corrupt(line.offset, fault.to_string()))?;
    }
    Ok(graph)
}

/// Checks the header line: parseable, current, and committed (terminated).
fn check_header(line: &LogLine<'_>) -> Result<(), StoreError> {
    let header: LogHeader = from_bytes(line.bytes)
        .map_err(|error| corrupt(line.offset, prefixed(DETAIL_BAD_HEADER, &error)))?;
    if header.format_version != FORMAT_VERSION {
        return Err(StoreError::VersionMismatch {
            found: header.format_version,
            expected: FORMAT_VERSION,
        });
    }
    if !line.terminated {
        return Err(corrupt(line.offset, DETAIL_TORN_LINE));
    }
    Ok(())
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
struct LogLine<'a> {
    /// Where the line starts in the file — the offset a repair truncates
    /// from (§4.3).
    offset: usize,
    /// The line's own bytes, terminator excluded.
    bytes: &'a [u8],
    /// Whether an LF followed them. `false` means the write was torn.
    terminated: bool,
}

/// Splits log bytes into [`LogLine`]s in file order.
///
/// The offset arithmetic lives here and nowhere else: a line reports the
/// position the splitter itself had consumed when it reached it, so the
/// offsets in §4.3's errors cannot drift from the split that produced
/// them.
struct LineSplit<'a> {
    /// The bytes not yet split.
    rest: &'a [u8],
    /// How many bytes of the input the yielded lines accounted for.
    consumed: usize,
}

impl<'a> LineSplit<'a> {
    /// A splitter over a whole log.
    fn new(bytes: &'a [u8]) -> Self {
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

/// An io fault as this crate reports it: what was attempted and the
/// [`io::ErrorKind`] that refused it. Never the error itself (no foreign
/// type crosses the boundary, §2) and never a path — `io::Error`'s own
/// message can carry one.
fn io_error(context: &str, error: &io::Error) -> StoreError {
    StoreError::Io {
        detail: format!("{context}: {:?}", error.kind()),
    }
}
