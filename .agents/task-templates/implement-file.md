# Task Template: Implement File(s)
# Template version: 0.1

Use this template when spawning a worker to implement one or more source files.

---

## Prompt Structure

```
You are implementing part of dacode-todo-mcp. Load the `dacode-todo-mcp`
skill first.

## Your Instructions
{paste full content of .agents/roles/implementer.md}

## Domain Knowledge
{paste relevant sections from .agents/skills/dacode-todo-mcp/SKILL.md}

## Pitfalls to Avoid
{paste relevant sections from .agents/pitfalls.md}
{paste ALL entries from .agents/pitfalls-live.md}

## Task
- **Task ID:** {task_id}
- **Milestone:** {milestone}
- **Specification:** {design-doc section reference, e.g. "docs/todo-mcp-design.md §5.2"}
- **Crate:** {crates/<crate>}
- **Source files to create:**
  {list of files with expected paths}
- **Test files to create:**
  {list of test file paths / fixture paths}

## Specifications
{paste the relevant design-doc section content}

## Dependencies (interfaces you can import)
{paste public signatures from dependency crates — respecting spec §2 boundaries}

## Reference Code
{paste existing source file(s) for context}
This code is reference only. Improve upon it — don't copy across crate
boundaries.
Key improvements to make:
- {specific improvements for this task}

## Expected Output
Create each file listed above. For each source file:
1. Implementation following the specification
2. Result<T, E> error handling with context at boundaries
3. /// docs on every public item
4. Determinism discipline: BTreeMap/BTreeSet for anything persisted or
   returned, documented total order on every list (spec §8)

For each test file / #[cfg(test)] module:
1. Happy-path tests
2. Edge case tests (empty graph, missing id, boundary values, duplicate edge)
3. Error path tests (invalid input, cycle on write, unknown repo)
4. Store/wire surface touched: golden or protocol fixtures updated in
   the same change

Write all files. Do not ask questions — implement based on the spec.
If something is genuinely ambiguous, report status: blocked rather than
silently picking a side of a spec contradiction.
```

---

## Variable Substitution Guide

| Variable | Source |
|----------|--------|
| `{task_id}` | From `tools/tasks.py show` |
| `{milestone}` | Current milestone (agent-rules.yaml → milestones) |
| `{spec reference}` | `docs/todo-mcp-design.md` section numbers |
| `{crate}` | Spec §2 crate map |
| `{file paths}` | From the decomposition (one task never shares a file with another) |
| `{test file paths}` | `crates/<crate>/tests/` or inline `#[cfg(test)]` |
| `{specification content}` | From docs/todo-mcp-design.md |
| `{dependency signatures}` | From crate AGENTS.md / source files |
