---
name: todo-next-imp
description: "Resume dacode-todo-mcp work from a cold start. Reads the store (the tracker of record) and the live epic's handover, reports what is blocked on what, proposes the single next actionable step, and enforces the exit gate (nodes closed with evidence, the sweep run, the handover moved, checks green) before work counts as delivered. Use at the start of a session, or any time you need to know what to work on next in this repo."
---

# todo-next-imp — find, start and *close* the next step

## Purpose

This skill answers two questions, and the second is the one that gets
skipped:

1. **What is the next thing to do, and is anything blocking it?**
2. **What must be true before that work counts as delivered?**

It exists because the project state is spread across a store, a
handover file, a decision record and a spec, and a cold session should
not have to rediscover how they relate — or quietly finish work
without updating any of them.

It is a **read-and-report** skill first. Do not start implementing until
step 5 has produced an agreed next step.

## How it is invoked

Two slash commands load this skill and inject the live state, so the
reads in steps 1–3 arrive already done:

| Command | Runs |
|---|---|
| `/todo-next-imp` | The whole procedure below, ending at step 5 — propose, then stop |
| `/todo-close` | The **exit gate** section only, over the current unit of work |

The commands are generated from `.agents/commands/*.md`; when invoked
directly rather than through them, run steps 1–3 yourself.

**Prefer unfinished work over new work.** An open store node, a node
closed on evidence that does not name a command, a handover whose
"landed" claim has no commit behind it — all of those come before
anything new. A half-finished unit of work is the most expensive thing
in the repository, and the reason this skill reads the store before
anything else.

## Procedure

Run these in order. Steps 1–3 are reads and can be batched.

### 1. Read the store (the tracker of record)

This repo's nodes live in the server's own store, under the
`dacode-todo-mcp` slug in the durable root. Query it; do not parse
anything:

```
todo-mcp --root ~/.local/share/dacode-todo progress --json
```

