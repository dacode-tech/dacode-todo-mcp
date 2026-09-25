# Decision record — dacode-todo-mcp

The sign-off dossier for the spec's §12 open decisions. Each `D` entry
mirrors the §12 item of the same number. **Only the user signs off;
agents propose.** A resolved entry is struck in the summary, carries a
resolution banner, and its `On sign-off:` work order is executed in the
same change — most orders delete the §12 item, so the spec moves too.

`tools/attention.py` reads this file's summary table, its detail entries
and the meta block below; keep the shapes it parses.

<!-- attention-meta — machine-readable; read by tools/attention.py.
action: D1=vote
action: D2=vote
action: D3=vote
action: D4=vote
action: D5=vote
action: D6=vote
action: D7=vote
action: D8=vote
-->

## Sign-off summary

| ID | Decision | Recommendation | Confidence | Gates | Status |
|---|---|---|---|---|---|
| **D1** | Storage engine | (a) JSONL event log + in-memory graph, behind the Store trait | medium | M1 | open |
| **D2** | Graph model and edge semantics | adopt §3/§5.1 as written | medium | M1 | open |
| **D3** | Identity scheme | per-repo sequential integers per kind, never reused | medium | M1 | open |
| **D4** | Store layout | one shared store for all repos under ~/.local/share/dacode-todo/ | medium | M2 | open |
| **D5** | MCP tool surface | adopt the §6 list; ship todo-cli | medium | M3 | open |
| **D6** | Selection tiebreaker | topology, then epic order, then id | medium | M2 | open |
| **D7** | Importer scope | one-shot read-only import, camera dialect first | medium | M4 | open |
| **D8** | Cross-repo ergonomics | defer cwd auto-detection to M5; id-keyed access in v1 | high | M4 | open |

---

## Details

### D1 — Storage engine

**Gates** M1 · **Action** vote

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

**Gates** M1 · **Action** vote

**Current.** §3 proposes three node kinds (repo, epic, task), four task
kinds (task, gate, finding, debt), and §5.1 three edge kinds (parent
tree, blocks DAG, relates annotation) with write-time cycle rejection and
the no-orphans rule.

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

**Gates** M1 · **Action** vote

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

**Gates** M2 · **Action** vote

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

**Gates** M3 · **Action** vote

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

**Gates** M2 · **Action** vote

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

**Gates** M4 · **Action** vote

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

**Gates** M4 · **Action** vote

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

**On sign-off:** delete §12 item 8; record the deferral in §7 and in the
M5 row of TODO.md §2; if (b) is chosen instead, add a `repo_detect` row
to §6 and a determinism rule for its resolution order.
