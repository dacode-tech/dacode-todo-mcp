//! The mapping's suite (t43): the D18 conventions over directly-built
//! ASTs (the types are public), the plan's total order, the id
//! reproduction against a real store through the public
//! create/register path, the refusals, and the parse→map→apply
//! pipeline. All corpora are synthetic; the live board is t45's
//! fixture, never copied here.

use std::collections::BTreeMap;
use std::str::FromStr;

use todo_domain::{BlocksEdge, EpicDraft, RepoSlug, Status, Timestamp};
use todo_mcp::import::dacode::{
    ADossier, ADossierKind, ADossierStatus, AIndexRow, BDossier, BDossierKind, BDossierStatus,
    BSummaryRow, DacodeBoard, DecisionsAppendix, DeltasAppendix, DocDepth, DocEntry, GatesCell,
    ImportError, Marker, MilestoneItem, SubStep, Subsection, SummaryRow, SummaryStatus, TodoMd,
    UItem, map_board, parse_board,
};
use todo_mcp::import::plan::{ApplyError, ImportOptions, ImportPlan, PlanOp, apply_plan};
use todo_store::{DataRoot, Store};

// =================================================================
// The corpus helpers — ASTs built directly, the types being public.
// =================================================================

/// The import options' fixed parameters: one slug, one instant, and
/// the doc paths a caller knowing the repo tree would supply.
fn opts(doc_paths: BTreeMap<String, String>) -> ImportOptions {
    ImportOptions {
        slug: RepoSlug::from_str("dacode").unwrap(),
        imported_at: Timestamp::parse("2026-10-06T12:00:00Z").unwrap(),
        doc_paths,
    }
}

/// Doc paths for `keys`, one synthetic path per key.
fn paths(keys: &[&str]) -> BTreeMap<String, String> {
    keys.iter()
        .map(|key| ((*key).to_owned(), format!("docs/{key}.md")))
        .collect()
}

/// The marker constructors — each carries the raw line it occurred
/// on, which the status and finding conventions read.
fn check(line: &str) -> Marker {
    Marker::Check {
        line: line.to_owned(),
    }
}
fn in_work(line: &str) -> Marker {
    Marker::InWork {
        line: line.to_owned(),
    }
}
fn warning(line: &str) -> Marker {
    Marker::Warning {
        line: line.to_owned(),
    }
}
fn brainstorm(line: &str) -> Marker {
    Marker::Brainstorm {
        line: line.to_owned(),
    }
}
fn blocked_on(id: &str, line: &str) -> Marker {
    Marker::BlockedOn {
        id: id.to_owned(),
        line: line.to_owned(),
    }
}
fn asserts_milestone(milestone: &str, line: &str) -> Marker {
    Marker::MilestoneAssertion {
        milestone: milestone.to_owned(),
        line: line.to_owned(),
    }
}

/// A marker's raw line, whatever kind it is.
fn marker_line(marker: &Marker) -> String {
    match marker {
        Marker::Check { line }
        | Marker::InWork { line }
        | Marker::Warning { line }
        | Marker::Brainstorm { line }
        | Marker::BlockedOn { line, .. }
        | Marker::MilestoneAssertion { line, .. } => line.clone(),
    }
}

/// A U item whose body is its markers' raw lines.
fn u_item(id: &str, title: &str, checked: bool, markers: Vec<Marker>) -> UItem {
    let body = markers
        .iter()
        .map(marker_line)
        .collect::<Vec<_>>()
        .join("\n");
    UItem {
        id: id.to_owned(),
        title: title.to_owned(),
        checked,
        body,
        markers,
    }
}

/// An M item whose body is its markers' raw lines.
fn m_item(id: &str, title: &str, checked: bool, markers: Vec<Marker>) -> MilestoneItem {
    let body = markers
        .iter()
        .map(marker_line)
        .collect::<Vec<_>>()
        .join("\n");
    MilestoneItem {
        id: id.to_owned(),
        title: title.to_owned(),
        checked,
        body,
        markers,
    }
}

/// A sub-step row.
fn sub_step(id: &str, text: &str, gate: &str) -> SubStep {
    SubStep {
        id: id.to_owned(),
        text: text.to_owned(),
        gate: gate.to_owned(),
    }
}

/// A B dossier, resolved by default.
fn b_dossier(id: &str, title: &str, date: &str, outcome: &str) -> BDossier {
    BDossier {
        seq: 1,
        id: id.to_owned(),
        title: title.to_owned(),
        status: BDossierStatus {
            kind: BDossierKind::Resolved,
            date: date.to_owned(),
            applied: None,
        },
        outcome: outcome.to_owned(),
        body: format!("Dossier body for {id}."),
    }
}

/// An A dossier.
fn a_dossier(id: &str, span: &str, kind: ADossierKind, date: &str) -> ADossier {
    ADossier {
        id: id.to_owned(),
        title: span.to_owned(),
        status: ADossierStatus {
            kind,
            date: date.to_owned(),
            via: None,
        },
        outcome: String::new(),
        body: format!("Dossier body for {id}."),
    }
}

/// A B summary row with its Gates cell.
fn b_summary(id: &str, docs: Vec<&str>, milestone: Option<&str>) -> BSummaryRow {
    BSummaryRow {
        id: id.to_owned(),
        title: format!("Decision {id}"),
        gates: GatesCell {
            docs: docs.into_iter().map(str::to_owned).collect(),
            milestone: milestone.map(str::to_owned),
        },
    }
}

/// An A index row.
fn a_index(id: &str, delta: &str, depends_on: Vec<&str>) -> AIndexRow {
    AIndexRow {
        id: id.to_owned(),
        sections: vec![format!("§{id}")],
        delta: delta.to_owned(),
        depends_on: depends_on.into_iter().map(str::to_owned).collect(),
    }
}

/// A doc register entry.
fn doc_entry(key: &str, depth: DocDepth) -> DocEntry {
    DocEntry {
        key: key.to_owned(),
        depth,
        milestone: None,
    }
}

