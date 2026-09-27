//! The store-level error codes of spec §6.1: `corrupt_store`,
//! `version_mismatch`, `locked`. The domain's error module delegates
//! these three here by name.
//!
//! This commit lands the one variant the serializer needs,
//! [`StoreError::CorruptStore`]. The growth path is one variant per
//! code as the engine lands — `version_mismatch` with t10's log header
//! check, `locked` with t11's advisory locks, plus an io variant for
//! the file machinery — each defined here, never as a foreign type: the
//! engine's error does not cross this crate's boundary (§2), so engine
//! messages are carried as `detail` strings.

use std::fmt;

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
}

impl StoreError {
    /// The stable §6.1 code of this error — total, one code per variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::CorruptStore { .. } => "corrupt_store",
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
        }
    }
}

impl std::error::Error for StoreError {}

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
}
