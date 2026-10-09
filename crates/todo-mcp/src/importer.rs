//! The importer cluster (§9, D12): the `import_md` wire tool — the
//! one-shot sibling-board importer, the file-reading half of the
//! import pipeline over the pure dialect modules. The near-name
//! collision is deliberate and worth stating: [`crate::import`] is the
//! pure dialect module (parse, map, apply — no I/O, no clock, no
//! store), while this cluster is the wire surface that drives it; the
//! tool reads files, the dialects never do.
//!
//! The `dialect` parameter selects which sibling board grammar and
//! file set the tool reads: `dacode` (the default — the absent
//! parameter keeps every earlier request's meaning), `camera` or
//! `ikvm-html`, the two live-test subjects, or `dacode-todo-mcp`,
//! this server's own board (the U28 dogfooding cutover — the same
//! two-file layout as camera's, its own measured grammar). An
//! unknown value is §6.1's `invalid` naming it. The board's location
//! is not a parameter for any dialect: the tool resolves the store
//! handle, reads the repo node's registered checkout path (§7's
//! mutable attribute — one source of truth, so `repo_set_path` moves
//! the import with the checkout) and derives the board directory as
//! `<checkout>/docs/implementation` — every dialect's measured
//! filesystem layout, part of the dialect like its grammar. The
//! board filenames are constants per dialect for the same reason.
//!
//! The dacode path walks the board directory to build the doc
//! register's file paths: one recursive walk collects `.md` files,
//! sorted by walk-relative path bytes before matching (`read_dir`
//! order is unspecified and determinism is the product property, §8),
//! and a register key matches a file iff the file's stem is the key
//! or the key plus a hyphen prefix. The map's values are
//! repo-relative, composed from the board-directory constant — never
//! by stripping a prefix off a canonicalized path. Unmatched files
//! are ignored (the register governs; `TODO.md` is legitimately
//! unregistered); a key with no match is `map_board`'s
//! `MissingDocPath` refusal, not this layer's (one home); a key
//! matching more than one file is this layer's `invalid` refusal — a
//! tree contradiction is refused, never first-wins. The camera and
//! ikvm-html paths read their two board files and map with an empty
//! `doc_paths` — neither board has a doc register.
//!
//! Faults carry §6.1 codes through D15's carrier: parse and map
//! refusals — and the apply refusal, via [`ApplyError::Refused`] —
//! as `invalid` (a client-fixable precondition, not corruption);
//! store faults as the store's own codes; file-read faults as `io`
//! in the store's attempt-plus-`ErrorKind` convention, naming the
//! file's role, never a path.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use todo_domain::{Graph, RepoSlug};
use todo_store::StoreError;

use crate::import::camera::{CameraError, map_camera, parse_camera_board};
use crate::import::dacode::{DocEntry, ImportError, map_board, parse_board};
use crate::import::ikvm::{IkvmError, map_ikvm, parse_ikvm_board};
use crate::import::own::{OwnError, map_own, parse_own_board};
use crate::import::plan::{ApplyError, ImportOptions, ImportPlan, apply_plan};
use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

/// The board directory, relative to the repo's registered checkout
/// path — every dialect's measured filesystem layout.
const BOARD_DIR: &str = "docs/implementation";
/// The TODO.md board file, relative to the board directory — every
/// dialect's first file.
const TODO_MD: &str = "TODO.md";
/// The dacode B decisions appendix, relative to the board directory.
const B_DECISIONS: &str = "appendices/B-decisions.md";
/// The dacode A design-deltas appendix, relative to the board directory.
const A_DELTAS: &str = "appendices/A-design-deltas.md";
/// The sibling decisions file (camera and ikvm-html), relative to the
/// board directory.
const DECISIONS_MD: &str = "decisions.md";

/// `import_md`'s parameters (§6): the repo whose board is imported,
/// and which sibling dialect's grammar reads it. The board's location
/// is not a parameter: it derives from the repo node's registered
/// checkout path (§7's mutable attribute) plus the dialect's measured
/// layout, so `repo_set_path` moves the import with the checkout and
/// the client never repeats what the registry already holds. The slug
/// arrives as a plain `String` for the same reason every
/// repo-scoped tool's does: a malformed slug is §6.1's `invalid`
/// through the carrier, not a codeless rmcp argument fault.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportMdParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose board is imported; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The board dialect to import: `dacode` (the default when absent), `camera`, `ikvm-html` or `dacode-todo-mcp` (this server's own board); any other value is `invalid` naming it.
    pub dialect: Option<String>,
}

