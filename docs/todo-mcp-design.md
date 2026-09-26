# dacode-todo-mcp — Design

> **Status: signed — all eleven §12 decisions resolved 2026-09-25.**
> Every section is normative: a requirement the user stated, or a
> proposal made normative by its signed decision (record:
> `docs/implementation/decisions.md`). A new decision opens a new
> D-entry (D12+); signed shapes move only by a new decision, never by
> edit. This document carries no markdown links by convention
> (`tools/check-links.py` enforces it) — paths and sections are cited
> in backticks.

## 1. Purpose and Scope

Three sibling projects — `camera` at `/Users/mouse/RPM/shelly/camera`,
`ikvm-html` at `/Users/mouse/RPM/ali/ikvm-html`, and `dacode` at
`/Users/mouse/RPM/dacode/dacode/dacode` — each run a markdown task
system with real semantics: stable `U` ids that are never renumbered,
`D`/`B` decisions that gate milestones, `tasks.py` trackers where
closure demands evidence. Those systems are excellent audit trails and
poor query surfaces. "What is the next actionable task in repo X under
epic Y, given dependencies?" is a paragraph of prose parsing away from an
answer, two readers can disagree about the result, and nothing verifies
the claim mechanically.

This project replaces the query surface, not the audit trail: **one local
MCP server holding tasks as typed nodes of a directed graph in a local,
queryable, non-markdown store**, so that every question a board answers
in prose answers in one deterministic query. Clients are agent harnesses
(opencode first) working in any registered repository; the server is
shared across repositories, and tasks are selectable **by repository and
by top-level epic**.

Stated user constraints, which are requirements, not proposals:

1. Deterministic handling — not markdown files.
2. Directed graph as the working model (storage settled by D1: JSONL
   event log + in-memory graph).
3. A local database that can be queried, **not a relational database**.
4. Selection by repository *and* by top-level epic; the tooling asks for
   an epic name for the specific repo.
5. Repository paths must be updatable — checkouts move on the
   filesystem, and the tracker may be accessed from a repository other
   than the one a task belongs to (id-keyed cross-repo access is v1;
   cwd auto-detection deferred to M5 by D8).

## 2. Architecture and Crate Layout

Normative (D1–D5 signed 2026-09-25). Four Cargo workspace members, one
binary:

| Crate | Job | Depends on |
|---|---|---|
| `todo-domain` | nodes, edges, graph invariants, selection rules. No I/O, no async, no workspace deps | nothing |
| `todo-store` | persistence behind a single `Store` trait; the JSONL engine (D1) is sealed inside this crate | `todo-domain` |
| `todo-mcp` | the server binary: rmcp (official Rust MCP SDK) wire surface over stdio. Translates, never decides | both |
| `todo-cli` | inspect/debug the store without an MCP client. Ships (D5) | domain + store |

Rules, enforced at review:

- A `use` that crosses a boundary above is a defect, not a style choice.
- `todo-domain`'s *no workspace deps* means no path dependency on a
  sibling crate. `serde` derives on domain types are the shared schema
  layer, not a boundary violation; the serializer itself (the JSON
  engine) is confined to `todo-store`.
- The storage engine does not leak above the `Store` trait: no engine
  type, engine error, or engine feature flag in `todo-domain` or
  `todo-mcp`.
- Selection and ordering logic lives in `todo-domain` and nowhere else;
  the binary exposes it, it does not implement it.
- The D1 engine (JSONL event log + in-memory graph) needs no async
  runtime: domain and store stay sync and runtime-agnostic; the binary
  may adopt tokio for rmcp, the libraries must not require it.

Data location (D4, signed with a user amendment — per-repo stores, not
one shared database): under `~/.local/share/dacode-todo/` (macOS: the
same path; no XDG relocation game), one directory per repository —
`repos/<slug>/` holding that repo's event log and snapshots — plus a
shared registry file holding repo metadata and the slug→store mapping.
No task data lives in a cross-project shared store; cross-repo queries
iterate the registry and open per-repo stores. Lock scope: §8.

## 3. Domain Model

Normative (D2, D3, D9, D10 signed 2026-09-25). Five node kinds,
mirroring what the sibling boards already mean:

