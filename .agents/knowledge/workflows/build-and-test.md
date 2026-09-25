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
```

Rules:

- **The order is the order.** fmt is cheapest and clippy the most
  expensive; a failure early saves the later runs.
- **Scope while iterating** (`-p <crate>`, plus a test-name filter for a
  single test). Workspace-wide is for the final gate, never the inner
  loop.
- **Pipe long output** (`2>&1 | tail -c 5000`). An unpiped clippy run can
  dump more tokens than the fix is worth.
- **Clippy must be clean.** No `#[allow]` without a one-line reason
  naming the invariant relied on (`.agents/roles/_rust.md`).
- A gate you did not run is `skipped` with a reason. Never infer a pass.

## Pre-M0 (no workspace yet)

Until the Cargo workspace exists, every code gate is `skipped` by
construction, and the doc gates carry the weight:

```bash
tools/attention.py --check        # spec/decisions/board consistency
tools/check-links.py              # every relative link and anchor resolves
tools/agents-sync.py --check      # .opencode/ matches agent-rules.yaml
tools/agents-index.py --check     # INDEX.md is current
tools/tasks.py check              # tracker discipline
```

## Golden files and fixtures

The store format and the MCP wire are pinned by fixtures (spec §10).
Conventions:

- Fixtures live under `crates/<crate>/tests/fixtures/` (path may be
  refined by the §10 work; update this article when it is).
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
request/response pair belongs in the evidence (`tools/tasks.py set <id>
--status done --evidence "..."`).

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
