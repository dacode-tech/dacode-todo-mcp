//! The TODO.md walker's suite (t42): the happy path over a synthetic
//! minimal board, the §5/§6 strictness rules, the §4 doc register,
//! the marker grammar, the composition and its cross-file errors, the
//! twice-parse determinism pin, and the board-order pin. All corpora
//! are hand-authored synthetic strings in the measured dialect — the
//! live board is t45's fixture, never copied here.

use todo_mcp::import::dacode::{
    ADossierKind, BDossierKind, DocDepth, ImportError, Marker, SummaryStatus, parse_board,
    parse_todo_md,
};

// =================================================================
// The corpora — a synthetic minimal board composed of section
// pieces, so the error-path corpora can swap one section out without
// duplicating the rest. §1 and §3 are walked past, so the corpora
// start at §2.
// =================================================================

/// §2 and §3: two decision rows, the B1a paragraph, walked-past §3.
const HEAD: &str = "\
## 2. Decisions (Appendix B)

| # | Decision | Recommendation | Status |
|---|---|---|---|
| ~~B1~~ | Storage shape | ✅ **SIGNED OFF 2026-09-12** — the recommendation taken | ✅ resolved |
| ~~B2~~ | Delivery surface | ✅ **SIGNED OFF 2026-09-17** — the deferral | ⚠️ resolved, half-applied |

B1a (`codec::Value`) is **done** — implemented and committed.

## 3. Spec deltas (Appendix A)

Queue prose, not imported.
";

/// §4: a small expanded list (with the appendices phrase) and the
/// milestone table with M0's em-dash cell.
const SECTION_4: &str = "\
## 4. Doc depth — 2 of 4 docs still summaries

Already expanded (4): `README`, `00-overview`, **`01`**, and the **seven** appendices (A–G).

| Milestone | Docs still summary | Count |
|---|---|---|
| **M0** | — *(two docs deepened before this row)* | **0** |
| **M1** | ~~03 firstdoc~~ *(deepened earlier)*, 08 seconddoc, 22 thirddoc | 3 |
";

/// §5: two sub-step tables (each preceding its milestone item, the
/// measured order), two milestones, an indented 🧠 checkbox as body
/// text.
const SECTION_5: &str = "\
## 5. Implementation work by milestone

| | Sub-step | Gate |
|---|---|---|
| **M0.1** | deepen the two foundation docs | docs only |
| **M0.2** | the workspace layout | both reviewers |

- [x] **M0 — groundwork. ✅ COMPLETE 2026-09-16** (one session; commits `a1b2c3d`.)
      Second body line of M0.

| | Sub-step | Gate |
|---|---|---|
| **M1.1** | the engine's first crate | both reviewers |

- [ ] **M1 — the engine and interface.** The first crate, its events, the envelope.
   - [ ] **🧠 the loop design** — brainstorm first, indented body text.
";

/// §6: the walked-past attention-meta comment, two subsections, five
/// U items exercising the marker grammar.
const SECTION_6: &str = "\
## 6. Open follow-ups not tied to a milestone

<!-- attention-meta — machine-readable; not imported.
action: U8=ack
-->

### first group

- [ ] **U1 · First follow-up.** The figures drift between runs.
      Continuation line of U1.
- [x] **U2 · Second follow-up. ✅ done 2026-09-18** blocked on U1.
      gates M1 in prose.

### second subsection

- [ ] **U3 · Third item.** due by M2.
- [ ] **U4 · Fourth item.** before M4 stays body text.
- [ ] **U5 · Fifth item.** ▶️ started, ⚠️ caveat.
";

/// The full synthetic TODO.md.
fn todo_md() -> String {
    format!("{HEAD}{SECTION_4}{SECTION_5}{SECTION_6}")
}

/// §1–§5 of the corpus, for the §6 error-path variants.
fn valid_through_5() -> String {
    format!("{HEAD}{SECTION_4}{SECTION_5}")
}

/// A synthetic B appendix aligned with `TODO_MD`'s §2 ids.
const B_DECISIONS: &str = "\
# Appendix B — Open decisions needing sign-off

Prose intro, walked past.

| # | Decision | Recommendation | Confidence | Gates |
|---|---|---|---|---|
| ~~**B1**~~ | ~~Storage shape~~ | ✅ **SIGNED OFF 2026-09-12** — the recommendation taken | high | ~~02, 03 → M0~~ |
| ~~**B2**~~ | ~~Delivery surface~~ | ✅ **SIGNED OFF 2026-09-17** — the deferral | ~~med~~ | ~~21, 19~~ ✅ |

### 1. [B1] Storage shape — ✅ **RESOLVED 2026-09-12: the recommendation taken**

