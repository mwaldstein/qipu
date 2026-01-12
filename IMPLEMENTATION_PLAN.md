# Qipu Implementation Plan

Status: Initial draft  
Last updated: 2026-01-12

This document tracks implementation progress against the specs in `specs/`.

---

## Phase 0: Project Bootstrap (P0 - Critical Path)

### Technology Decisions (UNRESOLVED)
- [ ] **Decide implementation language** - Specs do not mandate a language. Consider:
  - TypeScript/Node: fast iteration, good CLI tooling (commander, yargs), easy markdown/YAML parsing
  - Rust: performance, single binary distribution, beads precedent (if beads is Rust)
  - Go: single binary, good CLI ergonomics, fast compile times
- [ ] **Decide package manager / build tooling**
- [ ] **Set up project structure**: `src/`, `src/lib/`, tests, CI

### Initial Structure
- [ ] Create `src/` directory structure
- [ ] Create `src/lib/` for shared utilities
- [ ] Set up test framework and initial test harness
- [ ] Set up CI/linting/formatting

---

## Phase 1: Foundation (P1 - Required for Any Command)

All commands depend on these foundational layers.

### Storage Layer (`specs/storage-format.md`)
- [ ] **Store discovery** - walk up from cwd to find `.qipu/` (or use `--store`)
- [ ] **Store initialization** - create `.qipu/`, `notes/`, `mocs/`, `config.toml`
- [ ] **Config parsing** - read/write `.qipu/config.toml`
- [ ] **Note file parsing** - YAML frontmatter + markdown body
- [ ] **Note file writing** - deterministic serialization (stable key order, newline handling)
- [ ] **ID generation** - `qp-<hash>` with adaptive length
- [ ] **Slug generation** - `<id>-<slug(title)>.md` filename convention

### Knowledge Model (`specs/knowledge-model.md`)
- [ ] **Note struct** - id, title, type, tags, created, updated, sources, links, body
- [ ] **Note types** - fleeting, literature, permanent, moc
- [ ] **Tag model** - list of strings, validation
- [ ] **Typed links model** - related, derived-from, supports, contradicts, part-of
- [ ] **Source model** - url, title, accessed

### CLI Runtime (`specs/cli-tool.md`)
- [ ] **Argument parsing** - global flags (`--store`, `--root`, `--json`, `--token`, `--quiet`, `--verbose`)
- [ ] **Command dispatch** - route to subcommand handlers
- [ ] **Exit codes** - 0 success, 1 failure, 2 usage error, 3 data error
- [ ] **Error formatting** - human vs `--json` error output
- [ ] **Output determinism** - ensure stable ordering across all outputs

---

## Phase 2: Core Commands (P2 - MVP Feature Set)

Implements the essential commands for basic note capture and retrieval.

### `qipu init` (`specs/cli-interface.md`)
- [ ] Create store at default or specified location
- [ ] `--stealth` mode (gitignore the store)
- [ ] `--visible` mode (use `qipu/` instead of `.qipu/`)
- [ ] Idempotent (safe to run multiple times)

### `qipu create` / `qipu new` (`specs/cli-interface.md`)
- [ ] Generate new note with ID, slug, frontmatter
- [ ] `--type` flag (fleeting, literature, permanent, moc)
- [ ] `--tag` flag (repeatable)
- [ ] `--open` flag (open in `$EDITOR`)
- [ ] Print note ID/path on success

### `qipu capture` (`specs/cli-interface.md`)
- [ ] Create note from stdin
- [ ] `--title` flag
- [ ] `--type` flag
- [ ] `--tag` flag

### `qipu show <id-or-path>` (`specs/cli-interface.md`)
- [ ] Resolve ID or path to note file
- [ ] Print note content to stdout
- [ ] `--json` output format

### `qipu list` (`specs/cli-interface.md`)
- [ ] List all notes
- [ ] `--tag` filter
- [ ] `--type` filter
- [ ] `--since` filter
- [ ] `--json` output format
- [ ] Deterministic ordering (by created_at, id)

---

## Phase 3: Indexing & Navigation (P3 - Enables Search/Graph)

### Indexing (`specs/indexing-search.md`)
- [ ] **Metadata index** - id -> {title, type, tags, path, created, updated}
- [ ] **Tag index** - tag -> [ids...]
- [ ] **Link extraction** - parse wiki links `[[id]]` and markdown links from body
- [ ] **Backlink index** - id -> [ids that link to it]
- [ ] **Graph adjacency list** - inline + typed links
- [ ] **Incremental indexing** - track mtimes, re-parse only changed notes
- [ ] **Cache storage** - `.qipu/.cache/*.json` (gitignored)
- [ ] `qipu index` command
- [ ] `qipu index --rebuild` command

