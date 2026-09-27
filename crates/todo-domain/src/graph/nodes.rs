//! Node mutations (spec §3, §5.1, §6): the allocating adds the server
//! calls, the explicit-id inserts the store's replay calls, field
//! updates, reparenting, and the epic-delete cascade.
//!
//! Two callers, two entry points per kind, one validation funnel. An
//! **allocating add** takes a draft — the node's fields without its
//! identity — hands out the next number of that kind, and returns the
//! stored node (§6: writes return the new state, so a client never needs
//! a read-after-write round trip). An **explicit-id insert** takes a
//! whole node whose id was allocated elsewhere (the store replaying an
//! event log) and raises the counter to `max(counter, n)` — never lower,
//! so a replayed number is never issued twice. Both end in the same
//! checks, in the same fixed order, so the same node always yields the
//! same error:
//!
//! 1. **scope** — the node's `repo` field and its id's slug must be this
//!    graph's; a node of another repository has no parent here, which is
//!    §5.1's `orphan_not_allowed`;
//! 2. **duplicate** — the id must be free;
//! 3. **parent** — a task's `epic` must name an epic of this graph
//!    (`unknown_id`), the `parent` edge of §5.1 enforced as a field
//!    invariant;
//! 4. **closure discipline** — [`Task::validate`] / [`Decision::validate`]
//!    (§3: `verify` at creation, `closed`⇒evidence,
//!    `blocked`/`deferred`⇒note, decision `signed`/`superseded`⇒resolution).
//!
//! Updates follow the same discipline from the other end: the patch runs
//! on a clone, the identity fields are restored (id and repo are not
//! patchable), the structural and closure checks run, and only then does
//! the clone land. Status transitions are otherwise unconstrained — any
//! status may move to any status as long as the resulting node validates
//! (§3 pins the requirements, not a transition matrix).

use crate::error::DomainError;
use crate::graph::edges::BlockEnd;
use crate::id::{DecisionId, DocId, EpicId, TaskId};
use crate::node::{Decision, Doc, Epic, Repo, Task};
use crate::time::Timestamp;

use super::drafts::{DecisionDraft, DocDraft, EpicDraft, TaskDraft};
use super::state::Graph;

impl Graph {
    /// Allocates the next epic id, stores the epic and returns it.
    pub fn add_epic(&mut self, draft: EpicDraft) -> Result<Epic, DomainError> {
        let Some(number) = self.counters.next_epic() else {
            return Err(self.exhausted(EpicId::PREFIX));
        };
        let epic = Epic {
            id: EpicId::from_nonzero(self.repo.slug.clone(), number),
            repo: self.repo.slug.clone(),
            title: draft.title,
            description: draft.description,
            status: draft.status,
            references: draft.references,
        };
        let stored = epic.clone();
        self.insert_epic(epic)?;
        Ok(stored)
    }

    /// Allocates the next task id, stores the task and returns it. Both
    /// timestamps are the caller-passed `at` (§8); the closure discipline
    /// applies at creation, so a draft without `verify` is
    /// `missing_verify` and nothing is stored.
    pub fn add_task(&mut self, draft: TaskDraft, at: Timestamp) -> Result<Task, DomainError> {
        let Some(number) = self.counters.next_task() else {
            return Err(self.exhausted(TaskId::PREFIX));
        };
        let task = Task {
            id: TaskId::from_nonzero(self.repo.slug.clone(), number),
            repo: self.repo.slug.clone(),
            epic: draft.epic,
            kind: draft.kind,
            title: draft.title,
            body: draft.body,
            status: draft.status,
            verify: draft.verify,
            evidence: draft.evidence,
            note: draft.note,
            references: draft.references,
            priority: draft.priority,
            until: draft.until,
            created: at.clone(),
            updated: at,
        };
        let stored = task.clone();
        self.insert_task(task)?;
        Ok(stored)
    }

    /// Allocates the next decision id, stores the decision and returns
    /// it; both timestamps are the caller-passed `at`.
    pub fn add_decision(
        &mut self,
        draft: DecisionDraft,
        at: Timestamp,
    ) -> Result<Decision, DomainError> {
        let Some(number) = self.counters.next_decision() else {
            return Err(self.exhausted(DecisionId::PREFIX));
        };
        let decision = Decision {
            id: DecisionId::from_nonzero(self.repo.slug.clone(), number),
            repo: self.repo.slug.clone(),
            title: draft.title,
            status: draft.status,
            options: draft.options,
            recommendation: draft.recommendation,
            resolution: draft.resolution,
            confidence: draft.confidence,
            references: draft.references,
            created: at.clone(),
            updated: at,
        };
        let stored = decision.clone();
        self.insert_decision(decision)?;
        Ok(stored)
    }