Dossier body one for B1.

### 2. [B2] Delivery surface — ✅ **SIGNED OFF 2026-09-17, APPLIED 2026-09-18 via A8: the deferral taken as recommended**

Dossier body for B2.
";

/// A synthetic A appendix.
const A_DELTAS: &str = "\
# Appendix A — Design deltas

Prose intro.

| # | Section | Delta | Depends on | Recommend |
|---|---|---|---|---|
| ~~A1~~ | §3.1, §3.2 | **First delta** | ~~B3~~ ✅ signed, ~~B4~~ ✅ signed | ✅ **APPLIED 2026-09-18** — applied |
| ~~A2~~ | §4.2 | Second delta | — | ✅ **SIGNED OFF 2026-09-17** — approved |

### A1 — §3.1, §3.2: the first delta — ✅ **APPLIED 2026-09-18**

Dossier body for A1.

### A2 — §4.2: the second delta — ✅ **SIGNED OFF 2026-09-17**

Dossier body for A2.
";

/// Composes a corpus with `followups` as §6's content after the valid
/// §1–§5 prefix.
fn todo_with_section6(followups: &str) -> String {
    let prefix = valid_through_5();
    format!("{prefix}## 6. Open follow-ups not tied to a milestone\n\n{followups}")
}

/// The 1-based line number of the first line containing `needle` in
/// `corpus` — derived from the corpus itself so the corpora's blank
/// lines cannot desynchronize a line assertion.
fn line_of(corpus: &str, needle: &str) -> usize {
    corpus.split('\n').position(|l| l.contains(needle)).unwrap() + 1
}

// =================================================================
// 1. The happy path: the full AST.
// =================================================================

#[test]
fn the_board_happy_path_parses_every_register() {
    let board = parse_todo_md(&todo_md()).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // §2: ids, titles, parsed statuses, in board order.
    assert_eq!(board.decisions.len(), 2);
    assert_eq!(board.decisions[0].id, "B1");
    assert_eq!(board.decisions[0].title, "Storage shape");
    assert_eq!(board.decisions[0].status, SummaryStatus::Resolved);
    assert_eq!(board.decisions[1].id, "B2");
    assert_eq!(board.decisions[1].title, "Delivery surface");
    assert_eq!(board.decisions[1].status, SummaryStatus::HalfApplied);

    // The B1a paragraph, raw.
    assert_eq!(
        board.b1a,
        "B1a (`codec::Value`) is **done** — implemented and committed."
    );

    // §4: the expanded list (with the appendices phrase) then the
    // milestone table's summary entries, in board order.
    let keys: Vec<(&str, &DocDepth, Option<&str>)> = board
        .docs
        .iter()
        .map(|d| (d.key.as_str(), &d.depth, d.milestone.as_deref()))
        .collect();
    let expanded = DocDepth::Expanded;
    let summary = DocDepth::Summary;
    assert_eq!(
        keys,
        vec![
            ("README", &expanded, None),
            ("00-overview", &expanded, None),
            ("01", &expanded, None),
            ("A", &expanded, None),
            ("B", &expanded, None),
            ("C", &expanded, None),
            ("D", &expanded, None),
            ("E", &expanded, None),
            ("F", &expanded, None),
            ("G", &expanded, None),
            ("08", &summary, Some("M1")),
            ("22", &summary, Some("M1")),
        ]
    );

    // §5: milestones with titles, checked state, bodies and markers.
    assert_eq!(board.milestones.len(), 2);
    assert_eq!(board.milestones[0].id, "M0");
    assert!(board.milestones[0].checked);
    assert_eq!(
        board.milestones[0].title,
        "groundwork. ✅ COMPLETE 2026-09-16"
    );
    assert_eq!(
        board.milestones[0].body,
        // The trailing newline is the blank line before the M1
        // sub-step table — the raw span runs to the table's header.
        "**M0 — groundwork. ✅ COMPLETE 2026-09-16** (one session; commits `a1b2c3d`.)\n      \
         Second body line of M0.\n"
    );
    assert_eq!(
        board.milestones[0].markers,
        vec![Marker::Check {
            line: "**M0 — groundwork. ✅ COMPLETE 2026-09-16** (one session; commits `a1b2c3d`.)"
                .to_owned()
        }]
    );
    assert_eq!(board.milestones[1].id, "M1");
    assert!(!board.milestones[1].checked);
    assert_eq!(board.milestones[1].title, "the engine and interface.");
    // The indented 🧠 line is body text, but it sits in M1's span, so
    // its emoji is a marker — extraction runs over the full span.
    assert_eq!(
        board.milestones[1].markers,
        vec![Marker::Brainstorm {
            line: "   - [ ] **🧠 the loop design** — brainstorm first, indented body text."
                .to_owned()
        }]
    );

    // §5: sub-steps flat, in board order, gates raw.
    let substeps: Vec<(&str, &str, &str)> = board
        .substeps
        .iter()
        .map(|s| (s.id.as_str(), s.text.as_str(), s.gate.as_str()))
        .collect();
    assert_eq!(
        substeps,
        vec![
            ("M0.1", "deepen the two foundation docs", "docs only"),
            ("M0.2", "the workspace layout", "both reviewers"),
            ("M1.1", "the engine's first crate", "both reviewers"),
        ]
    );

    // §6: subsection titles and U items in board order.
    assert_eq!(board.followups.len(), 2);
    assert_eq!(board.followups[0].title.as_deref(), Some("first group"));
    assert_eq!(
        board.followups[1].title.as_deref(),
        Some("second subsection")
    );

    let first = &board.followups[0].items;
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].id, "U1");
    assert!(!first[0].checked);
    assert_eq!(first[0].title, "First follow-up.");
    assert_eq!(
        first[0].body,
        "**U1 · First follow-up.** The figures drift between runs.\n      Continuation line of U1."
    );
    assert_eq!(first[0].markers, vec![]);
    assert_eq!(first[1].id, "U2");
    assert!(first[1].checked);
    assert_eq!(first[1].title, "Second follow-up. ✅ done 2026-09-18");
    assert_eq!(
        first[1].markers,
        vec![
            Marker::Check {
                line: "**U2 · Second follow-up. ✅ done 2026-09-18** blocked on U1.".to_owned()
            },
            Marker::BlockedOn {
                id: "U1".to_owned(),
                line: "**U2 · Second follow-up. ✅ done 2026-09-18** blocked on U1.".to_owned()
            },
            Marker::MilestoneAssertion {
                milestone: "M1".to_owned(),
                line: "      gates M1 in prose.".to_owned()
            },
        ]
    );

    let second = &board.followups[1].items;
    assert_eq!(second.len(), 3);
    assert_eq!(second[0].id, "U3");
    assert_eq!(
        second[0].markers,
        vec![Marker::MilestoneAssertion {
            milestone: "M2".to_owned(),
            line: "**U3 · Third item.** due by M2.".to_owned()
        }]
    );
    assert_eq!(second[1].id, "U4");
    assert_eq!(second[1].markers, vec![], "before M4 is not recognized");
    assert_eq!(second[2].id, "U5");
    assert_eq!(
        second[2].markers,
        vec![
            Marker::InWork {
                line: "**U5 · Fifth item.** ▶️ started, ⚠️ caveat.".to_owned()
            },
            Marker::Warning {
                line: "**U5 · Fifth item.** ▶️ started, ⚠️ caveat.".to_owned()
            },
        ]
    );
}

