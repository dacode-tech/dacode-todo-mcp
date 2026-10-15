//! The [`Store`] trait's implementation over the JSONL engine (U75's
//! split out of `log`): the layout delegation, the reads,
//! `snapshot`/`compact`, the mutation methods, and §15's grant path —
//! `close_task`'s reward emission and `log_reflection`.
//!
//! The engine's own mechanics — the bare-path constructors, the append
//! discipline, the snapshot-aware replay, the header migration — stay
//! in the parent module; this one is the surface the trait names, and
//! it reaches the engine through the store's own private helpers
//! (`append`, `log_len`, `reopen_log`, `load_rewards_config`) exactly
//! as it did before the split. The crash-safety arguments moved with
//! their code: `compact`'s step order is documented beside it, and the
//! closure's grant path carries §15's atomicity notes.

use std::fs;

use todo_domain::{
    BlocksEdge, Decision, DecisionId, Doc, DocId, Epic, EpicId, Graph, Repo, RepoSlug, RewardInput,
    RewardsConfig, Size, Task, TaskId, Timestamp,
};
use todo_domain::{DecisionDraft, DocDraft, EpicDraft, TaskDraft};

use crate::FORMAT_VERSION;
use crate::codec::to_bytes;
use crate::config;
use crate::error::{StoreError, io_error};
use crate::event::{
    BlocksEdgeDto, DecisionPatch, DocPatch, EpicPatch, Event, LogHeader, ReflectionTrigger,
    TaskPatch, apply_decision_patch, apply_doc_patch, apply_epic_patch, apply_task_patch,
};
use crate::layout::{DataRoot, snapshot_path_for};
use crate::reader::RewardsTotals;
use crate::snapshot::{
    SnapshotDocument, rename_into_place, temp_path_for, write_snapshot_atomic, write_temp,
};
use crate::store::{Compaction, Store, TaskClosure};

use super::{CONTEXT_READ_LOG, JsonlStore, LINE_FEED};

impl JsonlStore {
    /// The root's rewards config, read fresh at the call (§15's opt-in
    /// is a document, not a session): `None` when this store has no
    /// config source or the root has no config — the inert root. A
    /// malformed config is the named error, never a silent inert root.
    fn load_rewards_config(&self) -> Result<Option<RewardsConfig>, StoreError> {
        match &self.config_path {
            Some(path) => config::load(path),
            None => Ok(None),
        }
    }
}

impl Store for JsonlStore {
    fn create(root: &DataRoot, repo: Repo) -> Result<Self, StoreError>
    where
        Self: Sized,
    {
        // The D4 directory is the layout's to make; the engine's own
        // `create_log` stays a bare-path constructor (§4.2).
        root.create_store(&repo)
    }

    fn open(root: &DataRoot, slug: &RepoSlug) -> Result<Self, StoreError>
    where
        Self: Sized,
    {
        // The layout's open: it resolves the path and attaches the
        // root's config document as this store's rewards source (§15).
        root.open(slug)
    }

    fn graph(&self) -> &Graph {
        &self.graph
    }

    fn format_version(&self) -> u32 {
        FORMAT_VERSION
    }

    fn snapshot(&mut self) -> Result<u64, StoreError> {
        let log_offset = self.log_len()?;
        let document = SnapshotDocument::from_graph(&self.graph, log_offset);
        write_snapshot_atomic(&snapshot_path_for(&self.path), &document)?;
        Ok(log_offset)
    }

