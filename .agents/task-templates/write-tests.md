# Task Template: Write Tests
# Template version: 0.1

Use this template when spawning a worker specifically to write tests for
already-implemented code. Used when a reviewer identifies insufficient test
coverage, or for adding property/stress tests.

---

## Prompt Structure

```
You are writing tests for dacode-todo-mcp. Load the `dacode-todo-mcp`
skill first.

## Context
{paste relevant sections of .agents/roles/implementer.md — Testing, and
 docs/todo-mcp-design.md §10 (verification and testing)}

## Code Under Test
{list the source file(s) that need tests — with paths, not pasted content}

## Existing Tests (if any)
{list existing test files — the worker should ADD to these, not replace}

## Reviewer's Coverage Gaps
{paste the reviewer's feedback about missing tests}

## Task
- **Task ID:** {task_id}-tests
- **Crate:** {crates/<crate>}
- **Source file:** {source path}
- **Test files to create/update:** {test file paths}

## Required Test Categories

### 1. Happy Path
- Normal usage patterns
- Common input combinations
- Expected output verification

### 2. Edge Cases
- Empty graph / empty repo / no tasks under an epic
- Missing or duplicate ids; self-edges; multi-parent attempts
- Single-node graphs; deep chains (topological order under length)
- Unicode/special characters in titles and notes
- Repo path updates while tasks exist (registry semantics, §7)

### 3. Error Paths
- Invalid input that should produce specific errors
- Cycle on `blocks` write — rejected before the write lands
- Unknown repo/epic/task ids — structured error, not a panic
- Corrupt store file — named error, not a panic

### 4. Determinism Verification (the product property, §8)
- Run the operation twice on unchanged state; compare BYTES
- Serialize → load → serialize round-trip is byte-identical
- Selection over a graph with many equal-priority candidates returns the
  same answer on every run (no HashMap iteration order leakage)

### 5. Store & Wire Specifics (§10)
- Golden files: every serialized shape touched
- MCP tools: request in over the protocol, response out — never by
  calling the handler around the wire
- Error responses carry the stable codes §6 defines

### 6. Property / Stress Targets (if applicable)
- Random mutation sequences keep `blocks` acyclic (assert after each op)
- Roundtrip: encode → decode for every persisted type
- Adversarial input: truncated store file, oversized title, unknown fields
- Bounded execution: no test depends on wall-clock time

## Output
Create the test files using the project's conventions: units in
#[cfg(test)] mod tests, integration in crates/<crate>/tests/, fixtures
under tests/fixtures/.
Verify: cargo test -p {crate} 2>&1 | tail -c 5000 — and for a bug fix,
show the new test FAILING on the pre-fix code if the orchestrator asks
for that evidence.
```
