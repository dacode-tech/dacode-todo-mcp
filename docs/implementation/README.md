# Implementation plan — index

Three files, three jobs. The division of labour is deliberate and
`tools/attention.py` cross-checks all three:

| File | Holds | Moves when |
|---|---|---|
| [`../todo-mcp-design.md`](../todo-mcp-design.md) | **Truth** — the spec: domain model, storage, graph semantics, tool surface, roadmap (§14), open decisions (§12) | only by explicit user decision |
| [`decisions.md`](decisions.md) | **Reasoning** — the `D`-numbered sign-off dossier mirroring spec §12: options, recommendation, confidence, `On sign-off:` work orders | an agent proposes; the user signs |
| [`TODO.md`](TODO.md) | **State** — the committed board: milestones (M0–M5), what landed (§3), follow-ups and debt with stable `U` ids (§4/§5), the session log (§6) | every delivery, in the same change |

The session-scoped tracker (`tools/tasks.py` →
`.agents/cache/tasks/<branch>.yaml`) is the fourth artifact and the only
gitignored one; its convention is
[`.agents/knowledge/workflows/task-tracking.md`](../../.agents/knowledge/workflows/task-tracking.md).

## Milestone mapping

Authoritative milestone names live in `.agents/config/agent-rules.yaml`
(`milestones:`) and their exit criteria in spec §14; the state table is
[`TODO.md`](TODO.md) §2. There are no numbered plan docs yet — unlike the
sibling projects, this plan is small enough that the spec plus the board
carry it. If implementation outgrows that (a doc per feature area, as
dacode runs), introducing `NN-feature.md` plan docs is itself a board
item, not a silent reorganization.

## Reading order for a cold session

1. `/todo-next-imp` (the slash command) — or manually:
   `tools/attention.py`, then [`TODO.md`](TODO.md) §1 and the newest §6
   entry, then [`decisions.md`](decisions.md)'s summary.
2. The spec section the proposed step touches.
3. `.agents/skills/todo-next-imp/SKILL.md` for the exit gate the work
   must pass before it counts as delivered.
