#!/bin/sh
# todo-mcp package postremove (D29). On a removal — not the old
# package's half of an upgrade — SIGTERM every daemon running the
# packaged binary: its file is gone, so it could never be respawned.
# dpkg passes "remove"/"purge" (and "upgrade" on the upgrade path),
# rpm the remaining-instance count (0 on a removal).
set -e

case "${1:-}" in
    remove | purge | 0) ;;
    *) exit 0 ;;
esac

# /proc, not pkill: minimal images (Fedora/EL bases, Debian slim) ship
# no procps. A daemon is argv[0] /usr/bin/todo-mcp with a --daemon
# token anywhere after it; tr turns the NUL separators into spaces,
# the trailing one included, so the token match is exact.
for cmdline in /proc/[0-9]*/cmdline; do
    args=$(tr '\000' ' ' < "$cmdline" 2> /dev/null) || continue
    case "$args" in
        "/usr/bin/todo-mcp --daemon "* | "/usr/bin/todo-mcp "*" --daemon "*)
            pid=${cmdline#/proc/}
            kill -TERM "${pid%/cmdline}" 2> /dev/null || true ;;
    esac
done
exit 0