### Search (`specs/indexing-search.md`)
- [ ] `qipu search <query>` - full-text search in title + body
- [ ] `--tag` filter
- [ ] `--type` filter
- [ ] `--json` output
- [ ] Result ranking (title > body, exact tag > text, recency boost)

### `qipu inbox` (`specs/cli-interface.md`)
- [ ] List unprocessed notes (type in {fleeting, literature})
- [ ] Option to exclude notes already in a MOC

---

## Phase 4: Link Management & Graph Traversal (P4)

### Link Commands (`specs/cli-interface.md`, `specs/graph-traversal.md`)
- [ ] `qipu link add <from> <to> --type <t>` - add typed link to frontmatter
- [ ] `qipu link remove <from> <to> --type <t>` - remove typed link
- [ ] `qipu link list <id>` - list links for a note
  - [ ] `--direction <out|in|both>`
  - [ ] `--typed-only` / `--inline-only`
  - [ ] `--type <t>` filter
  - [ ] `--json` output

### Graph Traversal (`specs/graph-traversal.md`)
- [ ] `qipu link tree <id>` - traversal tree from note
  - [ ] `--direction <out|in|both>` (default: both)
  - [ ] `--max-depth <n>` (default: 3)
  - [ ] `--type <t>` filter (repeatable)
  - [ ] `--exclude-type <t>` filter
  - [ ] `--typed-only` / `--inline-only`
  - [ ] `--max-nodes <n>` (optional)
  - [ ] Cycle detection (mark visited nodes as "(seen)")
  - [ ] Deterministic BFS ordering (sort by edge type, then target id)
  - [ ] `--json` output (nodes[], edges[], spanning_tree[])
  - [ ] Truncation reporting when limits hit
- [ ] `qipu link path <from> <to>` - find path between notes
  - [ ] `--direction`, `--max-depth`, `--typed-only`, `--inline-only` flags
  - [ ] `--json` output

---

## Phase 5: LLM Integration (P5 - Core Value Prop)

### Token-Optimized Output (`specs/token-optimized-output.md`)
- [ ] `--token` output format implementation
- [ ] Record types: H (header), N (note), S (summary), E (edge), B (body)
- [ ] Format versioning in header line (`token=1`)
- [ ] Summary extraction logic:
  - [ ] 1. Frontmatter `summary` field
  - [ ] 2. `## Summary` section first paragraph
  - [ ] 3. First paragraph of body
  - [ ] 4. Empty
- [ ] Token estimation: `ceil(chars / 4)`
- [ ] Budget enforcement (`--max-chars`, `--max-tokens`)
- [ ] Truncation handling (set `truncated=true`, no partial records)

### `qipu prime` (`specs/llm-context.md`)
- [ ] Emit bounded session primer (~1-2k tokens)
- [ ] Include: qipu explanation, command reference, store location
- [ ] Include: top MOCs and/or recently updated notes
- [ ] Deterministic output
- [ ] `--json` output
- [ ] `--token` output

### `qipu context` (`specs/llm-context.md`)
- [ ] Bundle selection:
  - [ ] `--note <id>` (repeatable)
  - [ ] `--tag <tag>`
  - [ ] `--moc <id>` (include MOC + linked notes)
  - [ ] `--query <text>` (search-based selection)
- [ ] Budgeting: `--max-chars`, `--max-tokens`
- [ ] Output formats:
  - [ ] Default: markdown bundle with metadata headers
  - [ ] `--json` output
  - [ ] `--token` output (summaries-first, `--with-body` for full content)
- [ ] Deterministic ordering
- [ ] Safety banner (optional)

---

## Phase 6: Export (P6)

### Export Commands (`specs/export.md`)
- [ ] `qipu export` command
- [ ] Bundle export (concatenate notes)
- [ ] Outline export (MOC-driven ordering)
- [ ] Bibliography export (extract sources)
- [ ] Selection inputs: `--note`, `--tag`, `--moc`, `--query`
- [ ] Deterministic ordering (MOC order or created_at, id)
- [ ] Link handling options:
  - [ ] Preserve wiki links
  - [ ] Rewrite to markdown links
  - [ ] Rewrite to section anchors
- [ ] `--json` output

---

## Phase 7: Compaction (P7 - Advanced)

