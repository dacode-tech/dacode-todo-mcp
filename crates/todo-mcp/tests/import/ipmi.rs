//! The ipmi-collector dialect's suite (t77): the parser (TODO.md's
//! §-prefixed section walker, §2's two-column milestone table, §4's
//! open-only checkbox register, §5's checkbox-less debt register),
//! the strictness and never-fired error pins (duplicates, absent
//! registers, column counts, table ids, the checked-item refusal),
//! the decisions file's five-column table and dossiers with the
//! composition's id cross-check, the mapping (register-order ids,
//! board-id references, the statuses, the kinds, the synthetics, the
//! Gates edges and findings), the id reproduction at synthetic
//! scale, the `dialect` param's wire drive, and determinism. All
//! corpora are hand-authored synthetic strings in the measured
//! dialect — invented titles, prose and cells over the measured
//! shapes — the live ipmi-collector board is the import probe's
//! subject (D12), never copied here: no live prose, no golden
//! pinning it.
//!
//! The wire test drives the real binary over a synthetic
//! ipmi-collector tree and asserts the counts response and spot ids
//! in-test — no golden (D12's signed consequence) — plus the
//! dispatch and fault pins: an unknown dialect value is `invalid`
//! naming it, the absent parameter still routes to the dacode path
//! (the default unchanged), and the dialect's missing-file io fault
//! names its own file's role.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{Repo, RepoSlug, Timestamp};
use todo_mcp::import::ipmi::{
    DecisionStatusCell, IpmiError, map_ipmi, parse_decisions, parse_ipmi_board, parse_todo_md,
};
use todo_mcp::import::plan::{ImportOptions, PlanOp, apply_plan};
use todo_store::{DataRoot, Store};

use crate::common::{assert_carrier, call, drive, structured};

// =================================================================
// The corpora — a synthetic minimal ipmi-collector board composed of
// section pieces, so the error-path corpora can swap one section out
// without duplicating the rest. §1, §3 and §6 are walked past, so
// their content exists to prove exactly that — §3 carries column-zero
// bullets and §6 carries a checkbox item, both register-shaped, both
// body. Every string is invented; only the shapes are measured.
// =================================================================

/// The intro and §1: current-state prose, walked past — its bold
/// paragraph and its foreign-header table never reach a register.
const HEAD: &str = "\
# TODO — synthetic

Rules prose, walked past.

## §1 Current state

**U4 resolved — the fixture swap.** The retention probe verifies it;
sessions open with the checklist and close with the report.

| thing | count |
|---|---|
| a | 1 |
";

/// §2: the milestone table — four rows, the measured two-column
/// shape, the State cells carrying `**done**` plus completion prose.
const SECTION_2: &str = "\
## §2 Milestones

| Milestone | State |
|---|---|
| M0 — scaffold, lint, and the release pipeline | **done** 2026-09-30, committed 2026-10-01 |
| M1 — settle the loader rewrite: land, adjust, or revert | **done** 2026-10-02 (`aa11b2c`) |
| M2 — the test foundation: table-driven suites for the loader | **done** 2026-10-02 (this commit) |
| M3 — repository hygiene and the CI wiring | **done** 2026-10-03 (the hygiene pair with D1) |
";

/// §3: the landed log, walked past — its column-zero bullets never
/// reach a register walker.
const SECTION_3: &str = "\
## §3 Landed

History highlights (see `git log` for the full record):

- the parser rewrite (`aa11b2c`), the retry backoff (`bb22c3d`),
  the fixture loader dedup (`cc33d4e`).
- 2026-09-30: the review workflow and the board itself landed.
";

/// §4: the follow-ups register — five items in a planted non-numeric
/// file order (the register-order convention's pin; the ids are
/// gapped and invented, none inside the store's U1–U6 range), two
/// bold runs that wrap, one title carrying an inner ` — `, one
/// carrying backticks.
const SECTION_4: &str = "\
## §4 Follow-ups (unowned)

- [ ] **U12 · the fixture loader ignores the retry budget** —
      the loader config caps retries at two; the flaky vendor
      endpoint needs five before it answers.
- [ ] **U20 · `--dry-run` and the port flag collide in the argument
      parser** — direction (user, 2026-10-02): the parser rejects
      both flags together, so split them or document the conflict.
- [ ] **U15 · the open split — fixtures as the source of truth** —
      the fixture tree becomes the canonical input for every
      suite; the mechanism is D4's to choose.
- [ ] **U23 · the sampler reads the shared map by name, not by
      snapshot** — two windows exposed by the concurrency tests;
      the fix passes the snapshot into the goroutine.
- [ ] **U18 · the README documents a key the parser rejects** —
      the example config carries a key the validator refuses;
      align the two surfaces.
";

/// §5: the debt register — one checkbox-less D-N bullet, its prose
/// wrapping.
const SECTION_5: &str = "\
## §5 Debt

- **D-7 · the vendored checksums are two releases stale** — CI
      verifies against the newer tag while the container build
      stays on the older pin; refresh them with the next planned
      bump instead.
";

/// §6: the session narrative, walked past — its `###` subsections and
/// its checkbox item are body, never register constructs.
const SECTION_6: &str = "\
## §6 Session log (newest first)

### 2026-10-09 — U4: the fixture swap

**Workspace**: main, clean at start.

- [ ] **U99 · a §6 checkbox item is body text.** Never a register item.
";

/// The full synthetic ipmi-collector TODO.md.
fn todo_md() -> String {
    format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}{SECTION_5}{SECTION_6}")
}

/// The synthetic ipmi-collector decisions.md, aligned with the
/// corpus's registers: four rows (two signed, two deferred — the
/// measured vocabulary), the Gates cells covering the three shapes
/// (the bare em-dash, id-free prose, one id-bearing token naming the
/// register's third item), and four dossiers — D2's header tail reads
/// `open` where its table row reads `deferred` (the measured shape;
/// the table is the status authority) — whose bodies carry option
/// bullets (body text, never register constructs).
const DECISIONS_MD: &str = "\
# Decisions — open `D` ids and sign-off record

Agents propose and sharpen options; **the user signs**. Prose intro,
walked past.

| Id | Title | Status | Gates | Recommendation |
|---|---|---|---|---|
| D1 | Direction of the loader rewrite | **signed** (b) 2026-10-03 | — | (b) land with the two fixes |
| D2 | the artifact publishing channel | **deferred** 2026-10-04 | enabling the `publish` job in CI | snapshots only for now |
| D3 | the CI lint runner | **signed** 2026-10-03 | — | the newer runner image |
| D4 | the fixture split mechanism | **deferred** 2026-10-03 | U15 rollout; standing up the fixture repo | (b) fresh repo, squashed history — deferred |

