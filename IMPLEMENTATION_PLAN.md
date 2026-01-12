# Qipu Implementation Plan

Status: Initial draft  
Last updated: 2026-01-12

This document tracks implementation progress against the specs in `specs/`.

**Implementation Status**: No implementation code exists yet. All items below are pending.

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
- [ ] Create CONTRIBUTING.md with development setup guide

---

## Phase 1: Foundation (P1 - Required for Any Command)

All commands depend on these foundational layers.

### Storage Layer (`specs/storage-format.md`)
- [ ] **Store discovery** - walk up from cwd to find `.qipu/` (or use `--store`)
- [ ] **Store initialization** - create `.qipu/`, `notes/`, `mocs/`, `attachments/`, `templates/`, `config.toml`
- [ ] **Config parsing** - read/write `.qipu/config.toml`
  - [ ] `format_version` field (for forward compatibility)
  - [ ] `id_scheme` field (`hash` | `ulid` | `timestamp`)
  - [ ] `default_note_type` field
  - [ ] `editor` field (editor preference override)
- [ ] **Note file parsing** - YAML frontmatter + markdown body
- [ ] **Note file writing** - deterministic serialization (stable key order, newline handling)
  - [ ] Preserve newline style (avoid unnecessary file rewrites)
- [ ] **ID generation** - `qp-<hash>` with adaptive length
- [ ] **Slug generation** - `<id>-<slug(title)>.md` filename convention
- [ ] **Template loading** - load note templates from `templates/`

### Knowledge Model (`specs/knowledge-model.md`)
- [ ] **Note struct** - id, title, type, tags, created, updated, sources, links, body
- [ ] **Note types** - fleeting, literature, permanent, moc
- [ ] **Tag model** - list of strings, validation
- [ ] **Typed links model** - related, derived-from, supports, contradicts, part-of
- [ ] **Source model** - url, title, accessed

### CLI Runtime (`specs/cli-tool.md`)
- [ ] **`--help` flag** - stable help output, exit 0
- [ ] **`--version` flag** - single line version info, exit 0
- [ ] **Argument parsing** - global flags (`--store`, `--root`, `--json`, `--token`, `--quiet`, `--verbose`)
  - [ ] `--json` and `--token` are mutually exclusive
- [ ] **Command dispatch** - route to subcommand handlers
- [ ] **Exit codes** - 0 success, 1 failure, 2 usage error, 3 data error
- [ ] **Error formatting** - human vs `--json` error output
- [ ] **Output determinism** - ensure stable ordering across all outputs
- [ ] **Offline operation** - no network access required for normal operation

---

## Phase 2: Core Commands (P2 - MVP Feature Set)

Implements the essential commands for basic note capture and retrieval.

### `qipu init` (`specs/cli-interface.md`)
- [ ] Create store at default or specified location
- [ ] `--stealth` mode (gitignore the store)
- [ ] `--visible` mode (use `qipu/` instead of `.qipu/`)
- [ ] `--branch <name>` (optional protected-branch workflow configuration)
- [ ] Idempotent (safe to run multiple times)

### `qipu create` / `qipu new` (`specs/cli-interface.md`)
- [ ] Generate new note with ID, slug, frontmatter
- [ ] `--type` flag (fleeting, literature, permanent, moc)
- [ ] `--tag` flag (repeatable)
- [ ] `--open` flag (open in `$EDITOR`)
- [ ] `--template` flag (use template from `templates/`)
- [ ] Print note ID/path on success
- [ ] `qipu new` alias for `qipu create`

### `qipu capture` (`specs/cli-interface.md`)
- [ ] Create note from stdin
- [ ] `--title` flag
- [ ] `--type` flag
- [ ] `--tag` flag

### `qipu show <id-or-path>` (`specs/cli-interface.md`)
- [ ] Resolve ID or path to note file
- [ ] Print note content to stdout
- [ ] `--json` output format
- [ ] `--links` flag - show links for the note (per `knowledge-model.md`)

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
- [ ] `--moc` / `--no-moc` filter - include/exclude MOCs from results
- [ ] `--json` output
- [ ] Result ranking (title > body, exact tag > text, recency boost)
- [ ] (Optional) ripgrep integration if available
- [ ] (Optional) SQLite FTS if `.qipu/qipu.db` is present