// =================================================================
// 2. §6 strictness.
// =================================================================

#[test]
fn a_duplicate_u_id_is_a_named_error_naming_both_lines() {
    let corpus = todo_with_section6(
        "### subsection\n\n- [ ] **U1 · First.** body\n- [ ] **U1 · Second.** body",
    );
    let first = line_of(&corpus, "**U1 · First");
    let second = line_of(&corpus, "**U1 · Second");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::DuplicateId {
            id: "U1".to_owned(),
            first,
            second,
        })
    );
}

#[test]
fn a_malformed_bold_run_in_section_6_is_a_named_error() {
    // Not U-shaped: the id is not a U id.
    let corpus = todo_with_section6("### sub\n\n- [ ] **X1 · Not a U id.** body");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MalformedLine {
            line: line_of(&corpus, "**X1 ·"),
            detail: "the bold run is not U-shaped: 'X1' is not a U id".to_owned(),
        })
    );
    // Not U-shaped: the separator is the M grammar's em-dash.
    let corpus = todo_with_section6("### sub\n\n- [ ] **U1 — em-dash form.** body");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MalformedLine {
            line: line_of(&corpus, "**U1 —"),
            detail: "the bold run is not U-shaped: no ' · ' separator after the id".to_owned(),
        })
    );
    // The bold run does not close.
    let corpus = todo_with_section6("### sub\n\n- [ ] **U1 · Unclosed. body");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MalformedLine {
            line: line_of(&corpus, "**U1 · Unclosed"),
            detail: "a U item's bold run does not close".to_owned(),
        })
    );
}

