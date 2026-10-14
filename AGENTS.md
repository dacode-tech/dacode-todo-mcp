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

**Phase: complete — the M0–M5 roadmap delivered (g13 closed
2026-10-08), and this repo cut over to its own server as the tracker
of record on 2026-10-12 (g21's flip, the user's 2026-10-09 sign).**
The store under `~/.local/share/dacode-todo/` holds this repo's
nodes; the markdown board is archived at
`docs/archive/board-2026-10-12/` (a frozen backup agents never read);
session tracking runs through the wired `todo-mcp_*` tools, and the
live epic's handover lives under `docs/implementation/handover/`.
The current picture is a query, not a paragraph: `todo-mcp --root
~/.local/share/dacode-todo progress --json`. §12 is closed — all
decisions signed (`docs/implementation/decisions.md`); the spec is
normative throughout, and signed shapes move only by a new user
decision.

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
and inject the live state (the store, the live epic's handover,
workspace and harness status) into the prompt, so neither costs a round
of exploration.

## Quick Reference

```bash
# The tracker of record: this repo's nodes live in the server's own
# store (the full-switch cutover). Session tracking runs through the
# wired todo-mcp_* MCP tools (task_add / task_update / status_summary /
# next_task); the state document:
todo-mcp --root ~/.local/share/dacode-todo progress --json     # totals, open items, decisions

# Total progress: the dependency graph, the totals, the open items
# (the todo-mcp subcommands, D20; --root required, the repo resolves
#  --repo over the cwd match, the render reads the store lock-free)
todo-mcp --root ~/.local/share/dacode-todo progress            # writes the rendered repo's .agents/cache/progress.{html,json}
todo-mcp --root ~/.local/share/dacode-todo progress --open     # ...and opens the page in a browser
todo-mcp --root ~/.local/share/dacode-todo refresh             # re-register + re-import this repo's board (refuses once cut over — the store is primary)
todo-mcp --root ~/.local/share/dacode-todo serve               # the live page on 127.0.0.1 (per-root listener, <root>/serve.port; every session's server ensures it); serve --open attaches or binds it and opens the page — the live view's discovery

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
├── README.md                       # the adoption front door: wiring, store, tools, dialects
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
│   ├── archive/                    # frozen backups agents never read (the board, 2026-10-12)
│   └── implementation/
│       ├── README.md               # plan index: which file holds Truth / Reasoning / State
│       ├── decisions.md            # sign-off record for §12
│       └── handover/               # the per-epic HANDOVER files (E<n>.md)
├── tools/                          # agents-sync, agents-index, check-links
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

Three layers, cheapest first — map (spec §2, `INDEX.md`, crate
`AGENTS.md`) → lexical (`rg`, scoped) → structural (`ast-grep`); the
semantic layer is gone (serena removed 2026-10-14). Zero results means
escalate a layer, not rephrase. Full policy: `.agents/roles/_search.md`.

## Boundaries

### Always Do

- Read the spec section a change implements before writing it.
- Report `skipped` for any check you did not run.
- Move the store and the epic's handover in the same change that
  delivers.
- Put a fact you established the hard way into the spec's numbered
  sections, not only into a commit message.
- Record findings and debt in the store the moment they appear.

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
- Track this repo's work anywhere but the store (the cutover landed
  2026-10-12; the board is a frozen archive, not a tracker).

## Committing — a required step of the cycle

When a unit of work is complete and verified, it ends in a commit. The
orchestrator owns that step after dual review; a directly-invoked worker
commits once its own verification passes. Reviewers never commit.

One convention, one repo:

- Subject ≤72 chars, imperative, no trailing period; scope prefix where
  it helps (`todo-store: add JSONL event log`).
- `knowledge:` / `docs:` prefixes for knowledge-base and doc changes
  (review-exempt, no trailer).
- Body short and wrapped at 72: the problem and the approach in a few
  lines — the diff already says what changed. Spell out an omission
  only when it is surprising.
- Code commits: both role `Reviewed-by:` trailers, and only after a real
  dual review of this exact diff.
- Inspect `git status`/`git diff` first; stage only intended files; one
  logical change per commit; report the hash.

## Common Workflows

### Starting a session

Run `/todo-next-imp`. It reads the store (the tracker of record), the
live epic's handover and the decision record, runs the harness checks,
reports what is blocked on what, and proposes one next step — then
stops for agreement. It prefers **unfinished** work to new work, because
a half-done unit is the most expensive thing in the repository.

### Finishing a unit of work

Run `/todo-close`. Work is not delivered until the store nodes are
closed with real evidence, the sweep has run (`status_summary` +
`next_task` read against what the session delivered), the epic's
handover has moved, any hard-won fact has landed in the spec, and the
gates have run or been reported `skipped` with a reason.

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
| What is open, what is verified, what is next | the store — `todo-mcp --root ~/.local/share/dacode-todo progress --json`, or the wired `todo-mcp_*` tools |
| The live epic's handover | `docs/implementation/handover/E<n>.md` |
| Sign-off record for open decisions | `docs/implementation/decisions.md` |
| Total progress: graph, totals, open items | `todo-mcp progress` → `.agents/cache/progress.{html,json}` |
| Roles, models, commands, gates, prohibitions | `.agents/config/agent-rules.yaml` |
| The tracking conventions | `.agents/knowledge/workflows/task-tracking.md` |
| Rust conventions, architecture rules, determinism | `.agents/roles/_rust.md` |
| Search policy | `.agents/roles/_search.md` |
| Domain knowledge | `.agents/skills/dacode-todo-mcp/SKILL.md` |
| Build gates on this host | `.agents/knowledge/workflows/build-and-test.md` |
| Cold start / exit gate | `.agents/skills/todo-next-imp/SKILL.md` |
