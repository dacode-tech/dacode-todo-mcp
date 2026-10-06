//! The dacode importer's named errors (t42, extended by the mapping):
//! every failure is a variant carrying the 1-based line number where
//! parsing failed, plus the id or key where relevant — the audit
//! surface, the store's `corrupt_store` offset discipline.
//!
//! One variant carries no line, deliberately: `IdSetMismatch` fails
//! *between* files, so there is no failure line, and the id is the
//! stable key — the board's own "cite the id rather than a line
//! number" rule. The absence errors (`MissingSection`,
//! `MissingB1a`, `MissingRegister`) carry the line the walk ended on,
//! which is where the absence was established. The mapping's variants
//! (`MissingDocPath`, `BadDate`, `UnresolvedParent`, `BadReference`,
//! `StoreNotEmpty`, `IdMismatch`) carry no line either: they fire
//! over the parsed AST or the store, which carry none — the id or key
//! is the stable key, the same rule. The two apply-time variants
//! travel as `ApplyError::Refused` out of `apply_plan`'s refusal
//! sites — the wire maps a refusal to §6.1's `invalid`, a
//! client-fixable precondition, not corruption — and not through a
//! `From` impl into `StoreError` here: a total impl would make every
//! parse variant convertible into the store's `corrupt_store`, which
//! is the repair-associated code, and no parse failure is a store
//! fault.

use std::fmt;

