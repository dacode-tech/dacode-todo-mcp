//! The `dump` document: the store's graph as one canonical JSON value,
//! plus the command that writes it.
//!
//! The document mirrors the store's own persisted shapes — the node and
//! edge types serialize through the domain's serde schema and
//! [`BlocksEdgeDto`] exactly as the snapshot document does, so a node or
//! an edge has **one spelling in the log, the snapshot and the dump** —
//! with two deliberate omissions: no `format_version`
//! ([`todo_store::Store::format_version`] returns this *build's*
//! constant, not a measurement of the file, and printing it would claim
//! a comparison the dump did not make; `verify` reports the store's own
//! facts instead) and no `log_offset` (a property of a snapshot's
//! coverage, not of the graph).
//!
//! The dump carries the §3 allocation [`Counters`] as [`CountersDto`]:
//! the domain's `Counters` is serde-free by rule (t7; spec §2 — the
//! domain stays schema-free where the store owns the schema), and a
//! burned number is exactly what an inspector needs to see — an id gap
//! after an epic-delete cascade reads like damage unless the counter
//! says the number was issued and burned (§3: counters only move up).
//! The mirror follows the store's own DTO precedent: four named fields,
//! never a tuple (`_rust.md`'s house rule for a persisted-or-pinned
//! schema).
//!
//! `relates` keeps the snapshot document's two-element-array spelling —
//! the format's one positional shape, which U12 kept there for byte
//! compatibility and for `relates_from_arrays`' diagnostic. Mirroring it
//! (rather than inventing a named-pair DTO here) keeps one spelling of a
//! relates pair across every document this project emits; the tension
//! with the house rule is the format's, recorded at U12, not the dump's
//! to resolve unilaterally.
//!
//! Order comes from the graph's accessors, which are already total orders
//! (`epics()`/`tasks()`/`decisions()`/`docs()` are `BTreeMap` values,
//! `blocks_edges()` and `relates_pairs()` are documented orders), so the
//! document is deterministic by construction (§8) and
//! `tests/golden/cli_dump.v1.golden` pins it. Field order *is* byte order
//! (the codec's contract); serialization goes through
//! [`todo_store::to_bytes`] — the engine stays sealed in the store, and
//! there is no `--pretty` because canonical is the only shape (U14).

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use todo_domain::{Counters, Decision, Doc, Epic, Graph, Repo, Task, TaskId};
use todo_store::{BlocksEdgeDto, DataRoot, Store, StoreError, to_bytes};

use crate::args::Invocation;
use crate::{EXIT_OK, store_refused};

/// The dump's line terminator: the log's discipline and the golden
/// workflow's "exactly one trailing newline".
const LINE_FEED: u8 = b'\n';

/// The `dump` document: the whole graph, in the field order the golden
/// pins (§8: declaration order *is* byte order, and every array is in its
/// accessor's documented total order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dump {
    /// The repo node — as mutated, not as registered (§7's mutables).
    pub repo: Repo,
    /// The §3 allocation counters: the highest number issued per kind,
    /// including numbers burned above the highest live id.
    pub counters: CountersDto,
    /// Every epic, ascending by id.
    pub epics: Vec<Epic>,
    /// Every task, ascending by id.
    pub tasks: Vec<Task>,
    /// Every decision, ascending by id.
    pub decisions: Vec<Decision>,
    /// Every doc, ascending by id.
    pub docs: Vec<Doc>,
    /// Every `blocks` edge in [`Graph::blocks_edges`]' total order, in the
    /// log's own edge spelling.
    pub blocks: Vec<BlocksEdgeDto>,
    /// Every `relates` annotation in [`Graph::relates_pairs`]' ascending
    /// canonical (lower, higher) order, as a two-element array of id
    /// strings — the snapshot document's spelling (module docs).
    pub relates: Vec<(TaskId, TaskId)>,
}

/// The §3 counters as four named fields — the cli-side mirror of the
/// domain's serde-free [`Counters`], shaped exactly like the store's own
/// snapshot DTO so the two documents read alike. Named per kind so a
/// transposition cannot ride a positional shape into the golden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountersDto {
    /// The highest epic number issued.
    pub epic: u64,
    /// The highest task number issued.
    pub task: u64,
    /// The highest decision number issued.
    pub decision: u64,
    /// The highest doc number issued.
    pub doc: u64,
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

impl Dump {
    /// The document describing `graph`, built from the read accessors
    /// whose orders §8 documents — two runs over unchanged state are
    /// byte-identical, whatever mutation order produced them.
    pub fn from_graph(graph: &Graph) -> Self {
        Self {
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

    /// The canonical bytes of this document plus one trailing LF — the
    /// whole of `dump`'s stdout. A serialization failure is the codec's
    /// own named error; the schema is closed (no floats, no maps), so it
    /// is unreachable for these types and still total.
    pub fn to_line(&self) -> Result<Vec<u8>, StoreError> {
        let mut bytes = to_bytes(self)?;
        bytes.push(LINE_FEED);
        Ok(bytes)
    }
}

/// Runs `dump`: opens the store through [`DataRoot::open`] (which takes
/// §8's lock and replays) and writes the canonical document plus one
/// trailing LF to stdout — and nothing else, so `> graph.json` and
/// `| jq .` both work. Any store refusal is its §6.1 code on stderr and
/// the exit-code table's 3; a dump classifies nothing and repairs
/// nothing.
pub fn run(invocation: &Invocation) -> ExitCode {
    let root = DataRoot::new(PathBuf::from(&invocation.root));
    let store = match root.open(&invocation.slug) {
        Ok(store) => store,
        Err(error) => return store_refused(&error),
    };
    let document = Dump::from_graph(store.graph());
    let bytes = match document.to_line() {
        Ok(bytes) => bytes,
        Err(error) => return store_refused(&error),
    };
    // A stdout write fault (a closed pipe, a full disk) is the same
    // honest `io` report as any other: the code, the attempt and the
    // kind — never a path, and never a silent short write.
    let mut stdout = std::io::stdout();
    if let Err(error) = stdout
        .write_all(&bytes)
        .and_then(|()| stdout.flush())
        .map_err(|error| StoreError::Io {
            detail: format!("write the dump to stdout: {:?}", error.kind()),
        })
    {
        return store_refused(&error);
    }
    ExitCode::from(EXIT_OK)
}
