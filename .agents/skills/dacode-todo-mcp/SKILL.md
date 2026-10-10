---
name: dacode-todo-mcp
description: "Domain knowledge for the dacode-todo-mcp server: the task-graph model, storage engine constraints, determinism discipline, the MCP tool surface, and the repo/epic registry. Load before reading, writing or reviewing any code in this repo."
---

# dacode-todo-mcp — Domain Knowledge

## Inherited Base Instructions

This skill inherits the mandatory protocols defined in
[`.agents/skill-base.md`](../../skill-base.md):

- **Pre-Task Knowledge Search** — spec section, nearest precedent, knowledge/, pitfalls-live.md before starting
- **Canonical Commands** — cargo gates / `tools/*.py`; never hand-rolled equivalents
- **Mandatory Review** — dual sign-off before every code commit; role trailers only after a real review
- **No AI attribution** — in anything, ever
- **Post-Task Knowledge Evolution** — capture patterns, workflows, pitfalls, and hard-won facts

**Read `.agents/skill-base.md` in full before resuming work in this skill.**

## What this project is

One MCP server that replaces markdown TODO boards with a deterministic
task graph. The three sibling projects it learns from — camera
(`/Users/mouse/RPM/shelly/camera`), ikvm-html (`/Users/mouse/RPM/ali/ikvm-html`)
and dacode (`/Users/mouse/RPM/dacode/dacode/dacode`) — each run a
U-id/D-id/M-id markdown system with real semantics: stable ids that are
never renumbered, decisions that gate milestones, evidence-backed
closure. Those systems are excellent audit trails and poor query
surfaces. This server keeps the semantics and makes every question
machine-answerable: *what is the next actionable task, in repo X, under
epic Y, given the dependency graph?*

The spec is `docs/todo-mcp-design.md`. This skill is the short map; the
spec is the territory. §12 is **closed** — all eleven decisions were
signed 2026-09-25, so the spec is normative throughout; a new decision
opens a new D-entry (D12+), and signed shapes move only by a new user
decision, never by edit.

## The model in one paragraph

A **repository** is a stable id with a mutable path attribute (checkouts
move; the tracker must not care). A repository's tasks are grouped under
**epics** (top-level, selectable). A **task** is a typed node with a
status, timestamps and an evidence field for closure. **Edges** are typed:
`parent` (repo → epic → task, a tree) and `blocks` (a DAG, enforced at
write time). Ordering is never stored — it is derived topologically, and
**next-task selection is a pure, documented function of graph state**
(§5); the tiebreaker (D6): topology → epic registration order → task
id. Everything that reaches disk or wire is canonically serialized and
pinned by golden files (§8, §10). Since the 2026-09-25 sign-off the
model has five node kinds (D10): repo, epic, task, **decision** (a
structured dossier whose open state can `block` work — "never write
past an unsigned decision", graph-enforced) and **doc** (a registered
document; the prose body stays in git). Statuses (D9): `open`,
`in_work`, `blocked`, `deferred`, `closed`, `wontfix`; deferred items
record until-targets as `blocks` edges, never auto-reopen. Epics and
tasks carry `references` lists (tracker keys, URLs, spec cites).
`next_items` (D11) returns a ranked actionable batch — repo/epic- or
task-scoped — annotated with what blocks what, a derived
transitive-dependent weight, and a deferred summary (`total`/`ready`,
D9); `graph_show` (§6.2) dumps the complete graph as canonical-JSON
nodes with per-node adjacency; the agent-set `priority` field is
display-only and never reorders. **Work streams** (the user's (a)
settlement 2026-10-10): a session assigned one epic passes its `epic`
on every selection call (`next_task`, `next_items`, `task_list`,
`status_summary` all take it) — the base instructions carry the rule,
and parallel sessions scoped to different epics of one repo never
cross; there is deliberately no flag or server-side default (the
global mcp entry serves every session the same command, so a flag
cannot vary per session).

## The property that makes this project exist

**Determinism.** Two runs over unchanged state return byte-identical
answers. The failure modes reviewers hunt (reviewer.md checklist item 2):
`HashMap` iteration reaching disk or wire; a list returned without a
total order; a sort with equal keys and no tiebreaker; a wall-clock read
below the binary layer; non-canonical JSON field order. When in doubt,
the test is: run it twice, compare bytes.

## Storage — settled and pending

Settled (D1, signed 2026-09-25): append-only JSONL event log +
in-memory graph + periodic snapshots behind the `Store` trait; redb is
the named fallback, reachable only as a new decision. Layout (D4, user
amendment): per-repo directories under `~/.local/share/dacode-todo/`
plus a shared registry — no cross-project shared database. Fixed
requirements (§4), whatever happens:

- local, single file, no server process to query it
- survives a repository moving on the filesystem (paths are attributes,
  ids are keys — §7)
- corruption is a named error, never a panic
- the format is pinned by golden files; a format change is a migration
  with a version header, not an edit

## MCP surface — settled and pending

Settled (D5): the §6 tool list is the v1 surface and `todo-cli` ships;
the list includes `next_items` (D11) and the decision/doc tools (D10).
JSON schemas firm up against rmcp's macros in M3. Fixed (§6's frame): tools are thin
translations over domain operations; every tool takes a repo parameter
(an absent one resolves param > flag > cwd — the cwd tier armed at
t53, a deterministic longest-ancestor match per §7); epic is a filter
parameter wherever tasks are
listed; errors are structured data with stable codes; every list has a
documented order; the base instructions (U16a) carry the tool-failure
recovery rule — strict JSON arguments, read the error text, retry
once, then stop and tell the user (f119/f120's lesson). The server
speaks stdio transport first — it is what opencode and the other
clients here use.

## Crate map (normative — §2, D1–D5 signed)

| Crate | Job | Depends on |
|---|---|---|
| `todo-domain` | nodes, edges, graph invariants, selection, **no I/O, no async** | nothing |
| `todo-store` | persistence behind one `Store` trait; engine (D1) sealed inside | todo-domain |
| `todo-mcp` | the server binary: rmcp wire surface, translates, never decides | both |
| `todo-cli` (ships, D5) | inspect/debug the store without a client | domain + store |

## Verification for this domain

Beyond the standard gates (`fmt`/`check`/`test`/`clippy`/`doc`), this project
has two verification shapes of its own:

1. **Byte-stability**: serialize → load → serialize is identical; a tool
   called twice on unchanged state returns identical output.
2. **Protocol-level tool tests**: request in over the wire, response out
   — never by calling the handler function around it. The wire is the
   product.

The workspace exists (M0, 2026-09-25), so the code gates run on
every delivery — `cargo doc` twice, plain and with
`--document-private-items` — with the harness self-checks
(`attention.py --check`, `check-links.py`, `agents-sync.py --check`,
`agents-index.py --check`) alongside them; on docs-only changes the
harness self-checks carry the weight alone.

## Where things live

| Need | Path |
|---|---|
| The spec (model, storage, surface, roadmap) | `docs/todo-mcp-design.md` |
| Progress board (committed state) | `docs/implementation/TODO.md` |
| Decision record (D1–D11 signed) + dossiers | `docs/implementation/decisions.md` |
| Rust conventions, architecture rules | `.agents/roles/_rust.md` |
| Search policy | `.agents/roles/_search.md` |
| Build gates on this host | `.agents/knowledge/workflows/build-and-test.md` |
| The three trackers | `.agents/knowledge/workflows/task-tracking.md` |
| Cold start / exit gate | `.agents/skills/todo-next-imp/SKILL.md` |
