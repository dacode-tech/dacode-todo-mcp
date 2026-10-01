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

_Empty — reset at the M1 exit gate (g5), 2026-09-30._

`P-LIVE-001` … `P-LIVE-006` were promoted to `.agents/pitfalls.md` at
that gate and their ids are carried in the promoted entries' **Promoted
from** lines, because test-file module docs across `crates/todo-store/tests/`
and `crates/todo-cli/tests/` cite `P-LIVE-002`, `P-LIVE-004` and
`P-LIVE-005` by id. Where they now live:

| Former id | Promoted to |
|---|---|
| `P-LIVE-001` (spawn deaths, "Connection reset by server") | pitfalls.md 5.4 |
| `P-LIVE-002` (libtest name filter, not a target selector) | pitfalls.md 4.1 |
| `P-LIVE-003` (serde collapses absent vs null on `Option`) | pitfalls.md 2.3 |
| `P-LIVE-004` (E0761: `tests/<name>.rs` + `tests/<name>/mod.rs`) | pitfalls.md 4.2 |
| `P-LIVE-005` (a filtered verify line can pass vacuously; zsh word-splitting) | pitfalls.md 4.1 |
| `P-LIVE-006` (`dacode-json` tuple deserialization; resolved upstream) | pitfalls.md 2.5 |

The next entry is `P-LIVE-009`. Do not reuse a promoted id.

---

### P-LIVE-007: a class sweep is only as wide as the grep that runs it
- **Scope**: doc-claim sweeps in `crates/todo-store/src` (and any repo-wide
  sweep for a defect class); the `verify:` lines written for such a task.
- **Discovered**: t30, 2026-10-02.
- **Symptom**: the task spec's class table listed twelve "nothing written /
  nothing on disk / untouched" claims, read each against the temp- and
  lock-debris facts and cleared eleven. Review found two more of the same
  defect in the same crate — `error.rs`'s `Locked` doc and `layout.rs`'s
  `Unnamed` rationale both said "nothing was **opened**, read or written",
  which `open_and_try_lock` falsifies. The same blindness hit the task's own
  `verify:` line, which asked for `rg 'never reported through it'` to return
  nothing after a fix whose content was changing the four words *before* that
  substring, so the check could never pass.
- **Root cause**: the sweep was run against one phrasing of the defect instead
  of against the defect — here, *an exhaustive claim narrower or looser than
  its mechanism*, which has many surface forms. A verify line written from the
  same phrasing inherits the blindness.
- **Workaround**: name the defect class first, enumerate its plausible surface
  forms ("nothing was opened/written/created/changed", "no trace", "leaves
  nothing", "untouched"), and grep each; then **run every verify-line grep
  against the intended fixed state before writing it** — a check that cannot
  pass is not a check.
- **Obsoleted by**: nothing planned; it is a discipline. Consolidate into
  `.agents/pitfalls.md` at g7 beside 5.1 (falsify the argument) and 3.2 (a
  caller-facing string is contract text).
- **Source**: review

### P-LIVE-008: whoever writes a contract sentence cannot falsify it
- **Scope**: doc-only and spec-text commits authored by the orchestrator;
  reviewer prompts for prose changes. Third occurrence after t11's round 3 and
  f47's round 1.
- **Discovered**: t30, 2026-10-02.
- **Symptom**: three of four review rounds found a false clause in text the
  orchestrator had just written — "a failure **at or** after the temp file's
  creation" (false when the creation is what refused; the reviewer built the
  two states rather than reading the summary), debris reuse scoped per process
  where `temp_path_for` keys the name per store directory, and "the whole
  effect **of the attempt**" against `lock_registry`'s `create_dir_all`, which
  runs strictly before the acquire.
- **Root cause**: the author inherits the claim they were correcting and reads
  their own sentence for intent rather than against the state space — the
  prose-side twin of pitfalls 5.1.
- **Workaround**: for a prose change, the reviewer prompt must ask for
  falsification against the code path (build the state, name the file:line),
  and the author must state, per clause, what it was checked against. Applying
  a reviewer's prescribed wording verbatim is safe; paraphrasing it is what
  costs the extra round.
- **Obsoleted by**: consolidation into `.agents/pitfalls.md` at g7.
- **Source**: struggle

