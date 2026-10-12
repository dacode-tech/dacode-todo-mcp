# Decision record — dacode-todo-mcp

The sign-off dossier for the spec's §12 open decisions. Each `D` entry
mirrors the §12 item of the same number. **Only the user signs off;
agents propose.** A resolved entry is struck in the summary, carries a
resolution banner, and its `On sign-off:` work order is executed in the
same change — most orders delete the §12 item, so the spec moves too.

`tools/attention.py` reads this file's summary table, its detail entries
and the meta block below; keep the shapes it parses.

<!-- attention-meta — machine-readable; read by tools/attention.py.
     D1–D12 resolved 2026-09-25/2026-09-29, D14 and D15 resolved 2026-09-30,
     D16 resolved 2026-10-03, D17 resolved 2026-10-04, D18 resolved 2026-10-05,
     D19 resolved 2026-10-08.
     D13 is reserved for the transport question board U18 defers, and is not
     drafted, so it has no action line either.
-->

## Sign-off summary

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| ~~**D1**~~ | Storage engine | (a) JSONL event log + in-memory graph, behind the Store trait | medium | M1 | resolved (a) 2026-09-25 |
| ~~**D2**~~ | Graph model and edge semantics | adopt §3/§5.1 as written | medium | M1 | resolved (a) 2026-09-25 |
| ~~**D3**~~ | Identity scheme | per-repo sequential integers per kind, never reused | medium | M1 | resolved (a) 2026-09-25 — repo keys are immutable slugs |
| ~~**D4**~~ | Store layout | one shared store for all repos under ~/.local/share/dacode-todo/ | medium | M2 | resolved (c) 2026-09-25 — user amendment: per-repo stores |
| ~~**D5**~~ | MCP tool surface | adopt the §6 list; ship todo-cli | medium | M3 | resolved (a) 2026-09-25 |
| ~~**D6**~~ | Selection tiebreaker | topology, then epic order, then id | medium | M2 | resolved (a) 2026-09-25 |
| ~~**D7**~~ | Importer scope | one-shot read-only import, camera dialect first | medium | M4 | resolved (b) 2026-09-25 |
| ~~**D8**~~ | Cross-repo ergonomics | defer cwd auto-detection to M5; id-keyed access in v1 | high | M4 | resolved (a) 2026-09-25 — plus default-repo slug config |
| ~~**D9**~~ | Status vocabulary and the deferred state | (a) open/in_work/blocked/deferred/closed + keep wontfix; until-date is metadata | medium | M1 | resolved (a) 2026-09-25 — until-targets ride blocks edges |
| ~~**D10**~~ | References, specs and decisions as server entities | (b) layered: references list; decision nodes that can block; registered docs, prose stays in git | medium | M1 | resolved (b) 2026-09-25 |
| ~~**D11**~~ | Next-items surface and derived weight | (a) next_items batch with blocks annotations + derived weight, display-only | medium | M2 | resolved (a) 2026-09-25 — plus agent-set priority field, display-only |
| ~~**D12**~~ | Importer fixture scope (amends D7) | dacode dialect first; camera and ikvm-html are live-test subjects only, never checked in here | high | M4 | resolved 2026-09-29 — user decision, reversing D7's order |
| ~~**D13**~~ | The default server's live view: the per-root listener | (a) startup ensure-listener + watchdog takeover; the publication per-root | medium | — | resolved (a) 2026-10-10 — user decision, in-thread at drafting; the implementation lands as t85 under g22 |
| ~~**D14**~~ | The wire's canonical JSON key order | (a) enable `serde_json/preserve_order` here, with a golden pinning the wire bytes | medium | M3 | resolved (a) 2026-09-30 — user decision, with the wire golden named as part of it |
| ~~**D15**~~ | Where a §6.1 code lives on the wire | (a) `structuredContent.code` via `structured_error`, §6.1 naming the carrier | medium | M3 | resolved (a) 2026-09-30 — user decision |
| ~~**D16**~~ | The progress roll-up: a tool, a field, or a client concern | (a) a `status_summary(repo?, epic?)` tool, plus `blocked_by` on `task_list` rows whichever option wins | medium | M2 | resolved (a) 2026-10-03 — signed after M2 closed at g7; the domain aggregation task lands under M3's goal |
| ~~**D17**~~ | Priority on the triage surfaces | (a+) echo `priority` on `next_task`'s answer and the `next_items`/`task_list` rows; the agent decides, the order untouched | medium | M3 | resolved 2026-10-04 — user decision, "let the agent decide" |
| ~~**D18**~~ | The dacode-dialect mapping (amends §9) | (a) A-deltas → tasks under a synthetic epic; sub-steps → gate tasks; docs → doc nodes; ids reproduced by import order | medium | M4 | resolved (a) 2026-10-05 — user decision, "let's do option (a)" |
| ~~**D19**~~ | The lock-free read path and §4.3's torn-tail reading | (a) a read-only open: no lock, committed prefix, unterminated tail = append in flight | medium | — | resolved (a) 2026-10-08 — user decision, in-thread at drafting; the implementation lands as two tasks |
| ~~**D20**~~ | The all-in-one binary: the MCP server gains the tool CLI | (a) one binary — no subcommand stays the server; progress/refresh/serve + the todo-cli absorption as subcommands | high | — | resolved (a) 2026-10-09 — user decision, in-thread at drafting; the implementation lands as the g19 arc (t68–t73) |
| ~~**D21**~~ | The output schemas' error union (amends D15) | (a) every declared outputSchema becomes anyOf of the success payload and D15's carrier | high | — | resolved (a) 2026-10-12 — user decision, in-thread; the implementation lands as U51's fix the same day |
| **D22** | The reward model: sizes, arithmetic, triggers, opt-in | (a) two size buckets (planned at creation, actual at closure), server-computed integer rewards, per-root config, milestone + accumulation triggers, strictly opt-in | high | E13 | open — drafted 2026-10-13 from the user's in-thread steering |
| **D23** | The storage move: the reward events and FORMAT_VERSION 2 | (a) two new event kinds plus the estimate on creation events; FORMAT_VERSION 1→2, in-place header migration, v1 replaying identically | high | E13 | open — drafted 2026-10-13 |
| **D24** | The wire move: the surface extension | (a) the optional estimate on task_add, the optional actual size on closure, the reward line and reflection question on the closure response, one new tool (reward_reflect), the base-instructions rule | high | E13 | open — drafted 2026-10-13 |

---

## Details

### D1 — Storage engine

**Resolved 2026-09-25 — option (a), signed by the user.** The user's
stated lean matched the recommendation: tiny dataset, traversals not
joins, human-auditable log, redb one crate away. Work order executed in
the sign-off commit: §12 item 1 deleted, §4/§4.1 made normative. A
fallback to (b) remains available only as a new decision brought to the
user — never a silent swap.

**Concretization approved 2026-09-26 (board U5).** The user ok'd the
crash/recovery story drafted in the M1 decomposition, landed as the new
§4.3: single-`write()` `O_APPEND` line appends with `fsync` per event
(the torn window is one in-flight event); a torn *last* line is a named
`corrupt_store` error carrying its byte offset, never silently skipped;
a malformed interior line is `corrupt_store` unconditionally; repair is
explicit (show, then truncate on confirmation, losing exactly that one
event); snapshots are atomic (temp+fsync+rename) and are the recovery
unit — replay = latest snapshot + log tail; the registry follows the
same rename discipline; v1 backup = copy the store directory under no
writer lock. Board U5 struck. The user declined to open a redb/D12
decision: plain files stand, and the fallback stays reachable only via
a new decision with measurements.

**Gates** M1 · **Resolved** 2026-09-25

**Current.** Spec §4 lists the requirements (local, single-file,
non-relational, deterministic bytes, versioned, corruption-safe,
human-auditable preferred) and §4.1 the candidates. Nothing is built, so
the choice is still free.

**Options.**
(a) Append-only JSONL event log + in-memory graph + periodic snapshots.
(b) Embedded KV store (redb) with adjacency encoded in key prefixes.
(c) SQLite — excluded by the user's non-relational constraint.
(d) A dedicated graph database — excluded by weight (§4.1).

**Recommendation.** (a). The dataset is tiny (10^3–10^5 nodes), every
query is a traversal rather than a join, the log stays human-readable
and diffable — which preserves the audit-trail property the markdown
boards had — and replay determinism is trivially testable. The `Store`
trait (§4.2) keeps (b) one crate away if snapshotting or replay proves
awkward in M1.

**Confidence: medium.** The snapshot/compaction component is the part
with real design surface; if M1 shows it dominating, fall back to (b)
before hardening it.

**On sign-off:** delete §12 item 1; strike the PROPOSED qualifier on
§4.1's lean and make the chosen engine normative in §4; update
`.agents/roles/_rust.md`'s determinism section if the engine constrains
it; the M1 row of TODO.md §2 unblocks.

### D2 — Graph model and edge semantics

**Resolved 2026-09-25 — option (a), signed by the user.** §3/§5.1
adopted as written; epic-level `blocks` noted in §5.1 as a follow-up
decision. In the same message and a follow-up, the user directed a
status-vocabulary amendment and server-resident specs/decisions —
recorded separately as D9/D10, which extend parts of this resolution
when signed, rather than folded into it silently. Work order executed:
§12 item 2 deleted, PROPOSED struck on §3 and §5.1.

**Gates** M1 · **Resolved** 2026-09-25