---

## D1 · Direction of the loader rewrite — signed (b) 2026-10-03

The tree carries a half-finished loader split behind a flag.

**Options**

- **(a) Land as-is** — commit the split and fix what review finds.
- **(b) Land with fixes** — repair the two findings first.

**Signed (b) on 2026-10-03.** Executed in the same change.

---

## D2 · the artifact publishing channel — open

The release config builds both binaries with publishing disabled.

**Deferred by the user 2026-10-04**: revisit once the first external
consumer asks for artifacts.

---

## D3 · the CI lint runner — signed 2026-10-03

The tag-pipeline lint job comes from the shared include.

---

## D4 · the fixture split mechanism — deferred 2026-10-03

How the fixture repository is born from this one.
";

/// The import options the mapping tests share: a fixed slug and
/// stamp, an empty `doc_paths` (the board has no doc register).
fn opts() -> ImportOptions {
    ImportOptions {
        slug: RepoSlug::from_str("ipmi-collector").unwrap(),
        imported_at: Timestamp::parse("2026-10-10T12:00:00Z").unwrap(),
        doc_paths: std::collections::BTreeMap::new(),
    }
}

/// The corpus board, parsed and composed.
fn corpus_board() -> todo_mcp::import::ipmi::IpmiBoard {
    parse_ipmi_board(&todo_md(), DECISIONS_MD).unwrap()
}

/// The 1-based line number of the first line containing `needle` in
/// `corpus` — derived from the corpus itself so the corpora's blank
/// lines cannot desynchronize a line assertion.
fn line_of(corpus: &str, needle: &str) -> usize {
    corpus.split('\n').position(|l| l.contains(needle)).unwrap() + 1
}

// =================================================================
// 1–4. The board parser: the registers and the walked-past prose.
// =================================================================

/// §2's census (four rows, ids, titles, the done verdict), §4's
/// census (ids in file order, titles, the joined prose), §5's census
/// (the D-N id, title, prose), and §1/§3/§6 walked past — §3's
/// column-zero bullets and §6's checkbox item are body, and §1's
/// foreign-header table is never a register.
#[test]
fn the_board_parser_extracts_the_registers_and_walks_the_prose_past() {
    let board = parse_todo_md(&todo_md()).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // §2: the milestone rows, in board order, titles from the
    // Milestone cell after the `M<n> — ` split.
    let milestones: Vec<(&str, &str, todo_mcp::import::ipmi::MilestoneState)> = board
        .milestones
        .iter()
        .map(|row| (row.id.as_str(), row.title.as_str(), row.status))
        .collect();
    assert_eq!(
        milestones,
        vec![
            (
                "M0",
                "scaffold, lint, and the release pipeline",
                todo_mcp::import::ipmi::MilestoneState::Done
            ),
            (
                "M1",
                "settle the loader rewrite: land, adjust, or revert",
                todo_mcp::import::ipmi::MilestoneState::Done
            ),
            (
                "M2",
                "the test foundation: table-driven suites for the loader",
                todo_mcp::import::ipmi::MilestoneState::Done
            ),
            (
                "M3",
                "repository hygiene and the CI wiring",
                todo_mcp::import::ipmi::MilestoneState::Done
            ),
        ]
    );

    // §4: the items in file order — the planted non-numeric order is
    // the register-order convention's pin — with titles (the wrapped
    // runs joined, the inner ` — ` kept, the backticks kept) and the
    // prose joined with single spaces.
    let followups: Vec<(&str, &str, &str)> = board
        .followups
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str(), item.prose.as_str()))
        .collect();
    assert_eq!(
        followups,
        vec![
            (
                "U12",
                "the fixture loader ignores the retry budget",
                "the loader config caps retries at two; the flaky vendor \
                 endpoint needs five before it answers."
            ),
            (
                "U20",
                "`--dry-run` and the port flag collide in the argument parser",
                "direction (user, 2026-10-02): the parser rejects both flags \
                 together, so split them or document the conflict."
            ),
            (
                "U15",
                "the open split — fixtures as the source of truth",
                "the fixture tree becomes the canonical input for every \
                 suite; the mechanism is D4's to choose."
            ),
            (
                "U23",
                "the sampler reads the shared map by name, not by snapshot",
                "two windows exposed by the concurrency tests; the fix \
                 passes the snapshot into the goroutine."
            ),
            (
                "U18",
                "the README documents a key the parser rejects",
                "the example config carries a key the validator refuses; \
                 align the two surfaces."
            ),
        ]
    );

    // §5: the debt register — the hyphenated D-N family, the prose
    // wrapped and joined.
    let debt: Vec<(&str, &str, &str)> = board
        .debt
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str(), item.prose.as_str()))
        .collect();
    assert_eq!(
        debt,
        vec![(
            "D-7",
            "the vendored checksums are two releases stale",
            "CI verifies against the newer tag while the container build \
             stays on the older pin; refresh them with the next planned \
             bump instead."
        )]
    );

    // §1/§3/§6 walked past: no U99 anywhere — the §6 checkbox is body,
    // and §3's bullets never parse.
    let ids: Vec<&str> = board
        .followups
        .iter()
        .chain(board.debt.iter())
        .map(|item| item.id.as_str())
        .collect();
    assert_eq!(ids, vec!["U12", "U20", "U15", "U23", "U18", "D-7"]);
}

// =================================================================
// 5–8. The strictness pins: §2, §4, §5, the sections.
// =================================================================

