#!/bin/sh
# Render the Homebrew formula for one release from its SHA256SUMS (D27).
#
#   tools/brew-formula.sh <version> <SHA256SUMS> [owner/repo] > todo-mcp.rb
#
# The release workflow runs this and pushes the result to the tap's
# Formula/todo-mcp.rb. Fails if any shipped tarball is missing a sum.
set -eu

[ $# -ge 2 ] || { echo "usage: $0 <version> <SHA256SUMS> [owner/repo]" >&2; exit 2; }
version=$1
sums=$2
repo=${3:-dacode-tech/dacode-todo-mcp}
tag="v$version"
base="https://github.com/$repo/releases/download/$tag"

sum_of() {
    file="todo-mcp-$tag-$1.tar.gz"
    sum=$(awk -v f="$file" '$2 == f || $2 == "*" f { print $1; exit }' "$sums")
    [ -n "$sum" ] || { echo "$0: no sum for $file in $sums" >&2; exit 1; }
    printf '%s' "$sum"
}

mac_arm=$(sum_of aarch64-apple-darwin)
linux_x86=$(sum_of x86_64-unknown-linux-musl)
linux_arm=$(sum_of aarch64-unknown-linux-musl)

cat <<EOF
# Rendered by tools/brew-formula.sh in $repo's release workflow — edits
# here are overwritten by the next release.
class TodoMcp < Formula
  desc "Deterministic task-graph MCP server for markdown-style task boards"
  homepage "https://github.com/$repo"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    depends_on arch: :arm64

    on_arm do
      url "$base/todo-mcp-$tag-aarch64-apple-darwin.tar.gz"
      sha256 "$mac_arm"
    end
  end

  on_linux do
    on_intel do
      url "$base/todo-mcp-$tag-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$linux_x86"
    end

    on_arm do
      url "$base/todo-mcp-$tag-aarch64-unknown-linux-musl.tar.gz"
      sha256 "$linux_arm"
    end
  end

  def install
    bin.install "todo-mcp"
  end

  def caveats
    <<~EOS
      Wire the server into opencode: add this under "mcp" in
      ~/.config/opencode/opencode.json (or .jsonc):

        "todo-mcp": {
          "type": "local",
          "command": ["todo-mcp", "--root", "~/.local/share/dacode-todo"],
          "enabled": true
        }

      After an upgrade, restart the per-root daemon so it runs the new
      binary. On Linux, close the open sessions FIRST: a session
      started before the upgrade respawns the daemon from the old keg,
      which the upgrade removed. On macOS open sessions reconnect on
      their own.

        pkill -TERM -f 'todo-mcp .*--daemon'
    EOS
  end

  test do
    assert_equal "todo-mcp #{version}", shell_output("#{bin}/todo-mcp --version").strip
  end
end
EOF
