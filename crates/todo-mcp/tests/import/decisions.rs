//! The B appendix's suite (t42): the summary table's Gates shapes,
//! the dossiers' status grammar forms, the structural 1:1 pairing,
//! the unknown-status rejection, and §2's three Status-cell
//! spellings. All corpora are hand-authored synthetic strings in the
//! measured dialect.

use std::collections::BTreeSet;

use todo_mcp::import::dacode::{
    AppliedVia, BDossierKind, ImportError, SummaryStatus, parse_b_decisions, parse_todo_md,
};

/// A synthetic B file exercising every measured Gates shape and every
/// dossier status grammar form.
const B_FILE: &str = "\
# Appendix B — Open decisions needing sign-off

Prose intro, walked past.

| # | Decision | Recommendation | Confidence | Gates |
|---|---|---|---|---|
| ~~**B1**~~ | ~~Storage shape~~ | ✅ **SIGNED OFF 2026-09-12** — the recommendation taken | high | ~~02, 03 → M0~~ |
| ~~**B2**~~ | ~~Delivery surface~~ | ✅ **SIGNED OFF 2026-09-17** — the deferral | ~~med~~ | ~~21, 19~~ ✅ |
| ~~**B3**~~ | ~~Update granularity~~ | ✅ **SIGNED OFF 2026-09-17** — the minimal form | high | ~~13~~ ✅ |

### 1. [B1] Storage shape — ✅ **RESOLVED 2026-09-12: the recommendation taken**

Dossier body one for B1.

### 2. [B2] Delivery surface — ✅ **SIGNED OFF 2026-09-17, APPLIED 2026-09-18 via A8: the deferral taken as recommended**

Dossier body for B2.

### 3. [B3] Update granularity — ✅ **RESOLVED 2026-09-17 — the minimal form stands**

Dossier body for B3.
";

/// A minimal TODO.md whose §2 carries the three measured Status-cell
/// spellings.
const TODO_THREE_SPELLINGS: &str = "\
## 2. Decisions

| # | Decision | Recommendation | Status |
|---|---|---|---|
| ~~B1~~ | one | rec | ✅ resolved |
| ~~B2~~ | two | rec | ⚠️ resolved, half-applied |
| ~~B3~~ | three | rec | ~~⚠️ reopened~~ → ✅ resolved |

B1a (`x`) is done.

## 4. Doc depth

Already expanded (1): `README`.

| Milestone | Docs still summary | Count |
|---|---|---|
| **M0** | — | **0** |

## 5. Work

- [ ] **M0 — start.** body

## 6. Follow-ups

### sub

- [ ] **U1 · One.** body
";

#[test]
fn the_b_summary_parses_each_gates_shape() {
    let appendix = parse_b_decisions(B_FILE).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(appendix.summary.len(), 3);
    // With a milestone: `~~02, 03 → M0~~`.
    assert_eq!(appendix.summary[0].id, "B1");
    assert_eq!(appendix.summary[0].title, "~~Storage shape~~");
    assert_eq!(appendix.summary[0].gates.docs, vec!["02", "03"]);
    assert_eq!(appendix.summary[0].gates.milestone.as_deref(), Some("M0"));
    // Without a milestone, strikes and a stray ✅: `~~21, 19~~ ✅`.
    assert_eq!(appendix.summary[1].id, "B2");
    assert_eq!(appendix.summary[1].gates.docs, vec!["21", "19"]);
    assert_eq!(appendix.summary[1].gates.milestone, None);
    // The bare struck form: `~~13~~ ✅`.
    assert_eq!(appendix.summary[2].id, "B3");
    assert_eq!(appendix.summary[2].gates.docs, vec!["13"]);
    assert_eq!(appendix.summary[2].gates.milestone, None);
}

