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

_Empty — reset at the M2 exit gate (g7), 2026-10-02._

`P-LIVE-001` … `P-LIVE-008` were promoted to `.agents/pitfalls.md` (the
first six at the M1 gate, the last two at the M2 gate) and their ids are
carried in the promoted entries' **Promoted from** lines, because
test-file module docs across `crates/todo-store/tests/` and
`crates/todo-cli/tests/` cite `P-LIVE-002`, `P-LIVE-004` and
`P-LIVE-005` by id, and board §6 prose cites `P-LIVE-007`/`P-LIVE-008`.
Where they now live:

| Former id | Promoted to |
|---|---|
| `P-LIVE-001` (spawn deaths, "Connection reset by server") | pitfalls.md 5.4 |
| `P-LIVE-002` (libtest name filter, not a target selector) | pitfalls.md 4.1 |
| `P-LIVE-003` (serde collapses absent vs null on `Option`) | pitfalls.md 2.3 |
| `P-LIVE-004` (E0761: `tests/<name>.rs` + `tests/<name>/mod.rs`) | pitfalls.md 4.2 |
| `P-LIVE-005` (a filtered verify line can pass vacuously; zsh word-splitting) | pitfalls.md 4.1 |
| `P-LIVE-006` (`dacode-json` tuple deserialization; resolved upstream) | pitfalls.md 2.5 |
| `P-LIVE-007` (a class sweep is only as wide as its grep) | pitfalls.md 5.7 |
| `P-LIVE-008` (whoever writes a contract sentence cannot falsify it) | pitfalls.md 5.8 |

The next entry is `P-LIVE-009`. Do not reuse a promoted id.

---
