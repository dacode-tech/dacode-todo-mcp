<!-- docs/implementation/TODO.md -->
# recipe-box — implementation plan

A fictional project used as the dacode-dialect importer's synthetic
fixture: a command-line recipe box with a local index and a few export
shapes. Nothing here names a real product, person, path or tracker.

The board's registers are the source of the imported graph: §2's
decision summary, §4's doc register, §5's milestones and sub-steps, and
§6's follow-ups. Ids are never renumbered, so gaps are normal.

## 1. Current state

The loader and its round-trip corpus are in place; the search index is
the live milestone. The table below duplicates per-item state and is
walked past by the importer. A row moving here is a summary of a
register entry, never a second source of truth: when the two disagree,
the register wins and the row is the bug.

| Group | State |
|---|---|
| Loader | closed |
| Index | in progress |
| Export | design blocked |
| Archive | decision blocked |

## 2. Decisions ([Appendix B](appendices/B-decisions.md))

| # | Decision | Recommendation | Status |
|---|---|---|---|
| ~~B1~~ | Recipe file format | ✅ **RESOLVED 2026-10-01** — one plain-text file per recipe | ✅ resolved |
| ~~B2~~ | Export formats | ✅ **SIGNED OFF 2026-10-03** — two shapes for the first release | ⚠️ resolved, half-applied |
| ~~B3~~ | Search index | ✅ **SIGNED OFF 2026-10-02** — the inverted index stands | ~~⚠️ reopened~~ → ✅ resolved |
| ~~B5~~ | Archive format | ✅ **SIGNED OFF 2026-09-28** — one tar file, no compression | ⏳ awaiting a demo | 05 → M4 |
| ~~B6~~ | Unit labels | ✅ **RESOLVED 2026-09-30** — metric only | ✅ resolved |

B1a (`recipe::Format`) is **done** — the loader parses every recipe in the
sample corpus and its round-trip test is green.

## 3. Spec deltas ([Appendix A](appendices/A-design-deltas.md))

The queue below is prose, walked past by the importer. It groups the
deltas by the sections they touch and records which of them are already
normative in the design notes. A group is a reading aid, not a
dependency: Appendix A's Depends-on column is the only dependency
register.

| Group | Deltas | State |
|---|---|---|
| Loader | ~~A1~~ | applied |
| Search | A2, A3 | in review |
| Archive | A6 | proposed |

## 4. Doc depth — 2 of 14 docs still summaries

Already expanded (12): `README`, `00-overview`, **`01`**, 02, 03, and the **seven** appendices (A–G).

| Milestone | Docs still summary | Count |
|---|---|---|
| **M0** | — *(both guide docs deepened in the design pass)* | **0** |
| **M1** | 04 search index, ~~05 archive format~~ *(deepened in the design pass)* | 1 |
| **M2** | 06 export profiles | 1 |
| **M4** | — *(nothing planned yet)* | **0** |

A doc leaves the summary column when its register marker is flipped, in
the same change as the work that discharges it. The count in the
heading is prose; the register above is the machine-read half.

## 5. Implementation work by milestone

Each milestone's first task is to deepen its docs (§4), then build. The
sub-step tables carry the gate that proves the step.

| | Sub-step | Gate |
|---|---|---|
| **M0.1** | the recipe file reader | `cargo test -p recipe-box -- format` |
| **M0.2** | the round-trip corpus | `cargo test -p recipe-box -- roundtrip` |

- [x] **M0 — recipe file format. ✅ COMPLETE 2026-10-01** (both steps in
      one pass; the loader and its round-trip corpus are green). The
      format is a plain-text file with a header line, one ingredient per
      line, and a trailing blank line. The loader rejects a duplicate
      ingredient name and reports the file and line it came from, so the
      authoring mistake is fixable without opening the parser.

| | Sub-step | Gate |
|---|---|---|
| **M1.1** | the inverted index | `cargo test -p recipe-box -- index` |
| **M1.2** | the query parser | `cargo test -p recipe-box -- query | tee query.log` |

