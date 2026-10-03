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

**Phase: M2 closed 2026-10-02 — g7, the exit gate, ran green on the final
tree.** The query layer is five tasks under tracker **g6** — t27–t31, plus
f47's tests-only commit — each one commit with its own file; §2's M2 row
names them and §3 carries the deliveries. The gate ran every code gate (fmt,
check, **322** workspace tests, clippy, `cargo doc` warning-free under
**both** invocations for the first time — f48 settled: the doc gate now runs
`--document-private-items`, and the one private-module link the flag exposed
is fixed, mutation-proven in both directions), the five goldens
byte-identical under `UPDATE_GOLDEN=1`, §5.2's selection re-proven a pure
function of state (no clock, no rand, no map iteration over the domain's
src), the U13 ledger re-measured and unmoved, and the **f41 sweep exercised
for the first time**: every open tracker item read against the tree —
f48/f49/f51 and the citation rule (f62) closed by this gate, **f44/f45
promoted to U21/U22** (user decisions that must outlive the branch), the
rest verified deliberately open with their named consumers. M2's lessons
consolidated into `.agents/pitfalls.md`: P-LIVE-007/-008 plus the citation
rule and the coverage-pin-constants rule (four new entries, 5.5–5.8; the
live file reset, next id P-LIVE-009). One decision is open out of the M2
decomposition — **D16**, the progress roll-up — and nothing waited on it;
it was **signed (a) on 2026-10-03**, after the gate. **M3 is next, and
unblocked**: on 2026-10-03 the user signed all three of its gates — the
**rmcp dependency ask** (rmcp 3.5.0 with `server`/`macros`/
`transport-io` plus `serde_json` with `preserve_order` per D14, recorded
in spec §6.3), **U20** resolved (b) (the registry format stays; the
server answers `repo_list` from handles it holds), and **D16** (a)
(`status_summary` and the `task_list` annotation in §6, with the
arithmetic as signed text; §12 closed again). M3 is decomposed (**g8**:
t32–t39 plus the **g9** exit gate) and its first three tasks are done:
**t32, D16's domain aggregation** (§3) — `status_summary` and
`task_list`'s annotated rows in `todo-domain` — **t33, rmcp + the
skeleton** (§3): the dependency line, the initialize handshake with our
own serverInfo and U16a's instructions, the §8 concurrency core, and
`repo_register` end-to-end — and **t34, the query read tools** (§3):
`next_task`, `next_items`, `status_summary`, `task_show` and
`decision_show` over the wire, with the first per-tool goldens — and
**t35, the listing read tools** (§3): `task_list` with D16's annotated
rows, `epic_list`, `decision_list`, `doc_list` and `graph_show` (§6.2's
dump), five more goldens, and the one domain rider
`Graph::gating_decisions_of_epic` — and **t36, the node-CRUD mutation
tools** (§3): `epic_add`/`epic_update`, `task_add`/`task_update`,
`decision_add`/`decision_update`,
`doc_register`/`doc_set_path`/`doc_update` over the wire, the
closure discipline carried by D15's carrier, and the first writer
goldens (`epic_add.v1`, `epic_update.v1` — the clock-free pair). The
next step is t37's dispatch: the edges, the registry-from-handles and
the default slug.
M1 is closed. The domain layer (§3's value types and closure discipline,
§5.1's graph, §5.2's canonical order), the storage engine (the signed D1
JSONL event log behind the `Store` trait
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
| — | nothing blocks M3: the rmcp ask, U20 and D16 were all signed 2026-10-03 (§3); the wire's shapes are settled (D14/D15/D16) and the dependency is approved (§6.3) | build — t37 next; t32–t36 done (§3) |

(Open but non-blocking: **U16** (ship base instructions with the server and
gate the client's native todo off, M3 — its mechanism is measured rather than
assumed, spec §6.3), **U18** (a live view for progress and whether the
server is shared per user, deferred until after M4's importer check, and
holding the **D13** reservation), and **U21**/**U22**, promoted out of the
g7 sweep — the duplicate-key check for `agent-rules.yaml` (f44) and
`progress.py`'s blind spot for open decisions (f45), both user calls that
must outlive the branch. Resolved: U2/U4/U5/U6/U7/
U8/U9/U10/U11/U12/U14/U17 and now **U15** and **U19**, both taken by t30 —
U17's mapping is settled and **D16 closed its one open piece** (signed
2026-10-03); **U20** resolved (b) 2026-10-03; U3 superseded by D12.
U13 is acknowledged debt, re-measured at the M2 decomposition and again at
g7 — unchanged; t31 grew `layout.rs` to 543 production lines, still under the
600 target, so the ledger does not move but the file is now close to it.)

## 2. Milestones

Authoritative names: `.agents/config/agent-rules.yaml` → `milestones`;
exit criteria: spec §14. Priority is derived from the Gated-on column by
`tools/attention.py`, never hand-written.

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | spec settled: §12 items 1–3 signed (done 2026-09-25); Cargo workspace skeleton with the lint regime; all gates green | — | done 2026-09-25 |
| M1 | domain model + storage engine: graph, statuses, persistence behind the `Store` trait, golden files | — | done 2026-09-30 — g5 ran green: every code gate, five goldens byte-identical, `cargo doc` clean, pitfalls consolidated |
| M2 | queries: repo/epic selection, dependency resolution, deterministic `next_task`; registry with path updates | **D16** adds one task if signed; no dispatched task waits on it | done 2026-10-02 — **g7 ran green**: every code gate, the five goldens byte-identical under `UPDATE_GOLDEN=1`, `cargo doc` warning-free under both invocations for the first time (f48 settled), §5.2's selection re-proven a pure function of state, the f41 sweep exercised for the first time (f44/f45 promoted to U21/U22, the rest verified open with named consumers), and M2's lessons consolidated into `.agents/pitfalls.md` (P-LIVE-007/-008, the citation rule, the coverage-pin rule). The five deliveries — t27 selection, f47's pins, t28 batch, t29 listing, t30 doc riders, t31 registry queries — are §3's entries; D16 stayed unsigned throughout and nothing waited on it — **signed (a) 2026-10-03, after closure**, its aggregation task landing under M3's goal |
| M3 | MCP server: §6's tool surface over the wire (import_md excepted — M4's, per D12), protocol-level tests, default-repo slug config (D8), smoke against a real client | gates cleared 2026-10-03: the **rmcp ask** signed (rmcp 3.5.0 + `serde_json/preserve_order`, §6.3), **U20** resolved (b) — the registry format stays and the server answers `repo_list` from handles — and **D16** signed (a), adding the domain aggregation task to the decomposition; D14/D15 settled the wire's key order and error carrier on 2026-09-30 | decomposed 2026-10-03 — **g8** holds **t32–t39** plus the **g9** exit gate: t32 the D16 domain aggregation, t33 rmcp + the skeleton (the initialize handshake, our serverInfo, U16a's instructions, the §8 concurrency core, `repo_register` as the pipeline proof), t34/t35 the read tools in two clusters (query, then listing), t36 the node-CRUD mutations, t37 edges + registry-from-handles (U20b) + the default slug (D8), t38 the wire golden suite + concurrency proof (D14/D15's pins), t39 the opencode smoke + U16b. Dispatch order t32 → t33 → … → t39 → g9; import_md stays M4's — pulling a minimal dacode-dialect importer into M3's tail is a one-task change if real-board data in the smoke test is wanted |
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
- 2026-09-30 — **D14 and D15 signed, both (a): the wire's canonical key
       order and §6.1's error carrier.** The two decisions U2's survey
       forced are settled and their work orders executed in the same
       change. **D14(a)**: this workspace enables
       `serde_json/preserve_order`, so a struct's declaration order reaches
       a client and §6.2's "fixed field order" holds as written — with the
       recommendation's second half signed along with it, because the flag
       is a *determinism dependency* that a build losing it would not fail
       on its own (it would re-sort every key and still compile): §10's
       protocol-level tests now pin response **bytes** as goldens, and §8
       names both serializers — this project's codec for the store, rmcp's
       `serde_json` for the wire. **D15(a)**: a §6.1 code travels as
       `structuredContent.code` on a tool-level error result (`isError:
       true`, the same JSON mirrored into a text block so a model reads the
       message), each tool declares its `outputSchema` explicitly, and an
       error payload is **not** required to conform to it, the schema
       describing a success. Protocol-level JSON-RPC errors stay reserved
       for what the protocol itself rejects, because clients render those
       opaquely and a code placed there is a code no caller can act on.
       §6.1 also states the two consequences for M3 that follow from where
       the carrier sits: `invalid` covers malformed *values this server
       parses*, not malformed argument shapes — rmcp's own deserializer
       answers those with unstructured text before the tool body runs — and
       every params struct denies unknown fields, because an unrecognised
       argument is otherwise silently dropped rather than reported. Spec
        §6/§6.1/§6.2/§6.3/§8/§10 moved, §12 items 14 and 15 deleted so the
        list is closed again, both `decisions.md` rows struck with resolution
        banners and their action lines retired, and board §1/§2/§4 moved with
        them. No Rust moved: the feature flag, the wire goldens and the
        carrier are M3's to implement, and this commit is what makes them
        normative. Landed as `5dfba51`.
- 2026-09-30 — **M2 decomposed: five tasks, one open decision, two debts
       measured.** The query layer is now a tracker tree (**g6**, with **g7**
       as its exit gate) rather than a milestone row: **t27** the domain's
       `graph/select.rs` — §5.2's actionability predicate (status
       `open`/`in_work`, no unfinished `blocks` predecessor, a task
       predecessor finished at `closed`/`wontfix` and a decision predecessor
       at `signed`/`superseded`, `blocked`/`deferred` never actionable
       whatever their edges say), `next_task` as the first actionable node in
       t8's canonical order, the until-target `ready` predicate and D9's
       deferred `{total, ready}` summary; **t28** `graph/items.rs` —
       `next_items`' batch with its blocked-by/blocks annotations, derived
       weight (transitive dependents) and task-scoped dependents; **t29**
       `graph/list.rs` — `task_list`'s epic/status/kind/`ready` filters;
       **t30** the store doc riders **U15** and **U19**; **t31** the registry
       query side (`repo_list`/`repo_show` over t11's registry, reading §7's
       mutable `name`/`path` off each store's `Repo` node, plus the cold-open
       measurement t11 handed forward as decision material for M3). The split
       is §2's, not convenience: selection is a pure function of graph state
       and lives in `todo-domain`, one new module per task so no two tasks
       write one file, with t28/t29 depending on t27's predicate rather than
       rederiving it; the *repo* dimension is not a domain concern at all,
       since t7's graph is single-repo, so repo selection is t31's and M3's.
       **U17 is answered**: everything in its mapping is already signed
       (`attention.py` → `next_task`/`next_items`, `progress.py` → §6.2's
       `graph_show`, `tasks.py` stays this repo's session tracker per §9, the
       HTML rendering stays a client concern per §11), and the one piece §6
       does not carry — the totals/open-items roll-up — is drafted as **D16**
       rather than settled, because adding it amends D5's signed tool list,
       which is how D11's `next_items` happened. **d1 is closed by
       measurement, not by preference**: `tasks_of_epic` has exactly one
       production caller (`delete_epic`'s cascade), `order.rs`'s judgment
       call (d) already records that the derivation builds its own maps and
       never calls it, and selection's per-candidate read is `blocked_by` —
       an indexed lookup into `blocks_in`, not a scan — so nothing in g6 puts
       the epic scan on a derivation or selection path; what remains is a
       listing consumer at §4.1's scale, the case the debt itself called
        fine. **U13 re-measured and unchanged.** No Rust moved, so every code
        gate is `skipped` by construction. Landed as `c088ba9`.
- 2026-09-30 — **M2 t27: §5.2 selection — the domain's `graph/select.rs`.**
       The first M2 code task and the one t28/t29 reuse: the actionability
       predicate, the until-target ("ready") predicate, D9's deferred
       `{total, ready}` summary and `next_task`'s two entry points. Split so
       the three consumers cannot drift: `is_until_satisfied` is the edge half
       alone — every predecessor holding a task back is finished, a task
       predecessor at `closed`/`wontfix` and a decision one at
       `signed`/`superseded`, reusing the crate's existing `is_terminal`/
       `is_resolved` rather than a new status match — and `is_actionable` is
       that plus status `open`/`in_work`. t29's `ready` filter and the
       summary's `ready` count are the edge half, so one implementation
       answers all three. `next_task()` is total and infallible;
       `next_task_of_epic()` has exactly one failure, `unknown_id`, propagated
       from `canonical_order_of_epic` rather than re-checked. The answer is a
       value, `NextTask { task: Option<TaskId>, deferred }`: **"nothing is
       actionable" is `None`, not an error**, because §6.1's list is closed
       and has no code for it, and the summary rides both entry points and
       both scopes unconditionally, including on a `None` answer. Counts are
       `u64` (`Counters`' precedent, fixed width per §8) and the two types
       carry **no serde** — a query result is not persisted and the consumer
       owns its wire shape, which is what the cli's `dump.rs` and the
       serde-free `Counters` already do.
       **Two readings of signed text, both flagged as veto-able** rather than
       settled silently. (i) The predecessors consulted are *two* adjacency
       lists — the task's own incoming `blocks` edges **and its epic's**
       decision gates, which is §5.1's "an open decision blocking an epic
       holds all work under it"; §5.2's summary sentence says only "every
       until-target `blocks` predecessor", and the gate is included in
       `ready` too so the two predicates cannot disagree — a readiness that
       ignored the gate would report a deferred task under an epic gated by an
       unsigned decision as worth revisiting now, while actionability refuses
       it the moment it is reopened. (ii) The summary's unit is **tasks**: an
       epic can be `deferred` (§3 shares `Status`) and is not counted, because
       the selection unit is a task and an epic's deferral is visible through
       its own status. Both are pinned by a test each, so a veto is a named
       diff and not an argument.
       **Epic *status* never gates — only epic edges do.** A task under a
       `closed`, `blocked` or `deferred` epic is still actionable and can be
       the answer; §3 gives an epic no closure discipline of its own and §5.1's
       gating mechanism is the edge. Stated plainly because it is the
       consequence a reviewer checks first, and pinned with the paired case (a
       decision-gated epic of the same status still holds its tasks).
       `until`, `priority` and `relates` never enter (§3/§8, D11/§11, §5.1),
       each pinned because each is a silent way to be wrong. Epic scope walks
       the **induced** order but tests each id against the **full** graph —
       t8's own documented contract — so a cross-epic predecessor contributes
       no in-degree to the list and still holds its successor back. Two
       fallbacks are unreachable by type and documented rather than tested: an
       epic endpoint as a *predecessor* (`BlocksEdge::source` has exactly two
       arms) and a predecessor id the graph does not hold (`link_blocks`
       requires both ends, the cascade removes edges with its nodes, and
       `assert_invariants` proves both indexes account for exactly the stored
       set); both answer "not finished", so an unclassifiable endpoint holds
       work back rather than releasing it, and neither panics. No new field,
       no cache, nothing stored — every call recomputes, which is the whole
       determinism argument. 240 production lines.
       19 tests, 130 in the crate (the pre-existing 111 unmodified), 272
       workspace-wide; no golden moved, `Cargo.lock` untouched, `cargo doc`
       clean, and the no-clock grep still empty. Dual-reviewed over two rounds
       on the same tree, and **the review earned its keep in the first one**:
       D9's epic-scope order had no falsifiable pin, and two wrong
       implementations survived all 18 tests — walking `canonical_order()`
       filtered to the epic (D9's rejected alternative) and walking the epic's
       tasks in ascending id, dropping topology and the signed D6 tiebreaker
       together. The reviewer built and ran both rather than reasoning about
       them, and the hand-built case could not catch them because with E1 < E2
       the induced and filtered orders coincide, while the seeded graphs reach
       the divergence once in 74 epics. T19 now pins both, with the induced
       order asserted beside each answer so the test names which of the two
       orders it means, and each mutant was observed failing exactly it. The
       same round caught a **spec error by the same mechanism**: as written,
       T18's chain held exactly one actionable task at every step, so
       first-equals-last and the last-not-first mutant survived it — the
       implementer reported the spec wrong rather than reporting the mutation
       as run, added an independent actionable task later in canonical order,
       and re-ran it to observe the failure. Strengthening the coverage pin
       then found a vacuity of its own: seed 14's graphs never hold a task by
       an epic gate *alone*, which is the shape that makes the seeded
       agreement test sensitive to deleting the gate, so the seed set changed
       and the pin now asserts **per seed** instead of over the union — a
       generator change that narrows one seed fails instead of hiding behind
       the other four. Three stale doc claims this change falsified
       (`order.rs`'s module doc calling selection M2's future work, the same
       tense in `tests/order.rs`, and `node.rs`'s "a later M1 commit") were
       fixed as riders, and eight more candidates were read and left with
       their reasons. Four lows from round two are recorded as **f47** rather
       than looped, with t28's dispatch named as their consumer. Landed as
       `9859bb6`.
- 2026-10-01 — **M2 f47: t27's four round-2 lows closed, tests-only.** The
       consumer named in the finding was t28's dispatch, and that routing did
       not survive contact with t28's own file ownership: three of the four
       items live in t27's files (`graph/tests/select.rs` three times,
       `graph/order.rs:134` once) while t28 owns `graph/items.rs` and its test
       file alone and must *import* `DeferredSummary` and the two predicates
       rather than edit them. So it became the `t21`/`f19` shape — its own
       commit with its own dual review — dispatched **before** t28 so the batch
       surface is written against a base where the induced epic order is
       pinned. `select.rs` is byte-identical to HEAD throughout (sha-proven at
       every step); the diff is two files, and the `order.rs` hunk is
       comment-only.
       T19 gained clause (iii), pinning the epic scope against a
       chain-following greedy walk of the induced subgraph (order, repo answer,
       epic answer); the coverage pin's sole-hold counter now counts only tasks
       where the agreement test can observe the epic gate — a live one, which
       is the answer the gate withholds, or a deferred one, which is the
       summary's `ready` count — instead of counting terminal sole-holds that
       can be neither; T13's `relates` end is `deferred` rather than `closed`,
       so both leak kinds move the whole `NextTask`; and `order.rs`'s two
       actionability doc sites link `Graph::is_until_satisfied` instead of
       naming a milestone. No new test function: the count stays 19 and the
       crate 130.
       **The finding's own text was wrong twice, and both corrections were
       measured rather than argued.** Item 1 called the greedy walk "ONE
       residual unpinned implementation" and gave its divergence condition as
       "an epic holding two independent inner chains". Review refuted both
       halves: **at least three** residuals survived all 19 tests — the
       chain-following greedy walk, a FIFO and a LIFO walk of the induced
       subgraph — and all three now die at the new clause; and the condition is
       false in both directions, since two independent inner chains **agree**
       under both drain variants while one chain plus a lone task **diverges**
       (`[1,2,3]` against `[1,3,2]`). The walks part wherever a just-released
       successor is taken ahead of a smaller-keyed task the frontier still
       holds, and the exact condition turns out to be drain-definition-dependent
       — a recursive drain gives diverge-if-and-only-if that condition, a
       one-level drain has counterexamples — so the comment names the property
       both variants share and claims no completeness. Item 3's premise was
       wrong too: the old `closed`-end shape caught **neither** leak, both being
       carried by the seeded agreement test alone, so the defect was a source
       comment claiming a pin its clause did not have (pitfalls 5.2) rather than
       a lost half of one. The new shape kills both at T13, and the trade is
       recorded rather than hidden: it gives up the old shape's kill of an
       inverted-`finished` `relates` reading, which the seeded test still
       catches — two kills for one.
       A repo-scoped residual was found on the way and is now pinned. A greedy
       rewrite of `next_task` alone survived all 130 tests, pre- and post-edit,
       and clause (iii)'s hand-built graph is single-epic, so its repo
       assertion — the one the clause's shape would otherwise have skipped as
       redundant — is the only thing that catches it. Eight order mutants were
       built across the review; four die on that assertion alone. The seeded
       generator reaches none of these shapes: 0 of 74 epic scopes across the
       five seeds diverge, which is why the clause is hand-built and why the
       agreement test let the greedy walk live through t27.
       Three review rounds, both reviewers `ok` on the committed bytes. Round
       one found the load-bearing defect and it was **the orchestrator's**: the
       replacement comment asserted "the two agree everywhere except an epic
       holding two independent inner chains", a sentence inherited unmeasured
       from the finding's own text — the exact class pitfalls 5.2 names, written
       by the party that rule binds. Both reviewers falsified it independently,
       with different counterexamples, and the primary additionally proved the
       divergence condition sufficient over every labelled DAG at n=3 and n=4
       plus roughly 26 000 sampled graphs. Round two's single low each was a
       wording fix ("outranks" parses backwards under D6's smaller-key-ranks-
       earlier sense; "the seeded graphs never reach" is true at epic scope and
       **false at repo scope**, where the shape fires on seeds 11/44/60), both
       applied and both verified in round three. The fixes were applied by the
       orchestrator rather than bounced back, per the t11 round-3 precedent,
       because they corrected its own prose.
       Three spec errors were reported by the implementer rather than worked
       around: this brief's `rg` count for `order.rs`'s `M2` mentions was five
       where the file holds six (judgment call (c) spans two lines), its
       `tests/select.rs` line count was 1067 where the file holds 1025, and a
       quoted mutant total was probe-count-dependent. The first two are the same
       house defect class in the brief itself. Two findings were recorded:
       **f48**, the `cargo doc` gate has a private-module blind spot and one
       link is already broken behind it — an unresolvable intra-doc link to
       `BTreeSet` in `reference.rs:6`, which only `--document-private-items`
       renders, so it prints exactly one warning where the gate prints none;
       and **f49**, `_rust.md`'s `awk` production-line measure counts a
       test-only file in full, because such a file never contains the
       `#[cfg(test)]` line the measure stops at — its parent carries it — so
       a crate-wide sweep for U13 would report `todo-domain` over budget on
       files the rule exempts.
       Gates on the committed tree: fmt, `check --workspace --all-targets`,
       clippy and `cargo doc --workspace --no-deps` clean; 272 workspace tests
       (domain 130, store 107, cli 35) with `-- select` 19 against a
       `#[test]` count of 19; the five goldens untouched; `Cargo.lock`
       untouched; `check-links`, `agents-sync --check` and `attention.py
       --check` green. Production lines: `order.rs` 241 (one more, a doc
       rewrap), `select.rs` 240 — no exception line and no U13 movement.
       Landed as `8581533`.
- 2026-10-01 — **M2 t28: §5.2's batch half — the domain's `graph/items.rs`.**
       The second M2 query task and the one t29 reads: `next_items`,
       `next_items_of_epic` and `next_items_of_task` (D1's three entry points,
       no scope enum). Repo and epic scope answer the actionable prefix of the
       scope's canonical order — the induced order for an epic, tested against
       the full graph — truncated at `limit`; task scope answers the queried
       task's irreflexive transitive dependents, unfiltered, each marked
       immediately actionable or still held. One row type, `NextItem { task,
       blocked_by, blocks, weight, actionable }`, and **no serde** — a query
       result is not persisted and the consumer owns its wire shape (t27's D3,
       the cli's `dump.rs` and the serde-free `Counters`): `blocked_by` is the
       structural list — own incoming edges plus the epic's decision gates —
       `blocks` is narrowed totally to `Vec<TaskId>` (§5.1 gives a task no other
       outgoing edge; filter, never panic), `weight` is the transitive-dependent
       count computed only for returned rows, and `actionable` is always
       `is_actionable`'s answer, never a second status match. `blocked_by`'s
       order is **two ascending runs** — own edges in `BlockEnd` order, then the
       epic's gates, deduplicated at first occurrence — deliberately not merged
       into one sorted list, because a global merge is the second sort D2's "do
       not re-sort" forbids; **M3's `outputSchema` must describe the field as
       two runs, not one `BlockEnd`-ordered list**, and that contract is stated
       in both the field doc and the judgment call so whoever declares the wire
       bytes reads it. `limit` is `u64`, `0` an empty answer with the summary
       still present, no `truncated` field (a field the signed text lacks is a
       wire change), and the default of 10 stays the wire's, not the domain's.
       The deferred summary is a second pass over the same derived list —
       `select.rs` byte-identical throughout, sha-proven at every step — with
       its agreement with `next_task` **pinned over the seeded graphs** rather
       than argued. `order.rs`'s judgment call (c) amended comment-only: the
       trigger its `order_key` seam anticipated was met and **did not fire** — a
       batch is a *filtered* prefix of the derived order and never reads the
       key. 387 production lines; 19 tests, crate 149, workspace 291, every
       gate green, no golden moved, `Cargo.lock` untouched.
       **Three readings of signed text, all flagged veto-able and pinned** so a
       veto is a named diff: (i) the two populations — repo/epic actionable-only,
       task scope every dependent marked, because the uniform reading makes
       §5.2's "still held" mark unreachable; (ii) the epic's decision gates
       inside `blocked_by`, because without them a gate-held task shows an empty
       list beside `actionable: false`; (iii) task scope's list derivation — the
       repo order restricted to the closure, not an order induced over the
       dependent subset, which would need a new seam in the order module first.
       (iii)'s pin is **honestly circular** — the naive recomputation reads the
       same contract, so the seeded agreement test cannot falsify the choice —
       and is recorded as such in the module doc so nobody assumes a pin exists.
       Veto (i) flips one flag and fails T4; veto (ii) deletes one loop and fails
       T16/T19. The dedup arm of (ii) — a decision gating a task both directly
       and through its epic, listed once — is hand-pinned by a fixture that gates
       U5 twice, and the unconditional-push mutant fails exactly that row.
       **The brief was wrong three ways, each measured rather than argued.** M5's
       stated expectation was unsatisfiable — T18's coverage counters are
       computed through the naive helpers only, so *no* production mutation can
       fail them; T18 is a generator-reachability pin, not a mutation detector,
       and M5 actually dies at T16 and T19. T18 was unreachable at the constants
       the brief inherited from t27 — 300 steps on select's seeds never reach a
       mixed-mark task scope or a truncated batch — so the suite measured a grid
       and picked 600 steps, limit 3, seeds `[13,30,50,60,128]`; the seed set and
       step depth are deliberately this suite's own and documented beside the
       constants, because whole-batch shapes need deeper graphs than the
       selection pin's per-task shapes, and t29's brief must not "inherit"
       select's blindly. And T15's terminal-task clause was over-broad: task
       scope does not filter, so a terminal task *with* dependents lists them
       marked.
       **Two dispositions out of the round, both recorded rather than looped.**
       **f50**: the dedup arm's coverage counter was *not* added to T18 — the
       generator reaches the double-gate shape in none of this suite's five seeds
       (proved by running the mutant: the seeded agreement test passes under it),
       an unasserted counter is the dead-counter class f47 item (2) closed, and
       widening `tests/seeded.rs` would stale t27's and f47's sign-offs for
       coverage a hand-built row already provides; f50 also carries the seam note
       that `naive_actionable`/`naive_until_satisfied`/`naive_end_finished` are
       now duplicated verbatim in two suites and t29 would make a third, so the
       next owner of `tests/mod.rs` should lift them (production-independence is
       the property that matters, not per-suite duplication). **f51**: `-- order`
       now matches 27 tests because four `items_*` names contain the substring
       (the collision pre-existed at 23) — a verify line must quote the count
       beside a name filter and read it against `rg -c '#[test]'`, or use a
       module-path filter; no test was renamed.
       Dual-reviewed over two rounds, both reviewers `ok` on the committed bytes.
       Round one ruled D2/D6's "the accessor's own order" **per-source** — the
       global-order reading is self-inconsistent with D2's "do not re-sort" — so
       judgment call (k)'s concatenation is a documented reading rather than a
       defect, and asked for the why-not-merged sentence M3 needs. It also found
       the load-bearing gap: the dedup arm of (k) had **no test** — a mutant
       pushing epic gates unconditionally survived all 19 — which is f9's
       vacuity class and exactly what f47 existed to close. Round two verified
       the fix (re-running the dedup mutant independently, 1 failed / 148
       passed, the duplicate `Decision(D1)` visible in the assertion), proved the
       seeded non-reachability that justifies f50 by the same run, and
       falsified the amended `order.rs` (c) against the code — its round-one
       wording ("a prefix of a list already in key order") was false, since the
       list is lexicographically-minimal *topological* and a batch is a
       *filtered* prefix. Both reviews re-ran all six mutations. Landed as
       `f5665dc`.
- 2026-10-01 — **M2 t29: §5.2/§6's listing half — the domain's
       `graph/list.rs`, and with it the M2 query surface is complete.** One
       method and three thin reads, 251 production lines: `task_list` takes a
       `TaskFilter` of four ANDing optional filters — epic, status, kind and
       `ready` — and returns `Result<Vec<TaskId>, DomainError>`, the only `Err`
       being §6.1's `unknown_id` for an epic the graph does not hold;
       `epic_list`/`decision_list`/`doc_list` are unfiltered ascending-id reads,
       each named and pinned as a **domain contract** rather than left to
       `BTreeMap` iteration that M3 would come to rely on by accident. Every
       list is in its documented total order, and the order is **reused, never
       re-derived**: `list.rs` calls `canonical_order`/`canonical_order_of_epic`
       and `is_until_satisfied`, so f47's pin on the induced epic order guards
       this module too and a fourth derivation never existed to be wrong.
       **`Vec<TaskId>` is the D16 guard and it is structural** — a bare id list
       has nowhere to hang the `blocked_by` annotation D16(a) would add, so this
       commit could not pre-build an unsigned decision even by accident; no
       counts, no aggregate, no `status_summary`, no serde on the new types, and
       no row struct. `ready` implements the **deferred-scoped** reading (ii) of
       §6's "for deferred items — until-satisfied (`ready`)", reusing t27's
       predicate so the two cannot drift, flagged veto-able against the
       status-agnostic reading (i) and pinned by a test that asserts its premise
       first — the sixth veto-able reading of signed text now pinned by a named
       diff. **D8's cross-module invariant is pinned rather than assumed**:
       `task_list(status=Deferred).len() == next_task().deferred.total` and
       `(…, ready=true).len() == deferred.ready`, at repo scope and every epic
       scope, hand-built and seeded. Decision **L** ran first, before any
       `list.rs`: the three byte-identical `naive_*` helpers lifted out of
       `tests/select.rs` and `tests/items.rs` into `tests/mod.rs` as `pub(super)`,
       pure 42-line deletions in both donors with select 19 and items 19
       unmoved, so this suite is not a third verbatim copy (f50). 18 tests.
       Dual-reviewed over two rounds, and the med that drove the second is the
       reason the round existed: **`ready`'s decision-gate dimension had no
       hand-built pin and no coverage counter** — the primary's gate-blind
       mutant (dropping the `BlockEnd::Epic` half of `is_until_satisfied`) was
       killed by exactly one seeded test, at one of five seeds, with a one-id
       margin, so a seed or `LIST_STEPS` change would have removed the pin with
       the coverage test still green. That is f9 one level down and t28's
       unpinned dedup arm in a new register. The fix round added the hand-built
       test covering **both** halves (a deferred task held by a decision gating
       its *epic*, and one held by a decision gating *itself*, premise asserted,
       the answer moving as each gate is signed) and a per-seed counter whose
       suppression fails naming the seed — and the per-seed assertion then
       **forced a seed re-pick**, the old constants being unsatisfiable for that
       branch (measured `0,0,0,1,2` across them, reproducing the primary's own
       numbers, and `s60` dropping to zero at 700 steps, which is why the
       constant's comment says the shape is rare and not monotone in depth —
       mechanism only, no figure). Round two re-measured **every** coverage
       branch at the new seeds, because a re-pick can silently vacate a branch
       that was pinned at the old ones, and found none vacated; the gate-blind
       mutant now dies on the hand-built test and on the seeded agreement test
       at a two-id margin. The primary's 20 mutants killed 19; the survivor is
       the resolution seam's unreachable-`None` arm, which drops rather than
       invents, matches two reviewed precedents (`select.rs`'s `filter_map`,
       `items.rs`'s `if let`) and is recorded as a finding with no change
       requested. The secondary re-bound its `ok` to the final bytes rather than
       letting the round-1 verdict stand over a changed file. Two spec errors
       were measured rather than worked around: **§3 D2 of the task spec
       misattributed `deny_unknown_fields` and the wire key order to D15** where
       D14 is the key order (`decisions.md:620`) and D15 the §6.1 carrier
       (`:689`), and `deny_unknown_fields` appears nowhere in `decisions.md` —
       the tree cites D14 correctly and carries no misattribution; and the
       coverage pin was unreachable at the inherited constants, the spec's own
       named contingency. Gates all run on the final tree: fmt clean; `check
       --workspace --all-targets` clean; `-- graph::tests::list` 18 passed / 0
       failed / 149 filtered out, equal to `rg -c '#[test]'` on that file (the
       only proof a name filter ran, and the module path rather than the bare
       word per f51); `-- graph::tests::select` 19 and `-- graph::tests::items`
       19; crate 167 = 149+18; workspace 309 = 291+18 with store 107 and cli 35
       unmoved; clippy clean on a forced recompile; `cargo doc --workspace
       --no-deps` 0 warnings and `--document-private-items` printing exactly the
       one known f48 warning; the five goldens untouched and `Cargo.lock` absent
       from status; `select.rs` and `items.rs` sha-equal to HEAD before, during
       and after, `order.rs`/`state.rs` never in the diff; `list.rs` at 251
       production lines needs no exception line, so U13 does not move. Two
       riders this diff could not take because both files were sha-pinned: the
       `select.rs`/`items.rs` module-doc markers calling the batch and listing
       surfaces "later M2 work", now false for both, which land as their own
       comment-only commit inside this unit; and a fourth per-suite `shuffle`
       copy plus near-duplicated spec builders, a rider on the next
       `tests/mod.rs` owner. **D16 decision material, measured not argued**: the
       dossier's "the annotation half is separable and cheap" does not survive
       contact with the code — `items.rs`'s `annotate` is private, and the
       composition rule it implements (append the epic's gates, dedup at first
       occurrence, two ascending runs) is a signed domain contract, so a server
       building D16(a)'s annotation over `Vec<TaskId>` would re-implement that
       rule in the binary; and six `task_list(status=s).len()` calls in the
       binary would be an aggregation in the server against §2's "translates,
       does not decide", re-deriving the canonical order six times per summary.
       A signed (a) therefore most likely forces a domain aggregate and a domain
       row type, which is recorded in the dossier and changes no recommendation.
       Landed as `f3ef61a`.
- 2026-10-01 — **M2 t29 rider: the "later M2 work" module-doc markers
       retired.** Three module docs carried a marker naming work that had
       since landed, and all three were false by the time t29 committed:
       `select.rs` and `items.rs` called the batch and listing surfaces
       "later M2 work" (false since t28 and since t29 respectively) and
       `order.rs` called `next_items`'s batch "M2's remaining selection"
       (false since t28, and unnoticed until this review). t29 could not
       take the first two — both files were sha-pinned as its review gate
       lines — so both its reviewers prescribed a separate comment-only
       commit inside the unit, and it was that commit's own second reviewer
       who found the third instance its first had passed, plus a judgment
       call in `select.rs` whose recorded anticipation had resolved. All
       three markers are rewritten **positively**, naming where each
       surface lives rather than what is not yet built, so t31 and M3
       cannot stale them a fourth time; the judgment call is amended on the
       t28 precedent — a judgment call here is a live contract doc, so it
       is amended when the anticipation it records resolves rather than
       left as dated history, and the primary withdrew its contrary
       round-one reading on that precedent. One clause of my own rewriting
       was corrected on review: "bare answers" is `list`'s property, not a
       shape it shares with `items`, whose answers are annotated rows.
       Comment-only, and every clause of the new text was read against the
       code by both reviewers rather than accepted as plausible — that the
       three surfaces do read `order`'s derivation, that `items` and `list`
       both call `select`'s predicates, that `items` carries
       `DeferredSummary`, that `list` calls nothing from `items`. 167 tests
        unchanged, `cargo doc` clean with `--document-private-items` at the
        one known f48 warning, no golden, no `Cargo.lock`. Dual-reviewed
        over two rounds, both bound to the final shas. Landed as
        `034d081`.
- 2026-10-02 — **M2 t30: the store doc riders — U15, U19, and two of their
       kind.** The two one-sentence fixes the board had parked on "the next
       store touch", taken as one comment-only commit, plus the two further
       claims of the same class its own review turned up. No behaviour moved
       anywhere in the crate: every changed line in `git diff -U0 -- crates/`
       begins with `///` or `//!`, and all four files — `store.rs`,
       `repair.rs`, `error.rs`, `layout.rs` — are byte-identical to the
       baseline once comments are stripped, which is the proof that makes a
       diff like this cheap to review and the reason it still carries both
       trailers (f55's caveat: a comment-only edit to production `.rs` is not
       review-exempt).
       **U15, remedy (a) — scope the sentence, do not add cleanup.**
       `Store::snapshot`'s doc claimed a failed snapshot "changes nothing on
       disk", which §4.3's own discipline falsifies: a rename that refuses
       leaves its pid-suffixed temp beside the snapshot, and t21's test
       measures exactly that. The doc now claims nothing **in place** — the
       log, the graph and the snapshot already on disk untouched — and states
       what a failure *after* the temp file's creation does leave, why the
       debris is inert (its name is never the snapshot's, and nothing in this
       crate reads a store directory's contents: no `read_dir` in either
       crate's `src`), and how it is reused (the same process's next snapshot
       of this store truncates it; a dead process's is left behind). It names
       the pin rather than counting what the pin counts, and says which path
       the pin drives — `compact`'s step (1) is the same `write_snapshot_atomic`
       a bare `snapshot()` calls, there being no test that faults a bare one
       (f57). Cleanup-on-failure was rejected on measurement, not taste:
       §4.3's signed text promises temp+`fsync`+rename "so they are never
       torn" and nothing about removing a temp; the debris assertion would
       have had to invert, i.e. move a t21 sign-off to fix a doc; and the
       change reaches four write sites while adding a failure mode — the
       cleanup failing — inside the one path whose promise is "not
       session-fatal".
       **U19, the one word.** `repair.rs`'s `Err` rationale closed with "so a
       snapshot fault is never reported through it", false read broadly
       because a snapshot-origin *reconcile* fault is reported through
       `Unrepairable { offset: None }` and a test pins that it is. Narrowed to
       "that fault" — the parse fault the sentence was argued for — and checked
       against the code rather than the report: classification-order item 2
       already states the contrast case in terms, so the narrowing removes a
       contradiction instead of needing a new clause.
       **The class sweep found two more, and they are why the diff is four
       files rather than the board's two.** `error.rs`'s `Locked` doc and
       `layout.rs`'s `Unnamed` rationale both said "nothing was **opened**,
       read or written" — and `open_and_try_lock` opens the lock file, with
       `create`, before `try_lock`, because that open is how the contention is
       discovered. The sweep's own grep had been too narrow: it looked for
       "nothing written / nothing on disk / untouched" and not for "nothing was
       opened". Both were fixed here rather than deferred to a board item on
       f55's rule — sweep the class, not the instances — since t30 *is* the
       store touch such an item would wait for and t31 is the next one, so
       deferral would have bought nothing; both reviewers ruled the four-file
       diff one logical change and one commit.
       **Review earned its keep in round one, on my own sentence.** The primary
       falsified "a failure **at or** after the temp file's creation does leave
       that temp" by building the two states that refute it — a `chmod 0500`
       store directory, where the `open` itself is refused and nothing is
       created, and a store directory removed out of band, where the handle's
       `fstat` still succeeds and the temp open reports `NotFound` — and it
       also measured the positive half at a bare `Store::snapshot()`, the seam
       no committed test drives. Three rounds of lows followed, all on prose:
       the debris reuse was scoped per store rather than per process (`temp_path_for`
       keys the name on the directory, so a process holding many stores reuses
       it only for this one); "no reader opens anything but the exact paths it
       derives" was loose because `JsonlStore::open_log` is `pub` and takes a
       bare path, so the enumeration fact replaced it; and "the whole effect
       **of the attempt**" survived neither `LockTarget::Registry` — whose
       `lock_registry` runs `create_dir_all` on the root strictly before the
       acquire, so a racing loser's refusal follows a mkdir — nor the phrase's
       own target-neutrality, and became "the whole effect **on the data**".
       The pointer this diff added follows the crate's convention
       (`[crate::JsonlStore::open_log]`, as `lock.rs` and `layout.rs` already
       do) and both reviewers independently found that convention lands one
       method off the fact it is attached to — the lock-debris sentence is in
       `create_log`'s doc, not `open_log`'s — at all three sites, two of them
       pre-existing. Left alone and recorded as f58 rather than fixed here:
       the honest remedy is one sentence in `open_log`'s doc, which makes all
       three pointers true at once, and it belongs to whoever next owns
       `log.rs`.
       Gates on the final tree: fmt clean; `check --workspace --all-targets`
       clean; `cargo test --workspace` 309 passed 0 failed, with `--test
       snapshot` 32, `--test repair` 19 and `--test layout` 20 all at 0
       filtered out and the named debris pin run by name; clippy clean on a
       forced recompile; `cargo doc --workspace --no-deps` 0 warnings and
       `--document-private-items` at exactly the one known f48 warning, which
       is the gate that would catch the new link being wrong; both golden dirs
       `git diff --exit-code` 0 including after an `UPDATE_GOLDEN=1` run over
       the twelve writers; `Cargo.lock` untouched; production lines 299
       (`store.rs`), 342 (`repair.rs`), 188 (`error.rs`), 383 (`layout.rs`),
       all under the 600 target, so U13 does not move. Dual-reviewed over four
       rounds, both reviewers bound to the final bytes. Landed as
       `aa107d9`.
- 2026-10-02 — **M2 t31: the registry query side, and the measurement M3's
       registry question was waiting for.** `DataRoot::repo_list` and
       `DataRoot::repo_show`, named after §6's tools so the M3 translation is
       auditable by name, answering the domain's **`Repo`** — no new row type,
       because §3's node already carries exactly the four fields a registry
       query answers with (`slug`, `name`, `path`, `registered_at`), and a
       `RepoSummary` would have been a second shape to keep in step. Order is
       **ascending slug and it is the registry's own**, not one the method
       imposes: `Registry::load` sorts and `slugs()` hands them back that way,
       so registration order never reaches the answer (§6/§8). `name`/`path`
       come from each store's node because §7's mutables live only there under
       t11's narrow reading, so a listing costs one store open per repo — which
       is exactly the cost t11 handed forward and what the measurement below
       prices. No serde (a query result is not persisted; M3 owns the wire
       shape) and **no counts or aggregates of any kind**, D16 being unsigned.
       **Locking is one store at a time in sorted order**, `open_many`
       deliberately not used: a listing protects no cross-store invariant,
       holding every descriptor at once has a per-process ceiling, and one lock
       at a time is the shorter contention window — §8's sorted-slug rule is
       satisfied trivially because no acquisition order exists to deadlock. The
       trap that follows is documented *and* tested rather than smoothed over:
       anything holding a listed store's lock fails the whole call, in three
       ways — a caller holding its own handle contends with itself (§8's
       contention is per descriptor), a second listing contends with the first,
       and `register` returns its new store still locked, so a listing that
       reads the index after the save but before the registrant drops the
       handle fails on the repo just registered. M3 answers `repo_list` from
       the handles it holds, or before it opens any.
       **A store that will not open fails the whole call**, with the error
       `DataRoot::open` itself reports byte-identical — corrupt log, foreign
       format version, a listed slug whose log is gone, a contended store, each
       pinned and each asserting an `Err` rather than a shorter list, because
       §6.1 is closed and has no "partially listed", and a row that quietly
       disappears is the silent-omission class. The mirror half holds too: a
       store directory the index does not list is not listed, and resolving it
       stays t13's inspector's.
       **The review's med is the delivery's most substantive addition.**
       `repo_node` trusted that `repos/<slug>/` holds `<slug>`'s graph, so a
       store directory copied out of band made `repo_show` answer *another
       repository's* record and `repo_list` carry one repo twice under two
       slugs — breaking the total order it documents — with nothing anywhere
       detecting it: t13's `verify` prints the requested slug and never
       compares it to the graph's (f61). It is now a cross-check returning
       §6.1's `corrupt_store`, unpositioned (the fault is a relation between
       two facts, not a byte in one file) with a detail naming both slugs, on
       `snapshot.rs`'s slug-reconcile precedent and its "a slug is an id, not a
       path" rule. Pinned by a test built by copying one real log over another
       — never hand-written JSON — and mutation-proven: deleting the check
       fails exactly that test with `unwrap_err()` on
       `Ok(Repo { slug: "camera", … })` asked for as `shelly-ng`.
       **Four readings flagged so a veto is a named diff.** (1) `repo_show`
       consults the registry, so an unlisted slug is `unknown_id` even with a
       store directory behind it: §6.1 lists the code and glosses nothing, the
       tension is the *domain's* node-scoped gloss ("no node exists under
       `id`", and a registry is not the graph), and the rejected alternative is
       `DataRoot::open`'s `io` — which would answer a different code for "this
       server has no such repository" depending on whether a directory happens
       to sit there. The review prescribed anchoring that choice on
       `decisions.md:848`; the anchor was refused because that line is inside
       **D16's unsigned dossier**, and citing a draft as signed support is the
       same defect inverted. (2) The foreign-store refusal itself, a
       `corrupt_store` D6 never enumerated. (3) Two tests beyond the spec's
       twelve: a registry this build cannot read fails both queries (an empty
       list substituted for it would be D6's omission one level up), and the
       slug mismatch. (4) `error.rs` as a fourth file, which D9's list did not
       name: t31 adds a third class of `CorruptStore` producer, so a variant doc
       describing only parse and serialize faults would have been falsified by
       code landing in the same commit. Both reviewers ruled it in scope on the
       f55-vs-f58 distinction — ride when the diff's own subject touches the
       class and the remedy is bounded; defer when the remedy describes another
       file's mechanism the diff neither changed nor owns.
       **The measurement, which is what M3's registry decision waits on.** A
       committed `#[ignore]` test — a measurement, not a property: it asserts
       structure only, never a duration, and its doc carries the exact command
       (`cargo test -p todo-store --test layout -- --ignored --nocapture`),
       which is a convention this repo did not have. `std::time::Instant`
       appears in that one test file; `src`'s no-clock grep stays empty. Grid
       `[1, 10, 50] repos × [0, 200] events`, first call and steady mean of six
       reported separately, on this host at this commit — release: registry
       floor **12–59 µs** whatever N is, cold `repo_list` **120 µs** (one
       seed-only repo) to **12 099 µs** (50 repos × 200 events), i.e. **~45–61
       µs per seed-only repo and ~238–270 µs per repo with a 200-event log**
       (≈1.2 µs per event), with the first call running 1.2–2.9× the steady
       mean; debug is 6–8× slower across the board. **So a cold `repo_list` is
       O(total history), not O(repos)** — at §4.1's top of scale one repo would
       cost ~120 ms and ten of them ~1.2 s steady — and the numbers are the
       *optimistic* case for opening every store, because a cold page cache
       would inflate the whole listing (every log) far more than the floor (one
       small document). Reproduce them with the command above; the table lives
       in this entry and the tracker node's evidence, never in a source
       comment. That is **U20**.
       Gates on the final tree: fmt, `check --workspace --all-targets` and
       clippy clean on a forced recompile; **322** workspace tests (was 309),
       the `layout` target **33 passed + 1 ignored with 0 filtered out**
       against a `#[test]` count of 14; `cargo doc --workspace --no-deps` clean
       and `--document-private-items` at exactly the one known f48 warning —
       the invocation that proves the new intra-doc links resolve; all five
       goldens untouched, `Cargo.lock` untouched, `git status` confined to the
       four files. Production lines **543** (`layout.rs`, from 383) and **193**
       (`error.rs`), both under the 600 target so U13 does not move — but
       `layout.rs` is now 57 lines from it, and the natural split when it
       crosses is the query side. Seven findings recorded (f59–f65), among them
       the one-lock-at-a-time discipline being unpinned (an `open_many` mutant
       survives all 33 tests and no non-flaky single-threaded pin exists) and
       `Registry::load` sorting without deduplicating. Dual-reviewed over four
       rounds, both reviewers bound to the final bytes; the review earned its
       keep in round one (the wrong-slug answer) and again in round three, on
       two clauses of measurement prose the orchestrator had just written — a
       duration figure in a comment falsified by the same test's own `build ms`
       column, and a claim that a warm page cache left the floor/whole ratio
        unaffected, which the byte counts refute. P-LIVE-008's third and fourth
        occurrences, an hour after it was written. Landed as
        `a1c4afe`.
- 2026-10-02 — **M2 g7: the exit gate** — M2 closed. Every code gate ran
        on the final tree: fmt clean; `check --workspace --all-targets`
        clean; **322** workspace tests summed over 17 result lines (domain
        167, store 120 = 9 lib + 8 golden + 33 layout + 19 log + 19 repair +
        32 snapshot, cli 35), 0 failed, 1 ignored (t31's measurement);
        clippy clean; `cargo doc --workspace --no-deps` warning-free under
        **both** invocations for the first time. The golden ritual: 12 store
        writers plus the cli suite under `UPDATE_GOLDEN=1`, `git diff
        --exit-code` over both golden dirs = 0 — five goldens
        byte-identical. §5.2's selection re-proven a pure function of state
        (the no-clock / no-rand grep over the domain's src is empty); the
        U13 ledger re-measured and unmoved (log.rs 812, event.rs 772,
        layout.rs 543, state.rs 460, repair.rs 342 production lines).
        **f48 settled, both halves**: `reference.rs`'s module doc linked
        `[`BTreeSet`]` without importing it — invisible to the plain doc
        gate because the module is private, one warning under
        `--document-private-items` since t6 — fixed to the fully-qualified
        path, and the gate line now names both invocations (`_common.md`'s
        gate order, `build-and-test.md`'s gate list, pitfalls 4.4).
        Mutation-proven in both directions: reverting the one-word fix
        prints exactly that warning under the flag and none without it;
        restoring it returns the count to zero. **f49 settled**:
        `_rust.md`'s awk measure now states it is for a production file
        carrying an in-file test block, and that the exempt files must not
        be fed to a sweep. **f51 landed** beside the gate list: a
        test-name filter is a substring match over the whole test path —
        quote the passed count beside every name filter, or use a
        module-path filter. **The f41 sweep ran for the first time** —
        every open tracker item read against the tree, each disposition
        verified rather than assumed: f48/f49/f51/f62 closed by this
        gate; **f44/f45 promoted to U21/U22** (their own verify lines
        demand the generalisation not survive only in the gitignored
        tracker); f4, f25, f50's generator half, f54, f57–f61, f63–f65
        verified deliberately open, each with its named consumer
        confirmed against the tree (todo-mcp is still a 16-line shell; no
        store-side inspect API; `seeded.rs` untouched since t27; the four
        shuffle copies at their measured lines; no test faults a bare
        `Store::snapshot()`; `open_log`'s doc still lacks the lock-debris
        sentence; `Registry::load` still sorts without dedup; `verify.rs`
        still prints the asked slug; the attention priority derivation is
        still Gates-only). **The consolidation**: four new pitfalls
        entries — 5.5 a per-seed coverage pin's constants are part of the
        pin (t27/t28/t29's forced re-picks), 5.6 a citation's content is
        a claim, and so is its signedness (f53/f62, five consecutive task
        specs with unreproducible claims), 5.7 a class sweep is only as
        wide as its grep (P-LIVE-007/f56), 5.8 whoever writes a contract
        sentence cannot falsify it (P-LIVE-008) — plus 4.4's
        blind-spot addition and two new top-ten rules; `pitfalls-live.md`
        reset with the former-id table extended to P-LIVE-008, next id
        P-LIVE-009. D16 unsigned throughout; U15/U19 already struck by
        t30; g6 closed with the gate's evidence. Landed as `73faff5` (the
        dual-reviewed code commit) and `d30cf08` (this consolidation).
- 2026-10-03 — **the M3 gates cleared: rmcp, U20 and D16 all signed** —
       one exchange, three work orders executed. **rmcp (a)**: rmcp 3.5.0
       with `server`/`macros`/`transport-io` plus `serde_json` with
       `preserve_order` (D14) is approved for the workspace, recorded in
       spec §6.3 beside the survey that measured it — the first heavy
       external dependency, asked and granted per U8's precedent; the
       approval is a decision, not a dependency, so `Cargo.lock` is
       untouched and the first M3 task adds it. **U20 (b)**: the registry
       format stays as t11 built it; M3's `repo_list`/`repo_show` answer
       from the handles the server holds, so t31's measured cold cost
       survives only on cold and cli paths, which is what they are for;
       (a) was declined on its real cost — the lock-order re-derivation,
       not the golden — and f63's lock-free-reader question stays M3's
       design call. **D16 (a)**: `status_summary(repo?, epic?)` joins
       §6's tool table and `task_list` rows gain their `blocked_by` ids
       with each blocker's status, with the arithmetic as signed text —
       "open" means non-terminal per kind, and `ready` is exactly D9's
       until-satisfied predicate, so the summary and the selection tools
       cannot disagree; §12 item 16 deleted and the list closed again;
       the domain aggregation task lands under M3's goal, since g6
       closed at g7 while the vote was open — the deviation from the
       work order's "under g6" is recorded in decisions.md D16's banner.
       U17 fully answered; U20 struck with its resolution. Landed as
       `9c7b30c` (backfilled in the next board commit).
- 2026-10-03 — **M3 decomposed: g8 + t32–t39 + g9** — the t26 pattern
       over the now-unblocked wire. Eight tasks, one reviewable commit
       each, file ownership disjoint: t32 owns the domain's D16 surfaces
       (`graph/summary.rs`, `list.rs`'s row type, the annotate lift from
       `items.rs`); t33 owns the todo-mcp skeleton and the workspace
       dependency line; t34–t38 own disjoint `tools/` modules and test
       targets; t39 owns the harness config. Every verify line is a
       runnable command — the protocol-level rule (§10: request bytes
       in, response bytes out, golden-pinned) is in every tool task's
       verify, and t38 owns the two cross-cutting pins: D14's (losing
       `serde_json/preserve_order` fails a golden, proven by mutation)
       and D15's (every tool's error carrier shaped identically). The
       concurrency design is named in t33's note — one serialization
       point behind the tool bodies, the guard never held across an
       await, the libraries staying sync per §2. import_md stays M4's
        per D12; pulling it forward is recorded as a one-task change.
        Landed as `dd45caf`.
- 2026-10-03 — **M3 t32: D16's domain aggregation** — the first M3
       commit, domain-only. `graph/summary.rs` (505 production lines):
       `Graph::status_summary()` and `Graph::status_summary_of_epic()`
       answer §6's signed arithmetic — counts by kind and status as
       named-field structs carrying their zeros (no maps: a fixed wire
       shape for D14, and scopes that compare field-for-field), the
       "open" totals as one top-level `OpenTotals` block (non-terminal
       per kind — epics/tasks `open`+`in_work`+`blocked`+`deferred`,
       decisions `open`), `DeferredSummary` reused verbatim over the
       same derived list the selection tools walk, so the summary and
       `next_task`/`next_items` cannot disagree about what "ready"
       means (pinned by type equality at both scopes), and docs as one
       total. Epic scope — the one veto-able reading, flagged in-tree
       with both alternatives named: the epic itself, its tasks (the
       induced order's set), and every decision holding any of them
       back, both `decision→epic` and `decision→task` gates, so the
       summary's gate picture agrees with `is_until_satisfied`'s
       consultation set; docs 0 (a doc has no epic field). `task_list`
       answers `Vec<TaskRow>` — the D16 guard's documented moment —
       each row the task plus its `blocked_by` ids with every blocker's
       status (`Blocker`: task-or-decision, the epic arm
       unrepresentable by §5.1's edge shapes); the annotate rule lifted
       from `items.rs`'s private fn into `Graph::blockers_of`, the
       two-run order (own predecessors, then epic gates, deduplicated
       at first occurrence, never globally merged) with one contract
       home that `items.rs`'s docs point at. Eleven tests in
       `graph::tests::summary`: the arithmetic against a hand-built
       graph (every status × kind), the open totals, the deferred
       split across until-target boundaries, the selection agreement,
       epic-scope membership including the negative half, determinism
       over shuffled rebuilds, seeded agreement with an independent
       naive recomputation, a per-seed coverage pin (seeds re-picked
       by grid measurement), the two-run row order against both
       `NextItem` and a naive builder, and blocker statuses tracked
       through mutations. Mutations M1–M6 run, observed failing,
       reverted, sha-verified — and M1 corrected the spec itself
       (f66: the select suite stays green, because the lift never
       touches `select.rs`'s own predicates). Dual review over three
       rounds — round 1's med was `list.rs`'s t29 judgment call (a)
       quoting mod.rs/lib.rs in the present tense, falsified by this
       diff (the claim class swept); round 2's med was the sweep's
       third instance, in `items.rs`'s judgment call (d) —
       "unreachable through the public API" false since the lift made
       the arm public; round 3 both ok on the final bytes. 178 domain
       tests (167 unmodified + 11), 333 workspace, every gate green
       including `cargo doc` under both invocations, the five goldens
       and `Cargo.lock` untouched, the six sha-pinned files
       byte-identical. f66/f67 recorded. Landed as `012af8c`.
- 2026-10-03 — **M3 t33: rmcp + the skeleton** — the first wire code.
       The dependency line lands at the workspace root (rmcp 3.5.0
       `server`/`macros`/`transport-io` plus `serde_json` with
       `preserve_order` per D14, and tokio/serde/schemars as the
       survey probe's exact companion set): `Cargo.lock` 27 → 93
       packages, +66, zero removals, zero version moves, every new
       package traced to the approved five — the one `-` line is
       serde_derive's `syn` dep-key rename on collision, verified
       independently by both reviewers. (The "26" in the t32 handover
       was stale by one; the pre-t33 count is 27.) `todo-mcp` grows
       from the M0 shell to lib + main: `lib.rs` — `TodoServer` and
       `ServerState` behind `Arc<std::sync::Mutex>`, the §8
       concurrency core: rmcp dispatches each request in its own
       `tokio::spawn`, so the state sits behind one serialization
       point, the guard taken and dropped inside the sync tool body,
       never across an await; the handles are
       `BTreeMap<RepoSlug, Box<dyn Store + Send>>` — the store's own
       lib.rs provisioned the boxed shape for M3's server ("if it
       would otherwise have to name the engine"), the `+ Send` is the
       trait-object bound rmcp's Send+Sync service requirement forces,
       and the engine type is never named in this crate. `registry.rs`
       — `repo_register` and its DTOs, the pipeline proof: params
       `{slug, name, path}` with `deny_unknown_fields` (the §7-prose
       divergence named in-tree as the veto-able reading), the D15
       carrier on every failure path, `registered_at` stamped by the
       server. `time.rs` — the clock: unix→civil→RFC 3339→
       `Timestamp::parse`, saturated at the domain's last instant
       9999-12-31T23:59:59.999999999Z (the year-9999 panic round 1's
       review caught — both reviewers' hand values were off by a day;
       the parse-oracle test is the arbiter). `main.rs` — `--root`
       required via lexopt, exit 2 usage / 1 serve fault, the exit
       contract narrowed to the measured behaviour: exit 0 only after
       a completed handshake, a pre-initialize close is exit 1. The
       initialize handshake answers 2025-11-25 with our own serverInfo
       (`todo-mcp` + `CARGO_PKG_VERSION` — unset, every client sees
       `rmcp/3.5.0`) and U16a's base instructions (the full text, one
       home); the 2026-07-28 discover flow measured **free** over one
       `serve(stdio())` and pinned both branches (with the two `_meta`
       keys → instructions; without → `-32602` naming them). 20 tests
       (6 time + 4 arg-parse + 10 protocol — the protocol suite spawns
       the real binary over raw stdio and freezes exact bytes: the
       handshake, `tools/list`, `repo_register`
       success/duplicate/contended/invalid, the unknown-field
       rejection, discover, determinism). Mutations M1–M5 run,
       observed failing, reverted, sha-verified. Dual review over two
       rounds — round 1 both changes_requested (the year-9999
       saturation; the exit-contract honesty and its zero coverage;
       the §2 engine-type boxing, ruled on per the store's provisioned
       shape), round 2 both ok on the final bytes. Every gate green;
       the three library crates untouched (`rg tokio` and the clock
       grep empty over them — §2's boundary proven); the goldens
       untouched. f68/f69/f70 recorded. Landed as `26528b4`.
- 2026-10-03 — **M3 t34: the query read tools + the first wire
       goldens** — `next_task`, `next_items`, `status_summary`,
       `task_show` and `decision_show` over the wire, each with a
       `deny_unknown_fields` params struct, a declared outputSchema,
       read-only annotations, and §5.2's total orders preserved through
       server-local DTOs. `src/dto.rs` (325 production lines) — the
       shared wire vocabulary: the vocabulary enums mirroring the
       domain's variant-for-variant with the same `snake_case`
       spellings (the outputSchema enumerates the closed sets),
       `TaskDto`/`DecisionDto` field-for-field in the domain's
       declaration order, ids/timestamps/dates as their canonical
       strings. `src/query.rs` (519) — the five tools and their
       DTOs; `NextItemDto.blocked_by` carries the blocker ids with the
       **two-run order** intact (own predecessors, then epic gates,
       dedup at first occurrence — the contract stated in the field
       doc, t28's ruling). `src/lib.rs` (305) — `ToolError` (the one
       error-mapping layer: `StoreError`/`DomainError` → their codes,
       `ValueError` → `invalid`), `ServerState::resolve` (handles hit
       → cached; miss → registry consult, an unlisted slug is
       `unknown_id`, then open + cache — the cold open amortized to
       once per repo per process, U20(b)), the copied-store
       cross-check on the open path (mirroring the store's
       `repo_node`), and `structured()` moved beside `error_carrier`
       as the shared success arm (f74: the dedup edited registry.rs,
       an unowned file, ratified on review). The router composes:
       `query.rs`'s own `#[tool_router]` block merged into the
       handler's router (`ToolRouter::merge` — the mechanism that
       makes the per-cluster file layout work). **The golden
       discipline starts here**: `tests/golden/` with one file per
       tool holding the full JSON-RPC response lines (success + error
       responses, fixed request ids), the store's `assert_golden`
       copied verbatim, the fixture store hand-built with fixed
       caller-passed timestamps (deterministic — the shapes: two
       epics, the full status vocabulary, deferred-ready and
       deferred-held, a decision gating both a task and its epic —
       the dedup case on the wire — a signed decision, `relates`,
       until-targets, a doc), and the drive interactive (one request,
       one response — f73: rmcp's concurrent dispatch reorders under
       rapid-fire, so arrival order is t38's concurrency property,
       not a byte property). Thirteen tests in `read::query` — the
       goldens, the domain agreement (the wire's parsed answers
       compared field-for-field with the domain's in-process answers),
       the unknown repo/epic/task/decision and malformed-id error
       paths, the unknown-field rejection, `limit`'s default, and the
       tools/list schema-discipline test (outputSchema on all six
       tools, no description carrying a newline — f72's rule pinned).
       Mutations M1–M6 run, observed failing, reverted, sha-verified —
       M2 (dropping `preserve_order`) killed all five goldens, D14's
       pin biting from the first golden. Dual review over three
       rounds: round 1 both changes_requested (the unpinned
       outputSchema/description rule — f72's verify line was false as
       written; the dropped envelope pin; f70's class twice; the
       structured() duplication; resolve's copied-store adoption; the
       fixture-spelled example ids), the fix round landed all six, and
       the binding round re-established both sign-offs on the exact
       final bytes after the re-bind round's sha reports proved
       unreliable (both reviewers' reported shas mismatched the tree;
       a clean confirmation round recomputed all thirteen from the
       repo at verdict time — the content verdicts stood, the
       bindings had not). Every gate green; the goldens regenerate
       byte-identical under `UPDATE_GOLDEN=1`; `Cargo.lock` and the
       three library crates untouched. f71/f72/f73/f74 recorded; U23
       opened for the unpinned copied-store check. Landed as `c7308aa`.
- 2026-10-03 — **M3 t35: the listing read tools + §6.2's dump** —
       `task_list` with D16's annotated rows, `epic_list`,
       `decision_list`, `doc_list`, and `graph_show`, each with a
       `deny_unknown_fields` params struct, a declared outputSchema,
       read-only annotations and a per-tool wire golden (five new:
       `task_list.v1`, `epic_list.v1`, `decision_list.v1`,
       `doc_list.v1`, `graph_show.v1` — ten goldens in the dir now,
       the t34 five byte-identical throughout). `src/listing.rs`
       (683 production lines, over the 600 target by recorded
       exception — **U24**, the split named) holds the cluster;
       `src/dto.rs` gains `EpicDto`/`DocDto`/`RepoDto` and the input
       direction of `StatusDto`/`TaskKindDto` (`Deserialize` + total
       inverse `From` impls — `task_list`'s `status`/`kind` params are
       typed enums, so a wrong vocabulary spelling is rmcp's own
       codeless argument fault, **not** `invalid`: §6.1's consequence
       text scopes `invalid` to values this server parses — the
       `ValueError` class — and the input schema itself enumerates the
       closed set; the boundary is pinned by a dedicated test so a
       future `serde(other)` leniency cannot move it silently).
       **`BlockerDto` is a tagged enum** — `{"shape":"task","id":…,
       "status":"open"}` — following `BlocksEdgeDto`, the project's one
       persisted tagged-union spelling (`tag = "shape"`); measured in a
       scratch probe at dispatch: key order `[shape, id, status]` under
       `preserve_order`, and schemars renders a `oneOf` with per-variant
       `const` discriminants and `$defs` enumerating both closed status
       sets. **The dump DTO uses `#[serde(flatten)]`**, also measured
       at dispatch: the flattened node's fields serialize first, in the
       node's declaration order, then the adjacency fields — D14's
       declaration-order pin holding by construction, and schemars
       merges the schema. Every node carries exactly the adjacency
       fields that can be non-empty for its kind (tasks:
       `blocked_by`/`blocks`/`relates`; epics: `blocked_by`, their own
       gates; decisions: `blocks`, what they gate; docs: none), every
       edge visible from both endpoints, no edge table — and each
       field's doc states its total order (the `BlockEnd` endpoint
       order the domain documents). **The thin listings answer
       id+title rows** — a veto-able reading of §6's silence (the
       alternative, bare id arrays, mirrors the domain surface
       exactly), taken because §6's own client-flow sentence needs the
       listing to carry names an agent can show; pinned by the goldens
       so a veto is a named diff. `relates` rides per task in the dump
       (also veto-able, same pinning). **`graph_show`'s three scopes**:
       no args = every registered repo in sorted-slug order (the
       registry's set — a dump may pay the cold open, amortized once
       per repo per process); `repo` = that repo; `epic` = the subtree
       plus the gating decisions, the repo derived from the epic id
       when absent, the explicit repo authoritative when present, a
       cross-repo epic answering the domain's own `unknown_id`. The
       optional `repo` is §6.2's all-repos semantics, **not** the
       default-slug case — recorded in the params doc so t37 does not
       "fix" it. **One domain rider**: `Graph::gating_decisions_of_epic`
       in `summary.rs` — the epic scope's gate set exposed publicly
       because the server must not re-derive the composition from
       adjacency (the t29 measurement, the reason t32 lifted
       `blockers_of`); one public method + one pin test, nothing else
       in `todo-domain`. The fixture gained a second repo **alpha**
       (minimal, timestamped *after* `fixture` but sorted before it, so
       the all-repos order pin distinguishes slug order from both
       arrival and timestamp order), and the three suite-generic test
       helpers (`call`, `structured`, `assert_carrier`) lifted from
       `read/query.rs` into `common/mod.rs` — f54's anticipated "next
       copy is M3's" moment, decided by the shared module's next owner
       as its rule prescribes; the query suite's half is pure deletion
       plus the `use` line. 14 tests in `read::listing` (the goldens,
       the domain agreement, the filter semantics, the two-run order
       with statuses, the epic scope, the all-repos order, the error
       paths, the vocabulary boundary, determinism) + the domain pin;
       381 workspace tests. Mutations M1–M6 run by the implementer and
       independently re-run by the primary in a scratch copy (kill sets
       matching; the spec's named sets understated — f75). Dual review
       over two rounds plus a binding round: round 1 primary ok (two
       lows: the unpinned vocabulary boundary, the all-repos order
       pin's slug-vs-timestamp ambiguity) / secondary
       changes_requested (three meds: the undocumented adjacency
       orders, a "five lines" figure in a doc comment — f30's class —
       and the domain pin test's section banner misplaced; two lows:
       the exception line's shape, the stale two-router merge comment);
       the orchestrator applied all seven fixes itself per the t11/t30
       precedent, and both reviewers re-bound ok on identical
       final-byte shas computed independently at verdict time. f78/f79/
       f80 recorded (the spec's kill-set understatement, its "0
       filtered out" filter wording, and its D1/D2 contradiction that
       forced the f80 walker fix in `query.rs` — ratified). Landed as
       `187ec44`.
- 2026-10-04 — **M3 t36: the node-CRUD mutation tools** —
       `epic_add`/`epic_update`, `task_add`/`task_update`,
       `decision_add`/`decision_update` and
       `doc_register`/`doc_set_path`/`doc_update` over the wire: every
       mutation already on the `Store` trait, the wire converting and
       deciding nothing (§2 — no domain or store change, `Cargo.lock`
       and the three library crates untouched). Writes return the new
       state (§6's frame). `src/nodes.rs` (769 production lines, over
       the 600 target by recorded exception — **U25**, the split
       named) holds the cluster: the add params mirroring the domain
       drafts field-for-field (`status` required — a defaulted status
       would hide which closure discipline the caller is reaching
       for; `references` absent meaning the empty set), the update
       params mirroring the store's patch DTOs field-for-field under
       RFC 7386's three-state reading (absent untouched, `null`
       clears, a value replaces — a server-local `double_option`
       copy, the store's own being private), `task_update`'s `epic`
       mapping straight onto `TaskPatch::epic` (§5.1: reparenting is a
       validated update, no separate verb), and `doc_update` carrying
       no `path` (the committed D10's audited verb `doc_set_path`
       owns it). **The clock discipline**: one `time::now()` read per
       tool call, taken after the parse and the resolve — an add
       stamps `created`/`updated` with the same instant, an update
       stamps `updated` alone; no injection seam. The consequence
       splits the cluster by clock-freeness: epics carry no
       timestamps, so `epic_add`/`epic_update` are golden-pinned —
       **the first writer goldens** (`epic_add.v1`, `epic_update.v1`,
       the property that decides being static bytes, not readness) —
       while the seven stamped writers assert in-test, byte-exactly
       around the extracted stamp (the t33 `registered_at_of` pattern
       generalized to suite-local `created_of`/`updated_of`); the
       principle is stated in the write suite's module doc so t37's
       writers know which side they fall on. **The closure discipline
       is the domain's, carried by the carrier**: `verify`,
       `evidence`, `note` and `resolution` are optional params whose
       absence answers `missing_verify`/`missing_evidence`/
       `missing_note`/`missing_resolution` through D15's carrier —
       never schema-required fields, which would demote §6.1's own
       example into rmcp's codeless argument fault. The vocabulary
       boundary carried from t35: enums typed in the params (a wrong
       spelling is the argument fault), ids/slugs/dates/references
       parsed in the body (`invalid` before the store is touched, and
       a malformed id outranks an unknown repo — parse before
       resolve). `src/dto.rs` (503) gains the input direction of
       `DecisionStatusDto`/`ConfidenceDto`/`PriorityDto`/
       `ReferenceKindDto` (Deserialize + total inverse `From` impls)
       and `ReferenceParam`, the input mirror of `ReferenceDto`
       converted per entry through the domain's validating
       constructor. `tests/write.rs` + `tests/write/nodes.rs` — the
       write target, fresh fixture per test (the writes mutate the
       store), the stamped writer families ending with the in-process
       reopen
       asserting the mutation is on disk (the t33 precedent: the wire
       proved the write, the reopen proves the log). Twelve tests:
       the golden pair, the new-state-and-persists trio per family,
       the three-state drive, the patch round trip (byte comparison
       over the raw structuredContent, not parsed values — map
       equality is order-insensitive and D14's pin is an order pin),
       the parse-before-resolve sweep, and the clock-free pair's
       byte-identity across two fresh spawns. The two f71-class name
       arrays (`tests/read/query.rs`'s schema-discipline test,
       `tests/protocol.rs`'s composition pin) grew from eleven to
       twenty tools. 393 workspace tests. Mutations M1–M6 run,
       observed failing, reverted, sha-verified (kill sets reported;
       the spec's named sets were wrong in both directions — M1's
       test 10 unreachable, `DocUpdateParams` having no three-state
       field, and M3's add half unrepresentable, an add having no
       pre-write state). Dual review over two rounds plus a binding
       round and a commit-round confirmation: round 1 primary ok (two
       lows: the write suite's doc overstating the reopen claim, a
       stale tracker sha) / secondary changes_requested (one med:
       `ReferenceParam` lacked `deny_unknown_fields`, so a nested
       typo'd key was silently dropped while every top-level params
       struct rejects unknown fields — §6.1's silent-drop consequence
       applies to nested input objects; four lows: a figure in the
       exception line, a dead `let _`, the M2 claim that did not
       reproduce, and common/mod.rs's stale-claim siblings recorded
       for t37); the orchestrator applied all five fixes plus the
       f70 sweep (three spec-local D-ids in `nodes.rs`'s module-doc
       headers, swept to self-contained wording before review), and
       both reviewers re-bound ok on identical final-byte shas
       computed independently at verdict time — then routed five
       commit-round lows to the orchestrator (the parent doc's
       reopen claim, the nested-field pin reusing the suite's
       `assert_argument_fault` helper, the `and_repaints` typo, the
       board's pre-fix dto.rs figure, and the tracker recordings),
       all applied and confirmed in a third round with the production
       files byte-identical throughout. f85–f89 recorded (the
       verify-clear-is-a-rejection discovery, three kill-set claims,
       and the M2 stale-stamp claim — the fifth consecutive task
       spec with a claim that did not reproduce). Landed as
       `«t36-hash»`.

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
      answer this item was placed here for. **Both were signed (a) the same
      day**, so §6.1/§6.2/§8/§10 now carry the carrier and the key order as
      normative text and §12 is closed again. Three facts M3 must design
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
- [x] **U15 · Scope or fix `Store::snapshot`'s "changes nothing on disk".**
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
       is the measurement to re-read first. **Resolved 2026-10-02 by t30,
       remedy (a) — the sentence is scoped, the code is not touched.**
       `store.rs`'s `snapshot` doc now claims a failed snapshot changes
       nothing **in place** (the log, the graph and the snapshot already on
       disk untouched) and says what a failure *after* the temp file's
       creation does leave, why the debris is inert (its name is never the
       snapshot's, and nothing in this crate reads a store directory's
       contents), and how it is reused — the same process's next snapshot of
       this store truncates it, a dead process's is left behind. It names the
       pin instead of counting what the pin counts. Remedy (b),
       cleanup-on-failure, was rejected on measurement rather than
       preference: §4.3's signed text promises only temp+`fsync`+rename "so
       they are never torn", the debris assertion above would have had to
       invert, and the change reaches four write sites (`snapshot.rs`, two in
       `log.rs`'s `compact`, one in `repair.rs`) while adding a failure mode
       inside the one path promised not session-fatal. Review then found the
       class sweep's grep had been too narrow — it looked for "nothing
       written / nothing on disk" and not for "nothing was **opened**" — and
       two more claims of the same class: `error.rs`'s `Locked` doc and
       `layout.rs`'s `Unnamed` rationale both said "nothing was opened, read
       or written", while `open_and_try_lock` opens the lock file, with
       `create`, before `try_lock` — that is how the contention is
       discovered. Both fixed in the same commit, on f55's sweep-the-class
       rule rather than deferred to a board item, since t30 *is* the store
       touch a deferral would wait for.
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
- [x] **U17 · Decide which harness tools become MCP tools.** The intent,
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
      the wire does. **Answered 2026-09-30 by the M2 decomposition**, in the
      two halves this item separates. The **mapping** needs no decision,
      because it is already signed: `attention.py`'s "what is next and what
      is blocked" is `next_task`/`next_items` with D11's derived weight and
      D9's deferred summary (t27/t28), `progress.py`'s graph is §6.2's
      `graph_show`, `tasks.py` stays this repo's session tracker (§9 bars
      dogfooding before M3), and the HTML/browser rendering stays a
      client-side concern (§11: "clients render"). The **roll-up** — totals by
      kind and status, and per open item what it waits on — is the one piece
      §6's list does not carry, and adding it amends D5's signed list, so it
      is drafted as **D16** with recommendation (a): a
      `status_summary(repo?, epic?)` tool plus `blocked_by` on `task_list`
      rows, whichever option wins. Its `On sign-off:` work order adds a
      domain aggregation task under g6 if (a) or (b) is signed, and records
      that the roll-up stays a script if (c) is. No task may implement a
      roll-up before then. **D16 signed (a) 2026-10-03** — the roll-up is
      behind the wire: §6 gains `status_summary` and the `task_list`
      annotation with the arithmetic as signed text, and the domain
      aggregation task lands under M3's goal (g6 closed at g7 while the
      vote was open; decisions.md D16 records the deviation). U17 is
      fully answered.
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
- [x] **U19 · One word in `repair.rs`'s module doc.** Its `Err` rationale
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
      takes one takes both. **Resolved 2026-10-02 by t30**: the one word,
      exactly as prescribed, and verified against the code rather than the
      report — the parse fault propagates as an `Err` at `repair.rs:250`
      (`load_snapshot(snapshot_path)?`), while a snapshot-origin *reconcile*
      fault arrives through `replay` as an unpositioned `corrupt_store` and is
      refused as `Unrepairable { offset: None }` at `:260-262`, which the
      module doc's classification-order item 2 (`:46-49`, "a fault with no
      offset — a semantic disagreement between two files") already states in
      terms. So the narrowing removes the contradiction with the paragraph
      below it instead of needing a new clause, and `Unrepairable.offset`'s
      field doc ("a position in the log, never in a snapshot") stays true of
      the `None` case.
- [x] **U20 · Decide whether the registry caches `name`/`path`, now that a
      cold `repo_list` is priced.** t11's narrow reading — the registry holds
      only `{registered_at, slug}` and §7's mutable `name`/`path` live solely on
      each store's `Repo` node — was accepted with its cost recorded rather than
      hidden: a cold `repo_list` must open every store, and extending the
      registry later is a golden-visible format change, cheap before M3 and
      expensive after. The instruction at the time was that the revisit be
      driven by a measurement and not a preference (U1's precedent), and **t31
      took it**: a cold listing is **O(total history), not O(repos)** — ~45–61
      µs per seed-only repo but ~238–270 µs per repo with a 200-event log in
      release (≈1.2 µs per event), against a registry floor of 12–59 µs that
      does not grow with N, and a first call 1.2–2.9× the steady mean. At §4.1's
      top of scale that is ~120 ms for one repo and ~1.2 s for ten, steady. The
      numbers are the *optimistic* case for opening every store, because a cold
      page cache inflates the whole listing far more than the floor.
      Three ways out, and the choice is the user's. **(a) Extend the registry**
      with `name`/`path`: a golden-moving change to `registry.v1`, two writes
      per `repo_set_path` instead of one — and it **kills the lock-order
      argument that makes t11's ordering safe**, which `layout.rs`'s module doc
      already names in terms: a mutable field in the index means
      `repo_set_path` needs the registry lock *while holding the store lock*,
      acquiring in the forbidden order, so the format may only be extended
      together with a re-derived lock order. **(b) Leave the format alone** and
      have M3 answer `repo_list` from the handles it already holds, which
      removes the cost for the server and leaves it only on a cold or cli path —
      the reading t11's hand-off assumed. **(c) Accept the cost** on §4.1's own
      scale argument that replay is already cheap. (a) is not the cheap option
      it looks: the lock order, not the golden, is what makes it expensive.
      Reproduce the measurement with `cargo test -p todo-store --test layout --
      --ignored --nocapture`; the grid, both profiles and the caveats are in
      t31's §3 entry and tracker evidence, and **f64** names the noisy cells.
      Due before M3 builds the wire, not after — and note that
      `tools/attention.py` renders this **P3**, because a §4 item carries no
      `Gates` field for it to derive a priority from (f65); §1's blocking table
      above is where the deadline lives. **Resolved 2026-10-03, option (b) —
      the format stays.** The registry keeps only `{registered_at, slug}`;
      M3's `repo_list`/`repo_show` answer from the handles the server already
      holds, so the measured cost survives only on cold and cli paths, which is
      what they are for. (a) was declined on its real cost — the lock-order
      re-derivation, not the golden — and stays reachable later as a format
      change if a cli-heavy workflow ever needs it. f63's lock-free-reader
      question is untouched by this: it was one of the three answers and
      remains M3's design call with its unsigned §4.3 reading.
- [ ] **U21 · Land the duplicate-key check for `agent-rules.yaml`, or close
      it wontfix.** f44, promoted out of the tracker at the g7 sweep because
      the generalisation must not survive only in a gitignored file:
      `tools/agents-sync.py` loads the config with plain `yaml.safe_load`,
      and PyYAML keeps the *last* of duplicate keys, so a second key
      silently shadows the first as dead config that reads like live
      config — f42 caught exactly this once (two top-level `commands:`
      keys, the whole build-command block dead). The config is clean today
      (re-verified at g7 with a duplicate-key-detecting loader: no
      duplicate anywhere; `commands` and `build_commands` both resolve),
      but nothing prevents a recurrence. Either the loader refuses a
      duplicate key anywhere in the file — proved by feeding it a copy
      with one spliced in, `--check` still green on the real config — or
      the check is declined and this item records that decision.
- [ ] **U22 · `progress.py` does not render open decisions.** f45,
      promoted at the g7 sweep for the same reason: the page reads board
      §2/§4/§5 and the tracker, so an open D-entry has no row — measured
      at g7, the page renders **D16**, the highest-priority open thing in
      the project, zero times. D16 is drawn as a graph node by hand (with
      a comment in the source saying its status is authoritative and must
      be flipped on sign-off), but the open-items table has no row and no
      waits-on cell for it. Either derive open D-entries from
      `decisions.md` — `attention.py` parses the same table, so the data
      is already machine-readable — or state the scope limit in the page
      footer, so a reader does not take the page for the whole picture.

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
      `replay_step`; append + snapshot-aware replay + `compact`). Both now
      carry their module-doc exception line, landed with t13a.
      **Re-measured at the M2 decomposition (2026-09-30): unchanged**, 812
      and 772 under the same awk measure, and still no split. M2 will not
      show these two's seams either — its query layer lands in new
      `todo-domain` modules (`select.rs`, `items.rs`, `list.rs`) and its one
      store task (t31) touches the registry, not the log or the event
      schema. Revisit at M3, whose wire mapping touches the store's error
      surface, or at the next commit that grows either file — whichever
       comes first.
- [ ] **U23 · The copied-store cross-check on the server's resolve path is
      unpinned.** t34's `ServerState::resolve` gained the one-line check
      `store.graph().repo().slug != slug → corrupt_store`, mirroring
      `todo-store`'s `repo_node` cross-check (layout.rs) so a copied store
      directory is refused rather than adopted — f61's recorded
      reader disagreement made client-visible otherwise. The branch is
      live (verified ad hoc: a store holding repo `aaa` opened under
      slug `bbb` answers `corrupt_store` naming both) but no test pins
      it, and the check re-spells the store's private detail string
      with no compile-time link, so a future resolve/handle-cache
      change could drop it silently. The remedy is a swapped-store
      fixture test (register `aaa`, copy its directory to `bbb`'s slot,
      query through the wire) — t37's registry cluster or t38's wire
      suite, whichever owns the fixture machinery first.
- [ ] **U24 · `todo-mcp/src/listing.rs` is over the 600-line target.**
      683 production lines (the awk measure; the `#[cfg(test)]` clause
      does not apply — the file holds no test module). Over target
      because the listing cluster's dispatch pins all five tools,
      their params and their answer DTOs — `graph_show`'s dump
      builders included — in one file, the `query.rs` shape the t35
      spec named; the cap of 1000 holds. The split, if the cluster
      ever grows a sixth tool or the dump grows a second scope shape,
      is the dump half (`GraphShowDto`, `RepoDumpDto`, `EpicGroupDto`,
      `TaskNodeDto`, `DecisionNodeDto` and their builders) into its
      own module — a named diff for whoever next owns the file.
- [ ] **U25 · `todo-mcp/src/nodes.rs` is over the 600-line target.**
      769 production lines (the awk measure; the file holds no test
      module). Over target because the mutation cluster's dispatch
      pins all nine tools, their params and their draft/patch
      conversions in one file, the `query.rs`/`listing.rs` shape the
      t36 spec named; the cap of 1000 holds. The split, if the
      cluster ever grows a tenth tool, is the doc family
      (`doc_register`/`doc_set_path`/`doc_update` and their
      conversions) into its own module — a named diff for whoever
      next owns the file.

## 6. Session log

Newest first. One entry per session, written for someone who was not
here. The final entry of a session is the handover
(`.agents/skills/todo-next-imp/SKILL.md` defines the shape).

### 2026-10-03 — t35 landed: the listing read tools and §6.2's dump

The session opened on `/todo-next-imp` with the previous handover
naming one next step — dispatch t35 — and the user said ok. Two
pieces of bookkeeping preceded the dispatch: f72's stale `active`
status was closed with a fresh verify run (the f41 class — its pin
had landed with t34's fix round and the status was never swept), and
the spec was pre-written at `tmp/t35-spec.md` (14 decisions, 13
required tests, 6 mandatory mutations, baseline shas pinned at
`9dc9806`). The two design surfaces the t34 handover named were
settled by **measurement before dispatch**, not assumption: a scratch
probe proved `serde(flatten)` preserves declaration order under
`preserve_order` (the flattened node's fields first, then the outer
struct's — D14's pin holding by construction, which is what made the
dump DTO's shape decidable), and a second probe proved the
internally-tagged `BlockerDto` serializes `[shape, id, status]` with
schemars rendering a `oneOf` + `$defs` enumerating both closed status
vocabularies. The implementer landed in one round; dual review ran
over two rounds plus a binding round — round 1 primary ok with two
lows, secondary changes_requested with three meds and two lows; the
orchestrator applied all seven fixes itself (the t11/t30 precedent
for small exactly-prescribed fixes), and both reviewers re-bound ok
on **identical final-byte shas computed independently at verdict
time** — the t34 binding-round discipline holding on its first
reuse. §3's entry carries the delivery.

Findings: **f78** (the spec's mutation kill sets understate — the
named tests fail, but the true sets are supersets; 5.6's class one
level down), **f79** (the spec's "0 filtered out" filter wording —
the read binary holds both suites after t35, so the count to quote is
"13 passed / 13 filtered out"; plus its siblings: a named "time" test
target that does not exist, and the spec's golden shas mapped to the
wrong files read positionally), **f80** (the spec's D1/D2
contradiction — EpicDto's prose `description` field became an
outputSchema *property* whose value is an object, and the
string-expecting description walker panicked; the fix keeps the pin's
strength and was ratified). **U24** opened: `listing.rs` is over the
600-line target by recorded exception, the dump half named as the
split. And a t36+ spec-authoring note from the secondary: baseline
shas are `shasum` file-SHA-1 prefixes, not git blob hashes — name the
tool in the spec, or a reviewer using `git hash-object` reads false
MOVEs.

**Handover.** Workspace: `main`, clean after the board commit, four
crates, **381** tests, rustc 1.98.1, MSRV 1.95; `Cargo.lock` 93
packages (unchanged — t35 added no dependency). **Next step: dispatch
t36** — the node-CRUD mutations: `epic_add`/`epic_update`,
`task_add`/`task_update` (the closure discipline mapped to §6.1's
codes — `missing_verify`/`missing_evidence`/`missing_note` through
D15's carrier), `decision_add`/`decision_update`
(`missing_resolution`), `doc_register`/`doc_set_path`/`doc_update`;
writes return the new state. Its spec wants pre-writing (the
t32–t35 pattern); the design surfaces to settle there: **the clock
question t33 handed forward** — a mutating tool's success response
carries the server-stamped timestamp, so it cannot be byte-golden-
pinned the way read tools can; the options are error-carrier goldens
for writers plus parsed-field assertions on success, a fixed-clock
injection seam, or another shape — and the params/patch DTOs (the
store's RFC 7386 three-state patch DTOs are the persisted precedent;
the wire's patch spelling is t36's to declare). The pieces it
inherits are landed: `dto.rs`'s node mirrors and the vocabulary enums
(now with the input direction), `ToolError`, `resolve`, the router
composition, the golden discipline, the fixture. **Blocked on you:
nothing.** Non-blocking and yours: U16 (t39's half), U21, U22, U18
(parked), U23 (t37/t38's swapped-store fixture test), U24 (new — the
`listing.rs` split rides its next owner), and the two one-line spec
asks — §10's gate enumeration and §6.3:474's feature condition
(f69). **Unverified:** no client has talked to this server; the wire
is measured by test harnesses only — the opencode smoke is t39.
Tracker: `tools/tasks.py check` exits 0 on the two expected lines
(g8 open with t36 next; g3 stands). Doc gates: check-links,
`attention.py --check`, `agents-sync.py --check` green; agents-index
untouched (no indexed doc moved in the code commit; this board
commit backfills the hash).

### 2026-10-03 — t34 landed: the query read tools and the first goldens

The session continued from t33's landing: the user said go on t34,
the spec was pre-written at `tmp/t34-spec.md`, and the implementer
landed in one round (one spawn death, the short-prompt retry landed).
Dual review ran over three rounds: round 1 both changes_requested
(the unpinned outputSchema/description rule, the dropped envelope
pin, f70's citation class twice, the structured() duplication,
resolve's copied-store adoption, the fixture-spelled example ids);
the fix round landed all six; and a **binding round** — the re-bind
round's sha reports from both reviewers mismatched the repository
tree, so a clean confirmation round recomputed all thirteen shas
from the repo at verdict time and both re-bound ok. The lesson is
recorded in the tracker: a reviewer's bound shas are only evidence if
computed from the repo at verdict time, never from a scratch copy or
mid-mutation. §3's entry carries the delivery.

Findings: **f71** (the spec's file ownership contradicted its own
gate — protocol.rs was necessarily edited, resolved toward the gate
and ratified), **f72** (schemars carries output DTO struct docs into
the outputSchema description verbatim — the single-line rule binds
them too, now pinned by the tools/list test), **f73** (the golden
drive is interactive — arrival order is t38's concurrency property),
**f74** (the structured() dedup edited an unowned file, ratified).
**U23** opened: the copied-store cross-check on resolve is live but
unpinned; a swapped-store fixture test is t37/t38's.

**Handover.** Workspace: `main`, clean after this commit, four
crates, **366** tests, rustc 1.98.1, MSRV 1.95; `Cargo.lock` 93
packages (unchanged since t33 — t34 added no dependency). **Next
step: dispatch t35** — the listing cluster: `task_list` (D16's rows
with `blocked_by` and blocker statuses), `epic_list`,
`decision_list`, `doc_list`, and `graph_show` (§6.2's dump shape
over the canonical serialization). Its spec wants pre-writing (the
t32/t33/t34 pattern); the pieces it inherits are all landed: the
golden discipline (extend `tests/golden/`, the fixture, the
interactive drive), `dto.rs` (extend with the listing DTOs —
single-line struct docs, f72), `ToolError`, `resolve`, the router
composition. Two design notes for its spec: `graph_show`'s wire shape
is §6.2's canonical JSON (the domain's own serialization spelling —
decide how the DTO mirrors the grouped-by-epic dump), and
`task_list`'s rows carry D16's blocker statuses (t32's `TaskRow`/
`Blocker` are the domain surface). t36–t38 follow; t39 is the smoke.
**Blocked on you: nothing.** Non-blocking and yours: U16 (t39's
half), U21, U22, U18 (parked), U23 (new, t37/t38), and the two
one-line spec asks — §10's gate enumeration and §6.3:474's feature
condition (f69). **Unverified:** no client has talked to this
server; the wire is measured by test harnesses only. Tracker:
`tools/tasks.py check` exits 0 (g8 open with t35 next; g3 stands).
Doc gates: check-links, `attention.py --check`, `agents-sync.py
--check` green; agents-index untouched (no indexed doc moved).

### 2026-10-03 — t33 landed: rmcp + the skeleton

The session continued from t32's landing: the user said go on t33,
the spec was pre-written at `tmp/t33-spec.md` (twelve decisions, the
probe at `tmp/rmcpprobe` as the working reference), and the
implementer landed in one round — no spawn deaths this time. Dual
review ran over two rounds: round 1 both changes_requested (the
year-9999 saturation in the clock, the exit contract's honesty and
its zero coverage, and the §2 engine-type question — settled by
boxing the handles into `Box<dyn Store + Send>`, the shape the
store's own lib.rs provisioned for M3's server), round 2 both ok on
the final bytes. §3's entry carries the delivery.

Three findings a cold reader should know: **f68** — T4's post-exit
store open failed once on the first full-suite run, unreproduced in
fifty-plus runs since, the panic now carries the `StoreError` so a
recurrence is capturable; **f69** — D14's `preserve_order` reaches
schemars too, so §6.3:474's "properties is alphabetical" is
feature-conditional (under D14, declaration order — the inputSchema
properties included); **f70** — task-spec-local decision ids collide
with committed `decisions.md` ids, so committed doc comments must
never cite a spec-local D-id (the f62/5.6 class; swept).

**Handover.** Workspace: `main`, clean after this commit, four
crates, **353** tests, rustc 1.98.1, MSRV 1.95; `Cargo.lock` now 93
packages — the rmcp tree is in (the pre-t33 count was 27; the "26"
in the t32 handover was stale). **Next step: dispatch t34** — the
query read tools: `next_task`, `next_items`, `status_summary`,
`task_show`, `decision_show`, one params struct per tool with
`deny_unknown_fields`, outputSchemas declared, §5.2's total orders
preserved, and the first per-tool wire goldens. Its spec wants
pre-writing (the t32/t33 pattern); two design decisions live there:
the wire-golden discipline's shape (D14's declaration-order pin per
tool, `UPDATE_GOLDEN` over a new golden dir), and the clock-field
question t33 handed forward — a mutating tool's success response
carries the clock-stamped `registered_at`, so it cannot be
byte-golden-pinned the way read tools can; t34's spec decides (read
goldens only, error-carrier goldens for writers, or another shape).
t35–t38 follow; t39 is the smoke. **Blocked on you: nothing.**
Non-blocking and yours: U16 (t39's half remains), U21, U22, U18
(parked), and two one-line spec asks whenever you want them — §10's
gate enumeration names four where every operative home runs cargo
doc twice, and §6.3:474's "properties is alphabetical" wants its
feature condition stated (f69). **Unverified:** no client has talked
to this server — the wire is measured by test harnesses only; the
opencode smoke is t39. The discover flow is measured free and pinned
both branches. Tracker: `tools/tasks.py check` exits 0 (g8 open with
t34 next; g3 stands). Doc gates: check-links, `attention.py
--check`, `agents-sync.py --check` green; agents-index untouched (no
indexed doc moved in this commit).

### 2026-10-03 — t32 landed: D16's domain aggregation

The session opened on `/todo-next-imp` with the previous handover
naming one next step — dispatch t32 — and the user said go. The spec
was pre-written at `tmp/t32-spec.md` (the t27/t29 pattern: twelve
decisions, thirteen required tests, six mandatory mutations, the six
pinned-file shas as gate lines); the implementer spawn died twice
before landing — the P-LIVE-001 short-prompt remedy worked on the
third try — and dual review ran over three rounds to sign-off, both
reviewers bound to the final bytes. §3's entry carries the delivery.

Two findings a cold reader should know: **f66** — the spec's M1
kill-set claim half-reproduced (the select suite stays green; a
mutation's named kill set is a claim to verify, pitfalls 5.6's
class), and **f67** — `select.rs`'s consumer enumeration is stale
post-D16 (`summary` now also carries `DeferredSummary` and calls
`is_until_satisfied`); the file is sha-pinned, so the fix belongs to
its next owner, the f58 precedent.

**Handover.** Workspace: `main`, clean after this commit, four
crates, **333** tests, rustc 1.98.1, MSRV 1.95; `Cargo.lock` still
26 packages — rmcp is approved but not added, which is t33's first
act. **Next step: dispatch t33** — rmcp + the skeleton: the
dependency line at the workspace root (rmcp 3.5.0
`server`/`macros`/`transport-io` plus `serde_json` with
`preserve_order` per D14), `ServerHandler` with our own serverInfo
and U16a's base instructions, the initialize handshake (2025-11-25),
the §8 concurrency core (one serialization point behind the tool
bodies, the guard never held across an await), and `repo_register`
end-to-end as the pipeline proof. Its spec wants pre-writing (the
t32/t27/t29 pattern); the concurrency core and U16a's instructions
text are the two design surfaces. There is no alternative next step:
t34–t38 all need t33's skeleton. **Blocked on you: nothing.**
Non-blocking and yours: U16 (lands with M3), U21, U22, U18 (parked),
and spec §10's gate-enumeration line (four named where every
operative home runs cargo doc twice — one line to sign whenever).
**Unverified:** no wire exists — D14/D15/D16 now have domain code
behind them but no wire code, and no client has talked to this
server; the rmcp approval is a decision on paper until t33 lands.
Tracker: `tools/tasks.py check` exits 0 (g8 open with t33 next; g3
stands). Doc gates: check-links, `attention.py --check`,
`agents-sync.py --check` green; agents-index untouched (no indexed
doc moved in this commit).

### 2026-10-03 — the M3 gates cleared and M3 decomposed

The session ran the g7 exit gate to completion (M2 closed; four commits
`c35c351`/`73faff5`/`d30cf08`/`3a112c5` — the g7 entry below carries the
full account), briefed the user on the three open decisions, and the user
signed all three as recommended: **rmcp (a)**, **U20 (b)**, **D16 (a)**.
The work orders are executed (§3's entry, `9c7b30c`): §6 gains
`status_summary` and the `task_list` annotation with the arithmetic as
signed text, §12 is closed again, U20 is struck with its resolution, and
the rmcp approval is recorded in §6.3 — a decision, not a dependency;
`Cargo.lock` still holds 26 packages and t33 adds the tree.

**M3 is decomposed** (§3's entry): **g8** holds **t32–t39** plus the
**g9** exit gate. t32 is the D16 domain aggregation; t33 is rmcp + the
skeleton (the initialize handshake, our serverInfo, U16a's instructions,
the §8 concurrency core, `repo_register` as the pipeline proof); t34/t35
are the read tools in two clusters; t36 the node-CRUD mutations; t37
edges + registry-from-handles (U20b) + the default slug (D8); t38 the
wire golden suite + concurrency proof; t39 the opencode smoke + U16b.
Dispatch order t32 → t33 → … → t39 → g9.

**Handover.** Workspace: `main`, clean, four crates, 322 tests, rustc
1.98.1, MSRV 1.95; `Cargo.lock` at 26 packages — rmcp approved but not
yet added (t33's first act). **Next step: dispatch t32** — the D16
domain aggregation, the only task with no dependency on the skeleton;
its spec wants pre-writing (the t27/t29 pattern: `tmp/t32-spec.md` with
the arithmetic transcribed from §6's signed text, the row type's
`blocked_by` order contract from `items.rs`, and the required tests).
The alternative is t33 first (the skeleton unblocks everything else) —
t32 first is preferred because it is domain-only and reviewable while
t33's spec is being written. **Blocked on you: nothing.** Non-blocking
and yours: U16 (lands with M3, t33/t39), U21, U22, U18 (parked); spec
§10's gate enumeration still lists four where every operative home now
runs cargo doc twice — surfaced at g7, one line to sign whenever you
want it. **Unverified:** no wire exists — D14/D15/D16 have no code
behind them, no client has talked to this server, and the rmcp approval
is a decision on paper; everything else the g7 handover carried stands
(f52, f57–f65; the imported-board test is M4's importer). Tracker:
`tools/tasks.py check` exits 0 (g8 open, the in-flight milestone goal;
g3 stands). Doc gates: check-links, `attention.py --check` (zero vote
items for the first time since D16 was drafted), `agents-sync.py
--check` green; agents-index backfilled by this commit.

### 2026-10-02 — M2 g7: the exit gate ran, M2 is closed

The session opened on `/todo-next-imp` with the previous handover naming one
next step — **g7** — and the user said go. The gate ran on the final tree and
every line of its verify method is green: fmt, `check --workspace
--all-targets`, **322** workspace tests (0 failed, 1 ignored — t31's
measurement), clippy, `cargo doc` warning-free under **both** invocations for
the first time, the five goldens byte-identical under `UPDATE_GOLDEN=1` (12
store writers plus the cli suite, `git diff --exit-code` over both golden
dirs = 0), §5.2's selection re-proven a pure function of state, the U13
ledger re-measured and unmoved.

**f48 was the gate's own defect, and settling it is the delivery's code
change.** The doc gate had a private-module blind spot: rustdoc renders only
public modules by default, so `reference.rs`'s module doc — the module is
private — was never rendered, and its `[`BTreeSet`]` link (the type not
imported into the module) had been broken since t6, visible only under
`--document-private-items`. Both halves landed: the one-word fix to the
fully-qualified path, and the gate line itself — `_common.md`'s gate order
now ends with `cargo doc` run twice, `build-and-test.md`'s gate list carries
both invocations with the why, and pitfalls 4.4 records the blind spot.
Mutation-proven in both directions on this tree: reverting the fix prints
exactly that one warning under the flag and none without it; restoring it
returns the count to zero. **f49** landed beside it (`_rust.md`'s awk measure
now says it is for a production file carrying an in-file test block and that
the exempt files must not be fed to a sweep), and **f51** landed in
`build-and-test.md` (a test-name filter is a substring match over the whole
test path — quote the passed count beside every name filter, or use a
module-path filter).

**The f41 sweep ran for the first time**, and every disposition was verified
against the tree rather than assumed: f48/f49/f51/f62 closed by this gate;
**f44/f45 promoted to U21/U22**, because their own verify lines say the
generalisation must not survive only in the gitignored tracker; f4, f25,
f50's generator half, f54, f57–f61, f63–f65 verified deliberately open with
their named consumers confirmed (todo-mcp is still a 16-line shell; no
store-side inspect API; `seeded.rs` untouched since t27; the four shuffle
copies at their measured lines; no test faults a bare `Store::snapshot()`;
`open_log`'s doc still lacks the lock-debris sentence; `Registry::load`
still sorts without dedup; `verify.rs` still prints the asked slug; the
attention priority derivation is still Gates-only).

**The consolidation**: four new `.agents/pitfalls.md` entries — 5.5 (a
per-seed coverage pin's constants are part of the pin), 5.6 (a citation's
content is a claim, and so is its signedness), 5.7 (a class sweep is only as
wide as its grep), 5.8 (whoever writes a contract sentence cannot falsify
it) — plus 4.4's blind-spot addition and top-ten rules 11–12;
`pitfalls-live.md` reset with the former-id table extended to P-LIVE-008,
next id P-LIVE-009.

**The harness edit landed as its own review-exempt commit**, per the previous
handover's prescription: the user's in-flight re-point (`heavy` → `glm-5.3`,
`reasoning: high` → `max` on the orchestrator, implementer and primary
reviewer) with the `version.custom` bump to 9, a full re-sync and a green
`--check` — it had been sitting partially regenerated (the secondary
reviewer's adapter was stale). A model re-point does not take effect
mid-session; the next session runs under it.

**Handover.** Workspace: `main`, four crates, tree clean after the gate's
commits (the harness edit, the dual-reviewed code commit, the consolidation,
and the hash/index backfill that follows this entry). todo-domain **167**
tests, todo-store **120** (four goldens), todo-cli 35 (one golden), todo-mcp
an M0 shell; **322** workspace tests. rustc 1.98.1, MSRV 1.95. `Cargo.lock`
untouched by the whole of M2 — no rmcp, no serde_json; deps unchanged (serde,
dacode-json + zmij by path, tempfile dev-only, lexopt 0.3.2); the tools need
Python 3 + PyYAML. Serena live. Nothing to install. **Landed: M2 is closed**
— the query layer (selection, batch, listing, registry queries) is in behind
dual-reviewed commits, and the exit gate ran green. **Next step: yours, not a
session's** — the **rmcp dependency ask** (Ask-First) is the only thing
gating M3's first task; **U20** (whether the registry caches `name`/`path`)
is due before the wire is built, with t31's measurement as its input; **D16**
(P1 vote) decides whether M3's decomposition carries an aggregation task.
Once the ask is signed, the next session decomposes M3 — the t26 pattern:
tracker tree, board §2 row, spec §6 already normative, D14/D15's wire
decisions already signed. **Blocked on you: D16** (vote), **U20** (decide
before M3 builds), the **rmcp dependency ask**; non-blocking but yours:
**U21** (the duplicate-key check for `agent-rules.yaml`), **U22**
(`progress.py`'s blind spot for open decisions), U16, U18. **Unverified —
read this before trusting anything above.** Nothing outside `todo-domain`
has ever carried a selection, batch or list answer, and nothing at all has
carried a *registry* answer: no store event, no wire, no cli subcommand, no
client — §5.2's rules and §7's registry semantics remain proven against
hand-built and seeded graphs plus store-level tests alone, and the veto-able
readings of signed text are pins against a future edit rather than evidence
about a present consumer. Named weak spots, all carried open with their
consumers: t28's judgment call (j) pin is honestly circular; the
`blocked_by` two-run order is a contract M3's `outputSchema` must state and
nothing enforces; `list.rs`'s resolution seam drops silently by design (f52);
the one-lock-at-a-time discipline is unpinned (f59); `Registry::load` sorts
without deduplicating (f60); the slug cross-check lives only in the query
side (f61); t31's measurement is warm-cache, host-specific and noisy in two
cells (f64); no committed test faults a bare `Store::snapshot()` (f57, three
seams measured); D14/D15 have no code behind them; no induced crash or power
loss anywhere; nothing calls `compact()` outside tests; no human has run
`dump`/`verify`/`repair` on a real store; multi-process lock contention is
still two handles in one process; `tools/progress.py` has no test of its
own. The strengthened doc gate's private-items half has exactly one
historical proof — the `reference.rs` link — and its ongoing value is
unproven until a second private-module link breaks, which is the point.
Tracker: `tools/tasks.py check` exits 0 (g6 closed at this gate; g3 stands
until M5). Doc gates: check-links, `attention.py --check`,
`agents-sync.py --check` green; `agents-index.py --check` stale by
construction (AGENTS.md moved) and backfilled by the commit after this one.
Code gates all run on the final tree, listed above; the five goldens and
`Cargo.lock` untouched.

### 2026-10-02 — M2 t30 and t31 landed: five tasks in, only g7 remains

The session opened on `/todo-next-imp` with the previous handover naming one
next step, and the user signed the recommendation it carried — **U15**'s
remedy (a), scope the doc rather than add cleanup — so t30 was dispatched
straight away. It is in: `store.rs`, `repair.rs`, `error.rs` and `layout.rs`,
comment-only, every gate green, both reviewers bound to the final bytes. Three
things the next session should know, all of them about doc work rather than
about the store.

**A sweep is only as wide as the grep that runs it.** The spec's class table
listed twelve "nothing written / nothing on disk / untouched" claims, read each
against the temp- and lock-debris facts, and cleared eleven. The two it missed
were the same defect in a different spelling — "nothing was **opened**, read or
written" — and the review found both. The generalisation is worth more than the
two fixes: the defect class here is *an exhaustive claim that is narrower or
looser than its mechanism*, and it has many surface forms, so a sweep has to be
run against the defect and not against one phrasing of it. That is f41's lesson
(a gate that cannot see the status it exists to catch) in a lexical register,
and the third time in this repo that the instances found were exactly the ones
the pattern matched. The same defect sat in this task's **own verify line**,
which asked for `rg 'never reported through it'` to return nothing after a fix
whose entire content was changing the four words *before* that substring — so
the grep matched the corrected sentence and could never have passed. f26 and
f51 again: a verification whose command does not mean what its author thought,
caught by running it rather than by reading it. The check that does mean it is
`rg 'so a snapshot fault is never reported'` → empty, beside
`rg -c 'so that fault is never reported through it'` → 1.

**When I write the replacement text, I inherit the claim I was correcting.** The
round-one med was not in the old sentence, it was in the new one: "a failure
**at or** after the temp file's creation does leave that temp" is false when the
creation is what refused, and the reviewer proved it by building the two states
rather than by reading my prose. Rounds two and three were the same shape —
reuse scoped per process where the name is keyed per store directory, a
"no reader opens anything but derived paths" clause loosened by `open_log`'s
public bare-path signature, and "the whole effect of the attempt" broken by
`lock_registry`'s `create_dir_all` on the `Registry` arm. This is the third
delivery where the orchestrator's own prose was the load-bearing defect (t11's
round three, f47's round one), and the remedy is the same each time: the party
that writes a contract sentence is the party least able to falsify it, so the
reviewer prompt asks for falsification against the code path and not for
agreement with the summary.

**What decided remedy (a) was a test, not a preference.** Cleanup-on-failure
looked like the stronger fix until the measurement: `compact.rs` already asserts
exactly one `.tmp` survives a refused rename, so (b) would have inverted a t21
sign-off to make a doc true, across four write sites, adding a failure mode
inside the one path promised not session-fatal. Where a doc claim and a pin
disagree, the pin is the fact and the doc moves — which is also why the new
sentence names the pin instead of restating a count of what it counts.

Two findings were recorded rather than fixed, both deliberately: **f57**, no
test faults a bare `Store::snapshot()`, with the three ready-made seams the
review measured (`chmod 0500` the store directory; remove it out of band while
the handle is open; park a directory at the snapshot path) — a tests-only commit
of the t21/f47 shape; and **f58**, the crate's lock-debris pointer convention
lands one method off the fact at all three of its sites, whose honest remedy is
one sentence in `open_log`'s doc rather than re-pointing one caller and
stranding two. **Blocked on the user, unchanged: D16** (P1, vote) — nothing in
this delivery waited on it, and no part of a roll-up may be built before it is
signed. The alternative front remains M3's rmcp dependency ask, which needs a
decision rather than a session.

The session then continued into **t31**, and four things from it are worth more
than its §3 entry.

**A measurement can change the shape of a decision, and the direction of its
error matters as much as its magnitude.** t11's hand-off assumed a cold
`repo_list` opening every store was acceptable because "M3 answers from handles
it already holds". The numbers say the cost is O(total history) rather than
O(repos) — ~1.2 µs per event of history per repo in release, against a registry
floor that does not grow with N — so at §4.1's top of scale the cold path costs
seconds, not microseconds. The half that nearly got lost is the *bias*: the
fixture is written in-process, so the page cache is warm for both terms, and a
cold cache would inflate the whole listing (every log) far more than the floor
(one small document). These are therefore the optimistic case for the design
being priced, which is the direction the U20 decision turns on. My first draft
of that paragraph claimed the ratio was unaffected; the byte counts refute it.

**The defects were in the prose again, twice more, an hour after writing the
rule about it.** P-LIVE-008 says whoever writes a contract sentence is the party
least able to falsify it. Round three found a duration figure in a comment
("wrote those bytes microseconds earlier") contradicted by the same test's own
`build ms` column, which is `_rust.md`'s no-measurement rule violated by the
person who had just cited it, and a false claim that a warm cache left the
floor/whole ratio unaffected. Both were mine, both were in text written that
morning, and neither was visible to any gate. Occurrences three and four of the
class, after t11's round three and f47's round one.

**A lost worker report is survivable exactly to the extent the spec was written
down.** A power outage killed the implementer's return but not its work: the tree
held three files of finished code and no account of what had been tried. What
made recovery mechanical rather than archaeological was that `tmp/t31-spec.md`
carried the decisions, the twelve required tests and the four mandatory
mutations, so the orchestrator could re-establish non-vacuity by running the
mutations and re-take the measurement instead of guessing. The lesson for
dispatch: a worker's report is a *claim*; the tree and the spec are the
recoverable artefacts, so the spec must be sufficient to verify the work without
the report. The cost of losing it was real but bounded — four mutations, two
measurement profiles and a full gate re-run, all of which a reviewer would have
re-run anyway.

**The load-bearing defect was one no gate could see, and review found it by
building a state.** `repo_node` trusted that `repos/<slug>/` holds `<slug>`'s
graph. Copy a store directory out of band and `repo_show` answers *another
repository's* record while `repo_list` carries one repo twice under two slugs —
breaking the total order the method documents — with nothing detecting it: t13's
`verify` prints the slug it was asked for and never compares it to the graph's.
The type system cannot see it (both are `Repo`), the suite could not see it (no
test built the state), and the mutation that proves the new check is
load-bearing fails with `unwrap_err()` on `Ok(Repo { slug: "camera" })` asked
for as `shelly-ng`. That is the fourth time in this repo a reviewer found the
gap by constructing a state rather than reading a diff, after t22's
interior-damage guard, t20's `--dry-run`/`--yes` swap and t29's `ready` gate
arm.

One reviewer citation was refused rather than inherited: the prescription to
anchor D7's `unknown_id` mapping on `decisions.md:848` ("an unknown repo or epic
is `unknown_id`") points inside **D16's dossier**, whose status row reads
"drafted 2026-09-30 — awaiting the user's vote". Citing a draft as signed
support is f53's class from the other side of the review, so the doc cites only
what is stable — §6.1's silence and the domain's node-scoped gloss — and the
unsigned fact is recorded here instead, where it stales on the vote rather than
in a source comment. The generalisation, for g7's consolidation: verify what a
citation says **and whether the thing it cites is signed**.

**Handover.** Workspace: `main`, t31 committed, and the tree carrying one thing
that is **not** this delivery's to commit: `.agents/config/agent-rules.yaml`
re-points the `heavy` alias (qwen3.8-max-0902 → glm-5.3) and raises `reasoning:
high` → `max` for the orchestrator, implementer and primary reviewer, with four
generated `.opencode/agent/*.md` files partially regenerated — so
`tools/agents-sync.py --check` **exits 1** and `version.custom` is still 8. It
appeared mid-session, it is the user's edit, and t31's four files were staged by
explicit path to keep out of it. Whoever owns it owes the bump to 9, a re-sync, a
green `--check` and a review-exempt commit with no trailers; note that `heavy` is
the alias both t31 reviewers ran under, and that a model re-point does not take
effect mid-session. Four crates: todo-domain **167** tests, todo-store **120**
(four goldens), todo-cli 35 (one golden), todo-mcp an M0 shell; **322** workspace
tests. rustc 1.98.1, MSRV 1.95. `Cargo.lock` untouched by the whole of M2 — no
rmcp, no serde_json; deps unchanged (serde, dacode-json + zmij by path on
`../rust/json-tiers`, tempfile dev-only, lexopt 0.3.2); the tools need Python 3 +
PyYAML. Serena live. Nothing to install. **Next step: g7**, the M2 exit gate —
every code gate on the final tree, §5.2's selection proven a pure function of
state, the five goldens unmoved unless a move is called out, board §2's M2 row
done, U13/U15/U19 and any D16 consequence dispositioned, the tracker swept
against `git log` per f41 (the first milestone gate to exercise that line), and
t29's two pitfalls candidates plus **P-LIVE-007/-008** consolidated into
`.agents/pitfalls.md`. Its verify line should also settle **f48** (the `cargo
doc` gate's private-module blind spot) and **f49** (the `awk` measure
mis-counting test-only files), because both are gate-shape questions and a gate
is what g7 is. The alternative fronts all need the user rather than a session:
the **rmcp dependency ask** (the only thing gating M3), **D16** and **U20**.
**Blocked on you: D16** (P1, vote) — a signed (a) or (b) adds one domain
aggregation task under g6 and, per t29's measurement, most likely a domain row
type with it; nothing waits on it today. **U20**, new out of t31's measurement —
whether the registry should cache `name`/`path`, due before M3 builds the wire,
where option (a) costs a re-derived lock order and not merely a golden.
**Unverified — read this before trusting anything above.** Nothing outside
`todo-domain` has ever carried a selection, batch or list answer, and nothing at
all has carried a *registry* answer: no store event, no wire, no cli subcommand,
no client. So §5.2's rules and §7's registry semantics remain proven against
hand-built and seeded graphs plus store-level tests alone, and the veto-able
readings of signed text are pins against a future edit rather than evidence
about a present consumer. Named weak spots: t28's judgment call (j) pin is
honestly circular; the `blocked_by` two-run order is a contract M3's
`outputSchema` must state and nothing enforces; `list.rs`'s resolution seam
drops silently by design (f52); **the one-lock-at-a-time discipline is
unpinned** — an `open_many` mutant survives all 33 tests and no non-flaky
single-threaded pin exists (f59); `Registry::load` sorts without deduplicating,
so a spliced repeated slug lists twice (f60); the slug cross-check lives only in
the query side, so the crate's readers now disagree about a copied store (f61);
t31's measurement is warm-cache, host-specific and noisy in two cells (f64).
Carried and still true: t30 asserts doc claims checked against code paths, not
test results — the debris story is pinned only through `compact`'s step (1) and
no committed test faults a bare `Store::snapshot()` (f57, three seams measured);
D14/D15 have no code behind them; no induced crash or power loss anywhere;
nothing calls `compact()` outside tests; no human has run `dump`/`verify`/`repair`
on a real store; multi-process lock contention is still two handles in one
process; `tools/progress.py` has no test of its own. Open and gating nothing: f4,
f25, f44, f45, f48, f49, f50's generator half, f51, f54, f57, f58, f59–f65,
U13's ack, U16, U18, U20. Tracker: `tools/tasks.py check` exits 1 on exactly two
lines — OPEN **g6**, which closes at g7, and STAND **g3** until M5. Doc gates:
check-links and `attention.py --check` green; `agents-sync --check` **red for
the reason above**, which is not this delivery's. Code gates all run on the
committed tree: fmt, `check --workspace --all-targets`, clippy, `cargo doc
--workspace --no-deps` clean and `--document-private-items` at exactly the one
known f48 warning; 322 tests; the five goldens and `Cargo.lock` untouched.

### 2026-10-01 — M2 t29 landed: the domain's query surface is complete

The session opened on `/todo-next-imp` with the previous handover naming one
next step and its spec already written, so it dispatched rather than re-derived:
**t29**, `graph/list.rs`, from `tmp/t29-spec.md`. It is in, dual-reviewed over
two rounds, and with it the domain half of M2 is done — §5.2's selection
(t27), batch (t28) and listing (t29) surfaces all exist, and what remains in
the milestone is store-side (t30, t31) plus the g7 gate. Four things the next
session should know.

**A coverage pin's constants are part of the pin.** The round-2 med was not a
missing test in the ordinary sense: `ready`'s decision-gate dimension was
pinned, but only by one seeded graph at one of five seeds with a one-id margin,
and the counter that was supposed to make that visible did not count it. The
remedy — a hand-built pin on both halves of `is_until_satisfied` plus a
per-seed counter — then *forced a seed re-pick*, because the per-seed assertion
was unsatisfiable at the inherited constants. That is the part worth carrying
forward: **changing a seeded suite's constants is a re-measurement event, not a
one-line edit**, since a branch that was non-zero at the old seeds can be zero
at the new ones and every other assertion would stay green. Round two therefore
re-measured *every* coverage branch at the new constants rather than only the
new counter, and found nothing vacated. A pin that is asserted per seed is
stronger than a union-across-seeds pin exactly because it fails loudly here —
t27's round-1 med was the union form, and this is the second time the
distinction has paid for itself.

**A citation is a claim like any other, and the ones I verify least are the ones
that break.** The task spec's §3 D2 attributed `deny_unknown_fields` and the
wire key order to **D15**; D15 is the §6.1 error *carrier*, D14 is the key
order, and `deny_unknown_fields` appears nowhere in `decisions.md`. Every quoted
line number in that spec reproduced — the implementer checked them all — and the
error was in what the cited decision *said*, which is the half the author read
once and never measured. Pitfalls 5.1 asks a spec author to name the state space
a safety argument was checked against; this is its citation-shaped sibling, and
it is the fifth consecutive task spec in this repo to contain a claim that did
not reproduce. The tree is clean: `list.rs` cites D14 correctly and nothing
under `crates/` mentions D15.

**The dispatch protocol survived a spawn death, third occurrence.** The first
implementer spawn died instantly with "Connection reset by server"; a control
spawn (`explore`, tiny prompt) succeeded in between, and the *identical* prompt
landed on the retry. That is pitfalls 5.4 exactly as written, now proven twice
on the remedy: control spawn to isolate binding from infrastructure, then retry
identically, and only escalate past that. Prompt size is still not the trigger —
the failing prompt was the same length as the one that worked.

**D16 gained decision material, and no recommendation moved.** t29's review
measured what the dossier's "the annotation half is separable and cheap" costs
in practice: `items.rs`'s `annotate` is private and its composition rule
(append the epic's gates, dedup at first occurrence, two ascending runs) is a
signed domain contract, so building D16(a)'s annotation over `Vec<TaskId>` in
the binary means re-implementing that rule there; and six
`task_list(status=s).len()` calls in the binary is an aggregation in the server,
which §2 bars, re-deriving the canonical order six times per summary. A signed
(a) therefore most likely forces a domain aggregate *and* a domain row type.
That is recorded in the dossier beside the claim it qualifies. The vote is the
user's and the recommendation is unchanged; t29's `Vec<TaskId>` keeps either
outcome open, which is what its D3 wanted.

**Handover.** Workspace: `main`, clean at `034d081` after this
delivery's two commits, plus the hash backfill that follows them — t29's
crate diff with its board movement, then the comment-only marker fix the sha
pin kept out of it. Four crates: todo-domain
**167** tests, todo-store 107 (four goldens), todo-cli 35 (one golden), todo-mcp
an M0 shell; **309** workspace tests. rustc 1.98.1, MSRV 1.95. `Cargo.lock`
untouched all of M2 — no rmcp, no serde_json; deps unchanged (serde, dacode-json
+ zmij by path on `../rust/json-tiers`, tempfile dev-only, lexopt 0.3.2); tools
need Python 3 + PyYAML. Serena live. Nothing to install. **Next step: t30**,
the two one-sentence store doc riders — **U15** (`store.rs`'s `snapshot` doc
claims "changes nothing on disk", which §4.3's temp debris falsifies: either
scope the sentence to "nothing in place" or add cleanup-on-failure) and **U19**
(`repair.rs`'s module doc, "so a snapshot fault" → "so that fault"). They are
independent of everything above and cheap in lines, but they are a `todo-store`
`src/` change, so a full dual-review cycle for two sentences — that is the
known cost, not a reason to skip the review. Then **t31**, the registry query
side (`repo_list`/`repo_show` plus t11's cold-open measurement, which is
decision material for M3's registry question), then **g7**. The alternative
front is the M3 rmcp dependency ask, which is the only thing gating M3 and
needs the user, not a session. **Blocked on you: D16** (P1, vote) — a signed
(a) or (b) adds one domain aggregation task under g6 and, per the measurement
above, most likely a domain row type with it; t30/t31/g7 do not wait on it, and
no task may build any part of a roll-up before it is signed. Also open and
gating nothing: f44 (a duplicate-key check in `agents-sync.py`), f45
(`progress.py` cannot render open D-entries, so the page cannot list D16 — the
highest-priority open thing), f48 (the `cargo doc` gate's private-module blind
spot, with one link already broken behind it), f49 (the `awk` line measure
mis-counts test-only files), f50 (the dedup coverage counter the generator
cannot reach, and the naive-helper lift — now landed by t29, so what remains is
the generator half), f51 (the filter-substring rule for verify lines), f52–f54
from this session, U13's ack, the carried f25 reading (`4c5782b` reverts it),
and two sibling-repo calls. D13 stays reserved for U18. Six veto-able readings
of signed text are now pinned by a test each (t27's two, t28's three, t29's
`ready` scoping), so any veto is a named diff; they are enumerated in the t27,
t28 and t29 §3 entries. **Unverified — read this before trusting anything
above.** Nothing outside `todo-domain` has ever carried a selection, batch or
list answer — no store event, no wire, no client — so §5.2's rules remain
proven against hand-built and seeded graphs alone, and the six veto-able
readings are pins against a future edit, not evidence about a present consumer.
Three weak spots survive by name: t28's judgment call (j) pin is honestly
circular (the naive recomputation reads the same contract), the `blocked_by`
two-run order is a contract M3's `outputSchema` must state and nothing enforces
yet, and the resolution seam's unreachable-`None` arm in `list.rs` drops
silently by design (f52) — its state-space argument is the only thing behind
it. Carried and still true: the signed wire contract (D14/D15) has no code
behind it; no induced crash or power loss anywhere; nothing calls `compact()`
outside tests; no human has run `dump`/`verify`/`repair` on a real store;
multi-process lock contention is still two handles in one process; the f41
sweep line has never been exercised by a milestone gate (g7 will be the first);
`tools/progress.py` has no test of its own; two harness checks are narrower than
the reason they exist and no gate fails on either (f48, f49). Tracker:
`tools/tasks.py check` exits 1 on exactly two lines — OPEN **g6**, the
in-flight milestone goal that closes at g7 (the g4/g5 precedent), and STAND
**g3** until M5. `.agents/pitfalls-live.md` is still empty; next entry is
**P-LIVE-007**, and this session added two candidates for the M2 gate: a
per-seed coverage pin's constants are part of the pin, so re-picking them is a
re-measurement event; and a citation's *content* is the half nobody measures —
verify what a decision says, not only where it lives. Doc gates green:
check-links, agents-sync `--check`, attention `--check` exit 0. Code gates all
run on the committed tree: fmt, `check --workspace --all-targets`, clippy,
`cargo doc --no-deps` clean, `--document-private-items` at exactly the one
known f48 warning; 309 tests; five goldens and `Cargo.lock` untouched.

### 2026-10-01 — M2 f47 closed and t28 landed: the brief was the defect twice

The session opened on `/todo-next-imp` with two candidate fronts named in the
handover — t28 and t30 — and the user took t28's chain, starting with f47.
Three things happened that the next dispatch should know about.

**The handover's routing for f47 was wrong, and file ownership is what showed
it.** f47 named "t28's dispatch" as its consumer, on the reasoning that t28's
implementer re-reads `select.rs` anyway, so the four items would cost no extra
review. But three of the four live in t27's files and t28 owns `items.rs` and
its test file alone, so folding them in would have broken the ownership rule
that made the M2 split safe and staled both t27 sign-offs for a coverage
strengthening. The `t21`/`f19` shape — its own tests-only commit with its own
dual review — was the alternative the finding itself named, and it is what
landed. The general rule is worth more than the instance: **a rider belongs to
the task that owns the file, not to the task that happens to read it.**

**Pre-measuring the brief's claims did not protect the brief's prose.** The
spec was authored with every mutation built and observed first, in a scratch
copy: the greedy mutant surviving all 131 tests, both `relates` leaks leaving
T13 green, the per-seed counts before and after the narrowing, and two traps
recorded so the implementer would not re-derive them (the killer graph's
induced order starts with the *closed* task, because statuses do not filter the
order; and `Closed → Deferred` moves a summary expectation, so the one-status
change alone fails on a pristine tree). All of that reproduced. What was *not*
measured was the divergence condition, which was copied out of the tracker
finding's own text into the spec and from there into a source comment —
"the two agree everywhere except an epic holding two independent inner chains".
Both reviewers falsified it independently and with different counterexamples:
two independent chains **agree**, one chain plus a lone task **diverges**. The
class is pitfalls 5.2 exactly, and this is the first time the author was the
party the rule binds rather than a worker it was quoted at. The mechanism that
let it through is specific enough to be worth stating: measurement discipline
was applied to every claim I intended to *test*, and none to the sentence I
only intended to *explain*. An inherited explanation is a claim like any other.

**Round 2 found a residual nobody had looked for, in the assertion the clause
was going to skip.** The implementer omitted clause (iii)'s repo-scope
assertion with a sound reason — a single-epic graph makes the repo order equal
the induced one, and the epic-scoped greedy mutant leaves `next_task` alone —
and the primary then wired a greedy walk into `next_task` *only* and watched it
survive all 130 tests, pre- and post-edit. The assertion that "could not fail
under the mutant it exists to kill" was the only thing that killed a different
one. Eight order mutants were built across the review and four die on that line
alone. The lesson is the converse of the one t27's round 1 taught: an assertion
is not redundant because the mutant you have in mind cannot reach it.

Two harness gaps were found by the implementer rather than by a gate, and both
are recorded: **f48**, the `cargo doc` gate cannot see private-module docs, and
one link behind that blind spot has been broken since t6; **f49**, `_rust.md`'s
`awk` production-line measure counts a test-only file in full, because such a
file never contains the `#[cfg(test)]` line the measure stops at. Both are
cases of a check that is narrower than the reason it was added.

**Then t28 landed in the same sitting, and the brief was wrong three more
times.** `next_items` — the batch half of §5.2, three scopes, the annotations,
the derived weight — dispatched off a 600-line spec whose twelve decisions were
settled in advance so the implementer would not have to. Three of its claims
did not survive contact with the code, and all three were *measured* rather than
argued, which is the discipline f47's high finding had just failed. M5's stated
expectation was unsatisfiable: the coverage pin's counters are computed through
the naive helpers only, so no production mutation can fail them — it is a
generator-reachability pin, not a mutation detector, and the mutant dies at T16
and T19 instead. T18 was unreachable at the constants the brief inherited from
t27, so the suite measured a grid and picked its own seeds and step depth. And
T15's terminal-task clause was over-broad. The pattern across f47 and t28 is
now unmistakable: **a task spec's stated test expectations are claims to
falsify against the code, exactly like its safety arguments** (pitfalls 5.1),
and the ones an author writes from the shape of the test rather than from a run
are the ones that break.

**t28's review found the gap the spec's own mutation list did not.** Six
mutations were mandated and all six died where the brief said. The primary then
built a *seventh* — pushing the epic's gates into `blocked_by` unconditionally,
dropping the dedup — and it survived all 19, because the double-gate shape (one
decision gating a task and its epic) occurs in none of the suite's seeds and no
hand-built case had it. That is f9's vacuity class and the exact defect f47
existed to close, found one commit later in the next module. It is now pinned by
a hand-built row. The lesson generalises past this task: **a mutation list a
spec author writes is bounded by the shapes the author imagined**, so the
reviewer's job includes building the mutant the author did not think of — the
coverage pin catches a generator that never reaches a branch, but only a
reviewer catches a contract arm no test reaches at all.

**Two readings the review settled rather than the brief.** D2 said the
annotation lists keep "the accessor's own order"; D6 said `blocked_by` is two
adjacency lists concatenated. Both reviewers independently ruled the order
**per-source** — a global merge is the second sort D2 forbids, so the
global-order reading is self-inconsistent — which made the implementer's
own-edges-then-gates concatenation a documented reading instead of a defect, and
put a contract M3 must honour ("two ascending runs, not one ordered list") into
the field doc where whoever declares the wire bytes will read it. And the round
falsified a claim the *fix round* had just written into `order.rs`: "a batch is a
prefix of a list already in key order" is false, because the list is
lexicographically-minimal *topological* and a batch is a *filtered* prefix. A
doc comment corrected in the commit that corrected it, caught on re-review, is
the f47 lesson landing one level up — the fix's own prose is a claim like any
other.

**Handover** — supersedes t27's below.

```
Workspace:    main, clean at the t28 commit plus its backfill; M2 UNDER WAY —
              t27 and t28 of five tasks landed, f47 closed, t29 next in the
              domain chain. Four crates: todo-domain 149 tests (130 after f47,
              +19 from t28), todo-store 107 (four goldens), todo-cli 35 (one
              golden), todo-mcp an M0 shell. rustc 1.98.1, MSRV 1.95, 291
              workspace tests, Cargo.lock UNTOUCHED — no rmcp, no serde_json,
              t28 added no dependency. Deps unchanged (serde, dacode-json + zmij
              by path on ../rust/json-tiers, tempfile dev-only, lexopt 0.3.2);
              tools/ need Python 3 + PyYAML. Serena live. Nothing to install.
Landed:       t28 — the domain's §5.2 batch half: graph/items.rs (387
              production lines) with next_items/next_items_of_epic/
              next_items_of_task, the NextItems/NextItem value types (no
              serde), the blocked-by/blocks annotations, the derived
              transitive-dependent weight, and the deferred summary reused
              from select.rs by a second pass over the same derived list.
              select.rs byte-identical to HEAD, sha-proven at every step.
              order.rs's judgment call (c) amended comment-only (the
              order_key trigger was met and did not fire). graph/mod.rs,
              graph/tests/mod.rs and lib.rs wired. 19 tests, crate 149,
              workspace 291, dual-reviewed over two rounds, both ok on the
              committed bytes. Board §1/§2/§3/§6 moved in the same commit.
              Closed: f47 (earlier this session), t28. Opened: f48, f49, f50,
              f51. Still open: D16, U13, U15/U19 (owned by t30), U16, U18,
              f4, f25, f35(=U19), f40(=U15), f44, f45.
Next step:    write tmp/t29-spec.md, then dispatch t29 — the domain's
              graph/list.rs, task_list's filters (epic, status, kind, and the
              until-target 'ready' filter for deferred items) plus the listing
              reads §6 names, each returning its documented total order. It
              depends on t27 for is_until_satisfied (the 'ready' filter is the
              same question the deferred summary's ready count answers — one
              implementation, two callers) and reads t28 as its precedent. The
              repo dimension is NOT here (t7's graph is single-repo); repo
              selection is t31's and M3's. SEAMS the t28 review handed forward,
              all in f50: naive_actionable/naive_until_satisfied/
              naive_end_finished are now duplicated verbatim in tests/select.rs
              and tests/items.rs, so t29 would make a third copy — the next
              owner of tests/mod.rs should lift them (production-independence is
              the property that matters, not per-suite duplication), and t29's
              brief must decide that BEFORE writing a third; t29 must not
              'inherit' select.rs's 300 steps or seed set blindly, because a
              listing pin's shapes may need different depth (t28 measured this
              the hard way); and a verify line must quote the count beside any
              name filter and read it against rg -c '#[test]', or use a
              module-path filter, because '-- order' already matches four
              items_* names (f51). The alternative front is t30, the
              two-sentence store doc riders (U15 + U19) — independent of the
              domain chain, but a todo-store src change, so it costs a full
              dual-review cycle for two sentences. D16 gates neither.
Blocked on you: D16 — the progress roll-up: (a) a status_summary(repo?, epic?)
              tool plus blocked_by on task_list rows, (b) a counts block on
              §6.2's graph_show dump plus the same annotation, (c) leave it
              client-side per §11; recommendation (a) with the annotation
              whichever wins. A signed (a) or (b) adds one domain aggregation
              task under g6; no task may implement a roll-up before then, and
              t28 deliberately built none. Also open and gating nothing: f44 (a
              duplicate-key check in agents-sync.py), f45 (progress.py does not
              render open D-entries, so the page cannot list D16 — the
              highest-priority open thing in the project), f48 and f49 (both a
              one-line fix and a question about what the gate should be), f50
              (the dedup coverage counter the generator cannot reach, and the
              naive-helper lift t29 owes), f51 (the filter-substring rule for
              verify lines), U13's acknowledgement, the carried f25 reading
              (4c5782b is the commit to revert), and two sibling-repo calls (a
              cargo fmt --check gate for dacode-json; whether to benchmark
              c4aebe9). D13 stays reserved for U18. FIVE veto-able readings of
              signed text are now pinned by a test each, so a veto is a named
              diff: t27's two (the epic gate feeds the ready count as well as
              actionability; the deferred summary counts tasks, not epics) and
              t28's three (repo/epic scope filters to actionable while task
              scope marks; blocked_by includes the epic's decision gates; task
              scope's list is the repo order restricted to the closure, whose
              pin is honestly circular). And the consequence a reviewer checks
              first still stands: an epic's own STATUS never gates its tasks,
              only decision→epic edges do.
Unverified:   nothing outside todo-domain has ever carried a selection or batch
              answer — no store event, no wire, no client — so §5.2's rules are
              proven against hand-built and seeded graphs alone. Two pins guard
              a future edit rather than a present defect and are the only guard:
              f47's clause (iii) on the induced order (no production code
              re-derives it today), and t28's dedup row (the generator reaches
              the double-gate shape in none of the five seeds, proved by running
              the mutant and watching the seeded agreement test pass). One pin
              is honestly CIRCULAR and cannot falsify its own reading: t28's
              judgment call (j), task scope's list derivation, because the naive
              recomputation reads the same contract — recorded in the module doc
              so nobody assumes it is tested. The blocked_by two-run order is a
              contract M3's outputSchema must state and nothing enforces that
              yet. The two harness gaps are unverified in the other direction —
              no gate currently fails on either, which is why they were found by
              a reader: cargo doc --workspace --no-deps prints nothing while
              --document-private-items prints one warning (f48), and the awk
              measure counts a test-only file in full (f49). Carried forward and
              still true: the signed wire contract (D14/D15) has no code behind
              it; no induced crash or power loss anywhere; nothing calls
              compact() outside tests; no human has run dump/verify/repair
              against a real store; multi-process lock contention is still two
              handles in one process; the f41 sweep line has never been
              exercised by a milestone gate; tools/progress.py has no test of
              its own.
Tracker:      tools/tasks.py check exits 1 on exactly one line — OPEN g6, the
              in-flight milestone goal that closes at g7 (the g4/g5 precedent).
              Everything else clean: g3 standing until M5, t26/t27/t28 done
              with evidence, f47 done with evidence, d1 wontfix on measurement,
              no debt open. Open findings f4, f25, f35(=U19), f40(=U15), f44,
              f45, f48, f49, f50, f51. .agents/pitfalls-live.md is still EMPTY —
              the next entry is P-LIVE-007, and this session has four candidates
              for the M2 gate: "an inherited explanation is a claim like any
              other" (f47's false universal, copied from the tracker's own prose
              into a source comment); "an assertion is not redundant because
              the mutant you have in mind cannot reach it" (f47's repo-scope
              line); "a spec's stated test expectations are claims to falsify
              against a run, not against the shape of the test" (all three t28
              spec errors); and "a spec author's mutation list is bounded by the
              shapes they imagined, so the reviewer builds the one they did not"
              (t28's seventh mutant, the unpinned dedup arm). Doc gates ALL
              green on the committed tree, each run not inherited: check-links,
              agents-sync --check, attention --check, and agents-index --check —
              the last is green because this board is not an indexed AGENTS.md
              doc, so moving it does not stale the index (pitfalls 5.3's backfill
              is owed only after a commit that touches an AGENTS.md; the f47
              handover asserted the staleness without checking it, and this
              session's own lesson is that an inherited claim is a claim like any
              other). Code gates ALL RUN on the committed tree: fmt clean, check
              --workspace --all-targets clean, 291 workspace tests, clippy clean,
              cargo doc clean, the five goldens untouched, Cargo.lock untouched.
```

### 2026-09-30 (third session, continued) — M2 t27: §5.2 selection

The decomposition was agreed and dispatched in the same sitting, so M2's
first code landed hours after its plan did: **t27**, the domain's
`graph/select.rs`. §3 carries the delivery; this entry carries the process,
because two things happened that the next dispatch should know about.

**The round-1 gap was a test that could not fail, not code that was wrong.**
`next_task_of_epic` was implemented correctly and *nothing in the suite could
tell*: two wrong implementations of the same method survived all 18 tests —
walking the repo-wide order filtered to the epic (D9's rejected alternative)
and walking the epic's tasks in ascending id, which drops topology and the
signed D6 tiebreaker together. The hand-built case that looked like it pinned
the meeting of those two signed rules did not, because with E1 < E2 the
induced order and the filtered repo order **coincide**, and the seeded graphs
reach a divergence once in 74 epics. The reviewer found it by building the
mutants rather than reasoning about them, which is the only method that would
have. The generalisation is worth more than the instance: a test that pins an
interaction between two rules has to be derived from the *state where the two
candidate behaviours differ*, not from a description of either rule — the
same shape as pitfalls 2.1 (build the crash, do not describe it) on the
ordering side.

**The coverage pin earned its keep twice in one task.** f9's lesson — a
property test whose interesting branches the generator never reaches is green
and vacuous — was built into the spec as a required test, and it fired: when
branch (c) was strengthened from "a task under a gated epic" to "a task the
gate holds *alone*" (the shape that actually makes the seeded agreement test
sensitive to deleting the gate), seed 14 stopped qualifying, so the seed set
changed. A pin that only ever says "yes" would have hidden that. The
per-seed restructuring came from the same round: asserting the union across
five seeds let a generator change narrow one of them silently.

**The stopping rule was applied rather than a third round opened.** Both
reviewers returned `ok` on the fixed tree with four lows between them, so
those became **f47** with t28's dispatch named as their consumer — a rider
section in a spec whose implementer re-reads `select.rs` anyway, at no extra
review cost. One of the four is substantive and is recorded precisely because
it is *not* a present defect: a valid-but-not-key-minimal walk of the induced
subgraph still survives the suite, and t8's own minimality recomputation pins
that property inside `canonical_order_of_epic`, so what is missing is a guard
against someone re-deriving the order inside `select.rs` later.

The spec-error path ran twice more and worked both times: the implementer
reported that mutation M4 survived T18 **as the spec specified it** (the
chain held exactly one actionable task at every step, so first equalled last)
and fixed the test rather than reporting the mutation as run; and the primary
found that M1's stated condition was false — "a seed whose graphs contain an
epic gate" is not enough, since a task also held by an unfinished predecessor
answers the same either way. `tmp/t27-spec.md` carries both amendments
inline, per pitfalls 5.1 (amend the spec; do not qualify the docs around it).

**Handover** — supersedes the decomposition's below.

```
Workspace:    main, clean at 9859bb6 plus its backfill; M2 UNDER WAY — t27 of five
              tasks landed, nothing else dispatched. Four crates:
              todo-domain 130 tests (was 111; the 19 new are all in
              graph/tests/select.rs), todo-store 107 (four goldens),
              todo-cli 35 (one golden), todo-mcp an M0 shell. rustc
              1.98.1, MSRV 1.95, 272 workspace tests, Cargo.lock
              UNTOUCHED — no rmcp, no serde_json, and t27 added no
              dependency. Deps unchanged (serde, dacode-json + zmij by
              path on ../rust/json-tiers at f7f26f8, tempfile dev-only,
              lexopt 0.3.2); tools/ need Python 3 + PyYAML. Serena live.
              Nothing to install.
Landed:       t27 — the domain's §5.2 selection: graph/select.rs (240
              production lines) with is_until_satisfied (the edge half:
              the task's own blocks predecessors AND its epic's decision
              gates), is_actionable (that plus status open/in_work),
              next_task() total and next_task_of_epic() failing only
              unknown_id, and NextTask{task: Option<TaskId>, deferred:
              DeferredSummary{total: u64, ready: u64}} with None as an
              answer and no serde on either type. 19 tests including an
              independent longhand recomputation over seeded graphs and a
              per-seed coverage pin. Riders: three stale doc claims this
              change falsified (order.rs's module doc, tests/order.rs,
              node.rs). Board §1/§2/§3/§6 moved in the same commit;
              AGENTS.md's phase line, with the index backfilled after it.
              Closed: t27, and d1 earlier this session. Opened: f47 (four
              round-2 lows, consumer named). Still open: D16, U13, U15/U19
              (owned by t30), U16, U18, f4, f25, f35(=U19), f40(=U15),
              f44, f45, f47.
Next step:    dispatch t28 — the domain's graph/items.rs, next_items'
              batch: the blocked-by/blocks annotations, the derived weight
              (count of transitive dependents), the task-scoped variant
              (that item's transitive dependents, each marked immediately
              actionable or still held), limit defaulting to 10, and the
              deferred summary reused from select.rs. It depends on t27 and
              must not edit it: the two share DeferredSummary and the
              predicates by import, and t28 owns items.rs and
              tests/items.rs alone. Its spec carries f47's four items as a
              rider section. The alternative front is t30, the two-sentence
              store doc riders (U15 + U19) — independent of the domain
              chain, closes two board items, and is the cheap win if a
              session wants one. D16 gates neither.
Blocked on you: D16 — the progress roll-up: (a) a status_summary(repo?,
              epic?) tool plus blocked_by on task_list rows, (b) a counts
              block on §6.2's graph_show dump plus the same annotation,
              (c) leave it client-side per §11; recommendation (a) with the
              annotation whichever wins. A signed (a) or (b) adds one
              domain aggregation task under g6; no task may implement a
              roll-up before then. Also open and gating nothing: f44, f45,
              U13's acknowledgement, the carried f25 reading (4c5782b is
              the commit to revert), and two sibling-repo calls (a cargo
              fmt --check gate for dacode-json; whether to benchmark
              c4aebe9). D13 stays reserved for U18.
              Two readings of signed text landed in t27 and are flagged
              veto-able there: the epic gate feeds the ready count as well
              as actionability, and the deferred summary counts tasks, not
              epics. Both are pinned by a test each, so a veto is a named
              diff. Also worth knowing: an epic's own STATUS does not gate
              its tasks — only decision→epic edges do — so a task under a
              closed epic is a legal next_task answer.
Unverified:   t27 is verified by 19 tests and two review rounds, and NOT by
              anything outside the crate: no store event, no wire, no client
              has ever carried a selection answer, so §5.2's rules are
              proven against hand-built and seeded graphs alone. The
              seeded generator reaches an epic whose induced order differs
              from ascending id once in 74 epics, which is why T19 carries
              D9 rather than the property test — and f47 records the one
              wrong order implementation that still survives the suite.
              Carried forward and still true: the signed wire contract
              (D14/D15) has no code behind it; no induced crash or power
              loss anywhere; nothing calls compact() outside tests; no
              human has run dump/verify/repair against a real store;
              multi-process lock contention is still two handles in one
              process; the f41 sweep line has never been exercised by a
              milestone gate; tools/progress.py has no test of its own.
Tracker:      tools/tasks.py check exits 1 on exactly one line — OPEN g6,
              the in-flight milestone goal that closes at g7 (the g4/g5
              precedent). Everything else clean: g3 standing until M5, t26
              and t27 done with evidence, d1 wontfix on measurement, no
              debt open. Open findings f4, f25, f35(=U19), f40(=U15), f44,
              f45, f47. .agents/pitfalls-live.md is still EMPTY — the next
              entry is P-LIVE-007, and t27's "derive the fixture from the
              state where the two candidate behaviours differ" is a
              candidate for it at the M2 gate. Doc gates green: check-links,
              agents-sync --check, attention --check; agents-index --check
              is stale by construction because AGENTS.md moved in this
              commit and is backfilled by the next one (pitfalls 5.3). Code
              gates ALL RUN on the committed tree: fmt clean, check
              --workspace --all-targets clean, 272 workspace tests (domain
              130, store 107, cli 35), clippy --workspace --all-targets
              clean, cargo doc --workspace --no-deps warning-free, the
              no-clock/no-rand grep empty, the five goldens untouched,
              Cargo.lock untouched.
```

### 2026-09-30 (third session) — M2 decomposed, D16 drafted

The session opened on `/todo-next-imp` and took the handover's named step:
decompose M2. Nothing gated it — U2's survey had run, D14/D15 were signed,
and §12 was closed — so the decomposition is a planning change and no Rust
moved. Five tasks under **g6**, one commit each, with **g7** as the exit
gate.

**The split is §2's, and one consequence is worth stating because it is
easy to get backwards.** Selection is a pure function of graph state, so
`next_task`, `next_items` and `task_list`'s filters all land in
`todo-domain`, one new module per task (`select.rs`, `items.rs`, `list.rs`)
so that no two tasks ever write one file — t28 and t29 *depend* on t27's
actionability and until-target predicates rather than rederiving them,
which is a dispatch order, not a shared file. The **repo** dimension is not
a domain concern at all: t7's graph is single-repo, so §6's "every tool
takes a repository parameter" is discharged by t31's registry queries and
M3's handle table, and a domain task that grew a `repo` parameter would be
a boundary defect. t30 rides along as the store touch **U15** and **U19**
have been waiting on, which is why they are now owned rather than
incidental.

**U17 was answered by splitting it, and only one half needed a decision.**
The mapping half is already signed — `attention.py` is `next_task`/
`next_items`, `progress.py` is §6.2's `graph_show`, `tasks.py` stays a
script because §9 bars dogfooding before M3, and the HTML rendering stays a
client concern because §11 says clients render. The roll-up half is not:
totals by kind and status are an aggregate §6's list does not carry, and
adding one amends D5, which is how D11's `next_items` happened. So it is
drafted as **D16** with recommendation (a) and every task is barred from
implementing a roll-up until it is signed — the "never write past an
unsigned decision" rule, honored by drafting rather than by stopping.

**Both measurements the decomposition owed were taken, not assumed.** d1
closed as wontfix on its own stated criterion: `tasks_of_epic` has exactly
one production caller (`delete_epic`'s cascade), `order.rs`'s judgment call
(d) already records that the derivation builds its own maps and never calls
it, and selection's per-candidate read is `blocked_by` — an indexed lookup
into `blocks_in`, not a scan — so nothing in g6 puts the epic scan on a
derivation or selection path. U13 re-measured under the same awk and
unchanged, with its next revisit moved from "M2" to M3-or-next-growth and
the reason recorded: M2's code lands in new files, so it cannot show these
two's seams.

`progress.py`'s hand-maintained tables were refreshed in the same commit,
per the t23 and f41 precedent — a new lane for the M2 nodes, seven gating
edges with their citations, three retargeted onto t30/M3 now that the waits
are named, and the stale rows for closed items dropped. That refresh is
also what made **f45** concrete: D16 is drawn as a graph node because it
was added by hand, but the open-items table has no concept of a
decisions.md entry, so the highest-priority open thing in the project has
no row and no "waits on" cell. Drawing it properly is the user's call, not
this commit's.

**Handover**

```
Workspace:    main, clean at c088ba9 plus its index/hash backfill; M2
              DECOMPOSED, nothing dispatched. No Rust has moved since 7563fff (t21):
              this session changed the spec (§12 item 16), decisions.md
              (D16), this board, AGENTS.md and tools/progress.py's data
              tables. Four crates: todo-domain 111 tests, todo-store 107
              (four goldens), todo-cli 35 (one golden), todo-mcp an M0
              shell. rustc 1.98.1, MSRV 1.95, 253 workspace tests,
              Cargo.lock UNTOUCHED — no rmcp, no serde_json. Deps
              unchanged (serde, dacode-json + zmij by path on
              ../rust/json-tiers at f7f26f8, tempfile dev-only, lexopt
              0.3.2); tools/ need Python 3 + PyYAML. Serena live. Nothing
              to install.
Landed:       the M2 decomposition — tracker g6 with t27 (domain
              graph/select.rs: actionability, next_task, the until-target
              ready predicate, D9's deferred {total, ready}), t28
              (graph/items.rs: the next_items batch, annotations, derived
              weight, task scope), t29 (graph/list.rs: task_list's
              epic/status/kind/ready filters), t30 (the store doc riders
              U15 + U19), t31 (the registry query side: repo_list/
              repo_show plus t11's cold-open measurement) and the g7 exit
              gate, every task with a runnable verify line and disjoint
              file ownership. D16 drafted out of U17 (spec §12 item 16,
              decisions.md row + dossier + action line, recommendation (a),
              gates M2). Board §1/§2/§3/§4(U17 struck)/§5(U13
              re-measured)/§6 moved; AGENTS.md's Project State names D16 as
              the one open §12 item; progress.py's LANES/GATING/EXTRA/WAITS
              refreshed. Closed: t26, d1 (wontfix on measurement), U17.
              Open and live: D16, U15/U19 (owned by t30), U13, U16, U18,
              f4, f25, f35(=U19), f40(=U15), f44, f45.
Next step:    dispatch t27 — the domain's graph/select.rs, first in g6's
              order and the task everything else in M2 reuses. It needs a
              task spec written first (the tmp/t11-spec.md /
              tmp/t12-spec.md pattern: decisions, judgment calls, required
              tests), and pitfalls 5.1 binds that spec: any safety or
              determinism argument in it must name the state space it was
              checked against, and the reviewer prompt must ask for
              falsification rather than conformance. The alternative is
              t30, the two-sentence store doc riders — smaller, unblocks
              two board items, and independent of the domain chain, so it
              is the right second front if a session wants a cheap win
              first. D16 does not gate either.
Blocked on you: D16 — the progress roll-up: (a) a status_summary(repo?,
              epic?) tool plus blocked_by on task_list rows, (b) a counts
              block on §6.2's graph_show dump plus the same annotation,
              (c) leave it client-side per §11; recommendation (a) with the
              annotation whichever wins. It gates M2 only in that a signed
              (a) or (b) adds one domain aggregation task under g6. Also
              open, none gating anything: f44 (land a duplicate-key check
              in agents-sync.py, or promote it), f45 (draw open D-entries
              on the progress page, or disclaim them in the footer — now
              concrete, since D16 has no row there), U13's acknowledgement,
              and the carried f25 reading (4c5782b is the commit to revert)
              plus two sibling-repo calls (a cargo fmt --check gate for
              dacode-json; whether to benchmark c4aebe9). D13 stays
              reserved for U18's transport question.
Unverified:   the decomposition is a plan, so nothing in it is verified by
              execution — no selection code exists, and §5.2's
              actionability rules have never been run against a graph. The
              two measurements it does claim are re-runnable: rg -n
              'tasks_of_epic' crates/ returns three hits (definition,
              delete_epic's cascade, order.rs's doc), and _rust.md's awk
              returns 812 and 772 for log.rs and event.rs. Carried forward
              and still true: the signed wire contract (D14/D15) is
              normative text with no code behind it — nothing enables
              serde_json/preserve_order, no wire golden exists, no
              structuredContent.code has been emitted, no MCP client has
              been pointed at anything; no induced crash or power loss
              anywhere; nothing calls compact() outside tests; no human has
              run dump/verify/repair against a real store; multi-process
              lock contention is still two handles in one process; the f41
              sweep line has never been exercised by a milestone gate;
              tools/progress.py has no test of its own, so this session's
              table refresh is verified only by running it.
Tracker:      tools/tasks.py check exits 1 on exactly one line — OPEN g6,
              the in-flight milestone goal, which closes at g7 (the g4/g5
              precedent, reported rather than papered over with
              --status standing). Everything else clean: g3 standing until
              M5, t26 done with evidence, d1 wontfix with its measurement,
              no debt left open. Open findings f4, f25, f35(=U19),
              f40(=U15), f44, f45. .agents/pitfalls-live.md is still EMPTY
              — the next entry is P-LIVE-007 and a promoted id is never
              reused. Doc gates green: check-links (11 links / 10 files),
              agents-sync --check, attention --check exit 0 at 184 checks
              with D16 rendering as the one P1 vote; agents-index --check
              is stale by construction because AGENTS.md moved in this
              commit, and is backfilled by the next one (pitfalls 5.3).
              Code gates skipped by construction: git diff over '*.rs',
              Cargo.lock and Cargo.toml is empty.
```

### 2026-09-30 (second session, closing) — D14 and D15 signed

The user signed both recommendations as drafted, in the same message, so
the session's last act was executing two work orders rather than opening
anything. Both are wire-contract decisions out of U2's survey, and both
landed as normative spec text with no code behind them yet — which is the
right order here, because M3 would otherwise have discovered the shapes
while implementing them.

**One half of D14's signature is worth calling out.** The recommendation
was "(a), with a golden over the wire bytes whichever option wins", and the
user signed the recommendation, so the golden is part of the decision and
not a nice-to-have: `serde_json/preserve_order` is a cargo feature unified
across the whole dependency tree, and a build that lost it would re-sort
every key on the wire while still compiling and still passing any test that
merely *parses* a response. §10's protocol-level bullet now requires the
responses to be golden-pinned bytes, and §8 names the feature as a
determinism dependency rather than a preference. That is the difference
between a determinism property and a determinism hope.

**D15 closes a defect that had not happened yet.** rmcp's default
tool-error path is `Err(String)`, which reaches a client as `isError` plus
a text block and nothing else — so the obvious M3 implementation would have
satisfied "return an error" while violating §6's "never bare strings", and
no test in this repository would have caught it, because nothing here
speaks MCP yet. §6.1 now names the carrier (`structuredContent.code`),
states that an error payload need not conform to the tool's `outputSchema`,
reserves protocol-level errors for what the protocol itself rejects, and
carries the two consequences for M3 that fall out of it: `invalid` covers
values this server parses rather than argument shapes, and params structs
deny unknown fields.

**Handover** — this session's, superseding the two above it.

```
Workspace:    main, clean at 5dfba51 plus its backfills; **M1
              CLOSED**, phase M2-next. No Rust has moved since 7563fff
              (t21) — everything after it is harness, board and spec:
              1581007, 6f96e01, bf34068, d542065, 5c23f5f, 95a83eb +
              02475d0 (the f41 sweep), 8135d06 + cc80cc3 + f8ca08a (the U2
              survey), 5dfba51 (the D14/D15 sign-off) and 13a2e9d
              (progress.py's exec bit, f46) plus this index backfill. Four
              crates:
              todo-domain 111 tests, todo-store 107 (four goldens),
              todo-cli 35 (one golden), todo-mcp an M0 shell. rustc
              1.98.1, MSRV 1.95, 253 workspace tests, Cargo.lock
              UNTOUCHED — no rmcp, no serde_json. Deps unchanged (serde,
              dacode-json + zmij by path on ../rust/json-tiers at f7f26f8,
              tempfile dev-only, lexopt 0.3.2); tools/ need Python 3 +
              PyYAML. Serena live. Nothing to install.
Landed:       **D14 and D15 signed, both (a)** — spec §6 (error
              principle), §6.1 (the carrier paragraph, the conformance
              exemption, and `invalid`'s scope), §6.2 (declaration order
              plus the flag that carries it), §6.3 (both findings marked
              resolved), §8 (two named serializers, the feature called a
              determinism dependency), §10 (protocol-level responses are
              golden-pinned bytes); §12 items 14/15 deleted, so the list is
              closed again; decisions.md rows struck with resolution
              banners and their action lines retired. Earlier this session:
              the f41 sweep and the sweep rule in the exit gate, and the U2
              rmcp 3.5.0 survey as spec §6.3. Closed: t24, t25, f41 (with
              f31–f34, f36–f39, f42, f13, d2, d3), U2, D14, D15, and f46
              (opened and closed here: progress.py's missing exec bit).
              Opened and still live: U19 (= f35), f44, f45. Probe artifacts
              stay in gitignored tmp/rmcpprobe — vendored rmcp source,
              the probe crate,
              drive.py, six captures — and are NOT committed.
Next step:    **Decompose M2**, the query layer: repo/epic selection,
              dependency resolution, deterministic next_task, registry with
              path updates (board §2's M2 row; spec §5.2 + §6). Nothing
              gates it: U2 ran, and D14/D15 gate M3 only. Answer **U17**
              inside the decomposition — the totals/open-items roll-up as a
              new tool or a field on an existing one, which is a §6
              amendment and therefore a user decision, so raise it as one
              rather than settling it — and re-measure **d1**
              (Graph::tasks_of_epic's linear scan) and **U13** (the
              file-size ledger) there, since M2's queries are what would
              make either hot. M2's first task is the obvious one: §5.2's
              order is already a derivation, and next_task/next_items are
              the signed selection over it (D9/D11, including the deferred
              {total, ready} summary and task-scoped next_items). The
              alternative — making M3's rmcp dependency ask early — is
              legitimate but out of roadmap order and M2 needs no
              dependency at all.
Blocked on you: nothing. Four small calls are open and none gates M2:
              **f44** (land a duplicate-key check in agents-sync.py, or
              promote it), **f45** (progress.py has no concept of a
              decision, so an open D-entry is invisible on that page —
              draw them or disclaim them in the footer), **U13**'s
              acknowledgement, and **U15**/**U19**, two one-sentence store
              doc fixes riding the next store touch. Still yours from
              before: f25's veto-able reading (4c5782b is the commit to
              revert) and two sibling-repo calls (a cargo fmt --check gate
              for dacode-json; whether to benchmark c4aebe9). D13 stays
              reserved for U18's transport question, deliberately not
              drafted.
Unverified:   **the signed wire contract is normative text with no code
              behind it.** Nothing in the workspace enables
              serde_json/preserve_order, no wire golden exists, no
              structuredContent.code has ever been emitted by this project,
              and no MCP client has been pointed at anything — so U16's
              claims about a client surfacing `instructions`, and D15's
              claim that a client branches on isError plus a code, are both
              unexercised. D14's flag behaviour is measured on a probe
              crate, not in this tree: cargo features are additive, so a
              dependency cannot *disable* preserve_order once we enable it,
              but that reasoning has not been tested against a real M3
              dependency graph. D15's conformance exemption is accepted
              risk, not verified safety — rmcp does not validate
              structuredContent, and whether any real client's validator
              rejects a non-conforming error payload is unknown. The
              concurrency finding is two out-of-order captures plus a spawn
              site in source, with no stress test and no measurement of how
              much serialization M3 needs. Carried forward and still true:
              no induced crash or power loss anywhere; nothing calls
              compact() outside tests, so RepairOutcome::Diverged is
              reachable only from a fixture; no human has run
              dump/verify/repair against a real store; multi-process lock
              contention is still two handles in one process (U18's gap);
              the sweep line landed this session and has never been
              exercised by a milestone gate; tools/progress.py has no test
              of its own.
Tracker:      tools/tasks.py check exits 0 — g3 standing (until M5); t25
              done with evidence; one debt open (d1, its note names M2's
              queries); open findings f4 (M3's wire), f25 (parked
              diagnostics), f35 (= U19), f40 (= U15), f44, f45.
              .agents/pitfalls-live.md is still EMPTY — the next entry is
              P-LIVE-007 and a promoted id is never reused. Doc gates
              green: check-links, agents-sync --check, agents-index
              --check, attention --check (and with §12 closed again there
              are no P0/P1 vote items). Code gates skipped by
              construction: no Rust moved.
```

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
