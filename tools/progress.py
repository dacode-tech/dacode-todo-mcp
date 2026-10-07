#!/usr/bin/env python3
"""Total progress, as a page you can open and a JSON you can query.

`tools/attention.py` answers "what needs the user". This answers the other
question, which no committed view covers: **what is the shape of the work** —
which task fed which, what is open, what is blocked, and how much of the
roadmap is behind us.

    tools/progress.py                    print the summary, refresh the cache files
    tools/progress.py --open             ...and open the page in a browser
    tools/progress.py --json             print the JSON to stdout instead of writing it
    tools/progress.py --out DIR          write somewhere other than the cache
    tools/progress.py --root DIR         the server's data root (default
                                         ~/.local/share/dacode-todo)
    tools/progress.py --refresh          re-register + re-import this repo's board
                                         into the root first, printing the census

A thin client over the server (the U28 cutover): every render spawns
`target/release/todo-mcp --root <root>`, drives the MCP handshake, asks
**one** `graph_show {repo: "dacode-todo-mcp"}` and renders everything from
that single answer — the lanes are the epics, each lane's nodes are the
epic's tasks in id order, the decision nodes attach to the lane of the epic
their Gates edge targets, the edges are the graph's blocks edges, the
open-items table is the open tasks, the waits column derives from the
epic's gates, and the totals derive client-side. The footer shows the repo
node's `registered_at` as the import instant, so staleness is visible: the
page is a rendering of the imported board, and the board is the source —
if they disagree, the board wins and this page is stale until `--refresh`.

The markdown board and `tools/tasks.py` stay the trackers of record (U28:
the write surface is unchanged, the cutover reversible). Beside the
server's committed picture the page keeps one overlay, clearly marked: an
"in flight (session tracker)" section reading the tracker yaml directly —
the board is the committed picture, the tracker is the session state, and
a page showing only the committed picture would be strictly worse
mid-milestone.

Two rules this tool holds itself to, both because a renderer that quietly
shows less than everything is worse than no renderer:

* **It cannot pass by reading nothing.** A missing binary, a server that
  will not answer, or a repo the root does not carry is an error rather
  than an empty page.
* **It cannot drop a node silently.** Every task sits in its epic's lane
  by construction; a decision the renderer cannot place (no gates edge to
  resolve a lane from) is reported on the page, never dropped, and so is
  every edge with an unplaced endpoint.

Percentages always travel with their denominator, on the page and in the
JSON. "Progress" is not one number: milestones measure the roadmap, tasks
measure the work *decomposed so far*, and findings/debt are records rather
than deliverables.
"""

from __future__ import annotations

import argparse
import fcntl
import html
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

import yaml

REPO = Path(__file__).resolve().parent.parent
TRACKER_DIR = REPO / ".agents" / "cache" / "tasks"
CACHE = REPO / ".agents" / "cache"
BINARY = REPO / "target" / "release" / "todo-mcp"
DEFAULT_ROOT = Path.home() / ".local" / "share" / "dacode-todo"
SLUG = "dacode-todo-mcp"

# The mapping's provenance prefix for a §2 milestone epic — the one
# contract the client reads off the epic description to tell the roadmap
# epics from the synthetic containers (and to recover the board's M id,
# which is the description's last token).
MILESTONE_DESCRIPTION = "the dacode-todo-mcp board's §2 milestone "

DONE = {"closed", "wontfix"}
TRACKER_DONE = {"done", "wontfix"}

# The refresh's crash-order test seam (D5/M6): set to "registry" or
# "rmtree" to make --refresh exit 99 right after that surgery step, so a
# test can observe which crash state the step order leaves behind.
CRASH_SEAM = "PROGRESS_REFRESH_CRASH_AFTER"


def fail(message: str) -> "NoReturn":  # noqa: F821 - typing.NoReturn without the import
    print(f"progress.py: {message}", file=sys.stderr)
    raise SystemExit(1)


def crash_after(step: str) -> None:
    """The crash-order seam: exit 99 right after `step`, if asked."""
    if os.environ.get(CRASH_SEAM) == step:
        print(f"progress.py: crash seam fired after '{step}' (exit 99)", file=sys.stderr)
        raise SystemExit(99)