### `qipu inbox` (`specs/cli-interface.md`)
- [ ] List unprocessed notes (type in {fleeting, literature})
- [ ] `--no-moc` flag - exclude notes already linked into a MOC
- [ ] `--json` output

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
  - [ ] `--token` output

### Graph Traversal (`specs/graph-traversal.md`)
- [ ] `qipu link tree <id>` - traversal tree from note
  - [ ] `--direction <out|in|both>` (default: both)
  - [ ] `--max-depth <n>` (default: 3)
  - [ ] `--type <t>` / `--types <csv>` filter (repeatable)
  - [ ] `--exclude-type <t>` / `--exclude-types <csv>` filter
  - [ ] `--typed-only` / `--inline-only`
  - [ ] `--max-nodes <n>` (optional)
  - [ ] `--max-edges <n>` (optional cap on edges emitted)
  - [ ] `--max-children <n>` (optional cap per expanded node)
  - [ ] Cycle detection (mark visited nodes as "(seen)")
  - [ ] Deterministic BFS ordering (sort by edge type, then target id)
  - [ ] `--json` output
    - [ ] `nodes[]`, `edges[]`, `spanning_tree[]` fields
    - [ ] Edge `source` field: `"inline"` | `"typed"`
  - [ ] `--token` output
  - [ ] Truncation reporting when limits hit
- [ ] `qipu link path <from> <to>` - find path between notes
  - [ ] `--direction`, `--max-depth`, `--typed-only`, `--inline-only` flags
  - [ ] `--json` output
  - [ ] `--token` output

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
- [ ] `--with-body` flag for including full note bodies

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
    - [ ] Support both direct list and transitive closure modes
  - [ ] `--query <text>` (search-based selection)
  - [ ] `--walk <id> --max-depth <n>` (graph traversal shortcut, optional future feature)
- [ ] Budgeting: `--max-chars`, `--max-tokens`
- [ ] Output formats:
  - [ ] Default: markdown bundle with metadata headers
  - [ ] `--json` output (with `generated_at`, `store`, `notes[]` including `content` and `sources[]`)
  - [ ] `--token` output (summaries-first, `--with-body` for full content)
- [ ] Truncation handling
  - [ ] Explicit truncation marker (e.g., `…[truncated]`) when notes are cut
- [ ] Deterministic ordering
- [ ] Safety banner (notes are untrusted; optional prompt-injection warning)

---

## Phase 6: Export (P6)

### Export Commands (`specs/export.md`)
- [ ] `qipu export` command
- [ ] Bundle export (concatenate notes)
- [ ] Outline export (MOC-driven ordering)
- [ ] Bibliography export (extract sources)
  - [ ] (Future) BibTeX/CSL JSON format support
- [ ] Selection inputs: `--note`, `--tag`, `--moc`, `--query`
- [ ] Deterministic ordering (MOC order or created_at, id)
- [ ] Link handling options:
  - [ ] Preserve wiki links (default)
  - [ ] Rewrite to markdown links
  - [ ] Rewrite to section anchors
