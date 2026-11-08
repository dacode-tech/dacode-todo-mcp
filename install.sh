#!/bin/sh
# todo-mcp installer (D27): download a release, verify its checksum,
# install the binary by rename, restart a running daemon, and print the
# opencode entry. It never writes an opencode config.
#
#   curl -fsSL https://github.com/dacode-tech/dacode-todo-mcp/releases/latest/download/install.sh | sh
#
# Environment:
#   TODO_MCP_VERSION      tag to install, e.g. v0.1.0 (default: the latest release)
#   TODO_MCP_INSTALL_DIR  where the binary goes (default: ~/.local/bin)
#   TODO_MCP_ROOT         store root shown in the printed entry (default: ~/.local/share/dacode-todo)
#   TODO_MCP_NO_RESTART   set to 1 to leave a running daemon alone
#   TODO_MCP_BASE_URL     download base overriding GitHub (mirrors, testing)
set -eu

REPO=dacode-tech/dacode-todo-mcp

say() { printf '%s\n' "$*"; }
die() { printf 'install.sh: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" > /dev/null 2>&1; }

[ -n "${HOME:-}" ] || die "HOME is not set"

# ── platform ────────────────────────────────────────────────────────────
os=$(uname -s)
arch=$(uname -m)
case "$os" in
    Linux)
        case "$arch" in
            x86_64 | amd64) target=x86_64-unknown-linux-musl ;;
            aarch64 | arm64) target=aarch64-unknown-linux-musl ;;
            *) die "no prebuilt binary for Linux $arch" ;;
        esac ;;
    Darwin)
        # A Rosetta shell reports x86_64 on Apple Silicon.
        if [ "$arch" = arm64 ] || [ "$(sysctl -n hw.optional.arm64 2> /dev/null || true)" = 1 ]; then
            target=aarch64-apple-darwin
        else
            die "no prebuilt binary for Intel Macs; build from source: cargo install --locked --git https://github.com/$REPO todo-mcp"
        fi ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        die "native Windows is not supported yet (decision D28); run this script inside WSL2" ;;
    *) die "unsupported OS: $os" ;;
esac

# ── download tooling ────────────────────────────────────────────────────
if have curl; then
    fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }
elif have wget; then
    fetch() { wget -q -O "$2" "$1"; }
else
    die "need curl or wget"
fi
if have sha256sum; then
    sha256() { sha256sum "$1" | awk '{ print $1 }'; }
elif have shasum; then
    sha256() { shasum -a 256 "$1" | awk '{ print $1 }'; }
else
    die "need sha256sum or shasum"
fi
have tar || die "need tar"

if [ -n "${TODO_MCP_BASE_URL:-}" ]; then
    base=$TODO_MCP_BASE_URL