# ── the thin client ────────────────────────────────────────────────────
def drive(root: Path, requests: list[str]) -> list[str]:
    """Spawn the release binary over stdio, handshake, ask, collect.

    One request at a time — each response line is read before the next
    request is written, so the response order is the request order by
    construction. A missing binary fails with the build command; a
    non-zero exit or server stderr fails the render.
    """
    if not BINARY.exists():
        fail(f"{BINARY.relative_to(REPO)} is missing — build it with: "
             "cargo build --release -p todo-mcp")
    proc = subprocess.Popen(
        [str(BINARY), "--root", str(root)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True,
    )
    initialize = (
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":'
        '{"protocolVersion":"2025-11-25","capabilities":{},'
        '"clientInfo":{"name":"progress","version":"0"}}}'
    )
    out = []
    proc.stdin.write(initialize + "\n")
    proc.stdin.flush()
    handshake = proc.stdout.readline()
    if not handshake.startswith('{"jsonrpc":"2.0","id":1,"result":{"protocolVersion"'):
        fail(f"the server's handshake did not answer: {handshake[:120]}")
    proc.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    proc.stdin.flush()
    for request in requests:
        proc.stdin.write(request + "\n")
        proc.stdin.flush()
        out.append(proc.stdout.readline())
    proc.stdin.close()
    trailing = proc.stdout.read()
    if trailing:
        fail(f"response bytes exist beyond the driven ones: {trailing[:120]}")
    stderr = proc.stderr.read()
    status = proc.wait()
    if status != 0:
        fail(f"the server exited {status}; stderr: {stderr[:500]}")
    if stderr:
        fail(f"the server wrote to stderr on a green path: {stderr[:500]}")
    return out


def call(i: int, name: str, arguments: str) -> str:
    return (f'{{"jsonrpc":"2.0","id":{i},"method":"tools/call",'
            f'"params":{{"name":"{name}","arguments":{arguments}}}}}')


def tool_answer(response: str, what: str) -> dict:
    """One tool response's structuredContent, or the carrier's message."""
    result = json.loads(response)["result"]
    if result.get("isError"):
        message = result["structuredContent"]["message"]
        fail(f"{what} failed: {message}")
    return result["structuredContent"]


def fetch_graph(root: Path) -> dict:
    """The one call every render reads: `graph_show` for this repo."""
    responses = drive(root, [call(2, "graph_show", json.dumps({"repo": SLUG}))])
    return tool_answer(responses[0], "graph_show")


# ── the refresh (D5) ───────────────────────────────────────────────────
def load_registry(path: Path) -> list[dict]:
    if not path.exists():
        return []
    document = json.loads(path.read_text())
    return document.get("repos", [])


def write_registry(path: Path, entries: list[dict]) -> None:
    """The store's own discipline: compact JSON, slug-sorted, one trailing
    newline, a pid-suffixed temp file in the same directory, fsync, then
    the atomic rename (§4.3)."""
    ordered = sorted(entries, key=lambda e: e["slug"])
    document = {"format_version": 1, "repos": ordered}
    body = json.dumps(document, separators=(",", ":")) + "\n"
    temporary = path.with_name(f"{path.name}.{os.getpid()}.tmp")
    with open(temporary, "w") as handle:
        handle.write(body)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)


