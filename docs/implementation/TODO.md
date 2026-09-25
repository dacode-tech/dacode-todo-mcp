# TODO — dacode-todo-mcp

The committed project board: what has landed, what is next, and what is
blocking. Rules, which `tools/attention.py` and the reviewers enforce:

- **A status change is a one-line edit; resolved items are struck, never
  deleted** — gaps in the `U` numbering are information.
- **The id lives inside the leading bold run** (`**U7 · Title**`) because
  the tooling keys on it; checking an item off must not change what the
  tools read.
- **Reasoning never goes on the board.** It goes in
  `docs/implementation/decisions.md` or the spec. The board is status.
- **A delivery that does not move this file is a review finding.** Update
  it in the same change as the work.

## 1. Current state

**Phase: M0 — spec signed; the skeleton is the next unit.** D1–D8 were
signed 2026-09-25 (D4 and D8 with user amendments: per-repo stores; a
per-instance default repo slug), their §12 items deleted and their
sections made normative. Three follow-on decisions drafted from the
sign-off message's new requirements — D9 statuses, D10
references/specs/decisions as server entities, D11 next-items — gate
M1/M2. The Cargo workspace skeleton (M0) is unblocked and presumes
nothing D9–D11 could invalidate.

| Blocking | What | Action |
|---|---|---|
| D9 | status vocabulary and the deferred state — gates M1's domain model | vote |
| D10 | references, specs and design decisions as server entities — gates M1's node schema | vote |
| D11 | next-items batch and derived weight — gates M2's selection surface | vote |

## 2. Milestones

Authoritative names: `.agents/config/agent-rules.yaml` → `milestones`;
exit criteria: spec §14. Priority is derived from the Gated-on column by
`tools/attention.py`, never hand-written.

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | spec settled: §12 items 1–3 signed (done 2026-09-25); Cargo workspace skeleton with the lint regime; all gates green | — | unblocked — next |
| M1 | domain model + storage engine: graph, statuses, persistence behind the `Store` trait, golden files | D9, D10 | not started |
| M2 | queries: repo/epic selection, dependency resolution, deterministic `next_task`; registry with path updates | D11 | not started |
| M3 | MCP server: the full §6 tool list over the wire, protocol-level tests, default-repo slug config (D8), smoke against a real client | — | not started |
| M4 | adoption: camera-dialect importer first (D7), board fixtures checked in per §10; ikvm-html and dacode follow; the three repos tracked for real | — | not started |
| M5 | packaging, docs, cwd auto-detection (deferred by D8), polish | — | not started |

## 3. Done

Oldest first. One line per delivery, with the commit hash once committed.

- 2026-09-25 — **harness bootstrap**: the agent system (five generated
  agents, two slash commands, three skills), the repo tooling
  (`attention.py`, `tasks.py`, `agents-sync.py`, `agents-index.py`,
  `check-links.py`), the seed spec with eight §12 decisions, this board
  and the decision dossier. Landed as `e6d4761`; the tracker-keying
  knowledge note as `5062862`.
- 2026-09-25 — **D1–D8 signed**: all eight decisions signed by the
  user, with amendments — D4 resolved to (c) per-repo store directories
  under a shared registry (no cross-project shared database), D8 to (a)
  plus a per-instance default repo slug. §12 items 1–8 deleted,
  §2–§9/§13 made normative. **D9–D11 drafted** from the sign-off
  message's new requirements: status vocabulary + deferred state;
  references and specs/decisions as server entities; next-items batch +
  derived weight.

## 4. Follow-ups

Unowned work. Take the next unused `U` id; never renumber, never reuse.

