# AGENTS.md — dacode-todo-mcp

Repository-wide instructions. opencode appends this to every agent's
context, so it is additive to the generated agent prompts under
`.opencode/agent/` — it does not replace them.

If `MYAGENTS.md` exists at the repo root, it is untracked, personal, and
**overrides** this file where they conflict.

## User Personalization

- Be direct. Correct a wrong premise rather than working around it.
- No superlatives, no validation, no closing summaries. Report outcomes
  and `path:line`.
- State what was verified and what was not. A gate you did not run is
  `skipped` with a reason, never silence.

## Project State

**Phase: M4 (adoption) is closed — g11, the exit gate, ran green
2026-10-07 — and M5 (packaging, docs, cross-repo ergonomics if D8
arms them) is next, its decomposition the next session's first
act.** The gate ran every line of its verify method on the committed
tree: every code gate (543 workspace tests), the golden ritual
byte-identical over all 24 goldens (19 mcp including the fixture
golden, 4 store, 1 cli), the §6 tool-surface check (a live
tools/list drive: exactly the spec table's 26 tools, import_md
included), the live-run evidence (t46/t47's probes + t48's real
imports), f102 swept in wire.rs (dual-reviewed), the f41 sweep (f99
promoted to U32), and M4's lessons consolidated into
.agents/pitfalls.md (4.7, 4.8, 5.11–5.13; top rules 15–16). The M4
deliveries, oldest first: the dialect study and D18 signed (a); t42
the dacode parser — parse_board total over the live board; t43 the
D18 mapping — the ImportPlan + apply_plan with id reproduction
pinned at the live board's scale; t44 the import_md wire tool — the
importer cluster with the ApplyError refusal seam and the
import_md.v1 golden; t45 the checked-in dacode fixture + the
end-to-end golden — the three real files per D12/§10, the first
composed run on the real board clean, dacode_dump.v1 pinning the
expected graph; t46 the camera dialect — the study first, the
sibling module, the dialect param on import_md, the live probe at
178 tasks with no golden per D12; t50 the U31 lift — the plan types
and the shared helpers in dialect-neutral homes under src/import/
(plan.rs + helpers.rs behind a HelperError seam), zero behavior
change, U31 resolved; t47 the ikvm-html dialect — the study, the
third sibling module on the neutral homes, the ikvm-html dialect
value, the live probe at 132 tasks with no golden per D12; and t48
the three repos tracked for real — the §9 cutover executed:
dacode/camera/ikvm-html registered in the durable D4 root
(~/.local/share/dacode-todo/), every import clean on the first
composed run (dacode's census = the golden's, camera = its probe's,
ikvm-html = the predicted post-U129 count), all three stores
verified, the query session recorded and byte-identical across two
runs.**
An MCP server that replaces
markdown TODO boards with a deterministic task graph: tasks are typed
nodes, dependencies are typed edges, everything lives in local
per-repo non-relational stores (D4), and "what is next in repo X under
epic Y" is a query, not a prose parse. Rust, rmcp, stdio transport.

