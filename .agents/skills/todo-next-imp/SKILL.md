---
name: todo-next-imp
description: "Resume dacode-todo-mcp work from a cold start. Runs the attention gate, reads docs/implementation/TODO.md and decisions.md, reports what is blocked on what, proposes the single next actionable step, and enforces the exit gate (both trackers updated, checks green) before work counts as delivered. Use at the start of a session, or any time you need to know what to work on next in this repo."
---

# todo-next-imp — find, start and *close* the next step

## Purpose

This skill answers two questions, and the second is the one that gets
skipped:

1. **What is the next thing to do, and is anything blocking it?**
2. **What must be true before that work counts as delivered?**

It exists because the project state is spread across a spec, a decision
record, a committed board and a gitignored session tracker, and a cold
session should not have to rediscover how they relate — or quietly finish
work without updating any of them.

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

**Prefer unfinished work over new work.** An `active` or `blocked`
tracker item, a `done` item with no evidence, a board item whose change
landed without the board moving — all of those come before anything new.
A half-finished unit of work is the most expensive thing in the
repository, and the reason this skill reads the tracker before the
board.

## Procedure

Run these in order. Steps 1–3 are reads and can be batched.

### 1. Run the gate

```
tools/attention.py                # prints the table, refreshes .agents/cache/ATTENTION.md
tools/attention.py --check        # exit 1 if anything is P0 or any hygiene check fails
```

This is the only view spanning all four sources, and the only thing that
catches a §12 open decision with no matching `D` entry, a dangling `§`
cross-reference, or a strikethrough that disagrees with a `Status`
column. Read it before anything else: it tells you which docs are
currently inconsistent.

`--check` exiting non-zero while decisions are open is expected in the
design phase and is not a failure to fix — it means the user has
something to decide. Once implementation is under way it should be green,
and a red run means the docs have drifted from the code.

### 2. Read the board

```
docs/implementation/TODO.md
```

§1 is the current state and what is blocked on the user. §2 is the
milestone state. §3 is what has actually landed. §4 is unowned
follow-ups with stable `U` ids. §5 is debt. §6 is the session log —
**read the last entry**, which is usually a more honest account of where
things stand than the milestone table.

### 3. Check the gates and the tracker

```
tools/tasks.py show -v
tools/tasks.py check
tools/agents-sync.py --check
tools/check-links.py --quiet
```

An empty tracker at the start of a session is normal; a tracker with
`done` items lacking evidence is not.

### 4. Determine the phase

- **Pre-M0** — §12 decisions are open and no Cargo workspace exists.
  Almost no code can start; the work is decisions, spec depth, and
  measurements that decisions ask for. Do not scaffold crates past §2's
  proposal.
- **M0** — decisions gating the workspace are signed, the skeleton
  exists, the gates run. Code shape is still thin; the spec is the
  precedent.
- **M1/M2+** — crates carry real logic. Now the code gates
  (fmt/check/test/clippy) apply to everything, and golden files exist to
  break.

Check which by looking at the repo itself, not at the board:

```bash
ls Cargo.toml crates 2>/dev/null
git status --short | head
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
Gate:             <attention.py --check: N items at P0, M hygiene failures>
Blocking now:     <decision ids, each with what it gates and its action type>
Ready to start:   <candidate next steps, milestone-ordered>
Proposed next:    <one step, with its verification method>
Deferred/blocked: <what you are deliberately not doing and why>
Tracker:          <tasks.py check result — clean, or the items it flagged>
```

Seven or eight lines plus whatever detail the proposed step needs. Cite
the decisions; do not restate them.

## The exit gate — work is not delivered until this passes

This is the half that gets skipped, and the reason this skill exists
alongside `attention.py`. A change that lands code without landing its
state is a change the next session will redo or contradict.

Before reporting any unit of work as done, in this order:

1. **`tools/tasks.py set <id> --status done --evidence "<what you
   actually ran>"`.** Evidence is a command and its result, not a
   restatement of the title. An item whose `verify:` is still `TBD`
   cannot be closed — decide the verification when the item is *created*,
   and if it is not obvious, ask rather than inventing one.
2. **`tools/tasks.py check`** exits 0. If it does not, say so in the same
   message and name what it flagged. Never report success over a failing
   gate. **`check` is not the sweep**: it flags a `done` item with no
   evidence and a `blocked` item with no reason, and is blind to a
   `pending` item whose work already landed or a `blocked` item whose
   blocker is gone. So read the open list against `git log` and disposition
   what you find — `done` with evidence, `wontfix` with a note, or a `U` id
   on the board where the deferral must outlive this branch. Every delivery
   sweeps the items it consumed; a milestone gate sweeps the whole open
   list, which is the sweep M1 closed without.
