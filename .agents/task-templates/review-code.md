# Task Template: Review Code
# Template version: 0.1

Use this template when spawning a reviewer after a worker completes.

---

## Prompt Structure

```
You are reviewing code for dacode-todo-mcp. Load the `dacode-todo-mcp`
skill first.

## Your Instructions
{paste full content of .agents/roles/reviewer.md}

## Task Being Reviewed
- **Task ID:** {task_id}
- **Milestone:** {milestone}
- **Original spec:** {brief summary of what was requested}
- **Specifications implemented:** {design-doc section list}
- **Your emphasis:** {Deep semantic and logic analysis | Pattern consistency, spec compliance, convention drift}

## Source Files to Review
{list ALL source files created by the worker, with paths — the reviewer
 has read access; cite paths, do not paste whole crates}

## Test Files to Review
{list ALL test files / fixtures created by the worker, with paths}

## Dependency Context
{signatures of crates the code imports from}

## Base for Diff
{git ref the change is based on — reviewers scope clippy/ast-grep to
 `git diff --name-only <base>... -- '*.rs'`}

## Previous Review Notes (if any)
{paste previous reviewer feedback if this is a re-review iteration}
{indicate which iteration this is: "Review iteration 2 of 5, Phase 1"}

## Your Task
Review all files against the checklist in your instructions.
Run the diff-scoped checks: cargo fmt --check, cargo clippy -p {crate}
--all-targets, cargo test -p {crate}, ast-grep scan on changed files.
Produce a structured verdict per the report contract: status ok or
changes_requested.

If changes_requested:
- List all issues in the `issues` block (sev/at/why)
- sev high: malfunction, store corruption, determinism break, boundary
  or format violation, forged trailer, code past an unsigned decision
- Cite path:line for each; one-line why; a fix suggestion where it fits
- List any missing test cases / fixtures

If this is a re-review (iteration > 1):
- Verify previous issues are resolved
- Check that fixes didn't introduce new problems
- Note any regression from previous version
```

---

## Iteration Tracking

Include in the prompt:

```
## Escalation Status
- Phase: {1|2|3} (1=same agent retry, 2=re-scoped task, 3=fresh worker)
- Iteration: {N} of {max}   (limits: agent-rules.yaml → escalation)
- Previous attempts: {what was tried and what the reviewers said}
- Total review cycles so far: {count}
```

Roles, not model IDs, are named here — bindings live in
`.agents/config/agent-rules.yaml` only.

This helps the reviewer understand context and adjust thoroughness:
- Early iterations: be thorough, flag everything
- Later iterations: focus on the specific issues that keep failing