def refresh(root: Path) -> dict:
    """The on-demand refresh: a registry surgery with a stated crash order.

    `apply_plan` refuses a non-empty graph, so a refresh re-registers:
    (1) flock `registry.json.lock` exclusive; (2) load `registry.json`,
    drop this repo's entry, write temp + fsync + `os.replace`; (3)
    `rmtree` this repo's store directory; (4) release; (5) spawn the
    server, `repo_register` + `import_md` over the wire. The ORDER is the
    safety argument: a crash after (2) leaves an orphan store directory —
    invisible, because the registry no longer lists it, cleaned by the
    next refresh — while the reverse order could leave a dangling entry,
    which breaks `repo_list` for every repo (the all-or-nothing rule).
    The dogfooding store is a derived cache of the board, rebuildable at
    any time; the other repos' stores are untouched by the surgery.
    """
    root.mkdir(parents=True, exist_ok=True)
    registry = root / "registry.json"
    lock_path = root / "registry.json.lock"
    store_dir = root / "repos" / SLUG
    lock_fd = os.open(lock_path, os.O_CREAT | os.O_RDWR)
    try:
        fcntl.flock(lock_fd, fcntl.LOCK_EX)
        entries = load_registry(registry)
        kept = [e for e in entries if e["slug"] != SLUG]
        if len(kept) != len(entries):
            write_registry(registry, kept)
        elif not registry.exists():
            # A fresh root: write the empty registry so the surgery's
            # step (2) is real even when there was nothing to drop.
            write_registry(registry, kept)
        crash_after("registry")
        if store_dir.exists():
            shutil.rmtree(store_dir)
        crash_after("rmtree")
    finally:
        fcntl.flock(lock_fd, fcntl.LOCK_UN)
        os.close(lock_fd)
    # (5) the re-register + the import, over the wire.
    requests = [
        call(2, "repo_register", json.dumps({
            "slug": SLUG, "name": SLUG, "path": str(REPO)})),
        call(3, "import_md", json.dumps({
            "repo": SLUG, "dialect": SLUG})),
    ]
    responses = drive(root, requests)
    tool_answer(responses[0], "repo_register")
    census = tool_answer(responses[1], "import_md")
    print(f"refresh: imported {census['epics']} epics, {census['decisions']} decisions, "
          f"{census['docs']} docs, {census['tasks']} tasks, {census['blocks']} blocks edges "
          f"into {root}")
    return census


# ── the tracker overlay (D7) ───────────────────────────────────────────
def read_tracker_overlay() -> tuple[list[dict], str]:
    """The session tracker's non-done items, as a flat list — no edges, no
    lanes. The tracker is a tracker of record (U28's own framing), so the
    overlay is a view of it, not hand-maintained state; a missing tracker
    degrades to a note, because the server is the page's primary source —
    the overlay reports its own absence rather than failing the page."""
    branch = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        cwd=REPO, capture_output=True, text=True, check=True,
    ).stdout.strip()
    candidates = sorted(TRACKER_DIR.glob("*.yaml"))
    if not candidates:
        return [], "(no session tracker found)"
    named = TRACKER_DIR / f"{branch}.yaml"
    path = named if named.exists() else candidates[0]
    data = yaml.safe_load(path.read_text() or "")
    if not isinstance(data, dict) or not data.get("nodes"):
        return [], f"({path.relative_to(REPO)} parsed empty)"
    items = []

    def walk(nodes):
        for node in nodes:
            if node.get("status", "pending") not in TRACKER_DONE:
                items.append({
                    "id": node["id"],
                    "kind": node.get("kind", "task"),
                    "status": node.get("status", "pending"),
                    "title": node.get("title", ""),
                })
            walk(node.get("children") or [])

    walk(data["nodes"])
    return items, str(path.relative_to(REPO))


def head_sha() -> str:
    return subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        cwd=REPO, capture_output=True, text=True, check=True,
    ).stdout.strip()


# ── the model, from the one graph_show answer ──────────────────────────
def id_number(node_id: str) -> int:
    """The numeric part of `<slug>:U17` — the id-order sort key."""
    return int(re.search(r"([0-9]+)$", node_id).group(1))


def short_id(node_id: str) -> str:
    return node_id.split(":", 1)[-1]


def milestone_of(epic: dict) -> str | None:
    """The board's M id for a milestone epic, from the mapping's
    provenance description; `None` for the synthetic containers."""
    if epic["description"].startswith(MILESTONE_DESCRIPTION):
        return epic["description"].rsplit(" ", 1)[-1]
    return None