    /// Allocates the next doc id, stores the doc and returns it; both
    /// timestamps are the caller-passed `at`.
    pub fn add_doc(&mut self, draft: DocDraft, at: Timestamp) -> Result<Doc, DomainError> {
        let Some(number) = self.counters.next_doc() else {
            return Err(self.exhausted(DocId::PREFIX));
        };
        let doc = Doc {
            id: DocId::from_nonzero(self.repo.slug.clone(), number),
            repo: self.repo.slug.clone(),
            title: draft.title,
            path: draft.path,
            version: draft.version,
            references: draft.references,
            created: at.clone(),
            updated: at,
        };
        let stored = doc.clone();
        self.insert_doc(doc)?;
        Ok(stored)
    }

    /// Stores an epic whose id was allocated elsewhere — the replay path
    /// (§4.1). Raises the epic counter to `max(counter, n)`.
    pub fn insert_epic(&mut self, epic: Epic) -> Result<(), DomainError> {
        if !self.in_scope(&epic.repo, epic.id.repo()) {
            return Err(DomainError::OrphanNotAllowed {
                id: epic.id.to_string(),
            });
        }
        if self.epics.contains_key(&epic.id) {
            return Err(DomainError::Duplicate {
                id: epic.id.to_string(),
            });
        }
        let key = epic.id.clone();
        self.counters.raise_epic(key.number());
        self.epics.insert(key, epic);
        Ok(())
    }

    /// Stores a task whose id was allocated elsewhere — the replay path.
    /// The `parent` invariant is checked here too: a task whose `epic`
    /// names no epic of this graph cannot enter it, whatever its origin.
    pub fn insert_task(&mut self, task: Task) -> Result<(), DomainError> {
        if !self.in_scope(&task.repo, task.id.repo()) {
            return Err(DomainError::OrphanNotAllowed {
                id: task.id.to_string(),
            });
        }
        if self.tasks.contains_key(&task.id) {
            return Err(DomainError::Duplicate {
                id: task.id.to_string(),
            });
        }
        if !self.epics.contains_key(&task.epic) {
            return Err(DomainError::UnknownId {
                id: task.epic.to_string(),
            });
        }
        task.validate()?;
        let key = task.id.clone();
        self.counters.raise_task(key.number());
        self.tasks.insert(key, task);
        Ok(())
    }

    /// Stores a decision whose id was allocated elsewhere — the replay
    /// path. Decisions attach directly to their repo (§5.1), so there is
    /// no parent to check.
    pub fn insert_decision(&mut self, decision: Decision) -> Result<(), DomainError> {
        if !self.in_scope(&decision.repo, decision.id.repo()) {
            return Err(DomainError::OrphanNotAllowed {
                id: decision.id.to_string(),
            });
        }
        if self.decisions.contains_key(&decision.id) {
            return Err(DomainError::Duplicate {
                id: decision.id.to_string(),
            });
        }
        decision.validate()?;
        let key = decision.id.clone();
        self.counters.raise_decision(key.number());
        self.decisions.insert(key, decision);
        Ok(())
    }

    /// Stores a doc whose id was allocated elsewhere — the replay path.
    pub fn insert_doc(&mut self, doc: Doc) -> Result<(), DomainError> {
        if !self.in_scope(&doc.repo, doc.id.repo()) {
            return Err(DomainError::OrphanNotAllowed {
                id: doc.id.to_string(),
            });
        }
        if self.docs.contains_key(&doc.id) {
            return Err(DomainError::Duplicate {
                id: doc.id.to_string(),
            });
        }
        let key = doc.id.clone();
        self.counters.raise_doc(key.number());
        self.docs.insert(key, doc);
        Ok(())
    }

    /// Patches an epic in place and returns its new state. An epic has no
    /// timestamps (§3), so the patch records no `updated`; identity is
    /// not patchable.
    pub fn update_epic(
        &mut self,
        id: EpicId,
        patch: impl FnOnce(&mut Epic),
    ) -> Result<Epic, DomainError> {
        let Some(stored) = self.epics.get(&id) else {
            return Err(DomainError::UnknownId { id: id.to_string() });
        };
        let mut epic = stored.clone();
        patch(&mut epic);
        epic.id = id.clone();
        epic.repo = self.repo.slug.clone();
        self.epics.insert(id, epic.clone());
        Ok(epic)
    }