/// The composed board: §2's rows mirror the B summary (the parser's
/// cross-check would enforce set equality), and §6's items sit in one
/// subsection.
#[allow(
    clippy::too_many_arguments,
    reason = "the corpus composer mirrors the board's eight registers"
)]
fn board(
    milestones: Vec<MilestoneItem>,
    substeps: Vec<SubStep>,
    u_items: Vec<UItem>,
    docs: Vec<DocEntry>,
    b_summary: Vec<BSummaryRow>,
    b_dossiers: Vec<BDossier>,
    a_index: Vec<AIndexRow>,
    a_dossiers: Vec<ADossier>,
) -> DacodeBoard {
    let decisions = b_summary
        .iter()
        .map(|row| SummaryRow {
            id: row.id.clone(),
            title: row.title.clone(),
            status: SummaryStatus::Resolved,
            extra: None,
        })
        .collect();
    DacodeBoard {
        todo: TodoMd {
            decisions,
            b1a: "B1a (codec::Value) is done — implemented and committed.".to_owned(),
            docs,
            milestones,
            substeps,
            followups: vec![Subsection {
                title: Some("group".to_owned()),
                items: u_items,
            }],
        },
        decisions: DecisionsAppendix {
            summary: b_summary,
            dossiers: b_dossiers,
        },
        deltas: DeltasAppendix {
            index: a_index,
            applied: Vec::new(),
            dossiers: a_dossiers,
        },
    }
}

/// The plan's add-task drafts, in plan order.
fn tasks_of(plan: &ImportPlan) -> Vec<&todo_domain::TaskDraft> {
    plan.ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, .. } => Some(draft),
            _ => None,
        })
        .collect()
}

/// The plan's link ops' edges, in plan order.
fn edges_of(plan: &ImportPlan) -> Vec<&BlocksEdge> {
    plan.ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect()
}

/// A fresh registered store in a temp directory — the public
/// create/register path, the shape the import tool will use.
fn fresh_store(slug: &str) -> (tempfile::TempDir, todo_store::JsonlStore) {
    let dir = tempfile::TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let repo = todo_domain::Repo {
        slug: RepoSlug::from_str(slug).unwrap(),
        name: slug.to_owned(),
        path: format!("/repos/{slug}"),
        registered_at: Timestamp::parse("2026-10-06T00:00:00Z").unwrap(),
    };
    let store = root.register(repo).unwrap();
    (dir, store)
}

// =================================================================
// The sink corpus — every population, with planted order inversions
// (M1 before M0, U2 before U1, U4 before U3, B2 before B1, A2 before
// A1): the measured shape of the live board's registers.
// =================================================================

/// The sink board.
fn sink_board() -> DacodeBoard {
    board(
        vec![
            m_item(
                "M1",
                "engine. ▶️ STARTED",
                false,
                vec![in_work("▶️ started")],
            ),
            m_item(
                "M0",
                "groundwork. ✅ COMPLETE 2026-09-16",
                true,
                vec![check("✅ COMPLETE 2026-09-16")],
            ),
        ],
        vec![
            sub_step("M1.2", "the interface", "both reviewers"),
            sub_step("M0.2", "the workspace layout", "both reviewers"),
            // A ✅ planted in a Gate cell under an unchecked milestone:
            // the parent's checkbox governs, the cell's own mark does
            // not (the negative half of the sub-step status rule).
            sub_step("M1.1", "the first crate", "docs only ✅"),
            sub_step("M0.1", "deepen the foundation docs", "docs only"),
        ],
        vec![
            u_item(
                "U2",
                "Second follow-up. ✅ done",
                true,
                vec![check("✅ done 2026-09-18")],
            ),
            u_item(
                "U1",
                "First follow-up",
                false,
                vec![blocked_on("U2", "blocked on U2 for the marker")],
            ),
            u_item("U4", "Fourth item", false, Vec::new()),
            u_item(
                "U3",
                "Third item",
                false,
                vec![
                    brainstorm("🧠 brainstorm the design"),
                    asserts_milestone("M1", "due by M1 in prose"),
                ],
            ),
        ],
        vec![
            doc_entry("README", DocDepth::Expanded),
            doc_entry("01", DocDepth::Summary),
            doc_entry("A", DocDepth::Expanded),
        ],
        vec![
            b_summary("B2", vec!["01"], None),
            b_summary("B1", vec!["README", "01"], Some("M0")),
        ],
        vec![
            b_dossier("B2", "Delivery surface", "2026-09-17", "the deferral"),
            b_dossier(
                "B1",
                "Storage shape",
                "2026-09-12",
                "the recommendation taken",
            ),
        ],
        vec![
            a_index("A2", "Second delta", vec!["B1"]),
            a_index("A1", "First delta", vec!["B9", "A2"]),
        ],
        vec![
            a_dossier(
                "A2",
                "§4.2: the second delta",
                ADossierKind::SignedOff,
                "2026-09-17",
            ),
            a_dossier(
                "A1",
                "§3.1: the first delta",
                ADossierKind::Applied,
                "2026-09-18",
            ),
            a_dossier(
                "A3",
                "§3.2: the third delta",
                ADossierKind::HalfApplied,
                "2026-09-18",
            ),
        ],
    )
}

/// The sink board's plan.
fn sink_plan() -> ImportPlan {
    map_board(&sink_board(), &opts(paths(&["README", "01", "A"]))).unwrap()
}

// =================================================================
// 1. The status table.
// =================================================================

