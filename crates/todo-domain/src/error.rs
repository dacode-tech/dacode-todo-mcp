//! The crate's error types (spec §6.1).
//!
//! Two types with different jobs:
//!
//! - [`DomainError`] is the error of domain *operations*. Its variants
//!   carry exactly the eight §6.1 codes the domain can produce — never
//!   the store-level `corrupt_store`/`version_mismatch`/`locked`, which
//!   live in `todo-store`. Codes are stable machine-readable identities
//!   a client may branch on; messages are context for a human reader and
//!   never the contract.
//! - [`ValueError`] is the error of value *construction*: a malformed
//!   slug, id, timestamp, date or reference handed in by a caller. It
//!   deliberately carries no §6.1 code — the code list is versioned like
//!   the store format and closed to extension by this crate. The server
//!   layer presents a `ValueError` as invalid input; the store maps one
//!   raised while replaying a log to `corrupt_store`.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The stable machine-readable error codes of spec §6.1 that the domain
/// can produce. The wire spelling of each variant is its `as_str()`;
/// serde uses the same spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// No node exists under the given id.
    UnknownId,
    /// The write would create a second node under an id already taken.
    Duplicate,
    /// The write would create a `blocks` cycle.
    Cycle,
    /// A task was created without its `verify` method.
    MissingVerify,
    /// A task was closed without `evidence`.
    MissingEvidence,
    /// A task was blocked or deferred without a `note`.
    MissingNote,
    /// A decision was signed or superseded without a `resolution`.
    MissingResolution,
    /// The write would orphan a node (spec §5.1: no orphans, ever).
    OrphanNotAllowed,
}

impl ErrorCode {
    /// The stable §6.1 code string, e.g. `"missing_evidence"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownId => "unknown_id",
            Self::Duplicate => "duplicate",
            Self::Cycle => "cycle",
            Self::MissingVerify => "missing_verify",
            Self::MissingEvidence => "missing_evidence",
            Self::MissingNote => "missing_note",
            Self::MissingResolution => "missing_resolution",
            Self::OrphanNotAllowed => "orphan_not_allowed",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An error from a domain operation, carrying one of the eight §6.1
/// codes the domain produces plus deterministic payload context.
///
/// Payloads are formatted ids, never display attributes, so an error is
/// as byte-stable as the state it describes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainError {
    /// No node exists under `id`.
    UnknownId {
        /// The formatted id that named nothing.
        id: String,
    },
    /// A node already exists under `id`.
    Duplicate {
        /// The formatted id already taken.
        id: String,
    },
    /// The write would create a `blocks` cycle; `path` lists the
    /// formatted ids along it, starting and ending at the same node.
    Cycle {
        /// The cycle path, in edge order.
        path: Vec<String>,
    },
    /// The task `id` has no `verify` method.
    MissingVerify {
        /// The formatted task id.
        id: String,
    },
    /// The task `id` is `closed` without `evidence`.
    MissingEvidence {
        /// The formatted task id.
        id: String,
    },
    /// The task `id` is `blocked` or `deferred` without a `note`.
    MissingNote {
        /// The formatted task id.
        id: String,
    },
    /// The decision `id` is `signed` or `superseded` without a
    /// `resolution`.
    MissingResolution {
        /// The formatted decision id.
        id: String,
    },
    /// The write would leave the node `id` without its required parent.
    OrphanNotAllowed {
        /// The formatted id of the node that would be orphaned.
        id: String,
    },
}

impl DomainError {
    /// The stable §6.1 code of this error — total, one code per variant.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::UnknownId { .. } => ErrorCode::UnknownId,
            Self::Duplicate { .. } => ErrorCode::Duplicate,
            Self::Cycle { .. } => ErrorCode::Cycle,
            Self::MissingVerify { .. } => ErrorCode::MissingVerify,
            Self::MissingEvidence { .. } => ErrorCode::MissingEvidence,
            Self::MissingNote { .. } => ErrorCode::MissingNote,
            Self::MissingResolution { .. } => ErrorCode::MissingResolution,
            Self::OrphanNotAllowed { .. } => ErrorCode::OrphanNotAllowed,
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownId { id } => write!(f, "unknown id: {id}"),
            Self::Duplicate { id } => write!(f, "duplicate id: {id}"),
            Self::Cycle { path } => write!(f, "cycle: {}", path.join(" -> ")),
            Self::MissingVerify { id } => write!(f, "task {id} has no verify method"),
            Self::MissingEvidence { id } => write!(f, "task {id} closed without evidence"),
            Self::MissingNote { id } => write!(f, "task {id} blocked or deferred without a note"),
            Self::MissingResolution { id } => {
                write!(f, "decision {id} signed or superseded without a resolution")
            }
            Self::OrphanNotAllowed { id } => write!(f, "node {id} would be orphaned"),
        }
    }
}

