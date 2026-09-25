opencode's task tool takes `subagent_type` and **has no `model`
parameter** — a model cannot be chosen per call. Models are bound to
named agents in `.agents/config/agent-rules.yaml` and propagated by
`tools/agents-sync.py`. Spawn by name only:

- `dacode-todo-mcp-implementer` — code + tests for one scoped task
- `dacode-todo-mcp-reviewer-primary` — deep semantic and logic analysis
- `dacode-todo-mcp-reviewer-secondary` — pattern consistency, spec compliance, convention drift
- `explore` — read-only search across the codebase and docs

Spawn both reviewers in one message so they run in parallel.