#[test]
fn the_status_table_maps_every_checkbox_marker_combination() {
    let items = vec![
        u_item(
            "U1",
            "checked with a check",
            true,
            vec![check("✅ done 2026-09-18")],
        ),
        u_item("U2", "checked without a marker", true, Vec::new()),
        u_item(
            "U3",
            "unchecked blocked",
            false,
            vec![blocked_on("U9", "blocked on U9")],
        ),
        u_item(
            "U4",
            "unchecked brainstorm",
            false,
            vec![brainstorm("🧠 gating prose")],
        ),
        u_item(
            "U5",
            "unchecked in work",
            false,
            vec![in_work("▶️ started")],
        ),
        u_item("U6", "unchecked plain", false, Vec::new()),
        // A ✅ on an unchecked item is prose; ⚠️ never affects status.
        u_item(
            "U7",
            "unchecked with a check",
            false,
            vec![check("✅ struck prose")],
        ),
        u_item(
            "U8",
            "unchecked with a warning",
            false,
            vec![warning("⚠️ caveat")],
        ),
        u_item(
            "U9",
            "checked with a warning",
            true,
            vec![warning("⚠️ caveat")],
        ),
    ];
    let corpus = board(
        Vec::new(),
        Vec::new(),
        items,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    let status_of = |title: &str| {
        tasks
            .iter()
            .find(|draft| draft.title == title)
            .map(|draft| {
                (
                    draft.status,
                    draft.evidence.as_deref(),
                    draft.note.as_deref(),
                )
            })
            .unwrap_or_else(|| panic!("no task titled {title}"))
    };
    assert_eq!(
        status_of("checked with a check"),
        (Status::Closed, Some("✅ done 2026-09-18"), None)
    );
    assert_eq!(
        status_of("checked without a marker"),
        (Status::Closed, Some("imported checked"), None)
    );
    assert_eq!(
        status_of("unchecked blocked"),
        (Status::Blocked, None, Some("blocked on U9"))
    );
    assert_eq!(
        status_of("unchecked brainstorm"),
        (Status::Deferred, None, Some("🧠 gating prose"))
    );
    assert_eq!(status_of("unchecked in work"), (Status::InWork, None, None));
    assert_eq!(status_of("unchecked plain"), (Status::Open, None, None));
    assert_eq!(
        status_of("unchecked with a check"),
        (Status::Open, None, None)
    );
    assert_eq!(
        status_of("unchecked with a warning"),
        (Status::Open, None, None)
    );
    // A checked item with only a ⚠️ has no ✅ marker, so the fallback
    // evidence applies.
    assert_eq!(
        status_of("checked with a warning"),
        (Status::Closed, Some("imported checked"), None)
    );
}

#[test]
fn the_first_status_bearing_marker_in_emission_order_decides() {
    // An InWork marker on an earlier line wins over a BlockedOn on a
    // later one; on one line, the emoji kinds precede BlockedOn in
    // the parser's emission order, so Brainstorm wins.
    let items = vec![
        u_item(
            "U1",
            "in work then blocked",
            false,
            vec![in_work("▶️ started"), blocked_on("U9", "blocked on U9")],
        ),
        u_item(
            "U2",
            "brainstorm and blocked on one line",
            false,
            vec![brainstorm("🧠 first"), blocked_on("U9", "blocked on U9")],
        ),
    ];
    let corpus = board(
        Vec::new(),
        Vec::new(),
        items,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    assert_eq!(tasks[0].status, Status::InWork);
    assert_eq!(tasks[1].status, Status::Deferred);
    assert_eq!(tasks[1].note.as_deref(), Some("🧠 first"));
}

#[test]
fn a_delta_statuses_map_the_three_kinds() {
    let corpus = board(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            a_dossier(
                "A1",
                "§3.1: the first delta",
                ADossierKind::Applied,
                "2026-09-18",
            ),
            a_dossier(
                "A2",
                "§4.2: the second delta",
                ADossierKind::SignedOff,
                "2026-09-17",
            ),
            a_dossier(
                "A3",
                "§3.2: the third delta",
                ADossierKind::HalfApplied,
                "2026-09-18",
            ),
        ],
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    // The A dossiers are the first tasks; B1a follows them.
    assert_eq!(tasks[0].title, "the first delta");
    assert_eq!(tasks[0].status, Status::Closed);
    assert_eq!(tasks[0].evidence.as_deref(), Some("APPLIED 2026-09-18"));
    assert_eq!(tasks[1].status, Status::Open);
    assert_eq!(tasks[1].evidence, None);
    assert_eq!(tasks[2].status, Status::InWork);
    assert_eq!(tasks[2].evidence, None);
}

#[test]
fn an_applied_delta_appends_its_outcome_to_the_evidence() {
    let mut dossier = a_dossier(
        "A1",
        "§3.1: the first delta",
        ADossierKind::Applied,
        "2026-09-18",
    );
    dossier.outcome = "applied as recommended".to_owned();
    let corpus = board(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![dossier],
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    assert_eq!(
        tasks[0].evidence.as_deref(),
        Some("APPLIED 2026-09-18: applied as recommended")
    );
}

// =================================================================
// 2. Decisions: kind, resolution, id order, Gates.
// =================================================================

#[test]
fn decisions_map_to_signed_with_resolution_and_sorted_ids() {
    // The dossiers arrive unsorted (B2 before B1); the plan's D ids
    // follow the numeric sort.
    let corpus = board(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            b_summary("B2", Vec::new(), None),
            b_summary("B1", Vec::new(), None),
        ],
        vec![
            b_dossier("B2", "Delivery surface", "2026-09-17", "the deferral"),
            b_dossier(
                "B1",
                "Storage shape",
                "2026-09-12",
                "the recommendation taken",
            ),
        ],
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let decisions: Vec<&todo_domain::DecisionDraft> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddDecision { draft, .. } => Some(draft),
            _ => None,
        })
        .collect();
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].title, "Storage shape");
    assert_eq!(decisions[1].title, "Delivery surface");
    for draft in &decisions {
        assert_eq!(draft.status, todo_domain::DecisionStatus::Signed);
        assert_eq!(draft.options, Vec::<String>::new());
        assert_eq!(draft.recommendation, None);
        assert_eq!(draft.confidence, todo_domain::Confidence::Medium);
    }
    assert_eq!(
        decisions[0].resolution.as_deref(),
        Some("the recommendation taken")
    );
    let ids: Vec<String> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddDecision { id, .. } => Some(id.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec!["dacode:D1", "dacode:D2"]);
}

#[test]
fn gates_milestone_edges_and_doc_references() {
    let corpus = board(
        vec![m_item("M0", "groundwork", true, Vec::new())],
        Vec::new(),
        Vec::new(),
        vec![
            doc_entry("README", DocDepth::Expanded),
            doc_entry("01", DocDepth::Summary),
            doc_entry("A", DocDepth::Expanded),
        ],
        // A Gates doc key with no register entry ("99") keeps the raw
        // key as the reference target.
        vec![b_summary("B1", vec!["README", "01", "99"], Some("M0"))],
        vec![b_dossier(
            "B1",
            "Storage shape",
            "2026-09-12",
            "the recommendation taken",
        )],
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(paths(&["README", "01", "A"]))).unwrap();
    // The milestone half: B1 gates M0's epic.
    assert_eq!(
        edges_of(&plan),
        vec![&BlocksEdge::DecisionToEpic {
            from: "dacode:D1".parse().unwrap(),
            to: "dacode:E1".parse().unwrap(),
        }]
    );
    // The doc half: references on the decision, targets in the
    // domain's canonical order.
    let decision = plan
        .ops
        .iter()
        .find_map(|op| match op {
            PlanOp::AddDecision { draft, .. } => Some(draft),
            _ => None,
        })
        .unwrap();
    let targets: Vec<(todo_domain::ReferenceKind, String)> = decision
        .references
        .iter()
        .map(|r| (r.kind(), r.target().to_owned()))
        .collect();
    assert_eq!(
        targets,
        vec![
            (todo_domain::ReferenceKind::Doc, "99".to_owned()),
            (todo_domain::ReferenceKind::Doc, "dacode:S1".to_owned()),
            (todo_domain::ReferenceKind::Doc, "dacode:S2".to_owned()),
        ]
    );
}

