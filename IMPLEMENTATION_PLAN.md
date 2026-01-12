# Qipu Implementation Plan

Last updated: 2026-01-12

This document tracks implementation progress against the specs in `specs/`.

---

## Current Status

**Phase 0 (Project Bootstrap)**: COMPLETED

- TypeScript + Node.js, npm, Vitest, ESLint + Prettier

**Phase 1 (Foundation)**: COMPLETED

- Storage layer, knowledge model, CLI runtime all implemented

**Phase 2 (Core Commands)**: IN PROGRESS

- Completed: `init`, `create`, `list`, `show`
- Remaining: `capture`, `new` alias registration

**Testing**: 50 unit tests passing (models, storage, parsing)

**Next Steps**: Complete `qipu capture` and `qipu new` alias, then move to Phase 3 (Indexing & Navigation)

---

## Phase 0: Project Bootstrap - COMPLETED

- [x] TypeScript with Node.js
- [x] npm package manager
- [x] Project structure: `src/`, `src/lib/`, `src/commands/`
- [x] Test framework: Vitest
- [x] CI/linting: ESLint + Prettier configured

---

## Phase 1: Foundation - COMPLETED

### Storage Layer (`src/lib/storage.ts`)

- [x] Store discovery (walk up from cwd to find `.qipu/`, `--store` flag support)
- [x] Store initialization with all subdirectories
- [x] Config parsing (TOML)
- [x] Note file parsing (YAML frontmatter + markdown)
- [x] Note file writing with deterministic serialization
- [x] ID generation (`qp-<hash>` format)
- [x] Slug generation

### Knowledge Model (`src/lib/models.ts`)

- [x] Note struct with all fields
- [x] Note types (fleeting, literature, permanent, moc)
- [x] Typed links model
- [x] Source model
- [x] Exit codes

### CLI Runtime (`src/cli.ts`)

- [x] `--help` and `--version` flags
- [x] Global flags (`--store`, `--root`, `--json`, `--token`, `--quiet`, `--verbose`)
- [x] Mutual exclusivity validation for `--json` and `--token`
- [x] Exit codes (0 success, 1 failure, 2 usage error, 3 data error)

### Pending Foundation Items

- [ ] Template loading from `templates/`
- [ ] Git integration defaults (`.gitignore` setup)
- [ ] Error formatting for `--json` mode
- [ ] Unknown flags/args exit code 2 handling

---

## Phase 2: Core Commands - IN PROGRESS

### `qipu init` - COMPLETED

- [x] Create store at default or specified location
- [x] `--stealth` mode (gitignore the store)
- [x] `--visible` mode (use `qipu/` instead of `.qipu/`)
- [ ] `--branch <name>` (optional protected-branch workflow configuration)
- [x] Idempotent (safe to run multiple times)

### `qipu create` - COMPLETED

- [x] Generate new note with ID, slug, frontmatter
- [x] `--type` flag (fleeting, literature, permanent, moc)
- [x] `--tag` flag (repeatable)
- [x] `--open` flag (open in `$EDITOR`)
- [ ] `--template` flag (use template from `templates/`)
- [x] Print note ID/path on success

### `qipu new` alias - NOT IMPLEMENTED

- [ ] Register `qipu new` as alias for `qipu create`

### `qipu list` - COMPLETED

- [x] List all notes
- [x] `--tag` filter
- [x] `--type` filter
- [x] `--since` filter
- [x] `--json` output format
- [x] Deterministic ordering

### `qipu show <id-or-path>` - COMPLETED

- [x] Resolve ID or path to note file
- [x] Print note content to stdout
- [x] `--json` output format
- [x] `--links` flag

### `qipu capture` - NOT IMPLEMENTED

- [ ] Create note from stdin
- [ ] `--title` flag
- [ ] `--type` flag
- [ ] `--tag` flag

---

## Phase 3: Indexing & Navigation (P3)

### Indexing (`specs/indexing-search.md`)

