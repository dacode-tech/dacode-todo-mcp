//! The reflection event (§15, D23): the verbatim answer, the trigger,
//! the totals at the moment — logged as-is, replayed as a no-op, and
//! round-tripped byte-stably through the codec.

use std::fs;

use tempfile::TempDir;
use todo_domain::Size;

use todo_store::{Event, ReflectionTrigger, RewardsTotals, Store, from_bytes, to_bytes};

use super::fixtures::{at, closing_patch, enabled_root_with_task, open};
use super::timeline::{camera_slug, epic_id, task_id};

/// The totals fixture: a root total and this repo's, at the moment.
fn totals(root: u64, repo: u64) -> RewardsTotals {
    RewardsTotals {
        total: root,
        by_repo: [(camera_slug(), repo)].into_iter().collect(),
    }
}

#[test]
fn a_reflection_is_logged_verbatim_and_replays() {
    let dir = TempDir::new().unwrap();
    let (root, task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    store
        .close_task(
            task,
            at("2026-09-25T10:00:00Z"),
            closing_patch(),
            Some(Size::M),
        )
        .unwrap();
    store
        .log_reflection(
            "the closure went clean and the estimate held.".to_owned(),
            ReflectionTrigger::Threshold,
            totals(28, 28),
            at("2026-09-25T10:05:00Z"),
        )
        .unwrap();
    let written = store.graph().clone();
    drop(store);

    // The line: the verbatim answer, the trigger, the totals — in
    // declaration order, one LF.
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    let text = String::from_utf8_lossy(&log);
    assert!(
        text.contains(
            "{\"kind\":\"reward_reflect\",\"ts\":\"2026-09-25T10:05:00.000000000Z\",\
             \"answer\":\"the closure went clean and the estimate held.\",\
             \"trigger\":{\"trigger\":\"threshold\"},\"root_total\":28,\"repo_total\":28}\n"
        ),
        "{text}"
    );

    // Replay: the graph is unchanged — the reflection mutates no state
    // — and the line survives the round trip.
    let reopened = open(&root);
    assert_eq!(reopened.graph(), &written);
    let line = text
        .lines()
        .find(|line| line.contains("\"kind\":\"reward_reflect\""))
        .unwrap();
    let event: Event = from_bytes(line.as_bytes()).unwrap();
    assert_eq!(
        to_bytes(&event).unwrap(),
        line.as_bytes(),
        "serialize → replay → serialize is identical (§8)"
    );
}

#[test]
fn both_trigger_variants_round_trip_through_the_codec() {
    // The epic-closure variant is not on the golden timeline; its bytes
    // are pinned here, beside the threshold variant's.
    let epic = ReflectionTrigger::EpicClosure { epic: epic_id(2) };
    let bytes = to_bytes(&epic).unwrap();
    assert_eq!(bytes, br#"{"trigger":"epic_closure","epic":"camera:E2"}"#);
    assert_eq!(from_bytes::<ReflectionTrigger>(&bytes).unwrap(), epic);

    let threshold = ReflectionTrigger::Threshold;
    let bytes = to_bytes(&threshold).unwrap();
    assert_eq!(bytes, br#"{"trigger":"threshold"}"#);
    assert_eq!(from_bytes::<ReflectionTrigger>(&bytes).unwrap(), threshold);

    // The event carrying the epic-closure trigger round-trips too.
    let event = Event::RewardReflect {
        ts: at("2026-09-25T11:00:00Z"),
        answer: "the milestone closed green.".to_owned(),
        trigger: epic,
        root_total: 500,
        repo_total: 120,
    };
    let bytes = to_bytes(&event).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        "{\"kind\":\"reward_reflect\",\"ts\":\"2026-09-25T11:00:00.000000000Z\",\
         \"answer\":\"the milestone closed green.\",\
         \"trigger\":{\"trigger\":\"epic_closure\",\"epic\":\"camera:E2\"},\
         \"root_total\":500,\"repo_total\":120}"
    );
    assert_eq!(from_bytes::<Event>(&bytes).unwrap(), event);
    assert_eq!(
        to_bytes(&from_bytes::<Event>(&bytes).unwrap()).unwrap(),
        bytes
    );
}

#[test]
fn the_reward_grant_event_round_trips_through_the_codec() {
    // The grant event's bytes, pinned beside the reflection's: the
    // fields are the domain's Reward spellings.
    let event = Event::RewardGrant {
        ts: at("2026-09-25T12:00:00Z"),
        task: task_id(7),
        amount: 98,
        size: Size::L,
        priority_pct: 150,
        kind_pct: 130,
    };
    let bytes = to_bytes(&event).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        "{\"kind\":\"reward_grant\",\"ts\":\"2026-09-25T12:00:00.000000000Z\",\
         \"task\":\"camera:U7\",\"amount\":98,\"size\":\"l\",\
         \"priority_pct\":150,\"kind_pct\":130}"
    );
    assert_eq!(from_bytes::<Event>(&bytes).unwrap(), event);
    assert_eq!(
        to_bytes(&from_bytes::<Event>(&bytes).unwrap()).unwrap(),
        bytes
    );
    // The single-source constructor's output is the golden timeline's
    // reward lines — the store's grant path builds every event through
    // it, so the bytes above and the golden's agree by construction.
}

#[test]
fn the_totals_are_recorded_verbatim_not_recomputed() {
    // The store records the totals it was handed — the caller's
    // derived numbers, whatever they are. A repo absent from the map
    // records zero: the caller's inconsistency is visible in the line,
    // not silently repaired.
    let dir = TempDir::new().unwrap();
    let (root, _task) = enabled_root_with_task(&dir);
    let mut store = open(&root);
    let other: todo_domain::RepoSlug = "other".parse().unwrap();
    store
        .log_reflection(
            "an answer".to_owned(),
            ReflectionTrigger::EpicClosure { epic: epic_id(1) },
            RewardsTotals {
                total: 1234,
                by_repo: [(other, 1234)].into_iter().collect(),
            },
            at("2026-09-25T10:05:00Z"),
        )
        .unwrap();
    let log = fs::read(root.log_path(&camera_slug())).unwrap();
    let text = String::from_utf8_lossy(&log);
    assert!(
        text.contains("\"root_total\":1234,\"repo_total\":0"),
        "{text}"
    );
}
