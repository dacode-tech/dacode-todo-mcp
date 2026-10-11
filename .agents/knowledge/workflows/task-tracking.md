# Task Tracking

## When to Use

Always, once work has more than three steps. This article defines the
convention concretely, so nobody has to infer it from the tool surface.

## The record homes

The store is the tracker of record (the full-switch cutover, the user's
2026-10-09 sign). This repo's nodes live in the server's own store,
under the `dacode-todo-mcp` slug in the durable root, and session work
is tracked through the wired `todo-mcp_*` tools. The native todo tool
stays denied for the orchestrator (U16b) — the MCP server is that
role's wired todo surface; the live view is the messages plus the
store.

| | the store | the handover file | `decisions.md` + the spec |
|---|---|---|---|
| Lives at | the durable root, per-repo | `docs/implementation/handover/E<n>.md` | this repo, committed |
| Scope | arcs, work units, findings, debt | the epic's session state | signed decisions, the normative design |
| Updated by | `task_add` / `task_update` | every session working that epic | the decision's work order |
| Answers | "what is open, what is verified, what is next" | "where did this epic stand" | "what was decided, and why" |

Reasoning never goes in the store or the handover — it goes in
`decisions.md` or the spec. The store and the handover are state.

## The node mapping

The old tracker's kinds map onto the store's node kinds:

| Old tracker kind | Store shape |
|---|---|
| `goal` (an arc) | an epic — `epic_add` at arc start, closed when its tasks close |
| `task` | a task — `task_add`, **one reviewable commit** |
| `gate` | a task with `kind: gate` |
| `finding` | a task with `kind: finding` |
| `debt` | a task with `kind: debt` |

The id vocabulary: the store's own numbering. Pre-cutover nodes carry
`board:<id>` in `references` (the cutover playbook's rule); new work
takes the next server id.

## The session-tracking convention

- **`task_add` with `verify` decided up front.** The domain does not
  enforce verify-at-creation — the sweep catches a close whose verify
  was never set, and the convention is to decide the method when the
  node is created, not when it is closed. If the method is not
  obvious, ask; do not write a placeholder and move on.
- **`task_update` → `status: closed` with `evidence`** — the domain
  refuses the close without it. Evidence names the command that was
  run and what it printed, never a restatement of the title.
- **`task_update` → `status: blocked` with `note`** — the domain
  requires the note. An unexplained block is indistinguishable from an
  abandoned node.
- Findings and debt are recorded **as they appear**, mid-session: a
  shortcut written down is a decision; one that is not is a defect
  waiting to be rediscovered.

## The sweep — what `tasks.py check` used to do

The domain now enforces at the write what `check` used to flag after
the fact: a close without evidence and a block without a note are
impossible. What the domain cannot see, the sweep must:

At session end, run `status_summary` and `next_task` over the repo and
read the open nodes **against what this session actually delivered**.
The store cannot see `git log` — the session's delivery knowledge is
the cross-check. Disposition every open node you touched:

- work finished → closed with evidence
- work stopped → blocked with a note naming what it is blocked on
- work still owed → left open, with the consumer named
- a record gone stale (the work landed in git but the node never
  moved) → closed with the evidence, or marked `wontfix` with a note

**Never report work complete over an undispositioned node.** Reporting
success over a gap spends the user's trust to hide one.

## Every node needs a `verify` when it is created

Not when it is closed. Deciding it late means deciding it to fit
whatever happened.

Acceptable verification methods on this project:

| Method | Example |
|---|---|
| A test | `cargo test -p todo-domain -- graph::tests::cycle_rejected` |
| A build gate | `cargo check --workspace --all-targets` clean |
| A lint gate | `cargo clippy --workspace --all-targets` clean |
| A golden review | "regenerated `tests/fixtures/store_v1.json`, diff read and explained in the commit" |
| A smoke run | `cargo run -p todo-mcp -- --stdio` answers `initialize` + one tool call |
| A harness check | `tools/agents-sync.py --check`, `tools/check-links.py` |
| A store query | `todo-mcp progress --json` shows the node closed, the totals moved |
| A doc assertion | "spec §4 names the engine and §12 item 1 is deleted" |
| `manual:` | `manual: user confirms the tool list against their client` |

## How a session flows

```
user asks something
  → epic_add (if a new arc) / task_add with verify (one reviewable commit each)
  → work happens; findings and debt recorded as they appear
  → task_update → closed with evidence (the domain enforces it)
  → the sweep: status_summary + next_task read against the deliveries
  → the epic's handover gains the session entry
  → a hard-won fact goes into the spec's numbered sections
  → a signed decision executes its On sign-off: order
  → commit
```

`/todo-next-imp` reads the store and the handover, and proposes one
step. `/todo-close` runs the exit gate over all of it.

## Pitfalls

- **Closing a node with its own title as evidence proves nothing.**
  "Implemented the event log" is a restatement. "cargo test -p
  todo-store: 14 passed; golden round-trip byte-identical" is evidence.
- **Two tasks must never write the same file.** That is a decomposition
  rule, and the store is where it becomes visible.
- **The store is outside the repo.** Its durability is the durable
  root's, not git's — the board backup beside the store at cutover is
  the pre-cutover record's insurance. Anything that must be
  reproducible from a fresh clone (goldens, fixtures, the spec) stays
  in git; the store holds the tracking state.
- **A render is not a record.** `progress --json` and the page are
  renderings of the store, never sources of it. If a rendering
  disagrees with the store, the store wins.
- **Read-your-writes.** A lock-free read (a render, a fresh server's
  read tool) sees only committed appends — after a mutation, answer
  from the same session's tools, not from a second process spawned
  before the write landed.

## References

- `.agents/skills/todo-next-imp/SKILL.md` — the procedure and the exit gate
- `.agents/roles/orchestrator.md` — when to open a node, how to size work
- `docs/implementation/decisions.md` — the decision record
- `README.md` — the cutover playbook (the per-epic HANDOVER convention)
