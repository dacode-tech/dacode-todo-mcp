<!-- docs/implementation/appendices/B-decisions.md -->
# Appendix B — Open decisions needing sign-off

> Index: [README](../README.md) · Spec: [`dacode-design.md`](../../dacode-design.md)

Blocking or high-impact choices (§16's list, plus new ones surfaced by the
implementation-plan split).

**This appendix is now the decision dossier.** Each entry carries the options,
the evidence, a recommendation, and the exact text to write once you sign off.
Resolved items are **struck, not deleted** — this file is the audit trail
(B4's rule).

Evidence for B5, B7, B8 and B9 comes from two cross-harness digests written for
this pass:
[sandboxing-and-checkpoints](../../../.agents/knowledge/reference-harnesses/sandboxing-and-checkpoints.md)
and
[dialects-and-acp](../../../.agents/knowledge/reference-harnesses/dialects-and-acp.md).
Evidence for B15 comes from a third, written later:
[task-tracking-and-gating](../../../.agents/knowledge/reference-harnesses/task-tracking-and-gating.md).
All three carry `path:line` citations and line counts into the four reference
checkouts.

<!-- attention-meta — machine-readable; read by tools/attention.py.
     One `action:` line per decision whose action type is not a plain vote.
     Absence means `vote`, which is the safe default.

     Declared rather than detected. This used to be inferred from prose, and
     that failed four times in one session: the last detail section ran to
     end-of-file and inherited the footer's description of B12; B18 was
     mislabelled the moment it *quoted* B12, which it must do because it is
     what unblocks it; and backticking does not help, because emphasis is
     stripped before matching. A sentence about a decision is not a property
     of it. `attention.py`'s hygiene block validates every id below.
     No decision needs one today. B12 was `measure` and B20 was `session`; both
     resolved 2026-09-17, and a marker is deleted with its row because the hygiene
     block rightly rejects an id that no longer has one.

     One `confidence-lead:` line per decision whose confidence cell does not lead
     with its *current* overall level; absence means "trust the first token",
     which is right for every other row. Values: high, med, low, open.
     No row needs one today. B12 did until it resolved 2026-09-17 — its cell read
     "was high → open", so the first token was a history lesson rather than a
     verdict. Same rule as `action:` above, for the same reason: the cell is prose,
     and prose about a decision is not a property of it. A marker is deleted when
     its decision resolves, because the hygiene block rightly rejects an id that no
     longer has an open row.
-->

## Sign-off summary

Read this table, then the detail only where you disagree. "Confidence" is how
strongly the evidence points one way — **low means it is genuinely your call on
taste, not on facts.**

| # | Decision | Recommendation | Confidence | Gates |
|---|---|---|---|---|
| ~~**B1**~~ | ~~JSON strategy~~ | ✅ **SIGNED OFF 2026-09-12** — hybrid adopted, §13 rewritten via A16. **M0 unblocked.** | high | ~~02, 03 → M0~~ |
| ~~**B2**~~ | ~~Web stack~~ | ✅ **SIGNED OFF 2026-09-17** — the split taken as recommended: `dacode-protocol` ships a **JSON-Schema export** as a non-optional **M1** item (it is a property of the protocol crate and benefits extension authors, an SDK, `run --output jsonl` consumers and the fixture suite regardless of framework), and the framework decision gets a **deadline of M3** with **TS SPA (Solid)** standing, mitigated by generating `protocol.d.ts` in CI and failing on drift. Makes honest that TODO §5's M1 list already carried the export while doc 03 called it optional. ~~**Approved, not yet applied — owes A8**~~ ✅ **applied 2026-09-18 via A8** → §9.1 records the split with the framework **open but deadline-bound to M3**, and doc 03's optionality is removed (it was a false claim, not a stale marker, so corrected under hard rule 1 with its marker untouched). **The application also resolved a contradiction this row could not have seen**: §9.1's old `build.rs`-in-`dacode-protocol` clashed with §13.3's "never in a shipped crate" and §2.4's placement of codegen in `xtask`, so the generator is `xtask`'s and `protocol.d.ts` is generated in CI with a drift check that fails the build | ~~med (facts) / low (taste)~~ — the taste half is deadlined (M3), not decided | ~~21, 19~~ ✅ |
| ~~**B3**~~ | ~~Delta vs full updates~~ | ✅ **SIGNED OFF 2026-09-17** — pure deltas taken as recommended: **deltas live, full value at the durable `*Ended` boundary**. **Approved, not yet applied — owes A1** → §3.2/§3.7, which must land **before any golden fixture** | ~~high~~ | ~~03, 08~~ ✅ |
| ~~**B4**~~ | ~~WS vs SSE~~ | ✅ **SIGNED OFF 2026-09-12** — WS kept as the single live channel; no SSE second transport in v1. The robustness half landed separately via **A18** → §3.7 | ~~med — the most genuinely two-sided~~ | ~~19, 03~~ ✅ |
| ~~**B5**~~ | ~~In-band tool dialects~~ | ✅ **SIGNED OFF 2026-09-17** — the **~700 LoC v1 subset** taken as recommended (Harmony control-token escaping on the transport copy, a `supports_tools: false` capability flag driving history demotion, one XML grammar with stream holdback plus a tool-name allowlist before promotion); the ten other dialects and per-request history re-encoding deferred. Both carve-outs stand as follow-ups: malformed tool-arg JSON repair is a different axis belonging to doc 14 (**still U10, still open**), and repair never scrubs inside fenced/indented code (**now normative §6 text**, not a follow-up — it is a rule an implementation can violate silently). ~~**Approved, not yet applied — owes A5**~~ ✅ **applied 2026-09-18 via A5** → §4.3/§4.4/§6/§14, with **`ToolDialect` as a reserved slot** so un-deferring the ten dialects adds variants rather than restructuring the engine. ⚠️ **The §16 row this entry asked for was deliberately not added** — §16 lists *open* decisions and B5 is resolved (A19's precedent); the deferral lives in §6's prose instead | ~~high~~ | ~~13~~ ✅ |
| ~~**B6**~~ | ~~Tool schema gen~~ | ✅ **SIGNED OFF 2026-09-12** (delegated by the user alongside B1) — offline codegen; the isolation boundary is now §13.3. The [14](../part3-capabilities/14-tools.md) half is **deferred to M1** by the just-in-time rule. | high | ~~14, 02~~ |
| ~~**B7**~~ | ~~`bash` sandboxing & honesty~~ | ✅ **SIGNED OFF 2026-09-17** — all five parts as recommended (deny-list only in v1, documented as *advisory* over §4.3/§4.4/§5.2; the interpreter-disqualification rule; the tight critical-pattern list; the reserved seam; the 2026-09-16 polarity), **with the user's scoping amendment recorded as a decision, not a footnote**: M1 ships the `SandboxBackend` seam as an **inert stub** — no Landlock/seccomp code — and §4.7's worktree remains the only real containment until the seam is filled. Coupling recorded: **B20's "a remote member must have a sandbox or be refused" cannot be satisfied until the seam is filled**, so B20's M5 build inherits "fill the seam first" (or uses container containment, which violates §1.5). The polarity amendment stands: sandbox = immutable session attribute set at creation, `yolo` = mutable opt-out, neither a default, indicator by colour with **U89** open on colour being the only channel. **Approved, not yet applied — owes A12**'s §16.3 half, doc 14 and 16 updates, the `bash:readonly` wording in §4.3/§5.2, and the confirmation-or-overturn of **A23's `yolo` carve-out** in §4.4 | ~~high~~ | ~~14, 16~~ ✅ |
| ~~**B8**~~ | ~~Checkpoint strategy~~ | ✅ **SIGNED OFF 2026-09-17** — shadow git kept with **all six additions** §4.6 omits (`.gitignore` filtering, 2 MiB untracked cap with skips reported, absent-from-tree ⇒ delete, selective per-file restore, path-escape rejection, the boundary turn id in the `Checkpoint` event), gitoxide in-process, and **both unsolved problems spec'd** (a retention policy — no harness has one; the concurrent-session shadow-store race stated as a trade, not discovered in M2). **The worktree question is answered rather than deferred**: a feature-level working directory is expressed through **§5.4 workflow instructions**, not a new session↔worktree binding, so §4.6 (undo) and §4.7 (containment) stay separate and the existing composition rule suffices. One residual owed to doc 12: a shared worktree needs a **one-writer rule** — B21's lock covers the session log, not the working tree. **Approved, not yet applied — owes A12**'s §16.5 half plus the §4.6 amendments | ~~high~~ | ~~12~~ ✅ |
| ~~**B9**~~ | ~~ACP commitment~~ | ✅ **SIGNED OFF 2026-09-17** — defer the build to **M5** (`dacode acp`: the fs-free, terminal-free, `mcpServers`-rejecting subset, 2 500–3 500 LoC, its stdio JSON-RPC framing shared with the M4 extension host) and commit the **two M1 constraints**: everything a client needs is derivable from the log via fold + `Subscribe { since }`, and **no tool may branch on client capabilities or locality**. The user's "be aware during implementation" gets a **mechanism, not an intention**: the two constraints plus doc 14's review rule, verified when doc 14 is written via the M1 doc-deepening checklist. Open input kept: no maintained Rust ACP crate was verified — check before M5 planning. **Approved, not yet applied — owes A12**'s §16.4 half, doc 26 moved to M5, the two constraints added to M1's docs | ~~high~~ | ~~26~~ ✅ |
| ~~**B10**~~ | ~~Config on disk~~ | ✅ **SIGNED OFF 2026-09-17** — TOML-on-disk / JSON-on-wire confirmed with **both consequences recorded**: `dacode config edit` must **preserve comments and formatting** (a format-preserving editor, `toml_edit`) — which matters more for **A7**'s config-write endpoints, since eating a user's comments from a web UI is worse than from a CLI — and a generated **`dacode.schema.json`** reusing B6's machinery and pairing with B2's schema export. The B16↔B10 coupling stays dissolved: nothing ports, so no YAML crate enters the Rust tree. ~~**Approved, not yet applied**~~ ⚠️ **HALF-APPLIED 2026-09-19 via A7**: consequence (1) is **normative at §5.1**, and the pass found that §12 already *cited* "(§5.1)" for it while §5.1 carried no such text. **Still owes A12**'s §16.2 half, consequence (2)'s `dacode.schema.json`, and both items in doc 17 | ~~high~~ | ~~17~~ ✅ |
| ~~**B11**~~ | ~~New crates~~ | ✅ **SIGNED OFF 2026-09-17** — as recommended: **defer `dacode-store`** (traits in `dacode-core::store`, extract the crate at M3 — a boundary with one implementation is indirection without payoff, and extraction later is mechanical); **approve `dacode-introspect` in principle, create at M5**, but its **exchange-record format is an M1 deliverable** in doc 13, because retrofitting provider instrumentation in M5 costs far more than emitting it from the start, and it gives doc 09 the harness that validates A4's cache invariants. The in-tree-vendoring settlement stands. ~~**Approved, not yet applied — owes A13**~~ ✅ **applied 2026-09-18 via A13** → §2.1/§2.2 record both as approved-with-dates with **neither added to the eleven-member tree**, the vendoring settlement made explicit, and Appendix D's *legend* corrected (its graph already carried both crates; what was stale was the word "proposed" and the absence of any milestone). **The record-format item is not merely added to M1 — it is already normative** in §6, landed via A10's carve-out the same day. ⚠️ A13 also recorded a hazard this entry could not have anticipated: `gates/boundary-edges.txt` is an *exact* diff against `cargo tree`, so the deferred crates' edges must **not** be pinned until their directories exist | ~~med~~ — the respectable dissent (separate `dacode-store` now for dependency isolation from day one) was available and not taken | ~~05, 25, D~~ ✅ |
| ~~**B12**~~ | ~~Session compression~~ | ✅ **RESOLVED 2026-09-17 — `zstd` accepted as a class-1 exception.** *"We can allow as exception to use zstd with C library."* No pure-Rust zstd *encoder* exists (`ruzstd` is decode-only), so the write path needs libzstd whichever way the vote went; [Appendix F](F-c-dependencies.md) §F.3's row moves from *avoidable* to **accepted**, and `zstd-sys` enters `gates/class1-allowlist.txt` in the same commit as the dependency (§2.3's rule). **U14's cross-compile cost is now the condition on the exception, not a reason to reopen it**, and B18's T1 bench still measures level and ratio so the default is tuned rather than guessed. **A15's hold is fully lifted** | ~~was high → open~~ | ~~05~~ ✅ |
| ~~**B13**~~ | ~~Session organization~~ | ✅ **SIGNED OFF 2026-09-17** — tags + virtual folders + projects confirmed as fold events with **Fix 1** (organization changes are durable log events but never part of `Context`; the store's mapping tables are a rebuildable projection) and **Fix 2** (project derivation maps a worktree back through `git rev-parse --git-common-dir`). **Amendment, and it reverses part of the recommendation: virtual folders are a first-class requirement**, not the deferrable UI-heavy half of the M2/M3 split — a feature with many steps does not fit one session's context, so it spans sessions, and a folder is what holds them; every harness surveyed offers a flat session list at best sortable by repo location, so this is a differentiator, not a nicety. Couplings recorded: the same multi-session feature shape **A3's hand-over/resume** amendment serves, and **B8's** answer puts such a feature's worktree in §5.4 workflow instructions. ~~Approved, not yet applied — owes A14~~ ✅ **applied 2026-09-18 via A14** → ten spec sections rather than the two the delta named (§3.1/§3.2/§3.3/§3.5/§3.7/§4.3/§4.7/§8.4/§12/§14), plus both fixes and the **names carve-out** in doc 06 | ~~high~~ | ~~06~~ ✅ |
| ~~**B14**~~ | ~~Prompt/context buffer~~ | ✅ **SIGNED OFF 2026-09-17** — appender + segment index taken as recommended (`Segment { kind, offset, len, digest }`): cutting later parts is **O(1) truncation**, re-parsing is never required, and a model/provider switch **replays the segment list through the new encoder**. Also settles §5.6's environment-block placement — after the cache-stable portion, excluded from the fingerprint. ~~**Approved, not yet applied — owes** folding into **A4**'s §4.5 text~~ ✅ **applied 2026-09-18 inside A4**: `Segment`/`ContextBuffer` normative in §4.5, and both predicted consequences load-bearing — §4.5's resync rule and §5.6's placement — plus a third A4 added, §4.8 citing the index for why a status flip costs one segment. **Docs 09 and 17 still owe the plan-side reference**, blocked by hard rule 1 (both still summaries) | ~~high~~ | ~~09, 17~~ ✅ |
| ~~**B16**~~ | ~~Repository tooling language~~ | ✅ **SIGNED OFF 2026-09-16, narrowed.** Not "Python now, Rust `xtask` from M0" but **two classes**: tooling the shipped harness depends on is Rust in `xtask` (B6's schema generator); repo-*authoring* tooling stays a script **permanently**, not pending a port. All five tools stay Python. `xtask` is still created at M0 | ~~high / med~~ — the med half was resolved by measurement, not by vote | ~~01, 31~~ ✅ |
| ~~**B17**~~ | ~~The built-in `reviewer`~~ | ✅ **SIGNED OFF 2026-09-17** — option (c), the split: `reviewer` stays an ordinary agent file and the engine gains **reason-tagged changeset selection**, **`finding` as a durable structured event with no line numbers**, and the **snippet-anchoring positioning module**; no LLM grouping in v1; the reflection filter ships defaulted off; review checklists are skills. **Amendment recorded as a requirement on the future delta**: the reviewer's *usage* must be defined by its system prompt **and by at least one pre-defined workflow** (§5.4), so how it is invoked is shipped content rather than something each user reinvents. Still **brainstorm-gated twice over** — signing releases neither the built-in-tools (M1) nor built-in-agents (M2) session, and the delta is deliberately **not written yet** because all three sections it would edit sit inside those gates; `finding` is durable, so it lands **after A1** | ~~high (positioning + structured findings) / med (where the pipeline stops)~~ — the med fork taken as recommended: deterministic bundling first, revisit only on a B18 measurement | ~~14, 17, 11~~ ✅ |
| ~~**B18**~~ | ~~Harness benchmarking~~ | ✅ **SIGNED OFF 2026-09-17** — **four tiers, three adopted**, plus the publication rule (no number without an in-tree re-runnable scorer and a recorded environment): **T1** deterministic replay benches in CI from M1 over a committed `events.jsonl` corpus — **this produces B12's missing number**; **T2** token/cost led by prompt-cache hit rate (needs A4's `Usage` cache fields); **T3** task-success runner at M5 and never in CI; **T4** cross-harness comparison an explicit non-goal. **Approved, not yet applied — owes A21** → §14 split, doc 33 deepened when M1 starts, the T1 harness added to M1's task list, and **B12 re-run as a measurement** | ~~high (T1/T2/rule) / med (T3 timing)~~ — the med fork taken as recommended: T3 at M5, earlier if agent-quality regressions bite during M2 | ~~29, 31, 33~~ ✅ |
| ~~**B15**~~ | ~~Built-in task tracking & gating~~ | ✅ **SIGNED OFF 2026-09-13** — all five recommendation points plus all four consequences, none redirected. Applied via **A19** → new design **§4.8** + six consequential edits. **Gating strength landed as "hard but bounded"** — the low-confidence taste fork, taken as recommended. One gap found while applying is now open decision §16.6 | ~~high / low~~ | ~~32, 03, 08, 14, 18~~ ✅ |
| ~~**B19**~~ | ~~Identifier privacy~~ | ✅ **SIGNED OFF 2026-09-16** — options **2 + 4** taken as recommended: inject the ids a tool needs as env vars under a reserved prefix, an opaque scratch-dir handle, an exact `args` check restricted to the four ULID-backed ids, and redaction only on §8.2's config + four *projection* endpoints plus §7.1's hook payloads. ✅ **Applied via [A23](A-design-deltas.md) 2026-09-16** — normative at §4.3/§4.4/§5.1/§7.1/§8.2/§12 | ~~high / med / low~~ | ~~14, 16, 24~~ ✅ |
| ~~**B20**~~ | ~~Remote agent execution & swarms~~ | ✅ **RESOLVED 2026-09-17 — the session happened.** Survey run (the sixth digest), nine questions answered, and the five the survey raised settled. **Option 2 in B9's shape: commit the constraints now, build at M5.** Model access is **option A — a credential broker at the controller** (the member runs `dacode-providers`, points its base URL at the controller, which injects the key and forwards **verbatim**); a dead member **blocks and asks**, with retry-on-host-loss only as a per-step opt-in that must declare its effect class; orchestration is a **§5.4 DAG over one shared §4.8 task tree** with its own remote cap; and the survey produced a **fifth constraint — a fencing token** (epoch + idempotency key + leaf CAS), which is wire-visible and so lands **with A1 at M1**. Forge's *remote services* form is declined: A10 covers it and hosted indexing is a design non-goal. Owes **A25** | ~~med — the shape is high-confidence, the checkout/log-locality/sandbox answers are open~~ → the three open answers are now given | ~~03, 11, 19, 20, 24~~ ✅ |
| ~~**B21**~~ | ~~Session-log concurrency & indexing~~ | ✅ **SIGNED OFF 2026-09-17** — the recommendation **and all five sub-answers** as recorded, none redirected. **One writer, one lock, an index that is a projection**: exactly one process appends to a session's log, holding an advisory `flock` on `<session>/lock` that the kernel releases on crash, so there is **no stale-lock recovery and no force flag**; everyone else is a *client* over §3.7 (`attach`), which is principle 1's job. The log stays append-only JSONL and stays the truth; the index is a rebuildable sidecar (`cursor → byte offset` at M1, SQLite/`redb` at M3 per B11). **Rewrite is not an operation** — the same fact that makes B19's prevention necessary. Applied via **[A24](A-design-deltas.md)** → §3.6/§4.1/§8.1/§12/§14 + doc 05 §5.5, and it **discharged U67** on the way. ⚠️ **Sub-answer 5's exit-code half is superseded 2026-10-04 by [U93](../TODO.md)'s ruling** (arm (b): a held session exits `4`, design §11) — the fail-fast/holder-named/`attach` machinery the sub-answer actually reasoned about stands unchanged; see the annotation at the sub-answer's site below | ~~high / med~~ — the med half is now a recorded exit route rather than an open question | ~~05, 07, 19~~ ✅ |

Four decisions changed shape because of the new evidence: **B5** (a pure skip
is not available — Harmony escaping is required for *native* tool use on
gpt-5.x), **B7** (zero precedent for OS sandboxing anywhere, so the worktree
sandbox is the real boundary), **B8** (opencode independently built dacode's
§4.6 design, which is the strongest possible confirmation — and it exposes six
edge cases §4.6 does not mention), and **B12** (reopened by the
[A17](A-design-deltas.md) pure-Rust dependency policy: `zstd` links C and no
pure-Rust zstd *encoder* exists).

Four entries were amended after drafting rather than rewritten, so the original
reasoning stays visible: **B6** (drop the `serde_json` dependency via offline
codegen), **B1**, which gained the clarification that `'static` is a
dacode-protocol requirement and does not constrain the JSON crate or its
embedded `no_std` users, **B4**, which gained the backpressure/lag and
seq→gap→resync contract that its own recommendation silently assumes, and **B7**,
which gained the 2026-09-16 configuration-polarity banner and a fifth
recommendation part on the user's sandboxing note.
**B14** (prompt/context buffer representation), **B15** (built-in task tracking
and gating), **B16** (repository tooling language), **B17** (the built-in
`reviewer`), **B18** (harness benchmarking), **B19** (identifier privacy) and
**B20** (remote agent execution and swarms) are new — B14 raised during
sign-off review, the rest by user requirements added after the original pass.
**B19 and B20 arrived together on 2026-09-16, the day M0 started**, and they are
coupled rather than merely adjacent: a swarm that runs agents on N hosts retains
every identifier in N append-only logs, so B19 is what makes B20 safe to log.
**B20 is the second entry after B12 that is not a plain vote** — it needs a
reference survey and a design session first, which is declared in the
`attention-meta` block above rather than left for a reader to infer from prose.
**B17 and B18 share one piece of evidence**, the fifth reference survey
([open-code-review](../../../.agents/knowledge/reference-harnesses/open-code-review.md)):
its review architecture is B17's positive evidence, and its *unreproducible
benchmark* is B18's negative one. **B15's own framing was overturned by
its evidence**: it was scoped on the assumption that no harness gates on task
tracking, and the
[task-tracking-and-gating](../../../.agents/knowledge/reference-harnesses/task-tracking-and-gating.md)
digest found two that do (forge and oh-my-pi, both mechanically). What *is*
unprecedented is gating on **evidence** — no harness in the set has a
verification method, an evidence field, or a `check` that fails. That asymmetry is
the decision.

---

## Detail

### 1. [B1] JSON strategy — ✅ **RESOLVED 2026-09-12: hybrid adopted**

**Gates** [02](../part1-foundations/02-json-strategy.md),
[03](../part1-foundations/03-protocol.md) · **Unblocks** M0 and the protocol
crate · Original entry: *hybrid recommended — serde-on-our-types for control,
`RawJson` for opaque sub-JSON, `flat`/zstd for the log, `pull` for inspection.
Still awaiting sign-off; blocks the protocol crate.*

> **✅ RESOLVED 2026-09-12 — hybrid adopted; delta A16 applied to §13.**
>
> Signed off together with **B6**, which the user delegated alongside it,
> because A16 depends on both: its item 4 records `schemars`' isolation
> boundary, and that is B6's content. §13 is rewritten as 13.1 tiers · 13.2
> per-surface table · 13.3 the serde rule re-read · 13.4 byte-exactness · 13.5
> `Codec` + conventions. B6's other half — recording the generator in
> [14](../part3-capabilities/14-tools.md) — is **deferred to M1**, because doc
> 14 is still at summary depth and deepening it now would violate the
> just-in-time rule.
>
> **"Could the library be changed instead?" — asked at sign-off, answered from
> source.** Option (b) is *not* impossible, and the wording below overstates the
> case: "(b) breaks prompt caching" holds only for a **normalising** tree.
> `Pool` is a `Vec<Node { tag: u64, payload: u64 }>` with strings as
> `(offset, len)` into a separate buffer (`src/pool.rs:36,72`), so it carries no
> lifetime — only `Doc<'a>`'s `&'a [u8]` does. An owned
> `OwnedDoc { bytes: Box<[u8]>, pool: Pool }` would therefore erase the lifetime
> in a few lines **and stay byte-exact**, since the pool indexes the original
> bytes rather than normalising them. Three reasons it is still the wrong move:
>
> 1. **The library fix already shipped, and it is `RawJson`** (B1a, `90def19`).
>    Owned `Box<str>`, `Clone + Send + Sync + 'static`, validated so the verbatim
>    splice cannot corrupt the host document (`src/raw.rs:79`). Its own doc
>    comment states the `'static` rationale this decision turns on.
> 2. **An owned tape is dominated for dacode's access pattern.** On the opaque
>    path dacode stores `args`, re-emits them verbatim, and reads *one* field
>    once at permission time. `RawJson` + `pull::select` covers that with **2
>    inherent allocations against the tree's 114 080** at 853 KB. An owned tape
>    buys cheap *repeated* traversal of a large document, which dacode does not
>    do — and the one surface that would (introspection scanning large bodies) is
>    already assigned to `pull` in
>    [02 §2.4](../part1-foundations/02-json-strategy.md).
> 3. **It would not retire the hybrid anyway.** The control-type half —
>    `Event`/`Command`/`Snapshot` as serde-derived structs — is independent of the
>    owned-tree question. Those are dacode's own types with known shapes; a
>    generic tree would trade compile-time guarantees for runtime field access.
>
> Note also that the crate already *has* a self-contained owned form: that is
> what the `flat` tier is (`src/flat/mod.rs` — strings moved inside the buffer,
> escapes decoded once at build time), and A15/A16 already use it for the log.
> So an owned representation is not a missing feature; it exists at two points on
> the cost curve, and the hybrid picks per surface.

- **[B1a] DONE** — `dacode_json::RawJson` is implemented, benchmarked and verified
  behind an opt-in `raw_value` feature (byte-exact splice on serialize;
  byte-exact capture on the positional path, canonical on the pool path). The
  default `dacode-json` build is unaffected (202 tests + clippy green). See
  [02 §2.3](../part1-foundations/02-json-strategy.md). What remains for
  B1 is confirming the *overall* strategy, not the `RawJson` precursor.

**Options.** (a) Literal §13 — `dacode_json::Value` everywhere, no serde.
(b) Add an owned `Value` enum to `dacode-json`. (c) Depend on `serde_json`.
(d) The hybrid in [02 §2.4](../part1-foundations/02-json-strategy.md).

**Why (a)–(c) are not available.**

- **(a) is factually impossible.** `dacode-json` has no owned value type — this is
  the §13 discrepancy recorded as delta A11. `Value<'a>`/`Doc<'a>` are borrowed
  from the input slice plus a `Pool`; `Value::to_json()` re-serializes to a
  `String`. A `ToolCall { args }` inside an `Event` must be
  `Clone + Send + Sync + 'static` (stored in the log, broadcast to N clients,
  persisted) — a borrowed `Value<'a>` cannot live in an event ([02 §2.2](../part1-foundations/02-json-strategy.md)).
- **(b) breaks prompt caching.** Re-serializing an owned tree can reorder keys
  and reformat floats. Byte-exactness is a *correctness* property for the
  cache stable-prefix discipline ([09](../part2-engine/09-context-caching.md)),
  not a nicety.
- **(c) violates the single-JSON-engine principle** and buys nothing `dacode-json`
  does not already do.

**Evidence for (d).** The per-surface table is in
[02 §2.4](../part1-foundations/02-json-strategy.md). `RawJson` is
measured, not hoped-for: splice is **≈110× faster than re-encoding** the same
tree (57–59 GiB/s vs 462–532 MiB/s at 853 KB), the win is **per-byte** rather
than per-value, and **capture is 1.2–1.5× cheaper than building a tree** with
**2 inherent allocations against the tree's 114 080** — while staying
byte-exact, which the tree is not.

**Recommendation: adopt (d).** The load-bearing part is the re-reading of §13's
"serde isolation" rule: its *intent* is to keep **third-party** serde types out
of `dacode-protocol`/`dacode-core`. dacode's **own** types deriving
`serde::{Serialize,Deserialize}` and going through `dacode-json` is the intended
path, not a violation. Without that re-reading the rule forbids the only
strategy that works.

**Clarification asked for during review — does this force `'static` on the JSON
crate, and does it threaten the embedded use case?** No to both. `'static` is a
*dacode-protocol* requirement — an `Event` is stored in the log, broadcast to N
clients and persisted, so what it owns cannot borrow from a parser's input. It is
not a requirement on the JSON crate, which already ships both halves: `RawJson`
is owned and `'static`, while the DOM (`Doc<'a>`/`Value<'a>`) and `pull` tiers are
borrowed and zero-alloc.

`candroid-fw` is the proof the borrowed path is real and unaffected. It depends on
the crate with `default-features = false` (`fw/Cargo.toml:45`) — **no `std`, no
`alloc`, no `serde`** — on `thumbv7em-none-eabihf` (Cortex-M4F, ATSAME51J19A, the
Feather M4 CAN), reading config with `pull::object` / `pull::for_each_object` over
borrowed `pull::Raw` spans walked in place (`fw/src/main.rs:2529,2572`;
`crates/config/src/parse.rs:5-6,35`). And since `raw_value` requires `serde`,
which requires `alloc`, **that build cannot even enable `RawJson`** — so adopting
B1 adds nothing to the embedded target and takes nothing from it.

**On sign-off:** apply delta **A16** (which supersedes A11's literal reading) to
§13; flip [02](../part1-foundations/02-json-strategy.md)'s status to
*decided*; strike the B1 marker here and in doc 02.

---

### 2. [B2] Web stack — ✅ **SIGNED OFF 2026-09-17, APPLIED 2026-09-18 via A8: the split — schema export a non-optional M1 item, framework due by M3**

> **✅ SIGNED OFF 2026-09-17** — *"let's do as you recommend."* **The split taken as
> recommended**: **(1)** `dacode-protocol` ships a **JSON-Schema export** as a
> non-optional **M1** item — it is a property of the protocol crate and benefits
> extension authors, a future SDK, `run --output jsonl` consumers and the
> golden-fixture suite regardless of framework; **(2)** the framework decision gets a
> **deadline of M3** with **TS SPA (Solid)** standing, mitigated by generating
> `protocol.d.ts` in CI and failing on drift. Worth stating explicitly, because the
> split is what makes it honest: TODO §5's M1 list already carried "JSON-Schema
> export" while doc 03 called it optional.
>
> ~~**The delta is approved and owed, not applied** — nothing from this sign-off has
> touched `docs/dacode-design.md` yet.~~ ✅ **Applied 2026-09-18 via
> [A8](A-design-deltas.md)**: §9.1 records the split with the framework still **open but
> deadline-bound to M3** and TS SPA (Solid) as the *standing recommendation* rather than a
> decision, and **the optionality is removed from
> [03](../part1-foundations/03-protocol.md)** — whose line was a false claim rather than a
> stale marker, so it was corrected under hard rule 1 with its summary marker untouched.
>
> **Two things the application settled that this entry did not.** First, §9.1 now says
> *why* the old conditionality was backwards rather than merely removing it: the export is
> what makes the TS branch **cheap**, so requiring the framework decision first would have
> made that decision harder rather than easier — which is the argument that makes "a
> deferral with a committed prerequisite" a real resolution instead of a punt. Second, a
> **contradiction this entry could not have seen**: §9.1's old text had a `build.rs`
> emitting the export *inside `dacode-protocol`*, while §13.3 confines `schemars` to an
> offline generator ("never in a shipped crate") and §2.4 puts build-time codegen in
> `xtask`. Resolved in favour of §13.3/§2.4 — the generator is `xtask`'s, the protocol
> crate grows no `build.rs`, and `protocol.d.ts` is generated **in CI with a drift check
> that fails the build**, layered on the golden fixtures. This entry's "mitigated by
> generating `protocol.d.ts` in CI and failing on drift" is thereby implemented, and it
> turned out to need the `xtask` placement to be coherent at all.
>
> **Still owed, unchanged:** the framework decision itself, by M3, and
> [21](../part4-hosts-clients/21-web-ui.md)'s plan-side write-up (still a summary, so
> §9.1/§9.2/§9.3 are its normative source).

**Gates** [21](../part4-hosts-clients/21-web-ui.md),
[19](../part4-hosts-clients/19-server-gateway.md) · **Blocks** M3 web UI ·
Original entry: *Leptos/WASM vs TS SPA (Solid recommended). Blocks M3 web UI.*

**The observation that changes this decision:** it is not one decision. Two
things are entangled and they gate different milestones.

1. **Does `dacode-protocol` export a machine-readable schema of its own types?**
   This gates **M1**, because it is a property of the protocol crate. And the
   answer is *yes regardless of framework* — extension authors, a future SDK,
   `dacode run --output jsonl` consumers and the golden-fixture suite all
   benefit. [03](../part1-foundations/03-protocol.md) currently lists it as
   "Optional … if the web UI is a TS SPA". It should not be optional.
2. **Which framework renders the UI?** This gates **M3** only, and §9.1 already
   records that *"Either way the server is unaffected."*

Splitting them lets you defer the genuinely taste-driven half without blocking
anything.

**Evidence on the framework half.** All three reference web UIs chose
TypeScript: opencode a Solid SPA (Vite + Kobalte + TanStack virtual), oh-my-pi
`collab-web`, openclaw Lit + CodeMirror. The four hard problems in this UI are
**streaming markdown, syntax highlighting, virtualized lists and diff
rendering** — and the harnesses solve them with worker-offloaded streaming
markdown + shiki and `@pierre/diffs` (opencode `session-ui`). Leptos'
fine-grained reactivity genuinely suits streaming, but the Rust/WASM ecosystem
for those four is thin, and bundle size works against the mobile-first goal in
§9.2.

**Recommendation.** Commit now to **(1) yes — the protocol crate ships a
JSON-Schema export**, and set **(2) a decision deadline of M3**, with **TS SPA
(Solid)** as the standing recommendation. Mitigate the type-drift risk that is
the real cost of (2): generate `protocol.d.ts` in CI and fail on drift, on top
of the golden fixtures that already pin the wire format (§14).

**Confidence: medium on facts, low on taste.** If single-language purity is
weighted heavily, Leptos is defensible and the schema export is still worth
building. Say so and I will write it up the other way.

**On sign-off:** ~~apply delta **A8** to §9.1 recording the split (schema export
committed, framework due by M3); make the schema export a non-optional M1 item
in [03](../part1-foundations/03-protocol.md).~~ ✅ **Done 2026-09-18**, both halves,
plus the `xtask` placement that made the CI drift check coherent with §13.3 and §2.4.

---

### 3. [B3] Delta vs full message updates — ✅ **SIGNED OFF 2026-09-17: pure deltas, full value at the durable `*Ended` boundary**

> **✅ SIGNED OFF 2026-09-17** — *"let's do pure deltas (how you recommend)."* Taken
> as recommended: **deltas live, full value at the durable `*Ended` boundary** — the
> `TextEnded`/`ThinkingEnded` counterparts that make replay lossless, so a client
> that missed every delta, or replays from the log a year later, renders correctly
> from durable events alone. As the entry already argued, this is less a choice than
> a consequence of accepting A1's two-tier model, and the sign-off treats it as such.
>
> ~~**The delta is approved and owed, not applied.**~~ ✅ **Both halves are now
> discharged.** Owes: **A1** → §3.2/§3.7
> including the `*Ended` boundary variants — and A1 must land **before any golden
> fixture**, the sequencing rule every protocol-touching sign-off in this pass
> repeats — plus the note in
> [03](../part1-foundations/03-protocol.md) that B3 is subsumed by the tier split.
> **A1 applied 2026-09-18** with the precondition intact (the crates were still stubs,
> so no fixture was retrofitted), and **the doc-03 note landed 2026-09-19 at M1.1a** —
> in its §3.4, beside the correction of the doc's stale tier marker, which offered
> `Event::tier()` *or* a `durable: bool` as an open choice when A1 had taken the first
> and taken it as a rule.

**Gates** [03](../part1-foundations/03-protocol.md),
[08](../part2-engine/08-agentic-loop.md) · Original entry: *pure deltas
(current §3.2) vs oh-my-pi's "full accumulating message" per update. Affects
fold and wire volume.*

**Options.** (a) Pure deltas — §3.2's `TextDelta { text }`, appended
client-side. (b) oh-my-pi's `message_update` carrying the **full accumulating
message** each time.

**Evidence.** opencode runs (a) *plus* durable `Ended` full-value boundaries,
and states the rule explicitly: "Stream fragments are live-only; Text.Ended is
the replayable full-value boundary" (`schema/src/session-event.ts:209`). That is
exactly dacode's two-tier model (delta A1, [00-overview §0.4](../00-overview.md)).
oh-my-pi runs (b) — no client-side delta math, no ordering hazards.

**Why (b) is wrong for dacode.** Its wire cost is **quadratic**: a 100 KB
assistant message streamed in 1 KB chunks sends ~5 MB. Over a multiplexed WS to
N observers (§8.4) that is material, and it scales with the exact thing dacode
is good at (long tool-heavy sessions). Its simplicity is real but it is
*client*-side simplicity, and dacode already pays that cost once — the fold is
shared code in `dacode-protocol` (§3.5), written once for TUI, web and
extensions.

**Recommendation: (a), with the boundary made explicit.** Live tier = deltas.
Durable tier = a `TextEnded { turn, text }` (and `ThinkingEnded`) carrying the
**complete** value, so a client that missed every delta — or replays from the
log a year later — renders correctly from durable events alone. This is the
property that makes replay lossless, and §3.2 as written does not guarantee it:
`TextDelta` is in the enum with no full-value counterpart.

**Confidence high.** B3 is less a choice than a consequence of accepting A1.

**On sign-off:** apply delta **A1** to §3.2/§3.7 including the `*Ended`
boundary variants; note in [03](../part1-foundations/03-protocol.md) that B3 is
subsumed by the tier split.

---

### 4. [B4] WS vs SSE for the live channel — ✅ **RESOLVED 2026-09-12: keep WS**

> **✅ RESOLVED 2026-09-12 — WebSocket stays the single live channel; SSE is not
> shipped as a second live transport in v1.** The recommendation below was
> accepted as written.
>
> **The robustness half was settled separately, and first.** Delta
> [A18](A-design-deltas.md) — bounded fan-out, drop-oldest, and the
> seq→gap→resync contract — was signed off and applied to §3.7 alongside this.
> That is the split this entry's own amendment argued for: the correctness
> invariant is transport-independent, so landing it first reduced B4 to the pure
> transport choice, which is the part that was genuinely a taste call.
>
> **No spec change was needed for the transport itself.** §3.6 already specifies
> WebSocket (one connection per client, multiplexed over sessions, up frames
> `{cmd_id, command}`, down frames `Envelope | Ack`), so resolving B4 *confirms*
> existing text rather than rewriting it. What this decision changed is the
> status of that text: from "specced but contested" to decided.
> [19](../part4-hosts-clients/19-server-gateway.md)'s `[DECISION]` marker is
> flipped to resolved; the doc itself is not deepened, being M3 and still at
> summary depth.
>
> **Dissent recorded, not deleted.** SSE remains materially simpler through
> proxies and browser-native, and opencode proves the shape works for exactly
> this product. The decision is to revisit only on a concrete deployment
> failure — and if that happens, it is a larger rewrite of §3.6 and §8.2 than any
> other decision here, so it should be flagged early rather than absorbed.
>
> **One follow-up this creates, logged in [TODO](../TODO.md) §6: decide the HTTP
> version deliberately.** The ~6-concurrent-connections-per-origin browser cap
> under HTTP/1.1 does not only affect SSE — several dashboard tabs can also
> exhaust it against the §8.2 HTTP endpoints that carry files, diffs and blobs,
> whichever live transport is in use. Serving HTTP/2 removes the cap. That is a
> small server-configuration decision, but it should be made rather than
> inherited from whatever the HTTP crate defaults to.

**Gates** [19](../part4-hosts-clients/19-server-gateway.md),
[03](../part1-foundations/03-protocol.md) · Original entry: *keep WS (specced)
or adopt SSE (opencode).*

**Evidence.** A genuine split among the references. **opencode: HTTP + SSE, no
WebSocket** — a global `GET /api/event` stream plus per-session
`GET /api/session/:id/event?after=<seq>` ("replay durable events after an
aggregate sequence, then continue with new"), with commands as **plain POSTs**
and a 15 s heartbeat merged into the stream. **openclaw: WebSocket** frames with
monotonic `seq` + `stateVersion` and client-side gap detection → resync
(`gateway-client/src/protocol-client.ts:413-425`).

**Why WS for dacode.** §3.6 specifies a **bidirectional multiplexed** channel:
up frames `{cmd_id, command}`, down frames `Envelope | Ack`, one connection per
client multiplexed over sessions. SSE is server→client only, so adopting it
means splitting commands onto separate POSTs — which is a *different* protocol
shape, not a transport swap, and it complicates `Ack` correlation (§3.3) and the
controller/observer model (§8.4). `dacode attach <url>` (§8.1) also wants a
single socket. And §8.2 already captures most of SSE's benefit: *"Large payloads
(files, diffs, blobs, history pages) are HTTP; the WS carries only live
traffic."*

**Why this is the least certain recommendation here.** SSE is materially simpler
through proxies and is browser-native (`EventSource`, automatic reconnect). If
dacode's realistic deployment is "behind a corporate proxy or a Caddy/Tailscale
front end" (§8.3 already recommends that for `--bind 0.0.0.0`), WS-through-proxy
pain is a real operational cost, and opencode proves the SSE shape works for
exactly this product.

**Recommendation: keep WS as the single live channel, and adopt openclaw's
seq-numbered events + client-side gap detection → resync as the robustness
contract.** Explicitly **do not** ship SSE as a second live transport in v1: two
live channels means two fold paths, two replay contracts and two sets of golden
fixtures, for a fallback nobody has yet needed. Revisit only on a concrete
deployment failure.

**Amended after a "what is best for fast notifications?" question exposed two
gaps in the spec text — not in this recommendation.** Both concern *what makes
drop-on-lag safe*, and neither is stated anywhere normative:

1. **The backpressure/lag policy is in the plan but not in the spec.** §4.1 says
   "a broadcast channel per session" and stops. `tokio::sync::broadcast`
   **drops the oldest messages when a receiver lags** and surfaces
   `RecvError::Lagged` — so the primitive §4.1 already picked *has* a lossy
   slow-consumer behaviour, which makes §3.7's flat "No gap, no duplicate" claim
   false as written. [07-session-registry](../part2-engine/07-session-registry.md)
   does call for "a bounded queue and explicit drop/backpressure policy … a slow
   client must not stall the engine" (from openclaw's `allBounded` capacity-256
   subscriber queue), but a summary-depth plan doc is not where an implementer
   looks for a correctness invariant. **The drop policy belongs in §3.7, beside
   the "no gap" claim it qualifies.**
2. **The contract that makes dropping safe is never written as one invariant.**
   The pieces exist and compose: bounded fan-out → drop on lag → monotonic
   per-session `seq` (§3.2's `Envelope.seq`) → client-side gap detection →
   resync from the log via `Subscribe { since }` (§3.7). That chain is what turns
   a *lossy channel* into a *lossless view*, and it is currently spread across
   §3.7, §4.1, doc 07 and this entry. State it once, in §3.7, as the
   reconnect/replay contract. **This is also why A1's resync half is not
   optional** — without gap detection, a bounded queue silently loses events
   rather than trading staleness for liveness.

**Both are transport-independent, which is itself the useful finding: they can
be settled before B4 rather than after.** Whoever wins between WS and SSE needs
the same seq/gap/resync invariant (opencode reaches it with `?after=<seq>` plus
`EventSource`'s automatic reconnect; openclaw with WS frames plus explicit
detection). So the correctness contract should be written now and *not* wait on
what is, on this entry's own admission, the most taste-driven call in the set.

**Split out as delta [A18](A-design-deltas.md)** (§3.7 + §4.1, paste-ready spec
text, depends on nothing). **Signing off A18 discharges this half of B4** and
leaves B4 as the pure transport choice it should have been — and A18 also
subsumes A1's §3.7 resync bullet, narrowing the most structural delta in the set.

**One further asymmetry the evidence above omits, in WS's favour.** Browsers cap
concurrent connections per origin (~6 under HTTP/1.1). SSE holds one open stream
*per tab*, so several dashboard tabs can exhaust that budget and stall every
other request to the server — including the §8.2 HTTP endpoints that carry
files, diffs and blobs, which is the traffic the "large payloads are HTTP" split
depends on. Mitigable by serving HTTP/2 or funnelling tabs through a
`SharedWorker`, but both are extra machinery SSE needs and WS does not. Note the
cap does not apply if the server speaks HTTP/2, so this is a reason to decide the
HTTP version deliberately, not a decisive argument on its own.

**On sign-off:** ~~note the decision in
[19](../part4-hosts-clients/19-server-gateway.md)~~ ✅ *done — its `[DECISION]`
marker is flipped to resolved, the doc not deepened (M3, still at summary
depth)*. ~~Apply delta A1's resync half to §3.7~~ — **that work moved to
[A18](A-design-deltas.md)** ✅ *applied the same day*, carrying the
bounded-fan-out/drop-on-lag policy and the seq→gap→resync invariant as
paste-ready §3.7 text, replacing the unqualified "No gap, no duplicate" with the
conditions under which it holds. §3.6 needed no edit: it already specified WS.
**Owed:** the HTTP-version follow-up in the banner above.

---

### 5. [B5] In-band tool dialects in v1 — ✅ **SIGNED OFF 2026-09-17, APPLIED 2026-09-18 via A5: the ~700 LoC subset ships, the dialect matrix deferred**

> **✅ SIGNED OFF 2026-09-17** — *"a recommended approach."* Taken as recommended:
> v1 ships the **~700 LoC subset** — Harmony control-token escaping on the transport
> copy (a correctness fix for the `openai-compat` path, not a dialect feature), a
> `supports_tools: false` capability flag driving history demotion, and **one XML
> grammar** with stream holdback plus a tool-name allowlist before promotion. The
> other ten dialects and per-request history re-encoding are **deferred**. The two
> carve-outs stand as follow-ups, because they are not B5: **malformed tool-arg JSON
> repair is a different axis** belonging to
> [14](../part3-capabilities/14-tools.md), and **repair never scrubs inside fenced
> or indented code**.
>
> ~~**The delta is approved and owed, not applied.**~~ ✅ **Applied 2026-09-18 via
> [A5](A-design-deltas.md)** → §6, with the whole subset normative: Harmony escaping
> **on the transport copy only** (and §6 now says *why* the persisted transcript keeps the
> original — §13.4 makes byte-exactness end-to-end a correctness property and §3.6 makes
> the log append-only, so escaping the stored copy would silently rewrite history to match
> a transport quirk), `supports_tools: false` driving **history demotion rather than an
> error**, one XML grammar with stream holdback, the tool-name allowlist checked **before
> promotion**, and the never-scrub-inside-code rule.
>
> ⚠️ **One instruction in this banner was stale and was deliberately NOT followed.** It
> said A5 "must also add the missing numbered decision — §16 has no B5 entry today". But
> §16 is titled **Open decisions** and B5 is **resolved**, so adding a row would have put a
> closed decision in the spec's open list — and **A19 set the precedent explicitly** when
> its applied note recorded that "§16 gained no B15 row (it lists *open* decisions)". The
> concern behind the instruction was legitimate and is answered differently: the spec
> assumed native tool use everywhere with no record that dialects were *considered and
> deferred*, so **the deferral now lives in §6's prose**, where a reader of the provider
> section actually meets it, and this appendix keeps the decision. A12 owns §16's
> bookkeeping and applies last.
>
> **What A5 added beyond this entry.** The capability model the redirect asked for turned
> out to be **already mostly built** — A4 had put `supports_tools`, `supports_thinking`
> and `supports_cache_breakpoints` on `ModelInfo` the day before, because its four-rate
> cost rule needed the struct — so A5 extended it with the one genuinely new capability,
> **`ToolDialect { Native, XmlGrammar }`**, as a *reserved slot*: this entry's subset
> makes the grammar reachable only when `supports_tools` is false and v1 ships exactly
> one, so the value is derived rather than chosen, and the point of the enum is that
> **un-deferring the ten dialects adds variants instead of restructuring the engine**.
> `CompletionRequest` and `StreamChunk` were also defined — §6's trait had referenced both
> without either existing anywhere. **Both carve-outs stand unchanged**: JSON repair is
> still **U10**'s and still a tools axis, and §6 now records that exclusion where a reader
> meets the dialect rules rather than leaving it only here.

**Gates** [13](../part3-capabilities/13-providers.md) · Original entry: *include
tool-call repair/dialects or native-tool-calling providers only.*

Full evidence:
[dialects-and-acp §Part 1](../../../.agents/knowledge/reference-harnesses/dialects-and-acp.md).

**The cost range is enormous.** opencode: **zero** — no dialects, no repair,
restricts itself to native-tool-calling providers. forge: **one** dialect behind
a capability flag (~1.7 k LoC Rust pipeline). openclaw: **4 grammars**, 3 562
LoC, **response-side only** with no history re-encoding, applied per-provider
(ollama/lmstudio/xai) rather than auto-per-model. oh-my-pi: **11 dialects**,
~6.9 k LoC, fully bidirectional with per-request history re-encoding.

**The finding that decides it.** Harmony **control-token escaping is required
even for native tool use** against gpt-5.x / gpt-oss: the backend rejects
reserved control-token spellings appearing as *data* with `invalid_prompt` /
"Request blocked" (oh-my-pi `harmony-leak.ts:22-26,33`). Only the transport copy
is escaped; the persisted transcript keeps the original. **That is not a dialect
feature — it is a correctness fix for the `openai-compat` path dacode already
commits to in §6.** So "native-tool-calling providers only" does not actually
avoid this work.

**Recommendation: ship the ~700 LoC subset in v1, defer the matrix.**

v1 (in order of necessity):
1. **Harmony control-token escape/unescape** on the transport copy (regex,
   under 100 LoC). Required for correctness, not a feature.
2. **A `supports_tools: false` capability flag driving history demotion** —
   strip `tool_calls` from assistant messages, rewrite tool results as user
   messages (forge `transform_tool_calls.rs:26-77`, ~200 LoC Rust).
3. **One XML grammar** with stream holdback, plus a **tool-name allowlist check
   before promotion** (openclaw `contracts.ts`). oh-my-pi's anthropic/hermes/
   qwen/dsml variants all reduce to this with a tagset switch — they made it a
   single option (`dialect/types.ts:58-59`).

Deferred: the other ten dialects, per-request history re-encoding (oh-my-pi's
~6.9 k LoC is the price of full market coverage), fabricated-tool-result abort,
native/in-band channel de-confliction.

**Two things to record separately, because they are not B5.**

- **Malformed tool-arg JSON repair is a different axis.** forge carries a
  *separate* 3 694-LoC `forge_json_repair` crate (parser 1 073 + schema
  coercion 1 397) for models that emit almost-valid JSON in `args`. dacode will
  likely need some of this regardless of B5, and it belongs in
  [14](../part3-capabilities/14-tools.md) (args parsing), not
  [13](../part3-capabilities/13-providers.md) (transport). Log it as a new
  follow-up rather than folding it in.
- **Never scrub inside code regions.** openclaw protects fenced and indented
  code before promoting (`provider-stream-shared.ts:186-187`). A repair pass
  that rewrites a shell snippet in a markdown block is worse than no repair.

**On sign-off:** ~~apply delta **A5** to §6 (which must also add the missing
numbered decision — §16 has no B5 entry today); record the two carve-outs above
as follow-ups in [TODO](../TODO.md).~~ ✅ **Done 2026-09-18**, to four sections rather
than the one named here (§4.3, §4.4, §6, §14), **except the §16 row, which was
deliberately not added** — §16 lists *open* decisions and B5 is resolved, per A19's
precedent; see the banner. The two carve-outs were already logged (**U10** for JSON
repair) and the never-scrub-inside-code rule landed as normative §6 text rather than
staying a follow-up, since it is a rule an implementation can violate silently.

---

### 6. [B6] Tool schema generation — ✅ **RESOLVED 2026-09-12: offline codegen**

**Gates** [14](../part3-capabilities/14-tools.md),
[02](../part1-foundations/02-json-strategy.md) · Original entry:
*`schemars` (pulls serde) vs hand-built `dacode-json` schemas. Interacts with B1.*

> **✅ RESOLVED 2026-09-12 — offline codegen, as amended below.** Signed off by
> delegation alongside B1, since A16 depends on both and its item 4 is this
> decision's §13 wording. **Now normative at design §13.3**: `schemars` appears
> only in an offline generator emitting canonical schema JSON for the tools to
> `include_str!`, never in a shipped crate; generated bytes are canonical and
> byte-stable (`title` stripped, key order pinned) because tool-definition churn
> *including pure reordering* kills the cache-stable prefix.
>
> **The [14](../part3-capabilities/14-tools.md) half is deferred to M1.** Doc 14
> is still at summary depth, and the just-in-time rule forbids deepening a doc
> whose milestone has not started — the generator's concrete shape (xtask layout,
> CI diff check) would be speculative now. The *boundary* is settled and lives in
> §13.3, which is what M1's doc-14 deepening will cite.

**The interaction with B1 resolves it.** If B1 adopts the hybrid, then serde
derives on dacode's *own* types are the intended path — so `schemars` deriving a
schema from the same struct the tool deserializes its args into is consistent,
not a violation. Hand-building ~15 schemas is 400–600 lines of tedious code that
**drifts from the arg structs**, and drift between a tool's schema and its
actual deserialization target is a silent failure the model discovers for you.

**Verified cost.** `schemars` depends on `serde_json` **non-optionally** — both
0.9.0 (`dyn-clone, ref-cast, serde, serde_json`) and 1.2.2 (plus `chrono`,
`schemars_derive`), confirmed from forge's `Cargo.lock`. So `schemars` puts
`serde_json` in the dependency tree of whichever crate uses it, and
[02 §2.4](../part1-foundations/02-json-strategy.md) states
*"`serde_json` is never a direct dependency of any dacode crate."*

**That makes the isolation boundary load-bearing, not cosmetic.** Recommendation:

- `schemars` is a dependency of **`dacode-tools` only** — exactly the precedent
  §13 already sets for MCP crates ("isolate it behind an adapter in
  `dacode-tools` that converts at the boundary").
- The adapter converts `schemars::Schema` → **`RawJson`** at the boundary.
  `ToolSpec.schema` is `RawJson` under B1, so the schema is opaque bytes spliced
  verbatim into the provider request — precisely what `RawJson` exists for.
  `serde_json` never crosses into `dacode-protocol` or `dacode-core`.
- **Add a canonicalization pass, and treat it as a caching requirement.** Strip
  `title` (forge ships a `RemoveSchemaTitles` transform for exactly this) and
  **pin key order**. [09](../part2-engine/09-context-caching.md) lists
  "tool-definition churn incl. pure *reordering*" as a cache kill: an unstable
  schema byte-order across builds silently defeats the stable prefix. The
  generated schema must be **byte-stable**, and that deserves a golden-fixture
  test (§14), not a comment.
- Steal forge's `derive(ToolDescription)` idea — description from the doc
  comment — so the doc and the schema cannot disagree.

**Confidence high**, conditional on B1.

**Amended during sign-off review — drop the `serde_json` dependency entirely.**
The recommendation above confines `schemars` to `dacode-tools`, but since
`schemars` depends on `serde_json` non-optionally, that still puts it in a
shipped crate's tree. It does not have to.

**`build.rs` is not a way out**: a build script runs *before* compilation and has
no access to type information, so it cannot derive a schema from a Rust struct.
The workable routes are:

- **Offline codegen — recommended.** A small `xtask`-style generator that depends
  on `dacode-tools` + `schemars`, runs in CI, and emits canonical schema JSON
  that the tools `include_str!`. CI fails if regeneration produces a diff, so
  drift is impossible, and checked-in JSON makes schema changes reviewable — the
  same discipline §14 already applies to golden fixtures. `serde_json` then lives
  only in the generator. **The same machinery produces B2's protocol JSON-Schema
  export**, which is the strongest argument for it.
- **A hand-written `#[derive(ToolSchema)]`** emitting `dacode-json` builder calls —
  canonical by construction, dependency-free, but we own JSON Schema correctness
  for every shape tool args use (~400–600 LoC of proc-macro). Worth it only if
  the codegen step proves awkward.
- **Hand-written schemas** — rejected: ~15 tools, and drift between a schema and
  its arg struct is a silent failure the model discovers for you.

Everything else stands, and canonicalization becomes a property of the generator
rather than a runtime transform: strip `title`, pin key order, golden-fixture the
bytes — because A4 makes tool-definition churn, *including pure reordering*, a
cache kill.

**On sign-off:** record in [14](../part3-capabilities/14-tools.md); the §13
wording change rides along with delta **A16**.

---

### 7. [B7] `bash` sandboxing and honesty — ✅ **SIGNED OFF 2026-09-17: deny-list + advisory honesty; the seam ships as an inert stub**

> **✅ SIGNED OFF 2026-09-17** — *"recommended approach, but keep in mind - actual
> sandboxing implementation would be much later, we just need to have stubs to not
> refactor much in the future."* All five parts of the recommendation are taken —
> the deny-list + permission engine in v1 with no OS sandbox, the
> interpreter-disqualification rule, the tight critical-pattern list, the
> advisory-honesty wording, and the 2026-09-16 configuration polarity — **and the
> user's scoping note is recorded as a decision, not a footnote**: M1 ships the
> `SandboxBackend` seam as an **inert stub** — no Landlock/seccomp code — with §4.7's
> worktree remaining **the only real containment** until the seam is filled. The
> stub's whole purpose is the user's: filling it later must be additive, not a
> refactor.
>
> **The coupling this creates, recorded where it will be re-read.** B20's rule — *a
> remote member must have a sandbox configured or be refused* — **cannot be
> satisfied until the seam is filled**, because an inert stub contains nothing. So
> B20's M5 build inherits **"fill the seam first"**, or it uses container
> containment, which violates §1.5's single-binary / no-external-runtime rule.
> B20's design session must start from this sentence.
>
> **The polarity amendment already in this entry stands, unchanged**: the sandbox is
> an **immutable session attribute set at creation**, `yolo` is the **mutable
> opt-out**, neither is a default, and the indicator is by colour — with **U89 still
> open** on colour being the *only* channel for a security state.
>
> **The delta is approved and owed, not applied.** Owes: **A12**'s §16.3 half;
> [14](../part3-capabilities/14-tools.md) and
> [16](../part3-capabilities/16-permissions.md) updates; the `bash:readonly` wording
> in §4.3 and §5.2 (advisory where the deny-list is the mechanism, enforceable
> denial kept on the `write`/`edit` path scope); and ~~the **confirmation-or-overturn
> of A23's `yolo` carve-out** in §4.4 — the debt A23's row in the resolved table
> names as B7's to confirm~~ ✅ **CONFIRMED 2026-09-18, inside A22's pass** so §4.4 was
> edited once rather than twice, exactly as A22's banner required. The carve-out stands
> and now says why: a mode whose whole meaning is that the user opted out of being asked
> cannot start asking, and a mode that silently becomes `ask` under one condition is a
> mode nobody can reason about — consistent with this entry's polarity amendment, which
> makes `yolo` the deliberate **mutable opt-out**. Auditability is not lost, since the
> detection is still recorded on `PermissionResolved`. **The user added one thing this
> entry did not contemplate**: `yolo` is also the mode where a leak is *most* likely
> (everything runs automatically, so nothing passes a human on the way to exec) and
> therefore the one mode where an escalation-based warning would never fire, so the first
> detection in a session now raises a **non-blocking notice** — rendered once, pausing
> nothing, requiring no answer, not repeated. Once per session rather than per detection,
> because a notice on every occurrence in a mode chosen for not being interrupted *would
> be* the interruption. **The rest of this owed list is unchanged**: A12's §16.3 half,
> docs 14 and 16, and the `bash:readonly` wording.
>
> **A named route for filling the seam, supplied by the user at sign-off — v2 work,
> not v1.** For the sandbox variant, **intercept `execve` through a BPF filter** and
> read the **exact program path and, optionally, its arguments** at the point of
> execution: the technique behind Google's `sandboxed-api`
> (github.com/google/sandboxed-api) and its `sandbox2` substrate
> (developers.google.com/code-sandboxing/sandbox2). This matters more than it looks,
> because it answers B7's own central argument rather than working around it: a glob
> over a command *string* cannot decide what the command does, but the kernel at
> `execve` sees the resolved binary and its argv — so `git diff; git push`,
> `bash -c …`, a shell function, an alias and a symlink all stop being evasions. It
> is therefore the mechanism that could make `bash:readonly` **enforced** instead of
> advisory, i.e. the one that *retires* the honesty fix rather than documenting it,
> and it is what B20's remote members actually need. Tracked as **U98** with its two
> open costs: a dependency-policy answer (Rust BPF loaders are `aya` / `libbpf-rs`,
> and the filter is compiled at build time, which is §2.3/§2.4 territory and not the
> same question as linking C at runtime), and the fact that `sandboxed-api` /
> `sandbox2` are themselves C++ — **the lead is the technique, not the library.**

**Gates** [14](../part3-capabilities/14-tools.md),
[16](../part3-capabilities/16-permissions.md) · Original entry (§16.3):

> **⚠️ SCOPE WIDENED 2026-09-13 (session 8), and this is the more important
> half.** B7 was scoped to `bash:readonly`. But the argument it rests on — *a
> shell command is a program, and a glob over its text cannot decide what it
> does* — applies **verbatim** to §4.4's *main* `bash` allow-list, which the spec
> presents as a control with no caveat at all:
>
> ```toml
> bash = { allow = ["cargo *", "git status", "git diff*"], ask = ["*"], deny = ["rm -rf *", "git push*"] }
> ```
>
> `git diff; git push` satisfies `git diff*`. `cargo run` invokes anything. So
> does a shell function, a `$(...)`, or an alias. The narrow entry fixed the
> labelled instance and left the unlabelled, more prominent one asserting a
> guarantee it cannot keep — which is the same defect §3.7's "No gap, no
> duplicate" had before A18, in a place where being wrong is worse.
>
> **What changes:** the recommendation's *advisory* wording covers §4.4 as well,
> and §4.4 gains one sentence naming where the real boundary is — the §4.7
> worktree sandbox plus the OS. The glob layer is a guardrail against
> **accidents**, which is genuinely worth having; it is not a control against an
> adversary, and the spec should not imply otherwise.
>
> Deliberately kept in B7 rather than folded into delta **A22** with the other
> capability seams: A22 holds *corrections*, while advisory-versus-enforced is a
> judgement B7 already owns and has not been signed off. One decision should
> answer the whole question.

> **⚠️ AMENDED 2026-09-16 (session 11) — the user fixed the configuration
> polarity: *"sandboxing should have opt-in configuration. `yolo` is opt-out."***
> Recorded before sign-off rather than after, which is the B4/B6/B1 pattern — an
> amendment to an unsigned recommendation, not a reinterpretation of a signed one.
>
> **The reading taken: two axes, both explicit, neither a silent default.**
> - **Sandbox axis — opt-in.** Off unless configuration turns it on. That matches
>   the evidence rather than fighting it: openclaw's is `mode: "off"` by default,
>   no harness in the set ships in-process OS sandboxing at all, and §4.7's
>   worktree sandbox is *already* per-session opt-in (`dacode --sandbox <name>`,
>   §11). Nothing about v1 changes; what changes is that the polarity is now
>   stated instead of inherited from an example.
> - **Mode axis — `yolo` is the opt-out.** §4.4's default stays `ask`, `yolo` is
>   never a default, and choosing it is how a user opts *out* of the guardrails.
>   Opting out is a deliberate act, not a configuration drift.
>
> **✅ The ambiguity was resolved the same day, and the two-axis reading is
> confirmed — with a sharper distinction than the one argued here.** The user's
> answer: *"The sandbox — is how we start the session (it's a session's attribute)
> and yolo can be on/off during the session."* So the axes differ not only in what
> they control but in **when they are decided and whether they can change**:
>
> | | Sandbox | Mode (`yolo` included) |
> |---|---|---|
> | Set | **at session creation** | any time during the session |
> | Mutable | **no** — it is an attribute of the session | **yes** |
> | Spec mechanism | `CreateSession { sandbox: Option<SandboxSpec> }` (§3.3) | `SetPermMode { session, mode }` (§3.3), `/mode` (§9.2) |
>
> **This ratifies §3.3 and §4.7 as written and needs no spec edit** — both shapes
> are already there. What it *adds* is a prohibition: the sandbox must **not** also
> become a layered `[permissions]` value under §4.4, because that is precisely what
> would make it mutable per agent and reopen consequence 3 below.
>
> **One consequence is new and worth stating plainly:** a session started
> unsandboxed **cannot be retro-contained**. If containment is wanted, fork a new
> session with it — the attribute is fixed at creation, so there is no command that
> adds one, and adding `SetSandbox` would undo the decision.
>
> **Three consequences, of which the third is new work.**
> 1. **The honesty fix extends to the mode indicator — and the user settled its
>    form: colour, not a label.** *"In TUI mode we can inform the modes
>    (sandbox/yolo) in colors, not necessary to have explicit name it."* So the
>    status line carries the containment state as a colour where §4.7 already shows
>    the branch, rather than spelling out `yolo · unsandboxed`. That is this
>    decision's own theme applied to the status line instead of to `bash:readonly`
>    — with one caveat recorded as **U89**: colour is the *only* channel, and a
>    security state that is invisible on a monochrome terminal, to a colour-blind
>    user, in a screenshot and in the web UI is a state nobody can check. Cheap
>    mitigation, not a relitigation: colour plus a glyph, or the name available on
>    demand (`/status`) even though it is not printed.
> 2. **B20's prerequisite becomes a rule, not a warning.** Remote execution lands
>    exactly in the dangerous combination, and a remote host's blast radius is not
>    the user's machine. So: **opt-in stays opt-in locally and becomes mandatory
>    remotely** — a member on another host either has a sandbox configured or is
>    refused. Recorded in B20's fork 6.
> 3. ~~**⚠️ Layered config could silently drop a parent's sandbox**~~ ✅
>    **DISSOLVED by the resolution above, not merely deferred.** The sandbox is a
>    session attribute set at creation and inherited by children (§5.3 already says
>    a child inherits cwd/sandbox), so there is no per-agent config key to weaken
>    and **A22 item 3 / U64 does not need the sandbox added to its intersection
>    rule**. The hazard below is what happens *if* the sandbox is ever made
>    configurable per agent — kept as the argument for the prohibition, not as open
>    work. §4.4's
>    `[permissions]` block is documented as "global defaults, overridable per agent
>    and per session", and §4.4's resolution order puts **agent config above
>    project and user config**, so an agent-level value wins. §5.3 meanwhile says a
>    child *inherits* cwd/sandbox from its parent, and **there is no sandbox key in
>    §4.4's example to weaken yet** — so this is a hazard this amendment *creates*
>    the moment a sandbox setting becomes configurable per agent, not one that
>    exists today. Stating it conditionally is the point: U64/A22 item 3 already
>    establishes the fix for scopes — **intersection, not `min(mode)`** — and
>    whatever sandbox key B7 adds must be inside what intersects, never something a
>    child may relax. Owed to A22's item 3, and it is the reason this amendment is
>    not just a documentation note.
*OS-level (Landlock/seccomp/sandbox-exec) in v1 vs deny-list only.*

Full evidence:
[sandboxing-and-checkpoints §Part 1](../../../.agents/knowledge/reference-harnesses/sandboxing-and-checkpoints.md).

**The decisive negative result: no harness ships in-process OS sandboxing.**
Landlock, seccomp-bpf, `sandbox-exec`/Seatbelt profiles, bwrap, `unshare`,
chroot and read-only bind mounts are **absent from all four trees** — including
the Rust one. All four converge on glob/pattern *approval* engines. The only
real enforcement found anywhere is openclaw's Docker/Podman sandbox
(**11 258 LoC**, `--read-only` rootfs, seccomp + apparmor profiles, capability
drops), and it is **`mode: "off"` by default**. That route also violates
dacode's single-binary / no-external-runtime principle (§1.5).

**And a deny-list is weaker than §4.3 implies.** All three glob engines are
defeated by `bash -c "…"`, `$(…)`, `env X=1 rm`, pipelines, and **symlinks** (an
`allow = ["src/**"]` write rule permits `src/link -> /etc/passwd`). opencode
says so in its own tool description — the external-directory scan is *"advisory
only"* — and forge's shipped default deny-list is **empty**. Notably, opencode
*removed* its parser-based approval reduction in v2 and now carries it as debt
(`bash.ts:66-68`).

**Recommendation: deny-list + permission engine in v1, no OS sandbox.** Five
parts (the fifth added by the 2026-09-16 amendment above):

1. **Reserve the seam now, build it later.** `ToolCtx` gains something like
   `SandboxBackend: Option<&dyn Fn(Command) -> Command>`. §4.3 already hedges
   "no write FS *if available*" — make that hedge an explicit extension point so
   v2 (Landlock on Linux, `sandbox-exec` on macOS) is additive rather than a
   redesign. Landlock is the right v2: in-process, no runtime dependency, Rust
   crate available.
2. **Borrow oh-my-pi's discipline, not openclaw's volume.** A *tight*
   high-confidence critical-pattern list (~21 regexes) whose stated philosophy
   fits dacode exactly: false positives are user-fixable, false negatives cost
   data. Do **not** copy openclaw's 8-binary argument-shape profile system —
   7 334 LoC for marginal recall.
3. **Copy openclaw's one cheap good idea: interpreters are disqualifying.**
   `INTERPRETER_LIKE_SAFE_BINS` (`exec-safe-bin-runtime-policy.ts:24-59`) means a
   readonly mode that auto-allows `bash`, `python` or `sed -i` has no meaning.
   This is the only concrete anti-`bash -c` mechanism found in any harness, and
   it costs a list.
4. **Real containment is §4.7, already specced.** The worktree sandbox puts
   writes somewhere disposable with merge-back review. That is the actual
   security boundary; the deny-list is the last line when it is off.
5. **Configuration polarity: the sandbox is opt-in, `yolo` is the opt-out, and
   neither is a default.** `mode = "ask"` stays §4.4's default; the OS sandbox and
   the §4.7 worktree are both off unless configured, and a `yolo` + unsandboxed
   session is *permitted but announced* — on screen, where the branch already
   shows. Mandatory only where the blast radius is not the user's own machine,
   i.e. B20's remote members. The sandbox setting must also be inside §5.3's
   inheritance intersection (U64), so a child can narrow it and never widen it.

**One honesty fix this decision forces.** `bash:readonly` is a **misleading
name** for a deny-list. The `reviewer` built-in agent (§5.2) is granted
`bash:readonly` under `mode = "plan"` and is described as "Read-only code
review; never edits" — that claim is not enforceable by a pattern list. Either
document it as advisory the way opencode does, or enforce read-onlyness where it
*is* enforceable: the `write`/`edit` tools gate on path-based `PermScope::Edit`
and can be genuinely denied, while `bash` stays advisory. Recommend the latter
plus explicit doc wording.

**On sign-off:** apply delta **A12**'s §16.3 half; update
[14](../part3-capabilities/14-tools.md) and the `bash:readonly` wording in §4.3
and §5.2.

---

### 8. [B8] Checkpoint strategy — ✅ **SIGNED OFF 2026-09-17: shadow git kept with all six additions; workdirs via §5.4 workflow instructions**

> **✅ SIGNED OFF 2026-09-17** — *"let's sign as recommended. Using workdirs could
> be through workflows' instructions."* Taken as recommended: **keep shadow git**,
> with all six additions §4.6 omits — `.gitignore` filtering via `git check-ignore
> --no-index`; a 2 MiB untracked-file cap with skips reported; absent-from-tree ⇒
> delete; **selective per-file restore, not `checkout-index --all --force`**;
> path-escape rejection on restore; and the boundary turn id recorded in the
> `Checkpoint` event — with **gitoxide in-process** so the single-binary principle
> holds, and **both unsolved problems spec'd** rather than inherited: a snapshot
> retention policy (no harness has one) and the concurrent-session shadow-store
> race, stated as a trade now instead of discovered in M2.
>
> **The worktree question is answered by this sign-off rather than deferred.** A
> feature-level working directory is expressed through **§5.4 workflow
> instructions** — the user's own words above — *not* by a new session↔worktree
> binding mechanism. So §4.6 (undo) and §4.7 (containment) stay separate concerns,
> and B8's existing composition rule is sufficient unchanged: snapshot against
> `source.worktree`, key the shadow dir by its hash.
>
> **One residual is owed to doc 12.** A shared worktree needs a **one-writer rule**:
> B21's advisory lock covers the *session log*, not the *working tree*, and a
> rollback in one session must not undo another session's files. Selective per-file
> restore narrows that hazard; it does not close it.
>
> **The delta is approved and owed, not applied.** Owes: **A12**'s §16.5 half, plus
> the §4.6 amendments (items 1–6 and the retention policy).

**Gates** [12](../part2-engine/12-checkpoints-sandboxes.md) · Original entry
(§16.5): *shadow git (specced) vs content-addressed store vs forge-style
per-write file snapshots.*

Full evidence:
[sandboxing-and-checkpoints §Part 2](../../../.agents/knowledge/reference-harnesses/sandboxing-and-checkpoints.md).

**The strongest signal available: opencode independently built §4.6.** Separate
git dir seeded from the repo (`snapshot.ts:98-122`), **content-addressed git
tree IDs** from `capture()` (`:129-144`), capture at each assistant turn start
and end, and a revert that is **itself event-sourced** — `stage` / `clear` /
`commit` are events (`revert.ts:60-121`) carrying the boundary `messageID`, so
files and conversation rewind together. That is dacode's `Checkpoint` +
`Rollback` pair, arrived at from a different direction.

**The alternatives lose on specifics.** forge's per-write whole-file copies
(`forge_snaps`, 253 LoC) are simple but have **no dedup**, grow O(edits × size),
and cannot express "the tree before turn N" — §4.6's own rationale ("cheap
because git dedups") is literally the content-addressing argument. oh-my-pi's
checkpoint does **no filesystem work at all** (`checkpoint.ts:72-88`) — it is
conversation-only rewind for read-only investigation phases, a different
feature. openclaw has **no agent-edit undo**; its `src/snapshot/` (6 919 LoC)
backs up the server's own databases, and it substitutes container containment.

**Recommendation: keep shadow git**, with these additions — all from opencode's
implementation, none currently in §4.6:

1. Filter the capture set through the **source repo's `.gitignore`**
   (`git check-ignore --no-index`, `git.ts:447-458`). Capture changed +
   untracked files only, not the whole tree.
2. **Cap untracked file size** (opencode: 2 MiB, `snapshot.ts:138`) and report
   what was skipped. This is the `node_modules`-in-an-untracked-turn killer.
3. **Absent-from-tree ⇒ delete** (`snapshot.ts:70-74`), so files a tool
   *deleted* roll back. A naive "restore what we captured" misses this.
4. **Selective per-file restore, not a whole-index checkout.** opencode's own
   code warns to prefer selective restore (`snapshot.ts:76-81`); a global
   `checkout-index --all --force` clobbers the user's concurrent edits in a
   shared tree. §4.6's "checks that snapshot out" should be amended to say
   *selectively, for paths the agent touched*.
5. **Path-escape rejection on restore** (`snapshot.ts:181-183`).
6. **Record the boundary turn id in the `Checkpoint` event** so rollback rewinds
   `Context` to the matching turn — §4.6 says it truncates `Context` but does
   not say how the turn is identified.

**Implementation notes.** Use **gitoxide** (in-process, no subprocess) so the
single-binary principle holds — oh-my-pi's `pi-vcs` (913 LoC, gitoxide +
jj-lib, no subprocess) proves this works in Rust today. Snapshot against
`source.worktree` and key the shadow dir by its hash, which makes §4.7 worktree
sandboxes work unchanged.

**Two problems no harness solved — dacode must spec both.**

- **Unbounded snapshot growth.** No pruning code exists in any of the four.
  §4.6 needs a retention policy (by age, by count, or drop-on-session-close).
- **Concurrent sessions sharing one shadow store race.** opencode's per-worktree
  git dir has a single mutable index (`git.ts:481-489`) — two sessions capturing
  the same project race. dacode's per-`<session>` shadow dir avoids the race but
  pays the re-add cost per session. State the trade explicitly; do not discover
  it in M2.

**On sign-off:** apply delta **A12**'s §16.5 half and amend §4.6 with items 1–6
plus the pruning policy.

---

### 9. [B9] ACP commitment — ✅ **SIGNED OFF 2026-09-17: build deferred to M5, the two M1 constraints committed**

> **✅ SIGNED OFF 2026-09-17** — *"let's defer ACP as suggested. just be aware
> during implementation that we would have it."* Taken as recommended: defer the
> build to **M5** (`dacode acp` — the fs-free, terminal-free, `mcpServers`-rejecting
> subset, 2 500–3 500 LoC, its stdio JSON-RPC framing shared with the M4 extension
> host), and commit the **two M1 constraints** now: everything a client needs is
> derivable from the log via fold + `Subscribe { since }`, and **no tool may branch
> on client capabilities or locality**.
>
> **The user's "be aware during implementation" is recorded as having a mechanism
> rather than being an intention.** Awareness is enforced by exactly three things:
> constraint 1, constraint 2, and the review rule in
> [14](../part3-capabilities/14-tools.md) — the pressure to violate it arrives with
> ACP, not before. The item is added to the **M1 doc-deepening checklist** so the
> awareness is verified when doc 14 is written, not hoped for.
>
> **The open input stays open**: no maintained Rust ACP crate was verified (no
> registry access during the evidence pass); check before M5 planning — a usable
> crate cuts ~1 k LoC from the estimate.
>
> **The delta is approved and owed, not applied.** Owes: **A12**'s §16.4 half
> (resolved as *deferred with constraints*),
> [26](../part5-outreach/26-acp.md) moved to M5 in the milestone mapping, and the
> two constraints added to M1's docs.

**Gates** [26](../part5-outreach/26-acp.md) · Original entry (§16.4): *v1 or
defer; "cheap once the protocol crate exists".*

Full evidence:
[dialects-and-acp §Part 2](../../../.agents/knowledge/reference-harnesses/dialects-and-acp.md).

**The "cheap" claim survives — conditionally.** Measured adapters: opencode
**3 519 LoC**, purely additive (consumes only its own client API,
`acp/service.ts:34,76`); oh-my-pi **4 201 LoC**, *not* additive; openclaw
**13 393 + 1 906 + 6 938 LoC**; forge none.

**dacode gets openclaw's most expensive component for free.** openclaw maintains
a **558-LoC SQLite ledger mirroring every `session/update` with seq numbers**
because its gateway state is not ACP-replayable, then does four-way
ledger-vs-transcript replay resolution for `loadSession`
(`translator.session-lifecycle.ts:100-160`). dacode's append-only log + fold +
cursor and `Subscribe { since }` (§3.2, §3.3) make that entire subsystem
unnecessary — `session/load` is a cursor replay.

**What does not map, and the trap.** Two ACP features assume the *editor* owns
I/O: `fs/read_text_file` + `fs/write_text_file`, and `terminal/*`. Three
precedents for handling it: skip both (openclaw — works fine in real clients),
write-on-approval only (opencode `permission.ts:102`), or inject client
capabilities into the engine's tool layer (oh-my-pi, whose
`tools/bash.ts:1149-1242` **branches on `clientBridge?.capabilities.terminal`**).
The third is the trap: it puts a client-capability branch inside a tool, which
violates §1's first principle and [00-overview §0.2](../00-overview.md)'s "a
client never branches on whether the engine is local or remote."

**Recommendation: defer the implementation to M5; commit to two M1 constraints.**

- **Constraint 1:** everything a client needs must be derivable from the event
  log via fold + `Subscribe { since }`. No state that only the engine can
  render. (This is already §1 principle 2 — ACP makes it load-bearing.)
- **Constraint 2:** no tool may branch on client capabilities or locality.
  Record it in [14](../part3-capabilities/14-tools.md) as a review rule, because
  the pressure to violate it arrives with ACP, not before.

Then implement in **M5** as `dacode acp`: the fs-free, terminal-free,
`mcpServers`-rejecting subset, exposing an "agent"-only authenticate method.
Estimate **2 500–3 500 LoC Rust** (ACP serde types ~1 k, stdio JSON-RPC framing
~400 — *shared with the M4 extension host*, event→`session/update` mapper
~800–1 000, permission + prompt-turn correlation ~400, tests ~500).

**Bonus worth recording:** `acpx` shows ACP is also useful to dacode as a
**consumer** — the §7 stdio extension host could spawn *external* ACP agents as
sub-agents while dacode keeps sessions and permissions. That is an M5+ idea, not
a v1 commitment.

**Open input:** no maintained Rust ACP crate was verified (no registry access
during this pass). The estimate assumes hand-written serde types; a usable crate
would cut ~1 k LoC. Check before M5 planning.

**On sign-off:** apply delta **A12**'s §16.4 half (resolve as *deferred with
constraints*); move [26](../part5-outreach/26-acp.md) to M5 in the milestone
mapping and add the two constraints to M1's docs.

