#!/usr/bin/env python3
"""Persistent task tree for multi-step work.

Harness todo tools are session-scoped, flat, and vanish when the session
ends. Long-running work needs the opposite: a tree that survives across
sessions, records how each item will be *verified*, and is legible to a
human without opening a chat log.

State lives at `.agents/cache/tasks/<branch>.yaml` — gitignored, so it
can never be committed, but sitting in the worktree where it can be
opened in an editor. Keyed by branch, so switching branches switches
trackers.

    tools/tasks.py show                    render the tree
    tools/tasks.py add  <parent|-> <kind> "title" [--verify ...]
    tools/tasks.py set  <id> --status done --evidence "..."
    tools/tasks.py check                   what blocks a clean sign-off
    tools/tasks.py budget [--model ID]     context window and work budget

Node kinds:
  goal     what the user actually asked for; the thing to check against
  task     a unit of work
  gate     a checkpoint that must pass (build, tests, review)
  finding  something discovered that was not the point of the task
  debt     a deliberate shortcut, recorded rather than forgotten

`--status standing` marks a goal that is *meant* to stay open — a multi-week
programme whose individual deliveries close underneath it. `check` reports it
rather than failing on it, and its `verify` field holds the per-delivery
condition instead of a completion test. Only goals may be standing: applied to a
task it would be a way to dodge the gate, so both `set` and `check` refuse it.

Without it a long-lived goal makes `check` permanently red, and a gate that is
always red gets ignored — which is the failure the mandatory-checker rule exists
to prevent. The same defect is tracked against the project in docs/implementation/TODO.md.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import date
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("PyYAML required: pip install pyyaml")

REPO = Path(__file__).resolve().parent.parent
TASKS_DIR = REPO / ".agents/cache/tasks"
CONFIG = REPO / ".agents/config/agent-rules.yaml"

KINDS = ("goal", "task", "gate", "finding", "debt")
STATUSES = ("pending", "active", "blocked", "done", "wontfix", "standing")
MARK = {"pending": "· ", "active": "▶ ", "blocked": "✗ ", "done": "✓ ", "wontfix": "~ ",
        "standing": "∞ "}


def branch() -> str:
    out = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=REPO, capture_output=True, text=True
    )
    return (out.stdout.strip() or "detached").replace("/", "-")


def state_path() -> Path:
    return TASKS_DIR / f"{branch()}.yaml"


def load() -> dict:
    p = state_path()
    if not p.exists():
        return {"branch": branch(), "created": str(date.today()), "nodes": []}
    return yaml.safe_load(p.read_text()) or {"branch": branch(), "nodes": []}


def save(doc: dict) -> None:
    TASKS_DIR.mkdir(parents=True, exist_ok=True)
    state_path().write_text(yaml.safe_dump(doc, sort_keys=False, allow_unicode=True, width=100))


def walk(nodes: list, depth: int = 0):
    for n in nodes:
        yield n, depth
        yield from walk(n.get("children") or [], depth + 1)


def find(doc: dict, node_id: str) -> dict | None:
    for n, _ in walk(doc["nodes"]):
        if n["id"] == node_id:
            return n
    return None


def next_id(doc: dict, kind: str) -> str:
    prefix = kind[0]
    used = {n["id"] for n, _ in walk(doc["nodes"]) if n["id"].startswith(prefix)}
    i = 1
    while f"{prefix}{i}" in used:
        i += 1
    return f"{prefix}{i}"


# ── commands ──────────────────────────────────────────────────────────


def cmd_show(args) -> int:
    doc = load()
    if not doc["nodes"]:
        print(f"no tasks for branch {branch()} ({state_path().relative_to(REPO)})")
        return 0
    print(f"{doc.get('branch')}  ({state_path().relative_to(REPO)})\n")
    for n, d in walk(doc["nodes"]):
        pad = "  " * d
        print(f"{pad}{MARK.get(n.get('status'), '? ')}{n['id']:<4} [{n['kind']}] {n['title']}")
        if args.verbose:
            for k in ("verify", "evidence", "note"):
                if n.get(k):
                    print(f"{pad}      {k}: {n[k]}")
    return 0


def cmd_add(args) -> int:
    doc = load()
    if args.kind not in KINDS:
        sys.exit(f"kind must be one of {KINDS}")
    node = {
        "id": next_id(doc, args.kind),
        "kind": args.kind,
        "title": args.title,
        "status": "pending",
        "verify": args.verify or "TBD",
    }
    if args.parent in ("-", "", None):
        doc["nodes"].append(node)
    else:
        parent = find(doc, args.parent)
        if parent is None:
            sys.exit(f"no such node: {args.parent}")
        parent.setdefault("children", []).append(node)
    save(doc)
    print(f"{node['id']}  [{node['kind']}] {node['title']}")
    if node["verify"] == "TBD":
        print("  verify: TBD — set it before this can be marked done")
    return 0


def cmd_set(args) -> int:
    doc = load()
    node = find(doc, args.id)
    if node is None:
        sys.exit(f"no such node: {args.id}")
    if args.status:
        if args.status not in STATUSES:
            sys.exit(f"status must be one of {STATUSES}")
        if args.status == "standing":
            if node["kind"] != "goal":
                sys.exit("standing is for goals only — on a "
                         f"{node['kind']} it would just dodge the check gate")
            # Marking a goal standing must not be cheaper than marking it done.
            # `done` demands evidence and a decided verify; without a reason,
            # `standing` would demand nothing and still turn the gate green.
            if not (args.note or node.get("note")):
                sys.exit("standing needs --note saying why this goal cannot close yet")
        node["status"] = args.status
    for k in ("verify", "evidence", "note", "title"):
        v = getattr(args, k, None)
        if v:
            node[k] = v
    save(doc)
    print(f"{node['id']}  {node['status']}  {node['title']}")
    return 0


def cmd_check(args) -> int:
    """What stands between here and a defensible sign-off."""
    doc = load()
    problems: list[str] = []
    nodes = [n for n, _ in walk(doc["nodes"])]
    for n in nodes:
        nid, kind, st = n["id"], n["kind"], n.get("status")
        # cmd_set refuses this, but the tracker is a file in the worktree and can
        # be edited by hand, so check is the backstop.
        if st == "standing" and kind != "goal":
            problems.append(f"{nid} is standing but is a {kind}, not a goal")
        if st == "done" and kind in ("goal", "task", "gate"):
            if not n.get("evidence"):
                problems.append(f"{nid} done with no evidence recorded")
            elif str(n.get("verify", "TBD")).strip() in ("TBD", "", "None"):
                problems.append(f"{nid} done but verify was never decided")
        if st in ("pending", "active", "standing") and str(n.get("verify", "TBD")).strip() == "TBD":
            problems.append(f"{nid} has no verification method — decide it, or ask the user")
        if st == "blocked" and not n.get("note"):
            problems.append(f"{nid} is blocked with no reason recorded")
        # Same rule as blocked, for the same reason: an unexplained exemption from
        # the gate is indistinguishable from a dodged one.
        if st == "standing" and not n.get("note"):
            problems.append(f"{nid} is standing with no reason recorded")

    open_goals = [n for n in nodes if n["kind"] == "goal"
                  and n.get("status") not in ("done", "wontfix", "standing")]
    # Goals only: a hand-edited standing *task* is already a BLOCK above, and
    # printing it here too would label a task a goal.
    standing = [n for n in nodes if n["kind"] == "goal" and n.get("status") == "standing"]
    open_debt = [n for n in nodes if n["kind"] == "debt" and n.get("status") == "pending"]

    for p in problems:
        print(f"  BLOCK  {p}")
    for n in open_goals:
        print(f"  OPEN   {n['id']} goal not closed: {n['title']}")
    # Reported, not suppressed: a standing goal is still live, and dropping it
    # from the output is how a long-running programme stops being looked at.
    for n in standing:
        print(f"  STAND  {n['id']} standing goal, does not close: {n['title']}")
    for n in open_debt:
        print(f"  DEBT   {n['id']} {n['title']}")

    if not problems and not open_goals:
        print("clean: every goal closed or declared standing, every done item has evidence")
        if standing:
            print(f"({len(standing)} standing goal(s) still live — reported, not blocking)")
        if open_debt:
            print(f"({len(open_debt)} debt item(s) recorded — not blocking)")
        return 0
    return 1


def model_limits(model: str | None) -> tuple[str, int, str]:
    """(model, context_window, source). opencode already caches this."""
    cache = Path.home() / ".cache/opencode/models.json"
    default = "claude-opus-5"  # harness.aliases.heavy in agent-rules.yaml
    if cache.exists():
        data = json.loads(cache.read_text())
        for prov in data.values():
            for mid, m in (prov.get("models") or {}).items():
                if model in (None, mid) and (m.get("limit") or {}).get("context"):
                    if model is None and mid != default:
                        continue
                    return mid, m["limit"]["context"], "opencode models.json"
    cfg = yaml.safe_load(CONFIG.read_text())
    fb = (cfg.get("session", {}).get("context_budget") or {}).get("fallback_context")
    if fb:
        return model or "(unknown)", int(fb), "agent-rules.yaml fallback"
    sys.exit("could not determine context window; set session.context_budget.fallback_context")


def cmd_budget(args) -> int:
    cfg = yaml.safe_load(CONFIG.read_text())
    cb = (cfg.get("session", {}) or {}).get("context_budget") or {}
    if not cb.get("enabled", False):
        print("context budgeting is disabled (session.context_budget.enabled: false)")
        return 0
    mid, ctx, src = model_limits(args.model)
    work = int(ctx * float(cb.get("work_fraction", 0.5)))
    search = int(ctx * float(cb.get("search_fraction", 0.15)))
    print(f"model            {mid}")
    print(f"context window   {ctx:,}   (source: {src})")
    print(f"work budget      {work:,}   ({cb.get('work_fraction')} of window)")
    print(f"search sub-budget{search:>9,}   ({cb.get('search_fraction')} of window)")
    print("\nEstimates only. Nothing measures tokens before dispatch; these are")
    print("ceilings for the orchestrator to size work against, not enforcement.")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("show"); s.add_argument("-v", "--verbose", action="store_true")
    s.set_defaults(func=cmd_show)

    a = sub.add_parser("add")
    a.add_argument("parent"); a.add_argument("kind"); a.add_argument("title")
    a.add_argument("--verify")
    a.set_defaults(func=cmd_add)

    t = sub.add_parser("set")
    t.add_argument("id")
    for f in ("status", "verify", "evidence", "note", "title"):
        t.add_argument(f"--{f}")
    t.set_defaults(func=cmd_set)

    sub.add_parser("check").set_defaults(func=cmd_check)

    b = sub.add_parser("budget"); b.add_argument("--model")
    b.set_defaults(func=cmd_budget)

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
