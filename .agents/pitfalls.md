# Pitfalls Knowledge Base
# Template version: 0.1

## Purpose

Lessons learned, organised by the area they affect. Workers read the
section relevant to their task to avoid repeating a past mistake.

Entries are promoted here from `pitfalls-live.md` at milestone gates,
once confirmed as generalisable.

The standing *rules* — Rust conventions, crate boundaries, determinism
discipline, commit format, no AI attribution — live in
`.agents/roles/_rust.md`, `.agents/roles/_common.md` and
`.agents/config/agent-rules.yaml`, not here. This file is for
**discovered** mistakes.

Entries below were promoted at the **M1 exit gate (g5, 2026-09-30)** and
the **M2 exit gate (g7, 2026-10-02)**. Each names what it was promoted
from, because `P-LIVE-NNN` ids are cited in test-file module docs across
`crates/todo-store/tests/` and `crates/todo-cli/tests/` and those
citations must keep resolving.

---

## Category 1: The domain model and graph

### 1.1 A counter cannot be reconstructed from the live nodes
- **Scope**: `todo-domain`'s `Counters`, `todo-store`'s snapshot restore
  and log replay; anything in M2/M3 that derives state from nodes rather
  than from the log
- **What happened**: §3's allocation never re-issues a number, and an epic
  cascade burns the deleted tasks' numbers. A restore that re-derived
  counters from the highest *live* id would therefore re-issue a burned
  number on the next allocation. `Counters` had read accessors only, so
  there was no way to restore one.
- **Do instead**: the write entry point is `Counters::from_highest` +
  `Graph::raise_counters_to` — a per-kind max, so it can never lower a
  counter. A snapshot restore calls it *after* inserting its nodes (the
  inserts raise-to-max themselves; only numbers burned above the highest
  live id are left), and replay separately requires an add-event's id to
  be the next sequential number of its kind, so a burned-by-cascade number
  and a skipped number are each `corrupt_store` naming the id.
- **Promoted from**: f5, f8 (t7/t12)

### 1.2 A derived `Ord` is declaration order — "higher" is semantic, not `>`
- **Scope**: every enum that carries a human ordering (`Priority`,
  severities, ranks) in `todo-domain`
- **What happened**: `Priority` is declared `High, Medium, Low`, so its
  derived `Ord` runs `High < Medium < Low` — a bare `>` comparison
  "preferring the higher priority" would prefer **low**. No production
  code compares priorities today (D11: display-only), which is the only
  reason this is a trap note and not a defect.
- **Do instead**: the first code that ever compares priorities either
  uses an explicit rank mapping (a `fn rank(&self) -> u8` written in the
  semantic order) or reverses the declaration — and the test that
  introduces it pins the direction with a hand-computed case. Until
  then, "prefer higher priority" in prose is an instruction to an agent,
  never a comparison.
- **Promoted from**: f100 (t40, measured while calibrating the M5
  mutation)

## Category 2: Storage and serialization

### 2.1 A multi-file write sequence's *order* is the safety argument, and a test that builds the aftermath cannot see it
- **Scope**: `todo-store`'s `compact()` (five steps) and any future
  sequence that writes more than one file; the task specs that describe
  them
- **What happened**: the first `compact()` order staged both temps and
  renamed log-then-snapshot. Two crash-reachable corners lost events
  **silently**: a first-ever compaction crashing between the renames left
  `header + seed` with no snapshot at all, which the no-snapshot branch
  opens clean as a bare repo graph; a repeat compaction could leave a
  prior re-based `log_offset` exactly *equal* to the new short length, so
  every offset check passed and the reopen yielded the previous
  compaction's state. The tests passed, because they constructed the
  window's on-disk state from real bytes — which pins what a crash
  *leaves*, never the order that produced it. Reversing the five steps
  left all 31 of them green.
- **Do instead**: enumerate the crash point between every pair of steps
  and ask which of them is silent; a silent one is a spec defect, not a
  documented limitation. Then pin the order by **inducing** a step
  failure. The seam that works here: a directory parked at
  `<log>.snapshot`, which `rename(2)` refuses, so step (1) fails on demand
  with no injection API and no `src` change. Assert the file-level
  consequences of the order (the log byte-unchanged; an append on the same
  handle still landing in it), and prove the assertions live by mutation.
- **Promoted from**: f19, f20, t12's amended Decision 8, t21

### 2.2 An advisory lock follows the inode, not the path — and the release can lag the close
- **Scope**: `todo-store`'s layout and locking (§8), `compact()`,
  `repair()`, any future atomically-rewritten file
