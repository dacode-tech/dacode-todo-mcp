//! §4.3's crash and corruption semantics: a torn last line, a malformed
//! interior line, header faults and a log that contradicts itself are all
//! named `corrupt_store` errors carrying the offending line's absolute
//! byte offset — never a panic, never a silently skipped line.

use std::fs;

use tempfile::TempDir;
use todo_domain::Status;
use todo_store::{
    BlocksEdgeDto, Event, FORMAT_VERSION, JsonlStore, Store, StoreError, TaskPatch, to_bytes,
};

use super::fixtures::{
    at, epic_draft, expect_corrupt, line_offsets, lines, lines_of_kind, log_path, repo, short_log,
    task_draft, task_id, without_ts, write_fixture,
};

// =================================================================
// §4.3: corruption is a named error carrying its byte offset.
// =================================================================

#[test]
fn a_torn_last_line_is_a_named_error_even_when_its_bytes_parse() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    short_log(&path);
    let valid = fs::read(&path).unwrap();
    let offsets = line_offsets(&valid);
    // The last line's start: a terminated file's final LF is its EOF, so
    // the offset comes from the split, not from a search for the newline.
    let start = offsets[offsets.len() - 1];

    // The final LF stripped: the remaining bytes still parse, and are
    // still not a committed event.
    let stripped = write_fixture(&dir, "torn-stripped.jsonl", &valid[..valid.len() - 1]);
    expect_corrupt(&stripped, start, "no terminating newline");

    // The last line cut mid-byte.
    let cut = write_fixture(&dir, "torn-cut.jsonl", &valid[..valid.len() - 6]);
    expect_corrupt(&cut, start, "no terminating newline");

    // The file cut inside an interior line: what is left of it is now
    // the unterminated tail, and the offset is that line's start.
    let interior = offsets[2] + 4;
    let truncated = write_fixture(&dir, "torn-interior.jsonl", &valid[..interior]);
    expect_corrupt(&truncated, offsets[2], "no terminating newline");
}

#[test]
fn a_malformed_interior_line_is_corrupt_unconditionally() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    short_log(&path);
    let valid = fs::read(&path).unwrap();
    let log_lines = lines(&valid);
    let offsets = line_offsets(&valid);
    // Line 2 is the first event after the seed: an epic_add.
    assert!(String::from_utf8_lossy(log_lines[2]).contains("epic_add"));

    let garbage: &[u8] = b"{{{ not json\n";
    let unknown_kind = String::from_utf8_lossy(log_lines[2])
        .replace("\"kind\":\"epic_add\"", "\"kind\":\"epic_added\"")
        .into_bytes();
    let missing_field = without_ts(log_lines[2]);

    for (name, replacement, needle) in [
        ("garbage", garbage.to_vec(), "not a parseable event"),
        ("unknown-kind", unknown_kind, "not a parseable event"),
        ("missing-field", missing_field, "not a parseable event"),
    ] {
        let mut spliced = Vec::new();
        for (index, line) in log_lines.iter().enumerate() {
            if index == 2 {
                spliced.extend_from_slice(&replacement);
            } else {
                spliced.extend_from_slice(line);
            }
        }
        // An interior line does not tear in a crash (§4.3), so this is
        // corruption wherever it sits — never a skipped line.
        let fixture = write_fixture(&dir, &format!("interior-{name}.jsonl"), &spliced);
        expect_corrupt(&fixture, offsets[2], needle);
    }
}

