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

use std::fs::File;
use std::io::Read;
use std::path::Path;

use todo_domain::Graph;

use crate::error::{StoreError, io_error};
use crate::log::replay;

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
