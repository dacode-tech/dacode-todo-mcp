<!-- Shared CODE SEARCH layer. Inlined into any agent that navigates
     this tree — coding agents and the read-only explore agent alike.
     Kept separate from _rust.md so an agent that only searches does not
     pay for conventions it will never apply. -->

## Code search: pick the layer, cheapest first

Three layers. Escalate; do not start at the top. The commonest mistake is
reaching for the structural layer on a question a `grep` would have
answered in 20 ms.

| Query | Layer | Tool |
|---|---|---|
| "Which crate/module handles X?", conceptual, unfamiliar area | **map** | `docs/todo-mcp-design.md`, `.agents/knowledge/INDEX.md`, crate `AGENTS.md` |
| Exact string, identifier, log message, config key | lexical | `grep` / `rg` |
| Syntactic shape — "every `unwrap()` outside tests", "all `impl Store for`" | structural | `ast-grep` |

Rules of thumb:

- **Narrow scope first, then search cheap.** For a conceptual question,
  read the design doc / index to find the crate, then grep inside it. Do
  not run a repo-wide semantic query to answer "where do dependencies
  live" — that is the expensive layer doing the cheap layer's job.
- **Start with grep** for anything with a literal in it. Most questions
  are, and end there.
- **Escalate to ast-grep when the query is about shape**, not text — when
  you would otherwise write a regex with nested brackets and give up.
- **The semantic layer is gone** (the user's 2026-10-14 call — serena
  never featured in delivered work): "who calls this", trait impls and
  safe renames have no compiler-certain answer here anymore. grep for
  the literal, ast-grep for the shape, and a rename goes grep → edit →
  review, with the reviewer as the correctness check.
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

### ast-grep's reach

ast-grep is tree-sitter based: it parses **both** branches of every
`cfg`, sees macro *call sites* as written, needs no build and no
index, and runs sub-second. That is its edge over any resolver — code
across configurations and inside unexpanded macro input is visible to
it — and its limit: it matches shape, never semantics.

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

### The semantic layer is gone

Serena was removed from the harness on 2026-10-14 (the user's call: it
never featured in delivered work, and its ~20 tool schemas sat in every
session's cached prefix). **ast-grep for structure and grep for text**
answer most questions — without the compiler's certainty about trait
resolution, generics and macro expansion.

Do not present an ast-grep result as if it were a resolved reference.
Say which layer answered.
