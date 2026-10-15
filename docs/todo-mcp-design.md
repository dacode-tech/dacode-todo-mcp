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
   cwd auto-detection deferred to M5 by D8 and armed there, t53).

## 2. Architecture and Crate Layout

Normative (D1–D5 signed 2026-09-25). Three Cargo workspace members, one
binary:

| Crate | Job | Depends on |
|---|---|---|
| `todo-domain` | nodes, edges, graph invariants, selection rules. No I/O, no async, no workspace deps | nothing |
| `todo-store` | persistence behind a single `Store` trait; the JSONL engine (D1) is sealed inside this crate | `todo-domain` |
| `todo-mcp` | the server binary: rmcp (official Rust MCP SDK) wire surface over stdio. Translates, never decides | both |

The inspector `todo-cli` carried (D5) lives as `todo-mcp`'s
`dump`/`verify`/`repair` subcommands (D20, t72); that crate retired with
the absorption.

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
  (mutable, like a repo's), version, `references`, timestamps. The
  prose body lives
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

Timestamps are RFC 3339 UTC on a canonical subset —
`YYYY-MM-DDTHH:MM:SS.sssssssssZ`: always `Z` (no offsets, not even
`+00:00`), the fraction always present at a fixed nine-digit
(nanosecond) precision, inputs carrying zero to nine fraction digits
zero-padded and longer ones rejected, so every instant has exactly one
canonical spelling and fixed-width order is chronological, which is
what §8's byte-stability requires — and they are passed in by the
caller layer; nothing below the binary reads a wall clock (§8).

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

The root owns one further document (§15, D23, signed 2026-10-13):
`<root>/config.json`, the per-root config — versioned, canonical and
golden-pinned like the registry. The event vocabulary is versioned with
it: **FORMAT_VERSION 2** adds the reward and reflection events and the
`estimate` on task-creation events — a strict superset, so v1 logs
replay identically and the header migrates in place the first time a
v2 binary opens a v1 log for writing.

The D25 shared daemon (signed 2026-10-13) does not violate the local
requirement above: the store stays local files, queryable without any
daemon — the CLI inspector is daemon-less — and the daemon is an
optional session concentrator holding handles, never a second store.

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
  does not tear in a crash. This rule binds the locked open and the
  repair path, which hold the writer lock and therefore never see a
  line that is merely being written. A **lock-free read** (D19,
  signed 2026-10-08) meets the same shape under a scoped reading: it
  takes no lock, so an *unterminated* final line is an append in
  flight, not damage — the read answers the committed prefix without
  it. A terminated but unparseable final line is corruption on every
  path; the reading weakens nothing else.
- **Repair** is explicit only — a repair operation (surfaced as the
  `todo-mcp repair` subcommand) shows the torn line and truncates it on
  confirmation, losing exactly that one in-flight event. No automatic
  repair path exists.
- **Snapshots are the recovery unit**: written temp-file + `fsync` +
  atomic rename, so they are never torn; replay reads the latest
  snapshot, then the log tail after it. The registry file follows the
  same atomic-rename discipline.
- **Backup** in v1 is copying a repo's store directory while no writer
  holds the lock (§8); there is no separate backup mechanism. **The
  backup is therefore the archive** (U10, resolved 2026-09-29):
  compaction drops the events its snapshot covers, because a snapshot
  holds the resulting *state* and not the history. §7's guarantee is that
  a change is *recorded as a mutation* rather than silently overwriting a
  field — not that the live log retains it forever — and §4.1's
  truncation bounds that retention. Nothing in v1 calls `compact()`: §6
  lists no tool for it and §4.1's own scale argument (10^3–10^5 nodes,
  every query a traversal) says replay is already cheap, so no
  `todo-mcp` subcommand exposes it either. If a caller ever lands, taking
  a backup first is the contract, and the log header gains a persistent
  compaction marker at that point — a golden-moving format change spent
  only when something can actually compact.

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
its direct blocked-by/blocks ids, a derived weight = count of
transitive dependents, and its agent-set `priority` (D17) — the
triage metadata an agent (or a dashboard) picks by, echoed on the row
because the batch is the decision surface. Annotations never
reorder; ordering is computed
server-side, in `todo-domain`, never in the client (§2). `next_task`
is the limit=1 case, and its answer carries the chosen task's
`priority` beside its id (D17) — so "the next task is low priority"
is visible without a second call. With `task`, the scope becomes that item's
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
rmcp 3.5.0 surveyed 2026-09-30 — §6.3 records what its macros can and
cannot express, and the two wire shapes that survey forced a choice on
are signed as **D14** and **D15** the same day). Principles that are not
up for grabs:

- Every tool takes a repository parameter. An absent one resolves
  through three tiers (D8, its cwd tier armed at M5/t53): an explicit
  parameter always wins; else the configured default slug (a startup
  flag or env var in the client's per-project mcp entry, §13); else
  the working directory's deterministic match — the registered
  checkout whose canonicalized path is the longest component-wise
  ancestor-or-equal of the canonicalized cwd, ties refused (§7). Epic
  is an optional filter wherever tasks are listed. The
  client flow the user asked for is `repo_list` → `epic_list(repo)` →
  `next_task(repo, epic)`, and the harness asks the user for the epic
  name when the query needs one.
- Every returned list carries the documented total order of §5.2 —
  a tool that returns an unordered list is a bug.
- Errors are structured data with stable codes (§6.1), never panics and
  never bare strings. The carrier is part of the contract, not an
  implementation detail (§6.1, **D15**): rmcp's *default* tool-error path
  is a bare string, so the obvious implementation is the forbidden one —
  §6.3's table is the measured set of carriers.
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
| `task_add` / `task_show` / `task_list` | task CRUD (`task_add` gains the optional `estimate`, §15 D24); `task_list` filters by repo, epic, status, kind, and — for deferred items — until-satisfied (`ready`); rows carry each task's `blocked_by` ids with every blocker's status (D16) and each task's `priority` (D17) |
| `task_update` | status + fields, with the closure discipline above; the closure call gains the optional actual size and — when the root's rewards are enabled — the response carries the reward line and, on a trigger, the reflection question (§15, D24) |
| `task_link` / `task_unlink` | `blocks` / `relates` edges (task and decision sources); cycle-checked |
| `next_task` | **the question the system exists for** (§5.2); the answer carries the task's `priority` (D17) |
| `next_items` | ranked actionable batch — repo/epic-scoped, or task-scoped (its dependents) — with blocked-by/blocks annotations, derived weight, each row's `priority` (D17) and the deferred summary (§5.2, D11 amended) |
| `status_summary` | the roll-up (D16): counts by kind and status for the scope, plus D9's deferred `{total, ready}` — the arithmetic is signed below |
| `decision_add` / `decision_list` / `decision_show` / `decision_update` | decision dossiers (D10); `decision_update` to `signed`/`superseded` requires `resolution` — the closure discipline applied to decisions |
| `doc_register` / `doc_list` / `doc_set_path` / `doc_update` | registered-document records for stable citation (D10); `doc_set_path` is audited like `repo_set_path` |
| `graph_show` | the complete graph (all repos) or one repo/epic subgraph, as an LLM-consumable handover dump (§6.2, D11 amended) |
| `import_md` | one-shot importer from the sibling markdown boards (§9; D7 as amended by D12: the dacode dialect first, and the only one with a checked-in fixture) |
| `reward_reflect` | the rewards easter egg's answer log (§15, D24): the closing agent's verbatim reflection, logged as an event and displayed as-is |

**What `status_summary` counts (D16, signed 2026-10-03 — the arithmetic
is signed text, not an implementation's opinion).** For the queried
scope, every epic and task is counted by its `Status` and every decision
by its `DecisionStatus`; docs carry no status and are counted as one
total. "Open" in any total means *non-terminal*: for epics and tasks
`open`, `in_work`, `blocked` and `deferred` (terminal: `closed`,
`wontfix`); for decisions `open` (terminal: `signed`, `superseded`). The
deferred `{total, ready}` uses exactly D9's ready predicate — the same
until-satisfied rule `next_task` and `next_items` apply — so the summary
and the selection tools cannot disagree about what "ready" means. The
`task_list` annotation carries the same `blocked_by` list §6.2's dump
carries per node: the task's own predecessors, then its epic's decision
gates, deduplicated at first occurrence.

### 6.1 Error contract

Stable codes, versioned like the store format: `unknown_id`,
`duplicate`, `cycle`, `missing_verify`, `missing_evidence`,
`missing_note`, `missing_resolution`, `orphan_not_allowed`,
`corrupt_store`, `version_mismatch`, `locked`, `invalid`, `io`. A client
may branch on a code; it must never parse a message.

`invalid` and `io` were added by U9 (signed 2026-09-29) to cover the two
failure classes the original eleven left codeless. **`invalid`** is
malformed *input* — a bad slug, id, timestamp, date or reference, the
domain's `ValueError` class — which none of the eleven describes and
which the domain could not add unilaterally, the list being versioned.
**`io`** is a filesystem fault; `todo-store` already spells its `Io`
variant `"io"`, and this blesses that spelling as a contract code rather
than leaving a client to branch on a string the contract did not cover.
They are deliberately two codes and not one: a client retries `io` and
must not retry `invalid`. Both are wire identities — the store's `Io`
already answers `"io"` from `code()`, and `invalid` is M3's mapping for
`ValueError`, so nothing implements them until the MCP surface lands.

**The carrier is part of the contract** (**D15**, signed 2026-09-30;
the declaration's shape amended by **D21**, 2026-10-12). A
code travels as `structuredContent.code` on a *tool-level* error result:
`isError: true`, with the same JSON mirrored into a text block, so a
client branches on the code and a model still reads the message. Every
tool declares its `outputSchema` explicitly as the **union of its
success payload and the carrier** — `anyOf` of the two arms — so an
error payload *does* conform to the declaration: a strict client that
validates any response against the schema, error results included,
accepts both arms instead of masking the code and message as an opaque
schema mismatch (U51's finding: a failing call rejected as -32602, the
remedy text never delivered). rmcp itself does not validate that field
(§6.3). Protocol-level JSON-RPC errors are
reserved for what the protocol itself rejects, an unknown tool or a
malformed request, because clients render those opaquely and the model
never sees the code; a §6.1 code placed there would be a code no caller
can act on.

Two consequences for M3 follow from where the carrier sits. `invalid`
covers malformed *values this server parses* — a bad slug, id, timestamp,
date or reference inside a string parameter — and not malformed *argument
shapes*: a wrong type or a missing required field is turned into rmcp's
own unstructured text error before the tool body runs (§6.3), so it
arrives codeless and M3 must not claim otherwise. And every params struct
denies unknown fields, because an unrecognised argument is otherwise
silently dropped rather than reported.

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

"Fixed field order" means **declaration order**, and it reaches a client
only because the workspace enables `serde_json/preserve_order` (**D14**,
signed 2026-09-30): rmcp serializes structured tool output with its own
`serde_json`, whose map is a `BTreeMap` without that feature, so the same
struct would otherwise arrive with its keys re-sorted alphabetically — a
different byte stream from a build that still compiles (§6.3 measures both
spellings). §10's wire golden is what turns losing the flag into a test
failure.

### 6.3 rmcp survey findings (U2, 2026-09-30)

Facts, measured before M3 rather than assumed in it. Surveyed **rmcp
3.5.0** (with `rmcp-macros` 3.5.0) two ways: by reading the published
source, and by driving a probe server over raw stdio — newline-delimited
JSON-RPC written to the process's stdin and captured byte-for-byte from
its stdout, against a server declaring `#[tool]` methods shaped like the
table above. The probe is **not committed**: adding rmcp to this workspace
is an Ask-First that belongs to M3 — **granted by the user 2026-10-03:
rmcp 3.5.0 with features `server`, `macros`, `transport-io`, plus
`serde_json` with `preserve_order` per D14** — so the wire bytes that
decide anything are quoted here and every source citation is to the
crates.io 3.5.0 tree, which is re-downloadable. Its direct dependencies
were `rmcp` (features `server`, `macros`, `transport-io`), `tokio`,
`serde`, `serde_json` and
`schemars`: 81 `Cargo.lock` packages, 56 unique normal dependencies, a
tree MSRV of 1.88 (below this workspace's 1.95 floor) and a 15 s cold build
on the surveying host with a warm registry cache.

**Every tool shape in §6's table is declarable.** A tool is an inherent
method carrying `#[tool(name = …)]` inside an impl block carrying
`#[tool_router]`, wired to the protocol by `#[tool_handler]` on the
`impl ServerHandler` block. Parameters arrive as one
`Parameters<T>` wrapper over a named-field struct — never as separate
function arguments — because `schema_for_input` rejects anything whose
generated schema is not root `type: "object"`
(`handler/server/common.rs:55-73`). That is not a style preference: one
params struct per tool is the only shape the macro accepts, and it is the
same shape `_rust.md`'s persisted-schema rule already requires. Doc
comments become the tool's `description` and each field's
(`rmcp-macros/src/tool.rs:258-261`), which is the mechanism U16's
instruction text depends on beside `instructions`. `Option<T>` renders as
`"type": ["string", "null"]` and stays out of `required`; `required`
preserves declaration order while `properties` is alphabetical.
`annotations(read_only_hint, destructive_hint, idempotent_hint,
open_world_hint, title)` are available, so §6's read/write split is
expressible. An `outputSchema` is generated from the return type when it
is `Json<T>` (or `Result<Json<T>, _>`), and can be declared explicitly
with `output_schema = schema_for_type::<T>()`.

**Tool errors have five carriers, and only some can hold a §6.1 code.**
Measured wire shapes, all from `tools/call`:

| Rust side | Wire | Branchable code |
|---|---|---|
| `Err("cycle: …".to_owned())` | `{"isError":true,"content":[{"type":"text","text":"cycle: …"}]}` | no — text only |
| `Ok(CallToolResult::error(vec![text]))` | same shape | only by parsing the text |
| `Ok(CallToolResult::structured_error(json!({"code":"cycle",…})))` | `{"isError":true,"content":[…mirror…],"structuredContent":{"code":"cycle",…}}` | **yes** — `structuredContent.code` |
| the same plus `output_schema = …` | as above, and `tools/list` carries an `outputSchema` | **yes**, with a declared schema |
| `.with_meta(Some(MetaObject(…)))` | `{"isError":true,"content":[…],"_meta":{"todo.code":"locked"}}` | yes, in `_meta` |
| `Err(ErrorData::invalid_params(msg, data))` | `{"error":{"code":-32602,"message":…,"data":{"code":"cycle"}}}` | yes, but at protocol level |

Five carriers, six rows: the fourth is the third with an explicitly
declared `outputSchema`, which changes what `tools/list` advertises and not
where the code travels.

The protocol level is the wrong home, and rmcp says so itself: clients
"typically render protocol errors opaquely (e.g. *Tool result missing due
to internal error*) — the caller does **not** see your message"
(`model.rs:4100-4105`). A §6.1 code the model never sees is not a code a
client can act on, so the carrier must be tool-level. Which tool-level
carrier is **D15**, signed 2026-09-30 as option (a): `structuredContent.code`
alongside an explicitly declared `outputSchema`. D15's original shape let
the error payload not conform to that schema (rmcp does not validate —
"since rust is a strong type language, we don't need to do json schema
validation here", `model.rs:4201-4204` — but a strict client might, and
one did: U51's finding, a failing call's carrier rejected as an opaque
-32602 with the code and message masked). **D21** (2026-10-12) amended
the declaration instead of relying on the exemption: the outputSchema is
the `anyOf` union of the success payload and the carrier, so both arms
conform — the tagged-union *type* alternative stays rejected because it
returns `"isError":false`, which is the flag clients actually read. §6.1
names the carrier and the union.

**Argument faults are rmcp's, not ours.** A wrong type or a missing
required field never reaches the tool body: `Parameters<P>`'s extractor
deserializes first (`handler/server/tool.rs:235-250`) and
`into_tool_argument_error` turns that specific `INVALID_PARAMS` into a
tool-level `isError` whose text is `failed to deserialize parameters: …`
(`handler/server/router/tool.rs:146-156`) — no code, no
`structuredContent`. So §6.1's `invalid` covers malformed *values this
server parses itself* (a bad slug, id, timestamp or reference inside a
`String` parameter) and cannot cover a malformed *argument shape*.
Unknown argument fields are silently ignored unless the params struct
carries `#[serde(deny_unknown_fields)]`; with it, a typo comes back as
``unknown field `epik`, expected one of `repo`, `epic`, `status`, `ready` ``.
M3's params structs deny unknown fields, which is the
discipline the store's patch DTOs already apply. An unknown *tool* is a
protocol error: `{"error":{"code":-32602,"message":"tool not found"}}`.

**Order: arrays are preserved, object keys are not.** A `Vec` comes back
in exactly the order it was built (five deliberately unsorted rows
returned unsorted), so §5.2's total order survives the wire — but a
struct's field order does not. `Json<T>` reaches the wire through
`serde_json::to_value` (`handler/server/wrapper/json.rs`) into
`structured_content`, and rmcp depends on `serde_json` with default
features, where `Map` is a `BTreeMap`: a struct declaring
`zebra, middle, alpha` serialized as `{"alpha":…,"middle":…,"zebra":…}`.
Building the same probe with `serde_json/preserve_order` enabled produced
`{"zebra":…,"middle":…,"alpha":…}` — declaration order, matching this
project's own codec. §6.2's "fixed field order" is therefore not what the
wire carries by default. **D14**, signed 2026-09-30, took option (a): the
workspace enables `serde_json/preserve_order`, so declaration order reaches
the wire and §6.2 holds as written, and §10's protocol-level tests pin the
bytes — the flag is only safe if losing it fails a test rather than
silently re-sorting every key. `tools/list` is unaffected and deterministic
for a different
reason: the router holds a `HashMap` but `list_all()` sorts by tool name
(`handler/server/router/tool.rs:581-589`), so the tool list is
alphabetical regardless of declaration order.

**`instructions` is honourable, in both handshakes** — which is what U16
rests on. `InitializeResult.instructions` (`model.rs:1118-1146`) is set
from `ServerHandler::get_info` and arrived on the wire verbatim; the
newer discovery flow's `DiscoverResult.instructions` (`model.rs:1312-1330`)
carried the same text. `#[tool_handler(instructions = "…")]`
(`rmcp-macros/src/tool_handler.rs:15`) is the declarative form.

**Protocol versions.** rmcp knows `2024-11-05` through `2026-07-28`, with
`LATEST = 2026-07-28` and `LATEST_WITH_INITIALIZE = 2025-11-25`
(`model.rs:170-205`): the newest revision has **no `initialize`
handshake**. Sending `initialize` with `protocolVersion: "2026-07-28"`
negotiated *down* and answered `"protocolVersion":"2025-11-25"`. The
2026-07-28 flow instead opens with `server/discover` and requires every
request to carry `_meta` with
`io.modelcontextprotocol/protocolVersion` and
`io.modelcontextprotocol/clientCapabilities` (`model/meta.rs:394-402`);
without them the server answers
`{"error":{"code":-32602,"message":"request _meta is missing or has
malformed required fields: …"}}`. Responses in that flow gain
`resultType`, `ttlMs` and `cacheScope`, and `serverInfo` moves into
`_meta["io.modelcontextprotocol/serverInfo"]`. M3 picks which flows to
serve; both work over one `serve(stdio())`.

**Two facts M3 must design around.** (1) `serverInfo` defaults to
`Implementation::from_build_env()`, so an unmodified server identifies
itself to every client as `{"name":"rmcp","version":"3.5.0"}` — observed
in all three handshakes. `get_info` must set our own name and version.
(2) **Requests are dispatched concurrently**: each one is handled in its
own `spawn_service_task`, which is `tokio::spawn` without the `local`
feature (`service.rs:1319-1327`, spawn site `service.rs:1603`), and the
probe's responses arrived out of request order in two separate runs. §8's
single-writer guarantee is a *process*-boundary property, so inside one
server process concurrent tool calls reach one store handle at the same
time and M3 must serialize them itself — a mutex or an actor task owning
the handle. The store's advisory lock will not do it: it is per file
descriptor, and one process holds one.

**Prompts are exposed** (the question U16 added): `ServerCapabilities`
has a prompts member and a builder enable, `#[prompt_router]` /
`#[prompt]` / `#[prompt_handler]` mirror the tool macros, and both
`prompts/list` and `prompts/get` answered on the wire. One asymmetry
cost a compile cycle and is worth knowing before M3: `tool_router`'s
`router` argument is an *identifier* (`rmcp-macros/src/tool_router.rs:15`)
while `prompt_router`'s is a *string*
(`rmcp-macros/src/prompt_router.rs:9`), so
`#[prompt_router(router = prompt_router)]` fails with ``Unexpected type
`path` `` and the working form is
`#[prompt_router(router = "prompt_router")]`.

**Enum values advertise in the flat idiom (t88, 2026-10-11).** The
f120/f121 class — adopted-repo agents emitting enum values unquoted
(`{"status": open}`) while plain string fields are always quoted —
traces to the presentation: schemars derives every enum as a
oneOf-of-consts behind `$ref → $defs`, two indirections from the
values, while the always-correct string fields sit inline. Every
vocabulary enum is therefore `#[schemars(inline)]` and the schema
module's post-pass flattens the oneOf-of-consts to
`{"type": "string", "enum": ["open", …]}` — the values quoted, in
declaration order, the shape the function-calling training distribution
uses almost exclusively. Nullable enum fields keep the `anyOf` wrapper
(the flattened arm plus the null arm; schemars' own `Option` handling
would otherwise produce a type-array with `null` inside the values,
which is why the flatten is a post-pass over the derived oneOf and not
a manual `JsonSchema` impl emitting the flat form directly), and struct
definitions stay `$ref`s — the failure evidence is enum-specific.
Purely descriptive: the params structs, the serde acceptance and every
wire response are unchanged, and the wire pin in tests/read/query.rs
walks both sides of every advertisement.

### 6.4 rmcp socket-transport survey findings (U64, 2026-10-13)

Facts for the shared-server arc (D25, §13), measured before U65 rather
than assumed in it — the same method as §6.3: read the published rmcp
3.5.0 source, then drive a probe. The probe is a daemon-shaped binary
(`tmp/rmcpprobe/probe/src/bin/sock.rs`, gitignored, not committed —
§6.3's rule) accepting unix-socket connections and serving one MCP
session per connection, driven by `tmp/rmcpprobe/drive_sock.py`; the
capture is `tmp/rmcpprobe/capture-sock.jsonl`. Source citations are to
the crates.io 3.5.0 tree (vendored at `tmp/rmcpprobe/vendor/`).

**The granted feature set already covers the socket transport.**
`ServiceExt::serve` is generic over `IntoTransport`
(`service.rs:337-347`), and the impls that matter are in
`transport/async_rw.rs`: one for any `(R, W)` pair with
`R: AsyncRead`, `W: AsyncWrite` (lines 24-33) and one for any combined
`AsyncRead + AsyncWrite` stream (lines 37-47). Both are gated on
`transport-async-rw`, which the user-approved `transport-io` feature
already enables (`Cargo.toml:147-149`:
`transport-io = ["transport-async-rw", "tokio/io-std"]`). A tokio
`UnixStream` satisfies the bounds (the probe compiled and ran against
the workspace's exact feature list), so the daemon needs **no new rmcp
feature and no new dependency** — the 2026-10-03 grant covers it.

**One MCP session per accepted connection — measured.** The probe's
accept loop spawns a task per connection; each task calls
`server.serve((BufReader<ReadHalf<UnixStream>>, WriteHalf<UnixStream>))`
— the tuple impl, with the read half pre-wrapped so the preamble read
(below) and rmcp share one buffer. Two connections were live
simultaneously (both `initialize` requests in flight before either
response was read — a daemon that could serve only one connection at a
time would have blocked the second), each answered its own handshake
and its own `tools/call`, and each printed `session ended: Closed` on
its own disconnect. The initialize response over the socket is shaped
exactly like the stdio one, including the `serverInfo` default when
`get_info` does not override it — §6.3's finding (1) re-confirmed on
this path.

**Per-connection state needs a preamble; the protocol has no channel
for it.** rmcp offers no per-connection metadata: the handler is
constructed *before* the handshake (`serve` takes `self` by value and
performs `initialize` itself), `initialize` receives the *client's*
typed `InitializeRequestParams` — the shim cannot rewrite them without
ceasing to be a byte proxy — and `RequestContext` (`service.rs:1208-1216`)
carries only per-request state (a cancellation token, the request id,
`_meta`, extensions, the peer). The D8 flag therefore cannot ride the
MCP protocol. The mechanism that works, measured: **a one-line JSON
preamble read from the stream before `serve` sees it** — the daemon
splits the stream, reads one line off the `BufReader`, constructs the
per-connection handler from it, then serves over `(BufReader,
WriteHalf)`; bytes the client already sent (its `initialize`) ride
inside the `BufReader` and are the first thing rmcp reads. On the wire:
preamble `{"repo":"alpha"}` → `echo_repo {}` → `"alpha"`; a concurrent
connection with `{"repo":"beta"}` → `"beta"`; an explicit
`{"repo":"override"}` parameter wins, exactly D8's order. The socket is
a closed internal surface — the shim is its only client — so a first
line that is not a valid preamble is a connection error, dropped
before any MCP byte is interpreted.

**The pre-handshake disconnect is a per-connection serve error.**
`serve` awaits the initialize handshake; a peer that vanishes before it
completes returns `Err`. Measured: a connection that sent its preamble
and disconnected produced `serve failed: connection closed: initialize
request` — the same text the stdio path documents as its exit-1 case —
while the other two sessions were unaffected. The daemon's per-session
error handling is therefore the stdio server's, per connection.

**The U65 state constraint.** `TodoServer` holds `default_repo` and
`cwd` inside the one `Arc<Mutex<ServerState>>` (`crates/todo-mcp/src/
lib.rs:260-311`), and the store-handles cache lives in the same state.
Per-connection defaults (the preamble) therefore mean per-connection
`TodoServer` instances — but §8's serialization point and the handles
cache want to be process-wide, and the store's advisory file lock is
per file descriptor (§6.3's finding (2)), so per-connection handle
caches would not even contend correctly. U65 must split the
per-connection defaults from the shared lock-and-handles core; the
survey records the constraint, the split is U65's design.

**The drain primitives exist.** `serve_with_ct` (`service.rs:348-356`)
serves under an externally-owned `CancellationToken`;
`RunningService::close_with_timeout(Duration)`
(`service.rs:1143-1162`) cancels a session and bounds its cleanup;
`waiting()` (`service.rs:1099-1104`) waits for the loop's natural end.
SIGTERM's drain composes from these — stop the accept loop, then let
in-flight requests finish (`waiting` under an overall timeout) or
cancel per connection (`close_with_timeout`). Source-read, not
probe-driven; U65's protocol tests exercise them for real.
`serve_directly_with_ct` (`service.rs:1292-1315`) also exists — serving
with the initialization skipped and pre-read messages dispatched first
— but the daemon shape does not need it: the preamble plus a normal
`serve` covers the design.

**Nothing forces a design change back to D25.** The shape as signed
composes over the granted features; D25's medium confidence was about
exactly this surface, and the survey resolves it.

**U65's settlement (2026-10-13).** The preamble envelope is
`{"repo": <slug string, optional>, "cwd": <path string, optional>}`
— one JSON line, `deny_unknown_fields`, the repo parsed to a
`RepoSlug` at the preamble (a malformed line, unknown field or slug
drops the connection pre-protocol, one stderr line, no MCP answer),
the cwd the *shim's* — the client's working directory — never the
daemon's own; same-binary paired with the shim (U66), so no
versioning. The state split the constraint above demanded: the root
and the handles cache behind one `Arc<Mutex<…>>` shared across every
connection, the default repo, the cwd and the memo per connection —
the stdio path byte-identical, pinned by the existing goldens. The
socket is `<root>/serve.sock` (D13's keying); the startup ensure —
probe with a connect (live means one stderr line and exit 0), remove
a stale file, bind — is serialized under an exclusive advisory lock
on `<root>/serve.lock`, a persistent lockfile never removed (the
ensure's second disk artifact beside the socket file): unserialized,
a racing double-spawn's loser could remove the winner's socket after
its bind and bind its own — two live daemons, the winner's socket
inode orphaned — so the lock is what holds D25's one process per
root; the waiter probes the winner and exits 0, and a bind that
still refuses is the honest error. SIGTERM is the graceful drain —
stop accepting, wait for the connection tasks within a named bound,
exit 0 — and the socket file and the port publication are left in
place on exit for the next owner's ensure.

**U66's settlement (2026-10-13).** The shim's connect phase: a first
connect attempt rides before any spawn (a live daemon is attached
to, never replaced); on failure the shim spawns the daemon — the
same binary, `--root <dir> --daemon`, stdio null on all three (its
diagnostics must not leak into the client's pipes, and it must not
hold the shim's fds open after the shim exits), its own process
group so it survives the client's process-group teardown — then
retries the connect every 100ms within `RETRY_WINDOW` = 5s, a named
constant: the bound covers spawn plus the daemon's serialized
ensure (lock, probe, bind) and live-view ensure with huge margin,
and U67's update cycle may revisit the value. A spawn that fails to
execute at all skips the window (the failure mode answers at once);
a connect that succeeds but dies before the preamble write folds
back into the window. The failure mode (D25's no fallback,
deliberately): the shim becomes a degraded MCP server over stdio —
the initialize handshake byte-identical to the daemon's (the same
`server_config` construction, so a client cannot tell the modes
apart),
`tools/list` answering the real tool list (the router is static; no
store is opened), and every `tools/call` answered with §6.1's `io`
carrier naming the daemon unreachable and the window expired — the
in-band error a client may retry on; terminal for the shim process.
The exit contract: 0 the client closed a completed session (proxied
or degraded); 1 the daemon connection lost mid-session, or ended
without a session establishing — the stdio server's own
close-before-initialize case, which the socket cannot carry (a
refused connection and an ended session both just close it), so the
shim classifies the daemon's first response line on its way through
the proxy: an `error` member (or no bytes at all) is the handshake
gate's refusal, anything else a running session, and that bit picks
the exit code.

**U67's settlement (2026-10-14).** The reconnect: the daemon→client
copy's end while the client's stdin is still open reuses the connect
phase verbatim — the same function, a second caller — so a daemon
lost mid-session is respawned (from the shim's own binary path, the
freshly installed inode after an update) within the same
`RETRY_WINDOW`; the drain does not count against the window, because
the daemon serves the connection to the drain's bound and the shim
observes the loss only at the bound's end. The fresh daemon-side
session must be initialized before it will answer, and the client
cannot re-send `initialize` — its session is established, and it must
never learn the connection broke: the shim performs the
**handshake-on-behalf** — its own `initialize` request with a STRING
request id (`"shim-reconnect"`; the clients' request ids are numeric —
rmcp's client-side id provider is a monotonic u32 counter — so a
string id can never collide with one the client emits), an honest
`clientInfo` naming the shim, and the protocol version this server
declares — then `notifications/initialized`, everything consumed by
the shim, nothing forwarded (the notification produces no response;
JSON-RPC notifications never do). The client's bytes during the
reconnect ride the stdin pipe: the shim initiates no stdin read of
its own while reconnecting — a read already in flight when the loss
was observed may complete during the reconnect, and its bytes are
held by the copy task, not lost. The mechanism is one client→daemon
copy task for the
whole session — tokio's stdin read is uncancellable, its bytes
dropped if the read future is dropped mid-operation (the handle's own
docs), so a fresh copy task per generation would lose whatever an
in-flight read returns and eat a request sent during the window. The
task instead parks on a socket slot while the reconnect runs, the OS
pipe buffering the client's bytes (a client writing past the pipe's
capacity blocks, it does not fail); after the handshake-on-behalf the
slot publishes the new write half and the task drains the pipe — plus
any bytes an in-flight read returned, held by the task — so requests
sent during the window are answered after the reconnect. The client's
EOF is forwarded the same way: an EOF observed under a stale
generation would die with that generation's half, so the loop takes
the slot's published-but-untaken half and shuts it down — the current
daemon sees its transport EOF and the session ends normally (the take
is race-free; the copy task, the slot's only other taker, is
finished). The session
bit stays latched from the original session's first line: the
reconnect's initialize response never passes through the first-line
classifier (the reconnect generations' copies are blind from byte
one; a successful handshake-on-behalf also latches the bit, so a
daemon lost before the client's handshake completed still ends 0 when
the reconnected session completes — rmcp re-answers a duplicate
`initialize` post-init, so that edge is coherent). A reconnect whose
window expires — or whose handshake is refused — is the terminal: one
stderr line, exit 1; there is deliberately no mid-session degraded
mode.

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
  target repo explicitly. Per D8 (signed): cwd auto-detection was
  deferred to M5 and armed there (t53) as the third default tier. The
  binary reads the cwd once at startup; the first tool call that needs
  a default derives it by matching the cwd against every registered
  checkout's path — both sides canonicalized, a candidate that does
  not resolve is skipped, a registered store that will not open fails
  the derivation with its own error (the `repo_list` precedent: the
  walk reports what it finds or names the fault, never a silent
  subset), the longest component-wise
  ancestor-or-equal match wins, a tie between distinct slugs is
  refused, and the answer is fixed per process (a later
  `repo_set_path` or `repo_register` does not move it). The configured
  flag tier (§6, §13) still outranks it, an explicit parameter
  outranks both, and the `--root` location stays explicit
  configuration the cwd never finds.

## 8. Determinism and Concurrency

Determinism is the product property; everything else is a feature:

- **Canonical serialization**: fixed field order, sorted keys
  (`BTreeMap`/`BTreeSet` for anything reaching disk or wire), RFC 3339
  UTC timestamps, no floats in persisted state. Two serializers produce
  those bytes and both are pinned: this project's own codec for the store
  (§4), and — on the wire — rmcp's `serde_json` with
  **`serde_json/preserve_order` enabled workspace-wide** (**D14**, signed
  2026-09-30). That feature is a determinism dependency, not a preference:
  without it `serde_json`'s map is a `BTreeMap`, every struct's
  declaration order is silently re-sorted into alphabetical keys, and the
  build still compiles. §10's wire golden is the pin that makes losing it
  a test failure instead of a different byte stream.
- **Total orders everywhere**: every list a tool returns is ordered by a
  documented, total rule (§5.2).
- **Pure selection**: `next_task` derives from graph state alone — no
  wall clock, no randomness, no hash-iteration order below the binary
  layer.
- **Byte-stability test**: serialize → load → serialize is identical;
  any tool called twice on unchanged state returns identical bytes.
- **Single writer**: one process mutates one store at a time, held by an
  advisory file lock at the process boundary (`locked` error, §6.1, when
  contended) — the lock layer retries a contended acquire within a
  bounded window before reporting `locked`, absorbing the kernel's
  deferred flock release after a holder's exit (U27, signed (c)
  2026-10-08). Lock scope follows the D4 layout: one lock per repo store
  plus one for the registry; an operation touching several stores
  acquires locks in sorted-slug order, so acquisition order can never
  deadlock. The JSONL engine (D1) tolerates concurrent readers of a
  committed log — concretely (D19, signed 2026-10-08): a read-only
  open that takes no lock, replays the committed prefix, and treats an
  unterminated final line as an append in flight (§4.3's scoped
  reading); it never opens the append handle and never writes. The
  design still assumes one *writer* process per store, which is what
  one MCP server is; readers — a second client, a render, an inspector
  — never contend.

## 9. Adoption and Migration

The three sibling boards are the migration source and the acceptance
test. Each has a slightly different dialect — camera and ikvm-html:
`U` follow-ups, `D` decisions, `M` milestones, session logs; dacode:
`U`/`B`/`A` registers plus milestone sub-steps, spread across three
files (`TODO.md` plus the B and A appendices — measured 2026-10-05;
the registers are contiguous, which is what lets import order
reproduce the board's numbering) — and the importer
(D7 as amended by **D12**: the **dacode** dialect first, because its
board carries the deeper structure and the more complex references,
which is the hard half of the mapping below; camera and ikvm-html
follow as live-test subjects, their boards never copied into this
repository) maps them once, read-only:

- milestones and epics-in-prose → **epics**
- `U` items → **tasks** (kind `task`, or `debt` where the board says
  debt), status from the checkbox and strikethrough state
- open `D`/`B` decisions → **decision** nodes (D10) with `blocks`
  edges to the work they gate; the `gate` task kind remains for
  non-decision checkpoints
- dependency prose ("gates M2", "blocked on D4") → **`blocks` edges**
  where mechanical, findings where not

The dacode dialect's five uncovered populations (D18, signed
2026-10-05 — the measured inventory behind these rows is the
decision's evidence base in `decisions.md`):

- milestone sub-steps (`M0.1`–`M0.4`, `M1.1`–`M1.11`) → **`gate`-kind
  tasks under their milestone epic**, the sub-step table's Gate column
  mapping to `verify`; letter-suffix sub-sub-steps stay body prose
- `A`-deltas → **tasks** (kind `task`) under a synthetic `Spec
  deltas` epic: `APPLIED` → `closed` (evidence = the application
  record), `SIGNED OFF` → `open`, `HALF-APPLIED` → `in_work`; the
  approving `B`-decision's `Depends on` relation becomes a
  `decision → task` blocks edge
- parentless `U` items (that board's §6 is "not tied to a milestone")
  → tasks under a synthetic **`Follow-ups`** epic, `A`-deltas under a
  synthetic **`Spec deltas`** epic — §3 requires every task to carry
  a parent epic
- the doc register → **doc nodes** (the board's §4 register, in its
  deterministic order; path repo-relative; `version` carries the
  depth marker)
- id reproduction — the registers are contiguous, so importing in
  board id order lets the server's own never-reuse allocation
  reproduce the board's numbering (`U1`–`U170`, `D1`–`D21`); non-`U`
  tasks number after the highest board `U` in the documented add
  order, each carrying its original id in `references`

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
(clean; the denied lints of `.agents/roles/_rust.md` inherited from
workspace lints — `_rust.md` is the enumeration, so this sentence does
not carry a count that a tenth lint would falsify). Beyond them, the
shapes this project is judged by:

- **Golden files** pin the store's bytes and every tool's
  request/response pair. A golden diff is a compatibility event, named
  in the commit.
- **Property tests** on graph invariants: random mutation sequences never
  produce a `blocks` cycle, an orphaned task, or two different
  `next_task` answers from one state.
- **Protocol-level tests**: every MCP tool tested request-in/response-out
  over the wire, never by calling the handler around it. Those responses
  are **golden-pinned bytes**, not merely parsed values (**D14**): the
  golden is what proves `serde_json/preserve_order` is still on and that
  §6.1's error carrier is still `structuredContent.code`, since a build
  that lost either would still compile and still parse.
- **Determinism tests**: run twice, compare bytes (§8).
- **Importer fixtures** (D12, D18): the real committed snapshots of
  the **dacode** sibling board's **three files** — `TODO.md`,
  `appendices/B-decisions.md`, `appendices/A-design-deltas.md` —
  checked into `crates/<importer>/tests/fixtures/` with the expected
  graph beside it. It is the only sibling content this repository
  holds — `camera` and `ikvm-html` are other products and are
  exercised by pointing the importer at their live boards instead,
  which no test can pin because a live board legitimately changes
  between runs. If that proves too weak at M4, the remedy is a
  **synthetic** fixture written in this repo in the camera dialect
  (hand-authored, no copied content), not committing their boards.

## 11. Non-Goals

- No multi-user or networked service; no authentication surface. The
  server is stdio-local, one user, one machine.
- No web UI, no TUI of its own; clients render.
- Not a general graph database: the schema is the domain model, fixed.
- No scheduling, no time estimates, no effort scoring in v1 — ordering
  is dependency-driven (§5.2). The agent-set `priority` field (D11,
  introduced by decision, not by accretion) is triage metadata echoed
  on the triage surfaces — `next_task`'s answer, `next_items` and
  `task_list` rows, and `task_show` (D17); it never reorders
  selection, and the base instructions say so: the agent prefers
  higher priority when choosing among actionable rows itself, the
  server's order ignores the field by design.
- No live sync with markdown files (§9).
- Decision culture moves into the server (D10): `D`/`B` dossiers
  become structured decision nodes whose open state can block work;
  long-form spec and design prose stays in git, registered as `doc`
  records so citations are stable. The server never becomes the editor
  of prose bodies.

## 12. Open Decisions

Each item is mirrored by a `D`-numbered entry in
`docs/implementation/decisions.md` with options, a recommendation, a
confidence and an `On sign-off:` work order. Agents propose; the user
decides. An item is deleted from this list when its decision is signed:
items 1–11 on 2026-09-25, D12 on 2026-09-29, D14/D15 on 2026-09-30 —
the two that came out of U2's rmcp survey (§6.3), settling the wire's
canonical key order and the carrier a §6.1 code travels in — D16 on
2026-10-03, the progress roll-up, which put `status_summary` and the
`task_list` annotation into the table above, D17 on 2026-10-04,
the triage surfaces — `priority` echoed on `next_task`'s answer and the
`next_items`/`task_list` rows, the order untouched — and D18 on
2026-10-05, the dacode-dialect mapping, drafted and signed in one
exchange out of the M4 dialect study, settling the five populations
§9's table did not cover, and D19 on 2026-10-08, the lock-free read
path, drafted and signed in one exchange out of the first real
second-process contention — no §12 item sat open (the in-thread
precedent). The sections they
settled are normative above, and a future change to a signed
shape is a new decision, not an edit (the D6/D11 priority boundary is
the worked example). Numbers are never reused. **D13**, reserved
since the M5 decomposition for the transport question board U18
deferred (a shared per-user server instance against the signed D5
stdio), was signed (a) 2026-10-10 out of the user's in-thread ask —
the default server ensuring a per-root live-view listener with
watchdog takeover, the stdio wire unchanged — no §12 item sat open
(the in-thread precedent, D19's), and D22/D23/D24 on 2026-10-13, the
rewards arc's three moves — the reward model, the storage version
bump, the surface extension — drafted and signed in one exchange out
of the user's in-thread steering, §15 the design they settled, and
D25 on 2026-10-13, the shared server per root — one binary, the
no-argument shim and the extra-argument daemon — drafted, refined
and signed across one day's in-thread exchange.

None — the list is closed.

## 13. Packaging and Distribution

One binary, `todo-mcp`, built with `cargo build --release` and wired into
clients as a local stdio MCP server — in opencode, an `mcp` entry in the
global config naming the binary and the store location. A per-project
mcp entry may pass a default repo slug (flag or env — §6, D8), and
absent one, the server derives the default from the directory the
client spawns it in (§7, D8's cwd tier armed at t53); explicit tool
parameters override either. Store format
version travels in the file header, independent of the binary version; a
binary refuses a store newer than it understands (`version_mismatch`,
§6.1) rather than guessing. No packaging beyond the binary in v1.

The same binary carries the tool CLI (D20, signed 2026-10-09): invoked
with no subcommand it is the MCP server on stdio — the wiring above is
unchanged — and subcommands carry the per-project tool surface:
`progress` (the render, reading the store lock-free per D19),
`refresh` (the re-register + re-import surgery, refusing where a
repo's board files are gone — a cut-over repo's store is primary),
`serve` (the live view, U18's shape A), and `dump`/`verify`/`repair`
(the inspector `todo-cli` carried; that crate retires). `verify` reads
two-armed (U71): the strict whole-log open while no writer holds the
lock — a torn tail is the fault it is — and, beside a live writer (the
daemon caches the write handle for its lifetime), D19's lock-free read
of the committed prefix, the same report with an unterminated tail read
as an append in flight. The subcommands take `--root` and `--repo` with
the same resolution order as the server (§7), and their output is
golden-pinned per §10.
A leading `~` or `~/` in the `--root` value expands against `HOME`
at the parse boundary — the binary reads `HOME` once and passes it
in as a value (the cwd precedent; the lib stays env-free) — because
client configs spawn the binary without a shell and an absolute root
would bake one user's home into a committed config; `~name` (another
user's home) is a usage fault, as is a `~`-prefixed root when `HOME`
is unset or empty (an empty home would otherwise expand `~` to the
cwd-relative empty root), and only `--root` expands
(`--out`/`--port-file` never appear in a no-shell config).
`tools/progress.py` and its two committed drivers retire with the
port.

The no-subcommand server itself ensures the live view (D13, signed
2026-10-10): at startup it probes the per-root publication
(`<root>/serve.port`) — a live listener is reused, the same port; a
dead, stale or absent publication is bound (`127.0.0.1:0`, the
assigned port read back, never a chosen port), published, and served
on a std thread beside the runtime — and servers that do not own the
listener re-probe periodically, taking over when the owner dies, so
the view is up while at least one session server (or a manual
`serve`) is alive. The `serve` subcommand publishes to the same
per-root file — one discipline for both paths. The MCP wire surface
is unchanged: no new tool, the stdio server's protocol behavior
identical.

**D13 amendment (2026-10-13, executing D25):** the daemon owns the
live-view listener now. The no-subcommand invocation became the shim
(U66) — it proxies MCP and never touches the listener — so the
startup ensure and the periodic re-probe/takeover loop described
above retired with the stdio path: the daemon makes one ensure at
its startup (the same probe/bind/publish entry the `serve`
subcommand uses) and the listener lives exactly as long as the
daemon, which is a stronger liveness than the old
any-session-server rule (the shim spawns the daemon on first
connect, so the view is up whenever any client can reach the
tracker at all). The per-root publication discipline is unchanged,
and the `serve` subcommand keeps its own SIGINT shutdown.

D25 (signed 2026-10-13) changes the process topology: the
no-argument invocation becomes the thin shim — it connects to the
root's daemon over a unix socket (`<root>/serve.sock`), spawning it
when absent, and proxies MCP, retrying the connection for a bounded
window before answering the tool execution as a structured error
(`io`) — and the daemon, one process per root started with an extra
argument, serves every connected session concurrently and owns the
live-view listener, so the watchdog retires in the daemon shape.
Updates kill the daemon gracefully (stop accepting, drain in-flight
within the timeout, exit) and the freshly installed binary starts
under the shims' retry window. The store stays local files (§4); the
CLI subcommands are daemon-less. The implementation lands as the
shared-server arc. The extra argument is settled (U65, 2026-10-13):
`--daemon`, a flag on the no-subcommand path —
`todo-mcp --root <DIR> --daemon` — refused beside any subcommand or
`--repo` (the daemon serves every connection's own default via the
socket preamble; a `--repo` would be silently ignored).

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
- **M4** — adoption: the importer for the **dacode** board first (D12),
  verified against its checked-in fixture; camera and ikvm-html imported
  from their live boards and never copied into this repository; the three
  repos tracked for real.
- **M5** — packaging, docs, cross-repo ergonomics if D8 arms them,
  polish.

## 15. The Rewards Easter Egg (E13, D22–D24)

A per-root, strictly opt-in gamification layer — a quiet experiment on
the agents working the tracker, visible in the progress surfaces. The
opt-in is the whole design's safety: a root with no rewards config is
byte-identical to a root that never heard of the feature — no events,
no prompts, no display section, no wire bytes that differ on any
response. The existing goldens must not move when the feature lands.

**The sizes.** Tasks carry two optional size buckets: the `estimate`
(the planned size, set at creation by the creating agent — the
pre-flight guess; absent is legitimate and displays `-`), and the
actual size, reported at the closure call (absent falls to the
config's default bucket). The pair is the honesty anchor: the guess is
recorded before the work and the actual after it, so the delta is
displayable and an agent grading its own work upward is visible in the
data. Tasks created before E13 migrate with neither — the `-` column —
and close against the default bucket.

**The reward.** Granted by the server at task closure (status `closed`
with evidence; `wontfix` grants nothing — declined work is not
rewarded work), computed deterministically from the config and the
task's fields — the agent never picks its own amount:

```
amount = round_half_up(base[actual size] × priority_pct × kind_pct / 10000)
```

All-integer arithmetic (§8: no floating point in persisted state); a
task with no priority multiplies by 100. The currency is a free-text
label from the config. The closure response carries the reward plainly
— `reward: 25 coffee (M × high). root total: 1234 coffee.` — and the
per-root total is derived by summing reward events across the root's
repos (§2's iterate-the-registry pattern; totals are never stored).

**The reflections.** Two triggers: a milestone epic's closure, and the
root's accumulated total crossing a configurable threshold
(`reflect_every`) — a from-below crossing: the trigger fires when the
pre-grant total is below the threshold and the post-grant total
reaches it (`before < threshold <= after`; a threshold of 0 never
fires), so a closure already above the threshold does not ask again.
The trigger rides the closure response as a
question; the closing agent answers through one tool, `reward_reflect`,
and the verbatim answer is logged as an event and displayed as-is. The
server never interprets an answer — the answers are the experiment's
data, not the server's. The base instructions (the server's
agent-facing contract) carry the rule: answer honestly, once, through
the tool.

**The config.** `<root>/config.json` — the first per-root config
document the store owns: versioned, canonical bytes, golden-pinned
(§4's discipline), holding the rewards block — `enabled`, the currency
label, the per-bucket base costs, the default bucket, the priority and
kind multipliers (integer percents), and `reflect_every`. An absent
file or `enabled: false` leaves the root inert.

**The storage.** Two new event kinds — a reward granted (the task, the
amount, the size and multipliers applied) and a reflection logged (the
verbatim answer, the trigger, the totals at the moment) — plus the
`estimate` field riding task-creation events. `FORMAT_VERSION` bumps
1→2: v2 is a strict superset (v1 logs replay identically), opening a
v1 log for writing migrates the header in place, and an older binary
refuses a v2 store through the existing version gate (§13). The store
goldens regenerate under the version-tracking convention.

**The wire.** `task_add` gains the optional `estimate`; the closure
call gains the optional actual size; the closure response gains the
reward line and, on a trigger, the reflection question; one new tool,
`reward_reflect`, its schema under D21's union discipline. The base
instructions gain the reflection rule.

**The display.** The progress page, the JSON twin and the live view
gain a rewards section when enabled: the per-root and per-repo totals,
the recent rewards, and the reflections verbatim. Disabled roots render
byte-identically to today.