- **Repo** — a stable slug id (the key, immutable — D3), a display
  name, a filesystem path (a mutable attribute, §7), and a
  registration timestamp.
- **Epic** — a repo's top-level grouping: id, repo, title, description,
  status, `references` (D10 — e.g. JIRA/tracker links; an epic may be
  an external issue). The selection unit the user asked for: every
  list query may name one.
- **Task** — id, repo, epic (its `parent`), kind, title, body, status,
  `verify` (the method that will prove it done — required at creation),
  `evidence` (required to reach `closed`), `note` (required to reach
  `blocked` and `deferred`), `references` (D10), an optional
  agent-set `priority` (D11 — triage metadata, never an ordering
  input), an optional `until` (D9 — a date, pure metadata; graph
  targets ride `blocks` edges), timestamps.
- **Decision** (D10) — a structured sign-off dossier: id, repo, title,
  status (`open`/`signed`/`superseded`), options, recommendation,
  resolution, confidence, `references`, timestamps. An open decision
  may `block` tasks and epics (§5.1), so "never write past an unsigned
  decision" is enforced by the graph, not by habit. Transitions mirror
  task closure: reaching `signed` or `superseded` without a
  `resolution` is an error (`missing_resolution`, §6.1). Signing flips
  the status and the decision's `blocks` edges simply stop gating — no
  edge mutation, the history stays. Gating a milestone is a `blocks`
  edge to an epic (§9 maps milestones to epics), not a field.
- **Doc** (D10) — a registered document: id, repo, title, path
  (mutable, like a repo's), version, timestamps. The prose body lives
  in git; the server holds the stable address so citations survive
  file moves and resolve from any repo. The server never becomes the
  editor of prose bodies. A path update is an audited mutation, like
  `repo_set_path` (§7).

**Identity (D3, extended to the D10 kinds).** Repos are keyed by an
immutable slug chosen at registration (`camera`); display name and
path stay mutable. Epics, tasks, decisions and docs get per-repo
sequential integers per kind, allocated by the server, never reused,
never renumbered — gaps are meaningful. Prose cites ids with a
per-kind prefix — task `U` (board heritage), epic `E`, decision `D`,
doc `S`: `camera:U7`, `camera:D3`. Sequential allocation rides the
single-writer lock (§8).

**References (D10).** Epics, tasks, decisions and docs carry an
optional `references` list: `{kind, target, note?}` entries, kind ∈
`tracker`, `url`, `doc`, `spec`, `file`, `other`, canonically sorted
by (kind, target, note). A citation of a registered doc uses kind
`doc` with the doc's id as target and an optional `#anchor` for a
section (`camera:S1#5.2`) — stable across file moves.

Task kinds carry the sibling projects' vocabulary: `task` (one unit of
work), `gate` (a checkpoint that must pass), `finding` (discovered, not
the point of the task), `debt` (a deliberate shortcut).

Statuses (D9): `open`, `in_work`, `blocked`, `deferred`, `closed`,
`wontfix` — renamed from the `tools/tasks.py` set the culture comes
from (pending→open, active→in_work, done→closed) plus `deferred`. The
closure discipline is product behaviour (§6), not convention:
`closed`⇒evidence, `blocked`⇒note, `deferred`⇒note, `verify` decided
at creation. `deferred` means deliberately postponed, usually until
other work lands: the until-targets are `blocks` edges from those
items, and the optional `until` date is metadata. Nothing reactivates
by calendar or by an edge completing — selection reads neither (§8);
when an until-target closes, queries surface the item as
until-satisfied and reopening is an explicit update. `wontfix` remains
the one terminal that unblocks successors without evidence of
completed work (§5.1).

Timestamps are RFC 3339 UTC, passed in by the caller layer; nothing below
the binary reads a wall clock (§8).

## 4. Storage

Requirements — these bind the chosen engine (D1: JSONL event log):

- **Local and single-file** (or single-directory): no server process to
  query it, no network, no daemon to keep alive.
- **Not relational** — stated user constraint.
- **Queryable without loading prose**: by repo, by epic, by status, by
  kind, by dependency neighbourhood.
