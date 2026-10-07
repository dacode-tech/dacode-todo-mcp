#!/usr/bin/env python3
"""The serve-mode pin for `tools/progress.py --serve` (U18 shape A).

The serve mode's contract, pinned against a fresh temp root and a temp
port file (via `PROGRESS_SERVE_PORT_FILE`, so the driver never touches
the user's real publication and never attaches to a real listener):

* the **default invocation stays static**: it exits 0, writes the page
  pair, and publishes no port — a listener would keep it alive;
* `--serve` **publishes and answers**: the port file appears, the
  serving line names the published URL, `/` answers the page, `/json`
  answers a parsing document, `/health` answers the marker without
  rendering, anything else is 404;
* a **second invocation attaches**: exit 0, the same URL on stdout, the
  publication unchanged, the first listener still answering — never a
  second listener;
* a **stale publication is taken over**: SIGTERM skips the cleanup (no
  `finally` on the default disposition), the port file survives as
  stale, and the next `--serve` binds its own port and answers;
* **SIGINT is the clean shutdown**: exit 0 and the publication removed.
  A shell's background jobs ignore SIGINT and the ignore survives exec,
  so this pin runs from the driver — a Popen child gets the default
  disposition, like a real terminal's Ctrl-C.

f109 rides the same file: `--json --refresh` must print pure JSON on
stdout with the import census on stderr.

The wait helper polls the publication **and** probes `/health`, so a
stale port left in the file cannot masquerade as the new listener's
publish. The "no port scanning" rule (bind `:0`, read the assigned port
back) and the per-request render are code shape, pinned by review.
Requires `target/release/todo-mcp` (the renders spawn it). Exit 0 when
every pin holds, 1 otherwise.
"""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

# progress.py is a sibling script, not a package member: load it by
# path so the driver reuses its functions instead of re-implementing
# the wire protocol.
_spec = importlib.util.spec_from_file_location("progress", REPO / "tools" / "progress.py")
progress = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(progress)

PROGRESS = REPO / "tools" / "progress.py"
TITLE_MARKER = "<title>dacode-todo-mcp · progress</title>"


def http_get(url: str, timeout: float = 10) -> tuple[int, str]:
    """(status, body) — a transport failure is (0, ""), never a raise."""
    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            return response.status, response.read().decode("utf-8", "replace")
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode("utf-8", "replace")
    except (OSError, ValueError):
        return 0, ""


def read_port(port_file: Path) -> int | None:
    try:
        text = port_file.read_text().strip()
    except OSError:
        return None
    return int(text) if text.isdigit() else None


