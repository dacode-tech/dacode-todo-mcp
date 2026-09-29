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
§5.2) have landed — the domain layer of M1 is complete — followed by
t9 (canonical serialization + golden harness), t10 (the JSONL event
log + `Store` trait, §4), t11 (the D4 layout + registry + advisory
locking, §2/§7/§8) and t12 (snapshots + compaction, §4.1/§4.3) — four of
the five store/cli commits are in, and `Store` now carries all five of
§4.2's responsibilities; next is t13, the `todo-cli` inspector, the last
M1 code task before the g5 exit gate.

| Blocking | What | Action |
|---|---|---|
| U10 | t13's `verify`/`repair` semantics, and f14's log-header marker (a golden-moving format change, cheap only while no store exists outside tests) | decide: drop as landed / archive sibling / persistent header marker |

(U7, U12 and U2/U3 remain open but non-blocking; U5/U8/U9/U11 are
resolved. t15 is the one M1 task gated on nothing.)

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
- 2026-09-27 — **M1 t10: JSONL event log + `Store` trait (§4)**: the
    signed D1 engine. Versioned header (`{"format_version":1}` reading
    `FORMAT_VERSION`); 17 event kinds mirroring the domain mutation
    surface 1:1 as flat internally-tagged JSONL lines (kind first,
    caller-passed `ts` on every event — the repo audit instant §3's
    `Repo` has no field for, closing f6); full-node payloads on adds,
    store-owned patch DTOs on updates — **zero `todo-domain` changes**,
    as `drafts.rs`'s module doc mandates. Patch DTOs carry no
    `id`/`repo`/`created` and deny unknown fields, so node history is
    unreachable through the persisted surface (f7 closed at the store
    boundary), with RFC 7386 three-state semantics (absent untouched,
    `null` clears, value replaces) via a double-option deserializer.
    Replay: `repo_register` seeds the graph; an add whose id is not
    the next sequential number is `corrupt_store` — a number burned by
    an epic cascade can never be revived from a log (f8 closed
    store-side); any malformed line is `corrupt_store` with the
    absolute line-start offset for t13's repair; a final line without
    its LF is torn even when parseable (§4.3); a wrong header version
    is `version_mismatch`. Append = one `write_all` of the complete
    line under `O_APPEND` + `sync_all`; domain rejections write
    nothing; io failure after apply is documented session-fatal
    (reopen replays the disk truth). `trait Store` is the §4.2 surface
    (graph + version + 16 mutations returning the new state;
    open/create join at t11, snapshot/compact at t12); `StoreError`
    grows by exactly `VersionMismatch`, `Io` (detail string only —
    `"io"` documented as not a §6.1 code, pending U9) and
    `Domain(DomainError)`. Golden `event_log.v1` (35 lines) exercises
    every kind with cascade id gaps and mixed-precision timestamps;
    the t9 golden regenerates byte-identical; f12's doc-span fix rode
    along. 33 store tests (19 new); the domain's 108 unmodified;
    Cargo.lock 13 → 26 (`tempfile` dev tree). Dual-reviewed in one
     round, four low findings dispositioned in §6 and the tracker.
     Landed as `df79982`.