/// The board dialect `import_md` reads: which sibling grammar and
/// file set the import runs over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialect {
    /// The dacode dialect: TODO.md plus the B and A appendices, the
    /// doc register walked from the board directory.
    Dacode,
    /// The camera dialect: TODO.md plus decisions.md, no doc register.
    Camera,
    /// The ikvm-html dialect: TODO.md plus decisions.md, no doc
    /// register — the same two files as camera's.
    IkvmHtml,
    /// The own dialect (`dacode-todo-mcp`): this server's own board —
    /// TODO.md plus decisions.md, the same two files as camera's, its
    /// own measured grammar.
    Own,
}

impl Dialect {
    /// The dialect a parameter value names: absent means `dacode`
    /// (the signed ordering — the first dialect keeps the default, so
    /// every request written before the parameter exists keeps its
    /// meaning); an unknown value is `invalid` naming it.
    pub(crate) fn from_param(value: Option<&str>) -> Result<Self, ToolError> {
        match value {
            None | Some("dacode") => Ok(Self::Dacode),
            Some("camera") => Ok(Self::Camera),
            Some("ikvm-html") => Ok(Self::IkvmHtml),
            Some("dacode-todo-mcp") => Ok(Self::Own),
            Some(other) => Err(ToolError::invalid(&format!(
                "the dialect '{other}' is not known: 'dacode', 'camera', 'ikvm-html' or 'dacode-todo-mcp'"
            ))),
        }
    }
}

/// `import_md`'s answer: the landed counts per node population and blocks edges, read off the graph after the import applied — the new state as a summary.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImportedDto {
    /// The repository's stable slug key — the store the board was imported into.
    pub repo: String,
    /// The epic count — the board's milestones plus the synthetic `Follow-ups` and `Spec deltas` epics.
    pub epics: u64,
    /// The decision count — the B appendix's dossiers.
    pub decisions: u64,
    /// The doc count — the §4 register's keys.
    pub docs: u64,
    /// The task count — the U items, sub-steps, A dossiers, B1a and the findings.
    pub tasks: u64,
    /// The blocks-edge count — the mechanical dependency edges the mapping resolved.
    pub blocks: u64,
}

/// The one error-mapping home for the importer's own refusals: every
/// [`ImportError`] — a parse failure, a map failure, or the apply
/// refusal carried by [`ApplyError::Refused`] — is §6.1's `invalid`
/// (U9: the board text is a value this server parses). The `Display`
/// passes through unwrapped: its details carry the file context where
/// it matters, the message is never the contract, and wrapping the
/// three parsers individually would bypass the composition's
/// cross-checks.
impl From<ImportError> for ToolError {
    fn from(error: ImportError) -> Self {
        Self::invalid(&error.to_string())
    }
}

/// The camera error-mapping home, the dacode one's twin: every
/// [`CameraError`] — a parse failure or a map failure — is §6.1's
/// `invalid` (U9: the board text is a value this server parses), its
/// `Display` passing through unwrapped for the same reasons.
impl From<CameraError> for ToolError {
    fn from(error: CameraError) -> Self {
        Self::invalid(&error.to_string())
    }
}

/// The ikvm-html error-mapping home, the dacode one's twin: every
/// [`IkvmError`] — a parse failure or a map failure — is §6.1's
/// `invalid` (U9: the board text is a value this server parses), its
/// `Display` passing through unwrapped for the same reasons.
impl From<IkvmError> for ToolError {
    fn from(error: IkvmError) -> Self {
        Self::invalid(&error.to_string())
    }
}

/// The own-dialect error-mapping home, the dacode one's twin: every
/// [`OwnError`] — a parse failure or a map failure — is §6.1's
/// `invalid` (U9: the board text is a value this server parses), its
/// `Display` passing through unwrapped for the same reasons.
impl From<OwnError> for ToolError {
    fn from(error: OwnError) -> Self {
        Self::invalid(&error.to_string())
    }
}

// =================================================================
// The answer helper
// =================================================================

