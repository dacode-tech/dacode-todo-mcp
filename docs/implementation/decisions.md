# Decision record — dacode-todo-mcp

The sign-off dossier for the spec's §12 open decisions. Each `D` entry
mirrors the §12 item of the same number. **Only the user signs off;
agents propose.** A resolved entry is struck in the summary, carries a
resolution banner, and its `On sign-off:` work order is executed in the
same change — most orders delete the §12 item, so the spec moves too.

`tools/attention.py` reads this file's summary table, its detail entries
and the meta block below; keep the shapes it parses.

<!-- attention-meta — machine-readable; read by tools/attention.py.
     No open decisions: D1–D11 all resolved 2026-09-25. A new entry
     declares its action here (e.g. `action: D12=vote`).
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
