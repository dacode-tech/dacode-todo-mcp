//! The cwd default tier (§7, D8 armed at t53): the deterministic
//! longest-ancestor match between the server's working directory and
//! the registered checkouts.
//!
//! Two halves with different natures. [`match_slug`] is the matcher:
//! a function of the passed cwd and the candidate paths, deterministic
//! given the filesystem — the lib stays env-free (the cwd arrives as a
//! parameter, read once by the binary, never from the environment
//! here). The `derive` walk is what feeds it: the registry
//! load, the handle-cache resolution and the `Repo` node path reads
//! (D5). The walk is deliberately its own, not `answer_repo_list`'s —
//! the listing answers DTOs for the wire, the derivation answers a
//! match; different concerns, and registry.rs stays frozen this task.
//!
//! The match rule (§7): both sides canonicalized — a candidate that
//! fails to canonicalize is skipped, because a checkout that does not
//! exist cannot be the cwd's ancestor — a candidate matches iff its
//! canonical path is a component-wise ancestor-or-equal of the
//! canonical cwd (`Path::starts_with`, never a string prefix), the
//! LONGEST match (most path components) wins, and a tie between
//! distinct slugs at equal length is [`MatchOutcome::Ambiguous`] —
//! refused, never guessed. The refusal is an answer with content (the
//! tied slugs), not a nested absence (J2).
//!
//! [`CwdDefault`] is the per-process memo (D3): the derivation runs
//! once, at the first default-needing call, and the outcome is fixed
//! for the process — a later `repo_set_path` or `repo_register` does
//! not move it. The memo caches the full outcome, not only the slug:
//! the Ambiguous refusal's message names the tied slugs (D4), so the
//! refusal's content must survive the cache like the match does. An
//! errored derivation (a registry that will not load, a store that
//! will not open) leaves the slot `Pending` — the memo caches
//! answers, not faults.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use todo_domain::RepoSlug;
use todo_store::Registry;

use crate::{ServerState, ToolError};

/// The matcher's answer for one candidate walk (J2): the longest
/// unambiguous match, no match, or the refusal that names the tie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchOutcome {
    /// The longest component-wise ancestor-or-equal match won.
    Match(RepoSlug),
    /// No candidate's canonical path is an ancestor-or-equal of the
    /// canonical cwd — including the case where the cwd itself no
    /// longer canonicalizes.
    NoMatch,
    /// Distinct slugs tied at the longest match — refused, never
    /// guessed. The arm carries the tied slugs in the walk's order
    /// (the registry's slug-ascending order), so the refusal can name
    /// them.
    Ambiguous(Vec<RepoSlug>),
}

/// The per-process cwd-default memo (D3): `Pending` until the first
/// default-needing call derives the answer, then the derived
/// [`MatchOutcome`] — match, no-match or ambiguity — so the default's
/// identity, and the refusal's content, are fixed within the process.
/// An explicit two-state enum, never a double `Option`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CwdDefault {
    /// Not yet derived; the next default-needing call derives.
    Pending,
    /// The derived outcome, fixed for this process.
    Resolved(MatchOutcome),
}

/// The deterministic longest-ancestor match (D2): every candidate path
/// and the cwd are canonicalized before matching, a candidate matches
/// iff its canonical path is a component-wise ancestor-or-equal of the
/// canonical cwd, the longest match wins, and a tie between distinct
/// slugs at equal length is refused with the tied slugs.
///
/// Candidates arrive in the walk's order (slug-ascending from the
/// registry); the outcome's tie order is that order, so two runs over
/// the same candidates answer identical bytes (§8).
pub(crate) fn match_slug(cwd: &Path, candidates: &[(RepoSlug, PathBuf)]) -> MatchOutcome {
    // A cwd that no longer canonicalizes can have no ancestor among
    // the candidates — the honest answer is no match, not a fault.
    let Some(cwd_canonical) = canonicalize_or_none(cwd) else {
        return MatchOutcome::NoMatch;
    };
    let mut winner: Option<RepoSlug> = None;
    let mut winner_depth = 0;
    let mut ties: Vec<RepoSlug> = Vec::new();
    for (slug, path) in candidates {
        // A checkout that does not exist cannot be the cwd's ancestor.
        let Some(canonical) = canonicalize_or_none(path) else {
            continue;
        };
        // Component-wise ancestor-or-equal — `starts_with` compares
        // path components, never a string prefix.
        if !cwd_canonical.starts_with(&canonical) {
            continue;
        }
        let depth = canonical.components().count();
        match (&winner, depth.cmp(&winner_depth)) {
            // A shorter match never displaces the current winner.
            (_, Ordering::Less) => {}
            // A distinct slug at the winner's depth is a tie; the same
            // slug twice in one candidate list is one repo, not a tie.
            (Some(current), Ordering::Equal) => {
                if slug != current && !ties.contains(slug) {
                    ties.push(slug.clone());
                }
            }
            // A longer match — or the first — wins and clears the ties.
            _ => {
                winner = Some(slug.clone());
                winner_depth = depth;
                ties.clear();
            }
        }
    }
    match winner {
        None => MatchOutcome::NoMatch,
        Some(slug) if ties.is_empty() => MatchOutcome::Match(slug),
        Some(slug) => {
            let mut tied = Vec::with_capacity(ties.len() + 1);
            tied.push(slug);
            tied.extend(ties);
            MatchOutcome::Ambiguous(tied)
        }
    }
}

