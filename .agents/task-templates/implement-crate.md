# Task Template: Implement Crate/Module
# Template version: 0.1

Use this template when spawning a worker to implement an entire crate or
tightly-coupled set of modules (e.g., the task graph, the store backend,
the MCP tool surface).

---

## Prompt Structure

```
You are implementing a complete module for dacode-todo-mcp. Load the
`dacode-todo-mcp` skill first.

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
- **Module:** {crate or module name, e.g., "todo-domain task graph", "todo-store event log"}
- **Specifications:** {list of ALL design-doc sections this module implements}
- **Files to create:**
  {list of ALL files in this module with paths and estimated LOC}

## Architecture
{describe how the files in this module interact}
{which file is the entry point (lib.rs public surface)}
{data flow: what comes in, what goes out}
{who owns shared state; where errors surface}

## Specifications
{paste ALL relevant design-doc section content}

## Dependencies
{paste public signatures from crates this code imports — spec §2 boundaries apply}

## Reference Code
{paste existing module source for context}

## Expected Output
Create ALL files for this module. Ensure:
1. Each file ≤ 600 LOC (1000 max with justification)
2. Clear module boundaries; lib.rs re-exports the public surface
3. Every public item has /// docs
4. Result<T, E> error handling; crate-local error enum
5. Workspace-lint inheritance if this is a new crate (the six denied
   clippy lints + cfg_attr(test) exemption) — a new crate without it is
   a review finding
6. Golden files for any serialized shape owned by this module
7. Unit tests per module (#[cfg(test)]) + integration tests in tests/
8. cargo fmt --check, cargo check -p {crate} --all-targets,
   cargo test -p {crate}, cargo clippy -p {crate} --all-targets all pass

Write all files. Maintain consistency across the module.
```

---

## When to Use This Template

- Files within a module are tightly coupled and share internal types
- Multiple files need to be consistent with each other
- The module has a clear entry point and internal structure
- A whole crate is being scaffolded (M0 workspace bootstrap)

## When NOT to Use

- Independent files across different crates (use implement-file.md — and
  remember: one task per crate, never two tasks in one file)
- Single complex file (use implement-file.md)
- Utility code that's independently usable (use implement-file.md)
