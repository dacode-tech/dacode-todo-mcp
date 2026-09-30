<!-- Shared Rust CONVENTIONS layer. Inlined into agents that write or
     review this project's code, always after _search.md. Kept out of
     _common.md so non-coding agents (docs, search) do not pay for it.
     Seeded from docs/todo-mcp-design.md; grows as the codebase does. -->

## Coding Guidelines

### Rust

- Extract common code patterns into free functions or small helper types
  to avoid duplication; prefer composition over inheritance-style trait
  trees.
- Avoid nesting where possible. Prefer early return (`if !ok { return; }`)
  over `else` pyramids; `?` over manual match-and-propagate.
- `const`-like discipline: `const`/`static` for true constants, `let`
  bindings immutable by default (`mut` only where needed).
- Do not add superfluous turbofish or `as` casts when inference already
  works; do spell out types in public API signatures (no `impl Trait`
  leakage that hides the real type from callers reading docs).
- **Imports are always explicit — no wildcard `use`.** Library crates deny
  `clippy::wildcard_imports` at the crate root. Write
  `use std::collections::{BTreeMap, BTreeSet}`, never
  `use std::collections::*`. Globs hide what a module actually depends
  on, let name collisions resolve by accident when a dependency adds an
  item, and defeat the crate-boundary review that spec §2 depends on —
  you cannot audit a `use` you cannot see. The one accepted exception is
  `use super::*` inside a `#[cfg(test)] mod tests`, which clippy exempts
  by default.
- Try to find already-implemented patterns before inventing something.
  Search the workspace and the std/tokio docs first; a new utility is a
  last resort, and general-purpose helpers belong in the crate that owns
  the concept, not in a new `utils` dumping ground.
- **Avoid `unwrap()`/`expect()` on any path reachable from production
  code** — treat them as a last resort, never a convenience. Library
  crates deny `clippy::unwrap_used`, `clippy::expect_used`,
  `clippy::panic`, `clippy::unreachable`, `clippy::indexing_slicing` and
  `clippy::exit` at the crate root, so a stray `unwrap()` is a *build
  failure*, not a style note. The set is declared **once as workspace
  lints** and inherited per crate, so a new crate cannot forget it.
  `clippy::exit` is the mechanical form of "only the binary decides what
  is fatal": `main` returns an `ExitCode`, it does not call
  `process::exit`. **Tests are exempt through a mechanism, not a
  convention** — `#![cfg_attr(test, allow(...))]` at the crate root.
  Where an invariant genuinely guarantees success, prefer restructuring
  so the `None`/`Err` case is unrepresentable; if you truly must, use
  `expect("the invariant, named")` under a scoped `#[allow(...)]` carrying
  a one-line reason — never a bare `unwrap()`.
- The legitimate `#[allow]` cases are an invariant *you* control, or an
  **external library whose API leaves no fallible path** — a
  compile-time-validated regex, a `OnceLock` written exactly once at
  startup, a `FromStr` on a hardcoded literal. The reason must name the
  guarantee being relied on, so a reviewer can check it instead of
  trusting it.
- Reach for the non-panicking alternative first: `?` to propagate,
  `ok_or`/`ok_or_else` into the crate error, `unwrap_or`/`unwrap_or_else`
  with a *documented* default, `match`/`if let` with explicit arms, and
  `slice.get(i)` over `slice[i]`.
- No panics across crate boundaries — library crates return `Result`;
  only the server/CLI binaries decide what is fatal.
- Clippy is the lint of record: `cargo clippy --workspace --all-targets`
  must be clean. Do not `#[allow]` a lint without a one-line reason.

### Comments and docs

- Prefer `//` comments; `///` doc comments on every public API item
  (types, traits, functions, modules).
- Do not add comments that merely restate what the code does. Only
  comment non-obvious things. Reviewers explicitly flag "comment slop" —
  keep one concise sentence stating non-obvious rationale and stop; omit
  entirely when the code is self-evident.
- **The test is whether a reader could derive it from the code in front
  of them.** A comment naming a *remote* interaction — another module's
  invariant, a guarantee from `docs/todo-mcp-design.md`, a race this
  ordering avoids — earns its place. A comment re-explaining the next
  three lines does not.
- Length is a signal. Three lines explaining three lines usually means
  either the comment is redundant or the code needs restructuring.
- **No measurement in a source doc comment.** A line count, a test count,
  a ratio, a byte size or a duration written into a comment is a claim
  nobody re-measures, and it goes stale the moment the file moves — one
  such comment was restated three times and each version was wrong
  differently (`.agents/pitfalls.md` 5.2). State the mechanism and name
  the test or golden that enforces the property. Anything that genuinely
  must be counted has a ledger outside the source: file sizes are board
  §5's, test counts are the gate run's.
- `// SAFETY:` is mandatory on every `unsafe` block and must state the
  invariant that makes it sound — `unsafe` should be vanishingly rare in
  this workspace and each occurrence is a review event.

### Workspace architecture (spec §2 — enforced at review)

§2 is a proposal until its §12 decisions are signed; these rules bind
from the moment the crates exist:

- The domain crate (task graph, statuses, selection) has **no I/O, no
  async, and no workspace dependencies**. Determinism lives there and
  nowhere else.
- The store crate depends on the domain crate only, and owns persistence
  behind one trait. **The storage engine (D1) must not leak above that
  trait** — no engine type, no engine error, no engine feature flag in
  the domain or server crates.
