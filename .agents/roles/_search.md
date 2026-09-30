<!-- Shared CODE SEARCH layer. Inlined into any agent that navigates
     this tree — coding agents and the read-only explore agent alike.
     Kept separate from _rust.md so an agent that only searches does not
     pay for conventions it will never apply. -->

## Code search: pick the layer, cheapest first

Four layers. Escalate; do not start at the top. The commonest mistake is
reaching for the semantic layer on a question a `grep` would have
answered in 20 ms.

| Query | Layer | Tool |
|---|---|---|
| "Which crate/module handles X?", conceptual, unfamiliar area | **map** | `docs/todo-mcp-design.md`, `.agents/knowledge/INDEX.md`, crate `AGENTS.md` |
| Exact string, identifier, log message, config key | lexical | `grep` / `rg` |
| Syntactic shape — "every `unwrap()` outside tests", "all `impl Store for`" | structural | `ast-grep` |
| "Who calls this", "where is this defined", trait impls, rename, references | semantic | Serena (rust-analyzer) |

Rules of thumb:

- **Narrow scope first, then search cheap.** For a conceptual question,
  read the design doc / index to find the crate, then grep inside it. Do
  not run a repo-wide semantic query to answer "where do dependencies
  live" — that is the expensive layer doing the cheap layer's job.
- **Start with grep** for anything with a literal in it. Most questions
  are, and end there.
- **Escalate to ast-grep when the query is about shape**, not text — when
  you would otherwise write a regex with nested brackets and give up.
- **Use Serena when you need the compiler's answer**: which trait impl,
  which definition, all references, safe rename. It is the only layer
  that is correct about Rust semantics (generics, macro expansion,
  auto-deref), and the slowest per query.
- Zero results means escalate a layer, not rephrase at the same one.

### The map layer replaces a generated repo-map

`docs/todo-mcp-design.md` describes every crate's job and boundaries
(§2), `INDEX.md` lists the crate docs with a one-line scope, and each
crate `AGENTS.md` describes its own architecture. Hand-written intent
beats inferred centrality. While the workspace does not exist yet — the
design phase, until M0 — the spec *is* the answer to most questions;
cite it by section.

`ast-grep outline --json <path>` gives a cheap structural outline of a
file or directory when you need symbols rather than prose.

### Using grep well

Output discipline matters more than the search itself — an unbounded
`grep` can dump more tokens than the answer is worth.

- `rg -l <pattern> <scope>` first to get the **file set**, then a
  targeted search inside the few that matter.
- Always pass a path scope (`crates/todo-domain`). Repo-wide is rarely
  what you meant, and `target/` and `.agents/cache/` are noise — exclude
  or scope around them.
- `-C 2` is fine on a handful of matches and inflates output several-fold
  on hundreds. Add it deliberately, not by habit.
- Cap wide searches (`| head -50`) rather than truncating after the fact.
- Report `path:line`. Never paste a wall of matches into the
  conversation; the caller can open what it needs.

### ast-grep sees what Serena cannot

Serena/rust-analyzer resolves against **one** feature configuration, so
anything behind a disabled `#[cfg(feature = ...)]` or a non-host
`cfg(target_os)` is invisible to it. ast-grep is tree-sitter based: it
parses **both** branches of every `cfg`, sees macro *call sites* as
written, needs no build and no index, and runs sub-second.

So: use ast-grep to find code across configurations and inside unexpanded
macro input, Serena to reason correctly about the active one — including
what macros expand to.

Inline patterns can silently under-match when the shape is more complex
than the pattern (generics, lifetime parameters, attribute macros). **If a
pattern returns suspiciously few hits, dump the syntax tree before
believing it.**

`.agents/sgrules/` holds verified rules; check there before authoring a
pattern. It is intentionally small — general Rust lint belongs in
`clippy` (with `#![deny]`/`warn` attributes or `clippy.toml`), and
anything needing dataflow ("is this lock held across an await on every
path") cannot be expressed structurally at all. See that directory's
README before adding one.

### When Serena is unavailable

Before the Cargo workspace exists (and until rust-analyzer has finished
its first index), Serena returns nothing useful. The same applies in CI.
Fall back to **ast-grep for structure and grep for text** — between them
they answer most questions, just without the compiler's certainty about
trait resolution, generics and macro expansion.

Do not present an ast-grep result as if it were a resolved reference.
Say which layer answered.

### Serena specifics

**Always scope symbol searches** with `relative_path` (e.g.
`crates/todo-domain`). Unscoped `find_symbol` over a large workspace can
exceed the tool timeout.

Serena reports `body_location.start_line` **0-based**; grep and editors
are 1-based. Add one when quoting `path:line`.

Methods hang off the `impl` node, not the type: the absolute name path
`/Graph/raise_counters_to` resolves to nothing, while the bare
`raise_counters_to` scoped to the file resolves as
`impl Graph/raise_counters_to`. When an absolute name path returns empty,
run `get_symbols_overview` on the file before concluding the symbol is
missing.

`search_for_pattern`, `replace_content` and `replace_in_files` are
disabled — use the harness's own grep and edit tools. Serena ships an
`initial_instructions` tool that asks to be called first in the strongest
terms; the rule here, decided rather than inherited: **read it once per
session if you are going to edit through Serena**
(`insert_after_symbol`, `replace_symbol_body`, `rename_symbol`, ...), and
**skip it for a lookup-only task** (`find_symbol`, `find_declaration`,
`find_referencing_symbols`, `get_symbols_overview`). If a Serena tool
behaves unexpectedly, read the manual before concluding the tool is
broken, and say which layer answered.