- **What happened**: measured on this host — std `File::try_lock` returns
  `std::fs::TryLockError{WouldBlock, Error(io::Error)}`, **not** an
  `io::ErrorKind`; two fds in one process genuinely conflict; `Drop`
  releases; and renaming a locked file **defeats** the lock, because the
  lock is on the inode the handle holds. And measured at g9 (U26): the
  release, while triggered by the close, can **lag it by a
  millisecond-scale lifetime** — a re-open immediately after another
  process (or the same process, via a different open-file-description)
  closed the same store can see a spurious `locked` with no fd holder
  anywhere. The store's code is innocent; the kernel's teardown is
  deferred.
- **Do instead**: dedicated `<protected file>.lock` files (never a
  dir-level name, so two logs in one directory cannot share one); no
  atomically-rewritten file is ever its own lock target — the store's
  existing `.lock` covers its snapshot; and any operation that renames the
  log must **reopen the append handle** afterwards, or every later append
  lands on the orphaned inode. Lock order is total: registry first, then
  sorted slugs — safe only because the registry stores immutable fields.
  A caller that opens immediately after a close and cannot tolerate a
  transient `locked` retries briefly on that one error (the protocol
  suite's bounded retry is the worked example); whether the retry belongs
  inside `FileLock::acquire` for every caller is board U27.
- **Promoted from**: t11's measurements, t12 step (5), g9's U26 isolation

### 2.3 Absent vs null collapse on a plain `Option` patch field
- **Scope**: patch/partial-update schemas — `todo-store`'s event patch
  DTOs and M3's wire patch schemas (`task_update` & co.)
- **What happened**: with a plain `Option<T>`, "clear this field" and
  "leave it alone" are the same bytes — `{"note":null}` and an absent
  `note` both deserialize to `None`, so a clearing patch replays as a
  no-op.
- **Do instead**: `Option<Option<T>>` with `#[serde(default,
  deserialize_with = "double_option", skip_serializing_if =
  "Option::is_none")]` — the pattern in `crates/todo-store/src/event.rs`,
  pinned by a test that cannot even compile under the naive `Option`
  (`a_patch_keeps_untouched_cleared_and_replaced_apart`). Patch DTOs also
  carry no `id`/`repo`/`created` and deny unknown fields, so node history
  is unreachable through the persisted surface.
- **Promoted from**: P-LIVE-003 (t10)

### 2.4 A refused atomic rename leaves its temp file behind
- **Scope**: `todo-store`'s §4.3 write path (`write_temp` +
  `rename_into_place`), `snapshot()`, `compact()`, `repair()`
- **What happened**: `Store::snapshot`'s doc promises "a failed snapshot
  changes nothing on disk". A rename that refuses leaves the pid-suffixed
  temp file beside its target, so something *is* on disk that was not
  there before — measured, not reasoned: exactly one `.tmp` survives a
  refused `compact()`.
- **Do instead**: read the promise as "nothing **in place** changed",
  which is what actually holds and what the log-untouched assertion pins.
  The debris is inert, not self-cleaning across sessions: the name embeds
  `std::process::id()`, so the same process's next `write_temp` truncates
  it and a later session's is simply left. Do not assert cleanliness on a
  failure path, and do not add cleanup without deciding it for every
  §4.3 writer at once.
- **Promoted from**: f40 / board U15 (t21)

### 2.5 A serde engine's container-framing contract lives in the `Deserializer`, not in the access impl
- **Scope**: the `dacode-json` engine (rev-pinned git dependency,
  U8/U82) and any schema that uses a fixed-length serde shape
- **What happened**: `serde::de::SeqAccess` has **no** `end()` method.
  serde drives two protocols through it: a collection calls
  `next_element_seed` until `None`, while a *fixed-length* visitor (a
  tuple, tuple struct, tuple enum variant, `[T; N]`) calls it exactly
  `len` times and returns. `serde_json` splits the work accordingly —
  `next_element_seed` **peeks** `]` and `Deserializer::end_seq` eats it
  after `visit_seq` returns. The engine had that inverted, so the symptom
  presented as "trailing characters" rather than as a tuple problem, and
  two worse defects rode along: a **truncated array parsed clean** (`[1,2`
  → `Ok((1, 2))`, precisely the property §4.3's torn-tail detection rests
  on) and `stream` rejected **every enum object variant**.
- **Do instead**: check `end_seq`/`end_map` before blaming the visitor.
  And distrust a differential suite's coverage rather than its result:
  this one compared against `serde_json` over `JValue` only, which is a
  collection all the way down and exercises none of the fixed-length
  shapes. A differential suite is only as wide as the shapes it exercises.
- **Promoted from**: P-LIVE-006 (t12, resolved upstream at `c4aebe9`)

## Category 3: The MCP wire and clients

### 3.1 §6.1's code list is closed, and two rejections have no code in it
- **Scope**: M3's server, mapping `todo-domain`/`todo-store` errors onto
  the wire
