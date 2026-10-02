#!/usr/bin/env python3
"""Total progress, as a page you can open and a JSON you can query.

`tools/attention.py` answers "what needs the user". This answers the other
question, which no committed view covers: **what is the shape of the work** —
which task fed which, what is open, what is blocked, and how much of the
roadmap is behind us. It reads the same two sources of truth (the gitignored
session tracker and the committed board) and renders them as a dependency
graph plus totals.

    tools/progress.py             print the summary, refresh the cache files
    tools/progress.py --open      ...and open the page in a browser
    tools/progress.py --json      print the JSON to stdout instead of writing it
    tools/progress.py --out DIR   write somewhere other than the cache

Output goes to `.agents/cache/progress.{html,json}`, which is gitignored — a
rendering of committed state, never a source of it. If it disagrees with the
tracker or the board, those win and this script is wrong.

Two rules this tool holds itself to, both because a renderer that quietly
shows less than everything is worse than no renderer:

* **It cannot pass by reading nothing.** A missing or empty tracker, or a
  board with no milestone rows, is an error rather than an empty page.
* **It cannot drop a relationship silently.** Every declared edge whose
  endpoint has no lane is a hard failure, and every open tracker node that
  is not drawn is reported by id. The lanes are a drawing order and a
  judgment; the edges are citations. Add a new node to `LANES` when it
  appears, or the run fails and tells you.

Percentages always travel with their denominator, on the page and in the
JSON. "Progress" is not one number: milestones measure the roadmap, tasks
measure the work *decomposed so far* (which falls as M2–M5 are decomposed),
and findings/debt are records rather than deliverables.
"""

from __future__ import annotations

import argparse
import html
import json
import re
import subprocess
import sys
from collections import Counter
from datetime import date
from pathlib import Path

import yaml

REPO = Path(__file__).resolve().parent.parent
TRACKER_DIR = REPO / ".agents" / "cache" / "tasks"
BOARD = REPO / "docs" / "implementation" / "TODO.md"
CACHE = REPO / ".agents" / "cache"

DONE = {"done", "wontfix"}
STANDING = {"standing"}


def fail(message: str) -> "NoReturn":  # noqa: F821 - typing.NoReturn without the import
    print(f"progress.py: {message}", file=sys.stderr)
    raise SystemExit(1)


# ── sources ────────────────────────────────────────────────────────────
def read_tracker() -> dict:
    """The branch-keyed tracker file `tools/tasks.py` itself resolves."""
    branch = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        cwd=REPO, capture_output=True, text=True, check=True,
    ).stdout.strip()
    # tasks.py keys the file by branch, with `HEAD` as the detached fallback;
    # the mapping is its own, so ask the directory rather than guessing.
    candidates = sorted(TRACKER_DIR.glob("*.yaml"))
    if not candidates:
        fail(f"no tracker files under {TRACKER_DIR.relative_to(REPO)} — nothing to render")
    named = TRACKER_DIR / f"{branch}.yaml"
    path = named if named.exists() else candidates[0]
    data = yaml.safe_load(path.read_text() or "")
    if not isinstance(data, dict) or not data.get("nodes"):
        fail(f"{path.relative_to(REPO)} parsed empty — refusing to render an empty page")
    data["_path"] = str(path.relative_to(REPO))
    return data


def head_sha() -> str:
    return subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        cwd=REPO, capture_output=True, text=True, check=True,
    ).stdout.strip()


def read_board() -> tuple[list[dict], list[dict]]:
    """§2's milestone rows and §4/§5's `U` items, from the committed board."""
    if not BOARD.exists():
        fail(f"{BOARD.relative_to(REPO)} is missing — the board is a source of truth")
    text = BOARD.read_text()

    def section(name: str, nxt: str) -> str:
        m = re.search(rf"^## {name}\..*?$(.*?)(?=^## {nxt}\.)", text, re.M | re.S)
        return m.group(1) if m else ""

    milestones = []
    for line in section("2", "3").splitlines():
        if re.match(r"^\| M\d", line):
            cells = [c.strip() for c in line.strip("|").split("|")]
            if len(cells) >= 4:
                milestones.append({"id": cells[0], "deliverable": cells[1], "state": cells[3]})
    if not milestones:
        fail("board §2 has no milestone rows — refusing to render a roadmap from nothing")

    items = []
    for sec, kind, nxt in (("4", "follow-up", "5"), ("5", "debt", "6")):
        body = section(sec, nxt)
        blocks = [b for b in re.split(r"(?m)^(?=- \[)", body) if b.startswith("- [")]
        for block in blocks:
            # `re.S` because a board item's bold title may wrap onto the next
            # line; the block boundary is what keeps `.*?` inside one item.
            m = re.match(r"- \[( |x)\] \*\*(U\d+) · (.*?)\*\*", block, re.S)
            if not m:
                fail(f"board §{sec} has an item this parser cannot read: "
                     f"{block.splitlines()[0][:70]!r} — fix the regex, do not drop it")
            items.append({
                "id": m.group(2),
                "kind": kind,
                "status": "done" if m.group(1) == "x" else "pending",
                "title": re.sub(r"\s+", " ", m.group(3)).split(".")[0],
            })
        # Every block either parsed or failed above, so the count is exact by
        # construction — which is the point: a renderer that silently skips an
        # item reports less than the truth and still looks complete.
    return milestones, items


