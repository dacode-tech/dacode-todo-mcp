//! The end-to-end suite (t45): the composed importer over the real
//! dacode board. The three sibling board files are checked in
//! verbatim as the fixture — the only sibling content this repository
//! holds (D12/§10) — and the expected graph is golden-pinned as the
//! canonical dump document, so a corrupted or drifted fixture is a
//! named failure rather than a silently wrong import.
//!
//! Two surfaces, one census. The golden comes from the in-process
//! pipeline — parse the fixture, map with a fixed import stamp and the
//! derived doc-path map, apply into a fresh store, dump the graph —
//! so every timestamp in it is the fixed constant or a board-fixed
//! dossier date. The wire test drives the real binary over an
//! assembled tree — the three fixture files at their dialect paths
//! plus one empty stub per remaining register key — and pins the
//! landed counts as exact literals: the same numbers the golden pins,
//! read off the graph the tool itself applied. The wire tool's own
//! clock makes its store un-pinnable byte-exactly; the counts are its
//! deterministic surface.
//!
//! The doc register's file mapping is derived once and consumed twice
//! — the golden's `doc_paths` and the wire tree's stub files — so the
//! map and the tree cannot drift apart. The dump DTO is a test-local
//! mirror of the cli's `dump` document, field-for-field: a
//! cross-crate test dependency on `todo-cli` is barred, so the shape
//! is copied (the `common/mod.rs` verbatim-copy discipline) and a
//! shape fix propagates to both. The fixture's identity is pinned by
//! SHA-1, computed in-test — no checksum dependency may be added and
//! a host tool would make the pin host-dependent — with the
//! implementation checked against the published `abc` vector first.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Serialize;
use serde_json::{Value, json};
use tempfile::TempDir;
use todo_domain::{Decision, Doc, Epic, Graph, Repo, RepoSlug, Task, TaskId, Timestamp};
use todo_mcp::import::dacode::{DacodeBoard, map_board, parse_board};
use todo_mcp::import::plan::{ImportOptions, apply_plan};
use todo_store::{BlocksEdgeDto, DataRoot, Store, to_bytes};

use crate::common::{assert_golden, call, drive, structured};

// =================================================================
// The fixture and the shared derivation
// =================================================================

/// The fixture directory, anchored at the crate manifest — never the
/// process cwd, so the tests pass from any invocation root.
fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("dacode")
}

/// The board directory, relative to the repo's registered checkout
/// path — the dialect's measured layout, mirrored from the tool's
/// constant so the tree the wire test assembles is the tree the tool
/// walks.
const BOARD_DIR: &str = "docs/implementation";

/// The three board files' dialect paths, relative to the board
/// directory — also the fixture's own layout, so assembling the tree
/// is a copy of the whole fixture directory.
const BOARD_FILES: [&str; 3] = [
    "TODO.md",
    "appendices/B-decisions.md",
    "appendices/A-design-deltas.md",
];

/// The fixed import stamp: the pin that makes the dump deterministic.
/// Every timestamp in the golden is this constant, a board-fixed
/// dossier date, or the fixture repo's registration — which shares the
/// constant, so no clock read reaches the golden.
const IMPORTED_AT: &str = "2026-10-06T00:00:00Z";

/// Reads one fixture board file.
fn read_fixture(relative: &str) -> String {
    fs::read_to_string(fixture_dir().join(relative)).unwrap()
}

/// Parses the fixture board — the real dacode board, the same three
/// files the wire tool reads.
fn parse_fixture() -> DacodeBoard {
    parse_board(
        &read_fixture("TODO.md"),
        &read_fixture("appendices/B-decisions.md"),
        &read_fixture("appendices/A-design-deltas.md"),
    )
    .unwrap()
}

