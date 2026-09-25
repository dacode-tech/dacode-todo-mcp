# dacode-todo-mcp — Design

> **Status: seed spec, design phase.** Every section below is either a
> requirement the user has stated, or a proposal marked PROPOSED. The
> proposals move only when their §12 decision is signed; the sign-off
> record is `docs/implementation/decisions.md`. This document carries no
> markdown links by convention (`tools/check-links.py` enforces it) —
> paths and sections are cited in backticks.

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
2. Directed graph as the working model (one candidate among several —
   §12 item 1).
3. A local database that can be queried, **not a relational database**.
4. Selection by repository *and* by top-level epic; the tooling asks for
   an epic name for the specific repo.
5. Repository paths must be updatable — checkouts move on the
   filesystem, and the tracker may be accessed from a repository other
   than the one a task belongs to (cross-repo access is optional for
   now — §12 item 8).

## 2. Architecture and Crate Layout

**PROPOSED — see §12.** Four Cargo workspace members, one binary:

| Crate | Job | Depends on |
|---|---|---|
| `todo-domain` | nodes, edges, graph invariants, selection rules. No I/O, no async, no workspace deps | nothing |
| `todo-store` | persistence behind a single `Store` trait; the engine chosen by §12 item 1 is sealed inside this crate | `todo-domain` |
| `todo-mcp` | the server binary: rmcp (official Rust MCP SDK) wire surface over stdio. Translates, never decides | both |
| `todo-cli` | inspect/debug the store without an MCP client. Optional — §12 item 5 decides whether it ships | domain + store |

Rules, enforced at review:

- A `use` that crosses a boundary above is a defect, not a style choice.
- The storage engine does not leak above the `Store` trait: no engine
  type, engine error, or engine feature flag in `todo-domain` or
  `todo-mcp`.
- Selection and ordering logic lives in `todo-domain` and nowhere else;
  the binary exposes it, it does not implement it.
- Whether the server is async is constrained by §12 item 1; until then
  domain and store stay sync and runtime-agnostic.

Data location: one store shared by all registered repositories, under
`~/.local/share/dacode-todo/` (macOS: the same path; no XDG relocation
game). Single store vs one file per repository is §12 item 4.

## 3. Domain Model

**PROPOSED — §12 items 2 and 3 settle the edges and the ids.** Three
node kinds, mirroring what the sibling boards already mean:

- **Repo** — a stable id (the key), a display name, a filesystem path
  (a mutable attribute, §7), and a registration timestamp.
- **Epic** — a repo's top-level grouping: id, repo, title, description,
  status. The selection unit the user asked for: every list query may
  name one.
- **Task** — id, repo, epic (its `parent`), kind, title, body, status,
  `verify` (the method that will prove it done — required at creation),
  `evidence` (required to reach `done`), `note` (required to reach
  `blocked`), timestamps.

Task kinds carry the sibling projects' vocabulary: `task` (one unit of
work), `gate` (a checkpoint that must pass), `finding` (discovered, not
the point of the task), `debt` (a deliberate shortcut). Statuses:
`pending`, `active`, `blocked`, `done`, `wontfix` — the exact set
`tools/tasks.py` already enforces, whose closure rules (evidence for
done, note for blocked, verify decided at creation) this server adopts as
product behaviour (§6), not just convention.

Timestamps are RFC 3339 UTC, passed in by the caller layer; nothing below
the binary reads a wall clock (§8).

## 4. Storage

Requirements — these bind whatever §12 item 1 chooses:

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

This spec leans (a), with (b) as the fallback if event-log replay or
snapshotting proves awkward; the `Store` trait (§4.2) makes the choice
reversible at one crate's cost. §12 item 1 decides.

### 4.2 The Store trait

One trait, owned by `todo-store`, is the whole persistence surface:
open/create, read the graph, append a mutation, snapshot/compact,
format version. Whatever engine wins §12 item 1 implements it; the
domain and server crates never see past it.

## 5. Graph Semantics and Ordering

### 5.1 Nodes and edges