- **What happened**: the domain may not extend the versioned list
  unilaterally, which is why `ValueError` (a malformed slug/id/timestamp/
  reference) stayed codeless in the crate that raises it until U9 signed
  `invalid` and `io` into §6.1. Two graph-write rejections still have no
  code: a **forbidden `blocks` endpoint pair**, which `BlocksEdge`'s three
  variants make unrepresentable by type, and an **exhausted per-kind id
  space**, unreachable at §4.1's 10^5 scale and currently mapped to
  `duplicate`.
- **Do instead**: M3 must reject an untyped `task_link` at the wire via
  `BlocksEdge::from_ends(..) == None` — the type cannot carry the
  illegality, so the wire layer has to, and inventing a code for it is a
  spec amendment, not an implementation detail.
- **Promoted from**: f4 (t7), U9's resolution

### 3.2 A reason string handed to a caller is contract text, not a comment
- **Scope**: `todo-store`'s `RepairOutcome::Unrepairable.detail` and
  everything `todo-cli` prints from it; M3's tool-error messages
- **What happened**: one `REASON_*` constant covered two states — an
  interior fault, and a fault in a *terminated* final line, which replay
  positions at that line — and read false for the second. The cli prints
  that string as the rationale a human confirms a truncation against, so a
  false reason invites a destructive confirmation.
- **Do instead**: every state a reason can be reported for must read true
  of it; split constants rather than sharing one, give unreachable
  fallbacks their own name instead of borrowing another case's text, and
  remember the test needle is the only pin. Same discipline for the
  context half of an `io` detail: it is crate contract text and is pinned,
  while the platform's `ErrorKind` spelling beside it is not.
- **Promoted from**: f33 (t22), f31, t21's round-1 finding

### 3.3 An output DTO's struct doc reaches the wire verbatim — keep it single-line
- **Scope**: every `todo-mcp` output DTO, and any future server whose
  schemas are generated by schemars
- **What happened**: schemars carries a struct's own doc comment into the
  `outputSchema.description` verbatim, line breaks included — a multi-line
  struct doc was observed on the wire as a literal `\n` inside the
  description. The asymmetry that makes this easy to get wrong: rmcp
  strips top-level `title`/`description` from the **inputSchema** only
  (`handler/server/common.rs` `validate_and_strip`), so a params struct's
  doc may stay multi-line while its output twin's may not.
- **Do instead**: every output DTO struct doc is one line; the pin is a
  test that walks the live `tools/list` response and fails on any line
  break in any description (a property named `description` whose value is
  a schema object is skipped — it is a field, not a doc string — and
  still recursed into).