**Current.** §3 proposes three node kinds (repo, epic, task), four task
kinds (task, gate, finding, debt), and §5.1 three edge kinds (parent
tree, blocks DAG, relates annotation) with write-time cycle rejection
and the no-orphans rule.

**Options.**
(a) Adopt §3/§5.1 as written.
(b) Adopt with epics also allowed in `blocks` edges (epic-level gating).
(c) Flatten: drop task kinds, model gates/findings/debt as tags.

**Recommendation.** (a), and treat (b) as a follow-up decision when a
real query needs it — epic-level `blocks` complicates the topological
derivation and nothing in the three sibling boards demands it yet. (c)
loses the closure-discipline semantics the sibling trackers proved
useful (a `gate` that must pass is not a tag).

**Confidence: medium.** The vocabulary is inherited from tools that have
run for months on three repos; the risk is in edges, not nodes.

**On sign-off:** delete §12 item 2; strike the PROPOSED qualifier on §3
and §5.1; TODO.md §2 unblocks the M1 model rows.

### D3 — Identity scheme

**Resolved 2026-09-25 — option (a) with repo slugs, signed by the
user.** Per-repo sequential integers per kind for epics and tasks,
server-allocated, never reused, never renumbered. Repos are keyed by an
immutable slug — the user's own follow-up ("mcp per project can be
configured to use specific repo/project slug as default") confirms
slugs over integers for repo keys. Work order executed: §12 item 3
deleted, §3's identity paragraph and §7's registry key made normative,
never-reuse recorded in §3.

**Gates** M1 · **Resolved** 2026-09-25

**Current.** §7 requires stable ids (a repo's id survives a path
change); §12 item 3 asks which scheme.

