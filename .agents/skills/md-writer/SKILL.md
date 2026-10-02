---
name: md-writer
description: "AGENTS.md documentation management. Creates and maintains area documentation for dacode-todo-mcp — per-crate and per-area notes under crates/ and docs/, indexed by tools/agents-index.py."
---

# md-writer — AGENTS.md Documentation Skill

## Purpose

Manage the area documentation system. An area of the project — a crate,
the store format, the MCP surface — can have its own `AGENTS.md`
providing architecture, established facts, and the current state of that
part of the work.

## Where these files may live

**`AGENTS.md` files live at the repo root, under `crates/<crate>/`, and
under `docs/<area>/`.**

They must **never** be placed inside `target/`, `tmp/`,
`.agents/cache/`, or `.opencode/node_modules/` — those are generated or
scratch space, and `tools/agents-index.py` prunes them from its walk.

---

## Discovery Criteria

An area **should** have its own `AGENTS.md` when:

- It has landed 5+ commits, or a module over ~500 lines
- It carries facts that were expensive to establish — a measured
  benchmark, a format edge case, a client quirk
- It has a non-obvious relationship to the spec section it implements,
  including deliberate divergences
- It has behaviour that differs from the general conventions in
  `.agents/roles/_rust.md`
- It is a distinct subsystem someone could work on without touching the
  rest (domain graph, store engine, MCP surface, importer)

An area should **NOT** have one when:

- Nothing has been written for it yet — a plan belongs in
  `docs/todo-mcp-design.md`, not in an `AGENTS.md`
- It would duplicate the spec or the `dacode-todo-mcp` skill without
  adding implementation detail
- It is a single commit with no follow-on work

The spec describes the *plan*; an area `AGENTS.md` documents *what the
code actually does today*, which may legitimately be richer, and which
will sometimes contradict the spec. Where they disagree, flag it in the
document; do not paper over it.

---

## Workflow

### 1. Analyze → 2. Plan → 3. Ask → 4. Execute

**Always ask for permission** before creating or modifying an
`AGENTS.md`.

### Analysis Phase

1. **What landed** — `git log --oneline <base>..HEAD -- <paths>`
2. **The spec sections** the area implements, and where it deliberately
   diverges
3. **The established facts** — measurements, constraints, quirks — with
   `path:line` or the command that produced them
4. **Verification state** — which gates run, which are `skipped` and why,
   what has never been smoke-tested against a real client

### Plan Presentation

Present before writing:

```
## AGENTS.md Plan for <crates/todo-store | docs/<area>>

Commits landed:   <count>
Files touched:    <list>
Spec sections:    <§N.M list>
Unverified:       <what has never actually run>
Action:           CREATE / UPDATE / SKIP
Reason:           <why>

Shall I proceed?
```

---

## Area AGENTS.md Template

```markdown
# <area>: Agent Documentation

## Overview

<1–2 sentences: what this part of the project covers, which spec sections
it implements, which milestone it belongs to>

## Architecture

<How the files in this area interact. Entry point, data flow, who owns
shared state. Where it deliberately diverges from the spec, and why.>

## What has landed

| Commit | Subject | State |
|---|---|---|
| `a1b2c3d` | `todo-store: add JSONL event log` | reviewed, gated |

## Facts established

The expensive part. Every row cites its evidence.

| Fact | Value | Evidence |
|---|---|---|
| Cold-load 10k tasks | `<N> ms` | `cargo bench` output, commit `a1b2c3d` |
| rmcp tool schema quirk | <one line> | read in rmcp <version> source at `path:line` |

Mark anything that appears in exactly one place as unconfirmed. That
distinction is the whole value of the table.

## Files

| File | Description |
|---|---|
| `src/event_log.rs` | <what it does> |

## Verification state

- fmt / check / test / clippy / doc: <pass / fail / skipped>
- golden files: <which fixtures pin this area>
- **smoke against a real client**: <reference, or "never run">

Be explicit about what has never run.

## Open questions

<What the next person working here needs to resolve, with the board `U`
id if one exists.>
```

---

## Cross-File Consistency Rules

When creating or updating an area `AGENTS.md`:

1. Check the `dacode-todo-mcp` skill
   (`.agents/skills/dacode-todo-mcp/SKILL.md`) and keep its crate map
   consistent with what you found.
2. If the document records a fact that is not yet in the spec, **put it
   in the spec too** (its numbered section). The spec is what survives;
   an area doc is a working note.
3. If it records new debt or a follow-up, add it to
   `docs/implementation/TODO.md` with the next unused `U` id.
4. Re-run `tools/agents-index.py` so `.agents/knowledge/INDEX.md` stays
   current, and `tools/check-links.py` after adding or renaming a file.

---

## Quality Checklist

- [ ] Is the file under the repo root, `crates/` or `docs/` — never in a
      generated or scratch directory?
- [ ] Does it name the spec sections it implements, and the divergences?
- [ ] Does every established fact cite `path:line` or the command that
      produced it?
- [ ] Are single-source, unconfirmed facts marked as such?
- [ ] Is the verification state honest about what never ran?
- [ ] Are new facts also reflected in the spec?
- [ ] Are new follow-ups on the board with a `U` id?
- [ ] Was `tools/agents-index.py` re-run?
- [ ] Was `tools/check-links.py` re-run?
- [ ] No mention of an LLM, an AI assistant, a model, or generation?

---

## Currently Documented Areas

`.agents/knowledge/INDEX.md` (generated) is the authoritative list, or:

```bash
find crates docs -name AGENTS.md -type f | sort
```

The project starts with none: nothing has been built yet, so there is
nothing for an area document to describe. The first one should appear
alongside the first landed crate logic.