def wait_for_listener(port_file: Path, proc: subprocess.Popen,
                      timeout: float = 20) -> int | None:
    """Poll until the publication names a port that answers `/health` —
    or the process dies, or the timeout passes. Probing (not just
    reading the file) is what makes the stale-publication takeover
    below honest: a stale port in the file never counts as published."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            return None
        port = read_port(port_file)
        if port is not None and progress.listener_answers(port):
            return port
        time.sleep(0.1)
    return None


def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="progress-serve-check-"))
    port_file = root / "serve.port"
    env = {**os.environ, progress.SERVE_PORT_FILE_ENV: str(port_file)}
    failures: list[str] = []
    procs: list[subprocess.Popen] = []

    def check(name: str, ok: bool) -> None:
        print(f"  {'ok' if ok else 'FAIL'}  {name}")
        if not ok:
            failures.append(name)

    def spawn_serve() -> subprocess.Popen:
        proc = subprocess.Popen(
            [sys.executable, str(PROGRESS), "--serve", "--root", str(root)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)
        procs.append(proc)
        return proc

    try:
        # The registered state every render reads (a failed seed raises
        # here — the checks below are not vacuous).
        progress.refresh(root)

        # 1. The default invocation stays static.
        try:
            static = subprocess.run(
                [sys.executable, str(PROGRESS), "--root", str(root),
                 "--out", str(root / "static")],
                capture_output=True, text=True, env=env, timeout=120)
            check("the default invocation exits 0 (a listener would hang)",
                  static.returncode == 0)
        except subprocess.TimeoutExpired:
            check("the default invocation exits 0 (a listener would hang)", False)
        check("the default invocation writes the static pair",
              (root / "static" / "progress.html").exists()
              and (root / "static" / "progress.json").exists())
        check("the default invocation publishes no port",
              not port_file.exists())

        # 2. --serve publishes and answers.
        first = spawn_serve()
        port = wait_for_listener(port_file, first)
        check("--serve publishes a port that answers", port is not None)
        if port is None:
            raise SystemExit(1)
        status, body = http_get(f"http://127.0.0.1:{port}/")
        check("/ answers the page over 127.0.0.1",
              status == 200 and TITLE_MARKER in body)
        status, body = http_get(f"http://127.0.0.1:{port}/json")
        try:
            json.loads(body)
            parses = True
        except json.JSONDecodeError:
            parses = False
        check("/json answers a parsing document", status == 200 and parses)
        status, body = http_get(f"http://127.0.0.1:{port}/health")
        check("/health answers the marker", status == 200 and progress.HEALTH_MARKER in body)
        status, _ = http_get(f"http://127.0.0.1:{port}/nope")
        check("an unknown path is 404", status == 404)

        # 3. A second invocation attaches — never a second listener.
        try:
            second = subprocess.run(
                [sys.executable, str(PROGRESS), "--serve", "--root", str(root)],
                capture_output=True, text=True, env=env, timeout=60)
            check("the second invocation attaches and exits 0",
                  second.returncode == 0)
            check("the second invocation names the same URL",
                  f"http://127.0.0.1:{port}/" in second.stdout)
        except subprocess.TimeoutExpired:
            check("the second invocation attaches and exits 0", False)
            check("the second invocation names the same URL", False)
        check("the publication is unchanged (no second listener)",
              read_port(port_file) == port)
        status, body = http_get(f"http://127.0.0.1:{port}/")
        check("the first listener still answers after the attach",
              status == 200 and TITLE_MARKER in body)

        # 4. A stale publication is taken over.
        first.terminate()
        first.wait()
        startup_line = first.stdout.read() or ""
        check("the serving line names the published URL",
              f"serving http://127.0.0.1:{port}/" in startup_line)
        # Captured before the restore below: the restore exists so the
        # takeover check can run on the defect path too, and must not
        # mask this pin — a SIGTERM handler that cleans up is exactly
        # the regression it exists to catch.
        stale_left = port_file.exists()
        check("SIGTERM leaves the stale publication", stale_left)
        if not stale_left:
            port_file.write_text(f"{port}\n")
        third = spawn_serve()
        new_port = wait_for_listener(port_file, third)
        check("a stale publication is taken over", new_port is not None)
        if new_port is not None:
            status, body = http_get(f"http://127.0.0.1:{new_port}/health")
            check("the takeover listener answers",
                  status == 200 and progress.HEALTH_MARKER in body)

        # 4b. SIGINT is the clean shutdown: exit 0, publication removed.
        try:
            third.send_signal(signal.SIGINT)
            third.wait(timeout=10)
            check("SIGINT shuts the listener down cleanly (exit 0)",
                  third.returncode == 0)
        except subprocess.TimeoutExpired:
            check("SIGINT shuts the listener down cleanly (exit 0)", False)
        check("the clean shutdown removes the publication",
              not port_file.exists())

        # 5. f109: --json --refresh prints pure JSON on stdout.
        machine = subprocess.run(
            [sys.executable, str(PROGRESS), "--json", "--refresh",
             "--root", str(root)],
            capture_output=True, text=True, env=env, timeout=120)
        try:
            json.loads(machine.stdout)
            pure = True
        except json.JSONDecodeError:
            pure = False
        check("--json --refresh stdout is pure JSON (f109)",
              machine.returncode == 0 and pure
              and "refresh:" not in machine.stdout)
        check("the import census went to stderr (f109)",
              "refresh: imported" in machine.stderr)
    finally:
        for proc in procs:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait()
        shutil.rmtree(root, ignore_errors=True)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
