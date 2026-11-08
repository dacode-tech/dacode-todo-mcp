//! The layout suite's shared fixtures: repo and slug builders, the
//! injected data root over a `TempDir`, and small helpers the lock and
//! registry tests reuse.
//!
//! Nothing here is a test, so this module contributes no test names of
//! its own (P-LIVE-002). Every value is caller-passed and deterministic:
//! no clock, no environment, no absolute host path (§8) — a data root is
//! always injected from a `TempDir`, and a repo's `registered_at` is a
//! fixed timestamp string. The fixtures continue the crate's `camera`
//! world so the registry golden describes the same repos the other
//! subjects do.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use tempfile::TempDir;
use todo_domain::{EpicDraft, References, Repo, RepoSlug, Status, Timestamp};
use todo_store::DataRoot;

/// A fixed audit instant — the layout tests pass timestamps in, never
/// read a clock (§8).
pub(super) fn at(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

/// A repository slug, or a test failure naming the bad input.
pub(super) fn slug(value: &str) -> RepoSlug {
    RepoSlug::from_str(value).unwrap_or_else(|error| panic!("{value:?} is not a slug: {error}"))
}

/// A repository under `value` (a slug string), registered at
/// `registered_at`, with a deterministic display name and checkout path
/// derived from the slug.
///
/// The name and path are §7's mutables and live only on the node — they
/// never reach the registry bytes (which hold slug and `registered_at`),
/// so the registry golden is unaffected by their spelling.
pub(super) fn repo(value: &str, registered_at: &str) -> Repo {
    Repo {
        slug: slug(value),
        name: format!("{value} repository"),
        path: format!("/srv/repos/{value}"),
        registered_at: at(registered_at),
    }
}

/// The suite's camera repo at a fixed instant — the same repository the
/// log and serializer goldens describe.
pub(super) fn camera() -> Repo {
    repo("camera", "2026-09-25T08:00:00Z")
}

/// A minimal epic draft — the append a refused second writer must not
/// have cost the first (required test 4's "the refusal cost it nothing").
pub(super) fn epic_draft(title: &str) -> EpicDraft {
    EpicDraft {
        title: title.to_owned(),
        description: String::new(),
        status: Status::Open,
        references: References::new(),
    }
}

/// A data root injected over `dir`'s path — the layout's whole input, and
/// the only way these tests reach a filesystem location (never a fixed
/// path, never the cwd). The `TempDir` must outlive the returned root.
///
/// Named `data_root`, not `root`, so a test's `let root = data_root(&dir)`
/// binding does not shadow the builder it is calling.
pub(super) fn data_root(dir: &TempDir) -> DataRoot {
    DataRoot::new(dir.path())
}

/// Registers `repo` and immediately releases the store's lock by dropping
/// the returned handle — the common setup for a test that then opens the
/// store itself. Returns nothing: the registration's effect is on disk
/// (the log and the registry entry), which is what the caller reads back.
pub(super) fn register_and_release(root: &DataRoot, repo: Repo) {
    drop(root.register(repo).unwrap());
}

/// The names of `dir`'s entries, sorted — including dotfiles, so a
/// leftover temporary is caught. The debris tests' one directory reader.
pub(super) fn dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", dir.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