---

### 10. [B10] Config on disk — ✅ **SIGNED OFF 2026-09-17: TOML confirmed, both consequences recorded**

> **✅ SIGNED OFF 2026-09-17** — *"ok, let's use TOML-on-disk and json-on-wire."*
> Taken as recommended — a confirmation rather than a choice, since §5.1 already
> assumes it — with **both consequences recorded**, because both are cheap to
> design for and expensive to retrofit: **(1)** `dacode config edit` must
> **preserve comments and formatting** (a format-preserving editor, `toml_edit`),
> which matters *more* for **A7**'s config-write endpoints, since silently eating a
> user's comments from a web UI is worse than from a CLI; **(2)** a generated
> **`dacode.schema.json`** reusing B6's machinery and pairing with B2's protocol
> schema export. The B16↔B10 coupling stays **dissolved**: authoring tooling never
> ports, so no YAML crate enters the Rust tree.
>
> ~~**The delta is approved and owed, not applied.**~~ ⚠️ **HALF-APPLIED 2026-09-19,
> in A25's and A10's shape — and consequence (1) landed via A7, not via its own
> transcription.** Comment- and format-preserving writes are **normative at §5.1**:
> every dacode write to a config, agent, skill or workflow file mutates the
> *document* rather than round-tripping through a struct, so comments, key order,
> whitespace and table layout survive. A7 is what made that load-bearing rather than
> tidy, exactly as this entry predicted, and the transcription found something this
> entry could not see: **§12 already cited "(§5.1)" for the requirement while §5.1
> carried no such text**, so before A7 it existed in the spec only as a *citation* —
> a resolvable reference with false content, which `check_section_refs()` passes
> because the heading exists. **Still owed: consequence (2)**, the generated
> `dacode.schema.json` (B6's machinery in `xtask`, §2.4/§13.3, pairing with B2's
> export at §9.1 — note `xtask` now has *three* generators to specify if all land),
> plus **A12**'s §16.2 half (striking §16.2 from the spec) and both items in
> [17](../part3-capabilities/17-config-agents-skills.md), whose summary now carries
> the correction rather than the depth. ⚠️ **Do not mark B10 applied.** One clause
> A7 added bounds consequence (2) and is worth carrying into it: §8.5 **refuses the
> exec-bearing keys on every write route** (`[[extensions]]`' `cmd`, a hook's
> `run`/`post`) although the files holding them are writable, so a generated config
> schema is **not** the write endpoints' schema — advertising keys the server
> refuses would be a lie a client can act on.

