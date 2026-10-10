# dacode-todo-mcp

> **Purpose**: a deterministic task-graph MCP server — markdown TODO
> boards become typed, queryable graphs, and "what is next in repo X
> under epic Y" becomes a protocol call instead of a prose parse.

## What this is

Three sibling projects (dacode, camera, ikvm-html) each run a markdown
task board with real semantics — stable ids, decisions that gate
milestones, evidence-backed closure. Those boards are excellent audit
trails and poor query surfaces: every question is a prose parse, and
two readers can disagree about the answer.

This server keeps the semantics and makes the questions
machine-answerable. Tasks are typed nodes (`task`, `gate`, `finding`,
`debt`); dependencies are typed `blocks` edges (cycle-checked at write
time); decisions are dossiers that gate the work they block; every
mutation is validated against a closure discipline — closing a task
without evidence is an error, blocking one without a note is an error,
signing a decision without a resolution is an error. The board's
culture becomes the API's contract.

Determinism is the product property: the same store state answers
byte-identically across runs, every returned list carries a documented
total order, and the on-disk format is pinned by golden files. One
binary, four crates — `todo-domain` (the pure graph, no I/O),
`todo-store` (the JSONL engine behind a single `Store` trait),
`todo-mcp` (the server binary), `todo-cli` (a no-client inspector).

## Wiring it into opencode

Build the release binary and install a stable copy — the config names
the copy, not the checkout's `target/` (which `cargo clean` deletes).
The copy lands via a fresh inode (cp to a temp name, then `mv`), never
`cp` in place: on macOS, a signed binary modified in place while any
process executes it — an opencode session's server, normally — leaves
every NEW execution SIGKILL'd on a stale code-signature cache:

```sh
cargo build --release -p todo-mcp
cp target/release/todo-mcp ~/.local/bin/todo-mcp.next
mv ~/.local/bin/todo-mcp.next ~/.local/bin/todo-mcp
```

Then add an `mcp` entry to your global opencode config
(`~/.config/opencode/opencode.jsonc`). This is the live entry this
machine runs — substitute your own root path:

```jsonc
"todo-mcp": {
  "type": "local",
  "command": [
    "/Users/mouse/.local/bin/todo-mcp",
    "--root",
    "/Users/mouse/.local/share/dacode-todo"
  ],
  "enabled": true
}
```

Updating it is the same commands. No restart is needed: each opencode
session spawns its own server process, so the next session picks up
the new binary, and the rename leaves a running session's server on
its old inode, undisturbed. The staleness tradeoff is deliberate and
documented in the entry's comment in the live config: the installed
copy goes stale after pulls until re-copied, and the store-format
version gate (`version_mismatch`) makes a too-old binary refuse a
newer store rather than guessing.

A per-project mcp entry may pass a default repo slug (the `--repo`
flag), and absent one the server derives the default from the directory
the client spawns it in — the registered checkout that is the longest
component-wise ancestor-or-equal of the working directory. Explicit
tool parameters always win: parameter > flag > cwd.

## Where the data lives

Under `~/.local/share/dacode-todo/` (the same path on macOS; no XDG
relocation game): one `registry.json` holding the repo list and the
slug→store mapping, plus `repos/<slug>/` per repository — an
append-only `event-log.jsonl` and its covering snapshot. Local,
non-relational, no cross-project shared database; cross-repo queries
iterate the registry and open per-repo stores.

Inspect a store without any MCP client:

```sh
cargo run -p todo-cli -- dump   --root ~/.local/share/dacode-todo --slug dacode
cargo run -p todo-cli -- verify --root ~/.local/share/dacode-todo --slug dacode
cargo run -p todo-cli -- repair --root ~/.local/share/dacode-todo --slug dacode --dry-run
```

## The tool surface

Twenty-six tools over stdio, grouped: the registry
(`repo_register`/`repo_list`/`repo_show`/`repo_set_path`), epics, task
CRUD and listing (`task_list` filters by epic, status, kind and
deferred-readiness, each row carrying its `blocked_by` blockers), the
`blocks`/`relates` edges (`task_link`/`task_unlink`), the questions the
system exists for (`next_task`, `next_items`, `status_summary`),
decision dossiers, registered documents for stable citation,
`graph_show` (a complete handover dump of the graph), and `import_md`.
Writes return the new state, so a client never needs a
read-after-write round trip; errors are structured data with stable
codes, never bare strings.

The authoritative table — every tool, its parameters, the error code
list — is [docs/todo-mcp-design.md](docs/todo-mcp-design.md) §6. The
README groups; the spec is the contract.

## Adopting a repository

Register the repo (the path is its checkout — the importer reads the
board from there), import its board, verify, then cut over:

```jsonc
// repo_register: { "slug": "myrepo", "name": "myrepo", "path": "/abs/checkout" }
// import_md:     { "repo": "myrepo", "dialect": "dacode" }
```

The importer maps a board once, read-only: milestones and epics-in-prose
become epics, `U` items become tasks (status from the checkbox and
strikethrough state), open decisions become decision nodes with `blocks`
edges to the work they gate, dependency prose becomes edges where
mechanical. Importing in board id order reproduces the board's
numbering — `U1`…`U170` stays `U1`…`U170`.