§12 is closed: **D18** (the dacode-dialect mapping — the importer's
semantics for the five populations §9's signed table did not cover)
was signed (a) 2026-10-05, drafted and signed in one exchange out of
the M4 dialect study; its rows are in §9's mapping table and its
conventions ride t43's task spec. **D17**
(priority on the triage surfaces — the echo on
`next_task`/`next_items`/`task_list`, the order untouched) was signed
2026-10-04, the last decision of M3, and **D13 stays reserved** for the
transport question board U18 defers. Everything else is signed: all
eleven decisions on 2026-09-25, D12 added 2026-09-29, D14/D15 on
2026-09-30 out of U2's rmcp survey (the wire's canonical key order and
§6.1's error carrier), and D16 on 2026-10-03 (the progress roll-up)
(`docs/implementation/decisions.md`); the spec is normative
throughout, and signed shapes move only by a new user decision (new
D-entries), never by edit. M0 landed 2026-09-25: four crates, the
workspace lint regime, all code gates green. M1 landed 2026-09-30: the
domain model (§3's value types and closure discipline, §5's graph and
canonical order), the signed D1 JSONL store behind the `Store` trait with
the D4 layout, advisory locking, snapshots, compaction and §4.3's repair
primitive, and the `todo-cli` inspector — 253 tests, five goldens, every
gate green at the g5 exit. M2 landed 2026-10-02: the query layer — §5.2's
selection (`graph/select.rs`), batch (`graph/items.rs`) and listing
(`graph/list.rs`) surfaces, plus the registry queries
(`DataRoot::repo_list`/`repo_show`) — 322 tests, every gate green at the
g7 exit, M2's lessons consolidated into `.agents/pitfalls.md`. M3 landed
2026-10-04: the wire — 25 tools over rmcp (every §6 tool except
`import_md`, M4's per D12), 17 wire goldens, the D14/D15 pins, the §8
concurrency proof, the default-repo slug (D8), and a real opencode
session as the smoke (the server is wired into this repo's own harness
config, the orchestrator's native todo tool denied per U16b) — 423
tests, every gate green at the g9 exit, U26 reproduced/isolated/fixed
at the gate (U27 carries the store-side question), M3's lessons
consolidated into `.agents/pitfalls.md`. **M4 is next**: the
dacode-dialect importer first (D12), its board the only fixture
checked in per §10, camera and ikvm-html as live-test subjects never
copied into this repo, the three repos tracked for real — the
decomposition is the next session's first act. Run
`tools/attention.py` for the current picture rather than trusting this
paragraph.

## The three rules that are not negotiable

1. **No AI or tool attribution, anywhere.** Not in a commit message, a
   code comment, a markdown file, or a TODO entry. No `Co-authored-by:`
   for a tool, no "generated with", no "assisted by". The single
   exception is the two `Reviewed-by:` role trailers below, which name
   harness roles, never models.
2. **Never forge a review trailer.** Code commits carry
   `Reviewed-by: dacode-todo-mcp-reviewer-primary <primary@dacode.local>`
   and `Reviewed-by: dacode-todo-mcp-reviewer-secondary <secondary@dacode.local>`
   only after both reviewers actually returned `status: ok` on this exact
   diff. If the diff changed after a review, the review is stale —
   re-review or drop the trailers. Review-exempt commits (docs,
   `.agents/`, tracker, generated files) carry no trailer.
3. **Never write past an unsigned decision.** Spec §12 items are user
   decisions; `decisions.md` says what each gates. Agents propose, the
   user signs. Implementation that presumes an unsigned choice is a
   reject, not a nitpick — it will be invalidated when the user decides.

## Slash commands

| Command | What it does |
|---|---|
| `/todo-next-imp` | Runs every gate, reports what is blocked on what, proposes **one** next step, then stops for agreement. Prefers unfinished work over new work. Start every session here |
| `/todo-close` | Runs the exit gate over the current unit of work and reports what is still missing before it counts as delivered |

Both are generated from `.agents/commands/*.md` by `tools/agents-sync.py`
and inject the live state (attention table, board, tracker, workspace and
harness status) into the prompt, so neither costs a round of exploration.

## Quick Reference

```bash
# What needs the user's attention, and whether the docs are consistent
tools/attention.py
tools/attention.py --check

# Total progress: the dependency graph, the totals, the open items
tools/progress.py            # writes .agents/cache/progress.{html,json}
tools/progress.py --open     # ...and opens the page in a browser
tools/progress.py --json     # machine-readable, for other tools

# Session task tree (gitignored, branch-keyed).
# Convention: .agents/knowledge/workflows/task-tracking.md
tools/tasks.py show -v
tools/tasks.py add - goal "..." --verify "..."
tools/tasks.py add g1 task "one commit" --verify "cargo test -p <crate> <name>"
tools/tasks.py set t1 --status blocked --note "why"     # a block needs a reason
tools/tasks.py set t1 --status done --evidence "what you actually ran"
tools/tasks.py check

# Harness self-checks
tools/agents-sync.py --check        # .opencode/ matches agent-rules.yaml
tools/agents-index.py --check       # .agents/knowledge/INDEX.md is current
tools/check-links.py                # every markdown link resolves

# Code gates, once the workspace exists (order matters; pipe long output)
cargo fmt --check
cargo check --workspace --all-targets 2>&1 | tail -c 5000
cargo test -p <crate> 2>&1 | tail -c 5000
cargo clippy --workspace --all-targets 2>&1 | tail -c 5000
cargo doc --workspace --no-deps 2>&1 | tail -c 5000
cargo doc --workspace --no-deps --document-private-items 2>&1 | tail -c 5000
```

## Project Overview

The three sibling projects this harness is adopted from — camera
(`/Users/mouse/RPM/shelly/camera`), ikvm-html
(`/Users/mouse/RPM/ali/ikvm-html`), dacode
(`/Users/mouse/RPM/dacode/dacode/dacode`) — each run a markdown task
system with real semantics (stable `U` ids, decisions that gate
milestones, evidence-backed closure). Those are excellent audit trails
and poor query surfaces. This server keeps the semantics and makes every
question machine-answerable; determinism is the product property (spec
§8), and the domain skill
(`.agents/skills/dacode-todo-mcp/SKILL.md`) is the short map.

The spec is `docs/todo-mcp-design.md`, normative throughout (§12 closed
2026-09-25; a new decision opens a new D-entry there, mirrored by a new
§12 item). When code and spec disagree, surface the disagreement; the
spec moves only by user decision.

## Repository Structure

```
.
├── AGENTS.md                       # this file
├── .agents/
│   ├── config/agent-rules.yaml     # SINGLE SOURCE OF TRUTH: roles, models, commands, gates
│   ├── roles/                      # prompt layers: _common, _search, _rust, role bodies
│   ├── commands/                   # slash-command bodies: todo-next-imp, todo-close
│   ├── skills/                     # dacode-todo-mcp, todo-next-imp, md-writer
│   ├── skill-base.md               # protocols every skill inherits
│   ├── knowledge/                  # workflows, patterns, generated INDEX.md
│   ├── pitfalls.md                 # confirmed lessons, promoted from pitfalls-live at each milestone gate (never seeded with guesses)
│   ├── pitfalls-live.md            # adaptive, in-flight lessons
│   ├── task-templates/             # implement-file, implement-crate, write-tests, review-code
│   ├── sgrules/                    # verified ast-grep rules (sgconfig.yml)
│   └── cache/                      # gitignored: ATTENTION.md, tasks/<branch>.yaml
├── .opencode/                      # GENERATED by tools/agents-sync.py — do not hand-edit
│   ├── opencode.json               #   provider models + MCP servers + tool gating
│   ├── agent/                      #   the five agents
│   └── commands/                   #   the slash commands
├── docs/
│   ├── todo-mcp-design.md          # THE spec; §12 mirrors decisions (closed); no links by convention
│   └── implementation/
│       ├── README.md               # plan index: which file holds Truth / Reasoning / State
│       ├── TODO.md                 # the committed board
│       └── decisions.md            # sign-off record for §12
├── tools/                          # agents-sync, tasks, agents-index, check-links, attention, progress
├── crates/                         # the Cargo workspace — created at M0, not before
└── tmp/                            # scratch, gitignored
```

## Coding Guidelines

The full layer is `.agents/roles/_rust.md`, inlined into every coding
agent. rustfmt and clippy are the mechanical authorities; the spec's
shapes are the precedent while the codebase is young. The five that
reviewers enforce by hand:

1. No `unwrap()`/`expect()`/`panic!` on production paths — the denied
   lints are workspace lints, inherited per crate; a new crate without
   the inheritance is itself a finding. There are nine, not the "six"
   this item used to claim: seven clippy (the six panic-family —
   `unwrap_used`, `expect_used`, `panic`, `unreachable`,
   `indexing_slicing`, `exit` — plus `wildcard_imports`) and two rustc
   (`unsafe_code`, `missing_docs`). `Cargo.toml`'s
   `[workspace.lints]` is the enumeration and spec §10 delegates to
   `.agents/roles/_rust.md`, so no prose here needs a count that a tenth
   lint would falsify.
2. Crate boundaries (spec §2): domain has no I/O, no async, no workspace
   deps; the storage engine never leaks above the `Store` trait; the
   binary translates and does not decide.
3. Determinism discipline (spec §8): `BTreeMap`/`BTreeSet` for anything
   reaching disk or wire, a documented total order on every list, golden
   files pinning serialized bytes.
4. `///` on every public item; comments explain non-obvious rationale or
   nothing.
5. Files under 1000 **production** lines, target 600 — a `#[cfg(test)]`
   block and a test-only file are not counted, and an over-target file is
   an exception recorded in its module doc and on board §5 (the cap is not
   waivable). Unit tests in-file, integration under `crates/<crate>/tests/`.

## Code search: pick the layer

Four layers, cheapest first — map (spec §2, `INDEX.md`, crate
`AGENTS.md`) → lexical (`rg`, scoped) → structural (`ast-grep`) →
semantic (Serena/rust-analyzer). Zero results means escalate a layer,
not rephrase. Full policy: `.agents/roles/_search.md`.

## Boundaries

### Always Do

- Read the spec section a change implements before writing it.
- Report `skipped` for any check you did not run.
- Update `docs/implementation/TODO.md` in the same change that delivers.
- Put a fact you established the hard way into the spec's numbered
  sections, not only into a commit message.
- Record findings and debt in the tracker the moment they appear.

### Ask First

- Settling a §12 open decision, or deepening a section in a way that
  presumes one.
- Adding any external crate dependency.
- Changing the MCP tool surface or the on-disk format once signed.
- Scaffolding a crate the signed spec does not name.
- `git push`, `git reset`, `--amend`, `rebase -i`, force-push, branch
  deletion, `git config`, `--no-verify`.

### Never Do

- Mention an LLM, an AI assistant, a model, or that anything was
  generated or assisted — in anything.
- Add a `Reviewed-by:` trailer for a review that did not happen.
- Commit secrets, `target/`, `tmp/`, `.agents/cache/`, or
  `.opencode/node_modules/`.
- Hand-edit `.opencode/`, `.mcp.json`, or `.agents/knowledge/INDEX.md` —
  all are generated; edit `agent-rules.yaml` and re-sync.
- Bind anything to `0.0.0.0`; `127.0.0.1` only.
- Rewrite a golden fixture without calling the diff out as the
  compatibility event it is.
- Dogfood this server for this repo's tracking before M3 (spec §9).

## Committing — a required step of the cycle

When a unit of work is complete and verified, it ends in a commit. The
orchestrator owns that step after dual review; a directly-invoked worker
commits once its own verification passes. Reviewers never commit.

One convention, one repo:

- Subject ≤72 chars, imperative, no trailing period; scope prefix where
  it helps (`todo-store: add JSONL event log`).
- `knowledge:` / `docs:` prefixes for knowledge-base and doc changes
  (review-exempt, no trailer).
- Body wrapped at 72: the problem, why this approach, what it does not
  do.
- Code commits: both role `Reviewed-by:` trailers, and only after a real
  dual review of this exact diff.
- Inspect `git status`/`git diff` first; stage only intended files; one
  logical change per commit; report the hash.

## Common Workflows

### Starting a session

Run `/todo-next-imp`. It runs every gate, reads the board and the
decision record, reports what is blocked on what, and proposes one next
step — then stops for agreement. It prefers **unfinished** work to new
work, because a half-done unit is the most expensive thing in the
repository.

### Finishing a unit of work

Run `/todo-close`. Work is not delivered until the tracker is closed with
real evidence, `tools/tasks.py check` exits 0, the board has moved, any
hard-won fact has landed in the spec, and the gates have run or been
reported `skipped` with a reason.

### Implementing a piece of the server

1. Read the spec section, the crate `AGENTS.md` (via `INDEX.md`), and
   the nearest existing pattern.
2. Decompose so one task is one reviewable commit, files never shared
   between two tasks.
3. Implement + tests (templates: `.agents/task-templates/`).
4. fmt → check → test → clippy → doc (both invocations); golden/protocol
   fixtures in the same commit.
5. Dual review, then commit with the trailers.

### Fixing the harness

`.agents/config/agent-rules.yaml` is the only place model IDs may appear.
Edit it, bump `version.custom`, then run `tools/agents-sync.py`. Never
hand-edit `.opencode/` — `agent/`, `commands/` and `opencode.json` are
generated, and `--check` will catch the drift.

A new slash command is two edits: a `commands:` entry in the config and a
body at `.agents/commands/<name>.md`. The generator validates that the
`agent:` names a configured role, because a typo there fails silently in
the TUI.

Model bindings note: `harness.reasoning_values` records which
`reasoning:` values each bound model accepts, with the evidence for each
row. A spawn failure that kills exactly one role is usually a wrong value
there — regenerate, confirm the front matter changed, then **start a new
session**; a fix does not take effect mid-session.

## Key Files Reference

| Need | Path |
|---|---|
| Scope, domain model, storage, tool surface, §12, roadmap | `docs/todo-mcp-design.md` |
| What has landed and what is next | `docs/implementation/TODO.md` |
| Sign-off record for open decisions | `docs/implementation/decisions.md` |
| Prioritized "what needs you" | `tools/attention.py` → `.agents/cache/ATTENTION.md` |
| Total progress: graph, totals, open items | `tools/progress.py` → `.agents/cache/progress.{html,json}` |
| Roles, models, commands, gates, prohibitions | `.agents/config/agent-rules.yaml` |
| The three trackers and their conventions | `.agents/knowledge/workflows/task-tracking.md` |
| Rust conventions, architecture rules, determinism | `.agents/roles/_rust.md` |
| Search policy | `.agents/roles/_search.md` |
| Domain knowledge | `.agents/skills/dacode-todo-mcp/SKILL.md` |
| Build gates on this host | `.agents/knowledge/workflows/build-and-test.md` |
| Cold start / exit gate | `.agents/skills/todo-next-imp/SKILL.md` |
