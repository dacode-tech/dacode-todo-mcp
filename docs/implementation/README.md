# Implementation plan — index

Three homes, three jobs. The division of labour is deliberate:

| Home | Holds | Moves when |
|---|---|---|
| [`../todo-mcp-design.md`](../todo-mcp-design.md) | **Truth** — the spec: domain model, storage, graph semantics, tool surface, roadmap (§14), open decisions (§12) | only by explicit user decision |
| [`decisions.md`](decisions.md) | **Reasoning** — the `D`-numbered sign-off dossier mirroring spec §12: options, recommendation, confidence, `On sign-off:` work orders | an agent proposes; the user signs |
| the store + [`handover/`](handover/) | **State** — this repo's nodes in the server's own store (the tracker of record since the 2026-10-12 cutover), the live epic's handover beside it | every delivery, in the same change |

The markdown board that used to hold State is archived at
[`../archive/board-2026-10-12/`](../archive/board-2026-10-12/) — a
frozen backup agents never read. The tracking convention is
[`.agents/knowledge/workflows/task-tracking.md`](../../.agents/knowledge/workflows/task-tracking.md).

## Milestone mapping

The M0–M5 milestones are delivered and live in the store as the epics
E1–E6 (E7 Follow-ups, E8 Debt, E9+ the post-cutover arcs); their exit
criteria are spec §14. There are no numbered plan docs — the spec plus
the store carry the plan. If implementation outgrows that (a doc per
feature area, as dacode runs), introducing `NN-feature.md` plan docs
is itself a store node, not a silent reorganization.

## Reading order for a cold session

1. `/todo-next-imp` (the slash command) — or manually:
   `todo-mcp --root ~/.local/share/dacode-todo progress --json`, then
   the live epic's handover in [`handover/`](handover/), then
   [`decisions.md`](decisions.md)'s summary.
2. The spec section the proposed step touches.
3. `.agents/skills/todo-next-imp/SKILL.md` for the exit gate the work
   must pass before it counts as delivered.
