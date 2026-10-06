<!-- docs/implementation/appendices/A-design-deltas.md -->
# Appendix A — Design-doc deltas required

> Index: [README](../README.md) · Spec: [`dacode-design.md`](../../dacode-design.md)

Changes to `docs/dacode-design.md` implied by this plan. **The design doc moves
only by your explicit decision** — these are proposals, collected for one review
pass.

**This appendix is now the delta review.** The table below is the original index
(kept as the audit trail); each entry is expanded underneath with the current
spec text, the concrete proposed edit, its B-decision dependency, and a
recommendation. Spec line numbers are from `dacode-design.md` at commit
`456a118` and will drift once edits land.

> **⚠️ For whoever applies the next delta: `docs/dacode-design.md` contains zero
> markdown links, and must stay that way.** The plan cites the spec; the spec
> never cites the plan. The proposed edits below are written as *plan* text, so
> several of them contain links to appendices and feature docs (A17's did) —
> **strip those when transcribing**, and name the target in prose instead. A17
> landed as "a debt register maintained alongside the implementation plan"
> rather than a link to Appendix F, with the back-reference carried in Appendix
> F's own header. Check with `tools/check-links.py --conventions-only`, which
> **fails the build on any link in the spec** and is now the authority on this —
> prefer it over the raw `grep -c '](' docs/dacode-design.md` count it replaced,
> because grep cannot tell a link from a backticked link *shape*, and describing
> one is sometimes necessary.

> **Three states, not two — and sign-off is not application.** An entry here is
> *proposed* (awaiting a vote), ✅ **SIGNED OFF** (approved for application, **not
> yet transcribed into `docs/dacode-design.md`**), or ✅ **APPLIED** (transcribed,
> with the sections it became normative at named in its banner). A signed-off delta
> still owes its transcription, its golden fixtures and every amendment recorded in
> its banner — so §4.5 had no caching text while A4 read **SIGNED OFF**, §3.2
> had no tier marker while A1 did (*that* one is now **APPLIED**, 2026-09-18, and
> §3.2 carries `EventTier`), and §3.5 had no `SessionInfo` struct — only a comment
> listing its fields — while A14 read **SIGNED OFF** (*also* now **APPLIED**, the
> same day, and §3.5 carries the struct). Both settled states strike the index id, because
> `tools/attention.py` reads a struck id as resolved mechanically; the banner, not
> the strike, says which of the two you are looking at.

<!-- attention-meta — machine-readable; read by tools/attention.py.
     One `gates:` line per delta whose milestone cannot be derived from the
     decisions blocking it. Absence means "inherit from the still-open
     B-decisions named in the Depends-on cell", which is already correct for
     most rows, so only the exceptions are declared.

     Declared rather than detected, for the reason Appendix B's `action:` block
     gives: a property of a delta cannot be scraped from prose about it. The
     inherited rule had a hole — a delta that depends on nothing inherited
     nothing and silently fell to the lowest priority, so A2/A3/A4 rendered as
     "no open decision dependency" while the progress board named all three as
     M1 blockers. The inherited rule was also wrong in the other direction for
     three rows: A7, A10 and A15 took the milestone of whatever decision blocks
     them, which is when a *prerequisite* is needed, not when the delta's own
     subject matter is.

      Rule the declarations follow: **a delta is owed by the milestone of the
      plan docs covering the spec sections it edits.** A15 edits the storage
      sections, so it is owed by M1 even though the codec decision blocking it is
      an M3 concern; A10 edits a new introspection section whose crate is M5 but
      whose record format its own text calls an M1 deliverable; A14 adds durable
      events to the protocol sections, so it is owed by M1 with the protocol crate
      even though the organization feature it serves is M2.

      **The rule is a heuristic, and one row deliberately opts out of it.** A13
      edits the crate-tree section, which doc 01 covers at M0 — but its subject
      matter is two crates that B11 defers to M3 and M5, so declaring M0 would
      raise the priority of work that cannot usefully happen yet. A13 is left
      undeclared and inherits M3 from B11. Where the rule and the substance
      disagree, the substance wins and the disagreement is recorded here rather
      than silently encoded.

       Validated by attention.py's hygiene block: an id absent from the index
       below, or a value outside M0-M6, is reported rather than silently ignored —
       and so is any open delta left with no milestone at all, which is the
       regression this block exists to prevent.

       **A declaration is removed when its delta is signed off**, because a struck
       row leaves the open table the tool renders and a milestone for something no
       longer listed is a dangling id the hygiene block rightly rejects. Nine lines
       (A2, A3, A4, A6, A7, A9, A10, A14, A15) went on 2026-09-17 for that reason.
        That is a real loss of visibility and it is recorded rather than papered over:
         **the tool has no state for "approved but not yet applied"**, so the deltas
         whose transcription is still owed disappeared from the prioritized view —
         seventeen of them when this was written, five since A1, A14, A15, A2, A3, A4, A5,
         A9, A21, A22, A8 and A13 were applied on 2026-09-18, and **four since A7
         followed on 2026-09-19** (A6, A10's §17 body, A12, A25's M5 half). Until
       it learns the third state, [TODO](../TODO.md) §3's table is the application
       queue and is the only place that carries it (U95). A22's line went the same way
       an hour after it was declared, when A22 was signed.
-->

## Index

| # | Section | Delta | Depends on | Recommend |
|---|---|---|---|---|
| ~~A1~~ | §3.1, §3.2, §3.5, §3.6, §3.7, §4.3, §4.8, §12, §13, §14 | **Two-tier event model** (durable vs live-only), ~~+ seq-gap → resync~~ (A18's, discharged), **+ A25's M1 wire half** and **+ §16 item 6's `Standing`** | ~~B3~~ ✅ signed, ~~B4~~ ✅ signed | ✅ **APPLIED 2026-09-18** — `Event::tier()` returning an `EventTier`, the tier stated as a **rule** (`Durable` iff the fold consumes it), `seq: Option<Cursor>`, `TextEnded`/`ThinkingEnded`, `WriterEpoch`, `HostId`/`SessionRef`, `LogHeader` (U66 folded in), `TaskStatus::Standing`. Landed **before any golden fixture**, as required. Six scoping decisions in the banner, four of them the user's |
| ~~A23~~ | §4.3, §4.4, §5.1, §7.1, §8.2, §12 | **Identifier privacy** — env injection under a reserved prefix, an exact `args` check over the four ULID-backed ids, an opaque scratch handle, and redaction on the egress paths that serve copies | ~~B19~~ ✅ signed off | ✅ **APPLIED 2026-09-16** — normative at §4.3/§4.4/§5.1/§7.1/§8.2/§12. No `Event` variant, so no fixture and no A1 dependency |
| ~~A24~~ | §3.6, §4.1, §8.1, §12, §14 | **Session-log concurrency** — single-writer as an invariant enforced by an advisory lock the *kernel* owns, the cursor restated as store-assigned (discharges U67), and a `cursor → byte offset` sidecar that is a rebuildable projection, never the truth | ~~B21~~ ✅ signed off | ✅ **APPLIED 2026-09-17** — normative at §3.6/§4.1/§8.1/§12/§14, with the implementation obligations in doc 05 §5.5. No `Event`/`Command` variant, so no fixture |
| ~~A25~~ | §3.1, §3.6, §3.7, §4.7, §4.8, §5.3, §5.4, §8.4 | **Remote-execution constraints** — host-nameable session ids, a **fencing token** on the session writer, remote-principal capabilities, the executing host in `verify` evidence, sandbox backends as **declared host capabilities**, and the §5.4 **DAG** (step `host`/`needs`/`parallel`, one shared task tree, a separate remote cap) | ~~B20~~ ✅ resolved | ✅ **SIGNED OFF 2026-09-17 with B20**, **split by milestone** — and the **M1 half is now APPLIED 2026-09-18, with A1**: constraints (a) and (e) are in §3.1/§3.2/§3.6/§3.7 as `HostId`/`SessionRef` and `WriterEpoch` + `Snapshot.epoch`. The **M5 behaviour half is not yet in the spec** (§5.3 location, §5.4 DAG, §8.4 remote principals, §4.7 host capabilities, §4.8's executing host), so this row stays open |
| ~~A2~~ | §4.2, §4.3, §13.2, §14 | Replace `parallel_safe() -> bool` with a **concurrency policy** | — | ✅ **APPLIED 2026-09-18** — the amendment landed as signed (`Shared` / `Exclusive` / `Keyed(ConcurrencyKey)`, the tool computing its own key) **and its documentation obligation is discharged**: §4.3 now says how an author chooses, why the key must be canonical, and that the default is deliberately pessimistic. **No wire variant added**, so the audit-trail benefit is stated as a property of the key being a value rather than built as a field on `PermissionResolved`. **Per-tool policy assignments are marked pending**, not invented — §1's deferral names "which tools ship" as brainstorm-owned content. **U27 closes on both halves**: §13.2's bare-`Json` alias note is dropped, having no site left. `.agents/roles/_rust.md` + the `dacode` skill + four generated adapters corrected; **doc 08's `Dynamic(fn)` arm struck**, since it specified the design the amendment rejects |
| ~~A3~~ | §3.2, §3.6, §4.2, §4.8, §11, §14 | Loop **guards**, streaming/execution overlap, **synthetic tool results**, steering vs interrupt | — (rode in on ~~A1~~ ✅ for the marker; had to reconcile with ~~A2~~ ✅ for the site) | ✅ **APPLIED 2026-09-18** — six sections against the one named, because the guard set needed types §3.2 never defined (**`StopReason` and `IdleReason` were used but never declared**, and §4.2 already called a two-argument `AgentIdle` the one-field variant could not hold), a commit boundary §3.6 never listed, and an exit-code mapping §11 never stated. **`AgentIdle` gained a typed `guard: Option<GuardKind>`** — a wire change, additive because `Option` is omitted when `None` — chosen over growing `IdleReason` (which would break §11's four codes and §9.2's five chips) and over an untyped detail string (which would violate principle 3). **The guard *numbers* are marked pending, not invented** — §1's deferral names them as brainstorm-owned. **A19's debt discharged**: §4.8's gate is now `GuardKind::CompletionGate`, one of the guards rather than a mechanism beside them. **U96's hand-over stays open** — `GuardKind::HandOver` is a slot, not a mechanism |
| ~~A4~~ | §3.2, §4.5, §4.8, §5.6, §6, §14 | New **prompt-caching** subsection; expand **compaction**; cache fields on `Usage` | — (~~apply together with B6~~ ✅ **already normative since A16, 2026-09-12**; ~~**B14**~~ ✅ **applied with this**) | ✅ **APPLIED 2026-09-18** — caching half as proposed; **both compaction amendments as amended**, so automatic compaction is off by default and §4.5 no longer implies it ships at M1. **B6's byte-stable schema was already normative in §13.3 five days before this was signed**, so A4 cites it rather than duplicating it. **Three items composed into a hole none of them stated**: amendment (a) defers compaction, A3 landed the hand-over as a *slot*, and U68's prescribed fix was "compact and re-issue" — so v1 had no automatic response to a filling context. Resolution: overflow is a **retryable provider error with two branches**, and v1's honest default is *stop and tell the user with a remedy*. **`Usage` was used by two events and never defined** — now a real struct with four counters, which is what closes **U71** (a cached read priced at the input rate is a wrong cost, not an imprecise one). **U68 closed, U97 stays open** (partial compaction not adopted before §14.2's number). **U96 is now load-bearing**: two normative sections cite the undesigned hand-over |
| ~~A5~~ | §4.3, §4.4, §6, §14 | ~~Native **`gemini`~~ **dropped from v1**; capabilities as **data**; **in-band dialect subset**; SSRF/egress `net-policy` | ~~B5~~ ✅ signed, ~~A4~~ ✅ **had already built most of it**, ~~A3~~ ✅ for `StopReason` | ✅ **APPLIED 2026-09-18** — **redirected, and the redirect is what landed**: no native `gemini`, and §6 says why the capability model is the stronger position — a provider needing something unusual is a **value on `ModelInfo`**, not a `match` in §4.2. **A4 had already put three of the four capabilities on `ModelInfo`** (its four-rate cost rule needed the struct), so A5 *extends* a model rather than adding one; the only new capability is `ToolDialect`, landed as a **reserved slot** because B5's grammar is reachable only when `supports_tools` is false and v1 ships exactly one. **B5's instruction to add a numbered §16 decision was NOT followed** — §16 lists *open* decisions and B5 is resolved, per A19's precedent; the deferral lives in §6's prose instead. **`CompletionRequest`/`StreamChunk` were referenced and never defined**, and are now real types. Egress policy needed §4.4 (an IP layer *under* the hostname globs, since a glob cannot close SSRF) and §4.3 (`fetch` as the second of three egress paths). **U10 stays open** — JSON repair is a different axis and §6 now records the exclusion |
| ~~A6~~ | §7 | **Channel-plugin model** (plugin owns grammar/security; core owns the `message` tool) | — | ✅ **SIGNED OFF 2026-09-17** — approved as proposed. **Not yet in the spec**; M4 scope unchanged |
| ~~A7~~ | §8.2, §8.3, §8.4, §8.5, §3.6, §3.7, §5.1, §9.2, §12, §14 | **Config-write endpoints**; **multi-user/tenant** auth + pluggable **storage backends** | ~~B10~~ ✅ signed, ~~B11~~ ✅ signed | ✅ **APPLIED 2026-09-19** — ten sections against the two this entry named, because both caveats and the post-A19 amendment each needed a section the entry did not list, and **U99's** freshness channel needed a wire frame. **The scaling axis is normative at §8.2** (both trait families, `BlobStore` reconciled in as the third, `JsonlStore`+`SharedToken` the only M1 implementations), which is what makes §2.2's and §3.7's existing citations of "§8.2's scaling axis" resolve to content rather than to a heading. ⚠️ **Its own gate — "gated on controller status (§8.4)" — was NOT transcribed, because it is incoherent**: §5.1's four layers include no session-scoped one and §8.4's controller rule reaches only commands that mutate a session, so A14's established pattern was applied instead and §8.4 now states **one rule for every non-session write** (registry, config, board), discharging A22's assignment to answer all three at once. **Two rules this entry does not contain were added because the surface needs them**: the writable root is **derived server-side and never supplied** (one server hosts sessions in several repos, so a client-named root would let a browser in repo A write repo B's config), and **a write may not add an exec surface** — `[[extensions]]`' `cmd` and a hook's `run`/`post` are refused although the file holding them is inside the allowlist, since §9.2 renders model-authored markdown on the authenticated origin and one transcript could otherwise become persistent command execution. Plus `ETag`/`If-Match` with `412`, because four writers can reach one file. **B10's comment-preserving writes landed at §5.1**, where §12 already cited them and no section carried them. **`Authenticator`/`Tenant` placed in `dacode-server`, no new crate.** **U99 closed** (`org_version` + §3.6's `Hint` frame — a compatibility event, additive); **one gap recorded rather than filled**: §3.3 has no task-mutation `Command`, so the browser path to `promote` does not exist in the protocol |
| ~~A8~~ | §9.1 | Resolve the **web stack** decision — as a *deferral with a committed prerequisite* | ~~B2~~ ✅ signed | ✅ **APPLIED 2026-09-18** — B2's split transcribed and **the framework deliberately NOT picked**: §9.1 now says it is an open decision **due by M3** with **TS SPA (Solid)** as the *standing recommendation* and both reasons recorded, rather than "Decision pending". **The JSON-Schema export became unconditional**, and §9.1 states why the old conditionality was backwards — the export is what makes the TS branch *cheap*, so requiring the branch first would have made the framework decision harder rather than easier. ⚠️ **A live contradiction with §13.3/§2.4 was resolved**: the old text put a `build.rs` doing the export *inside `dacode-protocol`*, while §13.3 confines `schemars` to an offline generator ("never in a shipped crate") and §2.4 puts build-time codegen in `xtask` — so the generator is `xtask`'s, the protocol crate grows no `build.rs`, and `protocol.d.ts` is generated **in CI with a drift check that fails the build**. **Doc 03's "Optional … if the web UI is a TS SPA" was a false claim, not a stale marker**, and was corrected with its marker untouched |
| ~~A9~~ | §9.2, §10 | **One tree component**; **TUI mouse**; **collapse-on-paste** | — (~~A3/A4 must not land later~~ ✅ **both landed first**) | ✅ **APPLIED 2026-09-18 — and with it M1's delta *list* is COMPLETE** (`A1–A5, A9, A14, A15` all landed). ⚠️ **Not the milestone, though**: B11's condition made A10's exchange-record format an M1 deliverable assigned to A5's §6 side, and A5 landed without it — found by an inbound-assignment sweep, now landed, so **A10 is half-applied**. Taken **out of queue order**: A6 is next in the application order but is §5's **M4** dependency, while A9 was M1's last — the queue and the milestone lists are different axes. **§9.2's half was already normative via A19**, so A9 extends it and contributes §10's *matching* component plus the two TUI rules, with each rule's **reason** stated (a TUI that always captures the mouse costs the user their terminal's scroll-back and selection; a big bracketed paste can be misread as keystrokes) so neither reads as a convenience. Both are recorded as **§10 rules, not shared client behaviour**, so §9.3 correctly grows neither. ⚠️ **One gap recorded rather than filled**: nothing on the wire correlates a workflow *step* to its spawned child, and the correlator needs a wire change plus a `TaskId` (undefined — **U100**) plus a §4.3 scan clause, for a shape that depends on §5.4's step model — **A25's M5 half, undesigned**. **U101 opened**: §14 has no UI test tier, so both rules are normative with nothing that can fail on them |
| ~~A10~~ | new §17, **§6 + §12 + §14 applied** | **API introspection & caching analyzer** as a built-in subsystem | ~~B11~~ ✅ signed | ⚠️ **HALF-APPLIED 2026-09-18, in A25's shape — and the half that landed is the one that was nearly missed.** Approved as a new **§17**; the crate is M5 but **the exchange-record format is an M1 deliverable**, because the analyzer is what validates A4's cache invariants. **This entry assigned that §6 side to A5 twice, and A5 landed without it** — B11's signed condition ("if the provider layer does not emit it from the start, retrofitting instrumentation across every provider in M5 is far more expensive") was therefore unmet while the board said M1's delta list was complete. Found by an inbound-assignment sweep, not by any gate. Now landed: `ExchangeRecord` in §6 with fields mapped onto B11's signed list, `exchanges.jsonl` in §12 with the redaction asymmetry stated against `events.jsonl`, and a §14 test. **The §17 body — the logging proxy, SSE reassembly, the cache-failure taxonomy — is still M5 and still owed** |
| ~~A11~~ | §13 | Reconcile the **`dacode-json` reality** with the serde-isolation rule | B1 | ✅ **APPLIED 2026-09-12 via A16** — superseded, as recommended |
| ~~A12~~ | §16 | Resolve §16.1–§16.5; add **native GUI nodes** and **ACP** | ~~B2~~ ✅ signed, ~~B7~~ ✅ signed, ~~B8~~ ✅ signed, ~~B9~~ ✅ signed, ~~B10~~ ✅ signed | ✅ **SIGNED OFF 2026-09-17** — approved as proposed and **applies last**; its scope grew to carry **§16 item 6** as a `Standing` status, **goals only** (a `TaskStatus` wire change, so it settles with A1). **Not yet in the spec** |
| ~~A13~~ | §2.1, §2.2 | Add proposed crates **`dacode-store`** and **`dacode-introspect`** | ~~B11~~ ✅ signed, ~~A10~~ ✅ **its record format landed first** | ✅ **APPLIED 2026-09-18** — both recorded as **approved-with-dates** (M3, M5) and **neither added to §2.1's tree**, which still lists eleven members exactly as this row required. ⚠️ **A phantom-edge hazard was found and written into the spec**: `gates/boundary-edges.txt` is an *exact* diff against `cargo tree`, so §2.2's new rules name edges for crates that do not exist and say in the same breath that those edges **must not** be pinned until the directory does — adding one breaks the gate while looking like diligence. The file was verified unchanged at 29 lines. **Appendix D's instruction was already satisfied** (its graph carried both crates) but its *legend* was stale — it called them "proposed" with no milestone — so that is what changed. **§2.2's "enforced by review; consider a workspace-lint" was corrected**: the pinned exact-diff gate already exists, so the spec was calling its own enforcement mechanism aspirational. The fifth delta in a row whose named work was partly pre-done |
| ~~A14~~ | §3.1, §3.2, §3.3, §3.5, §3.7, §4.3, §4.7, §8.4, §12, §14 | **Session organization**: the three membership events folded into `SessionInfo`, the `Org` registry command, `ProjectId`/`FolderId`, and B13's two fixes | ~~B13~~ ✅ signed, ~~A1~~ ✅ applied first | ✅ **APPLIED 2026-09-18** — with **four scoping decisions, all the user's**: the registry is authoritative for **names** (`org.json`, the one non-projection in §12) while membership folds, so v1 accepts that a rename reaches only its issuer and §3.7's never-silently-stale promise is **scoped to a session**; registry mutation is a **`Command`** because §8.1's default TUI has no socket, and §8.4's controller rule does not reach it; **one op-carrying `Org`** rather than seven variants, with caller-allocated ids since `Ack` has no payload slot. Fix-2 landed normatively in **§4.7**, not only in doc 06. **A23's prediction that A14 would add `ProjectId` to the scan did not fire** and is annotated. **U16, U17 closed**; **U99 opened** (A7 owns registry freshness) |
| ~~A15~~ | §2.3, §3.6, §12, §13.1, §13.2, §13.5, §14 | **Selective compression** via a per-field codec discriminator, the codec named per *file* | ~~B12~~ ✅ resolved, ~~A1~~ ✅ applied first | ✅ **APPLIED 2026-09-18** — the **mechanism landed exactly as signed** (discriminator, allowlist, ~1 KiB floor, level, one `compress` knob, the searchability trade) but **three enumerations did not survive deltas that landed after it was drafted**: the codec moved to `LogHeader` (`plain | compressed`), since a config-only codec would orphan every already-written record; **`flat` was removed** from a per-field enum because it is a *record representation*, not a field encoding; and **`ToolOutputDelta`/`ThinkingDelta` are `Live`** under A1's rule, so they are never stored and were restated to their durable carriers (`ToolResult.output`, `ThinkingEnded.text`). **§13.2's `flat`-is-the-log row was corrected** — B21 signed JSONL five days after A16 wrote it. **§2.3** and **§13.5** no longer call the codec open. **U25 closed; U14 stays open** and is not discharged by this |
| ~~A16~~ | §13 | Adopt the **`dacode-json` hybrid strategy** + `dacode_json::RawJson` | B1, B6 | ✅ **APPLIED 2026-09-12** — §13 rewritten as §13.1–§13.5 |
| ~~A17~~ | §2 | **Dependency policy**: pure Rust by default; every C/C++ dependency recorded in [Appendix F](F-c-dependencies.md) with a justification, a replacement plan and a CI allowlist entry | — | ✅ **APPLIED 2026-09-12** — normative at §2.3 |
| ~~A18~~ | §3.7, §4.1 | **Bounded fan-out** with drop-oldest, and the **seq→gap→resync** contract stated as one invariant, replacing §3.7's unqualified "No gap, no duplicate" | — (subsumes A1's resync half) | ✅ **APPLIED 2026-09-12** — §3.7 rewritten, §4.1 + §14 extended |
| ~~A20~~ | §2.4 (new) | **Repository tooling** — *narrowed on sign-off*: not "all repo tooling is Rust", but **two classes**. Tooling the shipped harness depends on is Rust in `xtask`; repo-*authoring* tooling is any language and is **not awaiting a port**. Dogfood rather than port where a tool duplicates a feature; every authoring tool carries a recorded disposition | ~~B16; read with **B10**~~ — **the B10 coupling dissolved**, since nothing ports | ✅ **APPLIED 2026-09-16** — normative at §2.4 |
| ~~A21~~ | §14 → §14.1/§14.2 | **Benchmarking is a first-class concern, separate from correctness**: §14 splits into correctness tests and *benchmarks & evaluation*, naming prompt-cache hit rate as a measured property and making the **in-tree-scorer publication rule** normative | ~~B18~~ ✅ signed; ~~T2 needs **A4**'s `Usage` cache fields~~ ✅ **A4 landed first**, and **A10's `ExchangeRecord`** turned out to be the unrecorded dependency | ✅ **APPLIED 2026-09-18** — §14.1 Correctness + §14.2 Benchmarks and evaluation, all six items landed, links stripped. **It arrived late enough that the spec already cited `§14.2` twice** (§3.2 and §4.5, both written by A4), so applying it is what made those resolve — they were dangling forward references no gate could see. **§14.1 is twelve bullets, not the five "unchanged" text this entry assumed** (thirteen once A22 added `Trust boundary`, fourteen once A7 added `Config writes and the registry hint`), because A3/A4/A5/A9/A15/A10 each added tests and A21 itself added `Clients`. **A twelfth bullet it does not propose was added**: `Clients`, discharging **U101** (§14 had no UI test tier, so A9's mouse and paste rules were normative with nothing that could fail on them) — with the harness deliberately unnamed, since the TUI crate does not exist and §9.1 is open. All four inbound assignments discharged inside §14.2: the compression **level** as a measured knob, `ExchangeRecord` as what hit rate is computed *from*, partial compaction gated on that number, and the publication rule stated first. ⚠️ **This entry's own `Current.` list was stale in three places** (codec named by A15, §4.5's 80 % no longer default per A4, §14 eleven bullets not five) and was corrected rather than transcribed. **New standing gate as a by-product**: `check-links.py --check_section_refs` |
| ~~A22~~ | §3.3, §4.4, §5.3, §7.1, §7.2, §8.2, §8.3, §8.4, §9.2, §14 | **Close four capability seams** — places where the spec presents something as a boundary that is not one: hooks interpolate into a shell, `AllowPattern` is unbounded, sub-agent inheritance caps mode but not scope, and the browser token travels in a URL | — (corrections, not choices; ~~the §4.4 half belongs to **B7**~~ ✅ **landed together, as this row required**) | ✅ **APPLIED 2026-09-18** — all four seams closed across **ten** sections, with **B7's confirmation of A23's `yolo` carve-out in the same pass** so §4.4 was edited once rather than twice (discharging A23's debt). Items **1** (argv array, no shell, templating surviving *into argv elements*), **3** (child-scope intersection) and **4** (referrer/CSP/token hygiene) landed as individually confirmed. ⚠️ **Item 2 was redirected at sign-off and then re-answered**: the capability split stays **reserved** (`Hello` carries `answer_questions`/`resolve_permissions`, granted by default, so tightening is additive not a wire break) and §8.4 now states the trust model *explicitly* — but the **`AllowPattern` subset bound IS applied**, reversing this entry's own orchestrator reading, on the argument that "permissive" describes the *user's* experience and the bound does not touch it (a user may still grant `**` from their own terminal; what is constrained is a client acting on their behalf). **Closes U61.** No `Event` variant, no fold change, no fixture — exactly as predicted |
| ~~A19~~ | §4.8 (new), §3.2, §3.5, §4.2, §4.3, §5.4, §9.2/§10, §12, §14, §16 | **Built-in task tracking**: §4.3's one-line `todo` scratch list becomes a durable **fold-derived task tree** with item kinds and a mandatory typed `verify:`; a bounded **completion gate** in §4.2; `.dacode/plans/` finally specified | B15 ~~; needs **A1** first~~ — **that dependency claim was wrong**, see the banner | ✅ **APPLIED 2026-09-13** — new §4.8, written **self-contained** so it cites nothing A3/A4/B14 have yet created |

| ~~A26~~ | §3.2, §3.5, §5.3, §14.1 | **A turn is an ordered block list** (`TurnView.blocks: Vec<TurnBlock>`, the fold *closes* a block instead of replacing an accumulator) **+ `CloseReason::Completed`**, and §5.3's missing rule that a child session closes | — | ✅ **APPLIED 2026-09-19** — the register's first entry to arrive *already applied*: it came out of M1.1a's review of the three types `bc52e50` designed rather than transcribed ([TODO](../TODO.md) §6 **U108**/**U109**), was approved in-session by the user, and landed the same day. **One wire change and one folded-type change, and neither cost a fixture** — `SessionClosed`'s enum had not been fixtured yet, and `Snapshot` is §8.2's payload so §14.1's derivation never reaches it. Two §14.1 tests, the first the only thing that can fail on the fix |

| ~~A27~~ | §3.2, §3.5, §4.5, §8.2, §14.1 | **Four durable events with no fold destination, given one**: `TurnBlock::User` (a log-only transcript had no user side), the `SettingsChanged` event + `SessionCreated`'s two new fields (`SessionInfo.model`/`.perm_mode` were renderable but unreachable from the log), `SessionInfo.closed` (the session list could not say a session is over — the mechanism U109's own rationale presupposed), and `Compacted`'s meter effect. Plus two fold rules stated: openness is the trailing same-kind block, and a boundary with no deltas behind it *creates* its block | — | ✅ **APPLIED 2026-09-20** — the register's second entry to arrive already applied, and the second from a board finding rather than a B-decision ([TODO](../TODO.md) §6 **U111**): M1.2 Pass C1's fold walk ran A1's tier rule as a diagnostic (*every `Durable` variant must have a fold destination, or the rule's own "iff" is false*) and found four that did not. Approved in-session by the user on the recommendations, applied the same day, **again pre-fixture**: the one wire change (`SettingsChanged`, `SessionCreated` +2 fields — Event 30 → 31) costs zero fixture diff because no golden file existed yet, exactly A26's sequencing rule earning its keep a second time |

**Suggested application order.** ~~A16+A11 first~~ ✅ *applied* (they changed
§13, which A5 and A7 both reference), and **A17** ✅ *applied* ahead of them in
the end, since it depends on nothing and constrains every later dependency
choice, and **A18** ✅ *applied* next, ahead of A1 exactly as recommended — it
narrowed A1 to the tier marker and the `*Ended` variants and discharged its B4
dependency. ~~Next: A1/A14 (protocol and storage shapes — **A15 is held behind
[B12](B-decisions.md)**, see [TODO](../TODO.md) §6)~~ ✅ **A1 applied 2026-09-18**
and **B12 resolved to `zstd`**, so neither hold stands. ~~Next: **A14/A15**~~ ✅
**A14 applied 2026-09-18, the same day as A1.** ~~Next: **A15** — the last of the
protocol and storage shapes —~~ ✅ **and A15 followed the same day, so the
protocol/storage group is DONE**: A18, A1, A24, A14 and A15 have all landed, and
the spec's wire format, log format, storage layout, identifiers, fold and
compression are normative. That is the whole surface M1's protocol crate is built
against, which is why the group was sequenced first. ~~Next: **A2/A3/A4** (engine)~~ ✅ **the ENGINE GROUP IS DONE — A2, A3 and A4 all
landed 2026-09-18**, six deltas in one day. A3 reconciled its guard set with §4.8's
completion gate (now `GuardKind::CompletionGate`, one guard among eight) and with A2's
dispatch ordering, since all three sit in the same pseudocode; A4 then answered the
`compact()` pointer A3 had deliberately left in §4.2, which is why the two had to land
in that order. ~~**Next: A5/A6 (capabilities).**~~ ✅ **A5 applied 2026-09-18**, so the capability
group has started and **A6 is what remains of it** — and A5 is the second delta in six
whose named dependency turned out to be already discharged (A4 had put three of the four
capabilities on `ModelInfo` the day before), which is now a pattern worth expecting
rather than a surprise: six deltas landing in one day means each one's `Depends on` cell
describes a world several applications out of date. ~~then **A19** (task tracking)~~ ✅ **A19 applied
2026-09-13, out of this order**, ~~then A5/A6 (capabilities), then
A7/A8/A9/A10/A13 (hosts and crates)~~ → **A5 applied, and then A9 jumped the queue
from the hosts-and-crates group** — deliberately, and for a reason that outranks the
ordering: **A9 was the last delta M1 owed** (`A1–A5, A9, A14, A15`) while A6, which
this order puts next, is §5's **M4** dependency. The application order is sorted by
dependency and cost; the milestone lists are sorted by what the build needs. Those
are different axes, and when they disagree the milestone wins, because an unblocked
M1 is worth more than a tidy queue. ~~So the group remainders are now **A6**
(capabilities) and **A7/A8/A10/A13** (hosts and crates), and **A12 last**~~ →
**A8, A13 and then A7 all landed (2026-09-18, 2026-09-18, 2026-09-19), so the
hosts-and-crates group is down to A10's §17 body at M5.** What remains is **A6**
(capabilities, M4), **A10's §17 body** (M5), **A25's M5 half** and **A12 last** — it
is the bookkeeping pass that records every other resolution in §16, and its
precondition is now built (U54's marker classification, 2026-09-19), so its §16 job
is mechanical and self-checking. **No owed delta gates M1, M2 or M3.**

**As of 2026-09-17 this is no longer a vote queue — it is an application queue.**
Every delta ~~except **A22**~~ is signed off — **A22 followed the same day**, so
nothing is outstanding — and the ordering above now says what to
*transcribe* rather than what to weigh, and two of its holds have changed: ~~**A15's**
is lifted for the mechanism and retained only for the codec name (a B12 measurement,
not a vote)~~ ✅ **moot — B12 resolved to `zstd` and A15 applied 2026-09-18, so
neither half of that hold stands**, and **A23's `yolo` debt** has an owner now that B7 is signed, so its
confirmation lands with A12's §16.3 half instead of waiting. What has *not* changed:
**A1 precedes every golden fixture**, **A4 and B6's byte-stable tool schema are applied
together**, and **A12 applies last**. The queue is tracked in [TODO](../TODO.md) §3,
which is now the only view carrying it — `tools/attention.py` has no state for
"approved but not yet applied", so fourteen owed transcriptions left its table (U95).

~~**A23**~~ ✅ **applied 2026-09-16, also out of this order**, and for the reason
A19 established rather than a new one: its decision was signed, so holding it
behind unsigned deltas would have delayed a settled question for no benefit. It
was cheaper to jump than A19 was — no `Event` variant, so no fixture and no A1
dependency — but it left the same kind of debt, and it is recorded in A23's
banner: §4.4's `yolo` carve-out is a behaviour **B7 must confirm or overturn**,
which is A3/A4's §4.8 reconciliation in a second place.

~~**A20 is outside that sequence entirely, and should land before M0 rather than
within it.**~~ ✅ **A20 landed 2026-09-16, before M0, exactly as this paragraph
recommended** — like A17 it edits §2, and like A17 its value was in being settled
*before* the thing it constrains is written: M0 scaffolds the workspace, so M0 is
what creates the `xtask` crate, and a tooling policy arriving afterwards would
have meant retrofitting it.

~~It is also the one delta here that must be **read together with a B-decision it
does not depend on** — B10, because three of the five Python tools parse YAML and
porting them either adds a YAML crate or moves repo config to TOML, which is B10's
subject. Signing B16 alone would settle B10 by accident.~~ **That coupling
dissolved on sign-off and B10 is now free-standing.** The narrowed policy leaves
every authoring tool in Python, so nothing ports and **no YAML crate enters the
Rust tree** — the question the coupling existed to protect against cannot arise.
Two facts settled it: `agents-sync.py` *serializes* YAML as well as parsing it,
because the generated `.opencode/agent/*.md` frontmatter is opencode's format and
not ours to choose; and Python has no stdlib TOML *writer*, so moving the tracker
would trade PyYAML for `tomlkit` — a lateral dependency swap in a tool already
slated for retirement. **PyYAML is therefore permanent**, which also removes the
last dependency-based argument for converting `agent-rules.yaml` to TOML. B10 can
now be signed on its own merits, whenever.

**A19 jumped the queue, and the reason matters for whoever applies A3/A4.** It was
sequenced after the engine group because it edits §4.2 (where A3 puts the loop
guards) and §4.5 (where A4 puts the caching invariants), and because its
completion gate *is* one of A3's guards. B15 was signed off before A3/A4 were, so
rather than hold a signed-off decision behind three unsigned ones, §4.8 was made
**self-contained** — it describes the gate and the append-only rendering in its
own words instead of citing a §4.5 fingerprint or an A3 guard set that do not
exist yet. **The debt that creates is now A3's and A4's:** when they land, they
must tighten §4.8's wording into real cross-references, and A3 must reconcile its
guard set with a gate that is already in §4.2's pseudocode. It still must not go
later than A9, which unifies the workflow and sub-agent tree views — A19's task
tree is the third arm, and §9.2 now says so.

---

## Detail

### A1 — §3.1, §3.2, §3.5, §3.6, §3.7, §4.3, §4.8, §12, §13, §14: the two-tier event model — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, on [B3](B-decisions.md)'s sign-off of 2026-09-17, with
> **A25's M1 half riding along** as its own banner required and **§16 item 6's
> `Standing` status** landing where it is normative. A25 is the entry after A22 below,
> cited as plain text rather than by anchor, because an applied heading gains a suffix
> that moves the slug. **The "before any golden fixture" precondition held**: no
> fixture exists, because the crates are still stubs. Ten spec sections were edited —
> §3.1 (`HostId`, `SessionRef`, `Cursor` restated), §3.2 (`EventTier`, `tier()`,
> `seq: Option<Cursor>`, `TextEnded`/`ThinkingEnded`, `WriterEpoch`, the tier rule),
> §3.5 (`Snapshot.epoch`, one-fold-two-tiers), §3.6 (durable-only log, `LogHeader`,
> the three fencing obligations), §3.7 (both resync triggers), §4.3 (`HostId`
> excluded from the scan, for now), §4.8 (`Standing`), §12 (the header in the
> layout), §13 + §13.2 (the persistence claim and the `Json` alias), §14 (fixture
> rule + three tests) — plus §1's principle 2, which asserted that *every* event is
> in the log.
>
> **Six scoping decisions taken while transcribing. Four were put to the user and
> answered 2026-09-18; two are fold-ins the delta did not mention.**
>
> 1. **`seq` became `Option<Cursor>`, which the delta as drafted did not say.** It
>    said "add a tier marker" and was silent on the cursor of an event that is never
>    appended. A18's §3.7 makes a missing `seq` the *only* gap signal, so numbering
>    deltas would have put permanent holes in the log's sequence and made clients
>    resync forever — precisely the failure §3.6 names as worse than a corrupt log,
>    because the bytes still parse. A18's own step 1 ("the engine assigns `seq` when
>    it appends to the log") already implied durable-only; this states it. The
>    self-heal that makes a cursor-less delta safe is the boundary **replacing**
>    rather than extending a block's text, so a duplicated or dropped delta costs
>    nothing.
> 2. **The tier is a rule, not a list: `Durable` iff the fold consumes it.** Chosen
>    over enumerating ~30 variants so that A14's session-organization events and
>    B17's `finding` classify themselves rather than needing this entry reopened. It
>    leaves exactly four `Live` variants. Two classifications needed an argument
>    rather than an inspection and are written into §3.2 for that reason:
>    `ContextInfo` is durable (it is in `Snapshot.context`, so the rule decides it),
>    and **`Error` is live** — every error that *changes state* already has a durable
>    carrier in `TurnFinished.stop` or `AgentIdle.reason`, so one that changes
>    nothing is a diagnostic. `Error` is the only classification in this delta that
>    is expensive to reverse, and the user confirmed it explicitly.
> 3. **Only the epoch is wire-visible** of A25(e)'s three mechanisms. `WriterEpoch`
>    as a durable event plus `Snapshot.epoch`, with an epoch *decrease* as a second
>    resync trigger beside the `seq` gap. The **idempotency key** and the **leaf
>    CAS** are stated in §3.6 as obligations on the store API, with the mechanism
>    left to doc 05 — neither is something a client renders, so putting them on the
>    wire would have grown `Envelope` for no reader. B21's `flock` is **restated** as
>    the local mechanism, not weakened or replaced.
> 4. **A25(a) took a third answer, and the user approved the deviation.** Its entry
>    lists "a location component or an opaque handle"; both break A23's requirement
>    that the four scanned ids stay **exact-matchable**. What landed is neither:
>    `SessionId(Ulid)` is byte-identical, and the *route* is separate —
>    `HostId` + `SessionRef { host, session }`, used only where a command may name a
>    remote session. A ULID is already globally unique, so it can name a session
>    anywhere; what it lacked is a route, and a route is a host attribute — the same
>    shape A25 itself gives the model endpoint. §4.3 gained one clause recording that
>    `HostId` joins the scanned set when a tool argument can name a host (M5).
> 5. **U66 folded in: the log gains a header record.** Not in the delta, but §6 owed
>    it to A1 *by name* ("A1 is the natural place … cheaper before the first fixture
>    exists"), and the epoch needed a durable initial record anyway, so the two share
>    a mechanism. Line 1 of `events.jsonl` is now a `LogHeader` carrying
>    `format_version` — **the same counter §7.2's `Hello` sends**, so a log and a wire
>    version cannot drift. ⚠️ **It edits a sentence A24 landed one day earlier**: the
>    header carries no cursor and occupies a line, so §3.6 no longer says the cursor
>    "equals line number + 1" but that it is "derivable by arithmetic from the
>    record's position". The invariant A24 was protecting — do not weld the cursor to
>    the JSONL representation — is strengthened by the change, not weakened, since a
>    fixed `+1` was exactly such a weld.
> 6. **U27's §3.2 half discharged:** `ToolCall { args: Json }` → `args: RawJson`.
>    §13.2's alias note is **narrowed, not deleted** — §4.3's `perm_scope`/`call`
>    signatures still spell `Json` and are A2's to rename, so U27 stays open on that
>    half alone.
>
> **Debt this creates, recorded rather than left to the next reader.**
> **A12** strikes §16 item 6: `Standing` is normative in §4.8 now (goals only — a
> non-`Goal` in `Standing` is rejected, because applied to a task it is a way to dodge
> the checker), and §16's text is deliberately *left in place* so the bookkeeping pass
> owns the removal in one piece rather than piecemeal. Until then `attention.py`
> renders §16.6 as an open vote — **U95's false positive, accepted by the user.**
> ~~**A14** owes three *log* fixtures and needs no tier decision, the rule supplying it.~~
> ✅ **Discharged the same day** — A14 applied 2026-09-18 and took exactly that path:
> the rule classified all three events `Durable` with no tier decision to take, and §14's
> derivation put them in the log fixture set with no list to update.
> **Doc 05** (already expanded) should carry the `LogHeader` and the
> epoch/idempotency/CAS mechanism when M1 starts. **Doc 03 stays a summary** despite
> its `[DESIGN DELTA]` marker now being stale — flipping it would deepen an unstarted
> milestone's doc, which hard rule 1 forbids; the marker is corrected when M1 starts.

**Current.** §3.2 (`:89-124`) is a single flat `Event` enum mixing live deltas
(`TextDelta`, `ThinkingDelta`, `ToolOutputDelta`) with durable events
(`TurnStarted`, `ToolCall`, `ToolResult`, `TurnFinished`, `Compacted`) and no
tier marking. §3.7 (`:212-219`) reconnects via `GetSnapshot` → `Subscribe
{ since }` and asserts "No gap, no duplicate" but specifies no gap *detection*.

**Proposed.**
- Add a tier marker so the log writer and replay path can filter without
  matching every variant: `Event::tier() -> Tier { Durable, Live }`, or a
  `durable: bool` on `Envelope`. Prefer the **method on `Event`** — the tier is
  a property of the variant, not of the envelope instance, and a method cannot
  be set inconsistently by a producer.
- State the replay rule: **live deltas are never persisted**; on replay a client
  reconstructs the final value from the durable boundary, never from deltas.
- Add the **full-value boundary variants** that B3 requires and §3.2 lacks:
  `TextEnded { turn, text }` and `ThinkingEnded { turn, text }`. `ToolResult`
  already serves as the boundary for tool output. Without these, a client that
  misses the deltas cannot render the turn from the log alone — and since live
  deltas are never persisted, *replay has nothing else to render from*.
  *(Originally justified here as breaking §3.7's "no gap" claim; A18 restated
  that claim as a property of `seq` continuity rather than of value
  completeness, so the two are distinct guarantees and this bullet stands on its
  own.)*
- ~~§3.7: adopt **seq-numbered events + client-side gap detection → resync**
  (openclaw `gateway-client/src/protocol-client.ts:413-425`) as the reconnect
  contract, so a silently half-open WS is recovered rather than rendering stale
  state.~~ **→ moved to A18 (§3.7/§4.1 below), ✅ applied 2026-09-12 — this
  bullet is discharged.** A18 expanded it into the full bounded-fan-out contract
  and was signed off independently of B3/B4, so this delta now carries only the
  tier marker and the `*Ended` boundary variants.
  *(Deliberately a plain-text cross-reference, not an in-page anchor: these
  headings gain a resolution suffix when the delta is applied, which changes the
  slug and silently breaks any anchor pointing at them. See [TODO](../TODO.md)
  §6's repo-hygiene traps.)*

**Depends on.** B3 (which this delta effectively resolves). ~~and B4 (the resync
contract is what makes WS safe to keep)~~ — **B4 resolved 2026-09-12** (WS kept),
and its resync half landed in A18, so neither blocks this any more.

**Recommend accept.** This is the most structural delta in the set: it changes
the `Event` enum and the fold, and everything downstream ([03](../part1-foundations/03-protocol.md),
[05](../part2-engine/05-event-log-storage.md), [08](../part2-engine/08-agentic-loop.md))
assumes it. Apply it before any golden fixtures are written (§14) — retrofitting
a tier marker into existing fixtures is exactly the "compatibility event" §14
warns about. ✅ **Done 2026-09-18** — applied with the six scoping decisions above,
and the precondition held: the crates are still stubs, so no fixture was retrofitted.
The `Current.` paragraph below describes a §3.2 that no longer exists and is kept as
the record of what changed.

---

### A2 — §4.2, §4.3, §13.2, §14: concurrency policy instead of a bool — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, the first of the engine group. The amendment landed
> exactly as signed — `parallel_safe() -> bool` became
> `fn concurrency(&self, args: &RawJson) -> Concurrency` over `Shared`, `Exclusive`
> and `Keyed(ConcurrencyKey)`, with the **tool computing its own key** rather than
> returning a closure. **Its obligation on the transcription is discharged**: §4.3 now
> carries the author guidance — what each variant means, how to compute a key, why the
> key must be canonical, and what the default is — because an undocumented policy enum
> gets defaulted to `Exclusive` everywhere and the parallelism is silently lost.
>
> **Four scoping decisions taken while transcribing.**
>
> 1. **No wire variant was added, so the audit-trail benefit is stated rather than
>    built.** The amendment's argument for a value over a closure is that the key is
>    "loggable in the `PermissionResolved`/audit trail". Adding a field to
>    `PermissionResolved` would have been a wire change with a fixture, for something
>    the delta argued *for* but never proposed. What landed is the property — a key is a
>    value, so it **can** be written into the trail, where a closure can only be
>    described — with §4.3 saying so and the actual recording left to §4.4's audit path
>    and doc 08 when it is deepened. If a concurrency decision turns out to need its own
>    durable event, that is a new delta, not a reading of this one.
> 2. **Which tool gets which policy is brainstorm-owned content and is marked pending,
>    not invented.** §1's deferral banner allows a transcription to land the approved
>    *shape* while marking content for the session, and names "which tools ship" as
>    exactly that class. So §4.3 specifies the enum, the default, the canonical-key
>    requirement and the three tests that pin the semantics — and **does not assign a
>    policy to `read`, `edit`, `bash` or anything else.** A2's own proposal used two
>    `edit`s as its worked example, so §14's tests use it too, but that is a test of the
>    mechanism and not a policy assignment. §4.8 written self-contained by A19 is the
>    precedent that this works.
> 3. **U27's second half discharged, and §13.2's alias note dropped rather than
>    narrowed again.** A1 renamed §3.2's `ToolCall.args`; this renames §4.3's
>    `perm_scope` and `call` (and `concurrency`, which is new). With no bare `Json` left
>    anywhere in the spec, the note that defined the alias has no site — verified by
>    `rg '\bJson\b'` over the spec returning nothing outside `RawJson`/`dacode-json` —
>    so it is replaced by the rule it was covering for: opaque sub-JSON is always
>    spelled `RawJson`, and spelling it out at each site is the point. U27 closes on
>    both halves.
> 4. **Two files outside the plan were stale the moment this landed, and one of them
>    reaches every coding agent in the repository.** The banner named
>    `.agents/roles/_rust.md`, which told every agent that "tools that can run
>    concurrently declare `parallel_safe`"; it is corrected, and
>    `.agents/skills/dacode/SKILL.md`'s trait snippet with it, and the four **generated**
>    `.opencode/agent/*.md` adapters regenerated by `tools/agents-sync.py`. This is the
>    same class as A16's `_rust.md` false-type caveat: a role file is loaded into every
>    subagent's context, so a stale line there is not a doc lag, it is an instruction
>    being given to every worker. **Doc 08 needed more than a marker flip** — it
>    specified `Concurrency::Dynamic(fn(&args) -> Key)`, the closure arm the amendment
>    *rejects*, so a reader would have implemented the rejected design. Corrected with a
>    strike-and-annotate, **marker and depth untouched**: doc 08 is still a migrated
>    summary and M1 has not started, and removing one false line adds no depth. **Doc
>    14's content was already correct** (it names `concurrency`), so only its
>    `[DESIGN DELTA]` marker is stale and it stays, per A1's doc-03 precedent.
>
> **Debt this creates, recorded rather than left to the next reader.** **Doc 08** owes
> the dispatch algorithm when it is deepened — §4.2's pseudocode now says calls are
> partitioned by policy and scheduled, but *how* an `Exclusive` call interleaves with a
> stream still arriving, and whether a `Keyed` group may start while an `Exclusive`
> drains, is loop content and belongs to the 🧠 session. **A3 owns reconciling its guard
> set with both §4.8's gate and this dispatch order**, since all three now sit in §4.2's
> pseudocode. **Doc 14's** marker is corrected when M1 deepens it, and it owes the
> per-tool policy table the brainstorm produces. **A3/A4 still owe §4.8 tightened
> cross-references** (A19's scoping decision 1), unchanged by this.

**Current.** §4.3 (`:287`) `fn parallel_safe(&self) -> bool { false }`; §4.2
(`:259`) "parallel where tools are marked `parallel_safe`".

**Proposed.** Replace with a per-tool **concurrency policy** resolved at
dispatch (oh-my-pi `agent-loop.ts:2903-2929`): `Shared` runs concurrently with
other shared calls; `Exclusive` serializes behind the running shared set;
anything mutating the working tree defaults to `Exclusive`. Two `edit`s on
different files are shared, on the same file exclusive.

**Amendment.** [08](../part2-engine/08-agentic-loop.md) sketches the third arm
as `Concurrency::Dynamic(fn(&args) -> Key)` — a closure, which is awkward to
express as a trait return value and cannot be `Debug`/logged. Have the **tool
compute the key itself**:

```rust
pub enum Concurrency { Shared, Exclusive, Keyed(ConcurrencyKey) }
// in trait Tool:
fn concurrency(&self, args: &RawJson) -> Concurrency { Concurrency::Exclusive }
```

`ConcurrencyKey` is an opaque interned string (the file path, the host, the
lock name). Same expressiveness, no closure in the signature, and the key is
loggable in the `PermissionResolved`/audit trail. §4.2's pseudocode line 259
changes with it.

**Depends on.** Nothing. Independent of every B decision — and it turned out to be
independent of A1 too, since it adds no `Event` or `Command` variant and so needs no
fixture and no tier decision.

**Recommend accept, amended as above.** **Done 2026-09-18**, with the amendment and
its documentation obligation both discharged, and with the four scoping decisions in
the banner. **The "Current." and "Proposed." text below is historical**: the line
numbers predate four applied deltas, and the `Dynamic(fn)` arm it quotes from doc 08
is the one the amendment rejects.

---

### A3 — §3.2, §3.6, §4.2, §4.8, §11, §14: loop guards, overlap, synthetic tool results, steering — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, the second of the engine group. Everything the sign-off
> approved landed as proposed: the guard set, continuation driven by work remaining
> rather than a step count, streaming/execution overlap with the transcript fragment
> committed **before** a tool's side effects start, the pairing invariant with
> **synthetic tool results** on every abandoning path, and steering as distinct from a
> hard cancel. **Six spec sections, against the one the delta named** — because the
> guard set needs types §3.2 never defined, a commit boundary §3.6 never listed, an
> exit-code mapping §11 never stated, and the reconciliation with §4.8 that A19 left
> owed.
>
> **Four scoping decisions taken while transcribing.**
>
> 1. **Two types the spec used but never defined had to be created, and A3 could not
>    land without them.** `StopReason` appears in `TurnFinished { stop }` and §4.2's
>    pseudocode branches on `stop == EndTurn`, but no enum existed anywhere — and this
>    delta's synthetic-result rule fires "on any abort / **`length`** stop / error tail",
>    which is a `StopReason` value that did not exist. `IdleReason` was the same, its five
>    variants listed only in a trailing comment. Worse, §4.2 already called
>    `emit AgentIdle(NeedsInput, "turn limit")` — a **two-argument shape the one-field
>    variant could not hold** — so the spec had been describing an event it did not have.
>    Both are now real enums, with `MaxTokens`/`Paused`/`Cancelled`/`Error` named in
>    `StopReason` and `ToolOutput` + `SyntheticReason` defined beside them. This is the
>    same class as `SessionInfo` being a comment until A14 needed it: a type mentioned in
>    a signature and never defined is invisible until a delta has to add a variant, and
>    that delta pays for every earlier one that did not.
> 2. **`AgentIdle` gained a typed `guard: Option<GuardKind>` — a wire change to an
>    existing variant, recorded rather than folded in.** A guard termination has to reach
>    the client, and there were three shapes. Extending `IdleReason` with six guard
>    variants **breaks §11**, whose four exit codes map onto it, and §9.2's five status
>    chips. An untyped detail string **violates principle 3** — a reason a UI must parse
>    to render distinctly is hidden state behind a fold. So `IdleReason` stays coarse and
>    aligned to the exit codes and chips, and the guard is a separate typed field. It is
>    `Option` and omitted when `None` (§13.5's convention), so an ordinary `Done` is
>    byte-identical to before and the change is additive on the wire. **`GuardKind`
>    growing is a config-and-code change; `IdleReason` growing is a contract change in
>    two places** — that asymmetry is the point of the split and is now written into both
>    sections.
> 3. **The guard numbers are marked pending, not invented.** §1's deferral names "the
>    guard numbers" as brainstorm-owned content explicitly, so what landed is the *shape*
>    — a table of eight guards, each with a config key, a firing condition and the
>    `IdleReason` it exits through — and the reference-harness values
>    (`MAX_PAUSED_TURN_CONTINUATIONS=8`, `MAX_SOFT_TOOL_ESCALATIONS=3`) stay in the plan
>    as candidates. What is normative is that a guard is a **named, configured, reported**
>    termination, so tuning one is a config change and adding one is never a wire change.
> 4. **The hand-over resume is a slot, not a mechanism.** `GuardKind::HandOver` exists so
>    the termination has somewhere to report through, and §4.2 says plainly that what the
>    resume contains, whether it is durable, whether bail-out is a guard or a tool call,
>    and how it meets `Fork` and §3.6's single-writer lock are **not specified here**.
>    Inventing them would have pre-empted the 🧠 session the sign-off explicitly did not
>    bypass, and would have created an `Event` variant needing a fixture for a design
>    nobody has done. **U96 stays open and is annotated, not closed.**
>
> **Two debts discharged, both owed by other deltas.** **A19's**: §4.8 was written
> self-contained because it landed before the guard set existed, and its gate is now
> reconciled — §4.8 still owns *when* `check` is evaluated and the reminder escalation,
> while §4.2 owns the termination, which fires `GuardKind::CompletionGate`. So "the gate
> is hard but bounded" became a typed event rather than a property of pseudocode, and
> `task.reminders_max` is one row in the guard table. **A2's**, created yesterday: its
> dispatch ordering sits in the same pseudocode block, so §4.2 now shows scheduling under
> §4.3's policies *and* guard evaluation in one flow, with the pairing step between them
> — which is where the two interact, since a guard firing with calls outstanding is
> exactly the case that needs synthetic results.
>
> **Debt this creates, recorded rather than left to the next reader.** **The 🧠
> agentic-loop session** owns the guard numbers, the stuck-loop detector's actual
> heuristic (`loop.stuck_loop_*` is a key with no content behind it), and everything in
> **U96**. **A4** must land compaction as *one mechanism* with the hand-over — §4.2 now
> says so in the section rather than only in this banner — and `compact()` no longer
> appears in the pseudocode at all, so A4 has to put it back deliberately rather than find
> it. **Doc 08** owes the dispatch algorithm and the steering/interrupt detail when it is
> deepened; its `[DESIGN DELTA]` marker for this delta is now stale and stays, per A1's
> doc-03 precedent. **Doc 03** is now five deltas out of date (M1.1). **§3.2's `Live` set
> is unchanged** — no variant was added, only typed, so A1's "exactly four" still holds.

**Current.** §4.2 (`:251-267`) terminates on `if turn_count > agent.max_turns`;
cancellation at `:276-278` drops the provider stream and kills tools by process
group.

**Proposed.**
- **Reframe termination as a guard set**, not one counter. Keep `max_turns` as a
  hard ceiling and add: a **deadline** timer; a **pause-continuation cap** for
  non-terminal `pause_turn`; a **soft-tool-escalation cap**; a **per-tool error
  budget**; and a **loop-recovery intervention** that escalates warning →
  terminate on a detected stuck loop. Sources: oh-my-pi
  (`config.deadline`, `MAX_PAUSED_TURN_CONTINUATIONS=8`,
  `MAX_SOFT_TOOL_ESCALATIONS=3`), forge (`ToolErrorTracker`), openclaw
  (`critical-tool-loop`).
- **Continuation is driven by "more tool calls, or pending steering/follow-up
  messages?"**, not by a step count (oh-my-pi
  `while hasMoreToolCalls || pendingMessages`; openclaw's outer follow-up queue +
  inner tool loop).
- **Overlap streaming with execution**: enqueue async tool batches while the
  model is still streaming, and **commit the transcript fragment that owns a tool
  call before its side effects start**, so a crash never loses the record of
  what was attempted (openclaw `agent-stream-response.ts:178-218,296-333`).
- **The tool-pairing invariant is sacred.** On any abort / `length` stop / error
  tail, **synthesize placeholder tool results** so every `tool_use` has a
  matching `tool_result` — providers reject unpaired histories (oh-my-pi
  `createSyntheticToolResultMessage` with a `__synthetic` discriminator). This
  needs a representation in §3.2 too: a synthetic marker on `ToolResult`.
- **Steering vs interrupt**: a cooperative steering signal (a mid-batch user
  message that skips and releases remaining calls) is distinct from a hard
  cancel that kills side-effecting tools cleanly. §4.2 has only the hard one.

**Depends on.** Nothing, though the synthetic-result marker rides in on A1 — ✅ **and
it did**: A1 landed first, so the marker arrived as a `ToolOutput::Synthetic` variant
classified `Durable` by the tier rule with no tier decision to take, and §14's
derivation gives it a *log* fixture for free. It also turned out to depend on **A2**
in the other direction — not for its content but for its *site*: A2 had rewritten
§4.2's dispatch ordering the day before, so this delta had to reconcile the guard set
with it rather than with the pseudocode it was drafted against.

**Recommend accept.** Every one of these is a bug class dacode would otherwise
rediscover in production; all four harnesses converged on them independently. **Done
2026-09-18**, to six sections rather than the one named here, with the four scoping
decisions in the banner. **The "Current." and "Proposed." text below is historical** —
its line numbers predate five applied deltas, and its "this needs a representation in
§3.2 too" is now the `ToolOutput`/`SyntheticReason` pair plus two enums the spec had
been using without defining.

---

### A4 — §3.2, §4.5, §4.8, §5.6, §6, §14: prompt caching + expanded compaction — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, the engine group's last. **The caching half landed as
> proposed** — the stable fingerprinted prefix with breakpoints, append-only message
> growth, longest-stable-prefix resync, volatility hygiene as engine invariants, and
> `Usage` gaining both cache counters. **Both amendments to the compaction half landed
> as amended**, not as originally proposed: compaction is opt-in and *not* v1's
> default, and partial compaction is a candidate awaiting a measurement rather than an
> adopted feature. **B14 folded in** as the buffer representation. Six spec sections
> against the one the delta named.
>
> **Four scoping decisions taken while transcribing.**
>
> 1. **The dependency this delta's own banner warns about twice was already
>    discharged, five days before it was signed.** Both the banner and the
>    `Depends on` line say A4 "must be applied **together with B6**'s canonical
>    byte-stable tool schema or the ordering invariant is unenforceable". But §13.3 has
>    carried exactly that since **A16 landed on 2026-09-12** — "Generated schemas must be
>    **canonical and byte-stable**: `title` stripped, key order pinned. Tool-definition
>    churn, *including pure reordering*, destroys the cache-stable prefix (§13.4), so
>    byte-stability is a caching requirement and earns a golden fixture (§14), not a
>    comment" — and §13.4 independently states that "anything volatile — timestamps,
>    per-request ids, **tool ordering** — stays out of it". §14's `xtask` bullet already
>    promises the golden-output test §13.3 says the schema earns. **So A4 cites §13.3
>    and §13.4 rather than re-adding them.** Transcribing the warning literally would
>    have duplicated normative text in a second section and left the next reader to
>    reconcile two copies. The generalisation is the mirror image of A15's experience:
>    a delta's `Depends on` cell describes the world when it was *drafted*, so the
>    pre-flight must check whether each named dependency has already been discharged by
>    an earlier application.
> 2. **Three items composed into a hole none of them stated, and the resolution is
>    forced by amendments already signed.** Amendment (a) makes compaction opt-in and
>    says v1's answer to a filling context is A3's hand-over. But **A3 landed the
>    hand-over as a slot, not a mechanism** (`GuardKind::HandOver`, deliberately —
>    designing it would have pre-empted the 🧠 session). And **U68** says a token-estimate
>    under-run has no first-class failure path, prescribing "treat overflow as retryable,
>    **compact**, and re-issue" — which amendment (a) has just made non-default. So v1
>    had *no* automatic response to a filling context, and both candidates were
>    unavailable. What landed: overflow is a **retryable provider error with two
>    branches** — compact-and-re-issue when compaction is enabled, otherwise terminate
>    through §4.2's guard set **with a remedy the user can act on** (`/compact`, a
>    hand-over, a fork) rather than failing opaquely mid-turn after the tool work is
>    already paid for. §6 states it as a provider-level path and §4.5 as the estimate's
>    consequence, and §6 records why overflow is *not* a 429: backing off and retrying
>    the same request cannot succeed. **U68 closes**; the honest statement it forced is
>    that **v1's default answer to a filling context is "stop and tell the user"**.
> 3. **`Usage` was referenced by two events and never defined — the same defect A3
>    fixed twice the day before.** `TurnFinished { usage: Usage }` and
>    `SubAgentFinished { usage: Usage }` both named a type with no definition anywhere,
>    so A4's cache fields had nowhere to go. It is now a real struct with four counters,
>    and the reason all four are on the *event* rather than being a §6 computation detail
>    is stated: pricing a cached read at the input rate is not imprecision but a wrong
>    number that overstates exactly the saving the cache exists to deliver, and a wrong
>    cost is worse than an absent one because it is believed. **This closes U71**, which
>    said precisely that and offered "either A4 first, or `cost_usd` stays `None`".
> 4. **Compaction's trigger survives but its default does not.** `compact_at` and
>    `/compact` stay, because an opt-in feature still needs a threshold; what changed is
>    that automatic compaction is **off by default** and §4.5 now says so in terms rather
>    than leaving "compacts at 80 %" standing as v1's behaviour. The five rules the
>    proposal listed all landed (append-only boundary, turn-aligned tool-pair-safe cut,
>    prune-before-summarize with a protected window and savings floor, an incremental
>    structured summary that later compactions *update* under a hard cap, and a retention
>    floor). **Partial compaction is recorded as a candidate with both hazards named** —
>    it rewrites bytes already sent, so it must go through truncate-and-re-emit and never
>    a byte-rewrite; and it destroys the cache prefix from the cut point, trading a cache
>    miss for context space, a trade §14.2's hit-rate metric is the instrument for. **U97
>    stays open and is not adopted before that number exists.**
>
> **One debt discharged, owed jointly with A3.** A19 wrote §4.8 self-contained because
> it landed before the sections it needed existed, and recorded that A3 and A4 own
> tightening those into real cross-references. A3 did the guard-set half; this did the
> other two — §4.8's "context rendering never rewrites bytes already sent" now names
> itself as §4.5's append-only rule applied to one producer, says why a status flip costs
> **one segment** in B14's index rather than a rewrite, and places the rendering *outside*
> the fingerprinted prefix under §5.6's rule for volatile content. **B14 is thereby
> applied too**, and its own sign-off's second consequence landed with it: §5.6's
> environment block is placed after the prefix and **excluded from the fingerprint**, so
> switching branches costs one segment rather than the whole prompt.
>
> **Debt this creates, recorded rather than left to the next reader.** **U96 is now
> load-bearing in a way it was not yesterday**: §4.5 names the hand-over as v1's answer to
> a filling context and §6's overflow path offers it as a remedy, so an undesigned
> mechanism is now cited by two normative sections. It stays open and the 🧠 agentic-loop
> session owns it — but the session is no longer refining an idea, it is filling a slot
> the spec points at. ~~**§14.2 does not exist yet** (A21 is unapplied), so the cache-hit-rate
> metric that partial compaction and the T2 tier both depend on is cited by section number
> rather than by content; **A21 must place it**~~ ✅ **A21 applied later the same day and
> §14.2 exists**, naming the hit-rate metric, gating partial compaction on its number, and
> naming `ExchangeRecord` as what the rate is computed from — so the two citations this
> banner made by section number now resolve to content. **B18's T2** needs `Usage`'s four
> counters, which now exist. ⚠️ **Those two citations were dangling references while this
> banner was being written**, and no gate could see them: the spec forbids markdown links,
> so nothing checked its internal `§N.M` cross-references. A21's pass added
> `check_section_refs()` to `check-links.py` for exactly that reason. **Doc 09 and doc 10** carry the plan-side caching and
> compaction detail and are both still summaries; **doc 03** is now six deltas out of date
> (M1.1). **A4 added no `Event` or `Command` variant** — `Usage` is a field type, not a
> variant — so it costs no fixture of its own, though `TurnFinished`'s existing fixture
> now has four counters in it.

> **✅ SIGNED OFF 2026-09-17 — approved for application; NOT yet transcribed into the
> spec.** The **caching half is approved as proposed**: stable fingerprinted prefix with
> breakpoints, append-only message growth, longest-stable-prefix resync on any rewrite, and
> volatility hygiene as engine invariants (no per-request timestamps/UUIDs; **stable tool
> ordering, since reordering is a cache kill even when the set is identical**), with `Usage`
> gaining `cache_read_input_tokens` / `cache_creation_input_tokens`. **B14** folds in as the
> buffer representation, and this must be applied **together with B6**'s canonical
> byte-stable tool schema or the ordering invariant is unenforceable.
>
> **Two amendments to the compaction half, in the user's words:** *"compaction could be a
> later feature (opt-in). Also it could be partial compaction (by stripping some later tools
> output that doesn't affect the following conversation, but requires some experiments while
> could affect caching)."* What each means, precisely: **(a)** compaction is **opt-in and
> later**, so §4.5's "compact at 80 % of the window" is **not v1's default answer** — v1's
> answer to a filling context is A3's hand-over/resume, and the transcription must not leave
> §4.5 implying that automatic compaction ships at M1. **(b) Partial compaction** (stripping
> later tool output that the following conversation does not depend on) is recorded as a
> **candidate requiring experiment, not an adopted feature**, with both hazards named: it
> rewrites bytes already sent, so it must go through the truncate-and-re-emit resync path
> and never a byte-rewrite (§4.8 forbids rewriting sent bytes for exactly this reason); and
> it destroys the cache prefix from the cut point onwards, so it trades a cache miss for
> context space — a trade **B18's T2 cache-hit-rate metric is the instrument for**.

**Current.** §4.5 (`:318-328`) covers context, a token estimate, and a
single-paragraph compaction description. No caching discipline at all.

**Proposed — new "Prompt caching" subsection.**
- **Stable fingerprinted prefix**: freeze system prompt + tool definitions,
  fingerprint them, place cache breakpoints at the end of the prefix
  (Anthropic `cache_control`, OpenAI explicit breakpoints, Gemini implicit).
  The prefix must be **byte-stable** across turns or the cache is defeated.
- **Append-only message growth.** Prefix caching is content-addressed from the
  front: *any* edit to an earlier message invalidates everything after it.
- **Longest-stable-prefix resync**: when history must be rewritten (compaction,
  prune, model switch, field injection), truncate to the longest byte-stable
  digest prefix and re-emit only the suffix — never rebuild the whole array.
  Version-tag message digests to detect incoherence.
- **Volatility hygiene** as engine invariants: no per-request
  hashes/timestamps/UUIDs in the system prompt; **stable tool ordering —
  reordering is a cache kill even when the set is identical**; dedup repeated
  tool output; `ttl: "1h"` breakpoints for slow interactive sessions.
- **`Usage` gains cache-token fields**: `cache_read_input_tokens`,
  `cache_creation_input_tokens`, so hit-rate is measurable rather than inferred.
  Source: oh-my-pi `append-only-context.ts` (`StablePrefix` fingerprint `:355`,
  `syncMessages` `:226,:296`); validated by the api-introspect cache-failure
  taxonomy ([25](../part5-outreach/25-introspection.md)).

**Proposed — expand the compaction paragraph.**
- The boundary is a **new append-only entry, never a byte-rewrite** (openclaw).
- Cut only at a **turn-aligned, tool-pair-safe cut point** (`findCutPoint`).
- **Prune before summarizing**, with a protected recent window and a
  minimum-savings floor (oh-my-pi protects the newest ~40 k tool-output tokens,
  needs ≥20 k savings, floor 50 tokens; never cuts at a `toolResult`).
- **Incremental structured summary** (Goal / Progress / Decisions / Next-Steps)
  that later compactions **update** rather than replace, hard-capped (openclaw
  caps at 16 k chars).
- A **retention floor** on recent tokens (openclaw keeps ~20 k).
- An optional **method chain** with fallthrough (`remote → snapcompact →
  handoff → shake → soft`), recorded as future work, not v1.

**Depends on.** Nothing — but it *creates* a dependency: **B6's canonical,
byte-stable tool schema is required by A4's stable-tool-ordering invariant.**
~~Apply A4 and B6 together or the caching guarantee is unenforceable.~~ ⚠️
**That condition was already satisfied five days before this delta was signed**:
B6's half has been normative in §13.3 since **A16 applied on 2026-09-12**, with
§13.4 independently ruling tool ordering out of the cached prefix, so A4 cites both
rather than re-adding them. See scoping decision 1 in the banner — a `Depends on`
cell describes the world at drafting time, not at application time.

**Recommend accept.** [09](../part2-engine/09-context-caching.md) calls this
"the highest-leverage performance/cost work in the whole system", and the spec
currently has no hook for it at all. **Done 2026-09-18**, to six sections rather
than the one named here, with the four scoping decisions in the banner. **The
"Current." and "Proposed." text below is historical**: §4.5 is now *Context, prompt
caching and compaction* and carries all of it. **One proposal was deliberately not
transcribed** — the optional method chain with fallthrough
(`remote → snapcompact → handoff → shake → soft`) — because this entry itself records
it as "future work, not v1", and three of its five stages are the hand-over whose
mechanism **U96** leaves to the 🧠 session. Specifying a fallthrough chain over
undesigned stages would have been invention, not transcription.

---

### A5 — §4.3, §4.4, §6, §14: the capability model as data, B5's subset, egress policy — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, the capability group's first. **The redirect is the
> substance and it landed as redirected**: native `gemini` is *not* in v1, and §6
> instead says why the capability model is the stronger position — a provider that
> needs something unusual is a **value on `ModelInfo`**, not a `match` in §4.2, so
> onboarding a non-openai-compatible provider means implementing `complete` and
> reporting honest capabilities. B5's subset, the SSRF/egress policy and the retry
> detail all landed unchanged. **Four spec sections against the one named**, because
> the egress policy pairs with §4.4's `net` scope and reaches §4.3's `fetch`, and
> because §14 owes tests for rules whose failure is silent.
>
> **Four scoping decisions taken while transcribing.**
>
> 1. **A4 had already built most of the capability model, one day earlier.** The
>    redirect asks for "capabilities expressed as **data** (`supports_tools`, caching
>    semantics, dialect/grammar choice)". Three of those four are already `ModelInfo`
>    fields — `supports_tools`, `supports_thinking`, `supports_cache_breakpoints` —
>    because A4's four-rate cost rule needed `ModelInfo` to be a real struct rather than
>    a trailing comment. So A5 **extends** a capability model rather than adding one, and
>    the only genuinely new capability is the dialect. This is the second delta in six
>    whose named dependency turned out to be already discharged (A4/B6 was the first),
>    and the pre-flight is what caught it: transcribing the redirect literally would have
>    re-added fields that were normative since yesterday.
> 2. **`ToolDialect` is a reserved slot, and in v1 it is *derived* rather than chosen.**
>    B5's subset makes the in-band grammar reachable **only when `supports_tools` is
>    false** — a model with native tool use needs no grammar at all — so with exactly one
>    grammar shipping there is no choice to make and the value follows from the
>    capability (with a config override for a provider that reports dishonestly). What the
>    redirect actually needs is the **slot**: `ToolDialect { Native, XmlGrammar }`, so
>    un-deferring B5's ten dialects adds variants rather than restructuring the engine.
>    That is the inert-stub pattern B7 and A22 both use, applied to a capability model,
>    and it is what makes "onboarding is cheap" a property of the types rather than an
>    intention.
> 3. **B5's instruction to add a numbered §16 decision was stale and was NOT followed.**
>    Both B5's banner ("§16 has no B5 entry today") and this entry's `Current.` paragraph
>    ask for one. But §16 is titled **Open decisions** and B5 is **resolved**, so adding
>    it would put a closed decision in the spec's open list — and **A19 set the precedent
>    explicitly**: "§16 gained no B15 row (it lists *open* decisions)". The concern behind
>    the instruction is legitimate and is answered differently: the spec assumed native
>    tool use everywhere with no record that dialects were *considered and deferred*, so
>    **the deferral now lives in §6's prose** where a reader of the provider section meets
>    it, and Appendix B keeps the decision. A12 owns §16's bookkeeping and applies last.
> 4. **`CompletionRequest` and `StreamChunk` were referenced by §6's trait and never
>    defined** — the same defect class A3 and A4 each hit, in the very section this delta
>    owns, and A5 cannot specify transport-copy escaping or stream holdback without saying
>    what the request and the chunk are. Both are now real types. `StreamChunk`'s variants
>    are deliberately the *provider-side* granularity and carry no cursor, because §3.2's
>    two tiers decide durability and §3.6 assigns a cursor at log append — a chunk is not
>    an envelope.
>
> **Debt this creates, recorded rather than left to the next reader.** **U10 stays open
> and A5 deliberately does not touch it** — malformed tool-argument JSON repair is a
> separate axis belonging to the tools, and this entry's own "explicitly out of scope"
> said so; §6 now records the exclusion where a reader will meet it. **Doc 13** is the
> provider plan doc and is still at summary depth, so B5's and A5's machinery has no
> plan-side write-up; it is the natural home for the dialect matrix if it is ever
> un-deferred. **The ten deferred dialects and per-request history re-encoding** remain
> deferred, and `ToolDialect` is where they land. **Doc 03** is now seven deltas out of
> date (M1.1), and **U100** gains two more types that are *defined* rather than
> comment-only, which narrows it slightly.

**Current.** §6 (`:479-494`) defines the `Provider` trait and lists v1
implementations as `anthropic` + `openai-compat`. §16 has **no** numbered
decision for in-band dialects — the spec currently assumes native tool use
everywhere.

**Proposed.**
- Add a **native `gemini`** provider (Google Generative AI / Vertex). All three
  TS harnesses carry it separately from openai-compat because tool-calling and
  caching semantics differ (oh-my-pi `google-generative-ai` / `-gemini-cli` /
  `-vertex`; openclaw `google/vertex`).
- Add the **B5 v1 subset**: Harmony **control-token escaping** on the transport
  copy (required for *native* tool use on gpt-5.x / gpt-oss, which reject
  reserved control-token spellings in data); a `supports_tools: false`
  capability flag driving **history demotion**; **one XML grammar** with stream
  holdback; a **tool-name allowlist** checked before promotion; and the rule
  that repair **never scrubs inside fenced or indented code regions**.
- Add **SSRF/egress `net-policy`**: IP-range deny + URL secret redaction for any
  provider or `fetch` egress, paired with §4.4's `net` scope (openclaw
  `packages/net-policy/src/ip.ts`).
- Add retry detail already implied by §6: backoff **with jitter** and
  **abort-aware sleep** (openclaw `packages/retry/src/index.ts:9`).

**Explicitly out of scope for this delta.** The ten remaining dialects and
per-request history re-encoding (deferred by B5), and **malformed tool-arg JSON
repair** — that is a separate axis (forge carries a distinct 3 694-LoC
`forge_json_repair` crate for it) belonging in
[14](../part3-capabilities/14-tools.md), not §6. Record it as a follow-up in
[TODO](../TODO.md) rather than folding it in here.

**Depends on.** ~~B5~~ ✅ **signed 2026-09-17**, and — as the pre-flight found — on
**A4** in a way nobody recorded: A4's four-rate cost rule made `ModelInfo` a real
struct carrying three of the four capabilities this redirect asks for, so A5 extends a
capability model rather than adding one. It also turned out to depend on **A3** for
`StopReason`, which `StreamChunk::Done` carries.

**Recommend accept**, scoped exactly as above. **Done 2026-09-18**, to four sections
rather than the one named here, with the four scoping decisions in the banner. **The
first proposed bullet (native `gemini`) was not transcribed**, as the redirect
required, and §6 now records *why* its absence is the stronger position rather than
leaving a reader to wonder whether it was forgotten. **The "Explicitly out of scope"
paragraph below was honoured rather than merely noted**: malformed tool-argument JSON
repair stays **U10**'s and belongs to the tools, and §6 says so where a reader meets
the dialect rules — so the exclusion is visible in the spec and not only in this
appendix.

---

### A6 — §7: the channel-plugin model — ✅ **SIGNED OFF 2026-09-17**

> **✅ SIGNED OFF 2026-09-17 — approved for application; NOT yet transcribed into the
> spec.** *"is okay"* — approved as proposed: the **plugin** owns account/security, DM
> pairing, session grammar and outbound formatting; **core owns the single shared `message`
> tool** and its dispatch, so no bridge grows its own send tool and the audit trail does not
> fragment. Canonical conversation resolution, editable draft-stream outbound and DM pairing
> codes stand as written. §7.2 still ships four bridges as independent binaries, and M4
> scope is unchanged.

**Current.** §7.2 (`:532-539`) ships bridges as separate binaries
(`dacode-ext-telegram`, `-discord`, `-slack`, `-desktop`), each independently
posting messages on `AgentIdle`/`QuestionAsked`/`PermissionRequested`.

**Proposed.** Adopt openclaw's **plugin/core split** (`docs/plugins/sdk-channel-plugins.md:19-33`,
`src/channels/AGENTS.md`), which runs ~50 channel plugins on one contract:
- The **plugin** owns account/security, DM pairing, session grammar, and
  outbound formatting/threading.
- **Core owns the single shared `message` tool and its dispatch.** Without this,
  every bridge grows its own send tool and the audit trail fragments.
- **Canonical conversation resolution**: `channel + account + thread → session`
  (`src/channels/conversation-resolution.ts`), with DM/group scoping.
- **Editable draft-stream outbound** (`src/channels/draft-stream-loop.ts`) —
  streamed replies edit one message rather than spamming many.
- **DM pairing codes** gating unknown senders (`README.md:75-80`) — a security
  control dacode's §7 currently lacks entirely.

**Depends on.** Nothing. M4 scope.

**Recommend accept.** dacode's §7 is the differentiator no reference harness
has end-to-end (oh-my-pi has **no** external messaging bridge at all), so getting
the model right early is cheap insurance.

---

### A7 — §8.2, §8.3, §8.4, §8.5: config-write endpoints, the scaling axis, and the one rule for every non-session write — ✅ **APPLIED 2026-09-19**

> **✅ APPLIED 2026-09-19.** Signed off 2026-09-17 (*"accept"* — as proposed, with both
> caveats, the post-A19 amendment and the multi-user-is-architecture-not-v1-scope
> statement, released by **B10** and **B11**). **Ten spec sections were edited against the
> two this entry named** — §8.2, §8.3, §8.4, §8.5, §3.6, §3.7, §5.1, §9.2, §12, §14 —
> because each of its three parts needed a section the entry did not list: caveat 1 needed
> §8.5's allowlist *and* §8.2's write rule, caveat 2 needed §5.1 (which is where §12
> already cited it), the amendment needed §8.4 and §9.2, and **U99**'s freshness channel
> needed a wire frame in §3.6 plus a field in §12.
>
> **Six scoping decisions.**
>
> 1. **The scaling axis landed in §8.2 rather than in a new section, and the reason is
>    that two sections already cited it there.** §2.2 (via A13) and §3.7 (via A14) both
>    forward-reference "§8.2's scaling axis" for storage and auth traits that §8.2 did not
>    contain — a **resolvable reference with false content**, which is the A21/`§14.2`
>    hazard class one level subtler, because `check_section_refs()` passes: the heading
>    exists. Creating a §8.6 would have fixed the content and left both citations pointing
>    at the wrong place, so §8.2 carries it (it is not purely a route list — the redaction
>    and content-disposition rules were already prose there) and **§8.6 stays free**, which
>    matters because doc 25 proposes exactly that number for introspection. Both
>    trait families are stated with their M1-only implementations, `BlobStore` is
>    **reconciled in as the third storage trait** (§2.1 named it, this entry did not), the
>    method lists are left to the plan, and the placement A13 recorded is cited rather than
>    restated. **`Authenticator`/`Tenant` are placed in `dacode-server` with no new crate**,
>    on the test §2.1 applies to the storage traits: no engine rule depends on identity,
>    because the engine sees a caller only as a `ClientId` and a controller status.
> 2. ⚠️ **This entry's own gate was not transcribed, because it is incoherent.** The
>    proposal says config writes are "gated on controller status (§8.4)". §5.1's four
>    layers are built-in defaults, user config, repo config and env/CLI — **none
>    session-scoped** — while §8.4's controller rule reaches only commands that *mutate a
>    session*. A14 had already ruled on exactly this for the registry ("not session-scoped,
>    so the controller rule does not reach it") and §8.4 said the rule "should be decided
>    once for every non-session write rather than per command"; A22 then assigned A7 that
>    single answer. So §8.4 now states **one rule for all three surfaces** — registry,
>    config, board — as instances of *a client changed something durable outside any
>    session*, with tightening named per principal kind. **The seam had to be corrected on
>    the way**: §8.4 said the rule "lands" on `Hello`'s reserved capability flags, but
>    **`Hello` is not a `Command`** (it is §7.2's extension handshake), so a flag there
>    cannot gate a browser and a tenant cannot gate a bridge. Both seams are now named and
>    required to express one policy.
> 3. **Two rules this entry does not contain were added, because the surface is unsafe
>    without them and both are the "too permissive" class no gate can catch.**
>    **(a) The writable root is derived server-side and never supplied.** One server
>    legitimately hosts sessions in several repositories (`CreateSession` carries a `cwd`,
>    §4.7 maps worktrees), so "`<repo>/.dacode/`" has no single referent: a client-named
>    root would let a browser working in one repository write another's config, and an
>    ambiguous root would make §8.5's allowlist decorative. **(b) A write may not add an
>    exec surface** — `[[extensions]]`' `cmd` (§7.2) and a hook's `run`/`post` (§7.1) are
>    refused *although the file holding them is inside the allowlist*, because at those keys
>    editing configuration **is** executing code, as the server's user, at every later
>    start, from a file that outlives the request. The composition is A22's shape again:
>    §9.2 renders model-authored markdown on the authenticated origin, so a script reaching
>    that origin could turn one transcript into persistent command execution. It costs the
>    *operator* nothing — `dacode config edit` in a terminal still reaches those keys — and
>    constrains only a client acting on their behalf, which is the distinction the user
>    settled A22's item 2 on.
> 4. **Writes are conditional: `ETag` on every read, `If-Match` required on every write,
>    `412` on mismatch.** Four writers can reach one file — a browser, `dacode config edit`,
>    the user's own editor, and for a board §4.8's promotion — and last-writer-wins between
>    them is silent data loss. "A human reviews the diff" is not a reconciliation mechanism
>    for a clobber: the diff shows the result, not the bytes that were overwritten.
> 5. **The board route is repo-scoped, following the file.** §12 puts boards at
>    `<repo>/.dacode/plans/<board>.md`, so `GET /api/plans` and `GET`+`PUT
>    /api/plans/:board` resolve against the **server's** cwd and return `409` outside a
>    repository rather than writing into whatever directory happens to be current. The
>    "plausibly a project-level artifact" reading survives as **presentation** (§9.2 groups
>    boards) and not as a second routing axis. **`promote` is two checks**: controller-gated
>    at the session owning the item, allowlist- and `If-Match`-gated at the path — and from
>    the model a promotion is a **write through §4.4** (§4.8's `Verify::Command` rule is the
>    precedent), while from a browser it is not a tool call at all, so neither door inherits
>    the other's check. **No `POST`**: the client names the file, so create and replace are
>    one idempotent operation — §3.3's caller-allocated-id reasoning applied to HTTP.
> 6. **U99 landed, and its socket half is a compatibility event.** `org_version` on all
>    four tree reads (§12's monotonic `version`, stored rather than derived from mtime
>    because mtime is not monotonic across a rename-on-write, a restore, or two hosts
>    sharing a home directory) plus **§3.6's third down-frame kind, `Hint`**. It is not an
>    `Envelope` and gets **no golden fixture**, because §14.1's fixture set derives from
>    `Event`'s variants: no `session`, no `seq`, no cursor, never written, never replayed,
>    and safe to lose, since the authoritative version travels on the read it prompts. That
>    is exactly the line §3.7 draws, and it is why the registry does not become an event
>    stream here.
>
> **One gap recorded rather than filled.** §3.3 has **no task-mutation `Command`**, so the
> client→engine path a browser would use to `promote` — or to close a `Verify::Manual` item,
> which §4.8 says requires a controller action recorded as its own event — does not exist in
> the protocol; `promote` is a `TaskOp` on the `TaskUpdated` **event** only. Inventing a
> variant would have been a wire change in a section this entry does not own, for a shape the
> task-tree brainstorm holds, so it is a new §6 item instead.
>
> **Debt this creates.** **B10 is now half-applied in A25's shape**: its comment-preserving
> writes are normative at §5.1 (where §12 already cited them and no section carried them),
> while its generated `dacode.schema.json` stays owed to A12/doc 17 — so B10 must not be
> marked applied. **Doc 20** is the scaling axis's plan-side home and is still a summary;
> **docs 19, 21, 17, 18 and 32** each carried a line A7 falsified and were corrected under
> hard rule 1 with their markers untouched. **`xtask`** now has a third thing to specify if
> the config schema is generated (B6's tool schema, B2's protocol schema, B10's config
> schema), and doc 01 §1.5's register row owes that. **`toml_edit`** is pure Rust, so it
> needs no Appendix F row — checked rather than assumed.

**Current.** *(historical — the line numbers are from the pre-A7 spec)* §8.2
(`:565-584`) exposes `GET /api/config`, `/api/agents`, `/api/skills`,
`/api/models` — read-only, **16 routes in total**. §8.3 (`:589-596`) is
single-user and states plainly: "No users, no roles." **After application §8.2
carries 23 routes and the scaling axis, and §8.3 carries the trait pair.**

**Proposed.**
- **Config-write endpoints** for UI-driven agent/skill/workflow customization
  (`PUT`/`POST` over `/api/{config,agents,skills,workflows}`), ~~gated on
  controller status (§8.4)~~ → ⚠️ **this gate was NOT transcribed; see scoping
  decision 2.** It is incoherent as written, and A14's pattern was applied
  instead. **`PUT`/`DELETE` landed rather than `PUT`/`POST`** — the client names
  the file, so create and replace are one idempotent operation.
- The **scaling axis** from [00-overview §0.3](../00-overview.md): `EventStore`/
  `SessionStore` and `Authenticator`/`Tenant` as **pluggable traits from day
  one**, with `JsonlStore` + `SharedToken` as the only M1 implementations and
  `SqliteStore`/`PostgresStore`/`MultiUser` later. → ✅ **landed at §8.2, with
  `BlobStore` reconciled in as the third storage trait** — §2.1 named three and
  this entry named two, so transcribing it literally would have left the spec
  disagreeing with itself about how many storage traits exist.

**Two caveats that must be written into the delta, not left implicit.** ✅ **Both
discharged 2026-09-19, and each needed a section this entry did not name.**
1. **Config-write is a new write surface outside the session cwd.** §8.5's path
   safety canonicalizes and rejects anything outside the session's
   cwd/worktree — but these endpoints write to `~/.config/dacode/` and
   `<repo>/.dacode/`. §8.5 must be extended with an explicit allowlist of
   writable config paths, or the endpoints bypass the only path guard the spec
   has. → ✅ **§8.5 is now split Reads/Writes**, with a three-class allowlist,
   five exclusions and the three read-side properties carried over. **What this
   caveat could not see is that "`<repo>`" has no single referent** on a server
   hosting sessions in several repositories, so the allowlist alone does not
   close the hole — the root has to be *derived* (scoping decision 3a).
2. **UI-driven writes make B10's comment-preservation load-bearing.** A CLI
   `dacode config edit` eating comments is annoying; a web UI silently
   destroying a user's annotated `config.toml` is a data-loss bug. Format-
   preserving writes (`toml_edit`) become a requirement of *this* delta, not
   just of B10. → ✅ **landed at §5.1**, which is where §12 already cited it and
   no section carried it: before this pass the requirement existed in the spec
   only as a *citation*. **B10's other half (the generated `dacode.schema.json`)
   stays owed**, so B10 is now half-applied rather than done.

**Scope discipline to state explicitly:** multi-user is **architecture, not v1
scope**. The traits ship in M1; `MultiUser` and `PostgresStore` do not. §8.3's
"No users, no roles" stays true of the shipped default and should be reworded
as "no users or roles *in the default single-user configuration*" rather than
deleted. → ✅ **reworded exactly as instructed**, with the addition that it now
names what the design *does* permit (§8.2's scaling axis), so the sentence cannot
be read later as a prohibition. The two other "§8.3 has no users and no roles"
claims in §8.4 were swept to match.

**Depends on.** ~~B10, B11.~~ ✅ **both signed 2026-09-17**; B11's crate
placement had already been transcribed by **A13** (2026-09-18), which is why this
entry cites §2.1 rather than restating it — and why A13's §2.2 sentence about "the
scaling axis §8.2's storage and auth traits describe" was a real obligation rather
than prose.

**Recommend accept**, with both caveats and the scope statement. **Done
2026-09-19**, with the six scoping decisions in the banner. **The `Current.` and
`Proposed.` text above is historical**: §8.2 was read-only and carried 16 routes,
§8.3 said "No users, no roles" flatly, and §8.5 was two sentences about reads.

**Amended after A19 landed — caveat 1 gains a third path, and a new surface
arrives with it.** §4.8's promoted task board lives at `<repo>/.dacode/plans/<board>.md`
(§12), which is exactly the class of path caveat 1 describes: a write surface
outside the session cwd. So:

✅ **All four bullets discharged 2026-09-19.**

- **`.dacode/plans/` must be on §8.5's writable-path allowlist** if any UI is to
  save a board. Without it the endpoints cannot write there, or they bypass the
  only path guard the spec has. → ✅ **the third of the three writable classes**,
  and §9.2's config editor is where a user reaches it.
- **A board is cross-session, which this delta's §8.2 framing does not cover.**
  Its endpoints are scoped `/api/sessions/:id/…`; a board aggregates items promoted
  from *several* sessions, so it needs either a project-scoped route (which makes
  it a §6/B13 concern) or a repo-scoped one. ~~Unspecified today, and recorded in
  [32](../part2-engine/32-task-tracking.md) and [TODO](../TODO.md) §6 rather than
  left for M3.~~ → ✅ **repo-scoped, following the file**: §12 puts a board at
  `<repo>/.dacode/plans/<board>.md`, so `GET /api/plans` and `GET`+`PUT
  /api/plans/:board` resolve against the **server's** cwd and `409` outside a
  repository. The project-level reading survives as **presentation** (§9.2 groups
  boards) and not as a second routing axis — which is the answer
  [32](../part2-engine/32-task-tracking.md) and [TODO](../TODO.md) §6 were holding
  the question open for.
- **`promote` from a browser is a controller action.** Writing a *committed* file
  from a UI is a different trust level from a model writing it, so §8.4's
  controller/observer split has to say who may. → ✅ **answered as two checks
  rather than one**, because the act is session-scoped while its effect is not:
  the controller of the session owning the item may promote, *and* the write is
  bounded by §8.5's allowlist and §8.2's `If-Match`. **The asymmetry this bullet
  did not state is now in §8.4**: from the model a promotion is a write through
  §4.4 (§4.8's `Verify::Command` rule is the precedent), while from a browser it is
  not a tool call at all — so neither door inherits the other's check. ⚠️ **And the
  browser door does not exist in the protocol yet**: §3.3 has no task-mutation
  `Command`, recorded as a new §6 item rather than invented here.
- **The live tree needs none of this.** It is folded into `Snapshot.tasks` (§3.5),
  so every client already renders it over the existing channel — worth stating so
  the board question is not read as larger than it is. → ✅ **and it stayed true**:
  the board work added one file route family and no `Event` variant, so the tree's
  rendering path is untouched.

---

### A8 — §9.1: the web stack, as a deferral with a committed prerequisite — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18.** B2's split transcribed, and **the framework was not
> picked** — which is the whole discipline of this delta, since the split *is* the
> decision. §9.1 now says the framework is an **open decision with a deadline** (due by
> **M3**) rather than "Decision pending", with **TS SPA (Solid)** as the *standing
> recommendation* and both reasons recorded so the choice is not relitigated from
> scratch: all three reference web UIs chose TypeScript, and the four hard parts of this
> UI (streaming markdown, highlighting, virtualised lists, diff rendering) have mature TS
> solutions and weak WASM ones. A recommendation is not a decision, so §9.2's screens and
> §9.3's behaviour stay framework-neutral and the server is unaffected either way. The
> Leptos/TS/HTMX table is kept as the record of what was weighed.
>
> **Three scoping decisions taken while transcribing.**
>
> 1. **The JSON-Schema export is now unconditional, and the old conditionality was
>    backwards.** B2 made it a non-optional **M1** deliverable, but §9.1 carried it *only*
>    inside the "If TS:" branch — so an M1 obligation was conditional on an M3 decision.
>    §9.1 now states the inversion explicitly: **the export is what makes the TS branch
>    cheap**, so requiring the branch first would have made the framework decision harder
>    rather than easier. Its framework-independent uses are listed (extension authors
>    validating a `Command`, a future SDK, `run --output jsonl` consumers, and §14.1's
>    golden fixtures, which already pin the wire format in JSON and so are half a schema).
> 2. ⚠️ **A live contradiction between §9.1 and §13.3/§2.4 was resolved, and it was not a
>    choice.** §9.1's old text had "a `build.rs` step emits `protocol.d.ts` from the Rust
>    types (via a small JSON-Schema export **in `dacode-protocol`**)" — but §13.3 confines
>    `schemars` to an offline generator because it depends on `serde_json`
>    non-optionally and says **"never in a shipped crate"**, and §2.4 puts build-time
>    codegen in **`xtask`** on the test of whether its output ships. So the generator lives
>    in `xtask`, `dacode-protocol` neither depends on `schemars` nor grows a `build.rs`,
>    and `protocol.d.ts` is generated **in CI with a drift check that fails the build** —
>    layered on the golden fixtures, which catch a change to the serialized shape while the
>    drift check catches the generated types falling out of step with it (the failure a
>    second language cannot otherwise see). §9.1 names this as §14.1's existing `xtask`
>    rule applied to a second generator rather than as a new rule.
> 3. **Doc 03's line was a false claim, not a stale marker, so it was corrected under hard
>    rule 1 rather than left.** It read "Optional: a JSON-Schema export … **if** the web UI
>    is a TS SPA" — the exact conditionality this delta removes, in the plan doc for the
>    crate that owes the export. Strike-and-annotate, **marker and depth untouched**:
>    removing one false line from a summary adds no depth. Same argument that justified
>    correcting doc 08's `Dynamic(fn)` arm under A2.
>
> **Debt this creates.** **The framework decision is due by M3** and §9.1 names the
> deadline, so the deferral cannot drift; **doc 21** is the web-UI plan doc and is still a
> summary, so §9.1/§9.2/§9.3 are its normative source. **`xtask`** now has two generators
> to specify (the tool-schema one from §13.3 and this one), and doc 01 §1.5's register row
> should say so when M1 deepens it. **No wire change, no `Event` variant, no fixture** —
> the export is an artifact *derived from* the fixtures, not a new surface.

**Current.** §9.1 (`:619-631`) gives the Leptos/TS/HTMX table, says **"Decision
pending."**, and notes that either way the server is unaffected.

**Proposed (per B2's split).** Do **not** resolve the framework. Instead record:
- `dacode-protocol` **ships a JSON-Schema export of its own types as an M1
  deliverable** — non-optional, and useful regardless of framework (extension
  authors, a future SDK, `run --output jsonl` consumers, golden fixtures).
  [03](../part1-foundations/03-protocol.md) currently calls this "Optional … if
  the web UI is a TS SPA"; that conditionality is removed.
- The **framework decision is due by M3**, with **TS SPA (Solid)** as the
  standing recommendation and the reason recorded (all three reference web UIs
  chose TS; streaming markdown, highlighting, virtualization and diffs all have
  mature TS solutions and weak WASM ones).
- If TS: `protocol.d.ts` is **generated in CI with a drift check that fails the
  build**, layered on the golden fixtures that already pin the wire format.

**Depends on.** ~~B2~~ ✅ **signed 2026-09-17**.

**Recommend accept as amended** — this delta records a *deferral with a
committed prerequisite*, which is a real resolution, not a punt. **Done
2026-09-18**, with the three scoping decisions in the banner and the framework left
open exactly as instructed. **The "Current." text below is historical** — §9.1 no
longer says "Decision pending", and its `build.rs`-in-`dacode-protocol` clause was
contradicted by §13.3 and §2.4 and is gone. **The prerequisite half of "a deferral
with a committed prerequisite" is what needed the care**: the prerequisite was
sitting *inside* the conditional branch it was carved out of, so it had inherited
the very conditionality the split removed. That is the fourth time a signed
decision's M1 half was found inside a conditional (A25's wire half, A10's
carve-out, A21's `§14.2` citations), which makes it a pattern worth checking for
rather than a run of bad luck.

---

### A9 — §9.2, §10: one tree component, TUI mouse, collapse-on-paste — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, and with it **M1's delta *list* is complete** — of
> `A1–A5, A9, A14, A15` every one has landed. ⚠️ **That claim was true of the list and
> false of the milestone, and the correction belongs here because this is the entry that
> made it**: **B11's signed condition also made A10's exchange-record format an M1
> deliverable**, A10's entry assigned its §6 side to **A5**, and A5 had landed without it.
> No gate could see that — A5's own `Depends on` is correct, and a complete list looks
> complete — so it took an **inbound-assignment sweep** (grep both appendices for what
> *other* deltas assign to each applied one) to find it. The record is now landed and
> **A10 is half-applied** in A25's shape. It was taken **out of queue order** for
> that reason: the application order puts A6 next, but A6 is §5's **M4** dependency
> while A9 is M1's last, and the queue and the milestone lists are different axes. Its
> own **sequencing constraint is satisfied** — A3 and A4 both landed earlier the same
> day, as its sign-off required.
>
> **Four scoping decisions taken while transcribing.**
>
> 1. **§9.2's half was already normative, so A9 extends it rather than adding it.**
>    "One tree component, three sources" landed with **A19** on 2026-09-13, verbatim in
>    shape: the task tree, the sub-agent tree and declarative workflow steps render in one
>    component because they are one shape. What A9 contributes is §10's **matching**
>    component — the TUI sidebar now names the same three sources, which it previously did
>    not, having listed sessions/sub-agents and the task tree as two bullets — plus the two
>    TUI rules. This is the third delta in eight whose named work turned out to be partly
>    done (A4/B6 and A5/A4 were the others), and as with those the pre-flight is what
>    caught it: transcribing the proposal literally would have duplicated a bullet that has
>    been normative for five days.
> 2. **The mouse rule landed with its *reason*, not just its condition.** The signed text
>    says "SGR tracking enabled only while a fullscreen overlay holds the alt screen",
>    which is a mechanism. §10 now also says why the condition is the point: a TUI that
>    captures the mouse all the time takes **scroll-back and text selection** away from the
>    user's own terminal, the two things a terminal does better than any UI. Without the
>    reason the rule reads as an implementation convenience and the next person enables
>    mouse support globally.
> 3. **Collapse-on-paste is recorded as a §10 rule and *not* shared client behaviour, and
>    that is substantive.** In a browser a large paste into a textarea is the browser's
>    problem and already works; in a terminal a big bracketed paste can be misread as a
>    burst of keystrokes. So §9.3 correctly grows no paste or mouse rule, and §10 says so
>    explicitly — otherwise the next reader generalises a terminal workaround into the web
>    UI. The signed parameters landed unchanged: 10 lines / 1000 chars, an **indivisible**
>    chip backed by a paste map, expanded on submit, a host hook to intercept
>    (wrap-in-codeblock / attach-file), bracketed-paste buffering under a size cap.
>    **Indivisible** is stated with its reason: a chip the cursor can half-edit is worse than
>    either alternative, because it looks like text and is not.
> 4. ⚠️ **A gap was found and deliberately recorded rather than filled.** §9.2's bullet
>    promises three sources in one tree, but **nothing on the wire correlates a declarative
>    workflow step with the child session it spawned**: `SubAgentSpawned` carries the
>    child, the agent *name* and a task *description*, not a step identity. The engine
>    knows the correlation — §4.2's `task` tool carries the parent task-item id, which is
>    what scopes the child's write access — but does not say so. The minimal fix is a wire
>    change plus a `TaskId` identifier (currently undefined; it is in **U100**'s list) plus
>    a §4.3 scan-exclusion clause, since `TaskId` *is* tool-nameable through `todo` and so
>    cannot silently join the scanned set. That is three sections A9 does not name, for a
>    correlator whose right shape depends on §5.4's step model — **which A25's M5 half owns
>    and has not designed**. Inventing it here would have been exactly the error avoided
>    for U96's hand-over. So §9.2 states the limitation openly, names it as an *edge*
>    problem rather than a *shape* problem, and leaves the correlator to the §5.4 step
>    model. **Neither signed text requires it**: A19's bullet and this entry's proposal
>    both claim the sources are the same *shape*, and shape is a property of the node, not
>    of the linkage.
>
> **Debt this creates, recorded rather than left to the next reader.** **U101 (new)**:
> §14 has **no UI test tier at all**, so A9's two client rules are normative with nothing
> that can fail on them — a mouse rule whose violation silently costs the user their
> terminal's scroll-back, and two paste thresholds that are numbers and will drift. Adding
> a tier means choosing a harness for a crate that does not exist yet, which is **A21**'s
> §14 restructuring, so it is recorded with A21 (or M1.1) as owner and A9's two rules
> named as the tier's first content. **U19** is annotated: the mouse and paste specifics
> are now normative in §10, so when doc 22 is deepened its source is the spec rather than
> the survey — and doc 22 stays a summary, now three deltas out of date (M1.1). **Doc 21**
> (web UI) is likewise still a summary and §9.2 is its normative source. **The step-linkage
> correlator** belongs to A25's §5.4 half.

**Current.** §9.2 (`:649-650`) renders a sub-agent tree from
`SubAgentSpawned`. §10 (`:671-689`) lists diff viewer, session tree, `/serve`,
`attach` — no mouse, no paste handling.

**Proposed.**
- **Unify the workflow/pipeline view with the sub-agent tree** (§9.2 and §10):
  declarative workflow steps (§5.4) and spawned sub-agents are the same shape —
  a node with live status that expands to a transcript. One component, not two.
- **TUI mouse support** (§10): SGR tracking (`?1000h ?1003h ?1006h`) enabled
  only while a fullscreen overlay holds the alt screen, with 0-based row/col
  hit-testing (oh-my-pi `tui.ts:88,:2775`, `mouse.ts:39,:85,:107`). Directly
  portable to crossterm.
- **Collapse-on-paste** (§10): a paste over **10 lines or 1000 chars** collapses
  to an indivisible chip backed by a paste map, expanded on submit, with a host
  hook to intercept (wrap-in-codeblock / attach-file) (oh-my-pi
  `components/editor.ts:598,:633,:885`). Bracketed-paste buffering with a size
  cap (64 MiB there).

**Depends on.** Nothing — but the sign-off attached a **sequencing constraint** ("A3
and A4 must not land later than A9, since A9 unifies the tree views A19's task tree
joins"), and ✅ **it held**: A3 and A4 both landed earlier on 2026-09-18. It also
turned out to depend on **A19** in a way nobody recorded — §9.2's "one tree
component, three sources" bullet was already normative, so half of this delta's first
proposal was done five days before it was applied.

**Recommend accept.** Both TUI items come with concrete, tested parameters
rather than aspirations, and both are cheap in ratatui/crossterm. **Done
2026-09-18**, with the four scoping decisions in the banner — including one gap
**deliberately not filled** (nothing on the wire correlates a workflow step to its
spawned child, and the correlator's shape depends on a §5.4 step model A25 has not
designed). **The "Current." and "Proposed." text below is historical**: §9.2 no longer
"renders a sub-agent tree" alone, and §10 now carries mouse and paste rules it did not
have. The reference-harness line citations are the survey's, not the spec's, and stay
here rather than moving into it.

---

### A10 — new §17 (M5) + §6/§12/§14 (M1 carve-out): API introspection & caching analyzer — ⚠️ **HALF-APPLIED 2026-09-18**

> **⚠️ HALF-APPLIED 2026-09-18, in A25's shape — and the half that landed is the one
> that was nearly missed, which is the interesting part.** The **M1 carve-out is
> APPLIED**: `ExchangeRecord` is normative in §6, `exchanges.jsonl` is in §12's layout
> with the redaction asymmetry stated against `events.jsonl`, and §14 has the test that
> can fail on getting that asymmetry wrong in either direction. **The §17 body is still
> owed** — the transparent logging proxy, SSE reassembly, the cache-failure taxonomy,
> and the standalone mode that does not involve `dacode-server` at all. That is M5 and
> stays in the queue.
>
> **How the M1 half was missed, because the mechanism matters more than the miss.** This
> entry assigned the §6 side to **A5** — twice, in the banner ("A5 carries the §6 side
> of it") and in the proposal ("A5 should carry the §6 side"). A5 was applied without
> it. Nothing caught that: A5's own `Depends on` cell says "B5" and is correct, its
> gates were green, and the board's claim "**M1's delta list is complete**" was *true of
> §5's list* (`A1–A5, A9, A14, A15` all applied) while being **false in substance**,
> because B11's signed condition — *"if the provider layer does not emit it from the
> start, retrofitting instrumentation across every provider in M5 is far more expensive
> than emitting it in M1"* — was unmet. What found it was an **inbound-assignment
> sweep**: for each applied delta, grep both appendices for other entries assigning work
> *to* it. Eight deltas, one unfulfilled assignment, one line of shell.
>
> **The generalisation is the reason this banner is long.** Every delta's pre-flight
> checked what it *depended on* and never what *depended on it*. A list of deltas is
> what its author remembered; cross-delta obligations are precisely what lists do not
> carry, and a complete list looks complete — which is what makes this failure invisible
> in a way a wrong milestone attribution (recorded separately) is not. The sweep is now
> part of the standing gate set beside the type-definition sweep and the two-form depth
> grep.
>
> **Two scoping decisions on the record itself.** Its fields map **one-to-one onto
> B11's signed list** — endpoint kind, request/response bodies, extracted `usage`
> including cache-token fields, timings, redaction applied — and no further, since
> inventing fields is the error the type sweep exists to catch. **One field beyond that
> list is `turn`**, and it earns its place: the record's purpose is to check §4.5's
> invariants against what actually happened in a turn, which needs the exchange and the
> event log to be joinable. The session id is *not* a field, because the record lives in
> that session's directory and a field would duplicate the path. Two deliberate absences:
> **`api_kind` is a string rather than an enum**, because the set is open and an enum
> would need a variant per dialect — the per-provider branching A5's redirect exists to
> remove — and **there is no cache-hit field**, because `usage.cache_read_input_tokens`
> *is* the hit signal and a second field would be derivable state that eventually
> disagrees with its source.
>
> **The redaction asymmetry is the part most likely to be got backwards, so it is stated
> in three places.** `events.jsonl` is append-only and is **never** redacted — §4.3 makes
> prevention the only control available for a stored log, since redacting it would be a
> rewrite. `exchanges.jsonl` is a **derived diagnostic copy** and **is** redacted, under
> §8.2's rule for copies. A log that is redacted is a rewritten log; a diagnostic copy
> that is not redacted is a leak of the entire conversation. §6 states the rule and its
> two failure directions, §12 states it where the two files sit side by side (because a
> file that looks like a log invites log treatment), and §14 tests both directions.
>
> **Debt this creates.** **Doc 13** is the provider plan doc B11 named for the record
> shape and is still a summary, so its deepening at M1.1 should take §6 as its source
> rather than restating it. **§17 remains owed** and its content is unchanged by this
> pass. **B18's T1/T2 and A21's §14.2** gain their instrument: the exchange record is
> what a cache-hit-rate metric is computed *from*, so until now §14.2 would have had
> nothing to read.

**Current.** Absent from the spec entirely.
[25](../part5-outreach/25-introspection.md) proposes "§8.6 or a new §17".

**Proposed.** Add a new section — **§17 "Debugging & introspection"**, not §8.6.
Rationale: the subsystem has two modes, and one of them (a standalone logging
proxy that *any* CLI can point at via `ANTHROPIC_BASE_URL`) does not involve
`dacode-server` at all. Burying it under §8 Server would misdescribe it. §16 is
"Open decisions" and will shrink as decisions land, so appending §17 after it is
not an ordering problem.

Content: the transparent logging proxy (bind local ports → upstream, forward
verbatim, tag endpoints with an `api_kind`); **SSE reassembly** into a final
message object while forwarding chunks with no added latency; **secret
redaction** in logs but not upstream; **usage extraction** including
`cache_read_input_tokens` / `cache_creation_input_tokens`; one JSON line per
exchange; the **cache-failure taxonomy** (missing/too-small `cache_control`
markers, measured hit-rate, volatile system prompts by first-changed-bytes
classification, tool-definition churn *including pure reordering*, non-append-only
history transitions, oversized or byte-identical repeated tool results,
cache-TTL gaps, mid-session model switches); and **in-process record emission**
from dacode's own provider layer so a session is self-introspecting without an
external proxy.

**The part that cannot wait for M5.** Per B11, the *crate* is M5, but the
**exchange-record format is an M1 deliverable** in §6 — retrofitting
instrumentation across every provider later costs far more than emitting it from
the start. This delta should say so explicitly, and A5 should carry the §6 side.

**Depends on.** B11.

**Recommend accept**, as a new §17 with the M1 record-format carve-out.

---

### A11 — §13: reconcile the JSON crate's reality → ✅ **APPLIED via A16, 2026-09-12**

> **✅ Superseded and applied as part of A16, 2026-09-12** — exactly as this
> entry recommended. Retained below as the record of *how* the discrepancy was
> found (tracker finding `f1`) and of the rename's measured blast radius. The
> `Value`-does-not-exist half is now resolved: §13.1 states it, and no dacode
> doc outside these appendices names `dacode_json::Value` any more.

**Current.** §13 (`:746-760`) names a type **`dacode_json::Value`** that does not
exist. The crate is published as **`dacode-json`** — correctly named since the
rename sweep — and has **no owned value type**: `Value<'a>`/`Doc<'a>` are
borrowed from the input slice plus a `Pool`.

**Half of this finding is now resolved.** A11 originally covered two defects: the
spec's crate name (`dacodec-json`, a crate that never existed) and its `Value`
type. The **name half was fixed by the `dacodec` → `dacode-json` rename**, which
swept the spec, the role files, the appendices and the generated adapters
together. The **`Value` half stands unchanged** — a rename cannot make a type
exist — and is the reason A16 is still needed.

**Recommend: do not apply separately.** A16 rewrites §13 wholesale and
subsumes this. A11 stays in the index as the record of *how the discrepancy was
found* (tracker finding `f1`), struck as superseded.

**Blast radius — measured before the sweep, kept as the audit trail.** Counting
only the two *wrong forms* (`dacodec-json` and `dacodec_json`), they appeared
**37 times across 17 files**, not just in the spec:

| Where | Occurrences |
|---|---|
| `docs/dacode-design.md` | 8 |
| `.agents/skills/dacode/SKILL.md` | 4 |
| `.agents/config/agent-rules.yaml` | 3 |
| `.opencode/agent/*.md` (**generated**) | 9 across 4 files |
| `AGENTS.md` | 2 |
| `.agents/roles/_rust.md`, `.agents/implement.md`, `.agents/reviewer.md`, `.agents/roles/implementer.md`, `.agents/skill-base.md`, `.agents/task-templates/write-tests.md` | 7 |

That table undercounts what the sweep actually touched, because it excludes the
*bare* `dacodec` and the `dacodec::` Rust paths: the full figure was **145
occurrences across 31 files**. All of it has landed, including the
`tools/agents-sync.py` regeneration of the `.opencode` adapters.

**What the sweep could not fix.** `.agents/roles/_rust.md`'s rule "All
protocol/tool/provider JSON goes through `dacode_json::Value`" still names a type
that does not exist, and that file is loaded into **every coding agent's
context**. Renaming the crate made the sentence accurate about the crate and left
it false about the type, so it now carries an inline caveat pointing at B1 and
A16. Applying A16 is what removes the need for the caveat — which remains a
stronger argument for accepting A16 promptly than spec hygiene alone.

---

### A12 — §16: resolve the open decisions — ✅ **SIGNED OFF 2026-09-17**

> **✅ SIGNED OFF 2026-09-17 — approved for application; NOT yet transcribed into the spec,
> and it APPLIES LAST, after every other delta.** *"ok"* — approved as proposed: resolve
> §16.1–§16.5, add native GUI nodes (thin gateway nodes, not engine forks) and ACP, keep §16
> listing what remains open. **Its scope grew today:** besides those, it now also carries the
> resolution of **§16 item 6** — the standing-goal gap A19 created — which the user signed as
> a **`Standing` status, goals only**. A task cannot be standing, because applied to a task it
> would be a way to dodge the gate; that is exactly how `tools/tasks.py --status standing`
> already behaves and refuses. **This is a wire-format change** (a `TaskStatus` variant),
> hence a compatibility event under §14, and it settles **with A1** rather than separately.

**Current.** §16 (`:801-810`) lists five open decisions: web stack, TOML vs
JSON, `bash:readonly` sandboxing, ACP, checkpoint strategy.

**Proposed** — record the B-pass resolutions:
- **§16.1 web stack** → *partially resolved*: schema export committed for M1,
  framework due by M3 (B2). **Stays open** in reduced form.
- **§16.2 TOML vs JSON** → *resolved*: TOML on disk, JSON on wire, plus
  comment-preserving writes and a generated `dacode.schema.json` (B10).
- **§16.3 `bash:readonly` sandboxing** → *resolved*: deny-list in v1, documented
  as advisory, `SandboxBackend` seam reserved, interpreters disqualifying, real
  containment via §4.7 worktrees (B7).
- **§16.4 ACP** → *resolved as deferred*: implement in M5; two M1 constraints
  recorded (log-derivable state; no client-capability branching in tools) (B9).
- **§16.5 checkpoint strategy** → *resolved*: shadow git, with the six edge-case
  additions and a pruning policy (B8).
- **Add** native GUI nodes (macOS Swift / Linux Tauri as **thin gateway nodes**,
  not engine forks) as a recorded M6 direction — openclaw's model, per
  [27](../part5-outreach/27-native-gui.md).
- **Add** the two decisions the plan surfaced that §16 does not currently list:
  **in-band tool dialects** (B5, now resolved as phased) and the **templating
  choice** for prompt partials and workflow `when` conditions
  ([17](../part3-capabilities/17-config-agents-skills.md) — forge uses
  handlebars compiled at build time; still genuinely open).

**Do not empty §16.** It should keep listing what remains open — after this pass
that is the web framework (B2's second half) and templating. An empty §16 would
falsely imply the design is settled.

**Depends on.** B2, B7, B8, B9, B10 — apply **last**.

**Recommend accept.**

---

### A13 — §2.1, §2.2: the two approved-with-dates crates, and in-tree vendoring — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18.** Both crates recorded as **approved-with-dates** — M3 and M5
> — and **neither was added to §2.1's tree**, which still lists eleven members, exactly as
> this entry required. The vendored-in-tree settlement is now explicit in §2.1 rather than
> implied by the tree.
>
> **Three scoping decisions taken while transcribing.**
>
> 1. **The instruction to "update Appendix D to mark both deferred" was already
>    satisfied, and Appendix D needed something else instead.** Its graph already carried
>    both crates with an asterisk and a deferral note, so what was actually stale was the
>    *legend* — it called them "**proposed** new crates" when B11 had approved them with
>    dates — and it carried **no milestone at all**. So the legend now says
>    approved-with-dates, names M3 and M5, and records the reasons (the trait lives in
>    `dacode-core::store` from M1 so the boundary arrives when a second implementor makes
>    it pay; the introspect *record format* is already normative in §6 via A10's carve-out,
>    so the crate arrives to read records written since M1). **This is the fifth delta in a
>    row whose named work was partly done before it started** — A4/B6, A5/A4, A9/A19,
>    A21/A4, A8/B2-and-doc-03 — which is what eleven transcriptions in one day does to a
>    queue: each entry's `Depends on` and instruction list describes the world as it was
>    when drafted.
> 2. ⚠️ **A phantom-edge hazard was found and written into the spec rather than left to
>    the reader.** `gates/boundary-edges.txt` declares itself an **exact** diff against
>    `cargo tree`, not a subset test — "a violating edge fails, and so does a legal new
>    one, until this file is updated in the same commit". A13 asks §2.2 to state where
>    `dacode-store` sits *when it arrives*, so the spec now names dependency edges for
>    crates that do not exist. **Those edges must not be added to the pinned list**, and
>    §2.2 says so in the same breath as the rules, with the reason: adding an edge for a
>    crate with no directory does not describe the graph, it breaks the gate — and it
>    *looks* like diligence to someone reading §2.2 and the gate file side by side. The
>    file was verified unchanged at 29 lines with zero store/introspect edges.
> 3. **§2.2's enforcement sentence was stale and was corrected.** It said the rules are
>    "Enforced by review; consider a `cargo-deny`/workspace-lint" — but the pinned
>    exact-diff gate **already exists**, so the spec was describing as aspirational the
>    mechanism that enforces it. Rewritten to name the mechanism and its same-commit rule,
>    which is §2.3's rule applied to the crate graph.
>
> **One coupling named rather than discovered later.** A13's placements must agree with
> **A7's** scaling axis (`EventStore`/`SessionStore` and `Authenticator`/`Tenant` as
> pluggable traits from day one, `JsonlStore` and `SharedToken` the only M1
> implementations). A7 is still unapplied, so §2.2 records the requirement that the two
> agree — pluggable backends from day one, crate boundary when a second implementor makes
> it pay for itself — instead of leaving M3 to find a contradiction.
>
> **No wire change, no fixture, no new type, and no gate-file change** — which is what
> makes this the cheapest delta in the set: it records two decisions and corrects one stale
> sentence about how they are enforced.

**Current.** §2 (`:37-58`) lists ten crates and four dependency rules.

**Proposed (per B11).** Record both crates as **approved with dates**, not as
present-day additions:
- **`dacode-store`** — `EventStore`/`SessionStore`/`BlobStore` traits + backends
  + compression. **Traits live in `dacode-core::store` from M1; extract the crate
  at M3** when `SqliteStore` becomes the second implementor. Rationale: a crate
  boundary with one implementation is indirection without payoff, and the trait
  risk is fixed by having a second implementor, not by where the trait lives.
- **`dacode-introspect`** — proxy + cache analyzer. **Created at M5**; its
  exchange-**record format** is an M1 deliverable in §6 (see A10).
- Update the dependency rules to state where each sits when it arrives
  (`core` → `store`; `server` → `core`, `protocol`, `store`, never UI crates),
  and update [Appendix D](D-crate-graph.md) to mark both **deferred**.

**Depends on.** ~~B11~~ ✅ **signed 2026-09-17**. It also turned out to depend on
**A10** in one direction: this entry says `dacode-introspect`'s "exchange-**record
format** is an M1 deliverable in §6 (see A10)", and that record **landed earlier
today**, so §2.1 now cites it as normative rather than owed.

**Recommend accept as amended.** If you prefer dependency isolation from day
one, say so and `dacode-store` becomes an M0 crate instead — that is the
respectable dissent, and the cost is designing the trait against one
implementor. **Done 2026-09-18**, with the dissent not taken and the three
scoping decisions in the banner. **The "Current." text below is historical** — §2
now lists eleven crates rather than ten (`xtask` arrived with A20) and has more
than four dependency rules.

**Settled alongside sign-off: all crates are vendored in-tree.** Every crate —
including `dacode-store` and `dacode-introspect` when they arrive — lives under
`crates/` in this repository with its full name (`dacode-protocol`,
`dacode-tools`, …). **One repo, not a workspace of repos**; the only external
crate is the JSON library, `dacode-json` (renamed from `dacodec`, landed). So B11's
deferral is about *when a directory appears under `crates/`*, never about
publishing anything. §2's tree already implies this; the amendment makes it
explicit, and it means [Appendix D](D-crate-graph.md) needs no repo/path column.

---

### A14 — §3.1, §3.2, §3.3, §3.5, §3.7, §4.3, §4.7, §8.4, §12, §14: session organization — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, on [B13](B-decisions.md)'s sign-off of 2026-09-17 and a
> scoping session with the user on 2026-09-18. **The delta as signed covered two
> sections; ten were edited**, because the question it left unanswered — what a folder
> *is*, as opposed to which folder a session is in — has no home in §3.2 or §3.5. The
> three signed events landed verbatim; everything around them is new and is listed as a
> scoping decision below rather than smuggled in.
>
> **Four scoping decisions, all four put to the user and answered 2026-09-18.**
>
> 1. **The registry is authoritative for names; membership stays a fold.** The signed
>    delta carried three *membership* events and nothing that carries what a folder
>    **is** — name, parent, order. A folder is a cross-session entity while every
>    `Envelope` is scoped to one session, so a folder tree has no log to live in; and
>    B13's own motivation defeats pure membership, since a folder for a many-step
>    feature must be creatable *before* the sessions that fill it. What landed:
>    `org.json` (§12) is authoritative for names and is the one file in that layout
>    that is **not** a projection, with B13's Fix-1 carve-out stated explicitly —
>    membership is rebuildable, names are not. **v1 accepts that a rename reaches no
>    client but its issuer**, which learns it from the `Ack`. §3.7's "never silently
>    stale" promise is therefore **scoped to a session** with the registry named as
>    the one exception, and the freshness channel (a version on the tree reads plus a
>    hint on the socket) is **A7's**, tracked as **U99** — it is §8.2 surface, not
>    replay semantics. **Rejected and recorded:** widening `Envelope.session` to make
>    the registry an event stream, which would re-open §3.6's cursor, lock and fencing
>    questions for a second stream that is not append-only anyway; and folder-as-path-
>    string, which cannot represent an empty folder and so cannot represent B13's
>    create-then-fill case at all.
> 2. **Registry mutation is a `Command`, and §8.4's controller rule does not reach
>    it.** A `Command` rather than an HTTP route because §8.1's default `dacode` is a
>    TUI with an in-process engine and **no socket** — an HTTP-only registry would be
>    unreachable from the client that ships first, and principle 1 would be false for
>    it. `CreateSession` is the precedent that the enum holds session-less commands.
>    §8.4 gained the reason the controller rule cannot apply (no session, and an empty
>    folder has no controller at all), that **any authenticated client** may mutate the
>    registry in v1 since §8.3 has no users and no roles and inventing a principal here
>    would be a capability system in all but name, and that `Hello`'s reserved flags are
>    where a rule lands if multi-user auth arrives — decided once for every non-session
>    write rather than per command. The three *membership* commands joined §8.4's
>    controller-only list.
> 3. **`org.json`, atomic, in §12.** Authoritative, written by temporary file plus
>    rename because a torn registry loses names nothing else holds, and bounded by the
>    number of folders rather than of sessions. `meta.json`'s comment corrected — it
>    *caches* `SessionInfo`, which the log remains the truth about. Conflict rule stated
>    where both files are described: names → the registry, membership → the log.
> 4. **One op-carrying `Org { op: OrgOp }` rather than seven registry commands**, on
>    `TaskUpdated`'s own stated argument: one golden fixture covers the subsystem at no
>    loss of expressiveness. **The three membership events were left exactly as
>    signed** — collapsing them into an op-carrying variant too would have deviated
>    from a signed delta, which needs the user in the same change. Sub-decision forced
>    by the existing `Ack`: **ids are caller-allocated**. `Ack` has no payload slot and
>    the registry has no log, so there is no channel to return a new `ProjectId` in;
>    both ids are ULIDs and `cmd_id` is already caller-chosen, so this costs nothing and
>    keeps `Ack` payload-free.
>
> **Three smaller items the user approved by not objecting, all landed:**
> `ProjectId(Ulid)` + `FolderId(Ulid)` in §3.1, **excluded from §4.3's `args` scan**
> with the clause shape `HostId` got; `SessionInfo` promoted from a comment to a real
> struct in §3.5; tags given a normalization rule (trimmed, case-folded, interior
> whitespace collapsed, empty rejected) and a config cap (`session.tags_max`, default
> 64) in §3.2.
>
> **Two fold-ins the delta did not mention.** **§4.7 now carries Fix-2 as normative
> spec text**: B13 assigned the worktree derivation to doc 06, but the spec may not
> cite a plan doc, and a rule that §14 must test has to be normative somewhere — so
> §4.7 states that derivation maps back through `--git-common-dir`, and §14 gains the
> test that can fail on it. **§14's fixture obligation is discharged by derivation
> rather than by a list**: A1's rule makes the durable subset self-maintaining, so the
> three events acquire *log* fixtures with nothing to update, and `Org` needs exactly
> one command fixture. Two engine tests added: *a rename is not an event* and *a
> sandboxed session derives to the main repository*.
>
> ⚠️ **This delta falsified a prediction a signed delta had already made.** A23's
> banner said twice that "A14 will add `ProjectId` to the scanned set when it lands".
> It does not, and cannot usefully: §4.3's scan catches ids the engine **allocated and
> injected**, and nothing injects a project id — `DACODE_RT_` starts at three
> (session, call, tmp) — nor can any v1 tool argument name one, so the scan could never
> fire. §4.3 now says so in `HostId`'s clause shape, and **both A23 sites carry an
> annotation** rather than being left as live claims the spec contradicts.
>
> **Debt this creates, recorded rather than left to the next reader.** **A7** owns the
> registry freshness channel (**U99**) and should answer "who may write a non-session
> resource from a browser" once, for `Org`, for config writes and for `promote`
> together. **Doc 03 stays a summary** despite now being further out of date — A14
> adds five `Command` variants and three `Event` variants to the crate it describes —
> because flipping it would deepen an unstarted milestone's doc; M1.1 corrects it, as
> A1's banner already recorded. **Doc 05** gained one sentence: its `SessionStore` row
> called the organization mappings a projection, which is now true of membership and
> false of names. **Closed: U16, U17.**

**Current.** §3.2 has `SessionCreated`, `SessionRenamed`, `SessionClosed`.
§3.5's `SessionInfo` is "id, agent, model, cwd, title, parent, children,
perm_mode".

**Proposed.**
- Three new **durable** events plus matching **controller-only** Commands (§8.4):
  `SessionProjectSet { project: Option<ProjectId> }`,
  `SessionTagged { add: Vec<String>, remove: Vec<String> }`,
  `SessionMoved { folder: Option<FolderId> }`.
- `SessionInfo` gains `project`, `tags`, `folder`; the fold applies them so TUI,
  web and every observer converge. Golden fixtures for each new variant (§14).
- **Include B13's Fix-1 wording verbatim**, because §6.2 and §6.3 of
  [06](../part2-engine/06-session-organization.md) currently contradict each
  other: organization changes are durable events **in the session event log**,
  are **never part of `Context`** (the provider never sees them), and the
  `SessionStore` mapping tables are a **rebuildable projection** for querying,
  never authoritative.
- Record B13's Fix-2 consequence for project auto-derivation: in a §4.7 worktree
  sandbox, `git rev-parse --show-toplevel` returns the **worktree** path, so
  derivation must map back through `--git-common-dir` to the main repository or
  every sandboxed session splits into its own project.

**Depends on.** ~~B13~~ ✅ signed, and on ~~A1~~ ✅ **applied first, as required** —
the three new events are durable and their fixtures are *log* fixtures, which only
means something once `tier()` exists.

**Recommend accept**, including both fixes. **Done 2026-09-18**, with both fixes and
with the scope widened as the banner records: the proposal below describes two
sections and three events, and what landed is ten sections, three events, four
commands, an `OrgOp` enum, two new identifiers and an authoritative registry file.
**The "Current." paragraph above describes the spec as it was before this pass** —
§3.5's `SessionInfo` is a real struct now, and §3.2's lifecycle group has three more
variants.

---

### A15 — §2.3, §3.6, §12, §13.1, §13.2, §13.5, §14: selective compression — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18**, on [B12](B-decisions.md)'s resolution of 2026-09-17
> (`zstd`, the dependency policy's first accepted class-1 exception) and the sign-off
> of the same day. **The mechanism landed exactly as signed** — per-field `codec`
> discriminator, config-driven allowlist, ~1 KiB floor, configurable level, the one
> `compress = "off" | "blobs" | "allowlist"` knob, and the explicit non-searchability
> trade. **Three of its enumerations did not survive contact with deltas that landed
> after it was drafted**, and each correction is recorded below as a deviation rather
> than folded in silently. Seven spec sections were edited against the two the delta
> named, because two of them were carrying claims B12 and B21 had already falsified.
>
> **Four scoping decisions. The banner's own instruction covered one; three are
> deviations from a signed enumeration and are put on the record for that reason.**
>
> 1. **The codec is named in the `LogHeader`, not in the discriminator.** The signed
>    variant list was `plain | zstd | flat | flat+zstd`; what landed is
>    **`plain | compressed`** plus `{"codec":"zstd"}` in the header A1 created for
>    exactly this kind of file-level property. The instruction to "generalize the
>    variant names" pre-authorized removing `zstd` from the enum, but generalizing
>    naively — `plain | compressed` with the codec in *config* alone — would have
>    opened a hole: change the configured codec and every already-written record
>    tagged `compressed` becomes undecodable. The log is append-only, so existing
>    records are never re-encoded and one file is written under one codec for its
>    whole life; the header states which. A per-field codec name would duplicate that
>    in every compressed record and grow the enum by a variant per codec, while still
>    obliging a reader to support every codec any surviving log used — the same
>    obligation at a per-record cost. **Blobs are the exception and must be**: they
>    live outside the log (§12), cannot inherit its header, and so each is
>    self-describing.
> 2. **`flat` is not a field codec and was removed from the enum.** A15 listed it
>    beside the codec names, but `flat` is a representation of a *record* while
>    `codec` is an encoding of one *field* inside a record — two axes. Keeping both in
>    one per-field discriminator would make a **format** change expressible as a
>    **compression** change, which is how a reader ends up unable to tell whether a log
>    was re-encoded or re-formatted. `flat` stays where §13.5 already put it: a
>    possible future binary `Codec` for the log, announced by `LogHeader` — the
>    reserved-not-implemented pattern B7 and A22 both use.
> 3. **Two compression targets were invalidated by A1 and are restated, not
>    deleted.** The signed allowlist named `ToolOutputDelta` chunks as "the dominant
>    cost" and `ThinkingDelta`/reasoning content beside them. **Both are `Live`** under
>    A1's tier rule — broadcast and never written (§3.2, §3.6) — so compressing them
>    saves zero bytes on disk. The cost *is* dominant, but in the stream, not in the
>    log. The durable carriers of the same content replaced them: `ToolResult.output`
>    (already listed) and **`ThinkingEnded.text`**, the whole-block boundary A1 added,
>    which is where reasoning content becomes durable. This is the clearest case of a
>    delta drafted before another one that reordered its premises.
> 4. **§13.2 asserted `flat` as the log's representation, and B21 had already
>    overruled it.** Its per-surface table said "On-disk event log → `flat` buffers",
>    landed with A16 on 2026-09-12; **B21 was signed five days later** with "the log
>    stays append-only JSONL and stays the truth", and §3.6, §12 (`events.jsonl`,
>    `LogHeader` on line 1, `index.bin` as `(seq, offset)`) and §8.2 (`/export` →
>    "events.jsonl download") all agree with B21. So the table row was the defect. A15
>    is the delta that specifies log compression and therefore the one that had to
>    resolve it — it could not name a per-field codec inside a record whose format was
>    stated two sections away as something else. §13.1's tier table and §13.2's row are
>    both corrected; §13.5 keeps `flat` as the reserved future codec.
>
> **Three stale claims corrected in the same pass, none of them in a section A15
> names.** **§2.3** still listed "the session compression codec" among the C
> dependencies that are "open choices to be settled on measurement" — B12 settled it by
> decision, and Appendix F's row had been moved to *accepted* while the spec still
> called it open. Per A17's applied note (only TLS is named in the spec as unavoidable)
> the fix moves the codec *out* of the open-choices list and records it as the policy's
> first accepted exception with its condition still open, rather than adding a `zstd`
> paragraph to §2.3. **§13.5** said the log's compression codec "is still open". And
> **nothing anywhere said the provider transcript is never compressed** — A15's
> "thinking/reasoning content" target could be read as the transcript, which §13.4
> forbids re-encoding because the cached prefix must stay byte-identical. §3.6 now says
> so explicitly, and it is a §14 test.
>
> **Debt this creates, recorded rather than left to the next reader.** **U14 stays
> open and A15 does not discharge it** — it is the *condition* on B12's exception
> (verify `libzstd`'s cross-compile cost against the CI matrix), and `zstd-sys` is
> rowed in Appendix F in the commit that adds the dependency, per §2.3's same-commit
> rule. **A21** (§14 → §14.1/§14.2) must place the three new storage-codec tests in
> the correctness half, and **B18's T1** benches tune the *level* rather than choosing
> the codec, so §14.2 should cite the level as the measured knob. **Doc 05** §5.2/§5.3
> corrected in the same change: its `flat`-compose bullet, its two invalidated targets,
> its write path and its stale `[DECISION B12]` marker. **Doc 03 stays a summary** and
> is now four deltas out of date (M1.1). **Closed: U25** — its transcription obligation
> was exactly the generalization instruction, and it is discharged with a sharper reason
> than the one it was written with.

**Current.** §3.6 (`:200-210`) specifies JSON Lines of `Envelope`, one file per
session, fsync'd at boundaries — no compression, no codec tagging. §12
(`:716-742`) gives the on-disk layout.

**Proposed.**
- A **per-field `codec` discriminator** on compressible fields:
  `plain | zstd | flat | flat+zstd`. The envelope skeleton (ids, `seq`, `ts`,
  `type`) stays plain so it remains indexable and cheap to scan.
- A **config-driven allowlist** of compression targets: `ToolResult.output` and
  `ToolOutputDelta` chunks (the dominant cost), thinking/reasoning content, and
  blob-store segments (always). **Never** the fields clients filter or search on:
  ids, `seq`, `ts`, event `type`, tool `name`, permission decisions, `usage`,
  titles, tags.
- A **size floor** (~1 KiB) below which compression is skipped, and a
  configurable level (default ~3).
- One knob: `[storage] compress = "off" | "blobs" | "allowlist"`. `JsonlStore`
  defaults to `blobs` so JSONL stays grep-able; `Sqlite`/`Postgres` default to
  `allowlist`.
- The **explicit searchability trade**: a compressed field is not SQL-`LIKE`
  searchable, and that is intended. If full-text search over tool output is ever
  wanted, either exclude the field or maintain a separate FTS index built at
  write time.
- §12 gains the blob/compressed-segment layout.

**Depends on.** ~~B12 (zstd)~~ ✅ **resolved 2026-09-17**, and on ~~A1~~ ✅ **applied
first** — which is why two of this delta's targets had to be restated rather than
transcribed. Note B12's open check: `libzstd` is C and touches
the cross-compile matrix in
[31](../part6-quality/31-ci-milestones.md) — verify before M1, since `ruzstd` is
decode-only and cannot replace it for writes. **That check is U14, it is still open,
and applying this delta does not discharge it.**

**Recommend accept.** **Done 2026-09-18**, with the mechanism exactly as proposed and
three enumerations corrected against deltas that landed after this was drafted. **The
"Proposed." list above is now historical** — its variant list, its two `*Delta` targets
and its silence on where the codec is named are all superseded by the banner, which is
the record of what landed and why.

---

### A16 — §13: the `dacode-json` hybrid strategy + `RawJson` — ✅ **APPLIED 2026-09-12**

> **✅ APPLIED 2026-09-12**, on sign-off of [B1](B-decisions.md) **and B6**
> (the latter delegated by the user alongside B1). All seven items below landed;
> §13 is now **§13.1 tiers · §13.2 per-surface table · §13.3 the serde rule
> re-read · §13.4 byte-exactness · §13.5 `Codec` + conventions**. The
> `_rust.md` caveat item 1 motivated is **deleted**, and the unqualified
> "serde types never leak" rule was corrected to *third-party* at all eight
> sites that carried it (`AGENTS.md` ×2, the `dacode` skill ×2,
> `implement.md` ×3, `reviewer.md`, `roles/implementer.md`, `agent-rules.yaml`
> ×4), with the adapters regenerated.
>
> **Three scoping decisions taken while applying it — recorded so a reviewer can
> check them rather than infer them:**
>
> 1. **§13.2 defines what the spec's bare `Json` alias denotes** (`RawJson`)
>    instead of editing the signatures that use it. `ToolCall.args` (§3.2) and
>    `Tool::perm_scope`/`call` (§4.3) both say `Json`; rewriting §13 alone would
>    have left the spec self-contradictory, but those signatures belong to
>    **A1** and **A2**, which are unsigned. Defining the alias in the section
>    that owns JSON representation removes the contradiction without pre-empting
>    either delta.
> 2. **§4.3's `ToolSpec` comment was fixed** — "JSON schema (dacode-json Value)"
>    named a type that does not exist, the same defect as §13's. Corrected to
>    "(canonical RawJson, §13.2)". The *signature* was left alone; that is A2's.
> 3. **The log-codec wording is deliberately codec-agnostic.** Item 6's
>    "`flat`-based binary codec" is kept, but §13.5 does **not** name a
>    compression codec, because [B12](B-decisions.md) was reopened by A17 and
>    `zstd` is no longer assumed. Doc 02 §2.4's table still says "zstd" and is
>    now stale — logged in [TODO](../TODO.md) §6 rather than folded in here.

**Current.** §13 (`:746-760`), five bullets: a `Codec` trait with
"`dacode-json` as the sole implementation"; "Tool schemas and provider
request/response bodies are `dacode_json::Value`"; the serde-isolation rule
("Do not let serde types leak into `dacode-protocol` or `dacode-core`"); and the
encoding conventions.

**Proposed — a full rewrite of §13** (it roughly triples in length). Contents:
1. **Correct the facts.** The crate *name* is now right everywhere — the
   `dacodec` → `dacode-json` rename landed and was swept through this repo — so
   this item reduces to its other half: there is **no owned value type**.
   `Value<'a>`/`Doc<'a>` are borrowed from the input slice plus a `Pool`, so
   §13's `dacode_json::Value` cannot exist as written and an `Event` cannot hold
   it. State the tiers that do exist: typed (serde), DOM (`Parser`/`Doc`),
   on-demand (`pull`), zero-copy (`flat`), builders (`write`/`builder`) — the
   survey is [02 §2.1](../part1-foundations/02-json-strategy.md).
2. **Adopt the hybrid per-surface strategy** ([02 §2.4](../part1-foundations/02-json-strategy.md)):
   serde-derived structs for control types with adjacent tagging; **`RawJson`**
   for opaque sub-JSON (`ToolCall.args`, `ToolSpec.schema`, provider raw
   bodies); typed decode for a tool reading its own args; `pull`/DOM for engine
   inspection; `flat` (+zstd) for the on-disk log; base64 for image bytes; raw
   `reqwest`/`hyper` + `dacode-json` for provider HTTP (no vendor SDK).
3. **Re-read the serde-isolation rule.** Its intent is that **third-party** serde
   types do not leak into `dacode-protocol`/`dacode-core`. dacode's **own** types
   deriving `serde::{Serialize,Deserialize}` and going through `dacode-json` is the
   intended path, not a violation. Without this re-reading the rule forbids the
   only workable strategy.
4. **Record `schemars`' isolation boundary as a consequence** (B6): `schemars`
   depends on `serde_json` non-optionally (verified from forge's `Cargo.lock`,
   both 0.9.0 and 1.2.2), so it is confined to **`dacode-tools`**, converting
   `schemars::Schema` → `RawJson` at the boundary — the same adapter precedent
   §13 already sets for MCP crates. Generated schemas must be **canonicalized
   and byte-stable** (strip `title`, pin key order) because A4 makes tool-order
   churn a cache kill.
5. **State byte-exactness as a correctness property.** Opaque JSON is preserved
   end-to-end (provider → event → log → wire) so the prompt-cache stable prefix
   survives round-trips. An owned-tree `Value` would reorder keys and reformat
   floats and silently defeat caching. Include the measured justification:
   splice ≈110× faster than re-encoding, capture 1.2–1.5× cheaper than a tree
   with 2 inherent allocations vs 114 080.
6. **Keep the `Codec` trait** with `dacode-json` as sole impl, leaving room for a
   `flat`-based binary codec for the log (A15).
7. Keep the encoding conventions bullet unchanged (adjacent tagging, `Option`
   omitted, bytes base64).

**Depends on.** B1, B6. **Supersedes A11.**

**Recommend accept.** This is the largest single rewrite in the set, and per
A11's blast-radius table it also triggers edits to `AGENTS.md`, the role/config/
skill files and a `tools/agents-sync.py` regeneration. The misleading
`dacode_json::Value` reference in `.agents/roles/_rust.md` is loaded into every
coding agent's context, which makes this the most urgent delta in the set
*despite* being nominally blocked on B1.

**The upstream rename has landed — this delta is no longer held on it.** The JSON
crate was renamed **`dacodec` → `dacode-json`** (module path `dacode_json`,
upstream commit `e739d6c`), and the whole tree was swept in the same pass: the
spec, both appendices, the two cross-harness digests, `TODO.md`, `AGENTS.md`, the
role and skill files, and the regenerated `.opencode` adapters.
[Appendix E](E-dacode-json-cheatsheet.md) and
[02](../part1-foundations/02-json-strategy.md) were renamed with it, since both
were named after the old crate.

What the sweep deliberately did **not** do is apply items 1–7 below. That is
still gated on **B1**, and the `Value`-does-not-exist half of the finding is
untouched by a rename — renaming a type that does not exist gives you a
correctly-spelled type that does not exist. Two consequences for whoever applies
this:

- Write `dacode-json` / `dacode_json` throughout. The names are already correct
  everywhere else, so the delta no longer has to introduce them, and item 1's
  "correct the facts" reduces to the `Value` half plus the tier survey.
- `.agents/roles/_rust.md`'s "JSON goes through `dacode_json::Value`" line now
  carries an inline caveat pointing here and at B1. Applying A16 is what lets
  that caveat be deleted.

---

### A17 — §2: the dependency policy — ✅ **APPLIED 2026-09-12**

> **✅ APPLIED 2026-09-12**, on the user's sign-off. All four proposed bullets
> landed; the policy is now normative at **design §2.3**, and
> [Appendix F](F-c-dependencies.md) is the register §2.3 points at rather than a
> proposal.
>
> **Five scoping decisions taken while applying it — recorded so a reviewer can
> check them rather than infer them:**
>
> 1. **§2 was restructured into §2.1 crate tree · §2.2 internal dependency rules
>    · §2.3 external dependency policy**, rather than the policy being appended
>    to §2's existing body. Every other multi-part section of the spec uses
>    `### N.M` (see §13.1–§13.5, which A16 created the same way), and a lone
>    `### 2.1` with no §2.2 would leave a numbering gap. §2 had no subsections
>    before, so nothing referenced §2.x; references to "§2" stay valid. The line
>    numbers in *Current* below are pre-restructure.
> 2. **The spec gained no link to Appendix F.** `docs/dacode-design.md` contains
>    **zero** markdown links — the plan cites the spec, never the reverse — so
>    §2.3 says "a debt register maintained alongside the implementation plan"
>    instead of naming the file. Appendix F carries the back-reference.
> 3. **`§1.5` is written as "principle 5" in the spec.** §1 is a five-item
>    numbered list with no subsections, so the `§1.5` shorthand this entry uses
>    reads as a dangling section reference inside the spec itself.
> 4. **The avoidable consequences are deliberately not enumerated in the spec.**
>    Only TLS — the one genuinely unavoidable class-1 dependency — is named.
>    Listing B12's codec, M3's store and `syntect`'s feature flags in the
>    normative doc would enshrine choices B12 has not made and A15 has not
>    applied; §2.3 instead states that they remain register rows settled on
>    measurement. The full register stays in Appendix F, which is where a
>    reviewer looks for it.
> 5. **A13/B11 content was kept out**, though it also concerns §2 (in-tree
>    vendoring, `dacode-store`, `dacode-introspect`). Both are unsigned, so
>    applying them would have ridden along on A17's approval.
>
> **Unrelated edit landing in the same commit:** the spec header gained a
> one-line **versioning convention** (the version moves at milestone boundaries;
> a delta application moves the date). That settles the TODO §6 follow-up raised
> during the A16 pass, which A17 made load-bearing — it is not part of this delta.

**Current.** §2 (`:35-58`) lists the crates and the dependency rules *between
them*, but says nothing about what may enter the tree from outside. The only
related statements are §1.5's single-binary principle and §13's "no vendor SDK"
for provider clients.

**Proposed.** Add a dependency policy to §2:
- **Pure Rust by default.**
- A dependency that compiles or links vendored C/C++ — detectable as a `build.rs`
  invoking `cc`, `cmake` or `bindgen`, or a published `*-sys` companion —
  requires a row in [Appendix F](F-c-dependencies.md) with a reason and a
  replacement plan, **and** a CI allowlist entry, in the same commit.
- FFI bindings to the platform's own ABI (`libc`, `windows-sys`) are **exempt** —
  no C is compiled or vendored, and banning them would ban the standard library's
  own platform layer.
- Enforced by `cargo-deny [bans]` plus a `cargo tree` scan for those markers
  against the allowlist, so the register is the machine-checkable form of the
  policy rather than a document nobody re-reads.

**Why adopt it before M0 rather than after M3.** forge's `Cargo.lock` — a
shipping Rust agent harness — carries `cc`, `cmake`, `openssl-sys`,
`libsqlite3-sys`, `ring`, `aws-lc-sys`, `aws-lc-rs`, `onig_sys`, `openssl`,
`rustls` and `native-tls`. That is a good approximation of what dacode
accumulates without a policy. Adopting it first means the initial `Cargo.toml`
files are written under it.

**Consequences already identified** (full detail in Appendix F):
- **TLS is unavoidable C.** There is no production pure-Rust TLS: `rustls`'
  default `aws-lc-rs` provider needs `cmake`, and its `ring` fallback also ships
  `cc`-compiled C/asm. Prefer `ring` (smaller build, no `cmake`); never
  `native-tls`/`openssl-sys`. This is the register's flagship entry and it is
  known-unavoidable on day one, which is itself the argument for having the
  register.
- **B12 is reopened.** `zstd` links C via `zstd-sys`, and `ruzstd` is
  **decode-only** so it cannot serve the write path. Pure-Rust alternatives:
  `lz4_flex`, `brotli`, `miniz_oxide` / `flate2` with `rust_backend`.
- **M3's `SqliteStore` conflicts.** `rusqlite`/`sqlx` bundle SQLite C. `redb` is
  pure Rust and ACID but has **no SQL**, and the session filters in
  [06 §6.4](../part2-engine/06-session-organization.md) assume queryability.
  Decide at M3; `JsonlStore` (pure Rust) is the default until then.
- **`syntect`** needs `default-features = false, features = ["regex-fancy"]` to
  avoid pulling `onig`.
- **Confirmed clean:** `dacode-json` (only non-optional dep is `zmij`, pure-Rust
  `no_std`; `cc` is used solely by the opt-in `cbench` baselines),
  `ratatui` + `crossterm`, `gitoxide`, `grep`/`regex`/`ignore`, `toml_edit`.

**Depends on.** Nothing.

**Recommend accept, early.**

---

### A18 — §3.7, §4.1: bounded fan-out and the seq→gap→resync contract — ✅ **APPLIED 2026-09-12**

> **✅ APPLIED 2026-09-12**, on the user's sign-off, together with **B4** (which
> this delta had already absorbed the robustness half of). The §3.7 and §4.1
> blocks below were applied **verbatim** — they were drafted as paste-ready spec
> text, so proposal and spec should diff clean against each other.
>
> **Three scoping decisions taken while applying it — recorded so a reviewer can
> check them rather than infer them:**
>
> 1. **§14 was extended, which the proposal only mentioned under *Interactions*.**
>    §3.7's step 5 asserts the fold is idempotent over `seq`, but §14's protocol
>    test list did not cover it, and nothing tested the bounded drop-oldest path
>    at all. Added both: fold-idempotence as a property test beside the existing
>    fold-equivalence one, and a deliberately lagged subscriber in the server
>    integration test that must detect the `seq` gap and resync to the same folded
>    state as a client that never lagged. Stating an invariant in §3.7 without a
>    test that can fail on it is how §3.7 got its false guarantee in the first
>    place.
> 2. **§3.7's heading was left as "Reconnect / replay semantics"** even though the
>    section now leads with fan-out. The bounded queue *is* what makes reconnect
>    and replay sound, so the title still describes the content, and renaming
>    would exceed the signed-off scope. (No doc anywhere anchors to a spec
>    heading — verified — so a rename would be safe if it is ever wanted.)
> 3. **Version stays `v0.1` and the date stays 2026-09-12**, per the versioning
>    convention recorded under the spec's `Status` field: a delta application
>    moves neither. This is the first application *under* that convention.
>
> **No new `Event`/`Command` variant**, so no new golden fixture — as the
> proposal argued when rejecting an explicit `Lagged` event. §14 gains tests, not
> fixtures.
>
> **Consequence for A1:** its §3.7 resync bullet is now *discharged*, not merely
> repointed, so A1 narrows to the tier marker and the `*Ended` full-value
> variants — and its B4 dependency is gone along with B4's resolution. A1 is the
> last structural blocker on M1's golden fixtures.

**New, raised after A17 landed.** Found by a client-transport question — "how do
TUI/web clients talk to `serve`, and what is best for fast notifications?" —
which exposed that §3.7's central guarantee is asserted rather than earned.
Recorded first as an amendment to [B4](B-decisions.md); split out as its own
delta because **it is transport-independent and should not wait on B4**, which
is by its own account the most taste-driven decision in the set.

**Current.** §3.7 (`:212-219`) is three bullets ending in an unqualified
assertion:

> - A client that reconnects does `GetSnapshot` → `Subscribe { since: snapshot.cursor }`.
>   No gap, no duplicate.

§4.1 (`:306-308`) says the registry is a `DashMap` "plus a broadcast channel per
session" and stops. Neither mentions what happens when a client cannot keep up.

**The defect.** `tokio::sync::broadcast` — the obvious reading of "a broadcast
channel per session" — **drops the oldest messages when a receiver lags**,
surfacing `RecvError::Lagged`. So the primitive §4.1 already picked is lossy
under a slow consumer, and §3.7's "No gap, no duplicate" is **false as written**.
[07-session-registry](../part2-engine/07-session-registry.md):16 does call for
"a bounded queue and explicit drop/backpressure policy … a slow client must not
stall the engine" (from openclaw's `allBounded` capacity-256 subscriber queue;
opencode's SSE fan-out queue is also capacity-256), but a summary-depth plan doc
is not where an implementer looks for a correctness invariant.

Note this is **not** an argument for unbounded queues. Unbounded fan-out means
one stalled browser tab applies backpressure to the engine, which is worse: it
trades a recoverable staleness for an unrecoverable hang. Bounded-and-lossy is
the right choice — it just has to be *stated*, together with what makes the loss
recoverable.

**Proposed §3.7 — replace the three bullets with:**

> **Fan-out is bounded; a slow client never stalls the engine.** Each session's
> broadcast channel (§4.1) has a bounded per-subscriber queue (default 256
> envelopes). A subscriber that falls behind loses its oldest queued envelopes.
> The engine does not block on any client, and one client's backpressure is
> invisible to every other.
>
> **Losing envelopes is safe because every `Envelope` carries a monotonic `seq`
> (§3.2) and clients detect discontinuities instead of trusting the stream.** The
> contract, end to end:
>
> 1. The engine assigns `seq` when it appends to the log — strictly increasing
>    within a session, no holes.
> 2. Fan-out is bounded; a lagging subscriber loses envelopes.
> 3. A client tracks the highest `seq` it has folded and treats any
>    discontinuity as a resync trigger. **Detection needs no extra protocol
>    surface: a missing `seq` is itself the signal.**
> 4. Resync is `GetSnapshot` → `Subscribe { since: snapshot.cursor }`, replaying
>    the missed range from the log.
> 5. The fold is idempotent over `seq`, so a resync whose range overlaps what the
>    client already applied converges instead of double-counting.
>
> A client is therefore never *silently* stale: it either holds every envelope or
> knows it missed some and is fetching them. **"No gap, no duplicate" is a
> property of this contract, not of the transport** — it holds identically over
> the WebSocket (§3.6) and over stdio extensions (§7.2).
>
> - `Subscribe { since: Some(c) }` streams every event with `seq > c` from the
>   log, then live events. `since: None` means live only, and is an explicit
>   opt-out of gap recovery: such a client must re-snapshot on reconnect rather
>   than assume continuity.
> - The server keeps the last N (default 5 000) events per session in memory;
>   older ranges are read from the log file. **N must exceed the subscriber queue
>   depth**, or a client lagging far enough to be dropped would also fall outside
>   the in-memory window and pay a disk read to recover. The two limits are one
>   invariant, not two independent tunables.
> - A client that reconnects does `GetSnapshot` → `Subscribe { since: snapshot.cursor }`.

**Proposed §4.1 — append one sentence to the registry paragraph:**

> Each session's broadcast channel is bounded with an explicit drop-oldest policy
> (§3.7); the engine never blocks on a subscriber.

**An explicit `Lagged` event was considered and rejected.** `seq` discontinuity
already carries the information, and a new `Event` variant would be a §14
compatibility event plus a golden fixture for no gain. If an implementation wants
to signal lag eagerly it may do so as a transport-level control frame — that is
an optimisation, not protocol, and clients must not depend on it.

**Depends on.** Nothing. Independent of every B decision, including B4.

**Interactions.**

- **Shrinks A1.** A1's §3.7 bullet ("adopt seq-numbered events + client-side gap
  detection → resync") is subsumed. Applying A18 first leaves A1 carrying only
  the tier marker and the `*Ended` full-value variants — welcome, since A1 is the
  most structural delta in the set and must land before any golden fixtures.
- **Settles part of B4 in advance.** The robustness contract B4 recommends is
  exactly this one, and it is identical under WS and SSE (opencode reaches it with
  `?after=<seq>` plus `EventSource` reconnect; openclaw with WS-side detection).
  B4 then reduces to the transport taste call it should have been.
- **Promotes doc 07 and doc 19 from recommendation to spec.** Doc 07's bounded
  queue and doc 19's "adopt seq-numbered events + gap detection → resync" both
  become statements of what §3.7 requires.
- **§14 gains one test, not one fixture.** No new variant, so no new golden
  fixture. But fold-idempotence-over-`seq` (step 5) deserves a property test
  alongside the existing `fold(events) == fold(snapshot_at(k)) + fold(events[k..])`
  — specifically that folding a range twice is a no-op, which is what makes
  overlapping resync safe.

**Recommend accept, and accept before A1.** It is small, depends on nothing,
removes a false guarantee from the spec, and reduces the blast radius of the
delta that everything else in M1 is waiting on.

---

### A19 — §4.8 (new), §3.2, §3.5, §4.2, §4.3, §5.4, §9.2/§10, §12, §14, §16: built-in task tracking & gating — ✅ **APPLIED 2026-09-13**

> **✅ APPLIED 2026-09-13**, on the user's sign-off of **B15** (all five
> recommendation points and all four consequences, none redirected). A new design
> **§4.8** plus nine further edits: §3.2's `TaskUpdated` variant and its
> one-variant-carrying-an-op rationale, §3.5's `Snapshot.tasks`, §4.2's gate in
> the loop pseudocode *and* its special-tool list, §4.3's `todo` entry, §5.4's
> `gate` → `approve` rename, §9.2's unified tree component, §10's TUI sidebar,
> §12's `plans/<board>.md`, and §14's two engine tests.
>
> **Four scoping decisions taken while applying it — recorded so a reviewer can
> check them rather than infer them:**
>
> 1. **§4.8 was written self-contained, NOT transcribed verbatim, because three
>    references in the proposed text were dangling.** It cited "the cache-stable
>    prefix … its fingerprint (§4.5)" — §4.5 has no caching content, **A4**
>    creates it; "rendered as a segment" — **B14**'s `Segment`/`ContextBuffer`,
>    unsigned; and "joins the guard set … the same shape as A3's per-tool error
>    budget" — §4.2 terminates on `if turn_count > agent.max_turns`, **A3**
>    creates the guard set. Landing those verbatim would have put three references
>    to nonexistent constructs into the normative spec: the `dacode_json::Value`
>    defect class again. §4.8 therefore states its own requirement in its own
>    words ("never rewrites bytes already sent … a provider's cached prefix
>    survives") and describes the gate without naming a guard set. **This is A16's
>    move** — it defined §13.2's `Json` alias locally rather than editing unsigned
>    signatures. A3/A4/B14 own tightening the cross-references when they land.
> 2. **This delta's own `Depends on` line was wrong and is corrected rather than
>    left in force.** It claimed *"A1 must land first — the tier marker is what
>    lets §3.2 say `TaskUpdated` is durable."* The spec has **no tier concept at
>    all** today (§3.6 persists *every* `Envelope`, deltas included — the defect A1
>    exists to fix), so §4.8 declaring durability in prose is consistent before and
>    after A1. A1 was never the blocker; A3/A4/B14 were, and decision 1 dissolved
>    them. The application-order paragraph below still places A19 after the engine
>    group, but for the *collision* reason (it edits §4.2 and §4.5) rather than the
>    dependency reason.
> 3. **§16 gained no B15 entry, though the text below asks for one.** §16 is the
>    *open*-decisions list and A12's proposal says not to empty it, so a closed
>    decision never listed there needs no row. What §16 *did* gain is a genuinely
>    open one — see decision 4.
> 4. **A gap found while applying was recorded, not patched.** `check` is red while
>    a `Goal` is unclosed with only `Blocked`/`WontFix` exempt, so a legitimately
>    *standing* goal reddens the gate on every turn until the reminder cap burns —
>    this repo's own `tools/tasks.py` failure mode (finding f21) reproduced in the
>    product. The fix is a `Standing` status or treating `Goal` like `Debt`, and
>    either invents beyond the signed-off text, so it is **§16.6** with a note that
>    it must settle before the gate is implemented and that it is a §14
>    compatibility event.
>
> **One golden fixture** for `TaskUpdated` (it is an `Event` variant, so §14's
> "every variant" rule already claims it) and **two engine tests** — rollback
> rewinds the tree, and a red `check` blocks then escalates rather than looping.
> The second is the only thing that can fail on the claim that the gate is
> *bounded*. Version stays **v0.1**, `Date` moves to 2026-09-13.

**New, raised by the user after the original pass.** The requirement: dacode's own
development already runs on a two-tier task discipline — `tools/tasks.py` (a
branch-keyed tree of goal/task/gate/finding/debt nodes, each with a mandatory
verification method, and a `check` that exits non-zero on a done-without-
verification item or an unclosed goal) plus the committed board at
[TODO](../TODO.md) — and that should be a **product feature**, not a repo-local
script. Decision dossier: **B15**. Evidence:
[task-tracking-and-gating](../../../.agents/knowledge/reference-harnesses/task-tracking-and-gating.md).

**Current.** Four fragments, none of them a design:

- §4.3's built-in list ends `task`, `todo` (per-session scratch list shown in
  UI)`. One line. No tree, no kinds, no verification, no gate.
- §4.2's loop terminates on its guard set (A3) and knows nothing about work items.
- §12 lists `<repo>/.dacode/plans/` and **specifies nothing about it**.
- §5.4's declarative workflows use the word **`gate`** for `ask_user` step
  approval — an unrelated concept wearing the same word.

**Proposed — a new §4.8, plus six edits that follow from it.**

> ### 4.8 Task tracking and completion gating
>
> Work state is a **fold, not a table**. Every mutation of the task tree is a
> durable event in the session log (§3.2) folded into `Snapshot` (§3.5), so
> replay, fork, undo and every client converge on the same tree, and a rollback
> (§4.6) rewinds it with no reconciliation code.
>
> ```rust
> pub enum TaskKind   { Goal, Task, Gate, Finding, Debt }
> pub enum TaskStatus { Pending, InProgress, Done, Blocked, WontFix }
> pub enum Verify {
>     Command { argv: Vec<String> },   // the engine runs it; the exit code is the gate
>     Manual  { what: String },        // only a controller (§8.4) may close it
> }
> pub struct TaskItem {
>     pub id: TaskId,
>     pub parent: Option<TaskId>,
>     pub kind: TaskKind,
>     pub title: String,
>     pub status: TaskStatus,
>     pub verify: Verify,               // mandatory at creation
>     pub evidence: Option<String>,     // recorded on close
>     pub blocked_on: Option<String>,   // required when status == Blocked
> }
> ```
>
> **`verify` is decided when the item is created, never retrofitted at close.** An
> `add` without one is rejected. The two flavours differ in what the engine can do
> with them: `Command` is *run* on close and its exit code is the gate — the only
> kind that can be enforced rather than asserted; `Manual` is not
> machine-checkable, so closing it requires a controller action (§8.4) recorded as
> its own event, and the model cannot self-certify.
>
> **`Verify::Command` is an exec surface and is treated as one.** Running it goes
> through the permission engine (§4.4) exactly as `bash` does — `PermScope::Exec`,
> the same deny-list, and the active sandbox (§4.7) when there is one. **The gate
> never auto-approves its own verification command.** A `verify` that needs
> approval blocks the close until the user grants it; the alternative is
> model-authored argv executed with no prompt, which is a privilege-escalation path
> rather than a feature.
>
> **The completion gate.** The loop (§4.2) may not emit a terminal turn while
> `check` is red. `check` is red when an item is `Done` whose `Verify::Command`
> failed or was never run, when a `Goal` is unclosed, or when a `Debt` is
> untriaged. The gate is **hard but bounded**: on a red check the engine appends a
> reminder and continues, up to `task.reminders_max` (default 3), after which it
> surfaces the red gate to the user and stops rather than looping forever. Items in
> `Blocked` — which must carry `blocked_on` — and `WontFix` are exempt from every
> reminder, because work genuinely waiting on a human must not be nudged
> indefinitely.
>
> **Context rendering is append-only.** The tree is rendered into the *new inbound*
> context as a droppable segment placed **after** the cache-stable prefix and
> excluded from its fingerprint (§4.5), so a status flip never rewrites bytes a
> provider has already cached. Tool results echo the **diff**, not the whole tree.
> The rendered segment is excluded from any export or review path that needs a
> stable prefix, since its counters change every turn.
>
> **Two tiers.** The session tree lives in the event log: durable, rewindable,
> per-session. Cross-session continuity is an **explicit promotion** to a committed
> board under `<repo>/.dacode/plans/`, written as markdown a human reviews like any
> other change. This is deliberately *not* cross-session memory: the board is a
> reviewed repository artifact and nothing accumulates silently.
>
> **Subagents** may update items they own — a `task` spawn (§4.3) carries the
> parent item id — and may not modify the parent's `Goal`s or `Gate`s.

**The six consequential edits.**

1. **§3.2** gains one durable variant, not five:
   `TaskUpdated { op: TaskOp, item: TaskItem }`, where `TaskOp` is
   `add | set | close | block | promote`. One variant carrying an op means one
   golden fixture for the whole subsystem instead of five, at no loss of
   expressiveness. Durable per A1's tier marker — a task tree that did not survive
   replay would be the whole point missing.
2. **§3.5** `Snapshot` gains `tasks: TaskTree`, so the fold is shared by TUI, web
   and extensions like every other state (§1 principle 2). No client keeps a
   private task list.
3. **§4.2** the completion gate joins the guard set as one more guard — it is the
   same shape as A3's per-tool error budget and stuck-loop intervention: a
   condition, a bounded number of interventions, then escalate to the user.
4. **§4.3** the `todo` entry becomes `todo` (the durable task tree and its
   `check`, §4.8)`. **The tool keeps the name `todo`** despite doing far more than
   a scratch list, because every reference harness uses that name and a familiar
   tool name is worth more to a model than an accurate one; `task` is already
   taken by subagent spawning.
5. **§12** `.dacode/plans/` is specified: `plans/<board>.md` — the promoted,
   committed, human-reviewed board. It joins the `.dacode/.gitignore` rule as
   *not* ignored, unlike `shadow/` and `worktrees/`.
6. **§9.2 / §10** the task tree renders in **the same component as the workflow
   and sub-agent trees** — a node with live status that expands to detail. A9
   already unifies those two on exactly that description; this is its third arm,
   one component rather than three.

**One separable cleanup, included but independently rejectable.** §5.4's workflow
step attribute `gate` is renamed **`approve`**, which is literally what it does
(*"pause for approval of the plan"*), leaving `gate` unambiguous for the task kind.
One word in one TOML example. Reject this bullet and nothing else in A19 changes,
but the two senses of "gate" then coexist in the spec.

**§16** gains the decision as resolved (B15), and A12 must record it.

**Depends on.** ~~**B15**, and **A1 must land first** — the tier marker is what
lets §3.2 say `TaskUpdated` is durable rather than live-only.~~ **Corrected at
application: B15 only.** The spec has no tier concept at all yet (§3.6 persists
every `Envelope`, deltas included), so §4.8 stating durability in prose is
consistent before *and* after A1 — see scoping decision 2 in the banner. It does
*edit* §4.2 and §4.5, which is a collision reason to sequence it after **A3** and
**A4**, and it must precede **A9** so the tree unification is written knowing the
third arm exists. **Applied 2026-09-13 ahead of A3/A4 by making §4.8
self-contained** (scoping decision 1), which is what removed the collision rather
than the ordering.

**Interactions.**

- **B14 supplies the mechanism §4.8 relies on.** "Rendered as a segment placed
  after the stable prefix and excluded from the fingerprint" is only expressible
  because B14's appender + segment index makes placement a decision rather than a
  rewrite. Without B14, §4.8's caching paragraph has nothing to operate on.
- **A14 is the structural precedent.** Session-organization events are durable,
  folded into `SessionInfo`, never part of `Context`, with store tables as a
  rebuildable projection. `TaskUpdated` → `Snapshot.tasks` is the same pattern —
  with one deliberate difference: the task tree **is** rendered into `Context`
  (append-only), because the model needs to see its own work, whereas organization
  metadata is exactly what the provider should never see.
- **§14 gains a fixture and two tests.** One golden fixture for `TaskUpdated`, a
  fold test that a rollback rewinds the tree, and a gate test that a red `check`
  blocks a terminal turn up to the reminder cap and then escalates rather than
  looping. The gate test is the important one: it is the only thing that can fail
  on the claim that the gate is bounded.

**Recommend accept.** The model, the persistence and the caching are all
precedent-backed and point the same way; the executable-`verify` gate is the one
part with no precedent anywhere, which is the reason to specify it carefully
rather than the reason to skip it.

---

### A20 — §2.4 (new): repository tooling language — ✅ **APPLIED 2026-09-16**

> **✅ APPLIED 2026-09-16**, on sign-off of **B16** — but **narrowed on the way
> in**, and the narrowing is the substance of the decision. The user's rule was
> *"the python scripts that relate to work on the repo/design docs, that are not
> shipped with the final harness and not required by harness for work — we can
> leave as python."* Applied to all five tools, that rule leaves **nothing to
> port**, so §2.4 is not "Rust by default with recorded exceptions"; it is **two
> classes with a scope boundary between them**.
>
> **Six scoping decisions taken while applying it — recorded so a reviewer can
> check them rather than infer them:**
>
> 1. **The proposal's principle-5 sentence was false as drafted and is rewritten.**
>    It claimed *"Principle 5's no-external-runtime claim then covers the project's
>    own development and not only its output: one toolchain for contributors and
>    for CI."* Under the narrowed policy contributors and CI still need Python, so
>    §2.4 now **states the boundary** — principle 5 scopes to the product and its
>    build, not the authoring environment — and names the cost as chosen rather
>    than leaving an unqualified claim that silently covers half the project. Same
>    defect class as §3.7's "No gap, no duplicate" before A18, and as B7's advisory
>    `bash` allow-list.
> 2. **Authoring tooling is *out of scope*, not an exception awaiting a port.** The
>    proposal's framing ("a script in another language is permitted only while it
>    has no Rust home … and is deleted when that disposition lands") would have made
>    all five Python tools temporary. They are not. §2.4's last-paragraph exception
>    survives only in its genuine case: tooling that must run on an unbuilt
>    checkout, which is why doc validation stays a script.
> 3. **§2.1 and §2.2 were edited as the proposal's items 2 and 3 specify**, plus one
>    rule the proposal did not contain: **no crate depends on `xtask`**. Without it
>    a shipped crate could import build-time codegen, and the "ships nothing"
>    property would be unenforced.
> 4. **§14 gained two bullets** (the proposal's item 4). One makes `xtask` ordinary
>    Rust with a **golden-output test** for generated bytes, on the same reasoning
>    as the protocol's fixtures — silent drift in generated bytes is not caught by
>    the consumer. The other states plainly that authoring tooling is outside §14,
>    so the gap is recorded rather than discovered.
> 5. **The disposition table was deliberately NOT written into the spec.** It names
>    five repo scripts, which are not product surface. §2.4 requires a register and
>    points at one maintained alongside the implementation plan — the same move §2.3
>    makes for the class-1 dependency register — and ~~the table itself lives in B16's
>    dossier~~ **the table itself now lives in [Appendix G](G-authoring-tools.md)**,
>    the register §2.4 points at (TODO §6's U80, closed 2026-09-16 — a dossier is an
>    audit trail, not a home for current state). The spec carries zero markdown links, so the register is named in
>    prose, not linked.
> 6. **Doc 01 was not deepened.** M0 has not started, so the just-in-time rule
>    holds and the depth count is unchanged. ~~The proposal's item 3 ("doc 01 gains
>    the disposition table when it is deepened") is therefore still owed, and lands
>    at M0 rather than here.~~ **Item 3 is superseded by Appendix G:** doc 01 gains
>    a *pointer* to the register when it is deepened, not a copy of the table.
>
> **Version stayed v0.1 and the `Date` moved to 2026-09-16**, per the convention
> under the spec's `Status` field.

**New, raised by the user after the original pass** — *"for the harness, we might
not want to use/rely on python in the system (for internal tooling) as much as we
can — build rust-based functions."* Decision dossier: **B16**.

**Current.** Nothing. The spec governs what may enter the Rust dependency tree
(§2.3) and what the crates are (§2.1) and how they depend on each other (§2.2),
but says **no word about what language the repository's own tooling is written
in**. The only statement anywhere is `AGENTS.md`'s tech-stack row — "Python 3 +
PyYAML for repo scripts (`tools/`)" — which records a habit as though it were a
policy. That habit is now ~~1 738~~ **2 073** lines across five tools, three of
which parse YAML *(recounted at sign-off; the original figure was already stale, and
the growth — +335 in one session, all of it untested — is itself part of the
argument)*.

**Proposed — a new §2.4, sibling to §2.3.** Both answer the same class of
question (*what does this project permit itself*), and §2 already uses the `### N.M`
convention that A16 and A17 established.

> ### 2.4 Repository tooling
>
> Tooling that operates on this repository — link checking, agent-adapter
> generation, doc indexing, work tracking — is **Rust, in an `xtask` crate inside
> the workspace**. Principle 5's no-external-runtime claim then covers the
> project's own development and not only its output: one toolchain for
> contributors and for CI.
>
> A script in another language is permitted only while it has no Rust home. Each
> one carries a recorded disposition — port, retire, or replace by dogfooding a
> product feature — and is deleted when that disposition lands. **Once `xtask`
> exists, a new tool is never written in another language.**
>
> Where a tool duplicates a dacode feature, the feature wins: the repository uses
> dacode on itself rather than maintaining a parallel implementation.
>
> The one documented exception is tooling that must run on a checkout that has not
> been built. Doc validation is the case in point: if it requires a compiled
> workspace, the docs cannot be checked until the code builds, which inverts the
> intended order. An exception is recorded here with its reason, never left
> implicit.

**Why a spec section and not just an `AGENTS.md` line.** Because the claim it
makes is architectural, not stylistic: principle 5 currently holds for the shipped
binary and not for the project that ships it, and §2.3 set the precedent that
"what this repo permits itself" is spec content. It also gives the disposition
table a normative home, so a lingering Python script is a recorded exception
rather than an oversight.

**The four edits that follow from it.**

1. **`AGENTS.md`'s tech-stack row is corrected**, since it states the habit as
   policy. It becomes "Rust `xtask` (Python only where §2.4 records an
   exception)".
2. **§2.1's crate tree gains `xtask`** as a workspace member that ships nothing —
   the same seam B6's offline schema generator already assumes, so this does not
   invent a location.
3. **M0 scaffolds it.** [01](../part1-foundations/01-workspace-scaffolding.md)
   gains the disposition table when it is deepened; the crate itself is part of
   M0's workspace, not a later addition.
4. **§14's testing strategy** should say whether repo tooling is tested. Today the
   five Python scripts have no tests, which is how `attention.py` shipped a
   milestone heuristic that over-triggered and an edit that deleted a function
   definition while still passing a syntax check. Rust tooling gets `cargo test`
   for free; that is an argument for the port independent of the toolchain count.

**Depends on.** **B16**, and it must be **read alongside B10** — three of the five
tools parse YAML, so porting them either adds a YAML crate (`serde_yaml` is
deprecated upstream, and no YAML crate appears in
[Appendix F](F-c-dependencies.md)) or moves `agent-rules.yaml` and the tracker to
TOML, which is B10's subject. Deciding B16 without B10 lets B16 settle B10 by
accident.

**Interactions.**

- **B15/§4.8 turns two of the five tools into retirements, not ports.**
  `tasks.py` *is* the task tree and `attention.py` is a view over it, so porting
  them would build a second implementation of a spec'd product feature. The
  disposition is retire-and-dogfood — cheaper than porting, and the stronger form
  of the user's instinct. **Retiring `tasks.py` is blocked on §16.6** (standing
  goals), because that bug was found in `tasks.py` and an unfixed §4.8 would
  inherit it.
- **A17 (§2.3) is the precedent for the section's shape and its enforcement
  style**: a stated default, a named exception class, and a machine-checkable
  register. §2.4's register is the disposition table; its enforcement is that a
  script with no row is out of policy.
- **B6's `xtask`-style schema generator** already assumes this crate exists. A20
  makes that assumption explicit rather than incidental, and the two should share
  one crate rather than two.

**Recommend accept**, with the `check-links.py` dissent recorded in B16 — it is
the one tool where "must run on an unbuilt checkout" is a genuine argument for
keeping Python permanently, and §2.4's final paragraph exists to let that
exception be stated rather than fought over later.

---

### A21 — §14 → §14.1/§14.2: benchmarking as a first-class concern — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18.** *"yes, let's do split"* — §14 is now **§14.1 Correctness** and
> **§14.2 Benchmarks and evaluation**, with all six proposed items landed and every link
> stripped as the transcription warning required. **It arrived later than planned and that
> changed what it was for**: A21 was drafted when §14 was a flat five-bullet list and
> nothing cited a subsection, and by the time it landed **the spec already referenced
> `§14.2` twice** — §3.2's `Usage` paragraph and §4.5's partial-compaction paragraph, both
> written by A4 — so those were **dangling forward references in the spec**, and applying
> A21 is what made them resolve.
>
> **Three scoping decisions taken while transcribing.**
>
> 1. **§14.1 is not "the existing text unchanged", because the existing text more than
>    doubled since this was drafted.** A3, A4, A5, A9, A15 and A10's carve-out each added
>    tests, so §14.1 carries **twelve** bullets rather than five — the eleven that had
>    accumulated plus the `Clients` bullet this delta itself adds (**thirteen once A22
>    added `Trust boundary` later the same day, fourteen once A7 added `Config writes
>    and the registry hint`**). The instruction was
>    right in intent (do not rewrite the correctness suite while adding a benchmarking one)
>    and wrong in fact, so it was followed as intent.
> 2. **A twelfth bullet was added that this delta does not propose: `Clients`.** **U101**
>    recorded that §14 had **no UI test tier at all**, so A9's two client rules were
>    normative with nothing that could fail on them, and named A21 as owner. Both properties
>    are now stated as tests — mouse tracking off unless an overlay holds the alternate
>    screen, and collapse-on-paste honouring its thresholds with an indivisible chip —
>    **with the harness deliberately not named**: the TUI crate does not exist and §9.1's
>    framework decision is open, so choosing a backend now would be choosing it for a crate
>    that has not been written. What is fixed is the properties, which outlive any harness.
>    **U101 closes.**
> 3. **The four inbound assignments were discharged inside §14.2 rather than left as
>    notes.** A15 required the compression **level** be named as a measured knob, since B12
>    settled the codec and what remains is which level trades ratio against latency — so the
>    default is a number tier 1 produces rather than a value someone reasoned about. A10
>    required §6's **`ExchangeRecord`** be named as what hit rate is computed *from*, which
>    is the sentence that explains why the record was an M1 deliverable while the analyzer
>    is M5: a metric cannot be retrofitted onto exchanges nobody recorded. U97 required
>    §4.5's partial compaction be gated on the hit-rate number existing, so its "candidate
>    not feature" status now has a named instrument. And B18's publication rule is stated
>    **normatively and first**, because everything below it depends on it.
>
> ⚠️ **This delta's own `Current.` list is now stale in three places, and was corrected
> rather than transcribed.** It describes §3.6/§12's codec as "deliberately unnamed and
> stalled on a measurement nobody can take" — **A15 named it `zstd` and B12 resolved it**.
> It cites §4.5's "compaction at 80 % of the window" as an unmeasured assertion — **A4 made
> that opt-in and explicitly not v1's default**. And it says §14 is a flat five-bullet list;
> it was eleven before this pass touched it. Its knock-on claim that "**B12 stops being a
> stalled vote**" is likewise historical: B12 was resolved by *decision* on 2026-09-17, so
> §14.2 tunes the **level** rather than producing the codec number B12 was waiting for.
>
> **One new gate came out of this pass rather than out of the delta.** The dangling `§14.2`
> references were invisible to every existing check, and the reason is structural: the spec
> is forbidden markdown links, and `check-links.py` validates links and anchors — so **the
> convention that keeps the spec clean had also removed the only mechanism that could catch
> a bad internal cross-reference**, in a file carrying 290 of them. `check-links.py` now
> has `check_section_refs()`, bound to the same `NO_LINK_FILES` list for exactly that
> reason, and it was **tested by injecting three dangling references** (`§14.9`, `§99.3`,
> `§2.7`) and confirming it named those three, ignored the two valid ones beside them, and
> exited 1 — then passed again on restore. A gate nobody has watched fail is a gate nobody
> can trust.
>
> **Debt this creates.** **Doc 33** carries the corpus contents, the scorer implementations
> and the recorded environments, and is still a summary — §14.2 says so explicitly and
> names it in prose rather than by link. **B18's tiers** are named in §14.2 by *property*
> (deterministic-in-CI, token/cost, task-success-out-of-CI) rather than by their T1/T2/T3
> labels, so doc 33 owns the mapping. **No wire change, no fixture, no new type** — which is
> why this was the cheapest delta in the set, as its own entry predicted.

**Depends on [B18](B-decisions.md).** ~~T2's cache-hit-rate metric additionally
needs **A4**'s `Usage` cache fields to exist; T1, T3 and the publication rule do
not, so A21 can land ahead of A4 with T2 stated as a requirement rather than a
formula.~~ ✅ **The ordering this anticipated did not happen, and the result is
better**: A4 landed *first*, so §14.2's hit-rate tier is stated against fields that
exist (`Usage`'s four counters) rather than as a requirement awaiting them. It also
gained a dependency nobody recorded — **A10's `ExchangeRecord`**, which is what the
hit rate is computed *from*, and which is why that record was an M1 deliverable while
the analyzer that reads it is M5.

**Current.** §14 is titled *Testing strategy* and is a flat five-bullet list —
protocol fixtures, engine `MockProvider` sequences, tool sandboxes, a server
integration test, the echo extension. Every item is a **correctness** test: it
passes or it fails. There is no notion of a measured property anywhere in the
spec, and yet the spec asserts several:

- §13.4 — byte-exactness exists so "the prompt-cache stable prefix survives".
  Nothing measures whether the cache hits.
- §13.3 — tool-schema churn "*including pure reordering*" destroys that prefix.
  A golden fixture catches a schema *change*; nothing catches the cache
  *consequence*.
- §3.7 — a 256-envelope queue and a 5 000-event window, "one invariant, not two
  independent tunables". Neither number is derived.
- §4.5 — compaction at 80 % of the window, on a token *estimate*.
- §4.8 — `reminders_max` default 3.
- §3.6/§12 — the log's compression codec, deliberately unnamed and stalled on a
  measurement nobody can take.

**Proposed.** Restructure §14 as `### 14.1` / `### 14.2`, matching the `### N.M`
convention §2 and §13 adopted when A17 and A16 landed. §14.1 is the existing text
unchanged. §14.2 is new:

1. **State the distinction.** Correctness tests pass or fail; benchmarks produce
   numbers that must be compared against a recorded baseline. Both are required;
   neither substitutes for the other.
2. **Name the deterministic tier as a CI gate.** Benchmarks that need no provider
   — fold throughput, snapshot latency, log append and codec throughput, fan-out
   cost, memory per session — run against a **committed event-log corpus**, since
   the log is already the fixture format and `MockProvider` already replays canned
   chunks.
3. **Name prompt-cache hit rate as a measured property of the engine**, alongside
   the harness's own token overhead before the first user token. This is the one
   addition that changes behaviour rather than recording intent: it converts
   §13.4's rationale from an assertion into something that can fail.
4. **State the publication rule normatively**: *a number may not be published,
   quoted in a specification, or used to justify a decision unless its scorer is
   in the tree, re-runnable, and its environment recorded alongside it.*
5. **Record task-success evaluation as deliberately out of CI** — pinned model and
   date, N repetitions reported as a distribution, invoked by hand — and note that
   its driver is `run -p --output jsonl` and its pass/fail primitive is §4.8's
   `Verify::Command`, so it needs **no new protocol surface**.
6. **Record cross-harness comparison as a non-goal**, so it is not relitigated.

**Why a spec change rather than a plan doc.** Three of the six numbered assertions
above are *in the spec*, and two of them (§13.3's reordering hazard, §3.7's paired
limits) are stated as invariants the design depends on. An invariant the spec
asserts and no artefact checks is the exact defect A18 was written to fix in §3.7
— the "No gap, no duplicate" claim that was asserted rather than earned. §14 is
where that gets fixed for the numeric claims, and the plan doc
[33](../part6-quality/33-benchmarking.md) then says *how*.

**Blast radius.** §14 only; no `Event` variant, no wire change, no fixture. §15's
roadmap gains nothing — the work attaches to existing milestones. ~~The one knock-on
is that **B12 stops being a stalled vote**: §14.2's deterministic tier names codec
throughput as a measured property, which is the number B12 has been waiting for.~~
⚠️ **Historical**: B12 was resolved by *decision* on 2026-09-17 (`zstd`, the
policy's first class-1 exception) and A15 applied it, so tier 1 tunes the **level**
rather than producing a codec number. **The blast radius was also understated in one
direction**: §14 gained a `Clients` bullet this entry does not propose, because
**U101** named A21 as the owner of the missing UI test tier — so the radius is "§14
plus one debt discharged", still with no protocol surface.

**Watch the link convention.** The proposed text above is *plan* text and cites
Appendix F, doc 33 and B12 by link. **Strip every link when transcribing** —
`grep -c '](' docs/dacode-design.md` must return 0. Name doc 33 as "the
implementation plan's benchmarking document" in prose, as A17 did for Appendix F.
✅ **Honoured** — `grep -c ']('` returns 0, and §14.2's closing paragraph points at
the plan's benchmarking document in prose. **This warning turned out to be pointing
at a bigger hole than the one it names**: the convention that keeps the spec
link-free had also removed the only mechanism that could catch a bad *internal*
cross-reference, and A4 had left two dangling `§14.2` citations that nothing could
see. `check-links.py` now checks those too; see the banner.

**Recommend accept.** It is the cheapest delta in the set — one section, no
protocol surface — and it is the only one that makes an existing spec claim
falsifiable. **Done 2026-09-18**, with the three scoping decisions in the banner and
**one new standing gate** as a by-product. **The `Current.` and `Proposed.` text below
is historical** — it describes a five-bullet §14 and a codec nobody had chosen, and its
list of unmeasured spec assertions has since been shortened by A4 and A15 landing.

---

### A22 — §3.3, §4.4, §5.3, §7.1, §7.2, §8.2, §8.3, §8.4, §9.2, §14: close four capability seams — ✅ **APPLIED 2026-09-18**

> **✅ APPLIED 2026-09-18, with B7's `yolo` confirmation landing in the same pass as this
> banner required** — so §4.4 was edited once rather than twice. All four seams are closed
> and **ten spec sections** carry the result against the six this entry named, because
> item 4 reaches §9.2's renderer and every item needs §14 tests. **Two questions this
> banner had left open were put to the user and answered 2026-09-18**, and one of the
> answers reverses this banner's own reading.
>
> **Three scoping decisions, the first of which is a reversal.**
>
> 1. ⚠️ **The `AllowPattern` bound IS applied — this banner's orchestrator reading was
>    wrong and the user corrected it.** The banner recorded the pattern-subset bound as
>    part of what "permissive" declines, reserved with the capability flags rather than
>    applied, and flagged it for redirect *"since it costs a user nothing and only
>    constrains what an extension may grant on the user's behalf."* **Redirected: the bound
>    is applied.** The argument that settled it is the one the flag anticipated —
>    *permissive* is about the **user's** experience, and the bound changes none of it,
>    since a user at their own terminal can still grant `**` explicitly. What it constrains
>    is a **client acting on the user's behalf**: without it, §3.3's `AllowPattern(String)`
>    plus §4.4 putting session grants at the *top* of the resolution order plus §8.4 letting
>    extensions resolve permissions compose into "one bridge answering one prompt can grant
>    `**` for the session", which makes any connected extension root-equivalent. That is not
>    permissiveness, it is the unreviewable three-section composition this delta was
>    promoted out of §6 prose to fix. Mechanically checkable at resolution time because the
>    request is already in `PermissionRequested { scope }`; a pattern outside it is an
>    `Ack Err`.
> 2. **The capability split stays reserved, as redirected.** §8.4 now states the trust model
>    **explicitly** — *a connected extension is as trusted as the local terminal* — with the
>    reason it is deliberate (a phone that can report a session blocked but cannot unblock
>    it is a pager, not a client) and the reason it was a defect before (it was true only by
>    composition of three separately-written sections, which is unreviewable). §7.2's `Hello`
>    **reserves** `answer_questions` and `resolve_permissions`, both granted by default in
>    v1, so tightening later is additive and not a wire break — B7's inert-stub pattern
>    applied to a trust model. The two flags are separate because the acts differ in kind:
>    answering a question supplies data, resolving a permission authorises an effect.
> 3. **B7 confirmed A23's `yolo` carve-out, and the user added a notice.** The carve-out
>    stands: under `yolo` a detection is recorded without escalating, because a mode whose
>    whole meaning is that the user opted out of being asked cannot start asking, and
>    auditability is not lost since the detection is still on `PermissionResolved`. **But it
>    does not stay silent either** — `yolo` is the mode where a leak is *most* likely
>    (everything runs automatically, so nothing passes a human on the way to exec) and
>    therefore the one mode where an escalation-based warning would never fire. So the first
>    detection in a session raises a **non-blocking notice**: rendered once, does not pause
>    the turn, requires no answer, not repeated. Once per session rather than per detection,
>    because a notice on every occurrence in a mode chosen for not being interrupted *would
>    be* the interruption.
>
> **Items 1, 3 and 4 landed as confirmed, each with the reason it is a rule rather than a
> preference.** **Item 1**: `run` is an argv array executed without a shell, and
> **templating survives into argv elements** — deleting it would have been safer and less
> useful, while the array form keeps the convenience and removes the surface. A hook that
> wants a shell must write `["sh","-c",…]`, which is the point: the dangerous case becomes
> visible in the config rather than being the default behaviour of any hook that
> interpolates. §7.1 now states *why* this is a rule and not an example-level detail — the
> example's own fields are client-controlled, but the natural fields to interpolate on
> `QuestionAsked`/`PermissionRequested` (the prompt, a tool name, an error message) are
> **model-authored**, so `"{{ prompt }}"` in a shell string is a model-reachable
> command-injection bug and nothing about the field name warns the author. **Closes U61.**
> **Item 3**: inheritance is an **intersection** — the child's effective scopes are its own
> resolved scopes ∩ the parent's, mode `min(child, parent)` — which makes §5.3's *existing*
> promise true on both axes instead of only the mode axis, and is not a restriction on an
> operator who can set the parent wide. **Item 4**: `Referrer-Policy: no-referrer`
> origin-wide plus `rel="noopener noreferrer"` as a *renderer* rule so it covers any surface
> rendering model output; `?token=` documented as bootstrap-only and stripped once the
> cookie is set; `/api/sessions/:id/file` and `/blob/:blob_id` served as `text/plain` with
> `Content-Disposition: attachment` under a CSP forbidding inline script, because both return
> arbitrary repository or tool content **on the authenticated origin** and are therefore a
> stored-XSS path. §8.3 records that none of this costs capability, which is why it is not
> part of the permissive trade-off: what stops leaking is dacode's **own** bearer token.
>
> **Debt this creates, recorded rather than left to the next reader.** **A25's M5 half**
> inherits §8.4's trust model and must argue remote principals **on its own merits**, since
> the capability split no longer arrives from here — the reserved flags are the mechanism it
> should use. **A7** owns the §8.5 writable-path allowlist and "who may `promote` from a
> browser", and should answer it together with A14's **U99** registry question rather than
> separately, because §8.4 now says the rule for non-session writes should be decided once.
> **Doc 16** (permissions) and **doc 19** (server gateway) are the plan-side homes for items
> 2 and 4 and are both still summaries. **U62, U63 and U65** were checked against this pass
> rather than assumed: see the board for which it discharges and which it does not.
> **No `Event` variant and no fold change, so no golden fixture** — item 2 changes a
> `Command`'s *validation*, not its shape, and the reserved flags are an additive field on
> `Hello`, exactly as this entry predicted.
> ---
>
> **The signed-off record follows, preserved verbatim except where it now contradicts
> what landed.** It is kept because the reasoning is the audit trail — but read it against
> the banner above, which supersedes it on one point.
>
> ~~**Item 2 (`AllowPattern` bound + capability split) is NOT taken for v1.**~~ ⚠️
> **Superseded on the bound, 2026-09-18: the capability split is indeed reserved rather
> than applied, but the `AllowPattern` subset bound IS applied** — the user redirected
> exactly the reading this paragraph flagged. The rest of this block stands as the reason
> §8.4 needed the trust model written down. A
> permissive first version means a connected extension may resolve permissions and may
> grant a session-wide pattern, which is what §8.4 already says. But the defect A22
> named was never the *choice* — it was that the choice is **implied by three
> separately-written sections composing badly**, and therefore unreviewable. So the
> transcription owes two things instead of a restriction: **(a)** §8.4 states the trust
> model **explicitly** — *a connected extension is as trusted as the local terminal; it
> may answer questions, resolve permissions and grant session-wide patterns; this is
> deliberate for v1 and is the reason a bridge is not a notification-only surface* — and
> **(b)** `Hello { capabilities }` **reserves** the `answer_questions` /
> `resolve_permissions` flags now, granted-by-default in v1, so tightening later is
> additive and not a wire break. That is B7's inert-stub pattern applied to a trust
> model: permissive now, no refactor later. *(Recorded as the orchestrator's reading:
> the pattern-subset bound — `AllowPattern` ⊆ the requested scope — is part of what
> "permissive" declines, and is reserved with the flags rather than applied. Redirect
> this if the bound was meant to survive, since it costs a user nothing and only
> constrains what an extension may grant on the user's behalf.)* ✅ **Redirected, and the
> redirect is the better answer: the bound costs the *user* nothing because they can still
> grant `**` from their own terminal, and it constrains only what a client acting on their
> behalf may infer from one request. "Permissive" was about the user's experience, and the
> bound does not touch it.** Note (a)'s phrase "grant session-wide patterns" is now bounded
> by §4.4 — session-wide, but never wider than the request that opened the grant.
>
> **Items 1, 3 and 4 were each confirmed individually** ("1. Hooks - ok. 2. Sub-agents -
> ok. 3. Ok."), in the forms proposed here, and none of them is a permission-model
> choice:
>
> - **1 — hooks:** `run` takes an **argv array executed without a shell**, and
>   **templating survives into argv elements** rather than being deleted. With no shell
>   there is nothing to inject into, so the convenience stays and the injection surface
>   goes; a hook that genuinely wants `sh -c` must say so explicitly, which is the point
>   of making it visible. `post` gains the redaction rule (or the plain statement that a
>   `post` hook sees the whole envelope and must target a trusted endpoint).
> - **3 — sub-agent inheritance:** the rule becomes an **intersection** — a child's
>   effective scopes are its own resolved scopes ∩ the parent's, and its mode is
>   `min(child, parent)`. This is not a restriction on the operator, who can set the
>   parent wide; it makes §5.3's **existing promise** ("a child cannot exceed its
>   parent") true for scopes and not only for modes, closing the case where a child's own
>   agent file carries `edit.allow = ["**"]` under a parent scoped to `src/**`.
> - **4 — web-UI hygiene:** `Referrer-Policy: no-referrer` plus `rel="noopener
>   noreferrer"` on rendered links, `?token=` stripped once the cookie is set,
>   `/api/sessions/:id/file` and `/blob/:blob_id` served as `text/plain` with
>   `Content-Disposition: attachment` under a CSP that forbids inline script, and `Bearer`
>   documented as the primary path with `?token=` as the bootstrap-only convenience it
>   is. No capability cost: this stops dacode's **own** bearer token leaking through the
>   `Referer` of a page that renders model-authored markdown.
>
> ~~**Still owed by the transcription:** the §4.4 half is **B7's**, not A22's, and B7 is
> now signed — so A22's application and B7's confirmation of A23's `yolo` carve-out
> should land together rather than twice.~~ ✅ **They did land together, 2026-09-18** —
> §4.4 was edited once, carrying both the `AllowPattern` bound and B7's confirmation of the
> `yolo` carve-out plus the user's non-blocking-notice addition. **A23's debt is thereby
> discharged** and B7's entry records it. No `Event` variant and no fold change, so **no
> golden fixture**; item 2's reserved flags are an additive field on `Hello` — ✅ **and that
> prediction held exactly**.

**Depends on nothing.** Every item below is a *correction* — the spec presents
something as a boundary that does not hold — so there is no fork to sign off
first, only whether the correction is right. That makes A22 the same shape as A17
and A18, both of which landed on their own.

**Provenance.** Found in the session-8 third-party review and first recorded as
prose bullets in [TODO](../TODO.md) §6. **They are promoted here because a §6
bullet is not sign-off-able**: it appears in no delta sweep, has no id to cite or
close, and is exactly how §16.6 became invisible. A spec change belongs in this
appendix; the §6 entries now point here.

**These are not hypotheticals.** The same class was already caught once, in
§4.8's `Verify::Command` — model-authored argv the engine would have run. That one
was patched back into the artefact before sign-off. These four were not caught,
and three of them predate it.

#### 1. §7.1 — hooks interpolate into a shell command

**Current.** *"`run` gets the envelope on stdin and templated args"*, with the
example `run = "notify-send 'dacode' '{{ session.title }}: {{ reason }}'"` — a
string interpolated into a command line.

**Stated precisely, because the strength of the claim matters:** the example's own
fields are client-controlled, not model-controlled. But the **mechanism is
general**, §7.1's second example fires hooks on `QuestionAsked` and
`PermissionRequested`, and the natural fields to interpolate *there* — the
question `prompt`, a tool name, an error message — **are model-authored**. A hook
author who writes `{{ prompt }}` has written a command-injection bug, and nothing
in §7.1 warns them.

**Proposed.**
- `run` takes an **argv array, not a shell string**, executed without a shell.
- **Delete templated args entirely.** The envelope already arrives on stdin,
  which is the complete mechanism; templating adds an injection surface and no
  capability. Hook scripts parse stdin, as `dacode-ext-*` already must.
- `when` keeps its expression language — it is evaluated, never executed.
- State a redaction rule for `post`, which currently ships the **whole
  `Envelope`** to an arbitrary URL while §8.2 redacts secrets on `/api/config`.
  Either the same redaction applies, or §7.1 says plainly that a `post` hook sees
  everything and should only target a trusted endpoint.

#### 2. §3.3 / §4.4 — `AllowPattern` is unbounded, and §8.4 hands it to extensions

**Current.** §3.3 has `ResolvePermission { decision: AllowPattern(String) }`;
§4.4 puts session grants at the **top** of the resolution order, above agent,
project and user config; §8.4 says extensions may answer permissions *"regardless
of control"*, deliberately, *"because that is their purpose"*.

Compose those three and a connected bridge answering one prompt can grant `**`
for the session. Any extension is therefore root-equivalent, which is not what
§8.3's "no users, no roles" model intends — it intends *one trusted operator*, and
a Telegram chat is a weaker authentication context than the local terminal.

**Proposed, two separable edits.**
- **Bound the pattern.** `AllowPattern` must be a **subset of the scope actually
  requested**. The request is already in `PermissionRequested { scope }`, so this
  is mechanically checkable at resolution time; a pattern outside it is an
  `Ack Err`. This keeps the feature — "allow all `cargo *` for this session" —
  and removes the escalation.
- **Split the capability.** §7.2's `Hello { capabilities }` gains separate flags
  for *answer questions* and *resolve permissions*. Answering a question is
  supplying data the model asked for; resolving a permission is authorising an
  effect. Bridging the first does not imply bridging the second, and the default
  for a chat bridge should be questions only.

#### 3. §5.3 — sub-agent inheritance caps the mode, not the scopes

**Current.** *"its own permission state (child cannot exceed parent's mode)"* —
and `PermMode` is only `Plan | Ask | AutoEdit | Yolo`. The allow/deny **lists**
are per-agent config, so a child whose own agent file carries
`edit.allow = ["**"]`, spawned from an `auto-edit` parent scoped to `src/**`, is
constrained by neither.

**Proposed.** State the rule as an **intersection**: a child's effective
permissions are the intersection of its own resolved scopes with the parent's,
and its mode is `min(child, parent)`. One sentence, and it makes §5.3's existing
promise true rather than partial.

#### 4. §8.3 / §8.2 — the browser token travels in a URL, on the origin that renders model output

**Current.** *"Accepted as `Authorization: Bearer` or, for browsers, `?token=` on
first visit which sets an HttpOnly, SameSite=Strict cookie."* The cookie handling
is right; the delivery is not.

A query token lands in browser history, in every proxy and server log, and in the
`Referer` of **any outbound link the page renders** — and §9.2 renders
model-authored markdown, so a transcript containing an external link is one click
from leaking it. Separately, §8.2's `/api/sessions/:id/file` returns arbitrary
repository content **on the authenticated application's own origin**, which is a
stored-XSS delivery path.

**Proposed.**
- `Referrer-Policy: no-referrer` on the app, and `rel="noopener noreferrer"` on
  every rendered link.
- Strip `?token=` from the URL once the cookie is set.
- Serve `/api/sessions/:id/file` and `/blob/:blob_id` as `text/plain` with
  `Content-Disposition: attachment`, under a CSP that forbids inline script.
- §8.3 keeps `Bearer` as the primary path and documents `?token=` as the
  bootstrap-only convenience it is.

#### Not in A22, on purpose

**§4.4's `bash` allow-list is advisory for exactly the reason B7 gives for
`bash:readonly`** — a shell command is a program, and a glob over its text cannot
decide what it does (`git diff; git push`, `cargo run` invoking anything,
`$(...)`). That is not a correction to write here, because **B7 already owns the
advisory-versus-enforced wording and is unsigned**; A22 would be pre-empting it.
B7's dossier has been widened to cover §4.4 instead, so one decision answers the
whole question rather than two artefacts answering halves of it.

**Blast radius.** No `Event` variant and no fold change, so **no new golden
fixture**. Item 2 changes a `Command`'s *validation*, not its shape. Item 1
changes a config schema that no code reads yet — which is the argument for landing
it now: every one of these is free before the thing exists and awkward afterwards.
Item 4 is server configuration only. ✅ **All four held, and the blast radius was
understated in one direction only**: ten sections were touched rather than the six
this entry names, because item 4's `rel="noopener noreferrer"` is a *renderer* rule
belonging in §9.2 (so it covers any surface rendering model output, not just one
view), item 2's bound needed §3.3's `PermDecision` comment as well as §4.4, and
every item needs §14 tests. No new section, no new type, no wire shape changed.

**Watch the link convention.** The text above is *plan* text and cites TODO and
B7 by link. **Strip every link when transcribing** — `grep -c '](' docs/dacode-design.md`
must return 0. ✅ **Honoured** — returns 0, and the §-reference check that A21's pass
added reports 307 resolving.

**Recommend accept.** If any single item is contentious it is item 2's capability
split, because §8.4 grants that power deliberately and the user may consider the
threat model out of scope for a single-operator tool. Say so explicitly if that is
the call — *"a bridge is as trusted as the terminal"* is a defensible position, and
writing it down is much better than leaving it implied by a composition of three
sections that were each written separately. **Done 2026-09-18, and the user did make
that call explicitly**: the position is defensible and is now written down, with the
capability split reserved rather than applied — but the `AllowPattern` bound was
**redirected back in**, on the argument that "permissive" describes the user's
experience and the bound does not touch it. **The `Current.`/`Proposed.` text below is
historical**; the banner is the record of what landed and why.

---

### A25 — §3.1, §3.6, §3.7, §4.7, §4.8, §5.3, §5.4, §8.4: remote-execution constraints — ✅ **SIGNED OFF 2026-09-17**

> **✅ SIGNED OFF 2026-09-17 with [B20](B-decisions.md) — approved for application; NOT
> yet transcribed into the spec.** This entry did not exist before B20's session: it is
> the transcription B20 owes, and B20's own "on session" list named the sections. It is
> **split by milestone**, because half of it is wire format and half is not, and the two
> halves have opposite urgency.

**M1 half — wire format, and it rides with A1.** ✅ **APPLIED 2026-09-18, inside A1's
transcription** — see A1's banner for the scoping decisions, two of which are this
half's: (a) took a **third answer** beyond the two options listed below, and (e) put
**only the epoch** on the wire. Free before golden fixtures exist, a rewrite after:

- **(a) A `SessionId` that can name a session on another host** — a location component or
  an opaque handle. §3.1's ids are ULID-backed and A23's scan depends on that, so
  whatever is chosen must keep the four ids exact-matchable.
- **(e) A fencing token on the session writer.** Not "one `seq` assigner" but **one
  committer, fenced by an epoch, deduped by an idempotency key, with a CAS on the leaf** —
  what openclaw's transcript commit does (`worker-rpc-transcript-client.ts:31-56`,
  `transcript-commit.ts:44,188-216`) and what opencode states in prose
  (`sync/README.md:31-33`). **B21's kernel-owned `flock` cannot do this remotely**: there
  is no shared filesystem to lock, and B20's Q1 permits a network FS where `flock`
  semantics are NFS-version-dependent. B21's single-writer invariant is *restated*, not
  weakened — the lock stays the local mechanism, and the epoch is what a partitioned
  remote writer cannot forge.

**M5 half — behaviour, none of it wire format.**

- **§5.3** — a child session's `cwd`/sandbox inheritance is meaningless across hosts, so
  the child gains a **location**; and the model endpoint becomes a **host/session
  attribute**. That attribute is B20's Q5 = **option A**: the member runs
  `dacode-providers` and points its base URL at a controller-side **credential broker**
  (A10/doc 25's proxy) which injects the key and **forwards verbatim**. The
  attribute form is what keeps option B (controller-executed inference) reachable later
  with no wire change, and *forward verbatim* is non-negotiable either way — a proxy that
  re-serialises destroys the cache prefix it exists to measure (§13.4).
- **§5.4** — a workflow is a **DAG**: a step gains `host` (absent = local), `needs` and
  `parallel`; **the swarm shares one §4.8 task tree**, because a gate per host is not a
  gate; remote fan-out gets **its own cap**, separate from `max_parallel_subagents`; and
  `task` may also name a host for ad-hoc delegation. A step that opts into
  **retry-on-host-loss must declare its effect class**, honoured only when the step is
  idempotent or its `verify:` can detect a double run — the default is *block and ask*,
  because no surveyed harness has idempotence keys.
- **§8.4** — capabilities for a **remote principal**, and this is the part that must be
  argued rather than inherited: **A22's item 2 was redirected to permissive for v1**, so
  the split no longer arrives from there. The minimum set the survey supports is
  *observe / prompt / execute / spawn / grant-permissions*, with **spawn and
  grant-permissions never held by a remote principal** — openclaw gateway-proxies both
  (`src/worker/tool-authority.ts:1-28`). Local permissiveness is untouched; this is about
  a process on another machine.
- **§4.8** — `Verify::Command` evidence records the **executing host**, without which a
  green gate is unattributable and Q6's block-and-ask cannot tell the human where to look.
- **§4.7** — sandbox backends become **declared host capabilities negotiated at
  admission** (`worker-admission.ts:47-69` is the precedent; openclaw's
  `backend-handle.types.ts:4` is B7's reserved seam already built), jointly with **B7**. A
  host that cannot declare one **refuses members** — and since B7 ships the seam as an
  **inert stub**, nothing can declare one until it is filled (**U98**).

**Not in this delta.** The build itself (M5, beside doc 20). Forge's *remote services,
local effects* form — declined, because A10 covers the only remote service dacode plans
and **hosted semantic indexing is an explicit non-goal**. And anything that would let a
member see less of the conversation: the survey is blunt that neither brokering form
hides the transcript from the member, so an untrusted host is a **containment** question,
not an inference-placement one.

**Depends on.** ~~B20~~ ✅ resolved. ~~The **M1 half must land with A1**~~ ✅ **it did,
2026-09-18**; the M5 half needs
**A3** (synthetic tool results plus an *aborted / effects unknown* outcome in §3.2,
without which Q6's default cannot be expressed) and **A4** (`Usage` cache fields, so a
member's spend rolls up correctly rather than pricing cached reads at full rate — U71).

**Watch the link convention.** The text above is *plan* text and cites B20 by link.
**Strip every link when transcribing** — `tools/check-links.py --conventions-only` is the
authority, not a raw grep.

**Recommend accept** — signed with B20 rather than separately, because the two are one
decision: B20 chose the shape, A25 writes it down.

---

## Deltas applied

Applied entries stay in place above, struck through in the index with a
resolution banner in the detail section — nothing is deleted.

| # | Section | Applied | Date | Note |
|---|---|---|---|---|
| ~~**A28**~~ | §3.2, §3.3, §4.3, §4.4 | **A denial carries a typed reason, and §4.4's pattern language is corrected** — `DenyReason` with five arms, `Deny { reason: Option<DenyReason> }`, the three pattern *domains* stated, `.env` → `**/.env`, and a `deny` made a floor no grant or mode overrides | 2026-10-01 | The register's third entry to arrive already applied, and its **first with a golden diff**: one string moved and the pre-change bytes no longer decode. Landed before M1.3 publishes, which is [U162](../TODO.md)'s deadline rather than M1.7's convenience |
| ~~**A27**~~ | §3.2, §3.5, §4.5, §8.2, §14.1 | **Four durable events with no fold destination, given one** | 2026-09-20 | The register's **second** entry to arrive already applied, found by **M1.2 Pass C1's fold walk** — the first time `fold` was actually written, which ran A1's tier rule as a diagnostic: *every `Durable` variant must have a fold destination, or the rule's own "iff" is false*. Four did not, and each fix is the minimum that makes an existing spec claim true rather than a new mechanism: **`TurnBlock::User { content, from }`** appended per `UserMessage` — without it a G4-bootstrap or replay client renders every transcript with no user side, and §9.2's "reconciled by `UserMessage{from}`" line presupposed the destination; **`SettingsChanged { model?, agent?, perm_mode? }`** (Event 30 → 31, `Durable`) plus `SessionCreated`'s two new fields — §3.3's `Set*` commands promise "resulting events" and §3.5 says `SessionInfo` is "all derived from the log", and both were false for `model`/`perm_mode`; **`SessionInfo.closed: Option<CloseReason>`**, first close wins — §8.2's list annotation has always said "(**status**, …)" and U109's rationale ("the session list *says* a session is over") presupposed it; **`Compacted` folds `context.tokens_used = after`**, with §4.5's divider made explicitly a live-rendering affordance. Plus **two fold rules stated where a second implementer (M1.3's TS client) meets them**: openness is a convention, not a flag — the open block is the turn's trailing same-kind block, exact under §6's flat stream (tracker f124); and "no open block" splits by turn existence — unknown turn ignored (totality), started turn *creates* the block (the half log-only replay depends on — tracker f126, correcting G5's own §14.1 transcription). **Three verified non-gaps** recorded rather than fixed: `SubAgentFinished` (summary flows via the parent's `ToolResult`, §5.3), `TaskUpdated{Promote}` (repo-side write; content is M1.10's), `Checkpoint` (**U112**, filed with M2/doc 12 as owner — no route, no `Snapshot` field, and a `Rollback` UI needs the ids from somewhere). Approved in-session on the recommendations; **again pre-fixture**, so the wire change costs zero golden diff — A26's sequencing rule earning its keep a second time |
| ~~**A26**~~ | §3.2, §3.5, §5.3, §14.1 | **A turn is an ordered block list, and a child session closes with `Completed`** | 2026-09-19 | The register's first entry to arrive **already applied**: it came out of M1.1a's review of the three protocol types `bc52e50` *designed* rather than transcribed, which the previous session's handover flagged as needing a second pair of eyes before M1.2 fixtures them — and **two of the three changed**, which is the gate earning its keep, since no structural check can tell whether a variant *set* is the right one. `TurnView.text: String` could not hold a turn with more than one text block while §6's `StreamChunk` is a flat sequence, and §3.2's replace-at-boundary rule then destroyed the first block — **a loss in durable state, not a mis-render**, because deltas are `Live` and never persisted. It is now `blocks: Vec<TurnBlock>` with the rule reworded to *closes the open block*, which needs **no block index on the events**: arrival order *is* block order, the boundary is authoritative, and a dropped close is a §3.7 `seq` gap rather than a silent merge. `CloseReason` gained `Completed` because a sub-agent whose task finished matched none of its four producers, and §5.3 — which never said whether a child session closes at all — now says so, carrying the **child-specific** warning that makes the variant safe (a primary session goes `AgentIdle(Done)` and stays open, and §11's exit codes map from `IdleReason`, so a leak would be visible only to a user reading a session list). **Neither change cost a fixture**: no `Event` variant changed shape, and `Snapshot` is §8.2's payload rather than a log record. Two §14.1 tests added, the first invisible to the two pre-existing two-tier tests because each uses a single-block turn. **The sequencing rule that falls out** is now in doc 03 §3.7: an `Event`/`Command` variant is fixture-pinned from M1.2, a folded type route-pinned until M1.3 — so review the first kind earlier |
| ~~**A7**~~ | §8.2, §8.3, §8.4, §8.5, §3.6, §3.7, §5.1, §9.2, §12, §14 | **Config writes, the scaling axis, and one rule for every non-session write** | 2026-09-19 | **Ten sections against the two this entry named**, because caveat 1 needed §8.5 *and* §8.2's write rule, caveat 2 needed §5.1 (where §12 already cited it and no section carried it), the post-A19 amendment needed §8.4 and §9.2, and **U99**'s freshness channel needed a wire frame (§3.6) plus a field (§12). **§8.2 grew from 16 routes to 23** and now carries the scaling axis, which is what makes §2.2's and §3.7's existing citations of "§8.2's scaling axis" resolve to *content* rather than to a heading — a resolvable reference with false content is the `§14.2` hazard class one level subtler, and `check_section_refs()` cannot see it. Both trait families are normative with `BlobStore` **reconciled in as the third** (§2.1 named three, this entry named two), `JsonlStore`+`SharedToken` the only M1 implementations, and **`Authenticator`/`Tenant` placed in `dacode-server` with no new crate** on §2.1's own test (no engine rule depends on identity). ⚠️ **Its stated gate was NOT transcribed**: "gated on controller status (§8.4)" is incoherent, since §5.1's four layers include no session-scoped one and the controller rule reaches only commands that mutate a session — so **A14's pattern was applied instead** and §8.4 now states one rule for the registry, the config writes and a board write together, discharging A22's assignment. **The seam was corrected on the way**: §8.4 said the rule lands on `Hello`'s capability flags, but `Hello` is not a `Command`, so a flag there cannot gate a browser — §8.3 now names two seams and requires one policy. ⚠️ **Two rules the entry does not contain were added, both in the "too permissive" class no structural gate can catch**: the writable root is **derived server-side and never supplied** (one server hosts sessions in several repos, so a client-named root would let a browser in repo A write repo B's config), and **a write may not add an exec surface** — `[[extensions]]`' `cmd` and a hook's `run`/`post` are refused although the file is inside the allowlist, because §9.2 renders model-authored markdown on the authenticated origin and one transcript could otherwise become persistent command execution as the server's user. Plus `ETag`/`If-Match` with `412` (four writers reach one file) and a board route that `409`s outside a repository. **U99 closed**; **B10 is now half-applied** (comment-preserving writes normative at §5.1, its `dacode.schema.json` still owed); **one gap recorded rather than filled** — §3.3 has no task-mutation `Command`, so the browser path to `promote` does not exist in the protocol. **The sixth delta in a row whose named work was partly pre-done or partly wrong** |
| ~~**A13**~~ | §2.1, §2.2 | **Two crates approved-with-dates rather than added**: `dacode-store` extracted at M3, `dacode-introspect` created at M5, with every crate vendored in-tree | 2026-09-18 | The cheapest delta in the set — no wire change, no fixture, no new type, **and no gate-file change** — and its whole discipline was *not* adding things: §2.1's tree still lists **eleven members**, because the banner's instruction was that neither crate may look like an M0 directory. Both are recorded as approved-with-dates with the reasons that make the dates principled rather than arbitrary: `dacode-store`'s traits live in `dacode-core::store` **from M1** so the boundary arrives when `SqliteStore` becomes the second implementor (a boundary with one implementation is indirection without payoff, and the trait risk is retired by a second implementor rather than by where the trait lives — extracting later is a *move*, not a redesign, precisely because M1 wrote against the trait); `dacode-introspect`'s **exchange-record format is already normative in §6** via A10's carve-out, so the crate arrives to read records written since M1. ⚠️ **A phantom-edge hazard was found and written into the spec rather than left to the reader**: `gates/boundary-edges.txt` is an **exact** diff against `cargo tree`, not a subset test, so §2.2's new rules name edges for crates that do not exist and state in the same breath that those edges **must not** be pinned until the directory does — adding one does not describe the graph, it breaks the gate, and it looks like diligence to someone reading both side by side. Verified unchanged at 29 lines with zero store/introspect edges. **Two of its own instructions were already stale**: Appendix D's graph already carried both crates, so what was actually wrong was its *legend* (it called them "proposed" and carried no milestone), and §2.2 said the rules are "enforced by review; consider a workspace-lint" when the pinned exact-diff gate **already exists** — the spec was calling its own enforcement mechanism aspirational. One coupling named rather than discovered at M3: A13's placements must agree with **A7's** scaling axis, still unapplied. **The fifth delta in a row whose named work was partly pre-done** |
| ~~**A8**~~ | §9.1 | **The web stack as a deferral with a committed prerequisite**: the framework open but deadline-bound, the JSON-Schema export unconditional | 2026-09-18 | B2's split transcribed with the framework **deliberately not picked**, which is the whole discipline of this delta — §9.1 now says the framework is an open decision **due by M3** with **TS SPA (Solid)** as the *standing recommendation* and both reasons recorded (all three reference web UIs chose TS; the four hard parts of this UI have mature TS solutions and weak WASM ones), rather than a bare "Decision pending". A recommendation is not a decision, so §9.2 and §9.3 stay framework-neutral. **The JSON-Schema export became unconditional**, with the inversion stated in the spec: it was carried only inside the "If TS:" branch, so an **M1** obligation was conditional on an **M3** decision — and the export is what makes the TS branch *cheap*, so requiring the branch first would have made the decision harder rather than easier. ⚠️ **A live contradiction was resolved, and it was not a choice**: the old text had a `build.rs` emitting the export *inside `dacode-protocol`*, while §13.3 confines `schemars` to an offline generator ("never in a shipped crate") and §2.4 puts build-time codegen in `xtask`. So the generator is `xtask`'s, the protocol crate grows no `build.rs` and no `schemars` dependency, and `protocol.d.ts` is generated **in CI with a drift check that fails the build** — layered on the golden fixtures, which catch a change to the serialized shape while the drift check catches the generated types falling out of step with it, the failure a second language cannot otherwise see. **Doc 03's "Optional … if the web UI is a TS SPA" was a false claim rather than a stale marker**, corrected under hard rule 1 with its marker and depth untouched. **The fourth time a signed decision's M1 half was found sitting inside a conditional** (A25's wire half, A10's carve-out, A21's `§14.2` citations), which makes it a pattern to check for rather than a run of bad luck |
| ~~**A22**~~ | §3.3, §4.4, §5.3, §7.1, §7.2, §8.2, §8.3, §8.4, §9.2, §14 | **Four capability seams closed**: hooks take an argv array with no shell, `AllowPattern` is bounded by the request it answers, sub-agent inheritance is an intersection, and the browser token stops leaking through `Referer` and stored content | 2026-09-18 | All four seams closed across **ten** sections rather than the six named, with **B7's confirmation of A23's `yolo` carve-out landing in the same pass** as this delta's banner required, so §4.4 was edited once rather than twice — discharging the debt A23's resolved-table row named as B7's. Items 1, 3 and 4 landed as individually confirmed, each with the *reason* it is a rule rather than a preference: templating survives **into argv elements** because deleting it would be safer and less useful while the array form keeps the convenience and removes the surface, and a hook wanting a shell must now write `["sh","-c",…]` so the dangerous case is visible in the config; inheritance is an **intersection** because §5.3's existing promise was true for modes and false for scopes; and item 4 is a **renderer** rule in §9.2 so it covers any surface rendering model output, not one view. ⚠️ **Item 2 was redirected at sign-off and then re-answered by the user**: the capability split stays **reserved** (`Hello` carries `answer_questions`/`resolve_permissions`, granted by default in v1, so tightening later is additive and not a wire break — B7's inert-stub pattern applied to a trust model) and §8.4 now states the trust model **explicitly** rather than leaving it implied by three separately-written sections composing badly, which was the actual defect. But the **`AllowPattern` subset bound IS applied**, reversing this entry's own orchestrator reading, on the argument that *permissive* describes the **user's** experience and the bound does not touch it — a user may still grant `**` from their own terminal, and what the bound constrains is a client acting on their behalf, which without it could widen permissions for a whole session from a single prompt. **The `yolo` carve-out was confirmed and then improved**: it stands (a mode meaning "do not ask" cannot start asking, and the detection is still recorded so the audit trail is complete), but `yolo` is also the mode where a leak is *most* likely and the one mode where an escalation-based warning would never fire, so the first detection in a session raises a **non-blocking notice** — once per session, because a notice on every occurrence in a mode chosen for not being interrupted would be the interruption. **Closes U61.** No `Event` variant, no fold change, no fixture, exactly as predicted; §14 gained a **Trust boundary** bullet with one test per seam |
| ~~**A21**~~ | §14 → §14.1 Correctness / §14.2 Benchmarks and evaluation | **Measurement separated from correctness**: a deterministic CI tier over a committed log corpus, prompt-cache hit rate as a measured property, the publication rule normative, task-success deliberately out of CI, cross-harness comparison a non-goal | 2026-09-18 | The cheapest delta in the set, as its own entry predicted — no wire change, no fixture, no new type — and the only one that makes existing spec claims falsifiable. **It arrived late enough that the spec already cited `§14.2` twice**, both written by A4 (§3.2's `Usage` paragraph and §4.5's partial-compaction paragraph), so applying it is what made those resolve: they were **dangling forward references in the spec**, invisible to every gate, because the convention that forbids markdown links had also removed the only mechanism that checks an internal cross-reference — in a file carrying 290 of them. **That hole is now closed by a new standing gate**, `check_section_refs()` in `check-links.py`, bound to the same `NO_LINK_FILES` list for exactly that reason and **tested by injecting three dangling refs** (`§14.9`, `§99.3`, `§2.7`) to confirm it fails on those, ignores the two valid ones beside them, and passes on restore. **§14.1 is twelve bullets rather than the five "existing text unchanged" this delta assumed** (thirteen once A22 landed, fourteen once A7 added `Config writes and the registry hint`), because A3/A4/A5/A9/A15/A10 each added tests — the instruction was right in intent and wrong in fact, so it was followed as intent. **A twelfth bullet it does not propose was added**: `Clients`, discharging **U101**'s finding that §14 had no UI test tier at all and so A9's mouse and paste rules were normative with nothing that could fail on them; the two *properties* are fixed and the **harness deliberately unnamed**, because the TUI crate does not exist and §9.1's framework decision is open. All four inbound assignments discharged inside §14.2 — the compression **level** as a measured knob (A15), **`ExchangeRecord`** as what hit rate is computed *from* (A10, and the reason that record was an M1 deliverable), partial compaction gated on the number existing (U97), and B18's publication rule stated normatively **first** because everything below depends on it. ⚠️ **Its own `Current.` list was stale in three places** and was corrected rather than transcribed: the codec is no longer unnamed (A15/B12), §4.5's 80 % is no longer the default (A4), and §14 was eleven bullets not five; its "B12 stops being a stalled vote" knock-on is historical, since B12 resolved by *decision* and §14.2 tunes the level |
| ~~**A9**~~ | §9.2, §10 | **One tree component across three sources**, **conditional TUI mouse**, **collapse-on-paste** — and with it M1's delta *list* is complete (⚠️ the milestone was not: A10's M1 carve-out, assigned to A5, was still missing and landed later the same day) | 2026-09-18 | Taken **out of queue order** and for a reason worth keeping: the application order puts A6 next, but A6 is §5's **M4** dependency while A9 was **M1's last**, and the queue and the milestone lists are different axes. Its own sequencing constraint held — A3 and A4 both landed earlier the same day. **§9.2's half was already normative**, landed by A19 on 2026-09-13 as "one tree component, three sources", so A9 *extends* it: what it contributes is §10's **matching** component (the TUI sidebar previously listed sessions/sub-agents and the task tree as two bullets and never named workflow steps) plus the two TUI rules. Each rule landed with its **reason** and not only its mechanism — a TUI that captures the mouse all the time takes scroll-back and text selection away from the user's own terminal, and a large bracketed paste can be misread as a burst of keystrokes — because a conditional rule with no stated reason reads as a convenience and gets "simplified" later. Both are recorded as **§10 rules, not shared client behaviour**, so §9.3 correctly grows neither. ⚠️ **One gap found and deliberately recorded rather than filled**: §9.2 promises three sources in one tree, but nothing on the wire correlates a declarative workflow *step* to the child session it spawned — `SubAgentSpawned` carries the child, the agent *name* and a task *description*. The engine knows (§4.2's `task` tool carries the parent task-item id) but does not say. The minimal fix is a wire change plus a `TaskId` identifier (still undefined — **U100**) plus a §4.3 scan-exclusion clause, since `TaskId` *is* tool-nameable via `todo`; that is three sections A9 does not name, for a correlator whose shape depends on §5.4's step model, **which A25's M5 half owns and has not designed**. Neither signed text requires it — both claim the sources are the same *shape*, and shape is a property of the node, not of the edge. **U101 opened**: §14 has no UI test tier at all, so both new client rules are normative with nothing that can fail on them; A21's §14 restructuring owns it |
| ~~**A5**~~ | §4.3, §4.4, §6, §14 | **The capability model as data** (`ToolDialect`, `CompletionRequest`, `StreamChunk`), B5's in-band subset as normative rules, SSRF/egress `[net_policy]`, retry with jitter and abort-aware sleep | 2026-09-18 | Applied **as redirected**, which is the substance: native `gemini` is not in v1, and §6 states why that is the stronger position rather than leaving a reader to wonder whether it was forgotten — a provider needing something unusual is a **value on `ModelInfo`**, not a branch in §4.2. **A4 had already built most of the capability model** one day earlier (its four-rate cost rule needed `ModelInfo` to be a real struct, and put `supports_tools`/`supports_thinking`/`supports_cache_breakpoints` on it), so A5 extends rather than adds; the second delta in six whose named dependency was already discharged. `ToolDialect` landed as a **reserved slot** — B5's grammar is reachable only when `supports_tools` is false and v1 ships exactly one, so the value is *derived* and the point of the enum is that un-deferring the ten dialects adds variants rather than restructuring the engine (the B7/A22 inert-stub pattern applied to a capability model). **B5's instruction to add a numbered §16 decision was deliberately not followed**: §16 lists *open* decisions, B5 is resolved, and A19 set that precedent explicitly — so the deferral lives in §6's prose where a reader meets it. **`CompletionRequest`/`StreamChunk` were referenced by §6's trait and never defined**, and are now real types, with `StreamChunk` deliberately cursor-less since a chunk is not an envelope. The egress policy needed two sections A5 does not name: §4.4 gains an **IP layer under the hostname globs** (a glob matches the wrong thing, so it cannot close SSRF) and §4.3's `fetch` becomes the second of three egress paths through one policy. **U10 stays open** and §6 now records the exclusion |
| ~~**A4**~~ | §3.2, §4.5, §4.8, §5.6, §6, §14 | **Prompt caching** as four engine invariants (fingerprinted prefix, append-only growth, longest-stable-prefix resync, volatility hygiene) + **compaction** opt-in with five rules + **B14's segment-index buffer** + `Usage`'s four counters | 2026-09-18 | The caching half landed as proposed and both compaction amendments as amended, so automatic compaction is **off by default** and §4.5 no longer implies it ships at M1. **Its own `Depends on` was already satisfied**: B6's canonical byte-stable schema has been normative in §13.3 since A16 (2026-09-12), five days *before* this was signed, with §13.4 independently ruling tool ordering out of the prefix — so A4 cites both rather than duplicating normative text in a second section. **Three items composed into a hole none of them stated**: amendment (a) defers compaction, A3 landed the hand-over as a *slot* not a mechanism, and U68's prescribed fix was "compact and re-issue" — leaving v1 with no automatic response to a filling context. Resolution forced by the signed amendments rather than a new choice: overflow is a **retryable provider error with two branches** (compact-and-re-issue when enabled; otherwise terminate through §4.2's guard set with an actionable remedy), and §4.5 now states plainly that **v1's default answer is "stop and tell the user"**. **`Usage` was referenced by two events and never defined** — the same defect A3 fixed twice the day before — and is now a real struct whose four counters are what closes **U71**: pricing a cached read at the input rate is a wrong cost, not an imprecise one. Partial compaction recorded as a **candidate** with both hazards named, not adopted before §14.2's number (**U97 open**). Discharged the other half of **A19's** cross-reference debt and applied **B14**, including its §5.6 consequence: the environment block sits outside the fingerprint. The optional method chain was deliberately **not** transcribed — this entry records it as future work and three of its five stages are the undesigned hand-over |
| ~~**A3**~~ | §3.2, §3.6, §4.2, §4.8, §11, §14 | **Loop guards**: termination as a named, configured, reported guard *set* rather than one counter; streaming/execution overlap with the owning transcript fragment committed before side effects; the sacred tool-pairing invariant with typed synthetic results; steering as distinct from a hard cancel | 2026-09-18 | Six sections against the one named, because the guard set needed types the spec had been **using without defining**: `StopReason` (branched on as `EndTurn` in §4.2, and A3's own rule fires on a `length` stop that did not exist) and `IdleReason` (five variants in a trailing comment), plus a §4.2 pseudocode line calling `AgentIdle(NeedsInput, "turn limit")` — **a two-argument shape the one-field variant could not hold**. Both are real enums now, with `ToolOutput`/`SyntheticReason` beside them. **`AgentIdle` gained `guard: Option<GuardKind>`**, a wire change recorded rather than folded in: growing `IdleReason` instead would break §11's four exit codes and §9.2's five chips, and an untyped detail string would violate principle 3. `Option` omitted when `None` keeps an ordinary `Done` byte-identical. **The guard numbers are marked pending**, since §1's deferral names them as brainstorm-owned content; what is normative is the shape, so tuning a guard is a config change and adding one is never a wire change. **Discharged two debts owed by other deltas** — A19's (§4.8's gate is now `CompletionGate`, one guard among eight) and A2's (its dispatch ordering shares the pseudocode block). §3.6 gained the pre-side-effect fsync, §11 the explicit `IdleReason` mapping, §14 three tests. **U96 deliberately not closed** |
| ~~**A2**~~ | §4.2, §4.3, §13.2, §14 | **Concurrency policy**: `Concurrency { Shared, Exclusive, Keyed(ConcurrencyKey) }` resolved per call from `args`, replacing `parallel_safe() -> bool`, with the author guidance the amendment made an obligation | 2026-09-18 | The amendment landed as signed — the tool computes its own key rather than returning a closure, so the key is a value that can be logged — and §4.3 now documents how an author chooses, why the key must be **canonical**, and that the `Exclusive` default is deliberately pessimistic (an unthought-through tool costs parallelism rather than causing a race). **No wire variant added**, so the "loggable in the audit trail" argument landed as a *property* rather than as a new field on `PermissionResolved`; if a concurrency decision needs its own durable event that is a new delta. **Per-tool assignments are marked pending**, not invented — §1's deferral names "which tools ship" as brainstorm-owned content, on A19's §4.8 precedent. **U27 closes on both halves**: with no bare `Json` left in the spec, §13.2's alias note was dropped rather than narrowed again. Touched two files outside the plan — `.agents/roles/_rust.md` and the `dacode` skill, plus four **generated** adapters — because a role file is loaded into every subagent's context, so a stale line there is an instruction being given to every worker. **Doc 08 needed more than a marker flip**: it specified `Concurrency::Dynamic(fn(&args) -> Key)`, the arm the amendment rejects, so a reader would have built the rejected design; corrected with the marker and depth untouched |
| ~~**A15**~~ | §2.3, §3.6, §12, §13.1, §13.2, §13.5, §14 | **Selective compression**: a per-field `codec` discriminator, a config-driven allowlist over durable fields, a size floor and level, one `[storage] compress` knob, the searchability trade, and the codec named per *file* | 2026-09-18 | The mechanism landed **exactly as signed**; three enumerations did not survive later deltas, and each is recorded as a deviation rather than folded in silently. The codec moved **into `LogHeader`** (`plain | compressed`) because generalizing it into *config alone* would orphan every record already written — the log is append-only, so one file lives under one codec and the header says which; blobs are self-describing since they cannot inherit it. **`flat` left the enum**: it is a *record representation*, not a field encoding, and keeping both axes in one discriminator would make a format change expressible as a compression change. **`ToolOutputDelta`/`ThinkingDelta` are `Live`** under A1's rule — never written, so compressing them saves nothing — and were restated to `ToolResult.output` and `ThinkingEnded.text`. It also closed a contradiction it did not own: **§13.2 said the log is `flat` buffers**, which B21 overruled five days after A16 wrote it, and §2.3/§13.5 still called the codec an open choice after B12 settled it. §3.6 gained the rule that **the provider transcript is never compressed** (§13.4's byte-identical prefix), and §14 three tests. **U25 closed; U14 deliberately not discharged** |
| ~~**A14**~~ | §3.1, §3.2, §3.3, §3.5, §3.7, §4.7, §4.3, §8.4, §12, §14 | **Session organization**: three durable membership events, `SessionInfo` as a real struct, `ProjectId`/`FolderId`, an `Org { op: OrgOp }` registry command, `org.json`, B13's Fix-1 carve-out and Fix-2 derivation | 2026-09-18 | The signed delta named two sections; **ten were edited**, because nothing in it carried what a folder *is* as opposed to which folder a session is in. Applied with **four scoping decisions, all the user's**: the registry is authoritative for **names** and membership stays a fold, so v1 accepts that a rename reaches only its issuer and §3.7's promise is **scoped to a session** with the freshness channel left to A7 (**U99**); registry mutation is a **`Command`**, since §8.1's default TUI has no socket, and §8.4's controller rule does not reach a session-less resource; **one op-carrying variant** on `TaskUpdated`'s argument, with **caller-allocated ids** because `Ack` has no payload slot; `org.json` atomic and non-rebuildable. Fix-2 landed normatively in **§4.7** rather than only in doc 06, because the spec may not cite a plan doc and §14 must be able to test it. **Falsified A23's prediction** that `ProjectId` would join the `args` scan — nothing injects one, so it could never fire — and annotated both A23 sites. **U16, U17 closed** |
| ~~**A1**~~ | §3.1, §3.2, §3.5, §3.6, §3.7, §4.3, §4.8, §12, §13, §14 | **The two-tier event model**: `Event::tier()` over an `EventTier`, the rule *`Durable` iff the fold consumes it*, `seq: Option<Cursor>`, `TextEnded`/`ThinkingEnded` full-value boundaries, live deltas never persisted — **plus A25's M1 wire half** (`HostId`/`SessionRef`, `WriterEpoch`, `Snapshot.epoch`) and **§16 item 6's `Standing`** | 2026-09-18 | Applied with **six scoping decisions**, four of them the user's: `seq` *had* to become `Option`, because A18 made a missing `seq` the only gap signal and numbering deltas would make clients resync forever; the tier is a **rule** so A14's events and B17's `finding` classify themselves; only the **epoch** is wire-visible, the idempotency key and leaf CAS being store obligations; and A25(a) took a **third answer** that keeps A23's scan exact-matchable. **U66 folded in** (`LogHeader`), which reworded a cursor sentence A24 had landed the day before. **U27's §3.2 half discharged**, its §4.3 half left to A2. No fixture existed, so nothing was retrofitted |
| ~~**A19**~~ | §4.8 (new), §3.2, §3.5, §4.2, §4.3, §5.4, §9.2/§10, §12, §14, §16 | **Built-in task tracking**: a fold-derived task tree, item kinds, a `verify:` mandatory at creation and typed by checkability, a hard-but-bounded completion gate, append-only context rendering, and `.dacode/plans/` finally specified | 2026-09-13 | Applied **self-contained**, not verbatim: three references in the proposal were dangling (§4.5 has no caching content until A4, no `Segment` until B14, §4.2 has no guard set until A3), so §4.8 states its own requirements — the A16 move. Its `Depends on: A1` claim was **wrong** and is corrected. §16 gained no B15 row (it lists *open* decisions) but did gain **§16.6**, the standing-goal gap found while applying. §5.4's `gate` → `approve`. One fixture, two engine tests |
| ~~**A18**~~ | §3.7, §4.1, §14 | Bounded fan-out + drop-oldest, and the seq→gap→resync contract as one invariant, replacing §3.7's unqualified "No gap, no duplicate" | 2026-09-12 | §3.7/§4.1 applied verbatim from the paste-ready proposal. §14 extended beyond the literal proposal (fold-idempotence + a lagged-subscriber test) because an untested invariant is how §3.7 became false. No new variant, so no new fixture. Discharged A1's resync bullet and its B4 dependency |
| ~~**A17**~~ | §2 | Dependency policy; §2 restructured as **§2.1 crate tree · §2.2 internal rules · §2.3 external policy** | 2026-09-12 | All four bullets landed. Only TLS is named in the spec as unavoidable; the avoidable consequences stay in Appendix F so unsigned choices are not enshrined. No link added — the spec carries zero markdown links by convention |
| ~~**A16**~~ | §13 | Hybrid JSON strategy + `RawJson`; §13 rewritten as **§13.1–§13.5** | 2026-09-12 | Items 1–7 all landed. §13.2 defines what the spec's bare `Json` alias denotes, so §3.2/§4.3 stop contradicting §13 without pre-empting A1/A2 |
| ~~**A11**~~ | §13 | Superseded by A16, as its own entry recommended | 2026-09-12 | Kept as the record of how the discrepancy was found (tracker finding `f1`) |

~~**21 of 25 deltas are applied and the other 4 are approved and owed a transcription**~~
~~**22 of 26 deltas are applied and the other 4 are approved and owed a transcription
— none is unsigned.**~~ **23 of 27 deltas are applied and the other 4 are approved and
owed a transcription — none is unsigned.** ⚠️ **The total grew without the owed count moving, which is new**:
**A26** arrived *already applied* on 2026-09-19, and is the register's first entry not to
come from a B-decision — it came out of M1.1a's review of the three protocol types `bc52e50`
designed rather than transcribed ([TODO](../TODO.md) §6 **U108**/**U109**), was approved
in-session by the user, and landed the same day. **A27 followed on 2026-09-20 in the same
shape** — out of M1.2 Pass C1's fold walk (**U111**), approved in-session, applied the same
day — so the class that arrives already applied, from a board finding rather than a
B-decision, now has two members, and both were free *because* they landed pre-fixture. Two of the four are *half*-applied and count in both columns:
**A25** (M1 wire half landed inside A1, M5 behaviour half owed) and **A10** (its M1
exchange-record carve-out landed 2026-09-18 in §6/§12/§14, its §17 body owed at M5).
What remains owed is therefore **A6** (M4), **A10's §17 body** (M5), **A12** (last)
and **A25's M5 half** — and since A7 landed 2026-09-19, *no* owed delta gates M1, M2
or M3.** ~~**16 of 22 deltas remain unapplied.**~~ Both halves of that
sentence were stale: the denominator predated A23 and A24, and the numerator hides
the distinction that now matters — an *approved* delta is work an agent can start,
an *unsigned* one is not. ~~**A21 and A22 are both outside the sequence**, like
A20.~~ **A21 was signed 2026-09-17**, ~~so A22 is the only one left outside the
sequence;~~ **and A22 followed it the same day**, so the sequence no longer has an
exception waiting on a vote. ~~A22 depends on nothing at all and is still cheapest
now, because every seam it closes is free before the code exists and awkward
afterwards — though its item 2 was **redirected** rather than accepted, so what it
owes §8.4 is an explicit trust model and a reserved capability field, not a
restriction.~~ ✅ **Both applied — A21 and then A22, on 2026-09-18.** A22's
"cheapest now" argument turned out to be right in a stronger sense than written: the
config schema its item 1 changes is still read by no code, so `run` became an argv
array for the cost of one edit rather than a migration. **Its item 2 landed as
redirected *and* as re-answered** — §8.4 does state the trust model explicitly and
`Hello` does reserve the capability flags, but the `AllowPattern` subset bound was
**redirected back in** on 2026-09-18, so §4.4 carries a restriction after all. Next in the
application order: ~~the protocol/storage group **A1** and **A14**~~ → **A1 applied
2026-09-18**, ~~so the group's remainder is **A14** and **A15**~~ → **and A14
followed it the same day**, so the group's remainder is **A15 alone**. A14 inherited
exactly what A1's banner promised: the tier rule classified its three events
`Durable` with no tier decision to take, and §14's derivation gave them *log*
fixtures without a list to update. It also **grew well past its own proposal** — two
sections became ten, because the signed text answered *which folder a session is in*
and nothing in it carried *what a folder is* — which is the second time a delta's
real scope was only visible on application (A19 was the first, and it wrote §4.8
self-contained for the same reason). ~~**A15 is held
behind [B12](B-decisions.md)**~~ → **the hold is gone entirely**: B12 resolved to
`zstd` on 2026-09-17 as an accepted class-1 exception, and A15's mechanism was
approved the same day. The reason to generalize the variant names anyway survives
the resolution — the discriminator carries the codec as a **serialized wire value**,
so a name baked into it makes the *next* switch a §14 compatibility event rather than
a config change, and the proposal hardcodes `plain | zstd | flat | flat+zstd`.
~~**A1 is now the last structural blocker on M1's golden
fixtures**~~: A18 narrowed it to the tier marker and the `*Ended` variants, and
both of its decision dependencies are gone (B4 resolved, ~~B3 effectively resolved
by A1 itself~~ **B3 signed 2026-09-17 outright**). ~~It must still land *before* any
fixture is written.~~ ✅ **It did, 2026-09-18** — and the precondition held rather
than being waived: the crates are still stubs, so no fixture existed to retrofit.
**M1's golden fixtures are now unblocked**, which is the last structural gate that
was ahead of them.

**A19 landed out of order and left A1 one more thing to do.** It added
`TaskUpdated` to §3.2's `Event` enum, so A1's tier marker now has one more variant
to classify — durable, and §4.8 says so in prose in the meantime. That does not
weaken the "A1 before any fixture is written" rule: no fixture exists yet (there
is no code), and `TaskUpdated`'s will be written after A1 lands like every other.
It does mean A1 can no longer be applied by editing only the variants that
existed when it was drafted. ✅ **Done 2026-09-18, and the warning generalised
better than expected**: `TaskUpdated` is classified `Durable`, and because the tier
landed as a **rule** rather than a list, the variant A19 added and the `finding`
variant B17 will add both classify themselves with no edit here. §4.8's prose claim
is now a property of the enum. **U48 closes.**

---

### A23 — §4.3, §4.4, §5.1, §7.1, §8.2, §12: identifier privacy — ✅ **APPLIED 2026-09-16**

> **✅ APPLIED 2026-09-16**, on [B19](B-decisions.md)'s sign-off. All seven
> proposed edits landed and the rule is normative across six sections: §4.3
> carries the three-surfaces argument, the injection, the opaque handle and the
> ULID-only scan; §4.4 places the scan; §5.1 reserves the prefix; §7.1 redacts a
> `post` body; §8.2 names the redaction points; §12 gains `sessions/<id>/tmp` and
> `scratch/<token>`. No `Event` variant, so **no golden fixture**, and it landed
> independently of A1 exactly as predicted. The spec gained no markdown link and
> no B- or U-number citation.
>
> **One correction and five scoping decisions taken while transcribing — recorded so
> a reviewer can check them rather than infer them. The correction changed what the
> spec asserts; scoping decisions 1 and 2 changed behaviour; the last three are
> wording an implementer would otherwise have guessed.**
>
> 0. **⚠️ CORRECTION, caught on self-review after a first draft had already landed:
>    the redaction half is about *secrets*, not identifiers, and the draft said
>    both.** Item 5 proposed "redaction on the four projection endpoints" without
>    naming what is redacted, and the first transcription read that as covering
>    §4.3's identifier set as well. It cannot, for two reasons. `/export` and
>    `/events` serve **envelopes**, whose structural ids — `session`, `seq`, `call` —
>    are how the fold correlates events (§3.5), so redacting them makes `/export`
>    produce a file nobody can replay and `/events` a page nobody can render. And it
>    contradicts [B19](B-decisions.md) itself, which says of the identifier control
>    *"it is not a redaction layer"* and rests on prevention precisely because
>    neither the log nor the cached prefix may be rewritten. **All three sites now
>    say secrets explicitly and state why identifiers are excluded**: §4.3's closing
>    paragraph, §7.1 and §8.2. Worth recording as a class: a signed decision does not
>    protect against a mis-transcription of it, and nothing but a re-read against the
>    decision's own text catches one.
> 1. **§4.4's automatic-decision path had to be written, and the `yolo` carve-out
>    in it is the one to check.** The proposal said only that the scan runs "after
>    the permission decision but before exec" and that a detection is surfaced
>    "inside the permission prompt" — which is silent on the case where the
>    decision was *automatic* and there is no prompt. Leaving it silent would have
>    made the scan advisory on exactly the paths where nothing else stops a call,
>    which is the defect B7 was widened to name. So §4.4 now says: record the
>    detection on `PermissionResolved` and **escalate to `ask`**. `yolo` records
>    without escalating, justified in spec-internal terms rather than by citing an
>    unsigned decision. ⚠️ **B7 owns the mode's polarity and is unsigned** — when
>    it lands it must confirm or overturn that carve-out. This is the same shape
>    of debt A19 created for A3/A4.
> 2. **§5.1's "rejected rather than ignored" was scoped to authoring surfaces, or
>    it breaks nested `dacode`.** Taken literally — rejected on *presence* in the
>    environment — the rule would break the case the reservation exists to
>    protect, because the engine injects `DACODE_RT_*` into every tool process and
>    a nested `dacode` launched from a tool call inherits them. That nested case
>    is not hypothetical: it is what a [B20](B-decisions.md) swarm member *is*.
>    So the spec makes a key **defined** in the reserved namespace a load-time
>    error, while mere presence in the environment is expected and not an error —
>    stated explicitly so an implementer does not "fix" it into a startup failure.
> 3. **Item 7 said the scratch directory is "gitignored"; it is not, because it is
>    not in a repository.** [B19](B-decisions.md)'s own worked example puts it at
>    `~/.local/share/dacode/sessions/<id>/tmp`, which is outside any repo, so
>    there is nothing to gitignore. §12 says so rather than carrying an
>    inapplicable instruction — the same reason A17's application refused to
>    enumerate avoidable consequences in the normative doc.
> 4. **The alias mechanism is deliberately left open, with the property stated
>    instead.** B19 offered "a symlink, a bind, or a directory named by a derived
>    token". A symlink *resolves* to the id-bearing path, so had the property been
>    written as "no resolution may reveal the id" a symlink would fail it and the
>    offer would be self-contradictory. §12 therefore makes the property about the
>    **exported path string**, with the reasoning recorded (an argv is free for
>    any local user to read; resolving a symlink is neither free nor accidental).
>    Naming a mechanism would have put an M1 implementation choice into the spec.
> 5. **`run` hooks are explicitly *not* redacted.** Item 4 said only "hook payload
>    redaction", but §7.1 has two hook kinds and only `post` crosses a network.
>    Saying which is redacted and why prevents "payload" being read as both —
>    redacting a local `run` hook would break the legitimate case of a user's own
>    script needing the real id.
>
> **Still open, unchanged:** how wide the injected set goes. It is written as a
> **config list starting at three** (`DACODE_RT_SESSION`, `DACODE_RT_CALL`,
> `DACODE_RT_TMP`), so widening is a configuration change and not a spec edit —
> which is what discharges the low-confidence part without settling it.
> [B18](B-decisions.md)'s T2 prompt-overhead number is what argues the rest.
> ~~**A14 will add `ProjectId`** to the scanned set when it lands, as predicted: one
> line, not a dependency.~~ ⚠️ **This prediction did not fire, and A14's banner
> records why (2026-09-18).** A14 added `ProjectId` and `FolderId` to §3.1 and
> **excluded both from the scan**, in the clause shape `HostId` got. The prediction
> assumed a new ULID-backed id is a new scan target; the scan's actual subject is
> narrower — ids the engine **allocated and injected**, where the risk is a tool
> echoing one into an argv. Nothing injects a project id (`DACODE_RT_` starts at
> three) and no v1 tool argument can name one, so the check could never fire and
> adding it would have made the enumeration look complete when it is not. §4.3 now
> states the exclusion with its condition, so it widens by the same one line if a
> tool ever can name a project.

**Status.** **B19 signed off 2026-09-16** ("if you recommend (2)+(4) — then is
fine"), so this delta is a transcription of a decision already made, not a proposal.
It is written as a delta rather than edited straight into the spec because the spec
moves only through one (B4's rule), and because six sections is too many to change
without a recorded blast radius.

**Current.** The spec has exactly one redaction concept — §8.2's `GET /api/config`
returning "secrets redacted" — and no general one. §4.3 defines the tool trait and its
`args`; §4.4 the permission resolution order; §5.1 four config layers, the fourth
being "environment (`DACODE_*`) and CLI flags"; §7.1 hooks, where `post` ships the
whole `Envelope` to an arbitrary URL; §8.2 the HTTP surface including four endpoints
that serve *copies* of the log; §12 the on-disk layout, whose session directory is
`~/.local/share/dacode/sessions/<session_id>/`.

**Proposed.**

1. **§4.3 — runtime injection.** The engine sets the identifiers a tool needs as
   environment variables in the tool's own process, under a **reserved prefix that is
   not `DACODE_`** (that namespace is §5.1's config-override layer, so sharing it
   would make a model-authored `export` a config-write surface). Minimum set: one
   variable per id class, plus a scratch directory. Tool descriptions and the
   built-in agents' prompts reference the variables rather than literals.
2. **§4.3 — the scratch directory is an opaque handle.** §12's session directory
   *embeds the session id*, so exporting it would put the id straight back into argv
   — the surface being protected. Export a path whose name is a derived token that is
   not the id; keep the id-bearing path engine-internal.
3. **§4.3/§4.4 — an exact `args` check before execution**, restricted to the **four
   ULID-backed ids** of §3.1 (`SessionId`, `ClientId`, `CallId`, `ReqId`).
   `TurnId(u32)` and `Cursor(u64)` are excluded on purpose: substring-matching small
   integers fires on ordinary numbers. Matching a known finite set is exact, not
   heuristic — which is the difference between this and B7's deny-list. Where it sits
   in §4.4's resolution order is part of the delta.
4. **§7.1 — hook payload redaction.** `post` currently ships the whole `Envelope` with
   no rule at all while §8.2 redacts secrets. This closes U61's second half.
5. **§8.2 — redaction on the four projection endpoints** (`/export`, `/events`,
   `/blob/:blob_id`, `/diff/:call`). These serve *copies*, so redacting them rewrites
   nothing — the distinction that makes prevention necessary for the log and the
   transcript but not for these.
6. **§5.1 — the reserved prefix is excluded from the `DACODE_*` layer**, stated where
   the layer is defined rather than only where it is used.
7. **§12 — the scratch directory** appears in the layout, gitignored, with its
   opaque-name rule.

**What this delta does not do.** No redaction of the log or of the provider
transcript: the log is append-only (§3.6, principle 2) and the cached prefix must stay
byte-identical (§13.4), so for those two surfaces prevention is the only control that
exists. **No new `Event` variant, so no new golden fixture** — which is also why it
can land independently of A1.

**Depends on.** Nothing. B19 is signed; A1 is not required (no wire-shape change);
~~A14 would add `ProjectId` to the checked set when it lands, which is a one-line
addition rather than a dependency~~ ⚠️ **superseded 2026-09-18**: A14 landed and
**excluded** `ProjectId` (and `FolderId`) from the scan instead, because nothing
injects either and so the check could never fire — see the annotation in this
entry's banner and §4.3's exclusion clause. It was never a dependency in either
direction, which is the part that mattered for sequencing.

**Left open by the sign-off, with the variant recommended for it.** How wide the
injected set goes — B19's *low*-confidence part. **Recommend exactly three**, and make
the set a config list rather than a hardcoded enum so widening is a config change:
`DACODE_RT_SESSION`, `DACODE_RT_CALL`, `DACODE_RT_TMP`. Those are the three a tool
command plausibly needs; `ClientId` and `ReqId` are engine-internal, so injecting them
buys nothing and costs prompt surface that B18's T2 number measures. Three reliability
details: the reserved prefix must be **rejected** by §5.1's config loader rather than
ignored, so a mistyped key fails loudly instead of silently doing nothing; the
variables go into the **tool process environment only**, never the environment block
the model can see, or the ids are straight back in the prompt; and the `args` check
runs **after** the permission decision but **before** exec, so a denied call does not
pay for the scan and a detected literal can be surfaced *inside* the permission prompt
("this command contains the session id") rather than as a separate failure mode.

**Recommend accept as a transcription.** ✅ **Done 2026-09-16** — the seven edits are
applied, and U61 and U65 in [TODO](../TODO.md) §6 are widened to cite them rather than
restating the argument a third time. **Still owed:** deepen
[16](../part3-capabilities/16-permissions.md) when M1 reaches it. Doc 16 is M1 and still
at summary depth, so hard rule 1 holds it — and it now has more to carry than it did
when B19 was drafted, because §4.4's escalation-on-detection behaviour is permission-
engine work that no plan doc describes yet.

---

### A24 — §3.6, §4.1, §8.1, §12, §14: session-log concurrency and the index sidecar — ✅ **APPLIED 2026-09-17**

> **✅ APPLIED 2026-09-17**, on [B21](B-decisions.md)'s sign-off, the same day the
> delta was written. All six proposed edits landed: §3.6 carries the single-writer
> invariant with the lock's three reliability properties and the kernel-ownership
> argument; §4.1 makes the registry the lock holder; §8.1 names `attach` as the
> remedy; §12 gains `lock`, `index.bin` and the locality rule; §14 gains the two
> tests. [Doc 05](../part2-engine/05-event-log-storage.md) gained **§5.5** with the
> implementation obligations. **No `Event` or `Command` variant, so no golden
> fixture.** The spec gained no markdown link and no B- or U-number citation.
>
> **Five scoping decisions taken while transcribing — recorded so a reviewer can
> check them rather than infer them. The first is the one that changed the delta's
> scope; the rest are placement.**
>
> 1. **U67 was folded in, and the delta as drafted did not mention it.** B21's
>    on-sign-off list said §3.6 gets "the single-writer invariant and the lock file"
>    and nothing about the cursor. But §3.6 also said `Cursor = line number + 1`, and
>    an index of `cursor → byte offset` is **only worth having if the cursor is not
>    already the line number** — so applying the delta as scoped would have left the
>    spec asserting an equivalence its own new index contradicts. [U67](../TODO.md)
>    had independently identified the same sentence as a wrong invariant a store
>    implementer would inherit, and owed it to M1. **Both are discharged here.** This
>    is P-LIVE-005's lesson applied rather than repeated: re-read the edit against
>    what else in the same section it invalidates, not only against the decision that
>    authorized it.
> 2. **The index's three invariants went to doc 05, not the spec.** B21 asked for
>    both (§12 for the files, doc 05 for the deepening) without drawing the line. The
>    split taken: §12 carries the two files and the **rebuildable-projection
>    asymmetry** — what a reader of the layout needs in order to know which artifact
>    is the truth. Doc 05 §5.5 carries the same-code-path, log→fsync→index write
>    order, and validate-and-truncate-on-open invariants, plus the crate choice, the
>    refusal message and the NFS warning — what an implementer needs. That is A17's
>    precedent (refuse to enumerate consequences in the normative doc) applied to a
>    section that already had a doc to hold them.
> 3. **`fd-lock` is named in doc 05 and deliberately *not* in the spec.** §3.6 says
>    "advisory lock (`flock` on POSIX, `LockFileEx` on Windows)" — **syscalls**, which
>    are the semantics being relied on and will not change. The crate is an M1
>    dependency decision, and naming it in the spec would pre-empt the ask-first rule
>    for external crates. Doc 05 names it with its §2.3 class-2 justification, which
>    is where a dependency choice belongs. The spec does name `reqwest`/`hyper` and
>    `schemars`, so this is a judgement about *this* crate's status, not a convention.
> 4. **The locality rule went into §12 rather than a new section.** Sub-answer 2 is
>    about where the session directory may live, and §12 is the on-disk layout, so it
>    sits next to the paths it constrains — stated as a rule plus a startup warning,
>    not as a restructure.
> 5. **§14's two tests are specified as needing a second *process*,** with the second
>    named as the only thing that can fail on the kernel-ownership claim. That is the
>    rhetorical move §14 already uses for §4.8's bounded gate ("the only thing that
>    can fail on the claim that the gate is *bounded*"), so it matches the section's
>    existing style rather than inventing a new one.
>
> **Still open, unchanged:** the medium confidence on the offset sidecar lasting
> through M1, now carrying its exit route — **the index grows a second table rather
> than the log changing format.** And **[B20](B-decisions.md) must re-read this
> entry**: B21 is B20's constraint (d) at local scale, and a swarm whose members each
> held their own lock would satisfy B21 and violate B20.
>
> **One gap the application surfaced and deliberately did not fill — [U93](../TODO.md).**
> §3.6 now guarantees a refusal is fast, named and carries a remedy, but §11 documents
> exit codes *only* for `run`, and B21's sub-answer 5 says the commands that hit the
> lock are `session fork` and resume — so a refusal falls through to a generic `1`,
> indistinguishable from a crash. That matters for the same reason `run` already has
> **3** for "blocked on permission": a held session is a *state a caller must branch
> on*, not a failure. Assigning a number was not this delta's to invent, and the
> honest fix extends §11's table to the other subcommands in one edit rather than
> bolting a code onto a section that documents one command. Owed by M1 and doc 23.

**Status.** **B21 signed off 2026-09-17**, all five sub-answers taken as
recommended, so this delta is a transcription of a decision already made rather than
a proposal. It is written as a delta because the spec moves only through one (B4's
rule), and because five sections is too many to change without a recorded blast
radius.

**Current.** The spec says nothing at all about concurrent access to a session —
verified by grep rather than recall: `flock`, lock file, single-writer and "two
processes" appear nowhere in it. §3.6 gives an append-only JSONL log fsync'd at
three boundaries and states `Cursor = line number + 1`; §4.1 gives one
`SessionRegistry` **per process**; §8.1 lists four process models, three of which can
be running at once. So when a TUI and a `run -p` open the same session directory,
today's answer is *both append*.

**Why that is worse than it sounds, and the reason this delta exists.** It is not a
torn-line problem — a single `write()` of one line is atomic on a local filesystem,
so the JSONL stays parseable, and that is exactly what makes the failure quiet. What
breaks is `seq`: §3.7 says the engine assigns it on append and that it is strictly
increasing, so two writers each produce a valid sequence whose union is neither.
§3.7's gap detection then reports phantom discontinuities, clients resync forever,
and `Cursor = line number + 1` makes the two writers' cursors *collide* on the same
line. A log corrupt in its **ordering** rather than its bytes still parses, which is
why the invariant has to be stated rather than assumed.

**Proposed.**

1. **§3.6 — single-writer as an invariant, enforced by an advisory lock.** Exactly
   one process appends to a session's log: the holder of `flock`/`LockFileEx` on a
   dedicated `<session>/lock`. Three details carry the reliability, and all three are
   from B21's sub-answer 1: held **for the whole session, not per append** (per-append
   locking admits a second writer *between* appends, which is precisely the `seq`
   collision); acquired with a **non-blocking try**, never a blocking lock (a second
   engine should fail in milliseconds with a remedy, not hang behind a session that
   may run for hours); and on a **dedicated lock file, not `events.jsonl`** (locking
   the data file conflates "I own this session" with "I have this file open", and
   breaks the moment anything opens the log read-only to export it). The decisive
   property is that **the kernel owns the lock** and releases it when the process
   dies, including on `SIGKILL` and on crash — so there is no stale-lock recovery code
   to write and no `--force-unlock` flag to grow. The pid and start time written
   inside the lock file are **informational only**: correctness rests on the `flock`,
   never on the contents, which is the mistake PID-file locking makes.
2. **§3.6 — the cursor is assigned by the store, not defined as a line number.** This
   discharges **U67**, which is coupled rather than merely adjacent: an index of
   `cursor → byte offset` is only worth having if the cursor is not *already* the line
   number, and B11 plans a SQLite store by M3 where the equivalence is simply false.
   Restated as: the cursor is assigned by the store and is strictly increasing; for
   the JSONL store it happens to equal line + 1. One sentence, and it stops a store
   implementer inheriting a wrong invariant. §3.7's fold is over `seq` and was never
   over a line count, so nothing downstream changes.
3. **§4.1 — the registry is the lock holder.** Acquiring the lock is part of opening
   a session and releasing it is part of closing one, so the invariant has exactly one
   owner per process and one process per session. A second opener is an **error, not a
   second writer**.
4. **§8.1 — `attach` is the remedy.** Principle 1 is what makes single-writer cheap:
   the protocol exists so that a second consumer never needs write access. Naming
   `attach` in the process model is what turns a refusal into a usable answer.
5. **§12 — `lock` and `index.bin` beside `events.jsonl`, plus the locality rule.** The
   index is a **rebuildable projection, never authoritative**: fixed-width
   `(u64 seq, u64 offset)` records, append-only like the log, so a torn tail truncates
   to a valid prefix by arithmetic alone and a corrupt or stale index is recovered by
   one scan of the log — while a corrupt log is not recoverable at all. That is the
   rule A14/B13's Fix-1 already states for organization data, extended. §12 also gains
   the **locality rule**: the working tree may be remote, the log may not, because
   advisory lock semantics are version-dependent over NFS and the failure is silent.
6. **§14 — the two tests that make the invariant falsifiable.** A second process
   opening a held session fails cleanly, names the holder and offers `attach`, and
   never appends; and a holder killed with `SIGKILL` leaves no unusable lock, so the
   next opener acquires it. Neither is reachable from a single-process unit test, so
   both spawn a second process.

**The index's three invariants live in doc 05, not in the spec.** Reconstruction and
initial construction are the **same code path** (scan the log, emit records), so there
is nothing to drift; write order is **log line → fsync → index record**, never the
reverse, because an index entry for a line that did not land is a hole while a missing
entry for a line that did is merely a rebuild; and **on open, validate the last record
against the log and truncate to the last good one** — the WAL-recovery pattern, and
the reason "rebuildable" is load-bearing rather than decorative. Those are
implementation obligations, and doc 05 is already expanded, so it is where an
implementer will look.

**What this delta does not do.** It does not make the log anything but JSONL: the log
stays plain, grep-able, append-only and the truth, and the index stays a sidecar. It
does not add SQLite — that is B11's M3 extraction, and Appendix F already records the
`libsqlite3-sys` trade. It does not add lock stealing or handover, which is a
consensus problem wearing a UX costume. **No `Event` variant and no `Command`
variant**, so **no golden fixture** — the lock is below the protocol, which is the
point of it.

**Depends on.** Nothing. B21 is signed; A1 is not required, since the tier marker does
not touch storage concurrency. **It must be written against the current §12**, which
A23 amended on 2026-09-16 to add `sessions/<id>/tmp` and `scratch/<token>` — the
sequencing the board predicted when it landed A23 ahead of B21.

**Coupling recorded, not resolved.** This is **B20's constraint (d) at local scale**:
"one `seq` assigner per session log" was written there as a distributed-systems
commitment, and a file lock is the same rule enforced on one host. Recording them as
one rule is the point — **a swarm whose members each held their own lock would satisfy
B21 and violate B20.** B21's sub-answer 2 dissolves the NFS collision with B20's Q1
rather than solving it: Q1 was about where a remote member gets the *repository*, and
nothing requires the *session directory* to be remote.

**Left open, as B21 left it.** Confidence is **medium** on the offset sidecar being
sufficient through M1. It is, until the session-list filters land; if `dacode session
list` needs more, **the index grows a second table rather than the log changing
format** — which is the property that makes the medium confidence safe to sign.

**Recommend accept as a transcription**, and apply it before M1 writes its first
`EventStore`. ✅ **Done 2026-09-17** — accepted, applied, and doc 05 deepened in the
same pass, so the invariant is normative before any storage code exists rather than
retrofitted into it. That timing was the point: §3.6's single-writer rule is the kind
of constraint that is free to state and expensive to add once two writers have shipped.

---

### A26 — §3.2, §3.5, §5.3, §14.1: a turn is an ordered block list, and a child session closes with `Completed` — ✅ **APPLIED 2026-09-19**

**Origin, and why this entry exists in a register of proposals.** Not a B-decision. It came
out of **M1.1a's review of the three protocol types `bc52e50` *designed* rather than
transcribed** — `CloseReason`, `Preview`, `TurnView`/`CallView` — which the previous
session's handover flagged as worth a second pair of eyes before M1.2 fixtures them, on the
argument that no gate can tell whether a variant *set* is the right one. Two of the three
changed; `Preview` and `CallView` stood (the one refinement wanted for `Preview` was already
in §3.2's own field comment). The findings are [TODO](../TODO.md) §6 **U108** and **U109**,
both approved in-session by the user and applied the same day — so **this entry records an
applied edit rather than proposing one**, the first in this register to arrive that way.
That is worth saying out loud, because the alternative was a change to a `Durable` event's
enum and to `Snapshot`'s shape with no register entry at all, and a reader auditing "what
changed in §3.2/§3.5 after A25?" would have found nothing.

**1. `CloseReason` gained a fifth variant, `Completed` (§3.2) — a wire change to a `Durable`
event.** Each of the four named a producer: an explicit `CloseSession`, a writer fence, a
parent that closed, a host that exited. **A sub-agent whose task finished was none of them**,
and §5.3 returned the child's final message as the tool result and emitted
`SubAgentFinished` on the parent without ever saying whether the child session closes, by
whom, or with what reason — so the alternative to a new variant was child sessions
accumulating in a session list forever with no state marking them finished. §5.3 now carries
both the rule and the constraint that makes it safe: **`Completed` is child-specific**, since
a *primary* session that finishes emits `AgentIdle(Done)` and **stays open** for more input
(§4.2), §11's exit codes map from `IdleReason` rather than `CloseReason`, and the fold renders
whatever it is given — so a leak would be visible only to a user reading a session list that
says a session is over while the engine still accepts messages. `Completed` and `ParentGone`
stay separate variants because "the work finished" and "the parent went away" are different
facts to someone reading a session tree, and only one of them is a success.

**2. `TurnView.text: String` + `thinking: Option<String>` became `blocks: Vec<TurnBlock>`
(§3.5) — *not* a wire change, which is the part worth recording.** §6's `StreamChunk` is a
flat sequence with no block index and no end-of-block chunk, so one completion can carry
text → tool call → text, and with interleaved thinking
`Thinking → Text → ToolCall → Thinking → Text`. §4.2 makes each completion one turn, so those
blocks share a `TurnId`. With one accumulator per kind, §3.2's boundary rule — the `*Ended`
event **replaces** the accumulated text — destroyed the first block, and since deltas are
`Live` and never persisted the log could not recover it: **a loss in durable state rather
than a rendering defect**, and replay reproduced it. §3.2's rule is therefore reworded to
***closes the open block, replacing the text the deltas accumulated***, and a new paragraph
states that a turn may carry several text and thinking blocks, with the mechanism. **A1's
rule is narrowed in wording and unchanged in substance**: the self-heal property it exists
for survives verbatim under close-semantics, and §14.1's duplicated-delta test passes
unmodified.

**Two consequences were made spec rather than left as inference**, both placed where an
implementer of `fold` meets them: a delta arriving *after* a boundary **opens a new block**,
which is why **no block index is needed on the four delta/boundary events** — arrival order
*is* block order and the boundary is authoritative; and a *dropped* boundary is **not** a
silent merge but a `seq` discontinuity that §3.7 resyncs. `CallView`'s comment gained the
third: match a `ToolResult` to its call **by `CallId`, never by position**, because §4.2
dispatches calls concurrently and results arrive in no order a block position predicts.

**§14.1 gained one test for each**, and the first is the only thing that can fail on the fix:
stream text → call → text inside one completion and assert both blocks survive, in order and
with the call between them. The two pre-existing two-tier tests cannot see it, because each
uses a single-block turn — so a fold written against the old shape would have passed both.
The Engine bullet gained the paired lifecycle test: a primary session driven to
`AgentIdle(Done)` emits **no** `SessionClosed`, while a sub-agent whose final message is
returned closes with `Completed`; paired because either half alone passes under the wrong
rule.

**Depends on.** Nothing. **Gates** [03](../part1-foundations/03-protocol.md) (which now
carries the fold's three block rules), [08](../part2-engine/08-agentic-loop.md) (the loop
emits the boundaries that close blocks), [22](../part4-hosts-clients/22-tui.md) and
[21](../part4-hosts-clients/21-web-ui.md) (both render a block list now, not a `text`
field).

**Recommend accept.** ✅ **Accepted and applied 2026-09-19**, before M1.2 wrote a line of
the crate — and the user's question before approving it (*does this mean dropping work
already done, and restoring it later?*) was answered by counting rather than by reassurance:
**neither**. Both types had been added two commits earlier and had never been compiled,
fixtured, generated into `protocol.d.ts`, or rendered by a client; each appeared in the spec
at exactly **two** lines, measured before editing. Both fixes are forward edits, `CallView`
is unchanged and is *reused* as `TurnBlock::Call`, and six of `TurnView`'s eight fields
survive. **The sequencing rule that falls out is now in doc 03 §3.7**: an `Event`/`Command`
variant is fixture-pinned from M1.2, while a folded type is route-pinned until M1.3 — so
review the first kind earlier, and a review of the second kind stays cheap right up until the
crate exists.

### A27 — §3.2, §3.5, §4.5, §8.2, §14.1: four durable events with no fold destination, given one — ✅ **APPLIED 2026-09-20**

**Origin, and the method that found it.** Not a B-decision and not a type review: **the
first fold walk** — M1.2 Pass C1 writing `fold` itself, variant by variant. The diagnostic
was A1's own tier rule run backwards: *an event is `Durable` iff the fold consumes it*, so a
`Durable` variant the fold has nowhere to put is either a misclassification or a missing
destination, and every one found was the latter. That is a stronger form of the per-field
walk that produced U100/U107 — the field walk asks whether each type *resolves*; the fold
walk asks whether each event *lands*. It ran once, over one module, and found four. Filed as
[TODO](../TODO.md) §6 **U111**, approved in-session by the user on the recommendations
below, applied the same day — the register's second entry to arrive already applied, and
again **pre-fixture**, so the one wire change costs zero golden diff.

**1. `TurnBlock::User { content: Vec<ContentBlock>, from: ClientId }` (§3.5).**
`TurnStarted { role: User }` opens a user turn, but `UserMessage` — durable, and the *only*
carrier of the user's content blocks — had no destination: `TurnBlock` was
`Text`/`Thinking`/`Call`. A client bootstrapping per §3.7's message-only form, or replaying
a log, would render **every conversation with the user's side missing**, and §9.2's
"optimistic rendering … reconciled by `UserMessage { from }`" presupposed a fold that lands
it. One block appended per `UserMessage`, fields copied verbatim — the fold copies, it does
not interpret — and a turn may hold several when steering interleaves (§4.2). `TurnView`'s
build list gains `UserMessage`; it had been incomplete since the type was written.

**2. `SettingsChanged` + `SessionCreated`'s two new fields (§3.2) — the one wire change.**
`SessionInfo.model`/`.perm_mode` are rendered by §9.2's header and promised by §3.5 as "all
derived from the log", yet no event ever carried them: `SessionCreated` had neither, and
§3.3's `Set{Model,Agent,PermMode}` promise "resulting events" that did not exist.
`SessionCreated` now seeds both (resolved from agent config at creation, §5.2 — never read
from config at render, which would be hidden state), and one new `Durable` variant
`SettingsChanged { model?, agent?, perm_mode? }` mirrors what the three commands change.
**Event 30 → 31 (4 `Live`, 27 `Durable`)**; one variant rather than three because the
commands are one family and the fields say which fired. Doc 03 §3.4's counts and §3.7's
fixture budget (80 → **82**) are re-measured, not incremented.

**3. `SessionInfo.closed: Option<CloseReason>` (§3.5).** `SessionClosed` was durable with no
fold effect, so no `Snapshot`-derived surface could say a session is over — while §8.2's
list annotation has always read "(**status**, agent, cwd, cost, last_activity)" and U109's
own rationale was phrased in exactly those terms ("a session list saying a session is over
while the engine still accepts messages"). The field is the mechanism that rationale
presupposed; first close wins, a second close ignored — totality (G5) applied to a field.
§8.2 gains the companion sentence that `status`/`cost`/`last_activity` are **route-derived
display values, not `SessionInfo` fields** — closing the annotation↔type mismatch without
growing the type.

**4. `Compacted` folds `context.tokens_used = after` (§4.5).** The event's `after` *is* the
post-compaction token count and `Snapshot.context` *is* the meter, so the fold is a copy;
what was missing was the sentence saying so. §4.5's "UIs can render a divider" is now
explicitly a **live-rendering** affordance: a replay-only client sees the meter drop and the
turns intact, because the log lost nothing — the divider was never reconstructible from
`Snapshot` and no longer claims to be.

**Two fold rules stated, both because a second implementer is coming.** M1.3 generates
`protocol.d.ts` and the web UI folds in TS, so conventions that live only in this crate's
code would be reimplemented from guesswork. **(a) Openness is a convention, not a flag**
(§3.2): no event carries it and the log cannot store it, so the open block is *the turn's
trailing same-kind block* — exact rather than heuristic under §6's flat stream, because a
same-kind run ends only where another kind intervenes or the turn ends (tracker f124).
**(b) "No open block" splits by turn existence** (§14.1): a boundary for an unknown turn is
ignored (totality), while a boundary inside a started turn with no deltas behind it
**creates** its block — the half log-only replay depends on, and the correction of G5's own
§14.1 transcription, which had listed it as an ignore case (tracker f126; both halves are
one test).

**Three verified non-gaps, recorded so the next walk does not re-find them.**
`SubAgentFinished` — the summary reaches the parent's transcript via the `task` tool's
`ToolResult` (§5.3), and the child's usage lives in the child's own log. `TaskUpdated
{ Promote }` — a repo-side write to `.dacode/plans/` (§4.8); the tree's *content* is
M1.10's brainstorm. `Checkpoint` — genuinely unexposed (no §8.2 route, no `Snapshot`
field, and a `Rollback` UI needs the ids from somewhere), but checkpoints are doc 12/M2
and the id's *contents* are f107's carve-out, so it is filed as **U112** with M2 as owner
rather than invented inside a protocol pass.

**Depends on.** Nothing; composes with A26 (the block list `User` joins), A1 (the tier rule
that diagnosed it), A14 (`SessionInfo`), A15 (`Compacted`'s shape). **Gates**
[03](../part1-foundations/03-protocol.md) (counts, budget, fold rules — all updated in the
same pass), and at M1.3 the schema export and `protocol.d.ts`, which is why the sequencing
rule mattered: applied pre-fixture, this bundle cost **zero** rework; after C2 it would
have cost two goldens and a fold rewrite; after M1.3, a published wire change.

**Recommend accept.** ✅ **Accepted and applied 2026-09-20** on the user's in-session
approval of all four recommendations, with the §14.1 additions (the user-turn replay test,
the creates-its-block positive, `SettingsChanged`/`SessionClosed` landing in `SessionInfo`)
landing in the same pass.

---

### A28 — §3.2, §3.3, §4.3, §4.4: a denial carries a typed reason, and §4.4's pattern language is corrected — ✅ **APPLIED 2026-10-01**

**Origin.** Not a B-decision and not a fold walk: the **M1.1f doc pass** — the
field-by-field walk of §4.4 behind [16](../part3-capabilities/16-permissions.md),
plus a five-arm dependency measurement and a survey of the same question in the five
reference checkouts. It found four holes needing the user rather than an
implementer; the user ruled all four the same day (**g128** the matcher arm, **g129**
three of §4.4's own words, **g130** the `deny` floor, **U162** the wire shape), and
this delta is the spec half of those rulings. Applied in the same change as the
protocol code, on A27's precedent and because the two must not drift.

**1. `DenyReason`, and `Deny { reason: Option<DenyReason> }` (§3.2, §3.3, §4.3) —
U162's arm (a), refined.** §4.3 has required since A5 that *"which of the two layers
refused is recorded on `PermissionResolved` … distinguishable in the audit trail and
in the prompt shown to the user"*, and the landed `Deny` was fieldless, so the
requirement was **not expressible on the wire**. Five arms: `Rule`, `Mode` and
`Timeout` (four from §4.4 and one from §4.2's ask timeout), plus `NetHost` and
`NetAddress` — §4.3's two egress layers, which is the requirement that forced the
enum. Typed rather than a string on §3.2's own criterion and A3's `GuardKind`
precedent: a client branches on it (`NetHost` and `NetAddress` have different
remedies) and a fold cannot match a rendering.

⚠️ **The refinement is the part worth keeping, and it came from the pass that built
it rather than from the ruling.** Arm (a) as recommended was `Deny { reason:
DenyReason }`, non-optional — which puts a *diagnostic* on the **command** path:
every client answering `ResolvePermission` with a denial would have to name an
engine taxonomy whose only correct value there is a constant. `Option` removes the
burden without a sixth `Client` arm, because `None` **is** the answerer case,
corroborated by `PermissionResolved.by`. The reason travels with the decision rather
than sitting on the event for two measured reasons: `dacode-core`'s
`PermissionResolution` carries a decision to its waiter **without** the event, and a
sibling field on `PermissionResolved` would have spent lines `event.rs` does not have
— it is at **998 of the ungated 1 000-line cap**, so that shape would have dragged a
file split into a Step 0. ⚠️ **A third reason was given for arm (a) and was measured
false**: that a `Snapshot` fold could not see an event-side field. `fold.rs` handles
`PermissionResolved` as `answered(snapshot, req)` — it **discards** the decision — and
`Snapshot` retains no permission decisions at all. The arm survives on the two
reasons that are true; the false one is recorded because it was the recommendation's
own stated ground, and a deferral's reason is what the next pass acts on.

**This is the register's first entry with a golden diff.** `{"type":"Deny"}` →
`{"type":"Deny","data":{}}`, because adjacent tagging emits `data` even when every
field inside it is omitted — so the answerer case is **not** wire-compatible with the
pre-change bytes, which no longer decode (`missing field data`). One string, pinned in
both directions. It lands before **M1.3** publishes the schema export and
`protocol.d.ts`, which is §1's ordering constraint doing its job: the same reason
B17's `finding` variant gates M1.3 is the reason this shape is cheap now and a
compatibility event later.

**2. §4.4's three pattern *domains*, stated (g129 ii).** One glob vocabulary was
written over three keys whose correct `*` semantics **contradict**: `edit` needs it to
stop at `/` (or `src/*` allows `src/a/b.rs`), `bash` needs it to cross `/` (or
`rm -rf *` fails to deny `rm -rf /tmp/x`), `net` needs it to stop at `.` (or
`*.example.com` allows `evil.a.example.com`). Measured across five matcher arms:
`glob` and `globset` expose **one** knob for all three, and **the strict setting is
the worse failure** — `literal_separator(true)` turns `rm -rf *` into a no-op. So the
spec now names the domain per key, which is what makes [16](../part3-capabilities/16-permissions.md)
§16.4's `Domain`-parameterised matcher (ruling **R-c**) a transcription rather than an
invention.

**3. `.env` → `**/.env`, and the anchoring rule (g129 i).** §4.4's own example was
wrong in every arm: patterns anchor to the **whole** subject, so `deny = [".env"]`
matches `.env` and **not** `config/.env` — uniformly across `glob`, `globset`,
`wildmatch`, `regex` and a hand-rolled matcher. The corrected example is accompanied
by the anchoring sentence, so the next reader does not re-derive it from a bypass.

**4. A `deny` is a floor (g130).** §4.4 put session grants at the *top* of the
resolution order and A22 bounded `AllowPattern` to a subset of its request — but that
bound limits how **wide** a grant may be, not **which rung it overrides**, so a client
acting on the user's behalf (which §8.4's trust model makes reachable) could lift an
operator's `deny` for the rest of a session, and §14.1's "a grant may not exceed its
request" test would pass on the escalation. One sentence closes it, plus the mode
mapping the section had left implicit (`plan` turns `ask` into `deny` for
`edit/bash/net`, `yolo` turns `ask` into `allow`, `auto-edit` likewise for `Edit`) and
the ruling that **`deny` is never mode-overridable, including under `yolo`** — a `deny`
that voids in the one mode where nothing passes a human voids exactly where it matters
most. openclaw's measured shape corroborates the floor (most-restrictive-wins,
`minSecurity`/`maxAsk`); forge's and opencode's contradict it, both deriving a grant
*wider* than the request that produced it.

**Deliberately NOT in this delta.** **g128's** arm (hand-rolled, `Domain`-parameterised,
**+0 packages**) is a plan-side choice with no spec text — adding no crate, it does not
engage §2.3, and the user's added requirement (that the matcher stay adoptable by the
strict sandbox) is recorded on [U98](../TODO.md) rather than here, since its owner is
the row that fills B7's stub. **g129 (iii)** — `bash`'s advisory wording — is **B7's
owed transcription, still owed**: U63 was acked with a measurement that *strengthens*
it (four of five semantically identical `rm -rf` argv spellings evade a glob deny with
no shell involved, so §7.1's argv-not-shell property does not save the allow-list), but
the §4.4 honesty sentence belongs to B7's A12-adjacent pass and is not folded in here.
**R-a**, the concatenation direction §4.4 leaves unstated, is ruled plan-side in doc 16
rather than spec'd: the approved edit was narrow, and the direction is enforceable by a
table test where the prose would be enforceable by nothing.