- **Deterministic bytes**: canonical serialization, fixed field order,
  stable key sorting (§8). Two saves of unchanged state are identical.
- **Versioned**: a format version in the header; a format change is a
  migration, never an in-place edit of meaning.
- **Corruption is a named error**, not a panic, and not silent
  truncation.
- **Human-auditable is a strong preference**: the store holds the audit
  trail the markdown boards held; a format a human can read in an editor
  keeps that property.

### 4.1 Candidate engines

- **(a) Append-only JSONL event log + in-memory graph.** Every mutation
  is one JSON line; load replays the log into the graph; periodic
  snapshot + truncation bounds replay. Scale is a non-issue: a task
  graph is 10^3–10^5 nodes, and every query here is a traversal, not a
  join. The log is diffable, appendable, and its determinism is easy to
  test (replay twice, compare bytes). Cost: load is O(log length);
  snapshotting is a component to get right.
- **(b) Embedded key-value store (redb).** Single file, ACID, pure Rust,
  no C dependency; adjacency encoded in key prefixes; queries without
  loading everything; copy-on-write pages give crash safety for free.
  Cost: the file is not human-readable, and byte-determinism is the
  engine's business, not ours — golden files test *our* exports instead.
- **(c) SQLite.** Relational; excluded by the stated constraint. Listed
  because it is the default answer and the rejection should be visible.
- **(d) A dedicated graph database.** Server weight or heavy
  dependencies for a graph that fits in a megabyte. Excluded.

**Signed (D1, 2026-09-25): (a)** — append-only JSONL event log +
in-memory graph + periodic snapshots. (b) redb remains the named
fallback if replay or snapshotting proves awkward in M1 — reachable
only as a new decision brought to the user, behind the `Store` trait
(§4.2), never a silent swap.

### 4.2 The Store trait

One trait, owned by `todo-store`, is the whole persistence surface:
open/create, read the graph, append a mutation, snapshot/compact,
format version. The JSONL engine (D1) implements it; the domain and
server crates never see past it.

### 4.3 Crash and recovery

The JSONL engine's crash semantics, concretizing the requirement above
that corruption is a named error, never a silent truncation:

- **Appends**: every event is one `write()` of a complete line under
  `O_APPEND`, followed by `fsync` — the torn window is at most one
  event, the one in flight when a crash happens.
- **Replay**: a malformed *last* line (no trailing newline, or
  unparseable) is a named `corrupt_store` error (§6.1) carrying its
  byte offset — never silently skipped. A malformed line anywhere else
  in the log is `corrupt_store` unconditionally: the interior of a log
  does not tear in a crash.
- **Repair** is explicit only — a repair operation (surfaced as a
  `todo-cli` command) shows the torn line and truncates it on
  confirmation, losing exactly that one in-flight event. No automatic
  repair path exists.
- **Snapshots are the recovery unit**: written temp-file + `fsync` +
  atomic rename, so they are never torn; replay reads the latest
  snapshot, then the log tail after it. The registry file follows the
  same atomic-rename discipline.
- **Backup** in v1 is copying a repo's store directory while no writer
  holds the lock (§8); there is no separate backup mechanism.

## 5. Graph Semantics and Ordering

### 5.1 Nodes and edges

Two structural edge kinds plus one annotation:

- **`parent`** — a tree: repo → epic → task; decisions and docs attach
  directly to their repo (D10). Every epic has exactly one repo; every
  task has exactly one epic. Reparenting a task moves it; deleting an
  epic requires its tasks to move first or be closed — no orphans,
  ever.
- **`blocks`** — a DAG over tasks, plus decision→task/epic gating
  edges (D10): an open decision blocking an epic holds all work under
  it. Epic-level `blocks` between tasks/epics remains a follow-up
  decision when a real query needs one (D2). `A blocks B` means B is
  not actionable until A is `closed` or `wontfix` — for a decision
  source, until it is `signed` or `superseded`. **A write that would
  create a cycle is rejected before it lands** — cycle detection at
  write time, not repair at query time. Deferred items (D9) record
  their until-targets as incoming `blocks` edges; the edges make the
  wait queryable, they never auto-reopen the item.