/// `import_md`'s answer: resolve, read, parse, walk, match, map,
/// apply, count. The counts read off the graph after apply — the
/// landed state, not the plan's prediction — and the one clock read
/// (`time::now()`) becomes `imported_at`, stamped on every op the
/// board carries no date for (the one-read-per-mutation precedent).
fn answer_import_md(
    state: &mut ServerState,
    params: &ImportMdParams,
) -> Result<ImportedDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    let handle = state.resolve(&slug)?;
    let checkout = handle.graph().repo().path.clone();
    let dialect = Dialect::from_param(params.dialect.as_deref())?;
    let plan = plan_for(&checkout, &slug, dialect)?;
    match apply_plan(&plan, handle.as_mut()) {
        Ok(()) => {}
        Err(ApplyError::Refused(error)) => return Err(ToolError::from(error)),
        Err(ApplyError::Store(error)) => return Err(ToolError::from(error)),
    }
    Ok(imported_counts(handle.graph(), &slug))
}

/// The import's read+parse+map core over one checkout — the shared
/// half of the `import_md` tool and the `refresh` subcommand (D20):
/// the board directory derives from the repo node's registered
/// checkout path (the dialect's measured layout), the files are read,
/// parsed and mapped into the plan the caller applies. Pure reads —
/// building the plan touches no store, which is what lets the refresh
/// refuse a gone or broken board before its surgery destroys
/// anything.
pub(crate) fn plan_for(
    checkout: &str,
    slug: &RepoSlug,
    dialect: Dialect,
) -> Result<ImportPlan, ToolError> {
    let board_dir = Path::new(checkout).join(BOARD_DIR);
    match dialect {
        Dialect::Dacode => dacode_plan(&board_dir, slug),
        Dialect::Camera => camera_plan(&board_dir, slug),
        Dialect::IkvmHtml => ikvm_plan(&board_dir, slug),
        Dialect::Own => own_plan(&board_dir, slug),
    }
}

/// Whether `checkout` holds a board to import: the board directory
/// with `TODO.md`, every dialect's first file. The refresh's
/// gone-board pre-flight — a cut-over repo (its board archived, the
/// store primary) answers false and is refused before anything is
/// touched.
pub(crate) fn board_present(checkout: &str) -> bool {
    Path::new(checkout).join(BOARD_DIR).join(TODO_MD).exists()
}

/// The census off the graph after the apply — the landed state, not
/// the plan's prediction. The shared half of the tool's answer and
/// the refresh's report.
pub(crate) fn imported_counts(graph: &Graph, slug: &RepoSlug) -> ImportedDto {
    ImportedDto {
        repo: slug.to_string(),
        epics: graph.epics().count() as u64,
        decisions: graph.decisions().count() as u64,
        docs: graph.docs().count() as u64,
        tasks: graph.tasks().count() as u64,
        blocks: graph.blocks_edges().len() as u64,
    }
}

/// The dacode path's plan: read the three board files, parse, walk
/// the board directory for the doc register's paths, map.
fn dacode_plan(board_dir: &Path, slug: &RepoSlug) -> Result<ImportPlan, ToolError> {
    let todo_md = read_board_file(board_dir, TODO_MD)?;
    let b_decisions = read_board_file(board_dir, B_DECISIONS)?;
    let a_deltas = read_board_file(board_dir, A_DELTAS)?;
    let board = parse_board(&todo_md, &b_decisions, &a_deltas)?;
    let mut files = Vec::new();
    walk_markdown(board_dir, "", &mut files)?;
    let doc_paths = doc_paths(&board.todo.docs, files)?;
    let opts = ImportOptions {
        slug: slug.clone(),
        imported_at: time::now(),
        doc_paths,
    };
    Ok(map_board(&board, &opts)?)
}

/// The camera path's plan: read the two board files, parse, map with
/// an empty `doc_paths` — camera has no doc register, so the walk is
/// dacode's alone.
fn camera_plan(board_dir: &Path, slug: &RepoSlug) -> Result<ImportPlan, ToolError> {
    let todo_md = read_sibling_file(board_dir, TODO_MD, "board file TODO.md", "camera")?;
    let decisions = read_sibling_file(
        board_dir,
        DECISIONS_MD,
        "decisions file decisions.md",
        "camera",
    )?;
    let board = parse_camera_board(&todo_md, &decisions)?;
    let opts = ImportOptions {
        slug: slug.clone(),
        imported_at: time::now(),
        doc_paths: BTreeMap::new(),
    };
    Ok(map_camera(&board, &opts)?)
}