### Compaction (`specs/compaction.md`)
- [ ] **Digest note type** - notes that summarize other notes
- [ ] **Compaction edges** - `digest -> source` relationships
- [ ] **Canonicalization** - `canon(id)` function following compaction chains
- [ ] **Contracted graph** - effective graph after canonicalization
- [ ] **Invariant enforcement**:
  - [ ] At most one compactor per note
  - [ ] Acyclic compaction
  - [ ] No self-compaction
  - [ ] All referenced IDs resolve
- [ ] `qipu compact apply <digest-id> --note <id>...` - register compaction
- [ ] `qipu compact show <digest-id>` - show compaction set
- [ ] `qipu compact status <id>` - show compaction relationships
- [ ] `qipu compact report <digest-id>` - compaction quality metrics
- [ ] `qipu compact suggest` - suggest compaction candidates
- [ ] `qipu compact guide` - print compaction guidance for LLMs
- [ ] **Output annotations**: `compacts=<N>`, `compaction=<P%>`, `via=<id>`
- [ ] **Flags**: `--no-resolve-compaction`, `--with-compaction-ids`, `--compaction-depth <n>`, `--expand-compaction`
- [ ] **Metrics**: compaction percent calculation, size estimation

---

## Phase 8: Maintenance & Validation (P8)

### Doctor (`specs/cli-interface.md`)
- [ ] `qipu doctor` - validate store invariants
- [ ] Check for duplicate IDs
- [ ] Check for broken links
- [ ] Check for invalid frontmatter
- [ ] Check compaction invariants
- [ ] `qipu doctor --fix` - attempt repairs

### Sync (`specs/cli-interface.md`)
- [ ] `qipu sync` - convenience command for workflows
- [ ] Run `qipu index`
- [ ] Run `qipu doctor` validations
- [ ] (Optional) protected-branch workflow support

---

## Phase 9: Setup & Integration (P9)

### Setup (`specs/cli-interface.md`, `specs/llm-context.md`)
- [ ] `qipu setup --list` - list available integrations
- [ ] `qipu setup <tool>` - install integration for tool
- [ ] `qipu setup --print` - print integration instructions
- [ ] `qipu setup <tool> --check` - verify integration
- [ ] `qipu setup <tool> --remove` - remove integration
- [ ] **AGENTS.md integration** - cross-tool standard

---

## Open Questions (from specs)

### From `specs/storage-format.md`:
- Should `mocs/` live inside `notes/` with a type flag?
- Should note paths be flat or date-partitioned?
- Should attachments be per-note folders?

### From `specs/knowledge-model.md`:
- Should qipu enforce a type taxonomy or allow arbitrary types?
- Minimal useful typed link set?
- Duplicate/near-duplicate detection and merge?

### From `specs/cli-interface.md`:
- Interactive pickers (fzf-style)?
- Should `qipu capture` default to `--type fleeting`?
- Should `qipu sync` manage git commits/pushes?

### From `specs/indexing-search.md`:
- JSON indexes, SQLite indexes, or both?
- Should backlinks be embedded into notes (opt-in)?

### From `specs/graph-traversal.md`:
- Default `--max-depth`: 2 or 3?
- Should inline links be materialized into `links[]` automatically?
- Additional traversal queries (neighbors, subgraph, cycles)?

### From `specs/token-optimized-output.md`:
- `--token-version` flag for stability?
- Include edges by default or require `--with-edges`?
- Summaries-only default, require `--with-body`?
- Model-specific tokenizer option?

### From `specs/compaction.md`:
- Inactive compaction edges for history/versioning?
- Exclude MOCs/spec notes from compaction suggestions?
- First-class "leaf source" vs "intermediate digest" concept?

### From `specs/llm-context.md`:
- Lightweight automatic summarization without LLM?
- Include backlinks as additional material in context?

### From `specs/export.md`:
- Pandoc integration for PDF?
- Include transitive links (depth-limited)?

---

## Implementation Notes

### Dependency Graph (Phases)
```
Phase 0 (Bootstrap)
    |
    v
Phase 1 (Foundation) -- required by all other phases
    |
    v
Phase 2 (Core Commands) -- MVP
    |
    v
Phase 3 (Indexing) -- required by P4, P5
    |
    +-----> Phase 4 (Graph Traversal)
    |
    +-----> Phase 5 (LLM Integration) -- core value prop
    |
    v
Phase 6 (Export)
    |
    v
Phase 7 (Compaction) -- depends on P3, P4, P5
    |
    v
Phase 8 (Maintenance)
    |
    v
Phase 9 (Setup)
```

### Testing Strategy
- Unit tests for all `src/lib/` utilities
- Integration tests for CLI commands (temporary directory stores)
- Golden tests for deterministic outputs (`prime`, `context`, traversal)
- Property-based tests for ID generation collision resistance
