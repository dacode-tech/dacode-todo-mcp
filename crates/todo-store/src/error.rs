//! The store-level error codes of spec §6.1: `corrupt_store`,
//! `version_mismatch`, `locked`, `io`. The domain's error module names
//! all four as this crate's and produces none of them.
//!
//! [`StoreError`] carries those codes plus [`StoreError::Domain`], which
//! wraps the domain's shared vocabulary rather than translating it — a
//! rejected mutation reports the same error the graph would have, because
//! that is what happened. §6.1's list is closed and versioned like the
//! format: `locked` — §8's single-writer refusal — landed with the D4
//! layout's advisory locks, and `io` joined at U9 (signed 2026-09-29),
//! which blessed the spelling this crate's file-fault variant already
//! returned as a contract code. A client may retry `io` — and must not
//! retry `invalid`, §6.1's other U9 code, which is malformed input and
//! M3's mapping of the domain's value faults, not this crate's.
//!
//! No foreign type appears in a variant: the engine's error does not
//! cross this crate's boundary (§2), so engine and `std::io` messages are
//! carried as `detail` strings.

use std::fmt;

use todo_domain::{DomainError, RepoSlug};

/// Which store §8's single writer found contended — the payload of
/// [`StoreError::Locked`], and the unit a lock is scoped to (spec §8:
/// one lock per repo store plus one for the registry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockTarget {
    /// One repository's store, named by its slug — never by a path
    /// (spec §7: the slug is the key).
    Repo(RepoSlug),
    /// The shared registry file.
    Registry,
    /// A store handed to this crate as a bare path outside the D4
    /// layout, which therefore has no slug to be named by (spec §7 —
    /// the slug is the key, and this store was not opened by one;
    /// f17's third variant, t13's decision). Unreachable through
    /// [`crate::DataRoot`], whose every open derives a slug.
    Unnamed,
}

/// An error raised by the store. Codes are stable machine-readable
/// identities a client may branch on (spec §6.1); `detail` text is
/// context for a human reader and never the contract — the same line
/// the domain's `DomainError` draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// Stored bytes could not be parsed into the value they claim to
    /// hold, or a value could not be serialized (see [`crate::to_bytes`]
    /// and [`crate::from_bytes`]). Corruption is this named error, never
    /// a panic and never a silent truncation (spec §4).
    ///
    /// Over a log, `offset` is the **absolute** byte offset of the
    /// offending line's start — where a repair truncates from (§4.3) —
    /// while the engine's own position *within* that line rides in
    /// `detail`. The codec's offsets, by contrast, are relative to the
    /// slice it was handed (see [`crate::from_bytes`]).
    CorruptStore {
        /// The byte offset where the engine located the fault (spec
        /// §4.3: the error carries its offset), when the fault has a
        /// position in the input. `None` for faults without one: a
        /// validated-newtype rejection, a missing field, or a failure in
        /// the serialization direction.
        offset: Option<usize>,
        /// The engine's message as its `Display` text. The engine error
        /// *type* never crosses this crate's boundary (§2).
        detail: String,
    },
    /// The stored bytes declare a format version this build does not read
    /// (spec §4: a format change is a migration, never an in-place edit
    /// of meaning). Nothing is written to such a log.
    VersionMismatch {
        /// The version the header declares.
        found: u32,
        /// The version this build writes and reads — [`crate::FORMAT_VERSION`].
        expected: u32,
    },
    /// Another writer holds the store's advisory lock (spec §8's single
    /// writer; §6.1's `locked` when contended). Nothing was opened, read
    /// or written: the refusal is the whole effect of the attempt.
    Locked {
        /// The contended lock — a repo store or the registry.
        target: LockTarget,
    },
    /// The filesystem refused a read, a write or an `fsync`.
    ///
    /// Carries §6.1's `io` code (U9, signed 2026-09-29): the spelling
    /// this crate already returned was blessed as a contract code, so a
    /// client may branch on it — and may retry it, the filesystem being
    /// the fault, not the request. That is the distinction from
    /// `invalid`, malformed input, which must not be retried.
    Io {
        /// What was attempted and the [`std::io::ErrorKind`] it failed
        /// with. Never the `io::Error` itself, and never a host path.
        detail: String,
    },
    /// The domain rejected the mutation — a validation, a duplicate, an
    /// unknown id, a cycle. The write is atomic (spec §8): nothing was
    /// applied and nothing was appended.
    Domain(DomainError),
}

