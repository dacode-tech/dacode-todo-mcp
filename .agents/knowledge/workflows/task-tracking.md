# Task Tracking

## When to Use

Always, once work has more than three steps. This article defines the
convention concretely, so nobody has to infer it from the tool's help
text.

## Three trackers, three different jobs

They are not interchangeable and substituting one for another loses
work.

| | Harness todo tool | `tools/tasks.py` | `docs/implementation/TODO.md` |
|---|---|---|---|
| Lives | in the session | `.agents/cache/tasks/<branch>.yaml` | `docs/implementation/TODO.md` |
| Committed | no | **no** (gitignored) | **yes** |
| Survives | nothing | a session; dies on branch switch | everything |
| Renders | in the TUI, live | `tasks.py show -v` | a diff a reviewer reads |
| Scope | this turn's steps | this branch's work units, gates, findings, debt | the project's progress |
| Answers | "what is happening right now" | "what am I doing, and how will I know it worked" | "what has this project done, and what blocks it" |

**Use all three.** The todo tool is how the user watches progress without
asking. `tasks.py` is how the *next* session knows what was half-done.
The board is the only one a reviewer can see, because the other two are
session-local or gitignored.

An instruction to "update the TODO" is ambiguous between the last two.
If context does not settle it, **ask**.

The irony is deliberate and temporary: this project *builds* a
deterministic task tracker, and until its own §12 decisions are signed
and it works, the markdown board is the tracker of record. Never dogfood
an unsigned design.

## `tools/tasks.py` — the convention

### Node kinds

| Kind | Id | Use for |
|---|---|---|
| `goal` | `g1` | What the user actually asked for. The thing to check the result against |
| `task` | `t1` | One unit of work — for this project, **one reviewable commit** |
| `gate` | `gN` … | A checkpoint that must pass: the cargo gates, dual review, a golden-file review |
| `finding` | `f1` | Something discovered that was not the point of the task |
| `debt` | `d1` | A deliberate shortcut, recorded rather than forgotten |

Ids are assigned by the tool from the kind's first letter. Note `goal`
and `gate` share the `g` prefix, so read the `[kind]` column, not the id.

### Statuses

`pending` · `active` · `blocked` · `done` · `wontfix` · `standing`

- `blocked` **requires** `--note` saying what it is blocked on. An
  unexplained block is indistinguishable from an abandoned item.
- `standing` is for a `goal` that is *meant* to stay open — the whole
  server, for instance. It requires `--note`, and the tool refuses it on
  anything that is not a goal, because otherwise it would be a way to
  turn the gate green by relabelling.
- `done` **requires** `--evidence` and a `verify:` that is not `TBD`.

### Every item needs a `verify:` when it is created

Not when it is closed. `tasks.py check` refuses to sign off a `done`
item that never had one, and deciding it late means deciding it to fit
whatever happened.

Acceptable verification methods on this project:

| Method | Example |
|---|---|
| A test | `cargo test -p todo-domain -- graph::tests::cycle_rejected` |
| A build gate | `cargo check --workspace --all-targets` clean |
| A lint gate | `cargo clippy --workspace --all-targets` clean |
| A golden review | "regenerated `tests/fixtures/store_v1.json`, diff read and explained in the commit" |
| A smoke run | `cargo run -p todo-mcp -- --stdio` answers `initialize` + one tool call |
| A harness check | `tools/attention.py --check`, `tools/agents-sync.py --check`, `tools/check-links.py` |
| A doc assertion | "spec §4 names the engine and §12 item 1 is deleted" |
| `manual:` | `manual: user confirms the tool list against their client` |

`verify: TBD` is not acceptable: either decide, or ask the user.

### Commands

```bash
tools/tasks.py show -v
tools/tasks.py add -  goal "build the store crate"       --verify "..."
tools/tasks.py add g1 task "todo-store: JSONL event log" --verify "cargo test -p todo-store event_log"
tools/tasks.py add g1 gate "dual review"                 --verify "both reviewers status: ok"
tools/tasks.py set  t1 --status active
tools/tasks.py set  t1 --status blocked --note "needs D1 signed first"
tools/tasks.py set  t1 --status done --evidence "cargo test -p todo-store: 14 passed; clippy clean"
tools/tasks.py check          # what blocks a defensible sign-off
tools/tasks.py budget         # context window and work budget
```