- 2026-09-28 — **M1 t11: D4 layout + registry + advisory locking
     (§2/§7/§8)**: the store now lives under an injected `DataRoot` —
     `repos/<slug>/event-log.jsonl` plus one lock file per protected file
     (`<file>.lock`, the single rule that keeps two logs in one directory
     from sharing a lock) — and §4.2's open/create half joined the `Store`
     trait as `where Self: Sized` constructors, which is how the tension
     t10 reported resolves: the trait stays the seam D1's redb fallback
     would plug into, and `dyn Store` still compiles behind a
     compile-time pin. Every store is locked, always — `JsonlStore` owns
     a `FileLock` over std `File::try_lock`, measured on this host
     (`TryLockError{WouldBlock, Error}`, two fds in one process conflict,
     `Drop` releases, and a lock follows the **inode**, so a rename
     defeats it: hence dedicated lock files, and no atomically-rewritten
     file is ever its own lock target). §6.1's `locked` arrives as
     `StoreError::Locked { target: LockTarget }` with a path-free
     `Display`. Lock order is total — the registry first, then sorted
     slugs — and is safe *because* the registry stores only immutable
     fields, so nothing ever wants the registry lock while holding a
     store lock. That is the one narrow reading of signed §2 text here,
     and it is flagged for veto: the registry holds
     `{format_version, repos: [{registered_at, slug}]}` sorted by slug,
     §7's mutable `name`/`path` stay solely on the `Repo` node (indexing
     them would need two writes per `repo_set_path` and would break the
     lock-order argument), and the slug→store mapping is derived rather
     than stored. The cost is recorded, not hidden: a cold `repo_list`
     must open stores, which M3 answers from handles it already holds —
     extending the registry later is a golden-visible format change,
     cheap before M3 and expensive after. `register` writes store bytes
     before the index, so the only crash state is a store directory the
     registry does not list (`register` then refuses `duplicate`; t13's
     inspector resolves it), never a registry entry with no store;
     `open_many` dedupes, sorts and is all-or-nothing, its rollback
     pinned by test rather than asserted. The registry document gets its
     own golden (`registry.v1`, 238 bytes) written through the real
     `register` path under §4.3's temp+fsync+atomic-rename discipline,
     with `deny_unknown_fields` and a version probe ordered before the
     strict parse; the two existing goldens are byte-identical. 55 store
     tests (22 new: 19 in the `layout` target, 3 lib unit tests), 163
     workspace-wide; the domain's 108 unmodified; `Cargo.lock` unchanged
     — no new dependency. Dual-reviewed over four rounds: eleven fix
     items in round one (three minors from the primary, two from the
     secondary, five lows and three lows — among them a
     self-contradictory recovery paragraph, an untested load-time sort
     whose removal left every test green while `contains` binary-searched
     an unsorted vector, and an error path that masked `PermissionDenied`
     as Decision 11's `NotFound`), one stale test doc in round two, and
     in round three a false clause in the orchestrator's own replacement
     text, which the primary disproved after the secondary had ratified
     it. f15's two test renames rode along; the verify line moved off
     name filters onto `cargo test -p todo-store --test layout`
     (P-LIVE-005). Landed as `5b72b76`.
- 2026-09-28 — **M1 t12: snapshots + compaction (§4.1/§4.3)**: the
     second half of the signed D1 engine, and the last of §4.2's five
     responsibilities. A store's snapshot is `<log>.snapshot` — t11's
     suffix rule extended, so two logs in one directory cannot share one
     — a whole-file JSON document (`{format_version, log_offset, repo,
     counters, epics, tasks, decisions, docs, blocks, relates}`, one
     trailing LF, `deny_unknown_fields`, a version probe ordered before
     the strict parse) written temp+`fsync`+atomic-rename per §4.3, and
     it gets **no lock of its own**: an atomically rewritten file is never
     its own lock target, so the store's existing `.lock` covers it.
     Replay reads the latest snapshot, validates it against the log
     (`log_offset` within the log, at or after the seed's end, exactly on
     a line boundary, and the snapshot's repo **slug** — only the slug,
     never the whole node, because the seed holds the repo as registered
     while §7's `repo_set_path` mutates it) and then applies the tail.
     Restore is `insert_*` per node (no sequential-id check: a snapshot
     legitimately holds cascade gaps), then edges, then the counters —
     which needed f5's domain entry point, `Counters::from_highest` +
     `Graph::raise_counters_to`, monotonic so a restore can never re-issue
     a burned number (§3), plus a check that no live id exceeds its kind's
     counter. `compact()` snapshots, truncates the log to
     `header + re-emitted seed`, re-bases the snapshot's offset and
     reopens the append handle — the reopen is mandatory, because t11
     established that the lock protects the `.lock` inode while the handle
     points at the *log* inode, so a rename orphans it. **The step order
     is the crash-safety argument and it was wrong first time**: the
     initial implementation staged both temps and renamed log-then-
     snapshot, which the primary reviewer proved loses events *silently*
     in two crash-reachable corners (a first-ever compaction leaves
     `header + seed` with no snapshot at all, which opens clean as a bare
     repo graph; a repeat compaction can leave a prior `log_offset`
     exactly equal to the new length, so every check passes and the reopen
     yields the previous compaction's state) — both violations of §4's
     "never silent truncation". Amended to snapshot-into-place first, then
     truncate, then re-base: every crash point is now either a correct
     open or a named `corrupt_store`, and the one window is recoverable
     because the live covering snapshot holds the whole graph. Two tests
     pin both corners from real bytes and prove the re-base repair
     recovers everything. The residual silent case is **not**
     crash-reachable and is documented as such: deleting a compacted
     store's snapshot out of band leaves `header + seed`, which is
     indistinguishable from a fresh store — closing it needs a persistent
     compaction marker in the log header, i.e. f14's decision and a
     golden-moving format change. 86 store tests (31 new), 111 domain
     (108 unmodified + 3), 197 workspace-wide; `Cargo.lock` unchanged;
     the three existing goldens byte-identical and `snapshot.v1` new
     (2025 bytes, four *distinct* counter values so a `from_highest`
     transposition cannot pass, with a number burned above the highest
     live id). Dual-reviewed over three rounds; the second round's split
     verdict — secondary `ok`, primary `major` on the same diff — is the
     clearest evidence yet that the cross-family pairing is load-bearing,
     and the secondary's own post-mortem (it instance-checked the spec's
     crash argument instead of falsifying it against the state space) is
     recorded in §6. Two engine/format follow-ups opened: U10 (does
     compaction archive the events it drops? §7's audit-trail sentence vs
     §4.1's truncation) and U11 (`dacode-json` cannot deserialize serde
     tuples — P-LIVE-006, worked around without moving a wire byte).
     Landed as `a22fced`.
- 2026-09-29 — **U11: `dacode-json`'s deserializer fixed upstream** (in
     the sibling checkout, not this repo — landed there as `c4aebe9`).
     The cause was not tuples: `serde::de::SeqAccess` has no `end()`, and
     serde_json splits container framing so that `next_element_seed`
     *peeks* `]` while `Deserializer::end_seq` eats it after `visit_seq`
     returns. Both byte-cursor tiers here had that inverted, so a
     fixed-length visitor — tuple, tuple struct, tuple enum variant,
     `[T; N]`, the only serde shapes that stop reading a container early —
     handed control back with the closer pending and the *enclosing*
     container consumed it as its own. The same fix closed two defects
     worse than the reported one: a **truncated array parsed clean**
     (`[1,2` → `Ok((1, 2))`, the exact property §4.3's torn-tail
     detection rests on), and `stream` rejected **every enum object
     variant** because `Payload::close` advanced the structural index
     without moving `vend`, the offset its trailing-whitespace check is
     measured from. Nothing in that crate's suite caught any of it: its
     serde_json differential ran the pool tier (unaffected — it
     pre-parses) and `JValue` only, which is a collection all the way
     down. `tests/serde_fixed_len.rs` is the missing column and fails
     8 of 12 on the parent commit. Verified there: fmt, clippy, 22 test
     binaries with zero failures including the 300+-file corpus
     differential, the full `check-features.sh` matrix down to
     `thumbv7em-none-eabihf` with no allocator, and `size.sh` on both
     trees (`direct` +488 B linked, +1.1 %); throughput **not** measured.
     Verified here: 197 tests pass, tree clean, all four goldens
     byte-identical — no byte moved, because t12 kept the canonical wire
     shape and narrowed only the read side. Deliberately **not** done:
     committing the checkout's pre-existing 53-file uncommitted `cargo
     fmt` pass (that repo has no fmt gate, so whether it belongs in
     history is the owner's call), and the pool tier's silent surplus
     truncation. Residual choices are U12.
- 2026-09-29 — **U9 resolved: §6.1 gains `invalid` and `io`** (spec
     amended, user-signed). Two codes rather than one because a client
     retries `io` and must not retry `invalid`; conflating them would cost
     exactly the distinction a versioned code list exists to give. `io`
     blesses the spelling `todo-store` already returns from
     `StoreError::code()`, so it is a contract change with no code change;
     `invalid` is M3's wire mapping for the domain's `ValueError`, which
     stays codeless in the crate that raises it — the list being versioned
     is why the domain could not add it unilaterally in the first place,
     and `DomainError` remains total over the original eight. Per-case
     mapping onto existing codes was considered and rejected: none of the
     eleven describes a malformed slug. Two doc comments now contradict
     §6.1 and are f22, split by crate boundary — the `ValueError` module
     doc rides t15, the `StoreError::Io` one rides t13.

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
- [x] **U9 · Decide the wire code for invalid input (M3).** t6's error
      model keeps `DomainError` total over exactly the eight §6.1 codes
      and puts malformed-input failures (bad slug/id/timestamp/date/
      reference) in a companion `ValueError` with no code — none of the
      eight fits, and §6.1's list is versioned, so the domain may not
      extend it unilaterally. At M3 the wire layer needs the decision:
      a new §6.1 code (e.g. `invalid`) or a per-case mapping rule.
      Surfaced by the t6 dual review; record the choice in §6.1 before
      the MCP surface lands. **Resolved 2026-09-29** (user took the
      recommended option): §6.1 gains **two** codes, `invalid` and `io`,
      rather than one — a client retries `io` and must not retry
      `invalid`, so conflating them would cost the distinction the list
      exists to give. `io` blesses the spelling `todo-store` already
      returns from `StoreError::code()`, so it needs no code change;
      `invalid` is M3's mapping for the domain's `ValueError`, which
      stays codeless in the crate that raises it (`DomainError` remains
      total over the original eight). Per-case mapping onto existing codes
      was rejected: none of the eleven fits a malformed slug. Two stale
      doc comments now contradict §6.1 and are tracked as **f22** — the
      `ValueError` module doc rides t15, the `StoreError::Io` one rides
      t13.
- [ ] **U10 · Decide whether compaction archives the events it drops.**
      §4.1 signs "periodic snapshot + truncation bounds replay"; §7 signs
      "the history of a repo's locations is part of the audit trail"; and
      §4 makes human auditability a strong preference because "the store
      holds the audit trail the markdown boards held". t12 implements the
      first and reconciles the other two as: §7 guarantees a change is
      *recorded as a mutation* rather than silently overwriting a field,
      §4.1 bounds how long the *live* log retains any event, and §4.3's
      backup (copy the directory under the lock) is the archive. That
      reading is argued in `snapshot.rs`'s module doc but it is a reading,
      and the alternative is cheap right now: (a) drop as landed, backups
      are the archive; (b) `compact()` renames the old log to an archive
      sibling (`event-log.1.jsonl`) with a bounded rotation, preserving
      the trail in the live store; (c) a persistent compaction marker in
      the log header, which also closes the one residual silent case (an
      out-of-band snapshot deletion) and rides f14 — but moves
      `event_log.v1.golden`, a called-out compatibility event. **Nothing
      in M1 calls `compact()`** — §6 lists no tool for it and t13's cli
      has not landed — so no history is destroyed by the code as shipped;
      the choice gets expensive the moment a caller exists. Surfaced by
      the t12 dual review.
- [x] **U11 · Report or fix `dacode-json`'s tuple deserialization.**
      The serializer this project adopted at U8 (path dependency on
      `/Users/mouse/RPM/dacode/rust/json-tiers`) serializes serde tuples
      correctly but cannot read them back: it consumes the first element
      and reports the rest as trailing characters. Reproduced twice
      independently during t12, in this tree and in a scratch crate over
      plain `Vec<(u64, u64)>` (P-LIVE-006). t12 keeps the canonical wire
      shape and narrows on the read side, so no byte moved and the
      workaround reduces to the derived impl with zero byte movement once
      the engine is fixed. U8's resolution grants modifications to that
      repo **proposed as a diff**, with its commits staying with its
      owner: decide whether to propose one, or to keep the workaround and
      a "no tuples in a persisted schema" house rule. Either way, prefer
      named-field structs over tuples in new schemas — they dodge the bug
      and read better in an editor, which is §4's auditability point.
      **Fixed upstream 2026-09-29** at `dacode-json` `c4aebe9`, under U8's
      diff-proposal path: the real cause was not tuples but the array
      closer being consumed by `SeqAccess` instead of by the
      `Deserializer`, and the same fix closed two worse defects (a
      truncated array parsing clean; `stream` rejecting every enum object
      variant). Mechanism, measurements and the residual upstream gaps are
      in `P-LIVE-006`. No byte moved here: all 197 tests pass and all four
      goldens are unchanged. The store-side narrowing is now optional
      rather than required, which is **U12**.
- [ ] **U12 · Settle the post-fix `relates` read path and the tuple rule.**
      Two small choices left behind by U11's fix, neither blocking
      anything. (a) `snapshot.rs`'s `relates_from_arrays` now duplicates
      what the derived impl does — but it is not equivalent: the derived
      impl rejects a 3-element entry as serde's `invalid length 3,
      expected a tuple of size 2`, where the helper says *"a relates entry
      is a two-element array of task ids"*, which is the better
      `corrupt_store` detail for whoever is repairing a store with t13's
      `repair`. Keep it for the diagnostic, or drop it for one less
      hand-written deserializer? Either way `snapshot.v1.golden` must not
      move, and dropping it is a store change needing dual review.
      (b) Whether "named-field structs over tuples in a persisted schema"
      becomes a house rule in `.agents/roles/_rust.md` — it is now argued
      on §4 auditability grounds rather than as an engine workaround, so
      it survives U11 either way. Best decided with t13, whose `repair`
      output is the consumer of (a).

## 5. Debt

Deliberate shortcuts, acknowledged rather than fixed. Same id rules as §4.

_No debt recorded yet — nothing has been built to take a shortcut in._

## 6. Session log

Newest first. One entry per session, written for someone who was not
here. The final entry of a session is the handover
(`.agents/skills/todo-next-imp/SKILL.md` defines the shape).

### 2026-09-29 — U11 fixed upstream; U9 resolved (§6.1 gains `invalid` and `io`)

The session opened at the t12 handover's fork — t13, t15, or the two open
board questions — and the user asked what U10 and U11 actually required
deciding. Explaining them properly meant reading the engine, and reading
the engine turned U11 from "report a bug in someone else's crate" into
"the bug is three lines from being diagnosed, go diagnose it". The user
chose the fix.

**The first diagnosis was wrong, and a compile error caught it.** I had
told the user the cause was a missing `SeqAccess::end()` override, and
wrote one. `serde::de::SeqAccess` has no `end` method — E0407, twice. The
real mechanism, read out of serde_json 1.0.151 rather than recalled:
`next_element_seed` *peeks* `]` and returns `None` without eating it, and
`Deserializer::end_seq` eats it **after** `visit_seq` returns. Framing is
the Deserializer's job, not the access adaptor's. Both of `dacode-json`'s
byte-cursor tiers had that inverted, which is why the symptom presented as
"trailing characters" instead of as a tuple problem: a fixed-length
visitor handed control back with the closer still pending and the
*enclosing* container consumed it as its own terminator. Same class as f20
— a claim stated from memory and treated as ground truth — except here the
compiler, not a reviewer, was the falsifier, and the reverted edits had to
be undone by hand because `git checkout` would have destroyed the
checkout's uncommitted work.

**Two worse defects fell out of measuring instead of reasoning.** A
scratch crate over both tiers plus serde_json as oracle, run against
pristine `HEAD` extracted with `git archive` (no `.git` mutation, no
stash), showed that `[1,2` deserialized as `Ok((1, 2))` — a **truncated
array parsed clean**, because the closer was never required. That is
precisely the property §4.3's torn-tail detection rests on, in the engine
§4.3 is written against. And `stream` rejected *every* enum object
variant, including ones with no array near them: `Payload::close` advanced
the structural index with `bump` instead of `Cursor::close`, leaving
`vend` — the offset `from_slice` measures its trailing check from —
pointing just past the payload. Neither was reachable from this project's
schema; both were found because the probe was written to falsify a
diagnosis rather than to confirm one.

Why that crate's own suite missed all three: its serde_json differential
runs the **pool** tier (`tests/serde_de.rs`), which pre-parses and so is
unaffected, and holds `direct`/`stream` to serde_json only through
`JValue` (`tests/stream.rs`) — a collection all the way down, which never
visits a fixed length. `tests/serde_fixed_len.rs` is the missing column,
and copied onto the parent commit it fails 8 of 12; the 4 that pass are
the rejection-side guards, which already rejected correctly and only
changed message. That non-vacuity run is the reason to trust the fix.

**The checkout was not mine to commit into.** 55 dirty files,
+1281/−658, which I verified is exactly `cargo fmt` on `HEAD` (reproduced
in a scratch tree; four untouched files byte-identical; green suite before
any edit). Two of them were files I needed. So the fix is committed as
`c4aebe9` with those two files' reflow staged alongside it and the mixing
disclosed in the commit body, and the **other 53 were left untouched**:
that repo has no CI, no `rustfmt.toml` and no fmt gate anywhere, so
`HEAD` has never been formatted and whether a 1 281-line reformat belongs
in its history is the owner's call, not a guest's. The commit body also
records what was measured and left alone — the pool tier's silent surplus
truncation, `MapAccess`'s latent identical inversion, `flat::typed::de`
being a binary reader rather than JSON — and that **throughput was not
measured**, only size (`direct` +488 B linked for bare-metal ARM32,
+1.1 %, attributed `dacode_json` +248 / `entry` +164 / `serde_core` +48).

Nothing changed here, which is the point of t12 having kept the canonical
wire shape and narrowed only the read side: 197 tests pass, tree clean,
all four goldens byte-identical. `P-LIVE-006` is rewritten from
"workaround" to "resolved", carrying the generalizable lesson (a serde
engine's framing contract lives in the `Deserializer`; and a differential
suite is only as wide as the shapes it exercises — `Value` exercises none
of the fixed-length ones). U11 is struck; **U12** opens for the two
residual choices, best made with t13 since its `repair` output is what
consumes the better diagnostic.

**U9 closed in the same session**, and it is a spec change rather than a
delivery: §6.1 gains `invalid` and `io`. The reason for two codes instead
of one is retry semantics — a client may retry a filesystem fault and must
not retry a malformed slug, so a single `invalid` would have destroyed the
only distinction that matters to a caller. `io` costs no code because
`StoreError::code()` already spells it that way, with a doc comment
protesting that it is *not* a §6.1 code; the amendment blesses the
spelling the store had already chosen. `invalid` is M3's mapping for
`ValueError`, which stays codeless in the crate that raises it, because
`DomainError` remaining total over the original eight is what kept the
versioned list honest in the first place. Two doc comments now contradict
§6.1 and are **f22**, split by crate boundary so each rides a commit that
is already in the right crate: the domain half with t15, the store half
with t13.

**Handover.** Workspace: `main` at `a22fced`/`7959281` plus this board
commit, clean; sibling `/Users/mouse/RPM/dacode/rust/json-tiers` at
`c4aebe9` with **53 files still dirty** (an uncommitted `cargo fmt` pass,
owner's call). Crates and test counts unchanged from the t12 handover:
`todo-domain` 111, `todo-store` 86 and four goldens, mcp/cli still
documented M0 shells; rustc 1.98.1, MSRV 1.95. A cold machine still needs
the sibling checkout beside this repo's parent, and now needs it at
`c4aebe9` or later only if U12 drops the narrowing — the wire bytes are
identical either way.

Next step: unchanged by this session — **t13**, the `todo-cli` inspector,
or **t15** first if a small independent commit is preferred (it is the
only M1 work not gated on a user decision, and g5's `cargo doc` gate needs
it). U10's answer still shapes t13's `repair`; U12 now joins it there.

Blocked on you: **U10** (does compaction archive what it drops? — and
prior to that, whether anything should call `compact()` at all before M5,
since §4.1's own scale argument says replay is already cheap at 10^3–10^5
nodes) and **U12** (keep `relates_from_arrays` for its diagnostic; adopt
the named-field-struct rule?). Non-blocking: U7, U2, U3, and t11's narrow
reading of §2's registry metadata, still open to veto. Also yours: whether
to commit the sibling's 53-file fmt pass — that repo has no fmt gate, so
committing without adding one just lets it drift — and whether to
benchmark `c4aebe9` before it goes anywhere near a release.

Unverified: **throughput of the engine fix** — no benchmark was run, and
it adds a call and a branch per array close in a crate whose pitch
includes speed. Everything else carries over from the t12 handover: no
real crash or power loss anywhere, `compact()`'s step order is not
externally observable without a fault-injection seam (f19), MCP smoke
awaits M3, per-crate lint tripwires for mcp/cli, multi-process lock
contention is two handles in one process, the `#[cfg(unix)]` permission
tests vanish rather than pass off-platform, snapshot durability without a
directory fsync is reasoned (f16) not measured, and nothing has called
`compact()` outside tests. New and smaller: the engine fix has had **no
independent review** — it was verified by measurement (non-vacuity against
`HEAD`, the full feature matrix down to a no-allocator target, the
300+-file corpus differential, size on both trees, 197 downstream tests),
which is stronger than a read-through but is not a second pair of eyes on
the `stream` tier's `vend` seeding, the subtlest hunk in the diff.

### 2026-09-28 — t12: snapshots + compaction

Dispatch was the proven one — decisions settled into `tmp/t12-spec.md`
(15 decisions, 6 judgment calls, 14 required tests), a short spawn
prompt, one implementer landing: the fifth clean dispatch in a row, no
P-LIVE-001 recurrence. t12 was the first M1 task to span two crates, and
the split held: `todo-domain` gained exactly two public items
(`Counters::from_highest`, `Graph::raise_counters_to`, +182/−0) plus
three additive tests, with the 108 pre-existing tests unmodified, and
everything else landed in `todo-store`.

**The process earned its keep, and the defect was mine.** Decision 8 of
the task spec prescribed `compact()`'s step order — stage both temps,
rename the log, rename the snapshot — with a safety argument attached: a
crash between the renames leaves an old snapshot whose `log_offset`
exceeds the new log's length, which replay reports as `corrupt_store`.
The argument silently presupposed that a prior snapshot exists and that
its offset is large. Both presuppositions are false, and the consequences
are the two things §4 forbids outright: a **first-ever** compaction
crashing in that window leaves `header + seed` with no snapshot at all,
which the no-snapshot replay branch opens cleanly as a bare repo graph —
every event silently gone; and a **repeat** compaction can leave a prior
re-based offset exactly *equal* to the new short length, so all three
offset checks pass and the reopen silently yields the previous
compaction's state. The secondary reviewer verified the code matched the
spec and that the spec's stated consequence held in the case the spec's
own reasoning instantiated — and returned `ok`. The primary re-derived
the crash state space in the next round, found both corners, and
retracted its own earlier clearance of the same area. Its post-mortem,
which the secondary then adopted verbatim as the lesson: **a spec's
safety argument is a claim to falsify against the full state space, not
ground truth to instance-check the code against.** Recorded as tracker
f20 for the g5 consolidation into `.agents/pitfalls.md`, because this is
the second M1 task whose *spec* was wrong and whose review caught it (t8
inverted the signed D6 tiebreaker inside a test description).

The fix was an amended Decision 8, not a doc qualification: `compact()`
now snapshots the covering document **into place** first (after which the
on-disk pair is consistent), then stages and renames the truncated log,
then re-bases the snapshot's offset, then reopens the append handle.
Every crash point is a correct open or a named `corrupt_store`, and the
one window is recoverable because the live covering snapshot holds the
whole graph — both corner tests prove the re-base repair restores
everything, counters included. No format change, no golden moved. What
remains silent is *not* crash-reachable and is documented as such:
deleting a compacted store's snapshot out of band leaves `header + seed`,
indistinguishable from a fresh store; closing that needs a persistent
compaction marker in the log header, which is f14's conversation and a
golden-moving one.

Two discoveries rode along. **`dacode-json` cannot deserialize serde
tuples** — it serializes `Vec<(A,B)>` correctly and then refuses to read
its own bytes, consuming the first element and reporting the rest as
trailing characters; reproduced by the implementer and independently by
the primary reviewer in a scratch crate. t12 keeps the canonical wire
shape and narrows on the read side (`Vec<Vec<TaskId>>` plus a pinned
non-pair rejection), so no byte moved and the workaround evaporates when
the engine is fixed. That repo is the user's (U8 grants modifications
proposed as a diff), so it is board U11 and P-LIVE-006, not a silent
patch. **§7's audit-trail sentence** — "the history of a repo's locations
is part of the audit trail" — is the strongest signed counter-text to
truncation, and the spec's reading did not address it; the reconciliation
now in `snapshot.rs` is that §7 guarantees *recorded as a mutation*,
§4.1 bounds live-log retention, and §4.3's backup is the archive. Whether
compaction should archive the events it drops instead is board U10 and is
cheap to decide now, because nothing in M1 calls `compact()`.

