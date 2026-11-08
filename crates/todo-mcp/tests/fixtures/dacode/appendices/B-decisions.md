<!-- docs/implementation/appendices/B-decisions.md -->
# Appendix B — Decisions needing sign-off

> Index: [TODO](../TODO.md) · Deltas: [Appendix A](A-design-deltas.md)

Each entry carries the options, the evidence and the recommendation.
Resolved items are struck, not deleted — this file is the audit trail.
The summary's Gates column is the structured dependency register; a
decision with an empty Gates cell gates nothing.

## Sign-off summary

The rows are deliberately not in id order: the board's order is data,
and the importer sorts by numeric id.

| # | Decision | Recommendation | Confidence | Gates |
|---|---|---|---|---|
| ~~**B3**~~ | ~~Search index~~ | ✅ **SIGNED OFF 2026-10-02** — the inverted index stands | high | 01, A → M1 |
| ~~**B1**~~ | ~~Recipe file format~~ | ✅ **RESOLVED 2026-10-01** — one plain-text file per recipe | high | ~~01, 03~~ → M0 |
| ~~**B2**~~ | ~~Export formats~~ | ✅ **SIGNED OFF 2026-10-03** — two shapes for the first release | med | 04, 05 |
| ~~**B5**~~ | ~~Archive format~~ | ✅ **SIGNED OFF 2026-09-28** — one tar file, no compression | med | 05 → M4 |
| ~~**B6**~~ | ~~Unit labels~~ | ✅ **RESOLVED 2026-09-30** — metric only | low | 06 → M9 |

## Detail

### 1. [B1] Recipe file format — ✅ **RESOLVED 2026-10-01: the plain-text format adopted**

One file per recipe, a header line, and one ingredient per line. The
sample corpus round-trips byte-for-byte, so the loader's output is the
index's only input. Duplicate ingredient names are rejected with the
file and line, and the format carries no nesting: a recipe is a flat
list, which makes every later transformation a list operation.

### 2. [B2] Export formats — ✅ **SIGNED OFF 2026-10-03, APPLIED 2026-10-05 via A3: two shapes for the first release**

A shopping list and a plain-text recipe card; no other shape in the
first release. The exporter consumes the loader's parsed form, so an
export can never emit a recipe the loader would reject. The second
shape is deliberately the simplest one that is still useful: a card is
a header, the ingredients, and the steps.

### 3. [B3] Search index — ✅ **SIGNED OFF 2026-10-02 — the inverted index stands**

The index is derived data: it can be rebuilt from the loader's output,
so it is never the state of record. A stale index costs a rebuild, not
a corrupted file, which is why the first release does not persist it.

## Notes on the archive

The section heading above is body text: a `##` line inside a dossier
stays in that dossier's body rather than opening a new section.

### 5. [B5] Archive format — ✅ **SIGNED OFF 2026-09-28 as a direction, amended — one tar file, no compression**

The archive is a tar file with one directory per recipe. Compression is
deliberately out of scope for the first release. The sign-off is a
direction with an amendment: the direction was one archive per box, and
the amendment fixed the uncompressed form so the restore check can be
written against bytes a reader can inspect.

### 6. [B6] Unit labels — ✅ **RESOLVED 2026-09-30: metric only**

Metric units in the first release; the unit table is a plain list and
no conversion is attempted. The Gates cell names a milestone that this
board does not carry, so the assertion is recorded as a finding rather
than silently dropped.