//! The hand-written usage text (U74's split out of `args`): the one
//! text `--help` prints to stdout and every argument fault appends on
//! stderr — a fault always carries the way to use the tool correctly.
//! One text, one exit-code table (D5): the server paths' clauses and
//! the inspector's, 0/1/2/3.
//!
//! The text is the operator contract the parse serves, pinned clause
//! by clause below — each subcommand's line, its flags' entries, its
//! exit-code clauses — so a clause that drifts from the behavior it
//! documents fails a test rather than shipping.

/// The hand-written usage text every argument fault appends — a fault
/// always carries the way to use the tool correctly, on stderr where
/// the fault belongs — and `--help` prints to stdout. One text, one
/// exit-code table (D5): the server paths' clauses and the
/// inspector's, 0/1/2/3.
pub const USAGE: &str = "\
todo-mcp — the dacode-todo MCP server and tool CLI (spec §13).

Usage:
  todo-mcp --root <DIR> [--repo <SLUG>]                    the stdio shim: connects to (spawning)
                                                           the root's daemon and proxies MCP (D25)
  todo-mcp --root <DIR> --daemon                           the shared daemon (D25): one process
                                                           per root, every session over
                                                           <root>/serve.sock
  todo-mcp --root <DIR> [--repo <SLUG>] progress [flags]   the total-progress render
  todo-mcp --root <DIR> [--repo <SLUG>] refresh            re-import the repo's board
  todo-mcp --root <DIR> serve [--port-file <PATH>] [--open]
                                                           the live progress listener
  todo-mcp --root <DIR> [--repo <SLUG>] dump               print the store's graph as one canonical JSON document
  todo-mcp --root <DIR> [--repo <SLUG>] verify             report the store's integrity
  todo-mcp --root <DIR> [--repo <SLUG>] repair [--dry-run] [--yes]
                                                           show a torn last line and truncate it on confirmation (§4.3)
  todo-mcp --help | --version                              (and -h/--help per subcommand)