# ── graph model ────────────────────────────────────────────────────────
# LANES is a drawing order: left to right is roughly time, and membership is
# by id. It is the one judgment on the page, and an open node that is not in
# it is reported rather than dropped.
LANES: list[tuple[str, list[str]]] = [
    ("bootstrap", ["g1", "g2"]),
    ("project", ["g3"]),
    ("M0 + signed decisions", ["t2", "t1", "t3", "t4", "t5", "t16", "f24", "g4",
                               "U8", "U14", "D12"]),
    ("M1 · domain (§3/§5)", ["t6", "t14", "t7", "t8", "t15", "f5"]),
    ("M1 · store (§4/§8)", ["t9", "t10", "t11", "t12", "f19"]),
    ("M1 · riders + cli", ["t18", "t19", "t17", "t22", "t13", "t20", "t21"]),
    ("M1 exit", ["g5"]),
    ("carried forward", ["f4", "f13", "f25", "f40", "f41", "d1", "d2", "d3",
                         "f31-39", "U13", "U15", "U19"]),
    ("harness (post-M1)", ["t23", "f42", "f43", "f44", "f45", "U21", "U22"]),
    ("next", ["U2", "U17", "M2"]),
    ("M2 · queries (§5.2/§6/§7)", ["g6", "t26", "t27", "t28", "t29", "t30",
                                    "t31", "g7", "D16", "f47"]),
    ("M2 findings (open unless noted)", ["f48", "f49", "f50", "f51", "f52",
                                         "f53", "f54", "f55", "f56", "f57",
                                         "f58", "f59", "f60", "f61", "f62",
                                         "f63", "f64", "f65"]),
    ("M3 · the wire (§6/§6.3)", ["g8", "t32", "t33", "t34", "t35", "t36",
                                 "t37", "t38", "t39", "g9", "M3", "U20"]),
    ("later", ["U16", "M4", "U18", "M5"]),
]

