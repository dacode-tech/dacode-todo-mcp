#!/bin/sh
# todo-mcp package postinstall (D29). On an upgrade, SIGTERM every
# daemon running the packaged binary — any user's — so the sessions'
# shims respawn it from the new one and reconnect on their own (§13). dpkg
# passes "configure <old-version>" (empty on a fresh install), rpm the
# installed-instance count (2 or more on an upgrade).
set -e

case "${1:-}" in
    configure) [ -n "${2:-}" ] || exit 0 ;;
    [0-9]*) [ "$1" -ge 2 ] || exit 0 ;;
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
