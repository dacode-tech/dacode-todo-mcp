//! The `verify` command: the report for a store that opened, and for one
//! that did not.
//!
//! Every fact comes from somewhere honest: the counts from the opened
//! graph's accessors, the file sizes from `fs::metadata` on the layout's
//! own [`DataRoot::log_path`]/[`DataRoot::snapshot_path`] — a *stat*, not
//! a second reading of the format — and the fault classifications from
//! the store itself ([`StoreError`]'s codes and fields, and
//! [`DataRoot::repair`]'s dry run for a corrupt store). There is
//! deliberately **no line count**: counting lines would put a second
//! notion of "line" beside the log's own `LineSplit` (f25's parked
//! boundary — the inspector reads no raw bytes at all).
//!
//! The report is deterministic for a given store state (§8): fixed lines
//! in a fixed order, no clock, no map iteration, and no host path beyond
//! the `--root` the operator supplied — the store's own error texts are
//! path-free by contract, and the stat lines report sizes, never names.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::ExitCode;

use todo_domain::{Graph, RepoSlug};
use todo_store::{DataRoot, Store, StoreError};

use super::repair::outcome_report;
use super::{EXIT_FAULT, EXIT_OK, fault_lines, store_refused};

/// Runs `verify`: open the store; on success print the state report to
/// stdout and exit 0; on a corrupt store print the fault and the store's
/// own dry-run classification to stderr and exit 1; on any other store
/// refusal print its §6.1 code and report to stderr and exit 3.
pub fn run(root: &DataRoot, slug: &RepoSlug) -> ExitCode {
    match root.open(slug) {
        Ok(store) => match opened_report(root, slug, store.graph()) {
            Ok(report) => {
                print!("{report}");
                ExitCode::from(EXIT_OK)
            }
            Err(error) => store_refused(&error),
        },
        // A corrupt store is verify's found-a-fault case: the command ran
        // and reported what it will not act on (exit-code table's 1). The
        // failed open already dropped its lock guard, so the dry-run
        // classification below is not contention.
        Err(StoreError::CorruptStore { offset, detail }) => {
            eprint!("{}", corrupt_report(root, slug, offset, &detail));
            ExitCode::from(EXIT_FAULT)
        }
        // `locked`, `version_mismatch`, `io` (a missing store directory
        // included — the engine's honest NotFound, never a guess): the
        // store refused the operation, exit-code table's 3. A contended
        // store gets no repair dry run — it would contend on the same
        // lock.
        Err(error) => store_refused(&error),
    }
}

/// The clean-store report: the slug, the node and edge counts from the
/// opened graph, and the two file sizes as a stat reports them. The
/// snapshot line is `none` when no snapshot file exists — its absence is
/// a state, not a fault (§4.3: a store without a snapshot replays its
/// whole log). A stat that fails any other way is the store refusing the
/// operation, reported through the module's io shape.
fn opened_report(root: &DataRoot, slug: &RepoSlug, graph: &Graph) -> Result<String, StoreError> {
    let log_bytes = size_or_none(stat_bytes(&root.log_path(slug), "stat the event log")?);
    let snapshot_bytes = size_or_none(stat_bytes(&root.snapshot_path(slug), "stat the snapshot")?);
    Ok(format!(
        "slug: {slug}\n\
         epics: {}\n\
         tasks: {}\n\
         decisions: {}\n\
         docs: {}\n\
         blocks: {}\n\
         relates: {}\n\
         log_bytes: {log_bytes}\n\
         snapshot_bytes: {snapshot_bytes}\n\
         status: ok\n",
        graph.epics().count(),
        graph.tasks().count(),
        graph.decisions().count(),
        graph.docs().count(),
        graph.blocks_edges().len(),
        graph.relates_pairs().len(),
    ))
}

/// A file's byte size: `None` when the file does not exist, and the io
/// fault in the store's own detail convention — the attempt and the
/// [`ErrorKind`], never a path — when the stat fails for any other
/// reason.
fn stat_bytes(path: &Path, context: &str) -> Result<Option<u64>, StoreError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata.len())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(StoreError::Io {
            detail: format!("{context}: {:?}", error.kind()),
        }),
    }
}

/// A stat'd size as its report spelling: the byte count, or `none` for an
/// absent file — its absence is a state, not a fault.
fn size_or_none(bytes: Option<u64>) -> String {
    match bytes {
        Some(bytes) => bytes.to_string(),
        None => "none".to_owned(),
    }
}

/// The corrupt-store report: §6.1's code on its own line, the offset with
/// the plain statement of what an offset can mean (a snapshot-document
/// fault carries an offset in the snapshot's own bytes, not a log byte —
/// f31, which t22 pinned), the engine's detail, and then the store's own
/// classification of the fault: [`DataRoot::repair`]'s dry run, whose
/// outcome lines are `repair`'s shared report. A classification the dry
/// run itself cannot produce — a lock taken in between, a snapshot
/// document that does not parse — says so with its own code instead of
/// guessing.
fn corrupt_report(root: &DataRoot, slug: &RepoSlug, offset: Option<usize>, detail: &str) -> String {
    let mut report = String::from("corrupt_store\n");
    if let Some(offset) = offset {
        report.push_str(&format!("offset: {offset}\n"));
        report.push_str(
            "note: a snapshot-document fault carries an offset in the snapshot's own bytes, \
             not a log byte\n",
        );
    }
    report.push_str(&format!("detail: {detail}\n"));
    match root.repair(slug, true) {
        Ok(outcome) => report.push_str(&outcome_report(&outcome)),
        Err(error) => report.push_str(&format!(
            "classification: unavailable\n{}",
            fault_lines(&error)
        )),
    }
    report
}
