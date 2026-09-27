//! The canonical serializer (spec §4, §8): one value in, deterministic
//! bytes out, and one document in, one value out.
//!
//! The engine — `dacode-json`, a serde_json-compatible JSON
//! implementation and D1's line format — is sealed inside this module:
//! the public signatures speak only `serde` traits and
//! [`crate::StoreError`], so no engine type crosses the crate boundary
//! (§2). The canonical form of the node schema is its serde contract:
//! struct fields in declaration order, [`todo_domain::References`]
//! sorted by `(kind, target, note)`, no floats and no maps (§8).
//!
//! The contract names deliberately match the domain's test-only codec:
//! it is the same contract — round-trip equality, twice-run byte
//! identity — in the real format the golden files pin (spec §10).

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::StoreError;

/// Serializes one value to its canonical bytes.
///
/// A serialization failure maps to [`StoreError::CorruptStore`] with
/// `offset: None`: the node schema is closed — no floats, no maps — so
/// the serialization direction is unreachable for the types this crate
/// persists, and §6.1 has no separate code for it.
pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, StoreError> {
    dacode_json::to_vec(value).map_err(|engine| StoreError::CorruptStore {
        offset: None,
        detail: engine.to_string(),
    })
}

/// Deserializes one value from canonical bytes.
///
/// Every failure — a parse fault, a validated newtype's rejection, a
/// missing field — is [`StoreError::CorruptStore`] (spec §4: corruption
/// is a named error, never a panic); parse faults carry the engine's
/// byte offset (spec §4.3), type and validation faults have no position
/// and carry `None`.
///
/// Trailing-byte discipline, pinned by `tests/golden.rs` for t10's
/// JSONL line splitter: bytes after the top-level value are rejected
/// with the offset of the first stray byte — *except* whitespace, so a
/// line's own terminating newline parses clean.
pub fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, StoreError> {
    dacode_json::from_slice(bytes).map_err(|engine| StoreError::CorruptStore {
        offset: engine.offset(),
        detail: engine.to_string(),
    })
}