def build(graph: dict) -> dict:
    """The render model from one `graph_show` answer: the lanes (the
    epics in the answer's order — ascending epic id, which is this
    repo's board order), each lane's tasks in id order, the decisions
    attached to the lane their Gates edge targets, the blocks edges,
    the open items with their waits, and the totals."""
    repos = graph.get("repos") or []
    if len(repos) != 1:
        fail(f"graph_show answered {len(repos)} repos — the render reads exactly one")
    repo_dump = repos[0]
    repo_node = repo_dump["repo"]
    if repo_node["slug"] != SLUG:
        fail(f"graph_show answered repo '{repo_node['slug']}', not '{SLUG}'")
    epics = repo_dump["epics"]
    decisions = repo_dump["decisions"]
    if not epics:
        fail("the graph holds no epics — refusing to render a roadmap from nothing")

    epic_ids = {epic["id"] for epic in epics}
    task_epic = {task["id"]: epic for epic in epics for task in epic["tasks"]}
    lane_of_epic = {epic["id"]: i for i, epic in enumerate(epics)}

    lanes = []
    for epic in epics:
        lanes.append({
            "epic": epic,
            "milestone": milestone_of(epic),
            "tasks": sorted(epic["tasks"], key=lambda t: id_number(t["id"])),
            "decisions": [],
        })

    # The decisions attach to the lane of the epic their Gates edge
    # targets; a decision gating tasks alone attaches to the lane of the
    # first gated task's epic; a decision gating nothing cannot be placed
    # and is reported, never dropped.
    unplaced = []
    placed_decisions = set()
    for decision in decisions:
        epic_targets = [b for b in decision["blocks"] if b in epic_ids]
        if epic_targets:
            lane = lane_of_epic[epic_targets[0]]
        else:
            task_targets = [b for b in decision["blocks"] if b in task_epic]
            if task_targets:
                lane = lane_of_epic[task_epic[task_targets[0]]["id"]]
            else:
                unplaced.append(f"{short_id(decision['id'])} (a decision gating nothing)")
                continue
        lanes[lane]["decisions"].append(decision)
        placed_decisions.add(decision["id"])
    for lane in lanes:
        lane["decisions"].sort(key=lambda d: id_number(d["id"]))

    # The edges: the graph's blocks edges, from the two adjacency lists
    # that carry them (a decision's blocks, a task's blocks — an epic's
    # blocked_by is their inverse and would double-count). An edge with
    # an unplaced endpoint is reported, never dropped.
    placed = epic_ids | set(task_epic) | placed_decisions
    edges = []
    dropped_edges = []
    for decision in decisions:
        for target in decision["blocks"]:
            if decision["id"] in placed and target in placed:
                edges.append({"from": decision["id"], "to": target})
            else:
                dropped_edges.append(f"{short_id(decision['id'])} → {short_id(target)}")
    for epic in epics:
        for task in epic["tasks"]:
            for successor in task["blocks"]:
                if task["id"] in placed and successor in placed:
                    edges.append({"from": task["id"], "to": successor})
                else:
                    dropped_edges.append(
                        f"{short_id(task['id'])} → {short_id(successor)}")

    # The waits column: an open item under an epic gated by an open
    # decision waits on it.
    open_decisions = {d["id"] for d in decisions if d["status"] == "open"}
    waits = {}
    for epic in epics:
        gates = [b for b in epic.get("blocked_by", []) if b in open_decisions]
        for task in epic["tasks"]:
            if task["status"] not in DONE and gates:
                waits[task["id"]] = ", ".join(short_id(g) for g in gates)

    tasks = [t for epic in epics for t in epic["tasks"]]
    open_items = sorted(
        (t for t in tasks if t["status"] not in DONE),
        key=lambda t: (lane_of_epic[task_epic[t["id"]]["id"]], id_number(t["id"])),
    )
    milestones = [epic for epic in epics if milestone_of(epic)]
    kind_state = {}
    for kind in ("task", "gate", "finding", "debt"):
        same = [t for t in tasks if t["kind"] == kind]
        kind_state[kind] = {
            "total": len(same),
            "done": sum(1 for t in same if t["status"] in DONE),
            "open": sum(1 for t in same if t["status"] not in DONE),
        }
    return {
        "repo": repo_node,
        "lanes": lanes,
        "decisions": decisions,
        "edges": edges,
        "waits": waits,
        "open_items": open_items,
        "milestones": milestones,
        "kind_state": kind_state,
        "unplaced": unplaced,
        "dropped_edges": dropped_edges,
        "task_count": len(tasks),
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


STATUS_COLOR = {
    "closed": ("#1f6f43", "#e6f4ec"),
    "wontfix": ("#1f6f43", "#e6f4ec"),
    "in_work": ("#8a5a00", "#fdf3e0"),
    "open": ("#8a5a00", "#fdf3e0"),
    "deferred": ("#8a5a00", "#fdf3e0"),
    "blocked": ("#9b1c1c", "#fdeaea"),
    "signed": ("#1f6f43", "#e6f4ec"),
}
KIND_MARK = {"task": "▢", "gate": "▣", "finding": "◆", "debt": "▲",
             "milestone": "◈", "decision": "✦"}
COLW, ROWH, PAD = 214, 44, 54


def node_status(status: str) -> str:
    return status if status in STATUS_COLOR else "open"


def render_svg(model: dict) -> tuple[str, int, int]:
    lanes = model["lanes"]
    pos = {}
    for i, lane in enumerate(lanes):
        nodes = lane["tasks"] + lane["decisions"]
        for j, node in enumerate(nodes):
            pos[node["id"]] = (PAD + i * COLW, PAD + 34 + j * ROWH)
    width = PAD * 2 + len(lanes) * COLW
    tallest = max(len(lane["tasks"]) + len(lane["decisions"]) for lane in lanes)
    height = PAD + 60 + tallest * ROWH
    out = [f'<svg id="graph" viewBox="0 0 {width} {height}" width="{width}" '
           f'height="{height}" xmlns="http://www.w3.org/2000/svg" role="img" '
           f'aria-label="task dependency graph">',
           '<defs><marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" '
           'markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" '
           'fill="#8a8f98"/></marker><marker id="arrg" viewBox="0 0 10 10" refX="9" '
           'refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">'
           '<path d="M0,0 L10,5 L0,10 z" fill="#2f6f4f"/></marker></defs>']
    for i, lane in enumerate(lanes):
        x = PAD + i * COLW
        header = (f"{lane['milestone']} · " if lane["milestone"] else "") \
            + short(lane["epic"]["title"], 24)
        out.append(f'<text class="lane" x="{x}" y="{PAD - 8}">{esc(header)}</text>')
        out.append(f'<line class="laneline" x1="{x - 10}" y1="{PAD + 4}" '
                   f'x2="{x - 10}" y2="{height - 20}"/>')
    for e in model["edges"]:
        if e["from"] not in pos or e["to"] not in pos:
            continue
        x1, y1 = pos[e["from"]]
        x2, y2 = pos[e["to"]]
        x1e, y1e, x2e, y2e = x1 + 186, y1 + 15, x2, y2 + 15
        mx = (x1e + x2e) / 2
        # D9: the edge's label derives from the nodes it connects — the
        # hand-maintained citations are accepted as lost.
        out.append(f'<path class="edge" d="M{x1e},{y1e} C{mx},{y1e} {mx},{y2e} {x2e},{y2e}" '
                   f'marker-end="url(#arrg)"><title>{esc(short_id(e["from"]))} gates '
                   f'{esc(short_id(e["to"]))}</title></path>')
    for lane in lanes:
        for node, kind in [(t, t["kind"]) for t in lane["tasks"]] \
                + [(d, "decision") for d in lane["decisions"]]:
            x, y = pos[node["id"]]
            status = node_status(node["status"])
            border, fill = STATUS_COLOR[status]
            out.append(
                f'<g class="node" data-id="{esc(node["id"])}" transform="translate({x},{y})">'
                f'<rect rx="7" width="186" height="30" fill="{fill}" stroke="{border}" '
                f'stroke-width="1.6"/><title>{esc(short_id(node["id"]))} · {esc(kind)} · '
                f'{esc(node["status"])}\n{esc(node["title"])}</title>'
                f'<text class="nid" x="8" y="19" fill="{border}">'
                f'{esc(KIND_MARK.get(kind, "•"))} {esc(short_id(node["id"]))}</text>'
                f'<text class="nlabel" x="46" y="19">{esc(short(node["title"], 22))}</text>'
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
table { border-collapse: collapse; width: 100%; background: #fff; font-size: 12.5px; }
th, td { text-align: left; padding: 6px 9px; border-bottom: 1px solid #eee; vertical-align: top; }
th { font-size: 11px; text-transform: uppercase; letter-spacing: .06em; color: #6b7280; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.st-closed { color: #1f6f43; } .st-open { color: #8a5a00; }
.st-in_work { color: #8a5a00; } .st-blocked { color: #9b1c1c; font-weight: 700; }
.st-signed { color: #1f6f43; }
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


def render_page(model: dict, tracker: tuple[list[dict], str], sha: str) -> str:
    repo = model["repo"]
    import_instant = repo["registered_at"]
    milestones = model["milestones"]
    kind_state = model["kind_state"]
    ms_done = sum(1 for m in milestones if m["status"] == "closed")
    blocked = sum(1 for t in model["open_items"] if t["status"] == "blocked")
    decisions = model["decisions"]
    signed = sum(1 for d in decisions if d["status"] == "signed")
    svg, _, _ = render_svg(model)
    graph_nodes = sum(len(l["tasks"]) + len(l["decisions"]) for l in model["lanes"])

    cards = [
        ("Milestones", f"{ms_done}/{len(milestones)}", pct(ms_done, len(milestones)),
         "board §2 via the server's epic nodes — the roadmap measure"),
        ("Tasks", f"{kind_state['task']['done']}/{kind_state['task']['total']}",
         pct(kind_state["task"]["done"], kind_state["task"]["total"]),
         "the board's §4 follow-ups, one unit each"),
        ("Gates", f"{kind_state['gate']['done']}/{kind_state['gate']['total']}",
         pct(kind_state["gate"]["done"], kind_state["gate"]["total"]),
         "milestone exit gates"),
        ("Findings", f"{kind_state['finding']['done']}/{kind_state['finding']['total']}",
         pct(kind_state["finding"]["done"], kind_state["finding"]["total"]),
         "an open finding is a fact written down, not a defect"),
        ("Debt", f"{kind_state['debt']['done']}/{kind_state['debt']['total']} closed",
         pct(kind_state["debt"]["done"], kind_state["debt"]["total"]),
         "the board's §5, acknowledged deferrals"),
        ("Decisions", f"{signed}/{len(decisions)} signed",
         pct(signed, len(decisions)),
         "the decisions file's dossiers; an open one gates work"),
        ("Blocked", f"{blocked}", pct(0, 1) if blocked == 0 else "—",
         "tasks whose status is blocked"),
    ]
    card_html = "".join(
        f'<div class="card"><i>{esc(t)}</i><span class="p">{esc(p)}</span><b>{esc(v)}</b>'
        f'<small>{esc(s)}</small></div>' for t, v, p, s in cards)

    ms_bar = "".join(
        f"<div class='ms {'msdone' if m['status'] == 'closed' else ''}' "
        f"title=\"{esc(m['title'])} — {esc(m['status'])}\"><b>{esc(milestone_of(m))}</b>"
        f"<span>{esc('done' if m['status'] == 'closed' else m['status'])}</span></div>"
        for m in milestones)

    rows = []
    for t in model["open_items"]:
        rows.append(f"<tr><td class='mono'>{esc(short_id(t['id']))}</td>"
                    f"<td>{esc(t['kind'])}</td>"
                    f"<td class='st-{esc(t['status'])}'>{esc(t['status'])}</td>"
                    f"<td>{esc(short(t['title'], 96))}</td>"
                    f"<td>{esc(model['waits'].get(t['id'], ''))}</td></tr>")

    decision_rows = []
    for d in decisions:
        gates = ", ".join(short_id(g) for g in d["blocks"]) or "—"
        decision_rows.append(
            f"<tr><td class='mono'>{esc(short_id(d['id']))}</td>"
            f"<td class='st-{esc(d['status'])}'>{esc(d['status'])}</td>"
            f"<td>{esc(short(d['title'], 72))}</td>"
            f"<td class='mono'>{esc(gates)}</td>"
            f"<td>{esc(d.get('confidence') or '')}</td></tr>")

    tracker_items, tracker_path = tracker
    tracker_rows = "".join(
        f"<tr><td class='mono'>{esc(n['id'])}</td><td>{esc(n['kind'])}</td>"
        f"<td class='st-{esc(n['status'])}'>{esc(n['status'])}</td>"
        f"<td>{esc(short(n['title'], 96))}</td></tr>"
        for n in tracker_items) or \
        f"<tr><td colspan='4'>{esc(tracker_path)}</td></tr>"

    unplaced = model["unplaced"] + model["dropped_edges"]
    warn = (f"<p class='legend'><b>Not placed:</b> {esc(', '.join(unplaced))} — nodes the "
            f"renderer cannot place are reported, never dropped.</p>" if unplaced else "")

    detail = {
        # The click box's fields only — created/updated are deliberately
        # absent: they are the import instant, and embedding them would
        # make every refresh move page bytes the box never shows.
        "nodes": {n["id"]: {
            "id": n["id"], "kind": n.get("kind", "decision"),
            "status": n["status"], "title": n["title"],
            "verify": n.get("verify"), "evidence": n.get("evidence"),
            "note": n.get("note"), "resolution": n.get("resolution"),
        } for lane in model["lanes"]
            for n in lane["tasks"] + lane["decisions"]},
        "edges": model["edges"],
    }

    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<title>dacode-todo-mcp · progress</title>
<style>{CSS}</style></head><body>
<h1>dacode-todo-mcp — total progress</h1>
<div class="sub">Rendered by <span class="mono">tools/progress.py</span> on {esc(date.today())}
at <span class="mono">{esc(sha)}</span> from one <span class="mono">graph_show</span> answer of
<span class="mono">target/release/todo-mcp</span> over this repo's imported board —
imported <span class="mono">{esc(import_instant)}</span> (the repo node's registered_at;
run <span class="mono">--refresh</span> when the board has moved). The board and
<span class="mono">tools/tasks.py</span> remain the trackers of record; if this page disagrees
with them, they win. Machine-readable twin: <span class="mono">progress.json</span>.</div>

<h2>Totals</h2>
<div class="cards">{card_html}</div>

<h2>Milestones (board §2 via the server's epics)</h2>
<div class="msrow">{ms_bar}</div>

<h2>Dependency graph — {graph_nodes} nodes, {len(model['edges'])} edges</h2>
<div class="scroll">{svg}</div>
<p class="legend">
<span><i class="sw" style="background:#1f6f43"></i>done / signed</span>
<span><i class="sw" style="background:#8a5a00"></i>open / in work</span>
<span><i class="sw" style="background:#9b1c1c"></i>blocked</span>
<span>— the lanes are the epics; a decision sits in the lane of the epic it gates</span>
<span>▢ task ▣ gate ◆ finding ▲ debt ✦ decision</span>
<span>click a node for its fields and edges</span>
</p>
{warn}
<div id="detail"></div>

<h2>Open items — {len(rows)} rows</h2>
<table><thead><tr><th>id</th><th>kind</th><th>status</th><th>what</th><th>waits on</th></tr></thead>
<tbody>{"".join(rows)}</tbody></table>

<h2>Decisions — {len(decision_rows)} rows</h2>
<table><thead><tr><th>id</th><th>status</th><th>what</th><th>gates</th><th>confidence</th></tr></thead>
<tbody>{"".join(decision_rows)}</tbody></table>

<h2>In flight (session tracker) — {len(tracker_items)} items</h2>
<div class="sub">The gitignored session state beside the committed picture, read from
<span class="mono">{esc(tracker_path)}</span> — a flat list, no edges, no lanes.</div>
<table><thead><tr><th>id</th><th>kind</th><th>status</th><th>what</th></tr></thead>
<tbody>{tracker_rows}</tbody></table>

<footer>
Reading the percentages: each card states its own denominator, because "progress" depends
entirely on what is counted. <b>Milestones</b> measure the roadmap. <b>Tasks</b> measure the
work decomposed so far, which is <em>not</em> the whole project. <b>Findings</b> and
<b>debt</b> are records rather than deliverables. The graph's edges carry no citation text —
the edge's label derives from the nodes it connects, and the rich citations stay in the
tracker where they were. The import instant above is the staleness signal: the page renders
the last import, not the board as it is now.
</footer>
<script>
const DATA = {json.dumps(detail, ensure_ascii=False)};
document.querySelectorAll('.node').forEach(n => n.addEventListener('click', () => {{
  const id = n.dataset.id, d = DATA.nodes[id];
  const box = document.getElementById('detail');
  if (!d) return;
  const rel = DATA.edges.filter(e => e.from === id || e.to === id)
    .map(e => `<div>${{e.from}} ${{e.from === id ? '→' : '←'}} ${{e.to}}</div>`).join('');
  box.innerHTML = `<b>${{id}}</b> · ${{d.kind || 'decision'}} · ${{d.status}}<br>`
    + `${{d.title || ''}}`
    + (d.verify ? `<br><br><b>verify:</b> ${{d.verify}}` : '')
    + (d.evidence ? `<br><b>evidence:</b> ${{d.evidence}}` : '')
    + (d.note ? `<br><b>note:</b> ${{d.note}}` : '')
    + (d.resolution ? `<br><b>resolution:</b> ${{d.resolution}}` : '')
    + `<br><br><b>edges</b><br>${{rel || 'none drawn'}}`;
  box.style.display = 'block';
}}));
</script>
</body></html>
"""


def render_json(model: dict, tracker: tuple[list[dict], str], sha: str) -> dict:
    tracker_items, tracker_path = tracker
    return {
        "generated": str(date.today()),
        "head": sha,
        "repo": model["repo"]["slug"],
        "import_instant": model["repo"]["registered_at"],
        "sources": ["the server's graph_show over the imported board",
                    f"{tracker_path} (the in-flight overlay)"],
        "totals": {
            "epics": len(model["lanes"]),
            "milestones": {"done": sum(1 for m in model["milestones"]
                                       if m["status"] == "closed"),
                           "total": len(model["milestones"])},
            "tasks": model["task_count"],
            "by_kind": model["kind_state"],
            "open_items": [short_id(t["id"]) for t in model["open_items"]],
            "decisions": {"signed": sum(1 for d in model["decisions"]
                                        if d["status"] == "signed"),
                          "total": len(model["decisions"])},
            "not_placed": model["unplaced"] + model["dropped_edges"],
        },
        "milestones": [
            {"id": milestone_of(m), "title": m["title"], "status": m["status"]}
            for m in model["milestones"]
        ],
        "open_items": [
            {"id": short_id(t["id"]), "kind": t["kind"], "status": t["status"],
             "title": t["title"], "waits_on": model["waits"].get(t["id"], "")}
            for t in model["open_items"]
        ],
        "decisions": [
            {"id": short_id(d["id"]), "status": d["status"], "title": d["title"],
             "gates": [short_id(g) for g in d["blocks"]]}
            for d in model["decisions"]
        ],
        "tracker": tracker_items,
        "graph": {
            "nodes": [{"id": short_id(n["id"]), "kind": n.get("kind", "decision"),
                       "status": n["status"], "lane": lane["epic"]["id"]}
                      for lane in model["lanes"]
                      for n in lane["tasks"] + lane["decisions"]],
            "edges": [{"from": short_id(e["from"]), "to": short_id(e["to"])}
                      for e in model["edges"]],
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--open", action="store_true", help="open the page in a browser")
    ap.add_argument("--json", action="store_true",
                    help="print the JSON to stdout instead of writing the cache files")
    ap.add_argument("--out", metavar="DIR", default=None,
                    help=f"write there instead of {CACHE.relative_to(REPO)}")
    ap.add_argument("--no-write", action="store_true", help="print the summary only")
    ap.add_argument("--root", metavar="DIR", default=str(DEFAULT_ROOT),
                    help=f"the server's data root (default {DEFAULT_ROOT})")
    ap.add_argument("--refresh", action="store_true",
                    help="re-register + re-import this repo's board into the root first, "
                         "printing the import census")
    args = ap.parse_args()
    root = Path(args.root).expanduser()

    if args.refresh:
        refresh(root)

    graph = fetch_graph(root)
    model = build(graph)
    tracker = read_tracker_overlay()
    sha = head_sha()

    kind_state = model["kind_state"]
    ms_done = sum(1 for m in model["milestones"] if m["status"] == "closed")
    signed = sum(1 for d in model["decisions"] if d["status"] == "signed")
    data = render_json(model, tracker, sha)

    if args.json:
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0

    page = render_page(model, tracker, sha)
    out = Path(args.out) if args.out else CACHE
    if not args.no_write:
        out.mkdir(parents=True, exist_ok=True)
        (out / "progress.html").write_text(page)
        (out / "progress.json").write_text(
            json.dumps(data, indent=2, ensure_ascii=False) + "\n")

    print(f"progress @ {sha} — imported {model['repo']['registered_at']} "
          f"({model['task_count']} tasks, {len(model['decisions'])} decisions)")
    for k, v in kind_state.items():
        print(f"  {k:8} {v['done']:3}/{v['total']:<3} done   {v['open']:2} open")
    print(f"  milestones {ms_done}/{len(model['milestones'])} done · "
          f"decisions {signed}/{len(model['decisions'])} signed · "
          f"open items: {', '.join(data['totals']['open_items']) or 'none'}")
    print(f"  graph: {sum(len(l['tasks']) + len(l['decisions']) for l in model['lanes'])} "
          f"nodes, {len(model['edges'])} edges")
    if model["unplaced"] or model["dropped_edges"]:
        print(f"  NOT PLACED (reported, never dropped): "
              f"{', '.join(model['unplaced'] + model['dropped_edges'])}")
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