impl StoreError {
    /// The stable §6.1 code of this error — total, one code per variant.
    /// [`StoreError::Io`] reports `"io"`, one of §6.1's codes since U9;
    /// see that variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::CorruptStore { .. } => "corrupt_store",
            Self::VersionMismatch { .. } => "version_mismatch",
            Self::Locked { .. } => "locked",
            Self::Io { .. } => "io",
            Self::Domain(error) => error.code().as_str(),
        }
    }

    /// The domain error this carries, when it wraps one.
    pub fn domain(&self) -> Option<&DomainError> {
        match self {
            Self::Domain(error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CorruptStore {
                offset: Some(offset),
                detail,
            } => write!(f, "corrupt store at byte {offset}: {detail}"),
            Self::CorruptStore {
                offset: None,
                detail,
            } => write!(f, "corrupt store: {detail}"),
            Self::VersionMismatch { found, expected } => write!(
                f,
                "store format version {found}, this build reads version {expected}"
            ),
            Self::Locked {
                target: LockTarget::Repo(slug),
            } => write!(f, "store is locked: repo {slug}"),
            Self::Locked {
                target: LockTarget::Registry,
            } => write!(f, "store is locked: the registry"),
            // No slug, no path (§7): the store was opened by bare path,
            // so the honest report says that and names nobody.
            Self::Locked {
                target: LockTarget::Unnamed,
            } => write!(
                f,
                "store is locked: an unnamed store (opened by path, not by slug)"
            ),
            Self::Io { detail } => write!(f, "io: {detail}"),
            Self::Domain(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<DomainError> for StoreError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

/// An io fault as this crate reports it: what was attempted and the
/// [`std::io::ErrorKind`] that refused it. Never the `io::Error` itself
/// (no foreign type crosses the boundary, §2) and never a path —
/// `io::Error`'s own message can carry one, which is exactly why the
/// detail is spelled from the kind. The one construction site of every
/// [`StoreError::Io`] in the crate.
pub(crate) fn io_error(context: &str, error: &std::io::Error) -> StoreError {
    io_error_kind(context, error.kind())
}

/// [`io_error`] for a kind already known — same one spelling, so a
/// reconstructed fault is byte-identical to an observed one.
pub(crate) fn io_error_kind(context: &str, kind: std::io::ErrorKind) -> StoreError {
    StoreError::Io {
        detail: format!("{context}: {kind:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corrupt(offset: Option<usize>) -> StoreError {
        StoreError::CorruptStore {
            offset,
            detail: "expected value at byte 3".to_owned(),
        }
    }

    #[test]
    fn corrupt_store_code_is_the_section_6_1_spelling() {
        assert_eq!(corrupt(Some(3)).code(), "corrupt_store");
        assert_eq!(corrupt(None).code(), "corrupt_store");
    }

    #[test]
    fn display_carries_the_offset_when_present() {
        assert_eq!(
            corrupt(Some(17)).to_string(),
            "corrupt store at byte 17: expected value at byte 3"
        );
        assert_eq!(
            corrupt(None).to_string(),
            "corrupt store: expected value at byte 3"
        );
    }

    #[test]
    fn display_is_non_empty_and_deterministic() {
        for err in [corrupt(Some(0)), corrupt(None)] {
            let once = err.to_string();
            assert!(!once.is_empty());
            assert_eq!(once, err.to_string());
        }
    }

    #[test]
    fn version_mismatch_reports_both_versions_and_its_code() {
        let err = StoreError::VersionMismatch {
            found: 2,
            expected: 1,
        };
        assert_eq!(err.code(), "version_mismatch");
        assert_eq!(
            err.to_string(),
            "store format version 2, this build reads version 1"
        );
        assert_eq!(err.domain(), None);
    }

    #[test]
    fn locked_names_its_target_and_code_without_a_path() {
        use std::str::FromStr;

        let repo = StoreError::Locked {
            target: LockTarget::Repo(RepoSlug::from_str("camera").unwrap()),
        };
        assert_eq!(repo.code(), "locked");
        assert_eq!(repo.to_string(), "store is locked: repo camera");
        assert_eq!(repo.domain(), None);
        assert!(!repo.to_string().contains('/'));

        let registry = StoreError::Locked {
            target: LockTarget::Registry,
        };
        assert_eq!(registry.code(), "locked");
        assert_eq!(registry.to_string(), "store is locked: the registry");
        assert_eq!(registry.domain(), None);
    }

    #[test]
    fn io_carries_its_context_and_its_section_6_1_code() {
        let err = StoreError::Io {
            detail: "open the event log: NotFound".to_owned(),
        };
        assert_eq!(err.code(), "io");
        assert_eq!(err.to_string(), "io: open the event log: NotFound");
    }

    #[test]
    fn a_domain_fault_keeps_the_domain_code_and_message() {
        let domain = DomainError::MissingEvidence {
            id: "camera:U7".to_owned(),
        };
        let err = StoreError::from(domain.clone());
        assert_eq!(err.code(), "missing_evidence");
        assert_eq!(err.domain(), Some(&domain));
        // The message is the domain's own: no store-level rewording
        // stands between a client and the shared vocabulary.
        assert_eq!(err.to_string(), domain.to_string());
        assert_eq!(err, StoreError::Domain(domain));
    }
}
