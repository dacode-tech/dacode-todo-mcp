#!/usr/bin/env python3
"""The crash-order pin for `tools/progress.py --refresh` (D5).

The refresh's registry surgery runs in a fixed order: write the
registry without this repo's entry, then remove the store directory,
then re-register and re-import over the wire. The order is the safety
argument, and this driver pins it by driving both crash seams against
a fresh temp root — never the live root, so the driver writes nowhere
but its own temp directory:

* a crash after the registry write leaves the **orphan-dir state**:
  the store directory remains but the registry no longer lists it —
  invisible to every query, cleaned by the next refresh;
* a crash after the rmtree leaves the clean slate: neither the entry
  nor the directory remains.

The failure the order exists to prevent is the **dangling-entry
state** — a registry entry whose store directory is gone — which the
reverse order could leave and which breaks `repo_list` for every repo
(the all-or-nothing rule). After each crash the driver asserts that
state never occurred and drives `repo_list` over the wire to confirm
the root still answers. Exit 0 when every pin holds, 1 otherwise.
"""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

# progress.py is a sibling script, not a package member: load it by
# path so the driver reuses its functions instead of re-implementing
# the wire protocol.
_spec = importlib.util.spec_from_file_location("progress", REPO / "tools" / "progress.py")
progress = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(progress)

SLUG = progress.SLUG


def crash_run(root: Path, step: str) -> None:
    """One crash-seam run: the refresh must exit 99 right after `step`."""
    os.environ[progress.CRASH_SEAM] = step
    try:
        progress.refresh(root)
        print(f"  FAIL  the '{step}' crash seam did not fire", file=sys.stderr)
        raise SystemExit(1)
    except SystemExit as fired:
        if fired.code != 99:
            raise
    finally:
        os.environ.pop(progress.CRASH_SEAM, None)


def state(root: Path) -> tuple[bool, bool]:
    """(the registry lists the repo, the store directory exists)."""
    registry = json.loads((root / "registry.json").read_text())
    listed = any(entry["slug"] == SLUG for entry in registry["repos"])
    return listed, (root / "repos" / SLUG).exists()


def repo_list_answers(root: Path) -> bool:
    """`repo_list` over the wire: the root still answers or this fails."""
    responses = progress.drive(root, [progress.call(2, "repo_list", "{}")])
    progress.tool_answer(responses[0], "repo_list")
    return True


def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="progress-refresh-check-"))
    failures: list[str] = []

    def check(name: str, ok: bool) -> None:
        print(f"  {'ok' if ok else 'FAIL'}  {name}")
        if not ok:
            failures.append(name)

    try:
        # The registered state the surgery starts from (a failed seed
        # refresh raises here — the crash checks are not vacuous).
        progress.refresh(root)

        # Crash after the registry write: the orphan-dir state — never
        # the dangling entry (listed, directory gone).
        crash_run(root, "registry")
        listed, directory = state(root)
        check("crash after registry: the entry is gone, the dir remains",
              not listed and directory)
        check("repo_list still answers over the orphan state",
              repo_list_answers(root))

        # Crash after the rmtree: the clean slate.
        crash_run(root, "rmtree")
        listed, directory = state(root)
        check("crash after rmtree: neither the entry nor the dir remains",
              not listed and not directory)
        check("repo_list still answers over the clean slate",
              repo_list_answers(root))
    finally:
        shutil.rmtree(root, ignore_errors=True)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