// =================================================================
// 3. A Depends-on and blocked-on: edges and findings.
// =================================================================

#[test]
fn a_depends_on_edges_and_the_unknown_id_finding() {
    let corpus = board(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![b_summary("B1", Vec::new(), None)],
        vec![b_dossier(
            "B1",
            "Storage shape",
            "2026-09-12",
            "the recommendation taken",
        )],
        vec![
            a_index("A1", "First delta", vec!["B1", "B9"]),
            a_index("A2", "Second delta", vec!["A1"]),
        ],
        vec![
            a_dossier(
                "A1",
                "§3.1: the first delta",
                ADossierKind::SignedOff,
                "2026-09-17",
            ),
            a_dossier(
                "A2",
                "§4.2: the second delta",
                ADossierKind::SignedOff,
                "2026-09-17",
            ),
        ],
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    // B1 gates A1; A1 blocks A2; B9 resolves to nothing.
    assert_eq!(
        edges_of(&plan),
        vec![
            &BlocksEdge::DecisionToTask {
                from: "dacode:D1".parse().unwrap(),
                to: "dacode:U1".parse().unwrap(),
            },
            &BlocksEdge::TaskToTask {
                from: "dacode:U1".parse().unwrap(),
                to: "dacode:U2".parse().unwrap(),
            },
        ]
    );
    let tasks = tasks_of(&plan);
    // A1 is U1, A2 is U2, B1a is U3, the finding is U4 — under the
    // Spec deltas epic (E2: no milestones, so E1 is Follow-ups).
    let finding = &tasks[3];
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "A1 depends on B9");
    assert_eq!(finding.body, "First delta");
    assert_eq!(finding.status, Status::Open);
    assert_eq!(finding.epic.to_string(), "dacode:E2");
    assert_eq!(
        finding
            .references
            .iter()
            .map(|r| (r.kind(), r.target()))
            .collect::<Vec<_>>(),
        vec![(todo_domain::ReferenceKind::Other, "A1")]
    );
}

#[test]
fn blocked_on_maps_status_note_and_edges_with_the_m_finding() {
    let items = vec![
        u_item(
            "U1",
            "blocked on a task",
            false,
            vec![blocked_on("U2", "blocked on U2")],
        ),
        u_item("U2", "the blocker", false, Vec::new()),
        u_item(
            "U3",
            "blocked on a decision",
            false,
            vec![blocked_on("B1", "blocked on B1")],
        ),
        u_item(
            "U4",
            "blocked on a milestone",
            false,
            vec![blocked_on("M0", "blocked on M0")],
        ),
    ];
    let corpus = board(
        vec![m_item("M0", "groundwork", true, Vec::new())],
        Vec::new(),
        items,
        Vec::new(),
        vec![b_summary("B1", Vec::new(), None)],
        vec![b_dossier(
            "B1",
            "Storage shape",
            "2026-09-12",
            "the recommendation taken",
        )],
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    assert_eq!(tasks[0].status, Status::Blocked);
    assert_eq!(tasks[0].note.as_deref(), Some("blocked on U2"));
    assert_eq!(tasks[2].status, Status::Blocked);
    // U/A targets → task→task; B targets → decision→task; the M
    // target resolves to no legal edge.
    assert_eq!(
        edges_of(&plan),
        vec![
            &BlocksEdge::TaskToTask {
                from: "dacode:U2".parse().unwrap(),
                to: "dacode:U1".parse().unwrap(),
            },
            &BlocksEdge::DecisionToTask {
                from: "dacode:D1".parse().unwrap(),
                to: "dacode:U3".parse().unwrap(),
            },
        ]
    );
    // The M target's finding: numbered after B1a (U6), under
    // Follow-ups.
    let finding = &tasks[5];
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "U4 blocked on M0");
    assert_eq!(finding.body, "blocked on M0");
    assert_eq!(finding.epic.to_string(), "dacode:E2");
}

#[test]
fn milestone_assertions_become_findings() {
    let items = vec![u_item(
        "U53",
        "the item",
        false,
        vec![asserts_milestone("M1", "U53 something. gates M1 in prose.")],
    )];
    let corpus = board(
        vec![
            m_item("M0", "groundwork", true, Vec::new()),
            m_item("M1", "engine", false, Vec::new()),
        ],
        Vec::new(),
        items,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    // U53 is U1, B1a is U2, the finding is U3 — under the asserting
    // item's epic (Follow-ups).
    let finding = &tasks[2];
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "U53 gates M1");
    assert_eq!(finding.body, "U53 something. gates M1 in prose.");
    assert_eq!(finding.status, Status::Open);
    assert_eq!(finding.epic.to_string(), "dacode:E3");
    assert_eq!(
        finding
            .references
            .iter()
            .map(|r| (r.kind(), r.target()))
            .collect::<Vec<_>>(),
        vec![(todo_domain::ReferenceKind::Other, "U53")]
    );
}