Options:
      --root <DIR>    The data root holding repos/<slug>/ and the shared
                      registry (required on every path; D4's default
                      location is ~/.local/share/dacode-todo/). A leading
                      '~' or '~/' expands against HOME — the spelling for
                      client configs that spawn without a shell; '~name'
                      (another user's home) and a '~' root with HOME unset
                      or empty are usage faults
      --repo <SLUG>   The repository: the shim's default slug (D8),
                      riding the socket preamble to its session and
                      applied whenever a tool's repo parameter is
                      absent, or the repo the progress subcommand
                      renders — an explicit value always wins over the
                      working-directory match (§7). Cannot combine
                      with serve: the listener serves every repo in
                      the root, the repo is a URL parameter. Cannot
                      combine with --daemon either: the daemon serves
                      every connection's own default via the socket
                      preamble
      --daemon        The shared daemon (D25): one process per root
                      serving every connection's own session over
                      <root>/serve.sock — the no-argument shim spawns
                      it. SIGTERM is the graceful drain (stop
                      accepting, wait for in-flight work within the
                      drain bound, exit 0); SIGINT keeps the default
                      disposition. Cannot combine with a subcommand
                      or --repo
      --json          With `progress`: print the JSON document to stdout
                      instead of the summary; writes nothing
      --out <DIR>     With `progress`: write progress.html and
                      progress.json there instead of the rendered
                      repo's .agents/cache
      --no-write      With `progress`: print the summary only, write
                      nothing
      --open          With `progress`: open the written page in a
                      browser; with `serve`: open the served page
      --port-file <PATH>
                      With `serve`: where the listener's port is
                      published (default <root>/serve.port, the same
                      publication the daemon's live view ensures) —
                      a second invocation attaches to a live listener
                      instead of starting another
      --dry-run       With `repair`: classify and report, write nothing
      --yes           With `repair`: truncate without prompting
  -h, --help          Print this help and exit 0
  -V, --version       Print the version and exit 0

With `refresh`: no flags of its own — the repo resolves through the
      same --repo/cwd tier, the board's location derives from the
      repo's registered checkout, and the dialect is the slug.

With `dump`/`verify`/`repair`: the repo resolves through the same
      --repo/cwd tier as every subcommand (§13). dump writes canonical
      bytes plus one trailing LF to stdout and nothing else, so
      `todo-mcp dump … > graph.json` and `| jq .` both work; a fault's
      machine-readable identity is §6.1's code, printed on its own
      line; reports are otherwise human-readable text on stdout and
      faults on stderr, deterministic for a given store state.

Exit codes:
  0  the client closed the connection after a completed session —
      proxied through the daemon, or
      served degraded after the retry window —
      the listener shut down cleanly on the interrupt (or another
      invocation attached to it), the daemon drained on SIGTERM
      within its bound (or found one already serving — one line, no
      second listener), or the subcommand succeeded
  1  the daemon connection was lost mid-session (or
      ended before a session established), the serve loop failed,
      the listener did not start (a root with no registry),
      the daemon's socket could not be ensured (the ensure's
      lock or bind refused), the render failed (a repo that does
      not resolve, or a store that will not read), the page did
      not write, or the refresh refused (the board is gone or
      broken, a live session holds the store, or the import
      failed); verify found a fault it will not act on; repair
      refused (unrepairable, diverged) or aborted at the prompt
  2  usage: a bad flag, a missing --root, a malformed --root or --repo
      value, an unknown subcommand, or flags that cannot combine
  3  the store refused the operation: any store error — io, locked,
      version_mismatch, a corrupt store document
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_usage_text_names_the_refresh_subcommand() {
        // The operator-facing contract: the usage every fault prints
        // carries the refresh line and its exit-1 clause.
        assert!(USAGE.contains("refresh            re-import the repo's board"));
        assert!(USAGE.contains("or the refresh refused"));
    }

    #[test]
    fn the_usage_text_names_the_daemon() {
        // The operator-facing contract: the usage every fault prints
        // carries the daemon line, its option entry (the drain and
        // SIGINT's disposition), and its exit-code clauses.
        assert!(USAGE.contains("todo-mcp --root <DIR> --daemon"));
        assert!(USAGE.contains("the shared daemon (D25)"));
        assert!(USAGE.contains("<root>/serve.sock"));
        assert!(USAGE.contains("SIGINT keeps the default"));
        assert!(USAGE.contains("the daemon drained on SIGTERM"));
        assert!(USAGE.contains("the daemon's socket could not be ensured"));
    }

    #[test]
    fn the_usage_text_names_the_serve_subcommand() {
        // The operator-facing contract: the usage every fault prints
        // carries the serve line, its flags and its exit-code clauses.
        assert!(USAGE.contains("serve [--port-file <PATH>] [--open]"));
        assert!(USAGE.contains("--port-file <PATH>"));
        assert!(USAGE.contains("the listener shut down cleanly on the interrupt"));
        assert!(USAGE.contains("the listener did not start"));
        // D13/D7: the port-file default is the per-root publication.
        assert!(USAGE.contains("(default <root>/serve.port"));
    }

    #[test]
    fn the_usage_text_names_the_shim() {
        // The operator-facing contract: the usage every fault prints
        // carries the no-subcommand line (the shim, D25) and the
        // shim's exit-code clauses.
        assert!(USAGE.contains("the stdio shim: connects to (spawning)"));
        assert!(USAGE.contains("the root's daemon and proxies MCP (D25)"));
        assert!(USAGE.contains("after a completed session"));
        assert!(USAGE.contains("served degraded after the retry window"));
        assert!(USAGE.contains("the daemon connection was lost mid-session"));
        assert!(USAGE.contains("ended before a session established"));
    }

    #[test]
    fn the_usage_text_names_the_inspector_subcommands() {
        // The operator-facing contract: the usage every fault prints
        // carries the three subcommands, repair's switches, the
        // help/version lines and the exit-3 clause (D5's one table).
        assert!(USAGE.contains("dump               print the store's graph"));
        assert!(USAGE.contains("verify             report the store's integrity"));
        assert!(USAGE.contains("repair [--dry-run] [--yes]"));
        assert!(USAGE.contains("      --dry-run       With `repair`"));
        assert!(USAGE.contains("      --yes           With `repair`"));
        assert!(USAGE.contains("  -h, --help          Print this help and exit 0"));
        assert!(USAGE.contains("  -V, --version       Print the version and exit 0"));
        assert!(
            USAGE.contains("3  the store refused the operation: any store error — io, locked,")
        );
    }

    #[test]
    fn the_usage_text_names_the_tilde_expansion() {
        // The operator-facing contract: the --root option's entry
        // carries the spelling rule, and the exit-2 enumeration
        // carries its faults.
        assert!(USAGE.contains("expands against HOME"));
        assert!(USAGE.contains("'~name'"));
        assert!(USAGE.contains("HOME unset"));
        assert!(USAGE.contains("a malformed --root or --repo"));
    }
}
