#!/usr/bin/env python3
"""Check relative links and in-page anchors across the markdown docs.

The implementation-plan split was verified with an ad-hoc walker that was
never committed, which left the B4 hygiene item ("re-run the link check
after adding/removing docs") un-runnable. This is that walker, kept.

Scope: every ``*.md`` under the given roots (default: ``docs``,
``.agents/knowledge``, and the root ``AGENTS.md``).

Two things are checked per link:

* **relative file targets** resolve to something that exists on disk;
* **in-page anchors** (``#heading``) match a heading in the same file, and
  **cross-file anchors** (``path#heading``) match a heading in the target.

External URLs (``http``, ``https``, ``mailto``) are skipped — this is a
repo-internal consistency check, not a network probe.

One whole-file convention is enforced as well, unconditionally and independently
of the roots given: **``docs/todo-mcp-design.md`` must carry no markdown links
at all.** See ``NO_LINK_FILES``.

Usage::

    tools/check-links.py                     # check the default roots
    tools/check-links.py docs                # check only docs/
    tools/check-links.py --quiet             # only print failures
    tools/check-links.py --conventions-only  # just the no-link convention

Exit code is 0 when every link resolves and every convention holds, 1 otherwise.
"""

from __future__ import annotations

import argparse
import re
import sys
import unicodedata
from pathlib import Path

# [text](target) — target captured without the closing paren. Markdown allows
# nested parens in link text, so anchor on the last `](` before the target.
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
HEADING_RE = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$", re.MULTILINE)
INLINE_CODE_RE = re.compile(r"(`+)(.+?)\1", re.DOTALL)

DEFAULT_ROOTS = ("docs", ".agents/knowledge")
DEFAULT_FILES = ("AGENTS.md",)
SKIP_PREFIXES = ("http://", "https://", "mailto:", "//")

# Files that must contain no markdown link whatsoever. The spec is cited *by* the
# implementation plan and never cites it back, so a link in the spec inverts that
# dependency: it would stop being readable on its own and start breaking whenever
# the plan is reorganised. Four deltas have now proposed spec text carrying links
# to appendices and feature docs, each caught by hand before transcribing — this
# is the machine holding the convention instead of the fourth reviewer.
NO_LINK_FILES = ("docs/todo-mcp-design.md",)

# The link roots are resolved against the CWD, as they always have been — the tool
# is invoked from the repo root. The convention files are anchored to the repo
# instead, because they name one specific file: resolved against the CWD, running
# the tool from anywhere else reported the spec as "renamed?", a false alarm about
# a convention that had not been violated.
REPO = Path(__file__).resolve().parent.parent


def slugify(text: str) -> str:
    """Approximate GitHub's heading-anchor algorithm.

    Good enough to catch a stale anchor, which is the point. Deliberately
    conservative: an anchor this function cannot reproduce is reported as
    unknown rather than guessed at.
    """
    text = unicodedata.normalize("NFKD", text)
    text = "".join(c for c in text if not unicodedata.combining(c))
    text = text.lower()
    # drop markdown emphasis and inline code markers
    text = re.sub(r"[`*_]", "", text)
    out = []
    for ch in text:
        if ch.isalnum() or ch in "-_":
            out.append(ch)
        elif ch.isspace():
            out.append("-")
        # everything else (punctuation, brackets) is dropped
    return "".join(out)


def strip_fenced(text: str) -> str:
    """Blank out fenced code blocks so their contents are not parsed as links.

    Each line is replaced by whitespace **of the same length**, not by an empty
    string. That preserves character offsets as well as line numbers, which is
    what makes ``line_of(raw, match.start())`` report the right line: a match
    offset taken from the stripped text is only meaningful against the raw text if
    nothing before it changed length. Emitting "" kept the line count but shifted
    every later offset earlier, so a link on line 300 of a file with fenced blocks
    above it was reported as line 178.
    """
    lines = text.split("\n")
    out: list[str] = []
    in_fence = False
    fence_marker = ""
    for line in lines:
        stripped = line.lstrip()
        if not in_fence and (stripped.startswith("```") or stripped.startswith("~~~")):
            in_fence = True
            fence_marker = stripped[:3]
            out.append(" " * len(line))
            continue
        if in_fence:
            out.append(" " * len(line))
            if stripped.startswith(fence_marker):
                in_fence = False
            continue
        out.append(line)
    return "\n".join(out)


def _blank(text: str) -> str:
    """Replace every non-newline character with a space, preserving offsets."""
    return "".join("\n" if c == "\n" else " " for c in text)


def strip_code_regions(text: str) -> str:
    """Blank out fenced blocks *and* inline code spans, for link extraction.

    A link shape inside backticks is prose about a link, not a link, so it must
    not be resolved. Inline spans are blanked after fences, so a backtick inside
    a fence cannot pair with one outside it.

    An unpaired backtick can swallow a region and hide a real link; that fails
    toward under-reporting, which is the safe direction for a hygiene check.
    """
    stripped = strip_fenced(text)
    return INLINE_CODE_RE.sub(lambda m: _blank(m.group(0)), stripped)


def heading_anchors(text: str) -> set[str]:
    """Every anchor a heading in ``text`` produces, including duplicates.

    GitHub disambiguates repeated headings with ``-1``, ``-2`` suffixes; we
    accept both the bare slug and the suffixed forms. Inline code is *not*
    stripped here — ``slugify`` drops the backticks but keeps the words, which
    is what GitHub does too.
    """
    body = strip_fenced(text)
    anchors: set[str] = set()
    seen: dict[str, int] = {}
    for match in HEADING_RE.finditer(body):
        slug = slugify(match.group(2))
        if not slug:
            continue
        anchors.add(slug)
        count = seen.get(slug, 0)
        seen[slug] = count + 1
        if count:
            anchors.add(f"{slug}-{count}")
    return anchors


