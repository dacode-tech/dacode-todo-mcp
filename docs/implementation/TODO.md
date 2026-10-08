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

**Phase: complete — M5 (packaging, docs, cross-repo ergonomics, polish)
closed 2026-10-08: g13, the exit gate, ran green on the final tree —
every code gate (584 workspace tests), the golden ritual
byte-identical over all 24 goldens, the f41 sweep (a dual-reviewed
doc-low rider closing f105/f101; f106/f107/f108 closed on their
landed pins; the rest verified open with consumers intact), the
packaging/cutover/live-view evidence re-run (both committed drivers,
the live refresh, the smoke transcripts) — closing g3 with it: the
M0–M5 roadmap is delivered. The tree was t52–t58 + t57 + g13 (t51's
decomposition), the two shaping calls confirmed — U28 (dogfooding)
yes, U18 (the live view) shape A with D13 left reserved. t52
(packaging per §13),
t58 (U32's schema-format strip, the user's in-thread sign mid-t53-prep),
t53 (cwd auto-detection, D8's deferred arming) and t54 (the U28
dogfooding cutover — this repo registered and imported in the durable
root, `progress.py` a thin client, U22 resolved) are landed; t55 (U18's
shape A, `progress.py --serve`) landed 2026-10-08 — the live listener
on 127.0.0.1:0 with the per-user port publication (a second invocation
attaches, a stale one is taken over, SIGINT cleans up), f109's census
to stderr; t56 (the adoption README) landed 2026-10-08 — the front
door (wiring, store, tool surface, dialects), both hygiene tools grown
to cover a root README; t57 (U21's duplicate-key check) landed
2026-10-08 — the strict loader in `tools/agents-sync.py` refusing a
duplicate key anywhere in `agent-rules.yaml`, the f42 silent-shadowing
failure made a named hard error, `--check` green on the real config
with both negative proofs recorded; g13 ran green 2026-10-08, closing
g3.**
**t51 — the M5 decomposition — landed 2026-10-07**: the tree is t52
packaging per §13 (the release binary, the global-config mcp entry
naming the durable root `~/.local/share/dacode-todo/`, this repo's own
wiring moved off the smoke-only `tmp/todo-data` — closing g11's standing
caveat that no live client has queried the durable root), t53 cwd
auto-detection (D8's deferred arming, the §7 amendment riding the task),
t54 the U28 dogfooding cutover (this repo registered and imported,
`progress.py` a thin client over a per-render stdio spawn, the four
hand-maintained tables retiring, U22 resolving with it), t55 U18's
shape A (`progress.py --serve`), t56 the adoption README, t57 U21's
duplicate-key check; g13 the exit gate, whose closure also closes g3.
**U28 confirmed (dogfood)** and **U18 confirmed (shape A)** by the user
2026-10-07: the markdown board + `tasks.py` stay the trackers of record
(the write surface is unchanged, the cutover reversible), and U18's
shape B — the shared per-user server instance — is declined with **D13
left reserved**. U27 and U32 are left as recorded calls (the
timing-contract question and the wire-visible schema change are
decisions, not polish).
**t52 — packaging per §13 — landed 2026-10-07**: the release binary
(`cargo build --release -p todo-mcp`, 30.75 s, the tree's first
release build) named by an mcp entry in the user's global opencode
config (`~/.config/opencode/opencode.jsonc`, the cve entry's pattern)
serving the durable D4 root `~/.local/share/dacode-todo/`; the repo's
own t39 wiring MOVED, not copied — the todo-mcp server block in
agent-rules.yaml became a pointer comment, so the project config
renders no entry and nothing shadows the global one, and the
orchestrator's `todo-mcp*` allow is a per-role permission override
beside U16b's deny. The smoke — a real opencode session through the
global entry — answered `repo_list` with the three real repos and
`next_task` = `dacode:U175` (t48's answer, the store unchanged), the
permission=allow lines and tool_use events in the transcript, and all
three stores verify clean post-session: **g11's standing caveat ("no
live client has queried the durable root yet") is closed**. The
staleness tradeoff is documented in the entry's comment: the binary
goes stale after pulls until rebuilt, and the store-format version
gate refuses a too-old binary rather than guessing.
The study measured the dacode board
rather
than reading it: a **three-file system** under `docs/implementation/`
(`TODO.md` 10,811 lines / 1.1 MB, `appendices/B-decisions.md` 3,071,
`appendices/A-design-deltas.md` 3,792), registers all stable and
**contiguous** (`U1`–`U170` with zero gaps — 75 unchecked, 95 checked;
`B1`–`B21` plus the `B1a` oddball; `A1`–`A28`; milestones `M0`–`M6`
(seven, not six) with 15 sub-step table rows; 50 docs), statuses
checkbox + ✅/▶️/⚠️/🧠 with day-precision dates, and dependency
structure split between table columns (B's `Gates`, A's
`Depends on`) and prose ("blocked on <id>" ×10, "before M<n>" ×47).
The full inventory is **D18's evidence base** in `decisions.md`.
**D18 — the dacode-dialect mapping — is signed (a) 2026-10-05**:
`A`-deltas → tasks under the synthetic `Spec deltas` epic, sub-steps
→ `gate`-kind tasks, docs → doc nodes, parentless `U` items under
`Follow-ups`, id reproduction by import order (no Store/domain
change). §9's mapping table and §10's fixture bullet carry the rows;
the conventions stay task-spec level, pinned by t43's tests.
**t42 — the dacode dialect parser — landed 2026-10-05**: the
three-file line parser into a dialect AST (registers, items, statuses,
tables, dependency columns, the recognized prose assertions), strict
named errors for unknown register shapes, pure std, no new deps, no
domain/store change — and `parse_board` is **total over the live board**
(probe-verified: U=170, M=7, 15 sub-steps, 42 docs = 23 expanded + 19
summary matching the board's own header, B 21/21, A 27+21/28). The
strictness rule fired on six real live-board shapes the study's
inventory missed — all widened as measured grammar, each pinned by test.
**t43 — the D18 mapping — landed 2026-10-06**: the AST mapped into an
executable `ImportPlan` (an ordered op list of domain drafts + edges
carrying the ids the store will allocate — D18's id reproduction made
mechanical, the sort load-bearing because §6's file order is not
id-sorted, measured) plus `apply_plan` over the `Store` trait with a
non-empty refusal and a per-add id check; the D18 conventions
operationalized with nine veto-able readings, each pinned by test; the
id-reproduction property pinned at the live board's scale (170 U, 21 B,
28 A, 15 sub-steps, 7 M, 50 docs, loop-generated with planted
inversions). **t44 — the `import_md` wire tool — landed 2026-10-06**:
the importer cluster (`src/importer.rs`) whose params are the repo
slug alone (the board's location derives from the repo node's
registered checkout path plus the measured dacode layout constant),
the `doc_paths` map built by one sorted `.md` walk matching register
keys by stem (measured against the live tree: 42 keys, 43 files,
every key unique, `TODO.md` unmatched), the apply refusal remapped to
§6.1's `invalid` through a typed `ApplyError` seam in `plan.rs`, and
a clock-free counts response golden-pinned as `import_md.v1` — nine
protocol-level tests plus two unit pins, 507 workspace tests green.
**t45 — the checked-in fixture + the end-to-end golden — landed
2026-10-06**: the three real sibling board files checked in
verbatim (the only sibling content this repository holds, D12/§10,
their shas and count pinned by test) and the expected graph
golden-pinned as `dacode_dump.v1` — and the headline fact: **the
first composed run on the real board was clean** (parse, map, apply,
no refusal, no importer code changed), with the census matching the
study's predictions exactly (9 epics, 21 decisions, 42 docs, 218
tasks with the board's `U1`–`U170` reproduced, 43 edges) and the
wire composition driven end to end beside the golden. **t46 — the
camera dialect — landed 2026-10-06**: the dialect study ran first
as measurements against the live board (a two-file board, the U
register in §4/§5, checkbox+strikethrough statuses with no emoji,
§2's milestone table, decisions.md's summary table with its Gates
column — every population §9-named, so no new D-entry was owed),
then the parser + mapping as a sibling module sharing no code with
dacode except the plan types, the `dialect` param on `import_md`
(default dacode), and the live import probe (178 tasks, the base
range reproduced, no golden per D12). **t50 — the U31 lift — landed
2026-10-07**: the plan types moved from `dacode/plan.rs` to the
dialect-neutral `src/import/plan.rs` and the byte-identical helpers
(cells, the table walk, the mapping primitives) into
`src/import/helpers.rs` behind a `HelperError` seam — the third copy
ikvm-html would have forced is not made, and the reaching-through-
dacode for a dialect-neutral contract is gone. Zero behavior change:
524 tests with every per-target count unchanged, both goldens
byte-identical, the sharing proven real by mutations that fail both
dialects at once. **t47 — the ikvm-html dialect — landed
2026-10-07**: the study (recorded in §6's 2026-10-07 entry and the
spec), the parser + mapping as the third sibling module on the
neutral homes, the `dialect` param's `ikvm-html` value, the
code-aware strike widening in the shared helpers, and the live
import probe — **132 tasks (128 U + 4 §3 blockers), 9 epics, 22
decisions, 22 edges, 0 docs, 0 findings, `U1`–`U128` reproduced**,
probed twice (implementer + primary over the wire), no golden per
D12. One live-shape widening beyond the study (the resolution
banner's outcome ends at the bold run's closing `**` — three
closings measured), pinned by synthetic tests. **t48 — the three repos
tracked for real — landed 2026-10-07**: the §9 cutover executed —
dacode/camera/ikvm-html registered in the durable D4 root
(`~/.local/share/dacode-todo/`, spec §2; `tmp/todo-data` stays the
smoke-only harness root per the t39 wiring comment, M5's packaging
moves the entry), each board imported over the wire with **every import
clean on the first composed run** (dacode 9/21/42/218/43 — exactly
t45's golden census, the sibling sitting at the fixture's snapshot
commit; camera 8/9/0/178/10 — the t46 probe's census; ikvm-html
9/22/0/133/22 — the predicted post-`U129` count), each store verified
(`todo-cli verify` exit 0, status ok), and a real query session over
the imported dacode board recorded (`next_task` = `dacode:U175`, the
`M1.1` gate; the read session byte-identical across two runs — §8's
determinism on the real board). The importer lives in `todo-mcp` as a
module —
§2's signed four-member workspace stands, no new crate, no new
dependency (the dialect is line-anchored, measured).

M3 closed 2026-10-04 — g9, the exit gate, ran green on the final
tree. The MCP wire is nine deliveries under tracker **g8** — t32–t40,
each one commit with its own file; §2's M3 row names them and §3 carries
the deliveries. The gate ran every code gate (fmt, check, **423**
workspace tests, clippy, `cargo doc` warning-free under both
invocations), the golden ritual over all three golden dirs
byte-identical, the §6 tool-surface check (25 tools over the wire, the
only exception `import_md` — M4's per D12, exactly the gate's
criterion), the smoke evidence (t39's real opencode session), the f41
sweep (f4/f59/f63/f64/f68/f70/f73/f97/f100 and the ratified deviations
closed; the spec-divergence records superseded by the folding
convention; the rest verified open with named consumers), **U26
reproduced, isolated and fixed** (the gate's own experiment: the
in-process open/close plus a millisecond-scale deferred kernel-side
flock release; a bounded `locked` retry, dual-reviewed; **U27** opened
for the store-side question), and M3's lessons consolidated into
`.agents/pitfalls.md` (1.2, 2.2's extension, 3.3, 3.4, 4.6, 5.9, 5.10;
top rules 13–14).
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
| — | nothing blocks: the M0–M5 roadmap is delivered and the recorded ledger is called — U27 signed (c), the debt ledger acked, U30's fix landing this session | U27's retry is the next `todo-store` task — see §6's handover |

(The user's calls 2026-10-08: **U27 signed (c)** — the bounded
`WouldBlock` retry lands inside `FileLock::acquire` as a dedicated
`todo-store` task; the cross-repo guard the sign rode on, answered
from the code: store locks are per-repo per-file, so parallel
consumers of different repos never contend on a store lock, and the
one shared lock, the registry's, is held for its short critical
section. The debt ledger **U13/U24/U25/U29** acked — no splits
scheduled, the entries stay as the recorded ledger, the named splits
riding their triggers. Earlier: **U32 is resolved** (2026-10-07, the
user's in-thread sign: strip the
integer-family format hints; landed as t58). Scheduled into
M5: **U21** (t57) and **U22** (t54). Resolved: U2/U4/U5/U6/U7/
U8/U9/U10/U11/U12/U14/U17, **U15** and **U19** (both taken by t30),
**U20** (b) 2026-10-03, **U16** 2026-10-04, U3 (superseded by D12), and
now **U28** and **U18**, both confirmed 2026-10-07 at the M5
decomposition (t51) — U28 dogfood, U18 shape A with D13 left reserved.)

## 2. Milestones

Authoritative names: `.agents/config/agent-rules.yaml` → `milestones`;
exit criteria: spec §14. Priority is derived from the Gated-on column by
`tools/attention.py`, never hand-written.

| Milestone | Deliverable | Gated on | State |
|---|---|---|---|
| M0 | spec settled: §12 items 1–3 signed (done 2026-09-25); Cargo workspace skeleton with the lint regime; all gates green | — | done 2026-09-25 |
| M1 | domain model + storage engine: graph, statuses, persistence behind the `Store` trait, golden files | — | done 2026-09-30 — g5 ran green: every code gate, five goldens byte-identical, `cargo doc` clean, pitfalls consolidated |
| M2 | queries: repo/epic selection, dependency resolution, deterministic `next_task`; registry with path updates | **D16** adds one task if signed; no dispatched task waits on it | done 2026-10-02 — **g7 ran green**: every code gate, the five goldens byte-identical under `UPDATE_GOLDEN=1`, `cargo doc` warning-free under both invocations for the first time (f48 settled), §5.2's selection re-proven a pure function of state, the f41 sweep exercised for the first time (f44/f45 promoted to U21/U22, the rest verified open with named consumers), and M2's lessons consolidated into `.agents/pitfalls.md` (P-LIVE-007/-008, the citation rule, the coverage-pin rule). The five deliveries — t27 selection, f47's pins, t28 batch, t29 listing, t30 doc riders, t31 registry queries — are §3's entries; D16 stayed unsigned throughout and nothing waited on it — **signed (a) 2026-10-03, after closure**, its aggregation task landing under M3's goal |
| M3 | MCP server: §6's tool surface over the wire (import_md excepted — M4's, per D12), protocol-level tests, default-repo slug config (D8), smoke against a real client | gates cleared 2026-10-03: the **rmcp ask** signed (rmcp 3.5.0 + `serde_json/preserve_order`, §6.3), **U20** resolved (b) — the registry format stays and the server answers `repo_list` from handles — and **D16** signed (a), adding the domain aggregation task to the decomposition; D14/D15 settled the wire's key order and error carrier on 2026-09-30; **D17** signed 2026-10-04 (priority on the triage surfaces) | done 2026-10-04 — **g9 ran green**: every code gate (423 workspace tests), the golden ritual byte-identical over all three golden dirs, the §6 tool-surface check (25 tools over the wire, `import_md` the only exception — exactly the criterion), the smoke evidence recorded (t39's real opencode session), U26 reproduced/isolated/fixed at the gate itself (U27 opened for the store-side question), the f41 sweep exercised, and M3's lessons consolidated into `.agents/pitfalls.md`. The nine deliveries — t32 aggregation, t33 skeleton, t34/t35 the read tools, t36 the mutations, t37 edges + registry + the default slug, t38 the wire suite, t39 the smoke + U16b, t40 D17's echo — are §3's entries |
| M4 | adoption: the dacode-dialect importer first (**D12**, amending D7), its board the only fixture checked in per §10; camera and ikvm-html imported from their live boards and never copied into this repo; the three repos tracked for real | — (D18 signed (a) 2026-10-05, the same day it was drafted) | in progress — **t42 the parser landed 2026-10-05** (parse_board total over the live board, probe-verified), **t43 the D18 mapping landed 2026-10-06** (the ImportPlan + apply_plan, id reproduction pinned at scale), **t44 the `import_md` wire tool landed 2026-10-06** (the importer cluster, the `ApplyError` refusal seam, the `import_md.v1` golden), **t45 the fixture + e2e golden landed 2026-10-06** (the three real files checked in, the first composed run clean, `dacode_dump.v1` pinning the expected graph), **t46 the camera dialect landed 2026-10-06** (the study, the sibling module, the `dialect` param, the live probe — 178 tasks, no golden per D12), **t50 the U31 lift landed 2026-10-07** (the plan types + shared helpers into dialect-neutral homes under `src/import/`, zero behavior change), **t47 the ikvm-html dialect landed 2026-10-07** (the study, the third sibling module on the neutral homes, the `ikvm-html` dialect value, the live probe — 132 tasks, no golden per D12), **t48 the three repos tracked for real landed 2026-10-07** (registered in the durable D4 root, imported over the wire — every import clean on the first composed run, all three stores verified, the query session recorded: the §9 cutover executed) | done 2026-10-07 — **g11 ran green**: every code gate (fmt, check, **543** workspace tests, clippy on a forced recompile, `cargo doc` warning-free under both invocations), the golden ritual byte-identical over all 24 goldens (19 mcp incl. the fixture golden, 4 store, 1 cli), the §6 tool-surface check (a live `tools/list` drive: exactly the spec table's 26 tools, `import_md` included), the live-run evidence (t46/t47's probes + t48's real imports), **f102 swept** in wire.rs (dual-reviewed, `240e7d2`), the f41 sweep (f99 promoted to **U32**; every other open item verified open with its named consumer intact), and M4's lessons consolidated into `.agents/pitfalls.md` (4.7, 4.8, 5.11–5.13; top rules 15–16) |
| M5 | packaging, docs, cwd auto-detection (deferred by D8), polish | — (the two shaping calls confirmed 2026-10-07: U28 dogfood, U18 shape A — D13 left reserved) | done 2026-10-08 — **g13 ran green**: every code gate (584 workspace tests, 0 failed, 1 ignored), the golden ritual byte-identical over all 24 goldens, the f41 sweep (the doc-low rider `b60c4d1` dual-reviewed closing f105/f101, f106/f107/f108 closed on their landed pins, the rest verified open with consumers intact), the packaging/cutover/live-view evidence re-run (both committed drivers green, the live `--refresh`, the t52/t58 smoke transcripts), pitfalls 5.14/5.15 consolidated, g3 closed with evidence. **decomposed 2026-10-07 (t51)**: **t52 packaging per §13 landed 2026-10-07** (the release binary + the global-config mcp entry + the durable root, g11's no-live-client caveat closed), **t58 U32's schema-format strip landed 2026-10-07** (the user's in-thread sign: the integer-family format hints stripped from every advertised schema, output and input, the Ajv noise gone from every client session), **t53 cwd auto-detection landed 2026-10-07** (D8's deferred arming: the three-tier default param > flag > cwd, the deterministic longest-ancestor match, the §7 amendment riding), **t54 the U28 dogfooding cutover landed 2026-10-07** (the `own` dialect, wire value `dacode-todo-mcp`: this repo's board imported into the durable root — the census matching the prediction exactly, 8 epics / 32 tasks / 17 decisions / 17 edges; `progress.py` a thin client over a per-render stdio spawn, the four hand-maintained tables deleted, U22 resolved with it; the `--refresh` registry surgery with the crash order pinned by a committed driver), **t55 U18's shape A landed 2026-10-08** (`progress.py --serve`: the live listener on 127.0.0.1:0, the per-user port publication with attach / stale-takeover / SIGINT-cleanup pinned by the committed `tools/progress-serve-check.py` driver, f109's census to stderr), **t56 the adoption README landed 2026-10-08** (the front door: what the server is, the §13 wiring reproducing t52's live entry verbatim — machine-verified identical, the store location, the tool-surface pointer, the three-repo adoption story, the dialects; `check-links` and `agents-index` grown to cover the root README, so the standing link gate and the doc index both see it), **t57 U21's duplicate-key check landed 2026-10-08** (the strict loader in `tools/agents-sync.py` refusing a duplicate key anywhere in `agent-rules.yaml`, both negative proofs recorded, `--check` green on the real config); dispatch order t52 → t53 → t54 → t55 → t56 with t57 interleavable (t58 slotted before t53 by the user's call) |

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
       `6d36651`.
- 2026-10-04 — **M3 t37: the edges, registry-from-handles and the
       default slug** — `task_link`/`task_unlink` (the `blocks` and
       `relates` edges, f4's named criterion at the wire: the
       endpoints arrive as untyped id strings, parse into `BlockEnd`
       through the domain's own parsers tried in kind order — task,
       decision, epic, at most one accepting any string — and a pair
       §5.1 forbids answers `invalid` naming both endpoints, the
       domain having made the shapes unrepresentable and §6.1's list
       carrying no code for them), `repo_set_path` (the audited verb,
       clock-free: a `Repo` node has no `updated` and its
       `registered_at` is fixed), `repo_list`/`repo_show` answered
       through the resolve path (U20(b): the registry is the
       membership source, the handles the answer source, one cold
       open per repo per process — pinned by the held-handles test,
       a re-opening implementation contending with its own advisory
       lock), and the default-repo slug (the committed D8: a
       `--repo <slug>` startup flag on `main.rs`, validated at the
       parse so a malformed value is a usage fault, the explicit
       parameter always winning, and the repo-field relaxation —
       `repo: String` → `repo: Option<String>` on the query
       cluster's five, the listing cluster's four and the nodes
       cluster's nine params structs, `GraphShowParams` excluded by
       §6.2's all-repos semantics, `repo_register`/`repo_list` taking
       no repo parameter at all; an absent repo with no default
       answers `invalid` from the tool body, the schema being static
       and unable to express "required iff no default"). New
       `src/edges.rs` (the two tools, the `EdgeKindDto` discriminator,
       the echo response `EdgeDto` — the store returns `()` for
       edges, so there is no node state to return and the echo gives
       the idempotent no-ops their honest shape; the endpoints are
       echoed as parsed, the domain's id `Display`/`FromStr`
       round-tripping every accepted string) and
       `tests/write/edges.rs` (fourteen tests, fresh fixture per
       test); `registry.rs` extended with the three repo tools;
       `main.rs` gains `--repo` and four parse tests; `lib.rs` gains
       `default_repo` + the shared `repo_of` helper + the fifth
       router; `query.rs`/`listing.rs`/`nodes.rs` carry only the
       mechanical repo-field relaxation; `common/mod.rs` gains the
       `--repo` drive variant and the stale-claim sweep (the t36
       hand-off: "the read suites' shared halves" reworded to the
       read *and* write suites). Five new goldens — `task_link.v1`,
       `task_unlink.v1`, `repo_set_path.v1`, `repo_list.v1`,
       `repo_show.v1`, all five tools clock-free (the t36 principle
       applied to its first full cluster) — seventeen in the
       directory, the twelve existing byte-identical throughout.
       **U23 closed here**: the swapped-store fixture test (register
       `aaa`/`bbb`, copy `aaa`'s log over `bbb`'s out of band,
       `repo_show` answers `corrupt_store` naming both slugs) pins
       the resolve cross-check that had been live since t34. The two
       f71-class name arrays grew from twenty to twenty-five tools.
       411 workspace tests. Mutations M1–M6 run, observed failing,
       reverted, sha-verified (kill sets reported; M1's named test 7
       survived — the swapped routing answers the relates forbidden-
       pair message, which also carries `invalid` naming both
       endpoints — M2's named set gained tests 7 and 12, non-task
       endpoints answering `invalid node id` under the mutation, and
       M5's gained test 10, the client flow's first call holding no
       handles; the primary independently reproduced all six and
       added a seventh of its own — deleting the resolve cross-check
       kills exactly test 14, proving U23's pin genuine). Dual review
       over two rounds plus a binding round: round 1 primary ok (four
       lows) / secondary changes_requested (three meds: the nine
       `nodes.rs` field docs joining the relaxation's sentence with
       a period where the other thirteen use a semicolon —
       wire-visible, the docs being the inputSchema descriptions;
       the common/mod.rs sweep replacing one false enumeration with
       another, "six helpers" false the moment `drive_with` landed;
       and **f68's flake recurring three times in ~83 runs** of the
       protocol target during the review — `store is locked: repo
       camera` after the server exits, green on every immediate
       re-run, pre-existing and outside this diff's owned set, so
       **U26** opens for it riding t38's concurrency work); the
       orchestrator applied the fixes (the joins, the count-free
       sweep, the echo doc's vacuous canonical-spelling half dropped,
       the parse doc's declined-alternative clause, the forbidden-
       pair message reworded to name the three legal shapes — a
       called-out regeneration of the new golden — the two unpinned
       D5 semantics given pins in test 6, and `reopened` lifted out
       of its second verbatim copy into the write target's own root,
       `common/` being barred by the read target's compilation of
       it), and both reviewers re-bound ok on identical final-byte
       shas. f92/f93 recorded (the two kill-set divergences). No
       domain, store or cli change; `Cargo.lock` untouched. Landed
       as `29a735b`.
- 2026-10-04 — **M3 t38: the wire suite's cross-cutting pins and the
       concurrency proof** — a tests-only fifth target,
       `crates/todo-mcp/tests/wire.rs` (the t21/f19/f57 shape: no
       production code moves, no goldens, `common/mod.rs` and
       `tests/write.rs` untouched, `tests/protocol.rs` untouched
       because U26 closed as unreproduced), five tests over two
       suite-local mechanisms the home rulings keep out of
       `common/` (the t37 `reopened` precedent): the **rapid-fire
       drive** — all request lines written before any response is
       read, then exactly as many responses read, each matched to
       its request **by JSON-RPC id, never by position** (the
       out-of-order tolerance is the matching rule itself; the
       arrival order measured genuinely out-of-order and varying
       per run) — and the **stamp normalization** (a bounded
       30-byte value replacement after `"created"`/`"updated"`/
       `"registered_at"` in both the structuredContent's and the
       mirror's spelling — normalization, not the write suite's
       extraction). The five pins: the **D15 carrier conformance
       sweep** over twenty-four tools (the unknown-repo uniform
       over the twenty-three repo-scoped tools, `repo_register`'s
       duplicate; `repo_list` excluded and the exclusion documented
       in the test — its cheap fault is codeless by design, its
       carrier paths need a corrupt registry this sweep does not
       build); the **twice-run byte-identity** over twenty-five
       representative happy paths driven across two fresh spawns —
       seventeen clock-free responses byte-exact, eight normalized
       (f95: the spec's eighteen/seven split miscounted
       `repo_register` as clock-free; its `registered_at` is the
       clock's, so the honest split is 17/8 with the third needle),
       plus the success assertion f97 added after M4 showed the
       comparison alone survives a uniformly-broken server; the
       **consistency burst** (three `task_add`s, an `epic_add`, a
       `task_link` and three reads over one connection — the write
       ids asserted **as a set** `{U22,U23,U24}` because the
       dispatch order under the §8 mutex is nondeterministic, every
       response a valid success, and the reopen showing the exact
       nodes and all ten blocks edges); the **write-then-read race**
       (`[task_update(U1's title), task_show(U1)]` — the read
       asserted to be the pre-state or **the write's own response's
       post-state exactly**, stamp included, so a mixed state
       matches neither; both disjuncts observed across runs); and
       the **U26 hammer** at 150 rounds (see U26's entry). The
       mutations, all observed failing then reverted with
       sha-verified restores: **M1** dropping `preserve_order`
       killed 21 tests (read 7, write 12, protocol 2; the wire
       suite survives by design — its comparisons are cross-run or
       parsed-Value, the byte pin is the goldens'; f98: five
       golden tests survive — the three list envelopes and the two
       edge echoes, their DTOs already alphabetically keyed; f96:
       `Cargo.lock` *does* move — the `indexmap` line —
       and was restored with the manifest), **M2** the bare-text
       carrier killed the sweep on its first row plus 37 across
       protocol 3/read 15/write 19, **M3** the diverged mirror
       killed the sweep's mirror assertions plus protocol 3/read
       15/write 18 (the forbidden-pairs test survives it, never
       comparing the mirror), **M4** the resolve cache bypass
       killed wire 3 (the burst, the race, the twice-run — the
       last only after f97's assertion) plus read 16/write 16, the
       sweep surviving because unknown-repo rows fail at the
       registry check before any open, protocol 0 because its
       drives are single-call. **M5 does not exist** — U26 closed
       as unreproduced, no fix landed to revert. 82 crate tests
       (77+5), 416 workspace (411+5), the gate run four times. No
       src, golden, domain, store or cli change; `Cargo.toml` and
       `Cargo.lock` byte-identical at the end. f95–f98 recorded.
       Landed as `709fbdb`.
- 2026-10-04 — **M3 t39: the opencode smoke + U16b — the server wired
       into its own harness and proven by a real client.** The
       orchestrator did the work itself (the routing rule: interactive
       client sessions do not survive a subagent boundary), from
       `tmp/t39-spec.md`. The wiring: `agent-rules.yaml`'s mcp block
       gains the `todo-mcp` entry — `cargo run -p todo-mcp -- --root
       tmp/todo-data`, the relative root resolving against the project
       root where opencode spawns MCP servers (the serena
       `--project-from-cwd` precedent), gitignored scratch
       pre-dogfooding with §13's global-config + durable-location
       move named as M5's — enabled for the orchestrator alone (the
       integration point; the other roles route findings through it
       into `tasks.py`, and widening later is a config edit), and no
       `--repo` default (D8: no dogfooding repo exists to default
       to). U16b: `todowrite: deny` on the orchestrator's permission
       block — the role that gets the server loses the native todo
       tool so the two surfaces cannot mix; this opencode version
       (1.18.34) exposes no `todoread` tool (measured against the
       binary: zero `todoread` hits), so the write half is the pair's
       whole gateable surface — and `orchestrator.md`'s live-view
       paragraph moved with the gate (the live view for that role is
       messages + `tasks.py`; the server is the work surface only
       once dogfooding is decided, which it is not). The smoke: a
       real `opencode run --agent dacode-todo-mcp --format json
       --print-logs` session — the full orchestrator prompt, the
       fresh config, the MCP handshake at session start — drove the
       complete lifecycle through the server's tools only:
       `repo_register` (slug `smoke`) → `epic_add` → `task_add` ×2 →
       `task_link` (U2 blocks U1) → `repo_list` → `epic_list` →
       `next_task`, which answered **`smoke:U2`**, the
       dependency-correct task (U1 is held by the edge). Eight
       tool_use events across seven distinct tools, every one in the
       `todo-mcp_*` namespace, zero native-todo or file calls — the
       deny held by construction. Post-session `todo-cli verify
       --root tmp/todo-data --slug smoke`: exit 0, status ok (2
       tasks, 1 blocks edge, 1403 log bytes) — also U26's watch
       point passed clean: a real client did what the harness does
       and the store survived. The first real-client compatibility
       note is **f99**: opencode's schema validator emits 52
       "unknown format uint64 ignored in schema" warnings against the
       server's schemars-1 `u64` fields — harmless today (ignored),
       but a strict-mode client could hard-error on the same schemas;
       the fix is the server's next owner's. Dual review ok (both
       reviewers, one round, bound to the five file shas — the
       config change is `.agents/`-class, review-exempt by the
       harness rule, but the tracker note asked for eyes on it and
       the change gates every future session's tool surface). No
       Rust moved: every code gate `skipped` by construction, proven
       by an empty diff over `*.rs`/`Cargo.*`. U16 struck (§4).
- 2026-10-04 — **M3 t40: D17's echo — `priority` on the triage
       surfaces.** The agent-set priority (D11's vocabulary, D17's
       echo) now appears where an agent or a dashboard picks by:
       `next_task`'s answer carries the chosen task's priority beside
       its id, `next_items` rows and `task_list` rows carry each
       row's. The domain row types (`NextTask`, `NextItem`,
       `TaskRow`) gained `priority: Option<Priority>` — read off the
       node, never derived — and the wire DTOs mirror it with
       `TaskDto`'s existing spelling (`null` when unset, the key
       never omitted). The base instructions gained D17's handling
       sentence (set it when triaging, prefer higher priority when
       choosing among actionable rows yourself; the server's order is
       dependency-driven and ignores it by design), and the protocol
       suite's frozen copy moved with it. The selection order is
       untouched — D6's canonical order stays dependency-driven — and
       the order-still-priority-free pin (a graph with differing
       priorities answers all three surfaces in the same order as its
       priority-free twin) plus mutation M5 (a priority-preferring
       `select_from`) make that a checked fact: the pin dies on its
       `next_task` assertion, beside the t27 priority pin and the
       `next_task`/first-row cross-pin. The fixture widened to three
       distinct values (U9 high, U10 medium, U11 low), so the moved
       goldens pin high/medium/low beside the nulls: **four goldens
       moved as additive fields — `next_task.v1`, `next_items.v1`,
       `task_list.v1`, and `graph_show.v1` (the widening reaches the
       dump's task nodes); `task_show.v1` measured byte-identical**
       (it pins U1/U2, untouched). Two files outside the spec's owned
       set were forced by the echo and are flagged for review
       ratification (the f71 precedent): `graph/list.rs` (the
       `TaskRow` construction site) and `tests/write/edges.rs` (two
       `next_task` payload assertions). 183 domain tests (+4), the
       read suite 27→30, wire 5 and write 26 unchanged; every gate
       green; M1–M5 run, observed and reverted sha-verified. One
       accepted residual, recorded: the wire suite has no
       order-vs-priority coverage of its own (the fixture's first
       canonical task is also its highest-priority one, so a
       wire-side re-sort would move no golden) — the domain's
       purity pin and the seeded agreement net are the guards, and
       the fixture's next touch should misalign one priority
       against its canonical position to close the gap.
- 2026-10-04 — **M3 g9: the exit gate — and U26 reproduced, isolated
       and fixed inside it.** The gate ran every line of its verify
       method on the committed tree: fmt, check, **423** workspace
       tests (0 failed, 1 ignored — the store's measurement test),
       clippy on a forced recompile, `cargo doc` warning-free under
       both invocations, the golden ritual byte-identical over all
       three golden dirs (store, cli, mcp), the §6 tool-surface
       check (a live `tools/list` drive: 25 tools, the spec's table
       diffed, `import_md` the only exception — M4's per D12,
       exactly the criterion), the smoke evidence (t39, §3), the
       f41 sweep, and the consolidation. The gate's own experiment
       was U26's: the flake reproduced (1-in-30 and 1-in-10
       full-crate runs), an lsof+ps diagnostic found no fd holder
       anywhere with the server dead, a retry-loop diagnostic
       measured the lock still held 1.33ms past the refusal, and
       the no-in-process-open variant ran 40/40 clean — the cause
       isolated as the in-process `repo_list()`'s open/close plus a
       deferred kernel-side flock release. The fix is tests-only,
       in the flaking sequence itself (this entry's own U26 rule):
       a bounded retry on `locked` at the final open — the initial
       open plus five retries 2ms apart, a ~10ms window against the
       measured ~1.3ms lifetime — in both sites (the protocol test
       and the wire hammer, mirrored; protocol.rs is self-contained
       by the t38 ruling). With the fix: 40/40 full-crate runs
       clean. Dual-reviewed at the gate over two rounds (both
       reviewers; the fix round was doc-only — the hammer's header
       rewritten to post-fix detector semantics, the measurements
       moved from the source comments to U26's ledger, the import
       merged). **U27** opened for the store-side question the fix
       deliberately does not answer. The consolidation: seven new
       pitfalls entries (1.2 the Ord trap, 2.2's deferred-release
       extension, 3.3 the single-line output-doc rule, 3.4 the
       id-matching drive rule, 4.6 the per-target test homes, 5.9
       the spec-local id rule, 5.10 the premise assertion) and top
       rules 13–14; f70/f73/f97/f100 closed into it. The sweep:
       f4 (consumed at t37, its wire criterion landed), f59/f63/f64
       (consumed by U20(b)'s answer), f68 (closed with U26), the
       ratified deviations f71/f74/f77/f80, and the spec-divergence
       records f66/f75–f98 superseded by the folding convention
       (a86b1cc) — the records stand in their tasks' evidence, no
       action owed. Deliberately open with named consumers: f25
       (the log backend), f50/f54 (tests/seeded.rs and tests/mod.rs
       owners), f57/f58/f60 (the store's next touches — f58 rides
       U27's lock.rs question), f61 (the cli's next owner), f65
       (the user's, with U21/U22), f67 (select.rs's next owner),
       f69 (spec §6.3's next touch), f94 (edges.rs's next touch),
       f99 (the server's next owner or a decision), U13/U24/U25
       (the ledger and the splits), U18 (parked, after M4's
       importer check), U21/U22 (the user's), U27 (the store's
       next owner). The U26 fix landed as `d3eb929` (dual-reviewed,
       both trailers); the gate's consolidation and board movement
       as `6383c9b`.

- 2026-10-05 — **M4 t41: the decomposition** — the dialect study ran
       as a measurement, not a read: the dacode board is a
       **three-file system** (`TODO.md` 10,811 lines / 1.1 MB,
       `B-decisions.md` 3,071, `A-design-deltas.md` 3,792 — §9's
       dialect sentence now records it), the registers are all
       **contiguous** (`U1`–`U170` zero gaps, `B1`–`B21` + the `B1a`
       oddball, `A1`–`A28`, `M0`–`M6` with 15 sub-step rows, 50
       docs), the status vocabulary is checkbox + ✅/▶️/⚠️/🧠 with
       day-precision dates, and the dependency structure splits
       between table columns and prose (the census is in D18's
       evidence base). **D18 drafted** — the dacode-dialect mapping:
       §9's signed table covers four shapes, the study found five
       populations it does not (the `A`-delta register — the one
       contested choice, tasks recommended over decision nodes on
       the queue-visibility argument; sub-steps → gate-kind tasks;
       docs → doc nodes; synthetic `Follow-ups`/`Spec deltas` epics
       for parentless items; id reproduction by import order, which
       the contiguity measurement makes possible with **no
       Store/domain change**). The task tree: t42 parser → t43
       mapping (gated on D18) → t44 the `import_md` wire tool → t45
       the checked-in fixture + e2e golden → t46 camera → t47
       ikvm-html → t48 the three repos tracked for real → g11 the
       exit gate.        The importer lives in `todo-mcp` as a module (§2's
       signed four-member workspace stands; no new dependency — the
       dialect is line-anchored, measured). No Rust moved: `git
       diff` over `*.rs`/`Cargo.*` empty by construction. Landed as
       `230d49c` (review-exempt, no trailer; the index backfill +
       P-LIVE-009 as `8b37055`).

- 2026-10-05 — **M4 t49: D18's sign-off executed** — the user signed
       option (a) in-thread ("let's do option (a) with 'A-deltas →
       tasks'"), the same day the decision was drafted. The work
       order, executed in one change: §9's mapping table gains the
       five dacode rows (sub-steps → `gate`-kind tasks, `A`-deltas →
       tasks under the synthetic `Spec deltas` epic, parentless `U`
       items under `Follow-ups`, docs → doc nodes, id reproduction by
       import order); §10's fixture bullet names the three files;
       §12 item 18 deleted with the preamble recording the signing;
       the decisions.md row struck with its resolution banner; the
       conventions (status mapping, timestamps, the mechanical-edge
       rule, `B1a`, the not-imported surfaces) stay task-spec level,
         pinned by t43's tests. t43 is unblocked; nothing blocks M4.
         No Rust moved. Landed as `b8bd5da` (review-exempt, no
         trailer; the index backfill as `8a49a4b`).

- 2026-10-05 — **M4 t42: the dacode dialect parser** — the first code
        of M4, and the risk mass of the importer: the three-file
        line parser into a dialect AST, in
        `crates/todo-mcp/src/import/dacode/` (ten files, all under the
        600-line target; `lib.rs` gains `pub mod import;` — the only
        existing file touched). Pure std, no new dependency
        (`Cargo.lock` byte-identical), no domain/store change, no
        mapping (t43's), no wire surface (t44's), no goldens (t45's).
        The design: three per-file parsers plus `parse_board` with
        cross-file id set checks (B set-equality; A one-directional —
        every index/applied-table id must have a dossier, dossiers may
        exceed the index, `A28` measured); the AST carries board order
        in `Vec`s, no serde, ids as the stable keys with 1-based line
        numbers on every named error; **strictness is the load-bearing
        rule** — register lines must parse or error (the
        anti-silent-omission discipline), unknown tables and prose are
        body, tables are recognized by header shape not position
        (measured: tables live inside §6 item bodies); markers (the
        four emoji, `blocked on <id>`, `gates/Blocking/due by M<n>`)
        are extracted only from checkbox items with their raw lines,
        `before M<n>` deliberately not recognized per D18. 46 tests,
        all synthetic corpora (the real files are t45's fixture — the
        review caught and removed pasted live-board prose twice). The
        five mandatory mutations run and re-run across rounds with
        matching kill sets. **The strictness rule earned its keep
        immediately**: a live-board probe (temporary, removed) fired on
        six real shapes the dialect study's inventory missed — §2's B20
        row carrying a fifth trailing cell, B20's prose Status tail,
        B11's `D` appendix doc in a Gates cell, the A file's second
        `| # | Section | Applied | Date | Note |` table (21 rows)
        holding `A28` which the 27-row index lacks, a raw pipe inside
        a code span, and A11's arrow status intro with the
        `APPLIED via A<n>, <date>` form — all widened as measured
        grammar, each pinned by test, plus two implementer-disclosed
        widenings (wrapped table rows, wrapped bold runs — 120 of 170
        live items) that the live board requires. **One cross-repo
        event**: the doc-register contradiction check (a doc both
        expanded and still-summary) fired on the live board's §4 —
        docs 22/23 were expanded per the board's own count paragraphs
        and expanded list but never struck in the M1 row, the board's
        own recorded "two edits" drift — and rather than relax the
        check, the M1 row was struck in the sibling dacode repo (their
        tree left dirty for their owner, the t16 precedent). With that
        one-line fix, `parse_board` is **total over the live board**:
        U=170, M=7, 15 sub-steps, 42 docs (23 expanded + 19 summary,
        matching the board's own "19 of 42" header), §2/B/A
        21/21/21/27/21/28, markers 154/151/31/16/4/0. Dual review over
        five rounds to sign-off, both reviewers bound to the final
        bytes: the primary caught the row-wrap silently swallowing
        register lines (a truncated row absorbed a following milestone
        — fixed with a register-construct termination rule); the
        secondary caught an id-shape off-by-one accepting two trailing
        letters at every id gate, stale docs contradicting the fix
        round's code, measurements in doc comments, and the pasted
        live-board prose; the last rounds swept a false empirical
        claim (the "live A25 header carries prose" rationale, false —
        no live A header carries post-date prose) and its duplicate —
        the class-sweep lesson twice in one review. Every gate green
        on the final tree: fmt, check, **469** workspace tests
        (todo-mcp 131 = 85 + 46 import), clippy, `cargo doc`
        warning-free under both invocations. Landed as `faeca22`.

- 2026-10-06 — **M4 t43: the D18 mapping** — the second code task of
        M4, and the semantic half of the importer:
        `crates/todo-mcp/src/import/dacode/{map,plan,resolve}.rs`
        mapping t42's frozen AST into an executable **`ImportPlan`** —
        an ordered op list (`epics → decisions → docs → tasks →
        edges`) of domain drafts + `BlocksEdge`s, each op carrying the
        id the store **will** allocate, which is D18's
        id-reproduction convention made mechanical — plus
        `apply_plan` over `&mut dyn Store` (refusing a non-empty
        graph or raised counters up front, and comparing every add's
        returned id with the op's constructed id, a named error
        naming both on mismatch). The sort is load-bearing, not a
        no-op: §6's file order is **not** id-sorted (three inversions
        measured: U77 after U90, U111 after U114, U126 after U132),
        and the B summary and A dossier orders are unsorted too — the
        plan sorts by numeric id, and the id-reproduction test plants
        inversions at the live board's scale (170 U, 21 B, 28 A, 15
        sub-steps, 7 M, 50 docs, loop-generated) asserting the
        store's ids are exactly `U1`–`U170`, `D1`–`D21`, `E1`–`E9`,
        `S1`–`S50` and the non-U tasks after them in the documented
        order. The D18 conventions operationalized: the status
        mapping with a precedence rule (the checkbox governs
        closed-ness; the first status-bearing marker in emission
        order refines open states; a ✅ on an unchecked item is
        prose — the live board's 304 ✅s include struck prose);
        sub-step status inheriting the parent milestone's checkbox
        (M0's complete sub-steps carry no Gate-cell marks —
        measured); the mechanical-edge rule (B's Gates milestone →
        `decision→epic`, A's `Depends on` B ids → `decision→task` and
        A ids → `task→task`, `blocked on` by target kind); each
        recognized-but-unresolvable assertion (milestone assertions,
        unknown ids) → one `finding`-kind task; `B1a` → a closed task
        under `Follow-ups` referencing `B1`; timestamps (dossier
        dates → `T00:00:00.000000000Z`, everything else a
        caller-passed `imported_at`); docs with `doc_paths` as a
        required parameter (the three board files do not carry the
        doc filenames — t44/t45 supply the key→path map; a register
        key with no path is a named error). **Nine veto-able
        readings, each pinned by a test** (D18's own rule), recorded
        in the module docs. 27 map tests (73 in the import target);
        the five mandatory mutations run with kill sets re-derived
        independently by the primary reviewer. Dual review over two
        rounds to sign-off, both bound to the final bytes: round 1
        found the references reading unpinned (both reviewers
        deleted the reference pushes and watched the suite stay
        green), the edge timestamps unpinned, a false D18 citation
        (the synthetic epics' description wording is the mapping's
        own, not the signed text's — the f62/5.6 class), the refusal
        gap (a used-then-emptied store passing — both reviewers from
        different angles), and three live-prose lifts in the corpus;
        all landed in the fix round with the probes pinned as tests.
        One ownership deviation, ratified by both: `resolve.rs` —
        map.rs hit 1024 lines after fmt and the 1000 cap is
        unwaivable, so the split landed at the spec's own named seam
        (node mapping vs edge/finding resolution). Every gate green
        on the final tree: fmt, check, **496** workspace tests
        (todo-mcp 158 = 131 + 27), clippy, `cargo doc` warning-free
        under both invocations; the 13 frozen baselines (ast.rs and
        every parser file) byte-identical; `Cargo.lock` unmoved. Two
        follow-ups opened: **U29** (map.rs over the 600-line target,
        the recorded exception) and **U30** (a t42 strictness gap the
        primary found: duplicate B/A dossier ids pass the set
        cross-check — no seen-map, unlike U ids — and would land a
        duplicated decision; frozen code, the parser files' next
        owner). The refusal's wire code is a t44 spec input,
        recorded in the tracker. Landed as `f3d08c6`.

- 2026-10-06 — **M4 t44: the `import_md` wire tool** — §6's one
        missing tool, and the importer's I/O half: a new importer
        cluster (`src/importer.rs`, 291 production lines) whose params
        are the repo slug **alone** — the board's location derives from
        the repo node's registered checkout path (§7's attribute, one
        source of truth; `repo_set_path` moves it and the import
        follows) plus the measured dacode layout constant
        (`docs/implementation` with the three filenames as constants),
        so t48's flow is `repo_register` then `import_md`. The
        `doc_paths` map is built by one recursive `.md` walk of the
        board directory, sorted by walk-relative path bytes, matching
        register keys by stem (`stem == key` or `stem` starts
        `key-` — measured against the live tree: 42 keys, 43 files,
        every key exactly one file, `TODO.md` the only unmatched);
        values are repo-relative per §9's signed text; a key matching
        two files is refused as `invalid` naming both (never
        first-wins — the silent-omission class), while a key matching
        none is left to `map_board`'s `MissingDocPath` (one home). The
        apply refusal is remapped to §6.1's `invalid` through a typed
        **`ApplyError { Refused, Store }`** seam in `plan.rs` — a
        client-fixable precondition is not corruption, and a new §6.1
        code would be a spec change — with `apply_plan`'s signature
        changing (its only callers were the map suite's five sites);
        store faults keep the store's codes, parse and map errors land
        as `invalid` through one `From<ImportError>`, and file-read
        faults are `io` in the store's attempt+`ErrorKind` convention
        naming the file's role, never a path. The response is a
        clock-free counts DTO (`repo`, `epics`, `decisions`, `docs`,
        `tasks`, `blocks`) read off the graph **after** apply —
        golden-pinned as `import_md.v1` (the repo_register precedent:
        stamped responses get in-test pins, clock-free get goldens) —
        with the one `time::now()` read asserted in-test around the
        extracted stamps. Nine protocol-level tests drive the real
        binary (the happy path with the golden and follow-up reads,
        the second-import refusal, unknown repo, malformed board, io,
        missing and colliding doc keys, the D8 default-repo cases, the
        argument-fault shape) plus two in-file unit tests pinning the
        pure `doc_paths` seam (the `time.rs` precedent — the crate's
        first unit tests beside it). The review's round 1 was the
        load-bearing one: the primary's own mutants survived the
        9-test suite twice — dropping the `.md` filter (the stray
        non-`.md` file matched no key either way) and dropping the
        sort (APFS `read_dir` order coincides with byte order for the
        test's name set, replicated three times) — both test-gap
        findings, both fixed by making the pins load-bearing (an
        extension-less `README` beside `README.md`; the sort moved
        into the pure seam with a deliberately-reversed-input unit
        test), both mutants now dying. One ownership deviation,
        ratified by both reviewers (the f71 precedent): the read
        target's tools_list walker holds an exhaustive 25-name pin
        that had to gain `import_md` — the spec's "covers it for
        free" claim did not reproduce. Every gate green on the final
        tree: fmt, check, **507** workspace tests (todo-mcp 169 = 8
        lib + 8 main + 82 import + 10 protocol + 30 read + 5 wire + 26
        write), clippy, `cargo doc` warning-free under both
        invocations; the golden ritual in order with the 17 existing
        wire goldens byte-identical; the 25 frozen baselines hold;
        `Cargo.lock` unmoved. Two findings opened: **f101** (the
        missing-golden hint names the `read::query` filter —
        regenerates neither this golden nor the write goldens; the
        helper is a verbatim twin of the store's, so a fix propagates
        to both) and **f102** (`tests/wire.rs`'s "all twenty-five"
        falsified by the 26th tool; the file was spec-frozen, sweep at
        its next touch). Landed as `276017c`.

- 2026-10-06 — **M4 t45: the checked-in dacode fixture + the
        end-to-end expected-graph golden** — the importer's last
        unverified gap closed, and the headline fact is the cleanest
        possible outcome: **the first composed run on the real board
        was clean** — parse, map (335 ops) and apply all succeed with
        no refusal, so no importer code changed (the task spec's
        conditional-ownership branch for mapping fixes never fired).
        The fixture: the three sibling board files copied verbatim
        into `crates/todo-mcp/tests/fixtures/dacode/` (mirroring the
        dialect's own layout), their SHA-1s and the directory's file
        count pinned by test — the **only** sibling content this
        repository holds (D12/§10), so a fixture edit or an
        accidental addition is a named failure, never a silent
        drift. The golden (`dacode_dump.v1`, 1.1 MB, one canonical
        line): the cli's `Dump` document shape mirrored test-locally
        (the per-crate rule — no cross-crate test dependency), built
        off the graph after the in-process pipeline runs under a
        **fixed** `imported_at` (the wire tool's clock makes a
        wire-driven dump un-pinnable; the tool layer is pinned
        separately), serialized with the store's own codec. It pins
        the whole expected graph byte-for-byte: 9 epics, 21
        decisions, 42 docs, **218 tasks with the board's `U1`–`U170`
        numbering reproduced** (D18's id-reproduction convention
        proven on the real board), 43 edges, 465 timestamps all the
        fixed constant or board-fixed dossier dates. A second test
        drives the real binary end to end — `repo_register` →
        `import_md` → `graph_show` against the assembled tree (the
        three real files plus empty `{key}.md` stubs derived from
        the parsed register, one derivation feeding both the
        pipeline's `doc_paths` map and the wire tree) — asserting
        the same census through the tool, so the pipeline and the
        tool are cross-checked against one answer. The census
        matched the dialect study's predictions exactly, including
        the 4 findings (the predicted M1 milestone assertions) and
        the A index's 27 rows vs 28 dossiers (the live board's own
        shape). The review: round 1 both reviewers changes_requested
        on the **same** med — the import target's module doc
        falsified by the diff itself ("its prose is never copied
        into this repo") — plus a wrong-mechanism comment on the
        stub guard; the orchestrator applied both prescribed fixes
        (the t11/t30 precedent) and the binding round re-bound both
        ok. The primary re-ran all five mutations (including the
        production mutation in `map.rs`, the task-closure arm) plus
        its own sixth (the mirror's prefix arm), all with matching
        kill sets; the in-test SHA-1 (no new dependency) was ruled
        sound — FIPS 180-1 correct, the known-vector pin bites, the
        digests cross-validated against the host `shasum`. Every
        gate green on the final tree: fmt, check, **510** workspace
        tests (todo-mcp 172 = 8 lib + 8 main + 85 import + 10
        protocol + 30 read + 5 wire + 26 write), clippy, `cargo doc`
        warning-free under both invocations; the golden ritual in
        the stage-first order with the 18 existing goldens
        byte-identical; all 37 frozen baselines hold; `Cargo.lock`
        unmoved. One process note: the first implementer spawn died
        on a connection error and the control-spawn-then-retry
        remedy (pitfalls 5.4) carried it — proven a third time; the
        dead spawn had already copied the fixture, which the retry
        verified verbatim and kept. Landed as `d315e9e`.

- 2026-10-06 — **M4 t46: the camera dialect** — the first of the two
        live-test subjects (D12: its board content is never copied
        into this repository — no golden, the signed consequence).
        The **dialect study ran first**, the t41 precedent, as
        measurements against the live board at
        `/Users/mouse/RPM/shelly/camera`: a **two-file board**
        (`TODO.md` + `decisions.md`; `merge-requests.md` and
        `product-av.md` are not board files); the U register is
        §4+§5's checkbox items — **base `U1`–`U169` contiguous plus
        eight oddballs** (U6a–d, U46a–c, the struck `U43
        (original)` variant; the study's first census undercounted
        by the seven space-indented suffixed items, corrected at
        review); **statuses are checkbox + strikethrough only — the
        emoji census is empty**, exactly §9's signed vocabulary;
        milestones are §2's table rows (the State cell carries the
        status); `decisions.md` holds a summary table
        (Recommendation/Confidence/Gates/Status columns) plus
        dossiers — 9 decisions, the Gates column nine mechanical
        `decision→epic` edges; the register dependency prose is
        tiny (held-back-by ×1, wrapped; blocked-on ×0 — the
        study's "blocked on U79" lives in §6's narrative, walked
        past); no doc register → no doc nodes; UB ids are not a
        register. **Every camera population is §9-named — no new
        D-entry was owed**; the differences from dacode are shapes
        and conventions, settled as six veto-able readings each
        pinned by test. The module (`src/import/camera/`, six
        files, all under the file budget) **shares no code with
        dacode except the plan types** — camera's mapping emits the
        same `ImportPlan` and executes through the same
        `apply_plan`. The base register reproduces its numbering on
        import; the oddballs land after it carrying their original
        ids in references (§9's signed fallback). `import_md` gains
        a **`dialect` param** (default `dacode` per D12's ordering —
        the t44 requests and its golden stay byte-identical
        unedited; an unknown value is `invalid`; §6's signed plural
        "the sibling markdown boards" is the authority). The **live
        import probe** ran against the real board and was removed
        (rg-proven gone; re-run independently by the primary,
        in-process and over the wire): **178 tasks** (the base range
        reproduced as `camera:U1`–`U170` — the board grew its own
        U170 mid-task — plus the eight oddballs carrying their
        original ids), 8 epics (6 milestones + the `Follow-ups` and
        `Debt` synthetics), 9 decisions (8 signed + D3 open), 0
        docs, 10 blocks; plan==plan at live scale. **Measured
        grammar widened per the t42 precedent**: two of seventeen
        struck items have an unclosed `~~` (U104, U84) — the
        leading `~~` alone is the strike, pinned by test. The
        review earned **three rounds**: round 1 both reviewers
        changes_requested — the primary's own mutants survived the
        suite (the unpinned DuplicateId/BadDate/word-boundary/
        wrapped-join paths) and the secondary found the blocked-on
        edge with no positive case, `push_finding_ops` firing in no
        corpus, `import/mod.rs`'s doc falsified by the diff itself
        (the f55 class, again), the never-fired error variants, a
        two-word live-prose lift, the untested camera io path, and
        copied visibility; round 2 landed eleven items including
        one behavior widening (the phrase scan now skips non-id and
        boundary-violating occurrences — live-neutral, re-probed by
        both); round 3 caught a **real defect**: the per-register
        duplicate check missed a U id duplicated **across §4/§5**
        (camera's U namespace spans two sections; the probe showed
        a silent double-import) — the check is hoisted to one
        namespace and pinned. Every gate green on the final tree:
        fmt, check, **524** workspace tests (todo-mcp 186 = 8 lib +
        8 main + 99 import + 10 protocol + 30 read + 5 wire + 26
        write), clippy, `cargo doc` warning-free under both
        invocations; both goldens byte-identical (no camera golden
        exists); all 39 frozen baselines hold — the dacode module,
        its tests, the t44/t45 suites **unedited**; `Cargo.lock`
        unmoved. Two follow-ups recorded: **f103** (§2 has no
        duplicate-M refusal — taste-level, rides the next
        camera/ikvm touch) and **U31** (the dialect-helper
        duplication: ikvm-html would make the third copy — resolved
        by t50, the lift). Landed as `fe14bc6`.

- 2026-10-07 — **M4 t50: the U31 lift** — the plan types
        (`ImportPlan`/`PlanOp`/`ImportOptions`/`apply_plan`/
        `ApplyError`) moved from `dacode/plan.rs` to the
        dialect-neutral `src/import/plan.rs`, and the helpers the
        two dialects had been duplicating — `cells` and
        `collect_rows` byte-identical across `dacode/table.rs` and
        `camera/text.rs`, `position`/`references`/`bad_reference`/
        `midnight`/`id_sort_key` identical modulo the error type
        across the two `map.rs`s — lifted into
        `src/import/helpers.rs` behind a new `HelperError` seam
        (each dialect's `From` maps variant-for-variant, so every
        error message is byte-identical to before).
        `collect_rows` gained an `is_construct` parameter because
        the two dialects' register-construct rules differ
        load-bearingly (dacode: an indented checkbox is body text;
        camera: any indentation is a register item) — the shared
        walk must not pick a side. Clean cut, no compatibility
        re-export: every consumer imports from the new homes.
        **Zero behavior change is the contract and the proof**:
        524 workspace tests with every per-target count unchanged,
        both goldens byte-identical, `cargo doc` warning-free under
        both invocations, and the mutations that prove the sharing
        is real — corrupting `helpers::cells`/`collect_rows`/
        `position` fails **both** dialects' suites at once, which
        two copies could never do. Four frozen-file deviations,
        each ratified by both reviewers: `dacode/items.rs` and
        `dacode/resolve.rs` (use-line splits the spec's owned set
        missed), `camera/error.rs` and `dacode/ast.rs` (one stale
        doc line each, class-swept — the f55 lesson). The review's
        two rounds: round 1 both changes_requested on doc-level
        stale claims the move created (the "one thing the two
        dialects share" appositive, three wrong-home pointers, the
        `ImportPlan` doc's order reference) plus one visibility
        nit (`bad_reference` needlessly `pub(crate)`); the binding
        round both ok on the fixed bytes, the primary hashing
        SHA-1 and the secondary SHA-256 — every file matching
        under its respective algorithm. One coupling the move
        exposed and records rather than fixes (**f104**):
        `ApplyError::Refused` carries dacode's `ImportError` (the
        refusal variants are store-level facts with dialect-neutral
        messages, but the type is one dialect's) — named in
        `plan.rs`'s module doc, the neutral refusal type is the
        debt for its next owner. Landed as `e8813cc`.

- 2026-10-07 — **M4 t47: the ikvm-html dialect** — the last of the
        three dialects (D12: a live-test subject, its board content
        never copied into this repository — no golden, the signed
        consequence). The **study ran first**, the t41/t46 precedent,
        as measurements against the live board at
        `/Users/mouse/RPM/ali/ikvm-html` (clean at `090c2e1`): a
        **two-file board** (`TODO.md` 15,589 lines +
        `decisions.md`; `README.md` is the plan index, not a board
        file); the U register is §4+§5's checkbox items —
        **`U1`–`U128`, zero gaps, no oddballs**; **statuses are the
        checkbox alone — strikethrough here is edit history** (8
        unchecked items carry struck title fragments: re-measured
        line counts, renamed scopes), which falsifies §9's "and
        strikethrough state" half for this board and became the
        first of four veto-able readings; §2's milestone table has
        **no State column** (milestone status not derivable — all
        epics open, the third reading); **§3 holds four id-less
        blocker checkbox items** — a population neither sibling has
        (tasks under a synthetic `Blockers` epic, the second
        reading); decisions.md carries a summary table + dossiers +
        a **Resolved footer** whose dates agree with the dossier
        banners exactly (20/20, zero mismatches — a pinnable
        three-way cross-check, one of four); the Confidence
        vocabulary is wider than camera's (`low-medium`,
        `medium-high` — both → `Medium`, the fourth reading); and
        the hard grammar: **the bold markers carry no title
        boundary** (U69's `**` tokens are all paired emphasis —
        measured), so the title rule is **first-sentence** (the
        first `.` outside struck and code spans), verified total
        over 128/128 by script. Zero mechanical dependency prose in
        register bodies — the Gates column is the only edge source.
        The module (`src/import/ikvm/`, five files, all under the
        file budget) consumes t50's neutral homes and shares no code
        with the other dialects below them. The **live import
        probe** ran twice (the implementer; the primary over the
        wire at `db23606`) and was removed (rg-proven gone): **132
        tasks** (128 U + 4 blockers), 9 epics, 22 decisions, 22
        edges, 0 docs, 0 findings, `U1`–`U128` reproduced,
        plan==plan at live scale — the board has since grown its own
        U129 (`3eec586`, the t46 mid-task-drift precedent). One
        **live-shape widening beyond the study**, reported per the
        t42 precedent: the resolution banner's outcome ends at the
        bold run's closing `**`, not at `.` — three closings
        measured (`.**` ×18, `** (` and `**,` one each), pinned by
        synthetic tests. The shared helpers gained the
        **code-aware strike scanner** (`strike_regions`; `~~` inside
        backticks is literal — the live U74 discusses `~~` in
        backticks), dacode-neutral both directions (the golden
        byte-identical; the primary's M7 mutation kills 8 ikvm
        tests while dacode's e2e stays green). The review's two
        rounds: round 1 both changes_requested — the primary found
        the never-fired error pins unpinned (MissingRegister/
        ColumnCount/BadId/BadDate — camera pins all four) and the
        id shapes letter-tolerant where the spec says strict; the
        secondary found lib.rs's stale "arrive later" claim, the
        `strike_regions` worked example off by one, BadId's doc
        naming a U shape that never raises it, a miscounted
        "Two variants", verbatim live tokens in the corpora (the
        t42 lesson), unasserted mappings, duplicated definitions,
        and a rewrap space. The fix round landed ten items (the
        strict `M<digits>`/`D<digits>` shapes, the new pin test,
        the dedup, the synthetic tokens, the assertions); the
        binding round both ok on identical SHA-256 prefixes. Three
        ratified owned-set deviations: `tests/import/camera.rs`
        (one line — the invalid-dialect message pin), `src/lib.rs`
        and `camera/error.rs` (doc-only, class-swept). Every gate
        green on the final tree: fmt, check, **543** workspace
        tests (todo-mcp 205 = 8 lib + 8 main + 118 import + 10
        protocol + 30 read + 5 wire + 26 write), clippy, `cargo
        doc` warning-free under both invocations; both goldens
        byte-identical; all 39 frozen baselines hold; `Cargo.lock`
        unmoved. Findings: **f105** (two doc-level lows the binding
        round dispositioned — a live-census count in a source doc
        comment, a ragged rewrap) and **f103 widened** (the
        duplicate-D question now named for both dialects' decisions
        registers, plus camera/error.rs's BadId doc). Landed as
        `80cb1de` (with `3d7623a` declaring the suite — the mod file
        sits beside the directory the staging path named).
- 2026-10-07 — **M4 t48: the three repos tracked for real** — the §9
     cutover executed for all three siblings, no code moved. The
     durable D4 data root (`~/.local/share/dacode-todo/`, spec §2 —
     fresh; `tmp/todo-data` stays the smoke-only harness root the t39
     wiring comment names, and M5's packaging moves the entry to the
     global config with this store location per §13) holds three
     registered stores: `repo_register` for **dacode**
     (`/Users/mouse/RPM/dacode/dacode/dacode`, clean at `671ce48`),
     **camera** (`/Users/mouse/RPM/shelly/camera`, at `c6783f5`, board
     files clean — one untracked PDF beside them) and **ikvm-html**
     (`/Users/mouse/RPM/ali/ikvm-html`, clean at `24734f6`), then
     `import_md` over the wire for each (dacode by the default
     dialect, camera and ikvm-html by the `dialect` parameter).
     **Every import landed clean on the first composed run**: dacode
     9 epics / 21 decisions / 42 docs / 218 tasks / 43 blocks —
     exactly t45's golden census, the sibling sitting at the
     fixture's snapshot commit so the real store's graph is the
     golden's graph; camera 8/9/0/178/10 — the t46 probe's census;
     ikvm-html 9/22/0/133/22 — the predicted post-`U129` count.
     `todo-cli verify` on each store: exit 0, status ok (log bytes
     1,135,746 / 380,313 / 82,473). The query session — a fresh
     server process, so the stores reopen from disk — answered
     `next_task` = `dacode:U175` (`M1.1`, the first open gate under
     E2, verify "docs only — review-exempt"), `epic_list` 9 rows
     (E1–E7 the milestones, E8 `Follow-ups`, E9 `Spec deltas`), and
     `task_list` under `dacode:E8` = 174 rows: `U1`–`U170` contiguous
     (D18's id reproduction on the real board) plus `U214` (B1a,
     closed) and the three `gates M1` findings, canonical order, no
     blockers — the fourth finding (`U218`, "M1 gates M1") sits under
     E2, the milestone it asserts about. The whole read session is
     **byte-identical across two runs** — §8's determinism
     demonstrated on the imported board. Transcripts:
     `tmp/t48-{import,query}-transcript.txt` (gitignored scratch;
     the salient lines are in the tracker evidence). No golden moved,
     no sibling content copied in (D12). The cutover state per §9:
     the server is each repo's state of record as of the import
     instant, the markdown boards keep the narrative, and there is no
      two-way sync (D7) — the stores and boards will drift, and the
      sibling HEADs above are the drift baseline.
- 2026-10-07 — **M5 t51: the decomposition** — the tracker tree (g12 +
      t52–t57 + g13), board §2's M5 row naming the ids, and the two
      shaping calls taken by the user in the opening exchange: **U28
      confirmed (dogfood)** — t54 lands the cutover with the markdown
      board + `tasks.py` unchanged as trackers of record — and **U18
      confirmed (shape A)** — t55 lands `progress.py --serve`, shape B
      declined with **D13 left reserved**. U21 scheduled as t57, U22
      resolves with t54; U27 and U32 left as recorded calls with the
      reasons on their entries. The ledger re-measured: `log.rs` 812 /
      `event.rs` 772 unchanged, `layout.rs` 543 under target,
      `listing.rs` 689 (t40's echo), `map.rs` 733 (t50's lift, the
      helpers now in `src/import/helpers.rs` at 382), `plan.rs` 244 —
      no new ledger entries, U24/U29's figures refreshed. No Rust
      moved. Landed as `2352f89`.
- 2026-10-07 — **M5 t52: packaging per §13** — the release binary and
      the global-config wiring, no Rust moved. `cargo build --release
      -p todo-mcp` (the tree's first release build, 30.75 s) produces
      `target/release/todo-mcp`; the mcp entry in the user's global
      opencode config (`~/.config/opencode/opencode.jsonc`, the cve
      entry's home) names the binary and the durable root
      `~/.local/share/dacode-todo/` per §13's signed text — the
      staleness tradeoff (stale after pulls until rebuilt; the
      store-format version gate refuses a too-old binary) documented
      in the entry's comment. The repo's own t39 wiring moved, not
      copied: the todo-mcp server block in agent-rules.yaml became a
      pointer comment (a servers entry renders into the project config
      and would shadow the global one — the cve pattern is a global
      server with no agent-rules declaration), so `.opencode/
      opencode.json` and `.mcp.json` render no todo-mcp entry, and the
      orchestrator's `todo-mcp*: allow` is a per-role permission
      override beside U16b's `todowrite: deny` (the generated
      orchestrator file is byte-identical to HEAD — the allow's source
      changed from enable_for rendering to the role override, its
      position and text did not). The smoke: a fresh `opencode run`
      session with the orchestrator agent drove `repo_list` (the
      three real repos with their checkout paths and t48's
      registration stamps — the durable root, not `tmp/todo-data`)
      and `next_task` repo=dacode (`dacode:U175`, deferred
      {total 6, ready 6} — t48's answer, the store unchanged), the
      permission=allow lines and tool_use events in
      `tmp/t52-smoke-transcript.txt` (gitignored; the salient lines in
      the tracker evidence); `todo-cli verify` on all three stores
      post-session: exit 0, status ok. **g11's standing caveat — no
      live client has queried the durable root yet — is closed.**
      Landed as `8b62aad`.
- 2026-10-07 — **M5 t58: U32's schema-format strip**: the
      integer-family `format` hints (schemars' OpenAPI spellings)
      removed from every schema the server advertises, so client-side
      JSON Schema validators stop warning on every `tools/list` —
      opencode's Ajv printed `unknown format "uint64" ignored` lines
      on each session (f99, board U32) — and a strict-mode client
      cannot reject the schemas. One strip walk in the new
      `src/schema.rs` (the family: every primitive integer spelling
      schemars emits, `int8`…`int128`, `uint8`…`uint128`, plus the
      `isize`/`usize` spellings `int`/`uint`; standard formats like
      `date-time` survive; `float`/`double` are out of scope because
      §8 bars floats from the wire), applied at two seams:
      `crate::schema::output_for_type` replacing rmcp's
      `schema_for_type` at all 26 `output_schema` sites (six cluster
      files, same-line replacements, `Arc::make_mut` detaching from
      rmcp's thread-local cache so the cached schema is never
      mutated), and `#[schemars(schema_with)]` on
      `next_items`'s `limit` — the one `u64` input field — delegating
      to schemars' own `Option<u64>` spelling so nullability is never
      hand-written. The class is closed from the wire: a `tools/list`
      test walks every advertised schema (input and output, `$defs`
      included) with its own independent copy of the family list, so
      a future integer field that bypasses both seams fails it; a
      second pin holds `limit`'s nullable-integer shape. No golden
      moved, no signed §6 surface moved, no §12 item opened (U32
      resolved by the user's in-thread sign, the f2/U5 precedent).
      210 todo-mcp tests (205 + 5), 548 workspace; the five mandatory
      mutations all killed at their predicted tests; the live smoke —
      release binary rebuilt, a real opencode session through the
      global-config entry — shows **zero `unknown format` lines**
      where the t52 transcript held dozens. Dual-reviewed to
      sign-off over two rounds (the primary's round-1 med — the
      family held 8 of schemars' 12 integer spellings, measured
      against its `primitives.rs` — fixed in round 2; the secondary
      re-bound on the final bytes).
- 2026-10-07 — **M5 t53: cwd auto-detection — D8's deferred arming**:
      an absent `repo` parameter now resolves through three tiers
      (§7, amended by this task as D8's signed work order): an
      explicit parameter, the `--repo` flag, then the working
      directory's deterministic match — the registered checkout whose
      canonicalized path is the longest component-wise
      ancestor-or-equal of the canonicalized cwd, ties between
      distinct slugs refused with both named, no match keeping the
      no-default behavior. The binary reads the cwd once at startup
      and passes it in (the lib stays env-free); the derivation is
      lazy — the first default-needing call walks the registry,
      resolves every slug through the handle cache (t31's cold-open
      cost, once per process) and caches the answer for the process
      lifetime (a later `repo_set_path` or `repo_register` does not
      move it); a registered store that will not open fails the
      derivation with its own error (the `repo_list` precedent —
      never a silent subset), and the errored derivation leaves the
      memo pending so the retry re-fails deterministically. The
      matcher is a pure function in the new `src/cwd.rs`
      (canonicalize both sides, skip unresolvable candidates,
      component-wise never string prefixes); `repo_of` owns the tier
      order, widened to `&mut` with no cluster-file edits (every call
      site already held `&mut`). Eleven wire tests in the new
      `tests/cwd.rs` target (the ten required plus the review round's
      errored-derivation pin) plus nine unit tests; the six mandatory
      mutations all killed at their predicted tests; the golden
      ritual byte-identical; the live smoke — the release binary
      against the durable root, spawned in the camera checkout —
      answered `next_task` with no repo parameter `camera:U6`,
      byte-identical to the explicit `repo: "camera"` call. The spec
      amendments (§2/§6/§7/§13) execute D8's deferral, ratified at
      review; two frozen-file message pins updated minimally (the
      f71 precedent, ratified). 230 todo-mcp tests, 568 workspace.
      Dual-reviewed to sign-off over two rounds (the round-1 meds —
      a stale doc quote in a frozen-deviation file and the
      `default_repo` field doc's pre-amendment framing — fixed; the
      round-1 lows — §6's "ancestor" precision, §7's fail clause,
      §13's spawn-directory phrasing, the unpinned errored-derivation
      path — all landed or dispositioned).
- 2026-10-07 — **M5 t54: the U28 dogfooding cutover**: this repo is
      tracked for real — the `own` dialect (wire value
      `dacode-todo-mcp`, the fourth sibling module on the neutral
      homes) reads this repo's board, and the durable root now holds
      four repos: the import landed clean on the first composed run
      with **the census matching the spec's prediction exactly** (8
      epics — M0–M5 plus the synthetic Follow-ups and Debt; 32 tasks
      with the board's U ids reproduced; 17 decisions; 17 D→M edges;
      0 docs), `next_task` answering `dacode-todo-mcp:U21` — and,
      composed with t53's arming, `next_task` with **no repo
      parameter, the server spawned in this repo's cwd, answers the
      same**: the cutover's whole point in one response. The study
      corrected the decomposition note's premise: the layout fits
      camera exactly (the shared `docs/implementation` board dir and
      camera's two-file set); the grammar differs in measured cell
      shapes — the 4-column §2 table with bare `M<n>` cells, the
      `done <date>` State vocabulary, camera's exact decisions-table
      columns with the id struck in column 1 (the spec's study
      misread the header; the implementer measured it), no item
      strikes, no register dependency prose. `progress.py` is now a
      thin client: one `graph_show` per render over a per-render stdio
      spawn of the release binary, the LANES/GATING/GROUPS/EXTRA/
      WAITS tables **deleted from the source** (rg-proven), the lanes
      derived from the epics, the decisions section rendering all
      seventeen (**U22 resolved**), the tracker overlay clearly
      marked as session state, the footer showing the import instant
      so staleness is visible. The `--refresh` flag re-registers and
      re-imports — a registry surgery with a stated crash order
      (registry entry dropped before the store dir is removed, so a
      crash leaves an invisible orphan, never the dangling entry that
      breaks `repo_list` for every repo), pinned by the committed
      `tools/progress-refresh-check.py` driver. 246 todo-mcp tests,
      582 workspace, the golden ritual byte-identical, six mutations
      killed, the page twice-run byte-identical with the refresh
      round-trip differing only in the import-instant line.
      Dual-reviewed to sign-off over two rounds (the round-1 meds —
      live tokens in the synthetic corpus, the under-reporting
      unknown-dialect message — fixed with the three pins that
      text-match it; the lows — duplicate M/D ids as named errors,
      the committed crash-order driver, the false 'neither' doc
      claim, the 6-cell residual pin — all landed; f109 rides t55:
      the refresh census line belongs on stderr for `--json`
      consumers).
- 2026-10-08 — **M5 t55: U18's shape A — `progress.py --serve`**: the
      live view. `--serve` binds 127.0.0.1:0 (never a chosen port — the
      assigned one is read back from the socket), publishes it to the
      per-user runtime file `~/.local/state/dacode-todo-mcp/serve.port`
      (pid-temp + atomic rename, the store's own write discipline), and
      answers every request with a fresh render — `/` the page, `/json`
      its machine twin, `/health` the marker without rendering, 404
      elsewhere — so a browser refresh re-renders: the page is as live
      as the last import (the footer's import instant stays the
      staleness signal; `--refresh` composes, running once before the
      listener starts). A second invocation probes the publication and
      **attaches** — same URL, exit 0, no second listener; a dead,
      stale or foreign publication is **taken over** (a foreign server
      is never killed, only replaced in the publication); SIGINT is the
      clean shutdown (exit 0, the publication removed iff it still
      names our port — a successor's file is never removed). One
      listener per user, serving the root it was started with;
      `PROGRESS_SERVE_PORT_FILE` is the test seam (CRASH_SEAM's
      pattern) so the committed driver never touches the real file.
      Single-threaded `HTTPServer` on purpose: requests serialize, so
      concurrent GETs never contend on a store lock mid-render. The
      fail-fast startup render makes a wrong root a startup error, not
      a listener that answers 500 on every request. **f109 closed**:
      the refresh census line moved to stderr — `--json --refresh`
      stdout is pure JSON. Pinned by the committed
      `tools/progress-serve-check.py` (20 checks: the static default,
      publish/answer, attach, stale takeover, SIGINT cleanup, f109 —
      its "serving line names the published URL" check caught a real
      defect on its first run: the URL line sat unflushed in a
      block-buffered pipe, fixed with `flush=True`) plus the live smoke
      against the real durable root (tmp/t55-serve-smoke.txt: one
      listener via lsof, the attach, the SIGTERM stale state, the
      takeover; its step 7 could not run from a shell script —
      non-interactive shells start background jobs with SIGINT ignored
      and the ignore survives exec — so the clean-shutdown path is
      pinned by the driver, whose Popen child gets the default
      disposition like a real terminal's Ctrl-C). Four mutations
      killed at exactly their predicted checks (the probe deleted, the
      attach branch deleted, f109 reverted, and the review round's
      SIGTERM-cleanup regression) and reverted sha-verified; the
      default mode twice-run byte-identical; no Rust moved (every
      cargo gate skipped by construction, proven). Dual-reviewed to
      sign-off over two rounds: round 1 both changes_requested on one
      shared med — the driver's stale-publication pin was provably
      vacuous, its check reading the port file after the driver's own
      fallback had recreated it (the primary injected a SIGTERM-cleanup
      regression and the driver stayed green) — fixed by capturing the
      state before the restore, and re-proven non-vacuous by re-running
      that exact regression as the fourth mutation; the round's lows
      all landed (unknown paths 404 before the render so a browser's
      favicon probe never pays one, the /health check renamed to what
      it pins, `publish_port` gaining `write_registry`'s fsync so its
      docstring's claim is literal).
- 2026-10-08 — **M5 t56: the adoption README**: the repo's front door
      — what the server is (markdown boards as audit trails vs query
      surfaces; typed nodes and edges; determinism as the product
      property), the §13 wiring reproducing t52's live global-config
      entry **verbatim** (machine-verified: both parsed and compared —
      type, command array, enabled, identical), the store location
      (§2's D4 root, `todo-cli` as the no-client inspector), the
      tool-surface pointer (§6's table is authoritative; the README
      groups, never restates), the three-repo adoption story (§9's
      cutover, no live two-way sync), the importer dialects (D12: live
      boards never copied in), the progress page (t55's `--serve`
      included), and a verifying-the-wiring section whose commands were
      **run on this machine as the task's verification** — the honest
      test caught one defect before any reviewer did: the binary has no
      `--help` flag (its usage prints on argument errors), so the
      section's first draft instructed a failing command; the section
      now uses only commands that exit 0, with the `progress.py` probe
      as the end-to-end binary check. Both hygiene tools grew
      root-README support (code, dual-reviewed): `check-links`'
      DEFAULT_FILES gains `README.md` — the standing gate checks the
      README's links, mutation-proven load-bearing (a planted broken
      link fails with the extension, passes without it) — and
      `agents-index`'s `discover()` yields the root README.md (the
      "new doc indexed" clause of the task's own verify line,
      mutation-proven the same way: the row disappears without it).
      The README's own refresh instruction exercised live (the board
      moved at t55 after the t54 import; `--refresh` re-imported, the
      page now renders the current board). No Rust moved.
      Dual-reviewed to sign-off over two rounds: round 1 the primary ok
      with lows, the secondary changes_requested on one med — both
      found the same defect, the verifying section promising
      `repo_list` would answer "the three repos" where the registry
      holds four (this repo itself, tracked since t54; the README's own
      adoption section already said "plus this repo itself") — fixed to
      "the tracked repos (dacode, camera, ikvm-html, and this one)";
      the round's lows all landed (the fixture-path code span reflowed
      onto one line — the f12 class, a newline inside a code span
      renders as a space; the generated index's title widened from
      "# AGENTS.md index" to "# Documentation index" now that the table
      carries the README row). The binding round's two lows landed at
      closure as review-exempt doc fixes, the code files byte-identical
      to the bound shas: AGENTS.md's Project State moved off "t56 is
      next" (the board had moved, the state paragraph had not), and the
      verifying section gained the one live failure mode a reader will
      actually hit — a live opencode session's server holds the stores'
      locks for its process lifetime (§8's single-writer discipline;
      measured on this machine: this session's own server holds all
      four), so       `verify` and the probe answer `store is locked` until
      that session ends.
- 2026-10-08 — **M5 t57: U21's duplicate-key check** — the last M5
      task before the gate. `tools/agents-sync.py`'s config load is now
      a `StrictLoader` (a `yaml.SafeLoader` subclass overriding
      `construct_mapping`): after `flatten_mapping` it constructs each
      key and raises `ConstructorError` naming the key and both marks
      on a duplicate, in any mapping at any depth — PyYAML keeps the
      last of duplicate keys, so a repeated key was dead config that
      read like live config, the failure that bit once as f42 (two
      top-level `commands:` keys, a whole build-command block dead)
      and f44 generalised into U21. `load_config` parses from a stream
      so the marks carry the real path and line numbers, and catches
      `yaml.YAMLError` into the tool's clean `sys.exit` style — which
      is required for the refusal itself to exit cleanly
      (`ConstructorError` is a `YAMLError`), closing the old
      traceback-on-syntax-error path with it. The check's keying is
      PyYAML's own collapse semantics (the same hash/eq contract as the
      dict it guards), unhashable keys delegate to `super()`'s named
      error, and the merge-key caveat is documented in the class
      docstring (the config has zero `<<` merges and zero anchors).
      Verified: `--check` green on the real config with the adapters
      byte-identical (the loader moves no output byte); the negative
      proofs ran as splice-into-the-config + sha-verified revert
      through the real CLI — the full main → load_config → StrictLoader
      gate path, standing in for the verify line's "feeding it a copy"
      because the tool reads one fixed path: a duplicated top-level
      `synced_sources: []` exits 1 naming the key with both line
      numbers, and an appended block with `inner:` twice exits 1 naming
      'inner' — the anywhere claim. Dual review ok in one round, both
      reviewers independently re-running the battery, the primary
      falsifying every attack question against PyYAML's installed
      source (block scalars, deep nesting, lists of mappings, anchors
      and merge-without-collision all parse clean; `true`/`1`-class
      collisions flagged exactly where plain `safe_load` provably
      collapses them; the unhashable delegation confirmed in
      `BaseConstructor`'s source) and the secondary ratifying the scope
      calls: `tasks.py`'s two lenient loads of the same config stay
      (the `budget` fallback's convenience read of validated config;
      the gated validator is agents-sync), and no committed driver (a
      parse-time refusal records its planted-failure proofs in
      evidence, the t56 check-links precedent — drivers are for
      stateful protocols). U21 struck. Landed as `3e5c4c2`.
- 2026-10-08 — **instructions: comment and commit brevity** (user
      decision): the default is no comment — self-documenting code is
      the norm, and only a genuinely tricky spot earns a short one (one
      or two lines, never a paragraph); commit bodies stay brief — the
      problem and the approach in a few lines, the diff already saying
      what changed. Landed in `_rust.md`'s comment section, the config
      summary, `implementer.md`, and AGENTS.md's committing section;
      `version.custom` 12→13, adapters regenerated.
- 2026-10-08 — **M5 g13 rider: the doc lows M5 touches passed over
      (f105, f101)** — the f41 sweep found three doc-level lows whose
      files M5 touched without the fixes riding: ikvm's decisions doc
      carried a live-board census (the no-measurement class), lib.rs's
      importer paragraph was ragged and named three dialects where the
      crate hosts four, and the missing-golden panic hint named a
      filter that regenerates neither `import_md.v1` nor the write
      goldens. All three helper copies (cli/store/mcp) now carry the
      same target-agnostic hint and a true provenance paragraph.
      Dual-reviewed over three rounds — both blockers were the same
      drift class the rider exists to close, found independently each
      time (a provenance clause the fix itself staled, then the
      unpropagated third copy). f94 deliberately untouched: its three
      sites are schemars-carried schema descriptions, wire-visible,
      the user's call. Landed as `b60c4d1`.
- 2026-10-08 — **M5 exit gate (g13): M5 closed, g3 closed — the
      roadmap delivered.** Every line of the verify method ran on the
      final tree: fmt clean; check --workspace --all-targets clean;
      test --workspace 584 passed 0 failed 1 ignored (domain 183,
      store 120, cli 35, mcp 246); clippy clean on a forced recompile;
      cargo doc warning-free under both invocations; UPDATE_GOLDEN=1
      over the store/cli/mcp suites with git diff --exit-code over all
      three golden dirs = 0 (all 24 goldens byte-identical); the
      committed drivers green (progress-refresh-check,
      progress-serve-check incl. f109's stdout-purity pins); the live
      refresh re-importing the moved board; the t52/t58 smoke
      transcripts on disk; attention --check exit 0; check-links
      17/11; agents-sync --check green. The f41 sweep: f105/f101
      closed by the dual-reviewed rider (`b60c4d1`), f106 closed (the
      word landed with t53's own edits — every site reads
      ancestor-or-equal), f107/f108 closed on their green pins, f94
      left open as wire-visible (the user's call), f67/f69 and the
      store/cli/domain findings verified open with consumers intact
      (M5 touched none of their files), the board ledger unchanged
      (U27 the user's call; U13/U24/U25/U29/U30 acknowledged debt).
       Pitfalls consolidated: 5.14 (a single-source config needs a
       duplicate-key refusal at its validator — f42/f44/t57) and 5.15
       (a tool whose stdout is a machine format keeps diagnostics off
       stdout — f109); the live file was empty at the gate, reset at
       g11. g12, g13 and g3 closed in the tracker with evidence.
       Landed as `9fdcf35`.
- 2026-10-08 — **the recorded ledger called and U30 fixed**: the
       user's calls recorded (U27 signed (c) — the bounded
       `WouldBlock` retry lands inside `FileLock::acquire` as a
       dedicated `todo-store` task, the cross-repo guard answered
       from the code: per-repo per-file store locks, the registry's
       the one shared lock; U13/U24/U25/U29 acked, no splits
       scheduled — landed as `23c47e8`), then U30's seen-maps: every
       dacode register that lacked one now refuses a duplicate id
       with `DuplicateId` naming both lines — the entry's named two
       files (B summary, B dossiers, A index, A applied, A dossiers)
       plus `board.rs`'s three on the class sweep (§2's decisions
       table, §5's milestones, the sub-step tables with the seen set
       hoisted across tables), both reviewers ratifying the
       beyond-the-entry edit (the f71 precedent). Nine new tests,
       each observed failing on HEAD before the fix (the duplicates
       parsed clean — U30's exact defect); the e2e fixture and all 24
       goldens byte-identical; 593 workspace tests; every gate green.
       The sibling dialects' decisions registers carry the same gap —
       opened as U33 (the primary reviewer's measurement, f103's
       camera/ikvm half made board-durable). Landed as
       `<hash-pending>`.



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
- [x] **U16 · Ship base instructions with the server, and stop the harness
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
      **Resolved 2026-10-04, both halves.** (a) landed at t33 (the
      instructions text in `lib.rs`, measured honourable in both
      handshakes per §6.3). (b) landed at t39, **narrowed to the
      orchestrator role** — the one role the server is enabled for —
      rather than "every role": a role that gets no server and loses
      its todo tool would have no todo surface at all, and the
      implementer/reviewer/explore roles keep theirs (no server, no
      competing surface, so U16's mixing failure mode cannot arise
      for them). The `todoread` half of "the pair" does not exist as
      a tool in opencode 1.18.34 (measured against the binary), so
      `todowrite: deny` is the pair's whole gateable surface. The
      limit recorded as asked: `.mcp.json` (the Claude-Code-compatible
      adapter) hands the server to any client session ungated — it
      has no per-role mechanism — so for a client we do not
      configure, two todo systems remain visible and the instruction
      text is the only discriminator, exactly as the entry predicted.
      The smoke (§3's t39 entry) is the proof the wired
      configuration works end to end. §9's no-dogfooding rule still
      governs: the board + `tasks.py` remain this repo's trackers of
      record.
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
- [x] **U18 · A live view for progress, and whether the server is shared per
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
      **Confirmed 2026-10-07 (shape A) at the M5 decomposition**: t55 lands
      the local page server — `progress.py --serve` over the thin-client
      render, `127.0.0.1:0` with the port published to a per-user runtime
      file, the default staying a static file with no listener. Shape B is
      declined: it is D13's reserved question, the spec's precondition for
      drafting D13 has now passed but nothing in M5 needs a transport past
      signed D5, and a shared instance is a §8 single-writer commitment this
      project has never tested — if wanted, it is a new D-entry drafted
      deliberately, not a rider in the polish milestone.
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
- [x] **U21 · Land the duplicate-key check for `agent-rules.yaml`, or close
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
      **Scheduled 2026-10-07: t57 lands the check** — the M5 decomposition
      dispositioned it into a task rather than closing it wontfix, because
      the failure is silent in the direction that matters (an edit that
      appears to land and does not) and it has bitten once already. The
       disposition is made; the item stays open until t57's commit lands
       the check (the U15/U19 precedent — the board item rides its task).
       **Resolved 2026-10-08 (t57)**: the loader refuses a duplicate key
       anywhere in the file — `tools/agents-sync.py` loads the config
       with a `StrictLoader` whose `construct_mapping` raises
       `ConstructorError` naming the key and both marks on a duplicate
       in any mapping at any depth; `--check` stays green on the real
       config, and the negative proofs (a spliced duplicate top-level
       key and a nested one, each exiting non-zero naming the key,
       reverted sha-verified) are recorded in §3's t57 entry.
- [x] **U26 · The protocol suite's post-exit store open flakes
      (`store is locked`).** f68's one-off flake recurred three times
      in ~83 executions of the protocol target during t37's review —
      `repo_register_creates_a_real_store_and_returns_the_new_state`
      failing on "the store opens after the server exits: store is
      locked: repo camera", green on every immediate re-run, under
      both `--test workspace` and `--test protocol` alone. The panic
      names the `StoreError` as f68 designed, so a recurrence is
      capturable; the prime suspects are the drive's
      `wait_with_output` reaping the child while its flock
      outlives the reap, and the in-process `repo_list()` one line
      earlier opening and dropping the same store's flock (t31's
      self-contention shape — flock is per open-file-description).
      Pre-existing (t33's machinery, untouched by t37) and outside
      every intervening task's owned set, which is why it survived
      four landings unreproduced. **t38's hammer closed it as
      unreproduced, and it stays open on those numbers**: the
      committed hammer (`the_u26_sequence_hammers_clean_or_reproduces`
      in `tests/wire.rs`, 150 rounds of the flaky test's exact
      sequence per run) plus a campaign of 20 wire-target runs
      (3000 rounds), 172 protocol-target executions (100
      sequential, 72 under four-way parallel load) and four
      workspace runs all ran clean on this host (macOS, Apple
      Silicon — build-and-test.md's standing machine), against the
      flake's recorded ~3/83 — no reproduction, so no isolation
      variant was run and
      no fix landed (the honest disposition; a fix without a
      reproduction would be unprovable). The hammer stays in the
      suite as the standing detector — a recurrence is captured by
      the round-indexed panic naming the `StoreError`, and the two
      named isolation variants (without the `repo_list()` call;
      with a bounded sleep before the open) are documented on the
      hammer's test for whoever next sees it. The flake remains
      t39's smoke session's watch item. **Recurred 2026-10-04, during
      t40's review**: one failure in six `cargo test -p todo-mcp`
      runs — the same test, the same "store is locked: repo camera"
      panic, green on every immediate re-run, exactly the recorded
      signature. First documented recurrence since t38's
      unreproduced disposition, which was conditioned on
      reproduction — so the condition is now met and the two
      isolation variants (without the `repo_list()` call; with a
      bounded sleep before the open) are the next owner's first
      move, at g9 or a dedicated task. The t39 smoke itself passed
      clean; the recurrence was a review-side test run. **Closed at
      g9 (2026-10-04): reproduced, isolated, fixed.** The gate ran
      the isolation the board prescribed, in-tree with sha-verified
      reverts: the baseline reproduced at 1-in-30 and 1-in-10
      full-crate runs (the exact signature, captured by the designed
      panic); an lsof+ps diagnostic at the failure found **no fd
      holder anywhere** and the server dead (reaped by the drive's
      `wait()`); a retry-loop diagnostic measured the lock **still
      held 1.33ms past the refusal** — a transient; and the variant
      with every in-process open removed (no `repo_list`, only the
      final open) ran **40/40 clean** where the full sequence
      flaked. The cause: the in-process `repo_list()` opens and
      closes the store's lock, the kernel's flock teardown outlives
      the close by a millisecond-scale lifetime (the release is
      deferred kernel-side — inferred from the measurements; the
      store's code is innocent, the handle drops synchronously), and
      the immediate re-open races the deferred release. The fix is
      tests-only, in the flaking sequence itself (this entry's own
      rule): a bounded retry on `locked` at the final open — the
      initial open plus five retries 2ms apart, a ~10ms window
      against the measured ~1.3ms lifetime — in both sites with the
      shape (the protocol test and the wire target's hammer,
      mirrored; protocol.rs is self-contained by the t38 ruling, so
      the retry is duplicated rather than lifted into `common/`).
      With the fix: **40/40 full-crate runs clean**. Dual-reviewed
      at the gate (both reviewers, one round plus the doc fixes).
      The store-side question the fix deliberately does NOT answer
      is **U27**: whether `FileLock::acquire` itself should retry
      briefly on `WouldBlock`, making every caller robust to the
      deferred release — deferred because it changes `locked`'s
      timing semantics for all callers on one-platform,
      one-workload evidence.
- [x] **U27 · Should `FileLock::acquire` retry briefly on
      `WouldBlock`?** Opened at g9 out of U26's closure: the
      kernel's flock teardown can outlive the fd close by a
      millisecond-scale lifetime (U26's measurements), so a caller
      that opens a store immediately after another process closed it
      can see a spurious `locked` — the store's own users hit this
      (the protocol suite did), not just tests. A bounded retry
      inside `acquire` would make every caller robust, but it
      changes `locked`'s timing contract: a genuinely contended
      store would report `locked` only after the retry window, and
      the error's meaning shifts from "a writer is active" to "a
      writer stayed active past the window". The decision needs
      multi-platform evidence (this is one macOS workload) and must
      reconcile `lock.rs`'s module doc, which states the release is
      the close. The store's next owner's, or a dedicated task's.
      **Left at the M5 decomposition (2026-10-07)**: no M5 task owns
      `todo-store`, and the entry's own bar — multi-platform evidence, the
      `locked` timing-contract reconciliation — makes it a decision, not
      polish. It stays the next store owner's or a dedicated task's.
      **Signed (c) 2026-10-08 (the user's call)**: the bounded retry
      lands inside `FileLock::acquire` as a dedicated `todo-store`
      task. The cross-repo guard the sign rode on, answered from the
      code: store locks are per-repo per-file
      (`repos/<slug>/event-log.jsonl.lock`), so parallel consumers
      of different repos never contend on a store lock; the one
      shared lock is the registry's, held for its short critical
      section, and the retry makes even that robust. The task owns
      the rest of this entry's bar: the window stays bounded, a
      genuinely held lock still reports `locked` after it, and
      `lock.rs`'s module doc (the release is the close) is
      reconciled in the same change.
- [x] **U28 · `progress.py` goes dynamic — the server as its data
      source, after M4.** Opened 2026-10-05 on the user's question:
      the page's hardcoded tables (`LANES`, `GATING`, `EXTRA`,
      `WAITS`) are a hand-maintained shadow of the server's own
      model — lanes ↔ epics, gating rows ↔ `blocks` edges, waits ↔
      `blocked`/`blocked_by`, extra entities ↔ nodes, the open-items
      table ↔ `status_summary` (D16). **No new §6 tool is needed**
      (`graph_show`/`status_summary`/`task_list` already return the
      shape, and §11 keeps rendering a client's job); what is
      missing is the **dogfooding cutover** — this repo's own
      tracking living in the server, which §9 has sanctioned since
      M3 closed and nobody has scheduled. After it, `progress.py`
      becomes a thin client and the four tables retire. Deferred by
      the user until after M4 proves the MVP (the importer + the
      three repos tracked for real, t48). Depends on **U18**'s
      territory either way: a per-render stdio spawn works under
      signed D5; a shared long-running instance is **D13**'s
      reserved question. One gap to carry into the design: the
      `GATING` rows' citation column (the *why* of an edge) has no
      server equivalent — the blocked task's `note` is the nearest
      home.
      **Confirmed 2026-10-07 (dogfood) at the M5 decomposition**: t54
      lands the cutover — this repo registered in the durable root and
      imported through the measured layout fit, `progress.py` a thin
      client over a per-render stdio spawn (D5-signed), the four tables
      retiring. The markdown board + `tasks.py` stay the trackers of
      record: the write surface and the exit gate are unchanged, so the
      cutover is reversible. The entry's named design gap — the GATING
      citation column — and the refresh cadence (a refresh is a
      re-register, `apply_plan` refusing a non-empty graph) are t54's
      task-spec questions.
- [x] **U22 · `progress.py` does not render open decisions.** f45,
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
      **Scheduled 2026-10-07: resolves with t54** — the cutover's thin
      client renders open decisions from the server's `decision_list`
      **Resolved 2026-10-07 (t54)**: the thin client renders every
      decision from the one `graph_show` answer — the page's decisions
      section shows all seventeen with their statuses (all signed
      today; an open decision renders as open the moment one exists),
      and the hand-maintained EXTRA table that drew D16 by hand is
      deleted with the other three.
      (this repo's decisions.md D-entries import as decision nodes), so
      the page gains the rows the finding asked for; if the import's
      decision statuses prove too coarse for the page, the footer
      scope-limit is the fallback, settled in t54's spec.

## 5. Debt

Deliberate shortcuts, acknowledged rather than fixed. Same id rules as §4.

- [x] **U13 · Two store files are over the 600-line target.** Recorded as
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
      **Re-measured at the M5 decomposition (2026-10-07): unchanged —
      812 and 772** under the same awk measure; `layout.rs` 543, still
      under target. M3's wire and M4's importer never grew either file.
      **Acknowledged 2026-10-08 (the user's ack)**: no split scheduled —
      the entry stays as the recorded ledger, the split riding the
      trigger the entry names.
- [x] **U23 · The copied-store cross-check on the server's resolve path is
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
      **Closed 2026-10-04 by t37**: `write/edges.rs`'s
      `a_swapped_store_is_refused_not_adopted` drives `repo_show` on
      `bbb` after the out-of-band log copy and pins the
      `corrupt_store` carrier naming both slugs — the branch is
      pinned, and the detail-string re-spelling rides the golden's
      bytes.
- [x] **U24 · `todo-mcp/src/listing.rs` is over the 600-line target.**
      689 production lines (the awk measure; the `#[cfg(test)]` clause
      does not apply — the file holds no test module; 685 at t35, 683 at
      t37, and 689 at the M5 decomposition re-measure 2026-10-07 — t40's
      D17 echo added the priority field to the rows). Over
      target
      because the listing cluster's dispatch pins all five tools,
      their params and their answer DTOs — `graph_show`'s dump
      builders included — in one file, the `query.rs` shape the t35
      spec named; the cap of 1000 holds. The split, if the cluster
      ever grows a sixth tool or the dump grows a second scope shape,
      is the dump half (`GraphShowDto`, `RepoDumpDto`, `EpicGroupDto`,
      `TaskNodeDto`, `DecisionNodeDto` and their builders) into its
      own module — a named diff for whoever next owns the file.
      **Acknowledged 2026-10-08 (the user's ack)**: no split scheduled —
      the entry stays as the recorded ledger, the split riding the
      trigger the entry names.
- [x] **U25 · `todo-mcp/src/nodes.rs` is over the 600-line target.**
      769 production lines (the awk measure; the file holds no test
      module). Over target because the mutation cluster's dispatch
      pins all nine tools, their params and their draft/patch
      conversions in one file, the `query.rs`/`listing.rs` shape the
      t36 spec named; the cap of 1000 holds. The split, if the
      cluster ever grows a tenth tool, is the doc family
      (`doc_register`/`doc_set_path`/`doc_update` and their
      conversions) into its own module — a named diff for whoever next
      owns the file.
      **Acknowledged 2026-10-08 (the user's ack)**: no split scheduled —
      the entry stays as the recorded ledger, the split riding the
      trigger the entry names.
- [x] **U29 · `todo-mcp/src/import/dacode/map.rs` is over the
      600-line target.** 733 production lines (the awk measure; the
      file holds no test module; 758 before t50's lift moved the shared
      mapping primitives to `src/import/helpers.rs` — 382, under target
      — re-measured at the M5 decomposition 2026-10-07). Over target because the mapping's
      dispatch pins the whole node half of D18's conventions — the
      status precedence, the drafts' fields, the doc register, the
      synthetic epics — in one file beside the plan's construction;
      the edge/finding resolution already split into `resolve.rs`
      (300) at the spec's named seam when the unwaivable 1000 cap
      forced it. The cap holds. The split, if the mapping grows
      another population (a camera/ikvm dialect reusing the shape),
      is the doc-register half into its own module — a named diff for
      whoever next owns the file.
      **Acknowledged 2026-10-08 (the user's ack)**: no split scheduled —
      the entry stays as the recorded ledger, the split riding the
      trigger the entry names.
- [x] **U30 · Duplicate B/A dossier ids pass `parse_board`.** The
      cross-file set checks collapse duplicates (a set cannot see
      them) and the B/A parsers keep no seen-map, unlike the U-id and
      doc-key registers — so a board holding one B id twice parses
      clean and `map_board` lands a duplicated decision with the
      index pointing at the last copy. Found by t43's primary
      reviewer (probed empirically); unreachable on the live board
      (the registers are unique) and not fixable in t43 (the parser
      files are frozen baselines). The fix is a seen-map in
      `decisions.rs`/`deltas.rs` plus a duplicate-id named error —
      whoever next owns those files; t45's fixture import is the
      natural consumer (a corrupted fixture would otherwise import
      silently wrong).
      **Fixed 2026-10-08 (the user's ack + sign)**: seen-maps landed in
      every dacode register that lacked one — the entry's named two
      files (the B summary, the B dossiers, the A index, the A applied
      table, the A dossiers) plus, on the f56 sweep-the-class rule and
      both reviewers' ratification (the f71 precedent for the edit
      beyond the entry's named files), `board.rs`'s three (§2's
      decisions table, §5's milestones, the sub-step tables with the
      seen set hoisted across tables — one `M<n>.<n>` namespace). A
      duplicate id within any register is now `DuplicateId` naming
      both lines; cross-register repetition stays legal (the B/A
      registers pair by design). Non-vacuity: the nine new tests were
      observed failing on HEAD before the fix (each returned `Ok`
      with the duplicated rows in the AST — this entry's exact
      defect); the e2e fixture and all 24 goldens byte-identical (no
      behavior change on clean boards). The class is swept for the
      dacode dialect only — the sibling dialects' decisions registers
      carry the same gap, opened as **U33**.
- [x] **U31 · The dialect modules duplicate dacode's helpers —
      ikvm-html would make the third copy.** t46's camera module was
      spec-sanctioned to share no code with dacode except the plan
      types, and the measurement held: `camera/text.rs`'s `cells` is
      byte-identical to `dacode/table.rs`'s, `collect_rows` differs
      only in visibility, and `camera/map.rs` re-implements
      `position`/`references`/`bad_reference`/`midnight`/
      `id_sort_key` from `dacode/map.rs` (only the error type
      differs). The deeper half: the shared plan types
      (`ImportPlan`/`PlanOp`/`ImportOptions`/`apply_plan`/
      `ApplyError`) — a dialect-neutral contract — are reached
      through `crate::import::dacode`, one dialect's module, and
      were frozen out of moving this task. **t47's first spec
      decision**: lift the genuinely shared helpers and the plan
      types into a dialect-neutral home under `src/import/`, or
      accept the copies with the three-copy count in hand — decided
      deliberately at t47's dispatch, not by default. Found by t46's
      secondary reviewer; recorded here so the decision outlives the
      branch.
      **Resolved 2026-10-07 by t50 (the lift)**: the plan types
      moved to `src/import/plan.rs` and the shared helpers to
      `src/import/helpers.rs` (the `HelperError` seam carrying the
      two error-typed ones), both dialects consuming, zero behavior
      change — the decision was taken at t47's dispatch exactly as
      the item asked, with the three-copy count in hand, and the
      declined option (accept the copies) is recorded in t50's spec.
      The residual is **f104**: `ApplyError::Refused` still carries
      dacode's `ImportError` (a type-level coupling with
      dialect-neutral messages), named in `plan.rs`'s module doc as
      the debt for its next owner.
- [x] **U32 · A strict-mode client could reject the server's `uint64`
      schema format.** Promoted from f99 at the g11 sweep (its own
      verify line: "a U-id if it outlives M3" — it has). The first
      real client's compatibility note: opencode's schema validator
      emits 'unknown format uint64 ignored in schema' warnings
      against the server's schemars-1 `u64` fields
      (`NextItemDto.weight`, the D16 count DTOs). Harmless today
      (the validator ignores what it does not know), but a
      strict-mode client could hard-error on the same schemas. The
      fix, if ever needed, is the server's: a schema-level integer
      spelling the validator knows — a wire-visible change and
      therefore a decision, not a patch.
      **Left at the M5 decomposition (2026-10-07)**: no strict-mode
      client exists, so nothing in M5 takes it; the fix stays a
      wire-visible decision if a real client ever rejects the format.
      **Resolved 2026-10-07** — the user's in-thread sign, mid-t53
      preparation, on the noise itself (every client session's
      console): strip the integer-family format hints. Landed as t58
      — the strip in `src/schema.rs` at both seams (every
      `output_schema` site plus `next_items`'s `limit`), the wire pin
      closing the class, the live smoke showing zero `unknown format`
      lines. No §12 item opened: the signed §6 surface, D14's key
      order and every golden are untouched.
- [ ] **U33 · Duplicate D ids pass the camera/ikvm/own decisions
      parsers.** Opened at U30's closure, on the primary reviewer's
      measurement (scratch-worktree probes): a duplicate D id parses
      clean in camera's summary table AND dossiers, ikvm's summary
      AND dossiers, and own's dossiers — each dialect's cross-check
      is set equality (camera/ikvm/own `mod.rs`), which collapses
      duplicates, U30's exact mechanism in the sibling dialects.
      f103 recorded the camera/ikvm half (unreachable on both
      measured boards); the own-dossiers half is new. The question is
      f103's, now board-durable: the next owner of either dialect's
      decisions parser hoists the D check the same way, or records
      why those table registers stay duplicate-tolerant. Not fixed
      with U30: different dialects' files, and the user's sign named
      U30's entry.

## 6. Session log

Newest first. One entry per session, written for someone who was not
here. The final entry of a session is the handover
(`.agents/skills/todo-next-imp/SKILL.md` defines the shape).

### 2026-10-08 — g13: the M5 exit gate — the roadmap delivered

The session's third delivery (the user's "let's start next - g13",
after the comment/commit-brevity instructions landed as `67295aa`).
Every line of the gate's verify method ran on the final tree: the
five code gates (fmt, check, 584 tests / 0 failed / 1 ignored, clippy
on a forced recompile, doc under both invocations), the golden ritual
(UPDATE_GOLDEN=1 over the store/cli/mcp suites, git diff --exit-code
over all three golden dirs = 0 — all 24 goldens byte-identical), the
committed drivers (progress-refresh-check, progress-serve-check with
f109's stdout-purity pins), the live refresh, the smoke transcripts
on disk, and every doc gate.

The f41 sweep found three doc lows whose files M5 touched without the
fixes riding — the rider (`b60c4d1`, dual-reviewed over three rounds;
both blockers were the same drift class the rider exists to close,
found independently each time: a provenance clause the fix itself
staled, then the unpropagated third helper copy). f106 closed (the
word landed with t53's own edits), f107/f108 closed on their green
pins, f94 left open as wire-visible (the user's call), the rest
verified open with consumers intact — M5 touched none of the
store/cli/domain files their remedies wait on. Pitfalls 5.14/5.15
consolidated; the live file was empty (reset at g11, no struggles
recorded in M5).

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; the gate
commits leave the tree clean (the index backfill follows, AGENTS.md
having moved). Landed this session: t57, the brevity instructions,
the g13 rider, and this gate — **g3 is closed: the M0–M5 roadmap is
delivered.** Next step: none is owed — the follow-ups are recorded
where they live (U27 the user's call; U13/U24/U25/U29/U30 the
acknowledged debt ledger; the tracker's doc-level items — f94
wire-visible, f67/f69, the store/cli findings — each with its named
consumer). D13 stays reserved (the shared per-user server instance,
declined at the M5 decomposition). Blocked on you: nothing. The two
standing caveats: the release binary goes stale after pulls until
rebuilt; the dogfooding store is a derived cache — after any
board-only edit, run `tools/progress.py --refresh`.

### 2026-10-08 — t57: U21's duplicate-key check

The session's delivery (the user's "let's continue from the last
session", taking the handover's offered step). Orchestrator-authored
per the t30/t58 precedent — one file, a decided verify method — then
dual-reviewed to sign-off in one round: both reviewers ok, both
independently re-running the verification battery, the primary
falsifying every attack question against PyYAML's installed source
(legal shapes — block scalars, deep nesting, lists of mappings,
anchors, merge-without-collision — all parse clean; collision keying
matches PyYAML's own collapse semantics; the unhashable-key delegation
confirmed in `BaseConstructor`; the double `construct_object`
cache-safe) and the secondary ratifying all four scope calls, including
the reframing that the `YAMLError` catch is required rather than a
rider — `ConstructorError` is a `YAMLError`, so the catch is what
makes the refusal itself exit cleanly.

The guard: `StrictLoader(yaml.SafeLoader)` overriding
`construct_mapping` — after `flatten_mapping`, construct each key and
raise `ConstructorError` naming the key and both marks on a duplicate,
at any depth; unhashable keys delegate to `super()`'s named error;
`load_config` parses from a stream (marks carry the real path and line
numbers) and exits cleanly on `YAMLError`. The negative proofs ran as
splice-into-the-config + sha-verified revert through the real CLI —
the full gate path, standing in for the verify line's "feeding it a
copy" because the tool reads one fixed path (the primary's one low:
word the closure evidence to the mechanism that actually ran — done
here and in the tracker). Scope calls: `tasks.py`'s two lenient loads
of the same config stay lenient (the `budget` fallback's convenience
read; the gated validator is agents-sync), and no committed driver (a
parse-time refusal, the t56 check-links precedent).

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; the commit
leaves the tree clean (the index backfill follows, AGENTS.md having
moved). Landed this session: t57 (this entry) — M5's remaining tree is
**g13 alone, the exit gate, whose closure also closes g3**. Next step:
**g13** — every code gate (fmt, check, test, clippy, `cargo doc` under
both invocations), the golden ritual over every golden dir
(`UPDATE_GOLDEN=1` byte-identical), the f41 sweep of the whole open
tracker list against `git log`, the packaging/cutover/live-view
evidence read (t52's smoke transcript, t54's cutover + refresh driver,
t55's serve driver), board §2's M5 row done, g3 closed with evidence —
the `/todo-close` shape over the milestone. Its alternative —
declaring M5 done without running the gate — is not one; the gate is
the deliverable. Blocked on you: nothing. Unverified: the t52
staleness caveat stands (the release binary goes stale after pulls
until rebuilt); the dogfooding store is a derived cache — after any
board-only edit, run `tools/progress.py --refresh` or the page renders
the stale import.

### 2026-10-08 — t56: the adoption README

The session's second delivery (the user's "let's start t56", continuing
the same session). Orchestrator-authored per the routing table (docs
are exempt from code review) — except the two hygiene tools the task's
own verify line forced into scope: "check-links green" and
"agents-index --check green with the new doc indexed" are only honest
if the tools actually see a root README, and as built neither did
(check-links' default scope was `docs/`, `.agents/knowledge/` and the
root `AGENTS.md`; agents-index discovers `AGENTS.md` files only). Both
grew minimal root-README support and were dual-reviewed as code; the
README itself rode the same commit review-exempt.

The honest test — following the README's own verifying section on this
machine — caught a defect before any reviewer could: the binary has no
`--help` flag (its usage prints on argument errors), so the section's
first draft instructed a command that exits non-zero. The section now
uses only commands that exit 0, with the `progress.py` probe as the
end-to-end binary check. The wiring match is machine-verified: the
README's mcp entry parses identically to the live global-config entry
(type, command array, enabled). The README's own refresh instruction
was exercised live — the board had moved at t55 after the t54 import,
so `--refresh` re-imported and the page now renders the current board.

The review round: the primary ok with lows, the secondary
changes_requested on one med — both found the same defect
independently, the verifying section promising `repo_list` would
answer "the three repos" where the registry holds four (this repo
itself, tracked since t54 — the README's own adoption section already
said "plus this repo itself", so the honest test contradicted the text
two sections up). Fixed; the round's lows all landed (the fixture-path
code span reflowed onto one line — the f12 class again, a newline
inside a code span renders as a space; the generated index's title
widened to "# Documentation index" now that the table carries the
README row). The primary also proved by a scratch commit that the
post-commit index backfill is mandatory, not optional — the committed
index goes stale on its own commit because both AGENTS.md's rev and
the README's untracked marker flip — which is exactly the flow the
handover plans. The binding round left two lows that landed at closure
as review-exempt doc fixes, the code files byte-identical to the bound
shas: the Project State paragraph (still saying "t56 is next" — the
board had moved and the state paragraph had not) and the verifying
section's one expected live failure mode, measured rather than guessed
(the primary caught this session's own server holding all four store
locks, so `verify` answers `store is locked` while a session that has
queried is running — the discipline working, worth one sentence so a
reader does not misread it as broken wiring).

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; this commit
leaves the tree clean (the index backfill follows, AGENTS.md and
README.md having moved). Landed this session: t55, t56 (this entry) —
M5's remaining tree is t57 (U21's duplicate-key check in
tools/agents-sync.py, independent and small) and g13. Next step: **t57
— the duplicate-key check**: the loader refuses a duplicate key
anywhere in agent-rules.yaml, proved by feeding it a copy with one
spliced in (the negative proof recorded), the real config verified
clean — f44's generalisation, the failure that bit once as f42 (a
silently shadowed config block). Its alternative — going straight to
g13 — is wrong because t57 is a five-line guard with a decided verify
method and g13 is the exit gate that should see it landed. Blocked on
you: nothing. Unverified: the t52 staleness caveat still stands (the
release binary goes stale after pulls until rebuilt); the dogfooding
store is a derived cache — after any board-only edit, run
`tools/progress.py --refresh` or the page renders the stale import.

### 2026-10-08 — t55: U18's shape A — progress.py --serve

The session's first delivery (the user's "yes, let's start t55",
continuing the deep 2026-10-07 session per that handover's note).
Orchestrator-authored per the t30/t58 precedent — one Python file plus
one committed driver, under the delegation threshold — with the task
spec at tmp/t55-spec.md (eight decisions, four judgment calls, the
verifications and three mutations named up front).

The driver earned its keep before review did: its "the serving line
names the published URL" check failed on its first run and the failure
was a real tool defect — the URL line sat in a block-buffered pipe,
never flushed before serve_forever blocked, so a piped consumer would
never have seen it. Fixed with `flush=True` rather than by weakening
the check. The live smoke's step 7 (SIGINT) could not run from a shell
script at all: non-interactive shells start background jobs with SIGINT
ignored and the ignored disposition survives exec, so `kill -INT` is a
no-op on the serve process. The clean-shutdown path is pinned by the
committed driver instead — a Popen child gets the default disposition,
exactly what a real terminal's Ctrl-C delivers — and the mechanism is
recorded in the smoke transcript and the driver's docstring.

The review round earned its keep the way the cross-family pairing is
designed to: both reviewers found the same med independently — the
driver's "SIGTERM leaves the stale publication" pin was provably
vacuous, its check reading the port file after the driver's own
fallback had recreated it, and the primary proved it by injecting a
SIGTERM handler that runs the cleanup (the exact regression the pin
exists to catch) and watching the driver stay green. Fixed by
capturing the state before the restore; the fixed pin was then proven
non-vacuous by re-running that exact regression as a fourth mutation
(it fails the pin, driver exit 1). The round's lows all landed: unknown
paths now 404 before the render (a browser's favicon probe was paying
a full render, serialized on the single thread), the /health check was
renamed to what it actually pins, and `publish_port` gained
`write_registry`'s fsync so its docstring's "the store's own write
discipline" is literal. The primary's state-machine enumeration
confirmed every reachable orphan lands in the spec's documented benign
class (including the probe-timeout-during-render takeover it
reproduced), the cleanup guard never removes a successor's file, and
the SystemExit catch is load-bearing (a 500 with the listener
surviving).

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; this commit
leaves the tree clean (the index backfill follows, AGENTS.md having
moved). Landed this session: t55 (this entry) — M5's remaining tree is
t56 (the adoption README), t57 (U21's duplicate-key check,
interleavable) and g13. Next step: **t56 — the adoption README**: what
the server is, the §13 wiring reproducing t52's live global-config
entry, the store location, the tool-surface pointer, the three-repo
adoption story, the importer dialects — md-writer conventions,
check-links + agents-index green with the new doc indexed, and the
honest test is following the README on this machine. Its alternative —
t57 first — is defensible (t57 is independent and interleavable) but
t56 is the dispatch order's next and it documents the landed shape
including t55's served page. Blocked on you: nothing. Unverified: the
t52 staleness caveat still stands (the release binary goes stale after
pulls until rebuilt); the dogfooding store is a derived cache — after
any board-only edit, run `tools/progress.py --refresh` or the page
renders the stale import (the footer's instant makes that visible).

### 2026-10-07 — t54: the U28 cutover — this repo dogfooded

The session's third delivery (t58 and t53's entries carry the first
two; the user's "ok, let's do U32 first, then t53" then "let's
continue with t54"). The study ran first (the t41/t46/t47 precedent)
and corrected the decomposition note's premise: the layout fits
camera exactly — the shared `docs/implementation` board dir and
camera's two-file set — where the note claimed "neither dialect's
layout constant"; the grammar is camera-shaped with measured cell
differences. The spec's own study then misread the decisions table's
header (inferring `[Decision, Area, …]` from the data rows where the
header is camera's exact `[ID, Decision, Recommendation, Confidence,
Gates, Status]`) — the implementer measured it and reported the
divergence, the folding convention's honest shape.

The dispatch needed the short-prompt pattern from the start (the
session's earlier spawn deaths taught it); the implementer landed
both halves — the `own` dialect and the thin client — with the live
probe's census matching the spec's prediction exactly. The review
round earned its keep twice over: the primary's meds caught live
tokens in the synthetic corpus (the t42/t46 rule the module's own doc
cites) and the under-reporting unknown-dialect message (t46/t47 moved
message and pins when the value set grew; the freeze is scoping, not
a reason to under-report — three pins text-matched it, not two), and
the lows landed the duplicate M/D id guards, the committed
crash-order driver, and two doc-precision fixes. The primary
independently re-ran the probe, the census, and the M6 crash-order
mutation (observing the dangling-entry state break `repo_list` for
every repo before the revert); the secondary verified the frozen set
and the conventions.

The orchestrator's half, on the reviewed tree: the release binary
rebuilt, `repo_register` + `import_md` over the wire into the durable
root (the census: 8 epics / 17 decisions / 0 docs / 32 tasks / 17
blocks — the prediction, exactly), `todo-cli verify` green, and the
query session recorded (tmp/t54-{import,query}-transcript.txt):
`next_task` answers `dacode-todo-mcp:U21`, and — composed with t53's
cwd tier — `next_task` with no repo parameter, the server spawned in
this repo's checkout, answers the same. The page renders from the
real root (49 nodes, 17 edges), twice-run byte-identical, the
`--refresh` round-trip green.

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; this
commit leaves the tree clean (the index backfill follows, AGENTS.md
having moved). Landed this session: t58, t53, t54 (this entry) —
three of M5's seven tasks. Next step: **t55 — U18's shape A,
`progress.py --serve`**: bind 127.0.0.1:0, publish the assigned port
to a per-user runtime file so a second invocation attaches instead of
spawning another listener; the default stays a static file with no
listener; no port scanning (bind :0, read the assigned port back).
f109 rides it (the refresh census line to stderr for `--json`
consumers). Its alternative — t56 first — is wrong because t55 is the
smaller change and t56 (the README) documents the landed shape
including the served page. Blocked on you: nothing. Unverified: the
t52 staleness caveat still stands (the release binary goes stale
after pulls until rebuilt — t54's sources are in it now; the version
gate refuses a too-old binary rather than guessing); the dogfooding
store is a derived cache — after any board-only edit, run
`tools/progress.py --refresh` or the page renders the stale import
(the footer's instant makes that visible).

### 2026-10-07 — t53: cwd auto-detection — D8's deferred arming

The session's second delivery (the user's "ok, let's do U32 first,
then t53" — t58's entry below carries the first). The t53 spec was
authored from preparation reads already in hand (spec §7/D8, main.rs,
lib.rs, registry.rs, the drive harness); the dispatch needed three
spawn attempts — the first two implementer spawns died degenerately
(empty, then a two-character return) with a control spawn succeeding
in between, and the third landed only after the prompt was shortened
to point at the spec file rather than carry it (P-LIVE-001's recorded
remedy, now proven a fourth time: the long prompt is the failure
mode, the spec file is the payload).

The implementer landed the whole task with three honest findings: the
two frozen-file message pins (the old no-default message was
byte-pinned in `tests/write/edges.rs` and `tests/import/wire.rs` —
updated minimally, the f71 precedent, ratified by both reviewers), a
correction to the spec's J3 (macOS `getcwd` hands a spawned process
the physical path spelling, so the spec's raw-spawn pin pinned
nothing — the corrected test registers the raw spelling and lets the
kernel's resolution do the separating, with M2's kill proving the pin
live), and the memo widening (D3's `Resolved(Option<RepoSlug>)` sketch
could not retain the Ambiguous refusal's tied slugs, so the memo
caches the full `MatchOutcome`).

The review round earned its keep: the primary falsified the matcher's
tie logic over constructed candidate orderings and found it sound,
but both reviewers caught the same two stale-doc meds (a retired
message quoted in a frozen-deviation file's doc comment; the
`default_repo` field doc still ending "never a cwd heuristic" — the
pre-amendment framing) plus four lows that all landed: §6's
"ancestor" → "ancestor-or-equal", §7's fail clause (a store that will
not open fails the derivation — the `repo_list` precedent, and the
primary's sharp reason: skipping could flip a refusal into a silent
match), §13's spawn-directory phrasing, and the errored-derivation
path pinned by a new eleventh wire test (a corrupted second store
fails the derivation with `corrupt_store` even when the cwd matches
the healthy repo; the retry re-fails identically — the memo stays
pending on faults). One non-blocking low dispositioned rather than
looped (the t15 stopping rule): lib.rs's module doc says "ancestor"
where the normative sections carry "ancestor-or-equal" — f106, riding
lib.rs's next touch, because a one-word edit would have staled the
just-bound review shas. The skill's map line ("never a cwd heuristic")
was synced at closure — the primary's named doc-sync, review-exempt.

The live smoke is the arming's proof: the release binary against the
durable root, spawned in the camera checkout, `next_task` with no
repo parameter — `camera:U6`, byte-identical to the explicit
`repo: "camera"` call (tmp/t53-smoke-transcript.txt).

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; this
commit leaves the tree clean (the index backfill follows, AGENTS.md
having moved). Landed this session: t58 (U32's schema-format strip)
and t53 (this entry). Next step: **t54 — the U28 dogfooding cutover**:
measure this repo's board layout against the importer's grammar
(camera-shaped, neither dialect's layout constant), extend the
importer minimally, register dacode-todo-mcp in the durable root and
import its board, then `progress.py` becomes a thin client over a
per-render stdio spawn with the four hand-maintained tables retiring
(U22 resolving with it). Its spec owns the measured questions: the
refresh cadence (`apply_plan` refuses a non-empty graph, so a refresh
is a re-register), the GATING citation column's home, what the page
renders for the gitignored tracker's in-flight items. Blocked on you:
nothing. Unverified: the t52 staleness caveat still stands (the
release binary goes stale after pulls until rebuilt — now doubly so
with t53/t58 in it; the version gate refuses a too-old binary rather
than guessing).

### 2026-10-07 — t58: U32's schema-format strip — the client noise is gone

The session opened on `/todo-next-imp` proposing t53 (cwd
auto-detection); the user confirmed it, and mid-preparation reported
the U32 noise — opencode's Ajv validator printing `unknown format
"uint64" ignored in schema` lines against every `tools/list`, with a
pasted analysis proposing a server-side strip. The report matched
f99's recorded diagnosis exactly (52 warnings at the t39 smoke,
promoted to board U32 at g11, left at the M5 decomposition as "a
wire-visible decision if ever taken"), so the user's "ok, let's do
U32 first, then t53" is U32's sign — recorded on the board (§5
struck), the f2/U5 precedent for in-thread user decisions, no §12
item opened: the signed §6 surface, D14's key order and every golden
are untouched.

The measurement pass before any code (the state space, pitfalls 5.1):
a live `tools/list` drive against the durable root found **26 tools,
26 `format` keys, all `uint64`** — 25 in output schemas across five
tools, exactly one in an input schema (`next_items`' `limit`); the
output schemas nest their DTOs under `$defs`; `Option<u64>`'s
schemars spelling is `["integer","null"]` + `minimum: 0` + the format
hint; rmcp's `schema_for_type` returns an `Arc` shared with its
thread-local cache; schemars' `extend` attribute can insert but never
remove a key, while `schema_with` replaces a field's schema
wholesale. Those facts settled the design: one recursive strip walk
in a new `src/schema.rs`, applied at the 26 `output_schema` sites
(`Arc::make_mut` detaching from rmcp's cache) and at `limit` via
`schema_with` delegating to schemars' own `Option<u64>`
implementation — nullability never hand-written, rmcp's
input-validation path untouched, and the declined alternatives (the
`input_schema` attribute route needing the repo's first production
`expect`; replicating rmcp's private `validate_and_strip`) recorded
in the task spec.

The class is closed from the wire, not by the call sites: a
`tools/list` test walks every advertised schema — input and output,
`$defs` included — with its own independent copy of the family list,
so a future integer field that bypasses both seams fails it; a second
pin holds `limit`'s nullable-integer shape. The primary review's
round-1 med is the pin earning its keep one level down: the family
initially held 8 of schemars' 12 integer spellings (`int128`,
`uint128`, and the `isize`/`usize` spellings `int`/`uint` missing,
measured against schemars' `primitives.rs`) — a future `i128` field
would have re-triggered the warning past both the strip and the pin;
both lists now carry all twelve, and the number formats
(`float`/`double`) are out of scope because §8 bars floats from the
wire. Five mutations all killed at their predicted tests; the golden
ritual byte-identical; 210 todo-mcp tests (205 + 5), 548 workspace.

The live smoke is the user-visible proof: the release binary rebuilt
(the global-config entry names it), a fresh opencode session through
the global entry drove `repo_list` (the three real repos answered),
and the transcript and stderr hold **zero `unknown format` lines**
where the t52 transcript held dozens.

**Handover.** Workspace: `main`, four crates, rustc 1.98.1; this
commit leaves the tree clean (the index backfill follows, AGENTS.md
having moved). Landed: t58 (this entry). Next step: **t53 — cwd
auto-detection, in this session** — the user's original confirmation
stands ("U32 first, then t53"); the preparation reads are done (spec
§7/D8, main.rs, lib.rs, registry.rs) and the dispatch follows
immediately. Blocked on you: nothing. Unverified: nothing new — the
smoke is the before/after pair (t52's transcript vs this one), and
the staleness caveat from t52's handover still stands (documented,
not demonstrated).

### 2026-10-07 — t52: packaging per §13 — the global wiring landed

The session opened on the user's "let's dispatch t52" — the word for
M5's first task. Orchestrator-run per the t39/t48 precedent (the work
is config + operational: a user-machine config edit and an interactive
client drive do not survive a subagent boundary); the
customize-opencode skill governed the global-config edit and settled
the merge semantics that shaped the move — project config deep-merges
OVER global, so a same-name project entry would shadow the global one,
and the implemented pattern for global servers is the cve entry: it
lives in the global config with no agent-rules declaration at all.

The decisions taken inline (the t48 pattern, recorded here): the
global entry names the release binary per §13's signed text (not
`cargo run --release` — the staleness tradeoff documented in the
entry's comment: stale after pulls until rebuilt, the store-format
version gate refusing a too-old binary rather than guessing); and the
repo's t39 wiring MOVED rather than repointed — the todo-mcp server
block in agent-rules.yaml became a pointer comment, so the project
config renders no entry (nothing shadows the global one, and the smoke
could only have been served by the global entry), with the
orchestrator's `todo-mcp*` allow as a per-role permission override
beside U16b's deny. One course-correction mid-task, worth the record:
the first attempt used `global: true` on the server block believing it
suppressed the project render — it does not (it only skips the
top-level permission deny); the project entry survived and would have
shadowed the global one, defeating the smoke's evidentiary value. The
cve-pattern removal is the corrected shape.

The smoke (a fresh `opencode run` with the orchestrator agent, the
transcript at `tmp/t52-smoke-transcript.txt`): `repo_list` answered
the three real repos with their checkout paths and t48's registration
stamps — the durable root, not `tmp/todo-data`; `next_task`
repo=dacode answered `dacode:U175` with deferred {total 6, ready 6} —
t48's answer, the store unchanged; the permission=allow lines fired
for both tools (the role override working); and `todo-cli verify` on
all three stores post-session: exit 0, status ok. **g11's standing
caveat — no live client has queried the durable root yet — is
closed.** No Rust moved (git diff over `*.rs`/`Cargo.*` empty), so
every code gate is skipped by construction; the release build itself
(30.75 s, the tree's first) is the only compile evidence and it is
green.

**Handover.** Workspace: `main`, clean before this commit, four
crates, rustc 1.98.1. Landed: t52 (this entry; the wiring commit +
the index backfill that follows). Next step: **dispatch t53 — cwd
auto-detection (D8's deferred arming)**: derive the default repo slug
by matching the server's cwd (or an ancestor) against the registered
checkout paths, longest match wins, no match keeps today's no-default
behavior; lazy derivation (first default-needing call — paths live on
Repo nodes, so derivation pays t31's cold-open cost); the §7 amendment
recording the resolution order (param > flag > cwd) rides the task,
ratified at review per the D8-amendment precedent. Its alternative —
t54 first — is wrong because t53 is the smaller, self-contained
server change and t54's cutover measures its layout fit against a
tree t53 does not touch. Blocked on you: nothing. Unverified: the
global entry's staleness behavior is documented, not demonstrated —
no pull-then-stale-binary run was made (the version gate is
golden-pinned from t10); the smoke proves the global entry on this
machine only — a fresh clone or machine needs the entry installed,
which is t56's README.

### 2026-10-07 — t51: the M5 decomposition — U28 and U18 confirmed

The session opened on the user's "ok, confirm. Let's do what you have
proposed" — the word for t51 and for the two shaping calls the g11
handover named. Both calls were made in the opening report and
confirmed: **U28 → dogfood** (the §9 cutover applied to this repo —
the server becomes the query surface, the markdown board + `tasks.py`
stay the trackers of record, so the write surface and exit gate are
unchanged and the cutover is reversible) and **U18 → shape A**
(`progress.py --serve`; shape B, the shared per-user server instance,
declined with **D13 left reserved** — the spec's precondition for
drafting D13 has passed, so the decline is a choice: nothing in M5
needs a transport past signed D5, and a shared instance is a §8
single-writer commitment never tested).

The decomposition itself, per the t26/t41 shape: the tracker tree
**g12 + t52–t57 + g13** with every verify line a runnable command;
board §2's M5 row naming the ids and the dispatch order (t52 → t53 →
t54 → t55 → t56, t57 interleavable); the ledger re-measured (U13
unchanged at 812/772, U24's `listing.rs` 689 — t40's echo, U29's
`map.rs` 733 — t50's lift, `plan.rs` 244 / `helpers.rs` 382 under
target, no new entries); U21 scheduled as t57 and U22 resolving with
t54; U27 and U32 left as recorded calls with the reasons on their
entries (the timing-contract question and the wire-visible schema
change are decisions, not polish). No Rust moved — the decomposition
is docs + tracker + progress tables, review-exempt.

**Handover.** Workspace: `main`, clean before this commit, four
crates, rustc 1.98.1. Landed: t51 (this entry; the decomposition
commit + the index/hash backfill that follows). Next step: **dispatch
t52 — packaging per §13**: `cargo build --release`, the mcp entry in
the user's global opencode config naming the binary and the durable
root `~/.local/share/dacode-todo/`, this repo's own t39 wiring moved
off `tmp/todo-data`, and the smoke — a real client session querying
the durable root, closing g11's standing caveat. Its alternative —
t53 first — is wrong because packaging is the milestone's namesake
deliverable and both the cutover (t54) and the README (t56) document
what t52 lands. The t52 spec should load the customize-opencode skill
for the global-config edit and settle the release-binary-vs-cargo-run
staleness tradeoff with the no-double-server rule. Blocked on you:
nothing — both shaping calls are taken; U27 and U32 stay open as
recorded calls, neither gating an M5 task. Unverified: nothing new —
no code moved; the standing caveats are g11's (no byte-comparison of
the real dacode store against the golden, camera/ikvm-html
golden-less per D12, store/board drift per D7, no live client has
queried the durable root yet — which is exactly what t52 closes).

### 2026-10-07 — g11: the M4 exit gate ran green — M4 is closed

The session opened on the user's "ok, go for it" — the word for g11.
Every line of the gate's verify method ran on the committed tree
(`2e75bd0` at open, `240e7d2` after the gate's own code fix):

- **Code gates**: `cargo fmt --check` clean; `cargo check --workspace
  --all-targets` clean; `cargo test --workspace` **543 passed, 0
  failed, 1 ignored** (the store's known `#[ignore]` measurement
  test) over 24 result lines — t47's recorded counts exactly;
  `cargo clippy --workspace --all-targets` 0 warnings on a forced
  recompile; `cargo doc --workspace --no-deps` warning-free under
  **both** invocations.
- **The golden ritual**: `UPDATE_GOLDEN=1` over the store (120
  tests), cli (35) and mcp (205) suites, then `git diff --exit-code`
  over all three golden dirs = 0 — **all 24 goldens byte-identical**
  (19 mcp including the 1.1 MB fixture golden `dacode_dump.v1`, 4
  store, 1 cli).
- **The §6 tool-surface check**: a live `tools/list` drive against
  the running server — exactly the spec table's **26 tools**,
  `import_md` included, the sets identical (the spec side parsed
  from §6's table, the live side from the wire).
- **The live-run evidence**: t46's camera probe (178), t47's
  ikvm-html probe ×2 (132), t48's real imports into the durable root
  (178/133, all three stores verified) — recorded in the tracker and
  §3.
- **f102 swept** — the gate was its named owner. The reword of
  wire.rs's "all twenty-five" became a real dual review: round 1
  (both changes_requested on the same med) caught the class sweep
  stopping at the named instance — the carrier sweep's "twenty-three
  repo-scoped tools" and the module doc's unscoped claim carried the
  same falsifier; the fix round reworded all three sites to
  membership rules; round 2 both ok, bound to the same sha
  independently. Committed as `240e7d2` with both trailers.
- **The f41 sweep**: f102 done (above); **f99 promoted to U32**
  (its own verify line: "a U-id if it outlives M3" — it has); every
  other open item (f25, f50, f54, f57–f61, f65, f67, f69, f94,
  f101, f103–f105) verified open with its named consumer intact —
  M4 touched none of their files.
- **Pitfalls consolidated**: 4.7 (P-LIVE-009 promoted — non-ASCII in
  inline `git commit -m`), 4.8 (a digest names its algorithm — t50's
  two-tools scare), 5.11 (a live board is a moving target — every
  census is point-in-time, record the HEAD), 5.12 (a dialect study's
  inventory is a claim — the parser's strict refusals are the
  detector), 5.13 (the folding convention — a measured spec-vs-run
  divergence folds into the evidence, not a finding); top rules
  15–16; the live file reset, next id P-LIVE-010.

Spec §14's M4 line and board §2's M4 deliverable are both met: the
dacode importer verified against its checked-in fixture (t45), camera
and ikvm-html imported from their live boards and never copied in
(t46/t47), the three repos tracked for real (t48).

**Handover.** Workspace: `main`, clean after the gate's commits, four
crates, rustc 1.98.1. Landed: the gate itself — `240e7d2` (f102's
sweep, dual-reviewed, both trailers), the knowledge commit
(pitfalls), the board/AGENTS.md movement, the index backfill. Next
step: **decompose M5** — packaging per §13 (the global-config mcp
entry naming the binary and the durable store location
`~/.local/share/dacode-todo/`, which t48 filled with the three real
repos), docs, the D8-deferred cwd auto-detection, polish. Its
alternative — taking U18/U21/U22 first — is wrong because M5's shape
depends on none of them and a decomposition is cheap. Blocked on
you: nothing gates M5's decomposition; the calls that shape it are
**U28** (dogfooding this repo's own tracking on the server — allowed
since M3 closed, still undecided) and **U18** (the live view for
progress, holding the D13 transport reservation). Unverified:
nothing new — the gate ran every line of its verify method; the
standing caveats are t48's (no byte-comparison of the real dacode
store against the golden, camera/ikvm-html golden-less per D12, the
stores drift from the boards with no sync per D7, no live client has
queried the durable root yet — M5's wiring).

### 2026-10-07 — t48: the three repos tracked for real, the §9 cutover executed

The session opened on the user's "let's continue" — the word for t48,
the last M4 delivery before the exit gate. The work was operational,
not code: no spec was needed (the task's verify line is "manual +
recorded"), nothing in `crates/` moved, and the orchestrator ran it
itself (interactive server drives do not survive a subagent boundary).
The one decision taken inline: **the durable D4 root**
(`~/.local/share/dacode-todo/`, spec §2's signed location) is where
the three repos are registered — `tmp/todo-data` stays the smoke-only
harness root the t39 wiring comment names, because M5's packaging
(§13) is what moves the mcp entry to the global config with the
durable store location; wiring clients to the durable root now would
pre-empt M5's move.

The run itself: `repo_register` × 3 with the real checkout paths, then
`import_md` × 3 over the wire (the default dacode dialect, `camera`,
`ikvm-html`), then `todo-cli verify` × 3, then a fresh-process query
session over the imported dacode board. **Every import landed clean on
the first composed run** — the t45 headline ("the first composed run
on the real board was clean") holds for the real registration too, and
for both live-test dialects at their current drift: dacode 9/21/42/
218/43 (exactly the golden census — the sibling is at the fixture's
snapshot commit `671ce48`), camera 8/9/0/178/10 (the t46 probe's
census, at `c6783f6`), ikvm-html 9/22/0/133/22 (the predicted
post-`U129` count, at `24734f6`). All three stores verify (exit 0,
status ok). The query session answered `next_task` = `dacode:U175`
(`M1.1`, the first open gate under E2), `epic_list` 9 rows,
`task_list` under E8 = 174 rows with `U1`–`U170` contiguous, and the
read session is byte-identical across two runs — determinism
demonstrated on the real imported board, not just the fixture.

**Handover.** Workspace: `main`, clean, four crates, rustc 1.98.1.
Landed: t48 (this entry; docs-only commit, review-exempt — no code
moved, so every code gate is skipped by construction and that is
proven by the empty diff over `*.rs`/`Cargo.*`). Next step: **g11, the
M4 exit gate** — every code gate, the golden ritual over every golden
dir including the fixture golden, the §6 tool-surface check (26 tools,
`import_md` included), the live-run evidence for camera/ikvm-html
(t46/t47/t48's probes and imports), the f41 sweep, pitfalls
consolidation, board §2's M4 row done. Its alternative — starting M5
— is wrong because a milestone gate is cheaper than any M5 work and
M4 is one gate from closed. Blocked on you: nothing (attention.py
renders 10 P3 items — U18/U21/U22 and recorded debt; none gates g11).
Unverified: the dacode store matches the golden's census across two
independent surfaces (import counts, `todo-cli verify`) but was not
byte-compared against `dacode_dump.v1` — the timestamps differ by
construction (`imported_at` is the real clock here, the fixed constant
in the golden), and the golden's graph is pinned at the same sibling
commit; camera and ikvm-html have no golden at all (D12's signed
consequence — their censuses are cross-checked against their probes
only); the stores will drift from the boards as the siblings evolve
(no sync, D7) — the sibling HEADs in §3's t48 entry are the baseline;
and this repo's own opencode wiring still talks to `tmp/todo-data`
(smoke only), so no live client session has queried the durable root
yet — that is M5's wiring, not a defect.

### 2026-10-07 — t50 dispatched and landed: the U31 lift, the study for t47, two specs written

The session opened on the user's "let's start" — the word for t47.
The t41/t46 precedent governed the order: **the study ran first**, as
measurements against the live ikvm-html board at
`/Users/mouse/RPM/ali/ikvm-html` (clean at `090c2e1`), and the study
settled U31's answer before any code moved. The study's headline
facts: a **two-file board** (`TODO.md` 15,589 lines +
`decisions.md`; `README.md` is the plan index, not a board file); the
U register is §4+§5's checkbox items — **`U1`–`U128`, zero gaps, no
oddballs**; **statuses are the checkbox alone — strikethrough here is
edit history** (8 unchecked items carry struck title fragments:
re-measured line counts, renamed scopes), which falsifies §9's "and
strikethrough state" half for this board and becomes the first
veto-able reading; §2's milestone table has **no State column**
(milestone status not derivable — the third reading); **§3 holds four
id-less blocker checkbox items** — a population neither sibling has
(the second reading: tasks under a synthetic `Blockers` epic);
decisions.md carries a summary table + dossiers + a **Resolved
footer** whose dates agree with the dossier banners exactly (20/20,
zero mismatches — a pinnable three-way cross-check); the Confidence
vocabulary is wider than camera's (`low-medium`, `medium-high` — the
fourth reading: both → `Medium`); and the hard grammar: **the bold
markers carry no title boundary** (U69's `**` tokens are all paired
emphasis — measured), so the title rule is **first-sentence** (the
first `.` outside struck and code spans), verified total over
128/128 by script. Zero mechanical dependency prose in register
bodies — the Gates column is the only edge source.

**U31 was decided at dispatch, as its own item demanded**: lift, as
a separate preflight commit (t50) rather than inside t47's diff —
the third copy is not made, and t47's module builds on the neutral
homes. Two specs were written (`tmp/t50-spec.md`, `tmp/t47-spec.md`
— the latter's frozen baselines computed at dispatch, post-t50).
t50's implementer round landed the lift with zero behavior change;
the review's round 1 (both changes_requested) found only doc-level
stale claims the move created plus one visibility nit, the fix round
landed seven items, and the binding round returned both ok — the
primary hashing SHA-1, the secondary SHA-256, every file matching
under its respective algorithm (the t34 sha-mismatch scare resolved
as two tools, not two trees). The mutations that matter: corrupting
the shared `cells`/`collect_rows`/`position` fails **both dialects'
suites at once** — the proof the sharing is real, which two copies
could never produce. What the session leaves behind:

- **The importer's shared layer is dialect-neutral**: plan types in
  `src/import/plan.rs`, helpers in `src/import/helpers.rs`, the
  `HelperError` seam, `collect_rows` parameterized over each
  dialect's register-construct rule. 524 tests, every count
  unchanged, both goldens byte-identical.
- **f104 recorded**: `ApplyError::Refused` carries dacode's
  `ImportError` — a type-level coupling with dialect-neutral
  messages, named in `plan.rs`'s module doc as the debt for its next
  owner. t47's dispatch note carries it (an ikvm import surfaces the
  type; the messages are store-level facts).
- **t47 is fully specified and unblocked**: the study, the four
  veto-able readings, the grammar, the conventions, the mutations
  and the live-probe protocol are all in `tmp/t47-spec.md`; the
  dispatch computes the post-t50 baselines and spawns the
  implementer.

**t47 then dispatched and landed in the same session**: the
implementer round landed the module (five files, all under the
budget), the wire value, the widening and the probe; the review's
round 1 (both changes_requested) found the never-fired pins
unpinned, the id shapes letter-tolerant, and a handful of doc-level
stale claims; the fix round landed ten items; the binding round
both ok on identical SHA-256 prefixes. The live probe ran twice
(implementer + primary over the wire), the census matching the
study's predictions exactly, and the board grew its own U129 after
the probe (the t46 precedent). The narrative is §3's t47 entry.

**Handover.** Workspace: main, clean after this commit; four crates,
rustc 1.98.1. Landed: t50 (the U31 lift, `e8813cc`) and t47 (the
ikvm-html dialect — this commit). Next step: **dispatch t48 — the
three repos tracked for real** (register dacode/camera/ikvm-html in
the server's data root, import each from its live board, verify
each store, record the cutover state per §9; a real query session
over the imported dacode board with the transcript as evidence).
Then g11, the M4 exit gate. Blocked on you: nothing (D13 stays
reserved, U18 deferred until after M4's importer check).
Unverified: the cutover itself (t48's deliverable — until then the
markdown boards remain the trackers of record for the three
siblings); the ikvm grammar's totality holds on the live board as
measured 2026-10-07 (the board drifts — U129 already landed — and
the strictness rule fires on unknown shapes rather than skipping
them).

### 2026-10-06 — t46 dispatched and landed: the camera dialect, the study first, no new decision owed

The session opened on the user's "ok, let's do t46" — the word for
the camera dialect. The t41 precedent governed the shape: **the
study ran first**, as measurements against the live board (the
census, the status vocabulary, the dependency shapes, the file
layout), and the study's central finding shaped everything after
it: **every camera population is named by §9's signed mapping
table** — no new D-entry was owed, and the differences from dacode
(one file plus decisions.md, table milestones, no emoji, no doc
register, a two-section U namespace) are shapes and conventions the
spec settles as veto-able readings. The spec was authored from the
study (`tmp/t46-spec.md`: 13 decisions, 6 veto-able readings, 7
required tests, 5 mutations, 39 frozen baselines), then one
implementer round, two fix rounds, and dual sign-off over three
review rounds. The narrative is §3's t46 entry. What the session
leaves behind:

- **The importer now speaks two dialects**: dacode (golden-pinned
  end to end) and camera (live-probed: 178 tasks, the base range
  reproduced, no golden per D12). `import_md` takes a `dialect`
  param whose default keeps the dacode path byte-identical. What
  remains of M4 is ikvm-html (t47) and the cutover (t48).
- **The review's three rounds each found something real**: round 1
  the inert pins (the reviewers' own mutants survived the suite —
  the unpinned refusals and grammar paths), round 2's behavior
  widening (the phrase scan skipping non-id occurrences,
  live-neutral), round 3 a genuine silent-double-import defect (the
  per-register duplicate check missing a cross-section U id —
  camera's U namespace spans §4/§5 where dacode's is one register).
  The cross-section shape is exactly the kind of thing the study's
  census could not see and the review's probe did.
- **The live board drifted mid-task** (its own U170 landed): the
  probe measured 178, the study 177, the first census 170 — all
  three recorded honestly in the tracker evidence, and the drift is
  the reminder that the live-test subjects are live.
- **Two follow-ups recorded**: f103 (§2's duplicate-M gap,
  taste-level, rides the next camera/ikvm touch) and **U31** (the
  dialect-helper duplication — t47's first spec decision: lift the
  shared helpers and the plan types into a dialect-neutral home, or
  accept the third copy deliberately).

**Handover.** Workspace: `main`, clean at the t46 commit; four
crates; rustc 1.98.1; 524 tests, all green. Landed: t46 — the camera
dialect (`src/import/camera/` six files, the `dialect` param on
`import_md`, 14 tests, the live probe removed; §3's entry carries
the detail and the census). Next step: **dispatch t47 — the
ikvm-html dialect** (study + parser + live import against
`/Users/mouse/RPM/ali/ikvm-html`, the last live-test subject per
D12): the study measures its board the same way, and **U31 is the
first spec decision** — the shared helpers and the plan types
either lift into a dialect-neutral home under `src/import/` or the
third copy is accepted deliberately. The camera module is the
pattern; the study's §9-coverage check decides whether a D-entry is
owed before any code. Alternative considered and declined: t48
first (the cutover) — the cutover needs all three dialects
importable, and ikvm-html is the last one. Blocked on you: nothing.
Unverified: nothing new — camera is live-probed and dacode
golden-pinned; ikvm-html is unstarted by design. Tracker:
`tasks.py check` exits 1 on exactly the two expected lines (g10 in
flight, g3 standing); t46 closed with evidence.

### 2026-10-06 — t45 dispatched and landed: the fixture checked in, the first composed run clean

The session opened on the user's "ok, let's continue" — the word for
t45. The spec was authored first (`tmp/t45-spec.md`: 10 decisions, 5
judgment calls, 4 veto-able readings, 3 required tests, 5 mandatory
mutations including one production mutation, 37 frozen baselines),
with the two design questions settled by measurement: the golden's
determinism (the wire tool's clock forces the in-process
fixed-stamp path for the byte-exact golden, with the tool layer
pinned separately through its deterministic counts) and the fixture
tree's shape (the three real files plus derived `{key}.md` stubs —
t44's params had already decided this). One implementer round (the
first spawn died on a connection error; the control-spawn-then-retry
remedy carried it, proven a third time), one orchestrator-applied
fix round, then dual sign-off. The narrative is §3's t45 entry. What
the session leaves behind:

- **The importer is proven end to end on the real board**: the first
  composed run was clean — no mapping gap, no parser gap, no
  importer code changed. The expected graph is now a checked-in
  golden (`dacode_dump.v1`) with the board's `U1`–`U170` numbering
  reproduced, and the wire composition is driven beside it. What
  remains of M4 is the other two dialects (t46 camera, t47
  ikvm-html) and the cutover (t48).
- **The census matched the study exactly** — 9 epics, 21 decisions,
  42 docs, 218 tasks, 43 edges, 0 relates, the 4 predicted findings
  — which is itself evidence the t41 dialect study measured rather
  than read.
- **The review's round 1 was the f55 class at its clearest**: both
  reviewers independently flagged the same med — the import target's
  module doc said the live board's prose is "never copied into this
  repo", falsified by the diff whose own subject was copying it in.
  The implementer had correctly left it (the spec's owned set
  allowed one line in that file); the fix rode the orchestrator
  round with both reviewers' ratification.
- **No new findings, no new debt**: the two f-ids of the t44 session
  (f101, f102) still ride their named next touches; the secondary's
  one dispositioned low (the retired P-LIVE citations, pre-existing
  in read.rs too) stands dispositioned.

**Handover.** Workspace: `main`, clean at the t45 commit; four
crates; rustc 1.98.1; 510 tests, all green. Landed: t45 — the
checked-in dacode fixture + the end-to-end expected-graph golden
(the three real files at `tests/fixtures/dacode/`,
`dacode_dump.v1` at 1.1 MB, 3 e2e tests; §3's entry carries the
detail). Next step: **dispatch t46 — the camera dialect** (study +
parser + live import against
`/Users/mouse/RPM/shelly/camera`, a live-test subject per D12 whose
board content is never copied into this repo): the study measures
camera's board shapes against §9's mapping table the way the t41
study measured dacode's, the parser lands as a sibling dialect
module under `src/import/`, and the live import runs against the
real board with the result recorded — no pinned golden for camera
(D12's signed consequence: two of three dialects get no golden, the
synthetic-fixture remedy named if M4 finds it too weak). The
importer's architecture is proven; t46's risk is dialect-shaped
(camera's board is simpler — `U`/`D`/`M` registers, session logs —
but its shapes were never measured). Alternative considered and
declined: t47 (ikvm-html) first — the two are swappable by the
decomposition and camera is the named next. Blocked on you: nothing.
Unverified: nothing new — the dacode pipeline is pinned end to end;
camera and ikvm-html are unstarted by design. Tracker: `tasks.py
check` exits 1 on exactly the two expected lines (g10 in flight, g3
standing); t45 closed with evidence.

### 2026-10-06 — t44 dispatched and landed: the import_md wire tool, the refusal mapped to invalid

The session opened on the user's "let's continue on t44" — the word
the previous handover waited for. The spec was authored first
(`tmp/t44-spec.md`: 12 decisions, 5 judgment calls, 5 veto-able
readings each pinned by test, 9 required tests, 5 mandatory
mutations, 25 frozen baselines), with the two open design questions
measured before dispatch: the doc-register key→file rule (a live-tree
walk: 42 keys, 43 `.md` files, every key resolving uniquely under
`stem == key || stem starts key-`) and the refusal's wire code (the
t43-review input, settled as `invalid` via a typed `ApplyError` seam
rather than a tool-side pre-check, which would duplicate apply's
precondition — the one-home rule). One implementer round, one fix
round, then dual sign-off over three rounds. The narrative is §3's
t44 entry. What the session leaves behind:

- **The importer is complete up to the fixture**: parse (t42) → map
  (t43) → apply → **the wire tool**, all pinned. What remains of M4
  is the checked-in fixture + the e2e golden (t45), the other two
  dialects (t46/t47) and the cutover (t48).
- **The review's round 1 found two surviving mutants** — the primary
  built its own (dropping the `.md` filter; dropping the sort) and
  both survived the 9-test suite: the stray-file pin was inert and
  the sort was defended only by APFS `read_dir` coinciding with byte
  order for the test's exact name set. Both fixes made the pins
  load-bearing rather than the code louder — the extension-less
  `README` stray, and the sort relocated into the pure `doc_paths`
  seam with a reversed-input unit test. The lesson is the recurring
  one: a pin that cannot fail under the mutation it exists for is
  not a pin.
- **One staleness rule exercised**: the fix round changed two files
  after the secondary's ok, so its sign-off was stale for exactly
  those bytes — a short binding round re-bound it to the final shas
  before the trailers were honest. Cheap, and the trailer rule
  demands it.
- **Two findings opened** (f101, f102) and three folding-convention
  divergences recorded in the tracker evidence (the D8 stamp-equality
  scoping — D18 stamps B/A dossiers with board dates, so the spec's
  literal "equal across the imported nodes" cannot reproduce; the
  D10 "for free" walker claim; the golden ritual's staging order).

**Handover.** Workspace: `main`, clean at the t44 commit; four
crates; rustc 1.98.1; 507 tests, all green. Landed: t44 — the
`import_md` wire tool (`src/importer.rs` + the `ApplyError` seam in
`plan.rs`, 9 wire tests + 2 unit pins, the `import_md.v1` golden; §3's
entry carries the detail). Next step: **dispatch t45 — the checked-in
dacode fixture + the end-to-end golden** (§10/D12: the three real
files are the only sibling content this repository holds; the e2e
imports them into a temp store whose canonical dump matches the
golden byte-for-byte). t44's params decided its shape: the fixture is
the three real files, and the e2e assembles the tree at runtime by
copying them into a temp `docs/implementation/` and generating stub
`{key}.md` files for the remaining register keys (derived from the
parsed register, so the fixture self-maintains) — the t44 spec's D2
records the consequence. The fixture snapshot should be taken from
the sibling's current files (their 22/23 strike is committed there as
`c5dd47d`). Alternative considered and declined: t46 (camera) first —
the e2e golden is the pin that proves the importer end-to-end against
the real board, and everything downstream (the dialect studies, the
cutover) inherits its confidence. Blocked on you: nothing.
Unverified: the importer composed end-to-end against the real board —
the exact gap t45 exists to close (the parser is total over the live
board and the mapping is pinned at synthetic scale, but the two have
not run composed on the real files through the tool). Tracker:
`tasks.py check` exits 1 on exactly the two expected lines (g10 in
flight, g3 standing); t44 closed with evidence.

### 2026-10-06 — t43 dispatched and landed: the D18 mapping, id reproduction pinned at scale

The session opened on the user's "ok, go ahead" — the word that
dispatched t43 against the spec authored the previous exchange (and the
sibling dacode repo's 22/23 strike committed there as `c5dd47d` at the
user's ask, both trees clean since). One implementer round, one review
fix round, then dual sign-off — the shortest delivery of M4 so far,
because the spec was authored against a frozen AST and the measured
facts (the §6 inversions) were settled before dispatch. The narrative
is §3's t43 entry. What the session leaves behind:

- **The importer's semantic half is done**: parse (t42) → map (t43) →
  apply, all pinned. What remains of M4 is the wire (t44), the fixture
  (t45), the other two dialects (t46/t47) and the cutover (t48).
- **The review's round 1 was the load-bearing one again**: both
  reviewers independently deleted the references push and watched the
  suite stay green (the reading was pinned in prose, not in a test);
  the secondary caught the edge timestamps unpinned the same way, a
  false D18 citation (the f62/5.6 class — the synthetic epics'
  description wording presented as signed text when decisions.md says
  only "descriptions carry their provenance"), and three live-prose
  lifts in the corpus; both found the refusal gap from different
  angles (a used-then-emptied store passing the node-count check).
  All landed with the probes pinned as tests.
- **Two follow-ups opened**: U29 (map.rs over target, the recorded
  exception) and U30 (the t42 duplicate-id strictness gap the primary
  probed — frozen code, the parser files' next owner). The refusal's
  wire code is a t44 spec input (a new §6.1 code would be a spec
  change; the honest t44 default is mapping the refusal to an
  existing code and flagging the question).
- **One spec claim corrected at review time**: the literal
  "cut at the first emoji" title rule empties 16 live leading-flag
  titles (U25/U37/U56… measured) — leading flags are stripped before
  the cut, reported not worked around.

**Handover.** Workspace: `main`, clean at the t43 commit; four crates;
rustc 1.98.1; 496 tests, all green. Landed: t43 — the D18 mapping
(`map.rs`/`plan.rs`/`resolve.rs`, 27 map tests, the ImportPlan +
apply_plan, id reproduction pinned at the live board's scale; §3's
entry carries the detail). Next step: **dispatch t44 — the
`import_md` wire tool** (params, the store writes through
`map_board` + `apply_plan`, every fault carrying its §6.1 code via
D15's carrier, the wire golden), authored against t43's public
surface and the refusal's wire-code question recorded above.
Alternative considered and declined: t45 first (the fixture) — the
wire tool is what the fixture's e2e drives, and its params decide the
fixture's shape. Blocked on you: nothing. Unverified: the importer
end-to-end against the real board (t45's fixture e2e — the mapping is
pinned at synthetic scale, and the parser is total over the live
board, but the two have not run composed on the real files; t45 is
that pin). Tracker: `tasks.py check` exits 1 on exactly the two
expected lines (g10 in flight, g3 standing); t43 closed with
evidence.

### 2026-10-05 — t42 dispatched and landed: the dacode dialect parser, total over the live board

The session opened on the user's "let's continue from the last session.
Next is a spec as per handover" — the word that dispatched t42. The task
spec was authored at `tmp/t42-spec.md` against D18's inventory and the
§9 rows, with the live board's shapes re-measured first (the study's
"'blocked on <id>' ×10" corrected in the spec before dispatch: the ten
live occurrences are prose, the id-bearing form appears once, in the A
appendix — the f62/5.6 class caught at spec time rather than review
time). Three implementer rounds (initial, the six-shape fix round, the
final probe round), then five review rounds to dual sign-off; the
narrative is §3's t42 entry. What the session leaves behind:

- **The parser is total over the live board** — the strongest single
  fact of the delivery, probe-verified by the implementer, both
  reviewers and the orchestrator independently, with the counts
  cross-checked against the board's own header (19 of 42) and register
  (U1–U170). t45's fixture work inherits a parser that already parses
  the real files.
- **The sibling dacode repo carries one uncommitted edit of ours**:
  docs 22/23 struck in §4's M1 row (their own recorded drift — the
  count paragraphs and expanded list said expanded, the row was never
  struck). Their tree is dirty for their owner to review and commit;
  reverting is a one-line strike if they disagree. t45's fixture
  snapshot should be taken from the fixed files.
- **The review earned five rounds**, and each one found something real:
  the primary's row-wrap swallowing register lines (a silent-omission
  shape in the parser itself), the secondary's id-shape off-by-one at
  every id gate, the pasted live-board prose in a test corpus (twice —
  the second time as two verbatim titles the first sweep missed), the
  spec-local D-id citations reaching committed files (the f70 class),
  and a false empirical claim with a duplicate (the class-sweep lesson
  applied twice). One process note recorded for the pattern: the
  primary's round-3 sha list did not reproduce against the tree (its
  substantive checks did) — sha reports are claims like any other, and
  the final rounds required both reviewers to paste `shasum` output
  computed on the repo files, verified against the orchestrator's own.
- **Dispositioned, not fixed**: five measured-shape pins in the test
  corpora carry short live-prose fragments (a struck Depends-on tail,
  `plain | compressed`, `APPLIED via A16`, a B20 status tail shape) —
  ruled dialect evidence, not prose lifts; t45's fixture will carry the
  real files wholesale anyway.

**Handover.** Workspace: `main`, clean at the t42 commit; four crates;
rustc 1.98.1; 469 tests, all green. Landed: t42 — the dacode dialect
parser (`crates/todo-mcp/src/import/dacode/`, ten files, 46 tests,
`parse_board` total over the live board; §3's entry carries the
detail). Next step: **dispatch t43 — the D18 mapping** (dialect AST →
domain drafts + edges: the status mapping, timestamps, evidence
bounding, the mechanical-edge rule, the synthetic epics, id
reproduction), authored against D18's conventions (decisions.md) and
t42's AST; its task spec settles the mapping semantics the parser
deliberately left open. Alternative considered and declined: t44 first
(the wire tool) — the mapping is the semantic risk and the AST it
consumes is now frozen. Blocked on you: nothing for t43; the sibling
dacode repo's dirty file (the 22/23 strike) is yours to review and
commit. Unverified: nothing new — the parser's totality is
probe-verified; the mapping (t43) and the wire tool (t44) are unstarted
by design. Tracker: `tasks.py check` exits 1 on exactly the two
expected lines (g10 in flight, g3 standing); t42 closed with evidence.

### 2026-10-05 — M4 decomposed: the dialect study, D18 drafted, t42–t48 + g11

The session opened on the user's "let's start with dialect study" —
agreement to the proposed decomposition, study first. The study was
run as a measurement against the live dacode board
(`/Users/mouse/RPM/dacode/dacode/dacode`), not a cover-to-cover read:
register censuses (contiguity checked id-by-id, not inferred), status
vocabulary counts, dependency-prose census, table-column shapes,
sub-step table structure, the doc register. The findings that shaped
the decomposition:

- **The board is three files.** Decisions live in
  `appendices/B-decisions.md`, deltas in
  `appendices/A-design-deltas.md`; `TODO.md` §2/§3 carry summary
  tables. An importer that reads only `TODO.md` cannot build decision
  dossiers. §9's dialect sentence now records the fact.
- **The registers are contiguous.** `U1`–`U170` with zero gaps (and
  `B1`–`B21`, `A1`–`A28`) — so importing in board id order lets the
  server's own never-reuse allocation reproduce the board's
  numbering, and the importer needs **no explicit-id insert surface,
  no Store change, no domain change**. This is the measurement the
  whole architecture rests on.
- **§9's signed mapping table covers four shapes; the board has
  nine populations.** The five uncovered (the `A`-delta register,
  sub-steps, docs, parentless `U` items — §3 requires an epic parent —
  and id reproduction) are **D18**, drafted with the inventory as
  evidence and one contested choice (A-deltas → tasks recommended
  over decision nodes: the board's §3 is a work queue, and mapping
  approved-but-unapplied deltas to *open decisions* would tell every
  cold session the user owes a vote on something already decided).
- **The dialect is line-anchored.** Items, tables and status markers
  are single-line shapes — a hand-rolled strict parser needs no
  markdown dependency, and §2's signed four-member workspace stands
  (the importer is a `todo-mcp` module).

The task tree under g10: t42 parser → t43 mapping (gated on D18) →
t44 the `import_md` wire tool → t45 the checked-in fixture + e2e
golden → t46 camera → t47 ikvm-html → t48 the three repos tracked
for real → g11 the exit gate. progress.py's tables moved with it.

The user then signed **D18 option (a)** in-thread — A-deltas → tasks
— the same day it was drafted. The work order executed in one change
(t49): §9's mapping table carries the five dacode rows, §10's
fixture bullet names the three files, §12 item 18 is deleted with the
preamble recording the signing, the decisions.md row is struck with
its resolution banner, and the conventions stay task-spec level
(pinned by t43's tests, veto-able as readings). t43 is unblocked;
nothing blocks M4.

The session's third act came out of the user's question: can
`progress.py` retrieve its information dynamically instead of the
hardcoded `LANES`/`GATING`/`EXTRA`/`WAITS` tables — perhaps with the
server behind it, as a follow-up after the MVP is proven? The answer
is **U28**: the tables are a hand-maintained shadow of the server's
own model (lanes ↔ epics, gating rows ↔ `blocks` edges, waits ↔
`blocked`/`blocked_by`, extra entities ↔ nodes), no new §6 tool is
needed, and the missing piece is the dogfooding cutover §9 has
sanctioned since M3 closed — recorded on the board, deferred until
after M4, depending on U18/D13's transport territory either way.

Handover: **M4 is decomposed and ungated; the next act is dispatching
t42 (the parser).** Workspace: main, four crates, rustc 1.98.1, 423
tests, clean. Landed: the decomposition as `230d49c` (+ the index
backfill `8b37055`), the D18 sign-off execution as `b8bd5da` (+ the
index backfill `8a49a4b`), U28 as this session's last board edit.
Next step: **dispatch t42** — the dacode
dialect parser, `crates/todo-mcp/src/import/`, strict line-oriented
parsing into a dialect AST, no new deps; its task spec is authored
against D18's inventory (decisions.md) and the §9 mapping rows. Then
t43 (the mapping, ungated since D18 signed). Blocked on you: nothing.
Non-blocking: U21/U22 (user calls), U18 and **U28** (both parked
until after M4's importer check — the live view/transport question
and the progress.py-goes-dynamic cutover), U27 (the store's next
owner), the rider set (f57/f58/f60/f61/f67/f94/f99, U13/U24/U25).
Unverified: nothing new — the study's claims are all commands
re-runnable against the live board; the importer itself is unstarted
(M4's work). One known cosmetic defect: `230d49c`'s message body
mangled its § signs to "SS" (P-LIVE-009 records the `commit -F`
workaround); fixing it needs `--amend`, the user's call.

### 2026-10-04 — g9 ran green: M3 is closed

The session opened on the user's "let's run g9". The gate ran its
whole verify method on the committed tree (f262db2): every code gate
(423 workspace tests, 0 failed), the golden ritual byte-identical
over all three golden dirs, the §6 tool-surface check via a live
`tools/list` drive (25 tools; `import_md` the only spec-only name —
M4's per D12, exactly the gate's criterion), the smoke evidence
(t39's, already recorded), the f41 sweep, and the consolidation.

The gate's own work was **U26**: the flake reproduced on demand
(1-in-30, then 1-in-10 full-crate runs), and the isolation
experiment the board prescribed ran in-tree with sha-verified
reverts — the lsof+ps diagnostic (no fd holder anywhere, the server
dead), the retry-loop measurement (the lock still held 1.33ms past
the refusal), and variant A (every in-process open removed: 40/40
clean). The cause: the in-process `repo_list()`'s open/close plus a
deferred kernel-side flock release. The fix — a bounded `locked`
retry at the final open, both sites, tests-only — ran 40/40 clean
and was dual-reviewed over two rounds (the second doc-only: the
hammer's header rewritten to post-fix semantics, the measurements
moved to U26's ledger per the no-measurement rule). **U27** opened
for the store-side question (`FileLock::acquire`'s WouldBlock retry
— a timing-contract change needing multi-platform evidence).

The consolidation promoted seven entries (1.2, 2.2's extension, 3.3,
3.4, 4.6, 5.9, 5.10; top rules 13–14) and closed f70/f73/f97/f100
into them. The sweep closed f4, f59, f63, f64, f68, the ratified
deviations, and the spec-divergence records (superseded by the
folding convention); the rest verified open with named consumers.

Handover: **M3 is closed; M4 is next** — the dacode-dialect importer
first (D12), its board the only fixture checked in per §10, camera
and ikvm-html as live-test subjects never copied in, the three repos
tracked for real. The decomposition is the next session's first act
(a g10 goal, the importer tasks, the fixture question). Nothing
blocks it. The open user calls are unchanged: U21, U22, U18 (parked
until after M4's importer check), U27 (new — the store's next
owner), and the small rider set (f57/f58/f60/f61/f67/f94/f99, the
U13/U24/U25 ledger and splits).

### 2026-10-04 — t40 landed: D17's echo, priority on the triage surfaces

The user signed **D17** ("let the agent decide") and this session
executed its implementation task, `tmp/t40-spec.md` (gitignored
scratch), against the amended spec rows (§5.2's batch description,
§6's `next_task`/`next_items`/`task_list` rows, §11's priority
bullet) and the D17 dossier in `decisions.md`.

The echo: `priority: Option<Priority>` on the domain row types
(`NextTask` beside the task field, `NextItem` and `TaskRow` appended
after their existing fields) and `Option<PriorityDto>` on the wire
DTOs in `TaskDto`'s spelling — `null` when unset, the key never
omitted, pinned by key-presence assertions so a `skip_serializing_if`
drift dies. The row constructors read the field off the node; no new
state, no derivation, no ordering change. The base instructions
gained D17's handling sentence and the protocol suite's frozen copy
moved with it (the exact-byte initialize pins enforce the sync).

The order is untouched, and that is now a checked fact: the
order-still-priority-free pin builds a graph whose tasks hold
differing priorities and asserts all three surfaces answer in the
same order as the priority-free twin; mutation M5 (a
priority-preferring `select_from`) dies on it — on its `next_task`
assertion — beside the t27 priority pin and the `next_task`/first-row
cross-pin. The wire suite stays green under M5 (measured: the
fixture's U9 is both canonical-first and highest-priority, so the
goldens do not move under that mutant — the pin is the killer, not
the fixture). Calibrating M5 found a trap worth naming: `Priority`'s
derived `Ord` is declaration order (`High < Medium < Low`), so a
"prefer higher" written as `>` prefers low — the mutant's rank is
spelled out.

The compatibility event: the fixture widened to three distinct
values (U9 high, U10 medium, U11 low — D2), and **four goldens moved
as additive fields** — `next_task.v1` (`"priority":"high"` beside the
id, `null` on the None answer), `next_items.v1` and `task_list.v1`
(per-row `"priority"`), and `graph_show.v1` (the widening reaches the
dump's U9/U10/U11 task nodes; two of its nine lines change).
`task_show.v1` measured byte-identical — it pins U1/U2, untouched —
and the other thirteen goldens are byte-identical (checked with
`git diff --exit-code` over the unmoved set).

Two spec divergences, both resolved toward the gate per the f71
precedent and flagged for the reviewers' ratification: the owned-file
set omits `crates/todo-domain/src/graph/list.rs` (the `TaskRow`
construction site — the field addition forces the edit) and
`crates/todo-mcp/tests/write/edges.rs` (two `next_task` payload
assertions the echo moves). The task prompt's "clean tree at 5a848b8"
and "committed D17 dossier" were also off: the D17 spec/board/decisions
amendments were already in the working tree, uncommitted — left in
place, since the orchestrator's commit carries them with the code.

Counts: 183 domain tests (179 + 4: the echo per surface, the purity
pin), the read suite 27→30 (the echo tests + the null-spelling pin),
protocol 10 (needles moved, count unchanged), wire 5 and write 26
unchanged. Mutations M1–M5 run, observed, reverted, sha-verified; the
named kill sets were supersets as usual (M2's includes the two
write-suite payload assertions; M1/M3's include the domain-agreement
and filter tests). Every gate green. Landed as `cf8e996` (the D17
sign-off record as `2a7749e`); the next step is **g9**, the M3 exit
gate.

### 2026-10-04 — t39 landed: the server wired into its own harness, smoked by a real client

The session opened on the user's go: t39's spec and implementation,
plus two side orders — the findings-folding convention (landed as
`a86b1cc`: an f-id is for what survives the branch; scratch-spec
divergences fold into the task's evidence line) and a priorities
proposal drafted as **D17** for the user's sign-off (presented in
chat; D11's signed "priority never reorders" stands until then).

t39 itself: the orchestrator did the work (interactive, per the
routing rule), from `tmp/t39-spec.md`. The config change — the
`todo-mcp` mcp entry, the orchestrator's `todowrite: deny`, the
live-view paragraph rewrite, `version.custom` 10→11, re-sync — then
the smoke. The transcript's meaningful exchanges (the full JSON
event stream is `tmp/t39-smoke-transcript.json`, gitignored; the
connection log `tmp/t39-smoke-stderr.log`):

- `repo_register` → `{"slug":"smoke","name":"Smoke fixture",
  "path":"/tmp/smoke-fixture","registered_at":"2026-10-04T20:20:50.470659000Z"}`
- `epic_add` → `{"id":"smoke:E1","repo":"smoke","title":"Bring-up",…}`
- `task_add` ×2 → `smoke:U1` ("First task"), `smoke:U2` ("Second task")
- `task_link` → `{"edge":"blocks","from":"smoke:U2","to":"smoke:U1"}`
- `repo_list` → `{"repos":[{"slug":"smoke",…}]}`
- `epic_list` → `{"epics":[{"id":"smoke:E1","title":"Bring-up"}]}`
- `next_task` → `{"task":"smoke:U2","deferred":{"total":0,"ready":0}}`
  — the dependency-correct answer: U2 blocks U1, so U1 is held.

The connection evidence is the log's eight
`evaluated permission=todo-mcp_* … action=allow` lines (there is no
literal "MCP connection" line — the spec's phrasing diverged from
what the log shows, folded here per the new rule). Post-session
`todo-cli verify`: exit 0, status ok. Dual review ok in one round
(both reviewers; five lows and two meds, all record-keeping — the
uint64 note became f99, the U16 resolution and the task-tracking
exception landed with this commit).

**f99** — the first real client's compatibility note: opencode's
schema validator emits 52 "unknown format uint64 ignored in schema"
warnings against the server's schemars-1 `u64` fields
(`NextItemDto.weight`, the D16 count DTOs). Harmless today — the
validator ignores what it does not know — but a strict-mode client
could hard-error on the same schemas. The fix (if ever needed) is
the server's: a schema-level integer spelling the validator knows,
which is a wire-visible change and therefore a decision, not a
patch.

Handover: t39 is committed (review-exempt class, no trailers — the
dual review is recorded in the tracker evidence); **D17 signed
2026-10-04** ("let the agent decide": `priority` echoed on
`next_task`'s answer and the `next_items`/`task_list` rows, the
order untouched, the handling rule in the base instructions) — its
task **t40** lands under g8 before the g9 exit gate, moving the three
tools' goldens as additive fields. The smoke store lives at
`tmp/todo-data` (fresh on clone; the config entry recreates it on
demand).

### 2026-10-04 — t38 landed: the wire suite's cross-cutting pins, U26 unreproduced

The session continued from t37's landing: the spec was pre-written at
`tmp/t38-spec.md` (8 decisions, 4 judgment calls, 5 required tests, 5
mandatory mutations, 11 gates, baseline shas pinned at `24153f7` and
verified to hold at `8ba153c`), and the implementer landed in one
round — a tests-only fifth target, `tests/wire.rs`, with the tree left
dirty for dual review. §3's entry carries the delivery.

What the suite proved, and how it was proven against:

- **The rapid-fire drive is real concurrency, not a shape.** The
  burst's arrival order measured genuinely out-of-order and different
  on every run (`["2","9","5","3","4","7","8","6"]`, then
  `["9","4","6","5","7","8","2","3"]`, …), and the race test observed
  **both** disjuncts across runs — the read before the write (pre-
  state) and after it (post-state) — so the id-matching rule is
  load-bearing and the disjunction is coverage, not vacuity.
- **The mutations measured, not trusted** (the seventh consecutive
  spec with kill-set claims, and the fourth with divergences —
  f95–f98): M1 killed 21 with five golden survivors
  (already-alphabetical DTOs — the three list envelopes plus the
  two edge echoes) and moved `Cargo.lock` (the `indexmap`
  line — the spec said it would not); M2/M3 killed the sweep plus 37
  and 36 elsewhere, with one test surviving M3 alone (it never
  compares the mirror); M4 killed the two concurrency tests as named
  — and exposed that the twice-run as spec'd survives a
  uniformly-broken server (two identical `locked` failures are still
  byte-identical), so the success assertion was added and M4's wire
  kill grew to 3.
- **U26 closed as unreproduced, honestly.** The committed 150-round
  hammer plus 3000 more hammer rounds, 172 protocol-target
  executions (sequential and under four-way parallel load) and four
  workspace runs, all clean against the flake's recorded ~3/83. No
  reproduction → no isolation variant, no fix (a fix without a
  reproduction would be unprovable); U26 stays open with the numbers,
  the hammer stays as the standing detector, and the flake remains
  t39's watch item.

Findings: **f95** (the spec's twice-run clock split miscounted
`repo_register` as clock-free — its `registered_at` is the clock's,
so the honest split is 17/8 with a third normalization needle),
**f96** (M1's "the lock does not move" is false — the `indexmap`
line goes with the feature), **f97** (the twice-run's vacuity under
uniform failure, closed by the success assertion), **f98** (M1's
"the goldens fail" overstates — five golden tests survive on
already-alphabetical DTOs). No production code moved; the seventeen
goldens, `common/mod.rs`, `tests/write.rs` and `tests/protocol.rs`
are byte-identical; `Cargo.toml`/`Cargo.lock` restored and
sha-verified after every mutation.

Handover: t38 is committed as `709fbdb` after a two-round dual
review — the fix round was docs-and-comment-only (f98's survivor
count corrected to five, the wire doc comments made count-free, the
U26 entry's machine named), and both reviewers re-bound ok on the
final bytes. The next step is **t39** — the
opencode smoke + U16b (the mcp entry in the generated config, the
native todo pair gated off, a real client session with the transcript
as evidence) — which is also U26's next watch point. The g9 exit gate
follows.

### 2026-10-04 — t37 landed: edges, registry-from-handles, the default slug

The session continued from t36's landing: the user said go, the spec
was pre-written at `tmp/t37-spec.md` (13 decisions, 14 required
tests, 6 mandatory mutations, baseline shas pinned at `3099515`),
and the implementer landed in one round. Dual review ran over two
rounds plus a binding round: round 1 primary ok (four lows) /
secondary changes_requested (three meds, five lows); the orchestrator
applied the combined fixes (the t11/t30/t35/t36 precedent), and both
reviewers re-bound ok on identical final-byte shas computed
independently at verdict time. §3's entry carries the delivery.

Three things the fix round discovered, worth more than the fixes:

- **The common test module cannot host write-only helpers.** The
  `reopened` lift's first attempt (into `common/mod.rs`, the obvious
  home) failed clippy: the read target compiles that module and sees
  a write-only helper dead. The honest home is the write target's
  own root (`tests/write.rs`), compiled by no other target — the
  pattern is now precedent in t38's note, and `drive_with` survives
  in common only because `drive` delegates to it internally.
- **The self-annotation pin corrected by the test itself.** The
  orchestrator's first pin asserted `relates_of(U1)` empty after the
  self-annotation — the domain stores it and lists it ("inert" means
  it gates nothing, not that it is absent), and the failing test
  said so. The honest pin (`relates_of(U1) == [U1]`) is what landed.
- **f68's flake is real.** Three recurrences in ~83 protocol-target
  runs during the secondary's review — `store is locked: repo
  camera` after the server exits, green on every immediate re-run.
  f68's designed panic captured it; **U26** opens riding t38's
  concurrency work with the two suspects named (the drive's
  `wait_with_output` reaping vs the flock outliving it; the
  in-process `repo_list()` one line earlier, t31's
  self-contention shape).

Findings: **f92/f93** (the sixth consecutive task spec with a
kill-set claim that did not reproduce — M1's named test 7 survives
the routing swap because the relates forbidden-pair message also
carries `invalid`; M2/M5's supersets), **f94** (EdgeDto's three
field docs keep the vacuous "canonical id spelling" phrase — true
given the round-trip, swept on the next touch, the third re-bind
round declined under the t15 stopping rule). **U23 closed** (the
swapped-store fixture pinning the resolve cross-check — the
primary's own seventh mutation proved the pin genuine). **U26
opened** (the flake). The t36 hand-off's common/mod.rs debts landed
(the `--repo` drive variant, the stale-claim sweep reworded
count-free).

**Handover.** Workspace: `main`, clean after the board commit, four
crates, **411** tests, rustc 1.98.1, MSRV 1.95; `Cargo.lock` 93
packages (unchanged). **Next step: dispatch t38** — the wire golden
suite + concurrency proof: D14's pin (losing
`serde_json/preserve_order` fails a golden, proven by mutation), the
D15 carrier conformance across every tool, concurrent dispatch
serialized per §8 (rmcp spawns each request; the server's state
stays consistent and out-of-order responses tolerated), twice-run
byte-identity. **Its spec is written** — `tmp/t38-spec.md`, this
session's last act (8 decisions, 5 required tests, 4+1 mandatory
mutations, the baseline pinned at `24153f7`): a **tests-only task**
(the t21/f19/f57 shape — no production code moves unless U26's fix
demands it, and a tests-only commit still carries both trailers)
whose deliverables are the carrier conformance sweep (twenty-four
tools, `repo_list`'s exclusion documented), the twice-run
byte-identity modulo the one honest clock (eighteen clock-free
responses byte-exact, seven stamped ones normalized), the
rapid-fire concurrency proof (responses matched by JSON-RPC id, the
write ids asserted as a set, the reopen proving consistency, the
write-then-read race asserted as a disjunction), D14's pin by the
feature-drop mutation over all seventeen goldens, and **U26's
reproduce-or-close hammer** (the flaky sequence at fifty rounds, the
two suspects varied independently, the disposition — fix or the
recorded numbers — stated by the report, not presumed). The surfaces
it inherits are all landed (twenty-five tools, seventeen goldens,
the carrier, the fixture). **t38's tracker note carries three
hand-offs**: U26
(the flake — the wire suite owns the drive's reaping semantics and
the two-threaded proof is the machinery that makes the window
reproducible or closes it), the clock-grep wording (name the
PRIMITIVE grep — a `time::now` grep hits every call site by
design), and the write-root helper precedent. **Blocked on you:
nothing.** Non-blocking and yours: U16 (t39's half), U21, U22, U18
(parked), U24/U25 (the two over-target splits, riding their next
owners), f94 (edges.rs's next touch), and the two one-line spec
asks — §10's gate enumeration and §6.3:474's feature condition
(f69). **Unverified:** no client has talked to this server; the
wire is measured by test harnesses only — the opencode smoke is
t39, the last task before the g9 exit gate. Tracker:
`tools/tasks.py check` exits 0 on the two expected lines (g8 open
with t38 next; g3 stands). Doc gates: check-links,
`attention.py --check`, `agents-sync.py --check` green;
agents-index untouched (no indexed doc moved in the code commit;
this board commit backfills the hash).

### 2026-10-04 — t36 landed: the node-CRUD mutation tools

The session continued from t35's landing: the user said ok to the
proposed dispatch, the spec was pre-written at `tmp/t36-spec.md`
(14 decisions, 12 required tests, 6 mandatory mutations, baseline
shas pinned at `17f2a68` — the sha tool named this time, the
secondary's t35 note), and the two design surfaces the t35 handover
named were settled there: **the clock question t33 handed forward**
(one `time::now()` per tool call, taken after the parse and the
resolve; the epic tools golden-pinned because epics carry no
timestamps — clock-freeness is the property that decides, not
readness; the stamped writers asserted in-test byte-exactly around
the extracted stamp, the t33 pattern generalized) and **the patch
DTOs' wire spelling** (mirroring the store's patches field-for-field
under RFC 7386's three-state, through a server-local `double_option`
copy). The implementer landed in one round; dual review ran over two
rounds, a binding round, and a commit-round confirmation — the
secondary's med (ReferenceParam lacking `deny_unknown_fields`, the
nested silent-drop) and both reviewers' lows were applied by the
orchestrator (the t11/t30/t35 precedent), and both re-bound ok on
identical final-byte shas computed independently at verdict time,
twice, with the production files byte-identical through the last two
rounds. §3's entry carries the delivery.

Findings: **f85** (the discovery that the closure discipline binds
on the *update* path — clearing `verify` through a patch is a domain
rejection answering `missing_verify`, so the three-state's clear
case for verify is pinned as the carrier, not a cleared field),
**f86–f88** (three more kill-set claims that did not reproduce —
M1's named test 10 unreachable, M3's add half unrepresentable, M6's
superset), **f89** (M2's "leaves updated untouched" did not kill
what it named — the fixture's `updated` equals its `created`, so a
stale stamp satisfied the suite; closed by hardening tests 4 and 7
with `assert_ne!` against the fixture's fixed instant). The fifth
consecutive task spec with a claim that did not reproduce — the
citation rule (pitfalls 5.6) is holding, and the spec-authoring
lesson is now simply: *every* named kill set and every "this fails
if" in a spec is a claim to measure. **U25** opened (`nodes.rs` 769
production lines, over target by recorded exception, the doc-family
split named). The f70 sweep ran before review this time (three
spec-local D-ids in the module-doc headers, caught by the
orchestrator's own gate check). And t37's note carries the
common/mod.rs hand-off: two stale claims in the shared test module's
doc ("The read suites' shared halves", "every read suite needs
exactly these three") are false now that the write suite imports six
of its helpers — the file was frozen by t36's baseline gate, so the
sweep rides t37, common's next owner.

**Handover.** Workspace: `main`, clean after the board commit, four
crates, **393** tests, rustc 1.98.1, MSRV 1.95; `Cargo.lock` 93
packages (unchanged). **Next step: dispatch t37** — the edges +
registry + config cluster: `task_link`/`task_unlink` (f4's named
criterion: an untyped pair `BlocksEdge::from_ends` rejects at the
wire), `repo_set_path`, `repo_list`/`repo_show` answered from the
handles the server holds (U20(b)), and the default-repo slug (the
committed D8: flag or env, the explicit parameter always wins). Its
spec wants pre-writing (the t32–t36 pattern); the design surfaces to
settle there: the default slug's mechanism (a `--repo` flag on
`main.rs` and/or an env var — §13/D8's per-project mcp entry is the
client-side home, and the params' `repo` fields relax
required→optional, backward-compatible), the edges' params shapes
(`task_link`'s untyped pair vs the three `BlocksEdge` shapes — f4's
criterion decides), and `repo_list`'s row shape (t31's `Repo`-node
answer, `RepoDto` now in `dto.rs`). The pieces it inherits are
landed: the write target and its helpers, the carrier, the golden
discipline with the clock-freeness principle, `resolve`'s handle
cache. **Blocked on you: nothing.** Non-blocking and yours: U16
(t39's half), U21, U22, U18 (parked), U23 (this task's or t38's
swapped-store fixture test — whoever owns the fixture machinery
first), U24/U25 (the two over-target splits, riding their next
owners), and the two one-line spec asks — §10's gate enumeration and
§6.3:474's feature condition (f69). **Unverified:** no client has
talked to this server; the wire is measured by test harnesses only —
the opencode smoke is t39. Tracker: `tools/tasks.py check` exits 1
on the two expected lines only (g8 open with t37 next; g3 stands —
an open goal is a structural exit, not a failure; corrected in
t38's backfill, the claim as first written did not reproduce). Doc
gates: check-links, `attention.py --check`, `agents-sync.py --check`
green; agents-index untouched (no indexed doc moved in the code
commit; this board commit backfills the hash).

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
