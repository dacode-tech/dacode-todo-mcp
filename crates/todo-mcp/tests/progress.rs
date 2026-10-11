//! The progress subcommand's suite (spec §13, D20, t68 + t69): the
//! render model over the domain graph, the JSON document and the HTML
//! page golden-pinned (`progress.v1`, `page.v1`), the resolution
//! discipline (the registry consult, the lock-free read, the slug
//! cross-check, the cwd tier), the page's switcher and script-guard
//! invariants, the binary's dispatch and write behavior — driven
//! in-process where the surface is a library function and through the
//! real binary where the surface is the process (the cwd tier, the
//! exit codes and the writes, the tests/cwd.rs pattern).
//!
//! The fixture is hand-built through the store's public API under a
//! temp root with fixed caller-passed timestamps — fully
//! deterministic, which is what makes the golden honest — and shaped
//! to exercise every derivation rule: two milestone-prefixed epics
//! plus the synthetic container, tasks of all four kinds with mixed
//! statuses, a decision gating an epic, one gating a task, one gating
//! nothing (unplaced, reported), and task→task edges. Every store
//! handle is dropped before the binary spawns (the advisory lock is
//! per process).

//! Every test lists as `progress::…` so the module-path filter
//! `cargo test -p todo-mcp --test progress -- progress` selects
//! exactly this suite (P-LIVE-002/P-LIVE-005: quote the counts beside
//! the filter).
//!

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
mod progress {

    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::str::FromStr;

    use serde_json::{Value, json};
    use tempfile::TempDir;
    use todo_domain::{
        BlocksEdge, Confidence, DecisionDraft, DecisionStatus, EpicDraft, References, Repo,
        RepoSlug, Status, TaskDraft, TaskKind, Timestamp,
    };
    use todo_mcp::render;
    use todo_store::{DataRoot, Store};

    use crate::common::assert_golden;

    /// The render fixture's slug.
    const FIXTURE: &str = "fixture";
    /// The second repo's slug — it sorts before `fixture`, so the cwd
    /// tier's match has a real alternative to lose to.
    const ALPHA: &str = "alpha";

    /// A fixed canonical timestamp.
    fn ts(value: &str) -> Timestamp {
        Timestamp::parse(value).expect("a fixed canonical timestamp")
    }

    /// The fixture repo's slug, parsed.
    fn fixture_slug() -> RepoSlug {
        RepoSlug::from_str(FIXTURE).expect("a valid slug")
    }