# GATING edges carry the citation that makes them an edge rather than a guess.
# Containment edges come from the tracker's own nesting and need none.
GATING: list[tuple[str, str, str]] = [
    ("g1", "g3", "the harness exists, so the server can be built"),
    ("t1", "t2", "D1–D3 signed → M0's workspace could be scaffolded"),
    ("t2", "g4", "M0's crates → M1 could fill them"),
    ("t3", "t6", "D9–D11 signed → statuses/references/next_items shapes"),
    ("t5", "t6", "U6 concretization → the id prefixes t6 types"),
    ("t16", "t12", "f21/U12: the engine fix made the relates narrowing optional"),
    ("f5", "t12", "f5: a snapshot restore needs a raise-to-max counter write"),
    ("U8", "t9", "f2: the dependency ask gated the store commits t9–t13"),
    ("t6", "t7", "value types before the graph that holds them"),
    ("t7", "t8", "§5.1's graph before §5.2's order over it"),
    ("t7", "t9", "the domain's serde derives are what the codec serializes"),
    ("t9", "t10", "the codec + golden harness before the log that uses them"),
    ("t10", "t11", "the log before the layout/registry that stores it"),
    ("t11", "t12", "the inode-lock measurement forced compact()'s reopen step"),
    ("t8", "t15", "f9/f10 rode the domain cleanup after the order landed"),
    ("t12", "t19", "f14/f17/f18/f23 are t11/t12 riders"),
    ("t19", "t22", "the riders cleared the store surface the repair primitive extends"),
    ("t22", "t20", "board §3: t22 is 'the commit t13b needs and did not have'"),
    ("U14", "t20", "board U14 'gates t13b only'; t20 was gated on the parser pick"),
    ("t17", "t20", "t17's measurement produced U14's pick, which t20 implements"),
    ("f19", "t21", "t21 is 'f19-test': the order-sensitive compact() test"),
    ("t20", "t13", "t13's umbrella closed when its three commits landed"),
    ("t21", "t13", "t21 was the last piece t13's note named"),
    ("t21", "g5", "board §1: the only M1 work left was t21, then the gate"),
    ("g5", "g4", "g4's note: M1 stays open until g5 passes"),
    ("g5", "f41", "the gate closed M1 without a tracker status sweep"),
    ("f40", "U15", "f40 promoted to the board so it outlives the branch"),
    ("f31-39", "U19", "f35 promoted to the board so the deferral outlives the branch"),
    ("U19", "t30", "board §4: U19 rides the next store touch, which t30 now is"),
    ("f25", "M3", "f25: reopen when the log backend settles; the cli is the consumer"),
    ("f4", "M3", "f4: M3's server must reject an untyped task_link at the wire"),
    ("f31-39", "f41", "f41 dispositions the nine t20/t22 spec-error records"),
    ("d1", "M2", "d1's note named M2's queries as its consumer, and the M2 decomposition closed it: neither the derivation nor the selection path calls tasks_of_epic"),
    ("d2", "M2", "U13: revisit the file ledger at M2"),
    ("d3", "M2", "U13: revisit the file ledger at M2"),
    ("U13", "M3", "U13 re-measured at the M2 decomposition and unchanged; M2's query layer lands in new todo-domain files, so the next seam-showing moment is M3"),
    ("U15", "t30", "board §4: U15 rides the next store touch, which t30 now is"),
    ("g5", "U2", "board: U2 is scheduled after g5, before M2"),
    ("U2", "M2", "U2 could invalidate a signed §6 shape M2 would be built on"),
    ("t31", "U20", "t31's cold-open measurement is the input U20's own text names"),
    ("U20", "M3", "board §4: due before M3 builds the wire — extending the registry later is a golden-visible format change, and it costs a re-derived lock order"),
    ("M2", "M3", "spec §14's milestone order"),
    ("M3", "M4", "spec §14's milestone order"),
    ("M4", "M5", "spec §14's milestone order"),
    ("D12", "M4", "D12 reversed D7: the dacode board is M4's only fixture"),
    ("t23", "f42", "promoting the renderer found the shadowed config key"),
    ("t23", "f43", "cross-checking the rendered rows against the board found the dropped items"),
    ("f42", "U21", "f42's generalisation — the duplicate-key check — is U21 now, promoted from f44 at the g7 sweep"),
    ("U2", "U16", "U16's instruction mechanism rests on U2's fourth question"),
    ("U16", "M3", "U16 lands with M3, and §9 still bars dogfooding before it"),
    ("U17", "D16", "U17's one unsettled piece — the progress roll-up — is drafted as D16"),
    ("D16", "M3", "a signed (a) or (b) adds one domain aggregation task to M3's decomposition (board §1, g6 closed at g7)"),
    ("t32", "t34", "status_summary's domain surface is what t34's wire tool serialises"),
    ("t32", "t35", "task_list's row type is what t35's wire listing carries"),
    ("t33", "t34", "the skeleton is what every tool task dispatches against (g8's dispatch order)"),
    ("t38", "g9", "the wire golden suite is the M3 exit gate's proof (D14/D15's pins)"),
    ("t39", "g9", "the opencode smoke is the M3 exit gate's last criterion"),
    ("t26", "t27", "the decomposition is what t27 is dispatched against (g6's note)"),
    ("t27", "t28", "t28 reuses select.rs's actionability predicate and DeferredSummary (tracker note)"),
    ("f47", "t28", "f47's note names t28's dispatch as its consumer: a rider section in a spec that re-reads select.rs anyway"),
    ("t27", "t29", "t29's 'ready' filter is t27's until-target predicate (tracker note)"),
    ("t30", "t31", "g6's dispatch order: the store riders before the registry queries"),
    ("g7", "g6", "g6's note: M2 stays open until g7 passes (the g5/g4 precedent)"),
    ("M4", "U18", "U18 is deferred by the user until after M4's importer check"),
]

# Open tracker nodes drawn as one node, because they are one class. Members are
# listed by id, not matched by a pattern: a new member is a data edit.
GROUPS: list[dict] = [
    {
        "id": "f31-39",
        "members": [f"f3{i}" for i in range(1, 10)],
        "label": "f31–f39 · the t20/t22 spec-error records — dispositioned by f41, f35 lives on as U19",
        "kind": "finding",
    },
]

# Board and decision entities that are not tracker nodes but belong in the
# picture. `(kind, status, label)`; statuses for board items come from the
# board itself where it has one.
EXTRA: dict[str, tuple[str, str, str]] = {
    "M2": ("milestone", "done", "M2 · queries: selection, dependency resolution, next_task (closed at g7)"),
    "M3": ("milestone", "pending", "M3 · MCP server: §6 over the wire"),
    "M4": ("milestone", "pending", "M4 · adoption: the dacode importer (D12)"),
    "M5": ("milestone", "pending", "M5 · packaging, docs, polish"),
    "U2": ("decision", "pending", "U2 · survey rmcp's tool macros against §6"),
    # Not a board item and not a milestone, so this status is authoritative
    # rather than derived: flipped to done when D16 was signed (f45 records
    # that the page cannot derive an open decision from decisions.md itself).
    "D16": ("decision", "done", "D16 · the progress roll-up (signed (a) 2026-10-03)"),
    "U13": ("debt", "pending", "U13 · two store files over the 600-line target"),
    "U15": ("follow-up", "pending", "U15 · scope Store::snapshot's 'changes nothing on disk'"),
    "U16": ("follow-up", "pending", "U16 · server base instructions + the client's native todo off"),
    "U17": ("follow-up", "pending", "U17 · which harness tools become MCP tools"),
    "U18": ("follow-up", "pending", "U18 · live view / shared per-user server (after M4)"),
    "U19": ("follow-up", "pending", "U19 · one word in repair.rs's module doc (f35 promoted)"),
    "U20": ("follow-up", "done", "U20 · the registry format stays; the server answers from handles (resolved (b) 2026-10-03)"),
    "U21": ("follow-up", "pending", "U21 · the duplicate-key check for agent-rules.yaml (f44 promoted at g7)"),
    "U22": ("follow-up", "pending", "U22 · progress.py's blind spot for open decisions (f45 promoted at g7)"),
    "U8": ("decision", "done", "U8 · the first external dependencies (resolved)"),
    "U14": ("decision", "done", "U14 · todo-cli's argument parser (resolved: lexopt)"),
    "D12": ("decision", "done", "D12 · dacode-first importer (signed)"),
}

