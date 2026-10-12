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

_Empty — reset at the M2 exit gate (g7, 2026-10-02), the M3 exit gate
(g9, 2026-10-04) and the M4 exit gate (g11, 2026-10-07)._

`P-LIVE-001` … `P-LIVE-009` were promoted to `.agents/pitfalls.md` (the
first six at the M1 gate, the next two at the M2 gate, the last at the
M4 gate) and their ids are carried in the promoted entries'
**Promoted from** lines, because test-file module docs across
`crates/todo-store/tests/` and `crates/todo-cli/tests/` cite
`P-LIVE-002`, `P-LIVE-004` and `P-LIVE-005` by id, and board §6 prose
cites `P-LIVE-007`/`P-LIVE-008`. Where they now live:

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
| `P-LIVE-009` (non-ASCII in inline `git commit -m` gets mangled) | pitfalls.md 4.7 |

The next entry is `P-LIVE-015`. Do not reuse a promoted id.

---

### P-LIVE-010 — macOS `accept()` inherits the listener's `O_NONBLOCK`

`t71` (2026-10-10): a `TcpListener` set non-blocking (so a stop flag
can be observed between accepts) hands its `O_NONBLOCK` down to every
accepted stream **on macOS** — Linux does not. The symptom is
delayed and misleading: plain HTTP requests keep working (their bytes
are usually already in the socket buffer when the connection thread
reads), and the first genuinely blocking read — the WS connection
loop's frame read — fails instantly with `EAGAIN` ("Resource
temporarily unavailable"), closing the connection right after a
successful handshake. The fix is one line in the accept loop:
`stream.set_nonblocking(false)` on every accepted stream (a no-op
where the platform already hands over a blocking socket). Any future
listener in this tree carries it; a read that "should block" erroring
with EAGAIN on a silent socket is this entry's signature.

---

### P-LIVE-011: a census comment is a tree-wide claim one commit elsewhere falsifies
- **Scope**: test-file module docs across `crates/*/tests/` (the
  `std::time::Instant` censuses are the confirmed instances)
- **Discovered**: U43/U49, 2026-10-12
- **Symptom**: two censuses claiming "`Instant` appears in this file
  alone" were both false — repos.rs's since t62's lock.rs pin,
  progress.rs's since t85's serve.rs deadline loops — each falsified by
  a commit in another file that never touched the census
- **Root cause**: a scoping claim in a comment has no mechanical
  enforcement; the grep that would catch it runs only when someone
  thinks to run it
- **Workaround**: grep before trusting a census
  (`rg -n "<item>" crates/`), and when adding a new use of a censused
  item, grep for the census and move it in the same commit
- **Obsoleted by**: promotion into `_rust.md`'s comment rules if the
  class keeps recurring
- **Source**: struggle

---

### P-LIVE-012: `Map::remove` is a swap-remove under `preserve_order`
- **Scope**: any code mutating `serde_json::Map` whose key order reaches
  the wire or a golden (schema.rs's advertisement pipeline; D14's
  byte-stable order everywhere)
- **Discovered**: U51, 2026-10-12
- **Symptom**: the error-union wrap's first draft removed `$schema`,
  `title`, `description` and `$defs` with four `Map::remove` calls —
  the success arm's remaining keys serialized REVERSED
  (`[type, properties, required]` became `[required, properties, type]`),
  and only the order-pinning unit test caught it (Value equality is
  order-insensitive, so every `assert_eq!` on parsed JSON passes
  through a shuffle)
- **Root cause**: serde_json's `Map::remove` under `preserve_order`
  delegates to IndexMap's `swap_remove` — the LAST entry backfills the
  removed slot, so any mid-map removal shuffles the survivors. The
  pre-existing strip/flatten removes never surfaced it only because
  their targets sit last-or-second-to-last in schemars' shapes
- **Workaround**: rebuild in one ordered pass (iterate
  `std::mem::take(map)`, route each key to its destination) instead of
  removing in place; pin order with string-contains on the serialized
  bytes, never with Value equality
- **Obsoleted by**: nothing — this is serde_json's semantics, not a
  version quirk; promotion into `_rust.md` if map surgery recurs
- **Source**: struggle

---

### P-LIVE-014: an install without a rebuild serves yesterday's pages, silently
- **Scope**: the stable binary at `~/.local/bin/todo-mcp` (the README's
  install ritual; every opencode session's server and the live view's
  listener)
- **Discovered**: U54's delivery window, 2026-10-12
- **Symptom**: hours after the per-epic-pages arc landed, the browser
  still showed the old sidebar (`#lane-<n>` anchors) and the epic
  routes 404'd — even though an install at 19:33 had followed the
  cp-to-temp-then-mv mechanics perfectly
- **Root cause**: the install copied a stale `target/release/todo-mcp`
  (built before the change landed); the ritual's first line —
  `cargo build --release` — was skipped, and nothing in the mechanics
  catches it: the cp+mv succeeds, codesign passes, `--version` runs,
  and the old code serves
- **Workaround**: the ritual in full, build first. When the browser
  disagrees with the tests, compare `target/release/todo-mcp`'s mtime
  against `git log -1` before blaming the code — and remember the
  live view lags one more step: the listener serves its owning
  session's inode, so the new pages arrive only after the old
  sessions exit and a new-binary watchdog re-ensures the listener
- **Obsoleted by**: nothing — process discipline; the README's
  install section names the live-view lag
- **Source**: struggle

---

### P-LIVE-013: the stable-binary install ritual is load-bearing — cp in place SIGKILLs the path
- **Scope**: the stable binary at `~/.local/bin/todo-mcp` (README's
  install section and the config entry's comment both prescribe the
  same ritual)
- **Discovered**: the U51 install, 2026-10-12
- **Symptom**: minutes after `cp target/release/todo-mcp
  ~/.local/bin/todo-mcp` over a path a session's server was executing,
  opencode showed "MCP error: -32000: Connection closed" (the running
  servers died as the cp rewrote their executing inode), and every NEW
  spawn died at exec: exit 137 (SIGKILL), zero stdout, zero stderr —
  while `codesign` said "valid on disk" and the content was
  byte-identical to the build. The kill persisted across invocations
- **Root cause**: the ritual was documented in two places and skipped
  anyway — a plain cp modifies the executing file's inode in place;
  macOS's kernel keeps a per-vnode code-signature cache that the
  modification invalidates, and exec of that vnode is SIGKILL'd until
  the file is replaced by a FRESH inode
- **Workaround**: the README's ritual, exactly: cp to a temp name,
  then mv over the target (the rename swaps the directory entry to a
  new inode; running servers keep their old one, undisturbed). The
  exit-137-with-no-output signature means the ritual was skipped —
  redo the install; do not debug the binary
- **Obsoleted by**: nothing — macOS semantics; the README documents it
  permanently
- **Source**: struggle

---
