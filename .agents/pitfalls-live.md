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

_No entries yet. Do not seed this file with guesses — an invented
pitfall is worse than an empty file, because worker prompts include
everything written here. The first entry should come from a real
struggle or a real review finding._