### When opening a tracker is mandatory

Not a judgement call. Any one of these:

- the work has more than three steps, or spans more than one file
- you are about to spawn a subagent
- you record a finding or take on debt
- the user asks a question you cannot fully answer this session

If unsure, open it. A tracker nobody needed costs one file write.

### `check` is the gate, not a formality

It exits non-zero on a `done` item with no evidence, a `done` item whose
`verify` was never decided, an unclosed goal, an unexplained block, or a
`standing` item with no reason.

**Never report work complete while `check` exits non-zero** without
saying so in the same message and naming what it flagged. Reporting
success over a failing gate spends the user's trust to hide a gap.

What `check` **cannot** see is the failure that actually happened at M1's
gate: a `pending` item whose work is already in git history, a `blocked`
item whose blocker was resolved days earlier, a finding whose consumer
shipped. Each reads as "still to do" and hides the truth, which is that
nothing is. So the open list needs a sweep against `git log` — every
delivery disposes of the items it consumed, a milestone gate disposes of
all of them. An item that is deliberately never going to be fixed is
`wontfix` with a note, not `pending` forever, and a deferral that must
outlive the branch gets a `U` id on the board, because this tracker is
gitignored and branch-keyed.

## `docs/implementation/TODO.md` — the convention

- **§4 is unowned follow-ups. §5 is debt.** Those two are the only
  sections `tools/attention.py` reads as action items.
- An item is `- [ ] **UN · Title** — description`, and the `U` number is
  **permanent**. Take the next unused one; never renumber, never reuse.
  Gaps mean an item was resolved, which is information.
- §1 mirrors the open decisions, §2 the milestone state, §3 what landed,
  §6 the session narrative — newest first, one short paragraph, written
  for someone who was not here.
- **Reasoning does not go on the board.** It goes in
  `docs/implementation/decisions.md` or in the spec. The board is status.
- A delivery that does not move the board is a review finding, not a
  follow-up.

## How they connect

```
user asks something
  → harness todo (live, this turn)
  → tasks.py goal + tasks + gates (durable, this branch)
      each task = one reviewable commit, each with its verify decided up front
  → work happens; findings and debt recorded as they appear
  → tasks.py check exits 0
  → TODO.md moves: §3 gains a line, §4/§5 gain new U ids, §6 gains a paragraph
  → a hard-won fact goes into the spec's numbered sections
  → a signed decision executes its `On sign-off:` order
  → commit
```

`/todo-next-imp` reads the first three and the spec, and proposes one
step. `/todo-close` runs the exit gate over all of it.

## Pitfalls

- **The `tasks.py` tracker is keyed by branch and gitignored.** Switching
  branches switches trackers, and none of it survives a fresh clone.
  Anything that must outlive the branch goes on the board.
- **An unborn branch keys the tracker as `HEAD`.** Before the first
  commit, git reports no branch name, so the file is
  `.agents/cache/tasks/HEAD.yaml`; the first commit re-keys to the branch
  and orphans it. Move `HEAD.yaml` to `<branch>.yaml` right after the
  initial commit.
- **`.agents/cache/` is gitignored on purpose.** It is a rendering of
  committed state, never a source of it. Never commit it, and if it
  disagrees with the docs, the docs win.
- **Closing an item with its own title as evidence proves nothing.**
  "Implemented the event log" is a restatement. "cargo test -p
  todo-store: 14 passed; golden round-trip byte-identical" is evidence.
- **`standing` is not a way to close a goal.** The tool demands a reason
  and `check` still reports it every run.
- **Two tasks must never write the same file.** That is a decomposition
  rule, and the tracker is where it becomes visible.

## References

- `.agents/skills/todo-next-imp/SKILL.md` — the procedure and the exit gate
- `.agents/roles/orchestrator.md` — when to open a tracker, how to size work
- `docs/implementation/TODO.md` — the board itself, with its rules at the top
- `.agents/config/agent-rules.yaml` → `session.tasks` — the declared config