**Gates** [17](../part3-capabilities/17-config-agents-skills.md) · Original
entry (§16.2): *confirm TOML-on-disk/JSON-on-wire.*

There is no live alternative. §5.1 already assumes TOML on disk and JSON on the
wire, every agent-definition example in §5.2 is TOML, and forge ships layered
TOML (`forge_config`). JSON-everywhere makes human-edited config worse
(no comments, trailing-comma hazards); YAML adds whitespace sensitivity.

**Recommendation: confirm.** Two consequences worth recording now, because both
are cheap to design for and expensive to retrofit:

1. **`dacode config edit` (§11) must preserve comments and formatting.** A
   serialize round-trip through a TOML library destroys user comments and
   reorders tables. That needs a format-preserving editor (`toml_edit`), and it
   is the kind of thing that is trivial to get right at design time and a
   rewrite later. It also matters for delta **A7**: config-write endpoints let
   the *web UI* edit agent/skill files, and silently eating a user's comments
   from a UI action is worse than from a CLI one.
2. **Ship a generated `dacode.schema.json`.** forge does this for agent
   definitions (serde + schemars → `forge.schema.json`), which gives editor
   completion and validation for free. This reuses the B6 machinery and pairs
   with B2's schema export.

**Confidence high; low stakes.** The only way this goes wrong is by not noting
the two consequences above.