#[test]
fn an_indented_checkbox_under_a_u_item_is_body_text() {
    let corpus = todo_with_section6(
        "### subsection\n\n- [ ] **U1 · Parent.** body\n  - [ ] **🧠 indented child** not an item",
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(board.followups.len(), 1);
    assert_eq!(
        board.followups[0].items.len(),
        1,
        "the child is not an item"
    );
    assert_eq!(board.followups[0].items[0].id, "U1");
    assert!(
        board.followups[0].items[0]
            .body
            .contains("  - [ ] **🧠 indented child**"),
        "the indented line is body text, verbatim"
    );
}

// =================================================================
// 3. §5 strictness.
// =================================================================

#[test]
fn an_unknown_header_table_inside_a_milestone_body_is_body_text() {
    // A third milestone whose body carries a table with an unknown
    // header: the table rides in the body, and no sub-step appears.
    let prefix = valid_through_5();
    let corpus = format!(
        "{prefix}- [ ] **M2 — the second milestone.** body start\n| Ask | Produced |\n|---|---|\n\
         | one | two |\nbody continues\n\n## 6. Open follow-ups\n\n### sub\n\n\
         - [ ] **U1 · One.** body"
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(board.milestones.len(), 3);
    let m2 = &board.milestones[2];
    assert_eq!(m2.id, "M2");
    assert!(
        m2.body.contains("| Ask | Produced |") && m2.body.contains("| one | two |"),
        "the unknown table rides in the body: {}",
        m2.body
    );
    // No new sub-steps: the base three only.
    assert_eq!(board.substeps.len(), 3);
}

#[test]
fn a_substep_row_with_the_wrong_column_count_is_a_named_error() {
    let prefix = valid_through_5();
    let corpus = format!(
        "{prefix}| | Sub-step | Gate |\n|---|---|---|\n\
         | **M2.1** | text | gate | extra |\n\n## 6. Open follow-ups\n\n### sub\n\n\
         - [ ] **U1 · One.** body"
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::ColumnCount {
            line: line_of(&corpus, "| extra |"),
            expected: 3,
            found: 4,
        })
    );
}

#[test]
fn an_indented_brainstorm_checkbox_in_section_5_is_body_text() {
    // The base corpus's M1 body carries the indented 🧠 line.
    let board = parse_todo_md(&todo_md()).unwrap();
    assert_eq!(
        board.milestones.len(),
        2,
        "the indented line is not an item"
    );
    assert!(
        board.milestones[1]
            .body
            .contains("   - [ ] **🧠 the loop design**"),
        "the indented line is M1's body text, verbatim"
    );
}

// =================================================================
// 4. The doc register.
// =================================================================

#[test]
fn the_appendices_phrase_expands_to_a_through_g() {
    let board = parse_todo_md(&todo_md()).unwrap();
    let appendices: Vec<&str> = board
        .docs
        .iter()
        .filter(|d| d.key.len() == 1)
        .map(|d| d.key.as_str())
        .collect();
    assert_eq!(appendices, vec!["A", "B", "C", "D", "E", "F", "G"]);
}

#[test]
fn struck_entries_are_not_summary_entries() {
    // M1's cell strikes 03; the expanded list does not carry it, and
    // struck-but-not-expanded is legal — no entry, no error.
    let board = parse_todo_md(&todo_md()).unwrap();
    assert!(
        !board.docs.iter().any(|d| d.key == "03"),
        "the struck 03 yields no doc entry"
    );
}

#[test]
fn a_doc_both_expanded_and_unstruck_summary_is_a_named_error() {
    let section4 = SECTION_4.replace(
        "Already expanded (4): `README`, `00-overview`, **`01`**, and the **seven** appendices (A–G).",
        "Already expanded (5): `README`, `00-overview`, **`01`**, 08, and the **seven** appendices (A–G).",
    );
    let corpus = format!(
        "## 2. Decisions\n\n| # | Decision | Recommendation | Status |\n|---|---|---|---|\n\
         | ~~B1~~ | one | rec | ✅ resolved |\n\nB1a (`x`) is done.\n\n{section4}\
         ## 5. Work\n\n- [ ] **M0 — start.** body\n\n## 6. Follow-ups\n\n### sub\n\n\
         - [ ] **U1 · One.** body"
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::DocContradiction {
            key: "08".to_owned(),
            line: line_of(&corpus, "| **M1** |"),
        })
    );
}

#[test]
fn the_em_dash_annotation_cell_is_legal_and_empty() {
    let board = parse_todo_md(&todo_md()).unwrap();
    assert!(
        !board
            .docs
            .iter()
            .any(|d| d.milestone.as_deref() == Some("M0")),
        "M0's — *(annotation)* cell contributes no entries"
    );
}

// =================================================================
// 5. The marker grammar.
// =================================================================