/// §2's strictness: an unknown State word refuses, a duplicate M id
/// refuses naming both lines, a wrong column count refuses, and the
/// table's absence is the named missing register.
#[test]
fn the_milestone_table_strictness_pins() {
    // An unknown State word: the measured vocabulary is `done` alone.
    let corpus = todo_md().replace(
        "| M3 — repository hygiene and the CI wiring | **done** 2026-10-03 (the hygiene pair with D1) |",
        "| M3 — repository hygiene and the CI wiring | **in progress** 2026-10-03 |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::BadStatus {
            line: line_of(&corpus, "**in progress**"),
            text: "**in progress** 2026-10-03".to_owned(),
            detail: "the State cell's bold word must read 'done'".to_owned(),
        })
    );

    // A State cell without its bold word.
    let corpus = todo_md().replace(
        "| M3 — repository hygiene and the CI wiring | **done** 2026-10-03 (the hygiene pair with D1) |",
        "| M3 — repository hygiene and the CI wiring | done 2026-10-03 |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::BadStatus {
            line: line_of(&corpus, "| done 2026-10-03 |"),
            text: "done 2026-10-03".to_owned(),
            detail: "the State cell's bold word is missing".to_owned(),
        })
    );

    // A duplicate M id.
    let corpus = todo_md().replace(
        "| M3 — repository hygiene and the CI wiring | **done** 2026-10-03 (the hygiene pair with D1) |",
        "| M0 — a duplicate row | **done** 2026-10-03 |",
    );
    let first = line_of(&corpus, "| M0 — scaffold");
    let second = line_of(&corpus, "| M0 — a duplicate row");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::DuplicateId {
            id: "M0".to_owned(),
            first,
            second,
        })
    );

    // A row with the wrong cell count.
    let corpus = todo_md().replace(
        "| M3 — repository hygiene and the CI wiring | **done** 2026-10-03 (the hygiene pair with D1) |",
        "| M3 — repository hygiene and the CI wiring |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::ColumnCount {
            line: line_of(&corpus, "| M3 — repository hygiene"),
            expected: 2,
            found: 1,
        })
    );

    // A Milestone cell without the ` — ` split.
    let corpus = todo_md().replace(
        "| M3 — repository hygiene and the CI wiring | **done** 2026-10-03 (the hygiene pair with D1) |",
        "| M3 repository hygiene | **done** 2026-10-03 |",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "| M3 repository hygiene |"),
            detail: "the Milestone cell has no ' — ' separator after the id".to_owned(),
        })
    );

    // The table's absence.
    let corpus = todo_md().replace("| Milestone | State |\n|---|---|\n", "");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MissingRegister {
            register: "§2's milestones table".to_owned(),
            line: corpus.split('\n').count(),
        })
    );
}

/// §4's strictness: a checked item is the open-only refusal (V2), a
/// duplicate U id refuses, and the malformed shapes refuse — no bold
/// run, no ` · ` separator, a non-U id, a missing ` —` separator, and
/// a §5-shaped bullet misplaced in §4.
#[test]
fn the_followups_register_strictness_pins() {
    // V2: a checked item is a named refusal, never a closed task.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- [x] **U15 · the open split — fixtures as the source of truth** —",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::CheckedItem {
            line: line_of(&corpus, "- [x] **U15"),
        })
    );

    // A duplicate U id.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- [ ] **U12 · a duplicate id** — body prose.",
    );
    let first = line_of(&corpus, "**U12 · the fixture loader");
    let second = line_of(&corpus, "**U12 · a duplicate id**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::DuplicateId {
            id: "U12".to_owned(),
            first,
            second,
        })
    );

    // No bold run: the marker is followed by plain prose.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- [ ] Plain prose with no bold run.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "Plain prose with no bold run"),
            detail: "a register item's bold run is missing".to_owned(),
        })
    );

    // No ` · ` separator.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- [ ] **U15 has no separator** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "**U15 has no separator**"),
            detail: "the bold run has no ' · ' separator".to_owned(),
        })
    );

    // A non-U id: the family is §4's own.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- [ ] **D-7 · a debt id in §4** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::BadId {
            line: line_of(&corpus, "**D-7 · a debt id in §4**"),
            id: "D-7".to_owned(),
            detail: "not a U id".to_owned(),
        })
    );

    // A missing ` —` separator after the bold run.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- [ ] **U15 · a title with no separator** and no em-dash.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "**U15 · a title with no separator**"),
            detail: "the item's ' —' separator after the bold run is missing".to_owned(),
        })
    );

    // A §5-shaped bullet misplaced in §4.
    let corpus = todo_md().replace(
        "- [ ] **U15 · the open split — fixtures as the source of truth** —",
        "- **D-7 · a §5-shaped bullet in §4** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "- **D-7 · a §5-shaped bullet in §4**"),
            detail: "a §4 register line is not a checkbox item".to_owned(),
        })
    );
}

/// §5's strictness: a checkbox item refuses (the register is
/// checkbox-less), a duplicate D-N id refuses, and the malformed
/// shapes refuse — a non-D-N id and a missing bold run.
#[test]
fn the_debt_register_strictness_pins() {
    // A checkbox item in the checkbox-less register.
    let corpus = todo_md().replace(
        "- **D-7 · the vendored checksums are two releases stale** — CI",
        "- [ ] **D-7 · the vendored checksums are two releases stale** — CI",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "- [ ] **D-7 · the vendored"),
            detail: "a §5 register line is a checkbox item — the register is checkbox-less"
                .to_owned(),
        })
    );

    // A duplicate D-N id.
    let corpus = format!(
        "{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}\
         ## §5 Debt\n\n\
         - **D-7 · the vendored checksums are two releases stale** — CI\n      lint.\n\n\
         - **D-7 · a duplicate debt id** — body prose.\n\n{SECTION_6}"
    );
    let first = line_of(&corpus, "**D-7 · the vendored");
    let second = line_of(&corpus, "**D-7 · a duplicate debt id**");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::DuplicateId {
            id: "D-7".to_owned(),
            first,
            second,
        })
    );

    // A non-D-N id: the hyphenated family is §5's own.
    let corpus = todo_md().replace(
        "- **D-7 · the vendored checksums are two releases stale** — CI",
        "- **D7 · an un-hyphenated id in §5** — body prose.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::BadId {
            line: line_of(&corpus, "**D7 · an un-hyphenated id in §5**"),
            id: "D7".to_owned(),
            detail: "not a D-N id".to_owned(),
        })
    );

    // A missing bold run.
    let corpus = todo_md().replace(
        "- **D-7 · the vendored checksums are two releases stale** — CI",
        "- Plain prose with no bold run.",
    );
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "- Plain prose with no bold run."),
            detail: "a register item's bold run is missing".to_owned(),
        })
    );
}

/// The section walker's own pins: an absent imported section is a
/// named error, a § carried twice is a named error, and a
/// sibling-shaped `## 4. ` header is walked past (the §-prefix is the
/// grammar), which surfaces as the missing section.
#[test]
fn the_section_walker_strictness_pins() {
    // An absent §5.
    let corpus = format!("{HEAD}{SECTION_2}{SECTION_3}{SECTION_4}{SECTION_6}");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MissingSection {
            section: "§5".to_owned(),
            line: corpus.split('\n').count(),
        })
    );

    // §4 carried twice.
    let corpus = todo_md().replace("## §5 Debt\n", "## §4 Follow-ups (unowned)\n## §5 Debt\n");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MalformedLine {
            line: line_of(&corpus, "## §5 Debt") - 1,
            detail: "TODO.md carries §4 twice".to_owned(),
        })
    );

    // A sibling-shaped header: `## 4. Follow-ups` is not `## §4` —
    // the section is missing, not silently mis-read.
    let corpus = todo_md().replace("## §4 Follow-ups (unowned)", "## 4. Follow-ups (unowned)");
    assert_eq!(
        parse_todo_md(&corpus),
        Err(IpmiError::MissingSection {
            section: "§4".to_owned(),
            line: corpus.split('\n').count(),
        })
    );
}