    fn compact(&mut self) -> Result<Compaction, StoreError> {
        // `events_dropped` without a second parse: every committed line —
        // header and seed included — is exactly one LF-terminated line
        // (§4.1's line discipline, held by construction on an opened
        // store), so counting LF bytes over the old log and subtracting
        // the two lines that survive is the count. A torn fragment from a
        // crashed earlier session carries no LF and is not counted: it
        // was never a committed event, and the rewrite drops it like the
        // rest of the covered region.
        let old_log = fs::read(&self.path).map_err(|error| io_error(CONTEXT_READ_LOG, &error))?;
        let log_bytes_before = old_log.len() as u64;
        let committed_lines = old_log.iter().filter(|byte| **byte == LINE_FEED).count() as u64;
        let events_dropped = committed_lines.saturating_sub(2);

        // The step order IS the crash-safety argument; the table lives in
        // `Store::compact`'s doc (store.rs). Snapshot BEFORE truncating,
        // so a covering document is on disk before the log it covers is
        // shortened. The old truncate-then-re-base order had two silent
        // corners §4 bars: a first-ever compaction crashed between the
        // renames left NO snapshot beside the short log (a clean bare-repo
        // open), and a repeat compaction left the prior re-based snapshot
        // whose offset *equalled* the new short length (a clean stale open
        // that dropped the intervening events). Both are named errors now.

        // (1) The covering snapshot: `log_offset` = the current full log
        // length, through its own temp+fsync+rename. After it the pair on
        // disk is consistent — full log + a snapshot covering all of it,
        // empty tail.
        let cover_offset = self.snapshot()?;

        // (2) The new log: header + a re-emitted seed + the events after
        // the covering snapshot's offset — empty by construction ((1)
        // covered the whole log), but kept general so the shape is right
        // if that ever changes. The re-emitted seed carries the *current*
        // repo node (Decision 9 of the task — after compaction the log's
        // seed and the snapshot's `repo` are the same node, which is what
        // makes replay's slug check exact and keeps the compacted log
        // self-describing). `registered_at` is immutable (§3), so the
        // re-emitted seed's `ts` is the registration's own audit instant —
        // no clock is read (§8).
        let repo = self.graph.repo().clone();
        let seed = Event::RepoRegister {
            ts: repo.registered_at.clone(),
            repo,
        };
        let mut new_log = to_bytes(&LogHeader::current())?;
        new_log.push(LINE_FEED);
        new_log.extend_from_slice(&to_bytes(&seed)?);
        new_log.push(LINE_FEED);
        let tail_start = usize::try_from(cover_offset).unwrap_or(old_log.len());
        if let Some(tail) = old_log.get(tail_start..) {
            new_log.extend_from_slice(tail);
        }
        let log_bytes_after = new_log.len() as u64;
        let log_temp = temp_path_for(&self.path);
        write_temp(&log_temp, &new_log, "the compacted log")?;

        // (3) Rename the log into place.
        rename_into_place(&log_temp, &self.path, "the compacted log")?;
        // A (4)-failure — or a crash between (3) and (4) — leaves the
        // short log beside the COVERING snapshot, whose `log_offset` (the
        // old full length) now exceeds the short log: reopen reports
        // "beyond the log's end" naming both numbers, and the live
        // snapshot still holds the whole graph, so t13's repair recovers
        // by re-basing the offset. The append handle is still on the
        // orphaned old inode at that point: session-fatal, and the caller
        // must not append before reopening.

        // (4) Re-base the snapshot: the same graph, `log_offset` = the new
        // log's length — so the bytes differ from (1)'s document only in
        // that field. Its own temp+fsync+rename; after it the pair is
        // consistent again (short log + snapshot at its length, empty tail).
        // No directory fsync follows either rename, by decision (see the
        // `snapshot` module docs, f16).
        let snapshot_path = snapshot_path_for(&self.path);
        let mut rebased = to_bytes(&SnapshotDocument::from_graph(&self.graph, log_bytes_after))?;
        rebased.push(LINE_FEED);
        let snapshot_temp = temp_path_for(&snapshot_path);
        write_temp(&snapshot_temp, &rebased, "the snapshot")?;
        rename_into_place(&snapshot_temp, &snapshot_path, "the snapshot")?;

        // (5) Reopen the append handle: it must follow the log's new
        // inode, or every later append lands on the orphaned old one
        // (see `reopen_log`).
        self.reopen_log()?;

        Ok(Compaction {
            log_bytes_before,
            log_bytes_after,
            events_dropped,
        })
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

    fn close_task(
        &mut self,
        id: TaskId,
        at: Timestamp,
        patch: TaskPatch,
        actual_size: Option<Size>,
    ) -> Result<TaskClosure, StoreError> {
        // The config is read before anything is applied, so a malformed
        // config fails the whole call atomically (§8's write contract)
        // rather than leaving a closure half-granted.
        let config = self.load_rewards_config()?;
        let before = self.graph.task(&id).map(|task| task.status);
        let event_id = id.clone();
        let task = self
            .graph
            .update_task(id, at.clone(), |task| apply_task_patch(task, &patch))
            .map_err(StoreError::Domain)?;
        self.append(&Event::TaskUpdate {
            ts: at.clone(),
            id: event_id.clone(),
            patch,
        })?;
        // The two-append window (U77's recorded note): a crash or io
        // failure between the TaskUpdate append and the reward_grant
        // append loses the grant permanently — a retry sees
        // before == Closed, so no re-grant. No corruption: a lost
        // reward only, the same class as the config-read atomicity
        // above.
        // The grant fires on the transition into `closed` alone: a task
        // already closed (a later patch on a closed task) grants nothing
        // twice, and `wontfix` never was a closure. The computation is
        // the domain's, single-sourced — the store never re-derives the
        // fallback rules — and its `None` (overflow, non-closure) means
        // no event, per the reviewer note the domain's contract pins.
        let granted = match (&config, before) {
            (Some(config), Some(before))
                if config.enabled
                    && before != todo_domain::Status::Closed
                    && task.status == todo_domain::Status::Closed =>
            {
                todo_domain::reward(
                    config,
                    RewardInput {
                        status: task.status,
                        kind: task.kind,
                        priority: task.priority,
                        size: actual_size,
                    },
                )
            }
            _ => None,
        };
        if let Some(reward) = &granted {
            self.append(&Event::reward_grant(at, event_id, reward))?;
        }
        Ok(TaskClosure {
            task,
            reward: granted,
        })
    }

    fn log_reflection(
        &mut self,
        answer: String,
        trigger: ReflectionTrigger,
        totals: RewardsTotals,
        at: Timestamp,
    ) -> Result<(), StoreError> {
        // The store records the answer it was handed, verbatim (§15:
        // the server never interprets an answer, and neither does the
        // store). The enabled-gate for reflections is the caller's —
        // the question is asked only on a rewards-enabled root — so the
        // totals arrive with the answer, derived by the caller through
        // the root's own query.
        let repo_total = totals
            .by_repo
            .get(&self.graph.repo().slug)
            .copied()
            .unwrap_or(0);
        self.append(&Event::RewardReflect {
            ts: at,
            answer,
            trigger,
            root_total: totals.total,
            repo_total,
        })
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