# What each open item waits on, for the table. Absent = "nothing declared".
WAITS: dict[str, str] = {
    "f4": "M3's wire mapping",
    "f25": "the log backend settling (M3)",
    "f57": "a tests-only commit (the t21/f47 shape); its three seams are measured in the finding",
    "f58": "the next owner of crates/todo-store/src/log.rs — one sentence in open_log's doc fixes all three pointers",
    "U13": "the next commit that grows either file, or M3 — whichever comes first",
    "U16": "lands with M3 (t33's instructions, t39's gating); its mechanism is measured (spec §6.3)",
    "U18": "M4's importer check, by the user's own deferral",
    "g8": "t32–t39, then g9",
    "U21": "a user call: land the duplicate-key check in agents-sync.py, or decline it (board U21)",
    "U22": "a user call: draw open D-entries on the page, or disclaim them in the footer (board U22)",
    "f59": "M3, where the server holds handles and the fd ceiling stops being theoretical",
    "f60": "the next owner of crates/todo-store/src/registry.rs",
    "f61": "t13's owner: does verify gain the same slug cross-check, or is the query side the only reader that refuses?",
    "f63": "M3, with t31's measurement as its input (a lock-free reader needs an unsigned §4.3 reading)",
    "f64": "whoever re-runs the measurement at M3 — the caveats are the point",
    "f65": "a user call: give board §4 items a Gates field attention.py reads, or document that P3 does not mean 'no deadline'",
    "f50": "the next owner of tests/seeded.rs for the generator half — the naive-helper lift landed with t29",
    "f54": "the next owner of tests/mod.rs (= f50's)",
}

COLOR = {
    "done": ("#1f6f43", "#e6f4ec"),
    "standing": ("#1d4e89", "#e8f0fb"),
    "pending": ("#8a5a00", "#fdf3e0"),
    "active": ("#8a5a00", "#fdf3e0"),
    "blocked": ("#9b1c1c", "#fdeaea"),
}
KIND_MARK = {"goal": "◎", "gate": "▣", "task": "▢", "finding": "◆",
             "debt": "▲", "milestone": "◈", "decision": "✦", "follow-up": "◇"}
COLW, ROWH, PAD = 214, 44, 54


# ── model ──────────────────────────────────────────────────────────────
def flatten(raw: list[dict], parent: str | None = None,
            out: dict | None = None, order: list | None = None):
    out = {} if out is None else out
    order = [] if order is None else order
    for n in raw:
        nid = n["id"]
        out[nid] = {
            "id": nid,
            "kind": n["kind"],
            "status": n.get("status", "pending"),
            "title": n.get("title", ""),
            "parent": parent,
            "verify": n.get("verify", ""),
            "note": n.get("note", ""),
            "children": [c["id"] for c in (n.get("children") or [])],
        }
        order.append(nid)
        flatten(n.get("children") or [], nid, out, order)
    return out, order


def build(nodes: dict, order: list, milestones: list[dict], board_items: list[dict]) -> dict:
    open_nodes = [n for n in nodes.values() if n["status"] not in DONE | STANDING]
    kind_state = {}
    for k in ("goal", "gate", "task", "finding", "debt"):
        same = [n for n in nodes.values() if n["kind"] == k]
        kind_state[k] = {
            "total": len(same),
            "done": sum(1 for n in same if n["status"] in DONE),
            "standing": sum(1 for n in same if n["status"] in STANDING),
            "open": sum(1 for n in same if n["status"] not in DONE | STANDING),
            "blocked": sum(1 for n in same if n["status"] == "blocked"),
        }
    grouped = {m for g in GROUPS for m in g["members"]}
    graph_nodes, seen = [], set()
    for _, ids in LANES:
        for nid in ids:
            if nid in nodes:
                n = nodes[nid]
                graph_nodes.append({"id": nid, "kind": n["kind"], "status": n["status"],
                                    "label": n["title"], "lane": None,
                                    "detail": n["note"] or n["verify"]})
            elif any(nid == g["id"] for g in GROUPS):
                g = next(g for g in GROUPS if g["id"] == nid)
                members = [m for m in g["members"] if m in nodes]
                graph_nodes.append({
                    "id": nid, "kind": g["kind"],
                    "status": "pending" if any(nodes[m]["status"] not in DONE for m in members) else "done",
                    "label": g["label"], "lane": None,
                    "detail": f"collapsed group of {len(members)}: {', '.join(members)}",
                })
            elif nid in EXTRA:
                k, s, label = EXTRA[nid]
                if nid in {b["id"] for b in board_items}:
                    s = next(b["status"] for b in board_items if b["id"] == nid)
                if nid in {m["id"] for m in milestones}:
                    m = next(x for x in milestones if x["id"] == nid)
                    s = "done" if m["state"].startswith("done") else "pending"
                graph_nodes.append({"id": nid, "kind": k, "status": s, "label": label,
                                    "lane": None, "detail": ""})
            seen.add(nid)
    for i, (_, ids) in enumerate(LANES):
        for g in graph_nodes:
            if g["id"] in ids:
                g["lane"] = i

    edges, dropped = [], []
    for nid, n in nodes.items():
        for c in n["children"]:
            if nid in seen and c in seen:
                edges.append({"from": nid, "to": c, "kind": "contains",
                              "why": "tracker nesting"})
    for a, b, why in GATING:
        if a in seen and b in seen:
            edges.append({"from": a, "to": b, "kind": "gates", "why": why})
        else:
            missing = a if a not in seen else b
            dropped.append(f"{a}→{b} (no lane for {missing})")
    if dropped:
        fail("declared edges with no lane for an endpoint — add it to LANES: "
             + "; ".join(dropped))

    unplaced = [nid for nid in order
                if nid not in seen and nid not in grouped
                and nodes[nid]["status"] not in DONE]
    return {
        "open_nodes": open_nodes, "kind_state": kind_state, "graph_nodes": graph_nodes,
        "edges": edges, "unplaced": unplaced, "grouped": sorted(grouped), "seen": seen,
    }


