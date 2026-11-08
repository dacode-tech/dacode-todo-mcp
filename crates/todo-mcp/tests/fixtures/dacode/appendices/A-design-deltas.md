<!-- docs/implementation/appendices/A-design-deltas.md -->
# Appendix A — Design deltas required

> Index: [TODO](../TODO.md) · Decisions: [Appendix B](B-decisions.md)

Changes to the design notes implied by this plan. The design moves only
by an explicit decision, so these entries are proposals until applied.
An entry is proposed, signed off, or applied; sign-off is not
application, and the index's Recommend cell says which of the three a
reader is looking at.

## Index

| # | Section | Delta | Depends on | Recommend |
|---|---|---|---|---|
| ~~A1~~ | §3.1, §3.2 | **The recipe loader** | ~~B1~~ ✅ signed | ✅ **APPLIED 2026-10-02** |
| ~~A2~~ | §4.1 | **The inverted index** | A1 | ✅ **SIGNED OFF 2026-10-03** |
| ~~A3~~ | §4.2, §5.1 | **The query parser** | B9 | ⚠️ **HALF-APPLIED 2026-10-04** |
| ~~A7~~ | §6.3 | **The ingredient cache** | M2 | ✅ **SIGNED OFF 2026-10-05** |

## Detail

### A1 — §3.1, §3.2: the recipe loader — ✅ **APPLIED 2026-10-02: landed with its round-trip test**

The loader reads one recipe per file, validates the header, and keeps
the ingredient order as written. The validation names the file and the
line, and a rejected recipe never reaches the index.

### A2 — §4.1: the inverted index — ✅ **SIGNED OFF 2026-10-03**

Build the index from the loader's terms; no ranking in the first pass.
The index is a map from term to recipe id, rebuilt on demand, and the
delta deliberately does not make it persistent.

### A3 — §4.2, §5.1: the query parser — ⚠️ **HALF-APPLIED 2026-10-04**

The parser landed for ingredient terms; phrase queries remain open, so
the delta is half-applied. The remaining half is the phrase syntax and
its tests, not a change to the first half's shapes.

### A6 — the archive header → ✅ **APPLIED via A1, 2026-10-06**

The header borrows the loader's format field, and its span carries no
colon, so the task takes the whole span as its title and no section
references. This dossier also carries no index row: the cross-check is
one-directional, so dossiers may exceed the tables.

### A7 — §6.3: the ingredient cache — ✅ **SIGNED OFF 2026-10-05**

The cache is a plain map keyed by ingredient name, rebuilt whenever the
loader's output changes. The index row names M2 as its dependency, but
a milestone is never a legal edge target, so the assertion is recorded
as a finding instead.

## Deltas applied

| # | Section | Applied | Date | Note |
|---|---|---|---|---|
| ~~**A1**~~ | §3.1, §3.2 | **The recipe loader** | 2026-10-02 | applied with its test |
| ~~**A3**~~ | §4.2, §5.1 | **The query parser** | 2026-10-04 | first half only |