// =================================================================
// 9–12. The decisions parser.
// =================================================================

/// The decisions census: the summary rows (ids, titles, statuses,
/// gates, recommendations) and the dossiers (ids, titles, bodies
/// carrying the option bullets) — plus the never-fired pins (a bad
/// Status bold word, a missing bold word, an empty Recommendation, a
/// wrong column count, the table's absence).
#[test]
fn the_decisions_parser_reads_the_table_and_the_dossiers() {
    let file = parse_decisions(DECISIONS_MD).unwrap_or_else(|e| panic!("parse failed: {e}"));

    // The summary rows, in table order, every cell parsed.
    assert_eq!(file.summary.len(), 4);
    assert_eq!(file.summary[0].id, "D1");
    assert_eq!(file.summary[0].title, "Direction of the loader rewrite");
    assert_eq!(
        file.summary[0].status,
        DecisionStatusCell::Signed {
            resolution: "(b) 2026-10-03".to_owned()
        }
    );
    assert_eq!(file.summary[0].gates, "—");
    assert_eq!(
        file.summary[0].recommendation,
        "(b) land with the two fixes"
    );
    assert_eq!(
        file.summary[1].status,
        DecisionStatusCell::Deferred {
            remainder: "2026-10-04".to_owned()
        }
    );
    assert_eq!(file.summary[1].gates, "enabling the `publish` job in CI");
    assert_eq!(
        file.summary[2].status,
        DecisionStatusCell::Signed {
            resolution: "2026-10-03".to_owned()
        }
    );
    assert_eq!(
        file.summary[3].status,
        DecisionStatusCell::Deferred {
            remainder: "2026-10-03".to_owned()
        }
    );
    assert_eq!(
        file.summary[3].gates,
        "U15 rollout; standing up the fixture repo"
    );

    // The dossiers: ids, titles, bodies. D2's header tail reads
    // `open` — validated for presence, dropped, never cross-checked
    // against the table's `deferred`.
    assert_eq!(file.dossiers.len(), 4);
    assert_eq!(file.dossiers[0].id, "D1");
    assert_eq!(file.dossiers[0].title, "Direction of the loader rewrite");
    assert!(file.dossiers[0].body.contains("- **(a) Land as-is**"));
    assert!(
        file.dossiers[0]
            .body
            .contains("**Signed (b) on 2026-10-03.**")
    );
    assert_eq!(file.dossiers[1].id, "D2");
    assert_eq!(file.dossiers[1].title, "the artifact publishing channel");

    // The composition's cross-check passes on the aligned corpus.
    assert!(parse_ipmi_board(&todo_md(), DECISIONS_MD).is_ok());

    // A bad Status bold word.
    let bad = DECISIONS_MD.replace(
        "| D2 | the artifact publishing channel | **deferred** 2026-10-04 |",
        "| D2 | the artifact publishing channel | **open** 2026-10-04 |",
    );
    assert_eq!(
        parse_decisions(&bad),
        Err(IpmiError::BadStatus {
            line: line_of(&bad, "**open** 2026-10-04"),
            text: "**open** 2026-10-04".to_owned(),
            detail: "the Status cell's bold word must read 'signed' or 'deferred'".to_owned(),
        })
    );

    // A Status cell without its bold word.
    let bad = DECISIONS_MD.replace(
        "| D2 | the artifact publishing channel | **deferred** 2026-10-04 |",
        "| D2 | the artifact publishing channel | deferred 2026-10-04 |",
    );
    assert_eq!(
        parse_decisions(&bad),
        Err(IpmiError::BadStatus {
            line: line_of(&bad, "| deferred 2026-10-04 |"),
            text: "deferred 2026-10-04".to_owned(),
            detail: "the Status cell's bold word is missing".to_owned(),
        })
    );

    // An empty Recommendation cell.
    let bad = DECISIONS_MD.replace(
        "| D1 | Direction of the loader rewrite | **signed** (b) 2026-10-03 | — | (b) land with the two fixes |",
        "| D1 | Direction of the loader rewrite | **signed** (b) 2026-10-03 | — |  |",
    );
    assert_eq!(
        parse_decisions(&bad),
        Err(IpmiError::MalformedLine {
            line: line_of(&bad, "| — |  |"),
            detail: "the Recommendation cell is empty".to_owned(),
        })
    );

    // A row with the wrong cell count.
    let bad = DECISIONS_MD.replace(
        "| D1 | Direction of the loader rewrite | **signed** (b) 2026-10-03 | — | (b) land with the two fixes |",
        "| D1 | Direction of the loader rewrite | **signed** (b) 2026-10-03 | — |",
    );
    assert_eq!(
        parse_decisions(&bad),
        Err(IpmiError::ColumnCount {
            line: line_of(&bad, "**signed** (b) 2026-10-03 | — |"),
            expected: 5,
            found: 4,
        })
    );

    // The table's absence.
    let bad = DECISIONS_MD.replace(
        "| Id | Title | Status | Gates | Recommendation |\n|---|---|---|---|---|\n",
        "",
    );
    assert_eq!(
        parse_decisions(&bad),
        Err(IpmiError::MissingRegister {
            register: "the decisions summary table".to_owned(),
            line: bad.split('\n').count(),
        })
    );
}

/// V3's both halves: a signed row's resolution is the Status cell's
/// remainder after the bold word, and a signed row with an empty
/// remainder is a named refusal — the closure discipline demands a
/// resolution, refuse rather than invent.
#[test]
fn a_signed_row_without_a_resolution_is_a_named_refusal() {
    let decisions = DECISIONS_MD.replace(
        "| D3 | the CI lint runner | **signed** 2026-10-03 | — | the newer runner image |",
        "| D3 | the CI lint runner | **signed** | — | the newer runner image |",
    );
    assert_eq!(
        parse_decisions(&decisions),
        Err(IpmiError::BadStatus {
            line: line_of(&decisions, "| **signed** | — |"),
            text: "**signed**".to_owned(),
            detail: "a signed decision's Status cell carries no resolution".to_owned(),
        })
    );
}

