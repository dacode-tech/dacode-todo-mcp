//! The dacode dialect's AST (t42): the three files' registers as
//! structured data, in board order.
//!
//! Board order is the discipline — `Vec`, never a map: the order IS
//! data, because id reproduction depends on importing in board
//! order. No line numbers on entries: board ids are the stable keys
//! (the board's own rule — cite the id rather than a line number);
//! errors carry the lines instead. No serde: the AST is never
//! persisted, and the mapping consumer (t43) owns the wire shapes —
//! the `Counters`/`NextTask` precedent. `PartialEq` is derived so the
//! twice-parse determinism test is a plain `assert_eq!`.
//!
//! Judgment calls bound here (the task spec's (a) and (d)):
//!
//! - (a) `UItem::body` and `MilestoneItem::body` carry the item's full
//!   raw span after the checkbox marker — the bold run included —
//!   because markers are extracted from that same text and never
//!   removed from it, and a marker may sit inside the bold run; the
//!   title is additionally extracted from the bold run. `BDossier`
//!   and `ADossier` carry their bodies raw for the same reason (t43's
//!   evidence fields read them).
//! - (a) `Marker` emission is one marker per kind per line — the line
//!   is the marker's stated unit, so a line carrying an emoji twice
//!   yields one marker, and the first phrase occurrence wins.
//! - (a) `ADossier::outcome` carries the header text after the status
//!   date by symmetry with `BDossier::outcome` (whose remainder the
//!   dialect names "the raw outcome"); raw text is carried, never
//!   silently dropped.
//! - (a) `BDossier::seq` carries the header's sequence number —
//!   validation plus provenance, never a key.
//! - (d) Field naming follows the dialect's own words: `title` for the
//!   pre-marker span of both dossier kinds (A's carries its sections
//!   too, raw), `outcome` for the post-marker remainder, `span`-free
//!   names everywhere else.
//! - (d) `Subsection::title` is `Option` so a §6 item before the first
//!   `###` still parses (the register-line rule: it must parse
//!   wherever it appears).
//!
//! Fix-round widenings (the ruling's dispositions, plus the
//! grammar-widening class it established): `SummaryRow::extra`
//! carries §2's fifth trailing cell raw; `SummaryStatus::Prose`
//! carries a Status cell with no known spelling; `AAppliedRow` is the
//! A file's second table; `ADossierStatus::via` carries the
//! via-form's credit. The wrapped-row and code-span-pipe shapes live
//! in `table.rs`'s module doc.

use std::fmt;

/// The parsed TODO.md: the four imported sections' registers, in
/// board order. §1 and §3 are walked past (the blocker table and
/// the delta queue duplicate per-item state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoMd {
    /// §2's decisions-table rows, in board order.
    pub decisions: Vec<SummaryRow>,
    /// §2's B1a paragraph, raw (the line starting `B1a (` plus its
    /// continuation lines) — required, because the mapping conventions
    /// map it to a closed
    /// task and missing it would silently omit one.
    pub b1a: String,
    /// §4's doc register: the expanded list's entries (list order)
    /// then the milestone table's summary entries (row order).
    pub docs: Vec<DocEntry>,
    /// §5's milestone checkbox items, in board order.
    pub milestones: Vec<MilestoneItem>,
    /// §5's sub-step table rows, flat and in board order — the id
    /// prefix encodes the parent milestone, so they are not attached
    /// to the milestone items.
    pub substeps: Vec<SubStep>,
    /// §6's subsections, in board order.
    pub followups: Vec<Subsection>,
}

/// One §2 decisions-table row: the id, the Decision cell raw, the
/// Status cell parsed, and — when the row carries a fifth trailing
/// cell (the B appendix's Gates column inlined)
/// — that cell raw. The fifth cell is never parsed as Gates: the B
/// appendix's Gates column stays the one structured source. The
/// Recommendation cell is not carried (it duplicates the dossier).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryRow {
    /// The B id with strikes and bold stripped, e.g. `B1`.
    pub id: String,
    /// The Decision cell, raw.
    pub title: String,
    /// The Status cell's verdict.
    pub status: SummaryStatus,
    /// The fifth trailing cell, raw, when the row carries one.
    pub extra: Option<String>,
}

/// §2's Status cell verdict: the `→`-separated segments scan
/// right-to-left for the last one reading a known spelling; a cell
/// with none carries its final segment as prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryStatus {
    /// The `✅ resolved` spelling.
    Resolved,
    /// The `⚠️ resolved, half-applied` spelling.
    HalfApplied,
    /// No segment reads a known spelling: the raw final segment.
    Prose(String),
}

