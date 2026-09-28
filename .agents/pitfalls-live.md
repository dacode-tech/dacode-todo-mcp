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

### P-LIVE-002: cargo's post-`--` test argument is a libtest NAME filter, not a target selector
- **Scope**: workspace test layout — every verify line of the form
  `cargo test -p <crate> -- <filter>`, in particular t10–t13's
  `todo-store`/`todo-cli` integration test files
- **Discovered**: t9, 2026-09-27
- **Symptom**: `cargo test -p todo-store -- golden` selects only tests
  whose *names* contain `golden`. Tests sitting flat in
  `tests/golden.rs` are named after their functions, so the filter can
  match zero tests and a verify line passes vacuously.
- **Root cause**: libtest treats the positional argument after `--` as
  a substring filter over full test paths; an integration file's name
  is not part of its tests' names unless the tests are nested in a
  module.
- **Workaround**: nest every test in the file in a `mod` whose name
  matches the verify filter (t9: `mod golden` → names become
  `golden::…`), or select the target with `--test <name>`. Treat
  `0 filtered out` plus the expected passed-count in the *right*
  target as the only proof a filter-selected run actually ran.
- **Obsoleted by**: nothing planned — this is libtest behavior. It is a
  standing layout discipline for new store/cli test files; consolidate
  into `pitfalls.md` at the M1 gate.
- **Source**: review

### P-LIVE-003: serde collapses absent vs null on plain `Option` patch fields
- **Scope**: patch/partial-update schemas — `todo-store`'s event patch
  DTOs (t10) and M3's wire patch schemas (`task_update` & co.)
- **Discovered**: t10, 2026-09-27
- **Symptom**: with a plain `Option<T>` patch field, "clear this field"
  and "leave this field alone" are the same bytes: `{"note":null}` and
  an absent `note` both deserialize to `None`, so a clearing patch
  replays as a no-op.
- **Root cause**: serde's derived `Deserialize` maps both an absent
  field (under `#[serde(default)]`) and an explicit JSON `null` to
  `None`. RFC 7386-style three-state patch semantics (absent = keep,
  null = clear, value = replace) need `Option<Option<T>>`, whose derived
  `Serialize` in turn needs help to keep the states apart on the way out.
- **Workaround**: `Option<Option<T>>` with
  `#[serde(default, deserialize_with = "double_option",
  skip_serializing_if = "Option::is_none")]` — the pattern in
  `crates/todo-store/src/event.rs`; pinned by a test that cannot even
  compile under the naive `Option`
  (`a_patch_keeps_untouched_cleared_and_replaced_apart`).
- **Obsoleted by**: nothing planned — M3's wire patch schemas should
  reuse the pattern (or the store DTOs directly); consolidate into
  `pitfalls.md` at the M1 gate.
- **Source**: struggle

### P-LIVE-004: a `tests/<name>.rs` crate root cannot pair with `tests/<name>/mod.rs` (E0761)
- **Scope**: workspace integration-test layout — `todo-store`'s
  `tests/log.rs` + `tests/log/` (t10) and t11–t13's test files under
  the P-LIVE-002 mod-wrapper discipline
- **Discovered**: t10, 2026-09-27
- **Symptom**: `mod log;` inside `tests/log.rs` fails with E0761: file
  for module `log` found at both `tests/log.rs` and `tests/log/mod.rs`.
- **Root cause**: rustc module resolution — for a `mod log;` declared in
  the crate root `tests/log.rs`, both `tests/log/mod.rs` and the root
  file itself are candidate paths, which is ambiguous.
- **Workaround**: declare the wrapper inline in the crate root —
  `mod log { mod fixtures; mod replay; mod corrupt; }` — with the
  submodules as `tests/log/<file>.rs` and no `mod.rs` in the directory.
  Test names still come out `log::…`, so the P-LIVE-002 verify-line
  filter keeps working.
- **Obsoleted by**: nothing planned — rustc module resolution; standing
  layout rule for new test targets. Consolidate into `pitfalls.md` at
  the M1 gate.