Dialects: `dacode` (the three-file board under `docs/implementation/`,
the only one with a checked-in fixture, at
`crates/todo-mcp/tests/fixtures/dacode/`), `camera`, `ikvm-html`,
`ipmi-collector`, and `dacode-todo-mcp` (this repo's own board). Live
boards are never copied
into this repository: the importer reads them from their checkouts at
import time, so the board stays the source of record for prose the
server does not model.

After an import is verified against its board, the server is that
repo's query surface and the markdown keeps the narrative — session
logs stay files. There is no live two-way sync: re-run `import_md`
after the board moves (the importer refuses a non-empty graph; the
refresh is a re-register plus re-import — `todo-mcp refresh --repo
<slug>`).

Four repositories are tracked for real on this machine — dacode,
camera, ikvm-html, ipmi-collector — plus this repo itself.

### The cutover: making the server the tracker of record

The import leaves the board as the tracker. When a repo is ready to
switch, the playbook (executed for dacode 2026-10-09 and ipmi-collector
2026-10-10):

1. **Refresh before archiving** — the store must match the board's
   final state by construction, not by inference: `todo-mcp refresh
   --repo <slug>` (a re-register plus re-import; it refuses once the
   board files are gone). Verify with `todo-cli verify`.
2. **Back the board up beside the store** —
   `~/.local/share/dacode-todo/board-backup/<slug>/<date>/`, the board
   files copied byte-identical. The pre-cutover state now survives
   outside every repo.
3. **Archive in-repo** — `git mv` the board files to
   `docs/archive/board-<date>/` (git records 100% renames). The
   archive is a frozen backup agents never read.
4. **Rewire the harness** — `todo-mcp*: allow` plus `todowrite: deny`
   on the working roles' front matter; the server is served by the
   global mcp entry, so no project entry (it would shadow the global
   one).
5. **Rewrite the instructions** — AGENTS.md's tracking sections, the
   agent prompts, the knowledge files: the cold start becomes
   `HANDOVER.md` → `decision_list` / `next_task` / `status_summary`;
   the exit gate becomes the store moving with evidence. Retire the
   board-reading commands and skill with the board.
6. **Write `docs/implementation/HANDOVER.md`** — the one prose record
   a cold session reads before asking the server, rewritten at the end
   of every session. Record the id vocabulary: the store's task ids
   are the server's own numbering (a gapped board register cannot be
   reproduced), pre-cutover nodes carry `board:<id>` in `references`,
   and new work takes the next server id.
7. **Smoke** — a real opencode session in that repo driving the cold
   start end to end: HANDOVER read, the server queried, the quick
   gates run, `next_task` answering the current work, every
   `todo-mcp_*` call allowed by the role override.

Two traps worth writing into the adopted repo's HANDOVER, both
measured: the harness mangles enum-typed MCP tool params (`status`,
`edge`, `kind` — reads are unaffected; enum-carrying writes fall back
to the server's stdio interface), and a stdio one-shot client can
leave its server process alive holding the store's write lock
(`ps aux | grep todo-mcp`; kill the orphans, not the opencode
children).

## The progress page

`tools/progress.py` renders the tracked graph to a page and a JSON
twin: the static invocation writes `.agents/cache/progress.{html,json}`,
`--json` prints the document, `--refresh` re-imports this repo's board
first (the store is a derived cache of the board), and `--serve` serves
the page live on 127.0.0.1 — a fresh render per request, the port
published to a per-user runtime file so a second invocation attaches
instead of starting a second listener.

## Verifying the wiring

The honest test is following this section on the machine that runs the
server:

```sh
cargo build --release -p todo-mcp
cargo run -p todo-cli -- verify --root ~/.local/share/dacode-todo --slug dacode
python3 tools/progress.py --no-write
```

The last command is the end-to-end probe: it spawns the release binary
over stdio, drives the MCP handshake exactly like a client, asks one
`graph_show`, and prints the summary — a missing or stale binary fails
the probe by name. Green looks like `verify` exiting 0 with
`status ok` and the probe printing the totals. One live failure mode is
expected, not broken wiring: while an opencode session's server has the
stores open (it caches handles for its process lifetime — the
single-writer lock discipline), `verify` and the probe answer
`store is locked`; re-run once that session has ended. From inside
opencode, the equivalent is asking the agent for `repo_list` — the
tracked repos with their checkout paths (dacode, camera, ikvm-html,
and this one).

## Reading more

- [docs/todo-mcp-design.md](docs/todo-mcp-design.md) — the spec,
  normative throughout: the domain model, the storage format, the tool
  surface, the determinism discipline.
- [docs/implementation/TODO.md](docs/implementation/TODO.md) — the
  project board: what has landed, what is next, what is blocking.
- [docs/implementation/decisions.md](docs/implementation/decisions.md)
  — the sign-off record for every design decision.
- [AGENTS.md](AGENTS.md) — repository-wide instructions for agents
  working in this checkout.
