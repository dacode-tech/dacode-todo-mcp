//! The deterministic core of the task graph (spec §2): nodes, typed
//! edges, graph invariants and the selection rules over them.
//!
//! Boundary: no I/O, no async, no workspace dependencies. Persistence
//! lives in `todo-store`, the wire in `todo-mcp`; this crate is pure
//! computation over in-memory graph state, and a `use` crossing that
//! line is a defect. `serde` derives on these types are the shared
//! schema layer (§2's rules); the serializer itself is confined to the
//! store.
//!
//! Determinism lives here and nowhere else (spec §8): canonical ordering
//! and `next_task`/`next_items` selection are pure functions of graph
//! state — topology first, then epic registration order, then task id
//! (spec §5.2). Same store bytes in, same answer out, on every run and
//! every platform; no wall clock, no randomness, no hash-iteration
//! order.
//!
//! Current surface (spec §3 and §5): the core value types — identity
//! ([`RepoSlug`], the typed node ids [`TaskId`]/[`EpicId`]/
//! [`DecisionId`]/[`DocId`]), time values ([`Timestamp`], [`Date`]),
//! [`References`], the status/kind vocabularies with the closure
//! discipline ([`Task::validate`], [`Decision::validate`]), and the five
//! node structs ([`Repo`], [`Epic`], [`Task`], [`Decision`], [`Doc`]) —
//! plus the in-memory task graph over them: [`Graph`] holds one
//! repository's nodes, its §5.1 edges and the §3 id-allocation counters,
//! enforcing the `parent` tree, the acyclic `blocks` DAG
//! ([`BlocksEdge`], [`BlockEnd`], cycle-checked at write time), the
//! `relates` annotation and the no-orphans rule. The §5.2 canonical
//! order lands on top of it — [`Graph::canonical_order`] and
//! [`Graph::canonical_order_of_epic`] derive the topological order with
//! the D6 tiebreaker (topology, epic registration order, task id) — and
//! §5.2's selection half on top of that: [`Graph::next_task`] and
//! [`Graph::next_task_of_epic`] answer with the first actionable task
//! and the deferred summary ([`NextTask`], [`DeferredSummary`]), over
//! the [`Graph::is_actionable`] and [`Graph::is_until_satisfied`]
//! predicates, and §5.2's batch half beside it: [`Graph::next_items`],
//! [`Graph::next_items_of_epic`] and [`Graph::next_items_of_task`]
//! answer with the annotated batch — a scope's actionable nodes, or one
//! task's transitive dependents, each marked — and the same deferred
//! summary ([`NextItems`], [`NextItem`]). M2's remaining surface:
//! `task_list`'s filters.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

mod error;
mod graph;
mod id;
mod node;
mod reference;
mod status;
mod time;

pub use error::{DomainError, ErrorCode, ValueError};
pub use graph::{
    BlockEnd, BlocksEdge, Counters, DecisionDraft, DeferredSummary, DocDraft, EpicDraft, Graph,
    NextItem, NextItems, NextTask, TaskDraft,
};
pub use id::{DecisionId, DocId, EpicId, MAX_SLUG_LEN, RepoSlug, TaskId};
pub use node::{Confidence, Decision, Doc, Epic, Repo, Task};
pub use reference::{Reference, ReferenceKind, References};
pub use status::{DecisionStatus, Priority, Status, TaskKind};
pub use time::{Date, Timestamp};

#[cfg(test)]
mod testcodec;

#[cfg(test)]
mod tests {
    use super::testcodec::{from_bytes, to_bytes};
    use super::*;
    use std::str::FromStr;

    fn slug(s: &str) -> RepoSlug {
        RepoSlug::from_str(s).unwrap()
    }

    fn ts(s: &str) -> Timestamp {
        Timestamp::parse(s).unwrap()
    }

    fn reference(kind: ReferenceKind, target: &str, note: Option<&str>) -> Reference {
        Reference::new(kind, target.to_owned(), note.map(str::to_owned)).unwrap()
    }

    // A representative node set: every kind, with the optional fields
    // populated and references inserted out of canonical order.