**Options.**
(a) Per-repo sequential integers per kind (`camera:U7`-style, echoing
the boards' `U` ids), allocated by the server, never reused, gaps
meaningful.
(b) ULIDs — sortable, collision-free without coordination, opaque to
humans.
(c) Typed slugs (`camera/import-board-fixtures`) — readable, but rename
becomes an identity event.

**Recommendation.** (a) for tasks/epics within a repo, with a small
server-level integer or slug for repos themselves. The sibling projects'
whole culture is "cite the id in prose" (`U46`, `D2`, `t87`); short
per-repo integers are the only scheme a human will type into a chat
message, and the importer (D7) can preserve the boards' existing
numbers, making migration auditable item-by-item. Never-reuse and
never-renumber are part of the guarantee.

**Confidence: medium.** Sequential allocation needs the single-writer
lock (§8) — which the design already assumes.

**On sign-off:** delete §12 item 3; make §3's id fields and §7's
registry key normative; record the never-reuse rule in §3.

### D4 — Store layout

**Resolved 2026-09-25 — option (c), a user amendment over the
recommendation.** User direction: keep per-repo/project store files
rather than one database shared across projects. Layout: under
`~/.local/share/dacode-todo/`, one directory per repository
(`repos/<slug>/` — event log + snapshots) plus a shared registry file
holding repo metadata and the slug→store mapping. No task data lives in
a cross-project shared store. Epics and tasks remain nodes inside their
repo's store — per-epic files would fragment single-writer atomicity
for cross-epic edges (interpretation flagged to the user in the
sign-off reply; a veto reopens this entry). Accepted trade-off:
cross-repo queries iterate the registry and open per-repo stores
instead of reading one file. Work order executed: §12 item 4 deleted,
§2's data-location paragraph made normative, §8 lock scope noted.

**Gates** M2 · **Resolved** 2026-09-25

**Current.** §2 proposes one shared store under
`~/.local/share/dacode-todo/`; §12 item 4 asks shared vs per-repo.

**Options.**
(a) One store file for all repos (the §2 proposal).
(b) One file per repo plus a registry index that maps repo ids to files.
(c) One directory per repo (log + snapshots per repo) under a shared
registry.

**Recommendation.** (a) if D1 is the event log: one lock, one replay,
cross-repo queries (§12 item 8) are free, and the audit trail is one
file. Revisit only if a single repo's log grows past what replay should
handle — snapshots bound that, and (c) is the migration path if they
ever do not.

**Confidence: medium.** Depends on D1; sign D1 first.

**On sign-off:** delete §12 item 4; make §2's data-location paragraph
normative; note the lock scope in §8.

### D5 — MCP tool surface

**Resolved 2026-09-25 — option (a), signed by the user.** The §6 list
is the v1 surface; `todo-cli` ships as the fourth workspace member.
Note: D10, if signed, extends this list (decision/doc tools) by its own
work order — an amendment by decision, not by accretion. Work order
executed: §12 item 5 deleted, PROPOSED struck on §6 and §2's CLI row,
tool list folded into TODO.md's M3 exit criteria.

**Gates** M3 · **Resolved** 2026-09-25

**Current.** §6 proposes the tool list, the principles (repo named by
every tool, epic as filter, total orders, structured errors, writes
return new state, server-enforced closure discipline) and §6.1 the error
codes.

**Options.**
(a) Adopt the §6 list as the v1 surface; ship `todo-cli` as a fourth
workspace member.
(b) Adopt the list, defer `todo-cli` (the MCP client is the only
consumer in v1).
(c) Amend the list (name it in the sign-off).

**Recommendation.** (a). The CLI costs one thin binary over the same
domain API and pays for itself the first time a store needs inspecting
without a client — including during M1/M2, before the server exists.

**Confidence: medium.** Schemas will firm up against rmcp's macros
during M3; names and semantics are what this signs, not JSON shapes.

**On sign-off:** delete §12 item 5; strike PROPOSED on §6 and §2's CLI
row; add the tool list to the M3 exit criteria in TODO.md §2.

### D6 — Selection tiebreaker

**Resolved 2026-09-25 — option (a), signed by the user.** Topology,
then epic registration order, then task id. The user's follow-up ask
for "weights/priorities" is drafted as D11: derived, display-only
weight (transitive dependent count) that annotates without reordering —
ordering itself is untouched by this signature, and a hand-set priority
field would be a new decision amending this one. Work order executed:
§12 item 6 deleted, the tiebreaker written into §5.2 as normative, to
be pinned by §10's determinism tests.

**Gates** M2 · **Resolved** 2026-09-25

**Current.** §5.2 requires a total order behind `next_task`; topology
alone leaves ties.

**Options.**
(a) Topology, then epic registration order, then task id (creation
order) — everything derived from existing fields, no new state.
(b) Topology, then an explicit `priority` field users must maintain.
(c) Topology, then id alone (ignoring epic grouping in ordering).

**Recommendation.** (a). It is a pure function of state the server
already holds, it matches how the sibling boards read (milestone/epic
order first, `U`-id order inside), and it adds no field for anyone to
forget. (b) reintroduces the hand-maintained priority the projects
deliberately derive instead; (c) interleaves epics, which defeats the
per-epic selection the user asked for.

**Confidence: medium.** If a priority field is ever wanted, it is a new
decision, not an amendment by accretion (spec §11).

**On sign-off:** delete §12 item 6; write the chosen order into §5.2 as
normative; pin it with the determinism tests of §10.

### D7 — Importer scope

**Resolved 2026-09-25 — option (b), signed by the user.** One-shot
read-only import, camera dialect first; ikvm-html and dacode follow as
mechanical extensions once the camera pipeline and fixtures prove the
mapping. (c) (live sync) stays excluded: two sources of truth is the
disease, not a feature. Work order executed: §12 item 7 deleted, §9
made normative for the chosen scope, fixture plan folded into TODO.md's
M4 deliverable.

**Amended 2026-09-29 by D12.** The camera-first *order* is reversed to
dacode-first, and camera/ikvm-html become live-test subjects whose board
content is never checked into this repository. The one-shot, read-only,
no-live-sync scope of D7 is unchanged. See D12.

**Gates** M4 · **Resolved** 2026-09-25 · **Amended** 2026-09-29 (D12)

**Current.** §9 proposes one-shot, read-only import of the sibling
boards with a fixed mapping (milestones→epics, U items→tasks, open
decisions→gates, mechanical dependency prose→edges), and no live sync.

**Options.**
(a) As §9: one-shot import, all three dialects in v1, verified against
checked-in fixtures (§10).
(b) One-shot import, camera dialect first (it is the newest harness and
closest to this spec's vocabulary), the other two as follow-ups.
(c) Bidirectional sync with the markdown boards.

**Recommendation.** (b). One dialect end-to-end proves the mapping and
the fixture pipeline; the other two are then mechanical. (c) is
excluded: two sources of truth is the disease, not a feature.

**Confidence: medium.** The dialects differ mostly in registers
(`B`/`A` vs `D`), which the mapping table absorbs.

**On sign-off:** delete §12 item 7; make §9 normative for the chosen
scope; add the fixture-plan row to the M4 block in TODO.md §2.

### D8 — Cross-repo ergonomics

**Resolved 2026-09-25 — option (a) plus a user-directed amendment.**
No cwd auto-detection in v1 (deferred to M5); id-keyed access is the
normal case. Amendment from the sign-off message: a server instance may
be configured with a **default repo slug** (flag or env in the client's
per-project mcp entry); an explicit repo parameter always wins. This is
deterministic configuration, not a detection heuristic, so (a)'s
rationale is untouched. Work order executed: §12 item 8 deleted, the
deferral recorded in §7 and TODO.md's M5 row, the default-slug
mechanism added to §6's principles and §13's wiring.

**Gates** M4 · **Resolved** 2026-09-25

**Current.** §7 makes id-keyed cross-repo access the normal case (any
client can query any repo). What is open is the convenience layer:
auto-detecting "my repo" from the client's cwd, and whether that ships
in v1. The user marked this optional for now.

**Options.**
(a) v1: explicit repo naming only; cwd auto-detection deferred to M5.
(b) v1 includes cwd auto-detection (a `repo_detect` tool reading the
client's working directory from the MCP roots capability or an env
hint).

**Recommendation.** (a). Explicit naming is deterministic — the property
the project exists for — and detection heuristics (which root? nested
repos? worktrees?) are exactly where non-determinism creeps in. Stable-id
keying (D3) already makes cross-repo access work; the sugar can come
later without a migration.

**Confidence: high.** Deferring is reversible at zero cost; shipping
detection early is not.

**On sign-off:** delete §12 item 8; record the deferral in §7 and in
the M5 row of TODO.md §2; if (b) is chosen instead, add a `repo_detect` row
to §6 and a determinism rule for its resolution order.

### D9 — Status vocabulary and the deferred state

**Resolved 2026-09-25 — option (a) with the user's `until`
refinement.** Statuses: `open`, `in_work`, `blocked`, `deferred`,
`closed`, `wontfix`. User refinement: a deferred item is "mostly
blocked by some other tasks", so until-targets are primarily graph
references — recorded as `blocks` edges from the target to the
deferred task (cycle-checked at write, visible to every dependency
query), while the optional `until` date stays pure metadata. No
auto-reactivation on either: selection reads no calendar (§8) and no
mutation silently flips a status; when an until-target closes, queries
surface the item as until-satisfied and reopening is an explicit
`task_update`. Closure discipline: closed⇒evidence, blocked⇒note,
deferred⇒note. Work order executed: §3 statuses/closure rewritten,
§5.2 actionability amended, §12 item 9 deleted, TODO.md M1 rows.

**Amended 2026-09-25 (same day, user-directed).** Deferred items must
be surfaced, not merely queryable: `next_task`/`next_items` responses
carry a deferred summary for the queried scope — `deferred: {total,
ready}`, where `ready` counts deferred items whose until-target
`blocks` predecessors are all finished — and `task_list(status=deferred)`
gains a ready filter for when the agent decides to look. The summary
informs; auto-reactivation remains excluded. Landed in §5.2/§6 in the
amendment commit.

**Gates** M1 · **Resolved** 2026-09-25

**Current.** §3 specifies `pending`, `active`, `blocked`, `done`,
`wontfix` — the exact `tools/tasks.py` set — with the closure
discipline (done⇒evidence, blocked⇒note). The user's sign-off message
of 2026-09-25 directs a revised set: open, in-work, blocked, deferred
(optionally with an until-when), closed — with blocked items carrying
references and/or being determined from the graph. Two clarifications:
graph-derived blocking already exists (§5.2 — a task with an unfinished
`blocks` predecessor is not actionable whatever its status), and D2 was
signed in the same message adopting §3 as written; this entry is the
user's explicit amendment of the status vocabulary only — node kinds
and edges are untouched.

**Options.**
(a) Six statuses: `open`, `in_work`, `blocked`, `deferred`, `closed`,
`wontfix` — renames pending→open, active→in_work, done→closed; adds
deferred; keeps wontfix. Closure discipline maps one-to-one:
closed⇒evidence, blocked⇒note, deferred⇒note, plus an optional `until`
date (RFC 3339) that is metadata only.
(b) The five directed statuses exactly, folding wontfix into closed.
(c) Keep the tasks.py set; model deferred as blocked with a dated note.

**Recommendation.** (a). The renames are cosmetic and match the user's
vocabulary; deferred deserves first-class status because "deliberately
postponed" is a different fact than "externally stuck". The `until`
date stays metadata — selection remains wall-clock-free (§8); an item
leaves deferred by an explicit status change, never by the calendar.
Calendar-driven reactivation would need a client-passed as-of timestamp
in selection (a §5.2 amendment) — possible and still deterministic, but
a new decision, not folded in here. Keeping `wontfix` preserves the one
terminal that unblocks successors without evidence of completed work
(§5.1), and the §9 importer mapping for struck-through board items; (b)
either forces evidence onto deliberately-skipped work or punches a hole
in the closure discipline; (c) conflates the two blocked flavors.

**Confidence: medium.** Renames are zero-risk; deferred interacts with
§5.2 actionability and §6.1 (`missing_note` already covers
deferred-without-note).

**On sign-off:** rewrite §3's status sentence and closure requirements;
amend §5.2's actionability clause (deferred joins blocked as
non-actionable); delete §12 item 9; the M1 rows of TODO.md §2.

### D10 — References, specs and design decisions as server entities

**Resolved 2026-09-25 — option (b), signed by the user.** References
lists on epics/tasks/decisions/docs; `decision` nodes carrying full
dossiers whose open state can `block` tasks and epics; `doc`
registration for stable addresses with prose bodies staying in git.
This resolution extends signed D2 (two new node kinds) and rewrites
§11's "dossiers stay prose" non-goal — both moves are part of this
sign-off, recorded here rather than folded into D2 silently. The
full-content store (option (c)) is declined for v1; doc registration
is the hook to add server-side retrieval later without migration.
Work order executed: §3 node kinds/identity/references, §5.1 edge
extensions, §6 tool rows, §9 mapping amended (open D/B → decision
nodes), §11 rewritten, §12 item 10 deleted.

**Concretization approved 2026-09-25.** The user ok'd the concrete
shapes proposed in the sign-off thread: per-kind id prefixes (task
`U`, epic `E`, decision `D`, doc `S`); decision transitions enforced
like task closure (`signed`/`superseded` require `resolution`, else
the new §6.1 code `missing_resolution`); signing flips the status and
the decision's `blocks` edges simply stop gating — no edge mutation;
milestone gating is a `blocks` edge to an epic, not a field;
`doc_set_path` audited like `repo_set_path`; registered-doc citations
take an optional `#anchor`. Landed in §3/§6/§6.1 (board U6 struck).

**Gates** M1 · **Resolved** 2026-09-25

**Current.** Nodes carry no external references; specs and decisions
are git prose that this harness regex-parses (decisions.md +
attention.py) — the strain the user wants gone, visible in the harness
itself. The sign-off message asks for: epics that are JIRA issues
(several links to external trackers), tasks referencing spec paragraphs
or work items, and reviewable design docs that are non-actionable but
citable; a follow-up message directs that **specs and design decisions
should also live in the MCP server so they can be reliably referenced**.
This overrides part of §11 ("D/B dossiers stay prose; the server tracks
the gate they hang on") and extends signed D2's node kinds — recorded
here rather than folded into D2's resolution silently.

**Options.**
(a) Pointers only: a `references` list on epics and tasks
({kind, target, note?}, kind ∈ tracker/url/doc/spec/file/other); docs
and decisions stay unregistered git prose.
(b) Layered: the references list for externals (tracker keys, URLs,
files), plus two new node kinds —
  * `decision`: a structured dossier in the store {id, repo, title,
    status open/signed/superseded, options, recommendation, resolution,
    confidence, timestamps}. An open decision can carry `blocks` edges
    to the tasks/epics it gates, so "never write past an unsigned
    decision" becomes graph-enforced and `next_task` can never surface
    work behind an open decision.
  * `doc`: a registered document {id, repo, title, path, version} — a
    stable address with a mutable path (the same trick as repo slugs vs
    paths, §7). The prose body stays in git, where review, diff, blame
    and backup live.
(c) Full content store: spec-section bodies also live in the server and
are edited through tools; the git markdown becomes a rendering/export.

**Recommendation.** (b). Decisions are structured, lifecycle-driven and
gating — exactly what a deterministic queryable store beats prose at;
this repo currently regex-parses its own decision dossiers to render
the attention table, which is the markdown-board disease reappearing in
the harness. Spec *bodies* belong in git: merge review, diff and backup
are git's determinism, and duplicating long prose into the store
creates a second source of truth with none of those properties.
Registering docs makes citations survive file moves and resolve
cross-repo, which is what "reliably referenced" needs. (c) stays
revisitable without migration: doc registration is the natural hook for
a content cache or retrieval tool if cross-repo text access proves
necessary. (a) is now below the user's direction: file+section cites
rot and are unreachable from other repos.

**Confidence: medium.** The decision-node schema has real design
surface: dossier fields, supersession semantics, and the interplay with
the signed §9 mapping — open D/B decisions currently map to `gate`
tasks; under (b) they become decision nodes instead, and `gate` remains
for non-decision checkpoints.

**On sign-off:** add the references field and the decision/doc node
kinds to §3; extend §5.1's `blocks` to decision→task/epic edges; add
decision/doc tool rows to §6 (extending signed D5's list by this
decision; schemas ride M3); amend §9's mapping (open D/B → decision
nodes); narrow §11's "dossiers stay prose" clause to spec bodies;
delete §12 item 10; the M1 rows of TODO.md §2.

### D11 — Next-items surface and derived weight

**Resolved 2026-09-25 — option (a) with the user's priority
amendment.** `next_items(repo, epic?, limit?)` returns the top-N
actionable nodes in D6's canonical order (default limit 10), each
annotated with its direct blocked-by/blocks ids and derived weight =
transitive dependent count; `next_task` remains the limit=1 case.
Amendment: an optional `priority` field on tasks, set by agents
through `task_add`/`task_update` only, echoed in item payloads — no
priority parameter on any query, and priority never enters the
ordering, so signed D6 stands untouched. This is §11's "priority may
come later by decision" being decided: as triage metadata, not an
ordering input. Promotion into the tiebreaker would be a new decision
amending D6 (flagged to the user at sign-off). Work order executed:
§5.2 batch selection, §6 `next_items` row, §11 amended, §12 item 11
deleted, TODO.md M2/M3 rows.

**Amended 2026-09-25 (same day, user-directed).** Two extensions:
`next_items` also takes a `task` scope — the transitive dependents of
a non-top-level item, in the same canonical order, each marked
immediately actionable or still held by other predecessors — and
`graph_show` is specified as the complete-graph handover function
(new §6.2): canonical-order nodes with per-node `blocked_by`/`blocks`
adjacency, wire format canonical JSON, any YAML/Mermaid render a pure
function of that structure behind a `format` parameter. Ordering
remains server-side (`todo-domain`) and D6-pure; `priority` remains
display-only.

**Gates** M2 · **Resolved** 2026-09-25

**Current.** §5.2/§6 specify `next_task` returning the single first
actionable node; D6(a) is signed (topology → epic order → id). The
user's sign-off message asks for a way to query the **next open items**
(plural) with references to the items they block, and "some
weights/priorities". §11 excludes priority/effort fields in v1 — "by
decision, not by accretion" — and D6's dossier says the same.

**Options.**
(a) `next_items(repo, epic?, limit?)`: the top-N actionable nodes in
D6's canonical order, each annotated with its direct blocked-by/blocks
ids and a derived weight = count of transitive dependents. `next_task`
remains the limit=1 case. Weight is display-only: a pure function of
the graph, no new state, no reordering.
(b) (a) plus a hand-set `priority` field participating in the
tiebreaker — amends the just-signed D6 and repeals §11's exclusion.
(c) Keep the single `next_task`, no annotations.

**Recommendation.** (a). It answers the agent's real question — a
ranked batch with dependency context, "what unblocks the most" — while
keeping the order a pure derivation: weight is computed, never entered,
so there is nothing to maintain and nothing to drift. If hand-set
priority that reorders selection is ever wanted, that is an explicit
amendment of D6 (a new decision), which §11 already anticipates.

**Confidence: medium.** The annotations are cheap traversals; the only
open sub-question is limit's default (proposed: 10).

**On sign-off:** amend §5.2 (batch selection) and §6's tool table (a
`next_items` row); delete §12 item 11; the M2/M3 rows of TODO.md §2.