#[test]
fn each_emoji_marker_is_recognized_with_its_raw_line() {
    let corpus = todo_with_section6(
        "### sub\n\n- [ ] **U7 · Markers.** start\n✅ done on this line\n▶️ started here\n\
         ⚠️ caveat here\n🧠 brainstorm here",
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(
        board.followups[0].items[0].markers,
        vec![
            Marker::Check {
                line: "✅ done on this line".to_owned()
            },
            Marker::InWork {
                line: "▶️ started here".to_owned()
            },
            Marker::Warning {
                line: "⚠️ caveat here".to_owned()
            },
            Marker::Brainstorm {
                line: "🧠 brainstorm here".to_owned()
            },
        ]
    );
}

#[test]
fn blocked_on_recognizes_the_id_bearing_form_only() {
    let corpus = todo_with_section6(
        "### sub\n\n- [ ] **U8 · Blocked.** blocked on U53 in the body.\n\
         - [ ] **U9 · Prose.** it stayed blocked on the store for weeks.\n\
         - [ ] **U10 · Unblocked.** ✅ unblocked on B1 today.",
    );
    let board = parse_todo_md(&corpus).unwrap();
    let items = &board.followups[0].items;
    assert_eq!(
        items[0].markers,
        vec![Marker::BlockedOn {
            id: "U53".to_owned(),
            line: "**U8 · Blocked.** blocked on U53 in the body.".to_owned()
        }]
    );
    assert_eq!(items[1].markers, vec![], "prose without an id is no marker");
    // The ✅ still counts; the `unblocked on` prefix does not match.
    assert_eq!(
        items[2].markers,
        vec![Marker::Check {
            line: "**U10 · Unblocked.** ✅ unblocked on B1 today.".to_owned()
        }]
    );
}

#[test]
fn the_three_milestone_assertion_spellings_are_one_marker_kind() {
    let corpus = todo_with_section6(
        "### sub\n\n- [ ] **U11 · Assertions.** start\none: gates M1\ntwo: Blocking M2\n\
         three: due by M3",
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(
        board.followups[0].items[0].markers,
        vec![
            Marker::MilestoneAssertion {
                milestone: "M1".to_owned(),
                line: "one: gates M1".to_owned()
            },
            Marker::MilestoneAssertion {
                milestone: "M2".to_owned(),
                line: "two: Blocking M2".to_owned()
            },
            Marker::MilestoneAssertion {
                milestone: "M3".to_owned(),
                line: "three: due by M3".to_owned()
            },
        ]
    );
}

#[test]
fn before_mn_is_not_recognized() {
    let corpus = todo_with_section6("### sub\n\n- [ ] **U12 · Before.** land this before M4.");
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(
        board.followups[0].items[0].markers,
        vec![],
        "vague temporal prose stays body text"
    );
}

#[test]
fn a_marker_inside_the_bold_run_counts() {
    let corpus = todo_with_section6("### sub\n\n- [x] **U13 · Inside. ✅ done 2026-09-18** rest.");
    let board = parse_todo_md(&corpus).unwrap();
    let item = &board.followups[0].items[0];
    assert_eq!(
        item.markers,
        vec![Marker::Check {
            line: "**U13 · Inside. ✅ done 2026-09-18** rest.".to_owned()
        }]
    );
    assert!(
        item.body.contains("✅ done 2026-09-18"),
        "the marker is never removed from the raw body"
    );
}

// =================================================================
// 6. The composition and its errors.
// =================================================================

#[test]
fn parse_board_happy_path_composes_the_three_files() {
    let board = parse_board(&todo_md(), B_DECISIONS, A_DELTAS)
        .unwrap_or_else(|e| panic!("composition failed: {e}"));
    assert_eq!(board.todo.decisions.len(), 2);
    assert_eq!(board.decisions.summary.len(), 2);
    assert_eq!(board.decisions.dossiers.len(), 2);
    assert_eq!(board.deltas.index.len(), 2);
    assert_eq!(board.deltas.dossiers.len(), 2);
    assert_eq!(
        board.decisions.dossiers[0].status.kind,
        BDossierKind::Resolved
    );
    assert_eq!(board.deltas.dossiers[0].status.kind, ADossierKind::Applied);
}

#[test]
fn a_cross_file_id_mismatch_is_a_named_error_naming_the_id() {
    // A third dossier the TODO.md §2 table does not carry.
    let b_with_extra = format!(
        "{B_DECISIONS}### 3. [B3] Third decision — ✅ **RESOLVED 2026-09-17 — the minimal form stands**\n\n\
         Dossier body for B3.\n"
    );
    assert_eq!(
        parse_board(&todo_md(), &b_with_extra, A_DELTAS),
        Err(ImportError::IdSetMismatch {
            id: "B3".to_owned(),
            detail: "B3 is in the B dossiers but not in TODO.md §2".to_owned(),
        })
    );
}

#[test]
fn a_missing_section_4_is_a_named_error_naming_it() {
    let corpus = format!("{HEAD}{SECTION_5}{SECTION_6}");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MissingSection {
            section: "§4".to_owned(),
            line: corpus.split('\n').count(),
        })
    );
}