    /// Builds the render fixture under `root` with `checkout` as the
    /// registered path:
    ///
    /// - **E1 "Milestone zero"** (closed, the provenance description M0)
    ///   holds the done pair — U1 a closed task, U2 a closed gate.
    /// - **E2 "Milestone one"** (open, M1) is gated by the open decision
    ///   D1, so every non-done task under it waits on D1: U3 open, U4
    ///   in_work, U5 a finding, U6 deferred debt, U7 blocked. D2 gates U3
    ///   directly — a decision placed by its gated task's epic.
    /// - **E3 "Follow-ups"** (the synthetic container's description) holds
    ///   U8 open, U9 wontfix, U10 an open gate; the signed D3 gates U8.
    /// - **D4** gates nothing — unplaced, reported, never dropped.
    /// - Edges: D1→E2, D2→U3, D3→U8, U1→U3, U8→U10.
    fn build_render_fixture(root: &DataRoot, checkout: &str) {
        let mut store = root
            .register(Repo {
                slug: RepoSlug::from_str(FIXTURE).expect("a valid slug"),
                name: "Fixture".to_owned(),
                path: checkout.to_owned(),
                registered_at: ts("2026-10-02T08:00:00Z"),
            })
            .expect("the fixture repo registers");

        let m0 = store
            .add_epic(
                EpicDraft {
                    title: "Milestone zero".to_owned(),
                    description: "the fixture board's §2 milestone M0".to_owned(),
                    status: Status::Closed,
                    references: References::new(),
                },
                ts("2026-10-02T08:01:00Z"),
            )
            .expect("E1 lands");
        let m1 = store
            .add_epic(
                EpicDraft {
                    title: "Milestone one".to_owned(),
                    description: "the fixture board's §2 milestone M1".to_owned(),
                    status: Status::Open,
                    references: References::new(),
                },
                ts("2026-10-02T08:02:00Z"),
            )
            .expect("E2 lands");
        let follow_ups = store
            .add_epic(
                EpicDraft {
                    title: "Follow-ups".to_owned(),
                    description: "the fixture board's §6 open follow-ups, not tied to a milestone"
                        .to_owned(),
                    status: Status::Open,
                    references: References::new(),
                },
                ts("2026-10-02T08:03:00Z"),
            )
            .expect("E3 lands");

        let d1 = store
            .add_decision(
                DecisionDraft {
                    title: "Gate milestone one".to_owned(),
                    status: DecisionStatus::Open,
                    options: vec!["(a) go".to_owned(), "(b) hold".to_owned()],
                    recommendation: Some("(a)".to_owned()),
                    resolution: None,
                    confidence: Confidence::Medium,
                    references: References::new(),
                },
                ts("2026-10-02T08:04:00Z"),
            )
            .expect("D1 lands");
        let d2 = store
            .add_decision(
                DecisionDraft {
                    title: "Gate the wiring task".to_owned(),
                    status: DecisionStatus::Open,
                    options: vec!["(a) wire".to_owned(), "(b) wait".to_owned()],
                    recommendation: Some("(a)".to_owned()),
                    resolution: None,
                    confidence: Confidence::Low,
                    references: References::new(),
                },
                ts("2026-10-02T08:05:00Z"),
            )
            .expect("D2 lands");
        let d3 = store
            .add_decision(
                DecisionDraft {
                    title: "The encoder profile".to_owned(),
                    status: DecisionStatus::Signed,
                    options: vec!["(a) baseline".to_owned(), "(b) main".to_owned()],
                    recommendation: Some("(a)".to_owned()),
                    resolution: Some("signed (a) 2026-10-02".to_owned()),
                    confidence: Confidence::High,
                    references: References::new(),
                },
                ts("2026-10-02T08:06:00Z"),
            )
            .expect("D3 lands");
        store
            .add_decision(
                DecisionDraft {
                    title: "A decision gating nothing".to_owned(),
                    status: DecisionStatus::Open,
                    options: vec!["(a) something".to_owned(), "(b) nothing".to_owned()],
                    recommendation: None,
                    resolution: None,
                    confidence: Confidence::Medium,
                    references: References::new(),
                },
                ts("2026-10-02T08:07:00Z"),
            )
            .expect("D4 lands");

        // The add instant of task number `n`, spread across days so the
        // census and the day histogram have a real shape — including
        // the zero-event day 2026-10-04 the continuous timeline renders
        // as a zero-height bar: U1–U2 on the 2nd, U3–U5 on the 3rd,
        // U6–U8 on the 5th, U9–U10 on the 6th; the edges land on the
        // 7th. Fixed, caller-passed, distinct per task.
        let at = |n: u64| {
            let day: u64 = match n {
                1 | 2 => 2,
                3..=5 => 3,
                6..=8 => 5,
                _ => 6,
            };
            ts(&format!("2026-10-{day:02}T08:10:{n:02}Z"))
        };
        let draft =
            |epic: &todo_domain::EpicId, title: &str, kind: TaskKind, status: Status| TaskDraft {
                epic: epic.clone(),
                kind,
                title: title.to_owned(),
                body: String::new(),
                status,
                verify: Some("cargo test -p fixture".to_owned()),
                evidence: None,
                note: None,
                references: References::new(),
                priority: None,
                until: None,
            };

        let u1 = store
            .add_task(
                TaskDraft {
                    evidence: Some("the milestone closed green".to_owned()),
                    ..draft(
                        &m0.id,
                        "Close the first milestone",
                        TaskKind::Task,
                        Status::Closed,
                    )
                },
                at(1),
            )
            .expect("U1 lands");
        store
            .add_task(
                TaskDraft {
                    evidence: Some("the gate ran green".to_owned()),
                    ..draft(&m0.id, "The zero gate", TaskKind::Gate, Status::Closed)
                },
                at(2),
            )
            .expect("U2 lands");
        let u3 = store
            .add_task(
                draft(&m1.id, "Wire the sensor", TaskKind::Task, Status::Open),
                at(3),
            )
            .expect("U3 lands");
        store
            .add_task(
                draft(&m1.id, "Calibrate", TaskKind::Task, Status::InWork),
                at(4),
            )
            .expect("U4 lands");
        store
            .add_task(
                draft(&m1.id, "A finding", TaskKind::Finding, Status::Open),
                at(5),
            )
            .expect("U5 lands");
        store
            .add_task(
                TaskDraft {
                    note: Some("after milestone zero".to_owned()),
                    ..draft(&m1.id, "Known debt", TaskKind::Debt, Status::Deferred)
                },
                at(6),
            )
            .expect("U6 lands");
        store
            .add_task(
                TaskDraft {
                    note: Some("needs D2".to_owned()),
                    ..draft(&m1.id, "The blocked one", TaskKind::Task, Status::Blocked)
                },
                at(7),
            )
            .expect("U7 lands");
        let u8 = store
            .add_task(
                draft(&follow_ups.id, "A follow-up", TaskKind::Task, Status::Open),
                at(8),
            )
            .expect("U8 lands");
        store
            .add_task(
                draft(&follow_ups.id, "Declined", TaskKind::Task, Status::Wontfix),
                at(9),
            )
            .expect("U9 lands");
        let u10 = store
            .add_task(
                draft(
                    &follow_ups.id,
                    "The next gate",
                    TaskKind::Gate,
                    Status::Open,
                ),
                at(10),
            )
            .expect("U10 lands");

        store
            .link_blocks(
                BlocksEdge::DecisionToEpic {
                    from: d1.id.clone(),
                    to: m1.id.clone(),
                },
                ts("2026-10-07T08:20:01Z"),
            )
            .expect("D1 gates E2");
        store
            .link_blocks(
                BlocksEdge::DecisionToTask {
                    from: d2.id.clone(),
                    to: u3.id.clone(),
                },
                ts("2026-10-07T08:20:02Z"),
            )
            .expect("D2 gates U3");
        store
            .link_blocks(
                BlocksEdge::DecisionToTask {
                    from: d3.id.clone(),
                    to: u8.id.clone(),
                },
                ts("2026-10-07T08:20:03Z"),
            )
            .expect("D3 gates U8");
        store
            .link_blocks(
                BlocksEdge::TaskToTask {
                    from: u1.id.clone(),
                    to: u3.id.clone(),
                },
                ts("2026-10-07T08:20:04Z"),
            )
            .expect("U1 blocks U3");
        store
            .link_blocks(
                BlocksEdge::TaskToTask {
                    from: u8.id.clone(),
                    to: u10.id.clone(),
                },
                ts("2026-10-07T08:20:05Z"),
            )
            .expect("U8 blocks U10");
    }