elif [ -n "${TODO_MCP_VERSION:-}" ]; then
    tag=${TODO_MCP_VERSION#v}
    base="https://github.com/$REPO/releases/download/v$tag"
else
    base="https://github.com/$REPO/releases/latest/download"
fi

tmp=$(mktemp -d 2> /dev/null || mktemp -d -t todo-mcp)
trap 'rm -rf "$tmp"' EXIT
trap 'exit 130' INT TERM

# SHA256SUMS carries an unversioned name, so it also resolves "latest":
# the line for this target names the versioned tarball.
fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS" || die "cannot download $base/SHA256SUMS"
line=$(awk -v t="-$target.tar.gz" 'substr($2, length($2) - length(t) + 1) == t { print; exit }' "$tmp/SHA256SUMS")
[ -n "$line" ] || die "the release has no tarball for $target"
expected=${line%% *}
file=${line##* }
file=${file#\*}

say "Downloading $file"
fetch "$base/$file" "$tmp/$file" || die "cannot download $base/$file"
actual=$(sha256 "$tmp/$file")
[ "$actual" = "$expected" ] || die "checksum mismatch for $file: expected $expected, got $actual"

tar -xzf "$tmp/$file" -C "$tmp"
new="$tmp/${file%.tar.gz}/todo-mcp"
[ -f "$new" ] || die "the tarball has no todo-mcp binary"

# ── install by rename (§13): never overwrite a running binary in place ──
dir=${TODO_MCP_INSTALL_DIR:-$HOME/.local/bin}
mkdir -p "$dir"
dir=$(cd "$dir" && pwd -L)
dest="$dir/todo-mcp"
replacing=
old_version=
if [ -e "$dest" ]; then
    replacing=1
    old_version=$("$dest" --version 2> /dev/null || true)
fi
# Staged beside the destination, so the smoke run below also works
# when the temp dir is mounted noexec, and the mv is a same-fs rename.
staged="$dir/.todo-mcp.new.$$"
trap 'rm -rf "$tmp"; rm -f "$staged"' EXIT
cp "$new" "$staged"
chmod 755 "$staged"
new_version=$("$staged" --version) || die "the downloaded binary does not run on this host"
mv -f "$staged" "$dest"

if [ -n "$old_version" ]; then
    say "Updated $dest: $old_version -> $new_version"
else
    say "Installed $dest: $new_version"
fi

# ── restart the daemon so it runs the new inode; shims reconnect alone ──
# Only daemons launched from this install path: the daemon's argv[0] is
# the path its shim resolved at startup — the logical path on macOS
# (a symlink on PATH stays the symlink), the resolved one on Linux.
ere_escape() { printf '%s' "$1" | sed 's/[][\\.^$*+?(){}|]/\\&/g'; }
if [ -n "$replacing" ] && [ "${TODO_MCP_NO_RESTART:-0}" != 1 ] && have pkill; then
    candidates="$dest
$(cd "$dir" && pwd -P)/todo-mcp"
    on_path=$(command -v todo-mcp 2> /dev/null || true)
    if [ -n "$on_path" ] && [ "$on_path" -ef "$dest" ]; then
        candidates="$candidates
$on_path"
    fi
    signalled=
    seen=
    while IFS= read -r exe; do
        case "$seen" in *"|$exe|"*) continue ;; esac
        seen="$seen|$exe|"
        if pkill -TERM -u "$(id -u)" -f "^$(ere_escape "$exe") --root .*--daemon" 2> /dev/null; then
            signalled=1
        fi
    done << EOF
$candidates
EOF
    if [ -n "$signalled" ]; then
        say "Signalled the running todo-mcp daemon; open sessions reconnect to the new binary."
    fi
fi

# ── PATH ────────────────────────────────────────────────────────────────
command_word=todo-mcp
case ":$PATH:" in
    *":$dir:"*)
        found=$(command -v todo-mcp 2> /dev/null || true)
        if [ -n "$found" ] && [ "$found" != "$dest" ]; then
            say ""
            say "warning: '$found' shadows $dest on your PATH."
        fi ;;
    *)
        command_word=$dest
        say ""
        say "$dir is not on your PATH. Add it to your shell profile:"
        say "  export PATH=\"$dir:\$PATH\""
        say "(the entries below use the absolute path until then)" ;;
esac

# ── opencode: detect and print, never write ─────────────────────────────
# A literal "~": the server expands it against HOME, so the printed
# entry carries no user-specific path and works in a committed config.
# shellcheck disable=SC2088
default_root='~/.local/share/dacode-todo'
root=${TODO_MCP_ROOT:-$default_root}
config_dir=${XDG_CONFIG_HOME:-$HOME/.config}/opencode
configured=
for f in "$config_dir/opencode.jsonc" "$config_dir/opencode.json" "$config_dir/config.json"; do
    if [ -f "$f" ] && grep -q '"todo-mcp"' "$f"; then
        configured=$f
        break
    fi
done

say ""
if [ -n "$configured" ]; then
    say "opencode: $configured already has a \"todo-mcp\" entry; nothing to add."
    say "Check that its command names $command_word."
else
    have opencode || say "opencode was not found on PATH; the entries below are for when it is installed."
    say "opencode: add ONE of these entries (a project entry shadows the global one)."
    say ""
    say "Global, under \"mcp\" in $config_dir/opencode.json (or .jsonc):"
    say ""
    cat << EOF
  "todo-mcp": {
    "type": "local",
    "command": ["$command_word", "--root", "$root"],
    "enabled": true
  }
EOF
    say ""
    say "Per repository, in <repo>/opencode.json (--repo pins the default slug;"
    say "without it the server picks the registered repo containing the cwd):"
    say ""
    cat << EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "mcp": {
    "todo-mcp": {
      "type": "local",
      "command": ["$command_word", "--root", "$root", "--repo", "<slug>"],
      "enabled": true
    }
  }
}
EOF
fi
