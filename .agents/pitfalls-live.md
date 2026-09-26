# Live Pitfalls (Auto-Updated)
# Template version: 0.1

> **Scope awareness:** Each entry must state its scope — which crate,
> subsystem, milestone or tool version it applies to. Live pitfalls are
> temporary workarounds, not permanent design decisions.
>
> **Do not carry these restrictions into:**
> - `docs/todo-mcp-design.md` or any spec section
> - The MCP tool surface or the on-disk format, whose shapes are
>   user-signed decisions, not workarounds
> - Code that a client of this server will have to understand

---

## How This Works

1. **Post-struggle**: when a build/fix cycle needs ≥2 iterations on the
   same root cause, the orchestrator appends an entry here.
2. **Post-review**: when a reviewer flags a bug class workers should
   avoid, it gets recorded here too.
3. **Worker prompts**: include both `pitfalls.md` (static) and this file
   — workers see the latest known issues.
4. **Consolidation**: at each milestone gate, merge confirmed entries
   into `pitfalls.md` and reset this file.

### Quality Gate for New Entries

- [ ] **Scoped correctly** — names the crate, subsystem, milestone or
      tool version it applies to.
- [ ] **Not a design compromise** — the workaround is a process
      constraint. If it would change the shape of the store format or
      the wire surface, flag it for human review instead.
- [ ] **Temporary** — the entry notes what future work makes it
      obsolete.
- [ ] **No AI attribution** — as everywhere.

### Entry Format

```markdown
### P-LIVE-NNN: <short title>
- **Scope**: <crate/subsystem/milestone/tool this applies to>
- **Discovered**: <task-id>, <date>
- **Symptom**: What went wrong (error message, wrong value, failed gate)
- **Root cause**: Why it happened
- **Workaround**: What to do instead
- **Obsoleted by**: Which future work removes this limitation
- **Source**: struggle | review
```

---

## Entries

### P-LIVE-001: subagent spawn dies "Connection reset by server" on a large inline prompt
- **Scope**: harness / subagent dispatch (the task tool), observed on
  the implementer binding; 2026-09-26, M1 t6
- **Discovered**: t6 dispatch, 2026-09-26
- **Symptom**: two consecutive implementer spawns failed instantly with
  "Connection reset by server" on a ~7 KB inline prompt. A control
  spawn (explore role, tiny prompt) succeeded immediately; the third
  implementer attempt — same task, spec moved to `tmp/t6-spec.md`,
  prompt a few lines pointing at it — succeeded first try.
- **Root cause**: unconfirmed. Correlates with large inline prompts on
  the heavy-model binding; the same role spawned fine with large
  prompts on 2026-09-25 (M0), so it may be transient provider trouble
  or a payload-related rejection.
- **Workaround**: write the full task spec to a scratch file and keep
  the spawn prompt short — pointer + hard constraints only. Cheaper to
  read for humans too, and it survives a spawn retry verbatim.
- **Obsoleted by**: a confirmed root cause. If transient: nothing. If
  payload-related: a size guideline in the orchestrator's dispatch
  docs.
- **Source**: struggle