# ── rendering ──────────────────────────────────────────────────────────
def esc(s) -> str:
    return html.escape(str(s), quote=True)


def rel(path: Path) -> str:
    """A printable path: repo-relative where it is, absolute where it is not
    (`--out` may point anywhere, and a renderer must not crash on that)."""
    try:
        return str(path.relative_to(REPO))
    except ValueError:
        return str(path)


def short(label: str, n: int = 30) -> str:
    label = re.sub(r"\s+", " ", str(label)).strip()
    return label if len(label) <= n else label[: n - 1] + "…"


def pct(a: int, b: int) -> str:
    return f"{round(100 * a / b)}%" if b else "—"


def render_svg(graph_nodes: list[dict], edges: list[dict]) -> tuple[str, int, int]:
    pos = {}
    for i, (_, ids) in enumerate(LANES):
        for j, nid in enumerate(ids):
            pos[nid] = (PAD + i * COLW, PAD + 34 + j * ROWH)
    width = PAD * 2 + len(LANES) * COLW
    height = PAD + 60 + max(len(ids) for _, ids in LANES) * ROWH
    out = [f'<svg id="graph" viewBox="0 0 {width} {height}" width="{width}" '
           f'height="{height}" xmlns="http://www.w3.org/2000/svg" role="img" '
           f'aria-label="task dependency graph">',
           '<defs><marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" '
           'markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" '
           'fill="#8a8f98"/></marker><marker id="arrg" viewBox="0 0 10 10" refX="9" '
           'refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">'
           '<path d="M0,0 L10,5 L0,10 z" fill="#2f6f4f"/></marker></defs>']
    for i, (name, _) in enumerate(LANES):
        x = PAD + i * COLW
        out.append(f'<text class="lane" x="{x}" y="{PAD - 8}">{esc(name)}</text>')
        out.append(f'<line class="laneline" x1="{x - 10}" y1="{PAD + 4}" '
                   f'x2="{x - 10}" y2="{height - 20}"/>')
    for e in edges:
        if e["from"] not in pos or e["to"] not in pos:
            continue
        x1, y1 = pos[e["from"]]
        x2, y2 = pos[e["to"]]
        x1e, y1e, x2e, y2e = x1 + 186, y1 + 15, x2, y2 + 15
        mx = (x1e + x2e) / 2
        cls, mk = ("edge", "arrg") if e["kind"] == "gates" else ("edgec", "arr")
        out.append(f'<path class="{cls}" d="M{x1e},{y1e} C{mx},{y1e} {mx},{y2e} {x2e},{y2e}" '
                   f'marker-end="url(#{mk})"><title>{esc(e["from"])} → {esc(e["to"])}: '
                   f'{esc(e["why"])}</title></path>')
    for g in graph_nodes:
        x, y = pos[g["id"]]
        border, fill = COLOR.get(g["status"], COLOR["pending"])
        out.append(
            f'<g class="node" data-id="{esc(g["id"])}" transform="translate({x},{y})">'
            f'<rect rx="7" width="186" height="30" fill="{fill}" stroke="{border}" '
            f'stroke-width="1.6"/><title>{esc(g["id"])} · {esc(g["kind"])} · '
            f'{esc(g["status"])}\n{esc(g["label"])}</title>'
            f'<text class="nid" x="8" y="19" fill="{border}">'
            f'{esc(KIND_MARK.get(g["kind"], "•"))} {esc(g["id"])}</text>'
            f'<text class="nlabel" x="46" y="19">{esc(short(g["label"], 22))}</text>'
            f'<circle class="dot" cx="176" cy="8" r="4" fill="{border}"/></g>')
    out.append("</svg>")
    return "\n".join(out), width, height