Two structural edge kinds plus one annotation:

- **`parent`** — a tree: repo → epic → task. Every epic has exactly one
  repo; every task has exactly one epic. Reparenting a task moves it;
  deleting an epic requires its tasks to move first or be closed — no
  orphans, ever.
- **`blocks`** — a DAG over tasks (and optionally epics). `A blocks B`
  means B is not actionable until A is `done` or `wontfix`. **A write
  that would create a cycle is rejected before it lands** — cycle
  detection at write time, not repair at query time.
- **`relates`** — an annotation with no ordering semantics; it never
  affects selection.

### 5.2 Ordering and selection

Order is never stored; it is derived. The canonical order is a
topological order of `blocks` restricted to a repo (and optionally one
epic), with a total-order tiebreaker so the derivation is a function,
not a coincidence — the tiebreaker is §12 item 6.

`next_task(repo, epic?)` returns the first node in canonical order that
is **actionable**: status `pending` or `active`, not `blocked`, and no
unfinished `blocks` predecessor. The selection is a pure function of
graph state: same store bytes in, same answer out, on every run and
every platform.

## 6. MCP Tool Surface

**PROPOSED — §12 item 5 settles the list and the schemas.** Principles
that are not up for grabs:

- Every tool names its repository; epic is an optional filter wherever
  tasks are listed. The client flow the user asked for is
  `repo_list` → `epic_list(repo)` → `next_task(repo, epic)`, and the
  harness asks the user for the epic name when the query needs one.
- Every returned list carries the documented total order of §5.2 —
  a tool that returns an unordered list is a bug.
- Errors are structured data with stable codes (§6.1), never panics and
  never bare strings.
- Writes return the new state of the object written, so a client never
  needs a read-after-write round trip.
- **The closure discipline is enforced by the server, not by habit**:
  `task_update` to `done` without `evidence` is an error; to `blocked`
  without `note` is an error; creating a task without `verify` is an
  error. This is the "more deterministic than markdown" the project
  exists for — the board's culture becomes the API's contract.

Sketch (names are the proposal; rmcp's tool macros will shape schemas):

| Tool | Purpose |
|---|---|
| `repo_register` / `repo_list` / `repo_show` | registry CRUD (§7) |
| `repo_set_path` | a checkout moved; update the path attribute |
| `epic_add` / `epic_list` / `epic_update` | top-level grouping per repo |
| `task_add` / `task_show` / `task_list` | task CRUD; `task_list` filters by repo, epic, status, kind |
| `task_update` | status + fields, with the closure discipline above |
| `task_link` / `task_unlink` | `blocks` / `relates` edges; cycle-checked |
| `next_task` | **the question the system exists for** (§5.2) |
| `graph_show` | dump a repo's (or epic's) subgraph for debugging and handovers |
| `import_md` | one-shot importer from the sibling markdown boards (§9, §12 item 7) |

### 6.1 Error contract

Stable codes, versioned like the store format: `unknown_id`,
`duplicate`, `cycle`, `missing_verify`, `missing_evidence`,
`missing_note`, `orphan_not_allowed`, `corrupt_store`,
`version_mismatch`, `locked`. A client may branch on a code; it must
never parse a message.

## 7. Multi-Repo Registry and Path Handling

The server is global: one store, many repositories, reached from
whichever repo the client happens to be in. A repository is keyed by its
**stable id** (§12 item 3), never by its path — because paths are the
thing that changes:

- `repo_register(name, path)` mints the id and records the path.
- `repo_set_path(repo, new_path)` updates it when a checkout moves. The
  server records the change as a mutation like any other — the history of
  a repo's locations is part of the audit trail.
- Accessing the tracker **from** a repo other than the one a task
  belongs to is the normal case, not a special one: queries name the
  target repo explicitly. Auto-detecting "my repo" from the client's cwd
  is a convenience layer on top (§12 item 8), never the mechanism.

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
- **Single writer**: one process mutates the store at a time, held by an
  advisory file lock at the process boundary (`locked` error, §6.1, when
  contended). Readers may be concurrent only if the engine (a)/(b)
  choice makes that safe; until §12 item 1 lands, the design assumes
  one process, which is what one MCP server is.