/// One §4 doc-register entry. The key is the doc's board identifier:
/// `README`, `00-overview`, a two-digit number, or one letter of
/// `A`–`G`; the milestone-table entry's name is validation only (the
/// key is the number, so the cross-source contradiction check
/// compares like with like).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEntry {
    /// The doc key.
    pub key: String,
    /// The depth marker.
    pub depth: DocDepth,
    /// The owning milestone row (`M<n>`), carried by summary entries
    /// only.
    pub milestone: Option<String>,
}

/// The doc register's depth marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocDepth {
    /// From the "Already expanded" list.
    Expanded,
    /// An unstruck entry of the milestone table's middle cell.
    Summary,
}

/// One §5 milestone checkbox item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneItem {
    /// The M id, e.g. `M0`.
    pub id: String,
    /// The title from the bold run (between `M<n> — ` and the closing
    /// `**`) — markers inside the run are carried raw in it.
    pub title: String,
    /// The checkbox state.
    pub checked: bool,
    /// The item's full raw span after the checkbox marker — the bold
    /// run, the rest of the item line, and the continuation lines —
    /// joined by `\n`, verbatim.
    pub body: String,
    /// The markers extracted from the full span.
    pub markers: Vec<Marker>,
}

/// One §5 sub-step table row. The Gate cell is carried raw (the mapping
/// it to `verify`, a string field).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubStep {
    /// The sub-step id, `M<digits>.<digits>`.
    pub id: String,
    /// The Sub-step cell, raw.
    pub text: String,
    /// The Gate cell, raw.
    pub gate: String,
}

/// One §6 subsection with its U items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subsection {
    /// The subsection title after `### `, raw; `None` for §6's
    /// implicit leading group when an item precedes the first
    /// subsection header.
    pub title: Option<String>,
    /// The U items, in board order.
    pub items: Vec<UItem>,
}

/// One §6 U checkbox item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UItem {
    /// The U id, e.g. `U53`.
    pub id: String,
    /// The title from the bold run (between `U<n> · ` and the closing
    /// `**`).
    pub title: String,
    /// The checkbox state.
    pub checked: bool,
    /// The item's full raw span after the checkbox marker, joined by
    /// `\n`, verbatim — markers are extracted from this same text and
    /// never removed from it.
    pub body: String,
    /// The markers extracted from the full span.
    pub markers: Vec<Marker>,
}

/// A marker extracted from a checkbox item's full span (U and M items
/// only — B and A dossier bodies carry raw text, their statuses come
/// from headers and tables). Each marker carries the raw line it
/// occurred on.
///
/// Emission order is fixed: per line, the emoji kinds in declaration
/// order, then `BlockedOn`, then `MilestoneAssertion` — one marker per
/// kind per line, the first occurrence winning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    /// The ✅ discharge marker.
    Check {
        /// The raw line the marker occurred on.
        line: String,
    },
    /// The ▶️ started marker.
    InWork {
        /// The raw line the marker occurred on.
        line: String,
    },
    /// The ⚠️ caveat marker.
    Warning {
        /// The raw line the marker occurred on.
        line: String,
    },
    /// The 🧠 brainstorm marker.
    Brainstorm {
        /// The raw line the marker occurred on.
        line: String,
    },
    /// The `blocked on <id>` prose assertion, recognized only when an
    /// id-shaped token follows the phrase.
    BlockedOn {
        /// The id the phrase names.
        id: String,
        /// The raw line the phrase occurred on.
        line: String,
    },
    /// The `gates M<n>` / `Blocking M<n>` / `due by M<n>` assertion —
    /// one kind, the raw line preserving the spelling. `before M<n>`
    /// is deliberately not recognized — vague temporal prose stays
    /// body text).
    MilestoneAssertion {
        /// The milestone id the phrase names (`M<n>`).
        milestone: String,
        /// The raw line the phrase occurred on.
        line: String,
    },
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Marker::Check { .. } => write!(f, "Check"),
            Marker::InWork { .. } => write!(f, "InWork"),
            Marker::Warning { .. } => write!(f, "Warning"),
            Marker::Brainstorm { .. } => write!(f, "Brainstorm"),
            Marker::BlockedOn { id, .. } => write!(f, "BlockedOn({id})"),
            Marker::MilestoneAssertion { milestone, .. } => {
                write!(f, "MilestoneAssertion({milestone})")
            }
        }
    }
}

/// The parsed B appendix: the summary table and the dossiers, in
/// board order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionsAppendix {
    /// The summary table's rows, in board order (the live board's rows
    /// are not in id order — the order is the data).
    pub summary: Vec<BSummaryRow>,
    /// The dossiers, in board order.
    pub dossiers: Vec<BDossier>,
}

/// One B summary-table row. The Recommendation and Confidence cells
/// are not carried (they duplicate the dossier).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BSummaryRow {
    /// The B id with strikes and bold stripped.
    pub id: String,
    /// The Decision cell, raw.
    pub title: String,
    /// The parsed Gates cell.
    pub gates: GatesCell,
}