#[test]
fn header_faults_are_named_errors() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    short_log(&path);
    let valid = fs::read(&path).unwrap();
    let log_lines = lines(&valid);
    let offsets = line_offsets(&valid);

    // A newer format version is a migration question (§4), not
    // corruption, and is reported before anything is replayed.
    let mut newer = br#"{"format_version":2}"#.to_vec();
    newer.push(b'\n');
    for line in &log_lines[1..] {
        newer.extend_from_slice(line);
    }
    let fixture = write_fixture(&dir, "version-2.jsonl", &newer);
    let error = JsonlStore::open_log(&fixture).unwrap_err();
    assert_eq!(
        error,
        StoreError::VersionMismatch {
            found: 2,
            expected: FORMAT_VERSION
        }
    );
    assert_eq!(error.code(), "version_mismatch");
    assert!(error.to_string().contains("version 2"), "{error}");

    // An unknown field at the *current* version is damage, not a
    // migration: the strict `LogHeader` parse rejects it (f14's
    // `deny_unknown_fields`, on the registry/snapshot precedent), at
    // offset 0 with the bad-header detail. Hand-written header, spliced
    // real events — the block above's shape, and the header's own
    // precedent: no writer can produce one.
    let mut extra = br#"{"format_version":1,"extra":true}"#.to_vec();
    extra.push(b'\n');
    for line in &log_lines[1..] {
        extra.extend_from_slice(line);
    }
    let fixture = write_fixture(&dir, "version-1-extra.jsonl", &extra);
    expect_corrupt(&fixture, 0, "not a format-version header");

    // The same unknown field at a *newer* version stays the migration
    // question: the lenient probe reads the version before the strict
    // parse ever sees the field (D1's probe-first ordering — without it
    // this fixture reports `corrupt_store`, which is what the ordering
    // exists to prevent).
    let mut newer_extra = br#"{"format_version":2,"extra":true}"#.to_vec();
    newer_extra.push(b'\n');
    for line in &log_lines[1..] {
        newer_extra.extend_from_slice(line);
    }
    let fixture = write_fixture(&dir, "version-2-extra.jsonl", &newer_extra);
    let error = JsonlStore::open_log(&fixture).unwrap_err();
    assert_eq!(
        error,
        StoreError::VersionMismatch {
            found: 2,
            expected: FORMAT_VERSION
        }
    );
    assert_eq!(error.code(), "version_mismatch");

    // An unparseable header is corrupt at offset 0.
    let fixture = write_fixture(&dir, "bad-header.jsonl", b"not a header\n");
    expect_corrupt(&fixture, 0, "not a format-version header");

    // An empty file is corrupt at offset 0, not an empty store.
    let fixture = write_fixture(&dir, "empty.jsonl", b"");
    expect_corrupt(&fixture, 0, "the log is empty");

    // A header with no events: the seed is missing.
    let fixture = write_fixture(&dir, "header-only.jsonl", log_lines[0]);
    expect_corrupt(&fixture, 0, "holds no events");

    // A header whose line was never committed: torn, at offset 0.
    let uncommitted = &log_lines[0][..log_lines[0].len() - 1];
    let fixture = write_fixture(&dir, "torn-header.jsonl", uncommitted);
    expect_corrupt(&fixture, 0, "no terminating newline");

    // A log whose first event is not the seed.
    let mut unseeded = Vec::new();
    unseeded.extend_from_slice(log_lines[0]);
    for line in &log_lines[2..] {
        unseeded.extend_from_slice(line);
    }
    let fixture = write_fixture(&dir, "unseeded.jsonl", &unseeded);
    expect_corrupt(
        &fixture,
        offsets[1],
        "first event must be repo_register, found epic_add",
    );
}