/// The ikvm-html path's plan: read the two board files — the same two
/// filenames as camera's, the dialect's measured layout — parse, map
/// with an empty `doc_paths` (no doc register here either).
fn ikvm_plan(board_dir: &Path, slug: &RepoSlug) -> Result<ImportPlan, ToolError> {
    let todo_md = read_sibling_file(board_dir, TODO_MD, "board file TODO.md", "ikvm")?;
    let decisions = read_sibling_file(
        board_dir,
        DECISIONS_MD,
        "decisions file decisions.md",
        "ikvm",
    )?;
    let board = parse_ikvm_board(&todo_md, &decisions)?;
    let opts = ImportOptions {
        slug: slug.clone(),
        imported_at: time::now(),
        doc_paths: BTreeMap::new(),
    };
    Ok(map_ikvm(&board, &opts)?)
}

/// The own path's plan: read the two board files — the same two
/// filenames as camera's and ikvm-html's, the dialect's measured
/// layout — parse, map with an empty `doc_paths` (no doc register
/// here either).
fn own_plan(board_dir: &Path, slug: &RepoSlug) -> Result<ImportPlan, ToolError> {
    let todo_md = read_sibling_file(board_dir, TODO_MD, "board file TODO.md", "own")?;
    let decisions = read_sibling_file(
        board_dir,
        DECISIONS_MD,
        "decisions file decisions.md",
        "own",
    )?;
    let board = parse_own_board(&todo_md, &decisions)?;
    let opts = ImportOptions {
        slug: slug.clone(),
        imported_at: time::now(),
        doc_paths: BTreeMap::new(),
    };
    Ok(map_own(&board, &opts)?)
}

// =================================================================
// The file-reading half
// =================================================================

/// Reads one board file, naming its role in an `io` fault — the
/// store's attempt-plus-`ErrorKind` convention, never a path: the
/// client passed the checkout path, and echoing server-side path
/// derivations is what the path-free discipline bars.
fn read_board_file(board_dir: &Path, relative: &str) -> Result<String, ToolError> {
    fs::read_to_string(board_dir.join(relative)).map_err(|error| {
        ToolError::from(StoreError::Io {
            detail: format!("read the dacode board file {relative}: {:?}", error.kind()),
        })
    })
}

/// Reads one sibling board file, naming its role in an `io` fault —
/// the store's attempt-plus-`ErrorKind` convention, never a path: the
/// client passed the checkout path, and echoing server-side path
/// derivations is what the path-free discipline bars. The dialect
/// names the board family, so the message reads "read the camera
/// board file TODO.md" / "read the ikvm decisions file decisions.md".
fn read_sibling_file(
    board_dir: &Path,
    relative: &str,
    role: &str,
    dialect: &str,
) -> Result<String, ToolError> {
    fs::read_to_string(board_dir.join(relative)).map_err(|error| {
        ToolError::from(StoreError::Io {
            detail: format!("read the {dialect} {role}: {:?}", error.kind()),
        })
    })
}