/// The cwd default's derivation (D5): the candidate walk plus the
/// match. An absent cwd (the binary's read failed — a deleted cwd)
/// answers no match: there is nothing to match against.
pub(crate) fn derive(state: &mut ServerState) -> Result<MatchOutcome, ToolError> {
    let Some(cwd) = state.cwd.clone() else {
        return Ok(MatchOutcome::NoMatch);
    };
    let candidates = candidates(state)?;
    Ok(match_slug(&cwd, &candidates))
}

/// The candidate walk (D5): the registry's slugs, each resolved
/// through the handle cache — a miss opens the store once and caches
/// it, the cold-open cost paid once per process — with each `Repo`
/// node's path as the candidate. A store that will not open fails the
/// walk with the store's own error through the carrier: silently
/// skipping it would make the derivation report less than the truth
/// while still looking complete.
fn candidates(state: &mut ServerState) -> Result<Vec<(RepoSlug, PathBuf)>, ToolError> {
    let registry = Registry::load(&state.root)?;
    let mut walked = Vec::with_capacity(registry.slugs().len());
    for slug in registry.slugs() {
        let handle = state.resolve(slug)?;
        walked.push((
            slug.clone(),
            PathBuf::from(handle.graph().repo().path.clone()),
        ));
    }
    Ok(walked)
}