CSS = """
:root { color-scheme: light dark; }
body { font: 14px/1.5 -apple-system, "Segoe UI", Roboto, sans-serif; margin: 0;
       padding: 24px 28px 60px; color: #16181d; background: #fbfbfa; }
h1 { font-size: 20px; margin: 0 0 4px; }
h2 { font-size: 14px; text-transform: uppercase; letter-spacing: .08em;
     color: #6b7280; margin: 30px 0 10px; }
.sub { color: #6b7280; font-size: 12.5px; margin-bottom: 18px; }
.cards { display: flex; flex-wrap: wrap; gap: 10px; }
.card { border: 1px solid #e3e3e0; border-radius: 10px; padding: 10px 14px;
        background: #fff; min-width: 148px; }
.card b { display: block; font-size: 21px; font-variant-numeric: tabular-nums; }
.card i { font-style: normal; font-size: 11px; text-transform: uppercase;
          letter-spacing: .07em; color: #6b7280; }
.card small { display: block; color: #6b7280; font-size: 11.5px; margin-top: 4px;
              max-width: 210px; }
.card .p { float: right; font-size: 13px; color: #2f6f4f; font-weight: 600; }
.msrow { display: flex; gap: 8px; flex-wrap: wrap; }
.ms { border: 1px solid #e3e3e0; border-radius: 8px; padding: 6px 12px; background: #fff;
      font-size: 12.5px; }
.ms b { font-size: 13px; }
.ms span { display: block; color: #6b7280; font-size: 11px; }
.msdone { background: #e6f4ec; border-color: #bfe0cd; }
.msdone span { color: #1f6f43; }
.scroll { overflow-x: auto; border: 1px solid #e3e3e0; border-radius: 10px; background: #fff; }
svg text { font-family: inherit; }
.lane { font-size: 11px; fill: #6b7280; text-transform: uppercase; letter-spacing: .06em; }
.laneline { stroke: #efefec; stroke-width: 1; }
.nid { font-size: 11.5px; font-weight: 700; }
.nlabel { font-size: 10.5px; fill: #3f444c; }
.node { cursor: pointer; }
.node:hover rect { stroke-width: 2.6; }
.edge { fill: none; stroke: #2f6f4f; stroke-width: 1.3; opacity: .55; }
.edgec { fill: none; stroke: #b9bec6; stroke-width: 1; stroke-dasharray: 3 3; opacity: .8; }
table { border-collapse: collapse; width: 100%; background: #fff; font-size: 12.5px; }
th, td { text-align: left; padding: 6px 9px; border-bottom: 1px solid #eee; vertical-align: top; }
th { font-size: 11px; text-transform: uppercase; letter-spacing: .06em; color: #6b7280; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.st-done { color: #1f6f43; } .st-pending { color: #8a5a00; }
.st-blocked { color: #9b1c1c; font-weight: 700; } .st-standing { color: #1d4e89; }
tr.board td { background: #fcfaf3; }
#detail { border: 1px solid #e3e3e0; border-left: 4px solid #2f6f4f; border-radius: 8px;
          background: #fff; padding: 10px 14px; margin-top: 10px; display: none;
          font-size: 12.5px; }
#detail b { font-size: 13.5px; }
.legend { font-size: 11.5px; color: #6b7280; margin: 8px 0 0; }
.legend span { display: inline-block; margin-right: 14px; }
.sw { display: inline-block; width: 9px; height: 9px; border-radius: 2px; margin-right: 4px; }
footer { margin-top: 34px; color: #6b7280; font-size: 11.5px; border-top: 1px solid #eee;
         padding-top: 10px; }
"""