/// The tool's stem rule, mirrored: a key matches a file whose stem is
/// the key, or the key plus a hyphen prefix.
fn stem_matches(file: &str, key: &str) -> bool {
    let name = file.rsplit_once('/').map_or(file, |(_, name)| name);
    let stem = name.strip_suffix(".md").unwrap_or(name);
    stem == key
        || stem
            .strip_prefix(key)
            .is_some_and(|rest| rest.starts_with('-'))
}

/// The shared derivation: each register key mapped to its
/// repo-relative path — the board file that matches it under the stem
/// rule, or its `{key}.md` stub. The golden's `doc_paths` and the
/// wire tree's stub files both come from here, so the map and the
/// tree cannot drift apart.
fn derived_doc_paths(board: &DacodeBoard) -> BTreeMap<String, String> {
    board
        .todo
        .docs
        .iter()
        .map(|entry| {
            let matched = BOARD_FILES
                .iter()
                .find(|file| stem_matches(file, &entry.key));
            let file =
                matched.map_or_else(|| format!("{}.md", entry.key), |file| (*file).to_owned());
            (entry.key.clone(), format!("{BOARD_DIR}/{file}"))
        })
        .collect()
}

/// A fresh registered store in a temp directory — the public
/// create/register path, the shape the import tool uses. The repo
/// node's registration shares the fixed stamp, so the dump's repo
/// field is deterministic too.
fn fresh_store() -> (TempDir, todo_store::JsonlStore) {
    let dir = TempDir::new().unwrap();
    let root = DataRoot::new(dir.path());
    let repo = Repo {
        slug: RepoSlug::from_str("dacode").unwrap(),
        name: "dacode".to_owned(),
        path: "/repos/dacode".to_owned(),
        registered_at: Timestamp::parse(IMPORTED_AT).unwrap(),
    };
    let store = root.register(repo).unwrap();
    (dir, store)
}

// =================================================================
// The dump DTO — the cli's shape, mirrored field-for-field
// =================================================================

/// The dump document: the imported graph as one canonical JSON value,
/// mirroring the cli's `dump` document field-for-field — the same
/// field order, the same node and edge spellings, the same counters
/// mirror — so the golden pins the bytes the inspector would write.
/// The mirror is a copy, not a dependency: a cross-crate test
/// dependency on `todo-cli` is barred, and a shape fix propagates to
/// both (the `common/mod.rs` verbatim-copy discipline).
#[derive(Serialize)]
struct Dump {
    /// The repo node, as registered by the fixture.
    repo: Repo,
    /// The allocation counters: the highest number issued per kind.
    counters: CountersDto,
    /// Every epic, ascending by id.
    epics: Vec<Epic>,
    /// Every task, ascending by id.
    tasks: Vec<Task>,
    /// Every decision, ascending by id.
    decisions: Vec<Decision>,
    /// Every doc, ascending by id.
    docs: Vec<Doc>,
    /// Every blocks edge, in the graph's documented total order.
    blocks: Vec<BlocksEdgeDto>,
    /// Every relates annotation, ascending canonical order, as the
    /// format's one positional shape.
    relates: Vec<(TaskId, TaskId)>,
}

/// The counters as four named fields — the mirror of the domain's
/// serde-free `Counters`, shaped like the store's own snapshot DTO.
#[derive(Serialize)]
struct CountersDto {
    /// The highest epic number issued.
    epic: u64,
    /// The highest task number issued.
    task: u64,
    /// The highest decision number issued.
    decision: u64,
    /// The highest doc number issued.
    doc: u64,
}

impl Dump {
    /// The document describing `graph`, built from the read accessors
    /// whose orders the graph documents — deterministic by
    /// construction, whatever mutation order produced the state.
    fn from_graph(graph: &Graph) -> Self {
        let counters = graph.counters();
        Self {
            repo: graph.repo().clone(),
            counters: CountersDto {
                epic: counters.epic(),
                task: counters.task(),
                decision: counters.decision(),
                doc: counters.doc(),
            },
            epics: graph.epics().cloned().collect(),
            tasks: graph.tasks().cloned().collect(),
            decisions: graph.decisions().cloned().collect(),
            docs: graph.docs().cloned().collect(),
            blocks: graph.blocks_edges().into_iter().map(Into::into).collect(),
            relates: graph.relates_pairs(),
        }
    }
}