    /// Patches a task in place, stamps `updated = at` and returns its new
    /// state — the domain side of `task_update` (§6).
    ///
    /// The patch runs on a clone: the parent check and the closure
    /// discipline (§3) run over the result, and a rejection leaves the
    /// stored task untouched. Changing `epic` through the patch is a
    /// reparenting and is checked like one; [`Graph::reparent_task`] is
    /// the named form. Identity is not patchable — `id` and `repo` are
    /// restored before the checks, because the graph is keyed by the one
    /// and scoped to the other.
    pub fn update_task(
        &mut self,
        id: TaskId,
        at: Timestamp,
        patch: impl FnOnce(&mut Task),
    ) -> Result<Task, DomainError> {
        let Some(stored) = self.tasks.get(&id) else {
            return Err(DomainError::UnknownId { id: id.to_string() });
        };
        let mut task = stored.clone();
        patch(&mut task);
        task.id = id.clone();
        task.repo = self.repo.slug.clone();
        task.updated = at;
        if !self.epics.contains_key(&task.epic) {
            return Err(DomainError::UnknownId {
                id: task.epic.to_string(),
            });
        }
        task.validate()?;
        self.tasks.insert(id, task.clone());
        Ok(task)
    }

    /// Patches a decision in place, stamps `updated = at` and returns its
    /// new state — the domain side of `decision_update` (§6).
    ///
    /// Signing a decision changes no edge: its `blocks` edges simply stop
    /// gating (§3/D10 — "no edge mutation, the history stays"), so
    /// [`Graph::blocks`] returns the same list before and after.
    pub fn update_decision(
        &mut self,
        id: DecisionId,
        at: Timestamp,
        patch: impl FnOnce(&mut Decision),
    ) -> Result<Decision, DomainError> {
        let Some(stored) = self.decisions.get(&id) else {
            return Err(DomainError::UnknownId { id: id.to_string() });
        };
        let mut decision = stored.clone();
        patch(&mut decision);
        decision.id = id.clone();
        decision.repo = self.repo.slug.clone();
        decision.updated = at;
        decision.validate()?;
        self.decisions.insert(id, decision.clone());
        Ok(decision)
    }

    /// Patches a doc in place, stamps `updated = at` and returns its new
    /// state — the domain side of `doc_update` and of the audited
    /// `doc_set_path` (§6, D10).
    pub fn update_doc(
        &mut self,
        id: DocId,
        at: Timestamp,
        patch: impl FnOnce(&mut Doc),
    ) -> Result<Doc, DomainError> {
        let Some(stored) = self.docs.get(&id) else {
            return Err(DomainError::UnknownId { id: id.to_string() });
        };
        let mut doc = stored.clone();
        patch(&mut doc);
        doc.id = id.clone();
        doc.repo = self.repo.slug.clone();
        doc.updated = at;
        self.docs.insert(id, doc.clone());
        Ok(doc)
    }

    /// Moves a task to another epic — §5.1's "reparenting a task moves
    /// it". The target epic must exist (`unknown_id`); the task keeps its
    /// id, its edges and its history, and `updated` becomes `at`.
    pub fn reparent_task(
        &mut self,
        id: TaskId,
        epic: EpicId,
        at: Timestamp,
    ) -> Result<Task, DomainError> {
        self.update_task(id, at, move |task| task.epic = epic)
    }

    /// Updates the checkout path and returns the new repo node (§7: the
    /// path is a mutable attribute, the slug is the key, so a checkout
    /// moving changes nothing else). A [`Repo`] carries no `updated`
    /// field (§3) — the audit timestamp belongs to the store's event.
    pub fn set_repo_path(&mut self, path: String) -> Repo {
        self.repo.path = path;
        self.repo.clone()
    }

    /// Updates the display name and returns the new repo node (§7).
    pub fn set_repo_name(&mut self, name: String) -> Repo {
        self.repo.name = name;
        self.repo.clone()
    }

