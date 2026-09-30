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

**Phase: M1 done 2026-09-30 — M2 next.** The domain layer (§3's value
types and closure discipline, §5.1's graph, §5.2's canonical order), the
storage engine (the signed D1 JSONL event log behind the `Store` trait
carrying all five of §4.2's responsibilities, the D4 layout with its
registry and advisory locking, snapshots + compaction, §4.3's repair
primitive) and the `todo-cli` inspector are all in, and the **g5** exit
gate ran green on the final tree: every code gate, all five goldens
regenerating byte-identical under `UPDATE_GOLDEN=1`, `cargo doc
--workspace --no-deps` clean, the Serena semantic layer resolving over the
workspace, and M1's lessons consolidated into `.agents/pitfalls.md` —
fifteen entries promoted from f20/f24/f30 and `P-LIVE-001…006`, the live
file reset with a former-id table so the in-tree citations still resolve.
253 tests across three crates with real logic; `todo-mcp` is still an M0
shell, which is M3's business.

| Blocking | What | Action |
|---|---|---|
| — | nothing blocks M2: **U2**'s survey ran and left §6's signed shapes intact | decompose M2's query layer, answering **U17** inside the decomposition |
| **D14**, **D15** | the wire's canonical JSON key order, and where a §6.1 code lives on the wire — both out of U2's rmcp survey (spec §6.3), both gating M3, neither touching a domain shape | **vote** on each; M2 does not wait for them |

(Open but non-blocking: **U15** and **U19**, two one-sentence store doc
fixes that ride the next store touch; and three recorded 2026-09-30 —
**U16** (ship base instructions with the server and gate the client's
native todo off, M3 — its mechanism is now measured rather than assumed,
spec §6.3), **U17** (which harness tools become MCP tools, answered with
M2's decomposition) and **U18** (a live view for progress and whether the
server is shared per user, deferred until after M4's importer check, and
holding the **D13** reservation). Resolved: U2/U4/U5/U6/U7/U8/U9/U10/U11/
U12/U14, and U3 superseded by D12. U13 is acknowledged debt, not an open
question.)

## 2. Milestones

Authoritative names: `.agents/config/agent-rules.yaml` → `milestones`;
exit criteria: spec §14. Priority is derived from the Gated-on column by
`tools/attention.py`, never hand-written.

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | spec settled: §12 items 1–3 signed (done 2026-09-25); Cargo workspace skeleton with the lint regime; all gates green | — | done 2026-09-25 |
| M1 | domain model + storage engine: graph, statuses, persistence behind the `Store` trait, golden files | — | done 2026-09-30 — g5 ran green: every code gate, five goldens byte-identical, `cargo doc` clean, pitfalls consolidated |
| M2 | queries: repo/epic selection, dependency resolution, deterministic `next_task`; registry with path updates | — | next — U2's survey ran 2026-09-30 (spec §6.3), so the decomposition is unobstructed; it answers U17 and re-measures d1 and U13 |
| M3 | MCP server: the full §6 tool list over the wire, protocol-level tests, default-repo slug config (D8), smoke against a real client | **D14**, **D15** — the wire's canonical key order and §6.1's error carrier, both from U2's survey; plus the rmcp dependency ask itself (Ask-First) | not started |
| M4 | adoption: the dacode-dialect importer first (**D12**, amending D7), its board the only fixture checked in per §10; camera and ikvm-html imported from their live boards and never copied into this repo; the three repos tracked for real | — | not started |
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
- 2026-09-29 — **five open items settled in one exchange**: U10, U7, U3
     (as **D12**), U2's scheduling, and t11's registry veto window.
     **U10** resolved to (a) drop-as-landed plus *nothing calls
     `compact()` yet* — the reframe being that compaction is not garbage
     collection of done work and does not necessarily free space, since
     the snapshot holds the whole current graph and the trade is log bytes
     for snapshot bytes; §4.1's own scale argument says replay is already
     cheap, so the t12 reading is blessed into §4.3 rather than left as a
     crate-local argument. f14 was **split**: `deny_unknown_fields` on
     `LogHeader` rides t13 (deserialize-only, no golden moves), the
     compaction **marker is deferred** until something can actually
     compact, and **t13 ships no `compact` subcommand**. **U7** resolved by
     a third option — delete the count from §10 rather than enumerate nine
     names in signed text, since `_rust.md` is already the enumeration;
     AGENTS.md item 1 states the nine and says why the prose carries no
     count. **U3** was not executed as written: the user signed **D12**,
     reversing D7's order to **dacode-first** and making camera/ikvm-html
     **live-test subjects whose board content is never copied into this
     repository**. D12's dossier states the consequence plainly rather
     than hiding it — two of three dialects get **no pinned golden**, which
     is weaker and non-reproducible, with a synthetic camera-dialect
     fixture as the named remedy if M4 finds it too weak. **U2** is
     scheduled **after g5, before M2**, because it is the only open item
     that could invalidate a signed §6 shape and therefore U9's
     just-amended §6.1. **t11's** narrow registry reading — mapping
     derived, only the two immutable fields stored — is **accepted**; the
     veto window closes, and the revisit at M3 is to be driven by a
     measurement of cold `repo_list`, not a preference, per U1's own
     precedent. Spec work order executed across §4.3, §6, §9, §10 (twice),
     §14; D7's banner amended and D12 added to `decisions.md` with a
     sign-off-summary row.
- 2026-09-29 — **M1 t15: domain cleanup** — f9, f10 and f22's domain half
     in one commit. **f9**: `assert_invariants`, the whole-graph scan run
     after every step of the seeded property tests, checked `id.repo()`
     against the graph slug for epics and tasks but only `.repo` for
     decisions and docs, while `Graph::in_scope` is called at every
     explicit insert with **both** — so a regression that stopped checking
     the id half would have slipped the scan for those two kinds. The
     second `for epic` loop (counter check only) folded into the first.
     **Extending the assertions was not enough, and the two assertions it
     took are the least of it**: the generator only ever
     forged `.repo`, so the new assertions were unreachable and the guard
     would have been vacuous. `explicit_insert`'s `foreign: bool` became a
     `Forge` enum drawing `None`/`Repo`/`Id` off the same single
     `below(10)` in the same stream position, and three `other_*_id`
     helpers joined the pre-existing `other_task_id`. Then a **coverage
     pin** in the 5-seed test asserts all four insert kinds × both halves
     actually occur, so a seed or `STEPS` change cannot silently vacate
     the scan — and the traced half is read off the **built node**
     (`scope_half`), not off the draw, because label and effect can
     otherwise decouple. Non-vacuity was proven by mutation throughout,
     not asserted: making `in_scope` ignore the id's slug fails both seeded
     tests; starving either draw arm fails the pin naming kind and half;
     and `Forge::Repo => epic.id = other_epic_id(..)` — the sharpest
     mutation, which leaves the epic repo-half uncovered while its label
     stays green — is caught only by the node-derived form. **f10**: the
     four unresolved intra-doc links in `graph/drafts.rs` now use the
     workspace's single link convention (`[`crate::Epic`]`, 39 existing
     uses, and the `](crate::` form is now absent from the workspace), and
     `testcodec.rs`'s stale "before the canonical serializer lands" line is
     replaced with why the codec *stays* — §2 bars a domain→store
     dependency, and the two answer different questions. **f22 domain
     half**: `error.rs`'s module *and* type docs, both of which said
     `ValueError` "carries no §6.1 code", realigned to the amended §6.1;
     the store half rides t13. Gates: fmt, check, clippy clean; 111 domain
     tests (count unchanged — f9 extended a shared helper and a generator,
     it added no test); 197 workspace-wide; and **`cargo doc --workspace
     --no-deps` is warning-free for the first time**, which is g5's gate
     line and was the reason t15 existed. No golden touched.
     Dual-reviewed to sign-off, both reviewers `ok` on the committed tree.
     The findings that shaped it: the primary found the vacuous-generator
     gap, the omitted `ValueError` *type* doc, a measurement restated
     three times and wrong differently each time before the numbers were
     deleted rather than corrected again, and the label/effect decoupling
     in the coverage pin;
     the secondary found a present tense for an unimplemented M3 mapping,
     the convention drifts, an off-by-one in a size ledger whose point was
     that it had been measured, and — sharpest — that this very board entry
     asserted a dual-review sign-off the round record contradicted, which
     is the forged-trailer failure mode caught from inside the diff rather
     than       at the commit. Landed as `0a23350`.
- 2026-09-29 — **instructions: the production-line file budget and the
      persisted-schema tuple rule** (two user decisions taken while
      dispatching t13). `_rust.md`'s file-size rule now counts
      **production lines**: a `#[cfg(test)] mod tests` block inside a
      production file is not charged, and a file that *is* test code —
      declared `#[cfg(test)] mod x;` by its parent, or under
      `crates/<crate>/tests/` — is not charged at all, with the `awk`
      one-liner that measures it and an explicit warning not to "fix" a
      test-only file by splitting it. Over-target becomes a **recorded
      exception** rather than a silent one: a module-doc line saying why
      the file is over and what would split it, plus a board §5 entry —
      with one missing it is a review finding, with one it is acknowledged
      debt. The 1000-line cap stays unwaivable. AGENTS.md item 5 carries
      the same summary. Measured consequence, now committed as **U13**
      (previously d2/d3 in the gitignored tracker only): `graph/state.rs`
      reads 777 under `wc -l` and is **460** production lines, so it comes
      off the ledger entirely, three test-only files were never
      chargeable, and the real overage is `log.rs` 793 and `event.rs` 759.
      `_rust.md`'s determinism section gains U12(b)'s house rule —
      **named-field structs, never tuples, in a persisted schema** —
      argued on §4 auditability and §4.3's repair surface and stated to
      outlive the engine bug U11 fixed, with `relates` named as the
      format's one positional shape; the same file's stale "whether the
      server is async is the question D1/D6 constrain, until signed"
      bullet is corrected to cite §2's settled text. **U12 struck** ((a)
      `relates_from_arrays` stays for its `corrupt_store` detail, (b) the
      house rule landed) and **U14 opened** for the cli's argument parser,
      carrying the nine-variant measurement table so the numbers survive
      the gitignored probe.