#[test]
fn replay_rejects_a_burned_number() {
    // An epic-delete cascade burns camera:U1 (§3: counters only move up),
    // and the domain's explicit-id insert would happily re-insert it — the
    // one never-reuse surface the domain leaves open. A log this crate
    // wrote cannot contain that line, so replay refuses it.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    let mut store = JsonlStore::create_log(&path, repo()).unwrap();
    let epic = store
        .add_epic(epic_draft("one"), at("2026-09-25T08:05:00Z"))
        .unwrap();
    let task = store
        .add_task(task_draft(&epic.id, "one"), at("2026-09-25T09:00:00Z"))
        .unwrap();
    store
        .update_task(
            task.id.clone(),
            at("2026-09-25T09:30:00Z"),
            TaskPatch {
                status: Some(Status::Closed),
                evidence: Some(Some("ran it".to_owned())),
                ..TaskPatch::default()
            },
        )
        .unwrap();
    assert_eq!(
        store
            .delete_epic(epic.id, at("2026-09-25T10:00:00Z"))
            .unwrap(),
        vec![task.id.clone()]
    );
    drop(store);

    let valid = fs::read(&path).unwrap();
    let adds = lines_of_kind(&lines(&valid), "task_add");
    assert_eq!(adds.len(), 1);
    // Spliced from the log's own bytes: a real event line, re-appended
    // after the cascade that burned its number.
    let mut spliced = valid.clone();
    spliced.extend_from_slice(adds[0]);
    let fixture = write_fixture(&dir, "burned-number.jsonl", &spliced);
    let error = expect_corrupt(&fixture, valid.len(), "camera:U1");
    assert!(
        error.to_string().contains("next number of its kind is 2"),
        "{error}"
    );
}

#[test]
fn replay_rejects_a_skipped_number() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    short_log(&path);
    let valid = fs::read(&path).unwrap();
    let log_lines = lines(&valid);
    let offsets = line_offsets(&valid);
    let adds: Vec<usize> = (0..log_lines.len())
        .filter(|index| {
            String::from_utf8_lossy(log_lines[*index]).contains("\"kind\":\"task_add\"")
        })
        .collect();
    assert_eq!(adds.len(), 2);

    // The two add lines swapped: camera:U2 is replayed against a
    // counter that has issued nothing, so it skips allocation.
    let mut spliced = Vec::new();
    for (index, line) in log_lines.iter().enumerate() {
        let replacement = if index == adds[0] {
            Some(log_lines[adds[1]])
        } else if index == adds[1] {
            Some(log_lines[adds[0]])
        } else {
            None
        };
        spliced.extend_from_slice(replacement.unwrap_or(line));
    }
    let fixture = write_fixture(&dir, "skipped-number.jsonl", &spliced);
    let error = expect_corrupt(&fixture, offsets[adds[0]], "camera:U2");
    assert!(
        error.to_string().contains("next number of its kind is 1"),
        "{error}"
    );
}

#[test]
fn replay_rejects_a_log_the_domain_contradicts() {
    // A `blocks_link` that closes a cycle: the domain rejected it when
    // it was written, so a log holding one contradicts itself.
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    short_log(&path);
    let valid = fs::read(&path).unwrap();

    let event = Event::BlocksLink {
        ts: at("2026-09-25T11:00:00Z"),
        edge: BlocksEdgeDto::TaskToTask {
            from: task_id(2),
            to: task_id(1),
        },
    };
    let mut spliced = valid.clone();
    spliced.extend_from_slice(&to_bytes(&event).unwrap());
    spliced.push(b'\n');
    let fixture = write_fixture(&dir, "cycle.jsonl", &spliced);

    let error = expect_corrupt(&fixture, valid.len(), "the log contradicts itself");
    // The domain's own message rides in the detail, so a repair tool
    // can say why the line is wrong.
    assert!(error.to_string().contains("cycle"), "{error}");
    assert!(
        error
            .to_string()
            .contains("camera:U2 -> camera:U1 -> camera:U2"),
        "{error}"
    );
}

#[test]
fn a_second_seed_is_a_named_error() {
    let dir = TempDir::new().unwrap();
    let path = log_path(&dir);
    short_log(&path);
    let valid = fs::read(&path).unwrap();
    let seeds = lines_of_kind(&lines(&valid), "repo_register");
    assert_eq!(seeds.len(), 1);

    let mut spliced = valid.clone();
    spliced.extend_from_slice(seeds[0]);
    let fixture = write_fixture(&dir, "second-seed.jsonl", &spliced);
    expect_corrupt(&fixture, valid.len(), "registers camera twice");
}
