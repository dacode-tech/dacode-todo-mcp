//! The folded rewards read (U79): the totals and the events from one
//! walk per log — the render's read, pair-identical to the two
//! siblings' answers at half the reads. The pair-identity is the
//! fold's whole contract: the render's goldens pin the bytes one
//! layer up, these tests pin the fold against the siblings.

use std::fs;

use tempfile::TempDir;
use todo_store::{ReflectionTrigger, RewardsTotals, Store};

use super::events::{alpha_slug, events_root};
use super::fixtures::at;
use super::timeline::{camera_slug, epic_id};

/// The fold answers the two siblings' answers, pair-identical: the
/// totals' checked sums and per-repo map, the events' root order and
/// verbatim fields — one walk, both halves, no drift. The world is
/// the events suite's own (two repos, three grants, one reflection,
/// the root order disagreeing with both the registration order and
/// the timestamps'), so every arm of the fold's match carries data.
#[test]
fn the_fold_answers_the_siblings_answers_pair_identical() {
    let dir = TempDir::new().unwrap();
    let root = events_root(&dir);
    // One reflection after the grants, as the events suite's own
    // order test does.
    let mut camera = root.open(&camera_slug()).unwrap();
    camera
        .log_reflection(
            "The milestone closed clean.".to_owned(),
            ReflectionTrigger::EpicClosure { epic: epic_id(1) },
            RewardsTotals::default(),
            at("2026-09-25T13:00:00Z"),
        )
        .unwrap();
    drop(camera);

    let folded = root.rewards_totals_and_events().unwrap();
    assert_eq!(
        folded,
        (
            root.rewards_totals().unwrap(),
            root.rewards_events().unwrap()
        ),
        "the fold is the siblings' pair, byte for byte"
    );
    // The halves carry the world: three grants (alpha's 28, camera's
    // 11 and 28), one reflection, the totals summed.
    assert_eq!(folded.0.total, 67);
    assert_eq!(folded.1.grants.len(), 3);
    assert_eq!(folded.1.reflections.len(), 1);
}

/// A repo whose log will not parse fails the fold with the same
/// error the siblings report — the all-or-nothing reading, one
/// spelling on every path.
#[test]
fn a_damaged_log_fails_the_fold_with_the_siblings_error() {
    let dir = TempDir::new().unwrap();
    let root = events_root(&dir);
    let alpha_log = root.log_path(&alpha_slug());
    let valid = fs::read(&alpha_log).unwrap();
    let mut damaged = valid.clone();
    damaged.extend_from_slice(b"{{{ not json\n");
    fs::write(&alpha_log, damaged).unwrap();

    let error = root.rewards_totals_and_events().unwrap_err();
    assert_eq!(error, root.rewards_totals().unwrap_err());
    assert_eq!(error, root.rewards_events().unwrap_err());
}