    fn sample_repo() -> Repo {
        Repo {
            slug: slug("camera"),
            name: "Camera firmware".to_owned(),
            path: "/srv/repos/camera".to_owned(),
            registered_at: ts("2026-09-25T08:00:00Z"),
        }
    }

    fn sample_epic() -> Epic {
        let mut references = References::new();
        references
            .insert(reference(ReferenceKind::Url, "https://b.example", None))
            .unwrap();
        references
            .insert(reference(
                ReferenceKind::Tracker,
                "CAM-42",
                Some("the issue"),
            ))
            .unwrap();
        references
            .insert(reference(ReferenceKind::Tracker, "CAM-10", None))
            .unwrap();
        Epic {
            id: "camera:E2".parse().unwrap(),
            repo: slug("camera"),
            title: "Ship the sensor pipeline".to_owned(),
            description: "Everything from capture to encoding.".to_owned(),
            status: Status::InWork,
            references,
        }
    }

    fn sample_task() -> Task {
        let mut references = References::new();
        references
            .insert(reference(ReferenceKind::Doc, "camera:S1#5.2", None))
            .unwrap();
        references
            .insert(reference(
                ReferenceKind::File,
                "src/encode.rs",
                Some("hot path"),
            ))
            .unwrap();
        Task {
            id: "camera:U7".parse().unwrap(),
            repo: slug("camera"),
            epic: "camera:E2".parse().unwrap(),
            kind: TaskKind::Task,
            title: "Fix the dropped frames".to_owned(),
            body: "Under low light the encoder stalls.".to_owned(),
            status: Status::Closed,
            verify: Some("cargo test -p encoder frames".to_owned()),
            evidence: Some("42 passed, 0 failed".to_owned()),
            note: None,
            references,
            priority: Some(Priority::High),
            until: Some(Date::parse("2026-10-01").unwrap()),
            // created is second-precision, updated fractional, so the
            // node round-trips exercise both padding paths.
            created: ts("2026-09-25T09:00:00Z"),
            updated: ts("2026-09-26T17:30:00.123456789Z"),
        }
    }

    fn sample_decision() -> Decision {
        let mut references = References::new();
        references
            .insert(reference(
                ReferenceKind::Spec,
                "todo-mcp-design.md §4",
                None,
            ))
            .unwrap();
        Decision {
            id: "camera:D3".parse().unwrap(),
            repo: slug("camera"),
            title: "Which encoder profile".to_owned(),
            status: DecisionStatus::Signed,
            options: vec!["(a) baseline".to_owned(), "(b) main, 10-bit".to_owned()],
            recommendation: Some("(a) — the SoC tops out there".to_owned()),
            resolution: Some("signed (a) 2026-09-25".to_owned()),
            confidence: Confidence::Medium,
            references,
            created: ts("2026-09-24T10:00:00Z"),
            updated: ts("2026-09-25T11:00:00Z"),
        }
    }

    fn sample_doc() -> Doc {
        // Two references inserted out of canonical order (Url sorts
        // before Spec by kind), so the round-trip exercises the sort.
        let mut references = References::new();
        references
            .insert(reference(
                ReferenceKind::Spec,
                "todo-mcp-design.md §3",
                None,
            ))
            .unwrap();
        references
            .insert(reference(
                ReferenceKind::Url,
                "https://sensor.example",
                None,
            ))
            .unwrap();
        Doc {
            id: "camera:S1".parse().unwrap(),
            repo: slug("camera"),
            title: "Sensor bring-up notes".to_owned(),
            path: "docs/sensor.md".to_owned(),
            version: "1.4".to_owned(),
            references,
            created: ts("2026-09-20T08:00:00Z"),
            updated: ts("2026-09-26T08:00:00Z"),
        }
    }