/// The dossier cross-check fires both ways: a row without a dossier
/// refuses, a dossier without a row refuses — each naming the
/// smallest offending id.
#[test]
fn the_dossier_cross_check_fires_both_ways() {
    // A row without a dossier: D4's dossier dropped.
    let decisions = DECISIONS_MD.replace(
        "## D4 · the fixture split mechanism — deferred 2026-10-03\n\nHow the fixture repository is born from this one.\n",
        "",
    );
    assert_eq!(
        parse_ipmi_board(&todo_md(), &decisions),
        Err(IpmiError::IdSetMismatch {
            id: "D4".to_owned(),
            detail: "D4 is in the decisions summary table but not in the decisions dossiers"
                .to_owned(),
        })
    );

    // A dossier without a row.
    let decisions =
        format!("{DECISIONS_MD}---\n\n## D5 · a dossier with no row — open\n\nBody prose.\n");
    assert_eq!(
        parse_ipmi_board(&todo_md(), &decisions),
        Err(IpmiError::IdSetMismatch {
            id: "D5".to_owned(),
            detail: "D5 is in the decisions dossiers but not in the decisions summary table"
                .to_owned(),
        })
    );
}

/// Duplicate D ids refuse in both registers: the summary table and
/// the dossier headers — each naming both lines.
#[test]
fn duplicate_decision_ids_refuse_in_both_registers() {
    // In the summary table.
    let decisions = DECISIONS_MD.replace(
        "| D4 | the fixture split mechanism | **deferred** 2026-10-03 | U15 rollout; standing up the fixture repo | (b) fresh repo, squashed history — deferred |",
        "| D1 | a duplicate row | **signed** (b) 2026-10-03 | — | hold |",
    );
    let first = line_of(&decisions, "| D1 | Direction of the loader rewrite");
    let second = line_of(&decisions, "| D1 | a duplicate row");
    assert_eq!(
        parse_decisions(&decisions),
        Err(IpmiError::DuplicateId {
            id: "D1".to_owned(),
            first,
            second,
        })
    );

    // In the dossiers.
    let decisions = DECISIONS_MD.replace(
        "## D4 · the fixture split mechanism — deferred 2026-10-03",
        "## D1 · a duplicate dossier — open",
    );
    let first = line_of(&decisions, "## D1 · Direction of the loader rewrite");
    let second = line_of(&decisions, "## D1 · a duplicate dossier");
    assert_eq!(
        parse_decisions(&decisions),
        Err(IpmiError::DuplicateId {
            id: "D1".to_owned(),
            first,
            second,
        })
    );
}

// =================================================================
// 13–16. The mapping: the plan, the conventions, the reproduction.
// =================================================================

