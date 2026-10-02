# Skill Base — Mandatory Pre-Task Protocols
# Template version: 0.1
#
# This file is the **base instruction set** every domain `SKILL.md` inherits.
# Domain skills (`.agents/skills/<name>/SKILL.md`) MUST link this file at the
# top of their orchestration section and MUST follow these protocols before
# doing any non-trivial work — orchestrator and workers alike.
#
# Do not duplicate its content into individual skills; reference it instead.

---

## ⚠️ MANDATORY: Pre-Task Knowledge Search

**Applies to: orchestrator and workers, before every task — not only when
spawning workers.** Workers receive these files in their prompts; the
orchestrator must consult them itself before making plan-level decisions.

### Order of consultation

1. **Spec** — `docs/todo-mcp-design.md`. Read the section the task touches
   and its cross-references, including §12 open decisions. If the task
   contradicts the spec, stop and surface it — the spec moves only by
   explicit user decision, and §12 is never settled by an agent.

2. **The nearest precedent.** Before writing a module, find the existing
   code (or, while the workspace is young, the spec shape) that already
   does this kind of thing, and imitate it. A shape the codebase already
   accepted beats one you designed.

3. **Area documentation** — If an `AGENTS.md` covers the area, read it
   first; look it up in `.agents/knowledge/INDEX.md`, do not guess whether
   one exists.

4. **Workflow knowledge** — For any recurring procedure (build gates,
   golden-file updates, MCP smoke tests, task tracking), read the matching
   article in `.agents/knowledge/workflows/`. Index in
   `.agents/knowledge/README.md`.

5. **Pattern knowledge** — For a recurring code shape (a new store
   backend, a new MCP tool, a graph query), search
   `.agents/knowledge/patterns/`.

6. **Pitfalls** — Read the relevant sections of `.agents/pitfalls.md` and
   **ALL** entries in `.agents/pitfalls-live.md` matching the area (grep by
   crate, subsystem, tool, or symptom).

### Failure mode this prevents

Re-inventing a shape the project already has — a second selection rule
beside the one §5 defines, a hand-rolled serializer beside the canonical
one, an ad-hoc task list beside the trackers. A change that rediscovers
the project's own answer costs three times as long and then gets bounced
at review.

**Always read the spec section and the knowledge base first.**

---

## ⚠️ MANDATORY: Canonical Commands — Do Not Reinvent

| Need | Canonical command | Forbidden alternative |
|------|-------------------|------------------------|
| Format | `cargo fmt --check` | hand-aligning code |
| Build | `cargo check --workspace --all-targets 2>&1 \| tail -c 5000` | a hand-rolled `rustc` invocation |
| Test (iterating) | `cargo test -p <crate> [-- <name>]` | bare `cargo test --workspace` in the inner loop |
| Test (final gate) | `cargo test --workspace` | — |
| Lint | `cargo clippy --workspace --all-targets` | suppressing a lint without a one-line reason |
| Docs | `cargo doc --workspace --no-deps`, twice — plain and with `--document-private-items` | skipping the private-items run |
| Release build | `cargo build --release 2>&1 \| tail -c 5000` | — |
| Server smoke | `cargo run -p todo-mcp -- --stdio` + an initialize handshake | asserting "it should work" |
| Code search | the four-layer policy in `.agents/roles/_search.md` | an unscoped repo-wide grep |
| Task tracking | `tools/tasks.py` (convention: `knowledge/workflows/task-tracking.md`) | ad-hoc markdown checklists |
| What needs the user | `tools/attention.py` | re-reading three docs by hand |
| "What do I do next?" | `/todo-next-imp` | guessing from the board alone |
| "Is this delivered?" | `/todo-close` | declaring done and hoping |
| Agent adapters | `tools/agents-sync.py` | hand-editing `.opencode/` |
| Doc index | `tools/agents-index.py` | hand-editing `.agents/knowledge/INDEX.md` |
| Doc link check | `tools/check-links.py` | eyeballing links |

Pipe long build output: `2>&1 | tail -c 5000`.

Everything builds on the host; there is no toolchain excuse. A gate you
did not run is **`skipped`**, never `pass`.

If a task seems to require a non-canonical tool, **first** grep
`.agents/knowledge/workflows/` and `.agents/pitfalls-live.md` for the
canonical recipe. If it truly is missing, propose adding it as a knowledge
article — do not silently invent an ad-hoc alternative.

---

## ⚠️ MANDATORY: Review Before Every Code Commit

When `review.dual_review: true` in `.agents/config/agent-rules.yaml`, every
code commit requires sign-off from BOTH reviewers (`review_primary` and
`review_secondary` — see that file for the current bindings; the two run on
different model families on purpose).

**A code commit carries both `Reviewed-by:` role trailers**, exactly:

```
Reviewed-by: dacode-todo-mcp-reviewer-primary <primary@dacode.local>
Reviewed-by: dacode-todo-mcp-reviewer-secondary <secondary@dacode.local>
```