- [ ] Metadata index (id -> {title, type, tags, path, created, updated})
- [ ] Tag index (tag -> [ids...])
- [ ] Link extraction (wiki links, markdown links, typed links)
- [ ] Backlink index
- [ ] Graph adjacency list
- [ ] Incremental indexing (track mtimes)
- [ ] Cache storage (`.qipu/.cache/*.json`)
- [ ] `qipu index` command
- [ ] `qipu index --rebuild` command

### Search (`specs/indexing-search.md`)

- [ ] `qipu search <query>` - full-text search
- [ ] `--tag`, `--type`, `--moc`/`--no-moc` filters
- [ ] `--json` output
- [ ] Result ranking

### `qipu inbox` (`specs/cli-interface.md`)

- [ ] List unprocessed notes (type in {fleeting, literature})
- [ ] `--no-moc` flag
- [ ] `--json` output

---

## Phase 4: Link Management & Graph Traversal (P4)

### Link Commands

- [ ] `qipu link add <from> <to> --type <t>`
- [ ] `qipu link remove <from> <to> --type <t>`
- [ ] `qipu link list <id>` with direction/type filters

### Graph Traversal

- [ ] `qipu link tree <id>` with depth, type, direction options
- [ ] `qipu link path <from> <to>`
- [ ] Cycle detection
- [ ] `--json` and `--token` output formats

---

## Phase 5: LLM Integration (P5)

### Token-Optimized Output (`specs/token-optimized-output.md`)

- [ ] `--token` output format
- [ ] Record types: H (header), N (note), S (summary), E (edge), B (body)
- [ ] Budget enforcement (`--max-chars`, `--max-tokens`)
- [ ] `--with-body` flag

### `qipu prime` (`specs/llm-context.md`)

- [ ] Emit bounded session primer (~1-2k tokens)
- [ ] `--json` and `--token` output

### `qipu context` (`specs/llm-context.md`)

- [ ] Bundle selection (`--note`, `--tag`, `--moc`, `--query`)
- [ ] Budgeting (`--max-chars`, `--max-tokens`)
- [ ] Output formats (markdown, `--json`, `--token`)

---

## Phase 6: Export (P6)

- [ ] `qipu export` command
- [ ] Bundle, outline, and bibliography export modes
- [ ] Selection inputs (`--note`, `--tag`, `--moc`, `--query`)
- [ ] Link handling options
- [ ] Attachment handling

---

## Phase 7: Compaction (P7)

- [ ] Digest note type
- [ ] Compaction edges and canonicalization
- [ ] `qipu compact apply/show/status/report/suggest/guide` commands
- [ ] Global compaction flags for other commands

---

## Phase 8: Maintenance & Validation (P8)

- [ ] `qipu doctor` - validate store invariants
- [ ] `qipu doctor --fix` - attempt repairs
- [ ] `qipu sync` - convenience workflow command

---

## Phase 9: Setup & Integration (P9)

- [ ] `qipu setup --list/--print/<tool>/--check/--remove`
- [ ] AGENTS.md integration

---

## Open Questions (from specs)

See `specs/` for detailed open questions on:

- Storage format (MOC location, path structure, attachments)
- Knowledge model (type taxonomy, link types, deduplication)
- CLI interface (interactive pickers, default behaviors)
- Indexing (JSON vs SQLite, backlink embedding)
- Graph traversal (default depth, link materialization)
- Token output (versioning, default inclusions)
- Compaction (inactive edges, exclusions)
- LLM context (summarization, backlinks)
- Export (Pandoc integration, transitive links)

---

## Implementation Notes

### Dependency Graph (Phases)

```
Phase 0 (Bootstrap) - DONE
    |
    v
Phase 1 (Foundation) - DONE
    |
    v
Phase 2 (Core Commands) - IN PROGRESS
    |
    v
Phase 3 (Indexing) -- required by P4, P5
    |
    +-----> Phase 4 (Graph Traversal)
    |              |
    +--------------+-----> Phase 5 (LLM Integration)
    |
    v
Phase 6 (Export)
    |
    v
Phase 7 (Compaction)
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
- Golden tests for deterministic outputs
- Property-based tests for ID generation collision resistance