/// The parsed Gates cell: doc numbers plus an optional milestone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatesCell {
    /// The doc numbers (two-digit strings), in cell order.
    pub docs: Vec<String>,
    /// The milestone (`M<n>`) when the cell carries `→ M<n>`.
    pub milestone: Option<String>,
}

/// One B dossier: `### <seq>. [B<id>] <title> — ✅ **<status>…**`,
/// with its body running to the next `###` or end of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BDossier {
    /// The header's sequence number.
    pub seq: u32,
    /// The B id.
    pub id: String,
    /// The raw title: everything before the status marker.
    pub title: String,
    /// The parsed status marker.
    pub status: BDossierStatus,
    /// The raw outcome: the header text after the status marker, one
    /// leading `:`/`—` separator stripped when present, the closing
    /// `**` stripped.
    pub outcome: String,
    /// The raw body: the lines to the next `###` or end of input,
    /// joined by `\n`.
    pub body: String,
}

/// The parsed status marker of a B dossier header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BDossierStatus {
    /// The status kind.
    pub kind: BDossierKind,
    /// The day-precision date as written (`YYYY-MM-DD`).
    pub date: String,
    /// The optional `, APPLIED <date> via A<n>` clause.
    pub applied: Option<AppliedVia>,
}

/// The B dossier's status kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BDossierKind {
    /// `RESOLVED <date>`.
    Resolved,
    /// `SIGNED OFF <date>`.
    SignedOff,
}

/// The `, APPLIED <date> via A<n>` clause of a B dossier marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedVia {
    /// The application date as written.
    pub date: String,
    /// The A id credited.
    pub via: String,
}

/// The parsed A appendix: the index table, the applied table and the
/// dossiers, in board order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltasAppendix {
    /// The index table's rows, in board order.
    pub index: Vec<AIndexRow>,
    /// The "Deltas applied" table's rows, in board order — carried
    /// when the table is present; its rows also ride the enclosing
    /// dossier's raw body.
    pub applied: Vec<AAppliedRow>,
    /// The dossiers, in board order.
    pub dossiers: Vec<ADossier>,
}

/// One row of the A file's applied table (`| # | Section | Applied |
/// Date | Note |`): the id, the sections split as the index's, and
/// the Applied, Date and Note cells raw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AAppliedRow {
    /// The A id with strikes and bold stripped.
    pub id: String,
    /// The Section cell split by `,`, each trimmed.
    pub sections: Vec<String>,
    /// The Applied cell, raw.
    pub applied: String,
    /// The Date cell, raw.
    pub date: String,
    /// The Note cell, raw.
    pub note: String,
}

/// One A index-table row. The Recommend cell is not carried (it
/// duplicates the dossier status plus prose).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIndexRow {
    /// The A id with strikes and bold stripped.
    pub id: String,
    /// The Section cell split by `,`, each trimmed.
    pub sections: Vec<String>,
    /// The Delta cell, raw.
    pub delta: String,
    /// The id-shaped tokens found in the Depends-on cell, in scan
    /// order (duplicates kept) — strikes are noise, the dependency
    /// exists regardless.
    pub depends_on: Vec<String>,
}

/// One A dossier: `### A<id> — <span> — ✅ **<status>…**`, with its
/// body running to the next `###` or end of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ADossier {
    /// The A id.
    pub id: String,
    /// The pre-marker span, raw (the sections and title).
    pub title: String,
    /// The parsed status marker.
    pub status: ADossierStatus,
    /// The raw outcome: the header text after the status date, by
    /// symmetry with [`BDossier::outcome`].
    pub outcome: String,
    /// The raw body: the lines to the next `###` or end of input,
    /// joined by `\n`.
    pub body: String,
}

/// The parsed status marker of an A dossier header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ADossierStatus {
    /// The status kind.
    pub kind: ADossierKind,
    /// The day-precision date as written (`YYYY-MM-DD`).
    pub date: String,
    /// The via-form's credited A id (`APPLIED via A<n>, <date>` —
    /// A11's shape).
    pub via: Option<String>,
}

/// The A dossier's status kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ADossierKind {
    /// `APPLIED <date>`.
    Applied,
    /// `SIGNED OFF <date>`.
    SignedOff,
    /// `HALF-APPLIED <date>`.
    HalfApplied,
}

/// The composed three-file board: the three parses plus the cross-file
/// id checks `parse_board` enforces — set equality on the B side
/// (TODO.md §2 ↔ B summary ↔ B dossiers), and on the A side that
/// every index or applied-table id has a dossier while dossiers may
/// exceed the tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DacodeBoard {
    /// The parsed TODO.md.
    pub todo: TodoMd,
    /// The parsed B appendix.
    pub decisions: DecisionsAppendix,
    /// The parsed A appendix.
    pub deltas: DeltasAppendix,
}