---

### D12 — Importer fixture scope (amends D7)

**Resolved 2026-09-29 — user decision.** The **dacode** dialect is the
importer's first target and the only sibling board checked into this
repository as a fixture. `camera` and `ikvm-html` become **live-test
subjects after conversion**: the server is pointed at them in place, and
no content from either is copied into this repo's tree.

D7's *scope* is unchanged — one-shot, read-only, no live two-way sync.
Only its *order* and its fixture plan move.

**Gates** M4 · **Resolved** 2026-09-29

**Current (before this decision).** D7 signed camera-first, on the
reasoning that camera is the newest harness and closest to this spec's
vocabulary. §10 planned fixtures as "the real committed `TODO.md`
snapshots of the **three** sibling repos". §6's `import_md` row and §14's
M4 row both name camera first.

**Why the user reversed it.** Two reasons, and the second is a constraint
rather than a preference:

1. **`camera` and `dacode` are both actively developed**, so a snapshot
   of either goes stale; but `dacode`'s board has the **deeper structure
   and the more complex references**, which is the hard half of §9's
   mapping — D10's `references` entries, `#anchor` doc citations,
   decision→work `blocks` edges. `camera` has the more recent *manners*
   (registers, session-log shape), which the mapping table absorbs
   cheaply. Stressing the importer on the hard axis first is worth more
   than stressing it on the familiar one.
2. **No traces of `camera` or `ikvm-html` in this repository.** They are
   other products; their board content does not belong committed here even
   as a test fixture. `dacode` is this project's own family, so its board
   can be checked in. That makes the live-test route the only available
   one for the other two — and it is a better test anyway, since it runs
   the importer against a real, current board rather than a snapshot.

**Consequence, stated plainly rather than hidden.** §10's "verified
against checked-in fixtures" now holds for one dialect, not three. The
camera and ikvm-html dialects get **no pinned golden**: their correctness
is argued by a live run a human inspects, which is weaker and
non-reproducible — a live board changes, so the same importer run twice
can legitimately differ. If that proves too weak at M4, the remedy is a
*synthetic* fixture in camera's dialect (hand-written in this repo, no
copied content), not reversing this decision. That trade is accepted
here.

**On sign-off:** amend §9's dialect/order parenthetical; §10's
"Importer fixtures" bullet (one sibling snapshot, plus the synthetic-
fixture remedy); §6's `import_md` row; §14's M4 row; D7's banner above;
TODO.md §2's M4 row, §4's U3, §3 and §6.

---

### D14 — The wire's canonical JSON key order

**Resolved 2026-09-30 — option (a), signed by the user.** This workspace
enables `serde_json/preserve_order`, so a struct's declaration order
reaches the wire and §6.2's "fixed field order" holds as written. The
recommendation's second half is signed with it: §10's protocol-level tests
pin the response **bytes** as goldens, because the feature is a determinism
dependency that a build losing it would not fail on its own — it would
re-sort every key and still compile. Work order executed in the same
change: §6.2 states the chosen order and its pin, §8 names both
serializers and calls the feature a determinism dependency, §10's
protocol-level bullet gains the wire golden, §6.3's finding records the
outcome, §12 item 14 deleted, this row struck.

**Gates** M3 · **Resolved** 2026-09-30 · drafted from U2's rmcp survey
(spec §6.3) · **Confidence: medium**

**Current (before this decision).** §6.2 signs `graph_show`'s output as
"canonical JSON (§8: fixed field order, sorted keys, golden-pinned)", and
§8's argument is that one canonical serializer is one determinism surface.
The store's codec does exactly that — `dacode-json` emits struct fields in
declaration order and map keys sorted, and five goldens pin the bytes.

**What the survey measured.** A tool's structured output does not go
through that codec. rmcp serializes `Json<T>` with `serde_json::to_value`
(`handler/server/wrapper/json.rs`) into `structuredContent`, and it depends
on `serde_json` with default features, where `Map` is a `BTreeMap`. A probe
struct declaring `zebra, middle, alpha` reached the client as
`{"alpha":…,"middle":…,"zebra":…}`; the identical probe built with
`serde_json/preserve_order` reached it as `{"zebra":…,"middle":…,"alpha":…}`.
Arrays are unaffected — five deliberately unsorted rows came back unsorted
— so §5.2's total order, which is an order over a *list*, survives either
way. What does not survive by default is field order: half of §8's
canonical form.

**Options.**

- **(a) Enable `serde_json/preserve_order` in this workspace.** Declaration
  order reaches the wire and §6.2 holds as written. Cost: the wire bytes
  then depend on a cargo feature unified across the whole dependency tree,
  and nothing in our own code states that dependency — a build which lost
  the flag would re-sort every key, still compile, and differ only in
  bytes. `serde_json` also becomes a direct workspace dependency (rmcp
  brings it either way).
- **(b) Carry the dump as a text content block** produced by our own codec.
  Exact bytes, one serializer, no flag, and the wire golden is the store's
  own shape. Cost: no `structuredContent`, so a client that wants to branch
  on a field parses a JSON string out of a text block — and §6.2's "MCP
  results are JSON anyway" argument is spent on a string inside JSON.
- **(c) Accept sorted keys on the wire** and amend §6.2/§8 to name two
  canonical orders: the store's (declaration order, golden-pinned) and the
  wire's (sorted keys). No flag, still deterministic, and "sorted keys" is
  already half of §8's sentence. Cost: one project with two canonical
  orders, and a wire golden that differs from a store golden for the same
  data.

**Recommendation: (a)** — and whichever option is signed, a golden over the
wire bytes. The flag is only safe if losing it fails a test instead of
silently re-sorting, and M3's protocol-level tests (§10) are that golden's
natural home; naming the pin is the part of this recommendation that does
not depend on which option wins.

**On sign-off:** amend §6.2's wire-format sentence to state the chosen key
order and where it is pinned; §8 names the serializer the wire uses; §10's
server bullet gains the wire golden; delete §12 item 14; strike this row;
TODO.md §2's M3 row, §3 and §6.

---

### D15 — Where a §6.1 code lives on the wire

**Resolved 2026-09-30 — option (a), signed by the user.** A §6.1 code
travels as `structuredContent.code` on a tool-level error result
(`isError: true`, the same JSON mirrored into a text block so a model reads
the message too), each tool declares its `outputSchema` explicitly, and an
error payload is **not** required to conform to it — the schema describes a
success. Protocol-level JSON-RPC errors stay reserved for what the protocol
itself rejects, since clients render those opaquely. Work order executed in
the same change: §6.1 gained the carrier paragraph and the conformance
exemption (plus the two consequences for M3 — `invalid` covers values this
server parses, not argument shapes; params structs deny unknown fields),
§6's error principle points at §6.3's table, §6.3's finding records the
outcome, §12 item 15 deleted, this row struck.

**Gates** M3 · **Resolved** 2026-09-30 · drafted from U2's rmcp survey
(spec §6.3) · **Confidence: medium**

**Current (before this decision).** §6.1 signs twelve stable codes and one
rule about them: "a client may branch on a code; it must never parse a
message." The spec does not say where on the wire the code sits, because
until U2 there was no measured answer.

