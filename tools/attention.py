#!/usr/bin/env python3
"""What needs the user's attention, prioritized.

The design phase spreads "things only you can decide" across four artifacts:
the spec's §12 open-decision list, docs/implementation/decisions.md,
docs/implementation/TODO.md, and the gitignored tasks.py tracker. No single
view of them exists, and that is how a decision gets recorded correctly in
three places and still stays invisible to the one person who has to act on it.

This script is that view. It reads the sources of truth, derives a priority
from *which milestone the item gates*, and writes one table.

    tools/attention.py            print the table and refresh the cache file
    tools/attention.py --json     machine-readable, for other tools
    tools/attention.py --check    exit 1 if anything is P0 or hygiene fails

Output is written to `.agents/cache/ATTENTION.md`, which is gitignored — it is
a rendering of committed state, never a source of it. If it disagrees with the
docs, the docs win and this script is wrong.

Priority is derived, not assigned by hand:
    P0  gates M0 or M1 — the milestones that are unblocked or next
    P1  gates M2 or M3
    P2  gates M4 or later
    P3  gates no milestone: hygiene, debt, follow-ups

Action says what is actually being asked, because "sign off" is not one thing:
most items are a vote, some need a measurement rather than an opinion, and some
need a sitting-down session that sign-off alone does not release.

`--max-priority` filters what is *reported*; it never changes what `--check`
verdicts on, and the cache file always receives the complete table. A filtered
view that silently became the stored state would read as complete next time.

Hygiene is the other half of the job, and its first rule is that the tool must
not be able to pass by reading nothing: an unreadable or empty source is itself
a failure, because every check over an empty parse passes vacuously and prints
"all checks pass" over data that was never there.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SPEC = REPO / "docs" / "todo-mcp-design.md"
DECISIONS = REPO / "docs" / "implementation" / "decisions.md"
TODO = REPO / "docs" / "implementation" / "TODO.md"
TASKS_DIR = REPO / ".agents" / "cache" / "tasks"
OUT = REPO / ".agents" / "cache" / "ATTENTION.md"
CHECK_LINKS = REPO / "tools" / "check-links.py"

# Milestone -> priority. An item gating M0 is more urgent than one gating M5,
# which is the whole basis of the scheme. Declare what an item gates and the
# priority follows; never write a priority into a doc.
MILESTONE_PRIORITY = {"M0": 0, "M1": 0, "M2": 1, "M3": 1, "M4": 2, "M5": 2}
VALID_MILESTONES = set(MILESTONE_PRIORITY)

# Sort order within a priority band: a session needs the user's calendar, a
# measurement needs their hardware, a vote needs only their opinion.
ACTION_RANK = {"session": 0, "measure": 1, "vote": 2, "review": 3, "ack": 4}
ACTION_LEGEND = {
    "vote": "accept, reject or redirect the recommendation",
    "measure": "needs a number, not an opinion — do not vote on it",
    "session": "a design session with you; sign-off alone does not release it",
    "review": "an unowned follow-up needing a decision or an owner",
    "ack": "recorded debt; acknowledging it is the action",
}

# tasks.py's own vocabulary. `wontfix` is its closed-and-rejected state (there
# is no `cancelled`), and `standing` is deliberately open, so neither may be
# skipped as finished. Kept as a literal rather than imported: tasks.py exits
# the process when PyYAML is missing, which this tool must not do.
TRACKER_CLOSED = ("done", "wontfix")

SPEC12_HEADING = re.compile(r"^##\s+12\.\s+Open Decisions", re.M)
# Group 3 matters: §12 items put the title in bold and the substance *after* it
# on the same line ("**OIDC flow & token transport** — authorization-code +
# PKCE? …"). Stopping at the closing `**` throws the substance away.
SPEC_ITEM = re.compile(r"^(\d+)\.\s+\*\*(.+?)\*\*\s*(.*)$")
SECTION_HEADING = re.compile(r"^#{2,4}\s+(\d+(?:\.\d+)*)(?:\.|\s)", re.M)
SECTION_REF = re.compile(r"§(\d+(?:\.\d+)*)")
FENCE = re.compile(r"^\s*```")

# `| **D1** | ... |` — struck ID (`| ~~**D1**~~ |`) means resolved. The ID must
# stay inside the bold run: checking an item off must not change a value this
# script reads.
DECISION_ROW = re.compile(r"^\|\s*(~~)?\*\*(D\d+)\*\*(~~)?\s*\|")
DECISION_DETAIL = re.compile(r"^###\s+(D\d+)\s+—")
ANY_HEADING = re.compile(r"^#{2,3}\s")
META_BLOCK = re.compile(r"<!--\s*attention-meta(.*?)-->", re.S)
META_ACTION = re.compile(r"^action:\s*(.+)$", re.M)
# Values may be bold-wrapped (`**Action** **measure — …**`), which a plain
# `([a-z]+)` misses; `\*{0,2}` on both sides covers either form.
DETAIL_GATES = re.compile(r"\*\*Gates\*\*\s*\*{0,2}([A-Za-z0-9]+|—)")
DETAIL_ACTION = re.compile(r"\*\*Action\*\*\s*\*{0,2}([a-z]+)")
DETAIL_CONFIDENCE = re.compile(r"\*\*Confidence:?\*{0,2}\s*\*{0,2}([a-z]+)")
# What a signed-off decision owes the spec. Most "On sign-off:" lines say to
# delete their §12 item; some say to narrow it instead, so this is per-entry
# evidence rather than a blanket rule.
DETAIL_DELETES = re.compile(r"delete\s+§12\s+item\s+(\d+)", re.I)
TODO_ITEM = re.compile(r"^-\s+\[( |x)\]\s+(.*)$")
TODO_UID = re.compile(r"^\*\*U(\d+)\s*·\s*(.*)$", re.S)
TODO_SECTION = re.compile(r"^##\s+(\d+)\.\s+(.*)$")

# Only these TODO.md sections carry action items for the user. §4 is unowned
# follow-ups (U-ids) and §5 is debt. §1 is a summary of the decisions already
# listed, and §3 is narrative context — rendering those as items duplicates
# every D-entry and buries the asks that are actually new.
TODO_ACTION_SECTIONS = {"4", "5"}


def read(path: Path) -> str | None:
    """File text, or None if it could not be read.

    None is deliberately not "": a missing or undecodable source must fail
    loudly. Handed an empty string, every parser below returns nothing and every
    check over it passes, so the tool would report a clean bill of health over
    files it never read.
    """
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None


def strip_code_fences(text: str) -> tuple[str, bool]:
    """Blank out fenced blocks, and report whether the fences balanced.

    The spec quotes shell invocations and message layouts; a `§` inside one is
    an example, not a cross-reference. An unbalanced fence is reported because
    it would blank the rest of the file and make every check over it vacuous.
    """
    out, fence = [], False
    for line in text.split("\n"):
        if FENCE.match(line):
            fence = not fence
            out.append("")
            continue
        out.append("" if fence else line)
    return "\n".join(out), fence


def inline_code_spans(line: str) -> str:
    """Blank out `backtick spans` — prose *about* a reference, not one."""
    return re.sub(r"`[^`]*`", "", line)


def lines_between(text: str, start_pat: re.Pattern, end_pred) -> list[str]:
    """Lines from the first match of start_pat until end_pred(line) is true."""
    lines = text.split("\n")
    try:
        start = next(i for i, l in enumerate(lines) if start_pat.search(l))
    except StopIteration:
        return []
    out = []
    for line in lines[start + 1:]:
        if end_pred(line):
            break
        out.append(line)
    return out


def cells(line: str) -> list[str]:
    """Split a markdown table row. Does not unescape `\\|` inside a cell —
    no current row contains one; if one ever does, this splits it wrong and
    the column-count guard in decision_rows drops the row loudly enough to see."""
    return [c.strip() for c in line.strip().strip("|").split("|")]


# --------------------------------------------------------------------------
# source 1 — the spec's §12 open-decision list
# --------------------------------------------------------------------------

def spec_open_decisions(text: str) -> list[tuple[int, str]]:
    body = lines_between(
        text, SPEC12_HEADING, lambda l: l.startswith("---") or l.startswith("## ")
    )
    items: list[list] = []
    for line in body:
        m = SPEC_ITEM.match(line)
        if m:
            title, tail = m.group(2).strip(), m.group(3).strip()
            # The tail already opens with its own "—", so do not add one.
            items.append([int(m.group(1)), f"{title} {tail}".strip()])
        elif items and line.startswith("   ") and line.strip():
            # Continuation lines accumulate. Rebinding to a fresh tuple instead
            # of appending would keep only the last one, silently truncating
            # every multi-line item — which is most of them.
            items[-1][1] = f"{items[-1][1]} {line.strip()}"
    return [(n, t) for n, t in items]


def spec_sections(text: str) -> set[str]:
    return set(SECTION_HEADING.findall(text))


def own_sections(text: str) -> set[str]:
    """A doc's own top-level section numbers, so its internal `§4` references
    are not reported as dangling spec references."""
    return {n.split(".")[0] for n in SECTION_HEADING.findall(text)}


# --------------------------------------------------------------------------
# source 2 — decisions.md: sign-off table, detail entries, meta block
# --------------------------------------------------------------------------

def decision_rows(text: str) -> list[dict]:
    body = lines_between(
        text,
        re.compile(r"^##\s+Sign-off summary", re.M),
        lambda l: l.startswith("---") or (l.startswith("## ") and "Sign-off" not in l),
    )
    rows = []
    for line in body:
        m = DECISION_ROW.match(line)
        if not m:
            continue
        c = cells(line)
        if len(c) < 6:
            # A row this script cannot parse must not vanish quietly: it would
            # take its decision out of every count and every check below.
            print(f"warning: decisions.md row has {len(c)} cells, expected 6: "
                  f"{line[:70]}", file=sys.stderr)
            continue
        rows.append({
            "id": m.group(2),
            "struck": bool(m.group(1) or m.group(3)),
            "decision": c[1],
            "recommendation": c[2],
            "confidence": c[3],
            "gates": c[4],
            "status": c[5],
        })
    return rows


def decision_meta(text: str) -> dict[str, str]:
    """Declared action types. Declared, not scraped from prose: a sentence
    about a decision is not a property of it."""
    actions: dict[str, str] = {}
    for block in META_BLOCK.findall(text):
        for line in META_ACTION.findall(block):
            for tok in line.split():
                if "=" in tok:
                    k, v = tok.split("=", 1)
                    actions[k.strip()] = v.strip().lower()
    return actions


def decision_details(text: str) -> dict[str, dict]:
    """Per-entry Gates/Action/Confidence, to cross-check against the table.

    Scans to the next heading rather than a fixed line window: the confidence
    statement commonly sits twenty-plus lines below the entry heading, after the
    options and the argument, so a short window finds nothing and the
    cross-check silently stops checking.
    """
    out: dict[str, dict] = {}
    lines = text.split("\n")
    for i, line in enumerate(lines):
        m = DECISION_DETAIL.match(line)
        if not m:
            continue
        detail = {"gates": None, "action": None, "confidence": None,
                  "deletes_item": None}
        for follow in lines[i + 1:]:
            if ANY_HEADING.match(follow):
                break
            if detail["gates"] is None:
                g = DETAIL_GATES.search(follow)
                if g:
                    detail["gates"] = g.group(1).strip()
            if detail["action"] is None:
                a = DETAIL_ACTION.search(follow)
                if a:
                    detail["action"] = a.group(1)
            if detail["confidence"] is None:
                c = DETAIL_CONFIDENCE.search(follow)
                if c:
                    detail["confidence"] = c.group(1)
            if detail["deletes_item"] is None:
                d = DETAIL_DELETES.search(follow)
                if d:
                    detail["deletes_item"] = int(d.group(1))
        out[m.group(1)] = detail
    return out


# --------------------------------------------------------------------------
# source 3 — TODO.md: U-ids and debt
# --------------------------------------------------------------------------

def todo_items(text: str) -> list[dict]:
    items, section, title = [], "0", "TODO.md"
    for lineno, line in enumerate(text.split("\n"), start=1):
        sm = TODO_SECTION.match(line)
        if sm:
            section, title = sm.group(1), sm.group(2).strip()
            continue
        if section not in TODO_ACTION_SECTIONS:
            continue
        m = TODO_ITEM.match(line)
        if not m:
            continue
        body = m.group(2)
        um = TODO_UID.match(body)
        if um:
            # The ID column already carries the U-number; keeping it in the
            # text too renders as "U1 · U1 · …".
            uid, rest = f"U{um.group(1)}", um.group(2)
        else:
            uid, rest = None, body
        items.append({
            "checked": m.group(1).lower() == "x",
            "section": section,
            "section_title": title,
            "uid": uid,
            "line": lineno,
            "text": re.sub(r"\*\*", "", rest).strip(),
            # §5 is the debt section; its items are acknowledgements, not work.
            "kind": "debt" if section == "5" else "followup",
        })
    return items


# --------------------------------------------------------------------------
# source 4 — the tasks.py tracker (gitignored, session-scoped)
# --------------------------------------------------------------------------

def tracker_branch() -> str:
    """Mirror tools/tasks.py branch() exactly. Sanitizing differently, or
    falling back to a different name, means looking for a file tasks.py never
    wrote — which drops this whole source with no warning."""
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=REPO, capture_output=True, text=True,
        )
    except OSError:
        return "detached"
    return (out.stdout.strip() or "detached").replace("/", "-")


def tracker_items(path: Path | None) -> list[dict]:
    if path is None:
        return []
    try:
        # PyYAML is optional *here* but mandatory in tasks.py: the tracker is
        # gitignored session state, so this tool degrades to the committed docs
        # rather than exiting the process the way tasks.py does.
        import yaml
    except ImportError:
        print("note: PyYAML unavailable; skipping the tasks.py tracker source",
              file=sys.stderr)
        return []
    try:
        doc = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except Exception as e:
        print(f"warning: could not parse {path.name}: {e}", file=sys.stderr)
        return []
    if not isinstance(doc, dict):
        print(f"warning: {path.name} is not a mapping; skipping", file=sys.stderr)
        return []

    out: list[dict] = []

    def walk(nodes):
        if not isinstance(nodes, list):
            return
        for n in nodes:
            if not isinstance(n, dict):
                # A scalar where a node belongs means the tracker was edited by
                # hand or written by a different version; say so rather than
                # dropping it, since the item then simply stops being reported.
                print(f"warning: {path.name} has a non-mapping node: {n!r:.60}",
                      file=sys.stderr)
                continue
            if str(n.get("status", "")) in TRACKER_CLOSED:
                walk(n.get("children"))
                continue
            text = f"{n.get('title', '')} {n.get('verify', '')}".lower()
            if n.get("kind") == "debt":
                out.append({
                    # Debt gates no milestone, so it sorts last; acknowledging
                    # it is the whole ask.
                    "priority": 3,
                    "id": f"tasks:{n.get('id') or '?'}", "action": "ack",
                    "what": str(n.get("title", ""))[:110],
                    "where": f"tasks.py {n.get('id') or '?'}",
                    "why": "recorded debt in the session tracker",
                })
            elif n.get("kind") == "gate" and "user" in text:
                out.append({
                    # A gate needing the user cannot have its milestone derived
                    # from anything, so it goes above the P3 noise rather than
                    # being buried in it.
                    "priority": 1,
                    "id": f"tasks:{n.get('id') or '?'}", "action": "vote",
                    "what": str(n.get("title", ""))[:110],
                    "where": f"tasks.py {n.get('id') or '?'}",
                    "why": "a gate whose verification needs the user",
                })
            walk(n.get("children"))

    walk(doc.get("nodes"))
    return out


def tracker_path() -> Path | None:
    p = TASKS_DIR / f"{tracker_branch()}.yaml"
    return p if p.exists() else None


# --------------------------------------------------------------------------
# hygiene
# --------------------------------------------------------------------------

def docs_to_scan() -> list[Path]:
    paths = [SPEC, DECISIONS, TODO,
             REPO / "docs" / "implementation" / "README.md", REPO / "AGENTS.md"]
    agents = REPO / ".agents"
    if agents.is_dir():
        paths += [p for p in agents.rglob("*.md")
                  if "cache" not in p.relative_to(REPO).parts]
    return [p for p in paths if p.exists()]


def check_hygiene(sources: dict[str, str | None], rows: list[dict],
                  details: dict, actions: dict, spec12: list[tuple[int, str]],
                  todos: list[dict]) -> list[tuple[bool, str]]:
    """(ok, description) pairs. A False fails --check."""
    out: list[tuple[bool, str]] = []
    spec_text = sources["spec"] or ""

    # 0. The sources exist and parsed into something. Without this, a truncated
    #    or missing file makes every check below pass over empty input and the
    #    tool prints a clean bill of health having read nothing — the one
    #    failure this script exists to prevent.
    open_rows = [r for r in rows
                 if not r["status"].lower().startswith(("resolved", "applied"))]
    for name, path in (("spec", SPEC), ("decisions.md", DECISIONS), ("TODO.md", TODO)):
        out.append((sources[name] is not None, f"{name} is readable ({path.name})"))
    # An empty parse must never read as "nothing is open" — but the design
    # phase's correct END state is an emptied §12 with every row resolved, so
    # the guard is "parsed something, or there is genuinely nothing left open".
    out.append((len(spec12) > 0 or not open_rows,
                f"spec §12 parsed {len(spec12)} open decisions"))
    out.append((len(rows) > 0, f"decisions.md parsed {len(rows)} sign-off rows"))
    # TODO.md has no table to count, so assert its structure instead. Items are
    # struck, never deleted, so a healthy board always parses some — checked or
    # not — and zero means the file was truncated or emptied by accident.
    todo_headings = {m.group(1) for m in
                     re.finditer(r"^##\s+(\d+)\.", sources["TODO.md"] or "", re.M)}
    out.append(({"4", "5"} <= todo_headings,
                f"TODO.md has its §4 and §5 action sections (found: "
                f"{'§' + ', §'.join(sorted(todo_headings)) if todo_headings else 'none'})"))
    out.append((len(todos) > 0,
                f"TODO.md parsed {len(todos)} action items from §4/§5"))

    # 1. §12 <-> decisions.md, both directions. This is the check the tool
    #    exists for: an open decision in the spec with no D-entry is invisible
    #    to a sign-off sweep of decisions.md, and a D-entry with no spec item
    #    is a decision nobody can find by reading the spec. The D-numbering
    #    mirrors §12's item numbering by construction.
    d_ids = {r["id"] for r in rows}
    spec_nums = {n for n, _ in spec12}
    for num, _title in spec12:
        out.append((f"D{num}" in d_ids,
                    f"§12 item {num} has a decisions.md entry (D{num})"))
    for r in rows:
        n = int(r["id"][1:])
        resolved = r["status"].lower().startswith(("resolved", "applied"))
        # Membership, not a range test: a resolved entry's §12 item is DELETED
        # on sign-off, so the numbers stop being contiguous and an
        # `n <= len(spec12)` test would fail forever from the first resolution
        # onward. The exemption is safe only because the deletion itself is
        # checked below, per entry, against that entry's own work order.
        out.append((resolved or n in spec_nums,
                    f"{r['id']} has a matching §12 item"
                    + (" (resolved; its §12 item is retired)" if resolved else "")))
        # A resolution that struck the row but left the spec item in place is
        # the silent half-completed sign-off: the board says decided, the spec
        # still says open. Not every work order deletes its item — D2's narrows
        # it — so this is driven by what the entry actually says to do.
        owed = (details.get(r["id"]) or {}).get("deletes_item")
        if resolved and owed is not None:
            out.append((owed not in spec_nums,
                        f"{r['id']} is resolved and §12 item {owed} was deleted from the spec"
                        + ("" if owed not in spec_nums
                           else " — STILL PRESENT, its 'On sign-off:' order was not executed")))

    # 2. Strikethrough must agree with the Status column. The table is
    #    authoritative; the strike is for humans. They drift, and when they do
    #    a reviewer cannot tell which to believe.
    for r in rows:
        resolved = r["status"].lower().startswith(("resolved", "applied"))
        out.append((r["struck"] == resolved,
                    f"{r['id']} strikethrough matches Status='{r['status']}'"))

    # 3. Declared action ids must exist and be known types. A typo here silently
    #    downgrades an item to a plain vote — which for a `measure` item means
    #    the user is asked for an opinion that cannot answer the question.
    for did, act in actions.items():
        out.append((did in d_ids, f"declared action for {did} matches a real entry"))
        out.append((act in ACTION_RANK, f"declared action '{act}' for {did} is a known type"))

    # 4. Gates must be a real milestone, or explicitly none.
    for r in rows:
        out.append((r["gates"] in VALID_MILESTONES or r["gates"] in ("—", "-", "none", ""),
                    f"{r['id']} Gates='{r['gates']}' is a known milestone"))

    # 5/6. Every row must have a detail entry, and where it declares Gates or
    #      Action they must agree with the table and the meta block. A row with
    #      no detail entry is a decision with a one-line summary and no
    #      reasoning — the thing this dossier exists to prevent — and skipping
    #      it quietly would let that state persist unnoticed.
    for r in rows:
        d = details.get(r["id"])
        out.append((d is not None,
                    f"{r['id']} has a `### {r['id']} —` detail entry"))
        if not d:
            continue
        if d["gates"]:
            out.append((d["gates"] == r["gates"],
                        f"{r['id']} detail Gates='{d['gates']}' matches table '{r['gates']}'"))
        if d["action"] and r["id"] in actions:
            out.append((d["action"] == actions[r["id"]],
                        f"{r['id']} detail Action='{d['action']}' matches declared "
                        f"'{actions[r['id']]}'"))
        if d["confidence"]:
            out.append((d["confidence"] in ("low", "medium", "high"),
                        f"{r['id']} detail states a low/medium/high confidence"))

    # 7. Every open decision needs a recommendation and a confidence in the
    #    table. An entry without them is a description of a problem, not a
    #    decision to sign.
    for r in rows:
        if r["status"].lower().startswith(("resolved", "applied")):
            continue
        out.append((bool(r["recommendation"]) and r["recommendation"] not in ("—", "-"),
                    f"{r['id']} carries a recommendation"))
        out.append((r["confidence"].lower() in ("low", "medium", "high"),
                    f"{r['id']} carries a low/medium/high confidence (got '{r['confidence']}')"))

    # 8. Dangling § cross-references. The spec carries no markdown links by
    #    convention, so check-links.py cannot see these at all — a reference to
    #    a section that does not exist is silent, and one did (§13.3). A ref
    #    resolves against the spec's sections or the citing doc's own.
    valid_spec = spec_sections(spec_text)
    for path in docs_to_scan():
        text = read(path)
        if text is None:
            out.append((False, f"{path.relative_to(REPO)} is readable"))
            continue
        cleaned, unbalanced = strip_code_fences(text)
        out.append((not unbalanced,
                    f"{path.relative_to(REPO)}: code fences are balanced"))
        if unbalanced:
            # An unclosed fence blanks the remainder, so every check over it is
            # vacuous. Report that rather than letting it pass.
            continue
        valid = valid_spec | own_sections(text)
        rel = path.relative_to(REPO)
        bad = sorted({
            m.group(1) for line in cleaned.split("\n")
            for m in SECTION_REF.finditer(inline_code_spans(line))
            if m.group(1) not in valid
        })
        out.append((not bad,
                    f"{rel}: every § reference resolves to a spec or same-doc section"
                    + (f" (dangling: {', '.join('§' + b for b in bad)})" if bad else "")))

    # 9. Every open TODO.md action item needs a stable U-id, or it cannot be
    #    cited or closed — it can only be restated, which is how the same
    #    follow-up ends up in the board three times.
    for t in todos:
        if t["checked"]:
            continue
        out.append((t["uid"] is not None,
                    f"TODO.md §{t['section']}:{t['line']} carries a U-id"
                    + ("" if t["uid"] else f" (got: {t['text'][:50]}…)")))

    # 10. The docs' own link checker, which does cover this directory.
    try:
        r = subprocess.run([sys.executable, str(CHECK_LINKS), "--quiet"],
                           cwd=REPO, capture_output=True, text=True, timeout=120)
        detail = (r.stdout or r.stderr).strip()[:200]
        out.append((r.returncode == 0, "tools/check-links.py passes"
                    + ("" if r.returncode == 0 else f": {detail}")))
    except (OSError, subprocess.SubprocessError) as e:
        out.append((False, f"tools/check-links.py could not run: {e}"))

    return out


# --------------------------------------------------------------------------
# assemble
# --------------------------------------------------------------------------

def build() -> tuple[list[dict], list[tuple[bool, str]]]:
    sources = {
        "spec": read(SPEC),
        "decisions.md": read(DECISIONS),
        "TODO.md": read(TODO),
    }
    spec_text = sources["spec"] or ""
    dec_text = sources["decisions.md"] or ""
    todo_text = sources["TODO.md"] or ""

    spec12 = spec_open_decisions(spec_text)
    rows = decision_rows(dec_text)
    details = decision_details(dec_text)
    actions = decision_meta(dec_text)

    items: list[dict] = []
    spec_titles = dict(spec12)

    for r in rows:
        if r["status"].lower().startswith(("resolved", "applied")):
            continue
        num = int(r["id"][1:])
        gates = r["gates"] if r["gates"] in MILESTONE_PRIORITY else None
        pri = MILESTONE_PRIORITY[gates] if gates else 3
        action = actions.get(r["id"]) or (details.get(r["id"]) or {}).get("action") or "vote"
        items.append({
            "priority": pri, "action": action, "id": r["id"],
            "what": r["decision"] or spec_titles.get(num, "")[:80],
            "where": f"decisions.md {r['id']}; spec §12 item {num}",
            "why": (f"gates {gates}" if gates else "gates no milestone")
                   + f" · confidence {r['confidence'] or 'unstated'}",
        })

    # §12 items with no D-entry: the failure this tool exists to catch. Kept as
    # a rendered row *and* a hygiene failure — the row guarantees it is visible
    # in the table, the failure makes --check exit non-zero about it.
    have = {r["id"] for r in rows}
    for num, title in spec12:
        if f"D{num}" in have:
            continue
        items.append({
            "priority": 0, "action": "review", "id": f"§12.{num}",
            "what": title[:90],
            "where": f"spec §12 item {num}",
            "why": "NO D-number — invisible to a decisions.md sign-off sweep",
        })

    todos = todo_items(todo_text)
    for t in todos:
        if t["checked"]:
            continue
        # An item with no U-id is still worth rendering, but it cannot be cited
        # or closed, so point at its line and let hygiene complain about it.
        label = t["uid"] or f"TODO §{t['section']}:{t['line']}"
        items.append({
            # Follow-ups and debt gate no milestone, so both are P3 per the
            # derivation rule; the action column is what separates them.
            "priority": 3,
            "action": "ack" if t["kind"] == "debt" else "review",
            "id": label,
            "what": re.sub(r"\s+", " ", t["text"])[:110],
            "where": f"TODO.md §{t['section']} {t['section_title']}",
            "why": "recorded debt" if t["kind"] == "debt"
                   else "unowned follow-up"
                        + ("" if t["uid"] else " — NO U-id, cannot be cited or closed"),
        })

    items += tracker_items(tracker_path())

    hygiene = check_hygiene(sources, rows, details, actions, spec12, todos)
    # Every source builds its own dicts; a missing key here is a bug in one of
    # them, but it must degrade to a mis-sorted row, not a traceback.
    for i in items:
        i.setdefault("priority", 3)
        i.setdefault("action", "review")
    items.sort(key=sort_key)
    return items, hygiene


def sort_key(i: dict):
    """Priority, then action, then ID — numerically, so D2 sorts before D10
    rather than after it the way a lexicographic sort would."""
    m = re.match(r"^([^0-9]*)(\d+)", i["id"])
    prefix, num = (m.group(1), int(m.group(2))) if m else (i["id"], 0)
    return (i["priority"], ACTION_RANK.get(i["action"], 9), prefix, num, i["id"])


def render_markdown(items: list[dict], hygiene: list[tuple[bool, str]],
                    hidden: int = 0, wrote: bool = True) -> str:
    from collections import Counter
    pri_c = Counter(i["priority"] for i in items)
    act_c = Counter(i["action"] for i in items)
    failed = [d for ok, d in hygiene if not ok]

    L = ["<!-- GENERATED by tools/attention.py — a rendering of committed state,",
         "     never a source of it. If this disagrees with the docs, the docs win. -->",
         "", "# What needs your attention", ""]
    if items:
        L += [f"**{len(items)} item(s)** — "
              + ", ".join(f"{pri_c[p]} at P{p}" for p in sorted(pri_c)) + ".",
              "",
              "By action: "
              + ", ".join(f"{act_c[a]} {a}"
                          for a in sorted(act_c, key=lambda x: ACTION_RANK.get(x, 9))) + ".",
              ""]
    else:
        L += ["**Nothing needs your attention.**", ""]
    if hidden:
        where = "the cache file holds" if wrote else "an unfiltered run shows"
        L += [f"_{hidden} item(s) hidden by --max-priority; {where} the complete "
              f"table._", ""]
    L += ["Priority is derived from which milestone an item gates: P0 gates M0/M1,",
          "P1 gates M2/M3, P2 gates M4+, P3 gates nothing. Action types are declared",
          "in decisions.md's attention-meta block.", ""]

    if items:
        L += ["| Pri | Do | ID | What | Where | Why |", "|---|---|---|---|---|---|"]
        for i in items:
            what = i["what"].replace("|", "\\|")
            L.append(f"| P{i['priority']} | **{i['action']}** | `{i['id']}` | {what} "
                     f"| {i['where']} | {i['why']} |")
        L += ["", "## Action types", ""]
        for a in sorted(act_c, key=lambda x: ACTION_RANK.get(x, 9)):
            L.append(f"- **{a}** — {ACTION_LEGEND.get(a, '(unknown type)')}")
        L.append("")

    L += ["## Hygiene", ""]
    if failed:
        L += [f"**{len(failed)} check(s) FAILED:**", ""] + [f"- ❌ {d}" for d in failed]
    else:
        L.append(f"✅ all {len(hygiene)} checks pass.")
    L.append("")
    return "\n".join(L)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 if any P0 item exists or any hygiene check fails")
    ap.add_argument("--max-priority", type=int, default=3, choices=[0, 1, 2, 3],
                    help="report only items at this priority or higher (default 3 = all). "
                         "Affects --json and stdout only; never the --check verdict, and "
                         "never the cache file, which always gets the complete table")
    ap.add_argument("--no-write", action="store_true", help="do not refresh the cache file")
    args = ap.parse_args()

    items, hygiene = build()
    failed = [d for ok, d in hygiene if not ok]
    p0 = [i for i in items if i["priority"] == 0]
    # The verdict is computed over every item, so a reporting filter cannot
    # hide a P0 from --check.
    verdict_fail = bool(failed or p0)

    shown = [i for i in items if i["priority"] <= args.max_priority]
    hidden = len(items) - len(shown)

    if args.json:
        print(json.dumps({
            "items": shown,
            "hidden_by_max_priority": hidden,
            "hygiene": [{"ok": ok, "check": d} for ok, d in hygiene],
            "counts": {"total": len(items), "shown": len(shown),
                       "p0": len(p0), "failed_checks": len(failed)},
        }, indent=2))
        if args.check:
            for d in failed:
                print(f"FAIL: {d}", file=sys.stderr)
            if p0:
                print(f"FAIL: {len(p0)} P0 item(s) need the user", file=sys.stderr)
        return 1 if (args.check and verdict_fail) else 0

    # The cache file always holds the complete table: it is the shared artifact
    # other reads consult, so writing a filtered view there would later read as
    # though nothing else existed.
    full = render_markdown(items, hygiene)
    if not args.no_write:
        try:
            OUT.parent.mkdir(parents=True, exist_ok=True)
            OUT.write_text(full, encoding="utf-8")
        except OSError as e:
            print(f"warning: could not write {OUT}: {e}", file=sys.stderr)

    print(full if not hidden
          else render_markdown(shown, hygiene, hidden=hidden, wrote=not args.no_write))
    if not args.no_write:
        print(f"(written to {OUT.relative_to(REPO)})", file=sys.stderr)

    if args.check:
        for d in failed:
            print(f"FAIL: {d}", file=sys.stderr)
        if p0:
            print(f"FAIL: {len(p0)} P0 item(s) need the user", file=sys.stderr)
        return 1 if verdict_fail else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
