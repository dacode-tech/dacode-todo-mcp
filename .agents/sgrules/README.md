# ast-grep rules

Verified structural-search rules, referenced by `sgconfig.yml` at the
repo root (`ruleDirs`) and passed to the ast-grep MCP server via
`AST_GREP_CONFIG`.

This directory is intentionally small. Before adding a rule:

1. **General Rust lint belongs in clippy**, not here — `#![deny]`/`warn`
   attributes or `clippy.toml`. Clippy has dataflow; ast-grep does not.
2. **Anything needing dataflow cannot be expressed structurally at all**
   ("is this lock held across an await on every path") — do not try.
3. A rule earns its place when it answers a *shape* question reviewers or
   searchers ask repeatedly: `unwrap()` outside test modules, `impl Store
   for` inventory, `HashMap` in types that reach serialization.
4. Verify the rule against a known-positive and a known-negative file
   before committing it, and say in the rule's `note` what it is for.

Until the workspace exists there is nothing to scan; this README is the
whole content, on purpose.
