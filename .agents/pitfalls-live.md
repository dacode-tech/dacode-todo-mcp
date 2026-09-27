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

### P-LIVE-001: subagent spawn dies "Connection reset by server" on the implementer binding
- **Scope**: harness / subagent dispatch (the task tool), observed on
  the implementer binding; 2026-09-26 (M1 t6), recurred 2026-09-27
  (M1 t7)
- **Discovered**: t6 dispatch, 2026-09-26
- **Symptom**: implementer spawns fail instantly with "Connection reset
  by server". t6: two failures on a ~7 KB inline prompt, control spawn
  (explore, tiny prompt) fine, third attempt with spec-file + short
  prompt succeeded first try. t7: **the short-prompt protocol itself
  failed twice (~1.3 KB and ~0.4 KB prompts), the same control spawn
  succeeded in between, and the third identical attempt succeeded** —
  so prompt size is not the (only) trigger.
- **Root cause**: unconfirmed. Instant failure isolated to the
  implementer binding while other roles spawn fine in the same window;
  consistent with transient provider/relay trouble on the heavy-model
  binding rather than payload content.
- **Workaround**: unchanged — spec in a scratch file, short spawn
  prompt — plus the retry loop the t7 run confirmed: on failure, fire
  a control spawn (explore, tiny prompt) to isolate binding vs
  infrastructure, then retry the identical prompt; both incidents
  succeeded on attempt 3.
- **Obsoleted by**: a confirmed root cause. If transient: nothing. If
  binding-related: a provider/model change in `agent-rules.yaml`.
- **Source**: struggle