#[test]
fn a_missing_b1a_is_a_named_error() {
    let corpus = todo_md().replace(
        "B1a (`codec::Value`) is **done** — implemented and committed.\n\n",
        "",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MissingB1a {
            line: corpus.split('\n').count(),
        })
    );
}

// =================================================================
// 7. Determinism: the same corpus parsed twice yields equal ASTs.
// =================================================================

#[test]
fn parsing_the_same_corpus_twice_yields_equal_asts() {
    // The premise first (P-LIVE's comparison rule): both parses are
    // successes, or the equality below would compare two identical
    // failures.
    let first = parse_todo_md(&todo_md()).expect("the first parse succeeds");
    let second = parse_todo_md(&todo_md()).expect("the second parse succeeds");
    assert_eq!(first, second);

    let b_first = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).expect("composition succeeds");
    let b_second = parse_board(&todo_md(), B_DECISIONS, A_DELTAS).expect("composition succeeds");
    assert_eq!(b_first, b_second);
}

// =================================================================
// 8. Board order end-to-end.
// =================================================================

#[test]
fn board_order_is_preserved_end_to_end() {
    let corpus = todo_with_section6(
        "### alpha\n\n- [ ] **U10 · Alpha one.** body\n- [ ] **U11 · Alpha two.** body\n\n\
         ### beta\n\n- [ ] **U12 · Beta one.** body\n\n### gamma\n\n- [ ] **U13 · Gamma one.** \
         body\n- [ ] **U14 · Gamma two.** body",
    );
    let board = parse_todo_md(&corpus).unwrap();
    // §6: subsection titles in board order.
    let titles: Vec<&str> = board
        .followups
        .iter()
        .map(|s| s.title.as_deref().unwrap_or(""))
        .collect();
    assert_eq!(titles, vec!["alpha", "beta", "gamma"]);
    // §6: the U ids in board order, flattened across subsections.
    let ids: Vec<&str> = board
        .followups
        .iter()
        .flat_map(|s| s.items.iter().map(|i| i.id.as_str()))
        .collect();
    assert_eq!(ids, vec!["U10", "U11", "U12", "U13", "U14"]);
    // §2, §5 and the sub-steps keep their board order too.
    assert_eq!(
        board
            .decisions
            .iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>(),
        vec!["B1", "B2"]
    );
    assert_eq!(
        board
            .milestones
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        vec!["M0", "M1"]
    );
    assert_eq!(
        board
            .substeps
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        vec!["M0.1", "M0.2", "M1.1"]
    );
}

// =================================================================
// The fix round's widenings: §2's fifth cell, the Prose status
// fallback, the wrapped rows, and the relaxed A cross-check.
// =================================================================

