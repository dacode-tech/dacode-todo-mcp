//! The A appendix's suite (t42): the index table's Depends-on shapes,
//! the dossiers' three statuses, the structural 1:1 pairing, and the
//! unknown-status rejection. All corpora are hand-authored synthetic
//! strings in the measured dialect.

use std::collections::BTreeSet;

use todo_mcp::import::dacode::{ADossierKind, ImportError, parse_a_deltas};

/// A synthetic A file exercising the Depends-on shapes and all three
/// dossier statuses.
const A_FILE: &str = "\
# Appendix A — Design deltas

Prose intro.

| # | Section | Delta | Depends on | Recommend |
|---|---|---|---|---|
| ~~A1~~ | §3.1, §3.2 | **First delta** | ~~B3~~ ✅ signed, ~~B4~~ ✅ signed | ✅ **APPLIED 2026-09-18** — applied |
| ~~A2~~ | §4.2 | Second delta | — | ✅ **SIGNED OFF 2026-09-17** — approved |
| ~~A3~~ | §3.2, §4.8 | Third delta | — (rode in on ~~A1~~ ✅ for the marker) | ⚠️ **HALF-APPLIED 2026-09-18** — half |

### A1 — §3.1, §3.2: the first delta — ✅ **APPLIED 2026-09-18**

Dossier body for A1.

### A2 — §4.2: the second delta — ✅ **SIGNED OFF 2026-09-17**

Dossier body for A2.

### A3 — §3.2, §4.8: the third delta — ⚠️ **HALF-APPLIED 2026-09-18**

Dossier body for A3.
";

#[test]
fn the_a_index_parses_depends_on_shapes() {
    let appendix = parse_a_deltas(A_FILE).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(appendix.index.len(), 3);

    // Struck and plain B ids: strikes are noise, the dependency
    // exists regardless.
    let a1 = &appendix.index[0];
    assert_eq!(a1.id, "A1");
    assert_eq!(a1.sections, vec!["§3.1", "§3.2"]);
    assert_eq!(a1.delta, "**First delta**");
    assert_eq!(a1.depends_on, vec!["B3", "B4"]);

    // The empty cell.
    let a2 = &appendix.index[1];
    assert_eq!(a2.id, "A2");
    assert_eq!(a2.sections, vec!["§4.2"]);
    assert_eq!(a2.depends_on, Vec::<String>::new());

    // A cell carrying an A id.
    let a3 = &appendix.index[2];
    assert_eq!(a3.id, "A3");
    assert_eq!(a3.sections, vec!["§3.2", "§4.8"]);
    assert_eq!(a3.depends_on, vec!["A1"]);
}

#[test]
fn the_a_dossiers_parse_all_three_statuses() {
    let appendix = parse_a_deltas(A_FILE).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(appendix.dossiers.len(), 3);

    let a1 = &appendix.dossiers[0];
    assert_eq!(a1.id, "A1");
    assert_eq!(a1.title, "§3.1, §3.2: the first delta");
    assert_eq!(a1.status.kind, ADossierKind::Applied);
    assert_eq!(a1.status.date, "2026-09-18");
    assert_eq!(a1.outcome, "");
    assert!(a1.body.contains("Dossier body for A1."));

    let a2 = &appendix.dossiers[1];
    assert_eq!(a2.id, "A2");
    assert_eq!(a2.title, "§4.2: the second delta");
    assert_eq!(a2.status.kind, ADossierKind::SignedOff);
    assert_eq!(a2.status.date, "2026-09-17");
    assert!(a2.body.contains("Dossier body for A2."));

    let a3 = &appendix.dossiers[2];
    assert_eq!(a3.id, "A3");
    assert_eq!(a3.title, "§3.2, §4.8: the third delta");
    assert_eq!(a3.status.kind, ADossierKind::HalfApplied);
    assert_eq!(a3.status.date, "2026-09-18");
    assert!(a3.body.contains("Dossier body for A3."));
}

