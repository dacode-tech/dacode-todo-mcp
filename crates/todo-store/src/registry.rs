//! The shared registry (D4): the slug index of every repository this
//! data root holds, in one versioned JSON document.
//!
//! §2 calls it "repo metadata and the slug→store mapping". The mapping
//! is **derived, not stored**: §2 already fixes the layout
//! (`repos/<slug>/`), so writing the directory into the file would be a
//! second source of truth for a constant. What is stored is
//! `{slug, registered_at}` per entry — the two immutable fields (§3: the
//! slug is the key, `registered_at` is set once), and exactly the
//! immutability that keeps this index honest: it cannot diverge from the
//! store's `Repo` node, because neither side ever rewrites what the
//! other holds. `name` and `path` are §7's mutables and stay solely in
//! the node — indexing them would need a second write per
//! `repo_set_path`/`repo_set_name` and would break the lock-order
//! argument in [`crate::DataRoot`]'s module docs.
//!
//! The recorded cost of the narrow reading: a cold `repo_list` over many
//! repos must open stores to show names and paths (§2's own "iterate the
//! registry and open per-repo stores"); M3's long-lived server answers
//! from handles it already holds. If M3 finds that insufficient,
//! extending the registry is a golden-visible format change — cheap
//! before M3 ships, expensive after.
//!
//! Bytes: one compact JSON object followed by exactly one LF (the log's
//! line discipline), fields in declaration order, `repos` sorted by slug
//! ascending — the sort is an invariant of [`Registry`], established on
//! load and on write, so two runs over the same registrations are
//! byte-identical (§8). Pinned by `tests/golden/registry.v1.golden`.
//!
//! Faults: a **missing** `registry.json` loads as an empty registry and
//! creates nothing — a fresh root legitimately has none, and a read path
//! creates no files. A **malformed** one is [`StoreError::CorruptStore`]
//! carrying the codec's offset, which over a whole-file read *is*
//! absolute; a wrong `format_version` is
//! [`StoreError::VersionMismatch`], reported before the document is read
//! strictly, so a registry written by a newer build is the migration
//! question it is and not damage. `deny_unknown_fields` is deliberate,
//! and this document is the precedent for it: the registry is whole-file
//! rewritten and version-gated, so an unknown field at the *current*
//! version can only be damage — the reading t13's f14 landing extended
//! to the log header, which the snapshot document shares.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use todo_domain::{Repo, RepoSlug, Timestamp};

use crate::FORMAT_VERSION;
use crate::codec::{VersionProbe, from_bytes, to_bytes};
use crate::error::{StoreError, io_error};
use crate::layout::DataRoot;

/// The registry's on-disk document: the version header and the slug
/// index, in the field order the golden pins (§4's versioning
/// requirement; §8's canonical serialization).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryDocument {
    /// The [`FORMAT_VERSION`] this document was written with; a
    /// different value is [`StoreError::VersionMismatch`].
    format_version: u32,
    /// The registered repositories, sorted by slug ascending — an
    /// invariant of [`Registry`], re-established on every load and
    /// write.
    repos: Vec<RegistryEntry>,
}

/// One registered repository's immutable half (§3): the slug key and the
/// registration instant. The mutable half — `name`, `path` — lives only
/// in the store's `Repo` node (module docs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryEntry {
    /// When the repository was registered; the seed event's `ts`.
    pub registered_at: Timestamp,
    /// The immutable slug key (D3).
    pub slug: RepoSlug,
}

/// The loaded registry: read-only from outside this crate, because the
/// only writer is [`DataRoot::register`] and every write goes through
/// the registry lock (§8) — a public `save` would let a caller write the
/// index without holding it. §6 has no `repo_delete` and D3 never reuses
/// a slug, so registration is the whole mutation surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    /// The entries, sorted by slug ascending — established by
    /// [`Registry::load`] and maintained by [`Registry::insert`].
    entries: Vec<RegistryEntry>,
}