    #[test]
    fn string_newtypes_serialize_as_their_canonical_strings() {
        assert_eq!(to_bytes(&slug("ikvm-html")).unwrap(), b"9:ikvm-html");
        assert_eq!(
            to_bytes(&TaskId::new(slug("camera"), 7).unwrap()).unwrap(),
            b"9:camera:U7"
        );
        assert_eq!(
            to_bytes(&EpicId::new(slug("camera"), 2).unwrap()).unwrap(),
            b"9:camera:E2"
        );
        assert_eq!(
            to_bytes(&ts("2026-09-26T12:34:56Z")).unwrap(),
            b"30:2026-09-26T12:34:56.000000000Z"
        );
        assert_eq!(
            to_bytes(&ts("2026-09-26T12:34:56.5Z")).unwrap(),
            b"30:2026-09-26T12:34:56.500000000Z"
        );
        assert_eq!(
            to_bytes(&Date::parse("2026-10-01").unwrap()).unwrap(),
            b"10:2026-10-01"
        );
    }

    #[test]
    fn fractional_timestamp_serde_roundtrips_and_is_byte_stable() {
        // A fractional input normalizes to the 30-byte canonical spelling
        // on the way in, then survives serde unchanged.
        let stamp = ts("2026-09-26T12:34:56.5Z");
        assert_eq!(stamp.as_str(), "2026-09-26T12:34:56.500000000Z");
        let bytes = to_bytes(&stamp).unwrap();
        assert_eq!(bytes, b"30:2026-09-26T12:34:56.500000000Z");
        // twice-run byte identity
        assert_eq!(to_bytes(&stamp).unwrap(), bytes);
        // round-trip: deserialize re-parses the canonical spelling and
        // re-serializes identically.
        let reloaded: Timestamp = from_bytes(&bytes).unwrap();
        assert_eq!(reloaded, stamp);
        assert_eq!(to_bytes(&reloaded).unwrap(), bytes);
    }

    #[test]
    fn enums_serialize_as_their_wire_spellings() {
        assert_eq!(
            to_bytes(&ErrorCode::MissingResolution).unwrap(),
            b"18:missing_resolution"
        );
        assert_eq!(to_bytes(&Status::InWork).unwrap(), b"7:in_work");
        assert_eq!(to_bytes(&TaskKind::Finding).unwrap(), b"7:finding");
        assert_eq!(
            to_bytes(&DecisionStatus::Superseded).unwrap(),
            b"10:superseded"
        );
        assert_eq!(to_bytes(&Priority::High).unwrap(), b"4:high");
        assert_eq!(to_bytes(&Confidence::Medium).unwrap(), b"6:medium");
        assert_eq!(to_bytes(&ReferenceKind::Tracker).unwrap(), b"7:tracker");
    }