- **Promoted from**: f72 (t34's fix round; bit at t33 and again at t35)

### 3.4 rmcp dispatches each request in its own task — pin pairs, not positions
- **Scope**: every wire-level golden and every concurrent client
- **What happened**: rmcp spawns each request in its own `tokio::spawn`,
  and response arrivals measured genuinely out-of-order — many distinct
  arrival orders across repeated identical bursts, different on every
  run. A golden that writes all requests and then reads responses
  **positionally** would flake on the scheduler, not on bytes.
- **Do instead**: the golden drive is interactive — one request written,
  one response line read, repeat — so the response order is the request
  order by construction and the goldens pin request→response pairs.
  Concurrency tests that deliberately rapid-fire write all requests first
  must match each response to its request **by JSON-RPC id, never by
  position**; the out-of-order tolerance is the matching rule itself.
- **Promoted from**: f73 (t34), t38's rapid-fire drive and its measured
  arrival orders

## Category 4: Build, toolchain and macOS host

### 4.1 A filtered `cargo test` verify line can pass vacuously — three separate ways
- **Scope**: every verify line of the form `cargo test -p <crate> -- <filter>`,
  and any script or loop that builds one; rustc 1.98.1 / cargo / zsh on darwin
- **What happened**: (1) the positional after `--` is a libtest **name**
  substring filter, not a target selector, so tests sitting flat in
  `tests/golden.rs` are not selected by `-- golden`; (2) a filter matching
  nothing prints `0 passed` and **exits 0**; (3) zsh does not word-split an
  unquoted `$f`, so a loop variable holding `"log golden"` is passed as one
  literal filter — which is how "multiple filters are ANDed" got measured
  once and wrongly. They are **ORed**.
- **Do instead**: select the target (`--test <name>`), or nest every test
  in a `mod` whose name matches the filter so names come out `golden::…`.
  Treat the passed count **plus `0 filtered out` in the expected target**
  as the only proof a filtered run ran — never the exit code. Write verify
  commands literally in tracker and spec text; use `${=f}` in zsh when
  splitting is intended.
- **Promoted from**: P-LIVE-002 (t9), P-LIVE-005 (t11)

### 4.2 `tests/<name>.rs` cannot pair with `tests/<name>/mod.rs` (E0761)
- **Scope**: every integration-test target with submodules — all of
  `todo-store`'s and `todo-cli`'s
- **What happened**: `mod log;` declared in the crate root `tests/log.rs`
  is ambiguous: both `tests/log/mod.rs` and the root file itself are
  candidate paths.
- **Do instead**: declare the wrapper **inline** in the crate root —
  `mod log { mod fixtures; mod replay; mod corrupt; }` — with the
  submodules as `tests/log/<file>.rs` and no `mod.rs` in the directory.
  Names still come out `log::…`, so 4.1's filter discipline keeps working.
- **Promoted from**: P-LIVE-004 (t10)

### 4.3 Permission- and platform-corner tests must detect whether they fired
- **Scope**: `#[cfg(unix)]` tests, mode-based fault injection, any test
  whose premise the host can override
- **What happened**: a 0444/0555 permission test is uid-dependent — at
  502 it fires, under root the mode is overridden and it silently tests
  nothing. Off-platform, `#[cfg(unix)]` tests **vanish** rather than pass.
  And a fault induced by POSIX semantics (t21's EISDIR on renaming a file
  onto a directory) carries no `#[cfg(unix)]` guard, so its Windows
  spelling is unproven.
- **Do instead**: probe whether the premise is enforced (write a `.probe`
  file, restore permissions before asserting) and branch, printing a NOTE
  when the corner is not exercised — a test that cannot fail is worse than
  no test, because it is counted. Pin the crate's own contract text rather
  than the platform's spelling where the two can diverge.
- **Promoted from**: t11's permission corners, t21

### 4.4 `cargo doc` is not part of any default gate
- **Scope**: workspace gates; g5's verify line now names it
- **What happened**: four unresolved intra-doc links sat in
  `graph/drafts.rs` for two tasks because nothing ran `cargo doc`; they
  were found by hand.
- **Do instead**: `cargo doc --workspace --no-deps` is a gate line, and
  there is **one** link convention in this workspace: `[`crate::Epic`]`.
  The `](crate::` form is absent from the tree and must stay absent.
  And the gate has a **private-reachability blind spot**: rustdoc
  renders only publicly reachable items by default, so a private
  module's own doc page — and private items inside public modules — are
  never rendered and their broken intra-doc links are invisible to the
  plain invocation; run it with `--document-private-items` too. One
  broken link (`[`BTreeSet`]` in `reference.rs`, the type not imported
  into its module) sat behind that blind spot from t6 until the M2 gate,
  visible only under the flag; proven by mutation there — reverting the
  one-word fix prints exactly that warning under the flag and none
  without it.
- **Promoted from**: f10 (t8/t15), f48 (g7)

### 4.5 Fault seams that need no production code
- **Scope**: any test that must make a write, a rename or a read fail
- **What happened**: the obvious way to test a failure path is an injection
  API in `src`, which is a production seam nobody wants and which f25
  explicitly parks for the store's diagnostics surface.
- **Do instead**: the filesystem provides deterministic ones — park a
  **directory** where a file is expected (`rename(2)` refuses, EISDIR);
  set a mode the uid enforces (probe first, 4.3); remove a file the reader
  needs; splice real serialized bytes to build a torn or malformed
  document. Never hand-write JSON for a fixture: build it through the
  crate's own codec over real values, so drift fails loudly.
- **Promoted from**: f19/t21, t10's f8 splice discipline, t11

### 4.6 A test helper used by one target is dead code in the others — per-target homes
- **Scope**: every `tests/common/` module in a multi-target test layout
- **What happened**: `common/mod.rs` is compiled by **every** test target
  that declares `mod common;` — a helper only one target calls is dead
  code in the others, and clippy's `dead_code` kills the build (it cost a
  fix round at t37: the `reopened` helper, write-only, had to move to the
  write target's root). A single-file target's root is the file itself,
  so a wire-only helper lives in `tests/wire.rs`, not `common/`.
- **Do instead**: per-target homes are the structure — a helper goes
  where its consumers are; lifting into `common/` requires callers in
  every target that compiles it (or a scoped `#[allow(dead_code)]` on the
  `mod common;` declaration, which `tests/wire.rs` already carries for
  the read/write-only helpers it does not call). Duplication between
  self-contained targets (protocol.rs's own drive, wire.rs's rapid-fire
  drive) is the accepted cost — the alternative edits a file the task
  does not own.
- **Promoted from**: t37's `reopened` discovery, t38's home rulings

### 4.7 Non-ASCII in an inline `git commit -m` string gets mangled
- **Scope**: every commit on this host (zsh + the bash tool's string
  handling); this repo's messages cite `§` liberally
- **What happened**: `§9` and `§2` in a commit message body landed as
  `SS9`/`SS2` (commit `230d49c`, t41) — the section sign did not
  survive the multi-line inline `-m` argument.
- **Do instead**: write any commit message containing non-ASCII via a
  file — the file tool writes it, `git commit -F <file>` consumes it;
  never inline `-m` for a message carrying `§`, `—` or any non-ASCII
  punctuation.
- **Promoted from**: P-LIVE-009 (t41)

### 4.8 A digest is a claim that names its algorithm — or two honest verifiers mismatch
- **Scope**: every sha-pinned baseline, binding round and
  frozen-baseline list, in task specs, reviews and evidence lines
- **What happened**: t50's binding round reported a sha "mismatch"
  between implementer and reviewer that was no mismatch at all — one
  had hashed SHA-1, the other SHA-256; t34's binding round had the
  same scare before it. Two honest verifiers, two tools, one false
  contradiction.
- **Do instead**: every digest quoted in a spec, a review binding or
  an evidence line names the tool that produced it ("shasum -a 256",
  the practice since t36's "shasum prefixes, the tool named"), and a
  binding round that mismatches first asks which algorithm each side
  used, then whether the bytes moved.
- **Promoted from**: the t50 binding round (t34's scare is the
  corroborating record)

## Category 5: The harness itself (tools/, agents, docs)

### 5.1 A task spec's safety argument is a claim to falsify, not ground truth to instance-check
- **Scope**: every task spec the orchestrator writes, and every reviewer
  prompt spawned against one
- **What happened**: t12's Decision 8 prescribed `compact()`'s step order
  with a crash-safety rationale attached. The rationale silently
  presupposed that a prior snapshot existed and that its offset was large;
  both were false, and the two resulting corners lost events silently
  (2.1). The secondary verified that the code matched the spec and that the
  spec's stated consequence held in the case the spec's own reasoning
  instantiated — and returned `ok`. The primary re-derived the state space
  a round later, found both, and retracted its own earlier clearance of the
  same area. t8's spec inverted the signed D6 tiebreaker inside a test
  description: the second M1 task whose *spec* was wrong and whose review
  caught it.
- **Do instead**: a spec author states the **state space** a safety
  argument was checked against, not just its conclusion; a reviewer prompt
  asks for falsification ("which reachable state makes this argument
  false?") and not for conformance ("does the code match the spec?"). When
  a reviewer finds the spec wrong, amend the spec — do not qualify the
  docs around it.
- **Promoted from**: f20 (t12, both reviewers), t8

### 5.2 Prose claims are artifacts: a number or a meta-claim nobody re-measures is a defect waiting to be rediscovered
- **Scope**: source doc comments, board entries, tracker evidence, review
  briefings, commit messages
- **What happened**: the same class, five times in M1 — three successive
  versions of one doc comment stated a measurement that did not reproduce
  (fixed by deleting the numbers and pointing at the pin); a board entry
  asserted both reviewers caught something one of them filed as a low; a
  mutation count did not match the mutations named beside it; an
  implementer's report said three tests failed where four assert that exit
  code; and a re-review briefing claimed a tracker finding existed when it
  did not, which both reviewers blocked on.
- **Do instead**: no measurement in a source doc comment — state the
  mechanism and name the test that enforces the property (the rule is in
  `.agents/roles/_rust.md`'s comment section, which is where a coding agent
  will see it at write time). Never assert a fact about the review process
  inside the artifact under review: the fix to the previous finding
  falsifies it. Attribute findings to the role that filed them, at the
  severity it filed them at. If a claim cannot be re-derived by a reader,
  delete it rather than restating it more carefully — that was tried three
  times and each restatement was wrong differently.
- **Promoted from**: f30, t15's process finding, t19, t20's correction,
  t21 round 2

### 5.3 A commit that touches an indexed doc cannot carry its own regenerated index
- **Scope**: `tools/agents-index.py`, `.agents/knowledge/INDEX.md`, any
  commit touching an indexed `AGENTS.md`
- **What happened**: the index records `ver` = `git log -1 -- <doc>`, the
  doc's last-touching commit. Regenerating the index *inside* that commit
  records a commit that does not exist yet, so `--check` reports stale
  until a backfill regenerates it. (The original report of this was itself
  wrong and was refuted by reading the tool: an *empty* commit touches no
  doc and leaves `--check` green — staleness follows doc-touching commits
  only.)
- **Do instead**: commit the doc, then backfill the index (or the hash) in
  a follow-up — the `7959281`/`3606578`/`7282d28`/`673a49f` pattern. Do
  not "fix" it by hand-editing `INDEX.md`; it is generated. **Future
  defect, unhit**: crate-level `AGENTS.md` files would put `lag` in the
  output, and then *every* crate commit fails `--check`. The preferred fix
  is exempting `ver` from `--check` rather than dropping the column, since
  `ver` is also `--stale`'s lag input.
- **Promoted from**: f24 (t15, corrected in the same tracker)

### 5.4 Subagent spawn can die instantly with "Connection reset by server"
- **Scope**: harness dispatch via the task tool, observed on the
  implementer binding; M1 t6 and t7
- **What happened**: spawns failed instantly, twice on a ~7 KB inline
  prompt and then twice more on ~1.3 KB and ~0.4 KB prompts, while a
  control spawn (explore, tiny prompt) succeeded in between and the third
  identical attempt landed. So prompt size is not the trigger; the root
  cause is unconfirmed and looks like transient provider/relay trouble on
  the heavy-model binding.
- **Do instead**: keep the dispatch protocol — decisions settled into a
  scratch spec file (`tmp/<task>-spec.md`) plus a short spawn prompt
  pointing at it. On failure, fire a control spawn to isolate binding vs
  infrastructure, then retry the **identical** prompt; both incidents
  succeeded on attempt 3. Escalate only past that (agent-rules.yaml →
  `escalation`).
- **Promoted from**: P-LIVE-001 (t6, recurred t7)

### 5.5 A per-seed coverage pin's constants are part of the pin
- **Scope**: seeded property tests — `todo-domain`'s `graph/tests/`
  coverage counters; any test whose non-vacuity depends on generator
  reach
- **What happened**: three M2 tasks re-picked `GRAPH_SEEDS`/steps, and
  every re-pick was forced by a measurement, not a preference: t27's
  sole-hold branch found one seed never held a task by an epic gate
  alone, so the set became hand-tuned; t28 needed a *different* set,
  because whole-batch shapes need deeper graphs than per-task shapes;
  t29's ready-gate pin was killed by exactly one seeded test at one of
  five seeds with a one-id margin, and the per-seed assertion then
  forced a re-pick with every coverage branch re-measured at the new
  seeds. The unasserted counter is the same defect one level down —
  f47's dead-counter class: a coverage test that asserts the union
  across seeds stays green while a seed change vacates the very branch
  it exists to pin.
- **Do instead**: the constants (seeds, step count, limit) are part of
  the pin — changing them is a re-measurement event that re-derives
  every coverage branch, never a tuning knob. Assert coverage **per
  seed**, read the traced half off the built node rather than off the
  draw (f9's label/effect decoupling), and document the constants'
  rationale mechanism-only — why these seeds, not what they measure.
- **Promoted from**: t27, t28, t29; f47's dead-counter item, f50's grid

### 5.6 A citation's content is a claim, and so is its signedness
- **Scope**: task specs, reviewer prescriptions, board prose — anywhere
  a line number or a decision id is quoted
- **What happened**: five consecutive M2 task specs carried claims that
  did not reproduce, and the missing half was always the citation's
  *content*: t29's spec attributed `deny_unknown_fields` and the wire
  key order to D15, where D15 is §6.1's error carrier, D14 is the key
  order, and `deny_unknown_fields` appears nowhere in the decision
  record; t31's spec called §6.1 "the closed twelve-code list"
  (thirteen — U9 added `invalid` and `io`), glossed `unknown_id` with
  words §6.1 never says, mis-stated a mutant's kill set, and cited
  `node.rs` field positions off by one. The sharper half: a reviewer
  prescription anchored D7 on `decisions.md:848` as *signed* text —
  that line sits inside D16's unsigned dossier, so following it would
  have laundered a draft into a spec.
- **Do instead**: re-read the cited line when writing the citation —
  quoting where something lives without re-reading what it says is the
  defect. And check the cited thing is signed (a resolution banner,
  normative spec text), not a drafted dossier awaiting a vote. A line
  number that reproduces says nothing about the sentence beside it.
- **Promoted from**: f53 (t29), f62 (t31), t29's second candidate

### 5.7 A class sweep is only as wide as the grep that runs it
- **Scope**: doc-claim and defect-class sweeps anywhere in the tree; the
  `verify:` lines written for them
- **What happened**: t30's spec listed twelve "nothing written / nothing
  on disk / untouched" claims, cleared eleven, and review found two more
  of the same class in the same crate — "nothing was **opened**, read or
  written", falsified by `open_and_try_lock` opening the lock file. The
  sweep had greped one phrasing of the defect instead of the defect (an
  exhaustive claim narrower or looser than its mechanism). The same
  blindness hit the task's own verify line: it asked for
  `rg 'never reported through it'` to return nothing after a fix whose
  content changed the four words *before* that substring — a check that
  could never pass.
- **Do instead**: name the defect class first, enumerate its plausible
  surface forms ("nothing was opened/written/created/changed", "no
  trace", "leaves nothing", "untouched"), and grep each. Then run every
  verify-line grep against the intended fixed state before writing it —
  a check that cannot pass is not a check.
- **Promoted from**: P-LIVE-007 (t30), f56

### 5.8 Whoever writes a contract sentence cannot falsify it
- **Scope**: doc-only and spec-text commits; reviewer prompts for prose
  changes
- **What happened**: three of t30's four review rounds found a false
  clause in text the orchestrator had just written — "a failure **at or**
  after the temp file's creation" (false when the creation is what
  refused; the reviewer built the two states rather than reading the
  summary), debris reuse scoped per process where `temp_path_for` keys
  the name per store directory, and "the whole effect **of the
  attempt**" against `lock_registry`'s `create_dir_all` running strictly
  before the acquire. Third occurrence after t11's round 3 and f47's
  round 1.
- **Do instead**: for a prose change, the reviewer prompt must ask for
  falsification against the code path (build the state, name the
  file:line), and the author states, per clause, what it was checked
  against. Applying a reviewer's prescribed wording verbatim is safe;
  paraphrasing it is what costs the extra round.
- **Promoted from**: P-LIVE-008 (t30), t11 round 3, f47 round 1

### 5.9 A task spec's own decision numbers must never reach a committed file bare
- **Scope**: every task spec author, every doc comment written during
  implementation
- **What happened**: a task spec that numbers its own decisions D1–D12
  collides with `decisions.md`'s committed ids — and first-round doc
  comments cited the spec-local ids as if they were the committed ones
  (`(D7)` for the time source where committed D7 is the importer
  dialect, `(D9)` for server-local DTOs where committed D9 is the
  statuses). A reader resolving those citations against `decisions.md`
  lands on the **wrong decision** — the citation-identity defect (5.6)
  in its most damaging form.
- **Do instead**: a committed file cites only committed ids, or
  self-contained wording with the reasoning inlined. Spec-local decision
  numbers stay in the spec (prefixed where ambiguity is possible); the
  implementer's sweep is `rg -o 'D[0-9]+'` over the diff, resolving
  every hit against `decisions.md`.
- **Promoted from**: f70 (t33; five wrong citations caught in one sweep)

### 5.10 A comparison test passes on two identical failures — assert the premise
- **Scope**: every test that compares two runs, two responses, or any
  two artifacts for equality
- **What happened**: the twice-run byte-identity test compared run one's
  responses to run two's — and a uniformly-broken server answering every
  request with the **same** error passed it: two identical failures are
  still byte-identical. The mutation that broke the server survived
  until the premise ("these are happy paths") was asserted.
- **Do instead**: a comparison test asserts its premise first — the
  responses are successes (`isError` false), the values are present, the
  state is what the comparison presupposes — and only then compares.
  The general form: equality is only meaningful between two things
  already established to be the right kind of thing.
- **Promoted from**: f97 (t38; the M4 cache-bypass mutation survived
  the unpremised comparison)

### 5.11 A live board is a moving target — every census is point-in-time
- **Scope**: the importer's live probes and imports (t46/t47/t48), any
  measurement taken against a sibling checkout
- **What happened**: camera's board grew its own `U170` while t46 was
  still running (the study read 177, the import 178); ikvm-html grew
  `U129` after t47's probe (132 → 133 by t48); the dacode sibling
  happened to sit at the fixture's snapshot commit, which is luck, not
  a property. A census quoted without its HEAD is a figure nobody can
  reproduce — 5.2's rule with a live source underneath it.
- **Do instead**: record the sibling's HEAD beside every live census
  (t48's evidence carries all three), treat a probe's numbers as
  predictions for the next read, and never write a live census into a
  source doc comment (f105's first low is exactly that). A probe that
  must edit a sibling's board (t42's 22/23 strike) commits the edit in
  the sibling's own repo — their tree is never left dirty for their
  owner, and the probe's artifacts never land here.
- **Promoted from**: the t46/t47 mid-task drift records; t48's
  drift-baseline HEADs

### 5.12 A dialect study's inventory is a claim — the parser's strict refusals are the detector
- **Scope**: the import dialects (`src/import/`), any parser of a live
  document
- **What happened**: t42's strictness rule fired on six real
  live-board shapes the t41 study's inventory had missed; t47 found
  one more (the resolution banner's closing shape) beyond its own
  study. The studies were careful — the boards are simply larger than
  any one reading of them. The silent alternative (skip what does not
  parse) is the anti-silent-omission defect in its purest form: a
  parser that skips an item reports less than the truth and still
  looks complete (f43's class).
- **Do instead**: a parser of a live document refuses what it does not
  recognize, by name, with the line — and the refusal is the detector
  that finds what the study missed. Every widening is measured grammar
  (the real shapes, counted), pinned by a synthetic test, never a
  guess and never a skip.
- **Promoted from**: t42's six widenings; t46's grammar widenings;
  t47's banner-closing widening

### 5.13 A measured divergence between a gitignored task spec and the run folds into the evidence — it is not a finding
- **Scope**: findings bookkeeping (the tracker's f-ids), every task
  whose spec predicts kill sets, counts or phrasings
- **What happened**: M3 filed a stream of f-ids (the f75–f98 class)
  whose entire content was "the spec said X, the run said Y" — each
  superseded the moment it was filed, because the spec is gitignored
  scratch that dies with the branch, while a finding is supposed to
  outlive it. The tracker filled with items nobody would ever do.
- **Do instead**: a measured divergence between a task spec's
  prediction and the run is recorded in the task's evidence line
  (where the measurement already lives) and the spec is corrected in
  place — 5.1's "amend the spec rather than qualify the docs". An
  f-id is reserved for a defect in a committed artifact or a lesson
  that must outlive the branch. The convention was signed into
  practice by `a86b1cc` (2026-10-04).
- **Promoted from**: the folding convention (`a86b1cc`); the
  superseded notes on the f75–f98 class are the record of what it
  replaced

### 5.14 A config the harness calls a single source of truth needs a duplicate-key refusal at its validator
- **Scope**: `.agents/config/agent-rules.yaml`, `tools/agents-sync.py`,
  any future single-source config
- **What happened**: f42 — two top-level `commands:` keys. PyYAML (like
  most YAML loaders) keeps the last, so the whole build-command block
  was dead config that read like live config; nothing noticed until the
  failure was found by hand. f44 generalised it: the loader was plain
  `yaml.safe_load`, so nothing prevented a recurrence.
- **Do instead**: the loader at the config's validator refuses a
  duplicate key in any mapping, at any depth (t57's `StrictLoader`,
  proved by splicing duplicates and reverting sha-verified). A
  single-source config gets the refusal, not a lenient read plus hope.
- **Promoted from**: f42, f44, t57

### 5.15 A tool whose stdout is a machine format keeps diagnostics off stdout
- **Scope**: `tools/progress.py`, any tool a consumer pipes
- **What happened**: f109 — `--json --refresh` printed the import
  census to stdout, so a consumer piping the JSON got a non-JSON line
  first.
- **Do instead**: stdout carries only the declared format; everything
  else goes to stderr. Pinned by `tools/progress-serve-check.py`'s
  f109 checks.
- **Promoted from**: f109 (found t54, fixed t55)

---

## Quick Reference: Top Rules

Distilled from the entries above at the M1–M4 gates.

1. **Select the test target, never a name filter** — `--test <name>`, and
   the only proof a filtered run ran is the passed count plus
   `0 filtered out` (4.1, 4.2).
2. **Induce the fault; do not construct its aftermath** — a directory
   parked where a file is expected, a probed mode, spliced real bytes
   (4.5, 2.1).
3. **For any multi-file write sequence, name the crash point between every
   pair of steps and ask which is silent** (2.1).
4. **Prove an assertion live by mutation, then revert and verify the sha** —
   a test that cannot fail is worse than no test (2.1, 4.3).
5. **No measurement in a source doc comment; no review-process claim in the
   artifact under review** (5.2).
6. **A spec's safety argument states the state space it was checked
   against; a reviewer is asked to falsify it, not to confirm conformance**
   (5.1).
7. **Counters and other allocation state come from the log, never from the
   live nodes** (1.1).
8. **The lock is on the inode: renaming a locked file defeats it, and a
   rename of the log means reopening the append handle** (2.2).
9. **Caller-facing strings are contract text and are pinned by a needle**
   (3.2).
10. **A doc-touching commit backfills its own index and its own hash in a
     follow-up** (5.3).
11. **A citation's content is a claim: re-read the cited line, and check
    what it cites is signed** (5.6).
12. **A class sweep enumerates the defect's surface forms before
    grepping, and every verify-line grep is run against the intended
    fixed state before it is written** (5.7).
13. **An output DTO's struct doc is one line — schemars ships it to the
    wire verbatim, and rmcp strips the input side only** (3.3).
14. **A comparison test asserts its premise before it compares: two
    identical failures are still identical** (5.10).
15. **A live census is point-in-time: record the HEAD beside it, and
    never write it into a source comment** (5.11, 5.2).
16. **A digest names its algorithm and its tool — two honest verifiers
    otherwise mismatch** (4.8).