    /// The render fixture under a fresh temp root, registered at a
    /// placeholder path — the in-process tests never match a cwd against
    /// it.
    fn render_fixture() -> TempDir {
        let root = TempDir::new().expect("a fresh temp root");
        build_render_fixture(&DataRoot::new(root.path()), "/repos/fixture");
        root
    }

    /// A minimal second repo under `root` at `checkout` — one epic, one
    /// open task, so a render of it has an epic and the cwd tier's match
    /// has a real alternative.
    fn build_alpha(root: &DataRoot, checkout: &str) {
        let mut store = root
            .register(Repo {
                slug: RepoSlug::from_str(ALPHA).expect("a valid slug"),
                name: "Alpha".to_owned(),
                path: checkout.to_owned(),
                registered_at: ts("2026-10-02T09:00:00Z"),
            })
            .expect("the alpha repo registers");
        let epic = store
            .add_epic(
                EpicDraft {
                    title: "Alpha work".to_owned(),
                    description: "The alpha epic.".to_owned(),
                    status: Status::Open,
                    references: References::new(),
                },
                ts("2026-10-02T09:01:00Z"),
            )
            .expect("alpha's epic lands");
        store
            .add_task(
                TaskDraft {
                    epic: epic.id.clone(),
                    kind: TaskKind::Task,
                    title: "An alpha task".to_owned(),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test -p alpha".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                ts("2026-10-02T09:02:00Z"),
            )
            .expect("alpha's task lands");
    }

    /// Runs the binary with `args` in `cwd`, answering its exit code,
    /// stdout and stderr.
    fn run(cwd: &Path, args: &[&str]) -> (Option<i32>, String, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("the binary spawns");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// The JSON document over the fixture, rendered in-process.
    fn fixture_document(root: &TempDir) -> String {
        render::json(&DataRoot::new(root.path()), &fixture_slug()).expect("the render succeeds")
    }

    /// A registered repo whose graph holds no epics is a refusal — the
    /// renderer cannot pass by reading nothing (the Python tool's own
    /// rule, ported).
    #[test]
    fn a_graph_with_no_epics_is_refused() {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        data_root
            .register(Repo {
                slug: fixture_slug(),
                name: "Fixture".to_owned(),
                path: "/repos/fixture".to_owned(),
                registered_at: ts("2026-10-02T08:00:00Z"),
            })
            .expect("the repo registers");
        let message =
            render::json(&data_root, &fixture_slug()).expect_err("the epic-less graph is refused");
        assert_eq!(
            message,
            "the graph holds no epics — refusing to render a roadmap from nothing"
        );
    }

    /// The golden pin: the fixture's document, byte-identical to
    /// `tests/golden/progress.v1.golden`.
    #[test]
    fn the_json_document_is_golden_pinned() {
        let root = render_fixture();
        assert_golden("progress.v1", fixture_document(&root).as_bytes());
    }

    /// Two renders over unchanged state are byte-identical (§8) — the
    /// in-process twin of the golden's regeneration ritual.
    #[test]
    fn two_renders_are_byte_identical() {
        let root = render_fixture();
        assert_eq!(fixture_document(&root), fixture_document(&root));
    }

    /// The f110 scenario at the subcommand level: a render succeeds while
    /// the store's lock is genuinely held — the locked writer open beside
    /// the lock-free read, one process, two file descriptions.
    #[test]
    fn a_render_succeeds_while_the_store_lock_is_held() {
        let root = render_fixture();
        let data_root = DataRoot::new(root.path());
        let _holder = data_root
            .open(&fixture_slug())
            .expect("the locked open succeeds");
        let document = render::json(&data_root, &fixture_slug())
            .expect("the lock-free render succeeds beside the held lock");
        assert!(document.contains("\"repo\": \"fixture\""));
    }

    /// The registry consult comes before the read: an unlisted slug is
    /// `unknown_id`'s message, never a raw io.
    #[test]
    fn an_unregistered_repo_is_refused() {
        let root = render_fixture();
        let data_root = DataRoot::new(root.path());
        let slug = RepoSlug::from_str("elsewhere").expect("a valid slug");
        let message =
            render::json(&data_root, &slug).expect_err("the unregistered slug is refused");
        assert_eq!(message, "unknown id: elsewhere");
    }

    /// A store directory holding another repo's graph under the asked slug
    /// is the corrupt_store refusal — the same message the locked open
    /// produces, never the copied graph served as the named repo.
    #[test]
    fn a_copied_store_is_refused() {
        let root = render_fixture();
        let data_root = DataRoot::new(root.path());
        build_alpha(&data_root, "/repos/alpha");
        let alpha_log = root
            .path()
            .join("repos")
            .join(ALPHA)
            .join("event-log.jsonl");
        let fixture_log = root
            .path()
            .join("repos")
            .join(FIXTURE)
            .join("event-log.jsonl");
        fs::copy(&alpha_log, &fixture_log).expect("the store directory is copied");
        let message =
            render::json(&data_root, &fixture_slug()).expect_err("the copied store is refused");
        assert_eq!(
            message,
            "corrupt store: the store opened for fixture holds repo alpha"
        );
    }

    /// The model's derivations, asserted on the parsed document: the
    /// per-kind counts, the open items' order and waits, the milestones,
    /// the unplaced report, and the graph's documented total orders.
    #[test]
    fn the_document_carries_the_derived_model() {
        let root = render_fixture();
        let document = fixture_document(&root);
        let value: Value = serde_json::from_str(&document).expect("the document is JSON");

        assert_eq!(value["repo"], json!("fixture"));
        assert_eq!(
            value["import_instant"],
            json!("2026-10-02T08:00:00.000000000Z")
        );
        assert_eq!(
            value["totals"]["by_kind"]["task"],
            json!({"total": 6, "done": 2, "open": 4})
        );
        assert_eq!(
            value["totals"]["by_kind"]["gate"],
            json!({"total": 2, "done": 1, "open": 1})
        );
        assert_eq!(
            value["totals"]["by_kind"]["finding"],
            json!({"total": 1, "done": 0, "open": 1})
        );
        assert_eq!(
            value["totals"]["by_kind"]["debt"],
            json!({"total": 1, "done": 0, "open": 1})
        );
        assert_eq!(
            value["totals"]["open_items"],
            json!(["U3", "U4", "U5", "U6", "U7", "U8", "U10"])
        );
        assert_eq!(
            value["totals"]["milestones"],
            json!({"done": 1, "total": 2})
        );
        assert_eq!(
            value["totals"]["decisions"],
            json!({"signed": 1, "total": 4})
        );
        assert_eq!(
            value["totals"]["not_placed"],
            json!(["D4 (a decision gating nothing)"])
        );
        assert_eq!(
            value["milestones"],
            json!([
                {"id": "M0", "title": "Milestone zero", "status": "closed"},
                {"id": "M1", "title": "Milestone one", "status": "open"}
            ])
        );
        // U3 waits on D1 — the open decision gating its epic.
        let u3 = value["open_items"]
            .as_array()
            .expect("open_items is an array")
            .iter()
            .find(|item| item["id"] == json!("U3"))
            .expect("U3 is an open item");
        assert_eq!(u3["waits_on"], json!("D1"));
        // D1 gates the epic; D2 gates the task; D4 gates nothing.
        assert_eq!(value["decisions"][0]["gates"], json!(["E2"]));
        assert_eq!(value["decisions"][1]["gates"], json!(["U3"]));
        assert_eq!(value["decisions"][3]["gates"], json!([]));
        // The graph: 13 placed nodes, 5 edges, the documented orders —
        // decision-sourced first (decision ascending), then task-sourced.
        assert_eq!(value["graph"]["nodes"].as_array().expect("nodes").len(), 13);
        assert_eq!(
            value["graph"]["edges"],
            json!([
                {"from": "D1", "to": "E2"},
                {"from": "D2", "to": "U3"},
                {"from": "D3", "to": "U8"},
                {"from": "U1", "to": "U3"},
                {"from": "U8", "to": "U10"}
            ])
        );
    }

    /// The summary narrates the same model the JSON pins — it can never
    /// disagree with the document.
    #[test]
    fn the_summary_prints_the_derived_totals() {
        let root = render_fixture();
        let text = render::summary(&DataRoot::new(root.path()), &fixture_slug())
            .expect("the summary succeeds");
        assert!(text.contains("imported 2026-10-02T08:00:00.000000000Z (10 tasks, 4 decisions)"));
        assert!(text.contains("milestones 1/2 done"));
        assert!(text.contains("decisions 1/4 signed"));
        assert!(text.contains("open items: U3, U4, U5, U6, U7, U8, U10"));
        assert!(text.contains("graph: 13 nodes, 5 edges"));
        assert!(text.contains("NOT PLACED (reported, never dropped): D4"));
    }

    /// The `--repo` flag outranks the cwd tier: run from `fixture`'s
    /// checkout with `--repo alpha`, the document renders alpha.
    #[test]
    fn the_repo_flag_outranks_the_cwd() {
        let root = TempDir::new().expect("a fresh temp root");
        let first = TempDir::new().expect("the fixture checkout");
        let second = TempDir::new().expect("the alpha checkout");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &first.path().display().to_string());
        build_alpha(&data_root, &second.path().display().to_string());

        let (code, stdout, stderr) = run(
            first.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--repo",
                ALPHA,
                "--json",
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert!(stdout.contains("\"repo\": \"alpha\""), "stdout: {stdout}");
    }

    /// The cwd tier selects the repo: run from `fixture`'s checkout with
    /// no `--repo`, the document renders fixture.
    #[test]
    fn the_cwd_tier_selects_the_repo() {
        let root = TempDir::new().expect("a fresh temp root");
        let first = TempDir::new().expect("the fixture checkout");
        let second = TempDir::new().expect("the alpha checkout");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &first.path().display().to_string());
        build_alpha(&data_root, &second.path().display().to_string());

        let (code, stdout, stderr) = run(
            first.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--json",
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert!(stdout.contains("\"repo\": \"fixture\""), "stdout: {stdout}");
    }

    /// No `--repo` and a cwd matching nothing is a refusal (exit 1), not
    /// an empty document and not a usage fault — the environment did not
    /// produce a repo.
    #[test]
    fn no_default_is_a_refusal() {
        let root = TempDir::new().expect("a fresh temp root");
        let first = TempDir::new().expect("the fixture checkout");
        let second = TempDir::new().expect("the alpha checkout");
        let elsewhere = TempDir::new().expect("an unregistered directory");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &first.path().display().to_string());
        build_alpha(&data_root, &second.path().display().to_string());

        let (code, stdout, stderr) = run(
            elsewhere.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--json",
            ],
        );
        assert_eq!(code, Some(1));
        assert!(stdout.is_empty(), "stdout: {stdout}");
        assert!(
            stderr.contains("no --repo and the working directory matches no registered checkout"),
            "stderr: {stderr}"
        );
    }

