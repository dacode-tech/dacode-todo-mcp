//! The `refresh` subcommand (spec §13, D20's third slice): the
//! on-demand re-import of one repo's board — the store is a derived
//! cache of the board, and this is the operation that rebuilds it
//! when the board has moved. The Python tool's `--refresh` ported,
//! generalized per-repo (its slug was hardcoded to this repo) and
//! made in-process (the Python drove a spawned server over the wire;
//! this module calls the store and the importer directly).
//!
//! The flow, each step a refusal before the next touches anything:
//! resolve the repo (the registry consult, the lock-free read, the
//! slug cross-check — `render::resolve`, the one home of the
//! discipline); the gone-board pre-flight (a cut-over repo's board is
//! archived and its store primary — refused with nothing touched);
//! **the plan built before the surgery** (the load-bearing divergence
//! from the Python: a gone *or broken* board refuses while the store
//! is still intact, where the Python destroyed the store first and
//! let the import fail after); the surgery
//! ([`DataRoot::unregister`] — the registry entry and the store
//! directory under one lock, the t54 crash order); the re-register
//! (the node's name and path preserved, a fresh registration instant
//! — the staleness signal the page shows); the apply and the census.
//!
//! The dialect is the repo's slug — the observed coincidence (every
//! current repo's dialect wire value equals its slug) made the
//! default; the remedy if a repo ever diverges is a `--dialect` flag,
//! and a new repo's onboarding keeps the value equal to its slug.

use todo_domain::{Repo, RepoSlug};
use todo_store::{DataRoot, Store};

use crate::ToolError;
use crate::import::plan::{ApplyError, apply_plan};
/// The census the refresh answers — re-exported so the subcommand's
/// public return type is reachable where the function is.
pub use crate::importer::ImportedDto;
use crate::importer::{Dialect, board_present, imported_counts, plan_for};
use crate::render::resolve;
use crate::time;

/// Re-imports `slug`'s board into `root`: the registry surgery, the
/// re-register, the apply, the census. A refusal at any step before
/// the surgery leaves the store exactly as it was — the gone-board
/// pre-flight and the plan-before-surgery order are the two
/// guarantees that make this safe to run against a repo whose board
/// may be gone or broken.
pub fn refresh(root: &DataRoot, slug: &RepoSlug) -> Result<ImportedDto, String> {
    let graph = resolve(root, slug).map_err(|error| error.message)?;
    let node = graph.repo();
    let name = node.name.clone();
    let path = node.path.clone();
    if !board_present(&path) {
        return Err(format!(
            "no board to import for {slug} — the checkout holds no board directory; \
             nothing was touched (a cut-over repo's store is the record)"
        ));
    }
    let dialect = Dialect::from_param(Some(slug.as_str())).map_err(|error| error.message)?;
    let plan = plan_for(&path, slug, dialect).map_err(|error| error.message)?;
    root.unregister(slug)
        .map_err(|error| ToolError::from(error).message)?;
    let mut store = root
        .register(Repo {
            slug: slug.clone(),
            name,
            path,
            registered_at: time::now(),
        })
        .map_err(|error| ToolError::from(error).message)?;
    match apply_plan(&plan, &mut store) {
        Ok(()) => {}
        Err(ApplyError::Refused(error)) => return Err(ToolError::from(error).message),
        Err(ApplyError::Store(error)) => return Err(ToolError::from(error).message),
    }
    Ok(imported_counts(store.graph(), slug))
}