#[test]
fn a_section2_row_may_carry_a_fifth_trailing_cell() {
    // B20's measured shape: the B appendix's Gates column rides inline
    // as a fifth cell — carried raw, never parsed as Gates.
    let corpus = todo_with_section6("### sub\n\n- [ ] **U1 · One.** body").replace(
        "| ~~B2~~ | Delivery surface | ✅ **SIGNED OFF 2026-09-17** — the deferral | ⚠️ resolved, half-applied |",
        "| ~~B2~~ | Delivery surface | ✅ **SIGNED OFF 2026-09-17** — the deferral | ⚠️ resolved, half-applied | ~~03, 11, 19~~ ✅ |",
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(board.decisions[0].extra, None);
    assert_eq!(
        board.decisions[1].extra.as_deref(),
        Some("~~03, 11, 19~~ ✅")
    );
    // The Status cell still parses from the fourth cell.
    assert_eq!(board.decisions[1].status, SummaryStatus::HalfApplied);
}

#[test]
fn a_status_cell_without_a_known_spelling_carries_prose() {
    let corpus = todo_with_section6("### sub\n\n- [ ] **U1 · One.** body")
        .replace(
            "B1a (`codec::Value`) is **done** — implemented and committed.",
            "| ~~B3~~ | third | rec | ✅ resolved → superseded prose |\n\
             | ~~B4~~ | fourth | rec | ⚠️ resolved, half-applied |\n\n\
             B1a (`codec::Value`) is **done** — implemented and committed.",
        )
        .replace(
            "| ~~B2~~ | Delivery surface | ✅ **SIGNED OFF 2026-09-17** — the deferral | ⚠️ resolved, half-applied |",
            "| ~~B2~~ | Delivery surface | ✅ **SIGNED OFF 2026-09-17** — the deferral | ~~med — open~~ → settled in full |",
        );
    let board = parse_todo_md(&corpus).unwrap();
    // B20's measured shape: no segment reads a known spelling, the
    // final segment carries as prose.
    assert_eq!(
        board.decisions[1].status,
        SummaryStatus::Prose("settled in full".to_owned())
    );
    // The right-to-left scan: the last segment reading a known
    // spelling wins over later prose.
    assert_eq!(board.decisions[2].status, SummaryStatus::Resolved);
    // B10's half-applied spelling still parses — the pin the ruling
    // asked for.
    assert_eq!(board.decisions[3].status, SummaryStatus::HalfApplied);
}

#[test]
fn a_wrapped_table_row_joins_with_a_single_space() {
    // A row line that does not end with `|` wraps: the pieces join
    // with a single space, each break replacing a space.
    let base = todo_with_section6("### sub\n\n- [ ] **U1 · One.** body");
    let wrapped = base
        .replace(
            "| **M1** | ~~03 firstdoc~~ *(deepened earlier)*, 08 seconddoc, 22 thirddoc | 3 |",
            "| **M1** | ~~03 firstdoc~~ *(deepened earlier)*, 08 seconddoc,\n22 thirddoc | 3 |",
        )
        .replace(
            "| **M0.1** | deepen the two foundation docs | docs only |",
            "| **M0.1** | deepen the two\nfoundation docs | docs only |",
        );
    let unwrapped = parse_todo_md(&base).unwrap();
    let wrapped_board = parse_todo_md(&wrapped).unwrap();
    // The wrapped milestone row parses identically.
    assert_eq!(wrapped_board.docs, unwrapped.docs);
    // The wrapped sub-step row joins into one three-cell row.
    assert_eq!(
        wrapped_board.substeps[0].text,
        "deepen the two foundation docs"
    );
    assert_eq!(wrapped_board.substeps[0].gate, "docs only");
    assert_eq!(wrapped_board.substeps.len(), unwrapped.substeps.len());
}

#[test]
fn dossiers_may_exceed_the_a_index_but_not_dangle() {
    // A dossier in the applied table but absent from the index —
    // legal under the relaxed cross-check.
    let a_with_extra = format!(
        "{A_DELTAS}## Deltas applied\n\n| # | Section | Applied | Date | Note |\n|---|---|---|---|---|\n\
         | ~~**A3**~~ | §3.3 | **Third delta** | 2026-09-18 | applied |\n\n\
         ### A3 — §3.3: the third delta — ✅ **APPLIED 2026-09-18**\n\nBody for A3.\n"
    );
    let board = parse_board(&todo_md(), B_DECISIONS, &a_with_extra)
        .unwrap_or_else(|e| panic!("the relaxed cross-check accepts it: {e}"));
    assert_eq!(board.deltas.index.len(), 2);
    assert_eq!(board.deltas.applied.len(), 1);
    assert_eq!(board.deltas.dossiers.len(), 3);
    // A dangling table row — an id with no dossier — is still a named
    // error.
    let a_dangling = format!(
        "{A_DELTAS}## Deltas applied\n\n| # | Section | Applied | Date | Note |\n|---|---|---|---|---|\n\
         | ~~**A3**~~ | §3.3 | **Third guard set** | 2026-09-18 | never dossiered |\n"
    );
    assert_eq!(
        parse_board(&todo_md(), B_DECISIONS, &a_dangling),
        Err(ImportError::IdSetMismatch {
            id: "A3".to_owned(),
            detail: "A3 is in the A applied table but has no dossier".to_owned(),
        })
    );
}

#[test]
fn a_wrapped_u_item_bold_run_joins_with_a_single_space() {
    // The bold run may wrap; the break replaces a space, and the raw
    // body carries the wrap verbatim.
    let corpus = todo_with_section6(
        "### sub\n\n- [x] **U7 · A title long enough to wrap onto\n      \
         a second line.** The rest of the item line.\n      A continuation line.",
    );
    let board = parse_todo_md(&corpus).unwrap();
    let item = &board.followups[0].items[0];
    assert_eq!(item.id, "U7");
    assert_eq!(
        item.title,
        "A title long enough to wrap onto a second line."
    );
    assert!(item.body.contains("onto\n      a second line.**"));
    assert!(item.body.contains("The rest of the item line."));
}

// =================================================================
// The review round's negative pins: the id shape, the wrap bounds,
// and the unclosed bold run.
// =================================================================

#[test]
fn a_u_id_accepts_one_trailing_letter_and_rejects_two() {
    // The id shape is `<prefix><digits>[a-z]?` — one trailing letter
    // at most; two are a named error, not a parse.
    let ok = todo_with_section6("### sub\n\n- [ ] **U1a · One trailing letter.** body");
    let board = parse_todo_md(&ok).unwrap();
    assert_eq!(board.followups[0].items[0].id, "U1a");
    let bad = todo_with_section6("### sub\n\n- [ ] **U1ab · Two trailing letters.** body");
    assert_eq!(
        parse_todo_md(&bad),
        Err(ImportError::MalformedLine {
            line: line_of(&bad, "**U1ab ·"),
            detail: "the bold run is not U-shaped: 'U1ab' is not a U id".to_owned(),
        })
    );
    // The same shape gates the table ids.
    let bad_row = todo_with_section6("### sub\n\n- [ ] **U1 · One.** body")
        .replace("| ~~B1~~ | Storage shape |", "| ~~B1ab~~ | Storage shape |");
    assert_eq!(
        parse_todo_md(&bad_row),
        Err(ImportError::BadId {
            line: line_of(&bad_row, "B1ab"),
            id: "B1ab".to_owned(),
            detail: "not a B id".to_owned(),
        })
    );
}

#[test]
fn a_truncated_row_does_not_absorb_the_next_register_construct() {
    // A row missing its trailing pipe must not swallow the register
    // construct that follows it: the milestone stays a milestone, and
    // the partial row faces its own cells.
    let prefix = valid_through_5();
    let corpus = format!(
        "{prefix}| | Sub-step | Gate |\n|---|---|---|\n| **M2.1** | text | gate\n\
         - [ ] **M2 — the second milestone.** body\n\n## 6. Open follow-ups\n\n### sub\n\n\
         - [ ] **U1 · One.** body"
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(
        board.milestones.len(),
        3,
        "the milestone after the truncated row is present"
    );
    assert_eq!(board.milestones[2].id, "M2");
    assert_eq!(
        board.substeps.len(),
        4,
        "the truncated row parses on its own cells"
    );
    assert_eq!(board.substeps[3].id, "M2.1");
}

#[test]
fn an_unclosed_bold_run_before_the_next_item_is_a_named_error() {
    // The wrap scan never crosses a register construct: an unclosed
    // run is a named error, not a title polluted by the next item.
    let corpus = todo_with_section6(
        "### sub\n\n- [ ] **U1 · Title with no close\n- [ ] **U2 · Next item.** body",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MalformedLine {
            line: line_of(&corpus, "**U1 · Title"),
            detail: "a U item's bold run does not close".to_owned(),
        })
    );
}

#[test]
fn an_unclosed_bold_run_before_a_table_is_a_named_error() {
    // The scan never crosses a table row: an unclosed run is a named
    // error, not a title that absorbed the table.
    let corpus = todo_with_section6(
        "### sub\n\n- [ ] **U1 · Title with no close\n| Ask | Produced |\n|---|---|\n| one | two |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MalformedLine {
            line: line_of(&corpus, "**U1 · Title"),
            detail: "a U item's bold run does not close".to_owned(),
        })
    );
    // The §5 M variant: an unclosed milestone run before a sub-step
    // table is the same named error.
    let prefix = valid_through_5();
    let corpus = format!(
        "{prefix}- [ ] **M2 — engine title\n| | Sub-step | Gate |\n|---|---|---|\n\
         | **M2.1** | text | gate |\n\n## 6. Open follow-ups\n\n### sub\n\n- [ ] **U1 · One.** body"
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(ImportError::MalformedLine {
            line: line_of(&corpus, "**M2 — engine title"),
            detail: "a M item's bold run does not close".to_owned(),
        })
    );
}

#[test]
fn an_item_before_the_first_subsection_lands_in_the_implicit_group() {
    // A §6 checkbox before the first `###` opens the implicit leading
    // group — `flush_u` creates it, so the item always lands somewhere.
    let corpus = todo_with_section6(
        "- [ ] **U1 · Before any subsection.** body\n\n### late subsection\n\n\
         - [ ] **U2 · After.** body",
    );
    let board = parse_todo_md(&corpus).unwrap();
    assert_eq!(board.followups.len(), 2);
    assert_eq!(
        board.followups[0].title, None,
        "the implicit leading group has no title"
    );
    assert_eq!(board.followups[0].items.len(), 1);
    assert_eq!(board.followups[0].items[0].id, "U1");
    assert_eq!(board.followups[1].title.as_deref(), Some("late subsection"));
    assert_eq!(board.followups[1].items[0].id, "U2");
}