impl std::error::Error for DomainError {}

/// An error raised while constructing or parsing a value: the input was
/// malformed, so no valid value exists to return.
///
/// Carries no §6.1 code — see the module docs for the contract. The
/// `input` payload is the offending string as received, so a human can
/// see what failed; branching stays on the variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueError {
    /// The string is not a valid repository slug.
    InvalidSlug {
        /// The offending input.
        input: String,
    },
    /// The string is not a valid node id of the expected kind.
    InvalidId {
        /// The offending input.
        input: String,
    },
    /// The string is not a valid RFC 3339 UTC timestamp in canonical form.
    InvalidTimestamp {
        /// The offending input.
        input: String,
    },
    /// The string is not a valid `YYYY-MM-DD` date.
    InvalidDate {
        /// The offending input.
        input: String,
    },
    /// The reference violates its construction rules (empty target, an
    /// anchor on a non-`doc` kind, a malformed `doc` anchor, an empty
    /// note).
    InvalidReference {
        /// What the input got wrong.
        reason: String,
        /// The offending target.
        target: String,
    },
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSlug { input } => write!(f, "invalid repository slug: {input:?}"),
            Self::InvalidId { input } => write!(f, "invalid node id: {input:?}"),
            Self::InvalidTimestamp { input } => {
                write!(f, "invalid RFC 3339 UTC timestamp: {input:?}")
            }
            Self::InvalidDate { input } => write!(f, "invalid YYYY-MM-DD date: {input:?}"),
            Self::InvalidReference { reason, target } => {
                write!(f, "invalid reference target {target:?}: {reason}")
            }
        }
    }
}

impl std::error::Error for ValueError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_the_stable_snake_case_spellings() {
        let expected = [
            (ErrorCode::UnknownId, "unknown_id"),
            (ErrorCode::Duplicate, "duplicate"),
            (ErrorCode::Cycle, "cycle"),
            (ErrorCode::MissingVerify, "missing_verify"),
            (ErrorCode::MissingEvidence, "missing_evidence"),
            (ErrorCode::MissingNote, "missing_note"),
            (ErrorCode::MissingResolution, "missing_resolution"),
            (ErrorCode::OrphanNotAllowed, "orphan_not_allowed"),
        ];
        for (code, spelling) in expected {
            assert_eq!(code.as_str(), spelling);
            assert_eq!(code.to_string(), spelling);
        }
    }

    #[test]
    fn every_variant_maps_to_its_code() {
        let cases = [
            (
                DomainError::UnknownId {
                    id: "camera:U1".into(),
                },
                "unknown_id",
            ),
            (
                DomainError::Duplicate {
                    id: "camera:U1".into(),
                },
                "duplicate",
            ),
            (
                DomainError::Cycle {
                    path: vec!["camera:U1".into(), "camera:U1".into()],
                },
                "cycle",
            ),
            (
                DomainError::MissingVerify {
                    id: "camera:U1".into(),
                },
                "missing_verify",
            ),
            (
                DomainError::MissingEvidence {
                    id: "camera:U1".into(),
                },
                "missing_evidence",
            ),
            (
                DomainError::MissingNote {
                    id: "camera:U1".into(),
                },
                "missing_note",
            ),
            (
                DomainError::MissingResolution {
                    id: "camera:D1".into(),
                },
                "missing_resolution",
            ),
            (
                DomainError::OrphanNotAllowed {
                    id: "camera:E1".into(),
                },
                "orphan_not_allowed",
            ),
        ];
        for (error, spelling) in cases {
            assert_eq!(error.code().as_str(), spelling);
            // Display never empty: the message carries the payload.
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn display_formats_payloads_deterministically() {
        let err = DomainError::Cycle {
            path: vec!["camera:U1".into(), "camera:U2".into(), "camera:U1".into()],
        };
        assert_eq!(
            err.to_string(),
            "cycle: camera:U1 -> camera:U2 -> camera:U1"
        );
    }
}