/// Walks the board directory recursively, collecting every `.md`
/// file's walk-relative path (`/`-separated) into `files` — unsorted;
/// the matcher sorts (determinism is its job), so the order here is
/// `read_dir`'s and never reaches a client. Non-`.md` files are
/// ignored: the dialect's docs are markdown, and the register
/// governs. Any read failure is the walk's own `io` fault, whatever
/// depth it fired at.
fn walk_markdown(dir: &Path, relative: &str, files: &mut Vec<String>) -> Result<(), ToolError> {
    let entries = fs::read_dir(dir).map_err(|error| walk_fault(error.kind()))?;
    for entry in entries {
        let entry = entry.map_err(|error| walk_fault(error.kind()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| walk_fault(error.kind()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let child = if relative.is_empty() {
            name.clone()
        } else {
            format!("{relative}/{name}")
        };
        if file_type.is_dir() {
            walk_markdown(&dir.join(&name), &child, files)?;
        } else if name.ends_with(".md") {
            files.push(child);
        }
    }
    Ok(())
}

/// The walk's `io` fault: the attempt and the `ErrorKind`, never a
/// path.
fn walk_fault(kind: ErrorKind) -> ToolError {
    ToolError::from(StoreError::Io {
        detail: format!("walk the dacode board directory: {kind:?}"),
    })
}

/// Builds the doc register's path map from the walk's raw collection:
/// the files are sorted by walk-relative path bytes first —
/// `read_dir` order is unspecified and determinism is the product
/// property (§8), so the sort is load-bearing, not cosmetic: the
/// collision refusal names its paths in this order — then each
/// register key (in register order — the parser has already refused
/// duplicates and contradictions) is matched against the sorted list
/// by the stem rule. A key with no match is left absent —
/// `map_board`'s `MissingDocPath` refuses the board naming the key,
/// one home for that refusal. A key matching more than one file is
/// this layer's `invalid` refusal naming the key and the first two
/// repo-relative paths — the files are byte-sorted, so the pair is
/// too.
fn doc_paths(
    docs: &[DocEntry],
    mut files: Vec<String>,
) -> Result<BTreeMap<String, String>, ToolError> {
    files.sort();
    let mut paths = BTreeMap::new();
    for entry in docs {
        let key = entry.key.as_str();
        let matches: Vec<&str> = files
            .iter()
            .map(String::as_str)
            .filter(|file| stem_matches(file, key))
            .collect();
        match matches.as_slice() {
            [] => {}
            [one] => {
                paths.insert(key.to_owned(), format!("{BOARD_DIR}/{one}"));
            }
            [first, second, ..] => {
                return Err(ToolError::invalid(&format!(
                    "the doc key '{key}' matches more than one file: {BOARD_DIR}/{first} and {BOARD_DIR}/{second}"
                )));
            }
        }
    }
    Ok(paths)
}

/// Whether a walked file matches the register key: the file's stem is
/// the key, or the key plus a hyphen prefix — the measured dialect
/// layout (`README`/`00-overview` need the exact arm, `01-*`/`A-*`
/// the prefix arm).
fn stem_matches(file: &str, key: &str) -> bool {
    let name = file.rsplit_once('/').map_or(file, |(_, name)| name);
    let stem = name.strip_suffix(".md").unwrap_or(name);
    stem == key
        || stem
            .strip_prefix(key)
            .is_some_and(|rest| rest.starts_with('-'))
}

// =================================================================
// The tool
// =================================================================

#[tool_router(router = import_router, vis = "pub(crate)")]
impl TodoServer {
    /// One-shot import of the repo's sibling markdown board — the dialect parameter selects the grammar (dacode by default, camera and ikvm-html as the live-test subjects, dacode-todo-mcp for this server's own board); the files are read from the repo's registered checkout path, parsed strictly, mapped and applied to the repo's empty store — answering the landed counts of epics, decisions, docs, tasks and blocks edges.
    #[tool(
        name = "import_md",
        output_schema = crate::schema::output_for_type::<ImportedDto>(),
        annotations(open_world_hint = false)
    )]
    async fn import_md(
        &self,
        Parameters(params): Parameters<ImportMdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it (the lib module
        // doc's rule).
        let mut state = self.lock_state();
        match answer_import_md(&mut state, &params) {
            Ok(output) => structured("import_md", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::dacode::DocDepth;

    /// One expanded register entry, for the unit pins.
    fn entry(key: &str) -> DocEntry {
        DocEntry {
            key: key.to_owned(),
            depth: DocDepth::Expanded,
            milestone: None,
        }
    }

    /// The walk's collection is sorted by walk-relative path bytes
    /// before matching, whatever order `read_dir` handed the walk:
    /// the collision refusal names its paths in byte order. The
    /// deliberately reversed input is the pin — a host whose
    /// `read_dir` happens to return byte order does not make this
    /// pass vacuously.
    #[test]
    fn the_collision_refusal_names_its_paths_in_byte_order() {
        // Byte order puts `01-extra.md` (`-`) before `01.md` (`.`);
        // the input is the reverse.
        let files = vec!["01.md".to_owned(), "01-extra.md".to_owned()];
        let error = doc_paths(&[entry("01")], files).unwrap_err();
        assert_eq!(
            error,
            ToolError::invalid(
                "the doc key '01' matches more than one file: \
                 docs/implementation/01-extra.md and docs/implementation/01.md"
            )
        );
    }

    /// The matcher's happy path at the unit layer: the map is
    /// repo-relative off the board-directory constant, the exact and
    /// prefix stem arms both visible.
    #[test]
    fn the_map_is_repo_relative_with_both_stem_arms() {
        // Unsorted input: byte order puts `README.md` (`R`) before
        // `appendices/…` (`a`).
        let files = vec![
            "appendices/B-decisions.md".to_owned(),
            "README.md".to_owned(),
        ];
        let docs = [entry("README"), entry("B")];
        let paths = doc_paths(&docs, files).unwrap();
        let expected: BTreeMap<String, String> = [
            ("README", "docs/implementation/README.md"),
            ("B", "docs/implementation/appendices/B-decisions.md"),
        ]
        .into_iter()
        .map(|(key, path)| (key.to_owned(), path.to_owned()))
        .collect();
        assert_eq!(paths, expected);
    }
}