/// The mapping's conventions over the corpus: the plan's total order
/// (epics, decisions, tasks, findings, edges), the census, the epic
/// statuses (the State word → closed; the synthetics open), the
/// descriptions, the references (V4: every register node carries
/// `board:<id>`, the synthetics none), the task kinds and statuses
/// and verify literal and body, the decision statuses (V1: deferred →
/// open) and resolutions (V3) and recommendation and confidence, the
/// timestamps (imported_at everywhere), and the Gates edge (V5:
/// D4 → the task imported from board U15).
#[test]
fn the_mapping_pins_the_plan_order_census_and_conventions() {
    let board = corpus_board();
    let plan = map_ipmi(&board, &opts()).unwrap();
    let ops = &plan.ops;

    // The census: 6 epics, 4 decisions, 6 tasks, 1 edge — no
    // findings, every Gates token on the corpus resolves.
    let epic_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddEpic { .. }))
        .count();
    let decision_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddDecision { .. }))
        .count();
    let task_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::AddTask { .. }))
        .count();
    let edge_count = ops
        .iter()
        .filter(|op| matches!(op, PlanOp::LinkBlocks { .. }))
        .count();
    assert_eq!(
        (epic_count, decision_count, task_count, edge_count),
        (6, 4, 6, 1)
    );
    // The total order: every epic precedes every decision, every
    // decision precedes every task, every task precedes every edge
    // (the findings slot sits between tasks and edges; the corpus
    // carries none).
    let first_decision = ops
        .iter()
        .position(|op| matches!(op, PlanOp::AddDecision { .. }))
        .unwrap();
    let first_task = ops
        .iter()
        .position(|op| matches!(op, PlanOp::AddTask { .. }))
        .unwrap();
    let first_edge = ops
        .iter()
        .position(|op| matches!(op, PlanOp::LinkBlocks { .. }))
        .unwrap();
    assert_eq!(first_decision, epic_count);
    assert_eq!(first_task, epic_count + decision_count);
    assert_eq!(first_edge, epic_count + decision_count + task_count);

    // The epic ops: the milestones in board order, closed, their
    // descriptions carrying the M id, their references carrying the
    // board id; the synthetics open with no references.
    let epics: Vec<(String, &todo_domain::EpicDraft)> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddEpic { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    let ids: Vec<&str> = epics.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "ipmi-collector:E1",
            "ipmi-collector:E2",
            "ipmi-collector:E3",
            "ipmi-collector:E4",
            "ipmi-collector:E5",
            "ipmi-collector:E6",
        ]
    );
    assert_eq!(epics[0].1.title, "scaffold, lint, and the release pipeline");
    assert_eq!(epics[0].1.status, todo_domain::Status::Closed);
    assert_eq!(
        epics[0].1.description,
        "the ipmi-collector board's §2 milestone M0"
    );
    let targets: Vec<&str> = epics[0].1.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:M0"]);
    let PlanOp::AddEpic { at, .. } = &ops[0] else {
        unreachable!("filtered above");
    };
    assert_eq!(at.to_string(), "2026-10-10T12:00:00.000000000Z");
    for (id, draft) in epics.iter().take(4) {
        assert_eq!(draft.status, todo_domain::Status::Closed, "{id}");
    }
    assert_eq!(epics[4].1.title, "Follow-ups");
    assert_eq!(epics[4].1.status, todo_domain::Status::Open);
    assert_eq!(
        epics[4].1.description,
        "the ipmi-collector board's §4 follow-ups, not tied to a milestone"
    );
    assert_eq!(epics[4].1.references.iter().count(), 0);
    assert_eq!(epics[5].1.title, "Debt");
    assert_eq!(epics[5].1.status, todo_domain::Status::Open);
    assert_eq!(
        epics[5].1.description,
        "the ipmi-collector board's §5 debt, recorded rather than forgotten"
    );
    assert_eq!(epics[5].1.references.iter().count(), 0);

    // The decision ops: table order, the Status cell governing, the
    // recommendation carried, the confidence the fixed middle, the
    // board-id reference.
    let decisions: Vec<(String, &todo_domain::DecisionDraft)> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddDecision { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    let ids: Vec<&str> = decisions.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "ipmi-collector:D1",
            "ipmi-collector:D2",
            "ipmi-collector:D3",
            "ipmi-collector:D4",
        ]
    );
    assert_eq!(decisions[0].1.title, "Direction of the loader rewrite");
    assert_eq!(decisions[0].1.status, todo_domain::DecisionStatus::Signed);
    assert_eq!(decisions[0].1.resolution.as_deref(), Some("(b) 2026-10-03"));
    assert_eq!(
        decisions[0].1.recommendation.as_deref(),
        Some("(b) land with the two fixes")
    );
    assert_eq!(decisions[0].1.confidence, todo_domain::Confidence::Medium);
    assert_eq!(decisions[0].1.options, Vec::<String>::new());
    let targets: Vec<&str> = decisions[0]
        .1
        .references
        .iter()
        .map(|r| r.target())
        .collect();
    assert_eq!(targets, vec!["board:D1"]);
    // V1: deferred → open, no resolution.
    assert_eq!(decisions[1].1.status, todo_domain::DecisionStatus::Open);
    assert_eq!(decisions[1].1.resolution, None);
    assert_eq!(decisions[3].1.status, todo_domain::DecisionStatus::Open);
    assert_eq!(decisions[3].1.resolution, None);
    assert_eq!(decisions[2].1.status, todo_domain::DecisionStatus::Signed);
    assert_eq!(decisions[2].1.resolution.as_deref(), Some("2026-10-03"));

    // The task ops: register order (the planted non-numeric file
    // order), the kinds and epics per section, open, the verify
    // literal, the prose as the body, the board-id reference.
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    let ids: Vec<&str> = tasks.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "ipmi-collector:U1",
            "ipmi-collector:U2",
            "ipmi-collector:U3",
            "ipmi-collector:U4",
            "ipmi-collector:U5",
            "ipmi-collector:U6",
        ]
    );
    // U1: the first §4 item (board U12) — task kind under Follow-ups.
    assert_eq!(
        tasks[0].1.title,
        "the fixture loader ignores the retry budget"
    );
    assert_eq!(tasks[0].1.kind, todo_domain::TaskKind::Task);
    assert_eq!(tasks[0].1.epic.to_string(), "ipmi-collector:E5");
    assert_eq!(tasks[0].1.status, todo_domain::Status::Open);
    assert_eq!(
        tasks[0].1.verify.as_deref(),
        Some("imported: the board's open-only register")
    );
    assert_eq!(
        tasks[0].1.body,
        "the loader config caps retries at two; the flaky vendor endpoint \
         needs five before it answers."
    );
    let targets: Vec<&str> = tasks[0].1.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:U12"]);
    // U2: the second §4 item (board U20) — the register order, not
    // the numeric order.
    assert_eq!(
        tasks[1].1.title,
        "`--dry-run` and the port flag collide in the argument parser"
    );
    let targets: Vec<&str> = tasks[1].1.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:U20"]);
    // U3: the board U15 item — the Gates edge's target.
    let targets: Vec<&str> = tasks[2].1.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:U15"]);
    // U6: the §5 debt item — the debt kind under Debt.
    assert_eq!(
        tasks[5].1.title,
        "the vendored checksums are two releases stale"
    );
    assert_eq!(tasks[5].1.kind, todo_domain::TaskKind::Debt);
    assert_eq!(tasks[5].1.epic.to_string(), "ipmi-collector:E6");
    let targets: Vec<&str> = tasks[5].1.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:D-7"]);
    // Every task's stamp is the caller-passed imported_at.
    for op in ops.iter().skip(first_task).take(task_count) {
        let PlanOp::AddTask { at, .. } = op else {
            unreachable!("filtered above");
        };
        assert_eq!(at.to_string(), "2026-10-10T12:00:00.000000000Z");
    }

    // The edge ops: V5 — D4's Gates cell names U15, the one
    // id-bearing token, resolved to the task imported from board U15.
    let edges: Vec<&todo_domain::BlocksEdge> = ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(
        edges,
        vec![&todo_domain::BlocksEdge::DecisionToTask {
            from: "ipmi-collector:D4".parse().unwrap(),
            to: "ipmi-collector:U3".parse().unwrap(),
        }]
    );
}

/// The id reproduction at synthetic scale: applied to a fresh temp
/// store, the store's task ids are U1–U6 in register order (the
/// planted non-numeric file order proves the convention), the epic
/// ids E1–E6, the decision ids D1–D4 — and the landed graph carries
/// the census, the statuses, the references and the one edge.
#[test]
fn the_id_reproduction_at_synthetic_scale() {
    let board = corpus_board();
    let plan = map_ipmi(&board, &opts()).unwrap();

    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let mut store = root
        .register(Repo {
            slug: RepoSlug::from_str("ipmi-collector").unwrap(),
            name: "ipmi-collector".to_owned(),
            path: "/repos/ipmi-collector".to_owned(),
            registered_at: Timestamp::parse("2026-10-10T00:00:00Z").unwrap(),
        })
        .unwrap();
    apply_plan(&plan, &mut store).unwrap();

    // The census off the landed graph.
    assert_eq!(store.graph().epics().count(), 6);
    assert_eq!(store.graph().decisions().count(), 4);
    assert_eq!(store.graph().docs().count(), 0);
    assert_eq!(store.graph().tasks().count(), 6);
    assert_eq!(store.graph().blocks_edges().len(), 1);

    // The task ids in register order: the board's U12, U20, U15, U23,
    // U18 then D-7 land as U1–U6 — the file order, not the numeric
    // order.
    let tasks: Vec<&todo_domain::Task> = store.graph().tasks().collect();
    let ids: Vec<String> = tasks.iter().map(|task| task.id.to_string()).collect();
    let expected: Vec<String> = (1..=6).map(|n| format!("ipmi-collector:U{n}")).collect();
    assert_eq!(ids, expected);
    assert_eq!(
        tasks[0].title,
        "the fixture loader ignores the retry budget"
    );
    assert_eq!(
        tasks[1].title,
        "`--dry-run` and the port flag collide in the argument parser"
    );
    assert_eq!(
        tasks[2].title,
        "the open split — fixtures as the source of truth"
    );
    assert_eq!(
        tasks[3].title,
        "the sampler reads the shared map by name, not by snapshot"
    );
    assert_eq!(
        tasks[4].title,
        "the README documents a key the parser rejects"
    );
    assert_eq!(
        tasks[5].title,
        "the vendored checksums are two releases stale"
    );

    // The epic ids E1–E6, the milestone epics closed, the synthetics
    // open.
    let epics: Vec<&todo_domain::Epic> = store.graph().epics().collect();
    let ids: Vec<String> = epics.iter().map(|epic| epic.id.to_string()).collect();
    let expected: Vec<String> = (1..=6).map(|n| format!("ipmi-collector:E{n}")).collect();
    assert_eq!(ids, expected);
    for epic in epics.iter().take(4) {
        assert_eq!(epic.status, todo_domain::Status::Closed);
    }
    for epic in epics.iter().skip(4) {
        assert_eq!(epic.status, todo_domain::Status::Open);
    }

    // The decision ids D1–D4, the deferred rows open (V1).
    let decisions: Vec<&todo_domain::Decision> = store.graph().decisions().collect();
    let ids: Vec<String> = decisions
        .iter()
        .map(|decision| decision.id.to_string())
        .collect();
    let expected: Vec<String> = (1..=4).map(|n| format!("ipmi-collector:D{n}")).collect();
    assert_eq!(ids, expected);
    assert_eq!(decisions[0].status, todo_domain::DecisionStatus::Signed);
    assert_eq!(decisions[1].status, todo_domain::DecisionStatus::Open);
    assert_eq!(decisions[2].status, todo_domain::DecisionStatus::Signed);
    assert_eq!(decisions[3].status, todo_domain::DecisionStatus::Open);

    // The one edge: D4 → the task imported from board U15 (store U3).
    assert_eq!(
        store.graph().blocks_edges(),
        vec![todo_domain::BlocksEdge::DecisionToTask {
            from: "ipmi-collector:D4".parse().unwrap(),
            to: "ipmi-collector:U3".parse().unwrap(),
        }]
    );
}