That document carries the totals, the open items, the decision count
and the per-epic roll-up. For the live detail, use the wired
`todo-mcp_*` MCP tools (this role's allow covers them):

- `status_summary` — the open/closed roll-up per epic
- `next_task` — the dependency-correct answer to "what now"
- `task_list` / `task_show` — the open nodes, their `verify` and their
  evidence

Beside them, the harness checks:

```
tools/agents-sync.py --check
tools/check-links.py --quiet
```

### 2. Read the live epic's handover

`docs/implementation/handover/E<n>.md` — one file per epic, keyed by
the epic's number. Sessions are epic-scoped (the work-stream rule), so
the file a cold session reads first is its own epic's; the most
recently modified file is the live arc's. It carries the handover
shape — workspace, landed, next step, blocked on the user, unverified —
and is usually a more honest account of where things stand than any
roll-up.

### 3. Check the working tree

```bash
git status --short | head
```

A clean tree with open nodes means recorded work not yet committed, or
committed work the nodes do not know about — say which.

### 4. Determine the phase

- **Pre-M0** — §12 decisions are open and no Cargo workspace exists.
  Almost no code can start; the work is decisions, spec depth, and
  measurements that decisions ask for. Do not scaffold crates past §2's
  proposal.
- **M0** — decisions gating the workspace are signed, the skeleton
  exists, the gates run. Code shape is still thin; the spec is the
  precedent.
- **M1/M2+** — crates carry real logic. Now the code gates
  (fmt/check/test/clippy/doc) apply to everything, and golden files exist to
  break.

Check which by looking at the repo itself, not at any doc:

```bash
ls Cargo.toml crates 2>/dev/null
```

If there is no `Cargo.toml`, say so in the report — it means every code
gate this session will be `skipped` by construction, which changes what
"delivered" can mean.

### 5. Propose the next step, then stop and agree

Emit the report below and **wait for confirmation** before doing the
work. The exception is a step the user has already asked for in this
session.

## Report format

```
Phase:            <pre-M0 | M0 | M1 | …>
Workspace:        <crates that exist, toolchain, dirty state>
Gate:             <the harness checks, plus the store's open items and any decision waiting on the user>
Blocking now:     <open nodes needing the user or a decision, each with what it gates>
Ready to start:   <candidate next steps, dependency-ordered>
Proposed next:    <one step, with its verification method>
Deferred/blocked: <what you are deliberately not doing and why>
Tracker:          <the store's open nodes and the sweep result — what is unfinished, what lacks evidence>
```

Seven or eight lines plus whatever detail the proposed step needs. Cite
the node ids and decision ids; do not restate their contents.

## The exit gate — work is not delivered until this passes

This is the half that gets skipped, and the reason this skill exists.
A change that lands code without landing its state is a change the
next session will redo or contradict.

Before reporting any unit of work as done, in this order:

1. **Close the store nodes with real evidence.** Every unit of work
   this session finished is `task_update` → `status: closed` with
   `evidence` naming the command that was run and what it printed.
   The domain refuses a close without evidence, so the discipline is
   enforced at the write. A node whose `verify` was never set cannot
   close honestly — decide the verification when the node is *created*
   (`task_add` carries `verify`), and if it is not obvious, ask rather
   than inventing one.
2. **Run the sweep.** `status_summary` + `next_task` over the repo,
   read against what this session actually delivered. The store cannot
   see `git log` — your delivery knowledge is the cross-check.
   Disposition every open node you touched: closed with evidence,
   blocked with a note (the domain requires it), or left open with its
   consumer named. A node pending since before the session that
   nothing delivered against is either work still owed or a record
   gone stale — say which, and never report success over an
   undispositioned node.
3. **The epic's handover moved.** `docs/implementation/handover/
   E<n>.md` gains this session's entry in the handover shape. New
   findings and debt become store nodes (`task_add` with
   `kind: finding` / `kind: debt`), not prose — the handover is state,
   not a ledger. Reasoning does not go there either: it goes in
   `decisions.md` or the spec.
4. **If a decision was signed off:** strike its row in `decisions.md`'s
   sign-off summary, set its `Status`, record the outcome in its detail
   entry — and **execute its `On sign-off:` line**, which is a literal
   work order. Most of them include "delete §12 item N", so the spec
   moves in the same change.
5. **If a fact was established the hard way** — a benchmark number, a
   crate-API constraint, a format edge case — it goes into the **spec's
   numbered sections**, not only into a commit message. That is the
   expensive part of the design phase and it must survive the session.
6. **`tools/check-links.py`** passes (mandatory after adding, removing or
   renaming a doc) and the harness checks report no *new* failure.
7. **Code gates, if and only if code was written**: `cargo fmt --check` →
   `cargo check --workspace --all-targets` → `cargo test -p <touched
   crate>` → `cargo clippy --workspace --all-targets` → `cargo doc
   --workspace --no-deps`, run twice — plain and with
   `--document-private-items`
   (`.agents/knowledge/workflows/build-and-test.md`). Dual reviewer
   sign-off before the commit; docs and `.agents/` are review-exempt.
8. **Commit.** It is a step of the cycle, not a favour. Inspect
   `git status` and `git diff` first, stage only intended files, code
   commits carry both role `Reviewed-by:` trailers (only after a real
   dual review), report the hash.
9. **Hand over** — write the next session's starting point, in the shape
   below, both in chat and appended to the epic's handover file. This
   is the last step, not an optional courtesy.

If a step could not be run, report it as `skipped` and say why. Never
imply a check passed that you did not run.

## Handover — the last thing a session produces

Every session ends with a handover, in chat *and* appended to the epic's
handover file. The two are the same content; the chat copy is for the
user, the file copy is for whoever opens this repository next —
possibly on another machine, certainly without this context window.

It is not a summary of what you did. The store and the commit log
already record that. It is **the state a cold session needs before it
can act**, and it is written for a reader who knows the project and not
the session.

```
Workspace:    branch, clean or dirty, which crates exist, toolchain
              version, anything the next session must install first
Landed:       what was delivered, with the commit hash — or plainly
              "nothing landed"
Next step:    the one thing to do next, why it and not the alternative,
              and what it needs that is not already true
Blocked on you: the open decisions, each with what it gates
Unverified:   what is asserted but not proven — a gate that was skipped,
              a behaviour never smoke-tested against a real client
```

Three rules that make it worth reading:

- **State what is unverified before what is finished.** The expensive
  failure here is a next session trusting a claim that was never tested.
- **Name the next step and its alternative.** "D2 next" is an
  instruction; "D1 first, then D2, because the engine choice decides what
  the graph layer may assume" is a decision the next session can disagree
  with.
- **Prefer a command to a description.** The exact gate invocation, the
  smoke-test request JSON, the fixture path. A session that has to
  reconstruct an incantation has already lost the time the handover was
  meant to save.

## Hard rules

1. **Never write code past an unsigned gate.** If a decision is open,
   the code that presumes its outcome does not get written. Surface
   the gate instead of guessing.
2. **Never deepen a spec section whose decision is still open** in a way
   that presumes the outcome. Options may be sharpened; the choice may
   not be pre-empted.
3. **The store is the tracker of record — tracking is mandatory**, not
   a judgement call. Open a node (`task_add` with `verify`) whenever
   the work has more than three steps, spans more than one file,
   spawns a subagent, records a finding, or takes on debt. Arcs are
   epics (`epic_add`); work units, findings and debt are tasks with
   the right `kind`.
4. **The handover is an exit gate on every delivery.** A change is not
   delivered until the epic's handover reflects it — including debt and
   findings discovered mid-work, tasks that had to be split, and steps
   found during implementation.
5. **Evidence over inference.** A performance claim, a compatibility
   claim or a "the client accepts it" claim is backed by a command that
   was run and cited, or it is marked as inference.
6. **The spec moves only by user decision.** §12 is never settled by an
   agent; a signed decision executes its `On sign-off:` work order in
   the same change.
7. **No AI attribution, no forged review trailers.** Not in a commit
   message, a comment, or a doc. The two role trailers only after a real
   dual review of this exact diff.
8. **Adding an external crate, changing the wire/tool surface, or
   settling a §12 decision all require asking first.**
9. **Dogfood the design you shipped.** The server's own decisions are
   signed, the roadmap is delivered, and this repo cut over to its own
   server as the tracker of record (the user's 2026-10-09 sign). The
   store is not a demo — it is this repo's tracking surface, and its
   determinism guarantees are the ones your session records depend on.

## The record homes — do not conflate them

| | the store (this repo's nodes) | `decisions.md` + the spec |
|---|---|---|
| Lives at | the durable root, per-repo | this repo, committed |
| Scope | arcs, work units, findings, debt | signed decisions, the normative design |
| Updated by | `task_add` / `task_update` through the wired tools | whoever delivers the decision's work order |
| Survives | everything (the durable root) | everything (git) |
| Answers | "what is open, what is verified, what is next" | "what was decided, and why" |

The handover file is the narrative surface — state, not reasoning.
An instruction to "record it" is ambiguous between the store and the
decision record: if context does not settle it, **ask** — recording in
the wrong home silently loses work.

## Where things live

| Need | Path |
|---|---|
| The spec (model, storage, surface) | `docs/todo-mcp-design.md` |
| The tracker of record (this repo's nodes) | the store — `todo-mcp --root ~/.local/share/dacode-todo progress --json`, or the wired `todo-mcp_*` tools |
| The live epic's handover | `docs/implementation/handover/E<n>.md` |
| Open decisions + recommendations | `docs/implementation/decisions.md` |
| Domain knowledge | `.agents/skills/dacode-todo-mcp/SKILL.md` |
| Coding conventions | `.agents/roles/_rust.md` |
| The tracking conventions | `.agents/knowledge/workflows/task-tracking.md` |
| Search policy | `.agents/roles/_search.md` |
| Repo-wide agent instructions | `AGENTS.md` |

## Before writing code

Load the `dacode-todo-mcp` skill for domain knowledge and read
`.agents/roles/_rust.md` for conventions — no `unwrap()` on production
paths, crate boundaries per spec §2, determinism discipline (BTreeMap on
anything persisted, total orders, golden files), `///` on every public
item.

Then read the spec section the task implements. On this project that
single step saves more time than everything else in this skill combined:
the spec's shapes are the precedent while the codebase is young, and §12
tells you which shapes you are not allowed to assume yet.
