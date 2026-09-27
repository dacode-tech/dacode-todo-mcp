# TODO — dacode-todo-mcp

The committed project board: what has landed, what is next, and what is
blocking. Rules, which `tools/attention.py` and the reviewers enforce:

- **A status change is a one-line edit; resolved items are struck, never
  deleted** — gaps in the `U` numbering are information.
- **The id lives inside the leading bold run** (`**U7 · Title**`) because
  the tooling keys on it; checking an item off must not change what the
  tools read.
- **Reasoning never goes on the board.** It goes in
  `docs/implementation/decisions.md` or the spec. The board is status.
- **A delivery that does not move this file is a review finding.** Update
  it in the same change as the work.

## 1. Current state

**Phase: M1 executing — fully unblocked 2026-09-26.** U4 answered
(`.agents/knowledge/workflows/golden-files.md`), U5 signed (spec §4.3),
U8 approved (serde + derives on domain, `dacode-json` by path, tempfile
dev-only, `rust-version` → 1.95). The M1 tree is open in the session
tracker: three domain commits (value types §3, task graph §5.1,
canonical order §5.2), five store/cli commits (canonical serialization
+ golden harness, JSONL log + `Store` trait §4, D4 layout + registry +
locking §8, snapshots + compaction, minimal `todo-cli` inspector), and
the M1 exit gate. t6 (value types), t14 (fractional-second timestamps,
a user decision), t7 (the task graph §5.1) and t8 (the canonical order
§5.2) have landed — the domain layer of M1 is complete — and t9 (the
store's canonical serialization + golden harness) is the first store
commit; next is t10, the JSONL event log + `Store` trait.

| Blocking | What | Action |
|---|---|---|
| — | nothing: U4/U5/U8 all resolved 2026-09-26 | execute the M1 tree |

(U7 remains open but non-blocking.)

## 2. Milestones

Authoritative names: `.agents/config/agent-rules.yaml` → `milestones`;
exit criteria: spec §14. Priority is derived from the Gated-on column by
`tools/attention.py`, never hand-written.

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | spec settled: §12 items 1–3 signed (done 2026-09-25); Cargo workspace skeleton with the lint regime; all gates green | — | done 2026-09-25 |
| M1 | domain model + storage engine: graph, statuses, persistence behind the `Store` trait, golden files | — | in progress — tree open; U4/U5/U8 resolved 2026-09-26, dispatch started |
| M2 | queries: repo/epic selection, dependency resolution, deterministic `next_task`; registry with path updates | — | not started |
| M3 | MCP server: the full §6 tool list over the wire, protocol-level tests, default-repo slug config (D8), smoke against a real client | — | not started |
| M4 | adoption: camera-dialect importer first (D7), board fixtures checked in per §10; ikvm-html and dacode follow; the three repos tracked for real | — | not started |
| M5 | packaging, docs, cwd auto-detection (deferred by D8), polish | — | not started |

## 3. Done

Oldest first. One line per delivery, with the commit hash once committed.

- 2026-09-25 — **harness bootstrap**: the agent system (five generated
  agents, two slash commands, three skills), the repo tooling
  (`attention.py`, `tasks.py`, `agents-sync.py`, `agents-index.py`,
  `check-links.py`), the seed spec with eight §12 decisions, this board
  and the decision dossier. Landed as `e6d4761`; the tracker-keying
  knowledge note as `5062862`.
- 2026-09-25 — **D1–D8 signed**: all eight decisions signed by the
  user, with amendments — D4 resolved to (c) per-repo store directories
  under a shared registry (no cross-project shared database), D8 to (a)
  plus a per-instance default repo slug. §12 items 1–8 deleted,
  §2–§9/§13 made normative. **D9–D11 drafted** from the sign-off
  message's new requirements: status vocabulary + deferred state;
  references and specs/decisions as server entities; next-items batch +
  derived weight.
- 2026-09-25 — **D9–D11 signed; §12 closed**: statuses
  open/in_work/blocked/deferred/closed/wontfix, until-targets riding
  `blocks` edges (no calendar, no auto-reactivation); `references`
  lists plus `decision` and `doc` node kinds with decision→work
  `blocks` edges; `next_items` batch with derived weight and an
  agent-set, display-only `priority`. Spec normative throughout, §9
  mapping and §11 non-goals amended, §6 tool table extended.
- 2026-09-25 — **D9/D11 same-day amendments** (user-directed):
  deferred summary (`total`/`ready`) on next_task/next_items plus a
  ready filter on task_list (D9); task-scoped next_items and the §6.2
  graph_show complete-graph dump (D11). D10 concretization proposed in
  the sign-off thread, lands on the user's ok (U6).
- 2026-09-25 — **D10 concretization landed (U6 struck)**: per-kind id
  prefixes (U/E/D/S), decision transitions enforced
  (`missing_resolution`), audited `doc_set_path`, `#anchor` doc
  citations, `decision_update`/`doc_update` tool verbs. The user also
  confirmed canonical JSON as the `graph_show` wire format (§6.2).
- 2026-09-25 — **M0: Cargo workspace skeleton**: four crates
  (todo-domain, todo-store, todo-mcp, todo-cli), the lint regime
  declared once at workspace level (six panic-family clippy denies +
  wildcard_imports + unsafe_code + missing_docs) and inherited per
  crate, cfg_attr(test) exemption at both library roots, zero external
  dependencies (Cargo.lock = exactly the four members), documented
   crate roots stating each §2 boundary. All four code gates green for
   the first time; dual-reviewed, first spawn of all three worker roles
   confirmed the model bindings. Landed as `57b531c`.
- 2026-09-26 — **M1 t6: domain value types**: spec §3 as typed Rust —
   ids/slugs (U/E/D/S prefixes, `slug:U7` display form), canonical RFC
   3339 timestamps and dates, references as a canonical-sorted set with
   validating deserialization, closure discipline as errors total over
   §6.1's eight domain codes, five node structs with serde derives.
   serde is the crate's only dependency (U8); `rust-version` 1.95.
   Dual-reviewed in two rounds; surfaced a §3 Doc-references erratum
   (aligned to the signed D10 record), the timestamp canonical-subset
   sentence in §3, and U9 (M3 wire code for invalid input). 45 tests,
   all gates green. Landed as `730fc7a`.
- 2026-09-26 — **M1 t14: fractional-second timestamps** (user
   decision, recorded in §6): the canonical form becomes
   `YYYY-MM-DDTHH:MM:SS.sssssssssZ` — fraction always present, fixed
   nine digits, zero-padded from 0–9 input digits, longer rejected
   (never rounded), leap `:60` composes with a fraction; one spelling
   per instant, fixed-width Ord chronological. §3's timestamp sentence
   amended. 46 tests, dual-reviewed in one round. Landed as `73849bf`.
- 2026-09-27 — **M1 t7: the task graph (§5.1)**: the single-repo
   in-memory graph — `parent` tree kept as validated fields (§3's
   signed node shapes; no edge table), `blocks` DAG with exactly the
   three legal edge shapes (task→task, decision→task, decision→epic;
   the rest unrepresentable by type, so §6.1's closed code list needed
   no extension), write-time cycle rejection with a deterministic
   `[s, t, …, s]` path, atomic rejected writes, `relates` as canonical
   undirected task↔task annotation, epic-delete cascade (terminal
   tasks delete with their edges; anything else is
   `orphan_not_allowed`), per-repo per-kind sequential id allocation
   that never reuses a number plus an explicit-id insert path for t10's
   replay that only ever raises the counters, and the closure
   discipline re-run on every mutation. 47 graph tests including
    seeded mutation sequences against an independent whole-graph
    invariant scan; 94 crate tests total; dual-reviewed in one round
    (four low findings, recorded in §6 and the session tracker). Landed
    as `7ddfd79`.
- 2026-09-27 — **M1 t8: canonical order (§5.2)**: the deterministic
    topological derivation over the t7 graph — best-first Kahn over
    `TaskToTask` edges keyed `(epic number, task number)`, which is the
    signed D6 tiebreaker itself: topology constrains, the key chooses
    among simultaneously ready candidates. Order is never stored; no
    new state. The repo-scoped derivation is a total function
    (`Vec<TaskId>`); the epic-scoped one is the induced subgraph
    (`Result` only for an unknown epic). Statuses do not filter;
    `priority` (D11) and `relates` (§5.1) never enter the key; decision
    gating edges feed M2's actionability, not the order. The defensive
    Kahn-remainder fallback appends in key order — unreachable via the
    public API, pinned by the seeded tests, never a panic. 14 new tests
    including an independent naive best-first recomputation pinning
    lexicographic minimality on seeded DAGs, and byte identity across
    shuffled-insertion isomorphic rebuilds; 108 crate tests total;
    dual-reviewed in one round with zero findings (the implementer
    surfaced a task-spec parenthetical that inverted D6 — resolved
    toward signed §5.2, confirmed by both reviewers). Landed as
    `6cc4691`.
- 2026-09-27 — **M1 t9: store canonical serialization + golden
    harness**: the first store commit — `codec::to_bytes`/`from_bytes`
    over `dacode-json` (U8; path dep declared once at the workspace
    root), the engine sealed inside the crate (§2: no engine type in
    any public signature). `StoreError::CorruptStore { offset, detail }`
    makes corruption a named error carrying the engine's byte offset
    (§4.3, §6.1's `corrupt_store`); `version_mismatch`/`locked`
    variants are t10/t11's, growth path documented. `FORMAT_VERSION =
    1` is §4's single version source — goldens' `.v1` naming references
    it, t10's log header and t12's snapshot header will carry it. The
    golden harness per the U4 workflow: one shared `assert_golden`
    helper (`UPDATE_GOLDEN=1` temp+rename regeneration, bounded
    line-level diff on mismatch), every test nested in `mod golden` so
    the `-- golden` name filter selects all (f11). First golden
    `serialize_nodes.v1`: five fixture nodes as JSONL, pinning
    declaration field order, `Doc.references` after `version`, the
    30-byte nine-digit-fraction timestamp spelling, canonical reference
    sorting, and every enum wire spelling — the two t6/t14 review-nit
    pins. The trailing-bytes contract for t10's line splitter probed
    and pinned: non-whitespace rejected at the stray offset, trailing
    LF clean. `testcodec` stays as the domain-local serde-contract
    check (§2 forbids domain→store deps). 11 store tests; the domain's
    108 pass unmodified; Cargo.lock 11 → 13 (`dacode-json` + `zmij`).
    Dual-reviewed in one round, three low nits dispositioned in §6 and
    the tracker. Landed as `fc23f27`.

## 4. Follow-ups

Unowned work. Take the next unused `U` id; never renumber, never reuse.

- [x] **U1 · Sharpen §4.1's engine comparison with a measurement.** A
      replay+snapshot micro-benchmark at 10^5 nodes for the JSONL option,
      and a redb write/read equivalent — numbers turn D1 from a lean into
      a decision. `action: measure` material for D1, though D1 itself is a
      vote. **Superseded 2026-09-25**: D1 signed without the
      measurement; these numbers become decision material only if the
      redb fallback (D1's banner) is ever proposed.
- [ ] **U2 · Survey rmcp's tool macros against §6.** Confirm the signed
      tool list is expressible with stable error codes and ordered list
      responses as rmcp schemas; record findings in §6 before M3
      implementation (D5 is signed). Read the rmcp version's source; do
      not guess from its README.
- [ ] **U3 · Collect the three sibling board dialects as fixtures.**
      Snapshot camera/ikvm-html/dacode `TODO.md` + decisions files into
      the future importer's test fixtures (§9, §10), with a short dialect
      note each — needed before M4; D7 is signed camera-first, so the
      camera fixture is the critical one.
- [x] **U4 · Draft the golden-file and determinism test scaffolding
      shape** (spec §10) so M1 starts with the harness, not after it:
      where fixtures live, how they regenerate, how the byte-compare test
      is written once and reused. **Answered 2026-09-26**:
      `.agents/knowledge/workflows/golden-files.md` — `tests/golden/`
      layout with format-version naming, one shared `assert_golden`
      helper, the `UPDATE_GOLDEN=1` regeneration ritual, twice-run and
      round-trip determinism tests. The harness itself is the first
      store commit of the M1 tree.
- [x] **U5 · Decide the store's backup/corruption-recovery story** and
      write it into §4's requirements or §8: what a torn last line of an
      event log means, and whether snapshots are the recovery unit.
      Now scoped to the signed D1 engine (JSONL event log) and the D4
      per-repo layout. **Resolved 2026-09-26**: the proposal (§6 entry)
      was approved by the user as drafted and landed as spec §4.3
      "Crash and recovery", with the D1 banner amended in
      `decisions.md`; plain files stand, no redb decision opened.
- [x] **U6 · Land the D10 concretization on the user's ok**: per-kind
      id prefixes (task `U`, epic `E`, decision `D`, doc `S` —
      `camera:D3` echoes the board registers); decision status
      transitions enforced like task closure (`signed`/`superseded`
      require `resolution` — a `missing_resolution` code in §6.1);
      signing flips status and its `blocks` edges simply stop gating
      (no edge mutation, history preserved); milestone gating modeled
      as `blocks` edges to epics, not a field; `doc_set_path` audited
      like `repo_set_path`; references entries `{kind, target, note?}`
      with `#anchor` targets for registered docs. Proposed in the
      sign-off thread 2026-09-25; **approved and landed the same day**
      in §3/§6/§6.1 + the D10 banner.
- [ ] **U7 · Clarify the "six denied lints" wording.** Spec §10's gate
      sentence and AGENTS.md's coding-guidelines item 1 say "the six
      denied lints of `.agents/roles/_rust.md`"; the workspace regime
      declares six panic-family clippy denies (`unwrap_used`,
      `expect_used`, `panic`, `unreachable`, `indexing_slicing`,
      `exit`) plus `wildcard_imports` (from _rust.md's import rule),
      plus `unsafe_code` and `missing_docs` at the rustc level. The
      spec is signed text: either bless §10 to enumerate the full set,
      or keep "six" as the panic-family shorthand and let
      wildcard_imports ride its own rule. Surfaced by the M0 dual
      review; until it is answered, the workspace exceeds the spec
      floor, which is the safe direction.
- [x] **U8 · Approve the first external dependencies (M1 store).**
      Revised 2026-09-26 after the user offered their own JSON library:
      `serde` (+derive; the schema layer), **`dacode-json`** as the
      serializer (path dep on `/Users/mouse/RPM/dacode/rust/json-tiers`;
      probed on this host — output byte-identical to serde_json,
      twice-run stable, round-trip clean, torn-line errors carry byte
      offsets, panic-free under the same clippy regime), `tempfile`
      (dev-only), and workspace `rust-version` 1.85 → 1.95
      (`dacode-json`'s floor; subsumes the std `File::try_lock` bump, so
      no locking crate). Open sub-questions: serde derives on
      `todo-domain` types (recommended; §2's "depends on nothing" read
      as *no workspace deps*) vs store-side DTO mirrors; path dep vs
      registry pin. The user granted modifications to `dacode-json` if
      M1 needs one (proposed as a diff; that repo's commits stay with
      its owner). Gates the store commits (t9–t13) and shapes t6's
      derives. Ask-First per AGENTS.md. **Resolved 2026-09-26 —
      approved as recommended**: serde derives on `todo-domain` (spec
      §2's rules clarified: *no workspace deps* = no sibling path deps;
      the serializer stays confined to `todo-store`), `dacode-json` by
      path, `tempfile` dev-only, `rust-version` 1.95. The deps ride the
      first code commits (t6: serde + the bump; t9: `dacode-json`).
- [ ] **U9 · Decide the wire code for invalid input (M3).** t6's error
      model keeps `DomainError` total over exactly the eight §6.1 codes
      and puts malformed-input failures (bad slug/id/timestamp/date/
      reference) in a companion `ValueError` with no code — none of the
      eight fits, and §6.1's list is versioned, so the domain may not
      extend it unilaterally. At M3 the wire layer needs the decision:
      a new §6.1 code (e.g. `invalid`) or a per-case mapping rule.
      Surfaced by the t6 dual review; record the choice in §6.1 before
      the MCP surface lands.

## 5. Debt

Deliberate shortcuts, acknowledged rather than fixed. Same id rules as §4.

_No debt recorded yet — nothing has been built to take a shortcut in._

## 6. Session log

Newest first. One entry per session, written for someone who was not
here. The final entry of a session is the handover
(`.agents/skills/todo-next-imp/SKILL.md` defines the shape).

### 2026-09-27 — t8: canonical order

Dispatch was cheap because the spec was pre-written last session
(`tmp/t8-spec.md`): one short spawn prompt, one implementer landing —
no P-LIVE-001 recurrence, no control spawn needed.

The process earned its keep elsewhere: the task spec's required-test
1(i) parenthetical ("a later epic's U1 precedes an earlier epic's U9")
inverted the signed D6 tiebreaker. The implementer surfaced the
contradiction instead of silently fixing it, followed §5.2 — the
earlier-registered epic wins between topologically independent tasks —
and both reviewers independently confirmed the direction. Lesson for
pre-written specs: example orders inside test descriptions are spec
text too, and get checked against the signed decision before dispatch.

Outcome: `graph/order.rs` (repo-scoped total `canonical_order`,
epic-scoped induced-subgraph `canonical_order_of_epic`, private
`(epic number, task number)` key with the M2 seam documented) plus 14
tests — tiebreaker levels, byte identity across twice-runs and
seeded-shuffled isomorphic rebuilds, status/priority/relates
indifference, seeded DAGs checked against an independent naive
best-first recomputation (lexicographic minimality), degenerate
shapes; 108 crate tests total. All four gates green in the
implementer's, both reviewers' and the orchestrator's runs. Dual
review approved in one round with zero findings; the primary traced
Kahn correctness (unique keys, exactly one in-degree decrement per
edge, panic-free remainder fallback), the secondary verified all eight
settled decisions and the required-test list line by line. Tracker
debt d1 stays cold: the derivation builds its own in-degree/successor
maps and never calls `tasks_of_epic`. The primary noted four
pre-existing intra-doc-link warnings in `drafts.rs` — untouched by
this diff, recorded as tracker f10.

**Later the same session — t9: store codec + golden harness.** On the
user's go, t9 was dispatched the same way: the orchestrator decisions
settled into `tmp/t9-spec.md` first (dep wiring, the `StoreError`
shape with `corrupt_store` as the only t9 variant, codec naming, the
`testcodec` verdict — kept, because §2 forbids a domain→store
dependency and the domain's 108 tests use it as their byte-level
instrument — golden fixture coverage, the integration-test-crate lint
exemption mechanism), then a short spawn prompt, one implementer
landing. No P-LIVE-001 recurrence. Outcome: `src/{codec,error}.rs` +
`FORMAT_VERSION`, the shared `assert_golden` harness, the first golden
(`serialize_nodes.v1`, five JSONL lines, 1671 bytes), 11 store tests;
`dacode-json` + `zmij` enter the lock (11 → 13 packages); the domain's
108 pass unmodified; `tempfile` deferred to t10 (the first task that
needs temp data roots). Two facts established for t10: the engine
rejects non-whitespace trailing bytes at the stray offset but parses a
trailing LF cleanly — the JSONL line splitter can rely on it, pinned
by test and doc — and libtest's post-`--` filter is a test-*name*
filter, so every store test file needs a `mod` wrapper matching its
verify line (f11). Dual review approved in one round with three low
nits: a doc-comment newline inside a code span renders a path with a
space in it (f12, fix rides t10 — it re-reads these files anyway); an
interrupted `UPDATE_GOLDEN` run can leave a hidden `.tmp` debris file
between write and rename, golden integrity holds (f13, accepted); and
the git-diff half of the regeneration ritual is only meaningful once
the golden is tracked — re-run after the commit, which it was.

**Handover.** Workspace: `main` at this commit, clean; `todo-domain`
carries the §3 value types + the §5 graph and order (108 tests);
`todo-store` carries the canonical codec, `StoreError` and the golden
harness (11 tests, one golden); mcp/cli are documented M0 shells;
rustc 1.98.1, MSRV 1.95. Dependencies: serde (domain), `dacode-json`
+ `zmij` (store) — `dacode-json` is a path dep on the sibling checkout
`../rust/json-tiers`, so a cold machine needs that repo beside this
one's parent; nothing else to install. Serena's semantic layer is
live. Landed this session: t8 (`6cc4691`, backfill `4c32815`) and t9
(this commit, backfill follows). Next step: **dispatch t10** — the
JSONL event log + `Store` trait (§4.1/§4.2/§4.3): versioned header
line carrying `FORMAT_VERSION`, single-write `O_APPEND` appends +
fsync, replay into the in-memory graph through t7's explicit-id insert
path (counters only ever raised), named corruption errors (torn last
line → `corrupt_store` with offset; interior garbage unconditionally),
and f8's validation (replay must reject, or prove impossible, an
explicit-id insert of a number burned by an epic cascade). t10's
manifest moves: serde gains the `derive` feature for the event
structs, `tempfile` enters as dev-dep (both approved, U8). Its spec
inherits f11 (the `mod` wrapper for the `-- log` filter), f12 (fix
the doc-span newline while in these files) and t9's trailing-bytes
contract; f5 (counters raise-to-max entry point) and f6 (repo_set_*
audit instant rides the store event) land in t10/t12's scope. Then
t11–t13 in tracker order, then g5's M1 exit gate. Blocked on you:
nothing. Open, non-blocking: U7 (lint wording), U9 (M3 wire code), U2
(pre-M3), U3 (pre-M4); findings f4–f13 and debt d1 carry verify lines
in the tracker. Unverified: MCP smoke (no wire until M3); per-crate
lint tripwires for mcp/cli; real crash behavior — fsync/torn-window
semantics will be tested with hand-torn fixtures, not an actual
crash (t10's verify line).

### 2026-09-27 — t7: the task graph

Dispatch note first: P-LIVE-001 recurred in a new shape — two
implementer spawns died instantly with "Connection reset by server"
*despite* the proven protocol (spec in `tmp/t7-spec.md`, a few-line
spawn prompt), a control explore spawn succeeded in between, and the
third implementer attempt landed. The correlation is not purely prompt
size; retry-with-control remains the method.

The architectural decisions were settled by the orchestrator before
dispatch and written into the task spec so t8/t10 do not relitigate
them: **single-repo graph** (D4's per-repo stores, §8's per-repo lock,
§5.2's repo-scoped order — multi-repo views stay server-side);
**`parent` stays a field** (§3's signed node shapes; the graph enforces
the tree, no edge table); **deterministic cycle paths** (`[s, t, …, s]`
via sorted-successor DFS); **counters never reused**, with an
explicit-id insert path for t10's replay that only raises them; **no
serde on graph types** — the persisted shape is the event log's and the
snapshot's call (t10/t12).

Outcome: `graph/{state,edges,drafts,nodes}` + a tests module, 47 graph
tests (cycle paths asserted whole, orphan and cascade rules, allocation
gaps, a seeded mutation sequence checked after every step by an
independent whole-graph invariant scan), 94 crate tests total; all four
gates re-run green by the orchestrator. Dual review approved in one
round with four low findings, dispositioned rather than silently
dropped: `update_*` patches can rewrite `created` (tracker f7 — M3's
`task_update` schema must not expose it); an explicit-id insert can
revive a number burned by an epic cascade (f8 — t10's replay validation
owns it, and t10's tracker note now carries it); the seeded scan checks
foreign id slugs for epics/tasks but not decisions/docs (f9); the
counter-exhaustion → `duplicate` mapping (f4, documented, unreachable
at §4.1's scale). Serena's rust-LS was confirmed still dark (f3): the
daemon predates the config fix and needs a restart — reviewers used the
lexical and read layers, which was sufficient. Next: **t8, canonical
order (§5.2)** — topology, then epic registration order, then task id.

**Later the same session — Serena restored, t8 spec pre-written.** The
user restarted opencode; `get_symbols_overview` and `find_symbol` now
resolve Rust symbols (`graph/state.rs` probed), so f3 is closed — the
`f4315c5` config fix was correct and the stale daemon was the whole
cause. The t8 task spec was assembled while the t7 API was fresh:
`tmp/t8-spec.md` (gitignored scratch — if it is gone, the orchestrator
decisions below are its skeleton). Settled there: the canonical order
covers **tasks only** over `TaskToTask` edges (decision gating feeds
M2's actionability, `relates` never orders — §5.1); best-first Kahn
keyed `(epic number, task number)`, which *is* the D6 tiebreaker;
status-independent; epic scope = induced subgraph; total function
returning `Vec<TaskId>` (`Result` only for an unknown epic);
determinism proven by insertion-order-robust byte identity plus an
independent minimality recomputation in seeded tests. Selection
(`next_task`/`next_items`) stays out — it is M2.

**Handover.** Workspace: `main` at this commit, clean; `todo-domain`
carries §3 value types (t6 + t14) and the §5.1 task graph (t7, `7ddfd79`,
94 tests); store/mcp/cli are documented M0 shells; rustc 1.98.1, MSRV
1.95; serde the only external dependency — nothing to install. Serena's
semantic layer is live again (f3 closed). Landed this session: t7 +
backfill (`7ddfd79`, `2294771`), the P-LIVE-001 recurrence evidence,
tracker findings f7–f9 (t10's note carries f8). Next step: **dispatch
t8** — the spec is already at `tmp/t8-spec.md`, so the whole dispatch is
a short spawn prompt pointing at it (P-LIVE-001: on "Connection reset
by server", fire a control explore spawn to isolate, then retry the
identical prompt — both incidents succeeded on attempt 3), then dual
review, gates, commit, board move. Why t8 before t9: it completes §5's
domain layer while the t7 API is fresh, and it is small; the
alternative — starting t9's golden harness in parallel in a different
crate — trades the proven single-implementer loop for concurrency the
task does not need. After t8: t9–t13 in tracker order (t9 pins Doc's
references position and the 30-byte timestamp spelling in the goldens;
t10 carries f8 and §4.3; d1's `tasks_of_epic` scan may go hot under t8
— its report must say). Then g5's M1 exit gate. Blocked on you:
nothing. Open, non-blocking: U7 (lint wording), U9 (M3 wire code), U2
(pre-M3), U3 (pre-M4); findings f7–f9 and debt d1 carry verify lines in
the tracker. Unverified: MCP smoke (no wire until M3); per-crate lint
tripwires for store/mcp/cli (domain's proven at M0, the rest rest on
manifest reads + green clippy); t8's byte-identity and minimality tests
do not exist yet — they are t8's verify line, not this session's claim.

### 2026-09-26 — M1 decomposition

M1 decomposed into the session tracker: three domain commits (core
value types §3; task graph with write-time cycle rejection and
no-orphans §5.1; canonical order with the D6 tiebreaker §5.2), five
store/cli commits (canonical serialization + the golden harness; JSONL
event log + `Store` trait §4.2; D4 layout + registry + advisory
locking §8; snapshots + compaction §4.1; a minimal `todo-cli`
inspector), and the M1 exit gate — each with its `verify:` method
decided at creation. Two facts established while planning: std
`File::try_lock` compiles and locks on this host's rustc 1.98.1
(measured — no locking crate needed, at the cost of a `rust-version`
1.85→1.89 bump), and pre-sign-off wording had gone stale in three
places (AGENTS.md's project-overview paragraph, the domain skill's
verification section, build-and-test.md's Pre-M0 section) — fixed in
this commit.

**U4 answered**: the golden-file scaffolding shape is now
`.agents/knowledge/workflows/golden-files.md` (fixture layout + `v<N>`
naming, one shared `assert_golden` helper, the `UPDATE_GOLDEN=1`
regeneration ritual, twice-run/round-trip determinism tests); the
harness lands as the first store commit.

**U5 proposal — awaits the user's ok; lands in spec §4 on approval.**
Crash and recovery for the JSONL log: (1) every event is appended as
one `write()` of a complete line under `O_APPEND`, fsync per append —
the torn window is one event at most; (2) on replay, a malformed
*last* line (no trailing newline, or unparseable) is a named
`corrupt_store` error carrying the byte offset — never silently
skipped, per §4's "not silent truncation"; a malformed line anywhere
else in the log is `corrupt_store` unconditionally; (3) recovery is
explicit: a `todo-cli` repair command shows the torn line and
truncates it on confirmation, losing exactly that one in-flight event;
(4) snapshots are the recovery unit — written temp+fsync+atomic-rename
so they are never torn — and replay is latest snapshot + log tail;
(5) the registry file follows the same atomic-rename discipline;
(6) backup in v1 is copying a repo's store directory while no writer
holds the lock — no separate backup mechanism.

Two asks ride this decomposition and gate the store commits only (the
domain commits need neither): U8 (first external dependencies) and the
U5 ok above.

**Later the same day — U5 approved.** The user ok'd the proposal as
drafted: it lands as spec §4.3 "Crash and recovery" with the D1 banner
amended in `decisions.md`; U5 is struck below. The same message offered
the user's own JSON library for U8 (`dacode-json` at
`/Users/mouse/RPM/dacode/rust/json-tiers`) and granted modifications to
it if M1 needs one (proposed as a diff; that repo's commits stay with
its owner). A probe crate (`tmp/dj-probe`, scratch) measured it against
the store's use case: serialization byte-identical to serde_json,
twice-run identical, round-trip equal, and torn JSONL lines produce
named errors carrying byte offsets — the exact shape §4.3's
`corrupt_store` needs. U8 was revised accordingly (serde + dacode-json
+ tempfile dev, `rust-version` → 1.95) and explained to the user in
plain terms; it remains the single gate on the store commits. No redb
decision was opened — plain files stand per D1.

**U8 approved.** The user ok'd the revised ask as recommended: serde
with derives on `todo-domain` types — §2's rules clarified so *no
workspace deps* reads as no sibling path deps, the serializer confined
to the store — `dacode-json` consumed by path, `tempfile` dev-only,
`rust-version` 1.85 → 1.95. U4/U5/U8 all resolved the same day, the M1
tree is fully unblocked; dispatch started at t6 (domain value types).
The promised Serena index check instead surfaced a python-only
language-server configuration — fixed in `f4315c5`, activation to be
verified next session. The `dacode-json` modification offer stands
(diffs proposed, its repo's commits stay with its owner); the probe
needed none.

**t6 landed — M1's first code.** Two implementer spawns died with
"Connection reset by server"; the third succeeded with the task spec
moved to a scratch file and a short spawn prompt (recorded as
P-LIVE-001). Dual review: the primary requested changes — `Doc` lacked
the `references` field §3's own References paragraph and the D10
record grant (a spec self-contradiction, resolved by erratum toward
the signed record), and `Reference` deserialization skipped validation
— while the secondary approved with overlapping minors (a false MSRV
allow-reason; both reviewers independently proved `is_multiple_of`
stable since 1.87). The fix round applied all six items plus the
timestamp canonical subset (no fractions, no offsets — now a §3
sentence) and synthetic fixture paths; both reviewers re-approved the
fixed diff. U9 (wire code for invalid input at M3) opened from the
ValueError judgment call.

**t14 — fractional seconds (user decision).** Of the two §3 errata
surfaced at the t6 checkpoint, the user confirmed Doc.references
stands and directed that timestamps accept fractional seconds. §8's
byte-stability forces the normalization: the canonical form is now
`YYYY-MM-DDTHH:MM:SS.sssssssssZ` — fraction always present, fixed
nine digits, inputs with 0–9 fraction digits zero-padded, longer
rejected (never rounded), leap `:60` composing with a fraction, so
one instant keeps exactly one spelling and fixed-width Ord stays
chronological. Deserialize shares the constructor's normalization, so
the store can never persist a non-canonical spelling. §3's timestamp
sentence amended; 46 tests; both reviewers approved in one round, the
primary tracing every parse boundary and the ordering invariant. The
user's decision is recorded here — the traceability ledger the primary
reviewer's commit condition asked for; no D-entry, since this refines
§3's timestamp sentence rather than reopening a signed decision.

**Handover.** Workspace: `main` at `73849bf` plus a hash-backfill
commit, clean; `todo-domain` carries the §3 value types (t6 + t14),
the other three crates are still documented M0 shells; rustc 1.98.1,
MSRV 1.95; serde is the only external dependency in the tree — t9
adds the `dacode-json` path dep and `tempfile` dev-dep, both already
approved under U8. Landed this session: the M1 decomposition + golden
scaffolding (`5ed9b6e`), U5 → spec §4.3 (`0caeff2`), the U8 approval
+ §2 serde rule (`7aa49cc`), the Serena rust-LS enablement
(`f4315c5`), t6 (`730fc7a`, backfill `65491a2`), t14 (this commit).
Next step: **t7 — the task graph (§5.1)**: parent tree, `blocks` DAG
with write-time cycle rejection, no-orphans + epic-delete rules,
decision gating edges, per-repo id-allocation counters (never reused),
seeded mutation-sequence tests; then t8 (canonical order §5.2), then
the store commits t9–t13 (golden harness first per
`workflows/golden-files.md`; §4.3 corruption semantics in t10; t9's
tracker note carries two review nits — pin Doc's references position
and the 30-byte timestamp spelling in the goldens). Dispatch protocol
proven this session: task spec in `tmp/tN-spec.md`, short spawn prompt
(P-LIVE-001). Blocked on you: nothing — U4/U5/U8 resolved; open and
non-blocking: U7 (lint wording), U9 (M3 invalid-input wire code), U2
(pre-M3), U3 (pre-M4). Unverified: Serena rust-LS activation (config
fixed, no session has run under it yet — start with
`get_symbols_overview` on `crates/todo-domain/src/node.rs`); MCP smoke
(no wire until M3); per-crate lint tripwires for store/mcp/cli.

### 2026-09-25 — D10 concretization, JSON confirmed, M0 scaffolded

Three moves this session. (1) The user ok'd the D10 concretization and
confirmed canonical JSON as the `graph_show` wire format: id prefixes
(U/E/D/S), decision transitions enforced with `missing_resolution`,
audited `doc_set_path`, `#anchor` citations — landed in §3/§6/§6.1
(commit `bdfcd65`, U6 struck). (2) M0 scaffold dispatched to the
implementer and dual-reviewed: four crates, the workspace lint regime
inherited per crate, zero external deps, all four code gates green for
the first time. The primary reviewer proved the lint regime with a
fully reverted tripwire (unwrap + missing_docs both denied; test-cfg
code exempt by mechanism) and flagged one medium finding — the board
had not moved with the delivery; this entry and the M0 row are that
move, in the same commit as the scaffold. The secondary verified the
file set, manifest inheritance and every spec citation in the crate
docs. First spawn of all three worker roles: the agent-rules model
bindings work. (3) `_common.md`'s pre-M0 gate clause was made
conditional so it cannot go stale (agents regenerated, config version
4). The re-review pass caught two docs-side defects in this very move
— an untrue session-log claim (an earlier draft of item 3 asserted a
spec §10/AGENTS.md wording alignment that did not happen; rewritten to
fact) and a stale knowledge INDEX (AGENTS.md edited without
regenerating) — both fixed before the commit. The wording question
itself is recorded as U7 rather than fixed silently: spec §10 is
signed text and moves only by user decision.

**Handover.** Workspace: exists — four empty documented crates, all
gates green. Landed: `bdfcd65` (D10 concretization), the M0 scaffold
commit (`57b531c`), this board move. Next step: M1 —
decompose the domain model (five node kinds, D9 statuses, edges and
invariants per §3/§5) and the store (`Store` trait, JSONL event log,
per-repo layout, golden files per §4/§8/§10); U4 (golden-file
scaffolding shape) and U5 (torn-line/corruption story) feed it and
should be settled inside the M1 decomposition, not before it. Blocked
on you: nothing — M1 decomposition starts on your go. Unverified: MCP
smoke (no wire until M3); per-crate lint tripwires for the other three
crates (domain's was proven, the rest rest on manifest reads + green
clippy); rust-analyzer/Serena index now buildable — the first M1 task
should confirm it.

### 2026-09-25 — D9–D11 signed; §12 closed

The user voted all three remaining decisions the same day. **D9**:
option (a) with a refinement — a deferred item is "mostly blocked by
some other tasks", so until-targets ride `blocks` edges
(cycle-checked, queryable) and the optional `until` date stays pure
metadata; nothing reactivates by calendar or edge completion, queries
surface until-satisfied items, reopening is an explicit update.
**D10**: option (b) as drafted — references lists, decision nodes
whose open state blocks work, registered docs with prose staying in
git; extends signed D2's node kinds and rewrites §11's decision-culture
non-goal, both recorded in its banner. **D11**: option (a) plus an
amendment — an optional `priority` field set by agents on
task_add/task_update, echoed in payloads, never a query parameter and
never an ordering input; D6's tiebreaker stands untouched. All three
work orders executed: §12 items 9–11 deleted and the section closed,
§3/§5/§6/§9/§11 rewritten normative, board and harness docs updated.

Right after the sign-off the user amended two of them (same day, same
authority). **D9**: deferred items get surfaced, not just queried —
next_task/next_items carry `deferred: {total, ready}` for the scope,
task_list gains a ready filter; still no auto-reactivation. **D11**:
next_items also takes a `task` scope (a non-top-level item's
transitive dependents, canonical order), and `graph_show` became the
complete-graph handover function, specified in the new §6.2 —
canonical-order nodes with per-node blocked_by/blocks adjacency, JSON
on the wire, any YAML/Mermaid render a pure function of that
structure. The user also asked for the D10 concretization proposal;
it was delivered in the sign-off thread (id prefixes, decision
transition enforcement, doc path auditing) and lands on their ok
(U6).

**Handover.** Workspace: none yet — M0 is fully unblocked (§12
closed) and is the only thing standing between here and M1. Landed:
the sign-off commit and the D9/D11 amendment commit (docs only). Next
step: scaffold the M0 workspace — empty crates, workspace lints, the
gate order green — on the user's go. Blocked on you: the go for M0,
and an ok (or edits) on the D10 concretization (U6). Unverified: as
before — no agent spawn-tests, no serena/ast-grep smoke until the
workspace exists.

### 2026-09-25 — D1–D8 signed; D9–D11 drafted

The user signed all eight decisions in one message, with amendments.
D4 resolved to (c) against the recommendation: per-repo store
directories under a shared registry — "not one common database shared
across many projects". D8 resolved to (a) plus a per-instance default
repo slug passed through the client's per-project mcp entry (explicit
configuration; cwd detection still deferred to M5). D3's repo-key
sub-choice settled on immutable slugs, confirmed by the user's own
"repo/project slug" phrasing. All eight work orders executed: §12
items 1–8 deleted, PROPOSED banners struck across §2–§9, the JSONL
engine normative in §4.1, the D6 tiebreaker normative in §5.2,
per-repo lock scope in §8, camera-first importer in §9, default-slug
wiring in §6/§13.

The same message added three requirement clusters, and a follow-up
message directed that specs and design decisions should live in the
server so they can be reliably referenced. Drafted as: **D9** (status
vocabulary open/in_work/blocked/deferred/closed — reconciled with
keeping `wontfix`; until-dates as metadata so selection stays
wall-clock-free), **D10** (layered: references list for externals;
`decision` nodes with full dossiers that can `block` work while open;
`doc` registration for stable addresses with prose bodies staying in
git — a full content store is offered as option (c), recommended
against for v1) and **D11** (`next_items` batch with blocked-by/blocks
annotations and derived, display-only weight; a hand-set priority would
amend signed D6 and is flagged as a separate choice). Also this
session: committed the bootstrap (`e6d4761`) and the tracker-keying
note (`5062862`).

**Handover.** Workspace: none yet — the M0 skeleton is unblocked and
authorized (D1–D3 signed) but not started. Landed: this sign-off
commit (docs only). Next step: the user votes D9–D11 (rendered by
`tools/attention.py`); the M0 scaffold may proceed in parallel on the
user's go — empty crates plus the lint regime presume nothing D9–D11
could invalidate. Blocked on you: D9, D10, D11; plus one D4
interpretation to veto if wrong — epics/tasks stay nodes inside their
repo's store, not per-epic files. Unverified: as before — no agent
spawn-tests yet, no serena/ast-grep smoke until a workspace exists.

### 2026-09-25 — bootstrap

Repo had a git init and nothing else. Adopted the camera harness
(the newest of the three sibling setups): `agent-rules.yaml` as the
single source of truth, five agents generated by `tools/agents-sync.py`
(orchestrator, implementer, cross-family dual reviewers, read-only
explore), `/todo-next-imp` + `/todo-close` commands, three skills, the
knowledge/pitfalls structure, and the four doc tools ported with this
repo's paths. Models bound cross-family (qwen heavy / glm mid / flash
light) per the dacode convention. Wrote the seed spec
(`docs/todo-mcp-design.md`) around the user's stated constraints —
deterministic, non-markdown, directed graph, local non-relational store,
repo+epic selection, mutable repo paths — with the engine choice and
seven other questions left open as §12/D1–D8 rather than pre-empted.

**Handover.** Workspace: none (pre-M0; no `Cargo.toml`, every code gate
skipped by construction). Landed: the whole harness + docs, uncommitted
on `main` with no prior commits. Next step: the user votes D1–D3 — they
gate M0 and everything after; U1's measurement is the cheapest way to
de-risk D1 first if wanted. Blocked on you: all eight D-items
(`tools/attention.py` renders them). Unverified: the tools have been run
only against this bootstrap's own docs; agents-sync output has not been
spawn-tested (first spawn of each role will confirm the
`reasoning_values` rows, per the table's own caveat); no serena/ast-grep
smoke until a workspace exists.