- [ ] Attachment handling:
  - [ ] `--no-attachments` (default - don't copy)
  - [ ] `--attachments <dir>` - copy attachments to export folder
- [ ] `--json` output

---

## Phase 7: Compaction (P7 - Advanced)

### Compaction (`specs/compaction.md`)
- [ ] **Digest note type** - notes that summarize other notes
- [ ] **Compaction edges** - `digest -> source` relationships
- [ ] **Canonicalization** - `canon(id)` function following compaction chains
- [ ] **Contracted graph** - effective graph after canonicalization
- [ ] **Visibility rules** - compacted notes hidden by default in resolved view
- [ ] **Invariant enforcement**:
  - [ ] At most one compactor per note
  - [ ] Acyclic compaction
  - [ ] No self-compaction
  - [ ] All referenced IDs resolve
- [ ] `qipu compact apply <digest-id> --note <id>...` - register compaction
  - [ ] `--notes-file <file>` - read IDs from file
  - [ ] `--from-stdin` - read IDs from stdin
- [ ] `qipu compact show <digest-id>` - show compaction set
  - [ ] `--compaction-depth <n>` - depth-limited compaction tree view
- [ ] `qipu compact status <id>` - show compaction relationships
- [ ] `qipu compact report <digest-id>` - compaction quality metrics
  - [ ] Boundary edge ratio metric
  - [ ] Staleness indicator (sources updated after digest)
- [ ] `qipu compact suggest` - suggest compaction candidates
  - [ ] Detailed JSON output shape per spec
- [ ] `qipu compact guide` - print compaction guidance for LLMs
  - [ ] Include prompt template for digest authoring
- [ ] **Output annotations**: `compacts=<N>`, `compaction=<P%>`, `via=<id>` (in human, `--json`, and `--token` modes)
- [ ] **Global flags** (affect `show`, `search`, `context`, `link tree`, etc.):
  - [ ] `--no-resolve-compaction` - disable canonicalization, show all notes
  - [ ] `--with-compaction-ids` - include compacted note IDs in output
  - [ ] `--compaction-depth <n>` - depth of compaction expansion
  - [ ] `--expand-compaction` - include compacted source note bodies
  - [ ] `--compaction-max-nodes <n>` - optional bound on expansion
- [ ] **Metrics**: compaction percent calculation, size estimation
- [ ] **Search/traversal behavior**: When search matches compacted note, show `via=<id>` annotation

---

## Phase 8: Maintenance & Validation (P8)

### Doctor (`specs/cli-interface.md`, `specs/storage-format.md`)
- [ ] `qipu doctor` - validate store invariants
- [ ] Check for duplicate IDs
- [ ] Check for broken links (unresolved wiki-links and typed link targets)
- [ ] Check for invalid frontmatter (missing required fields: id, title)
- [ ] Check for orphaned notes (no incoming links, not in any MOC)
- [ ] Check compaction invariants (acyclic, no self-compaction, at most one compactor)
- [ ] `qipu doctor --fix` - attempt repairs (remove broken links, regenerate IDs)

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

### From `README.md`:
- Should qipu ship a `setup` command with recipes for common agent tools (AGENTS.md, Cursor rules, Claude hooks)?
- Should there be a global (cross-repo) store option?

### From `docs/workflows.md`:
- Should qipu provide a first-class "promote" command (fleeting -> permanent)?
- Should qipu support per-repo and global stores simultaneously?

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
    |              |
    +--------------+-----> Phase 5 (LLM Integration) -- core value prop
    |                      (depends on P4 for MOC transitive closure)
    v
Phase 6 (Export)
    |
    v
Phase 7 (Compaction) -- depends on P3, P4, P5
    |                    NOTE: Compaction flags are cross-cutting and will
    |                    require modifications to P2-P5 commands
    v
Phase 8 (Maintenance)
    |
    v
Phase 9 (Setup)
```

### Cross-Cutting Concerns

The following features span multiple phases and will require modifications across phases:

1. **Compaction resolution (Phase 7)**
   - Affects: `qipu show`, `qipu search`, `qipu context`, `qipu link tree/path/list`
   - Global flags (`--no-resolve-compaction`, `--expand-compaction`, etc.) must be added to commands in P2-P5
   - Plan for this when implementing those commands

2. **Token output format (Phase 5)**
   - Affects: `qipu link list/tree/path` (P4), `qipu prime`, `qipu context`
   - The `--token` flag should be planned during P4 implementation

3. **Deterministic output**
   - All phases: ensure stable ordering in lists, JSON arrays, token output
   - Required for golden tests and LLM reproducibility

### Testing Strategy
- Unit tests for all `src/lib/` utilities
- Integration tests for CLI commands (temporary directory stores)
- Golden tests for deterministic outputs (`prime`, `context`, traversal)
- Property-based tests for ID generation collision resistance