**On sign-off:** strike §16.2 from the spec via delta **A12**; add the two items
to [17](../part3-capabilities/17-config-agents-skills.md).

---

### 11. [B11] New crates `dacode-store` / `dacode-introspect` — ✅ **SIGNED OFF 2026-09-17: store deferred to M3, introspect approved for M5 with its record format at M1**

> **✅ SIGNED OFF 2026-09-17** — *"ok, let's do as suggested."* Taken as
> recommended: **defer `dacode-store`** — the traits live in `dacode-core::store`
> and the crate is extracted at **M3**, because a boundary with one implementation
> is indirection without payoff and extraction later is mechanical; and **approve
> `dacode-introspect` in principle, create it at M5** — but its **exchange-record
> format is an M1 deliverable** in
> [13](../part3-capabilities/13-providers.md), because retrofitting provider
> instrumentation in M5 costs far more than emitting it from the start, and it gives
> [09](../part2-engine/09-context-caching.md) the harness that validates A4's cache
> invariants. The in-tree-vendoring settlement stands: all crates under `crates/`,
> one repository.
>
> **The dissent is recorded as available and not taken.** Confidence was medium, and
> the respectable other side — separate `dacode-store` *now* for dependency
> isolation from day one — was on the table at sign-off and was not chosen.
>
> ~~**The delta is approved and owed, not applied.**~~ ✅ **APPLIED — and this banner
> line was the one site A13's pass missed**, which is worth recording because it is the
> same defect class the board keeps finding: a status claim updated in the summary row
> and the resolved-entries row but not in the entry's own banner, so a reader who opens
> the decision rather than the table gets the stale answer. **A13** landed 2026-09-18
> (§2 recording both crates as approved-with-dates, [Appendix D](D-crate-graph.md)'s
> *legend* corrected rather than its graph, the record-format item already normative via
> A10's carve-out).
>
> ⚠️ **A7 completed the half of this decision that A13 could only promise.** A13 wrote
> into §2.2 that "these placements must agree with the scaling axis **§8.2's** storage
> and auth traits describe" — while §8.2 described no traits at all, so the coupling it
> recorded was a citation to absent content. **A7 (2026-09-19) put the axis there**: both
> trait families with `JsonlStore`+`SharedToken` as the only M1 implementations, and
> **`BlobStore` reconciled in as the third storage trait** — §2.1 named three, A7's own
> delta entry named two, and transcribing that entry literally would have left the spec
> disagreeing with itself about how many storage traits exist. The two placements do now
> agree, and §8.2 cites §2.1 for placement rather than restating the M1/M3 dates.

**Gates** [05](../part2-engine/05-event-log-storage.md),
[25](../part5-outreach/25-introspection.md), [Appendix D](D-crate-graph.md) ·
Original entry: *approve as separate crates vs folding into `core`/`server`.*

**`dacode-store`: defer — define the traits in `dacode-core::store`, extract the crate at M3.**

The case for separating now is that `core` and `server` both need the storage
traits, and [05](../part2-engine/05-event-log-storage.md) warns the trait shape
must accommodate Postgres + multi-tenancy "without churn". The case against is
stronger:

- **A crate boundary with one implementation is indirection without payoff.**
  M1 ships `JsonlStore` only. Designing the trait against a single implementor
  is the actual risk, and *the crate boundary does not fix that* — a second
  implementor does. SQLite lands at M3, which is exactly when the abstraction
  becomes checkable.
- **The dependency-isolation argument is weaker than it looks.** `dacode-tui`
  depends on `dacode-protocol` only (§2), so it never sees store deps anyway,
  and the `dacode` binary always composes `core`. There is no TUI-only build to
  protect. Heavy drivers (`sqlx`/`tokio-postgres`) arrive with Postgres, which is
  post-M3 — after the extraction point.
- **Extraction later is mechanical**: move the module, re-export, update
  `Cargo.toml` and the CI boundary lint. Nothing about doing it at M3 costs more
  than doing it at M0.
- dacode is already at ten crates (§2); an eleventh needs to earn its boundary.

**`dacode-introspect`: approve in principle, create at M5 — but define its *record format* in M1.**

The dependency set is genuinely disjoint from the engine (HTTP MITM proxy, SSE
reassembly, charting), and [25](../part5-outreach/25-introspection.md) describes
two modes — a standalone proxy any CLI can point at, *and* in-process record
emission from dacode's own provider layer. Only a crate boundary keeps those two
honest. But it is M5 work; creating it during M0 scaffolding is an empty shell.

**The part that cannot be deferred:** the **exchange-record format** the
analyzer consumes. If the provider layer does not emit it from the start,
retrofitting instrumentation across every provider in M5 is far more expensive
than emitting it in M1. Define the record shape (endpoint kind, request/response
bodies, extracted `usage` incl. cache-token fields, timings, redaction applied)
as an M1 deliverable in
[13](../part3-capabilities/13-providers.md), with the crate arriving later to
read it. This also gives [09](../part2-engine/09-context-caching.md) its test
harness earlier — the analyzer is what *validates* the cache invariants.

**Confidence medium** — this is architecture taste, and the dissent is
respectable: if you prefer dependency isolation from day one, separate
`dacode-store` now and accept designing the trait against one implementor.

**Settled alongside sign-off: crates are vendored in-tree.** All of them, under
`crates/`, with their full names — `dacode-store` and `dacode-introspect`
included when they arrive. **One repository, not a workspace of repos**; the only
external crate is the JSON library. So this decision is purely about *when a
directory appears under `crates/`*, never about publishing. That also weakens the
dependency-isolation dissent above: crates in one workspace share a single
`Cargo.lock` and a single build regardless of where the boundary sits, so the
benefit is namespace and lint enforcement, not build isolation.

**On sign-off:** ~~apply delta **A13** to §2 recording *both* crates as
approved-with-dates (`dacode-store` at M3, `dacode-introspect` at M5) rather
than as present-day additions; update [Appendix D](D-crate-graph.md) to mark
them deferred; add the record-format item to M1.~~ ✅ **Done 2026-09-18**, all three,
with two corrections to what "done" meant. **Appendix D needed its *legend* fixed, not
its graph** — the graph already carried both crates with an asterisk, while the legend
still called them "proposed" and named no milestone. And **the record-format item did not
need adding to M1 so much as recognising it had already landed**: A10's carve-out put
`ExchangeRecord` in §6 earlier the same day, so §2.1 cites it as normative rather than
owed. §2.2 also gained the reason its rules are enforceable at all, correcting a sentence
that called the pinned exact-diff gate merely something to "consider".

---

### 12. [B12] Session compression codec — ✅ **RESOLVED 2026-09-17: `zstd`, accepted as a class-1 exception — APPLIED 2026-09-18 via A15**

> **✅ RESOLVED 2026-09-17 — by decision, not by measurement.** The user's call:
> *"We can allow as exception to use zstd with C library. That's fine."* So the
> **original recommendation stands and the reopening closes**: `zstd`, with
> [Appendix F](F-c-dependencies.md) §F.3's row moved from *avoidable* to
> **accepted** — the **first class-1 exception this policy grants**. The case that
> justifies it is the one the reopening itself surfaced: `ruzstd` is
> **decode-only**, so every pure-Rust option was a *different codec* rather than a
> different build of this one, and the log's write path needs a real encoder.
>
> **What still has to happen, in order.** `zstd-sys` enters
> `gates/class1-allowlist.txt` **in the same commit as the dependency** (§2.3's
> rule); its §F.3 row already exists, so `tools/dep-audit.py` passes the moment the
> allowlist entry lands, and the allowlist's own comments now name it. **U14 becomes
> the condition on the exception** rather than a reason to reopen it: verify the
> `libzstd` cross-compile cost against
> [31](../part6-quality/31-ci-milestones.md)'s matrix, and if it is painful the
> fallback is `flate2` + `rust_backend` — a cheap fallback, because A15's per-field
> discriminator stays **codec-agnostic**, so switching is a config change and not a
> §14 wire break. **B18's T1 bench still runs**: it no longer *chooses* the codec,
> but it chooses the **level** and confirms the ratio, which is the difference
> between a tuned default and an assumed one. Dictionaries stay deferred behind the
> tag A15 provides.
>
> **Two knock-ons, recorded so they are not rediscovered.** ~~**A15's hold is fully
> lifted** — mechanism *and* name — though its transcription should still generalize
> the variant names (`plain | compressed | flat | flat+compressed`, codec in config)
> exactly as **U25** proposed, because a wire value that names a codec makes the
> *next* switch a compatibility event.~~ ✅ **A15 applied 2026-09-18, and it
> generalized further than this predicted, for a reason the prediction missed.** The
> list above keeps `flat | flat+compressed` as per-field values and puts the codec in
> *config*; what landed is **`plain | compressed` with the codec in `LogHeader`**.
> Config alone would have orphaned every record already written the moment the codec
> changed — the log is append-only, so existing records are never re-encoded and one
> file lives under one codec for its whole life, which makes the codec a property of
> the *file* and the header is where A1 had already put file-level properties. And
> `flat` is not a field encoding at all but a *record representation*, so keeping it in
> a per-field discriminator would let a format change masquerade as a compression
> change; it stays reserved as §13.5's future binary `Codec`. **U25 closes** — its
> instruction was discharged, with the hole in its own answer closed too. And **U26 closes**: the three places that
> still assumed `zstd` (doc 05 §5.3's heading and its `[DECISION B12]` line, the
> README's "selective zstd compression") are correct again rather than pending.
> **U14 does not close and A15 does not discharge it** — it is the *condition* on this
> exception, and `zstd-sys` is rowed in the commit that adds the dependency.

**Gates** [05 §5.3](../part2-engine/05-event-log-storage.md) · Original entry:
*zstd (recommended) vs lz4/brotli; confirm the default compression targets and
that compressed fields are intentionally non-searchable.*

**Recommendation: zstd**, and confirm the allowlist and the searchability trade
as written in [05 §5.3](../part2-engine/05-event-log-storage.md).

- **Ratio/CPU is the right axis here**, and zstd wins it. The workload is
  write-once, rarely-read tool output and reasoning text — not a hot path where
  lz4's speed advantage matters. brotli's better ratio arrives at compression
  speeds that suit HTTP pre-compression, not a live write path.
- **Dictionary support is the differentiator that actually matters**, because
  session events are *many small payloads* — the case where generic compression
  performs worst. zstd dictionaries are the standard answer. **Do not commit to
  this in v1**: a dictionary must be stored alongside the data and immutable
  once trained, and rotating one mid-session complicates the per-field `codec`
  tag. Record it as the reason zstd beats lz4, and as a later optimization
  behind the tag that already exists.
- **One real cost to check before M1:** the `zstd` crate links C (libzstd),
  which touches the cross-compile matrix in
  [31](../part6-quality/31-ci-milestones.md) and the single-binary principle
  (§1.5 — satisfied by static linking). `ruzstd` is pure Rust but
  **decode-only**, so compression needs libzstd regardless. Verify the matrix
  cost when [31](../part6-quality/31-ci-milestones.md) is deepened; if it is
  painful, `lz4_flex` (pure Rust) is the fallback and this decision reopens.

The size floor in §5.3 (skip below ~1 KiB) is correct and should stay — it is
what makes the no-dictionary v1 acceptable.

**Confidence high**, conditional on the cross-compile check.

**⚠️ REOPENED by the [A17](A-design-deltas.md) dependency policy.** The
recommendation above treated `zstd`'s C toolchain as a cross-compile
inconvenience to verify. Under "pure Rust by default" it is a policy violation
needing an [Appendix F](F-c-dependencies.md) row — and the fallback is worse than
it looks, because **`ruzstd` is decode-only**, so it cannot serve the write path
at all. There is no pure-Rust zstd encoder.

| Codec | Pure Rust | Ratio | Compress speed | Note |
|---|---|---|---|---|
| `zstd` | **no** (`zstd-sys`) | best | good | dictionaries were the main argument, and they were already deferred |
| `lz4_flex` | yes | weakest | fastest | this workload is write-once/rarely-read, so speed is not the priority |
| `brotli` | yes (pure-Rust port) | good | **slow** | aimed at HTTP pre-compression, not a live write path |
| `flate2` + `rust_backend` | yes | modest | moderate | `miniz_oxide`; the boring safe choice |

**Revised recommendation: `flate2` with the pure-Rust backend**, unless measured
ratios on real session bytes justify either accepting the C dependency (with an
Appendix F row) or trading ratio for `lz4_flex`'s speed. Everything else in B12
stands unchanged — the allowlist, the ~1 KiB floor, the
`compress = "off" | "blobs" | "allowlist"` knob and the explicit non-searchability
trade are all codec-independent.

This is now a **measured** decision rather than a reasoned one: it needs a
ratio-and-speed comparison on representative session bytes before M1. That is
cheap, because the benchmark corpus already exists in the JSON crate.

**On sign-off:** ~~apply delta **A15** to §3.6/§12; strike B12 here; add the
libzstd cross-compile verification as an M0/M1 task in [TODO](../TODO.md).~~
✅ **A15 applied 2026-09-18**, to seven sections rather than the two named here —
§3.6 and §12 as instructed, plus §2.3 and §13.1/§13.2/§13.5, which were all still
calling the codec an open choice or calling the log `flat`, and §14 for the tests.
**B12 is not struck**, deliberately: §16-style bookkeeping belongs to A12, and this
entry is the record of why the policy granted its first exception. The
cross-compile verification is **U14**, still open and still the condition.

---

### 13. [B13] Session organization model — ✅ **SIGNED OFF 2026-09-17, APPLIED 2026-09-18 via A14: confirmed with both fixes; virtual folders promoted to a first-class requirement**

> **✅ SIGNED OFF 2026-09-17** — *"yes, we need some virtual folders when working on
> specific features that has many steps (cannot fit single session' context).
> Existing harnesses creates a plain list of sessions (maximum: sortable by repo
> location)."* Taken as recommended: tags + virtual folders + projects confirmed as
> **fold events**, with **Fix 1** — organization changes are durable log events but
> **never part of `Context`**, and the store's mapping tables are a rebuildable
> projection — and **Fix 2** — project derivation maps a worktree back through `git
> rev-parse --git-common-dir`, so sandboxed sessions do not silently split into
> their own project.
>
> **The amendment, and it reverses part of the recommendation: virtual folders are a
> first-class requirement**, not the deferrable UI-heavy half of the M2/M3 split
> below. The motivation is the user's, quoted above: a feature with many steps does
> not fit one session's context, so it spans sessions, and a folder is what holds
> them. Every harness surveyed offers a flat session list at best sortable by repo
> location — this is a **differentiator, not a nicety**, and "projects + tags first"
> no longer describes the plan of record.
>
> **The couplings are recorded, not resolved here.** This is the same multi-session
> feature shape that **A3's hand-over/resume** amendment serves, and **B8's** answer
> (signed the same day) puts such a feature's worktree in **§5.4 workflow
> instructions** rather than in a session↔worktree binding.
>
> ~~**The delta is approved and owed, not applied.**~~ ✅ **Applied 2026-09-18 via
> [A14](A-design-deltas.md)**, which landed the three event variants **and grew well
> past §3.2/§3.5** — ten spec sections, because nothing in the signed text carried what
> a folder *is* as opposed to which folder a session is in. **Both fixes landed**: Fix-1
> verbatim in doc 06 §6.2, with the carve-out its own wording did not need (membership
> is a projection, **names are not** — they live in §12's authoritative `org.json`), and
> Fix-2 normatively in **§4.7** as well as in doc 06, because the spec may not cite a
> plan doc and §14 has to be able to test the derivation. **The amendment is what forced
> the growth**: a first-class folder that spans sessions must be creatable *before* its
> sessions exist, which pure membership cannot express — so the registry, its `Org`
> command and §8.4's answer for a session-less resource are all consequences of the
> amendment rather than additions to it.

**Gates** [06](../part2-engine/06-session-organization.md) · Original entry:
*confirm tags + virtual folders + projects as fold events (not side-channel
metadata), one-project/one-folder + many-tags per session, and auto-deriving the
default project from the git repo root.*

All three are sound and [06](../part2-engine/06-session-organization.md) is
already hand-expanded. Confirming them needs two corrections, one of them a
genuine contradiction in the current text.

**Fix 1 — §6.2 and §6.3 contradict each other, and the resolution matters.**
§6.2 says organization lives in `SessionStore` as metadata and mapping tables,
*"not in the agent event log"*; §6.3 says organization changes *are* events
folded into `SessionInfo`. As written, a reader cannot tell whether these are
log entries. The clean resolution, which preserves §6.2's intent:

> Organization changes are **durable events in the session event log** (so every
> client converges through the shared fold, per §1 principle 2), but they are
> **never part of `Context`** — the provider never sees them, which is what
> §6.2 means by "not part of the conversation". The `SessionStore` mapping
> tables are a **projection** for querying (`GET /api/sessions?tag=`),
> rebuildable from the log and never authoritative.

That keeps the event-sourcing invariant, keeps the query path fast, and removes
the ambiguity about which store wins on conflict (the log does, always).

**Fix 2 — worktree sandboxes break project auto-derivation.** §6.2 derives the
default project from the git repo root. In a §4.7 worktree sandbox,
`git rev-parse --show-toplevel` returns the **worktree** path
(`.dacode/worktrees/<name>`), not the main repo — so every sandboxed session
would silently split into its own project, defeating the grouping. Derivation
must map back through `git rev-parse --git-common-dir` (or `git worktree list`)
to the main repository. Worth one line in
[06](../part2-engine/06-session-organization.md) §6.2 and a test.

**Also confirm:** one project + one folder + many tags per session (a session
has one location so the folder tree stays a tree; tags carry the
many-to-many), and the M2/M3 split (projects + tags first — they deliver most of
the value; virtual folders are the UI-heavy part).

**Confidence high.** The model is right; these are defects in the write-up, not
in the design.

**On sign-off:** ~~apply delta **A14** to §3.2/§3.5 including the three event
variants; amend [06](../part2-engine/06-session-organization.md) with both
fixes.~~ ✅ **Done 2026-09-18** — A14 applied, to ten sections rather than two,
and doc 06 carries both fixes plus the names carve-out Fix-1's own wording did not
cover.

---

### 14. [B14] Prompt/context buffer representation — ✅ **SIGNED OFF 2026-09-17, APPLIED 2026-09-18 via A4: appender + segment index, never re-parse**

> **✅ SIGNED OFF 2026-09-17** — *"ok, let's do a recommended approach."* Taken as
> recommended: **appender + segment index** (`Segment { kind, offset, len, digest }`)
> — cutting later parts is **O(1) truncation**, re-parsing an assembled prompt is
> **never** required, and the one genuine rebuild case (a model or provider switch)
> **replays the segment list through the new encoder** rather than parsing the
> previously assembled string. The segments are the source of truth; the assembled
> bytes are a derived artifact. The sign-off also settles §5.6's environment-block
> placement: it goes **after** the cache-stable portion and is **excluded from the
> fingerprint**, so switching branches does not invalidate the whole prompt.
>
> ~~**The delta is approved and owed, not applied.**~~ ✅ **Applied 2026-09-18 inside
> [A4](A-design-deltas.md)**, exactly where this entry said it would go: `Segment` and
> `ContextBuffer` are now normative in §4.5 as the buffer representation, with the four
> properties this recommendation argued for stated as spec text rather than as a
> rationale — appending is `push` + `extend_from_slice` with the digest computed once;
> cutting is O(1) `truncate` on both vectors and byte-identical to the prefix already
> there; **the segments are the source of truth and the assembled bytes a derived
> artifact**; and a model/provider switch replays the segment list through the new
> encoder rather than parsing the assembled string.
>
> **Both of the consequences this entry predicted turned out to be load-bearing, and one
> grew a third.** "A4's caching guarantees are byte-level, so without a segment index
> *the longest byte-stable prefix has no representation to operate on*" is now visible in
> §4.5's resync rule, which says the index is what makes resync cheap — with an edited
> `String` the longest stable prefix is something you discover by scanning text, with
> digests it is something you already know. **The §5.6 consequence landed too**: the
> environment block sits *after* the fingerprinted prefix and is **excluded from it**, so
> switching branches costs one segment rather than the whole prompt. The third is new and
> came from A4 rather than from this entry: **§4.8's task-tree rendering now cites the
> index by name**, because a status flip appending *one segment* is what makes "never
> rewrite bytes already sent" cheap rather than merely correct — which is also what
> discharged the second half of A19's cross-reference debt.
>
> **Still owed, and unchanged:** the plan-side references from
> [09](../part2-engine/09-context-caching.md) and
> [17](../part3-capabilities/17-config-agents-skills.md). Both are still migrated
> summaries, so hard rule 1 keeps them at summary depth until M1 deepens them; the spec
> is normative and the plan citation is the debt.

**Gates** [09](../part2-engine/09-context-caching.md),
[17](../part3-capabilities/17-config-agents-skills.md) · *Raised during sign-off
review; not in the original §16 list.*

**The requirement.** Prompts are built by appending — base harness prompt → agent
prompt → `AGENTS.md` → custom rules → skill index → environment block (§5.6) —
and the conversation then grows append-only on top. But three operations must
**cut later parts** rather than only append: compaction, prune-before-summarize,
and the longest-stable-prefix resync that A4 makes the core caching technique.
The question raised was whether that reconstruction can be done by **re-parsing**
the assembled prompt.

**Recommendation: no — keep a segment index, and never re-parse.**

```rust
struct Segment { kind: SegmentKind, offset: u32, len: u32, digest: [u8; 32] }
struct ContextBuffer { bytes: Vec<u8>, segments: Vec<Segment> }
```

- **Appending** is `segments.push` + `bytes.extend_from_slice`; the digest is
  computed once, at append time.
- **Cutting later parts** is `segments.truncate(n)` +
  `bytes.truncate(segments[n].offset)` — O(1), exact, and byte-identical to the
  prefix that was already there. That is precisely what longest-stable-prefix
  resync needs: walk digests from the front, find the last match, truncate.
- **Re-parsing is never required**, because parsing is only needed to *find*
  boundaries and the index already has them. Re-parsing an assembled prompt to
  locate segment boundaries would be lossy (a code fence containing something
  that looks like a boundary), slow, and would reintroduce exactly the byte
  instability the caching discipline exists to prevent.
- **The one genuine rebuild case** is a model or provider switch, where
  serialization changes (tool-call encoding, cache markers). That rebuilds by
  **replaying the segment list** through the new encoder — still not by parsing
  the previously assembled string. The segments are the source of truth; the
  assembled bytes are a derived artifact.

**Why it matters beyond tidiness.** A4's caching guarantees are byte-level. With
a `String` that gets edited and re-parsed, "the longest byte-stable prefix" is
something you discover by scanning text; with a segment index it is something you
already know, and the digest comparison that detects volatility is free. oh-my-pi's
`append-only-context.ts` is the same shape — a fingerprinted `StablePrefix` plus
an `AppendOnlyLog`, with `syncMessages` finding the longest byte-stable prefix.

**It also settles a §5.6 question.** The environment block (cwd, git branch, OS,
date) is the usual volatility culprit, and assembly order currently places it
*inside* the stable prefix. As a segment it can be placed **after** the
cache-stable portion and excluded from the fingerprint, so switching branches
does not invalidate the whole prompt. The segment model makes that a placement
decision; a flat string makes it a rewrite.

**Confidence high.** This is the mechanism A4's caching subsection already
assumes — without it, "truncate to the longest byte-stable digest prefix" has no
representation to operate on.

**On sign-off:** ~~fold into A4's proposed §4.5 text as the buffer representation,
and reference it from [09](../part2-engine/09-context-caching.md) and
[17](../part3-capabilities/17-config-agents-skills.md).~~ ✅ **Done 2026-09-18** for
the spec half — §4.5 carries `Segment`/`ContextBuffer` normatively, and §4.8 and §5.6
both cite the index for the properties this entry predicted. The two plan-doc
references remain owed and are blocked by hard rule 1, not by anyone forgetting:
docs 09 and 17 are still migrated summaries.

---

### 15. [B15] Built-in task tracking & gating — ✅ **RESOLVED 2026-09-13: fold-derived tree with an executable `verify:`**

**Gates** [32](../part2-engine/32-task-tracking.md),
[03](../part1-foundations/03-protocol.md),
[08](../part2-engine/08-agentic-loop.md),
[14](../part3-capabilities/14-tools.md),
[18](../part3-capabilities/18-workflows.md) · **Blocks** part of M1 (the tool and
its events) and part of M2 (the UI tree) · *Raised by the user after the original
pass; not in §16's list.*

Full evidence:
[task-tracking-and-gating](../../../.agents/knowledge/reference-harnesses/task-tracking-and-gating.md).

> **✅ RESOLVED 2026-09-13 — all five recommendation points and all four
> consequences signed off, none redirected.** Applied the same day via delta
> **A19**: a new design **§4.8** plus six consequential edits (§3.2
> `TaskUpdated`, §3.5 `Snapshot.tasks`, §4.2's gate in the loop pseudocode and
> its special-tool list, §4.3's `todo` entry, §12's `plans/`, §9.2/§10's tree),
> and the separable §5.4 `gate` → `approve` rename. **Gating strength landed as
> "hard but bounded"** — that was the fork this entry flagged as *low confidence,
> genuinely taste*, and it was taken as recommended rather than softened to
> advisory.
>
> **Applied by the A16 precedent, not verbatim — and this is the part worth
> reading.** A19's proposed §4.8 text cited three things the spec does not have:
> "the cache-stable prefix … its fingerprint (§4.5)" (§4.5 has no caching content
> at all; **A4** creates it), "rendered as a segment" (**B14**'s
> `Segment`/`ContextBuffer`, unsigned), and "joins the guard set … the same shape
> as A3's per-tool error budget" (§4.2 terminates on `if turn_count >
> agent.max_turns`; **A3** creates the guard set). Transcribing that verbatim
> would have put three dangling references into the normative document — exactly
> the defect class as §13's `dacode_json::Value`, a type that did not exist and
> had to be cleaned out of every agent's context. So §4.8 was made
> **self-contained**: it states its own caching requirement in its own words
> ("never rewrites bytes already sent … a provider's cached prefix survives")
> instead of citing a §4.5 fingerprint, and describes the gate without naming a
> guard set. A16 did the same thing when it defined §13.2's `Json` alias locally
> rather than editing unsigned §3.2/§4.3 signatures. When A3, A4 and B14 land, the
> cross-references get tightened — they are owned by those deltas, not by this one.
>
> **A19's own dependency claim was wrong, and is corrected here rather than left
> in force.** It said *"A1 must land first — the tier marker is what lets §3.2 say
> `TaskUpdated` is durable."* The spec has **no tier concept at all** today: §3.6
> persists *every* `Envelope`, deltas included, which is the defect A1 exists to
> fix. So §4.8 declaring task events durable in prose is consistent both before
> and after A1, and A1 was never the blocker. The real blockers were A3/A4/B14,
> and making §4.8 self-contained dissolved them instead of waiting on them.
>
> **One gap found while applying, and deliberately *not* patched.** `check` is red
> while a `Goal` is unclosed, with only `Blocked` and `WontFix` exempt — so a
> legitimately *standing* goal makes the gate red on every turn until the reminder
> cap burns. That is this repo's own `tools/tasks.py` failure mode (tracker
> finding f21: a gate that is always red gets ignored) reproduced in the product.
> Fixing it means a `Standing` status or treating `Goal` like `Debt`, and either
> is a wire-format change **inventing beyond the signed-off text**, so it is
> recorded as open decision **§16.6** instead — with the note that it must settle
> before the gate is implemented, and that it is a §14 compatibility event.
>
> **Two smaller scoping decisions.** §16 gained **no** B15 entry, though A19's
> text asked for one: §16 is the *open*-decisions list and A12's own proposal says
> not to empty it, so a closed decision that was never listed there needs no row.
> And [32](../part2-engine/32-task-tracking.md) was **not deepened** — only its
> `[DECISION B15]` and `[DESIGN DELTA]` markers were flipped — because M1 has not
> started and hard rule 1 forbids speculative depth.
>
> Version stays **v0.1** and the `Date` moved to 2026-09-13, per the convention
> under the spec's `Status` field: a delta application is not a version event.

**The requirement.** dacode's own development already runs on a two-tier task
discipline: `tools/tasks.py` (a branch-keyed tree of `goal`/`task`/`gate`/
`finding`/`debt` nodes, each with a mandatory `verify:` method, and a `check` that
exits non-zero on a done-without-verification item or an unclosed goal) plus
[TODO.md](../TODO.md) (the committed, cross-session board). The ask is to make
that a **product feature** — built-in task tracking with proper gating — rather
than a repo-local Python script.

**What the spec has today is one line, and three collisions.**

- §4.3 lists a `todo` built-in: *"per-session scratch list shown in UI"*. No tree,
  no kinds, no verification, no gate. This decision replaces it.
- §5.4's declarative workflows already use the word **`gate`** for `ask_user`
  step approval. Unrelated concept, same word.
- §12 lists `<repo>/.dacode/plans/` and **specifies nothing about it**. This is
  its natural occupant.
- Line 16's non-goal is *cross-session "memory"/"soul"*. A durable task board must
  be explicitly distinguished from that, or it reads as a violation.

**Options.**

- **(a) Keep the scratch list.** §4.3 as written: per-session, flat, UI-only, no
  gating. opencode's shape, ~800 LoC. Cheap, and it does not deliver the ask.
- **(b) A fold-derived tree with soft reminders.** oh-my-pi's shape: phases →
  tasks, state folded from the transcript, bounded stop-time continuation. ~2.4 k
  LoC. Delivers the tree and the events, not the verification.
- **(c) A flat list with an End-lifecycle hook.** forge's shape: a hook pushes a
  reminder and the loop refuses to yield. ~800–1 000 LoC, the gate itself ~25
  lines. Delivers gating, not structure — and forge ships it **off by default**.
- **(d) Recommended: a typed task tree as durable events, with a `verify:` field
  the engine can actually execute.** Below.

**Evidence that decides the three hard parts.** All from the digest.

1. **Gating is real but always soft.** forge's `PendingTodosHandler` fires on the
   End lifecycle and `orch.rs:419-441` sets `should_yield = false` when the hook
   added messages; oh-my-pi has four mechanisms including a prewalk gate that
   refuses to advance until a list is committed. But **every gate in the reference
   set is bounded** — forge dedups so a second consecutive Stop exits, oh-my-pi
   caps reminders at 3 — and none *fails* a run. An unconditional hard gate is
   therefore a hang risk with no precedent.
2. **Nobody gates on evidence.** No harness has a verification method, an evidence
   field, or a `check` that fails. Every mechanical enforcement found anywhere is
   concurrency or accounting integrity (busy-rejects, token budgets,
   terminal-transition errors), never outcome quality. **This is the
   differentiator, and it is green field.**
3. **Append-only holds in all four.** None renders a mutable list into a cached
   prefix: opencode never injects it at all, forge appends diffs and renders the
   full list *once per resume*, openclaw recomputes its goal line only inside the
   fresh inbound prefix, oh-my-pi appends a fresh hidden message and **excludes it
   from reviewer/export context because its counters change every turn**. The
   caching fork is settled by evidence, not taste.
4. **oh-my-pi's fold is dacode principle 2 already implemented.** State is derived
   by walking transcript entries backwards, so it is **branch-correct by
   construction** — rewinding a branch rewinds the list, with no reconciliation
   code. That is the same property §4.6 checkpoints need.
5. **`blocked` is excluded from reminders** (oh-my-pi), and that exclusion is
   load-bearing: without it, work genuinely waiting on a human gets nudged forever.

**Recommendation: (d), on these five points.**

1. **State is a fold, not a table.** Task mutations are **durable events** in the
   session log, folded into `Snapshot` (§3.5) — so replay, fork, undo and every
   client converge for free, and a rollback rewinds the tree. One `TaskUpdated`
   event carrying an op (oh-my-pi's op enum is the precedent) rather than five
   variants: fewer golden fixtures, same expressiveness. Per A1's two-tier model
   these are durable, never live-only.
2. **Item kinds, and a `verify:` that is typed by checkability.** Kinds are
   `goal`/`task`/`gate`/`finding`/`debt`. `verify:` is **mandatory at creation**
   and comes in exactly two flavours, because the digest's lesson is that only
   some verification is mechanically checkable:
   - `verify: command <argv>` (and `test <name>` as sugar) — **the engine can run
     it**. Closing the item runs it and the gate is the exit code. This is the only
     kind that can be *enforced* rather than asserted.
   - `verify: manual <what a human must look at>` — not machine-checkable. Closing
     it requires a **controller** action (§8.4), recorded as an event; the model
     cannot self-certify.
   That split is what makes "proper gating" real instead of theatrical, and nothing
   in the reference set has it.
3. **The gate is hard but bounded.** The loop may not emit a terminal turn while
   `check` is red — `check` being: a `done` item whose `verify` command fails or
   was never run, an unclosed `goal`, or untriaged `debt`. **Bounded** by a
   reminder cap (default 3, oh-my-pi's number) after which the engine surfaces the
   red gate to the user and stops rather than looping. **Exempt:** items in
   `blocked` (waiting on a human) and `wontfix`. Never unconditional — see
   evidence 1.
4. **Context rendering is append-only and outside the cache fingerprint.** The tree
   is rendered into the *new inbound* context as a droppable segment placed
   **after** the stable prefix and excluded from the fingerprint (B14's
   environment-block placement); tool results echo the **diff**, not the list
   (forge's `<todos_updated>`). This is A4-compatible by construction and it is
   what all four harnesses do.
5. **Two tiers, which is also how the non-goal is respected.** The session tree
   lives in the event log (durable, rewindable, per-session). Cross-session
   continuity is an **explicit promotion** to a committed board under
   `<repo>/.dacode/plans/` — a file the user reviews in a merge request, following
   forge's `plan_create` precedent of write-once repo-committed markdown. A
   *reviewed repo artifact* is not cross-session "memory"/"soul"; a silently
   accumulating agent store would be. Say so in the spec.

**Four consequences worth stating rather than discovering.**

- **`Verify::Command` is an exec surface, and signing off on this decision means
  signing off on that.** The engine runs an argv taken from a model-authored task
  item. It must therefore go through §4.4's permission engine exactly as `bash`
  does — `PermScope::Exec`, the same deny-list, the active §4.7 sandbox — and **the
  gate must never auto-approve its own verification command**. A `verify` needing
  approval blocks the close until the user grants it. Without that clause the
  feature is a privilege-escalation path: model-written argv, executed by the
  engine, possibly with no prompt. A19 carries the clause; it is called out here
  because it is the part of this decision with a security consequence, and it was
  found late enough that it deserves a second mention.
- **Subagents are a deliberate departure.** Both harnesses that have subagents
  *deny* them the tool (opencode by permission, oh-my-pi by stripping the
  registry). dacode's orchestrator delegates and needs subagents reporting into a
  shared tree, so: a subagent may update **items it owns** (spawned with the
  parent item id) and may not touch the parent's `goal`s or `gate`s.
- **A9's tree unification gains a third arm.** A9 already merges the workflow view
  and the sub-agent tree because both are "a node with live status that expands to
  a transcript". The task tree is the same shape. One component, not three.
- **The `gate` word collision should be resolved, cheaply.** Rename §5.4's
  workflow step attribute `gate` → `approve`, which is literally what it does
  (*"pause for approval of the plan"*), and keep `gate` for the task kind. One
  word in one example; separable from the rest of this decision if you would
  rather not touch §5.4.

**Sizing.** ~800 LoC for option (a), ~2.4 k for (b), and **(d) is at or above
(b)** — the tree and the fold are oh-my-pi's cost, and the executable-`verify`
gate has no precedent to copy. Budget ~2.5–3 k LoC excluding UI. Do **not** copy
forge's 192-line tool description (~90 % examples, ~1 KB of every session's tool
surface, and A4 makes tool-definition churn a cache kill): keep the description
short and put examples in the cached system prompt.

**Confidence: high on the model, the persistence and the caching** (points 1, 4, 5
— all three are precedent-backed and point the same way). **Low on the gating
strength** (point 3): "hard but bounded" is my recommendation and the evidence
supports bounded, but *how hard* is genuinely your call on taste. Say "advisory
only" and points 1/2/4/5 all stand unchanged.

**On sign-off:** apply delta **A19** (§4.3 `todo` → the task tree; new
`TaskUpdated` event + fold in §3.2/§3.5; the gate in §4.2; `.dacode/plans/`
specified in §12; the tree view in §9.2/§10); create
[32](../part2-engine/32-task-tracking.md) and deepen it at M1 under the
built-in-tools brainstorm gate; strike B15 here.

---

### 16. [B16] Repository tooling language — ✅ **RESOLVED 2026-09-16: two classes, narrowed on sign-off**

> **✅ RESOLVED 2026-09-16 — and the resolution is *narrower* than the
> recommendation below, which is the substance of it.** The user's rule was:
> *"the python scripts that relate to work on the repo/design docs, that are not
> shipped with the final harness and not required by harness for work — we can
> leave as python."* Applied to all five tools, that rule leaves **nothing to
> port**, so option (c)'s two-phase framing collapses. What landed is not "Rust by
> default with recorded exceptions pending deletion" but **two classes with a scope
> boundary**: tooling the shipped harness depends on is Rust in `xtask`;
> repo-*authoring* tooling is a script and is **permanent**, not transitional.
> Applied via delta **A20** → new design **§2.4** (+ §2.1, §2.2, §14).
>
> **Three things the recommendation below got wrong, corrected here rather than
> left to be inferred:**
>
> 1. **The line counts were stale by the time of sign-off.** The table said 1 738;
>    the five tools are **2 073 lines**, +335 (+19 %) in a single session, all of it
>    untested. Corrected below. The growth is itself an argument — see U79.
> 2. **"Drop the YAML dependency entirely" is not achievable.** `agents-sync.py`
>    *serializes* YAML as well as parsing it: the frontmatter of the generated
>    `.opencode/agent/*.md` is **opencode's** format, not ours to choose. And Python
>    has **no stdlib TOML writer** (`tomllib` is read-only), so moving the tracker
>    would trade PyYAML for `tomlkit` — a lateral swap in a tool already slated for
>    retirement. **PyYAML is permanent**, which removes the last dependency-based
>    argument for converting `agent-rules.yaml` to TOML.
> 3. **The B10 coupling dissolved.** It existed because porting three YAML-parsing
>    tools would force a YAML-crate choice. Nothing ports, so **no YAML crate enters
>    the Rust tree** and B10 is now free-standing — sign it on its own merits,
>    whenever. This is the one way the resolution made the decision set *smaller*.
>
> **`agent-rules.yaml` is harness-specific, and was checked rather than assumed.**
> Zero references to `agent-rules` in any of the five reference checkouts
> (opencode, oh-my-pi, openclaw, forge, open-code-review); its own header records it
> as "Inherited from the shelly-ng agent-rules layout". The only genuine
> cross-harness standard in this space is `AGENTS.md`, which the repo already has.
> **So TOML is the right eventual format** — it matches §5.1 ("TOML on disk") and
> §5.2's `.dacode/agents/<name>.toml` — **but the migration is a *layout* change,
> not a syntax change, and it belongs at M2** with the dogfood move, not as a
> standalone conversion now. Converting syntax alone would produce a TOML file that
> still is not a dacode config, at the cost of migrating the single source of truth
> every adapter is generated from. No YAML anchors or aliases are in use, so nothing
> is lost when it happens; the eight `|` prose blocks map to TOML `'''`.
>
> **The `check-links.py` dissent below won**, and not on its own argument: it wins
> by default, because nothing is being ported. Its "must run on an unbuilt checkout"
> case is nonetheless recorded in §2.4 as the one genuine exception class, since
> that reasoning holds independently of this decision.

**Gates** [01](../part1-foundations/01-workspace-scaffolding.md),
[31](../part6-quality/31-ci-milestones.md) · **Blocks** nothing today, but decides
what M0 scaffolds · *Raised by the user: "for the harness, we might not want to
use/rely on python in the system (for internal tooling) as much as we can — build
rust-based functions."*

**There is no policy today, only a habit.** `AGENTS.md`'s tech-stack table says
"Python 3 + PyYAML for repo scripts (`tools/`)" and the spec says nothing at all —
§2.3 (delta A17) governs what enters the *Rust dependency tree*, which is a
different question from what the repository requires of a contributor. The habit
has grown to ~~**1 738 lines across five tools**~~ **2 073 lines across five
tools** (recounted at sign-off; the table below is current, the original is kept
struck because a stale count that still reproduces in prose is how numbers get
quoted forever):

| Tool | ~~As drafted~~ | At sign-off | Parses YAML | *Writes* YAML | Has a dacode-feature analogue? |
|---|---|---|---|---|---|
| `attention.py` | ~~642~~ | **860** | yes | no | **yes — it is a view over §4.8** |
| `check-links.py` | ~~255~~ | **335** | no | no | no |
| `agents-sync.py` | 313 | 313 | yes | **yes — opencode's frontmatter** | no |
| `tasks.py` | ~~261~~ | **298** | yes | **yes — the tracker file** | **yes — it *is* §4.8's task tree** |
| `agents-index.py` | 267 | 267 | no | no | no |
| **Total** | ~~**1 738**~~ | **2 073** | 3 of 5 | 2 of 5 | 2 of 5 |

The *writes* column is what was missing from the original table, and it is the
column that decides the YAML question: reading YAML is replaceable by `tomllib`
from stdlib, **writing it is not**, because Python ships no TOML writer. Two of the
five tools write YAML, so PyYAML stays regardless of what the config files are
called.

**The tension is real and cuts both ways.**

- **Against Python:** principle 5 is a single binary with no external runtime.
  That claim currently covers the *product* and not the *project* — operating this
  repo needs Python 3 plus PyYAML, so a contributor and CI both carry a second
  toolchain. It is also ~~1 738~~ **2 073** lines of untyped, untested code doing
  work the product will eventually do typed and tested. *(Under the resolution this
  argument loses: authoring tooling is out of §2.4's scope, so the product will
  never do this work typed and tested. The untested half stands, and is U79.)*
- **For Python, right now:** there is no Cargo workspace. A Rust tool has nothing
  to compile against, so during the design phase Python is not a preference, it is
  the only thing that runs. `check-links.py` is the clearest case — if link
  checking needs a compiled workspace, the docs cannot be validated until the code
  builds, which inverts the current order.

**Options.**

- **(a) Status quo.** Python for all repo tooling, indefinitely. Cheapest, and it
  leaves principle 5 half-true.
- **(b) Rust immediately.** Not available: no workspace exists. Would need a
  standalone crate outside the workspace, which is more machinery than the problem.
- **(c) Two-phase, with an `xtask` crate at M0.** Python through the design phase;
  at M0 a `cargo xtask` crate in the workspace becomes the home for repo tooling;
  each Python tool is ported or retired as its replacement lands; **new tooling is
  Rust by default from M0**.
- **(d) Dogfood-first.** Where a tool duplicates a dacode feature, do not port it —
  **retire it and use dacode on its own repo.**

~~**Recommendation: (c) + (d), with this per-tool disposition.**~~ **Superseded —
every row of the table below was wrong once the user's rule was applied, because
all five rows assumed a port that is not happening.** Kept struck as the audit
trail; the confirmed disposition is the second table.

| Tool | ~~Disposition as drafted~~ | Why it was thought |
|---|---|---|
| `tasks.py` | ~~retire → dogfood §4.8~~ | It is the prototype; two implementations of one task tree will diverge. **This half survives** — see below. |
| `attention.py` | ~~split — the §4.8 view → dogfood; the scraping half → `xtask`~~ | The §4.8 view is product work. **The split survives; the `xtask` destination does not.** |
| `check-links.py` | ~~port to `xtask`~~ | Pure doc tooling, std plus a GitHub-slug function, cheapest first candidate. |
| `agents-sync.py` | ~~port to `xtask`~~ | Generates `.opencode/*` from `agent-rules.yaml`. |
| `agents-index.py` | ~~port to `xtask`~~ | Same shape, smaller. |

**✅ Confirmed disposition at sign-off — ~~this is the table §2.4's register
holds~~ this is the table §2.4's register was seeded from.** The normative copy is
[Appendix G](G-authoring-tools.md) (TODO §6's U80, 2026-09-16): this dossier is an
audit-trail document whose convention is that its content is historical, and a
register consulted as current state should not live in one. Rows below are kept as
signed off; **G is what a contributor reads.**

| Tool | Disposition | Trigger, and why |
|---|---|---|
| `check-links.py` | **Python, permanent** | §2.4's one genuine exception class: doc validation must run on a checkout that has not been built. Porting it would invert the order the design phase depends on. |
| `agents-index.py` | **Python, permanent** | Authoring only — indexes `AGENTS.md` docs. No product analogue, nothing shipped. |
| `attention.py` | **Python, permanent** — the §4.8-view half dogfoods later | The appendix/§6-scraping half has no product analogue and never will. When §4.8 exists, the task-tree view is product work and that half is deleted, not ported. |
| `agents-sync.py` | **Python, until the M2 dogfood migration** | Repo-only: it generates adapters for the *host* harnesses (opencode, Claude-Code-compatible `.mcp.json`), not for dacode. **Retirement trigger: dacode's config loader lands and this repo dogfoods itself.** By then `.dacode/agents/*.toml` is the real source and the opencode adapters are either generated from it or dropped. Kept rather than retired now on a measured cost: 878 source lines compose into **2 488 generated lines**, with `_search.md` and `_common.md` inlined into all five adapters and `_rust.md` into four — and `_common.md` must stay **byte-identical across agents because it is the cacheable prompt prefix**. Hand-maintaining that prefix across five files is exactly the drift B18/A21 plan to *measure*. |
| `tasks.py` | **Python + YAML, until §4.8 exists → retire & dogfood** | The one row that survives unchanged. **Blocked on §16.6** (standing goals): `tasks.py` is *where that bug was found*, so retiring it onto an unfixed §4.8 ships the bug. The repo-side half is already fixed — `--status standing` landed 2026-09-14 — which is *evidence for* §16.6, not a resolution of it. |
| `agent-rules.yaml` | **stays YAML**; becomes `.dacode/*.toml` at **M2** | A *layout* migration into §5.2's per-agent shape, not a syntax conversion. PyYAML is permanent either way, so there is no dependency argument for doing it early, and it is the single source of truth every adapter is generated from. |

Nothing ports to Rust. **`xtask` is still created at M0**, for the build-time
codegen the harness *does* require — B6's offline tool-schema generator, whose
output is compiled into `dacode-tools` — and §2.2 now records that no crate depends
on it.

~~**The wrinkle that makes this interact with B10.** Three of the five tools parse
YAML, so porting them puts a YAML crate in the Rust tree~~ — **⚠️ dissolved at
sign-off. B10 is now free-standing and this paragraph no longer applies.** Nothing
ports, so **no YAML crate enters the Rust tree** and the choice the coupling existed
to force cannot arise. Two further facts killed the other half of the paragraph:
`agents-sync.py` *writes* YAML as well as reading it, because the generated
`.opencode/agent/*.md` frontmatter is **opencode's** format and not ours to choose;
and Python has **no stdlib TOML writer** (`tomllib` is read-only, verified on
3.14.3), so moving the tracker to TOML would trade PyYAML for `tomlkit` — a lateral
dependency swap in a tool already slated for retirement. **PyYAML is permanent**,
which is also why `agent-rules.yaml` does not convert early: with no dependency win
available, the only remaining reason is consistency with §5.1/§5.2, and that reason
is realised by the M2 *layout* migration rather than by a syntax conversion now.
`serde_yaml`'s deprecation and its absence from
[Appendix F](F-c-dependencies.md) are now moot for dacode, since no Rust tool reads
YAML at all.

Original text, kept as the audit trail: ~~and `serde_yaml` is deprecated upstream
(verify before relying on it; it is not in Appendix F either way). **The cheaper
answer may be to stop using YAML for repo config at all**: move `agent-rules.yaml`
and the tracker file to TOML, drop the YAML dependency entirely, and reuse
`toml_edit` — already in Appendix F's confirmed-pure-Rust list and already required
by §5.1. That is a larger change than it sounds, because `agent-rules.yaml` is the
single source of truth every generated adapter comes from, and B10 is the decision
that owns TOML-on-disk. **Decide them together or B16 will quietly decide B10.**~~
The warning was well-founded — signing B16 alone *would* have settled B10 by
accident — and it was discharged by narrowing the policy instead of by deciding
both.

**✅ The dissent won — and by default rather than on its own argument**, since
nothing is being ported at all. Its reasoning is nonetheless preserved in §2.4 as
the one genuine exception class ("tooling that must run on an unbuilt checkout"),
because it holds independently of how this decision came out. Original text:

**The dissent, stated fairly.** Python scripts run instantly with no build step;
an `xtask` needs `cargo build` first, which on a cold checkout is slower than
`python3 tools/x.py`. For a link checker run on every commit that friction is real
and recurring. Mitigations: CI already builds the workspace, and a pre-commit hook
can use the cached binary. But the honest cost is that **doc validation stops
working on a checkout that has not been built**, and for a docs-heavy design phase
that is a downgrade. This is the strongest argument for keeping `check-links.py`
in Python permanently rather than porting it last.

**A third option worth naming for `tasks.py` specifically.** It could move into the
`dacode` binary as a subcommand (`dacode tasks …`) rather than `xtask`, since §4.8
is product functionality and a repo-only `xtask` wrapper would be re-exposing what
the binary already does. That is the dogfood argument taken to its conclusion: the
repo uses the shipped CLI on itself, and there is no second code path at all.

~~**Confidence: high on the direction** (it is your stated preference and principle 5
supports it), **medium on the schedule and on the port-vs-retire split** — the
disposition table above is a recommendation, not a derivation, and the
`check-links.py` dissent may well win.~~ **Both halves resolved, and the
medium-confidence half was settled by measurement rather than by vote** — counting
what `agents-sync.py` actually composes (878 → 2 488 lines, a byte-identical
cacheable prefix across five adapters) is what made "keep it" a derivation instead
of a preference. That is the pattern B12 keeps needing and keeps not getting.

~~**On sign-off:** apply delta **A20**~~ ✅ **done, 2026-09-16** — new design
**§2.4**, plus §2.1 (`xtask` in the crate tree), §2.2 (**no crate depends on
`xtask`** — a rule the delta did not contain, added because "ships nothing" is
otherwise unenforced) and §14 (`xtask` tested as ordinary Rust with a golden-output
test for generated bytes; authoring tooling explicitly outside the strategy).
~~Correct `AGENTS.md`'s tech-stack row~~ ✅ **done** — it stated the habit as
policy, and now states the two classes. ~~Read it alongside **B10**~~ — **no longer
needed**, the coupling dissolved. ~~**Still owed:** add the confirmed disposition
table to [01](../part1-foundations/01-workspace-scaffolding.md) **when it is
deepened at M0**, since that is the milestone that creates `xtask`.~~ ✅ **owed no
longer — the table is now [Appendix G](G-authoring-tools.md)**, the register §2.4
requires (TODO §6's U80, closed 2026-09-16). Doc 01 *points at* G when it is
deepened at M0 rather than carrying a second copy that would drift. Doc 01 was
deliberately *not* deepened here — M0 has not started, so the just-in-time rule
holds and the depth count is unchanged.

---

### 17. [B17] The built-in `reviewer` — ✅ **SIGNED OFF 2026-09-17: option (c), the split — usage defined by shipped workflows; the delta stays brainstorm-gated**

> **✅ SIGNED OFF 2026-09-17** — *"similar to a reviewer agent we have currently,
> just going to be shipped with harness. so, quite likely is your recommendation.
> just system instructions/prompt should properly define the use (perhaps, via
> pre-defined workflows)."* Taken as recommended — **option (c), the split**:
> `reviewer` stays an ordinary agent file, and the engine gains the three
> deterministic pieces — pure **reason-tagged changeset selection**, **`finding` as
> a durable structured event with no line numbers**, and the **snippet-anchoring
> positioning module**. No LLM grouping in v1; the reflection filter ships
> **defaulted off**; review checklists are **skills**.
>
> **The amendment is recorded as a requirement on the future delta**: the reviewer's
> *usage* must be defined by its system prompt **and by at least one pre-defined
> workflow** (§5.4), so how the reviewer is invoked is **shipped content** rather
> than something each user reinvents. That is the user's "perhaps, via pre-defined
> workflows" made concrete: a workflow, not only a prompt.
>
> **Still brainstorm-gated twice over, and the sign-off says so explicitly.**
> Signing B17 releases **neither** the built-in-tools (M1) nor the built-in-agents
> (M2) session; it gives them a concrete proposal to argue with, exactly as B15 did
> for task tracking. The delta is deliberately **not written yet**, because all
> three sections it would edit — §4.3's built-in list, §5.2's `reviewer`
> description, §3.2's `Finding` variant — sit inside those gates. `finding` is
> durable, so when the delta lands it lands **after A1**.
>
> **Owed, not applied**: the delta itself, drafted after the built-in-tools
> brainstorm — plus §5.2's "never edits" description documented as advisory on the
> `bash` half, which B17 owns while B7 (signed the same day) owns the mechanism.

**Gates** [14](../part3-capabilities/14-tools.md),
[17](../part3-capabilities/17-config-agents-skills.md),
[11](../part2-engine/11-subagents.md) · **Milestone** M2, with one M1 hook ·
**Evidence**
[open-code-review](../../../.agents/knowledge/reference-harnesses/open-code-review.md)
· **New — raised by the user after the original pass.**

**The question.** §5.2 ships `reviewer` as a five-line TOML file: read-only
tools, `mode = "plan"`, a prompt. That is the whole design. Alibaba's
`open-code-review` argues from two years of production that a general agent
given a review prompt is *measurably* worse than a purpose-built pipeline — and
names three failure modes: incomplete coverage on large changesets, **line-number
drift**, and quality that swings with prompt wording. Its numbers are not
reproducible (see B18), but the **failure modes are independently recognisable**
and two of them are structural rather than a matter of tuning.

**Options.**

| | Shape | Cost | Problem |
|---|---|---|---|
| **(a)** | Status quo — a prompt and a toolset | 0 | Ships the three failure modes. Line drift in particular is not a prompting problem. |
| **(b)** | Port the pipeline: deterministic selection, LLM grouping, plan phase, multi-round loop, positioning, reflection | ~1.5–2 k LoC Rust | A special case inside a harness whose first principle is that every client and agent goes through one protocol. A `reviewer` that is not an agent file breaks §5.2. |
| **(c)** | **Split it** — `reviewer` stays an ordinary agent; the *engine* gains the three deterministic pieces | ~700–900 LoC | Needs one new `Event` variant, so it sequences after A1. |

**Recommend (c).** The test applied to each piece was: *is a general agent worse
at this structurally, or merely untuned?* Only three pass.

1. **Positioning — `dacode-core`, not a tool.** The model emits a verbatim
   `existing_code` snippet and **never a line number**; the engine anchors it by a
   normalized sliding window over the hunk, then the whole file, then a unique
   cross-file search, and only then asks the model to regenerate the snippet.
   ~500 lines, no dependency beyond a diff parser
   (`internal/diff/resolver.go:62,151,217,301`). Two details are load-bearing and
   would not be rediscovered: blank lines are dropped on both sides so
   "consecutive" means *adjacent non-blank*, and the cross-file search must run
   **before** the model fallback, because that fallback overwrites the evidence
   (`internal/llmloop/loop.go:674-677`). Failure is declared, never silent — an
   unanchored finding is emitted with no line plus a warning.
2. **`finding` as a durable structured event.** `{path, existing_code, content,
   suggestion_code, category, severity}` with **closed enums and no confidence
   field**. This is principle 2 applied to review output: findings become a fold,
   not prose a second parser re-reads. It costs one `Event` variant and one golden
   fixture, and buys the TUI/web rendering, the dual-review flow, and
   `run -p --output jsonl` as a CI reviewer with no new surface.
3. **Pure, reason-tagged changeset selection.** One function that decides which
   changed files are in scope and *why* each excluded one was excluded, shared by
   the agent, a `--preview`, and any external consumer. OCR's five gates are
   individually unremarkable; what is worth copying is that the function is pure
   and returns a reason enum, so "why was this skipped" is never reconstructed
   (`internal/agent/selection.go:33-42,70-96`).

**Four things to explicitly *not* do**, each of which OCR does:

- **No LLM grouping in v1.** Deterministic bundling first (same directory,
  interface/impl pairs, i18n suffixes). OCR itself gates the model call at ≥4
  files, which is the admission that it rarely pays. Revisit only on a *measured*
  coverage failure — i.e. B18.
- **Reflection filter: ship the prompt, default it off.** Its design is the best
  precision idea in the survey — removal allowed only on two *proof* grounds, with
  memory-safety/concurrency/behavioural-change findings vetoed from removal
  outright, and the explicit asymmetry *"when your evidence falls short of proof,
  approve"*. But it is a second model call per round, which doubles the cost of a
  small review. Opt-in per invocation.
- **No 51-language rule-doc layer.** OCR needs one because it has no skills.
  dacode has §5.5 skills, loaded on demand with an index in the prompt — review
  checklists *are* skills. The transferable insight is the **content**: those
  checklists are mostly *do-not-report-when* lists with tool-verification hints,
  and their false-positive sections are longer than their issue sections.
- **No parallel session/manifest store.** dacode's event log subsumes it.

**One honesty fix rides along, and this is where it actually gets decided.**
§5.2 describes `reviewer` as *"Read-only code review; never edits"* while granting
it `bash:readonly`, which B7 establishes is a deny-list and therefore advisory.
(c) makes half of it real — `write`/`edit` path denial is enforceable — and the
`bash` half must then be *documented* as advisory rather than asserted as a
guarantee. B7 owns the mechanism; B17 owns the description.

**The delegation-mode insight is the strategic one, and it is free here.** OCR's
highest-leverage module is 415 lines that export only the deterministic half — the
file decision and the resolved rules — and let the *host* supply the intelligence.
Under (c) dacode gets that shape by construction: piece 3 is the export, and the
same function serves the TUI, the server, a sub-agent and an external caller.

**Interactions.**

- 🧠 **Brainstorm-gated twice over.** The built-in tools (M1) and the built-in
  agents (M2) both carry the gate in [TODO](../TODO.md) §5. **Signing B17 does not
  release either** — it gives those sessions a concrete proposal to argue with,
  exactly as B15 did for task tracking.
- **A1 first.** `finding` is durable, so it needs the tier marker and a golden
  fixture; the "A1 before any fixture" rule applies unchanged.
- **A2** — `finding` is a shared-concurrency tool.
- **B7** — the advisory-`bash` wording.
- **B18** — every "revisit if measured" above is a B18 measurement.

**Confidence: high** on positioning and structured findings (both are structural,
both are cheap, both are independently motivated by dacode's own principles);
**medium** on where the pipeline stops — deterministic vs LLM grouping is a
genuine fork and the honest answer is "measure it"; **medium** on reflection
defaults.

**On sign-off:** a delta is owed — §4.3's built-in list, §5.2's `reviewer`
description, and a §3.2 `Finding` variant — but it is deliberately **not written
yet**, because all three sections are inside the two brainstorm gates and writing
spec text before the session would be exactly the speculative depth the
just-in-time rule forbids. Sign B17 as a *direction*; the delta is drafted after
the built-in-tools brainstorm.

---

### 18. [B18] Harness benchmarking — ✅ **SIGNED OFF 2026-09-17: four tiers, three adopted; the publication rule normative**

> **✅ SIGNED OFF 2026-09-17** — *"ok, I agree with your recommendation."* Taken as
> recommended: **four tiers, three adopted** — **T1** deterministic replay benches
> in CI from M1 over a committed `events.jsonl` corpus, which **produces B12's
> missing number** as a by-product; **T2** token/cost accounting led by
> **prompt-cache hit rate**, which needs A4's `Usage` cache fields; **T3** the
> task-success runner at **M5 and never in CI**, over `run -p --output jsonl` with a
> `Verify::Command` as ground truth; **T4** cross-harness comparison an explicit
> **non-goal** — plus the **publication rule**: no number published, quoted in a
> doc, or used to justify a decision unless its scorer is in the tree, re-runnable,
> and its environment recorded alongside it. The med-confidence fork (T3 at M5,
> earlier if agent-quality regressions bite during M2) is taken as recommended.
>
> **The delta is approved and owed, not applied.** Owes: **A21** → §14 restructured
> into §14.1 correctness and §14.2 benchmarking with the publication rule
> normative; [33](../part6-quality/33-benchmarking.md) deepened when M1 starts; the
> T1 harness added to M1's task list; and **B12 re-run as a measurement** rather
> than continued as a stalled vote — this sign-off is what finally lets B12 be
> answered.

**Gates** [29](../part6-quality/29-testing.md),
[31](../part6-quality/31-ci-milestones.md),
[33](../part6-quality/33-benchmarking.md) · **Unblocks B12** ·
**New — raised by the user after the original pass.**

**The problem, stated plainly: the design has made a number of numeric and
performance claims, and nothing checks any of them.** §13.4 says byte-exactness
matters because it preserves the prompt-cache stable prefix — nothing measures
whether the cache ever hits. §3.7 picks a 256-envelope queue and a 5 000-event
replay window. §4.5 compacts at 80 %. §4.8 sets `reminders_max = 3` because that
is oh-my-pi's number. §3.6/§12's compression codec is **stuck as B12, which the board flags as needing a
benchmark rather than an opinion** — and there is nothing to benchmark it with.

> **B18 itself is an ordinary sign-off.** It decides *what dacode measures and
> what it may publish*; the numbers come afterwards. The distinction matters
> because B12 is the one entry in this appendix that genuinely cannot be settled
> by an opinion — and B18 is the thing that unblocks it.
>
> *(This section used to need careful phrasing, because `tools/attention.py`
> inferred an action type by scanning detail text and labelled B18 a measurement
> the moment it quoted B12. **That is fixed**: the action type is now declared in
> the `attention-meta` block at the top of this file, absence means `vote`, and
> the tool validates every id it names. Prose may now say whatever is clearest.)*

So this is not a "quality" nicety scheduled after the product works. It is the
instrument several already-made decisions are waiting on, and one already-blocked
decision cannot resolve without.

**Model benchmarks are not harness benchmarks.** SWE-bench and its relatives
measure the model. A harness benchmark holds the model fixed and isolates what the
*harness* contributes: how many tokens it spends before the user says anything,
whether its prompt assembly preserves a cache prefix, how much of a turn is its
own overhead, whether a change to compaction made things worse. Almost all of that
is measurable **without calling a provider at all**, which is what makes it
affordable.

**Four tiers. Adopt three.**

**T1 — deterministic replay benches. No model, in CI, from M1.**
criterion benches over a committed `events.jsonl` corpus: `fold` throughput,
`GetSnapshot` latency at N events, log append + fsync latency, bytes per 1 000
events, **encode/decode throughput and ratio per candidate compression codec**,
memory per live session, TUI frame cost under ≤16 ms coalescing, fan-out cost at
K subscribers.

dacode is unusually well placed here and it is worth saying why: **the event log
is already the fixture format**, so a benchmark corpus is a committed session and
`MockProvider` already replays canned chunks (§14). The harness is mostly wiring
that exists. **T1 produces B12's missing number as a by-product** — `flate2`
vs `lz4_flex` vs `brotli` over a real session log, on the machines we ship to.

**T2 — token and cost accounting. Deterministic given a recorded session. M1/M2.**
The harness's own product metric, and the thing OCR is right to lead with (its
one credible-sounding claim is ~1/9 the tokens, not the F1).

- **Prompt-cache hit rate** — cached-read input tokens ÷ total input tokens across
  a scripted multi-turn session. **This is the headline number**, because it is
  the one §13.4, §4.5 and §5.6 are all written to protect and the one that
  silently degrades. It requires cache fields on `Usage`, which is **A4's**
  content: B18 has a real dependency there.
- **Harness overhead before the user speaks** — tokens in the assembled system
  prompt (§5.6's six layers) + tool schemas + skill index + environment block +
  task-tree rendering. One number, and a design constraint rather than trivia.
- Tokens and turns per completed task; context bytes per turn; what `reminders_max`
  and the compaction threshold actually cost.
- **Stable-prefix drift as a benchmark, not only an assertion.** §13.3 already
  states that tool-schema churn *including pure reordering* destroys the cached
  prefix. Fingerprint the prefix across turns and across builds; fail on drift.

**T3 — task success against a real model. Runner at M5. Never in CI.**
The driver already exists: `dacode run -p "..." --output jsonl` (§8.1, §11) is a
headless one-shot emitting the raw envelope stream. So a benchmark task is a repo
at a pinned commit, a prompt, and a scorer — and **§4.8 already defines dacode's
pass/fail primitive**: a task item with a `Verify::Command` whose exit code is the
gate. A benchmark task *is* a task tree. Nothing new on the wire, which is a
strong argument that the abstraction is right.

Rules that make it honest rather than decorative: pin the model id **and** date;
run N repetitions and report a distribution, never a single number; record the
environment the way `dacode-json`'s `docs/RESULTS.md:12` does; and keep it out of
CI, because it costs real money and is non-deterministic — an `xtask bench`
invoked deliberately.

**T4 — cross-harness comparison. Explicit non-goal for v1.**
We have five checkouts and the temptation is obvious. It is also expensive,
adversarial, and unstable — every harness in the set moves weekly, so any number
is stale before it is quoted, and a comparison we publish is a claim we then have
to defend. **Measure dacode against itself over time.** Revisit only if there is a
specific external claim worth answering.

**The publication rule — the part with teeth.**

> **No number is published, quoted in a doc, or used to justify a decision unless
> its scorer is in the tree, re-runnable, and its environment is recorded
> alongside it.**

This is adopted directly from OCR's failure: its headline precision/recall/F1 and
token ratios have **no harness, scorer, dataset loader or matching rule anywhere in
the repository** — and the matching rule ("did the reviewer find this ground-truth
issue") is where essentially all of the measurement risk lives. The claims are
therefore unfalsifiable, which makes them marketing regardless of whether they are
true.

The repo already enforces this informally and has the scar tissue to prove it:
[TODO](../TODO.md) §6 carries a *"Do NOT quote these"* list of six retracted
`dacode-json` figures, and the methodology constraint that **both arms of any ratio
must come from one build** (a single edit to a bench file moved an in-process
measurement 14.43 → 17.57 µs). B18 makes the rule normative and extends it from the
JSON library to the harness.

**Confidence: high** on T1, T2 and the publication rule — they are cheap, they
unblock B12, and they check claims the design has *already made*. **Medium** on
T3's timing: M5 is the safe answer, but if agent-quality regressions start biting
during M2's built-in-agent work, it moves earlier. **High** on T4 as a non-goal.

**On sign-off:** apply delta **A21** (§14 restructured into §14.1 correctness and
§14.2 benchmarking, with the publication rule normative); deepen
[33](../part6-quality/33-benchmarking.md) when M1 starts; add the T1 harness to
M1's task list; and **re-run B12 as a measurement** rather than continuing to
treat it as a stalled vote.

---

### 19. [B19] Identifier privacy — ✅ **RESOLVED 2026-09-16: options 2 + 4, applied via A23**

> **✅ SIGNED OFF 2026-09-16** — *"If you recommend (2)+(4) — then is fine."* Both
> halves taken as recommended, none redirected: **prevention** on the two surfaces
> that retain (env-var injection under a reserved prefix excluded from §5.1's
> `DACODE_*` layer, an opaque scratch handle instead of §12's id-bearing path, and an
> exact `args` check over the four ULID-backed ids), and **narrow egress redaction**
> on §8.2's config endpoint, its four projection endpoints, and §7.1's hook payloads
> (which closes U61's second half). The low-confidence part — how wide the injected
> set goes — was **not** settled by the sign-off and stays open inside A23: start
> with one variable per id class and let B18's T2 overhead number argue for the rest.
> **The user's sign-off came with a second question that is not B19's subject** —
> session-log locking, indexing and concurrent access — recorded separately as
> **B21**, because answering it inside B19 would have smuggled a storage decision in
> on a privacy approval.
>
> **✅ A23 applied the same day**, so this decision is normative in the spec and not
> merely signed. The *On sign-off* block below records what the transcription settled
> that this entry had left open — and the one piece of debt it created for B7.

**Gates** [14](../part3-capabilities/14-tools.md),
[16](../part3-capabilities/16-permissions.md),
[24](../part5-outreach/24-extensions-channels.md) ·
**New — raised by the user 2026-09-16**, as a rule for the harness's agent
instructions: *"obfuscating any ids in the commands via shell vars or specially
crafted file names."* The repo-side half of that is **already landed** — it is a
bullet in [`.agents/roles/_common.md`](../../../.agents/roles/_common.md), which
every agent in this repository carries, and it did not wait for sign-off because
it governs our own agents today. What follows is the **product** half: what dacode
itself must do, which is a decision rather than an instruction.

**The problem.** The spec has exactly one redaction concept — §8.2's
`GET /api/config` returning "secrets redacted" — and no general one. Meanwhile
identifiers flow through three surfaces that either retain them permanently or
export them off-host. Which identifiers is worth stating precisely, because the
recommendation's detection half depends on the list: §3.1 fixes **six** id types —
`SessionId`, `ClientId`, `CallId` and `ReqId` (all ULID), plus `TurnId(u32)` and
`Cursor(u64)` — and alongside them sit sandbox and branch names (§4.7), provider
request ids, blob ids (a `blob_id` *field* on §4.3/§12, not a §3.1 type) and any
id the user's own domain puts into a prompt. ~~`ProjectId` is **not** among them
yet: it arrives with the unapplied delta A14, and B19 must be re-read when A14
lands.~~ ✅ **A14 landed 2026-09-18 and the re-read happened**: `ProjectId` and
`FolderId` both arrived in §3.1, and **neither joined the scanned set** — §4.3 now
excludes them in the clause shape `HostId` got, because the scan's real subject is
ids the engine *allocated and injected*, and nothing injects a project id nor can
any v1 tool argument name one. The count of §3.1 id types in this paragraph is
therefore historical: it is **eight** now, of which four are scanned, three
excluded conditionally (`HostId`, `ProjectId`, `FolderId`) and two excluded
permanently (`TurnId`, `Cursor`). The three surfaces:

1. **the session log**, which is append-only and never rewritten (§3.6,
   principle 2);
2. **the provider transcript**, including the prompt-cache prefix (§13.4) — so
   every id in a command is sent to a third party and *cached there*;
3. **the OS process list**, where an argv is readable by every local user for as
   long as the command runs.

**Why the log and the provider prefix cannot be redacted — and what can be.** For
two of the three surfaces, redaction is a rewrite the design forbids: the log is
append-only by construction (principle 2, §3.6), so whatever lands in it is
permanent and every client's fold derives from those exact bytes; and the
transcript's cached prefix must stay byte-identical across turns (§13.4), so
rewriting it on the way out costs the cache. **For those two, prevention before
the event is created is the only control that exists.** That is the load-bearing
argument, and it is why the rule belongs in the tool and permission layers
(§4.3/§4.4) rather than in a document — the same reasoning that made U37 patch
`Verify::Command` into §4.8 rather than log it.

**But it is not the only control everywhere, and claiming that would be
overstating it.** §8.2's `/export`, `/events`, `/blob/:blob_id` and `/diff/:call`
serve *copies* — projections of the log, not the log — and redacting a copy
rewrites nothing. Those four endpoints are therefore legitimate redaction points
and belong in option 4 below, which is where the "prevent, never redact" slogan
stops being true and starts being a rule about two specific surfaces.

**Options.**

1. **Instruction only.** Tell agents to use shell variables. Costs nothing,
   enforces nothing, and the model forgets. This is B7/U12's honesty problem in a
   new place: a control that is a hope is not a control, and the spec should not
   present it as one.
2. **Inject, then detect — recommended.** dacode sets the identifiers a tool
   needs as **environment variables in the tool's own process**
   (`DACODE_SESSION_ID`, `DACODE_TOOL_CALL_ID`, `DACODE_TMP`, …), so the supported
   path never needs a literal, and the built-in agents' prompts plus the `bash`
   tool description say to reference them. Then — because the engine *knows* its
   own ids — check a tool call's `args` for those literal strings **before**
   executing. That is exact substring matching over a finite, known set, not a
   heuristic, which is the difference between this and B7's deny-list.
3. **Alias or redact at every boundary.** Strongest on paper and it collides with
   §13.4: rewriting bytes in the transcript destroys the cache-stable prefix, and
   storing the alias table next to the log re-leaks exactly what it hides.
4. **Narrow egress redaction.** Redact on the paths that hand bytes to a party the
   user did not choose, and never on the log or the transcript. §8.2's config
   endpoint already has the spec's only redaction concept, and its four projection
   endpoints are copies, so redacting them rewrites nothing. **§7.1's hook payloads
   have no redaction rule at all today** — `post` ships the whole `Envelope` to an
   arbitrary URL — so they are not an existing example of this option but the first
   instance U61 asks it to create.

**Recommendation: 2 + 4.** Prevention on the paths that *retain* (log,
transcript, argv); redaction on the paths that *egress to a third party the user
did not choose*. U61's finding — `post` ships the whole `Envelope` to an
arbitrary URL with no redaction rule, while §8.2 redacts secrets — is this
decision's first concrete instance, and it should be closed here rather than at
M4, together with the four §8.2 projection endpoints named above.

Add a per-session scratch directory so artifacts need no id in their names: that
is the supported form of the user's "specially crafted file names", and it removes
the reason to invent one per call site. **Two traps in the obvious way to do it,
both of which would undo the rule:**

- **⚠️ The path must not embed the session id.** §12's session directory *is*
  `~/.local/share/dacode/sessions/<session_id>/`, so exporting that as the scratch
  dir hands every child a string whose expansion puts the id straight back into
  argv — surface 3, reintroduced by the mitigation. Export an **opaque per-session
  handle** instead (a symlink, a bind, or a directory named by a derived token that
  is not the id), and keep the id-bearing path engine-internal.
- **⚠️ `DACODE_*` is already taken.** §5.1's fourth config layer is
  "environment (`DACODE_*`) and CLI flags", so an injected `DACODE_SESSION_ID`
  shares a namespace with config overrides — and a model-authored
  `export DACODE_…` in a `bash` call could then reach a nested `dacode`, which is
  U61's injection class wearing a different hat. Reserve a distinct prefix for
  runtime injection and **exclude it from §5.1's override parsing**, or the
  convenience variable becomes a config-write surface.

**What changes in practice.** The decision reads abstractly, so here is the concrete
difference it makes to one tool call. A model that wants to inspect this session's
scratch directory today writes:

```sh
ls ~/.local/share/dacode/sessions/01J8ZK3M9QX7T2W4E6R8Y0A1B3/tmp
```

That single line puts the `SessionId` into four places at once: the `ToolCall.args`
in the append-only log, the transcript sent to the provider *and into its cache
prefix*, the argv visible to every local user in `ps` while it runs, and every
observer's screen. Nothing can retrieve it from the first two, by construction.

Under option 2 the same intent is written as:

```sh
ls "$DACODE_RT_TMP"
```

and the identifier never appears in any of the four. The variable is injected into
the tool's process by the engine, which already knows it; the path it expands to is
an opaque per-session handle rather than §12's id-bearing directory, so even the
*expansion* does not reintroduce the id. And if a model writes the literal form
anyway — which it will, sometimes — the engine recognises its own ULID in the `args`
and can warn or require approval before executing, because matching a known set is
exact rather than heuristic.

Three things this is **not**. It is not a redaction layer (option 3, rejected: it
rewrites bytes the log may not rewrite and the cache prefix may not change). It is
not secret-scanning — credentials are U65's and §8.2's subject, and an identifier is
not a secret, it is a *correlator*, which is why the log and the transcript are the
surfaces that matter. And it is not model discipline: option 1 is model discipline,
and the recommendation exists because option 1 is B7's mistake in a new place — a
control that is a hope.

**Confidence: high** that prevention must precede creation *for the log and the
provider prefix* — append-only and byte-exactness force it there, and they are the
two surfaces that retain. **Medium** on the `args` check, and the reason is not
what it first looks like: §3.1 **does** fix every id, which is precisely the
problem, because four are ULIDs (26 characters, safe to substring-match) while
`TurnId(u32)` and `Cursor(u64)` are **small integers** — exact-matching those
against a command string would fire on ordinary numbers like `3` or `17`. So the
check must be **restricted to the ULID-backed ids** and must not be described as
covering "the engine's own ids" wholesale. **Low** on how wide the injected set
should go — one variable per id class is the minimum, and each additional one is
prompt surface that B18's T2 overhead number will have to pay for.

**Coupling.** **B20 turns this from tidy into load-bearing**: once an agent runs
on another host, its command text and ids are retained in *that* host's log
forever, and a swarm of N holds N copies. U61 (hook templating into a shell
string), U65 (`?token=` in a URL, then in `Referer`) and this are one family — an
identifier or a secret placed into a channel that outlives the decision to put it
there.

**On sign-off:** ~~a delta editing §4.3 (env injection and the ULID-only `args`
check), §4.4 (where the check sits in resolution order), §5.1 (the reserved
runtime-injection prefix, explicitly excluded from the `DACODE_*` config layer),
§7.1 (hook payload redaction, closing U61's second half), §8.2 (redaction on the
four projection endpoints) and §12 (an opaque-handle scratch directory that does
not embed the session id)~~ ✅ **all six applied 2026-09-16 via
[A23](A-design-deltas.md)**, whose banner records the five scoping decisions
taken while transcribing. ~~Widen U61 and U65 to cite it rather than restating the
argument three times~~ ✅ **both widened the same day.**

**Two things the application settled that this entry left open, and one it did
not.** It settled *where the scan bites*: §4.4 now says a detection on an
automatic decision is recorded on `PermissionResolved` and **escalates to `ask`**,
with `yolo` recording without escalating. ~~⚠️ **That carve-out is B7's to confirm**
— B7 owns the mode's polarity and is unsigned, so this is debt in the same shape
A19 left for A3/A4.~~ ✅ **Confirmed 2026-09-18 by B7, inside A22's pass**, and
improved rather than merely confirmed: the carve-out stands, but since `yolo` is the
mode where a leak is most likely and the one mode where an escalation-based warning
would never fire, the first detection in a session now raises a **non-blocking
notice**. §4.4 carries both, with the reasoning, so this entry's debt is closed. It settled *what "rejected" means* for the reserved prefix:
a key **defined** in it is a load-time error, while mere presence in the
environment is expected, because the engine injects it and a nested `dacode` —
which is what a B20 swarm member is — would otherwise fail to start. It did
**not** settle the width of the injected set: that is written as a config list
starting at three, so widening it is a configuration change and this entry's
*low* confidence is discharged procedurally rather than answered.

---

### 20. [B20] Remote agent execution and swarms — ✅ **RESOLVED 2026-09-17: option 2, constraints now, build at M5**

> **✅ RESOLVED 2026-09-17 — the session this entry was waiting for happened.** The
> [sixth survey](../../../.agents/knowledge/reference-harnesses/remote-execution-and-swarms.md)
> ran, the nine questions were already answered, and the five the survey raised are
> settled below. **Option 2 stands: reuse the protocol, invent no second one — in B9's
> shape, commit the constraints now and build at M5.** The survey both validated that
> and priced it: openclaw *is* this shape and still needed a closed second protocol,
> for **epoch fencing** and **scoped rotating credentials** rather than for
> distribution — which is why constraint (e) exists.
>
> | Question | Answer | Lands |
> |---|---|---|
> | **Q5** — how does a member reach the model? | **Option A: a credential broker / reverse proxy at the controller.** The member runs `dacode-providers` and points its base URL at the controller, which injects the real key and forwards **verbatim**; usage is metered where the key lives. *Not* controller-executed inference. Three reasons, in order of weight: **principle 5** means a member already contains the provider stack, so option B's main benefit is worth nothing; **A adds no §3 surface**, while B needs new `Command`s/`Event`s, more variants for A1 to classify, and N provider streams held by the controller; and **A10/doc 25 already builds this component**, so the marginal cost is a member-scoped short-lived token. **Reserved:** the model endpoint is a **host/session attribute**, not a hardcoded proxy URL, so B stays reachable later with no wire change. **Non-negotiable either way: forward verbatim, never re-encode** (§13.4 — a proxy that re-serialises destroys the cache prefix it exists to measure). **The honest limit, true of both forms:** neither hides the *conversation* from the member, which holds the transcript regardless — so an untrusted host is answered by containment (Q4, B7), not by moving inference | A10/doc 25 + §5.3, **M5** |
> | **Q6** — a member or its network dies mid-tool-call | **(c): the default is block-and-ask; retry-on-another-host only where a workflow step opts in — and an opt-in step must declare its effect class**, honoured only when the step is idempotent *or* its `verify:` can detect a double run. The survey is what makes that condition necessary rather than merely prudent: **no harness has idempotence keys**, openclaw converges only the transcript, oh-my-pi retries the whole turn. Still requires A3's synthetic tool result, a first-class *aborted / effects unknown* outcome in §3.2, and constraint (c)'s host in the evidence | §4.8/§5.4 with **A3**, M5 |
> | **Q7** — agent or workflow orchestrates? | **Confirmed as suggested:** a workflow is a **DAG** (§5.4's step gains `host`, `needs`, `parallel`), **the whole swarm shares one §4.8 task tree** — a gate per host is not a gate — remote fan-out gets **its own cap**, separate from `max_parallel_subagents`, and `task` may also name a host for ad-hoc delegation. The survey corroborates the separate cap and records honestly that the DAG is a **choice, not a convention**: openclaw has both forms, robomp is queue-driven | §5.4/§5.3, **M5** |
> | **(e)** — new constraint from the survey | **Accepted: a fencing token.** Not merely "one `seq` assigner" but **one committer per session, fenced by an epoch, deduped by an idempotency key, with a CAS on the leaf**. **B21's kernel-owned `flock` gives the local form and cannot give the remote one** — there is no shared filesystem to lock, and Q1's network-FS mode has NFS-version-dependent `flock` semantics. Wire-visible, therefore **M1, with A1** | §3.1/§3.6/§3.7, **M1** |
> | **Option 4** — forge's *remote services, local effects* | **Declined as a B20 option, twice over:** A10 covers the only remote service dacode plans, and **hosted semantic indexing is an explicit design non-goal**, so forge's remote pure-function services are ruled out on their own grounds rather than merely deprioritized | — |
>
> **The four original constraints, restated.** **(a)** a `SessionId` able to name a
> session on another host — unchanged, still **M1 with A1**. **(b)** the capability
> split — **and this one gained a complication worth stating plainly**: it *was* A22
> item 2, which was **redirected to permissive for v1** the same day, so the split no
> longer arrives with A22 and must be argued here on its own merits, **for remote
> principals only**, leaving local permissiveness untouched. openclaw, the only
> harness that ships a remote worker, splits hard: a worker **cannot spawn**, and
> `sessions_spawn`/`send` are gateway-proxied. **(c)** the executing host recorded in
> `Verify::Command` evidence — unchanged, and Q6 depends on it. **(d)** one writer per
> session log — **superseded by (e)**, which is the same invariant plus the fencing
> that a partitioned remote writer makes necessary.
>
> **Owes a new delta: [A25](A-design-deltas.md).** Its **M1 half** (constraints (a)
> and (e)) rides **with A1**; its **M5 half** edits §5.3 (child location), §5.4 (the
> DAG), §8.4 (remote capabilities), §4.8 (verify-evidence host) and §4.7 (sandbox
> backends as declared host capabilities, jointly with B7), plus doc 20's scope. The
> build stays at **M5**, and B7's **inert stub** means the "sandboxed or refused" rule
> has nothing to point at until the seam is filled — **U98** names the route
> (BPF `execve` interception).

**Gates** [03](../part1-foundations/03-protocol.md),
[11](../part2-engine/11-subagents.md),
[19](../part4-hosts-clients/19-server-gateway.md),
[20](../part4-hosts-clients/20-server-scaling.md),
[24](../part5-outreach/24-extensions-channels.md) ·
**New — raised by the user 2026-09-16**: a sandboxing feature to come later, plus
agent execution on another host, *"so, a swarm of agents could participate in one
workflow."* **Doc 03 is in that list for one reason and it changes the priority:**
constraint (a) below is a §3.1 id-scheme question that has to be settled *with* A1,
which makes part of this decision an M1 concern rather than the M5 one its build
timing suggests.

> **This is not a vote yet.** It is declared `action: session` in the
> `attention-meta` block above: a reference survey is owed before there is a
> recommendation strong enough to accept or reject, and the sandbox prerequisite
> below is a real fork rather than a detail. B12 is the other non-vote in this
> appendix, for the opposite reason — it needs a number.

**What the spec already gives, which is more than it looks.** §8.1's fourth
process model is `dacode attach <url>` — a remote client over the same protocol.
§7.2 makes extensions ordinary protocol clients. §5.3 already states that a child
session's **only outbound channel is its final message**, returned as the tool
result, with transcripts *not* merged into the parent. §3.7 already makes a lossy
channel safe (bounded queue, `seq`-gap detection, resync, a fold idempotent over
`seq`). §7.3 already routes questions out to the world. Put together: **a remote
swarm member is very nearly "a `dacode` process on another host, attached as a
client, running a child session"** — which needs no second protocol, and is why
option 2 below is the shape to commit to.

**What it does not give — the actual forks.**

1. **`cwd` and the sandbox are host-local.** §5.3's child "inheriting
   cwd/sandbox" is meaningless across hosts, and §4.7's sandbox *is* a
   `git worktree`: a directory on one filesystem. A remote member needs a
   checkout strategy — clone at a pinned commit is the honest default — or a
   shared filesystem, which is a deployment choice and not a design one.
2. **§8.3 says "No users, no roles", and §8.4 lets any authenticated client
   `TakeControl`.** A swarm member is therefore indistinguishable from a hostile
   controller. This is U62 (`AllowPattern` lets an extension widen permissions
   without limit; split the `Hello` capability) with a much stronger motivation:
   capabilities must separate *observe / control / answer questions / grant
   permissions / execute remotely*.
3. **Whose log?** Principle 2 says events are the truth. Either envelopes ship to
   the parent's store — network fan-out, which §3.7 already makes safe — or each
   host keeps its own and the parent holds a reference, and then "one fold" must
   span stores, which is a bigger change than it sounds.
4. **A remote `Verify::Command` is a trust hole in §4.8's gate.** Its exit code
   gates the task tree, so a compromised or merely misconfigured host can forge a
   green gate. The evidence must record which host ran it.
5. **Non-idempotence — and the spec has no outcome for it yet.** §4.2's
   cancellation path drops the provider stream, kills the running tool's process
   group and marks the partial turn `stop: Cancelled`; it emits **no synthetic tool
   result**, and *synthetic tool results on abort* are the content of the
   **unapplied delta A3**, not of the spec as it stands. Meanwhile §3.2's
   `ToolResult { call, output, is_error, duration_ms }` has no *aborted / state
   unknown* outcome at all. Locally that is a wart; remotely it is a hole, because
   a killed call may have mutated a filesystem nobody can inspect. So B20 needs A3
   to land first and then needs `ToolResult` to be able to say *did not complete,
   effects unknown* rather than encoding that in an error string.
6. **Sandboxing is the prerequisite, not a sibling — and the user's ordering was
   right.** B7's part 4 calls the §4.7 worktree "the actual security boundary",
   and B7's widened banner with U63 puts the boundary at *the worktree plus the
   OS* — which is the accurate form, because a worktree bounds **where writes
   land**, not **what a process may do**: it contains an edit, not a `curl | sh`,
   not a read of `~/.ssh`, and not anything the process can reach outside the
   tree. The missing half is the OS one, and remote execution removes the
   mitigation the local case quietly relies on — that the blast radius is the
   user's own machine and their own checkout. So remote execution *without* an
   OS-level sandbox is remote arbitrary code execution by design. B7's reserved
   `SandboxBackend` seam is precisely what B20 spends, and
   [12](../part2-engine/12-checkpoints-sandboxes.md) is where the two meet.
   **B7's 2026-09-16 polarity amendment turns this from a warning into a rule:**
   the sandbox is opt-in *locally*, where the blast radius is the user's own
   machine and an informed `yolo` is a legitimate choice, and **mandatory
   remotely** — a member on another host either has a sandbox configured or is
   refused. Opt-in configuration and a distributed swarm are only compatible
   because the requirement tightens with the distance.

**Options.**

1. **Defer entirely**; sub-agents stay local. Cheapest, and it drops a direction
   the user has stated.
2. **Remote sessions over the existing protocol — recommended shape.** A swarm
   member is a `dacode` process elsewhere that attaches to the controller's
   `serve`; the parent sees a child session whose only outbound channel is its
   final message, i.e. §5.3 as already written. Reuses `Command`/`Event`, `Hello`
   capabilities, question routing and seq-gap resync. Build at M5, beside
   [20](../part4-hosts-clients/20-server-scaling.md)'s scaling work.
3. **A distribution layer of its own** — a scheduler, an RPC surface, a swarm
   protocol. Most flexible, and it would be a second protocol beside §3, which is
   principle 1's exact failure mode.

**Recommendation: option 2, in B9's shape — defer the build, commit the
constraints now.** All four are wire-format questions, so they are free before
golden fixtures exist and expensive after (A1's argument, and the reason A1 must
land before any fixture is written):

- **(a)** `SessionId`/`parent` must be able to name a session on another host — a
  location component or an opaque handle — decided *with* A1, not after it;
- **(b)** §8.4's capability flags split as in fork 2 above, which is U62's fix
  widened;
- **(c)** §4.8's `Verify::Command` evidence records the executing host;
- **(d)** **one `seq` assigner per session log, stated as an invariant.** §3.7's
  resync contract assumes the engine assigns `seq` when it appends, and §3.6 welds
  `Cursor` to the JSONL line number — the weld U67 already asks to have restated as
  "assigned by the store, strictly increasing". Two hosts appending to one session
  breaks both at once, and gap-detection-from-`seq`-alone cannot distinguish a
  dropped envelope from a concurrent assigner. So either a session has exactly one
  writing host (the cheap answer, and the one §5.3's final-message-only child
  already implies) or `seq` allocation becomes a distributed problem, which is a
  different design. **Commit to the first now**, because it is free while nothing
  is distributed and is a rewrite afterwards.

**Confidence: medium.** The shape — reuse the protocol, never invent a second one
— is high-confidence and follows from principle 1. The checkout strategy, the
log-locality answer and the sandbox prerequisite are genuinely open.

**What this needs you to decide.** Asked for explicitly — B20 is a `session`, not a
vote, and a session needs questions rather than a blank page. Nine forks, each with a
recommendation and the consequence of taking it. **Five are genuinely yours** (Q3,
Q4, Q6, Q7, Q9 — policy, security and product shape); the other four are
architecture I can defend and only need confirming, and two of those the survey will
inform rather than settle.

| # | Question | Recommend | Consequence |
|---|---|---|---|
| **Q1** | Where does a remote member get the repository? | **Clone/fetch at a pinned commit on the remote host** — not a shared filesystem, not a bundle shipped by the parent | The remote host needs git credentials and network to the origin. Self-contained, and it reuses §4.7's per-worktree model rather than inventing a second one |
| **Q2** | Whose event log does a remote member write? | **One log, on the controller** — envelopes ship over the wire | Preserves principle 2 ("one fold") and reuses §3.7's resync; export and replay stay whole. Costs network fan-out on the hot path, and a partitioned member buffers or loses *live* deltas — recoverable, because gap detection already exists. The alternative (a log per host) breaks §3.5's `Snapshot` unless the fold can span stores, which is a much larger change than it sounds |
| **Q3** | Who may spawn a remote member, and with which capabilities? | **Only the controller spawns**; a member gets *execute remotely* + *report*, and **never** *grant permissions* | This is U62's capability split, widened. Without it §8.4's "any authenticated client may `TakeControl`" plus §3.3's `AllowPattern` means one connected bridge can mint a swarm member with session-wide `**` — i.e. any bridge is root |
| **Q4** | What satisfies the mandatory remote sandbox? | **In-process OS sandboxing is the floor** (B7's `SandboxBackend`: Landlock on Linux, Seatbelt on macOS); a container is permitted as a *stronger* option but never required | Keeps principle 5 (single binary, no external runtime) intact — openclaw's Docker route is the counterexample at 11 258 LoC. The cost is real: **a remote host whose OS has no supported backend cannot run members at all**, which is the honest price of the floor |
| **Q5** | How does a remote member's work come back? | **It pushes a branch; the parent merges** — §4.7's existing "merge back" action | Keeps a reviewed artifact instead of a blob in a tool result, and stays compatible with §5.3's "only outbound channel is its final message" for *state* while the *work* lands in git. Needs one small addition: the child's final message must carry the branch name |
| **Q6** | What happens when a remote host vanishes mid-tool-call? | **Mark the task item `blocked` and ask the human.** Retry-on-another-host only as an opt-in per workflow step | A silent retry of a non-idempotent remote mutation is how a filesystem ends up half-changed twice. Requires A3's *aborted, state unknown* outcome (fork 5) and constraint (c)'s host in the `verify` evidence, or a green gate is unattributable |
| **Q7** | Does an *agent* or a *workflow* orchestrate the swarm? | **Both, but the workflow is the primary form** — your phrase was "participate in one workflow", which is §5.4; §5.3's `task` may also target a host for ad-hoc delegation | §5.4's step gains an optional `host`. Fan-out gets a **separate** remote cap rather than reusing `max_parallel_subagents` (default 4), because the failure mode and the cost are different — a local sub-agent that dies is cheap, a remote one may have mutated something |
| **Q8** | Does a remote member's spend roll into the parent's cost? | **Yes, but only after A4 lands** | U71: `cost_usd` silently prices cached reads at full input rate until `Usage` gains cache fields. A wrong cost number is worse than an absent one, because it will be believed |
| **Q9** | When? | **Split it**: constraints (a) and (d) with **A1 at M1**, the capability split with M1's permission engine, the build at **M5** beside doc 20 | The wire-format halves are free before golden fixtures exist and expensive after; the build is genuinely M5 work. Signing B20 as "all at M5" would forfeit the only cheap moment |

**✅ The nine questions were answered 2026-09-16 — B20 is narrowed but still not
signed off**, because the survey is owed and Q6/Q7 need one more exchange. What the
answers settled, and what each one changes:

| # | Answer | Consequence |
|---|---|---|
| **Q1** | *"can be either a cloned repo or network fs"* — both, configurable | Two acquisition modes, not one. ⚠️ **This collides with B21:** `flock` semantics are NFS-version-dependent (v3 needs a lock daemon, v4 has it), so a session directory on a network FS either requires NFSv4 or must stay local. Recorded in both dossiers rather than one |
| **Q2** | Reframed, and the reframe is better than the question: *"running models through a common proxy — remote agents never use local's credentials, but instead have their model API pointed at a reverse proxy from the controller socket"* | **This is already designed — it is doc 25 / delta A10's transparent logging proxy** (bind local ports, forward verbatim, reassemble SSE with no added latency, redact secrets in logs but pass them upstream untouched, extract usage including `cache_read_input_tokens`). The user derived it independently as a *credential broker*; A10 has it as a *debugger*. **One component, three jobs**: credentials never reach a remote host, usage is metered where it is spent rather than reported by whoever spent it, and the exchange record doc 25 wants is a by-product. Constraint that makes or breaks it: **forward verbatim, never re-encode** — §13.4's byte-exactness is why `RawJson` exists, and a proxy that parses and re-serialises bodies destroys the prompt-cache prefix it is meant to measure |
| **Q3** | *"let it be controller only"* | Settled as recommended: only the controller spawns; a member gets execute + report and **never** grant-permissions (U62's split, widened) |
| **Q4** | *"as you suggest (can be docker, can be remote host with isolation, if the system allows, can be similar to nsjail or docker)"* | **Widens the recommendation, correctly.** An external isolator is acceptable on a remote host, so the floor is "the host provides real containment" rather than "in-process only". This does *not* breach principle 5, and the reason is already in the spec: §2.4 scopes principle 5 to **the product and its build**, not to the environment it runs in. So an isolator is a **declared host capability**, negotiated at `Hello`, never a dependency of the shipped binary. A host that cannot provide one refuses members |
| **Q5** | *"remote agents might not necessarily work on code… It can do local commits (then merge/pull/push) if needed. Could be configurable."* | The return channel is **per-step configurable**, not fixed: a git branch for code work, a payload in the final message for everything else. §5.3's "only outbound channel is its final message" survives — the branch name *is* part of that message |
| **Q6** | *"what means 'vanishes mid-tool-call'?"* | **Still open — the question was not clear enough.** It means: the parent dispatched work to a member and the member or its network died before returning a result, so nobody knows whether the command ran, half-ran, or never started. Locally §4.2 can inspect the filesystem; remotely it cannot. Answered in the session notes; the fork (block-and-ask vs retry-elsewhere) is re-put below |
| **Q7** | *"what could you suggest? workflow consists of different chain (seq and/or parallel) agents/sub-agents"* | **Suggestion recorded below** — a DAG of steps, not a flat fan-out |
| **Q8** | *"yes"* plus a new opt-in feature: *"the harness could inject current usage/context back as a prompt (live statistics/hints for the model)"* | Cost rollup confirmed. The new feature is **U90**, and it has one hard constraint: per-turn usage numbers are exactly the volatile data §13.4 forbids in the cached prefix, so it must ride in an appended, droppable **B14 segment** and never in the stable prefix. f25 is the precedent — oh-my-pi deliberately excludes its volatile goal counters from reviewer/export context for the same reason |
| **Q9** | *"not block too much in the beginning, but not refactor too much later"* | **That phrasing is the answer, and it is B9's shape**: commit the constraints, defer the build. Concretely — (a) host-nameable session ids and (d) one `seq` assigner **with A1 at M1**, the capability split **with M1's permission engine**, everything else **at M5** beside doc 20. Signing B20 as "all at M5" would forfeit the only cheap moment |

**Q7's suggestion, since it was asked for directly.** A workflow is a **DAG**, and
§5.4's declarative form already has steps — so: a step gains `host` (absent = local),
`needs` (sequence) and `parallel` (a group whose members run together); **the whole
swarm shares one §4.8 task tree**, because a completion gate per host is not a gate
— the parent's `check` must be able to see a remote step's `verify:` result, which is
what constraint (c)'s host field is for; remote fan-out gets **its own cap**, separate
from `max_parallel_subagents` (default 4), since a dead local sub-agent costs tokens
and a dead remote one may have mutated something; and `task` may *also* name a host,
so an agent can delegate ad hoc without a workflow. The workflow form is the durable
one — it is a task tree, so it is resumable, gated and reviewable — and the agent form
is the opportunistic one.

**✅ The survey is done — 2026-09-17, the sixth digest:**
[remote-execution-and-swarms](../../../.agents/knowledge/reference-harnesses/remote-execution-and-swarms.md).
**Headline: nobody has a swarm; one harness has remote execution and paid ~51.9 k
LoC for it.** Four of the five execute nothing off-host. openclaw does, and it *is*
option 2's shape (remote process, controller-owned log, controller-owned spawn and
permissions) — **and it still needed a closed second protocol**, not because
distribution is hard but because a remote writer must be **fenced** (`runEpoch`,
turn claims keyed on it, stale workers rejected `owner-epoch-mismatch`) and
**re-credentialed** (short-TTL rotating scoped credentials). Those two are what "a
swarm member is an ordinary client" lacks, and both are free before golden fixtures
exist. What the survey changed here:

- **Constraint (d) is understated and should be restated.** Not "one `seq` assigner
  per session log" but **one committer per session, fenced by an epoch, deduped by
  an idempotency key, with a CAS on the leaf** — openclaw's transcript commit and
  opencode's prose invariant (*"only one device is allowed to write… total ordering
  with a simple sequence id"*) agree. **B21's kernel-owned `flock` gives the local
  form and cannot give the remote one**: there is no shared filesystem to lock, and
  Q1's network-FS mode has NFS-version-dependent `flock` semantics. So (d) needs a
  wire-visible fencing token, and it is a fifth constraint rather than a refinement
  of the fourth.
- **Q1's answer is incomplete, not wrong: there are four acquisition modes.** Clone
  or network FS are two; the only real implementation uses the other two — a
  parent-prepared workspace **rsync'd with a git-pack bundle**, and **shipping the
  harness binary itself** (worker bundle prewarm). The second is the one dacode's
  single-binary principle makes cheapest. No surveyed harness uses a network FS for
  a remote member.
- **Q2's reframe is validated twice, at two different strengths**, and the choice
  between them is now explicit: a **credential broker** (oh-my-pi's `auth-broker`,
  4 191 LoC, refresh tokens never leave it — this is A10/doc 25's shape and keeps
  the worker's provider stack local) vs **controller-executed inference** (openclaw's
  `worker.inference.start/cancel`; the worker never holds the request, and the
  provider stream crosses the controller↔worker wire and needs bounded replay).
  Both live or die on **forward verbatim, never re-encode**; the survey found no
  evidence either way, so that constraint is asserted here rather than observed.
- **Q4's widening is corroborated, with the mechanism:** containment arrives as a
  **declared capability negotiated at admission** (protocol-feature set plus
  capability descriptors, placement eligibility and `availableSlots`), and openclaw's
  `backend-handle.types.ts:4` — *"Docker, SSH, and future providers implement these
  surfaces"* — is B7's reserved `SandboxBackend` seam already built.
- **Q6 now has evidence rather than only reasoning: no harness can say whether a
  side effect happened.** openclaw converges the *transcript* (idempotency keys,
  leaf-CAS, terminal `orphaned`) and forces a human-visible "retry locally" when
  provider replay is lost, but a death between side effect and commit leaves nothing
  markable, and no `ToolResult` shape anywhere has an *aborted / state unknown*
  outcome. oh-my-pi retries the whole turn with no idempotence keys. So
  **block-and-ask is not the cautious option, it is the only one anybody has**, and
  the dependency B20 already named is confirmed: A3's synthetic tool result, a
  first-class *effects unknown* outcome in §3.2, and constraint (c)'s host in the
  `verify` evidence.
- **Q7's DAG is a design choice, not an industry default.** openclaw has both forms
  (agent-runtime spawn tools *and* a swarm scheduler), robomp is queue-driven by
  webhook events. What *is* corroborated is the **separate remote cap** — openclaw's
  is "maximum concurrent worker sessions hosted by this node (default CPU cores)",
  independent of its swarm caps.
- **A third option is missing from the list above, and it is the only one any
  harness ships:** forge's **remote services, local effects** — the loop and every
  side effect stay local, only *pure* functions go remote over gRPC (≈850 LoC). It
  buys distribution of the expensive stateless work with none of the fencing,
  containment or effect-attribution problems, and it deserves a row beside options
  1–3 rather than a footnote.

**Still owed before this is signed:** the session itself — **Q6** (re-put below with
the evidence) and **Q7** (the DAG suggestion, now known to be a choice rather than a
convention), plus a decision on the two new items the survey produced: the **fencing
token** as constraint (e), and whether to adopt forge's *remote services* form as a
first stage that precedes remote *execution*.

~~**Owed before this is voted on:** a sixth reference survey of what the existing
harnesses do about remote or distributed execution.~~ The
[sandboxing-and-checkpoints](../../../.agents/knowledge/reference-harnesses/sandboxing-and-checkpoints.md)
digest already establishes the nearest thing any of them has — openclaw's opt-in
Docker, off by default — which is evidence about *sandboxing*, not about swarms,
and must not be quoted as if it were the other. **The new digest honours that
distinction explicitly**: containment appears in it only as a property of a *remote*
worker.

**On session:** ~~run the survey; settle B19 first, because a swarm logging
identifiers on N hosts is B19's problem multiplied by N;~~ ✅ **both done** — the
survey landed 2026-09-17 and **B19 was signed off 2026-09-16 and applied via A23**,
so a swarm logging identifiers on N hosts is now governed rather than open. What the
session still owes: Q6 and Q7 above, the fencing token as constraint (e), and
whether forge's *remote services, local effects* form is adopted as a stage that
precedes remote *execution*. Then a delta editing
§5.3 (child location), §8.4 (capabilities), §4.8 (verify evidence host) and §4.7
(sandbox backends, jointly with B7), plus [20](../part4-hosts-clients/20-server-scaling.md)'s
M5 scope — and note that **A22's item 2 was redirected to permissive for v1**, so
the capability split B20 needs is no longer landing with A22 and must be argued on
its own merits here.

---

### 21. [B21] Session-log concurrency, locking and indexing — ✅ **RESOLVED 2026-09-17: one writer, one lock, an index that is a projection**

> **✅ SIGNED OFF 2026-09-17** — the recommendation and **all five sub-answers taken
> as recorded**, none redirected. Applied the same day via
> **[A24](A-design-deltas.md)** → design **§3.6** (the single-writer invariant, the
> advisory lock and its three reliability properties, the store-assigned cursor),
> **§4.1** (the registry is the lock holder; a second opener is an error, not a second
> writer), **§8.1** (`attach` named as the remedy), **§12** (`lock` and `index.bin`
> beside `events.jsonl`, plus the locality rule) and **§14** (the two tests). Doc 05
> gained **§5.5** with the implementation obligations the spec deliberately does not
> carry. **No `Event` or `Command` variant, so no golden fixture** — the lock sits
> below the protocol, which is the point of it.
>
> **Two things this sign-off did that were not in the recommendation.** It
> **discharged [U67](../TODO.md)** rather than leaving it for M1: an index of
> `cursor → byte offset` is only worth having if the cursor is not *already* the line
> number, so §3.6's `Cursor = line number + 1` had to be restated as store-assigned in
> the same edit — the two were coupled, and landing them separately would have had the
> spec asserting an equivalence its own index contradicts. And it **had to be written
> against the post-A23 §12**, which gained `tmp/` and `scratch/<token>` the day before;
> that is the sequencing the board predicted when it landed A23 ahead of B21, and it
> held.
>
> **What stays open, and why it is safe to sign anyway.** Confidence was **medium** on
> the offset sidecar lasting through M1, and that has not changed — it is now a
> recorded exit route rather than a resolved question: **if `dacode session list` needs
> more, the index grows a second table rather than the log changing format.** The
> high-confidence halves (single-writer plus an advisory lock, JSONL-as-truth with a
> rebuildable index) are what the spec now asserts.
>
> **The coupling to B20 is recorded, not resolved.** This is B20's constraint (d) at
> local scale, and **a swarm whose members each held their own lock would satisfy B21
> and violate B20.** B20 remains `action: session` and must re-read this entry; the
> NFS question it raises is *dissolved* by sub-answer 2 (the working tree may be
> remote, the log may not) rather than answered, which removes `flock`'s
> version-dependence from the design instead of documenting it.

**Gates** [05](../part2-engine/05-event-log-storage.md),
[07](../part2-engine/07-session-registry.md),
[19](../part4-hosts-clients/19-server-gateway.md) ·
**New — raised by the user 2026-09-16** while signing off B19, as a separate
question: *"regarding the session, we should have a lock mechanism over it. and some
kind of index of messages (i.e. sqlite?). Re rewrite and so on. What is your
suggestion and best practices? What's more reliable and less error-prone? What might
not raise some conflicting (accidental session access by several instances)?"*

> **The gap is real and it is total: the spec says nothing about concurrent access to
> a session.** Verified by grep, not by recall — `flock`, lock file, single-writer and
> "two processes" appear nowhere in `docs/dacode-design.md`. §4.1 gives one
> `SessionRegistry` **per process**; §3.6 gives an append-only JSONL log fsync'd on
> `TurnFinished`/`ToolResult`/request-answer; §8.1 lists four process models, three of
> which can be running at once. Nothing states what happens when a TUI and a
> `run -p` open the same session directory, and today the answer is *both append*.

**Why "both append" is worse than it sounds.** It is not a torn-line problem. A
single `write()` of one line is atomic on a local filesystem, so the JSONL stays
parseable — and that is exactly what makes the failure quiet. What breaks is **`seq`**:
§3.7 says the engine assigns `seq` when it appends and that it is strictly increasing,
so two writers each produce a valid sequence whose union is neither. §3.7's gap
detection then reports phantom discontinuities, clients resync forever, and
`Cursor = line number + 1` (§3.6) makes the two writers' cursors *collide* on the same
line. A log that is corrupt in its ordering rather than its bytes is the hard kind to
notice, because it still parses.

**Options for the lock.**

1. **Advisory file lock** — `flock(2)` on POSIX, `LockFileEx` on Windows, on
   `<session>/lock`. **Recommended.** The decisive property is that the kernel owns
   it: the lock dies with the process, including on `SIGKILL` and on crash, so there
   is **no stale-lock recovery code to write and no `--force-unlock` flag to grow**.
   Pure-Rust crates exist (`fs2`, `fd-lock`) whose only non-Rust touch is `libc` /
   `windows-sys` — class 2 under §2.3, exempt, no Appendix F row.
2. **PID + timestamp file with stale detection.** The classic, and the wrong one:
   PID reuse, clock changes, a crashed process leaving the file behind, and a race
   between "is this PID alive?" and "take the lock". Every implementation of this
   eventually ships a force flag, which is the tell that the mechanism is unsound.
3. **`O_APPEND` alone.** Not a lock, for the `seq` reason above — and on NFS it makes
   no atomicity promise at all.
4. **SQLite as the log.** Buys transactions and an index, and costs: a C dependency
   (Appendix F's `libsqlite3-sys`, "avoidable only at a design cost"), documented
   unsafety over NFS, an un-grep-able log where §3.6 and doc 05 deliberately keep it
   plain, and — decisively — **it still does not stop two engines both writing
   events.** It prevents torn writes, not logical duplication.
5. **Single-instance daemon.** The lock is "whoever owns the socket". Already §8.1's
   `serve` model, and the right answer for *many clients* — just not for *two
   engines*, which is the case that needs a lock.

**Recommendation: single-writer as an invariant, enforced by 1, with 5 as the way to
have many clients.** Exactly one process appends to a session's log — the one holding
`<session>/lock`. Everyone else is a **client** over §3.7's protocol (`attach`, the web
UI, extensions), which is principle 1 doing the work it was designed for: the protocol
exists so that a second consumer never needs write access. A second engine gets a
clear error naming the holder and the remedy (`dacode attach <url>`), never a silent
second writer.

**This is B20's constraint (d) at local scale.** "One `seq` assigner per session log"
was written there as a distributed-systems commitment; a file lock is the same rule
enforced on one host. Recording them as one rule is the point — a swarm whose members
each held their own lock would satisfy B21 and violate B20.

**The index.** Three rules, in order of importance:

- **The log stays JSONL and stays the truth** — append-only, grep-able, crash-safe
  under fsync, no schema migration, readable by any tool. Doc 05 keeps it plain by
  default for exactly these reasons.
- **The index is a rebuildable projection beside it, never authoritative.** That is
  already the rule A14/B13's Fix-1 states for organization data ("a rebuildable
  projection for querying, never authoritative"); extending it to the cursor index
  costs nothing and inherits the property that matters — **a corrupt or stale index is
  recoverable by scanning the log, and a corrupt log is not recoverable at all.**
- **At M1 the index is a sidecar of `cursor → byte offset`.** That is all
  random-access replay needs: §3.7's window is 5 000 envelopes in memory, and beyond
  it is a disk read, which without an index means scanning from byte zero. It needs no
  SQL and no C dependency, it is rebuilt by one pass, and it degrades safely — an entry
  pointing past a truncated final line is detectable and droppable, which is the
  property a torn tail needs. **At M3** it becomes `SqliteStore` or `redb` per B11,
  when doc 06 §6.4's `?project=&tag=&folder=&q=` filters need real queryability;
  Appendix F already records that trade.

**"Rewrite" is not an operation, and that is the same fact twice.** The log is never
rewritten (§3.6, principle 2); compaction drops turns from the live `Context` and
records a `Compacted` boundary (§4.5) while the log keeps everything. It is worth
naming that this is *also* B19's premise: if the log could be rewritten, redaction
would be an option, and the reason it is not — one writer, append only — is the same
reason a lock is needed rather than a merge.

**Confidence: high** on single-writer plus an advisory lock; the alternatives are worse
in documented ways rather than as matters of taste. **High** on JSONL-as-truth with a
rebuildable index, which doc 05 and A14 already assert. **Medium** on the offset
sidecar being sufficient through M1 — it is, until the session-list filters land, and
if `dacode session list` needs more the index grows a second table rather than the log
changing format.

**The five sub-questions inside B21, with the variant recommended for each.** Asked
for explicitly — "reliable / least error-prone" is a different question from "what
works", and each answer below is chosen on that basis rather than on convenience.

1. **Which lock primitive, and held how?** A maintained cross-platform crate —
   **`fd-lock`** (rust-cli, `try_lock` plus a guard released on `Drop`) over `fs2`
   (older, `FileExt::lock_exclusive`); both are pure Rust over `libc`/`windows-sys`,
   class 2 under §2.3, no Appendix F row. Three details decide whether it is reliable:
   **hold the lock for the whole session, not per append** — per-append locking lets a
   second process in *between* appends, which is exactly the `seq` collision; **use
   `try_lock`, never blocking `lock`** — a second engine should fail in milliseconds
   with a remedy, not hang behind a session that may run for hours; and **lock a
   dedicated `<session>/lock`, not `events.jsonl`** — locking the data file conflates
   "I own this session" with "I have this file open", and breaks the moment anything
   opens the log read-only to export it. Note Windows' `LockFileEx` is *mandatory*
   rather than advisory, which is stronger and fine, provided the lock file is opened
   with share-read.
2. **The NFS collision with B20's Q1 — dissolved rather than solved.** Q1 was about
   where a remote member gets the *repository*; nothing requires the *session
   directory* to be remote, and §12 already puts it at `~/.local/share/dacode/`. So
   the rule is **"the working tree may be remote; the log may not"** — which is the
   same separation as "where the code lives" versus "where the truth lives", and it
   removes `flock`'s version-dependence from the design instead of documenting it.
   Belt and braces: `statfs` the data directory at startup and warn on an NFS magic
   number, since the failure it prevents is silent.
3. **The index format.** A **fixed-width binary sidecar** — `(u64 seq, u64 offset)`
   per event, append-only like the log. Binary rather than text because there is no
   delimiter to mis-parse and a torn tail truncates to a valid prefix by arithmetic
   alone. Three invariants make it safe: **reconstruction and initial construction are
   the same code path** (scan the log, emit records), so there is nothing to drift;
   **write order is log line → fsync → index record**, never the reverse, because an
   index entry for a line that did not land is a hole while a missing entry for a line
   that did is merely a rebuild; and **on open, validate the last record against the
   log** and truncate to the last good one — the WAL-recovery pattern, and the reason
   "rebuildable" is load-bearing rather than decorative.
4. **What a second process sees.** Fail fast, name the holder, offer the remedy:
   *"session … is held by dacode (pid 1234, since 10:02) — `dacode attach <url>` to
   observe, or close it first."* The PID and start time are written **inside** the lock
   file after acquisition and are **informational only** — correctness rests on the
   `flock`, never on the contents, which is precisely the mistake PID-file locking
   makes. **No lock stealing or handover**: that is a consensus problem wearing a UX
   costume.
5. **Which command hits this.** Rarely `run -p`, which creates a new session; the real
   cases are `session fork` and resume. Both: error plus `attach`.
   ⚠️ **Superseded in its exit-code half 2026-10-04, by [U93](../TODO.md)'s ruling
   (arm (b))**: "error" here meant the fail-fast refusal with the holder named and
   `attach` offered — all of which stands — but under §11's mapping it also meant
   the generic exit `1`, indistinguishable from a crash, and that half is
   overturned: a held session exits **`4`** (design §11, one pre-session value,
   not a vocabulary). The sub-answer's *diagnosis* — which commands hit the lock —
   is what U93 built on and is unchanged.

**On sign-off:** a delta editing §3.6 (the single-writer invariant and the lock file),
§4.1 (the registry holds the lock; a second opener is an error, not a second writer),
§12 (`lock` and `index` beside `events.jsonl`), §8.1 (`attach` as the remedy) and §14
(two tests: a second process opening a session fails cleanly and says why; a killed
holder leaves no unusable lock). Deepen doc 05 with it — it is already expanded, so
this is an addition, not a first deepening. **Two interactions, both recorded where
they bite:** B20's constraint (d) is this rule remotely, and **B20's Q1 answer permits
a network filesystem, which is precisely where `flock` gets version-dependent** — so
either the session directory stays local, or NFSv4 is a stated requirement.

---

## Decisions resolved by this pass

Resolved entries stay in place above, struck through in the summary table with a
resolution banner in the detail section — this file is the audit trail (B4's
rule), so nothing is deleted.

| # | Decision | Resolution | Date | Applied by |
|---|---|---|---|---|
| **B1** | JSON strategy | Hybrid adopted: serde-derived control types + `RawJson` for opaque sub-JSON; §13's serde rule re-read as *third-party only* | 2026-09-12 | delta **A16** → design §13.1–§13.5 |
| **B6** | Tool schema gen | Offline codegen with `schemars`, confined to the generator; canonical byte-stable schema JSON `include_str!`'d | 2026-09-12 (delegated, with B1) | design **§13.3**; the [14](../part3-capabilities/14-tools.md) half deferred to M1 |
| **B4** | WS vs SSE | **WebSocket kept as the single live channel**; no SSE second transport in v1. Revisit only on a concrete deployment failure | 2026-09-12 | **no spec change** — §3.6 already specified WS, so this confirms it; the robustness half landed via delta **A18** → §3.7/§4.1/§14; [19](../part4-hosts-clients/19-server-gateway.md)'s marker flipped. Owed: decide the HTTP version deliberately |
| **B15** | Built-in task tracking & gating | **Fold-derived task tree** with item kinds, a `verify:` mandatory at creation and typed by checkability (`Command` the engine runs, `Manual` only a controller may close), a **hard-but-bounded** completion gate with `blocked`/`wontfix` exempt, append-only context rendering, and a promoted `.dacode/plans/` board as the cross-session tier. `Verify::Command` routed through §4.4 as an exec surface | 2026-09-13 | delta **A19** → new design **§4.8** + §3.2/§3.5/§4.2/§4.3/§5.4/§9.2/§10/§12/§14. Applied **self-contained** (A16 precedent) so it cites nothing A3/A4/B14 have yet created. New open decision **§16.6** on standing goals |
| **B16** | Repository tooling language | **Two classes, narrowed on sign-off.** Tooling the shipped harness depends on is Rust in `xtask`; repo-*authoring* tooling is any language and is **not awaiting a port**. Principle 5 explicitly scopes to the product and its build, not the authoring environment. All five current tools stay Python; `agents-sync.py` retires only when dacode dogfoods this repo (M2); `tasks.py` retires onto §4.8 once §16.6 is settled. **No YAML crate enters the Rust tree**, so the B10 coupling dissolved and B10 is free-standing | 2026-09-16 | delta **A20** → new design **§2.4**, plus §2.1 (xtask in the tree), §2.2 (nothing depends on `xtask`) and §14 (xtask tested as ordinary Rust; authoring tooling explicitly outside). ~~Doc 01's disposition table still owed at M0~~ → **the register §2.4 requires is [Appendix G](G-authoring-tools.md)** (U80, 2026-09-16); doc 01 points at it |
| **B19** | Identifier privacy | **Options 2 + 4** — prevention on the surfaces that retain (env injection under a reserved `DACODE_RT_` prefix excluded from §5.1's config layer, an opaque scratch handle instead of §12's id-bearing path, an exact `args` check over the **four ULID-backed ids** only), and narrow egress redaction of **secrets** on §8.2's config + four *projection* endpoints and §7.1's hook payloads, closing U61's second half. The log and the transcript get prevention because neither may be rewritten | 2026-09-16 | delta **A23** → design §4.3/§4.4/§5.1/§7.1/§8.2/§12. Left open: the width of the injected set, now a *config* question. Debt: §4.4's `yolo` carve-out is **B7's to confirm** |
| **B21** | Session-log concurrency & indexing | **One writer, one lock, an index that is a projection.** Advisory `flock` on a dedicated `<session>/lock`, held for the whole session, acquired non-blocking, owned by the *kernel* so there is no stale-lock recovery and no force flag; every other process is a client over §3.7. The log stays append-only JSONL and stays the truth; `index.bin` is a rebuildable sidecar. **Rewrite is not an operation.** All five sub-answers taken as recorded | 2026-09-17 | delta **A24** → design §3.6/§4.1/§8.1/§12/§14 + doc 05 **§5.5**. **Discharged U67** (the cursor is store-assigned, not line+1). Coupling to B20's constraint (d) recorded, not resolved |
| **B3** | Delta vs full updates | **Pure deltas** — live tier deltas, full value at the durable `*Ended` boundary (`TextEnded`/`ThinkingEnded`), so replay is lossless; a consequence of accepting A1's two-tier model rather than an independent choice | 2026-09-17 | ✅ **applied 2026-09-18** via delta **A1** → design §3.2/§3.7, which landed **before any golden fixture** as the sequencing rule required; **and doc 03's note landed 2026-09-19 at M1.1a** (§3.4: B3 is subsumed by the tier split, the oh-my-pi full-message line struck) |
| **B5** | In-band tool dialects | **The ~700 LoC v1 subset**: Harmony control-token escaping on the transport copy, a `supports_tools: false` capability flag driving history demotion, one XML grammar with stream holdback + a tool-name allowlist before promotion. The other ten dialects and per-request history re-encoding deferred; malformed tool-arg JSON repair stays a doc-14 axis, and repair never scrubs inside fenced/indented code | 2026-09-17 | ✅ **applied 2026-09-18** via delta **A5** → design §4.3/§4.4/§6/§14, four sections rather than the one named. The subset is normative, with Harmony escaping scoped to the **transport copy only** and the reason stated (§13.4's byte-exactness plus §3.6's append-only log mean escaping the stored copy would rewrite history to match a transport quirk). **`ToolDialect { Native, XmlGrammar }` landed as a reserved slot** — derived from `supports_tools` in v1, since only one grammar ships, so its value is that un-deferring the matrix adds variants rather than restructuring the engine. `CompletionRequest`/`StreamChunk` defined (§6's trait referenced both without either existing). ⚠️ **§16 gained no B5 row, deliberately**: it lists *open* decisions, B5 is resolved, and A19 set that precedent — the deferral lives in §6's prose where a reader meets it. **U10 stays open** (JSON repair is a tools axis) and the never-scrub-inside-code carve-out became normative rather than staying a follow-up |
| **B7** | `bash` sandboxing & honesty | **Deny-list + permission engine in v1, no OS sandbox**, documented as *advisory* over §4.3/§4.4/§5.2; interpreters disqualifying; the tight critical-pattern list. **Scoping amendment (the user's, recorded as a decision): the `SandboxBackend` seam ships at M1 as an inert stub** — no Landlock/seccomp code — and §4.7's worktree is the only real containment until it is filled, so **B20's remote-sandbox rule inherits "fill the seam first"** (or violates §1.5 via containers). The 2026-09-16 polarity stands: sandbox immutable at creation, `yolo` the mutable opt-out, colour indicator with U89 open | 2026-09-17 | **approved, not yet applied** — delta **A12**'s §16.3 half; docs 14 + 16 updated; `bash:readonly` wording in §4.3/§5.2; **A23's `yolo` carve-out in §4.4 confirmed-or-overturned** |
| **B14** | Prompt/context buffer | **Appender + segment index** (`Segment { kind, offset, len, digest }`): cutting later parts is O(1) truncation, re-parsing never required, and a model/provider switch replays the segment list through the new encoder. Settles §5.6's environment-block placement — after the cache-stable portion, excluded from the fingerprint | 2026-09-17 | ✅ **applied 2026-09-18** inside delta **A4** → `Segment`/`ContextBuffer` normative in design §4.5 as the buffer representation, with all four argued properties stated as spec text. Both consequences this entry predicted proved load-bearing (§4.5's longest-stable-prefix resync, §5.6's environment block excluded from the fingerprint) and A4 added a third (§4.8 cites the index for why a status flip appends one segment), which also discharged the second half of **A19**'s cross-reference debt. **Docs 09 and 17 still owe the plan-side reference**, blocked by hard rule 1 rather than by neglect |
| **B17** | The built-in `reviewer` | **Option (c), the split**: `reviewer` stays an ordinary agent file; the engine gains reason-tagged changeset selection, `finding` as a durable structured event with no line numbers, and snippet-anchoring positioning. No LLM grouping in v1; reflection filter defaulted off; checklists are skills. **Amendment as a requirement on the future delta**: usage defined by the system prompt **and at least one pre-defined workflow** (§5.4) — shipped content, not per-user reinvention. **Still brainstorm-gated twice over**; `finding` lands after A1 | 2026-09-17 | **signed as a direction — the delta is deliberately not written yet** (all three sections it would edit sit inside the built-in-tools/M1 and built-in-agents/M2 brainstorm gates); owed after the tools brainstorm, plus §5.2's advisory-`bash` description |
| **B18** | Harness benchmarking | **Four tiers, three adopted**: T1 deterministic replay benches in CI from M1 over a committed `events.jsonl` corpus (**produces B12's missing number**), T2 token/cost led by prompt-cache hit rate (needs A4's `Usage` cache fields), T3 task-success runner at M5 and never in CI, T4 cross-harness comparison an explicit non-goal; plus the publication rule — no number without an in-tree re-runnable scorer and a recorded environment. The med fork (T3 at M5, earlier if regressions bite in M2) taken as recommended | 2026-09-17 | **approved, not yet applied** — delta **A21** → §14 split; doc 33 deepened when M1 starts; the T1 harness added to M1's task list; **B12 re-run as a measurement** |
| **B13** | Session organization | **Tags + virtual folders + projects confirmed as fold events** with **Fix 1** (durable log events, never in `Context`; mapping tables a rebuildable projection) and **Fix 2** (worktree→main-repo derivation via `git rev-parse --git-common-dir`). **Amendment reversing part of the recommendation: virtual folders are a first-class requirement**, not the deferrable UI-heavy half — a many-step feature spans sessions and a folder holds them; every surveyed harness offers a flat list at best. Same shape A3's hand-over/resume serves; B8 puts the worktree in §5.4 instructions | 2026-09-17 | ✅ **applied 2026-09-18** via delta **A14** → design §3.1/§3.2/§3.3/§3.5/§3.7/§4.3/§4.7/§8.4/§12/§14 (ten sections, not the two the delta named) + both fixes and the **names carve-out** in doc 06. The amendment is what grew it: a first-class folder spanning sessions must be creatable *before* its sessions, which pure membership cannot express, so the registry (`org.json`), its `Org` command and §8.4's rule for a session-less resource are consequences of the amendment. **U16 and U17 closed**; **U99 opened** (A7 owns registry freshness) |
| **B8** | Checkpoint strategy | **Shadow git kept** with all six additions §4.6 omits (`.gitignore` filtering, 2 MiB untracked cap with skips reported, absent⇒delete, **selective per-file restore**, path-escape rejection, boundary turn id in the `Checkpoint` event), **gitoxide in-process**, and both unsolved problems spec'd (retention policy; the shadow-store race as a stated trade). **Worktree question answered**: feature-level workdirs go through **§5.4 workflow instructions**, not a session↔worktree binding. Residual owed to doc 12: a shared worktree needs a **one-writer rule** — B21's lock covers the log, not the tree | 2026-09-17 | **approved, not yet applied** — delta **A12**'s §16.5 half + the §4.6 amendments; doc 12 gains the one-writer-rule residual |
| **B10** | Config on disk | **TOML-on-disk / JSON-on-wire confirmed** with both consequences: `dacode config edit` preserves comments and formatting (`toml_edit`) — which matters more for **A7**'s web config-write endpoints — and a generated **`dacode.schema.json`** reusing B6's machinery, pairing with B2's export. The B16↔B10 coupling stays dissolved: no YAML crate enters the Rust tree | 2026-09-17 | ⚠️ **half-applied 2026-09-19 via A7** — comment/format-preserving writes are **normative at §5.1** (and §12 was already citing "(§5.1)" for a requirement §5.1 did not carry). **Owed:** delta **A12**'s §16.2 half; the generated `dacode.schema.json`, which must **not** advertise the exec-bearing keys §8.5 refuses; both items added to doc 17 |
| **B2** | Web stack | **The split**: `dacode-protocol` ships a **JSON-Schema export** as a non-optional **M1** item regardless of framework; the framework decision gets a **deadline of M3** with **TS SPA (Solid)** standing, mitigated by generating `protocol.d.ts` in CI and failing on drift. Makes honest that TODO §5's M1 list already carried the export while doc 03 called it optional | 2026-09-17 | ✅ **applied 2026-09-18** via delta **A8** → design §9.1 records the split with the framework **open but deadline-bound to M3** and TS SPA (Solid) as the *standing recommendation*; doc 03's optionality removed as a false claim under hard rule 1, marker untouched. §9.1 also states *why* the old conditionality was backwards (the export is what makes the TS branch cheap) and resolves a contradiction this entry could not have seen — the old `build.rs`-in-`dacode-protocol` clashed with §13.3's "never in a shipped crate" and §2.4's `xtask` placement, so the generator is `xtask`'s and `protocol.d.ts` is generated in CI with a drift check that fails the build. **Still owed: the framework decision itself, by M3** |
| **B11** | New crates | **`dacode-store` deferred** — traits in `dacode-core::store`, crate extracted at M3, because a boundary with one implementation is indirection without payoff and extraction later is mechanical; **`dacode-introspect` approved in principle, created at M5**, with its **exchange-record format an M1 deliverable** in doc 13 (retrofitting in M5 costs far more; it gives doc 09 the harness that validates A4's cache invariants). In-tree vendoring stands. The med-confidence dissent (separate `dacode-store` now) was available and not taken | 2026-09-17 | ✅ **applied 2026-09-18** via delta **A13** → design §2.1/§2.2 record both crates as approved-with-dates with **neither added to the eleven-member tree**, and the in-tree vendoring settlement made explicit rather than implied. **Appendix D's *legend* was what needed fixing**, not its graph — the graph already carried both crates, while the legend still called them "proposed" and named no milestone. **The record-format item was already normative** in §6 via A10's carve-out the same day, so §2.1 cites it as done. ⚠️ A13 also recorded a hazard this entry could not have anticipated: `gates/boundary-edges.txt` is an *exact* diff against `cargo tree`, so the deferred crates' edges must **not** be pinned until their directories exist — adding one breaks the gate while looking like diligence. §2.2's "enforced by review; consider a workspace-lint" was corrected, since that gate already exists |
| **B9** | ACP commitment | **Deferred to M5** (`dacode acp` — fs-free, terminal-free, `mcpServers`-rejecting subset, 2 500–3 500 LoC, framing shared with the M4 extension host), with the **two M1 constraints** committed: everything derivable from the log via fold + `Subscribe { since }`, and no tool branches on client capabilities or locality. The user's "be aware" gets a mechanism: the constraints plus doc 14's review rule, checked when doc 14 is written. Open input kept: verify a maintained Rust ACP crate before M5 planning | 2026-09-17 | **approved, not yet applied** — delta **A12**'s §16.4 half; doc 26 moved to M5; the two constraints added to M1's docs |
| **B12** | Session compression codec | ✅ **`zstd`, accepted as a class-1 exception** — resolved by decision, not by measurement: *"We can allow as exception to use zstd with C library."* The reopening closes because the case it surfaced is decisive: `ruzstd` is **decode-only**, so every pure-Rust option was a *different codec*, not a different build of this one, and the log's write path needs a real encoder. Dictionaries stay deferred behind A15's tag | 2026-09-17 | [Appendix F](F-c-dependencies.md) §F.3's row moved *avoidable* → **accepted**; `gates/class1-allowlist.txt` gains `zstd-sys` in the commit that adds the dependency. ~~**A15's hold fully lifted** (transcribe the discriminator codec-agnostically anyway, per U25)~~ ✅ **A15 applied 2026-09-18**, to seven sections rather than its own two — and it generalized *further* than U25 proposed: `plain | compressed` with the codec in **`LogHeader`**, because config-only naming would orphan every record already written, and with `flat` **removed** from a per-field enum since it is a record *representation*, not a field encoding. **U25 closes**, **U26 closed**, and **U14 stays open** as the condition on the exception — A15 does not discharge it |
| **B20** | Remote agent execution & swarms | **Option 2 — reuse the protocol, invent no second one; constraints now, build at M5.** Model access by **credential broker** at the controller (the member forwards through it, the key never leaves, **verbatim**); a dead member **blocks and asks**, retry only as a per-step opt-in that declares its effect class; a **§5.4 DAG over one shared §4.8 task tree** with a separate remote cap; **five** wire-format constraints, the fifth (a **fencing token**: epoch + idempotency key + leaf CAS) added by the survey because B21's `flock` cannot fence a remote writer | 2026-09-17 | the [sixth survey](../../../.agents/knowledge/reference-harnesses/remote-execution-and-swarms.md) + delta **A25** (new): constraints (a) and (e) **with A1 at M1**, the rest at M5 across §5.3/§5.4/§8.4/§4.8/§4.7 + doc 20. **Constraint (b) no longer arrives via A22** — that item was redirected to permissive for v1, so the remote-principal split is A25's to argue |
| **B1a** | `dacode_json::RawJson` | Implemented, benchmarked, dual-reviewed | — | `dacode-json` `90def19` |

**Every one of the 21 decisions is resolved — none is open and none awaits a vote.** ~~1 of 21~~ ~~2 of 21~~ ~~14 of 21~~ —
**fourteen** were signed off
2026-09-17 in a single pass (B2, B3, B5, B7, B8, B9, B10, B11, B13, B14, B17,
B18, **B12** — resolved by *decision* rather than by measurement: `zstd`, accepted as
this policy's first class-1 exception — and **B20**, whose survey ran and whose session
closed the same day), every one of them **approved and owed rather than applied**: the deltas they
name (A1, A4, A5, A8, A12, A13, A14, A15, A21 — B17's is deliberately not written
yet) ~~have not touched `docs/dacode-design.md`~~ **had not touched
`docs/dacode-design.md` when this was written; A1 and A14 both landed 2026-09-18**,
so the remainder is A4, A5, A8, A12, A13, A15 and A21. ~~The two that remain are precisely
the two that were never votes — **B12** needs a measurement (§12 below) and **B20**
needs a survey and a design session — and **B18's sign-off is what finally lets B12
be answered**: its T1 replay benches produce the codec number.~~ **The one that
remains was never a vote: B20 needs a survey and a design session.**~~ ✅ **B20 got
both on 2026-09-17 and is resolved**, so what is left in this appendix is *application*,
not decision. B18's T1 benches still run against the codec, but they now tune the
*level* and confirm the ratio rather than choose it. B1a was resolved
earlier and is recorded in §1 above.