Outcome: `src/snapshot.rs` (555 lines) plus edits to `log.rs` (793),
`store.rs` (287), `layout.rs`, `lib.rs`; 31 tests in the new `snapshot`
target; the `snapshot.v1` golden (2025 bytes, four distinct counter
values so a `from_highest` transposition cannot pass, with a number
burned above the highest live id); 86 store tests, 111 domain, 197
workspace-wide; `Cargo.lock` unchanged; the three existing goldens
byte-identical; `cargo doc -p todo-store` warning-free for the first time
(the orchestrator's one-line fix to a stale intra-doc link at
`event.rs:658` — a t11 leftover nobody had looked for, because no gate
runs `cargo doc`; g5's verify line now does). f5 and f16 closed; f18
(the f16 paragraph's enumeration), f19 (`compact()`'s order is not pinned
by a test, with the technique that would pin it), f20 and d3 recorded.

**Handover.** Workspace: `main` at `a22fced` plus the hash-backfill
commit, clean; `todo-domain` carries §3's value types, §5's graph and order, and
the f5 counters entry point (111 tests); `todo-store` carries t9's codec
+ golden harness, t10's event log + `Store` trait, t11's layout +
registry + locks and t12's snapshots + compaction — 86 tests, four
goldens (`serialize_nodes.v1`, `event_log.v1`, `registry.v1`,
`snapshot.v1`); mcp/cli are still documented M0 shells; rustc 1.98.1,
MSRV 1.95. Dependencies unchanged: serde, `dacode-json` + `zmij` (path
dep on the sibling checkout `../rust/json-tiers` — a cold machine needs
that repo beside this one's parent), `tempfile` (dev). Serena's semantic
layer is live.

Next step: **dispatch t13** — the `todo-cli` inspector, the last M1 code
task before g5. It is bigger than a smoke command: `dump` (canonical
graph JSON), `verify` (log integrity including §4.3's torn-tail
diagnostic), `repair` (show the torn line, truncate on confirmation), and
it inherits five things t12 and t11 left on its plate — the **lock-free
read path** (every `open_log` takes the writer lock today, so `dump`
would contend with a live server), the **log+snapshot pair as one unit**
(a compacted store is only readable with its snapshot; report the pair
divergence; the (3)–(4)-window repair is *re-base the snapshot's
`log_offset`*, proven recoverable by both corner tests), f14 (does the
log header gain `deny_unknown_fields`? the registry and the snapshot both
have it — that is the precedent, and the header marker that would close
U10's residual silent case rides the same decision), f17 (`LockTarget`'s
bare-path corner), and f19 (the order-sensitive `compact` test, writable
today by parking a directory at `<log>.snapshot`). Plus the d2/d3 split
conversation (`event.rs` 759, `log.rs` 793, `state.rs` 777). Then g5's
M1 exit gate, which now includes `cargo doc --workspace --no-deps`.
Alternative, if the user prefers: t15 (domain: f9's scan scope + f10's
four intra-doc links) is a small independent commit that can land before
or after t13 and would make g5's `cargo doc` gate unconditionally clean.

Blocked on you: **U10** (does compaction archive what it drops? nothing
calls it yet, so the window is cheap) and **U11** (propose a `dacode-json`
diff, or keep the workaround and a no-tuples house rule). Non-blocking:
U7 (lint wording), U9 (M3's wire code, still also the `"io"` mapping), U2
(pre-M3), U3 (pre-M4), and t11's narrow reading of §2's registry
metadata, which is still open to veto.

Unverified: MCP smoke (no wire until M3); per-crate lint tripwires for
mcp/cli; **no real crash or power loss anywhere** — §4.3's semantics and
t12's crash-point table are tested with hand-constructed on-disk states
built from real bytes, never an induced crash, and `compact()`'s step
*order* is not externally observable without a fault-injection seam (f19
names the technique that would make it observable); multi-process lock
contention is two handles in one process; the two permission-corner tests
are `#[cfg(unix)]` and vanish rather than pass elsewhere; snapshot
durability without a directory fsync is unmeasured (f16's accepted
reasoning, not a measurement); and nothing has yet called `compact()`
outside tests.

### 2026-09-28 — t11: D4 layout, registry, advisory locking

Dispatch was the proven one: the orchestrator decisions settled into
`tmp/t11-spec.md` first (23 decisions, 6 judgment calls, 18 required
tests), then a short spawn prompt, one implementer landing — no
P-LIVE-001 recurrence, the fourth clean dispatch in a row.

Three measurements preceded the spec and shaped it. **std
`File::try_lock`** returns `std::fs::TryLockError{WouldBlock,
Error(io::Error)}`, not an `io::ErrorKind` — so §6.1's `locked` mapping
is a match on that enum, and `rust-version` 1.95 already covers its 1.89
stabilization (no bump). Two handles in one process genuinely conflict
and `Drop` releases, so §8's contention test needs no second process.
And a lock follows the **inode**, not the path: renaming a locked file
defeats the lock, which is why the layout uses dedicated `<file>.lock`
files and why no atomically-rewritten file (the registry, t12's
snapshots) is ever its own lock target. The second measurement corrected
an assumption of the orchestrator's own: libtest **ORs** multiple
positional filters, and a filter matching nothing **exits 0**. The first
probe appeared to show AND semantics because **zsh does not word-split an
unquoted `$f`** — `cargo test -- $f` with `f="log golden"` passed one
filter, the literal string, and selected nothing while looking green.
That is P-LIVE-005, and it moved t11's verify line off name filters onto
`cargo test -p todo-store --test layout`, which selects the target
exactly and fails loudly if renamed.

Outcome: `src/{layout,lock,registry}.rs` plus edits to
`log.rs`/`store.rs`/`error.rs`/`lib.rs`, 19 tests in the new `layout`
target and 3 lib unit tests, the `registry.v1` golden, 55 store tests
and 163 workspace-wide; `Cargo.lock` unchanged, no new dependency. The
§4.2 tension t10 reported is resolved: open/create are trait
constructors under `where Self: Sized`, so the trait remains the seam a
fallback engine would implement and `dyn Store` still compiles (pinned by
a const, verified by compiling three scratch traits against it). The
inherent engine constructors were renamed `create_log`/`open_log` to stop
them shadowing the trait's, and t10's 19 tests changed only mechanically.
One narrow reading of signed §2 text is recorded in §3 and flagged for
veto: the registry holds only `{slug, registered_at}` — both immutable —
and derives the slug→store mapping, because indexing §7's mutable
`name`/`path` would need two writes per `repo_set_path` and would break
the lock-order argument that keeps §8's acquisition order total.

Dual review took four rounds, and each earned its cost. Round one: three
minors and five lows from the primary, two minors and three lows from the
secondary, converging on a recovery paragraph in `layout.rs` that
contradicted itself within four lines, an untested load-time sort whose
removal left all 49 tests green while `contains` binary-searched an
unsorted vector, and a read-only-fallback error path that replaced the
original `PermissionDenied` with `NotFound` — Decision 11's exact "never
registered" signal, so a client could have been invited to register a
repo that already exists. Round two closed all eleven fix items and left
one new stale doc: retiring the test-name "token gymnastics" had
falsified `tests/layout.rs`'s claim that the three-token union selects
the identical set. Round three was one clause, in the orchestrator's own
replacement text: it claimed the golden catches a reordered mirror field,
which the primary disproved (the mirror is serialize-only, serde matches
by name, the golden comes from the production struct) **after the
secondary had ratified it** — a reviewer count is not a proof, and the
two families' blind spots correlated exactly where the claim was newest.

**Handover.** Workspace: `main` at `5b72b76` plus the hash-backfill
commit, clean; `todo-domain` unchanged since t8 (108 tests);
`todo-store` carries t9's codec + golden
harness, t10's event log + `Store` trait and t11's layout + registry +
locks — 55 tests, three goldens (`serialize_nodes.v1`, `event_log.v1`,
`registry.v1`); mcp/cli are documented M0 shells; rustc 1.98.1, MSRV
1.95. Dependencies unchanged: serde, `dacode-json` + `zmij` (path dep on
the sibling checkout `../rust/json-tiers` — a cold machine needs that
repo beside this one's parent), `tempfile` (dev). Serena's semantic layer
is live. Landed this session: t11 (`5b72b76`) + the hash backfill,
P-LIVE-005, tracker f16/f17 and t15, f15 closed.

Next step: **dispatch t12** — snapshots + compaction (§4.1): atomic
temp+fsync+rename snapshot, replay = latest snapshot + log tail, log
truncation, the snapshot golden. It carries f5 (the domain `Counters`
needs a raise-to-max write entry point, because replaying snapshot nodes
cannot restore a counter whose highest-numbered node was deleted), f16
(whether to fsync the parent directory after an atomic rename — §4.3
names temp+fsync+rename and a dir fsync is stronger than the signed text,
plus it needs a cross-platform story), and two constraints t11
established: the store lock protects the `.lock` **inode** while
`JsonlStore.file` is an append handle to the **log** inode, so compaction
must truncate in place or reopen that handle after any rename; and a
snapshot is atomically rewritten, so it is never its own lock target —
the store's existing `.lock` covers it. Why t12 before t13: the inspector
should dump and repair a store whose final on-disk shape already exists,
and t13's verify line reads a snapshot that t12 defines.

Then t13 (the `todo-cli` inspector + §4.3's repair command), which
carries: f14 (does the log header gain `deny_unknown_fields`? the
registry's is the precedent, and the reason for the divergence is
documented in `registry.rs`), f17 (a contended bare-path open names the
log's parent directory as a slug, so `LockTarget` can name an uninvolved
repo and its `None` arm reports `io` rather than §6.1's `locked` —
whether `LockTarget` grows a third variant is t13's call, §6.1's list
being closed), d2 (`event.rs` at 759 lines, split the replay half when
t13 grows it), the registry↔store divergence report (a store directory
the registry does not list), and a **lock-free read path** — today every
`open_log` takes the writer lock, so `dump`/`verify` over a committed log
would contend with a live server. Then g5's M1 exit gate. t15 (domain:
f9's scan scope + f10's four intra-doc links and the stale `testcodec`
doc line) is a small separate commit that can land anywhere; it was kept
out of t11 on crate boundaries, not on priority.

Blocked on you: nothing. Open, non-blocking: U7 (lint wording), U9 (M3
wire code — still also the `"io"` mapping), U2 (pre-M3), U3 (pre-M4);
and the registry's narrow reading of §2 above is yours to veto — the
window is cheap until M3 ships a `repo_list` against it.

Unverified: MCP smoke (no wire until M3); per-crate lint tripwires for
mcp/cli (domain's was proven at M0, the rest rest on manifest reads +
green clippy); real crash behavior — §4.3's semantics are tested with
hand-built fixtures and a documented write ordering, never an actual
crash or power loss; multi-process lock contention is tested as two
handles in one process, measured equivalent on this host but not proven
across processes; the two permission-corner tests are `#[cfg(unix)]` and
were confirmed non-vacuous at uid 502, so on a privileged or non-unix run
they print a note or vanish rather than pass; and the registry's
durability without a directory fsync (f16) is unmeasured.

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

**Later the same session — t10: the JSONL event log + `Store` trait.**
Dispatched on the user's go with the same protocol: the orchestrator
decisions settled into `tmp/t10-spec.md` (the 17-kind event taxonomy
mirroring the domain mutation surface 1:1; store-owned patch DTOs and
zero domain changes — `drafts.rs`'s module doc is the mandate; replay
equality on `Graph`'s existing `PartialEq` derive; f8 closed via a
store-side sequential-id check; f6 via the event `ts`), a short spawn
prompt, one implementer landing — no P-LIVE-001 recurrence, the third
clean dispatch in a row. Outcome: `src/{event,store,log}.rs`, 19 new
tests under `tests/log/`, the `event_log.v1` golden (35 lines, every
kind, the cascade gap visible in the ids), 33 store tests and 141
workspace-wide; `tempfile` enters as dev-dep (Cargo.lock 13 → 26, its
tree only). Two traps surfaced and were recorded as pitfalls: serde
collapses absent vs `null` on a plain `Option` patch field, so "clear
this field" replays as "leave it" — fixed with `skip_serializing_if` +
a double-option deserializer (RFC 7386 semantics), pinned by a test
that cannot compile under the naive `Option` (P-LIVE-003; M3's wire
patch schemas face the same trap); and a `tests/<name>.rs` crate root
cannot pair with `tests/<name>/mod.rs` (E0761), so the P-LIVE-002
wrapper must be an inline `mod log { mod fixtures; … }` (P-LIVE-004,
inherited by t11–t13). Dual review approved in one round with four low
findings: the log header accepts unknown fields (no
`deny_unknown_fields`; inert today — t13's tamper/repair story should
decide it consciously, f14); the `tests/log/` determinism test names
dropped the workflow's `*_deterministic`/`*_byte_stable` suffixes (f15,
rename rides a later tests-only commit, possibly with f9); `event.rs`
is 759 lines against the 600 target (split the replay half when t13
grows it — debt d2); and the `Option<Option<T>>` deviation from the
task spec's Decision 5 was ratified by both reviewers as an
improvement (documented, golden-pinned). Tensions reported, not
silently fixed: §4.2 lists open/create inside the trait, but they stay
inherent `JsonlStore` constructors until t11's layout exists (the
trait docs say when they join); `code()` returns `"io"` for io
failures, documented as not a §6.1 code pending U9. Tracker closures
this commit: f6, f7, f8, f11, f12 done with evidence.

**Handover.** Workspace: `main` at this commit, clean; `todo-domain`
carries the §3 value types + the §5 graph and order (108 tests,
untouched since t8); `todo-store` carries the canonical codec + golden
harness (t9) and the JSONL event log + `Store` trait (t10) — 33 tests,
two goldens; mcp/cli are documented M0 shells; rustc 1.98.1, MSRV
1.95. Dependencies: serde (domain; store with `derive`), `dacode-json`
+ `zmij` (store; path dep on the sibling checkout `../rust/json-tiers`
— a cold machine needs that repo beside this one's parent), `tempfile`
(store dev). Serena's semantic layer is live. Landed this session: t8
(`6cc4691`), t9 (`fc23f27`), t10 (this commit) + hash backfills. Next
step: **dispatch t11** — D4 layout + registry + advisory locking
(§2/§7/§8): `repos/<slug>/` under an injected data root, the registry
file with its own golden and §4.3's atomic-rename discipline, std
`File::try_lock` (no locking crate — measured at the M1
decomposition), sorted-slug lock acquisition order, the `locked` error
(the variant t9's error-module growth path announced), and §4.2's
open/create moving into the `Store` trait — t10's reported tension
resolves there. t11 inherits P-LIVE-002/004 (inline `mod` wrapper for
the `-- layout registry lock` filters) and may fold in f15's rename.
Then t12 (snapshots + compaction, carrying f5's raise-to-max counters
entry point and riding t10's event schema), then t13 (the cli
inspector + §4.3's repair command, carrying f14's header decision and
d2's split review), then g5's M1 exit gate. Blocked on you: nothing.
Open, non-blocking: U7 (lint wording), U9 (M3 wire code — now also
the `"io"` mapping), U2 (pre-M3), U3 (pre-M4); findings f4, f5, f9,
f10, f13, f14, f15 and debts d1, d2 carry verify lines in the
tracker. Unverified: MCP smoke (no wire until M3); per-crate lint
tripwires for mcp/cli; real crash behavior — §4.3's semantics are
tested with hand-built fixtures (torn lines, spliced garbage), never
an actual crash or power loss; multi-process lock contention will be
tested as two handles in one process at t11, not two processes.

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
