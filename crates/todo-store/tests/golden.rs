//! The golden file of the canonical serializer plus its determinism
//! instruments (spec §8, §10;
//! `.agents/knowledge/workflows/golden-files.md`).
//!
//! The fixtures are the store-side twins of the domain crate's
//! `sample_*` set — camera repo, `camera:U7`/`E2`/`D3`/`S1` — so both
//! crates' determinism instruments describe the same shapes.
//! Timestamps and ids are caller-passed constants; nothing reads a
//! clock, and no absolute host path appears in the golden.
//!
//! All tests live in the `golden` module so the tracker's verify line —
//! `cargo test -p todo-store -- golden` — selects every one of them by
//! name.

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

mod common;

mod golden {
    use std::fmt::Debug;
    use std::str::FromStr;

    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use todo_domain::{
        BlocksEdge, Confidence, Date, Decision, DecisionStatus, Doc, Epic, EpicDraft, Graph,
        Priority, Reference, ReferenceKind, References, Repo, RepoSlug, Status, Task, TaskDraft,
        TaskId, TaskKind, Timestamp,
    };
    use todo_store::{FORMAT_VERSION, StoreError, from_bytes, to_bytes};

    use crate::common::assert_golden;

    // =================================================================
    // Fixtures — the twins of todo-domain's `sample_*` set.
    // =================================================================

    fn slug(value: &str) -> RepoSlug {
        RepoSlug::from_str(value).unwrap()
    }

    fn ts(value: &str) -> Timestamp {
        Timestamp::parse(value).unwrap()
    }

    fn reference(kind: ReferenceKind, target: &str, note: Option<&str>) -> Reference {
        Reference::new(kind, target.to_owned(), note.map(str::to_owned)).unwrap()
    }

    fn sample_repo() -> Repo {
        Repo {
            slug: slug("camera"),
            name: "Camera firmware".to_owned(),
            path: "/srv/repos/camera".to_owned(),
            registered_at: ts("2026-09-25T08:00:00Z"),
        }
    }

