//! The per-root config document (§15, D23, signed 2026-10-13):
//! `<root>/config.json`, the first config document the store owns.
//!
//! Versioned, canonical and golden-pinned like the registry (§4's
//! discipline): one compact JSON object plus exactly one trailing LF,
//! fields in declaration order, written temp-file + `fsync` + atomic
//! rename. The persisted shape of the rewards block is the domain's
//! [`RewardsConfig`] — its declaration order is the canonical order, so
//! this crate adds only the version header around it.
//!
//! The document is born at [`FORMAT_VERSION`] 2 and has no predecessor
//! spelling: a config declaring any other version is
//! [`StoreError::VersionMismatch`] — for this document there is no v1
//! store to be lenient with, so the probe-first ordering's leniency has
//! nothing to accept. An **absent** file is no config — the inert root,
//! §15's whole safety: no events, no prompts, no display section, no
//! wire bytes that differ. A **malformed** one is
//! [`StoreError::CorruptStore`]: a config this build cannot parse is
//! not silently an inert root, because that would hide the damage.
//!
//! Writes take the registry lock — the root-level writer lock, shared
//! with the registry's writers rather than given a second lock target.
//! No lock-order impact exists: a config write holds no store lock, so
//! it can never sit on the wrong side of §8's registry-then-stores
//! order (see `layout`'s lock-order docs). Reads take no lock and
//! create nothing, like [`crate::Registry::load`].

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use todo_domain::RewardsConfig;

use crate::FORMAT_VERSION;
use crate::codec::{VersionProbe, from_bytes, to_bytes};
use crate::error::{StoreError, io_error};
use crate::layout::DataRoot;
use crate::snapshot::{rename_into_place, temp_path_for, write_temp};

/// The config's on-disk document: the version header and the rewards
/// block, in the field order the golden pins (§4's versioning
/// requirement; §8's canonical serialization).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigDocument {
    /// The [`FORMAT_VERSION`] this document was written with; a
    /// different value is [`StoreError::VersionMismatch`] — this
    /// document has no predecessor spelling (module docs).
    pub(crate) format_version: u32,
    /// The rewards block — the domain's persisted shape, its
    /// declaration order the canonical order.
    pub(crate) rewards: RewardsConfig,
}

/// The fault contexts, pinned by tests: deterministic, and never
/// carrying a host path.
const CONTEXT_READ: &str = "read the config";
const CONTEXT_PARSE: &str = "the config is not a parseable document";
/// The subject the write halves name in their io-fault contexts.
const SUBJECT_CONFIG: &str = "the config";

/// The line terminator the config document ends with — the log's line
/// discipline, and the golden workflow's "exactly one trailing
/// newline".
const LINE_FEED: u8 = b'\n';

/// Reads the config at `path`: `None` when the file is absent (the
/// inert root), the rewards block when it is there and parses, and a
/// named error otherwise. The engine's grant path reads through this —
/// a bare-path store with no config source never calls it, and a
/// layout store calls it fresh at each closure so a config change
/// takes effect without a reopen.
pub(crate) fn load(path: &Path) -> Result<Option<RewardsConfig>, StoreError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(CONTEXT_READ, &error)),
    };
    // The probe-first ordering (§4): a foreign version is the migration
    // question even when the rest of the document is unknown here.
    let probe: VersionProbe = from_bytes(&bytes).map_err(|error| corrupt(CONTEXT_PARSE, &error))?;
    if probe.format_version != FORMAT_VERSION {
        return Err(StoreError::VersionMismatch {
            found: probe.format_version,
            expected: FORMAT_VERSION,
        });
    }
    let document: ConfigDocument =
        from_bytes(&bytes).map_err(|error| corrupt(CONTEXT_PARSE, &error))?;
    Ok(Some(document.rewards))
}

impl DataRoot {
    /// Loads the root's rewards config: `None` when `<root>/config.json`
    /// is absent — the inert root, §15's opt-in — and the config when it
    /// is there. A malformed or foreign-versioned config is the named
    /// error, never a silent inert root. Takes no lock and creates
    /// nothing.
    pub fn load_config(&self) -> Result<Option<RewardsConfig>, StoreError> {
        load(&self.config_path())
    }

    /// Writes the root's rewards config as the canonical document under
    /// §4.3's atomic-rename discipline, holding the registry lock — the
    /// root-level writer lock — for the write (module docs). The lock
    /// creates the root directory when absent, which is the same
    /// creating step [`DataRoot::lock_registry`] documents.
    pub fn save_config(&self, config: &RewardsConfig) -> Result<(), StoreError> {
        let registry_lock = self.lock_registry()?;
        let document = ConfigDocument {
            format_version: FORMAT_VERSION,
            rewards: config.clone(),
        };
        let mut bytes = to_bytes(&document)?;
        bytes.push(LINE_FEED);
        let path = self.config_path();
        let temporary = temp_path_for(&path);
        write_temp(&temporary, &bytes, SUBJECT_CONFIG)?;
        rename_into_place(&temporary, &path, SUBJECT_CONFIG)?;
        drop(registry_lock);
        Ok(())
    }
}

/// A codec rejection as this crate reports it: the context prefix plus
/// the codec's own message, carrying the codec's offset through — over
/// a whole-file read that offset is absolute (§4.3). The registry's and
/// the snapshot's helper, for the same document shape and the same
/// reason.
fn corrupt(context: &str, error: &StoreError) -> StoreError {
    match error {
        StoreError::CorruptStore { offset, detail } => StoreError::CorruptStore {
            offset: *offset,
            detail: format!("{context}: {detail}"),
        },
        // `from_bytes` only ever fails as `CorruptStore`; the arm keeps
        // the mapping total rather than asserting that.
        other => other.clone(),
    }
}