**What the survey measured.** Five carriers exist (spec §6.3's table), and
they differ in what a client can actually do with them:

- `Err(String)` and `CallToolResult::error(text)` both produce
  `{"isError":true,"content":[{"type":"text","text":"…"}]}` — a message and
  nothing else. The first is rmcp's default tool-error path, so the obvious
  implementation of §6.1 would ship exactly the bare strings §6 forbids.
- `CallToolResult::structured_error(json!({"code":…}))` produces
  `isError:true` **and** a `structuredContent` carrying our code, with the
  same JSON mirrored into a text block, so a model reads it too.
- Adding `output_schema = schema_for_type::<T>()` keeps a declared
  `outputSchema` in `tools/list`, which returning `Result<CallToolResult, _>`
  otherwise loses.
- A tagged-union output type makes the error payload *conform* to the
  declared schema, but it is returned as `Ok`, so the wire says
  `"isError":false`.
- `Err(ErrorData)` produces a JSON-RPC error object with an i32 `code` and
  an optional `data` payload. Branchable by a JSON-RPC-aware client, but
  rmcp documents that clients "typically render protocol errors opaquely
  (e.g. *Tool result missing due to internal error*) — the caller does
  **not** see your message" (`model.rs:4100-4105`), so the model never sees
  the code. Protocol-level carriers are therefore out for §6.1's purpose,
  and remain right for what rmcp itself uses them for: an unknown tool, a
  malformed request.

**Options.**

- **(a) `structuredContent.code`, with an explicitly declared
  `outputSchema`.** One error struct shared by every tool
  (`{code, message, …}`), `isError:true`, branchable, and the tool keeps its
  schema. Cost: the error payload does not conform to that schema — rmcp
  does not validate ("since rust is a strong type language, we don't need to
  do json schema validation here", `model.rs:4201-4204`), but a strict
  client could reject the result.
- **(b) A tagged-union output type per tool.** Everything conforms to the
  declared schema. Cost: failures carry `isError:false`, which is the flag
  clients and models read first, and one union type per tool instead of one
  shared error struct.
- **(c) The code in `_meta`, message as content.** Schema untouched,
  `isError:true`, branchable by a client that looks there. Cost: `_meta` is
  protocol furniture — rmcp uses it for trace context and server identity —
  and a client may strip or ignore it, so this is the least guaranteed of
  the three.

**Recommendation: (a)**, with §6.1 amended to name the carrier
(`structuredContent.code`) and to state that an error payload is not
required to conform to the tool's `outputSchema`. (b) is rejected because a
contract whose failures advertise success is worse than one whose error
payload is off-schema, and `isError` is what a client branches on first;
(c) is rejected because it puts a contract field in the one place the
protocol does not promise to deliver.

**On sign-off:** amend §6.1 with the carrier sentence and the
schema-conformance exemption; §6's error principle ("Errors are structured
data with stable codes, never panics and never bare strings") gains a
pointer to §6.3's table; delete §12 item 15; strike this row; TODO.md §2's
M3 row, §3 and §6.

---

### D16 — The progress roll-up: a tool, a field, or a client concern

**Resolved 2026-10-03 — option (a), signed by the user.** A
`status_summary(repo?, epic?)` tool returning counts by kind and status
plus D9's deferred `{total, ready}`, and `task_list` rows carrying their
`blocked_by` ids with every blocker's status. Work order executed in the
same change: §6's tool table gains the `status_summary` row and the
`task_list` annotation, §6 states the arithmetic as signed text (what
"open" means per kind, and that `ready` is exactly D9's
until-satisfied predicate), §12 item 16 deleted and the list closed
again, this row struck. One deviation from the work order as written: it
predates g7 and says the tracker gains the domain aggregation task
"under g6" — g6 closed at g7 while this decision was unsigned, so the
task lands under M3's goal instead, exactly as board §1's D16 row
records. The t29-measured cost stands as written: the domain gains the
aggregate and the annotated row type, not only a tool and a schema.

**Gates** M2 · **Resolved** 2026-10-03 · drafted 2026-09-30 from the M2
decomposition answering board U17 · **Confidence: medium**

**Current (before this decision).** §6's signed list contains no
aggregate. Its two nearest pieces point in different directions:
`next_items` carries D9's `deferred: {total, ready}` summary for the
queried scope — a summary riding a query's response, not a tool of its
own — while `graph_show` (§6.2) dumps the *complete* graph, every node
with its `blocked_by`/`blocks` ids, status, kind, priority and
references, and signs "Size is the caller's problem: the dump is complete
and deterministic; clients slice." §11 signs "No web UI, no TUI of its
own; clients render." So the spec already contains both the precedent for
a small derived summary and the precedent for handing a client everything
and letting it compute.

**What U17 asked, and what the decomposition settled without a vote.**
U17's intent, stated by the user 2026-09-30: the tooling this repo uses
for todo handling belongs inside the server, so another project's agents
get it over the wire instead of each repository growing its own scripts.
Most of that mapping needs no decision, because it is already signed —
`attention.py`'s "what is next and what is blocked" is `next_task` and
`next_items` with D11's derived weight and D9's deferred summary;
`progress.py`'s graph is §6.2's `graph_show`; `tasks.py` stays this
repo's session tracker, since §9 bars dogfooding the server here before
M3; and the HTML/browser rendering is a client-side concern over
`graph_show`, which §11 already decides. The one piece §6 does not carry
is the roll-up this board page renders: totals by kind and status, and
per open item what it waits on.

**Why it is a decision rather than an implementation detail.** Adding it
amends D5's signed tool list, and §12's rule is that a change to a signed
shape is a new decision, never an edit. D11's `next_items` is the worked
example of doing this properly: a new surface, brought to the user, signed
before it was built.

**Options.**

- **(a) A `status_summary(repo?, epic?)` tool**, returning counts by kind
  and status for the scope plus D9's deferred `{total, ready}` so the two
  summaries cannot disagree about what "ready" means; and `task_list` rows
  gain their `blocked_by` ids with each blocker's status, so "what waits
  on what" needs no second round trip. The aggregation is a pure function
  of one graph, so it lands in `todo-domain` per §2, and the all-repos
  case is M3 composing per-graph summaries over handles it already holds
  — the domain stays single-repo. Cost: one more tool in a signed list,
  one more `outputSchema` for §10's byte-golden rule to pin, and a name to
  choose (§6's names are a proposal; the shape is the decision).
- **(b) No new tool — a `counts` block on §6.2's `graph_show` dump**, plus
  the same `task_list` annotation. Cost: `graph_show`'s bytes are
  golden-pinned, so the dump moves — a compatibility event, cheap before
  M3 ships a client and expensive after; and a caller that wants four
  counts pulls the whole graph, which at the top of §4.1's scale (10^5
  nodes) is the expensive way to answer the cheap question.
- **(c) Leave it client-side over `graph_show`**, per §11's "clients
  render": the dump is complete and deterministic, so any roll-up is a
  pure function of it. Cost: every consumer re-implements counting, and
  counting is where the determinism bugs live — which statuses are "open",
  whether a `deferred` item whose until-targets are all finished counts as
  ready, whether a `signed` decision still gates the work it blocked.
  Three clients, three answers, none of them pinned by a golden. That is
  the markdown-board failure mode this project exists to remove, in a
  smaller register.

**Recommendation: (a)**, with the `task_list` annotation whichever option
wins. The roll-up is a query; §2 puts a query the domain can host in the
domain; and (a) is the only shape whose cost is O(nodes) with an
O(1)-sized response, which is the property that makes it usable as the
call a session opens with — what `attention.py` is used for here. (b) is
rejected on the size argument and on spending a golden-moving diff on an
aggregate; (c) is rejected because "which statuses count as open" is
precisely the kind of question the server exists to answer once. The
annotation half is separable and cheap: per-node `blocked_by` ids already
exist in §6.2's dump, so `task_list` is the only listing that does not
carry them. No option needs a §6.1 code beyond the closed list — an
unknown repo or epic is `unknown_id`.

**Cost measured while building t29 (2026-10-01), qualifying the annotation
sentence above.** `task_list` landed returning `Vec<TaskId>` — bare ids,
deliberately, so that no part of an unsigned roll-up could be pre-built by
accident (spec §5.2/§6, board §3). Two consequences for this vote, both read
off the code rather than argued:

- The annotation is **not** composable in the server over those ids. The rule
  that builds a row's `blocked_by` — the task's own predecessors, then its
  epic's decision gates appended and deduped at first occurrence, as two
  ascending runs — is `items.rs`'s private `annotate`, and that order is a
  signed contract M3's `outputSchema` will have to state. A server assembling
  D16(a)'s annotation from `Vec<TaskId>` plus the public `blocked_by` accessor
  would re-implement the rule in the binary, where §2 says it translates and
  does not decide. `NextItem::blocked_by` is the precedent in the other
  direction: the last annotation this project added landed as a domain row
  field.
- Neither is the counting. Six `task_list(status=s).len()` calls in the binary
  is an aggregation in the server, and each call re-derives the canonical
  order, so one summary costs six derivations.

So a signed (a) — or (b), whose annotation is the same — most likely adds a
domain aggregate **and** a domain row type or annotated sibling, not only a
tool and a schema. That is a cost of the option, recorded beside the claim it
qualifies; it changes no recommendation, and (c) is untouched by it, since
§6.2's dump already carries per-node `blocked_by` under the signed D11
amendment and a client can read the annotation straight off it. Nothing was
built either way: t29 measured this while refusing to pre-empt the vote.

**On sign-off:** amend §6's tool table with the chosen shape (and §6.2 if
the dump carries it); state in §6 what the summary counts — which
statuses are open, what makes a deferred item ready — so the arithmetic
is signed text rather than an implementation's opinion; delete §12 item
16; strike this row; TODO.md §2's M2 and M3 rows, §4's U17, §3 and §6. If
(a) or (b) is signed, the M2 tracker gains a domain aggregation task
under g6; if (c) is signed, U17's resolution records that the roll-up
stays a script and no task is added.

---
### D17 — Priority on the triage surfaces

**Resolved 2026-10-04 — the triage-surface shape, signed by the user
("let the agent decide").** `priority` is echoed on the surfaces an
agent or a dashboard actually picks by: `next_task`'s answer carries
the chosen task's `priority` beside its id, `next_items` rows carry
each task's `priority`, and `task_list` rows carry it too (the
dashboard column — the user's stated motivation). The selection order
is untouched: D6's canonical order stays dependency-driven, priority
never enters it, and the base instructions state the handling rule —
set priority when triaging, prefer higher priority when choosing
among actionable rows yourself, the server's order ignores the field
by design. Work order executed in the same change: §5.2's batch
description and §6's tool table rows amended, §11's non-goals bullet
concretized to the named surfaces, §12's preamble records the
signing, this row struck. The implementation task lands under M3's
goal before the g9 exit gate, so M3 closes with the final v1 surface
rather than amending it immediately after closure.

**Gates** M3 · **Resolved** 2026-10-04 · drafted and signed in one
exchange, out of the user's priorities ask · **Confidence: medium**

**Current (before this decision).** D11 signed `priority` as
agent-set triage metadata, "echoed in item payloads, never an
ordering input" — and the echo landed on the detail view only:
`task_show`'s `TaskDto` carries it. Measured at the t39+1 session:
`next_items` rows carry task/blocked_by/blocks/weight/actionable and
no priority; `next_task`'s answer is a bare id; `task_list` rows are
id plus blocked-by. The one surface an agent picks by — the batch —
cannot see the field. The base instructions (t33, U16a) do not
mention priority at all.

**Options.**

- **(a) Instructions-only** — the base instructions gain the handling
  rule; no wire change. Cheapest, but leaves the batch blind: the
  agent is told to prefer higher priority and cannot see any
  priority.
- **(a+, signed)** — the instructions rule **plus** the echo on the
  three triage surfaces (`next_task`'s answer, `next_items` rows,
  `task_list` rows). The server informs the decision; the agent
  makes it; the order stays D6-pure. The dashboard motivation is
  served by the same fields.
- **(b) Priority enters selection** — `next_task`/`next_items` rank
  actionable candidates by priority. This is the path D11's sign-off
  pre-named ("promotion into the tiebreaker would be a new decision
  amending D6"). Declined: the canonical order would become sensitive
  to mutable agent input, and `next_task`'s answer would change
  whenever an agent re-triages — the auditability argument for D6
  weakens for no gain the agent cannot get from (a+)'s visible rows.
- **(c) Priority as a query filter** — a `priority` parameter on
  `task_list`/`next_items`, like status/kind. Not taken now; it is
  caller-side triage over the same rows and can ride a later
  decision if a client wants it.

**Recommendation was (a+)**, on the user's own framing: "let the
agent decide" means the server's job is to inform the decision, not
make it.

**On sign-off:** amend §5.2's batch description and §6's tool table
(the `next_task`, `next_items`, `task_list` rows); concretize §11's
priority bullet to the named surfaces; record the signing in §12's
preamble; strike this row; TODO.md §1/§2/§6. The implementation task
echoes the field through the domain row types (`NextItem`, `TaskRow`,
`NextTask`) and the wire DTOs, moves the pinned goldens for the three
tools (additive fields — a compatibility event, called out in the
commit), and extends the base instructions with the handling rule.

### D18 — The dacode-dialect mapping (amends §9's mapping table)

**Resolved 2026-10-05 — option (a), signed by the user ("let's do
option (a) with 'A-deltas → tasks'").** The five populations map as
proposed: `A`-deltas → tasks under the synthetic `Spec deltas` epic
(the contested choice — the queue-visibility argument carried),
sub-steps → `gate`-kind tasks under their milestone epic, docs → doc
nodes, parentless `U` items under the synthetic `Follow-ups` epic,
and id reproduction by import order, which the contiguity measurement
makes possible with no Store or domain change. Work order executed in
the same change: §9's mapping table gains the five dacode rows, §10's
fixture bullet names the three files, §12 item 18 deleted and the
preamble records the signing, this row struck. The conventions
recorded below (status mapping, timestamps, the mechanical-edge rule,
`B1a`, the not-imported surfaces) stay task-spec level — each pinned
by a test in the mapping task (t43), veto-able as readings. The
parser (t42) was never gated; t43 is unblocked by this signing.

**Drafted 2026-10-05 — out of the M4 dialect study; awaiting the user's
vote.** §9's mapping table was written against the dialects' shared
vocabulary and covers four shapes: `U` items → tasks, `D`/`B`
decisions → decision nodes, milestones → epics, dependency prose →
edges where mechanical. The study measured the dacode board and found
**five populations no signed row covers**: the `A`-delta register, the
milestone sub-step tables, the doc register, the parentless `U` items
(§6 is explicitly "not tied to a milestone" while every task requires
an epic parent, §3), and the id-reproduction convention. The importer
cannot produce a graph without choosing for all five; this decision is
that choice.

**Gates** M4 · **Resolved** 2026-10-05 · drafted and signed in one
exchange, out of the M4 dialect study · **Confidence: medium**

**The measured dialect (the study, 2026-10-05 — the importer's
evidence base).** The dacode board is a **three-file system** under
`docs/implementation/`:

- `TODO.md` — 10,811 lines / 1.1 MB: §1 a current-state banner plus a
  blocker table; §2 the decisions summary table; §3 the delta queue;
  §4 the doc-depth register; §5 the milestones `M0`–`M6` with
  sub-step tables; §6 the `U` items in ~45 thematic/session
  subsections.
- `appendices/B-decisions.md` — 3,071 lines: a sign-off summary
  (`| # | Decision | Recommendation | Confidence | Gates |`) and 21
  dossiers (`### N. [B<N>] Title — ✅ RESOLVED date: outcome`).
- `appendices/A-design-deltas.md` — 3,792 lines: an index
  (`| # | Section | Delta | Depends on | Recommend |`) and 28
  dossiers (`### A<N> — sections: title — ✅ APPLIED date`).

Registers, all stable and never renumbered, all **contiguous**
(measured): `U1`–`U170` (75 unchecked, 95 checked, zero gaps);
`B1`–`B21` plus the oddball `B1a` (a prose sub-entry, "done");
`A1`–`A28`; milestones `M0`–`M6` (seven, not six — `M6` native nodes)
with sub-steps `M0.1`–`M0.4` and `M1.1`–`M1.11` (15 table rows, each
with a Gate column: docs-only / dual review / 🧠-then-dual-review) and
letter-suffix sub-sub-steps `M1.1a`–`M1.1g` at prose level; 50 docs
(42 content + 7 appendices + README) with
`migrated summary; expand during implementation` markers (19 of 42
still summaries). `f`/`g`/`t` ids are cited in prose but live in that
repo's gitignored tracker — not board items.

Status vocabulary: checkboxes `[ ]`/`[x]`; inline strikethrough for
supersession (never whole items); emoji — ✅ (304 occurrences:
resolved/complete/discharged/applied), ▶️ (24: started), ⚠️ (275:
caveat/half-applied), 🧠 (53: brainstorm-gated); table statuses
("✅ resolved", "⚠️ resolved, half-applied", "✅ APPLIED date",
"✅ SIGNED OFF date", "⚠️ HALF-APPLIED"); day-precision dates
(`YYYY-MM-DD`) throughout.

Dependency structures: **structured** — B's `Gates` column (doc
numbers plus a milestone: "02, 03 → M0"), A's `Depends on` column
(B ids), §1's blocker-table rows; **semi-mechanical prose** —
"blocked on <id>" ×10, "gates M<n>" ×8, "Blocking M<n>" ×5,
"due by M<n>" ×2; **non-mechanical** — "before M<n>" ×47 (temporal,
mostly not edge-shaped). Cross-referencing is heavy: 1,170 `U`-id
mentions, 30 distinct doc links. An HTML `attention-meta` comment
block declares per-item action types — tool metadata, not domain
state.

**The proposal.** Four of the five populations have one sane answer
each and are proposed as a block; the contested one is the `A`-delta
register, which has two.

1. **Milestones `M0`–`M6` → epics `E1`–`E7`** (board order; §9's
   signed row). **Sub-steps → `gate`-kind tasks under their milestone
   epic** — the board's own words make them checkpoints ("the live
   schedule is §5's sub-step table"), the row's Gate column maps to
   `verify`, and epics cannot nest (§3), so a second epic level is
   not available. Letter suffixes (`M1.1a`–`g`) stay body prose.
2. **`U` items → tasks under a synthetic `Follow-ups` epic** (`E8`)
   — the board itself says §6 is "not tied to a milestone", and §3
   requires every task to carry a parent epic. **`A`-deltas → tasks
   under a synthetic `Spec deltas` epic** (`E9`) per the contested
   choice below. Both epics' descriptions carry their provenance.
3. **Docs → doc nodes** — the §4 register's 50 (README, 00-overview,
   parts 1–6 numeric, appendices A–G) in that deterministic order;
   path repo-relative; `version` carries the depth marker
   ("summary"/"expanded") — the one version-like fact the board
   holds (a convention detail the task spec may simplify).
4. **Id reproduction** — the registers are contiguous, so importing
   in board id order lets the server's own never-reuse allocation
   reproduce `U1`–`U170` and `D1`–`D21` exactly, with **no
   explicit-id insert surface and no Store/domain change**. Non-`U`
   tasks (sub-steps, then `A`-deltas, then `B1a`) number after `U170`
   in the documented add order, each carrying its original id in
   `references` (kind `other`, target `A14`).
5. **The `A`-delta register — the contested choice.**
   - **(a, recommended) tasks** (kind `task`) under the `Spec deltas`
     epic: `APPLIED` → `closed` (evidence = the application record),
     `SIGNED OFF` → `open`, `HALF-APPLIED` → `in_work`. The board's
     §3 **is a work queue** — "4 of 27 applied" is owed transcription
     work, and after cutover the server must answer "what spec work
     is owed" from `task_list`. A's `Depends on` column becomes
     `decision → task` blocks edges (a legal §5.1 shape), so the
     approving B-decision gates its delta exactly as the board
     asserts.
   - **(b) decision nodes** `D22`–`D49`: dossier-shaped and
     parent-free, but an approved-but-unapplied delta would map to an
     **open** decision — telling every cold session "the user needs to
     decide this" about something already decided, and rendering the
     queue as votes owed rather than work owed. The semantics invert;
     that is the argument against.

**Conventions under the signed umbrella** (task-spec level, each
pinned by a test, veto-able as readings): status mapping — `[ ]` →
`open`, `[x]` with a ✅ discharge marker → `closed` (evidence = the
discharge prose, bounded; `[x]` without a marker → `closed` with
evidence "imported checked"), ▶️ → `in_work`, 🧠 → `deferred` (note =
the gating prose), "blocked on <id>" → `blocked` (note = the prose),
⚠️ → prose, never a status; timestamps — board dates →
`T00:00:00.000000000Z`, and a caller-passed `imported_at` for
created-stamps with no board date, so the fixture stays deterministic
(§8); the mechanical-edge rule — B's `Gates` column (milestone half
only; §5.1 has no decision→doc edge, so doc-gating stays references),
A's `Depends on`, and "blocked on <id>" become edges, and each
**recognized but unresolvable** dependency assertion (one that names
ids but cannot become a legal edge) becomes one `finding`-kind task
rather than a silent omission — vague temporal prose is not
recognized and stays body text; `B1a` → a closed task under
`Follow-ups` referencing `B1`; the `attention-meta` block and §1's
narrative are not imported (the blocker table duplicates per-item
state and is superseded weekly).