- [ ] **M1 — search index. ▶️ started 2026-10-04** (the index lands
      first, then the query parser). ⚠️ The ranking pass is unmeasured:
      the corpus is too small to distinguish a real ordering from a
      stable sort, so the numbers wait until the corpus grows.

### Ranking notes

Ranking is deliberately out of scope until the index is green; the
heading above is body text, not a register.

| | Sub-step | Gate |
|---|---|---|
| **M2.1** | the shopping-list template | `cargo test -p recipe-box -- export` |

- [ ] **M2 — shopping-list export. 🧠 a design question is still open**
      gates M4: the export cannot ship before the archive header is
      settled. The open question is whether a list may merge two
      recipes' identical ingredients; merging is convenient and lossy.

| | Sub-step | Gate |
|---|---|---|
| **M4.1** | the archive writer | `cargo test -p recipe-box -- archive` |
| **M4.2** | the restore check | `cargo test -p recipe-box
-- restore` |

- [ ] **M4 — archive format.** blocked on B5 — one tar file, no
      compression, and a restore check beside the writer. The restore
      check re-reads the archive into a fresh directory and compares the
      loader's parsed form, not the bytes.

## 6. Open follow-ups not tied to a milestone

Every item carries a stable `U` id; ids are never renumbered, so gaps
are normal. A `before M4` in prose stays body text, never an assertion.
The subsections below are grouping only: an item's epic is the
`Follow-ups` container, not the subsection it sits in. An item with no
subsection before it opens the implicit leading group, which the
importer carries with a null title.

<!-- attention-meta — machine-readable; read by nothing here.
action: U2=ack
-->

- [ ] **U1 · Cache the parsed recipe file.** The loader re-reads the
      whole corpus on every query; cache the parsed form and invalidate
      on the file's modified time. overdue by M2 is prose, not an
      assertion.

### Search

- [ ] **U2 · Warm the index at startup.** ▶️ started — the first pass
      builds the index lazily, which makes the first query slow.
- [ ] **U4 · Rank by ingredient
      overlap.** blocked on U5 — the index must carry ingredient terms
      before ranking can be measured.
- [x] **U5 · Normalise ingredient names. ✅ done 2026-10-02** the loader
      lowercases names and trims whitespace.
- [x] **U6 · ⚠️ Drop empty recipes at load.** The loader refuses a recipe
      with no ingredients; the test corpus carries one. unblocked on
      U5 is prose; the item is already closed.
- [ ] **U7 · Cache warm-up ✅ verified by hand.** gates M8 in the plan
      notes, but no such milestone exists.
   - [ ] **U8 · An indented checkbox is body text.** It must not parse
         as a register line.

### Export

- [ ] **U9 · Emit a shopping list.** blocked on B1 — the loader's
      format decides what can be exported. The list is grouped by
      ingredient and keeps the recipe as a trailing note.
- [ ] **U10 · Archive a recipe box.** blocked on A3 — the archive header
      rides the search delta. The writer stays behind the loader's
      parsed form so an archive never carries unvalidated text.
- [ ] **U12 · Add a `--dry-run` flag.** blocked on M1 — the design of
      the flag belongs to the index pass. The flag must print the same
      lines the real run would write, in the same order.
- [ ] **U15 · Share a recipe box. 🧠 needs a threat model** — sharing is
      deferred until the export shapes settle, and it is the one item
      that would need a network surface the project does not have.
- [ ] **U17 · Tune the ingredient cache.** due by M4 in the planning
      note, which is prose here rather than a dependency. The cache is
      a plain map and the tuning is a measurement, not a design.
- [ ] **U18 · Record the loader's parse errors.** blocked on B9 — the
      error shape was proposed but no such decision exists, so this is
      recorded as a finding rather than an edge.

## 7. Session log

Not a register: this section is past §6 and the walker ignores it.
Sessions are files, and the importer reads no prose here.

## Glossary

An unnumbered header is walked past as well; the walker identifies a
section by the number in its header, never by position.

- **register** — a section the importer reads: §2, §4, §5 and §6.
- **dossier** — one decision's or delta's long-form entry.
- **gate** — the command a sub-step's completion is checked against.