- 2026-09-29 — **M1 t13a: the store riders** — the store half of t13,
      split along the crate boundary so the cli (t13b) can wait on U14. No
      new dependency, no golden moved, no file split. **f14**: `LogHeader`
      gains `deny_unknown_fields`, and `check_header` becomes probe-first
      — a lenient `VersionProbe` parse, then the version check, then the
      strict parse, then the torn check — so a header from a *newer* format
      carrying fields this build has never seen still reports
      `version_mismatch` (a migration question, §4) while an unknown field
      at the *current* version is `corrupt_store` at offset 0. The registry
      and the snapshot already ordered it that way; the log header was the
      one document that did not, and the third private copy of
      `VersionProbe` became one `pub(crate)` struct in `codec.rs` stating
      the rule once. **f17**: `LockTarget` gains `Unnamed`, so a
      *contended* bare-path open reports §6.1's `locked` — what actually
      happened — instead of degrading to `io` with a fixed detail;
      `DETAIL_NO_SLUG` is gone, §6.1's code list is untouched, and the
      variant is unreachable through `DataRoot`, t13b's inspector included
      (it opens by slug). **f18**: the no-directory-fsync paragraph's
      enumeration is corrected to the two states a lost rename can
      actually leave — the "missing new file, a `corrupt_store`/`io`
      report" state it had inherited does not exist, since a missing
      registry loads empty and a missing first snapshot replays the whole
      log, both correct opens — and keeps the amended `compact()` order's
      one real window (a lost step-(4) re-base: `log_offset` past the
      log's end, recoverable by re-basing).
      **f22's store half**: the three sites in `error.rs` claiming `Io`
      carries no §6.1 code now say `io` *is* one (U9, with its retry
      semantics), and the module doc's "delegates the first three here by
      name", which undercounts what the domain's own module doc names —
      all four — says so. **f23**: `snapshot.rs`'s "open question above
      this crate — a board follow-up for the user" now states U10's signed
      answer and the split f14 outcome (`deny_unknown_fields` here, the
      persistent compaction marker deferred until something compacts);
      `store.rs`'s `compact()` pointer no longer implies the question is
      open. **U12(a)**: `relates_from_arrays`'s doc drops the two sentences
      U11 falsified — "the JSON engine cannot drive serde's tuple
      deserializer" and "if the engine grows tuple support, this helper
      reduces to the derived impl" — and records that it now stays *by
      choice*, for the `corrupt_store` detail t13b's `repair` shows a
      human, with `_rust.md`'s new house rule cited and `relates` named as
      the format's one positional shape. **U13**: `event.rs` and `log.rs`
      carry their module-doc exception lines — why each is one
      responsibility and what would split it — with **no line count in
      either**, per t15's lesson that a figure in a source comment is a
      claim nobody re-measures; the ledger is U13's and was re-measured at
      commit. 87 store tests (one new: the contended bare-path
      `Locked { Unnamed }`), the domain's 111 unchanged, `Cargo.lock` and
      all four goldens untouched, `cargo doc` still warning-free. Both new
      header guards are proven non-vacuous by mutation, run and reverted:
      deleting `deny_unknown_fields` makes the unknown-field-at-v1 fixture
      open clean instead of failing, and deleting the probe makes the v2
      fixture report `corrupt_store` where `version_mismatch` is owed.
      Three sentences this commit *corrects* are review findings rather
      than spec items — the undercounted module doc, the nonexistent
      "missing new file" state, and a clause naming t13's inspector as a
      future *non-layout* caller that this commit's own decision falsified
      — all three in prose that a previous task had reworded, which is why
      the round record lives in the session tracker and not here.
      Dual-reviewed to sign-off on the committed tree. Landed as
      `ffa4be8`.
- 2026-09-29 — **M1 t22: the store's §4.3 repair primitive** — the commit
      t13b needs and did not have. Repairing a torn tail truncates the
      log, truncating is a write, and §8's lock plus the log's line
      discipline both live in `todo-store`; a cli that re-derived them
      would be a second implementation of the single-writer protocol, so
      the primitive lands here and the binary prints what the crate
      decides. `DataRoot::repair(slug, dry_run)` returns
      `RepairOutcome::{Clean, TornTail { torn, truncated }, Unrepairable
      { offset, detail }, Diverged { log_offset, log_len }}`, and a
      *refusal* is an `Ok`, never an `Err` — §6.1's list is closed and has
      no code for "not repairable". Diagnosis runs the crate's **own**
      `replay` over the bytes, so repair's classification agrees with
      `open`'s by construction rather than by parallel logic, and it is
      structural throughout: a `detail` string is explicitly not the
      contract, so none is ever matched. The order is the argument — lock,
      read, replay, then the snapshot divergence **before** the torn-tail
      test (an out-of-band truncation can present both, and the divergence
      dominates because truncating would not fix it), then **validate the
      truncated prefix by replaying it before writing anything, dry run
      included**, which is §8's atomic-rejected-write discipline applied
      to a repair and is what makes a dry run's promise identical to a real
      run's, then temp+`fsync`+atomic rename with no directory `fsync`
      (f16's closed decision covers this rename too; no append handle is
      open, and a future `open_log` opens by path, so the rename orphans
      nothing). `Err` is deliberately wider than the task spec's
      three-item list and the deviation is pinned by test: an unparseable
      snapshot document propagates, because `repair`'s `Err` is then
      byte-identical to `open`'s — the agreement property the primitive
      exists for — while `Unrepairable.offset` is a *log* position by
      contract and must never carry a snapshot byte. New `src/repair.rs`,
      342 production lines; `log.rs` and `event.rs` unchanged beyond
      visibility keywords, which is the ledger rule U13 now enforces. 106
      store tests (19 new), the domain's 111 unchanged, no golden moved,
      `Cargo.lock` unchanged, `cargo doc` clean. Dual-reviewed to sign-off
      on the committed tree, and the review earned its keep: the primary
      found that the one guard standing between "refuse" and §4's silent
      event loss — the check that the fault's offset is the *final* line's
      — was load-bearing and **unpinned**; deleting it left every test
      green while a store holding interior damage beside a torn tail
      truncated to its header-plus-seed prefix, showed the human confirming
      the repair a torn line that was not at the offset it reported, and
      then opened to a graph that was not the pre-fault one. The test that
      pins it now is
      `interior_damage_beside_a_torn_tail_is_refused_at_the_interior_offset`,
      and the mutation was re-run after the fix to confirm it fails. Four
      lows landed with it: a refusal reason that read false for one of the
      two states it covered (split in two, with the unreachable fallbacks
      given their own name rather than borrowing a reason), tests for the
      three arms that had none, a module-doc rationale resting on a
      half-true premise about where snapshot offsets come from (**f34**:
      `snapshot.rs`'s `corrupt()` keeps the codec's offset, only
      `unpositioned()` has none — the reviewer's premise, corrected by the
      implementer and confirmed by both on re-review), and a doc
      justification pointing at a constant in a file its own test target
      does not include. One taste-level wording finding was dispositioned
      rather than opening a fourth round for one word (**f35**). Landed as
      `4c5782b`.
- 2026-09-29 — **M1 t13b/t20: `todo-cli`, the store inspector** — D5's
      rationale met: the store can now be dumped, verified and repaired
      without an MCP client, which is what made M3 safe to build on. A lib
      target plus a 36-line `main.rs`, so the pieces worth testing are
      testable without spawning a process and the binary only translates
      (§2): `args.rs` parses with `lexopt` (U14) into a typed invocation,
      `dump.rs`/`verify.rs`/`repair.rs` format what `todo-store` decided.
      No `dacode-json` dependency — §2 permits this crate to name the
      engine and it deliberately does not, so the engine stays sealed in
      one crate and `--pretty` stays impossible (U14). `dump` serializes a
      cli-local document (`repo`, `counters`, the four node kinds, `blocks`
      in the log's own `BlocksEdgeDto` spelling, `relates` in the format's
      positional shape) with `to_bytes`, in the graph accessors' own total
      orders, so determinism is structural and the fifth golden
      (`cli_dump.v1`, 2539 bytes) pins it. **`Counters` stays serde-free**
      (t7's rule) and is mirrored in a named-field `CountersDto` rather
      than omitted, because a burned number is unreconstructable from
      graph state: the golden's cascade gaps read like damage without
      counters proving the numbers were issued and burned (§3). `verify`
      reports counts and file sizes — **no line count**, since counting
      lines would be a second notion of "line" beside the log's own
      `LineSplit` (f25's parked boundary) — and on a `corrupt_store` fault
      asks the store to classify it via `repair(slug, dry_run=true)` rather
      than guessing, printing t22's outcome verbatim (f33: a refusal reason
      is cli-facing contract text) with f31's caveat that a snapshot
      document's offset is not a log byte. `repair` prompts with the
      offset, byte count and bounded preview; EOF or garbage aborts,
      `--dry-run` never prompts and never writes, and `--dry-run` beats
      `--yes`. Exit codes are one pinned table — 0 worked (including "clean"
      and "nothing to repair"), 1 ran and reported a fault it will not act
      on, 2 usage, 3 the store refused — with §6.1's code printed on its own
      line as the machine-readable identity. The spec contradicted itself
      on one row (a corrupt snapshot document: table row 3 said exit 3,
      step 4 said every `CorruptStore` exits 1) and the resolution is the
      honest one, ruled correct on review: `verify` exits 1 with
      `classification: unavailable` because it *did* inspect and found a
      fault, while `dump`/`repair` exit 3 because they could not do their
      job — the exit code encodes the command↔store relation, not the store
      state. 35 cli tests, 252 workspace-wide, the store's 106 and the
      domain's 111 unchanged, store goldens untouched, `Cargo.lock` up by
      exactly one package. Dual-reviewed to sign-off on the committed tree,
      and again the review found the load-bearing gap: "`--dry-run` wins
      over `--yes`" was pinned only at the parse level, so swapping the two
      guards left all 34 tests green while the combination truncated — a
      write the operator had explicitly disclaimed. One test pins it now
      (`dry_run_wins_over_yes_and_writes_nothing`), mutation-proven in both
      directions by the implementer and re-run by the reviewer, with
      `src/repair.rs` sha-verified unchanged by the fix. Three spec errors
      were reported rather than worked around silently (f36–f39), one of
      them the exit-table contradiction above. Landed as `e5a9955`.
- 2026-09-30 — **M1 t21: the order-sensitive `compact()` test (f19)** —
      the last M1 code task: tests-only, one new test in
      `tests/snapshot/compact.rs`, no `src/` change. The two existing
      (3)–(4) window tests *construct* the crash's on-disk state from real
      bytes, so they pin what the window leaves and cannot see the order
      that produced it — reversing `compact()`'s five steps left the whole
      suite green, which is exactly what f19 recorded. This one *induces*
      the failure instead: a directory parked at `<log>.snapshot` is a
      rename target `rename(2)` refuses, so step (1)'s covering snapshot
      cannot land, and the test asserts the file-level consequences of the
      covering write coming first — §6.1's `io` naming the snapshot's
      rename rather than step (3)'s, the log byte-for-byte unchanged, the
      parked directory still a directory, exactly one §4.3 temp file left
      behind, an append on the *same handle* still landing in the log at
      that path (`Store::snapshot`'s "not session-fatal" promise, which no
      other test covers), the parked directory being a named `io` fault on
      the read side rather than a silent "no snapshot", and — with the
      injected fault removed — a reopen to the pre-compaction graph plus
      that append, burned numbers included. Non-vacuity by two mutations,
      both run and reverted with `src/log.rs`'s sha verified identical:
      reversing the step order (truncate, then cover) fails exactly this
      test at the byte assertion while both corner tests stay green, and
      changing the debris expectation to zero fails with `left: 1`, so the
      temp file's presence is measured rather than assumed. The platform's
      `ErrorKind` spelling is deliberately unpinned — the context string is
      this crate's contract text, the kind is the platform's. 32 tests in
      the `snapshot` target (was 31, 0 filtered out), 107 in `todo-store`,
      253 workspace-wide; `cargo doc --workspace --no-deps` clean; all five
      goldens byte-identical, including under `UPDATE_GOLDEN=1`;
      `Cargo.lock` untouched. Dual-reviewed to sign-off on the committed
      tree over three rounds. Round one found the load-bearing comment
      claim that did not reproduce: the `io` context was said to
      distinguish step (1)'s rename from step (4)'s, and both pass the same
      subject `"the snapshot"`, so they are indistinguishable there — it is
      the untouched log that places the refusal before step (3). With it
      went an over-generalized pid-truncation claim (the name is
      process-scoped, so a later session's debris is inert, not
      self-cleaning) and an unpinned debris sentence that sat in tension
      with the test's name, resolved by the rename to
      `a_refused_covering_snapshot_leaves_the_log_untouched` plus the new
      assertion. Round two found the same defect class one level up: the
      review briefing claimed a tracker finding (**f40**) existed when it
      did not, so the deferral it justified was unrecorded. f40 is now
      recorded and promoted to **U15** — `store.rs`'s "a failed snapshot
      changes nothing on disk" is falsified by the debris this test pins,
      and both remedies are `src` changes, so neither belongs in a
      tests-only commit. Landed as `7563fff`.
- 2026-09-30 — **M1 exit gate (g5): M1 closed.** Every code gate on the
      final tree — `cargo fmt --check`, `check --workspace --all-targets`,
      `test --workspace` (253: domain 111, store 107, cli 35), `clippy
      --workspace --all-targets`, `cargo doc --workspace --no-deps` — clean;
      all five goldens regenerate byte-identical under `UPDATE_GOLDEN=1`
      (twelve store writers plus the cli's `cli_dump.v1`); the Serena
      semantic layer resolves a `todo-domain` symbol
      (`impl Graph/raise_counters_to`, which is also the name-path shape
      this gate measured — `/Graph/raise_counters_to` resolves to nothing,
      because methods hang off the `impl` node); every doc gate green.
      **The consolidation the gate owed**: `.agents/pitfalls.md` goes from
      deliberately empty to fifteen entries across its five categories plus
      a distilled top ten, promoted from f20 (a spec's safety argument is a
      claim to falsify, not ground truth to instance-check), f24 (a commit
      touching an indexed doc cannot carry its own regenerated index), f30
      (no measurement in a source doc comment) and `P-LIVE-001…006`;
      `.agents/pitfalls-live.md` resets with a former-id table, because
      test-file module docs in both crates cite `P-LIVE-002`/`-004`/`-005`
      by id and those citations must keep resolving. Two rules moved to
      where they are *enforced* rather than where they are recalled —
      `_rust.md`'s comment section carries f30's rule, `_search.md`'s
      Serena specifics carry the impl-node name path (`version.custom` 6→7,
      five agents regenerated). The two entries that cost M1 most are 2.1
      and 5.1, and they are one defect seen from both sides: a task spec's
      crash-safety argument was instance-checked instead of falsified, and
      the tests built the crash's aftermath instead of inducing it — t12's
      two silent corners and t21's fix are the same lesson, which is why
      5.1's rule binds spec authors and reviewer prompts rather than code.
      Deliberately **not** claimed at this gate: no induced crash or power
      loss anywhere (t21's rename refusal is the first externally injected
      fault and is still not a crash), no MCP smoke (M3), and the per-crate
      lint tripwires for `todo-mcp` and `todo-cli` have never been fired.
      The harness consolidation landed as `1581007`; this board movement
      rides the gate's own commit.
- 2026-09-30 — **`tools/progress.py`: the total-progress view, promoted from
      scratch.** The page answering "what is the shape of the work" existed
      for one session as `tmp/gen-progress.py` and now has a home beside
      `attention.py`, which answers the other question ("what needs the
      user"). It renders the tracker and board §2/§4/§5 as a dependency graph
      plus totals, to `.agents/cache/progress.{html,json}` — the established
      home for a rendering of committed state, gitignored, never a source of
      it. House CLI (`--open`, `--json`, `--out`, `--no-write`), and two
      self-rules that a renderer needs because quietly showing less than
      everything is worse than showing nothing: **it cannot pass by reading
      nothing** (a missing or empty tracker, or a board with no milestone
      rows, is an error rather than an empty page) and **it cannot drop a
      relationship silently** (a declared edge whose endpoint has no lane
      exits non-zero naming it, an unparsable board item exits non-zero
      quoting it, and every open tracker node that is not drawn is reported
      by id). Both guards fired while it was landing: the edge guard on five
      gating edges whose endpoints had no lane, and the board parser on two
      items whose bold titles wrap onto a second line (**f43**), which had
      been rendering four open board items where the board holds six. The
      lanes are a drawing order and a judgment; the
      edges carry a citation each, surfaced on hover. Every percentage
      travels with its denominator, on the page and in the JSON, because
      "progress" is not one number: milestones measure the roadmap, tasks
      measure the work *decomposed so far* — a figure that falls as M2–M5 are
      decomposed, which the footer says out loud — and findings and debt are
      records rather than deliverables. Two runs at one HEAD are
      byte-identical. It also found a defect on the way in, recorded as
      **f42**: `agent-rules.yaml` had two top-level `commands:` keys, and a
      duplicate YAML key is silently shadowed, so the entire build-command
      block was dead config that read like live config (`fmt_check` is absent
      from the parsed file). Renamed to `build_commands`, with the reason in
      a comment; `version.custom` 7→8. The tool is harness, so it is
      review-exempt and carries no trailer. Landed as `d542065`.
- 2026-09-30 — **tracker sweep (f41): the exit gate now sweeps statuses.**
       M1's gate closed with items whose work was already in git history still
       marked `pending`, and `tools/tasks.py check` could not see them — it
       flags a `done` item with no evidence and a `blocked` item with no
       reason, never a stale `pending`. The rule now lives in two homes, the
       binding one and the explanatory one: `todo-next-imp`'s exit-gate step 2
       says **`check` is not the sweep**, names what it is blind to, and
       requires reading the open list against `git log` — every delivery
       disposes of the items it consumed, a milestone gate of all of them;
       `task-tracking.md`'s "`check` is the gate" section carries the
       mechanism plus the two dispositions a sweep produces (`wontfix` with a
       note for what will never be fixed, a board `U` id for a deferral that
       must outlive the branch). Running it closed the nine t20/t22
       spec-error records whose consumers had landed, each with evidence
       measured on `5c23f5f` rather than inferred — a `file:line` for every
       claim, and the pins re-run (`todo-store --test repair` and the whole
       `todo-cli` suite green, plus f38's own two filters each selecting
       exactly one test) — and found four stale items the finding had not
       named: **f42** (its duplicate-key verify re-run green; the fix landed
       in `d542065`), **f13** (`wontfix`, because its own text already read
       "accepted, not fixed", so `pending` contradicted the recorded
       decision), and the two size-ledger debts, whose verify moment *was*
       t13's closure and which are retired against **U13** instead of being
       carried in two places that had already drifted apart. One finding is
       deliberately **not** closed: **U19**, promoted from f35, is a one-word
       tightening of `repair.rs`'s module doc whose remedy has not landed, and
       closing it to shorten the open list would be the silent-omission class
       running in reverse. f4 (M3's wire), f25 (the parked diagnostics
       boundary), f40 (= U15) and d1 (M2's queries) stay open, each with a
       note naming the consumer it waits on so the next sweep can tell a
       deliberate open from a forgotten one. Harness and board only; no Rust
       moved, so every code gate is `skipped` by construction. Landed as
       `95a83eb`.
- 2026-09-30 — **U2: the rmcp survey against §6 (spec §6.3), and two
       decisions drafted out of it.** rmcp **3.5.0** — the version M3 would
       adopt — was read in the crates.io source *and* driven over raw stdio
       from a probe server under gitignored `tmp/rmcpprobe`, so the answers
       are captured wire bytes rather than a README reading, and no
       dependency entered the workspace: M3's Ask-First is untouched. **§6's
       signed shapes hold.** Every tool in the table is declarable, and one
       `Parameters<struct>` per tool is the only shape the macros accept
       (`schema_for_input` rejects a non-`object` root), which makes §6's
       per-tool params a requirement rather than a style; `instructions` is
       settable in **both** handshakes — `initialize` and 2026-07-28's
       `server/discover` — so U16's mechanism is measured rather than
       assumed; prompts are exposed; and array order, which is what §5.2's
       total order actually is, survives the wire intact. **Two facts force a
       choice and are drafted as D14/D15 rather than settled here.** Object
       key order does *not* survive: structured output is serialized by
       rmcp's own `serde_json` into a `BTreeMap` unless `preserve_order` is
       enabled, and both spellings were measured from the same probe source,
       so §6.2's "fixed field order" needs either that flag, a text-carried
       canonical dump, or an amendment naming two canonical orders. And
       §6.1's twelve codes have five possible carriers, of which the
       protocol-level one is unusable because rmcp documents that clients
       render those errors opaquely — the model never sees the code. Three
       facts M3 must design around are recorded in §6.3 with their citations:
       `serverInfo` defaults to the SDK's own `rmcp/3.5.0` identity; requests
       are dispatched **concurrently** (each in its own `tokio::spawn`,
       responses observed out of order twice), so §8's single writer needs a
       mutex or an actor *inside* the process, the store's advisory lock
       being per file descriptor; and argument-shape faults arrive as rmcp's
       own unstructured text, so `invalid` covers only values this server
       parses itself, and M3's params structs deny unknown fields. Dependency
       weight measured for the eventual ask: 81 `Cargo.lock` packages, 56
       unique normal dependencies, a tree MSRV of 1.88 against this
       workspace's 1.95 floor, and a 15 s cold build on this host with a warm
       registry cache. Spec §6/§6.3/§12 moved, `decisions.md` gained two
       dossiers plus their summary rows and action lines, and board §1/§2/§4
       moved with them (U2 struck). No Rust moved; every code gate is
       `skipped` by construction. Landed as `8135d06`.

## 4. Follow-ups

Unowned work. Take the next unused `U` id; never renumber, never reuse.

- [x] **U1 · Sharpen §4.1's engine comparison with a measurement.** A
      replay+snapshot micro-benchmark at 10^5 nodes for the JSONL option,
      and a redb write/read equivalent — numbers turn D1 from a lean into
      a decision. `action: measure` material for D1, though D1 itself is a
      vote. **Superseded 2026-09-25**: D1 signed without the
      measurement; these numbers become decision material only if the
      redb fallback (D1's banner) is ever proposed.
- [x] **U2 · Survey rmcp's tool macros against §6.** Confirm the signed
      tool list is expressible with stable error codes and ordered list
      responses as rmcp schemas; record findings in §6 before M3
      implementation (D5 is signed). Read the rmcp version's source; do
      not guess from its README. **Scheduled 2026-09-29: after g5, before
      M2.** Not at M3, and the reason is that this is the only open item
      that could invalidate a *signed* §6 shape — if rmcp cannot carry a
      stable per-tool error code, §6.1 and U9's just-signed `invalid`/`io`
      both need revisiting, and M2's query surface would be built on tool
      signatures M3 cannot honour. An afternoon of reading versus a
      redesign. Four questions: is each of §6's tools declarable at all;
      do tool errors carry a branchable code or only a string; are list
      responses order-preserving through schema generation; and — added
      2026-09-30 with U16 — can the server set the `instructions` field of
      its `initialize` result through rmcp, with a per-tool `description`
      as the only other text an agent is guaranteed to read? U16's whole
      mechanism rests on that answer and it is the same kind of question as
      the other three: a shape §6 signs, resting on what rmcp can express.
      Read-only, so it can also be pulled earlier without conflicting with
      t15/t13. **Resolved 2026-09-30 — surveyed, and §6's signed shapes
      hold.** rmcp **3.5.0** was read in the published source *and* driven
      over raw stdio from a probe server under `tmp/rmcpprobe` (gitignored;
      no dependency entered the workspace, so M3's Ask-First is untouched).
      The findings live in spec **§6.3**, not here, because they are facts
      about the wire that M3 implements against: every §6 tool shape is
      declarable — one `Parameters<struct>` per tool, since
      `schema_for_input` requires a root `object`, which makes §6's
      per-tool params struct the only available shape rather than a style
      choice; **five** error carriers were measured, and
      `structuredContent.code` via `CallToolResult::structured_error` is the
      one that is both branchable and visible to the model, because rmcp
      documents that clients render protocol-level errors *opaquely*;
      **array order survives the wire but object key order does not**, since
      structured output goes through `serde_json::to_value` and rmcp does
      not enable `preserve_order`; `instructions` is settable in *both*
      handshakes (`initialize` and 2026-07-28's `server/discover`), which
      is the answer U16 rests on; and prompts are exposed. Two
      consequences are decisions rather than findings and are drafted as
      **D14** (the wire's canonical key order) and **D15** (where a §6.1
      code lives) — both gate M3, neither gates M2, which is the scheduling
      answer this item was placed here for. Three facts M3 must design
      around are also in §6.3: `serverInfo` defaults to `rmcp/3.5.0` unless
      we set our own; requests are dispatched **concurrently** (each in its
      own `tokio::spawn`, responses observed out of order), so §8's
      single-writer property needs a mutex or an actor *inside* the server
      process — the store's advisory lock is per file descriptor and one
      process holds one; and argument-shape faults arrive as rmcp's own
      unstructured text, so `invalid` covers only values we parse
      ourselves, and M3's params structs deny unknown fields.
- [x] **U3 · Collect the three sibling board dialects as fixtures.**
      Snapshot camera/ikvm-html/dacode `TODO.md` + decisions files into
      the future importer's test fixtures (§9, §10), with a short dialect
      note each — needed before M4; D7 is signed camera-first, so the
      camera fixture is the critical one. **Superseded 2026-09-29 by D12**,
      which the user signed rather than this item being executed as
      written: the **dacode** board is the only fixture, because it carries
      the deeper structure and the more complex references — the hard half
      of §9's mapping — and because `camera` and `ikvm-html` are other
      products whose board content is not to be copied into this
      repository. Those two become **live-test subjects** after conversion,
      which is a better test and a weaker pin: a live board legitimately
      changes, so no golden can hold it. §10 records the remedy if that
      proves too weak at M4 — a *synthetic* fixture hand-written here in
      the camera dialect, no copied content. Work order executed: §9,
      §10's fixture bullet, §6's `import_md` row, §14's M4 row, D7's
      banner, a new D12 dossier, and §2's M4 row below.
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
- [x] **U7 · Clarify the "six denied lints" wording.** Spec §10's gate
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
      floor, which is the safe direction. **Resolved 2026-09-29** by a
      third option neither branch named: **delete the count**. §10's
      structure was already right — state the principle, delegate the list
      to `_rust.md` — and only the number was false, so §10 now says "the
      denied lints of `.agents/roles/_rust.md`" and notes that `_rust.md`
      is the enumeration. Enumerating nine names in signed text would have
      made every future lint a spec amendment. AGENTS.md's item 1 states
      the nine for a reader who needs them there, and says why the prose
      carries no count. `_rust.md` was already accurate and did not move,
      so no `agents-sync` regeneration was needed.
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
- [x] **U10 · Decide whether compaction archives the events it drops.**
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
      the t12 dual review. **Resolved 2026-09-29 — option (a), drop as
      landed, plus "nothing calls it yet".** The reframe that settled it:
      compaction is not garbage collection of done work and does not
      necessarily free space — it writes the *entire* current graph, open
      and closed alike, into the snapshot and rewrites the log as
      `header + seed`, trading log bytes for snapshot bytes. Its purpose
      is §4.1's "load is O(log length)", and §4.1's own scale argument
      (10^3–10^5 nodes, every query a traversal) says replay is already
      cheap, so (b)'s archive sibling would buy a second retention
      mechanism plus a rotation policy for a guarantee §4.3 already
      provides. The reading in `snapshot.rs` is therefore **blessed into
      §4.3** rather than left as a crate-local argument. f14 was **split**:
      `deny_unknown_fields` on `LogHeader` lands with t13 — deserialize-
      only, so **no golden moves**, and the registry and snapshot documents
      are the precedent — while the persistent **compaction marker is
      deferred** until something actually compacts, because its only job is
      making a compacted store distinguishable from a fresh one and a
      golden-moving compatibility event spent with no beneficiary is the
      wrong trade. Consequently **t13 ships `dump`, `verify` and `repair`
      but no `compact` subcommand**; `verify` still reports the reachable
      divergence (a snapshot whose `log_offset` exceeds its log, the
      (3)–(4) window, already a named error with a re-base repair) and
      still cannot detect out-of-band snapshot deletion, which is honest
      because no compacted store can exist yet.
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
- [x] **U12 · Settle the post-fix `relates` read path and the tuple rule.**
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
      output is the consumer of (a). **Resolved 2026-09-29**, both halves:
      (a) `relates_from_arrays` **stays** — t13's `repair` is the consumer
      of its `corrupt_store` detail, and "a relates entry is a two-element
      array of task ids" is what makes that output actionable where serde's
      `invalid length 3, expected a tuple of size 2` is not;
      `snapshot.v1.golden` does not move and the verdict is recorded in
      `snapshot.rs`'s doc, which rides t13a. (b) **House rule adopted** —
      `.agents/roles/_rust.md`'s determinism section now requires
      named-field structs over tuples in any persisted schema, argued on §4
      auditability and §4.3's repair surface, explicitly not on the engine
      bug U11 fixed (so it stands with a perfect engine). `relates` is
      named there as the format's one positional shape.
- [x] **U14 · Approve `todo-cli`'s argument parser (gates t13b only).**
      The inspector needs three subcommands (`dump`/`verify`/`repair`), two
      required options (`--root`, `--slug`), three switches (`--pretty`,
      `--dry-run`, `--yes`) and `--help`/`--version`. Hand-rolling over
      `std::env::args` holds the dependency count at zero but re-implements
      the GNU conventions that are easy to get subtly wrong (`--flag=value`,
      the `--` terminator, short-flag grouping) and leaves every error
      message ours to write. Ask-First per AGENTS.md, so nine variants of
      that *same* surface were built and measured on this host (rustc
      1.98.1, cold `target/` per variant, each passing the same 13-case
      behaviour check: three happy paths, six error cases, `--help`,
      `--version`, subcommand `--help`):

      | variant | main.rs LOC | normal deps | tree MSRV | cold build | debug binary |
      |---|---|---|---|---|---|
      | hand-rolled `std::env::args` | 147 | 0 | — | 1.0 s | 531 KB |
      | `clap` 4.6.7 derive, default features | 65 | 17 | 1.85 | 7.7 s | 3.6 MB |
      | `clap` 4.6.7 derive, minimal features | 65 | 10 | 1.85 | 6.3 s | 3.4 MB |
      | `clap` 4.6.7 builder API, minimal features | 119 | 4 | 1.85 | 3.1 s | 3.3 MB |
      | `lexopt` 0.3.2 | 135 | 1 | none declared | 2.0 s | 600 KB |
      | `pico-args` 0.5.0 | 127 | 1 | none declared | 1.4 s | 573 KB |
      | `bpaf` 0.9.28 | 123 | 1 | 1.68 | 2.2 s | 1.2 MB |
      | `argh` 0.1.19, default features | 122 | 10 | 1.71 | 9.0 s | 703 KB |
      | `argh` 0.1.19, `default-features = false` | 122 | 7 | 1.71 | 5.7 s | 702 KB |

      "normal deps" is `cargo tree -e normal`, unique, transitive, root
      excluded — so `lexopt`/`pico-args`/`bpaf` are genuinely
      one-crate trees with no proc macros, while every derive option pays
      for `syn`. Maintenance, crates.io 2026-09-29: clap 4.6.7 released
      2026-09-14 (239 M recent downloads), bpaf 0.9.28 2026-09-17, argh
      0.1.19 2026-03-16, lexopt 0.3.2 2026-02-28, **pico-args 0.5.0
      2022-06-04**. Two traps found by measuring rather than reading docs:
      `argh`'s *default* features pull in `serde` + `serde_derive`, so a
      "optimized for code size" parser costs ten crates unless they are
      turned off; and clap's `#[arg(global = true)]` on a **required**
      option panics at runtime (`Global arguments cannot be required`), so
      shared required options need a flattened `Args` struct repeated per
      subcommand. **Recommendation: `lexopt`** — one dependency, zero
      transitive, no proc macros (`Cargo.lock` 26 → 27), pedantic parsing
      that rejects what it does not understand instead of guessing,
      maintained, and the ~70 lines it costs against clap-derive are the
      hand-written `--help` text plus the per-subcommand switch table, both
      of which this cli wants to own because `verify`/`repair` output is
      user-facing diagnostics. Runner-up `bpaf` if generated help is worth
      the combinator API, the 1.2 MB binary and re-declaring shared options
      inside every subcommand (it has no global-flag concept); `clap`
      derive is the shortest code and the heaviest tree; `pico-args` is
      excluded on four years without a release. Gates **t13b only** — t13a
      (the store riders) needs no dependency and proceeds regardless.
      **Resolved 2026-09-29 — `lexopt` 0.3.2**, on the recommendation
      above. It rides t13b's commit, declared once at the workspace root
      like every other dependency (U8's precedent), so `Cargo.lock` grows
      by exactly one package. One surface change came with the pick:
      **`--pretty` is dropped**. `dump`'s contract is canonical bytes —
      §8's byte-identity is what its golden pins — `todo_store::to_bytes`
      exposes no pretty writer (the engine's `to_writer_with` does, but
      reaching it would mean a second crate depending on the engine, or a
      new store API for a display concern), and a second output shape is a
      second shape to pin; `jq .` is one pipe away. The measured surface
      keeps its three subcommands, both required options and `repair`'s two
      switches, so the comparison that chose the parser is unaffected.
- [ ] **U15 · Scope or fix `Store::snapshot`'s "changes nothing on disk".**
      `crates/todo-store/src/store.rs`'s `snapshot` doc promises that "a
      failed snapshot changes nothing on disk and is **not** session-fatal".
      The second half is now pinned by t21's test; the first is falsified by
      §4.3's own discipline, which the same test measures — a rename that
      refuses leaves its pid-suffixed temp file beside the snapshot, so
      something *is* on disk that was not there before. Nothing is corrupted
      and no reader can observe it (the temp is never the snapshot path, and
      this process's next `write_temp` truncates it), so this is a doc claim
      outrunning its mechanism rather than a defect: either scope the
      sentence to "nothing in place" and name the debris, or add
      cleanup-on-failure to `write_snapshot_atomic`/`write_temp`. Both are
      `src` changes, which is why t21 — tests-only by its own acceptance
      criterion — recorded it as tracker f40 instead of fixing it. Rides the
      next store touch; the debris assertion in
      `tests/snapshot/compact.rs::a_refused_covering_snapshot_leaves_the_log_untouched`
      is the measurement to re-read first.
- [ ] **U16 · Ship base instructions with the server, and stop the harness
      using its own todo handling.** Stated by the user 2026-09-30; lands
      with M3 and needs U2's fourth question answered first. Two halves, and
      the second is the one that decides whether the server gets used at all.
      **(a) The server arrives with its own instructions.** MCP's
      `initialize` result carries an `instructions` field that clients surface
      to the model, and every tool carries a `description`; between them they
      must tell an agent that has never seen this repository how to track work
      in it — what a task/epic/decision/doc node is, which tool answers "what
      is next", that closure demands `verify` + `evidence`, that ids are stable
      and never reused, and that a blocked item needs a reason. §6 signs the
      tool list and not this text, so it is an M3 deliverable to *add*, not a
      signed shape to reinterpret. **(b) The client's native todo handling
      must be off.** A server cannot disable a client's built-in tools — it can
      only ask — so the suppression is harness-side configuration: opencode
      gates tools per agent in `.opencode/opencode.json`, which this repo
      already generates from `agent-rules.yaml`, so the native todo pair gets
      gated off for every role here. Record the limit honestly rather than
      papering over it: for a client we do not configure, two todo systems are
      visible and the only discriminator is the instruction text. §9's
      no-dogfooding-before-M3 rule still governs this repo's own tracking.
- [ ] **U17 · Decide which harness tools become MCP tools.** The intent,
      stated 2026-09-30: the tooling we use for todo handling belongs inside
      the server, so another project's agents get it over the wire instead of
      each repo growing its own scripts. Most of the mapping is already
      signed — `attention.py`'s "what is next and what is blocked" is §6's
      `next_task`/`next_items` with D9/D11's derived weight and deferred
      summary, and `progress.py`'s graph is §6.2's `graph_show` over the
      canonical serialization. What §6 does **not** carry today is the
      roll-up this page renders: totals by kind and status, and the
      open-items table with its "waits on" column. Those are queries over the
      same graph, so they are a new tool or a field on an existing one — a §6
      amendment, which is a user decision and not an implementation detail.
      Decide also what stays a script: `tasks.py` is this repo's session
      tracker (§9 bars dogfooding the server for it before M3), and the
      HTML/browser rendering is a client-side concern over `graph_show`, not a
      server feature. Answer with M2's decomposition, before the query
      surface is built — a tool that should exist is cheaper to add before
      the wire does.
- [ ] **U18 · A live view for progress, and whether the server is shared per
      user.** Deferred by the user 2026-09-30 until after the primary part is
      implemented and checked against the existing markdown boards (M4's
      importer), so it is recorded with a recommendation rather than left
      vague. Two shapes were proposed. **A local HTTP server for the page**:
      do not scan a port range — bind `127.0.0.1:0`, read the assigned port
      back, and publish it to a per-user runtime file so a second session
      attaches instead of starting another (scanning above 22000 races
      whatever else is listening, serena's own dashboard included).
      `progress.py --serve` over the file it already writes is the whole
      feature, and the default stays a static file with no listener at all.
      **A shared per-user server instance**: the right long-term shape for
      §8's single-writer model, since one process holding the store lock for
      many clients removes the multi-process contention this project has never
      actually tested — but it collides with the signed D5 stdio transport, so
      it needs either a listening transport or a thin stdio proxy in front of
      the shared instance. That is a new decision (**D13** when it is taken),
      not an implementation choice, and it belongs at M3/M5 when there is a
      wire to put in front of. Nothing here starts before M4's check.
- [ ] **U19 · One word in `repair.rs`'s module doc.** Its `Err` rationale
      closes with "so a snapshot fault is never reported through it" — true of
      the parse fault that sentence follows, false read broadly, since the
      foreign-snapshot case *does* report a snapshot-origin reconcile fault
      through `Unrepairable { offset: None }`, and
      `tests/repair/refusals.rs::an_unpositioned_fault_is_refused_for_want_of_a_truncation_point`
      pins that it does. Tighten to "so **that** fault is never reported
      through it". Found by t22's primary reviewer, classified as taste and
      dispositioned under t15's stopping rule rather than opening a fourth
      review round for one word; promoted here by the 2026-09-30 sweep,
      because a deferral recorded only in a gitignored branch-keyed tracker
      does not outlive the branch. Rides the next touch of
      `crates/todo-store/src/` — the same touch **U15** waits on, so whoever
      takes one takes both.

## 5. Debt

Deliberate shortcuts, acknowledged rather than fixed. Same id rules as §4.

- [ ] **U13 · Two store files are over the 600-line target.** Recorded as
      d2/d3 in the gitignored session tracker since t10/t12 and promoted
      here so the ledger is committed; the sweep of 2026-09-30 retired both
      tracker duplicates against this entry, so the ledger has one home and
      the two cannot drift apart again. Measured under the production-line
      rule `_rust.md` now states — a `#[cfg(test)]` block and a test-only
      file are not counted — and re-measured on `5c23f5f`:
      `todo-store/src/log.rs` **812**,
      `todo-store/src/event.rs` **772** (they read 793 and 759 before
      t13a; their own U13 exception lines are part of the growth, and the
      cap of 1000 still holds). `todo-domain/src/graph/state.rs` reads 777
      under `wc -l` and is **460** production lines, so it is under target
      and comes off the ledger; `graph/tests/mod.rs` (655),
      `graph/tests/seeded.rs` (602) and `testcodec.rs` (602) are test-only
      files and were never chargeable. No split proposed: each of the two
      is one responsibility today (schema + patch application +
      `replay_step`; append + snapshot-aware replay + `compact`), and M2's
      query layer is what will show where the seams are. Both now carry
      their module-doc exception line, landed with t13a. Revisit at M2.

## 6. Session log

Newest first. One entry per session, written for someone who was not
here. The final entry of a session is the handover
(`.agents/skills/todo-next-imp/SKILL.md` defines the shape).

### 2026-09-30 (second session, continued) — U2: the rmcp survey

The sweep closed f41 and left the handover's named step unobstructed, so
the session continued into **U2** in the same sitting. It was scheduled
here on purpose — after g5, before M2 — because it is the only open item
that could invalidate a *signed* §6 shape, and the survey is exactly that:
an afternoon of reading instead of a redesign. It ran as a measurement, not
a literature review. rmcp 3.5.0 was unpacked from crates.io and read, and a
probe server declaring `#[tool]` methods shaped like §6's was built under
gitignored `tmp/rmcpprobe` and driven over raw stdio, so every answer below
is a captured wire byte or a `file:line` in the published source. Nothing
entered the workspace: `Cargo.lock` is untouched and M3's dependency
Ask-First is still M3's to make.

**§6's shapes hold, and one of them is now a requirement rather than a
preference.** The macros take parameters only as a single
`Parameters<struct>` wrapper, because `schema_for_input` rejects any
generated schema whose root is not `type: "object"` — so §6's "every tool
takes a repository parameter, epic is an optional filter" lands as one
named-field params struct per tool, which is also what `_rust.md`'s
persisted-schema rule already asks for. Doc comments become the tool
description *and* each field's, which is the second half of U16's
mechanism. `instructions` came back on the wire from `get_info()` in the
`initialize` result, and again in the 2026-07-28 flow's `server/discover`
result — so U16 rests on something measured now rather than hoped for.
Prompts are exposed. Arrays keep their order exactly, which is the property
§5.2's total order actually depends on.

**Two answers are decisions, and they are drafted rather than taken.**
Object *key* order does not survive: `Json<T>` is serialized by rmcp's own
`serde_json` into `structured_content`, and serde_json's `Map` is a
`BTreeMap` without the `preserve_order` feature. The same probe source
built two ways produced `{"alpha":…,"middle":…,"zebra":…}` and
`{"zebra":…,"middle":…,"alpha":…}` from a struct declaring
`zebra, middle, alpha`. §6.2 signs "fixed field order", so either the flag
is enabled here, the dump travels as a text block from our own codec, or
§6.2/§8 are amended to name two canonical orders — **D14**. The second is
sharper, because it is the one that would have shipped a defect: rmcp's
*default* tool-error path is `Err(String)`, which reaches the client as
`isError` plus a text block and nothing else. An M3 built the obvious way
would have satisfied "return an error" while violating §6's "never bare
strings", and no test in this repo would have caught it. Five carriers were
measured; `structuredContent.code` via `structured_error` is the one that
is both branchable and visible to the model, since rmcp documents that
protocol-level errors are rendered opaquely — **D15**.

**Three facts M3 has to design around are in §6.3 with their citations**,
and two were surprises: `serverInfo` defaults to the SDK's own
`rmcp/3.5.0` identity unless we set one, and requests are dispatched
concurrently — each in its own `tokio::spawn`, with responses arriving out
of request order in two separate captures. §8's single writer is a
*process*-boundary property, so inside one server process concurrent tool
calls reach one store handle simultaneously and M3 must serialize them
itself; the store's advisory lock cannot, because it is per file descriptor
and one process holds one. The third is that argument-shape faults (a wrong
type, a missing required field) are turned into rmcp's own unstructured
text error before our code runs, so §6.1's `invalid` covers malformed
*values* we parse ourselves and nothing else — and unknown argument fields
are silently ignored unless the params struct denies them, which the probe
confirmed both ways.

**Handover** — supersedes the one in the sweep entry below.

```
Workspace:    main, clean at 8135d06 plus its hash backfill; **M1 CLOSED**,
              phase M2-next. No Rust has moved since 7563fff (t21) —
              everything after it is harness, board and spec: 1581007,
              6f96e01, bf34068, d542065, 5c23f5f, 95a83eb + 02475d0 (the
              f41 sweep) and 8135d06 (spec §6/§6.3/§12, decisions.md
              D14/D15, this board). Four
              crates: todo-domain 111 tests, todo-store 107 (four goldens),
              todo-cli 35 (one golden), todo-mcp an M0 shell. rustc 1.98.1,
              MSRV 1.95, Cargo.lock UNTOUCHED by the survey. Deps unchanged
              (serde, dacode-json + zmij by path on ../rust/json-tiers at
              f7f26f8, tempfile dev-only, lexopt 0.3.2); tools/ need Python 3
              + PyYAML. Serena live. Nothing to install.
Landed:       **U2** — the rmcp 3.5.0 survey as spec §6.3 (facts, with
              source citations and the measured wire shapes), §12 items 14
              and 15, and the D14/D15 dossiers in decisions.md with their
              summary rows and action lines. Board §1/§2/§3/§4(U2 struck)
              moved in the same commit. Earlier this session: f41's sweep
              (95a83eb), which closed f31–f34, f36–f39, f42, f13, d2, d3 and
              f41, promoted f35 to U19, opened f44, and landed the sweep line
              in the exit gate. Probe artifacts live in gitignored
              tmp/rmcpprobe (vendored rmcp 3.5.0 source, the probe crate,
              drive.py, six captures) and are NOT committed — the Ask-First
              for rmcp is M3's.
Next step:    **Decompose M2**, the query layer: repo/epic selection,
              dependency resolution, deterministic next_task, registry with
              path updates (§2's M2 row, spec §5.2 + §6). U2 no longer
              obstructs it and D14/D15 gate M3, not M2 — the survey's whole
              scheduling point. Answer **U17** inside the decomposition (the
              totals/open-items roll-up as a new tool or a field: a §6
              amendment, so a user decision), and re-measure **d1**
              (Graph::tasks_of_epic's linear scan) and **U13** (the file-size
              ledger) there, since M2's queries are what would make either
              hot. M2's own first task is the obvious one: §5.2's order is
              already a derivation, and next_task/next_items are the signed
              selection over it (D9/D11). Alternative if you prefer to clear
              the deck first: vote D14/D15, which are short and both carry a
              recommendation.
Blocked on you: **D14** (the wire's canonical JSON key order — (a) enable
              serde_json/preserve_order here, (b) a text-carried canonical
              dump, (c) amend §6.2/§8 to name two orders; recommendation (a)
              plus a wire golden whichever wins) and **D15** (§6.1's code
              carrier — (a) structuredContent.code with an explicit
              outputSchema, (b) a tagged-union output type, (c) _meta;
              recommendation (a)). Both gate M3 only. **f44** is the smaller
              call: land a duplicate-key check in agents-sync.py or promote
              it. Still yours from before: U13's acknowledgement, U15/U19's
              deferral to the next store touch, f25's veto-able reading
              (4c5782b is the commit to revert), and two sibling-repo calls
              (a cargo fmt --check gate for dacode-json; whether to benchmark
              c4aebe9). D13 stays reserved for U18's transport question and
              is deliberately not drafted.
Unverified:   **the survey's probe is not a smoke test of our server** — it
              proves what rmcp 3.5.0 does with hand-written tools, not what
              todo-mcp will do; no MCP client (opencode included) has been
              pointed at anything, so U16's claims about a client surfacing
              `instructions` and about gating the native todo tools off are
              still unexercised config. D14's key-order claim is measured on
              a probe struct, not on a real graph_show payload, and the
              preserve_order build proves the mechanism but not that no other
              dependency in a real M3 tree would disable or re-unify it.
              The concurrency finding is two out-of-order captures plus the
              spawn site in source — no stress test, no measurement of how
              much serialization M3 needs. Nothing in §6.3 pins a version
              range: the crates.io index lists 60 non-yanked stable versions
              of rmcp up to 3.5.0 and no release dates were checked, so
              "current" here means current on 2026-09-30, and a later
              version could change any of it.
              Carried forward and still true: no induced crash or power loss
              anywhere; nothing calls compact() outside tests; no human has
              run dump/verify/repair against a real store; multi-process lock
              contention is still two handles in one process; the sweep line
              landed this session and has never been exercised by a
              milestone gate; tools/progress.py has no test of its own.
Tracker:      tools/tasks.py check exits 0 — g3 standing; t24 (U2) done with
              evidence; open findings f4 (M3's wire), f25 (parked
              diagnostics), f35 (= U19), f40 (= U15), f44; one debt, d1,
              with M2 named as its consumer. pitfalls-live.md is still EMPTY
              — next entry P-LIVE-007. Doc gates green: check-links,
              agents-sync --check, agents-index --check, attention --check
              (183 checks, and D14/D15 rendering as the two P1 vote items).
              Code gates skipped by construction: no Rust moved.
```

### 2026-09-30 (second session) — the tracker sweep (f41)

The session opened on the `/todo-next-imp` report and one question from the
user: why was t13 still open. It was not — the previous session had
corrected it to `done` with evidence naming all three of its commits — but
the question was the right one, because the item that *was* still open is
the finding that explains it. **f41** recorded that M1's gate closed without
a status sweep, and that `tasks.py check` is blind to the class: it flags a
`done` item with no evidence and a `blocked` item with no reason, and cannot
see a `pending` item whose work is already in git history. Its second half
had never landed, so the exit gate still carried no sweep line.

Both halves ran. The rule landed in the binding home (`todo-next-imp`'s
exit-gate step 2) and the explanatory one (`task-tracking.md`'s "`check` is
the gate" section), and the sweep was then applied to the **whole** open list
rather than only to the nine findings f41 names — which is how it earned its
keep in the same session it was written: it found four more items whose
status did not match reality, among them a config fix that had landed in the
previous commit (`d542065`) and was still recorded as pending.

**The judgment worth recording is what the sweep did *not* close.** f41's
verify offers two dispositions — `done` with evidence, or `wontfix` with a
note — and neither is honest for a finding whose remedy has not landed: f35
is a one-word tightening of `repair.rs`'s module doc still waiting for the
next touch of that file. Shortening the open list is precisely the temptation
a sweep exists to resist, so f35 is promoted to **U19** instead (the
f40→U15 precedent — a deferral recorded only in a gitignored branch-keyed
tracker does not outlive the branch) and stays open. The same reasoning left
f4, f25, f40 and d1 open with notes naming the consumer each waits on, so a
future sweep can tell a deliberate open from a forgotten one. The two
size-ledger debts went the other way and were *closed*: their verify moment
was t13's closure, it has passed, and their figures were re-measured on the
committed tree before being retired against **U13**, which is now the
ledger's only home. Every close is measured rather than inferred — a
`file:line` per claim and the pins re-run — because the alternative is the
defect class this project keeps meeting: a prose claim that does not
reproduce.

**Handover**

```
Workspace:    main, clean; **M1 CLOSED**, phase M2-next. No
              Rust has moved since 7563fff (t21) — everything after it is
              harness and board: 1581007, 6f96e01, bf34068, d542065,
              5c23f5f and 95a83eb (SKILL.md's exit-gate step 2,
              task-tracking.md, progress.py's lane/edge/wait tables, this
              board). Four crates: todo-domain 111
              tests, todo-store 107 (four goldens), todo-cli 35 (one
              golden), todo-mcp an M0 shell. rustc 1.98.1, MSRV 1.95. Deps
              unchanged (serde, dacode-json + zmij by path on
              ../rust/json-tiers at f7f26f8, tempfile dev-only, lexopt
              0.3.2); tools/ need Python 3 + PyYAML. Serena live. Nothing
              to install.
Landed:       f41 — the tracker sweep and the rule that binds it. Closed:
              f31, f32, f33, f34, f36, f37, f38, f39 (each with evidence
              measured on 5c23f5f: file:line plus the pins re-run —
              todo-store --test repair green, todo-cli green, f38's two
              filters each selecting exactly one test), f42 (its
              duplicate-key verify re-run: no duplicate key anywhere in
              agent-rules.yaml, commands and build_commands both resolve,
              agents-sync --check exit 0), f13 (wontfix — its own text
              already said "accepted, not fixed"), d2 and d3 (their verify
              moment was t13's closure; re-measured with _rust.md's awk on
              this tree and retired against board U13), and f41 itself.
              Opened: U19 (promoted from f35, the one finding whose remedy
              has not landed) and f44 in the tracker. Notes added, statuses
              unchanged: f4 (M3's wire), f25 (the parked diagnostics
              boundary), f40 (= U15), d1 (M2's queries). progress.py's data
              tables were refreshed in the same commit, because the sweep
              moved the state that page renders — U19 and f44 drawn, the
              f31–f39 group no longer labelled "undispositioned", the WAITS
              rows of now-closed items removed; re-run twice byte-identical
              with no NOT DRAWN advisory and its totals cross-checked against
              tasks.py show.
Next step:    **U2 — survey rmcp against §6**, unchanged by this session and
              now unobstructed: it is the handover's named step, scheduled
              after g5 and before M2 by user decision, because it is the only
              open item that could invalidate a *signed* §6 shape and
              therefore U9's invalid/io codes. Read the version's source, not
              its README, and probe in a scratch crate under tmp/ (the U14
              argprobe pattern) so no dependency lands without an Ask-First.
              FOUR questions: is each §6 tool declarable; do tool errors carry
              a branchable code or only a string; are list responses
              order-preserving through schema generation; can the server set
              the initialize result's `instructions` (U16 rests on it) — and
              whether it exposes `prompts` is worth a fifth. Record findings
              in §6 and strike U2; a shape that turns out not to be
              honourable is a new D-entry, not an edit. Then decompose M2 and
              answer U17 inside that decomposition, where d1 and U13 both
              want a re-measurement.
Blocked on you: nothing. f44 is the one new proposal: f42's generalisation
              ("any config this harness calls a single source of truth needs
              a duplicate-key check") is still preventive work nobody has
              landed — agents-sync.py loads agent-rules.yaml with plain
              yaml.safe_load, so a second top-level key would silently shadow
              the first again. Either land the check in agents-sync.py or
              promote f44 to a U id; it is recorded in the tracker either
              way. Still yours from before: U13's acknowledgement, U15/U19's
              deferral to the next store touch, f25's veto-able reading
              (4c5782b is the commit to revert), and two sibling-repo calls
              (a cargo fmt --check gate for dacode-json; whether to benchmark
              c4aebe9).
Unverified:   **the sweep line has never been exercised by a milestone
              gate** — it landed in this session and M2's gate is the first
              one that will run under it, so whether it prevents the class or
              merely documents it is unproven. No independent review of this
              diff: harness and board are review-exempt, so the only checks
              are the ones named here. Every code gate is `skipped` by
              construction (no Rust moved); the test runs cited above are
              verification of claims about the committed tree, not a gate on
              a change. Carried forward and still true: no induced crash or
              power loss anywhere; nothing calls compact() outside tests, so
              RepairOutcome::Diverged is reachable only from a fixture; no
              human has run dump/verify/repair against a real store (35 tests
              have); MCP smoke awaits M3, and so does every claim in U16;
              multi-process lock contention is still two handles in one
              process (U18's gap); #[cfg(unix)] permission tests vanish
              rather than pass off-platform and t21's EISDIR fault has an
              unproven Windows spelling; snapshot durability without a
              directory fsync is reasoned (f16), not measured;
              tools/progress.py has no test of its own.
Tracker:      tools/tasks.py check exits 0 — g3 standing; one debt open (d1,
              whose note names M2's queries as its consumer) after d2/d3 were
              retired against board U13. Open findings: f4, f25, f35 (= U19),
              f40 (= U15), f44.
              .agents/pitfalls-live.md is still EMPTY — the next entry is
              P-LIVE-007 and a promoted id is never reused. Doc gates green:
              check-links, agents-sync --check, agents-index --check
              (AGENTS.md untouched, so no backfill is owed), attention
              --check.
```

### 2026-09-30 — t21 induced the step order; then g5 closed M1

The session opened at the t20 handover's named next step and needed no
dispatch questions: t21 is one test in one file, so it was written here
rather than spawned — but it was still dual-reviewed, because the trailers
are conditional on a review of these exact bytes and not on the size of the
diff.

**The fault seam worked, and the step order is externally observable for the
first time.** f19's technique is a directory parked at `<log>.snapshot`:
`rename(2)` refuses to replace a directory with a file, so `compact()`'s step
(1) fails on demand, with no production seam and no `src` change. What that
buys is the property the two window tests could not reach — they build the
(3)–(4) state from real bytes and therefore pin what the window *leaves*,
not the order that produced it; reversing the five steps left all 31 of them
green, which is precisely f19's complaint. The new test pins the order twice
over: the log is byte-for-byte unchanged, and an append on the *same handle*
still lands in it — `Store::snapshot`'s "not session-fatal" promise extended
to the compaction path and covered nowhere else. Both halves were proven
live by mutation rather than asserted: reversing the order fails the first,
and with the first deleted the reversal fails the second. A third mutation,
the debris expectation changed to zero, fails with `left: 1` and turns "a
refused rename leaves its temp file" from prose into a measurement.

**The recurring defect class appeared twice more, once in the diff and once
in the narrative about it.** Round one: the comment beside the `io`
assertion claimed the error's context distinguishes step (1)'s rename from
step (4)'s. It does not — both pass the subject `"the snapshot"`, so the two
details are byte-identical, and the reviewers' own mutation run falsified the
claim empirically (the detail assertions passed under the reversal; only the
byte assertion caught it). The fix was not a re-wording of the claim but a
statement of what the detail *can* discriminate — the snapshot's rename
versus step (3)'s `"the compacted log"` — and of what places the refusal
before step (3) instead, which is the untouched log. Round two: the
re-review briefing said an out-of-scope observation had been "recorded as
tracker finding f40" when no f40 existed. Same class, one level up, in the
text describing the artifact rather than in the artifact; both reviewers
refused to take it on word and blocked on it, which is the gate working. f40
now exists and is promoted here to **U15**, because the tracker is
gitignored and branch-keyed, so a deferral recorded only there does not
outlive the branch.

With t21 in, M1's code was complete, so the session continued straight into
**g5, the exit gate** — a gate rather than a task, which is why it produced
no new code and one substantial piece of writing. Every line of its verify
method ran green on the committed tree (253 workspace tests, five goldens
byte-identical under `UPDATE_GOLDEN=1`, `cargo doc --workspace --no-deps`
clean, Serena resolving a `todo-domain` symbol, all four doc gates green),
and the consolidation it owed is now committed: `.agents/pitfalls.md` went
from deliberately empty to fifteen entries plus a distilled top ten, and
`.agents/pitfalls-live.md` reset with a former-id table, because
`P-LIVE-002`/`-004`/`-005` are cited by id in test-file module docs across
both crates and a reset that dropped those ids would have left a dozen
in-tree citations dangling.

**Two of the fifteen are the same defect seen from both sides, and they are
why M1 cost what it did.** 2.1 (a multi-file write sequence's order *is* the
safety argument, and a test that constructs the crash's aftermath cannot see
it) and 5.1 (a spec's safety argument is a claim to falsify against the state
space, not ground truth to instance-check) are t12's two silent corners and
t21's fix — one from the spec author's chair, one from the reviewer's. 5.2
generalizes the class this session hit twice more: a prose claim (a line
count, a mutation count, a review-process fact, a tracker id that does not
exist) is an artifact like any other, and the reliable fix is deleting the
number and pointing at the pin, not restating it more carefully. Two rules
consequently moved out of the knowledge base and into the role layers, where
a coding agent meets them at write time rather than at recall time:
`_rust.md`'s comment section carries "no measurement in a source doc
comment", and `_search.md`'s Serena specifics carry the impl-node name path
this gate measured the hard way — `/Graph/raise_counters_to` resolves to
nothing, while the bare name resolves as `impl Graph/raise_counters_to`.

**Then the user asked for the shape of the work as a page**, which is how
`tools/progress.py` came to exist: a dependency graph over the tracker and
board, totals with their denominators, and an open-items table saying what
each row waits on. It was written as a generator rather than typed out,
because a hand-written count in a report is the same defect class 5.2
describes — and it earned its place immediately, twice over. Reading the
tracker closely enough to draw it surfaced two statuses that were stale in
the direction that *hides* work: t13 still `pending` after all three of its
commits had landed, and t17 still `blocked` on a dependency the user had
resolved two days earlier and t20 had consumed. `tasks.py check` cannot see
either, because it flags a `done` item with no evidence and a `blocked` item
with no reason — not a `pending` item whose work is already in git history.
Both corrected; **f41** records the class, along with the nine
undispositioned t20/t22 spec-error records whose consumers have landed.
Promoting the scratch script into a tool then found **f42**: two top-level
`commands:` keys in `agent-rules.yaml`, the first silently shadowed by the
second, so the entire build-command block was dead config that read like live
config.

**Three requirements arrived with that request and are recorded rather than
implemented**, because two belong to M3 and one is a decision. **U16**: the
server ships its own base instructions — MCP's `instructions` field plus the
tool descriptions — and the harness stops using its client's native todo
handling, with the honest limit stated rather than papered over: a server
cannot disable a client's built-in tools, it can only ask, so the suppression
is *our* configuration and every other client gets the instruction text.
**U17**: which of the harness's Python tools become MCP tools, the intent
being that the todo tooling lives inside the server; most of that mapping is
already signed (`attention.py` → `next_task`/`next_items`, `progress.py`'s
graph → §6.2's `graph_show`) and what is not is a §6 amendment, i.e. a user
decision. **U18**: the live view — a local HTTP server for the page, or a
shared per-user server instance that sessions attach to — deferred by the
user until after M4's importer check, and recorded with its recommendation
(bind port 0 and publish it rather than scanning a range; `--serve` over the
file already written; the shared-instance shape is a new decision because it
collides with the signed D5 stdio transport). U2 gained the fourth question
U16 depends on.

**Handover**

```
Workspace:    main, clean; **M1 CLOSED** (board §2), phase M2-next. The last
              code commit is 7563fff (t21, tests-only); everything after it
              is harness and board — 673a49f (hash backfill), 1581007 (the
              pitfalls consolidation plus two role rules, version.custom 7),
              6f96e01 (board: close M1), bf34068 (index backfill) and this
              commit, which adds tools/progress.py, fixes f42 and carries
              version.custom 8. Four crates: todo-domain (111 tests),
              todo-store (107, four goldens), todo-cli (35, one golden) and
              todo-mcp still an M0 shell. rustc 1.98.1, MSRV 1.95, 253
              workspace tests, and no Rust has moved since the g5 gate ran
              (`git diff 7563fff..HEAD -- '*.rs'` is empty). Deps unchanged:
              serde, dacode-json + zmij (path dep on
              /Users/mouse/RPM/dacode/rust/json-tiers — a cold machine needs
              that checkout beside this repo's parent, at f7f26f8), tempfile
              dev-only, lexopt 0.3.2. Serena's semantic layer is live; tools/
              need Python 3 + PyYAML. Nothing to install.
Landed:       t21 (7563fff), the g5 M1 exit gate (1581007 consolidation,
              6f96e01 board, bf34068 backfill), and tools/progress.py with
              the f42 config fix. g5's whole verify line ran green on the
              code tree: fmt clean; check --workspace --all-targets
              0 warnings; test --workspace 253 summed over every result line
              (domain 111, store 107, cli 35); clippy --workspace
              --all-targets 0 warnings; cargo doc --workspace --no-deps
              0 warnings; all FIVE goldens byte-identical under
              UPDATE_GOLDEN=1 (twelve store writers plus cli_dump.v1); Serena
              resolving impl Graph/raise_counters_to in todo-domain; the four
              doc gates green. Closed: f19, f20, f24, f30, t13 and t17 (both
              stale statuses), t21, t23, g5 and g4 (M1). Opened: f40/U15,
              f41, f42, U16, U17, U18. `tools/progress.py --open` renders
              the picture; its machine-readable twin is
              .agents/cache/progress.json.
Next step:    **U2 — survey rmcp's tool macros against §6**, reading the
              version's source rather than its README, then decompose M2.
              U2 sits here by decision and not before M3: it is the only
              open item that could invalidate a *signed* §6 shape, and
              therefore U9's invalid/io codes, so finding that after M2's
              query surface is built would be the expensive order. Its FOUR
              questions: is each of §6's tools declarable at all; do tool
              errors carry a branchable code or only a string; are list
              responses order-preserving through schema generation; and can
              the server set the initialize result's `instructions` field,
              which is what U16 rests on. Probe in a scratch crate under
              tmp/ (the U14 argprobe pattern) so no dependency lands in the
              workspace without an Ask-First. Record the findings in §6 and
              strike U2; if a signed shape turns out not to be honourable
              that is a new D-entry for the user, not an edit. Then
              decompose M2 and answer U17 inside that decomposition — a tool
              that should exist is cheaper to add before the wire does. M2's
              own first task is the obvious one, since §5.2's order is
              already a derivation and next_task/next_items are the selection
              over it, both signed (D9/D11).
Blocked on you: nothing right now. Three decisions are coming and are
              recorded so they are not a surprise: **U17**'s §6 amendment
              (the totals/open-items roll-up as a new tool or a field on an
              existing one), **U18/D13**'s transport question (a shared
              per-user server instance versus the signed D5 stdio), and
              **U16**'s instruction text at M3. Still open from before:
              U13's acknowledgement, U15's deferral, the three P3 tracker
              debts, and f25's veto-able reading (a narrow §4.3 recovery
              primitive is not the parked diagnostics surface — 4c5782b is
              the commit to revert if you disagree). Two sibling-repo calls
              also remain yours: whether dacode-json gains a
              cargo fmt --check gate, and whether to benchmark c4aebe9.
Unverified:   **still no induced crash or power loss anywhere.** t21 injects
              the first fault from outside the crate's own logic — a rename
              refusal — but it is not a crash: no process has been killed
              mid-write, so the (3)–(4) window and the two corner states are
              still constructed from real bytes rather than produced by an
              interrupted run. Nothing has called compact() outside tests, so
              RepairOutcome::Diverged is reachable only from a fixture. No
              human has run dump/verify/repair against a real store — 35
              tests have. MCP smoke awaits M3, and so does every claim in
              U16: there is no wire yet, so no client has ever read an
              `instructions` field from this server, and the native-todo
              suppression is unexercised config rather than behaviour.
              Per-crate lint tripwires for todo-mcp and todo-cli have never
              been fired. Multi-process lock contention is still two handles
              in one process — exactly the gap U18's shared-instance shape
              would close. The #[cfg(unix)] permission tests vanish rather
              than pass off-platform; t21's fault carries no such guard, and
              while EISDIR for a file-over-directory rename is POSIX, its
              Windows spelling is unproven — which is why the test pins the
              crate's context string and deliberately not the ErrorKind.
              Snapshot durability without a directory fsync is reasoned
              (f16), not measured. **tools/progress.py has no test of its
              own**: it is harness Python verified by running it (two runs at
              one HEAD byte-identical, its totals cross-checked against
              `tasks.py show`, `--json` parsed, `--out` exercised outside the
              repo), and its empty-source and dropped-edge guards are
              invariants it enforces on itself, not tests someone runs.
Tracker:      tools/tasks.py check exits 0. g3 standing (the project goal,
              open until M5); g1/g2/g4/g5 closed; d1/d2/d3 acknowledged
              against the production-line ledger (log.rs 812, event.rs 772,
              state.rs 460 and off it); f4 (M3's wire rejection), f13
              (accepted), f25 (parked diagnostics), f31–f39 (awaiting f41's
              disposition pass), f40 (= U15), f41 and f42 recorded;
              f19/f20/f24/f30 closed. `.agents/pitfalls-live.md` is EMPTY —
              the next entry is P-LIVE-007 and a promoted id is never reused.
              Doc gates green except `agents-index --check`, which is stale
              inside the delivery commit by construction (AGENTS.md moved)
              and is backfilled by the next one — pitfalls.md 5.3.
```

### 2026-09-29 (second session) — the file budget, t13a, U14's measurement, t22

The session opened at the t15 handover's single next step (t13) and the
user answered the nine dispatch questions in one message, which changed the
shape of the work twice. Item 9 turned into a rule change — the file budget
counts **production lines**, so `graph/state.rs` went from 777 to 460 and
off the ledger, and over-target became a *recorded* exception rather than a
silent one (`8787a72`, index backfill `0ab91c2`). Item 7 was not a decision
but a request to measure, so nine parsers were built against one identical
surface before recommending any of them; the table is U14, and the two
traps in it (`argh`'s default features pulling `serde`; clap's
`global = true` + required panicking at runtime) were found by compiling,
not by reading docs. Item 3 — "leave it open, due to log backend" — parked
the diagnostics question as f25 and is the reason t13 grew a store commit:
§4.3's repair truncates, truncating is a write, and §8's lock and the log's
line discipline both live in `todo-store`, so a cli that re-derived them
would be a second implementation of the single-writer protocol. t22 landed
that primitive (`4c5782b`, hash backfill `a1d2e86`), and its review found
the session's sharpest defect: the guard standing between "refuse" and §4's
silent event loss had **no test**, and deleting it left the suite green
while a store truncated to its header-plus-seed prefix and reopened as a
different graph. t13a (`ffa4be8`) closed six riders in between.

The recurring failure class was prose, three times running: a board entry
asserting that both reviewers caught something when one filed it as a low;
a mutation count that did not match the mutations named beside it; a
module-doc rationale resting on a half-true premise about where snapshot
offsets come from (f34, where the implementer corrected the reviewer and
both confirmed the correction). Each fix was deleting the claim or
restating it as substance, never re-counting — which is t15's lesson and
f20's, and why f30 (the rule lives in task specs, not in `_rust.md`) is
routed to g5 rather than patched mid-commit.

**Handover**

```
Workspace:    main, clean; the last code commit is e5a9955 (t20, the cli)
              and this handover's lines were refreshed after it landed. Four crates: todo-domain (111 tests),
              todo-store (106 tests, four goldens, six modules + repair.rs),
              todo-cli (35 tests, dump/verify/repair) and todo-mcp still
              an M0 shell. rustc 1.98.1, MSRV
              1.95, 217 workspace tests. Deps: serde, dacode-json + zmij
              (path dep on /Users/mouse/RPM/dacode/rust/json-tiers — a cold
              machine needs that checkout beside this repo's parent, at
              f7f26f8), tempfile dev-only, lexopt 0.3.2 (U14, in
              `Cargo.lock` since e5a9955). Five goldens now: the store's
              four plus `cli_dump.v1`. Serena's semantic layer is live.
              Nothing to install.
Landed:       six commits — 8787a72 (production-line budget + the
              named-field-structs house rule + U13/U14 opened), 0ab91c2
              (index backfill), ffa4be8 (t13a: f14/f17/f18/f22-store/f23,
              U12(a) recorded in-tree, U13's exception lines), 934745f
              (U14 resolved: lexopt, --pretty dropped), 4c5782b (t22:
              DataRoot::repair + RepairOutcome/TornTail), a1d2e86 (hash
              backfill). Closed: U12, U14, f14, f17, f18, f21, f22, f23,
              f26-f29, t18, t19, t22. Opened: f25 (parked diagnostics),
              f30-f35.
Next step:    t20 LANDED (e5a9955), so M1 has one code task left: t21 —
              f19's order-sensitive compact() test, tests-only, no spec
              needed beyond its tracker verify line (park a DIRECTORY at
              <log>.snapshot so step (1)'s rename fails; assert compact()
              errors with the log byte-unchanged; non-vacuity by reversing
              the step order). Dispatch it the proven way: short spawn
              prompt (P-LIVE-001), control-spawn retry on Connection reset,
              target selection not name filters (P-LIVE-005), inline mod
              wrapper (P-LIVE-002/004). Then the g5 M1 exit gate, whose
              verify line now includes cargo doc --workspace --no-deps and
              FIVE goldens regenerating byte-identical under
              UPDATE_GOLDEN=1 (the cli's included), and which consolidates
              f20/f24/f30 into .agents/pitfalls.md. The alternative to t20 is
              U2 (the rmcp survey), which is scheduled after g5 and before
              M2 on purpose — it is the only open item that could
              invalidate a signed §6 shape, and therefore U9's new
              invalid/io codes.
Blocked on you: nothing. U2's scheduling, U13's acknowledgement and the
              three P3 tracker debts are the only attention items left, and
              f25 carries one veto-able reading (a narrow §4.3 recovery
              primitive is not the parked diagnostics surface) — if you
              disagree, 4c5782b is the commit to revert. Two sibling-repo
              calls also remain yours: whether dacode-json gains a
              cargo fmt --check gate, and whether to benchmark c4aebe9.
Unverified:   no induced crash or power loss anywhere — t22's write path is
              reasoned against crash points and tested against hand-built
              on-disk states, and compact()'s step order is still not
              externally observable without a fault seam (f19/t21 names the
              technique). Nothing has called compact() outside tests, so
              RepairOutcome::Diverged is reachable only from a fixture.
              The cli exists and its golden is pinned, but no human has run
              dump/verify/repair against a real store yet — 35 tests do. MCP smoke
              awaits M3. Per-crate lint tripwires for todo-mcp/todo-cli have
              never been fired. Multi-process lock contention is still two
              handles in one process. The #[cfg(unix)] permission tests
              vanish rather than pass off-platform. Snapshot durability
              without a directory fsync is reasoned (f16), not measured.
              lexopt was measured against a 13-case behaviour table in a
              gitignored probe (tmp/argprobe), not yet against the real cli.
Tracker:      tools/tasks.py check exits 0. g3/g4 standing; d1/d2/d3
              acknowledged with a corrected production-line ledger (log.rs
              812, event.rs 772, state.rs 460 and off it); f4 (M3), f13
              (accepted), f20/f24/f30 (g5's pitfalls consolidation), f25
              (parked), f31-f35 recorded. Doc gates: check-links ok
              (11 links / 10 files), agents-sync --check and agents-index
              --check up to date, attention.py --check exit 0.
```

### 2026-09-29 — t15 landed; U11 fixed upstream; U7/U9/U10/U3(D12)/U2/t11 settled

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

**Then the rest of the backlog was cleared in one exchange**, and two of
the five answers were not the ones proposed. **U10** went to (a), but only
after the premise was corrected: the question had been framed as "how
should compaction archive what it drops", and compaction does not drop
*done* work or necessarily free space at all — it writes the whole current
graph, open and closed alike, into the snapshot and trades log bytes for
snapshot bytes, its purpose being §4.1's bounded replay. Since §4.1's own
scale argument says replay is already cheap at 10^3–10^5 nodes, the answer
became (a) *plus nothing calls it yet*: f14 split so that
`deny_unknown_fields` rides t13 (deserialize-only, no golden moves) while
the compaction marker waits for a beneficiary, and **t13 ships no
`compact` subcommand**. A compatibility event spent with nothing to
protect is the wrong trade, and (a) forecloses nothing.

**U3 was not executed as written — it was overruled, correctly.** The
recommendation was to keep D7's camera-first order because a fixture has
to be stable and the importer's first target needs a pinned golden. The
user's answer added a constraint that was not on the table: `camera` and
`ikvm-html` are **other products**, and their board content is not to be
copied into this repository at all. That makes the live-test route the
only available one for them, and their technical reasoning is better than
the familiarity argument D7 was signed on — `dacode`'s board carries the
deeper structure and the more complex references, which is the hard half
of §9's mapping (D10's `references`, `#anchor` citations, decision→work
edges), while `camera`'s more recent *manners* are what the mapping table
absorbs cheaply. So **D12** reverses D7's order and records the cost
plainly instead of hiding it: two of three dialects get no pinned golden,
their correctness argued by a live run a human inspects, which is weaker
and non-reproducible; the named remedy at M4 is a *synthetic* fixture
hand-written here in the camera dialect, not committing their boards.
**Lesson for recommendations: a stability argument is not a rebuttal to a
boundary argument.** The fixture rationale was sound and irrelevant once
the content was not ours to copy.

**U7** took a third option neither branch on the board had named — delete
the count from §10 rather than enumerate nine lint names in signed text,
because `_rust.md` is already the enumeration and a tenth lint should not
require a spec amendment. **U2** is scheduled after g5 and before M2,
which is earlier than "before M3" and deliberate: it is the only open item
that could invalidate a signed §6 shape, so discovering it after M2's
query surface is built would be the expensive order. **t11's** narrow
registry reading is accepted and its veto window closes; the M3 revisit is
to be driven by a measurement of cold `repo_list` rather than a preference,
following U1's precedent where D1 was signed without the benchmark and the
numbers became decision material only if the fallback was proposed.

**With the backlog cleared, t15 ran** — and it is the clearest case yet
that a review gate earns its cost on a *small* change. It was scoped as a
doc-and-test tidy-up: extend the seeded invariant scan to decisions and
docs (f9), fix four unresolved intra-doc links and a stale module-doc line
(f10), and realign `error.rs` to the amended §6.1 (f22's domain half).
Extending `assert_invariants` took two assertions. **Those two assertions
would have been a vacuous guard**: the generator only ever forged a node's
`.repo` field, never its id's slug, so nothing the scan could reach ever
violated the new assertions. The fix was to widen the generator — a `Forge`
enum drawing `None`/`Repo`/`Id` off the same single `below(10)` in the same
stream position — and then to pin the coverage it buys, because a pin
nobody checks is the same mistake one level up: all four insert kinds ×
both halves must actually occur across the seeds.

The primary reviewer then found the hole in *that*: the pin keyed on the
trace label, which came from the same `Forge` value that drove the arm, so
label and effect could decouple. `Forge::Repo => epic.id =
other_epic_id(number)` leaves the epic repo-half with no attacker at all
while its cell stays green. Deriving the traced half from the **built
node** (`scope_half`) closes it, and that mutation now fails naming exactly
the coverage it lost. Every claim in this paragraph was established by
mutation rather than by reading: making `in_scope` ignore the id's slug
fails both seeded tests, starving either draw arm fails the pin, and the
four decoupling mutations the reviewer named all die under the
node-derived form.

The recurring defect was mine rather than the code's, and it took three
attempts to stop making it: **three successive versions of one doc
comment stated a measurement that did not reproduce** — first the wrong ratio, then figures
scoped to the wrong runs, then a total that was one run's. The primary
re-derived each with instrumented worktree pairs to check me. The fix was
not a fourth attempt at the number but deleting the numbers: the comment
now states the mechanism and points at the pin that enforces the property,
because a figure in a source comment is a claim nobody re-measures. That is
the same lesson as f20 in a different register — an unverified assertion
embedded in an artifact, trusted because it was written down — and it is
worth carrying into g5's `.agents/pitfalls.md` consolidation beside it.

**Handover.** Workspace: `main` at `0a23350`, clean.
Sibling `/Users/mouse/RPM/dacode/rust/json-tiers` at **`f7f26f8`, clean** —
the deserializer fix (`c4aebe9`) and then the 53-file `cargo fmt` pass,
committed separately so the fix stays reviewable on its own. `todo-domain`
111 tests and now warning-free under `cargo doc`; `todo-store` 86 and four
goldens; mcp/cli still documented M0 shells; rustc 1.98.1, MSRV 1.95; 197
workspace tests. A cold machine needs the sibling checkout beside this
repo's parent; it needs `c4aebe9` or later only if U12 drops the
narrowing, since the wire bytes are identical either way.

Next step: **t13**, the `todo-cli` inspector — the last M1 code task before
g5, and no longer waiting on anything. U10's resolution *shrank* its scope:
`dump`, `verify`, `repair`, **no `compact` subcommand**; f14 split so
`LogHeader` gains `deny_unknown_fields` (deserialize-only, no golden moves)
while the compaction marker stays deferred. It carries f17, f18, f19, f21
(=U12), f22's store half, f23, and the d2/d3 split conversation. Then g5,
whose `cargo doc --workspace --no-deps` line t15 just made unconditionally
clean. U2 is scheduled immediately after g5.

Blocked on you: **U12** only, and it is deferred to t13 by design (its
`repair` output is what consumes the better `corrupt_store` detail). Every
other open item is either scheduled (U2, after g5) or non-blocking (U7 is
resolved; U3 is superseded by D12; t11's veto window is closed). Also
yours, in the sibling repo: whether to add `cargo fmt --check` to its
`tools/check-features.sh` — the 53-file pass is now committed as `f7f26f8`,
but that repo has no fmt gate, so without one it drifts again — and whether
to benchmark `c4aebe9` before it goes anywhere near a release.

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