    #[test]
    fn validated_newtypes_reject_bad_strings_on_deserialize() {
        for bad in ["camera:U07", "camera:E7x", "", "camera:U0"] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            assert!(
                from_bytes::<TaskId>(&bytes).is_err(),
                "TaskId must reject {bad:?}"
            );
        }
        for bad in ["Bad Slug", "-x", ""] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            assert!(
                from_bytes::<RepoSlug>(&bytes).is_err(),
                "RepoSlug must reject {bad:?}"
            );
        }
        for bad in ["2026-09-26T12:34:56+02:00", "2026-09-26t12:34:56z", "nope"] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            assert!(
                from_bytes::<Timestamp>(&bytes).is_err(),
                "Timestamp must reject {bad:?}"
            );
        }
        for bad in ["2023-02-29", "2026-13-01"] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            assert!(
                from_bytes::<Date>(&bytes).is_err(),
                "Date must reject {bad:?}"
            );
        }
    }

    #[test]
    fn node_serde_roundtrips() {
        let repo = sample_repo();
        let epic = sample_epic();
        let task = sample_task();
        let decision = sample_decision();
        let doc = sample_doc();

        assert_eq!(from_bytes::<Repo>(&to_bytes(&repo).unwrap()).unwrap(), repo);
        assert_eq!(from_bytes::<Epic>(&to_bytes(&epic).unwrap()).unwrap(), epic);
        assert_eq!(from_bytes::<Task>(&to_bytes(&task).unwrap()).unwrap(), task);
        assert_eq!(
            from_bytes::<Decision>(&to_bytes(&decision).unwrap()).unwrap(),
            decision
        );
        assert_eq!(from_bytes::<Doc>(&to_bytes(&doc).unwrap()).unwrap(), doc);
    }

    #[test]
    fn struct_bytes_start_with_fields_in_declaration_order() {
        // Pins §8's field-order contract: 'd', then id, repo, epic,
        // kind with their canonical string values.
        let bytes = to_bytes(&sample_task()).unwrap();
        let prefix = "d2:id9:camera:U74:repo6:camera4:epic9:camera:E24:kind4:task";
        assert!(bytes.starts_with(prefix.as_bytes()), "got {bytes:?}");
        let repo_bytes = to_bytes(&sample_repo()).unwrap();
        assert!(
            repo_bytes.starts_with(b"d4:slug6:camera"),
            "got {repo_bytes:?}"
        );
    }

    #[test]
    fn twice_run_serialization_is_byte_identical() {
        let task = sample_task();
        let decision = sample_decision();
        let nodes: Vec<Task> = vec![
            task.clone(),
            sample_task(),
            Task {
                id: "camera:U8".parse().unwrap(),
                status: Status::Deferred,
                note: Some("until the pipeline lands".to_owned()),
                evidence: None,
                ..sample_task()
            },
        ];
        for _ in 0..2 {
            assert_eq!(
                to_bytes(&sample_repo()).unwrap(),
                to_bytes(&sample_repo()).unwrap()
            );
            assert_eq!(
                to_bytes(&sample_epic()).unwrap(),
                to_bytes(&sample_epic()).unwrap()
            );
            assert_eq!(to_bytes(&task).unwrap(), to_bytes(&task).unwrap());
            assert_eq!(to_bytes(&decision).unwrap(), to_bytes(&decision).unwrap());
            assert_eq!(
                to_bytes(&sample_doc()).unwrap(),
                to_bytes(&sample_doc()).unwrap()
            );
            assert_eq!(to_bytes(&nodes).unwrap(), to_bytes(&nodes).unwrap());
        }
        // serialize → load → serialize is identical (spec §8), per node
        // kind — see save_load_save_is_byte_identical_per_node.
        let loaded: Vec<Task> = from_bytes(&to_bytes(&nodes).unwrap()).unwrap();
        assert_eq!(to_bytes(&loaded).unwrap(), to_bytes(&nodes).unwrap());
    }

    #[test]
    fn save_load_save_is_byte_identical_per_node() {
        let repo = sample_repo();
        let reload: Repo = from_bytes(&to_bytes(&repo).unwrap()).unwrap();
        assert_eq!(to_bytes(&reload).unwrap(), to_bytes(&repo).unwrap());

        let task = sample_task();
        let reload: Task = from_bytes(&to_bytes(&task).unwrap()).unwrap();
        assert_eq!(to_bytes(&reload).unwrap(), to_bytes(&task).unwrap());

        let mut epic = sample_epic();
        // Out-of-order insertion after the round trip still yields the
        // canonical bytes.
        epic.references
            .insert(reference(ReferenceKind::Other, "zzz", None))
            .unwrap();
        let reload: Epic = from_bytes(&to_bytes(&epic).unwrap()).unwrap();
        assert_eq!(to_bytes(&reload).unwrap(), to_bytes(&epic).unwrap());
    }

    #[test]
    fn sample_nodes_are_valid_under_the_closure_discipline() {
        assert!(sample_task().validate().is_ok());
        assert!(sample_decision().validate().is_ok());
    }

    #[test]
    fn domain_error_codes_roundtrip_through_serde() {
        let err = DomainError::MissingEvidence {
            id: "camera:U7".to_owned(),
        };
        let bytes = to_bytes(&err).unwrap();
        // Externally tagged: the code spelling leads the payload.
        assert!(bytes.starts_with(b"16:missing_evidence"), "got {bytes:?}");
        assert_eq!(from_bytes::<DomainError>(&bytes).unwrap(), err);
        let code = ErrorCode::OrphanNotAllowed;
        assert_eq!(
            from_bytes::<ErrorCode>(&to_bytes(&code).unwrap()).unwrap(),
            code
        );
    }
}