// =================================================================
// 1. The golden: the composed pipeline over the real board.
// =================================================================

/// The composed pipeline over the three fixture files: parse, map
/// with the fixed stamp and the derived doc paths, apply into a fresh
/// store, dump, pin. The id reproduction, the status mapping, the
/// edges, the counters and the doc paths of the real board are all
/// pinned byte-for-byte in the golden — the canonical bytes plus one
/// trailing LF, the cli dump's own line discipline.
#[test]
fn the_real_board_dumps_to_the_expected_graph() {
    let board = parse_fixture();
    let plan = map_board(
        &board,
        &ImportOptions {
            slug: RepoSlug::from_str("dacode").unwrap(),
            imported_at: Timestamp::parse(IMPORTED_AT).unwrap(),
            doc_paths: derived_doc_paths(&board),
        },
    )
    .unwrap();
    let (_dir, mut store) = fresh_store();
    apply_plan(&plan, &mut store).unwrap();
    let dump = Dump::from_graph(store.graph());
    let mut bytes = to_bytes(&dump).unwrap();
    bytes.push(b'\n');
    assert_golden("dacode_dump.v1", &bytes);
}

// =================================================================
// 2. The fixture's identity: the pinned snapshot.
// =================================================================

/// The fixture is the pinned snapshot: the three files' SHA-1s and
/// the directory's recursive file count, so a fixture edit or an
/// accidental addition is a named failure, never a silent drift. The
/// digest is computed in-test (the module doc's reasoning) and pinned
/// against SHA-1's published `abc` vector before it is trusted with
/// the fixture's identity.
#[test]
fn the_fixture_files_are_the_pinned_snapshot() {
    assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    for (relative, sha) in [
        ("TODO.md", "53b5be82d9fa9cea88d13385b0e8a6ffaa41304d"),
        (
            "appendices/B-decisions.md",
            "2cfb59336bf683bc214231c2259c74cd37b8bd48",
        ),
        (
            "appendices/A-design-deltas.md",
            "a3ed564ffd2030cbf078f9a6f823b7166fa50ff0",
        ),
    ] {
        let bytes = fs::read(fixture_dir().join(relative)).unwrap();
        assert_eq!(sha1_hex(&bytes), sha, "{relative}");
    }
    assert_eq!(count_files(&fixture_dir()), 3);
}

/// Counts the files under `dir`, recursively.
fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            count += count_files(&entry.path());
        } else {
            count += 1;
        }
    }
    count
}

/// SHA-1 (FIPS 180-1) as lowercase hex — the fixture identity's
/// digest, computed in-test because no checksum dependency may be
/// added and a host tool would make the pin host-dependent. The
/// snapshot test pins the implementation against the published `abc`
/// vector before trusting it with the fixture.
fn sha1_hex(bytes: &[u8]) -> String {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let bit_len = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());
    for block in message.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, &word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}

// =================================================================
// 3. The wire composition: the real binary over the assembled tree.
// =================================================================

/// Assembles the wire test's board tree under a fresh temp directory:
/// the three fixture files copied to their dialect paths, plus one
/// empty stub per derived path that is not a board file — the
/// derivation's own stub arm, so the tree and the golden's doc-path
/// map share one source. The stubs' content is never read; only their
/// stems match. Returns the tree's root — the repo's registered
/// checkout path.
fn fixture_tree(board: &DacodeBoard) -> TempDir {
    let root = TempDir::new().unwrap();
    let dir = root.path().join(BOARD_DIR);
    for relative in BOARD_FILES {
        let path = dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, read_fixture(relative)).unwrap();
    }
    for path in derived_doc_paths(board).values() {
        let relative = path
            .strip_prefix(&format!("{BOARD_DIR}/"))
            .expect("a derived path is board-relative");
        // A key the board files already match needs no stub — its
        // derived path IS one of the copied board files, and writing
        // an empty stub there would overwrite the fixture copy (the
        // parse would then refuse the emptied appendix).
        if !BOARD_FILES.contains(&relative) {
            fs::write(dir.join(relative), "").unwrap();
        }
    }
    root
}