- **Source**: struggle
### P-LIVE-005: a filtered `cargo test` verify line can pass vacuously — filters OR, a no-match exits 0, and zsh does not word-split
- **Scope**: every verify line of the form `cargo test -p <crate> -- <a>
  <b> …`, and any script/loop that builds one from a variable; measured on
  rustc 1.98.1 / cargo on darwin, zsh (this harness's shell)
- **Discovered**: t11 spec, 2026-09-28
- **Symptom**: two independent ways a green run proves nothing. (1) A
  filter that matches no test prints `running 0 tests` / `test result:
  ok. 0 passed` and **exits 0** —
  `cargo test -p todo-store -- zzznomatch; echo $?` → `0`. (2) A loop
  variable holding several tokens is passed as ONE filter under zsh:
  `for f in "log golden"; do cargo test -p todo-store -- $f; done` selects
  0 tests, because zsh does not word-split an unquoted parameter and
  libtest then substring-matches the literal `"log golden"`. The first
  measurement of this looked like "multiple filters are ANDed"; it was the
  shell, not libtest.
- **Root cause**: libtest reports success for an empty selection (a filter
  is a convenience, not an assertion), and zsh's default is
  `SH_WORD_SPLIT` off — unlike bash, where `$f` would have split.
- **Workaround**: write verify commands literally in tracker/spec text, or
  use `${=f}` in zsh when splitting is intended. Treat **the passed count
  plus `0 filtered out` in the expected target** as the only proof a
  filtered run ran (P-LIVE-002), never the exit code. Established for
  real: multiple positional filters are **ORed** —
  `cargo test -p todo-store -- zzznomatch golden` runs 8 in `golden.rs`
  and 2 in `log.rs`, so the second filter alone decided the union.
- **Obsoleted by**: nothing planned — libtest and zsh behavior. Standing
  discipline for verify lines; consolidate into `pitfalls.md` at the M1
  gate with P-LIVE-002.
- **Source**: struggle
### P-LIVE-006: `dacode-json` serializes serde tuples but cannot deserialize them
- **Scope**: `todo-store`'s persisted documents (the snapshot's `relates`
  array, t12) and any future schema that uses a serde tuple or a
  tuple-shaped newtype; the engine is the sibling path dependency at
  `/Users/mouse/RPM/dacode/rust/json-tiers` (U8), outside this repo
- **Discovered**: t12, 2026-09-28
- **Symptom**: `to_bytes` over `Vec<(A, B)>` emits the expected
  `[[a,b],…]`, but `from_bytes` over those same bytes fails — it consumes
  the first element and reports the rest as trailing characters
  (`expected , or } at byte 21` nested, `trailing characters at byte 8`
  at top level). Reproduced by both the implementer and the primary
  reviewer, in this tree and in a scratch crate over plain
  `Vec<(u64, u64)>`.
- **Root cause**: unconfirmed inside the engine; consistent with the
  deserializer not driving a tuple visitor past its first element. Not a
  serde derive issue — the same types serialize correctly.
- **Workaround**: keep the canonical wire shape (Decision 4's two-element
  arrays, pinned by `snapshot.v1.golden`) and narrow on the read side —
  deserialize as `Vec<Vec<TaskId>>` and reject any inner array whose
  length is not 2 as `corrupt_store`
  (`crates/todo-store/src/snapshot.rs`, `relates_from_arrays`; the reject
  path is pinned by
  `snapshot::corrupt::a_relates_entry_that_is_not_a_pair_is_rejected`).
  Prefer a named-field struct over a tuple in any *new* schema: it avoids
  the bug entirely and reads better in an editor (§4's auditability).
- **Obsoleted by**: a fix upstream in `dacode-json`, at which point the
  narrowing reduces to the derived impl with **zero byte movement** (the
  wire shape never changed). Board follow-up U11 tracks proposing that
  diff; U8's resolution allows modifications proposed as a diff, with
  that repo's commits staying with its owner.
- **Source**: struggle