#[test]
fn an_m_sourced_blocked_on_becomes_a_decision_to_epic_edge_or_a_finding() {
    // An M item blocked on a B decision is the legal decision→epic
    // shape; blocked on anything else it is a finding (an epic is
    // never a blocks source).
    let corpus = board(
        vec![
            m_item(
                "M0",
                "groundwork",
                false,
                vec![blocked_on("B1", "blocked on B1")],
            ),
            m_item(
                "M1",
                "engine",
                false,
                vec![blocked_on("U9", "blocked on U9")],
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![b_summary("B1", Vec::new(), None)],
        vec![b_dossier(
            "B1",
            "Storage shape",
            "2026-09-12",
            "the recommendation taken",
        )],
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    assert_eq!(
        edges_of(&plan),
        vec![&BlocksEdge::DecisionToEpic {
            from: "dacode:D1".parse().unwrap(),
            to: "dacode:E1".parse().unwrap(),
        }]
    );
    let tasks = tasks_of(&plan);
    // B1a is U1; the finding is U2, under M1's own epic (E2).
    let finding = &tasks[1];
    assert_eq!(finding.title, "M1 blocked on U9");
    assert_eq!(finding.body, "blocked on U9");
    assert_eq!(finding.epic.to_string(), "dacode:E2");
}

#[test]
fn a_gates_milestone_naming_no_milestone_is_a_finding() {
    let corpus = board(
        vec![m_item("M0", "groundwork", true, Vec::new())],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![b_summary("B1", Vec::new(), Some("M9"))],
        vec![b_dossier(
            "B1",
            "Storage shape",
            "2026-09-12",
            "the recommendation taken",
        )],
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    // B1a is U1; the finding is U2, under Follow-ups (the assertion
    // is not tied to any milestone — the one it names does not exist).
    let finding = &tasks[1];
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "B1 gates M9");
    assert_eq!(finding.body, "Dossier body for B1.");
    assert_eq!(finding.epic.to_string(), "dacode:E2");
    assert!(edges_of(&plan).is_empty());
}

// =================================================================
// 4. The plan order and the count invariant.
// =================================================================

#[test]
fn the_plan_order_is_the_documented_total_order() {
    let plan = sink_plan();
    let labels: Vec<String> = plan.ops.iter().map(op_label).collect();
    assert_eq!(
        labels,
        vec![
            // Epics: M0, M1 (sorted), then the two synthetics.
            "epic dacode:E1 groundwork",
            "epic dacode:E2 engine",
            "epic dacode:E3 Follow-ups",
            "epic dacode:E4 Spec deltas",
            // Decisions: B1, B2 (sorted).
            "decision dacode:D1 Storage shape",
            "decision dacode:D2 Delivery surface",
            // Docs: the register's order.
            "doc dacode:S1 README",
            "doc dacode:S2 01",
            "doc dacode:S3 A",
            // Tasks: U items sorted (U1–U4), sub-steps (M0.1, M0.2,
            // M1.1, M1.2), A dossiers sorted (A1–A3), B1a, findings.
            "task dacode:U1 First follow-up blocked",
            "task dacode:U2 Second follow-up closed",
            "task dacode:U3 Third item deferred",
            "task dacode:U4 Fourth item open",
            "task dacode:U5 M0.1 closed",
            "task dacode:U6 M0.2 closed",
            "task dacode:U7 M1.1 open",
            "task dacode:U8 M1.2 open",
            "task dacode:U9 the first delta closed",
            "task dacode:U10 the second delta open",
            "task dacode:U11 the third delta in_work",
            "task dacode:U12 B1a closed",
            "task dacode:U13 A1 depends on B9 open",
            "task dacode:U14 U3 gates M1 open",
            // Edges: Gates milestone (B-id order), A-depends (A-id
            // order, cell order within), blocked-on (U-id order).
            "edge dacode:D1 -> dacode:E1",
            "edge dacode:U10 -> dacode:U9",
            "edge dacode:D1 -> dacode:U10",
            "edge dacode:U2 -> dacode:U1",
        ]
    );
}

/// One op rendered compactly, for order assertions.
fn op_label(op: &PlanOp) -> String {
    match op {
        PlanOp::AddEpic { draft, id, .. } => format!("epic {id} {}", draft.title),
        PlanOp::AddDecision { draft, id, .. } => format!("decision {id} {}", draft.title),
        PlanOp::AddDoc { draft, id, .. } => format!("doc {id} {}", draft.title),
        PlanOp::AddTask { draft, id, .. } => {
            format!("task {id} {} {}", draft.title, draft.status.as_str())
        }
        PlanOp::LinkBlocks { edge, .. } => format!("edge {edge}"),
    }
}

#[test]
fn the_count_invariant_no_population_vanishes() {
    let plan = sink_plan();
    let counts = |plan: &ImportPlan| {
        let mut counts = [0usize; 5];
        for op in &plan.ops {
            match op {
                PlanOp::AddEpic { .. } => counts[0] += 1,
                PlanOp::AddDecision { .. } => counts[1] += 1,
                PlanOp::AddDoc { .. } => counts[2] += 1,
                PlanOp::AddTask { .. } => counts[3] += 1,
                PlanOp::LinkBlocks { .. } => counts[4] += 1,
            }
        }
        counts
    };
    // epics == M + 2; decisions == B; docs == register; tasks == U +
    // sub-steps + A + 1 (B1a) + findings (2: A1's B9, U3's M1).
    assert_eq!(counts(&plan), [4, 2, 3, 14, 4]);
}

// =================================================================
// 5. Timestamps, B1a, docs, determinism.
// =================================================================

#[test]
fn timestamps_map_board_dates_and_imported_at() {
    let plan = sink_plan();
    let imported = "2026-10-06T12:00:00.000000000Z";
    for op in &plan.ops {
        let (label, at) = match op {
            PlanOp::AddEpic { id, at, .. } => (format!("epic {id}"), at),
            PlanOp::AddDecision { id, at, .. } => (format!("decision {id}"), at),
            PlanOp::AddDoc { id, at, .. } => (format!("doc {id}"), at),
            PlanOp::AddTask { id, at, .. } => (format!("task {id}"), at),
            // D9's "all edges → imported_at" is pinned here too: the
            // edge arm falls through to the imported expectation.
            PlanOp::LinkBlocks { edge, at } => (format!("edge {edge}"), at),
        };
        // The dossier dates are the only board dates: B1 2026-09-12,
        // B2 2026-09-17, A1/A3 2026-09-18, A2 2026-09-17.
        let expected = match label.as_str() {
            "decision dacode:D1" => "2026-09-12T00:00:00.000000000Z",
            "decision dacode:D2" => "2026-09-17T00:00:00.000000000Z",
            "task dacode:U9" | "task dacode:U11" => "2026-09-18T00:00:00.000000000Z",
            "task dacode:U10" => "2026-09-17T00:00:00.000000000Z",
            _ => imported,
        };
        assert_eq!(at.as_str(), expected, "{label}");
    }
}

#[test]
fn b1a_is_a_closed_followup_referencing_b1() {
    let plan = sink_plan();
    let tasks = tasks_of(&plan);
    // B1a is the task after the A dossiers: U12.
    let b1a = &tasks[11];
    assert_eq!(b1a.title, "B1a");
    assert_eq!(b1a.status, Status::Closed);
    assert_eq!(b1a.kind, todo_domain::TaskKind::Task);
    assert_eq!(
        b1a.evidence.as_deref(),
        Some("B1a (codec::Value) is done — implemented and committed.")
    );
    assert_eq!(b1a.body, b1a.evidence.as_deref().unwrap());
    assert_eq!(b1a.epic.to_string(), "dacode:E3");
    assert_eq!(
        b1a.references
            .iter()
            .map(|r| (r.kind(), r.target()))
            .collect::<Vec<_>>(),
        vec![(todo_domain::ReferenceKind::Other, "B1")]
    );
}

#[test]
fn docs_map_version_and_path_with_the_missing_path_error() {
    let corpus = board(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            doc_entry("README", DocDepth::Expanded),
            doc_entry("01", DocDepth::Summary),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(paths(&["README", "01"]))).unwrap();
    let docs: Vec<&todo_domain::DocDraft> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddDoc { draft, .. } => Some(draft),
            _ => None,
        })
        .collect();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].title, "README");
    assert_eq!(docs[0].path, "docs/README.md");
    assert_eq!(docs[0].version, "expanded");
    assert_eq!(docs[1].version, "summary");
    // A missing path is a named error naming the key.
    assert_eq!(
        map_board(&corpus, &opts(paths(&["README"]))),
        Err(ImportError::MissingDocPath {
            key: "01".to_owned()
        })
    );
    // Extra keys are ignored: the register governs.
    let mut extra = paths(&["README", "01"]);
    extra.insert("TODO".to_owned(), "TODO.md".to_owned());
    assert_eq!(
        map_board(&corpus, &opts(extra)).unwrap(),
        map_board(&corpus, &opts(paths(&["README", "01"]))).unwrap()
    );
}