/// A Gates token naming an id no register carries becomes a
/// `finding`-kind task under `Follow-ups` (the sibling dialects'
/// shape for a recognized-but-unresolvable assertion) — the register
/// is open-only (resolved items are deleted), so a Gates cell
/// referencing a resolved id is an expected future state, never a
/// refusal.
#[test]
fn an_unresolvable_gates_token_becomes_a_finding() {
    // A U token naming no register task: D4's cell names U99.
    let decisions = DECISIONS_MD.replace(
        "| D4 | the fixture split mechanism | **deferred** 2026-10-03 | U15 rollout; standing up the fixture repo | (b) fresh repo, squashed history — deferred |",
        "| D4 | the fixture split mechanism | **deferred** 2026-10-03 | U99 rollout | (b) fresh repo |",
    );
    let board = parse_ipmi_board(&todo_md(), &decisions).unwrap();
    let plan = map_ipmi(&board, &opts()).unwrap();

    // Seven tasks: the six register items, then the finding numbered
    // after them.
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    assert_eq!(tasks.len(), 7);
    let (finding_id, finding) = (&tasks[6].0, tasks[6].1);
    assert_eq!(finding_id, "ipmi-collector:U7");
    assert_eq!(finding.kind, todo_domain::TaskKind::Finding);
    assert_eq!(finding.title, "D4 gates U99");
    assert_eq!(finding.epic.to_string(), "ipmi-collector:E5");
    assert_eq!(finding.status, todo_domain::Status::Open);
    assert_eq!(
        finding.verify.as_deref(),
        Some("imported: the board's open-only register")
    );
    assert_eq!(finding.body, "U99 rollout");
    let targets: Vec<&str> = finding.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:D4"]);

    // D4's unresolvable cell yields no edge; no other row carries an
    // id-bearing token.
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 0);

    // An M token naming no milestone: the same finding shape, the
    // D4 edge remaining.
    let decisions = DECISIONS_MD.replace(
        "| D2 | the artifact publishing channel | **deferred** 2026-10-04 | enabling the `publish` job in CI | snapshots only for now |",
        "| D2 | the artifact publishing channel | **deferred** 2026-10-04 | M9 wiring | snapshots only for now |",
    );
    let board = parse_ipmi_board(&todo_md(), &decisions).unwrap();
    let plan = map_ipmi(&board, &opts()).unwrap();
    let tasks: Vec<(String, &todo_domain::TaskDraft)> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::AddTask { draft, id, .. } => Some((id.to_string(), draft)),
            _ => None,
        })
        .collect();
    assert_eq!(tasks.len(), 7);
    assert_eq!(tasks[6].0, "ipmi-collector:U7");
    assert_eq!(tasks[6].1.title, "D2 gates M9");
    assert_eq!(tasks[6].1.body, "M9 wiring");
    let targets: Vec<&str> = tasks[6].1.references.iter().map(|r| r.target()).collect();
    assert_eq!(targets, vec!["board:D2"]);
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 1);
}

/// The Gates scan's own pins: multiple id-bearing tokens yield
/// multiple edges in scan order, a repeated token yields one, and an
/// M token yields a DecisionToEpic edge.
#[test]
fn the_gates_scan_yields_edges_in_scan_order() {
    let decisions = DECISIONS_MD.replace(
        "| D2 | the artifact publishing channel | **deferred** 2026-10-04 | enabling the `publish` job in CI | snapshots only for now |",
        "| D2 | the artifact publishing channel | **deferred** 2026-10-04 | M1 and U15, then U15 again | snapshots only for now |",
    );
    let board = parse_ipmi_board(&todo_md(), &decisions).unwrap();
    let plan = map_ipmi(&board, &opts()).unwrap();
    let edges: Vec<&todo_domain::BlocksEdge> = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            PlanOp::LinkBlocks { edge, .. } => Some(edge),
            _ => None,
        })
        .collect();
    assert_eq!(
        edges,
        vec![
            &todo_domain::BlocksEdge::DecisionToEpic {
                from: "ipmi-collector:D2".parse().unwrap(),
                to: "ipmi-collector:E2".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToTask {
                from: "ipmi-collector:D2".parse().unwrap(),
                to: "ipmi-collector:U3".parse().unwrap(),
            },
            &todo_domain::BlocksEdge::DecisionToTask {
                from: "ipmi-collector:D4".parse().unwrap(),
                to: "ipmi-collector:U3".parse().unwrap(),
            },
        ]
    );
}

// =================================================================
// 17. Determinism.
// =================================================================