#[test]
fn the_b_dossiers_parse_each_status_grammar_form() {
    let appendix = parse_b_decisions(B_FILE).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(appendix.dossiers.len(), 3);

    // RESOLVED with the colon separator.
    let b1 = &appendix.dossiers[0];
    assert_eq!(b1.seq, 1);
    assert_eq!(b1.id, "B1");
    assert_eq!(b1.title, "Storage shape");
    assert_eq!(b1.status.kind, BDossierKind::Resolved);
    assert_eq!(b1.status.date, "2026-09-12");
    assert_eq!(b1.status.applied, None);
    assert_eq!(b1.outcome, "the recommendation taken");
    assert!(b1.body.contains("Dossier body one for B1."));

    // SIGNED OFF with the APPLIED-via clause.
    let b2 = &appendix.dossiers[1];
    assert_eq!(b2.seq, 2);
    assert_eq!(b2.id, "B2");
    assert_eq!(b2.title, "Delivery surface");
    assert_eq!(b2.status.kind, BDossierKind::SignedOff);
    assert_eq!(b2.status.date, "2026-09-17");
    assert_eq!(
        b2.status.applied,
        Some(AppliedVia {
            date: "2026-09-18".to_owned(),
            via: "A8".to_owned(),
        })
    );
    assert_eq!(b2.outcome, "the deferral taken as recommended");
    assert!(b2.body.contains("Dossier body for B2."));

    // RESOLVED with the em-dash separator.
    let b3 = &appendix.dossiers[2];
    assert_eq!(b3.seq, 3);
    assert_eq!(b3.id, "B3");
    assert_eq!(b3.status.kind, BDossierKind::Resolved);
    assert_eq!(b3.status.date, "2026-09-17");
    assert_eq!(b3.outcome, "the minimal form stands");
    assert!(b3.body.contains("Dossier body for B3."));
}

#[test]
fn the_b_file_pairs_summary_and_dossiers_one_to_one() {
    let appendix = parse_b_decisions(B_FILE).unwrap_or_else(|e| panic!("parse failed: {e}"));
    let summary: BTreeSet<&str> = appendix.summary.iter().map(|r| r.id.as_str()).collect();
    let dossiers: BTreeSet<&str> = appendix.dossiers.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(summary, dossiers);
    assert_eq!(summary.len(), 3);
}

#[test]
fn an_unknown_b_dossier_status_is_a_named_error() {
    let b_file = "| # | Decision | Recommendation | Confidence | Gates |\n|---|---|---|---|---|\n\
                  | ~~**B1**~~ | title | rec | high | ~~02~~ |\n\n\
                  ### 1. [B1] Title — ✅ **DECIDED 2026-09-12: outcome**\n\nBody.\n";
    assert_eq!(
        parse_b_decisions(b_file),
        Err(ImportError::BadStatus {
            line: 5,
            text: "DECIDED 2026-09-12: outcome**".to_owned(),
            detail: "the marker must read 'RESOLVED <date>' or 'SIGNED OFF <date>'".to_owned(),
        })
    );
}

#[test]
fn the_three_section2_status_spellings_parse() {
    let board = parse_todo_md(TODO_THREE_SPELLINGS).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(board.decisions.len(), 3);
    assert_eq!(board.decisions[0].status, SummaryStatus::Resolved);
    assert_eq!(board.decisions[1].status, SummaryStatus::HalfApplied);
    // The superseded-then-resolved spelling takes the last
    // arrow-separated segment.
    assert_eq!(board.decisions[2].status, SummaryStatus::Resolved);
}

#[test]
fn gates_doc_entries_accept_appendix_letters() {
    // B11's measured shape: `~~05, 25, D~~ ✅` — D is an appendix doc.
    let b_file = B_FILE.replace(
        "| ~~**B3**~~ | ~~Update granularity~~ | ✅ **SIGNED OFF 2026-09-17** — the minimal form | high | ~~13~~ ✅ |",
        "| ~~**B3**~~ | ~~Update granularity~~ | ✅ **SIGNED OFF 2026-09-17** — the minimal form | high | ~~05, 25, D~~ ✅ |",
    );
    let appendix = parse_b_decisions(&b_file).unwrap();
    assert_eq!(appendix.summary[2].gates.docs, vec!["05", "25", "D"]);
    // A letter outside A–G stays a named error.
    let bad = b_file.replace("~~05, 25, D~~ ✅", "~~05, 25, H~~ ✅");
    assert_eq!(
        parse_b_decisions(&bad),
        Err(ImportError::BadId {
            line: 9,
            id: "H".to_owned(),
            detail: "not a two-digit doc number or an A–G appendix letter".to_owned(),
        })
    );
}

#[test]
fn a_gates_milestone_that_is_not_an_m_id_is_a_named_error() {
    let b_file = B_FILE.replace("~~02, 03 → M0~~", "~~02, 03 → NOT_A_MILESTONE~~");
    assert_eq!(
        parse_b_decisions(&b_file),
        Err(ImportError::BadId {
            line: 7,
            id: "NOT_A_MILESTONE".to_owned(),
            detail: "not an M id".to_owned(),
        })
    );
}