#[test]
fn the_a_file_pairs_index_and_dossiers_one_to_one() {
    let appendix = parse_a_deltas(A_FILE).unwrap_or_else(|e| panic!("parse failed: {e}"));
    let index: BTreeSet<&str> = appendix.index.iter().map(|r| r.id.as_str()).collect();
    let dossiers: BTreeSet<&str> = appendix.dossiers.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(index, dossiers);
    assert_eq!(index.len(), 3);
}

#[test]
fn an_unknown_a_dossier_status_is_a_named_error() {
    let a_file = "| # | Section | Delta | Depends on | Recommend |\n|---|---|---|---|---|\n\
                  | ~~A1~~ | §3.1 | delta | — | rec |\n\n\
                  ### A1 — §3.1: title — ✅ **PENDING 2026-09-18**\n\nBody.\n";
    assert_eq!(
        parse_a_deltas(a_file),
        Err(ImportError::BadStatus {
            line: 5,
            text: "PENDING 2026-09-18**".to_owned(),
            detail: "the marker must read 'APPLIED <date>', 'SIGNED OFF <date>' or 'HALF-APPLIED <date>'"
                .to_owned(),
        })
    );
}

// =================================================================
// The fix round's widenings: the applied table, the code-span pipe,
// and the arrow locator with the via-form.
// =================================================================

#[test]
fn the_applied_table_rows_are_carried() {
    let a_file = format!(
        "{A_FILE}## Deltas applied\n\n| # | Section | Applied | Date | Note |\n|---|---|---|---|---|\n\
         | ~~**A1**~~ | §3.1, §3.2 | **First delta** | 2026-09-18 | applied first |\n"
    );
    let appendix = parse_a_deltas(&a_file).unwrap();
    assert_eq!(appendix.applied.len(), 1);
    let row = &appendix.applied[0];
    assert_eq!(row.id, "A1");
    assert_eq!(row.sections, vec!["§3.1", "§3.2"]);
    assert_eq!(row.applied, "**First delta**");
    assert_eq!(row.date, "2026-09-18");
    assert_eq!(row.note, "applied first");
    // The applied table is optional: the base file carries none.
    let base = parse_a_deltas(A_FILE).unwrap();
    assert_eq!(base.applied.len(), 0);
}

#[test]
fn a_pipe_inside_a_code_span_does_not_split_the_row() {
    // A15's measured shape: a cell carries `plain | compressed`.
    let a_file = A_FILE.replace(
        "| ~~A2~~ | §4.2 | Second delta | — | ✅ **SIGNED OFF 2026-09-17** — approved |",
        "| ~~A2~~ | §4.2 | codec `plain | compressed` | — | ✅ **SIGNED OFF 2026-09-17** — approved |",
    );
    let appendix = parse_a_deltas(&a_file).unwrap();
    assert_eq!(appendix.index.len(), 3);
    assert_eq!(appendix.index[1].delta, "codec `plain | compressed`");
}

#[test]
fn the_arrow_locator_and_the_applied_via_form_parse() {
    // The arrow marker intro ` → ✅ **` with the via-form status
    // `APPLIED via A16, 2026-09-12`.
    let a_file = A_FILE.replace(
        "### A2 — §4.2: the second delta — ✅ **SIGNED OFF 2026-09-17**",
        "### A2 — §4.2: the second delta → ✅ **APPLIED via A16, 2026-09-12**",
    );
    let appendix = parse_a_deltas(&a_file).unwrap();
    let a2 = &appendix.dossiers[1];
    assert_eq!(a2.title, "§4.2: the second delta");
    assert_eq!(a2.status.kind, ADossierKind::Applied);
    assert_eq!(a2.status.via.as_deref(), Some("A16"));
    assert_eq!(a2.status.date, "2026-09-12");
    assert_eq!(a2.outcome, "");
}