    /// The render fixture plus alpha, so the page's switcher renders a
    /// real choice — the golden's fixture.
    fn switcher_fixture() -> TempDir {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, "/repos/fixture");
        build_alpha(&data_root, "/repos/alpha");
        root
    }

    /// The fixture's page, rendered in-process.
    fn fixture_page(root: &TempDir) -> String {
        render::page(&DataRoot::new(root.path()), &fixture_slug()).expect("the render succeeds")
    }

    /// The page golden: the fixture's page (with alpha registered, so
    /// the switcher carries two entries), byte-identical to
    /// `tests/golden/page.v1.golden`.
    #[test]
    fn the_page_is_golden_pinned() {
        let root = switcher_fixture();
        assert_golden("page.v1", fixture_page(&root).as_bytes());
    }

    /// Two page renders over unchanged state are byte-identical (§8) —
    /// the in-process twin of the golden's regeneration ritual.
    #[test]
    fn two_page_renders_are_byte_identical() {
        let root = switcher_fixture();
        assert_eq!(fixture_page(&root), fixture_page(&root));
    }

    /// The switcher: every registered slug appears, the rendered repo
    /// as the marked non-link, the others as `?repo=<slug>` links —
    /// the URL shape the `serve` subcommand answers (the golden pins
    /// the full nav).
    #[test]
    fn the_switcher_lists_the_registered_repos() {
        let root = switcher_fixture();
        let page = fixture_page(&root);
        assert!(
            page.contains(r#"<a href="?repo=alpha">alpha</a>"#),
            "the switcher links alpha: {}",
            &page[..page.len().min(400)]
        );
        assert!(
            page.contains(r#"<span class="cur">fixture</span>"#),
            "the rendered repo is the marked non-link"
        );
    }

    /// The switcher is a registry nav, not a render: a registered repo
    /// whose store will not read still appears — the switcher itself
    /// pays no per-repo read (it renders from the sidebar's registry
    /// walk), and clicking the slug under `serve` answers the read
    /// error (the spec's veto-able reading (iii), pinned). The
    /// sidebar's per-repo reads (D3) do pay it, and report the same
    /// repo with the unreadable marker — the test below this one.
    #[test]
    fn the_switcher_lists_slugs_whose_stores_will_not_read() {
        let root = switcher_fixture();
        let alpha_log = root
            .path()
            .join("repos")
            .join(ALPHA)
            .join("event-log.jsonl");
        std::fs::write(&alpha_log, "not a log\n").expect("alpha's store is corrupted");
        let page = fixture_page(&root);
        assert!(
            page.contains(r#"<a href="?repo=alpha">alpha</a>"#),
            "the unreadable repo still appears in the nav"
        );
    }

    /// The sidebar (t84's D3/D4): every registered repo present with
    /// its epics — ascending epic id, the model's lane order — the
    /// rendered repo marked as the non-link, the other repo's name
    /// linking to `?repo=<slug>`, the epic entries linking to the lane
    /// anchors (same-page for the rendered repo, repo+anchor for the
    /// others) and carrying their done/total task counts.
    #[test]
    fn the_sidebar_lists_every_repo_with_its_epics() {
        let root = switcher_fixture();
        let page = fixture_page(&root);
        // The rendered repo: the marked non-link, its three epics
        // ascending with their counts.
        assert!(
            page.contains(r#"<span class="sname scur">Fixture</span>"#),
            "the rendered repo is the marked non-link"
        );
        let e1 = page
            .find(r##"<a href="#lane-1">E1 · Milestone zero</a><span>2/2</span>"##)
            .expect("E1's row with its count");
        let e2 = page
            .find(r##"<a href="#lane-2">E2 · Milestone one</a><span>0/5</span>"##)
            .expect("E2's row with its count");
        let e3 = page
            .find(r##"<a href="#lane-3">E3 · Follow-ups</a><span>1/3</span>"##)
            .expect("E3's row with its count");
        assert!(
            e1 < e2 && e2 < e3,
            "the epics are ascending epic id (the lane order)"
        );
        // The other repo: its name links, its epic links carry the
        // repo, and the SVG lanes carry the anchors the links target.
        assert!(
            page.contains(r#"<a class="sname" href="?repo=alpha">Alpha</a>"#),
            "the other repo's name links to its render"
        );
        assert!(
            page.contains(r#"<a href="?repo=alpha#lane-1">E1 · Alpha work</a><span>0/1</span>"#),
            "the other repo's epic links to its lane anchor"
        );
        assert!(
            page.contains(r##"<text id="lane-1" class="lane""##),
            "the SVG lane labels carry the lane anchors"
        );
    }

    /// The sidebar's anti-silent-omission rule (D3): a repo whose graph
    /// will not read — a corrupted log, or a store holding another
    /// repo's graph under the slug (the resolve cross-check one level
    /// out) — is reported with the unreadable marker and no epics,
    /// never dropped from the nav.
    #[test]
    fn an_unreadable_repo_is_reported_never_dropped() {
        let root = switcher_fixture();
        let alpha_log = root
            .path()
            .join("repos")
            .join(ALPHA)
            .join("event-log.jsonl");
        std::fs::write(&alpha_log, "not a log\n").expect("alpha's store is corrupted");
        let page = fixture_page(&root);
        assert!(
            page.contains(r#"<a class="sname" href="?repo=alpha">alpha</a>"#),
            "the unreadable repo's row is present, its name fallen back to the slug"
        );
        assert!(
            page.contains(r#"<span class="unread">unreadable</span>"#),
            "the unreadable marker is visible"
        );
        assert!(
            !page.contains("?repo=alpha#lane-"),
            "an unreadable repo shows no epic rows"
        );

        // The copied store: alpha's directory holding fixture's graph
        // under alpha's slug — the marker, never the foreign epics.
        let root = switcher_fixture();
        let alpha_log = root
            .path()
            .join("repos")
            .join(ALPHA)
            .join("event-log.jsonl");
        let fixture_log = root
            .path()
            .join("repos")
            .join(FIXTURE)
            .join("event-log.jsonl");
        fs::copy(&fixture_log, &alpha_log).expect("the store directory is copied");
        let page = fixture_page(&root);
        assert!(
            page.contains(r#"<span class="unread">unreadable</span>"#),
            "the foreign graph under the slug reads as unreadable"
        );
        assert!(
            !page.contains("?repo=alpha#lane-"),
            "the foreign epics are never displayed under the wrong slug"
        );
    }

    /// The activity panel (t84's D5): the census rendered — the header
    /// with the total and the first event's instant, the by-kind chips
    /// (alphabetical), the day histogram with one bar per day from
    /// first to last (the zero-event 2026-10-04 as a zero-height bar —
    /// a continuous timeline), and the last-activity line.
    #[test]
    fn the_activity_panel_renders_the_census() {
        let root = switcher_fixture();
        let page = fixture_page(&root);
        assert!(
            page.contains("<h2>Activity — 23 events since 2026-10-02T08:00:00.000000000Z</h2>"),
            "the header carries the total and the seed's instant"
        );
        assert!(
            page.contains(
                "<span class=\"chip\">blocks_link <b>5</b></span><span class=\"chip\">\
                 decision_add <b>4</b></span><span class=\"chip\">epic_add <b>3</b></span>\
                 <span class=\"chip\">repo_register <b>1</b></span><span class=\"chip\">\
                 task_add <b>10</b></span>"
            ),
            "the by-kind chips are alphabetical with their counts"
        );
        assert!(
            page.contains(
                "<div class=\"hday\" title=\"2026-10-02: 10 events\">\
                 <span class=\"hnum\">10</span>\
                 <span class=\"hbar\" style=\"height:64px\"></span>\
                 <span class=\"hdate\">2026-10-02</span></div>"
            ),
            "the busiest day is the capped max bar"
        );
        assert!(
            page.contains(
                "<div class=\"hday\" title=\"2026-10-04: 0 events\">\
                 <span class=\"hnum\">0</span>\
                 <span class=\"hbar\" style=\"height:0px\"></span>\
                 <span class=\"hdate\">2026-10-04</span></div>"
            ),
            "the zero-event day is a zero-height bar, not a gap"
        );
        assert_eq!(
            page.matches("class=\"hday\"").count(),
            6,
            "one bar per day from first to last"
        );
        assert!(
            page.contains(
                "<div class=\"lastact\">Last activity: 2026-10-07T08:20:05.000000000Z</div>"
            ),
            "the last-activity line carries the last event's instant"
        );
    }

    /// The JSON's `activity` object (t84's D6): the census verbatim —
    /// the page's activity panel's machine-readable twin.
    #[test]
    fn the_json_document_carries_the_activity_twin() {
        let root = switcher_fixture();
        let document = fixture_document(&root);
        let value: Value = serde_json::from_str(&document).expect("the document is JSON");
        assert_eq!(value["activity"]["total"], json!(23));
        assert_eq!(
            value["activity"]["by_kind"],
            json!({
                "blocks_link": 5,
                "decision_add": 4,
                "epic_add": 3,
                "repo_register": 1,
                "task_add": 10
            })
        );
        assert_eq!(
            value["activity"]["by_day"],
            json!({
                "2026-10-02": 10,
                "2026-10-03": 3,
                "2026-10-05": 3,
                "2026-10-06": 2,
                "2026-10-07": 5
            })
        );
        assert_eq!(
            value["activity"]["first_ts"],
            json!("2026-10-02T08:00:00.000000000Z")
        );
        assert_eq!(
            value["activity"]["last_ts"],
            json!("2026-10-07T08:20:05.000000000Z")
        );
    }

    /// A write fault is exit 1 with the message on stderr (D9): `--out`
    /// at an existing file cannot create the output directory — an
    /// environment fault, never a silent success.
    #[test]
    fn a_write_fault_is_exit_1() {
        let root = TempDir::new().expect("a fresh temp root");
        let checkout = TempDir::new().expect("the fixture checkout");
        let blocker = TempDir::new().expect("the blocker's directory");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &checkout.path().display().to_string());
        let blocked_out = blocker.path().join("not-a-directory");
        std::fs::write(&blocked_out, b"a file").expect("the blocker exists");

        let (code, stdout, stderr) = run(
            checkout.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--out",
                blocked_out.to_str().expect("a UTF-8 out path"),
            ],
        );
        assert_eq!(code, Some(1));
        // The summary prints before the write, so it is already on
        // stdout — the failure is the exit code and the stderr
        // message, and the wrote line never appears.
        assert!(stdout.contains("progress — imported"), "stdout: {stdout}");
        assert!(!stdout.contains("wrote "), "stdout: {stdout}");
        assert!(
            stderr.contains("create the output directory"),
            "stderr: {stderr}"
        );
    }

    /// The default write: the pair lands in the rendered repo's
    /// registered checkout `.agents/cache` with exactly the in-process
    /// render's bytes — both when the cwd tier resolves the repo (run
    /// from the checkout) and when `--repo` names it from an unrelated
    /// cwd, which is the run that distinguishes the registered
    /// checkout from the process's working directory. stdout carries
    /// the summary and the wrote line.
    #[test]
    fn the_default_run_writes_the_pair_beside_the_checkout() {
        let root = TempDir::new().expect("a fresh temp root");
        let checkout = TempDir::new().expect("the fixture checkout");
        let elsewhere = TempDir::new().expect("an unrelated cwd");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &checkout.path().display().to_string());

        // From the checkout: the cwd tier resolves the repo and the
        // default write composes with it.
        let (code, stdout, stderr) = run(
            checkout.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert!(stdout.contains("progress — imported"), "stdout: {stdout}");
        // The wrote line names both files of the pair (D7's shape).
        assert!(
            stdout.contains("progress.html and "),
            "the wrote line names the page: {stdout}"
        );
        assert!(
            stdout.contains("progress.json"),
            "the wrote line names the JSON twin: {stdout}"
        );

        // From an unrelated cwd with --repo: the pair still lands in
        // the REGISTERED checkout's cache, never the cwd's.
        let (code, stdout, stderr) = run(
            elsewhere.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--repo",
                FIXTURE,
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert!(stdout.contains("wrote "), "stdout: {stdout}");
        assert!(
            !elsewhere.path().join(".agents").exists(),
            "nothing lands in the unrelated cwd"
        );

        let cache = checkout.path().join(".agents").join("cache");
        let page =
            std::fs::read(cache.join("progress.html")).expect("the page lands beside the checkout");
        let document = std::fs::read(cache.join("progress.json"))
            .expect("the JSON twin lands beside the checkout");
        assert_eq!(
            page,
            render::page(&data_root, &fixture_slug())
                .expect("the in-process page")
                .as_bytes()
        );
        assert_eq!(
            document,
            render::json(&data_root, &fixture_slug())
                .expect("the in-process document")
                .as_bytes()
        );
    }

    /// `--out DIR` redirects the pair; nothing lands beside the
    /// checkout.
    #[test]
    fn the_out_flag_redirects_the_pair() {
        let root = TempDir::new().expect("a fresh temp root");
        let checkout = TempDir::new().expect("the fixture checkout");
        let out = TempDir::new().expect("the redirected output");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &checkout.path().display().to_string());

        let (code, stdout, stderr) = run(
            checkout.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--out",
                out.path().to_str().expect("a UTF-8 out dir"),
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert!(stdout.contains("wrote "), "stdout: {stdout}");
        assert!(
            out.path().join("progress.html").exists() && out.path().join("progress.json").exists(),
            "the pair lands in the --out directory"
        );
        assert!(
            !checkout.path().join(".agents").exists(),
            "nothing lands beside the checkout"
        );
    }

    /// `--no-write` prints the summary only — no files anywhere.
    #[test]
    fn no_write_prints_the_summary_only() {
        let root = TempDir::new().expect("a fresh temp root");
        let checkout = TempDir::new().expect("the fixture checkout");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &checkout.path().display().to_string());

        let (code, stdout, stderr) = run(
            checkout.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--no-write",
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert!(stdout.contains("progress — imported"), "stdout: {stdout}");
        assert!(!stdout.contains("wrote "), "stdout: {stdout}");
        assert!(
            !checkout.path().join(".agents").exists(),
            "no files land beside the checkout"
        );
    }

    /// `--json` prints the document and writes nothing (the Python
    /// tool's "instead of writing" contract).
    #[test]
    fn json_prints_the_document_and_writes_nothing() {
        let root = TempDir::new().expect("a fresh temp root");
        let checkout = TempDir::new().expect("the fixture checkout");
        let data_root = DataRoot::new(root.path());
        build_render_fixture(&data_root, &checkout.path().display().to_string());

        let (code, stdout, stderr) = run(
            checkout.path(),
            &[
                "progress",
                "--root",
                root.path().to_str().expect("a UTF-8 root"),
                "--json",
            ],
        );
        assert_eq!(code, Some(0), "stderr: {stderr}");
        assert_eq!(
            stdout,
            render::json(&data_root, &fixture_slug()).expect("the in-process document")
        );
        assert!(
            !checkout.path().join(".agents").exists(),
            "no files land beside the checkout"
        );
    }

    /// A hostile title cannot break out of the script element: the
    /// `</` guard turns the DATA embed's `</script>` into `<\/script>`
    /// (a valid JSON escape), so the page's raw `</script>` count is
    /// exactly one — the element's own closer. The specials title
    /// renders escaped everywhere it appears.
    #[test]
    fn a_hostile_title_cannot_break_out_of_the_script_element() {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        build_hostile(&data_root);
        let slug = RepoSlug::from_str("hostile").expect("a valid slug");
        let page = render::page(&data_root, &slug).expect("the hostile page renders");
        assert_eq!(page.matches("</script>").count(), 1);
        assert!(
            page.contains(r#""title":"<\/script>""#),
            "the DATA embed guards the closing sequence"
        );
        assert!(
            page.contains("&lt;b&gt;&amp;&quot;"),
            "the specials title renders escaped"
        );
    }

    /// The hostile fixture: one epic, one task titled `</script>`, one
    /// titled with HTML specials — separate from the render fixture so
    /// its golden never grows hostile bytes.
    fn build_hostile(root: &DataRoot) {
        let mut store = root
            .register(Repo {
                slug: RepoSlug::from_str("hostile").expect("a valid slug"),
                name: "Hostile".to_owned(),
                path: "/repos/hostile".to_owned(),
                registered_at: ts("2026-10-02T10:00:00Z"),
            })
            .expect("the hostile repo registers");
        let epic = store
            .add_epic(
                EpicDraft {
                    title: "Hostile titles".to_owned(),
                    description: "The hostile epic.".to_owned(),
                    status: Status::Open,
                    references: References::new(),
                },
                ts("2026-10-02T10:01:00Z"),
            )
            .expect("the hostile epic lands");
        for (title, n) in [("</script>", 1u64), ("<b>&\" specials", 2)] {
            store
                .add_task(
                    TaskDraft {
                        epic: epic.id.clone(),
                        kind: TaskKind::Task,
                        title: title.to_owned(),
                        body: String::new(),
                        status: Status::Open,
                        verify: Some("cargo test -p hostile".to_owned()),
                        evidence: None,
                        note: None,
                        references: References::new(),
                        priority: None,
                        until: None,
                    },
                    ts(&format!("2026-10-02T10:10:{n:02}Z")),
                )
                .expect("the hostile task lands");
        }
    }

    /// **Why this test is `#[ignore]`d**: it is a measurement, not a
    /// property (the `cold_repo_list_open_cost` precedent) — it asserts
    /// structure only and prints numbers, never a wall-clock threshold,
    /// which would be flaky on a loaded host. The root is injected
    /// through `TODO_MCP_MEASURE_ROOT` so the harness never reads a
    /// home directory on its own; absent the variable it returns before
    /// touching anything. Run it with:
    ///
    /// ```text
    /// TODO_MCP_MEASURE_ROOT=~/.local/share/dacode-todo \
    ///   cargo test -p todo-mcp --test progress -- --ignored --nocapture
    /// ```
    ///
    /// It prints, over the named root: the registered repo count, and
    /// for the registry's first slug the mean cost of one lock-free
    /// graph read, one census read (the second log walk, t84's judgment
    /// call (a)) and one full page render — the render walking the
    /// current log twice plus one graph read per other registered repo.
    /// Those numbers belong in this run's output and the task's report,
    /// never in a source comment.
    ///
    /// `std::time::Instant` appears in this file alone: §8 bars clocks
    /// below the binary layer as a *determinism* rule, and a
    /// measurement harness produces no bytes, touches no golden and
    /// asserts no duration.
    #[test]
    #[ignore = "a measurement, not a property; run with --ignored --nocapture"]
    fn the_render_cost_over_a_named_root() {
        use std::time::Instant;

        let Some(root_path) = std::env::var_os("TODO_MCP_MEASURE_ROOT") else {
            println!("TODO_MCP_MEASURE_ROOT is unset — nothing to measure");
            return;
        };
        const RUNS: usize = 20;
        let root = DataRoot::new(root_path);
        let registry = todo_store::Registry::load(&root).expect("the registry loads");
        let slugs: Vec<RepoSlug> = registry.slugs().into_iter().cloned().collect();
        drop(registry);
        assert!(!slugs.is_empty(), "the named root holds repositories");
        let slug = slugs.first().expect("a first slug").clone();
        let log_bytes = fs::metadata(root.log_path(&slug))
            .expect("the log exists")
            .len();

        fn mean<R: FnMut()>(label: &str, mut run: R) {
            let mut total = std::time::Duration::ZERO;
            for _ in 0..RUNS {
                let started = Instant::now();
                run();
                total += started.elapsed();
            }
            println!(
                "{label:>28}: {:>9.1} µs (mean of {RUNS})",
                total.as_micros() as f64 / RUNS as f64
            );
        }

        println!(
            "the render cost over {} — {} registered repos, rendering {} ({} log bytes)",
            root.root().display(),
            slugs.len(),
            slug,
            log_bytes
        );
        mean("one graph read (read_repo)", || {
            drop(root.read_repo(&slug).expect("the graph reads"));
        });
        mean("one census read (the 2nd walk)", || {
            drop(root.read_repo_census(&slug).expect("the census reads"));
        });
        mean("one full page render", || {
            drop(render::page(&root, &slug).expect("the page renders"));
        });
    }
}