3. **`docs/implementation/TODO.md` updated in the same change.** A status
   change is a one-line edit; new debt, findings and newly-discovered
   follow-ups get new `U` ids in §4/§5 (next unused number, never a
   renumber); the session narrative goes in §6. Reasoning does **not** go
   here — it goes in `decisions.md` or the spec.
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
   renaming a doc) and **`tools/attention.py --check`** reports no *new*
   hygiene failure.
7. **Code gates, if and only if code was written**: `cargo fmt --check` →
   `cargo check --workspace --all-targets` → `cargo test -p <touched
   crate>` → `cargo clippy --workspace --all-targets`. Dual reviewer
   sign-off before the commit; docs and `.agents/` are review-exempt.
8. **Commit.** It is a step of the cycle, not a favour. Inspect
   `git status` and `git diff` first, stage only intended files, code
   commits carry both role `Reviewed-by:` trailers (only after a real
   dual review), report the hash.
9. **Hand over** — write the next session's starting point, in the shape
   below, both in chat and as a `§6` entry. This is the last step, not
   an optional courtesy.

If a step could not be run, report it as `skipped` and say why. Never
imply a check passed that you did not run.

## Handover — the last thing a session produces

Every session ends with a handover, in chat *and* committed as the final
`docs/implementation/TODO.md` §6 entry. The two are the same content;
the chat copy is for the user, the committed copy is for whoever opens
this repository next — possibly on another machine, certainly without
this context window.

It is not a summary of what you did. §3 and the narrative already record
that. It is **the state a cold session needs before it can act**, and it
is written for a reader who knows the project and not the session.

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

1. **Never write code past an unsigned gate.** If D1 is open, no store
   crate gets written — the engine it would be written against is not
   chosen. Surface the gate instead of guessing.
2. **Never deepen a spec section whose decision is still open** in a way
   that presumes the outcome. Options may be sharpened; the choice may
   not be pre-empted.
3. **`tools/tasks.py` is mandatory**, not a judgement call. Open or
   refresh it whenever the work has more than three steps, spans more
   than one file, spawns a subagent, records a finding, or takes on debt.
4. **`TODO.md` is an exit gate on every delivery.** A change is not
   delivered until the board reflects it — including debt and findings
   discovered mid-work, tasks that had to be split, and steps found
   during implementation.
5. **Evidence over inference.** A performance claim, a compatibility
   claim or a "the client accepts it" claim is backed by a command that
   was run and cited, or it is marked as inference.
6. **The spec moves only by user decision.** §12 is never settled by an
   agent; a signed decision executes its `On sign-off:` work order in the
   same change.
7. **No AI attribution, no forged review trailers.** Not in a commit
   message, a comment, or a doc. The two role trailers only after a real
   dual review of this exact diff.
8. **Adding an external crate, changing the wire/tool surface, or
   settling a §12 decision all require asking first.**
9. **Never dogfood an unsigned design.** Until this server's own §12
   decisions are signed and it works, the markdown board + `tasks.py`
   remain the trackers of record for this repo.

## The two trackers — do not conflate them

| | `tools/tasks.py` | `docs/implementation/TODO.md` |
|---|---|---|
| Lives at | `.agents/cache/tasks/<branch>.yaml` | `docs/implementation/TODO.md` |
| Committed | no (gitignored) | yes |
| Scope | this branch's work units, findings, debt, gates | the project's progress across sessions |
| Updated by | whoever is orchestrating | whoever delivers a change |
| Survives | branch switch | everything |
| Answers | "what am I doing now, and how will I know it worked" | "what has this project done, and what is blocking it" |

An instruction to "update the TODO" is ambiguous between them. If context
does not settle it, **ask** — updating the wrong one silently loses work.
Both are part of the exit gate; neither substitutes for the other,
because a reviewer cannot see a gitignored cache.

## Where things live

| Need | Path |
|---|---|
| The spec (model, storage, surface) | `docs/todo-mcp-design.md` |
| Progress board (committed state) | `docs/implementation/TODO.md` |
| Open decisions + recommendations | `docs/implementation/decisions.md` |
| Prioritized "what needs you" view | `tools/attention.py` → `.agents/cache/ATTENTION.md` |
| Session work tree + verification | `tools/tasks.py` → `.agents/cache/tasks/<branch>.yaml` |
| Domain knowledge | `.agents/skills/dacode-todo-mcp/SKILL.md` |
| Coding conventions | `.agents/roles/_rust.md` |
| The three trackers and their conventions | `.agents/knowledge/workflows/task-tracking.md` |
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