- **`relates`** — an annotation with no ordering semantics; it never
  affects selection.

### 5.2 Ordering and selection

Order is never stored; it is derived. The canonical order is a
topological order of `blocks` restricted to a repo (and optionally one
epic), with a total-order tiebreaker so the derivation is a function,
not a coincidence. The tiebreaker (D6, signed): topology first, then
epic registration order, then task id (creation order) — all derived
from existing fields. The agent-set `priority` (D11) is payload
metadata and never enters the tiebreaker.

`next_task(repo, epic?)` returns the first node in canonical order that
is **actionable**: status `open` or `in_work`, and no unfinished
`blocks` predecessor (a predecessor is finished at `closed`/`wontfix`,
a decision predecessor at `signed`/`superseded`). `blocked` and
`deferred` items are never actionable whatever their edges say. The
selection is a pure function of graph state: same store bytes in, same
answer out, on every run and every platform.

Batch selection (D11, amended same day at the user's direction):
`next_items(repo?, epic?, task?, limit?)` returns up to `limit`
(default 10) actionable nodes in canonical order, each annotated with
its direct blocked-by/blocks ids and a derived weight = count of
transitive dependents. Annotations never reorder; ordering is computed
server-side, in `todo-domain`, never in the client (§2). `next_task`
is the limit=1 case. With `task`, the scope becomes that item's
transitive dependents — the work hanging off a non-top-level item —
in the same canonical order, each marked immediately actionable or
still held by other predecessors.

Both `next_task` and `next_items` responses also carry a deferred
summary for the queried scope (D9 amendment): `deferred: {total,
ready}` — how many deferred items the scope holds, and how many of
them have every until-target `blocks` predecessor finished, i.e. are
worth revisiting now. The summary informs; nothing reopens
automatically. An agent that decides to look queries
`task_list(status=deferred)`, optionally filtered to ready items.

## 6. MCP Tool Surface

Normative (D5 signed 2026-09-25, extended by D10/D11 the same day;
JSON schemas firm up against rmcp's macros in M3). Principles that are
not up for grabs:

- Every tool takes a repository parameter. A server instance may be
  configured with a default repo slug (D8 amendment: a startup flag or
  env var in the client's per-project mcp entry, §13); an explicit
  parameter always wins, and the default is configuration, never a cwd
  heuristic. Epic is an optional filter wherever tasks are listed. The
  client flow the user asked for is `repo_list` → `epic_list(repo)` →
  `next_task(repo, epic)`, and the harness asks the user for the epic
  name when the query needs one.
- Every returned list carries the documented total order of §5.2 —
  a tool that returns an unordered list is a bug.
- Errors are structured data with stable codes (§6.1), never panics and
  never bare strings.
- Writes return the new state of the object written, so a client never
  needs a read-after-write round trip.
- **The closure discipline is enforced by the server, not by habit**:
  `task_update` to `closed` without `evidence` is an error; to
  `blocked` or `deferred` without `note` is an error; creating a task
  without `verify` is an error. This is the "more deterministic than
  markdown" the project exists for — the board's culture becomes the
  API's contract.

Sketch (names are the proposal; rmcp's tool macros will shape schemas):

| Tool | Purpose |
|---|---|
| `repo_register` / `repo_list` / `repo_show` | registry CRUD (§7) |
| `repo_set_path` | a checkout moved; update the path attribute |
| `epic_add` / `epic_list` / `epic_update` | top-level grouping per repo |
| `task_add` / `task_show` / `task_list` | task CRUD; `task_list` filters by repo, epic, status, kind, and — for deferred items — until-satisfied (`ready`) |
| `task_update` | status + fields, with the closure discipline above |
| `task_link` / `task_unlink` | `blocks` / `relates` edges (task and decision sources); cycle-checked |
| `next_task` | **the question the system exists for** (§5.2) |
| `next_items` | ranked actionable batch — repo/epic-scoped, or task-scoped (its dependents) — with blocked-by/blocks annotations, derived weight and the deferred summary (§5.2, D11 amended) |
| `decision_add` / `decision_list` / `decision_show` / `decision_update` | decision dossiers (D10); `decision_update` to `signed`/`superseded` requires `resolution` — the closure discipline applied to decisions |
| `doc_register` / `doc_list` / `doc_set_path` / `doc_update` | registered-document records for stable citation (D10); `doc_set_path` is audited like `repo_set_path` |
| `graph_show` | the complete graph (all repos) or one repo/epic subgraph, as an LLM-consumable handover dump (§6.2, D11 amended) |
| `import_md` | one-shot importer from the sibling markdown boards (§9; D7: camera dialect first) |

### 6.1 Error contract

Stable codes, versioned like the store format: `unknown_id`,
`duplicate`, `cycle`, `missing_verify`, `missing_evidence`,
`missing_note`, `missing_resolution`, `orphan_not_allowed`,
`corrupt_store`, `version_mismatch`, `locked`. A client may branch on
a code; it must never parse a message.

### 6.2 The graph dump (D11 amendment)

`graph_show(repo?, epic?)` is the function that hands an agent the
whole picture: with no arguments, every registered repo in sorted-slug
order; with `repo`, that repo; with `epic`, that epic's subtree plus
the decisions gating it. Output: nodes grouped by epic in §5.2
canonical order, each node carrying its `blocked_by`/`blocks` id
lists — dependencies explicit per node, no separate edge table to
join — plus status, kind, priority and references. Wire format is
canonical JSON (§8: fixed field order, sorted keys, golden-pinned):
MCP results are JSON anyway, and one canonical serializer is one
determinism surface. A YAML or Mermaid rendering, if wanted later, is
a pure function of this structure behind a `format` parameter — never
a second source of truth. Size is the caller's problem: the dump is
complete and deterministic; clients slice.

## 7. Multi-Repo Registry and Path Handling

The server is global: one store, many repositories, reached from
whichever repo the client happens to be in. A repository is keyed by its
**stable slug** (D3), never by its path — because paths are the thing
that changes:

- `repo_register(name, path)` mints the id and records the path.
- `repo_set_path(repo, new_path)` updates it when a checkout moves. The
  server records the change as a mutation like any other — the history of
  a repo's locations is part of the audit trail.
- Accessing the tracker **from** a repo other than the one a task
  belongs to is the normal case, not a special one: queries name the
  target repo explicitly. Per D8 (signed): cwd auto-detection is
  deferred to M5; the v1 convenience layer is the per-instance default
  repo slug (§6, §13) — explicit configuration, never a heuristic.

## 8. Determinism and Concurrency

Determinism is the product property; everything else is a feature:

- **Canonical serialization**: fixed field order, sorted keys
  (`BTreeMap`/`BTreeSet` for anything reaching disk or wire), RFC 3339
  UTC timestamps, no floats in persisted state.
- **Total orders everywhere**: every list a tool returns is ordered by a
  documented, total rule (§5.2).
- **Pure selection**: `next_task` derives from graph state alone — no
  wall clock, no randomness, no hash-iteration order below the binary
  layer.
- **Byte-stability test**: serialize → load → serialize is identical;
  any tool called twice on unchanged state returns identical bytes.
- **Single writer**: one process mutates one store at a time, held by an
  advisory file lock at the process boundary (`locked` error, §6.1, when
  contended). Lock scope follows the D4 layout: one lock per repo store
  plus one for the registry; an operation touching several stores
  acquires locks in sorted-slug order, so acquisition order can never
  deadlock. The JSONL engine (D1) tolerates concurrent readers of a
  committed log; the design still assumes one server process, which is
  what one MCP server is.

## 9. Adoption and Migration

The three sibling boards are the migration source and the acceptance
test. Each has a slightly different dialect — camera and ikvm-html:
`U` follow-ups, `D` decisions, `M` milestones, session logs; dacode:
`U`/`B`/`A` registers plus milestone sub-steps — and the importer
(D7, signed: the camera dialect first in v1; ikvm-html and dacode
follow once the camera pipeline and fixtures prove the mapping) maps
them once, read-only:

- milestones and epics-in-prose → **epics**
- `U` items → **tasks** (kind `task`, or `debt` where the board says
  debt), status from the checkbox and strikethrough state
- open `D`/`B` decisions → **decision** nodes (D10) with `blocks`
  edges to the work they gate; the `gate` task kind remains for
  non-decision checkpoints
- dependency prose ("gates M2", "blocked on D4") → **`blocks` edges**
  where mechanical, findings where not

Import, then cut over: after an import is verified against its board,
the server is that repo's state of record and the markdown board keeps
only the narrative the server does not model (session logs stay files).
No live two-way sync — D7.

Dogfooding rule: **this repo adopts the server for its own tracking only
after M3** — never dogfood an unsigned design; until then the board and
`tools/tasks.py` remain the trackers of record here.

## 10. Verification and Testing

Gates on every change: `cargo fmt --check`, `cargo check --workspace
--all-targets`, `cargo test`, `cargo clippy --workspace --all-targets`
(clean; the six denied lints of `.agents/roles/_rust.md` inherited from
workspace lints). Beyond them, the shapes this project is judged by:

- **Golden files** pin the store's bytes and every tool's
  request/response pair. A golden diff is a compatibility event, named
  in the commit.
- **Property tests** on graph invariants: random mutation sequences never
  produce a `blocks` cycle, an orphaned task, or two different
  `next_task` answers from one state.
- **Protocol-level tests**: every MCP tool tested request-in/response-out
  over the wire, never by calling the handler around it.
- **Determinism tests**: run twice, compare bytes (§8).
- **Importer fixtures**: the real committed `TODO.md` snapshots of the
  three sibling repos, checked into `crates/<importer>/tests/fixtures/`,
  with the expected graph beside each.

## 11. Non-Goals

- No multi-user or networked service; no authentication surface. The
  server is stdio-local, one user, one machine.
- No web UI, no TUI of its own; clients render.
- Not a general graph database: the schema is the domain model, fixed.
- No scheduling, no time estimates, no effort scoring in v1 — ordering
  is dependency-driven (§5.2). The agent-set `priority` field (D11,
  introduced by decision, not by accretion) is triage metadata shown
  in payloads; it never reorders selection.
- No live sync with markdown files (§9).
- Decision culture moves into the server (D10): `D`/`B` dossiers
  become structured decision nodes whose open state can block work;
  long-form spec and design prose stays in git, registered as `doc`
  records so citations are stable. The server never becomes the editor
  of prose bodies.

## 12. Open Decisions

**None — the list is closed.** Items 1–11 were signed by the user on
2026-09-25 and deleted per their work orders; the resolutions and
dossiers live in `docs/implementation/decisions.md`, and the sections
they settled are normative above. Agents propose; the user decides. A
new decision gets a new number (D12 onward) mirrored by a new item
here — numbers are never reused, and a future change to a signed shape
is a new decision, not an edit (the D6/D11 priority boundary is the
worked example).

## 13. Packaging and Distribution

One binary, `todo-mcp`, built with `cargo build --release` and wired into
clients as a local stdio MCP server — in opencode, an `mcp` entry in the
global config naming the binary and the store location. A per-project
mcp entry may pass a default repo slug (flag or env — §6, D8);
explicit tool parameters override it. Store format
version travels in the file header, independent of the binary version; a
binary refuses a store newer than it understands (`version_mismatch`,
§6.1) rather than guessing. No packaging beyond the binary in v1.

## 14. Roadmap

Tentative — the milestone table with exit criteria lives in
`docs/implementation/TODO.md` §2, and priority derives from what an item
gates (`tools/attention.py`).

- **M0** — spec settled (§12 closed 2026-09-25), Cargo workspace
  skeleton with the lint regime and all gates green.
- **M1** — domain model + storage engine: graph, statuses, persistence
  behind the `Store` trait, golden files pinning the format.
- **M2** — queries: repo/epic selection, dependency resolution,
  deterministic `next_task`; registry with path updates.
- **M3** — MCP server: the §6 surface over the wire, protocol-level
  tests, smoke against a real client (opencode).
- **M4** — adoption: importers for the camera/ikvm-html/dacode boards;
  the three repos tracked for real.
- **M5** — packaging, docs, cross-repo ergonomics if D8 arms them,
  polish.