/// `std::fs::canonicalize` as the matcher uses it: `None` when the
/// path does not resolve.
fn canonicalize_or_none(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use tempfile::TempDir;

    /// A candidate pair from a slug and a path literal.
    fn pair(slug: &str, path: &str) -> (RepoSlug, PathBuf) {
        (
            RepoSlug::from_str(slug).expect("a valid slug"),
            PathBuf::from(path),
        )
    }

    /// A candidate pair whose path is the real temp directory `dir`.
    fn real(slug: &str, dir: &TempDir) -> (RepoSlug, PathBuf) {
        (
            RepoSlug::from_str(slug).expect("a valid slug"),
            dir.path().to_path_buf(),
        )
    }

    /// The cwd equals a registered checkout → that repo (wire test 1's
    /// property, no process).
    #[test]
    fn the_cwd_equal_to_a_checkout_matches_that_repo() {
        let checkout = TempDir::new().expect("a fresh checkout");
        let cwd = checkout.path().to_path_buf();
        let outcome = match_slug(&cwd, &[real("fixture", &checkout)]);
        assert_eq!(
            outcome,
            MatchOutcome::Match(RepoSlug::from_str("fixture").expect("a valid slug"))
        );
    }

    /// The cwd under a SUBDIRECTORY of a registered checkout → still
    /// that repo (wire test 2's property: ancestor matching).
    #[test]
    fn the_cwd_under_a_subdirectory_still_matches() {
        let checkout = TempDir::new().expect("a fresh checkout");
        let sub = checkout.path().join("src").join("deep");
        fs::create_dir_all(&sub).expect("the subdirectory lands");
        let outcome = match_slug(&sub, &[real("fixture", &checkout)]);
        assert_eq!(
            outcome,
            MatchOutcome::Match(RepoSlug::from_str("fixture").expect("a valid slug"))
        );
    }

    /// Nested checkouts — one at the parent, one deeper — → the longer
    /// match wins (wire test 3's property).
    #[test]
    fn the_longest_match_wins_over_the_ancestor() {
        let parent = TempDir::new().expect("a parent checkout");
        // The nested checkout lives inside the parent's tree, so both
        // are ancestors of the cwd.
        let nested = parent.path().join("nested");
        fs::create_dir_all(&nested).expect("the nested directory lands");
        let cwd = nested.join("work");
        fs::create_dir_all(&cwd).expect("the working directory lands");
        let candidates = vec![
            real("base", &parent),
            (
                RepoSlug::from_str("nested").expect("a valid slug"),
                nested.clone(),
            ),
        ];
        // `base` sorts first, so a first-candidate-wins bug would
        // answer it — the longest match must answer `nested`.
        assert_eq!(
            match_slug(&cwd, &candidates),
            MatchOutcome::Match(RepoSlug::from_str("nested").expect("a valid slug"))
        );
    }

    /// A cwd matching nothing → no match (wire test 4's property —
    /// here with candidates that do not exist, so no filesystem is
    /// touched: a checkout that does not exist cannot be the cwd's
    /// ancestor).
    #[test]
    fn a_cwd_matching_nothing_answers_no_match() {
        let outcome = match_slug(
            Path::new("/definitely/not/registered"),
            &[
                pair("fixture", "/repos/fixture"),
                pair("alpha", "/repos/alpha"),
            ],
        );
        assert_eq!(outcome, MatchOutcome::NoMatch);
    }

    /// Two repos at the same canonical path → the refusal names both
    /// slugs, in the candidates' (walk) order (wire test 7's
    /// property).
    #[test]
    fn two_repos_at_one_path_are_an_ambiguous_refusal() {
        let checkout = TempDir::new().expect("a shared checkout");
        let cwd = checkout.path().to_path_buf();
        let candidates = vec![real("aaa", &checkout), real("zzz", &checkout)];
        assert_eq!(
            match_slug(&cwd, &candidates),
            MatchOutcome::Ambiguous(vec![
                RepoSlug::from_str("aaa").expect("a valid slug"),
                RepoSlug::from_str("zzz").expect("a valid slug"),
            ])
        );
    }

    /// The same slug twice in one candidate list is one repo, not a
    /// tie — the refusal is between distinct slugs only.
    #[test]
    fn the_same_slug_twice_is_one_repo_not_a_tie() {
        let checkout = TempDir::new().expect("a shared checkout");
        let cwd = checkout.path().to_path_buf();
        let candidates = vec![real("fixture", &checkout), real("fixture", &checkout)];
        assert_eq!(
            match_slug(&cwd, &candidates),
            MatchOutcome::Match(RepoSlug::from_str("fixture").expect("a valid slug"))
        );
    }

    /// A sibling directory sharing a STRING prefix with the checkout
    /// (`proj1` vs `proj11`) → no match: the comparison is
    /// component-wise, never bytes (wire test 10's property).
    #[test]
    fn a_string_prefix_sibling_does_not_match() {
        let parent = TempDir::new().expect("a fresh parent");
        let proj1 = parent.path().join("proj1");
        let proj11 = parent.path().join("proj11");
        fs::create_dir_all(&proj1).expect("proj1 lands");
        fs::create_dir_all(&proj11).expect("proj11 lands");
        let candidates = vec![(
            RepoSlug::from_str("proj1").expect("a valid slug"),
            proj1.clone(),
        )];
        assert_eq!(match_slug(&proj11, &candidates), MatchOutcome::NoMatch);
    }

    /// The canonicalization pin (J3, wire test 9's property): a
    /// candidate registered under a symlinked spelling still matches
    /// the physically-spelled cwd. This host's temp roots live under
    /// `/var/folders/…`, a symlink to `/private/var/folders/…`, and
    /// the OS hands a process spawned there the physical spelling —
    /// so the registered raw spelling and the cwd's physical spelling
    /// differ, and only canonicalization reconciles them. On a host
    /// whose temp root is already canonical this test still passes
    /// but pins nothing; the mutation proof (M2) runs here.
    #[test]
    fn a_symlinked_spelling_still_matches_through_canonicalization() {
        let checkout = TempDir::new().expect("a fresh checkout");
        let raw = checkout.path().to_path_buf();
        let physical = fs::canonicalize(&raw).expect("the physical spelling");
        // The cwd arrives as the OS spells it (physical); the
        // candidate is registered as the raw spelling.
        let candidates = vec![(
            RepoSlug::from_str("fixture").expect("a valid slug"),
            raw.clone(),
        )];
        assert_eq!(
            match_slug(&physical, &candidates),
            MatchOutcome::Match(RepoSlug::from_str("fixture").expect("a valid slug"))
        );
    }

    /// A cwd that no longer canonicalizes answers no match — the
    /// deleted-cwd case, honest rather than faulted.
    #[test]
    fn a_cwd_that_no_longer_resolves_answers_no_match() {
        let outcome = match_slug(
            Path::new("/definitely/gone"),
            &[pair("fixture", "/repos/fixture")],
        );
        assert_eq!(outcome, MatchOutcome::NoMatch);
    }
}