    fn sample_epic() -> Epic {
        // Inserted out of canonical order (Url before both Trackers),
        // so the golden pins the sorted wire order.
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
            estimate: None,
            // created is second-precision, updated fractional: both must
            // appear in the golden as the 30-byte nine-digit-fraction
            // canonical spelling.
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
        // before Spec by kind), so the golden pins the sort.
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

    /// The golden's bytes: the five fixture nodes, one compact JSON
    /// object per line (JSONL — the event log's line discipline,
    /// human-auditable per §4), in `node.rs` declaration order, LF
    /// endings, exactly one trailing newline.
    fn golden_jsonl() -> Vec<u8> {
        let mut out = Vec::new();
        for line in [
            to_bytes(&sample_repo()).unwrap(),
            to_bytes(&sample_epic()).unwrap(),
            to_bytes(&sample_task()).unwrap(),
            to_bytes(&sample_decision()).unwrap(),
            to_bytes(&sample_doc()).unwrap(),
        ] {
            out.extend_from_slice(&line);
            out.push(b'\n');
        }
        out
    }

    /// A graph with one epic and three tasks, `camera:U1` blocking U2
    /// and U3 — enough topology for §5.2's order to be non-trivial.
    fn seeded_graph() -> Graph {
        let mut graph = Graph::new(sample_repo());
        let epic = graph
            .add_epic(EpicDraft {
                title: "Ship the sensor pipeline".to_owned(),
                description: "Everything from capture to encoding.".to_owned(),
                status: Status::Open,
                references: References::new(),
            })
            .unwrap();
        let at = ts("2026-09-26T12:00:00Z");
        let draft = |title: &str| TaskDraft {
            epic: epic.id.clone(),
            kind: TaskKind::Task,
            title: title.to_owned(),
            body: String::new(),
            status: Status::Open,
            verify: Some("cargo test -p todo-store".to_owned()),
            evidence: None,
            note: None,
            references: References::new(),
            priority: None,
            until: None,
            estimate: None,
        };
        let first = graph.add_task(draft("first"), at.clone()).unwrap();
        let second = graph.add_task(draft("second"), at.clone()).unwrap();
        let third = graph.add_task(draft("third"), at).unwrap();
        graph
            .link_blocks(BlocksEdge::TaskToTask {
                from: first.id.clone(),
                to: second.id.clone(),
            })
            .unwrap();
        graph
            .link_blocks(BlocksEdge::TaskToTask {
                from: first.id,
                to: third.id,
            })
            .unwrap();
        graph
    }

    fn parse_ids<const N: usize>(ids: [&str; N]) -> Vec<TaskId> {
        ids.iter().map(|id| id.parse().unwrap()).collect()
    }

    /// serialize → deserialize → equal value, re-serialize → identical
    /// bytes (spec §8's byte-stability test).
    fn round_trip_is_byte_stable<T>(value: &T)
    where
        T: Serialize + DeserializeOwned + PartialEq + Debug,
    {
        let bytes = to_bytes(value).unwrap();
        let reloaded: T = from_bytes(&bytes).unwrap();
        assert_eq!(&reloaded, value);
        assert_eq!(to_bytes(&reloaded).unwrap(), bytes);
    }

    // =================================================================
    // The golden test.
    // =================================================================

    #[test]
    fn serialize_nodes_matches_its_golden() {
        // The golden name embeds FORMAT_VERSION, so bumping the format
        // without landing the new golden fails as "missing golden".
        assert_golden(
            &format!("serialize_nodes.v{FORMAT_VERSION}"),
            &golden_jsonl(),
        );
    }

    // =================================================================
    // The determinism instruments (the non-golden half).
    // =================================================================

    #[test]
    fn twice_run_serialization_is_deterministic() {
        // Every fixture node, twice-run.
        assert_eq!(
            to_bytes(&sample_repo()).unwrap(),
            to_bytes(&sample_repo()).unwrap()
        );
        assert_eq!(
            to_bytes(&sample_epic()).unwrap(),
            to_bytes(&sample_epic()).unwrap()
        );
        assert_eq!(
            to_bytes(&sample_task()).unwrap(),
            to_bytes(&sample_task()).unwrap()
        );
        assert_eq!(
            to_bytes(&sample_decision()).unwrap(),
            to_bytes(&sample_decision()).unwrap()
        );
        assert_eq!(
            to_bytes(&sample_doc()).unwrap(),
            to_bytes(&sample_doc()).unwrap()
        );

        // A Vec<TaskId> — the shape §5.2 orders and t13's dump prints.
        let ids = parse_ids(["camera:U7", "camera:U3", "camera:U42"]);
        assert_eq!(to_bytes(&ids).unwrap(), to_bytes(&ids).unwrap());

        // A seeded graph's canonical order, twice-run — and pinned to
        // the §5.2 answer itself, so this ties the real serializer to
        // the order the cli dump will print.
        let graph = seeded_graph();
        let order = graph.canonical_order();
        assert_eq!(order, parse_ids(["camera:U1", "camera:U2", "camera:U3"]));
        assert_eq!(to_bytes(&order).unwrap(), to_bytes(&order).unwrap());
    }

    #[test]
    fn fixture_nodes_round_trip_is_byte_stable() {
        round_trip_is_byte_stable(&sample_repo());
        round_trip_is_byte_stable(&sample_epic());
        round_trip_is_byte_stable(&sample_task());
        round_trip_is_byte_stable(&sample_decision());
        round_trip_is_byte_stable(&sample_doc());
    }

    #[test]
    fn lean_task_round_trip_is_byte_stable() {
        // The `None`-optional paths the golden set (Decision 6) leaves
        // out, and references inserted out of canonical order before
        // the trip.
        let mut references = References::new();
        references
            .insert(reference(ReferenceKind::Url, "https://z.example", None))
            .unwrap();
        references
            .insert(reference(ReferenceKind::Doc, "camera:S1", None))
            .unwrap();
        references
            .insert(reference(ReferenceKind::Tracker, "CAM-9", None))
            .unwrap();
        let task = Task {
            id: "camera:U9".parse().unwrap(),
            repo: slug("camera"),
            epic: "camera:E2".parse().unwrap(),
            kind: TaskKind::Debt,
            title: "Record the shortcut".to_owned(),
            body: String::new(),
            status: Status::Open,
            verify: None,
            evidence: None,
            note: None,
            references,
            priority: None,
            until: None,
            estimate: None,
            created: ts("2026-09-26T08:00:00Z"),
            updated: ts("2026-09-26T08:00:00Z"),
        };
        round_trip_is_byte_stable(&task);

        // Out-of-order insertion still lands in the canonical
        // (kind, target, note) order on the wire: tracker < url < doc.
        let bytes = to_bytes(&task).unwrap();
        let json = String::from_utf8_lossy(&bytes);
        let tracker = json.find("\"tracker\"").unwrap();
        let url = json.find("\"url\"").unwrap();
        let doc = json.find("\"doc\"").unwrap();
        assert!(tracker < url && url < doc, "{json}");
    }

    // =================================================================
    // Corruption and validation are named errors, never panics.
    // =================================================================

    #[test]
    fn corrupt_bytes_are_a_named_error() {
        let valid = to_bytes(&sample_task()).unwrap();
        // (input, whether the fault has a position in the bytes).
        let cases: Vec<(Vec<u8>, bool)> = vec![
            // truncated mid-object
            (valid[..valid.len() * 3 / 5].to_vec(), true),
            // truncated mid-string
            (b"{\"title\":\"cut here".to_vec(), true),
            // non-JSON garbage
            (b"not json \x01\x02".to_vec(), true),
            // a wrong-kind document: Repo bytes read as a Task
            (to_bytes(&sample_repo()).unwrap(), false),
        ];
        for (input, positioned) in cases {
            // Constructed twice: the Display must be deterministic.
            let first = from_bytes::<Task>(&input).unwrap_err();
            let second = from_bytes::<Task>(&input).unwrap_err();
            assert!(matches!(first, StoreError::CorruptStore { .. }));
            assert_eq!(first.code(), "corrupt_store");
            assert!(!first.to_string().is_empty());
            assert_eq!(first, second);
            assert_eq!(first.to_string(), second.to_string());
            match (&first, positioned) {
                (
                    StoreError::CorruptStore {
                        offset: Some(_), ..
                    },
                    true,
                ) => {}
                (StoreError::CorruptStore { offset: None, .. }, false) => {}
                _ => panic!("unexpected offset shape for {first}: {input:?}"),
            }
        }
    }

    #[test]
    fn validated_newtypes_reject_bad_spellings_through_the_real_format() {
        // The domain's testcodec rejections, re-run against the real
        // serializer: a bad string is `corrupt_store`, not a value.
        for bad in ["camera:U07", "camera:E7x", "", "camera:U0"] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            let err = from_bytes::<TaskId>(&bytes).unwrap_err();
            assert!(matches!(err, StoreError::CorruptStore { .. }));
            assert_eq!(err.code(), "corrupt_store", "TaskId must reject {bad:?}");
        }
        for bad in ["Bad Slug", "-x", ""] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            let err = from_bytes::<RepoSlug>(&bytes).unwrap_err();
            assert_eq!(err.code(), "corrupt_store", "RepoSlug must reject {bad:?}");
        }
        for bad in [
            "2026-09-26T12:34:56+02:00",
            "2026-09-26t12:34:56z",
            "2026-09-26T12:34:56.1234567890Z",
            "nope",
        ] {
            let bytes = to_bytes(&bad.to_owned()).unwrap();
            let err = from_bytes::<Timestamp>(&bytes).unwrap_err();
            assert_eq!(err.code(), "corrupt_store", "Timestamp must reject {bad:?}");
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn enum_wire_spellings_are_the_section_3_vocabulary() {
        for (status, spelling) in [
            (Status::Open, "open"),
            (Status::InWork, "in_work"),
            (Status::Blocked, "blocked"),
            (Status::Deferred, "deferred"),
            (Status::Closed, "closed"),
            (Status::Wontfix, "wontfix"),
        ] {
            let bytes = to_bytes(&status).unwrap();
            assert_eq!(bytes, format!("\"{spelling}\"").into_bytes());
            assert_eq!(from_bytes::<Status>(&bytes).unwrap(), status);
        }
        for (kind, spelling) in [
            (TaskKind::Task, "task"),
            (TaskKind::Gate, "gate"),
            (TaskKind::Finding, "finding"),
            (TaskKind::Debt, "debt"),
        ] {
            let bytes = to_bytes(&kind).unwrap();
            assert_eq!(bytes, format!("\"{spelling}\"").into_bytes());
            assert_eq!(from_bytes::<TaskKind>(&bytes).unwrap(), kind);
        }
        for (status, spelling) in [
            (DecisionStatus::Open, "open"),
            (DecisionStatus::Signed, "signed"),
            (DecisionStatus::Superseded, "superseded"),
        ] {
            let bytes = to_bytes(&status).unwrap();
            assert_eq!(bytes, format!("\"{spelling}\"").into_bytes());
            assert_eq!(from_bytes::<DecisionStatus>(&bytes).unwrap(), status);
        }
        for (priority, spelling) in [
            (Priority::High, "high"),
            (Priority::Medium, "medium"),
            (Priority::Low, "low"),
        ] {
            let bytes = to_bytes(&priority).unwrap();
            assert_eq!(bytes, format!("\"{spelling}\"").into_bytes());
            assert_eq!(from_bytes::<Priority>(&bytes).unwrap(), priority);
        }
        for (confidence, spelling) in [
            (Confidence::Low, "low"),
            (Confidence::Medium, "medium"),
            (Confidence::High, "high"),
        ] {
            let bytes = to_bytes(&confidence).unwrap();
            assert_eq!(bytes, format!("\"{spelling}\"").into_bytes());
            assert_eq!(from_bytes::<Confidence>(&bytes).unwrap(), confidence);
        }
        for (kind, spelling) in [
            (ReferenceKind::Tracker, "tracker"),
            (ReferenceKind::Url, "url"),
            (ReferenceKind::Doc, "doc"),
            (ReferenceKind::Spec, "spec"),
            (ReferenceKind::File, "file"),
            (ReferenceKind::Other, "other"),
        ] {
            let bytes = to_bytes(&kind).unwrap();
            assert_eq!(bytes, format!("\"{spelling}\"").into_bytes());
            assert_eq!(from_bytes::<ReferenceKind>(&bytes).unwrap(), kind);
        }
    }

    /// The judgment-call (a) pin t10's JSONL line splitter relies on:
    /// non-whitespace trailing bytes after the top-level value are a
    /// positioned parse error; trailing whitespace — including the
    /// line's own terminating newline — parses clean.
    #[test]
    fn trailing_bytes_are_rejected_but_trailing_whitespace_is_not() {
        let valid = to_bytes(&sample_repo()).unwrap();

        let mut garbage = valid.clone();
        garbage.extend_from_slice(b"\"oops\"");
        match from_bytes::<Repo>(&garbage).unwrap_err() {
            StoreError::CorruptStore {
                offset: Some(offset),
                detail,
            } => {
                assert_eq!(offset, valid.len());
                assert!(!detail.is_empty());
            }
            other => panic!("expected a positioned corrupt_store, got {other:?}"),
        }

        for suffix in [b"\n".to_vec(), b" \t\r\n".to_vec()] {
            let mut padded = valid.clone();
            padded.extend_from_slice(&suffix);
            assert!(
                from_bytes::<Repo>(&padded).is_ok(),
                "suffix {suffix:?} must parse clean"
            );
        }
    }
}
