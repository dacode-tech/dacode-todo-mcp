//! The ikvm-html importer's named errors (t47): every failure is a
//! variant carrying the 1-based line number where parsing failed,
//! plus the id or key where relevant — the audit surface, the same
//! discipline the sibling dialects' errors follow.
//!
//! `IdSetMismatch` carries no line, deliberately: it fails *between*
//! the registers of one board (the summary table, the dossiers, the
//! footer, or the mapping's banner lookup), so there is no failure
//! line, and the id is the stable key — the board's own
//! "cite the id rather than a line number" rule. The mapping's
//! variants (`BadDate`, `BadReference`) carry no line either: they
//! fire over the parsed AST, which carries none. The store-refusal
//! and id-mismatch cases are not here at all: they belong to the
//! shared `ApplyError` the plan module owns, and an ikvm-html import
//! surfaces them through that type unchanged.

use std::fmt;

/// A named ikvm-html dialect parse failure with its audit surface:
/// the 1-based line, and the id or key where relevant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IkvmError {
    /// A required TODO.md section (`§2`/`§3`/`§4`/`§5`) is absent. §1
    /// and §6 onward are walked past, not required.
    MissingSection {
        /// The section's display name, e.g. `§4`.
        section: String,
        /// The line the walk ended on.
        line: usize,
    },
    /// A required register is absent: §2's milestones table, the
    /// decisions file's summary table, the resolved footer. Required
    /// because each is a register — an absent register silently
    /// omits every entry it holds.
    MissingRegister {
        /// The register's display name, e.g. `§2's milestones table`.
        register: String,
        /// The line the walk ended on.
        line: usize,
    },
    /// A register line failed to parse: a checkbox bold run that is
    /// missing or does not close, a bold run with no ` · ` separator
    /// or a non-`U<digits>` id, a title with no sentence-ending
    /// period, an empty Recommendation cell, a dossier without its
    /// Gates line, a milestones table with the wrong row count or a
    /// duplicated header.
    MalformedLine {
        /// The 1-based line number.
        line: usize,
        /// What failed, named.
        detail: String,
    },
    /// A known-header table row's column count differs from its
    /// header's.
    ColumnCount {
        /// The 1-based line number.
        line: usize,
        /// The header's column count.
        expected: usize,
        /// The row's column count.
        found: usize,
    },
    /// An id failed its shape check: a table cell that is not the M
    /// or D id its register requires.
    BadId {
        /// The 1-based line number.
        line: usize,
        /// The offending id as written (noise stripped).
        id: String,
        /// The shape it failed.
        detail: String,
    },
    /// A Status or Confidence cell, or a resolution banner, did not
    /// read a known spelling.
    BadStatus {
        /// The 1-based line number.
        line: usize,
        /// The offending text, raw.
        text: String,
        /// The spelling it failed.
        detail: String,
    },
    /// A duplicate register id — the board's ids are unique. Covers
    /// the U register (one namespace across §4 and §5) and §2's
    /// milestone ids.
    DuplicateId {
        /// The duplicated id.
        id: String,
        /// The 1-based line of the first occurrence.
        first: usize,
        /// The 1-based line of the second occurrence.
        second: usize,
    },
    /// The board's registers disagree: one of the four composition
    /// cross-checks (summary ids vs dossier ids; the three-way
    /// resolved-set equality; banner dates vs footer dates; the
    /// Gates cells vs the Gates lines), or a resolved summary row
    /// whose dossier carries no banner. No failure line exists — the
    /// id is the stable key.
    IdSetMismatch {
        /// The offending id.
        id: String,
        /// Which registers disagree, named.
        detail: String,
    },
    /// A board date is shape-valid (the parser checks `YYYY-MM-DD`)
    /// but not a real calendar date — `2026-02-30` parses as a shape
    /// and fails as a day.
    BadDate {
        /// The date as written.
        date: String,
    },
    /// The domain rejected a reference target the mapping tried to
    /// construct — reachable only through a pathological register id;
    /// the ids the parser validated cannot be rejected.
    BadReference {
        /// The rejected target.
        target: String,
        /// Why the domain rejected it.
        detail: String,
    },
}

impl fmt::Display for IkvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IkvmError::MissingSection { section, line } => {
                write!(f, "line {line}: TODO.md is missing section {section}")
            }
            IkvmError::MissingRegister { register, line } => {
                write!(f, "line {line}: {register} is absent")
            }
            IkvmError::MalformedLine { line, detail } => {
                write!(f, "line {line}: {detail}")
            }
            IkvmError::ColumnCount {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "line {line}: a table row has {found} cells, the header has {expected}"
                )
            }
            IkvmError::BadId { line, id, detail } => {
                write!(f, "line {line}: '{id}' is not a valid id: {detail}")
            }
            IkvmError::BadStatus { line, text, detail } => {
                write!(
                    f,
                    "line {line}: the status '{text}' is not a known spelling: {detail}"
                )
            }
            IkvmError::DuplicateId { id, first, second } => {
                write!(f, "the id '{id}' appears twice: lines {first} and {second}")
            }
            IkvmError::IdSetMismatch { id, detail } => {
                write!(
                    f,
                    "the id '{id}' does not agree across the registers: {detail}"
                )
            }
            IkvmError::BadDate { date } => {
                write!(f, "the board date '{date}' is not a real calendar date")
            }
            IkvmError::BadReference { target, detail } => {
                write!(f, "the reference target '{target}' was rejected: {detail}")
            }
        }
    }
}

impl std::error::Error for IkvmError {}