/// The same board parsed and mapped twice yields equal ASTs and
/// equal plans — the sibling dialects' pin, this dialect's own.
#[test]
fn mapping_the_same_board_twice_yields_equal_plans() {
    // The premise first: both parses and both mappings succeed, or
    // the equality below would compare two identical failures.
    let first_board = parse_ipmi_board(&todo_md(), DECISIONS_MD).unwrap();
    let second_board = parse_ipmi_board(&todo_md(), DECISIONS_MD).unwrap();
    assert_eq!(first_board, second_board);
    let first = map_ipmi(&first_board, &opts()).unwrap();
    let second = map_ipmi(&second_board, &opts()).unwrap();
    assert_eq!(first, second);
}

// =================================================================
// The wire: the dialect param's dispatch.
// =================================================================

/// The board directory, relative to the repo's registered checkout
/// path — the dialect's measured layout, mirrored from the tool's
/// constant.
const BOARD_DIR: &str = "docs/implementation";

/// Builds the corpus ipmi-collector tree under a fresh temp
/// directory: the two board files at their dialect paths. Returns the
/// tree's root — the repo's registered checkout path.
fn ipmi_tree() -> TempDir {
    let root = TempDir::new().unwrap();
    let dir = root.path().join(BOARD_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("TODO.md"), todo_md()).unwrap();
    fs::write(dir.join("decisions.md"), DECISIONS_MD).unwrap();
    root
}

/// A `repo_register` request's arguments for `slug` with its checkout
/// at `tree`.
fn register_args(slug: &str, tree: &Path) -> String {
    json!({
        "slug": slug,
        "name": slug,
        "path": tree.display().to_string(),
    })
    .to_string()
}

/// `import_md { dialect: "ipmi-collector" }` against the synthetic
/// ipmi-collector tree: the counts response and spot ids asserted
/// in-test (no golden, D12); the absent dialect parameter still
/// routes to the dacode path (the default unchanged — it faults on
/// the dacode appendices this tree does not carry); an unknown
/// dialect value is `invalid` naming all five values; and a tree
/// missing its decisions file faults as `io` naming the ipmi file's
/// role.
#[test]
fn import_md_with_dialect_ipmi_collector_drives_the_ipmi_path() {
    let root = TempDir::new().unwrap();
    let tree = ipmi_tree();
    let requests = vec![
        call(
            2,
            "repo_register",
            &register_args("ipmi-collector", tree.path()),
        ),
        call(
            3,
            "import_md",
            r#"{"repo":"ipmi-collector","dialect":"ipmi-collector"}"#,
        ),
        call(4, "graph_show", r#"{"repo":"ipmi-collector"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 3);
    // The premise: the register and the import both succeeded.
    let registered: Value = serde_json::from_str(&driven.responses[0]).unwrap();
    assert_eq!(registered["result"]["isError"], json!(false));
    let imported: Value = serde_json::from_str(&driven.responses[1]).unwrap();
    assert_eq!(
        imported["result"]["isError"],
        json!(false),
        "{}",
        driven.responses[1]
    );

    // The counts response: the corpus's census, read off the landed
    // graph.
    let answer = structured(&driven.responses[1]);
    assert_eq!(answer["repo"], json!("ipmi-collector"));
    assert_eq!(answer["epics"], json!(6));
    assert_eq!(answer["decisions"], json!(4));
    assert_eq!(answer["docs"], json!(0));
    assert_eq!(answer["tasks"], json!(6));
    assert_eq!(answer["blocks"], json!(1));

    // graph_show's spot checks: the register-order reproduction (U2
    // is the board's U20 item), the debt kind, the board-id
    // reference, and the closed milestone epic.
    let graph = structured(&driven.responses[2]);
    let repo = &graph["repos"][0];
    let tasks: Vec<&Value> = repo["epics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["tasks"].as_array().unwrap())
        .collect();
    assert_eq!(tasks.len(), 6);
    let u2 = tasks
        .iter()
        .find(|task| task["id"] == json!("ipmi-collector:U2"))
        .unwrap();
    assert_eq!(
        u2["title"],
        json!("`--dry-run` and the port flag collide in the argument parser")
    );
    assert_eq!(u2["kind"], json!("task"));
    assert_eq!(
        u2["references"][0]["target"],
        json!("board:U20"),
        "the register-order reproduction carries the board id"
    );
    let u6 = tasks
        .iter()
        .find(|task| task["id"] == json!("ipmi-collector:U6"))
        .unwrap();
    assert_eq!(u6["kind"], json!("debt"), "§5's items are debt");
    assert_eq!(u6["references"][0]["target"], json!("board:D-7"));
    let epics: Vec<&Value> = repo["epics"].as_array().unwrap().iter().collect();
    assert_eq!(epics.len(), 6);
    assert_eq!(epics[0]["status"], json!("closed"), "the done milestone");
    assert_eq!(epics[4]["title"], json!("Follow-ups"));

    // The absent dialect parameter: the default is still dacode —
    // this tree carries no dacode appendices, so the dacode path
    // faults on their read rather than importing the ipmi board.
    let root = TempDir::new().unwrap();
    let tree = ipmi_tree();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "repo_register",
                &register_args("ipmi-collector", tree.path()),
            ),
            call(3, "import_md", r#"{"repo":"ipmi-collector"}"#),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[1],
        "io",
        "io: read the dacode board file appendices/B-decisions.md: NotFound",
    );

    // An unknown dialect value: `invalid` naming all five values.
    let root = TempDir::new().unwrap();
    let tree = ipmi_tree();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "repo_register",
                &register_args("ipmi-collector", tree.path()),
            ),
            call(
                3,
                "import_md",
                r#"{"repo":"ipmi-collector","dialect":"shelly"}"#,
            ),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[1],
        "invalid",
        "the dialect 'shelly' is not known: 'dacode', 'camera', 'ikvm-html', 'dacode-todo-mcp' or 'ipmi-collector'",
    );

    // A tree missing its decisions file: the io fault names the ipmi
    // file's role, the same convention the camera path pins.
    let root = TempDir::new().unwrap();
    let tree = ipmi_tree();
    fs::remove_file(tree.path().join(BOARD_DIR).join("decisions.md")).unwrap();
    let driven = drive(
        root.path(),
        &[
            call(
                2,
                "repo_register",
                &register_args("ipmi-collector", tree.path()),
            ),
            call(
                3,
                "import_md",
                r#"{"repo":"ipmi-collector","dialect":"ipmi-collector"}"#,
            ),
        ],
    );
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_carrier(
        &driven.responses[1],
        "io",
        "io: read the ipmi decisions file decisions.md: NotFound",
    );
}