**On sign-off:** amend §9's mapping table with the dacode rows
(A-deltas, sub-steps, docs, the synthetic epics, id reproduction);
§10's fixture bullet names the three files; delete §12 item 18;
strike this row; TODO.md §1/§2/§6. The importer's mapping task (t43)
is gated on this decision; the parser (t42) is not.

### D19 — The lock-free read path and §4.3's torn-tail reading

**Resolved 2026-10-08 — option (a), signed by the user ("let's do
'(a)' with lock-free reads").** A read-only open that takes no lock,
replays the committed prefix, and treats an unterminated final line as
an append in flight — never `corrupt_store`, never a write. Work order
executed in the same change: §4.3's replay rule gains its scoping
sentence (it binds the locked open and repair; a lock-free read's
unterminated tail is concurrency, not damage), §8's single-writer
bullet makes its concurrent-readers sentence concrete, §12's preamble
records the signing, this row struck. No §12 item sat open — the
decision was drafted and signed in one exchange out of the first real
second-process contention (below), the in-thread precedent. The
implementation lands as two tasks (the store primitive, then the
server's read-path resolve); until they land, §8's new sentence
describes a signed shape the code does not yet have — TODO.md §1's
blocking table carries the gap.

**Drafted 2026-10-08 — out of the first real second-process contention
(f110, measured).** §8 says "the JSONL engine (D1) tolerates concurrent
readers of a committed log", but every open today takes the exclusive
writer lock — there is no read-only open — so a second process cannot
read a store whose lock a live server holds. Measured: a live opencode
session's server (PID 99063, 2026-10-08 16:45, parent opencode 99050)
holds all four store locks after one `repo_list` (handles are never
evicted — U20(b)'s answer-from-handles), and `tools/progress.py`'s
per-render spawn — t54's thin client, the dogfooding cutover — is
refused on its one `graph_show` with `locked`. The registry is already
safe to read without a lock (`Registry::load` takes none, and the file
is atomically renamed); the whole question is the log's torn tail.

**Gates** — (post-M5: the dogfooding cutover's usability) ·
**Resolved** 2026-10-08 · drafted and signed in one exchange ·
**Confidence: medium**

**Current (before this decision).** §4.3's replay rule — "a malformed
*last* line (no trailing newline, or unparseable) is a named
`corrupt_store` … never silently skipped" — was written for the locked
open and repair paths, which hold the writer lock and therefore never
see a line that is merely being written. A lock-free reader racing an
append *can* see one: an append is a single `write_all` under
`O_APPEND`, so an unterminated final line is definitionally a write in
progress. Reading it as corruption would make every lock-free read
fail exactly when a writer is active — the one case the path exists
for. f63 (t31) named this precisely and deferred it; U20(b) resolved
the server's own reads by answering from handles, which left
second-process readers uncovered — progress.py is the first.

**Options.**

- **(a, signed) The lock-free read path** — a read-only open that
  takes no lock, replays the committed prefix, and treats an
  unterminated final line as an append in flight (ignored, not
  `corrupt_store`); it never opens the append handle and never
  writes. A terminated but unparseable final line stays corruption on
  every path — the reading weakens nothing else. The server's read
  tools use it on a handles miss (U20(b) unchanged for armed
  servers); progress.py's per-render spawn picks it up on its next
  render, no restart choreography — the fix is reader-side, and every
  reader that matters spawns fresh.
- **(b) Degraded rendering in clients** — catch `locked`, render the
  last cached page with a banner. No spec change, but the first run in
  a locked state still fails and the cache stales during exactly the
  sessions being watched.
- **(c) Idle-handle eviction in the server** — release handles after
  an idle window. Rejected on measurement: re-open is O(total
  history) (t31's cold-open grid), lock availability becomes
  time-based, and real writers (`--refresh`) still contend — a
  deterministic refusal traded for intermittent success.

**Recommendation was (a)** — it is what §8's concurrent-readers
sentence exists for, and the torn-tail reading is narrow and
principled: the trailing LF is the commit marker, so no-LF means
not-committed, whatever the bytes.

**On sign-off:** §4.3's replay bullet gains the scoping sentence; §8's
single-writer bullet makes the concurrent-readers sentence concrete;
§12's preamble records the signing; strike this row; TODO.md
§1/§3/§4(U34)/§6. The implementation is two tasks — the store
primitive (a lock-free read returning the replayed graph), then the
server's read tools resolving lock-free on a handles miss — and the
blocking table carries them until they land.

### D20 — The all-in-one binary: the MCP server gains the tool CLI

**Status:** signed (the direction) 2026-10-09 — the user's in-thread
"yes, let's have one single binary, that has also tools CLI
commands", answering the framing raised at the dacode cutover.

**The question.** The tooling around the server is split across three
homes: the binary (the MCP server over stdio), `tools/progress.py`
(the render/refresh/serve thin client — 1056 lines of Python driving
the binary over a per-render stdio spawn, the slug hardcoded), and
`todo-cli` (the store inspector: dump/verify/repair). The Python
exists to drive a binary that cannot render; the split costs a spawn
plus a handshake per render, a second language in the toolchain, and
a per-repo parameterization the binary already carries (`--repo`,
the cwd tier).

**The signed shape.** One binary. `todo-mcp` invoked with no
subcommand stays the MCP server on stdio — the global-config entry's
command array is unchanged, zero wiring movement — and subcommands
carry the tool surface, per project:

- `todo-mcp progress` — the render (the page and the JSON), reading
  the store lock-free via `DataRoot::read_repo` (D19): no spawn, no
  handshake, no Python.
- `todo-mcp refresh` — the registry surgery (re-register + re-import,
  the t54 crash order), refusing gracefully where a repo's board
  files are gone — a cut-over repo's store is primary and there is
  nothing to re-import.
- `todo-mcp serve` — the live view (U18's shape A: 127.0.0.1:0, the
  per-user port publication, attach/stale-takeover).
- `todo-mcp dump | verify | repair` — todo-cli's inspector absorbed;
  that crate retires.

**Consequences settled with the sign (each veto-able):**

- `tools/progress.py` and the two committed drivers retire with the
  port; their behavior pins move to Rust integration tests under
  `crates/todo-mcp/tests/`.
- The subcommands' output is golden-pinned per §10 — the render's
  byte-identity, the refresh round-trip, the dump bytes (todo-cli's
  goldens move to the new spelling).
- No new dependencies: lexopt is already the workspace's parser
  (U14); the render is hand-built strings (the Python template
  ported); the listener is `std::net`.

**Recommendation** — execute as a six-task arc under one goal: the
dispatch skeleton with the JSON render, the HTML page, the refresh,
the serve, the todo-cli absorption, the Python retirement plus docs.
The blocking table carries the arc until it lands.

### D13 — The default server's live view: the per-root listener

**Status:** signed (a) 2026-10-10 — the user's in-thread "D13. Let's
take (a) option as you suggest", answering the shape presented at
their ask ("make the default todo-mcp serve http, reusing the same
port across sessions of the same root, with the port identifiable
per root").

**The question.** Reserved at the M5 decomposition as U18's shape B
territory: a listener whose lifetime is not any one session's. Every
opencode session spawns its own stdio server through the global
entry; the live view today is `todo-mcp serve`, a manual foreground
process publishing to a per-user port file — nothing guarantees a
listener is up when a session is, and two roots share one
publication slot.

**The signed shape (a) — the startup ensure-listener with watchdog
takeover.** Every no-subcommand server, at startup, probes the
per-root publication: a live listener means reuse — the same port, no
second listener; dead, stale or absent means bind `127.0.0.1:0`,
publish, and run the listener on a std thread beside the tokio
runtime (the serve module's existing design — no async-core change).
The publication is per-root (`<root>/serve.port`, the flip the serve
module's own doc already named), which is also the port's discovery:
a machine-readable file per root. Servers that do not own the
listener re-probe periodically and take over when the owner dies —
the view is up while at least one session server (or a manual
`serve`) is alive, with an interval-scale gap after an owner exits.
The MCP wire surface is untouched: no new tool, the stdio server
unchanged.

**Consequences settled with the sign (each veto-able):**

- The `serve` subcommand's publication moves to the same per-root
  file — one discipline for both paths; the per-user state file
  (`~/.local/state/dacode-todo-mcp/serve.port`) retires, its Python
  interop constraint dying with t73's retirement of the Python.
- A server spawn is no longer pure stdio: the listener thread is a
  side effect of startup. It is a std thread (the serve module's
  shape), so the async core and the wire are untouched.
- Stale publications are the norm — a session server dies with its
  process and cannot clean up; the probe-then-takeover handles them
  (the existing serve semantics, run periodically). The double-bind
  race leaves one benign orphan listener, already spec-recorded as
  acceptable.
- The declined alternatives: (b) a detached listener child (shape B
  proper — a process nobody owns, needing an explicit stop story)
  and (c) the status quo (`serve` stays manual).

**On sign-off:** §13's tool-CLI paragraph gains the ensure-listener
sentence; §12's reserved-D13 sentence becomes the signing record;
strike this row; TODO.md §1/§6. The implementation is one task (t85
under g22): the per-root publication, the startup ensure-listener,
the watchdog, and the pins — probe/attach/bind, per-root keying,
takeover after owner death.

---

### D21 — The output schemas' error union (amends D15)

**Resolved 2026-10-12 — option (a), signed by the user.** Every tool's
declared `outputSchema` becomes the `anyOf` union of its success payload
and D15's error carrier, so a strict client that validates any response
against the declaration — error results included — accepts both arms.
D15's conformance exemption ("an error payload is not required to
conform") is retired: the ipmi-collector session's failing `task_add`
(U51) proved the exemption's cost — the client rejected the carrier as
an opaque -32602 and the code and message never arrived. Work order
executed in the same change: §6.1's carrier paragraph and §6.3's D15
finding reworded to the union, the advertisement re-pinned
(protocol.rs's REPO_REGISTER_ENTRY re-frozen; the flat-enum walk gains
array descent so its coverage stays non-vacuous behind `anyOf[0]`).

**Gates** — · **Resolved** 2026-10-12 · drafted from U51's finding (the
ipmi session's -32602 transcript) · **Confidence: high**

**Current (before this decision).** D15's signed shape: each tool
declares its outputSchema explicitly, and an error payload is not
required to conform to it — the schema describes a success. rmcp does
not validate the field, so the server's own tests never caught a client
that does.

**What the incident measured.** The ipmi-collector session's first
`task_add` failed underneath; the server answered D15's carrier
(`{code, message}`); the calling client validated the carrier against
the declared success-only schema and rejected the whole response as
-32602 "Structured content does not match the tool's output schema",
listing the success DTO's required fields. The session never saw the
real error and retried blind (the user's report). The write suites pin
the server's carrier emission through the real binary — the defect was
the declaration, not the emission.

**Options.**

- **(a) The union declaration.** Every outputSchema becomes
  `anyOf: [success payload, carrier]`, with `$schema`/`title`/`$defs`
  hoisted to the union's root so `$ref` pointers resolve from the
  document root. Both arms validate under any client; the schema
  pinning stays. Cost: the `tools/list` advertisement changes — a wire
  surface move, hence this decision.
- **(b) Drop the outputSchema declarations.** Also a wire change, and
  it loses the schema pinning D15 committed to.

The user signed (a) in-thread 2026-10-12 ("ok, let's fix outputSchema").

### D22 — The reward model: sizes, arithmetic, triggers, opt-in

**Status: open — drafted 2026-10-13 from the user's in-thread
steering; awaiting signature.**

**Gates** — E13 · **Drafted** 2026-10-13 · **Confidence: high** (the
user steered every open point in-thread)

**Current (before this decision).** Tasks carry priority (display-only,
D11/D17) and kind; no size or effort estimate exists anywhere — the
"rough estimation" the feature needs is absent. No per-root config
document exists; the root holds the registry and the repos only.

**The user's steering (2026-10-13, in-thread).** Two size buckets —
the planned size at creation ("pre-flight") and the actual size after
implementation ("final"); existing tasks default to `-` (no estimate)
rather than retroactive guesses. Reflection triggers: milestone
closure plus the accumulated amount. Existing data migrates with zero
estimations and zero totals.

**Options.**

- **(a) The two-bucket model, server-computed rewards, per-root
  config, strictly opt-in.** §15's model paragraphs are the shape: the
  estimate at creation, the actual at closure, the integer arithmetic
  (`round_half_up(base × priority_pct × kind_pct / 10000)`), the
  currency label, the milestone and `reflect_every` triggers, the
  inert-by-default root. The agent never picks its own amount; the
  planned/actual pair is the honesty anchor.
- **(b) A single creation-time estimate.** A smaller surface, but the
  actual-work axis — the feature's honest measure — is lost, and the
  planned/actual delta (the estimation-accuracy data) with it.
- **(c) Priority-only rewards.** No new task fields at all; the volume
  axis the user asked for is absent.

**Recommendation: (a)** — it is the steering, written down.

**On sign-off:** delete §12 item D22; make §15's model paragraphs
normative; open the domain unit's task (the reward types and the pure
computation, property-tested on the integer arithmetic and the
default-bucket fallbacks).

### D23 — The storage move: the reward events and FORMAT_VERSION 2

**Status: open — drafted 2026-10-13; awaiting signature.**

**Gates** — E13 · **Drafted** 2026-10-13 · **Confidence: high**

**Current (before this decision).** The event vocabulary is v1's:
nodes, edges, patches — no reward or reflection events, no size fields
on creation events. `FORMAT_VERSION` is 1; the version gate
(`version_mismatch`, §6.1/§13) makes an older binary refuse a newer
store rather than guessing.

**Options.**

- **(a) The version bump with in-place header migration.** Two new
  event kinds (a reward granted: the task, the amount, the size and
  multipliers applied; a reflection logged: the verbatim answer, the
  trigger, the totals at the moment) plus the `estimate` riding
  task-creation events. v2 is a strict superset — v1 logs replay
  identically, so the migration is the header line alone, rewritten in
  place the first time a v2 binary opens a v1 log for writing.
  Existing roots migrate with zero estimations and zero totals (the
  user's steering: no retroactive rewards). The store goldens
  regenerate under the version-tracking convention (a format bump
  renames the golden).
- **(b) A sidecar rewards log per repo.** No version bump, but a
  second file per repo with its own discipline — and the reflection
  answers are store data; they belong in the log the inspector and the
  repair surface already own. More moving parts, no cross-binary gate
  on the new vocabulary.
- **(c) No persistence.** Rewards computed on the fly from closures —
  no totals history, no reflections, the feature forgets itself.

**Recommendation: (a).**

**On sign-off:** delete §12 item D23; amend §4 (the v2 event
vocabulary, the config document, the header migration); open the store
unit's task (the config document load/save with its golden, the event
kinds, the bump, the derived per-root totals).

### D24 — The wire move: the surface extension

**Status: open — drafted 2026-10-13; awaiting signature.**

**Gates** — E13 · **Drafted** 2026-10-13 · **Confidence: high**

**Current (before this decision).** The §6 tool list is D5's signed
v1 surface plus the D10–D17 amendments; the base instructions carry
the tool-failure recovery rule (U16a) but nothing about rewards —
nothing exists to instruct about.

**Options.**

- **(a) The minimal surface.** `task_add` gains the optional
  `estimate`; the closure call gains the optional actual size; the
  closure response gains the reward line and, on a trigger, the
  reflection question; one new tool, `reward_reflect` (the repo, the
  verbatim answer), its schema under D21's union discipline; the base
  instructions gain the rule — answer reflection prompts honestly,
  once, through the tool. Disabled roots answer byte-identically to
  today (the existing wire goldens must not move).
- **(b) A rewards tool family.** `reward_status`, `reward_history`
  and friends — more surface, and the progress surfaces already
  answer those questions; the wire stays minimal instead.
- **(c) No wire changes.** Rewards visible only through the CLI and
  the progress render — the agent never sees them, and "clearly
  prompted to the model" (the user's words) dies.

**Recommendation: (a).**

**On sign-off:** delete §12 item D24; amend §6's tool table (the two
optional parameters, the response lines, the new tool); open the
surface unit's task (protocol-level tests, the schema advertisement,
the base-instructions rule, disabled-root byte-identity).