#[test]
fn mapping_the_same_board_twice_yields_equal_plans() {
    // The premise first (P-LIVE's comparison rule): the plan is
    // populated, not an empty success.
    let plan = sink_plan();
    assert!(!plan.ops.is_empty());
    assert_eq!(
        map_board(&sink_board(), &opts(paths(&["README", "01", "A"]))).unwrap(),
        plan
    );
}

#[test]
fn references_carry_original_ids_and_sections() {
    // The veto-able reading the reference pushes pin: sub-steps and
    // A-deltas carry their original id as an `other` reference, and
    // the A-delta's section list rides along as `spec` references —
    // the colon cut would otherwise drop the sections with the title.
    let plan = sink_plan();
    let tasks = tasks_of(&plan);
    // A1 is the first A-delta task (U9); its span is
    // "§3.1: the first delta".
    let a1 = &tasks[8];
    assert_eq!(a1.title, "the first delta");
    assert_eq!(
        a1.references
            .iter()
            .map(|r| (r.kind(), r.target()))
            .collect::<Vec<_>>(),
        vec![
            (todo_domain::ReferenceKind::Spec, "§3.1"),
            (todo_domain::ReferenceKind::Other, "A1"),
        ]
    );
    // The sub-step gate task M0.1 (U5) carries its original id.
    let gate = &tasks[4];
    assert_eq!(gate.title, "M0.1");
    assert_eq!(
        gate.references
            .iter()
            .map(|r| (r.kind(), r.target()))
            .collect::<Vec<_>>(),
        vec![(todo_domain::ReferenceKind::Other, "M0.1")]
    );
}

#[test]
fn the_draft_fields_pin_the_conventions() {
    // The unpinned clauses, asserted over the sink plan: the epic
    // descriptions' provenance, the verbatim bodies, the verify
    // literals, priority/until as None, and the Gate-cell-✅ negative
    // half (M1.1's cell carries a ✅ under an unchecked milestone and
    // the task stays open).
    let plan = sink_plan();
    let epics: Vec<&todo_domain::EpicDraft> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddEpic { draft, .. } => Some(draft),
            _ => None,
        })
        .collect();
    assert_eq!(epics[0].description, "the dacode board's §5 milestone M0");
    assert_eq!(
        epics[2].description,
        "the dacode board's §6 open follow-ups, not tied to a milestone"
    );
    assert_eq!(
        epics[3].description,
        "the dacode board's A-register design deltas"
    );

    let tasks = tasks_of(&plan);
    // U1 (U items first), M0.1/M1.1 (sub-steps), A1, B1a, the finding.
    let (u1, m01, m11) = (tasks[0], tasks[4], tasks[6]);
    let (a1, b1a, finding) = (tasks[8], tasks[11], tasks[12]);
    // Bodies verbatim: the U item's marker line, the sub-step's text
    // cell, the A dossier's body, B1a's paragraph.
    assert_eq!(u1.body, "blocked on U2 for the marker");
    assert_eq!(m01.body, "deepen the foundation docs");
    assert_eq!(a1.body, "Dossier body for A1.");
    assert_eq!(
        b1a.body,
        "B1a (codec::Value) is done — implemented and committed."
    );
    // Verify: the sub-step's Gate cell; the fixed literal elsewhere.
    assert_eq!(m01.verify.as_deref(), Some("docs only"));
    assert_eq!(m11.verify.as_deref(), Some("docs only ✅"));
    assert_eq!(
        u1.verify.as_deref(),
        Some("imported: the board's checkbox and discharge marker")
    );
    // The Gate cell's own ✅ does not close the sub-step — the parent
    // milestone's checkbox does, and M1 is unchecked.
    assert_eq!(m11.status, Status::Open);
    // The dialect has no priority vocabulary and no until-dates.
    for draft in [u1, m01, m11, a1, b1a, finding] {
        assert_eq!(draft.priority, None);
        assert_eq!(draft.until, None);
    }
}

// =================================================================
// 6. The executor: refusals, id reproduction, the pipeline.
// =================================================================

#[test]
fn apply_refuses_a_non_empty_graph() {
    let plan = sink_plan();
    let (_dir, mut store) = fresh_store("dacode");
    // One pre-existing node makes the store non-empty.
    store
        .add_epic(
            EpicDraft {
                title: "pre-existing".to_owned(),
                description: String::new(),
                status: Status::Open,
                references: todo_domain::References::new(),
            },
            Timestamp::parse("2026-10-06T00:00:00Z").unwrap(),
        )
        .unwrap();
    let error = apply_plan(&plan, &mut store).unwrap_err();
    assert_eq!(
        error,
        ApplyError::Refused(ImportError::StoreNotEmpty {
            slug: "dacode".to_owned(),
        })
    );
    // Nothing was written: the graph still holds exactly the one
    // pre-existing epic and nothing else.
    let graph = store.graph();
    assert_eq!(graph.epics().count(), 1);
    assert_eq!(graph.tasks().count(), 0);
    assert_eq!(graph.decisions().count(), 0);
    assert_eq!(graph.docs().count(), 0);
}

#[test]
fn a_used_then_emptied_store_is_refused() {
    // The refusal gap the node count alone left open: a store whose
    // epics were cascade-deleted holds no nodes, but §3's
    // never-reuse rule keeps its counters raised — importing there
    // would land every id one past the plan's prediction.
    let plan = sink_plan();
    let (_dir, mut store) = fresh_store("dacode");
    let at = Timestamp::parse("2026-10-06T00:00:00Z").unwrap();
    let epic = store
        .add_epic(
            EpicDraft {
                title: "doomed".to_owned(),
                description: String::new(),
                status: Status::Open,
                references: todo_domain::References::new(),
            },
            at.clone(),
        )
        .unwrap();
    store.delete_epic(epic.id, at).unwrap();
    // The graph is empty; the counter is not.
    assert_eq!(store.graph().epics().count(), 0);
    assert_eq!(store.graph().counters().epic(), 1);
    let error = apply_plan(&plan, &mut store).unwrap_err();
    assert_eq!(
        error,
        ApplyError::Refused(ImportError::StoreNotEmpty {
            slug: "dacode".to_owned(),
        })
    );
    // Nothing was written: the graph is still empty.
    assert_eq!(store.graph().epics().count(), 0);
}

