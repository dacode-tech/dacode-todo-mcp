# Golden Files and Determinism Tests

## When to Use

Any task that produces bytes the project must keep stable: the store's
on-disk format (event log, snapshots, registry), canonical
serializations, and — from M3 — every MCP request/response pair
(spec §10). Read this before writing the first serializer, not after
the first golden breaks. This article is the U4 answer: the scaffolding
shape M1's store tasks implement.

## Where fixtures live

| Fixture | Path |
|---|---|
| Golden bytes owned by a crate's tests | `crates/<crate>/tests/golden/<subject>.v<N>.golden` |
| Importer inputs (sibling board snapshots) | `crates/<crate>/tests/fixtures/` (spec §10) |

`<N>` is the store format version the bytes pin (§4's versioning
requirement): during a migration, `event_log.v1.golden` and
`event_log.v2.golden` coexist, and the migration test replays v1 into
v2. Subjects are snake_case: `event_log`, `snapshot`, `registry`,
`serialize_graph`.

## The helper, written once

`crates/<crate>/tests/common/mod.rs` (the cargo pattern for shared
integration-test code; each test file starts with `mod common;`):

- `golden_path(name) -> PathBuf` — resolves via
  `env!("CARGO_MANIFEST_DIR")`, never the process cwd.
- `assert_golden(name, actual: &[u8])`:
  - under `UPDATE_GOLDEN=1`: write `actual` to the golden path **via
    temp+rename** (an interrupted update must not leave a torn golden),
    print what was written, pass.
  - otherwise: compare bytes. Missing golden → fail naming the
    regeneration command. Mismatch → fail with the path, both byte
    lengths, the first differing offset, and a line-level diff of the
    first differing region (the formats are line-oriented; the diff
    renderer itself iterates nothing unordered).

Every golden test in the workspace calls this one helper; a second
byte-compare implementation is a review finding.

## Regeneration ritual

```bash
UPDATE_GOLDEN=1 cargo test -p <crate> -- golden   # writes
git diff -- crates/<crate>/tests/golden/          # read every changed byte
```

- **A golden diff is a compatibility event** (spec §10): the commit
  message names the shape change and why.
- A golden that changed *without* an intended shape change is a
  determinism bug — something unordered (a `HashMap`, an unstabilized
  sort) reached the bytes. Fix the code, never the fixture.

## Determinism tests (the non-golden half)

Goldens pin bytes against history; these pin bytes against themselves:

- **Twice-run**: build state → serialize → serialize again → identical
  (`*_deterministic` tests).
- **Round-trip**: save → load → save → identical (`*_byte_stable`
  tests, spec §8).
- Fixed inputs throughout: timestamps are caller-passed constants
  (nothing below the binary reads a clock), ids fixed, no absolute
  paths inside goldens (tests inject a temporary data root).

## Pitfalls

- Integration tests run in parallel threads: one golden per subject,
  never two tests writing the same golden.
- LF endings, exactly one trailing newline, UTF-8; goldens are
  regenerated, never hand-edited.
- Goldens live under `tests/`, never `src/` (packaging) or `target/`.

## References

- `docs/todo-mcp-design.md` §4 (versioning, corruption), §8
  (determinism), §10 (golden files, determinism tests)
- `.agents/knowledge/workflows/build-and-test.md` — the gates around
  these tests
- `.agents/knowledge/workflows/task-tracking.md` — "a golden review" as
  a verify method