    /// Deletes an epic, and returns the task ids it took with it,
    /// ascending.
    ///
    /// §5.1: "deleting an epic requires its tasks to move first or be
    /// closed — no orphans, ever". Both terminals qualify as closed
    /// (§3 calls `wontfix` the terminal that needs no evidence), and a
    /// qualifying task cannot survive its parent, so it is deleted with
    /// the epic — every `blocks` and `relates` edge touching it goes too.
    /// One non-terminal task rejects the whole write with
    /// `orphan_not_allowed` naming the first offender in ascending id
    /// order, so the same state always reports the same task, and the
    /// epic and all its tasks stay put.
    ///
    /// Deleting an epic also removes its incoming decision→epic gating
    /// edges; the decision node itself is untouched. Task numbers are
    /// never reissued (§3), so the cascade leaves meaningful gaps.
    pub fn delete_epic(&mut self, id: EpicId) -> Result<Vec<TaskId>, DomainError> {
        if !self.epics.contains_key(&id) {
            return Err(DomainError::UnknownId { id: id.to_string() });
        }
        let children = self.tasks_of_epic(&id);
        if let Some(open) = children.iter().find(|task| !task.status.is_terminal()) {
            return Err(DomainError::OrphanNotAllowed {
                id: open.id.to_string(),
            });
        }
        let cascade: Vec<TaskId> = children.iter().map(|task| task.id.clone()).collect();
        for task in &cascade {
            self.remove_task(task);
        }
        self.remove_node_edges(&BlockEnd::Epic(id.clone()));
        self.epics.remove(&id);
        Ok(cascade)
    }

    /// Drops a task and every edge touching it. Private: §6 exposes no
    /// single-task delete, so a task leaves the graph only through the
    /// epic-delete cascade.
    fn remove_task(&mut self, id: &TaskId) {
        self.tasks.remove(id);
        self.remove_node_edges(&BlockEnd::Task(id.clone()));
        let touching: Vec<(TaskId, TaskId)> = self
            .relates
            .iter()
            .filter(|(a, b)| a == id || b == id)
            .cloned()
            .collect();
        for pair in touching {
            self.relates.remove(&pair);
        }
    }