and only after both reviewers actually returned `status: ok` on this exact
diff. If the diff changed after a review, the review is stale: re-review,
or drop the trailer. A trailer without a real review behind it is a forged
attestation. No model name or tool ever appears in a trailer.

Verdict-merge rules live in `.agents/roles/orchestrator.md`.

Exempt from **dual review** — not from committing, and not from
verification (`tools/check-links.py`, `tools/agents-sync.py --check`,
`tools/tasks.py check`, `tools/attention.py --check` as applicable):

- Documentation-only changes (`*.md`, `docs/`, `.agents/`, `AGENTS.md`)
- Task tracker updates
- Knowledge-base updates (`.agents/knowledge/`, `.agents/pitfalls-live.md`)
- Generated files (`.opencode/*`, `.mcp.json`, `.agents/knowledge/INDEX.md`)

Review-exempt commits carry no `Reviewed-by:` trailer.

---

## ⚠️ MANDATORY: What never appears in anything we write

No commit message, code comment, markdown file or TODO entry may mention
an LLM, an AI assistant, a model name, or that anything was generated or
assisted. No `Co-authored-by:` for a tool, no "generated with", no
"assisted by". The two role `Reviewed-by:` trailers above are the only
exception this repo makes, and they name harness roles, never models.

---

## Post-Task: Knowledge Evolution

After each successful commit, assess whether the task surfaced:

1. **A reusable code shape** (how a store backend is added, how an MCP
   tool is wired, how a graph query is tested) → an article in
   `.agents/knowledge/patterns/`.
2. **A new procedure** → an article in `.agents/knowledge/workflows/`.
3. **A trap you fell into** → an entry in `.agents/pitfalls-live.md`, in
   the format that file's header defines. A clippy lint that fired only
   on macOS, a golden file that broke from a BTreeMap ordering assumption,
   an MCP client quirk — all of these.
4. **A fact established the hard way** — a measured benchmark, a crate-API
   constraint discovered by reading source, a format edge case → into the
   **spec's numbered sections**, not just into the commit. It is the
   expensive part and it must survive the session.
5. **A plan-level discovery** → propose an update to
   `docs/todo-mcp-design.md` to the user. The spec does not move silently.
6. **A canonical-tool rediscovery** (you almost hand-rolled something a
   `tools/` script or a cargo subcommand already does) →
   `pitfalls-live.md`, and if the recipe is short, into the relevant
   `workflows/*.md` table.
7. **A recurring convention that reviewers keep enforcing by hand** →
   propose promoting it into `.agents/roles/_rust.md` or `pitfalls.md`.

Commit knowledge and doc updates with the prefix `knowledge:` or `docs:`;
they are exempt from dual review and carry no trailer.

Skip this step only when the task was purely mechanical with no new
learnings.

---

## Reference Files (loaded into worker prompts)

| File | Contents |
|------|----------|
| `.agents/skill-base.md` | THIS FILE — base instructions every skill inherits |
| `.agents/roles/orchestrator.md` | Spawning, escalation, verdict merge |
| `.agents/roles/implementer.md` | Worker standards, output format |
| `.agents/roles/reviewer.md` | Review checklist, verdict format, severity guide |
| `.agents/roles/_rust.md` | Rust conventions, architecture rules, determinism discipline |
| `.agents/knowledge/` | Workflows and patterns; index in `README.md` |
| `.agents/pitfalls.md` | Fixed lessons — include relevant sections in worker prompts |
| `.agents/pitfalls-live.md` | **Adaptive** — include ALL matching entries in worker prompts |
| `.agents/config/agent-rules.yaml` | Roles, models, slash commands, review config, escalation thresholds, gates |
| `.agents/commands/*.md` | Slash-command prompt bodies (rendered into `.opencode/commands/`) |
| `.agents/knowledge/workflows/task-tracking.md` | The three trackers and the conventions for each |
| `docs/todo-mcp-design.md` | THE spec |
| `docs/implementation/TODO.md` | The committed board |
| `docs/implementation/decisions.md` | Sign-off record for §12 |

---

## How a Domain Skill Inherits This

A domain `SKILL.md` MUST contain, near the top of its orchestration
section:

```markdown
## Inherited Base Instructions

This skill inherits the mandatory protocols defined in
[`.agents/skill-base.md`](../../skill-base.md):

- **Pre-Task Knowledge Search** — spec section, nearest precedent, knowledge/, pitfalls-live.md before starting
- **Canonical Commands** — cargo gates / `tools/*.py`; never hand-rolled equivalents
- **Mandatory Review** — dual sign-off before every code commit; role trailers only after a real review
- **No AI attribution** — in anything, ever
- **Post-Task Knowledge Evolution** — capture patterns, workflows, pitfalls, and hard-won facts

**Read `.agents/skill-base.md` in full before resuming work in this skill.**
```

The skill itself adds only domain-specific content on top of this base.