def render_page(model: dict, nodes: dict, milestones: list[dict], board_items: list[dict],
                sha: str, tracker_path: str) -> str:
    open_nodes, kind_state = model["open_nodes"], model["kind_state"]
    graph_nodes, edges = model["graph_nodes"], model["edges"]
    ms_done = sum(1 for m in milestones if m["state"].startswith("done"))
    blocked = sum(1 for n in nodes.values() if n["status"] == "blocked")
    svg, _, _ = render_svg(graph_nodes, edges)

    cards = [
        ("Milestones", f"{ms_done}/{len(milestones)}", pct(ms_done, len(milestones)),
         "spec §14 / board §2 — the roadmap measure"),
        ("Tasks", f"{kind_state['task']['done']}/{kind_state['task']['total']}",
         pct(kind_state["task"]["done"], kind_state["task"]["total"]),
         "tracker tasks, one reviewable commit each — decomposed work only"),
        ("Gates", f"{kind_state['gate']['done']}/{kind_state['gate']['total']}",
         pct(kind_state["gate"]["done"], kind_state["gate"]["total"]), "milestone exit gates"),
        ("Findings", f"{kind_state['finding']['done']}/{kind_state['finding']['total']}",
         pct(kind_state["finding"]["done"], kind_state["finding"]["total"]),
         "an open finding is a fact written down, not a defect"),
        ("Debt", f"{kind_state['debt']['done']}/{kind_state['debt']['total']} closed",
         pct(kind_state["debt"]["done"], kind_state["debt"]["total"]),
         "acknowledged deferrals, each with a revisit point"),
        ("Blocked", f"{blocked}", pct(0, 1) if blocked == 0 else "—",
         "waiting on a user decision; §12 is closed, so nothing is"),
    ]
    card_html = "".join(
        f'<div class="card"><i>{esc(t)}</i><span class="p">{esc(p)}</span><b>{esc(v)}</b>'
        f'<small>{esc(s)}</small></div>' for t, v, p, s in cards)

    ms_bar = "".join(
        f"<div class='ms {'msdone' if m['state'].startswith('done') else ''}' "
        f"title=\"{esc(m['deliverable'])} — {esc(m['state'])}\"><b>{esc(m['id'])}</b>"
        f"<span>{esc('done' if m['state'].startswith('done') else ('next' if i == ms_done else 'not started'))}"
        f"</span></div>" for i, m in enumerate(milestones))

    rows, listed = [], {n["id"] for n in open_nodes}
    for n in sorted(open_nodes, key=lambda n: (n["kind"], n["id"])):
        if n["id"] in model["grouped"]:
            continue
        rows.append(f"<tr><td class='mono'>{esc(n['id'])}</td><td>{esc(n['kind'])}</td>"
                    f"<td class='st-{esc(n['status'])}'>{esc(n['status'])}</td>"
                    f"<td>{esc(short(n['title'], 96))}</td>"
                    f"<td>{esc(WAITS.get(n['id'], ''))}</td></tr>")
    for g in GROUPS:
        members = [m for m in g["members"] if m in nodes and nodes[m]["status"] not in DONE]
        if members:
            rows.append(f"<tr><td class='mono'>{esc(g['id'])}</td><td>finding ×{len(members)}</td>"
                        f"<td class='st-pending'>open</td><td>{esc(g['label'])}</td>"
                        f"<td>{esc(WAITS.get(g['id'], ''))}</td></tr>")
    for b in board_items:
        if b["status"] != "done" and b["id"] not in listed and b["id"] not in nodes:
            rows.append(f"<tr class='board'><td class='mono'>{esc(b['id'])}</td>"
                        f"<td>{esc(b['kind'])} (board)</td><td class='st-pending'>open</td>"
                        f"<td>{esc(short(b['title'], 96))}</td>"
                        f"<td>{esc(WAITS.get(b['id'], ''))}</td></tr>")

    detail = {
        "nodes": {k: v for k, v in nodes.items()},
        "extra": {k: {"label": v[2], "kind": v[0], "status": v[1]} for k, v in EXTRA.items()},
        "groups": {g["id"]: {"label": g["label"], "members": g["members"]} for g in GROUPS},
        "edges": edges,
    }
    unplaced = model["unplaced"]
    warn = (f"<p class='legend'><b>Not drawn:</b> {esc(', '.join(unplaced))} — open tracker "
            f"nodes with no lane. Add them to <span class='mono'>LANES</span> in "
            f"<span class='mono'>tools/progress.py</span>.</p>" if unplaced else "")

    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<title>dacode-todo-mcp · progress</title>
<style>{CSS}</style></head><body>
<h1>dacode-todo-mcp — total progress</h1>
<div class="sub">Rendered by <span class="mono">tools/progress.py</span> on {esc(date.today())}
at <span class="mono">{esc(sha)}</span> from <span class="mono">{esc(tracker_path)}</span> and
<span class="mono">docs/implementation/TODO.md</span> §2/§4/§5. Every count is computed from those
two files; nothing is typed in. A rendering of committed state, never a source of it — if it
disagrees with the tracker or the board, those win. Machine-readable twin:
<span class="mono">progress.json</span>.</div>

<h2>Totals</h2>
<div class="cards">{card_html}</div>

<h2>Milestones (spec §14 / board §2)</h2>
<div class="msrow">{ms_bar}</div>

<h2>Dependency graph — {len(graph_nodes)} nodes, {len(edges)} edges</h2>
<div class="scroll">{svg}</div>
<p class="legend">
<span><i class="sw" style="background:#1f6f43"></i>done</span>
<span><i class="sw" style="background:#1d4e89"></i>standing goal</span>
<span><i class="sw" style="background:#8a5a00"></i>open</span>
<span><i class="sw" style="background:#9b1c1c"></i>blocked</span>
<span>— solid green: gates/feeds, hover for the citation · dashed grey: tracker containment</span>
<span>◎ goal ▣ gate ▢ task ◆ finding ▲ debt ◈ milestone ✦ decision</span>
<span>click a node for its verify line, note and edges</span>
</p>
{warn}
<div id="detail"></div>

<h2>Open items — {len(rows)} rows</h2>
<table><thead><tr><th>id</th><th>kind</th><th>status</th><th>what</th><th>waits on</th></tr></thead>
<tbody>{"".join(rows)}</tbody></table>