    /// The error for an exhausted per-kind number space. §6.1 has no code
    /// of its own for it and the list is closed, so this reports
    /// `duplicate`: at `u64::MAX` every id of that kind is taken.
    /// Unreachable at the scale §4.1 states (10^3–10^5 nodes).
    fn exhausted(&self, prefix: char) -> DomainError {
        DomainError::Duplicate {
            id: format!("{}:{}{}", self.repo.slug, prefix, u64::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::RepoSlug;
    use crate::reference::References;
    use crate::status::{Status, TaskKind};
    use std::str::FromStr;

    fn slug(s: &str) -> RepoSlug {
        RepoSlug::from_str(s).unwrap()
    }

    fn at(s: &str) -> Timestamp {
        Timestamp::parse(s).unwrap()
    }

    fn graph() -> Graph {
        Graph::new(Repo {
            slug: slug("camera"),
            name: "Camera firmware".to_owned(),
            path: "/srv/repos/camera".to_owned(),
            registered_at: at("2026-09-25T08:00:00Z"),
        })
    }

    fn epic_draft(title: &str) -> EpicDraft {
        EpicDraft {
            title: title.to_owned(),
            description: String::new(),
            status: Status::Open,
            references: References::new(),
        }
    }

    fn task_draft(epic: EpicId) -> TaskDraft {
        TaskDraft {
            epic,
            kind: TaskKind::Task,
            title: "a task".to_owned(),
            body: String::new(),
            status: Status::Open,
            verify: Some("cargo test".to_owned()),
            evidence: None,
            note: None,
            references: References::new(),
            priority: None,
            until: None,
        }
    }

    #[test]
    fn graph_adds_allocate_identity_and_stamp_the_caller_time() {
        let mut graph = graph();
        let epic = graph.add_epic(epic_draft("the epic")).unwrap();
        assert_eq!(epic.id.to_string(), "camera:E1");
        assert_eq!(epic.repo.as_str(), "camera");

        let task = graph
            .add_task(task_draft(epic.id.clone()), at("2026-09-26T09:00:00Z"))
            .unwrap();
        assert_eq!(task.id.to_string(), "camera:U1");
        assert_eq!(task.epic, epic.id);
        assert_eq!(task.created, at("2026-09-26T09:00:00Z"));
        assert_eq!(task.updated, at("2026-09-26T09:00:00Z"));

        let doc = graph
            .add_doc(
                DocDraft {
                    title: "notes".to_owned(),
                    path: "docs/notes.md".to_owned(),
                    version: "1.0".to_owned(),
                    references: References::new(),
                },
                at("2026-09-26T09:30:00Z"),
            )
            .unwrap();
        assert_eq!(doc.id.to_string(), "camera:S1");
        assert_eq!(doc.created, at("2026-09-26T09:30:00Z"));

        let counters = graph.counters();
        assert_eq!(
            (counters.epic(), counters.task(), counters.doc()),
            (1, 1, 1)
        );
        assert_eq!(counters.decision(), 0);
    }

    #[test]
    fn graph_add_task_enforces_the_closure_discipline_at_creation() {
        let mut graph = graph();
        let epic = graph.add_epic(epic_draft("the epic")).unwrap();
        let mut draft = task_draft(epic.id.clone());
        draft.verify = None;
        assert_eq!(
            graph.add_task(draft.clone(), at("2026-09-26T09:00:00Z")),
            Err(DomainError::MissingVerify {
                id: "camera:U1".to_owned()
            })
        );
        // The rejected add stored nothing and burned no number.
        assert_eq!(graph.tasks().count(), 0);
        assert_eq!(graph.counters().task(), 0);

        draft.verify = Some("cargo test".to_owned());
        draft.status = Status::Closed;
        assert_eq!(
            graph.add_task(draft.clone(), at("2026-09-26T09:00:00Z")),
            Err(DomainError::MissingEvidence {
                id: "camera:U1".to_owned()
            })
        );
        draft.evidence = Some("42 passed".to_owned());
        assert!(graph.add_task(draft, at("2026-09-26T09:00:00Z")).is_ok());
    }

    #[test]
    fn graph_add_task_needs_an_epic_that_exists() {
        let mut graph = graph();
        let draft = task_draft(EpicId::from_str("camera:E9").unwrap());
        assert_eq!(
            graph.add_task(draft, at("2026-09-26T09:00:00Z")),
            Err(DomainError::UnknownId {
                id: "camera:E9".to_owned()
            })
        );
        assert_eq!(graph.tasks().count(), 0);
    }

    #[test]
    fn graph_updates_stamp_updated_and_reject_bad_results() {
        let mut graph = graph();
        let epic = graph.add_epic(epic_draft("the epic")).unwrap();
        let task = graph
            .add_task(task_draft(epic.id.clone()), at("2026-09-26T09:00:00Z"))
            .unwrap();

        // A status change that needs a note and has none is rejected,
        // and the stored task keeps its old state.
        assert_eq!(
            graph.update_task(task.id.clone(), at("2026-09-26T10:00:00Z"), |task| {
                task.status = Status::Blocked;
            }),
            Err(DomainError::MissingNote {
                id: "camera:U1".to_owned()
            })
        );
        let stored = graph.task(&task.id).unwrap();
        assert_eq!(stored.status, Status::Open);
        assert_eq!(stored.updated, at("2026-09-26T09:00:00Z"));

        let updated = graph
            .update_task(task.id.clone(), at("2026-09-26T10:00:00Z"), |task| {
                task.status = Status::Closed;
                task.evidence = Some("42 passed".to_owned());
            })
            .unwrap();
        assert_eq!(updated.status, Status::Closed);
        assert_eq!(updated.updated, at("2026-09-26T10:00:00Z"));
        assert_eq!(updated.created, at("2026-09-26T09:00:00Z"));

        // Identity is not patchable: the patch's id and repo are
        // discarded, the stored node keeps its own.
        let patched = graph
            .update_task(task.id.clone(), at("2026-09-26T11:00:00Z"), |task| {
                task.id = TaskId::from_str("camera:U99").unwrap();
                task.repo = slug("dacode");
                task.title = "renamed".to_owned();
            })
            .unwrap();
        assert_eq!(patched.id.to_string(), "camera:U1");
        assert_eq!(patched.repo.as_str(), "camera");
        assert_eq!(patched.title, "renamed");

        assert_eq!(
            graph.update_task(
                TaskId::from_str("camera:U9").unwrap(),
                at("2026-09-26T11:00:00Z"),
                |_| {}
            ),
            Err(DomainError::UnknownId {
                id: "camera:U9".to_owned()
            })
        );
    }

    #[test]
    fn graph_repo_attributes_are_mutable_and_returned() {
        let mut graph = graph();
        let repo = graph.set_repo_path("/home/mouse/camera".to_owned());
        assert_eq!(repo.path, "/home/mouse/camera");
        assert_eq!(graph.repo().path, "/home/mouse/camera");
        // The slug — the key — survives a path update (§7).
        assert_eq!(graph.slug().as_str(), "camera");
        let repo = graph.set_repo_name("Camera firmware (gen2)".to_owned());
        assert_eq!(repo.name, "Camera firmware (gen2)");
        assert_eq!(repo.registered_at, at("2026-09-25T08:00:00Z"));
    }
}