/// A `repo_register` request's arguments for `slug` with its checkout
/// at `tree`.
fn register_args(slug: &str, tree: &Path) -> String {
    json!({
        "slug": slug,
        "name": slug,
        "path": tree.display().to_string(),
    })
    .to_string()
}

/// The real binary imports the assembled board tree and answers the
/// landed census: `repo_register` (the tree as the checkout) →
/// `import_md` → `graph_show`. The counts response's exact literals
/// are the observed census — the same numbers the dump golden pins,
/// so the tool and the pipeline are cross-checked against one answer
/// — and `graph_show`'s per-kind counts, `U1` and the doc paths are
/// asserted structurally. Not golden-pinned: the store's stamps carry
/// the tool's clock, and the counts are the deterministic surface.
#[test]
fn the_real_board_imports_through_the_wire_tool() {
    let root = TempDir::new().unwrap();
    let board = parse_fixture();
    let tree = fixture_tree(&board);
    let requests = vec![
        call(2, "repo_register", &register_args("dacode", tree.path())),
        call(3, "import_md", r#"{"repo":"dacode"}"#),
        call(4, "graph_show", r#"{"repo":"dacode"}"#),
    ];
    let driven = drive(root.path(), &requests);
    assert!(
        driven.status.success(),
        "a completed handshake ends 0; stderr: {}",
        driven.stderr
    );
    assert_eq!(driven.responses.len(), 3);
    // The premise: the register and the import both succeeded.
    let registered: Value = serde_json::from_str(&driven.responses[0]).unwrap();
    assert_eq!(registered["result"]["isError"], json!(false));
    let imported: Value = serde_json::from_str(&driven.responses[1]).unwrap();
    assert_eq!(
        imported["result"]["isError"],
        json!(false),
        "{}",
        driven.responses[1]
    );

    // The counts response's exact literals — the observed census.
    let answer = structured(&driven.responses[1]);
    assert_eq!(answer["repo"], json!("dacode"));
    assert_eq!(answer["epics"], json!(9));
    assert_eq!(answer["decisions"], json!(21));
    assert_eq!(answer["docs"], json!(42));
    assert_eq!(answer["tasks"], json!(218));
    assert_eq!(answer["blocks"], json!(43));

    // graph_show's per-kind counts — the same census, read off the
    // landed graph.
    let graph = structured(&driven.responses[2]);
    let repo = &graph["repos"][0];
    assert_eq!(repo["epics"].as_array().unwrap().len(), 9);
    assert_eq!(repo["decisions"].as_array().unwrap().len(), 21);
    assert_eq!(repo["docs"].as_array().unwrap().len(), 42);
    let tasks: Vec<&Value> = repo["epics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["tasks"].as_array().unwrap())
        .collect();
    assert_eq!(tasks.len(), 218);

    // U1 present — the id reproduction visible on the real board: the
    // store's first task id is the board's first follow-up.
    let u1 = tasks
        .iter()
        .find(|task| task["id"] == json!("dacode:U1"))
        .unwrap();
    assert_eq!(u1["title"], json!("Size-table re-measure"));

    // The doc paths repo-relative — exactly the derived map, the
    // board files' prefix arm and the stubs' exact arm both visible.
    let paths: BTreeMap<String, String> = repo["docs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| {
            (
                doc["title"].as_str().unwrap().to_owned(),
                doc["path"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(paths, derived_doc_paths(&board));
}