## 9. Adoption and Migration

The three sibling boards are the migration source and the acceptance
test. Each has a slightly different dialect — camera and ikvm-html:
`U` follow-ups, `D` decisions, `M` milestones, session logs; dacode:
`U`/`B`/`A` registers plus milestone sub-steps — and the importer
(§12 item 7) maps them once, read-only:

- milestones and epics-in-prose → **epics**
- `U` items → **tasks** (kind `task`, or `debt` where the board says
  debt), status from the checkbox and strikethrough state
- open `D`/`B` decisions → **tasks** of kind `gate` (a gate is exactly
  what an unsigned decision is)
- dependency prose ("gates M2", "blocked on D4") → **`blocks` edges**
  where mechanical, findings where not

Import, then cut over: after an import is verified against its board,
the server is that repo's state of record and the markdown board keeps
only the narrative the server does not model (session logs stay files).
No live two-way sync — §12 item 7.

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
  is dependency-driven (§5.2); priority fields may come later by
  decision, not by accretion.
- No live sync with markdown files (§9).
- No replacement of the sibling projects' decision culture: `D`/`B`
  dossiers stay prose; the server tracks the *gate* they hang on.

## 12. Open Decisions

Each item is mirrored by a `D`-numbered entry in
`docs/implementation/decisions.md` with options, a recommendation, a
confidence and an `On sign-off:` work order. Agents propose; the user
decides. An item is deleted from this list when its decision is signed.

1. **Storage engine** — (a) append-only JSONL event log + in-memory
   graph + snapshots, (b) embedded KV (redb), (c)/(d) excluded by §4.1.
   Spec leans (a). Gates M1. See decisions.md D1.
2. **Graph model and edge semantics** — the §3/§5.1 proposal: node kinds
   repo/epic/task, task kinds task/gate/finding/debt, edges
   parent/blocks/relates, write-time cycle rejection, no-orphans rule.
   Gates M1. See decisions.md D2.
3. **Identity scheme** — ids for repos, epics, tasks: per-repo sequential
   integers (board-style `U7`), ULIDs, or typed slugs. Stability and
   never-reuse guarantees are part of the choice. Gates M1. See
   decisions.md D3.
4. **Store layout** — one shared store file for all repos vs one file per
   repo plus a registry index; where it lives; what `graph_show` exports
   across repos. Gates M2. See decisions.md D4.
5. **MCP tool surface** — the §6 list and names as proposed, or amended;
   whether `todo-cli` ships as a workspace member. Gates M3. See
   decisions.md D5.
6. **Selection tiebreaker** — the total order §5.2 requires when
   topology alone does not decide: creation order, id order, an explicit
   priority field, or epic order first. Gates M2. See decisions.md D6.
7. **Importer scope** — one-shot read-only import of the three sibling
   boards (no live sync), with the §9 mapping; which dialects are in v1.
   Gates M4. See decisions.md D7.
8. **Cross-repo ergonomics** — cwd auto-detection of "my repo" and
   querying other repos' tasks from any client: in v1 or deferred.
   Keying by stable repo id happens either way (§7). Gates M4. See
   decisions.md D8.

## 13. Packaging and Distribution

One binary, `todo-mcp`, built with `cargo build --release` and wired into
clients as a local stdio MCP server — in opencode, an `mcp` entry in the
global config naming the binary and the store location. Store format
version travels in the file header, independent of the binary version; a
binary refuses a store newer than it understands (`version_mismatch`,
§6.1) rather than guessing. No packaging beyond the binary in v1.

## 14. Roadmap

Tentative — the milestone table with exit criteria lives in
`docs/implementation/TODO.md` §2, and priority derives from what an item
gates (`tools/attention.py`).

- **M0** — spec settled: §12 items 1–3 signed, Cargo workspace skeleton
  with the lint regime and all gates green.
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
