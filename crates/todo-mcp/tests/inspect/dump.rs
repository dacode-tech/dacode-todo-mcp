//! The dump's determinism instruments (spec §8, §10;
//! `.agents/knowledge/workflows/golden-files.md`): the moved golden
//! (`dump.v<N>`, the cli's `cli_dump.v<N>` under the host's per-surface
//! naming), twice-run byte identity, the parse-back round trip,
//! insertion-order robustness, and the no-host-path rule asserted on
//! the bytes rather than documented in prose.
//!
//! The fixture world is built through the real store API with fixed
//! timestamps and a fixed literal repo path
//! (`tests/inspect/fixtures.rs`); the end-to-end half spawns the built
//! binary and asserts its stdout is exactly the canonical document plus
//! one trailing LF.

use tempfile::TempDir;
use todo_mcp::inspect::dump::Dump;
use todo_store::{DataRoot, FORMAT_VERSION, Store, from_bytes};

use crate::common::assert_golden;
use crate::inspect::fixtures::{
    assert_no_debris, assert_no_tempdir_components, build_golden_store,
    build_golden_store_reordered, camera_slug, data_root, root_arg, run_cli,
};

/// The golden's subject name: `v1` tracks the store's `FORMAT_VERSION`,
/// since the dump mirrors the format's shapes — derived from the
/// constant, so a format bump renames the golden instead of silently
/// re-pinning the old one.
fn golden_name() -> String {
    format!("dump.v{}", FORMAT_VERSION)
}

/// The spawned binary's dump of a freshly built golden world, plus the
/// tempdir it was built in (the no-host-path assertion needs its path
/// components).
fn dumped_world(build: fn(&DataRoot) -> todo_store::JsonlStore) -> (TempDir, Vec<u8>) {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    drop(build(&root));
    let run = run_cli(&["dump", "--root", root_arg(&dir), "--repo", "camera"]);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stderr, "", "dump's stderr is silent on success");
    assert_no_debris(&data_root(&dir));
    (dir, run.stdout.into_bytes())
}

#[test]
fn the_dump_golden_pins_the_canonical_bytes() {
    let (dir, bytes) = dumped_world(build_golden_store);
    // The document is one canonical value plus exactly one trailing
    // LF — `> graph.json` and `| jq .` both work on these bytes.
    assert!(bytes.ends_with(b"}\n"), "one trailing LF: {bytes:?}");
    assert!(!bytes.ends_with(b"}\n\n"), "exactly one trailing LF");
    assert_golden(&golden_name(), &bytes);
    assert_no_tempdir_components(&bytes, &dir);
}

#[test]
fn twice_run_dump_is_byte_identical() {
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    drop(build_golden_store(&root));
    let args = ["dump", "--root", root_arg(&dir), "--repo", "camera"];
    let first = run_cli(&args);
    let second = run_cli(&args);
    assert_eq!(first.code, 0);
    assert_eq!(
        first.stdout, second.stdout,
        "two runs over unchanged state return byte-identical output (§8)"
    );
    assert_no_debris(&root);
}

#[test]
fn the_dump_parses_back_through_the_store_codec() {
    // The round trip goes through the store's public codec — the
    // engine stays sealed there (U14: no dacode-json dependency) —
    // into the inspector's own document type, with equal values.
    let dir = TempDir::new().unwrap();
    let root = data_root(&dir);
    drop(build_golden_store(&root));
    let run = run_cli(&["dump", "--root", root_arg(&dir), "--repo", "camera"]);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);

    let parsed: Dump = from_bytes(run.stdout.as_bytes()).unwrap();
    let store = root.open(&camera_slug()).unwrap();
    let in_memory = Dump::from_graph(store.graph());
    assert_eq!(parsed, in_memory, "the bytes are the graph");
    // And the lib target's own serialization is the binary's stdout,
    // byte for byte — the two halves cannot drift.
    assert_eq!(in_memory.to_line().unwrap(), run.stdout.as_bytes());
    assert_no_debris(&root);
}

#[test]
fn a_different_mutation_order_dumps_byte_identically() {
    // Insertion-order robustness, the domain's own test shape: the
    // same graph reached through a different mutation order is the
    // same bytes, because the dump's order comes from the graph's
    // total-order accessors and nothing else (§8).
    let (_, ordered) = dumped_world(build_golden_store);
    let (_, reordered) = dumped_world(build_golden_store_reordered);
    assert_eq!(ordered, reordered);
}

#[test]
fn the_dump_of_a_missing_store_is_the_store_fault() {
    // The dump classifies nothing: any `Err(StoreError)` is the
    // exit-code table's 3 with §6.1's code on its own line.
    let dir = TempDir::new().unwrap();
    let run = run_cli(&["dump", "--root", root_arg(&dir), "--repo", "camera"]);
    assert_eq!(run.code, 3);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("io\n"), "{:?}", run.stderr);
    assert!(
        run.stderr.contains("open the event log: NotFound"),
        "{:?}",
        run.stderr
    );
    assert_no_debris(&data_root(&dir));
}