<footer>
Reading the percentages: each card states its own denominator, because "progress" depends entirely on
what is counted. <b>Milestones</b> measure the roadmap. <b>Tasks</b> measure the work decomposed so
far, which is <em>not</em> the whole project — milestones not yet decomposed have no task nodes, so
that figure falls as they are. <b>Findings</b> and <b>debt</b> are records rather than deliverables:
an open finding is a fact written down, and an open debt is an acknowledged deferral with a named
revisit point. <b>Blocked</b> counts nodes waiting on someone else, which today means the user; §12
is closed, so it is normally zero and a non-zero value is news.
</footer>
<script>
const DATA = {json.dumps(detail, ensure_ascii=False)};
document.querySelectorAll('.node').forEach(n => n.addEventListener('click', () => {{
  const id = n.dataset.id, d = DATA.nodes[id] || DATA.extra[id] || DATA.groups[id];
  const box = document.getElementById('detail');
  if (!d) return;
  const rel = DATA.edges.filter(e => e.from === id || e.to === id)
    .map(e => `<div>${{e.from}} ${{e.from === id ? '→' : '←'}} ${{e.to}}
      <span style="color:#6b7280">(${{e.why}})</span></div>`).join('');
  box.innerHTML = `<b>${{id}}</b> · ${{d.kind || 'group'}} · ${{d.status || ''}}<br>`
    + `${{d.label || d.title || ''}}`
    + (d.verify ? `<br><br><b>verify:</b> ${{d.verify}}` : '')
    + (d.note ? `<br><br><b>note:</b> ${{d.note}}` : '')
    + (d.members ? `<br><br><b>members:</b> ${{d.members.join(', ')}}` : '')
    + `<br><br><b>edges</b><br>${{rel || 'none drawn'}}`;
  box.style.display = 'block';
}}));
</script>
</body></html>
"""


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--open", action="store_true", help="open the page in a browser")
    ap.add_argument("--json", action="store_true",
                    help="print the JSON to stdout instead of writing the cache files")
    ap.add_argument("--out", metavar="DIR", default=None,
                    help=f"write there instead of {CACHE.relative_to(REPO)}")
    ap.add_argument("--no-write", action="store_true", help="print the summary only")
    args = ap.parse_args()

    tracker = read_tracker()
    tracker_path = tracker.pop("_path")
    nodes, order = flatten(tracker["nodes"])
    milestones, board_items = read_board()
    model = build(nodes, order, milestones, board_items)
    sha = head_sha()

    kind_state = model["kind_state"]
    open_nodes = model["open_nodes"]
    ms_done = sum(1 for m in milestones if m["state"].startswith("done"))
    blocked = sum(1 for n in nodes.values() if n["status"] == "blocked")
    data = {
        "generated": str(date.today()),
        "head": sha,
        "sources": [tracker_path, "docs/implementation/TODO.md §2/§4/§5"],
        "totals": {
            "tracker_nodes": len(nodes),
            "by_kind": kind_state,
            "open_nodes": [n["id"] for n in open_nodes],
            "blocked": [n["id"] for n in open_nodes if n["status"] == "blocked"],
            "milestones": {"done": ms_done, "total": len(milestones)},
            "open_board_items": [b["id"] for b in board_items if b["status"] != "done"],
            "not_drawn": model["unplaced"],
            "collapsed_groups": {g["id"]: g["members"] for g in GROUPS},
        },
        "milestones": milestones,
        "nodes": [nodes[i] for i in order],
        "board_items": board_items,
        "graph": {"nodes": model["graph_nodes"], "edges": model["edges"]},
    }

    if args.json:
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0

    page = render_page(model, nodes, milestones, board_items, sha, tracker_path)
    out = Path(args.out) if args.out else CACHE
    if not args.no_write:
        out.mkdir(parents=True, exist_ok=True)
        (out / "progress.html").write_text(page)
        (out / "progress.json").write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")

    print(f"progress @ {sha} — {len(nodes)} tracker nodes, "
          f"{len(open_nodes)} open, {blocked} blocked")
    for k, v in kind_state.items():
        print(f"  {k:8} {v['done']:3}/{v['total']:<3} done   {v['open']:2} open"
              f"{'   ' + str(v['standing']) + ' standing' if v['standing'] else ''}")
    print(f"  milestones {ms_done}/{len(milestones)} done · "
          f"open board items: {', '.join(data['totals']['open_board_items']) or 'none'}")
    print(f"  graph: {len(model['graph_nodes'])} nodes, {len(model['edges'])} edges "
          f"({sum(1 for e in model['edges'] if e['kind'] == 'gates')} gating, "
          f"{sum(1 for e in model['edges'] if e['kind'] == 'contains')} containment)")
    if model["unplaced"]:
        print(f"  NOT DRAWN (add a lane): {', '.join(model['unplaced'])}")
    if not args.no_write:
        print(f"  wrote {rel(out / 'progress.html')} and {rel(out / 'progress.json')}")
    if args.open:
        target = (out / "progress.html")
        if sys.platform == "darwin":
            subprocess.run(["open", str(target)], check=False)
        elif sys.platform.startswith("linux"):
            subprocess.run(["xdg-open", str(target)], check=False)
        else:
            print(f"  open it yourself: {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
