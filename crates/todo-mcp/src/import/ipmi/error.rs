//! The ipmi-collector importer's named errors (t77): every failure is
//! a variant carrying the 1-based line number where parsing failed,
//! plus the id or key where relevant — the audit surface, the same
//! discipline the sibling dialects' errors follow.
//!
//! `IdSetMismatch` carries no line, deliberately: it fails *between*
//! the registers of one board (the decisions summary table vs the
//! dossiers), so there is no failure line, and the id is the stable
//! key — the board's own "cite the id rather than a line number"
//! rule. The mapping's `BadReference` variant carries no line either:
//! it fires over the parsed AST, which carries none. `BadDate` is the
//! shared `HelperError` seam's totality arm: this mapping reads no
//! board dates (every stamp is the caller-passed `imported_at`), so
//! the arm is unreachable and exists so the seam converts
//! variant-for-variant. The store-refusal and id-mismatch cases are
//! not here at all: they belong to the shared `ApplyError` the plan
//! module owns, and an ipmi-collector import surfaces them through
//! that type unchanged.

use std::fmt;

/// A named ipmi-collector dialect parse failure with its audit
/// surface: the 1-based line, and the id or key where relevant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpmiError {
    /// A required TODO.md section (`§2`/`§4`/`§5`) is absent. §1, §3
    /// and §6 are walked past, not required.
    MissingSection {
        /// The section's display name, e.g. `§4`.
        section: String,
        /// The line the walk ended on.
        line: usize,
    },
    /// A required register is absent: §2's milestones table, the
    /// decisions file's summary table. Required because each is a
    /// register — an absent register silently omits every entry it
    /// holds.
    MissingRegister {
        /// The register's display name, e.g. `§2's milestones table`.
        register: String,
        /// The line the walk ended on.
        line: usize,
    },
    /// A register line failed to parse: a checkbox or bullet without
    /// its bold run, a bold run with no ` · ` separator or a
    /// non-matching id, a missing ` —` separator after the bold run,
    /// an unclosed bold run, a checkbox item in the checkbox-less §5,
    /// a §2 row without its ` — ` split, an empty Recommendation cell.
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
    /// An id failed its shape check: a table cell or bold run that is
    /// not the M, U, `D-` or D id its register requires.
    BadId {
        /// The 1-based line number.
        line: usize,
        /// The offending id as written (noise stripped).
        id: String,
        /// The shape it failed.
        detail: String,
    },
    /// A State or Status cell did not read a known spelling: the
    /// milestone State word, the decision Status cell's bold word, or
    /// a signed row whose remainder (the resolution) is empty.
    BadStatus {
        /// The 1-based line number.
        line: usize,
        /// The offending text, raw.
        text: String,
        /// The spelling it failed.
        detail: String,
    },
    /// A §4 item is checked: the register is open-only (resolved
    /// items are deleted, the board's own rule), so a checked box
    /// means the convention drifted — surfaced, never guessed into a
    /// closure.
    CheckedItem {
        /// The 1-based line number.
        line: usize,
    },
    /// A duplicate register id — the board's ids are unique. Covers
    /// the U ids (§4), the M ids (§2), the `D-` ids (§5) and the D ids
    /// in the decisions summary table and the dossier headers.
    DuplicateId {
        /// The duplicated id.
        id: String,
        /// The 1-based line of the first occurrence.
        first: usize,
        /// The 1-based line of the second occurrence.
        second: usize,
    },
    /// The board's registers disagree: the decisions summary table's
    /// ids vs the dossiers' ids. No failure line exists — the id is
    /// the stable key.
    IdSetMismatch {
        /// The offending id.
        id: String,
        /// Which registers disagree, named.
        detail: String,
    },
    /// A board date is shape-valid but not a real calendar date.
    /// Unreachable on this dialect: the mapping reads no board dates
    /// (every stamp is the caller-passed `imported_at`), and the
    /// variant exists so the shared `HelperError` seam converts
    /// variant-for-variant.
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

impl fmt::Display for IpmiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpmiError::MissingSection { section, line } => {
                write!(f, "line {line}: TODO.md is missing section {section}")
            }
            IpmiError::MissingRegister { register, line } => {
                write!(f, "line {line}: {register} is absent")
            }
            IpmiError::MalformedLine { line, detail } => {
                write!(f, "line {line}: {detail}")
            }
            IpmiError::ColumnCount {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "line {line}: a table row has {found} cells, the header has {expected}"
                )
            }
            IpmiError::BadId { line, id, detail } => {
                write!(f, "line {line}: '{id}' is not a valid id: {detail}")
            }
            IpmiError::BadStatus { line, text, detail } => {
                write!(
                    f,
                    "line {line}: the status '{text}' is not a known spelling: {detail}"
                )
            }
            IpmiError::CheckedItem { line } => {
                write!(
                    f,
                    "line {line}: a §4 item is checked — the register is open-only \
                     (resolved items are deleted)"
                )
            }
            IpmiError::DuplicateId { id, first, second } => {
                write!(f, "the id '{id}' appears twice: lines {first} and {second}")
            }
            IpmiError::IdSetMismatch { id, detail } => {
                write!(
                    f,
                    "the id '{id}' does not agree across the registers: {detail}"
                )
            }
            IpmiError::BadDate { date } => {
                write!(f, "the board date '{date}' is not a real calendar date")
            }
            IpmiError::BadReference { target, detail } => {
                write!(f, "the reference target '{target}' was rejected: {detail}")
            }
        }
    }
}

impl std::error::Error for IpmiError {}