def markdown_files(roots: tuple[str, ...], files: tuple[str, ...]) -> list[Path]:
    found: list[Path] = []
    for root in roots:
        path = Path(root)
        if path.is_dir():
            found.extend(sorted(path.rglob("*.md")))
        elif path.is_file():
            found.append(path)
    for name in files:
        path = Path(name)
        if path.is_file():
            found.append(path)
    # de-duplicate while preserving order
    unique: dict[Path, None] = {}
    for path in found:
        unique.setdefault(path.resolve(), None)
    return sorted(unique)


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def check_no_link_convention() -> list[str]:
    """Fail if a file in ``NO_LINK_FILES`` gains a markdown link.

    Run regardless of the roots given, because the convention is a property of
    one named file rather than of whatever subtree is being checked — narrowing
    the scope must not turn the check off.

    Code regions are stripped first, so a backticked link *shape* used to write
    about a link is not mistaken for a link. That escape is the documented way to
    mention one, and is why a raw substring count over the file gives false
    failures.

    A listed file that has gone missing is itself reported: the convention would
    otherwise be silently dropped by a rename.
    """
    failures: list[str] = []
    for name in NO_LINK_FILES:
        path = REPO / name
        if not path.is_file():
            failures.append(f"{name}: listed in NO_LINK_FILES but not found — renamed?")
            continue
        raw = path.read_text(encoding="utf-8")
        for match in LINK_RE.finditer(strip_code_regions(raw)):
            failures.append(
                f"{name}:{line_of(raw, match.start())}: markdown link in a file that "
                f"must carry none: {match.group(0)[:60]}"
            )
    return failures


def check(paths: list[Path], quiet: bool) -> tuple[int, int, list[str]]:
    """Return (links_checked, failures, messages)."""
    anchor_cache: dict[Path, set[str]] = {}

    def anchors_for(path: Path) -> set[str]:
        if path not in anchor_cache:
            try:
                anchor_cache[path] = heading_anchors(path.read_text(encoding="utf-8"))
            except OSError:
                anchor_cache[path] = set()
        return anchor_cache[path]

    checked = 0
    failures: list[str] = []

    for path in paths:
        raw = path.read_text(encoding="utf-8")
        body = strip_code_regions(raw)
        for match in LINK_RE.finditer(body):
            target = match.group(1).strip()
            if target.startswith(SKIP_PREFIXES):
                continue
            checked += 1
            where = f"{path}:{line_of(raw, match.start())}"

            if target.startswith("#"):
                anchor = target[1:]
                if anchor and anchor not in anchors_for(path):
                    failures.append(f"{where}: in-page anchor not found: {target}")
                continue

            file_part, _, anchor = target.partition("#")
            # strip any title text that survived, and angle-bracket form
            file_part = file_part.strip("<>")
            if not file_part:
                continue
            resolved = (path.parent / file_part).resolve()
            if not resolved.exists():
                failures.append(f"{where}: target does not exist: {target}")
                continue
            if anchor and resolved.is_file() and resolved.suffix == ".md":
                if anchor not in anchors_for(resolved):
                    failures.append(
                        f"{where}: anchor not found in {file_part}: #{anchor}"
                    )

    if failures:
        failures.sort()
        return checked, len(failures), failures
    if not quiet:
        print(f"ok — {checked} relative links resolve across {len(paths)} files")
    return checked, 0, []


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "roots",
        nargs="*",
        help="directories or files to check (default: docs, .agents/knowledge, "
        "AGENTS.md)",
    )
    parser.add_argument(
        "--quiet", action="store_true", help="only print failures"
    )
    parser.add_argument(
        "--conventions-only",
        action="store_true",
        help="check only the NO_LINK_FILES convention, skip link resolution",
    )
    args = parser.parse_args()

    # Printed before the link summary so that summary stays the last line — it is
    # what attention.py reports as this check's detail.
    convention_failures = check_no_link_convention()
    for message in convention_failures:
        print(message)
    if not convention_failures and not args.quiet:
        print(f"ok — no markdown links in {len(NO_LINK_FILES)} convention-bound file(s)")
    if args.conventions_only:
        return 1 if convention_failures else 0

    if args.roots:
        roots = tuple(args.roots)
        extra_files: tuple[str, ...] = ()
    else:
        roots = DEFAULT_ROOTS
        extra_files = DEFAULT_FILES

    paths = markdown_files(roots, extra_files)
    if not paths:
        print("no markdown files found", file=sys.stderr)
        return 1

    # Quiet when the convention already failed, so "ok — N links resolve" is not
    # the last line of a run that is exiting 1. attention.py reports a check's
    # detail as its final stdout line, and a FAIL row whose detail says "ok" reads
    # as a contradiction.
    checked, failed, messages = check(paths, args.quiet or bool(convention_failures))
    for message in messages:
        print(message)
    if failed or convention_failures:
        if failed:
            print(
                f"\nFAIL: {failed} of {checked} links broken across {len(paths)} files",
                file=sys.stderr,
            )
        if convention_failures:
            print(
                f"FAIL: {len(convention_failures)} no-link convention violation(s)",
                file=sys.stderr,
            )
        return 1
    if args.quiet:
        print(f"ok — {checked} links across {len(paths)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