impl Registry {
    /// Loads the registry of the data root `root`.
    ///
    /// A missing file is an empty registry, not an error and not a
    /// creation: a fresh data root legitimately has none yet. A malformed
    /// one — bad JSON, an unknown field, a slug or timestamp that fails
    /// validation — is [`StoreError::CorruptStore`]: a registry this
    /// build cannot parse is not a registry (spec §4: corruption is a
    /// named error, never a silent truncation). A foreign
    /// `format_version` is [`StoreError::VersionMismatch`].
    pub fn load(root: &DataRoot) -> Result<Self, StoreError> {
        let bytes = match fs::read(root.registry_path()) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::empty()),
            Err(error) => return Err(io_error("read the registry", &error)),
        };
        Self::parse(&bytes)
    }

    /// The empty registry: what a fresh data root loads as.
    pub(crate) fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The whole load path below the file read: version probe, strict
    /// parse, sort.
    fn parse(bytes: &[u8]) -> Result<Self, StoreError> {
        let probe: VersionProbe =
            from_bytes(bytes).map_err(|error| corrupt(CONTEXT_PARSE, &error))?;
        if probe.format_version != FORMAT_VERSION {
            return Err(StoreError::VersionMismatch {
                found: probe.format_version,
                expected: FORMAT_VERSION,
            });
        }
        let document: RegistryDocument =
            from_bytes(bytes).map_err(|error| corrupt(CONTEXT_PARSE, &error))?;
        let mut entries = document.repos;
        entries.sort_by(|left, right| left.slug.cmp(&right.slug));
        Ok(Self { entries })
    }

    /// The entries in canonical order: slug ascending.
    pub fn entries(&self) -> &[RegistryEntry] {
        &self.entries
    }

    /// The registered slugs, ascending. Allocated like the domain's
    /// derived-collection accessors; the registry is small by nature —
    /// one entry per repository.
    pub fn slugs(&self) -> Vec<&RepoSlug> {
        self.entries.iter().map(|entry| &entry.slug).collect()
    }

    /// Whether `slug` is registered.
    pub fn contains(&self, slug: &RepoSlug) -> bool {
        self.entries
            .binary_search_by(|entry| entry.slug.cmp(slug))
            .is_ok()
    }

    /// Records `repo`'s immutable half, keeping the sorted invariant.
    /// Callers hold the registry lock; the duplicate refusal is
    /// [`DataRoot::register`]'s, which checks this index under that lock
    /// before creating any store.
    pub(crate) fn insert(&mut self, repo: &Repo) {
        let entry = RegistryEntry {
            registered_at: repo.registered_at.clone(),
            slug: repo.slug.clone(),
        };
        match self
            .entries
            .binary_search_by(|existing| existing.slug.cmp(&entry.slug))
        {
            // Unreachable for `register`'s call order — it refuses a
            // listed slug before creating the store — and replacing in
            // place keeps the invariant whatever happens.
            Ok(index) => {
                if let Some(slot) = self.entries.get_mut(index) {
                    *slot = entry;
                }
            }
            Err(index) => self.entries.insert(index, entry),
        }
    }

    /// Writes the registry of the data root `root` under §4.3's
    /// atomic-rename discipline: a pid-suffixed temp file in the same
    /// directory (same filesystem, so the rename is atomic; the pid keeps
    /// parallel writers from sharing a temp name — the registry lock
    /// already serializes them, the pid guards against debris), `fsync`,
    /// then the rename. The old registry stays readable until the rename
    /// lands.
    ///
    /// No directory `fsync` in t11: §4.3 names temp+fsync+rename, and a
    /// parent-dir fsync is a stronger guarantee than the signed text asks
    /// for plus a cross-platform story — recorded for t12's snapshot
    /// path, where durability actually decides recovery.
    pub(crate) fn save(&self, root: &DataRoot) -> Result<(), StoreError> {
        let document = RegistryDocument {
            format_version: FORMAT_VERSION,
            repos: self.entries.clone(),
        };
        let mut bytes = to_bytes(&document)?;
        bytes.push(LINE_FEED);

        let path = root.registry_path();
        let temporary = temp_path(&path);
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temporary)
                .map_err(|error| io_error(CONTEXT_WRITE_TEMP, &error))?;
            file.write_all(&bytes)
                .map_err(|error| io_error(CONTEXT_WRITE_TEMP, &error))?;
            file.sync_all()
                .map_err(|error| io_error(CONTEXT_FLUSH, &error))?;
        }
        fs::rename(&temporary, &path).map_err(|error| io_error(CONTEXT_RENAME, &error))
    }
}

/// The line terminator the registry document ends with — the log's line
/// discipline, and the golden workflow's "exactly one trailing newline".
const LINE_FEED: u8 = b'\n';

/// The fault contexts, pinned by tests: deterministic, and never
/// carrying a host path.
const CONTEXT_PARSE: &str = "the registry is not a parseable document";
const CONTEXT_WRITE_TEMP: &str = "write the registry's temporary file";
const CONTEXT_FLUSH: &str = "flush the registry to disk";
const CONTEXT_RENAME: &str = "rename the registry into place";

/// A codec rejection as this crate reports it: the context prefix plus
/// the codec's own message, carrying the codec's offset through — over a
/// whole-file read that offset is absolute (§4.3). Validation faults
/// (a slug or timestamp the newtypes reject) have no position and keep
/// the codec's `None`.
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

/// The pid-suffixed temp path beside `path` — the golden helper's
/// scheme, reused so the registry's write and the test harness's share
/// one debris story.
fn temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "registry.json".to_owned());
    let temporary = format!(".{file_name}.{}.tmp", std::process::id());
    match path.parent() {
        Some(parent) => parent.join(temporary),
        None => PathBuf::from(temporary),
    }
}