#[test]
fn an_id_mismatch_plan_is_a_named_error() {
    // A hand-filtered plan — one epic op removed — no longer matches
    // its positions: the store allocates E1 where the plan constructs
    // E2, and the walk's id comparison refuses naming both.
    let plan = sink_plan();
    let filtered = ImportPlan {
        ops: plan.ops.iter().skip(1).cloned().collect(),
    };
    let (_dir, mut store) = fresh_store("dacode");
    let error = apply_plan(&filtered, &mut store).unwrap_err();
    assert_eq!(
        error,
        ApplyError::Refused(ImportError::IdMismatch {
            expected: "dacode:E2".to_owned(),
            found: "dacode:E1".to_owned(),
        })
    );
    // The mismatch fires after the first add landed — apply is not
    // transactional — so exactly one epic is on the store.
    assert_eq!(store.graph().epics().count(), 1);
}

#[test]
fn id_reproduction_at_the_live_board_scale() {
    // The live board's scale: 170 U items with the three measured
    // inversions planted, 21 B dossiers unsorted (reversed), 28 A
    // dossiers (rotated), 15 sub-steps, 7 milestones, 50 docs.
    let mut milestones = Vec::new();
    for n in 0..=6 {
        milestones.push(m_item(
            &format!("M{n}"),
            &format!("milestone {n}."),
            n == 0,
            Vec::new(),
        ));
    }
    let mut substeps = Vec::new();
    for (major, count) in [(0u32, 4u32), (1, 11)] {
        for step in 1..=count {
            substeps.push(sub_step(
                &format!("M{major}.{step}"),
                &format!("step {major}.{step}"),
                "both reviewers",
            ));
        }
    }
    let mut ids: Vec<u32> = (1..=170).collect();
    for (a, b) in [(76usize, 89), (110, 113), (125, 131)] {
        ids.swap(a, b);
    }
    let u_items: Vec<UItem> = ids
        .iter()
        .map(|&n| {
            // A checked pair — one with a ✅ discharge marker, one
            // without — so the closed-evidence path runs at scale.
            let (checked, markers) = match n {
                1 => (true, Vec::new()),
                2 => (true, vec![check("✅ done 2026-09-18")]),
                _ => (false, Vec::new()),
            };
            let mut item = u_item(
                &format!("U{n}"),
                &format!("follow-up {n}."),
                checked,
                markers,
            );
            if n == 50 {
                item.markers = vec![asserts_milestone("M2", "U50 due by M2 in prose.")];
                item.body = "U50 due by M2 in prose.".to_owned();
            }
            item
        })
        .collect();
    let docs: Vec<DocEntry> = (0..50)
        .map(|n| doc_entry(&format!("{n:02}"), DocDepth::Expanded))
        .collect();
    let mut b_dossiers = Vec::new();
    let mut summary_rows = Vec::new();
    for n in (1..=21).rev() {
        b_dossiers.push(b_dossier(
            &format!("B{n}"),
            &format!("decision {n}"),
            "2026-09-12",
            &format!("resolved {n}"),
        ));
        let (gates, milestone) = match n {
            1 => (vec!["01"], Some("M0")),
            5 => (vec!["02"], Some("M2")),
            _ => (Vec::new(), None),
        };
        summary_rows.push(b_summary(&format!("B{n}"), gates, milestone));
    }
    let mut a_dossiers = Vec::new();
    for n in 1..=28 {
        let rotated = (n + 14 - 1) % 28 + 1;
        a_dossiers.push(a_dossier(
            &format!("A{rotated}"),
            &format!("§3.{rotated}: delta {rotated}"),
            ADossierKind::Applied,
            "2026-09-18",
        ));
    }
    let a_index = vec![
        a_index("A1", "delta 1", vec!["B1"]),
        a_index("A2", "delta 2", vec!["B2", "A1"]),
        a_index("A3", "delta 3", vec!["B99"]),
    ];
    let corpus = board(
        milestones,
        substeps,
        u_items,
        docs,
        summary_rows,
        b_dossiers,
        a_index,
        a_dossiers,
    );
    let keys: Vec<String> = (0..50).map(|n| format!("{n:02}")).collect();
    let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();
    let plan = map_board(&corpus, &opts(paths(&key_refs))).unwrap();

    let (_dir, mut store) = fresh_store("dacode");
    apply_plan(&plan, &mut store).unwrap();
    let graph = store.graph();

    // The reproduction: the store's ids are exactly the board's.
    let epic_ids: Vec<String> = graph.epics().map(|e| e.id.to_string()).collect();
    assert_eq!(
        epic_ids,
        (1..=9).map(|n| format!("dacode:E{n}")).collect::<Vec<_>>()
    );
    let decision_ids: Vec<String> = graph.decisions().map(|d| d.id.to_string()).collect();
    assert_eq!(
        decision_ids,
        (1..=21).map(|n| format!("dacode:D{n}")).collect::<Vec<_>>()
    );
    // The reproduction is positional: dacode:D1 is the board's B1 — the
    // reversed dossier order is sorted away.
    let decision_titles: Vec<&str> = graph.decisions().map(|d| d.title.as_str()).collect();
    assert_eq!(
        decision_titles,
        (1..=21)
            .map(|n| format!("decision {n}"))
            .collect::<Vec<_>>()
    );
    let doc_ids: Vec<String> = graph.docs().map(|d| d.id.to_string()).collect();
    assert_eq!(
        doc_ids,
        (1..=50).map(|n| format!("dacode:S{n}")).collect::<Vec<_>>()
    );
    let tasks: Vec<&todo_domain::Task> = graph.tasks().collect();
    assert_eq!(tasks.len(), 216);
    // U1–U170 reproduce the board's U ids — the planted inversions
    // (U77 after U90, U111 after U114, U126 after U132) are sorted
    // away, so task U77 is the board's U77.
    for (position, task) in tasks.iter().enumerate() {
        let expected = match position {
            0..=169 => format!("follow-up {}", position + 1),
            170..=184 => {
                let n = position - 170;
                let (major, step) = if n < 4 { (0, n + 1) } else { (1, n - 3) };
                format!("M{major}.{step}")
            }
            185..=212 => format!("delta {}", position - 184),
            213 => "B1a".to_owned(),
            214 => "A3 depends on B99".to_owned(),
            215 => "U50 gates M2".to_owned(),
            _ => unreachable!("216 tasks"),
        };
        assert_eq!(task.title, expected, "task {}", task.id);
    }
    // The task ids are U1–U216 in the documented order.
    assert_eq!(tasks[169].id.to_string(), "dacode:U170");
    assert_eq!(tasks[170].id.to_string(), "dacode:U171");
    assert_eq!(tasks[213].id.to_string(), "dacode:U214");
    assert_eq!(tasks[215].id.to_string(), "dacode:U216");
}