- [x] **U1 · Sharpen §4.1's engine comparison with a measurement.** A
      replay+snapshot micro-benchmark at 10^5 nodes for the JSONL option,
      and a redb write/read equivalent — numbers turn D1 from a lean into
      a decision. `action: measure` material for D1, though D1 itself is a
      vote. **Superseded 2026-09-25**: D1 signed without the
      measurement; these numbers become decision material only if the
      redb fallback (D1's banner) is ever proposed.
- [ ] **U2 · Survey rmcp's tool macros against §6.** Confirm the signed
      tool list is expressible with stable error codes and ordered list
      responses as rmcp schemas; record findings in §6 before M3
      implementation (D5 is signed). Read the rmcp version's source; do
      not guess from its README.
- [ ] **U3 · Collect the three sibling board dialects as fixtures.**
      Snapshot camera/ikvm-html/dacode `TODO.md` + decisions files into
      the future importer's test fixtures (§9, §10), with a short dialect
      note each — needed before M4; D7 is signed camera-first, so the
      camera fixture is the critical one.
- [ ] **U4 · Draft the golden-file and determinism test scaffolding
      shape** (spec §10) so M1 starts with the harness, not after it:
      where fixtures live, how they regenerate, how the byte-compare test
      is written once and reused.
- [ ] **U5 · Decide the store's backup/corruption-recovery story** and
      write it into §4's requirements or §8: what a torn last line of an
      event log means, and whether snapshots are the recovery unit.
      Now scoped to the signed D1 engine (JSONL event log) and the D4
      per-repo layout.

## 5. Debt

Deliberate shortcuts, acknowledged rather than fixed. Same id rules as §4.

_No debt recorded yet — nothing has been built to take a shortcut in._

## 6. Session log

Newest first. One entry per session, written for someone who was not
here. The final entry of a session is the handover
(`.agents/skills/todo-next-imp/SKILL.md` defines the shape).

### 2026-09-25 — D1–D8 signed; D9–D11 drafted

The user signed all eight decisions in one message, with amendments.
D4 resolved to (c) against the recommendation: per-repo store
directories under a shared registry — "not one common database shared
across many projects". D8 resolved to (a) plus a per-instance default
repo slug passed through the client's per-project mcp entry (explicit
configuration; cwd detection still deferred to M5). D3's repo-key
sub-choice settled on immutable slugs, confirmed by the user's own
"repo/project slug" phrasing. All eight work orders executed: §12
items 1–8 deleted, PROPOSED banners struck across §2–§9, the JSONL
engine normative in §4.1, the D6 tiebreaker normative in §5.2,
per-repo lock scope in §8, camera-first importer in §9, default-slug
wiring in §6/§13.

The same message added three requirement clusters, and a follow-up
message directed that specs and design decisions should live in the
server so they can be reliably referenced. Drafted as: **D9** (status
vocabulary open/in_work/blocked/deferred/closed — reconciled with
keeping `wontfix`; until-dates as metadata so selection stays
wall-clock-free), **D10** (layered: references list for externals;
`decision` nodes with full dossiers that can `block` work while open;
`doc` registration for stable addresses with prose bodies staying in
git — a full content store is offered as option (c), recommended
against for v1) and **D11** (`next_items` batch with blocked-by/blocks
annotations and derived, display-only weight; a hand-set priority would
amend signed D6 and is flagged as a separate choice). Also this
session: committed the bootstrap (`e6d4761`) and the tracker-keying
note (`5062862`).

**Handover.** Workspace: none yet — the M0 skeleton is unblocked and
authorized (D1–D3 signed) but not started. Landed: this sign-off
commit (docs only). Next step: the user votes D9–D11 (rendered by
`tools/attention.py`); the M0 scaffold may proceed in parallel on the
user's go — empty crates plus the lint regime presume nothing D9–D11
could invalidate. Blocked on you: D9, D10, D11; plus one D4
interpretation to veto if wrong — epics/tasks stay nodes inside their
repo's store, not per-epic files. Unverified: as before — no agent
spawn-tests yet, no serena/ast-grep smoke until a workspace exists.

### 2026-09-25 — bootstrap

Repo had a git init and nothing else. Adopted the camera harness
(the newest of the three sibling setups): `agent-rules.yaml` as the
single source of truth, five agents generated by `tools/agents-sync.py`
(orchestrator, implementer, cross-family dual reviewers, read-only
explore), `/todo-next-imp` + `/todo-close` commands, three skills, the
knowledge/pitfalls structure, and the four doc tools ported with this
repo's paths. Models bound cross-family (qwen heavy / glm mid / flash
light) per the dacode convention. Wrote the seed spec
(`docs/todo-mcp-design.md`) around the user's stated constraints —
deterministic, non-markdown, directed graph, local non-relational store,
repo+epic selection, mutable repo paths — with the engine choice and
seven other questions left open as §12/D1–D8 rather than pre-empted.

**Handover.** Workspace: none (pre-M0; no `Cargo.toml`, every code gate
skipped by construction). Landed: the whole harness + docs, uncommitted
on `main` with no prior commits. Next step: the user votes D1–D3 — they
gate M0 and everything after; U1's measurement is the cheapest way to
de-risk D1 first if wanted. Blocked on you: all eight D-items
(`tools/attention.py` renders them). Unverified: the tools have been run
only against this bootstrap's own docs; agents-sync output has not been
spawn-tested (first spawn of each role will confirm the
`reasoning_values` rows, per the table's own caveat); no serena/ast-grep
smoke until a workspace exists.
