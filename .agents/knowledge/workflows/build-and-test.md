# Build and Test

## When to Use

Every session that touches code, and every delivery. This is the host
recipe: macOS, Apple Silicon, Rust stable via rustup. Nothing here needs
a container or a cross toolchain — if a gate does not run, it is because
a tool is missing, and that is a `skipped` with a reason, not a pass.

## The gates, in order

```bash
cargo fmt --check
cargo check --workspace --all-targets 2>&1 | tail -c 5000
cargo test -p <crate> 2>&1 | tail -c 5000      # scoped while iterating
cargo test --workspace 2>&1 | tail -c 5000     # final gate before "done"
cargo clippy --workspace --all-targets 2>&1 | tail -c 5000
cargo doc --workspace --no-deps 2>&1 | tail -c 5000
cargo doc --workspace --no-deps --document-private-items 2>&1 | tail -c 5000
```

Rules:

- **The order is the order.** fmt is cheapest and clippy the most
  expensive; a failure early saves the later runs.
- **`cargo doc` runs twice, and both runs must be warning-free.** The
  plain invocation renders only publicly reachable items, so a private
  module's own doc page — and private items inside public modules — are
  never rendered and their broken intra-doc links are invisible to it;
  the `--document-private-items` run is what proofreads them. One broken
  link sat behind that blind spot from M1 until the M2 exit gate, seen
  only under the flag.
- **A test-name filter is a substring match over the whole test path**,
  not a module selector: tests in *other* modules whose names contain
  the word are selected too, and the collision is invisible in a verify
  line. Quote the passed count beside every name filter and read it
  against `rg -c '#\[test\]'` on the intended file, or use a
  module-path filter (`-- graph::tests::items`) rather than a bare
  word (`.agents/pitfalls.md` 4.1).
- **Scope while iterating** (`-p <crate>`, plus a test-name filter for a
  single test). Workspace-wide is for the final gate, never the inner
  loop.
- **Pipe long output** (`2>&1 | tail -c 5000`). An unpiped clippy run can
  dump more tokens than the fix is worth.
- **Clippy must be clean.** No `#[allow]` without a one-line reason
  naming the invariant relied on (`.agents/roles/_rust.md`).
- A gate you did not run is `skipped` with a reason. Never infer a pass.

## Harness self-checks (every delivery)

The workspace exists (M0, 2026-09-25), so these run *alongside* the code
gates at every delivery, and carry the weight alone on docs-only changes:

```bash
tools/check-links.py              # every relative link and anchor resolves
tools/agents-sync.py --check      # .opencode/ matches agent-rules.yaml
tools/agents-index.py --check     # INDEX.md is current
```

## Golden files and fixtures

The store format and the MCP wire are pinned by fixtures (spec §10).
The full scaffolding — fixture locations and `v<N>` naming, the shared
`assert_golden` helper, the `UPDATE_GOLDEN=1` regeneration ritual, the
twice-run/round-trip determinism tests — lives in
`.agents/knowledge/workflows/golden-files.md`. The non-negotiables:

- **A golden diff is a compatibility event.** Regenerating a golden is
  never a casual `--bless`; the commit message says what changed shape
  and why.
- Regeneration is a named command, run from the repo root, and the diff
  is read before committing — a golden that changed because a `HashMap`
  reordered is a determinism bug, not a fixture update.

## MCP smoke test

The protocol-level truth is a client handshake, not a unit test:

```bash
cargo run -p todo-mcp -- --stdio   # then send an initialize request on stdin
```

Until the server crate exists this is `skipped`. When it does, the smoke
test belongs in every delivery that touches the wire surface, and the
request/response pair belongs in the evidence (`task_update` →
`status: closed` with the pair quoted).

## Installing what is missing

| Tool | Install |
|---|---|
| Rust stable | `rustup toolchain install stable` |
| ast-grep CLI | `brew install ast-grep` |
| Serena | `uv tool install -p 3.13 serena-agent && serena init` |
| PyYAML (repo tooling) | `python3 -m pip install pyyaml` or a venv |

If an install is unavailable or refused, the dependent gate is
`skipped` with that reason — report it in the delivery rather than
silently dropping the gate.

## Pitfalls

- `cargo test --workspace` in the inner loop rebuilds and re-runs
  everything, including tests you cannot affect from your crate. Scope.
- `cargo clippy --all-targets` includes tests and benches; without
  `--all-targets` a lint in test code survives to bite the final gate.
- On a fresh clone, Serena/rust-analyzer returns nothing useful until the
  first `cargo check` has run and the index is built — grep and ast-grep
  carry that session (`.agents/roles/_search.md`).

## References

- `.agents/skill-base.md` — canonical command table
- `.agents/roles/_rust.md` — conventions the gates enforce
- `docs/todo-mcp-design.md` §10 — testing strategy
- `.agents/knowledge/workflows/task-tracking.md` — where gate results are recorded