#[test]
fn the_pipeline_composes_parse_map_and_apply() {
    let todo_md = "\
## 2. Decisions (Appendix B)

| # | Decision | Recommendation | Status |
|---|---|---|---|
| ~~B1~~ | Storage shape | ✅ **SIGNED OFF 2026-09-12** — the recommendation taken | ✅ resolved |

B1a (`codec::Value`) is **done** — implemented and committed.

## 3. Spec deltas (Appendix A)

Queue prose, not imported.

## 4. Doc depth — 1 of 2 docs still summaries

Already expanded (2): `README`, and the **seven** appendices (A–G).

| Milestone | Docs still summary | Count |
|---|---|---|
| **M0** | 01 firstdoc | 1 |

## 5. Implementation work by milestone

| | Sub-step | Gate |
|---|---|---|
| **M0.1** | deepen the foundation docs | docs only |

- [x] **M0 — groundwork. ✅ COMPLETE 2026-09-16** (one session.)

## 6. Open follow-ups not tied to a milestone

- [ ] **U1 · First follow-up.** The figures drift between runs.
- [x] **U2 · Second follow-up. ✅ done 2026-09-18** blocked on U1.
";
    let b_decisions = "\
# Appendix B — Open decisions needing sign-off

| # | Decision | Recommendation | Confidence | Gates |
|---|---|---|---|---|
| ~~**B1**~~ | ~~Storage shape~~ | ✅ **SIGNED OFF 2026-09-12** — the recommendation taken | high | ~~01 → M0~~ |

### 1. [B1] Storage shape — ✅ **RESOLVED 2026-09-12: the recommendation taken**

Dossier body one for B1.
";
    let a_deltas = "\
# Appendix A — Design deltas

| # | Section | Delta | Depends on | Recommend |
|---|---|---|---|---|
| ~~A1~~ | §3.1 | First delta | B1 | ✅ **APPLIED 2026-09-18** — applied |

### A1 — §3.1: the first delta — ✅ **APPLIED 2026-09-18**

Dossier body for A1.
";
    let parsed = parse_board(todo_md, b_decisions, a_deltas).unwrap();
    let keys = ["README", "A", "B", "C", "D", "E", "F", "G", "01"];
    let plan = map_board(&parsed, &opts(paths(&keys))).unwrap();
    let (_dir, mut store) = fresh_store("dacode");
    apply_plan(&plan, &mut store).unwrap();
    let graph = store.graph();
    // The store answers queries over the imported graph.
    assert_eq!(graph.epics().count(), 3);
    assert_eq!(graph.decisions().count(), 1);
    assert_eq!(graph.docs().count(), 9);
    assert_eq!(graph.tasks().count(), 5);
    let u2 = graph.task(&"dacode:U2".parse().unwrap()).unwrap();
    assert_eq!(u2.title, "Second follow-up");
    assert_eq!(u2.status, Status::Closed);
    assert_eq!(
        u2.evidence.as_deref(),
        Some("**U2 · Second follow-up. ✅ done 2026-09-18** blocked on U1.")
    );
    // U2 is blocked by U1 (the marker's edge), and the sub-step
    // inherited M0's checkbox.
    assert_eq!(
        graph.blocked_by(&todo_domain::BlockEnd::Task("dacode:U2".parse().unwrap())),
        vec![todo_domain::BlockEnd::Task("dacode:U1".parse().unwrap())]
    );
    let gate = graph.task(&"dacode:U3".parse().unwrap()).unwrap();
    assert_eq!(gate.title, "M0.1");
    assert_eq!(gate.status, Status::Closed);
    assert_eq!(
        gate.evidence.as_deref(),
        Some("imported: parent milestone M0 complete")
    );
}

// =================================================================
// 7. Titles and the named-error corners.
// =================================================================

#[test]
fn titles_truncate_the_marker_tail_and_strip_leading_flags() {
    let items = vec![
        u_item(
            "U1",
            "groundwork. ✅ COMPLETE 2026-09-16",
            false,
            Vec::new(),
        ),
        u_item(
            "U2",
            "⚠️ The nightly scan did not protect the fixture",
            false,
            Vec::new(),
        ),
        u_item("U3", "plain title", false, Vec::new()),
        u_item("U4", "tail. ▶️ started", false, Vec::new()),
        u_item(
            "U5",
            "✅ CLOSED 2026-09-19 by A3 — both seams",
            false,
            Vec::new(),
        ),
    ];
    let corpus = board(
        Vec::new(),
        Vec::new(),
        items,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let plan = map_board(&corpus, &opts(BTreeMap::new())).unwrap();
    let tasks = tasks_of(&plan);
    let titles: Vec<&str> = tasks.iter().map(|draft| draft.title.as_str()).collect();
    // B1a sorts after the U items; the U titles are the first five.
    assert_eq!(
        titles[..5],
        [
            "groundwork",
            "The nightly scan did not protect the fixture",
            "plain title",
            "tail",
            "CLOSED 2026-09-19 by A3 — both seams",
        ]
    );
}

#[test]
fn a_sub_step_whose_milestone_is_absent_is_a_named_error() {
    let corpus = board(
        vec![m_item("M0", "groundwork", true, Vec::new())],
        vec![sub_step("M9.1", "orphan step", "both reviewers")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(
        map_board(&corpus, &opts(BTreeMap::new())),
        Err(ImportError::UnresolvedParent {
            id: "M9.1".to_owned(),
            milestone: "M9".to_owned(),
        })
    );
}

#[test]
fn a_calendar_invalid_board_date_is_a_named_error() {
    // The parser validates the date's shape; the calendar check is
    // the mapping's.
    let corpus = board(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![b_dossier(
            "B1",
            "Storage shape",
            "2026-02-30",
            "the recommendation taken",
        )],
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(
        map_board(&corpus, &opts(BTreeMap::new())),
        Err(ImportError::BadDate {
            date: "2026-02-30".to_owned()
        })
    );
}