/// A named parse failure with its audit surface: the 1-based line,
/// and the id or key where relevant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// A required TODO.md section (`§2`/`§4`/`§5`/`§6`) is absent.
    /// §1, §3 and any section past §6 are walked past, not required.
    MissingSection {
        /// The section's display name, e.g. `§4`.
        section: String,
        /// The line the walk ended on.
        line: usize,
    },
    /// §2's B1a paragraph (a line starting `B1a (`) is absent.
    MissingB1a {
        /// The line the walk ended on.
        line: usize,
    },
    /// A section's required register is absent: §2's decisions table,
    /// §4's "Already expanded" list, §4's milestone table, the B
    /// summary table, the A index table. Required because each is the
    /// section's register — an absent register silently omits every
    /// entry it holds, the failure mode the B1a rule exists to
    /// prevent.
    MissingRegister {
        /// The register's display name, e.g. `§2's decisions table`.
        register: String,
        /// The line the walk ended on.
        line: usize,
    },
    /// A register line failed to parse: a checkbox bold run that is
    /// not U- or M-shaped, a dossier header without a status marker,
    /// a `###` line that is not a dossier header, a duplicate section
    /// header.
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
    /// An id failed its shape check.
    BadId {
        /// The 1-based line number.
        line: usize,
        /// The offending id as written (noise stripped).
        id: String,
        /// The shape it failed.
        detail: String,
    },
    /// A status cell or dossier marker did not read a known spelling.
    BadStatus {
        /// The 1-based line number.
        line: usize,
        /// The offending text, raw.
        text: String,
        /// The spelling it failed.
        detail: String,
    },
    /// The doc register rejected an entry shape.
    BadDocEntry {
        /// The 1-based line number.
        line: usize,
        /// The offending entry, raw.
        entry: String,
    },
    /// A duplicate doc key within one source (the expanded list or
    /// the milestone table).
    DuplicateDocKey {
        /// The duplicated key.
        key: String,
        /// The 1-based line of the first occurrence.
        first: usize,
        /// The 1-based line of the second occurrence.
        second: usize,
    },
    /// A doc key is both `Expanded` (the list) and unstruck `Summary`
    /// (the milestone table) — the board's §4 would be
    /// self-contradictory.
    DocContradiction {
        /// The contradictory key.
        key: String,
        /// The 1-based line of the summary occurrence.
        line: usize,
    },
    /// A duplicate U id — the board's ids are unique.
    DuplicateId {
        /// The duplicated id.
        id: String,
        /// The 1-based line of the first occurrence.
        first: usize,
        /// The 1-based line of the second occurrence.
        second: usize,
    },
    /// A cross-file id mismatch: a B-side set inequality (TODO.md §2
    /// ↔ B summary ↔ B dossiers), or an A-side table id with no
    /// dossier. No failure line exists — the id is the stable key.
    IdSetMismatch {
        /// The offending id.
        id: String,
        /// Which registers disagree, named.
        detail: String,
    },
    /// A doc-register key has no path in the import options — a doc
    /// with no path is an omission, not a skip, so the mapping refuses
    /// the whole board rather than dropping the doc.
    MissingDocPath {
        /// The register key that has no path.
        key: String,
    },
    /// A board date is shape-valid (the parser checks `YYYY-MM-DD`)
    /// but not a real calendar date — `2026-02-30` parses as a shape
    /// and fails as a day.
    BadDate {
        /// The date as written.
        date: String,
    },
    /// A sub-step's id names a milestone §5 does not carry, so the
    /// gate task has no parent epic to live under — the mapping
    /// refuses the board rather than inventing a parent.
    UnresolvedParent {
        /// The sub-step id.
        id: String,
        /// The milestone it names.
        milestone: String,
    },
    /// The domain rejected a reference target the mapping tried to
    /// construct — reachable only through a pathological dossier span
    /// (an anchor in a section string, which the domain rejects for
    /// non-`doc` kinds); the ids and doc keys the parser validated
    /// cannot be rejected.
    BadReference {
        /// The rejected target.
        target: String,
        /// Why the domain rejected it.
        detail: String,
    },
    /// `apply_plan` refused the store: its graph holds nodes or its
    /// allocation counters have burned numbers — importing into a
    /// used store would corrupt the id reproduction. Nothing was
    /// written.
    StoreNotEmpty {
        /// The slug of the store that was refused.
        slug: String,
    },
    /// The store allocated an id that disagrees with the one the plan
    /// constructed for the same op — the reproduction broke mid-walk.
    /// Catches a slug mismatch (the store allocates under its own
    /// slug), an offset counter, and a plan whose ops no longer match
    /// their positions.
    IdMismatch {
        /// The id the plan constructed.
        expected: String,
        /// The id the store allocated.
        found: String,
    },
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::MissingSection { section, line } => {
                write!(f, "line {line}: TODO.md is missing section {section}")
            }
            ImportError::MissingB1a { line } => {
                write!(
                    f,
                    "line {line}: §2 has no B1a paragraph (no line starts 'B1a (')"
                )
            }
            ImportError::MissingRegister { register, line } => {
                write!(f, "line {line}: {register} is absent")
            }
            ImportError::MalformedLine { line, detail } => {
                write!(f, "line {line}: {detail}")
            }
            ImportError::ColumnCount {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "line {line}: a table row has {found} cells, the header has {expected}"
                )
            }
            ImportError::BadId { line, id, detail } => {
                write!(f, "line {line}: '{id}' is not a valid id: {detail}")
            }
            ImportError::BadStatus { line, text, detail } => {
                write!(
                    f,
                    "line {line}: the status '{text}' is not a known spelling: {detail}"
                )
            }
            ImportError::BadDocEntry { line, entry } => {
                write!(
                    f,
                    "line {line}: the doc register entry '{entry}' is not a known shape"
                )
            }
            ImportError::DuplicateDocKey { key, first, second } => {
                write!(
                    f,
                    "the doc key '{key}' appears twice: lines {first} and {second}"
                )
            }
            ImportError::DocContradiction { key, line } => {
                write!(
                    f,
                    "line {line}: the doc '{key}' is both expanded and still summary"
                )
            }
            ImportError::DuplicateId { id, first, second } => {
                write!(f, "the id '{id}' appears twice: lines {first} and {second}")
            }
            ImportError::IdSetMismatch { id, detail } => {
                write!(
                    f,
                    "the id '{id}' does not agree across the registers: {detail}"
                )
            }
            ImportError::MissingDocPath { key } => {
                write!(
                    f,
                    "the doc register key '{key}' has no path in the import options"
                )
            }
            ImportError::BadDate { date } => {
                write!(f, "the board date '{date}' is not a real calendar date")
            }
            ImportError::UnresolvedParent { id, milestone } => {
                write!(
                    f,
                    "the sub-step '{id}' names the milestone '{milestone}', which §5 does not carry"
                )
            }
            ImportError::BadReference { target, detail } => {
                write!(f, "the reference target '{target}' was rejected: {detail}")
            }
            ImportError::StoreNotEmpty { slug } => {
                write!(
                    f,
                    "the import into '{slug}' was refused: the store is not empty or has burned ids"
                )
            }
            ImportError::IdMismatch { expected, found } => {
                write!(
                    f,
                    "the import's id reproduction broke: the store allocated {found}, the plan constructed {expected}"
                )
            }
        }
    }
}

impl std::error::Error for ImportError {}
