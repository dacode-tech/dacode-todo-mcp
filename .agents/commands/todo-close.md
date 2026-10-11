Close out the current unit of work. Run the exit gate, report honestly
what is missing, and do not claim delivery over a failing check.

**Load the `todo-next-imp` skill now** and follow its "exit gate" section.
The live state is injected below.

## Store state (the tracker of record)

!`todo-mcp --root ~/.local/share/dacode-todo progress --json 2>&1 | head -c 6000`

## Working tree state

!`echo "=== repo ==="; git status --short 2>&1 | head -30; echo "=== branch ==="; git symbolic-ref --short HEAD 2>/dev/null || echo "<no commits yet>"; echo "=== staged ==="; git diff --cached --stat 2>&1 | tail -5; echo "=== last commits ==="; git log --format='%h %s' -5 2>&1`

## Harness self-checks

!`echo '--- links ---'; tools/check-links.py --quiet 2>&1 | tail -3; echo '--- adapters ---'; tools/agents-sync.py --check 2>&1 | tail -3; echo '--- doc index ---'; tools/agents-index.py --check 2>&1 | tail -3`

## Code gates, if a workspace exists

!`if [ -f Cargo.toml ]; then echo '--- fmt ---'; cargo fmt --check 2>&1 | tail -5; echo '--- clippy ---'; cargo clippy --workspace --all-targets 2>&1 | tail -c 3000; echo '--- doc ---'; cargo doc --workspace --no-deps 2>&1 | tail -3; echo '--- doc (private items) ---'; cargo doc --workspace --no-deps --document-private-items 2>&1 | tail -3; else echo '<no Cargo.toml — pre-M0; code gates are skipped by construction>'; fi`

$ARGUMENTS

## What to check, in this order

1. **Store nodes closed with real evidence.** Every unit of work this
   session finishes is `task_update` → `status: closed` with `evidence`
   naming the command that was run and what it printed. "Should pass"
   is not evidence — and the domain refuses the close without it. A
   node whose `verify` was never set cannot close honestly.
2. **The sweep ran.** `status_summary` + `next_task` read against what
   this session actually delivered — the store cannot see `git log`,
   so the delivery knowledge is the cross-check. Every open node you
   touched is dispositioned: closed with evidence, blocked with a
   note, or left open with its consumer named. Never report success
   over an undispositioned node.
3. **The epic's handover moved.** `docs/implementation/handover/
   E<n>.md` gains this session's entry in the handover shape. New
   findings and debt became store nodes (`kind: finding` / `kind:
   debt`), not prose.
4. **A signed-off decision executed its `On sign-off:` line.** Strike the
   row in `decisions.md`, set its `Status`, record the outcome in the
   detail entry, and do the work order — most of which say "delete §12
   item N", so the spec moves in the same change.
5. **A fact established the hard way landed in the spec** — a storage
   benchmark number, a crate-API constraint, a format decision derived
   from measurement — in its numbered section, not only in a commit
   message. That is the expensive output of the design phase and a
   commit message is not where it survives.
6. **Code gates, if and only if code was written**: `cargo fmt --check` →
   `cargo check --workspace --all-targets` → `cargo test -p <touched
   crate>` → `cargo clippy --workspace --all-targets` → `cargo doc
   --workspace --no-deps`, run twice — plain and with
   `--document-private-items`. Report each as
   pass / fail / **skipped**. Pre-M0, every code gate is `skipped` with
   that reason.
7. **Commit message audit** on anything new in the log:
   - subject ≤72 chars, imperative, no trailing period, no "This patch"
   - code commits: both `Reviewed-by:` role trailers present **and** both
     reviews actually happened on this exact diff
   - review-exempt commits (docs, `.agents/`, generated): no
     trailer
   - **no** mention of an LLM, an AI assistant, a model, or that anything
     was generated or assisted — anywhere, including comments added in
     the diff
8. **Nothing secret, generated or cached was committed.** `target/`,
   `tmp/`, `.agents/cache/`, `.opencode/node_modules/` stay out; golden
   fixture diffs are called out in the message.
9. **A handover exists**, in the skill's shape — workspace state, what
   landed, the one next step and why it beats the alternative, what is
   blocked on the user, and what is asserted but unverified. It goes in
   chat *and* appended to the epic's handover file, so a cold session
   on another machine starts where this one stopped.

## Output

A short table of gate → result, then the specific list of what is still
missing before this counts as delivered. If everything passes and code
was written, say that both reviewers have or have not signed off, and
propose the commit — do not commit unreviewed code. End with the
handover.

Report `skipped` for anything you could not run, with the reason. Silence
must never imply a check passed.