- The server binary depends on domain + store and owns the MCP wire
  surface. No query logic that the domain crate could host; the binary
  translates, it does not decide.
- A `use` that crosses these lines is a defect, not a style choice.

### Determinism discipline (the product property — spec §8)

This server exists because prose boards are non-deterministic query
surfaces; the code must not reintroduce the property it removes:

- **Canonical serialization everywhere.** Fixed field order, stable key
  sorting (`BTreeMap` over `HashMap` in anything that reaches disk or the
  wire), no floating-point in persisted state, timestamps in one explicit
  format (RFC 3339 UTC).
- **Iteration order is part of the API.** Any list a tool returns —
  tasks, epics, dependents — has a documented total order (spec §5/§6),
  and two runs over unchanged state return byte-identical output. A test
  pins this, per §10.
- **Selection is a pure function.** "Next task" derives from graph state
  alone; never from wall-clock time, map iteration, or unspecified sort
  stability.
- The on-disk format is pinned by **golden files**: changing a serialized
  shape means changing its golden in the same commit, and a golden diff
  is a compatibility event — call it out.
- **Named-field structs, never tuples, in a persisted schema.** A
  positional shape is unreadable in the file that §4 makes
  human-auditable — `["camera:U1","camera:U2"]` does not say what either
  element is, while `{"shape":"task_to_task","from":…,"to":…}` does — and
  it degrades the repair surface §4.3 exists for, because serde's complaint
  about a wrong-length tuple is `invalid length 3, expected a tuple of size
  2` where a named field says which field failed. A field inserted in the
  middle is a silent reinterpretation of every existing file rather than a
  `deny_unknown_fields` rejection. The patch DTOs and the snapshot document
  are the precedent; `relates` is the format's one positional shape and
  carries a hand-written reader purely to keep its diagnostic (U12). This
  rule is argued on auditability and outlives the `dacode-json` tuple bug
  fixed at U11 — it would stand even with a perfect engine.

### Error handling

- Validate early, return before doing work; enrich with context at
  boundaries (`.map_err`, `.context`-style), never stringify an error
  into oblivion mid-stack.
- Library crates define their own error enums; the binary maps them to
  exit codes and MCP tool errors per spec §6's error contract.
- MCP tool errors are data, not panics: a bad id, a cycle, a missing
  repo returns a structured error the client can act on.

### Async and concurrency

- Settled by the signed D1 engine (spec §2): the JSONL event log and the
  in-memory graph need no async runtime, so the domain and store crates
  stay sync and runtime-agnostic — the binary may adopt tokio for rmcp,
  the libraries must not require it.
- Do not hold a `std::sync::Mutex` guard across an `.await` (once async
  exists). Prefer message passing over shared locks where the design
  allows.
- Single-writer store access (spec §8): concurrency control is a file
  lock at the process boundary, not optimistic retry logic scattered
  through call sites.

### Naming

| Item | Convention | Example |
|---|---|---|
| Crates | kebab-case | `todo-domain` |
| Modules/files | snake_case | `task_graph.rs` |
| Types/traits | PascalCase | `RepoRegistry`, `Store` |
| Functions/methods | snake_case | `next_task()` |
| Constants/statics | SCREAMING_SNAKE | `MAX_TITLE_LEN` |
| Lifetime params | short, meaningful | `'ctx` not `'a` for long scopes |

- Constants with units carry the unit: `TIMEOUT_MS`, `MAX_OUTPUT_BYTES`.

### File size and module shape

- The budget counts **production lines only**. Test code is not charged
  against it, in either of its two homes: a `#[cfg(test)] mod tests { … }`
  block inside a production file does not count, and a file that *is* test
  code — declared `#[cfg(test)] mod x;` by its parent, or living under
  `crates/<crate>/tests/` — does not count at all. Measure a production
  file with:

  ```sh
  awk '/^#\[cfg\(test\)\]/{exit} {n++} END{print n+0}' <file>
  ```

  which stops at the first in-file test module. A test-only file reports
  its whole length here; it is exempt by the rule above, not by the number,
  so do not "fix" one by splitting it.
- **Target 600 production lines, cap 1000.** Split by responsibility, not
  by size alone — a 900-line module with one coherent job can pass, a
  300-line grab-bag should not.
- Over target is an **exception and it is recorded**: one line in the
  file's module doc saying why it is over and what would split it, plus an
  entry in `docs/implementation/TODO.md` §5 (Debt). An over-target file
  with no recorded exception is a review finding; with one, it is
  acknowledged debt and not a finding. The cap is not waivable — at 1000
  production lines the file splits, exception or no.
- Unit tests live in `#[cfg(test)] mod tests` in the same file;
  cross-module/integration tests in `crates/<crate>/tests/`.

### Testing (spec §10)

- **Domain**: property tests on graph invariants — `blocks` stays acyclic
  under every mutation sequence, selection is a deterministic function of
  state, reparenting cannot orphan an epic's tasks.
- **Store**: golden files pin the on-disk bytes; round-trip
  (save → load → save) is byte-identical; corrupt-file behavior is a
  named error, not a panic.
- **Server**: every MCP tool is tested through the protocol (request in,
  response out), not by calling the handler function around the wire —
  the wire is the product.
- A bug fix ships with a test that fails before the change and passes
  after, encoding the *reported symptom*.
