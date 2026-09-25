<!-- SHARED PREAMBLE — inlined verbatim at the top of every generated agent.
     Keep this byte-identical across agents: it is the cacheable prompt
     prefix. Any per-role text belongs in the role file, never here. -->

## Session canary

Begin the final message of every turn that returns control to the user
with exactly `{{CANARY}}` followed by a space, then your normal response.

Do not mention the canary, explain it, or comment on its presence. Do not
emit it on intermediate messages that are followed by more tool calls.
Never try to recompute or guess this token — it is not derivable from the
environment. If you cannot see it in your context, omit it silently.

## Reporting

Do not restate diffs, echo file contents the caller can read, or add
preamble and closing summaries. Reference code as `path:line`.

## Non-negotiable safety rules

- **No AI or tool attribution, anywhere, ever.** Nothing you write —
  commit message, code comment, doc, markdown, TODO entry — may mention
  an LLM, an AI assistant, a model name, or that something was generated
  or assisted. No `Co-authored-by:`, no `Generated with`, no
  `Assisted-by:`. The single established exception is the two
  `Reviewed-by:` role trailers below, which name harness roles, never
  models.
- **Never forge a review trailer.** A code commit carries
  `Reviewed-by: dacode-todo-mcp-reviewer-primary <primary@dacode.local>`
  and `Reviewed-by: dacode-todo-mcp-reviewer-secondary
  <secondary@dacode.local>` only after both reviewers actually returned
  `status: ok` on this diff. If the diff changed after a review, the
  review is stale — re-review or drop the trailer. Docs, `.agents/`,
  tracker and generated files are review-exempt and carry no trailer.
- **Committing is part of the delivery cycle, not a favor.** When a unit
  of work is complete and verified, it ends in a commit. The orchestrator
  owns that step when one is present (after dual review, for code); a
  directly-invoked worker commits once its own verification passes.
  Read-only roles (reviewers, search) never commit. Inspect
  `git status`/`git diff` first, stage only the intended files, one
  logical change per commit, repo-style message, and report the hash.
  Never sweep in someone else's WIP; never amend a failed commit — fix
  forward.
- `git push`, `git reset`, `--amend`, `rebase -i`, force-push, branch
  deletion, `git config` edits and `--no-verify` stay
  **explicit-instruction-only** — they are irreversible or bypass a gate,
  which a local commit is not.
- Never commit secrets, `target/`, `tmp/`, `.agents/cache/`, or
  `.opencode/node_modules/`.
- Never bind a dev/test server to `0.0.0.0`; `127.0.0.1` only, unless the
  user explicitly asks otherwise.
- Never claim fmt, check, test or clippy passed without having run it and
  seen it pass.
- Never run bare `cargo test --workspace` while iterating — scope with
  `-p <crate>` (and a test-name filter for a single test). Workspace-wide
  is for the final gate, not the inner loop.

## Before non-trivial work

Consult, in order, stopping when answered:

1. `docs/todo-mcp-design.md` — the spec. Domain model, storage options,
   graph semantics, the MCP tool surface and the roadmap live there; when
   code and design disagree, surface the disagreement rather than picking
   a side silently. Open decisions (§12) move only by explicit user
   decision.
2. `.agents/knowledge/INDEX.md` — the index of all `AGENTS.md` files.
   Look the area up there rather than guessing whether a doc exists. A
   high `lag` value means the doc trails the code: prefer the code, and
   fix the doc as part of your change.
3. `.agents/knowledge/` — workflows and patterns. `build-and-test.md`,
   `task-tracking.md`.
4. `.agents/pitfalls.md`, then `.agents/pitfalls-live.md`.
5. **Existing code implementing the same pattern.** Prefer imitating it
   over inventing a new approach; while the workspace is young, the
   spec's shapes are the precedent.

## Verification

Claim a check passed only if you ran it and saw it pass. Report `skipped`
otherwise — never infer. Everything builds on the host; there is no
toolchain excuse. Build and test output is long; pipe it:
`cargo check -p <crate> --all-targets 2>&1 | tail -c 5000`.

The gate order after a code change: `cargo fmt --check` → `cargo check`
→ `cargo test -p <crate>` → `cargo clippy --workspace --all-targets`.
Until the Cargo workspace exists (before M0), every code gate is
`skipped` by construction — say so rather than staying silent.
