# Qipu Implementation Plan

Status: Greenfield  
Last updated: 2026-01-12

This plan tracks implementation progress against specs in `specs/`. Items are sorted by priority (foundational infrastructure first, then core features, then advanced features).

## Phase 1: Foundation (CLI Runtime + Storage)

### P1.1 Project Scaffolding
- [ ] Initialize Cargo workspace with `qipu` binary crate (Rust)
- [ ] Set up `src/lib/` for shared utilities
- [ ] Configure release profile for single native binary (no runtime dependencies)
- [ ] Add basic CI (build + clippy + test)
- [ ] Cross-platform support: macOS, Linux, Windows

### P1.2 CLI Runtime (`specs/cli-tool.md`)
- [ ] Implement argument parsing with `clap`
- [ ] `qipu --help` - stable help output, exit 0
- [ ] `qipu --version` - single-line version, exit 0
- [ ] Global flags: `--root`, `--store`, `--format`, `--quiet`, `--verbose`
- [ ] Exit codes: 0 (success), 1 (failure), 2 (usage), 3 (data error)
- [ ] `--format` validation (human|json|records, error on unknown)
- [ ] Unknown flag/arg detection (exit 2)
- [ ] Verbose timing output (parse args, discover store, load indexes, execute)
- [ ] Filesystem hygiene: avoid unnecessary file rewrites, preserve newline style
- [ ] Offline-first: no network access required for normal operation

### P1.3 Store Discovery (`specs/cli-tool.md`, `specs/storage-format.md`)
- [ ] `--store` explicit path resolution (relative to `--root` or cwd)
- [ ] Walk-up discovery for `.qipu/` directory
- [ ] Missing-store detection (exit 3 for commands requiring store)
- [ ] `qipu init` - create store directory structure
- [ ] `qipu init --stealth` - local-only store (add to .gitignore)
- [ ] `qipu init --visible` - use `qipu/` instead of `.qipu/`
- [ ] `qipu init --branch <name>` - protected-branch workflow configuration
- [ ] Create `config.toml` with format version, default note type, id scheme, editor override
- [ ] Default gitignore entries for `.qipu/qipu.db` and `.qipu/.cache/` (non-stealth mode)

### P1.4 Storage Format (`specs/storage-format.md`)
- [ ] Directory structure: `notes/`, `mocs/`, `attachments/`, `templates/`, `.cache/`
- [ ] Note file parser (YAML frontmatter + markdown body)
- [ ] Frontmatter schema: `id`, `title`, `type`, `created`, `updated`, `tags`, `sources`, `links`
- [ ] ID generation: `qp-<hash>` with adaptive length
- [ ] Filename format: `<id>-<slug(title)>.md`
- [ ] Wiki-link parsing: `[[<id>]]`, `[[<id>|label]]`
- [ ] Markdown link resolution to qipu notes

## Phase 2: Core Note Operations

### P2.1 Note CRUD (`specs/cli-interface.md`)
- [ ] `qipu create <title>` - create new note, print id/path
- [ ] `--type` flag (fleeting|literature|permanent|moc)
- [ ] `--tag` flag (repeatable)
- [ ] `--open` flag (launch `$EDITOR`)
- [ ] `qipu new` alias for `create`
- [ ] `qipu capture` - create note from stdin
- [ ] `qipu capture --title`
- [ ] `qipu capture --type` and `--tag` flags (same as create)
- [ ] `qipu show <id-or-path>` - print note to stdout

### P2.2 Note Listing (`specs/cli-interface.md`)
- [ ] `qipu list` - list notes
- [ ] `--tag` filter
- [ ] `--type` filter
- [ ] `--since` filter
- [ ] `qipu inbox` - list unprocessed notes (fleeting|literature, not in MOC)
- [ ] JSON output for list commands (schema: id, title, type, tags, path, created, updated)
- [ ] Deterministic ordering (by created, then id)

## Phase 3: Indexing and Search

### P3.1 Index Infrastructure (`specs/indexing-search.md`)
- [ ] Metadata index: `id -> {title, type, tags, path, created, updated}`
- [ ] Tag index: `tag -> [ids...]`
- [ ] Backlink index: `id -> [ids that link to it]`
- [ ] Graph adjacency list (inline + typed links)
- [ ] Cache location: `.qipu/.cache/*.json`
- [ ] Optional SQLite cache: `.qipu/qipu.db` (for FTS acceleration)
- [ ] `qipu index` - build/refresh indexes
- [ ] `qipu index --rebuild` - drop and regenerate
- [ ] Incremental indexing (track mtimes/hashes)

### P3.2 Link Extraction (`specs/indexing-search.md`)
- [ ] Extract wiki links from note bodies
- [ ] Extract markdown links pointing to qipu notes
- [ ] Extract typed links from frontmatter
- [ ] Track unresolved links for `doctor` reporting

### P3.3 Search (`specs/indexing-search.md`, `specs/cli-interface.md`)
- [ ] `qipu search <query>` - search titles + bodies
- [ ] Type/tag filters for search
- [ ] Include/exclude MOCs option
- [ ] Result ranking: title > tag > body, recency boost
- [ ] JSON output for search results

## Phase 4: Link Management and Graph Traversal

### P4.1 Link Commands (`specs/cli-interface.md`)
- [ ] Typed link types: `related`, `derived-from`, `supports`, `contradicts`, `part-of`
- [ ] `qipu link add <from> <to> --type <t>` - add typed link
- [ ] `qipu link remove <from> <to> --type <t>` - remove typed link
- [ ] `qipu link list <id>` - list links for a note
- [ ] `--direction` flag (out|in|both, default: both)
- [ ] `--typed-only`, `--inline-only` filters
- [ ] `--type` filter for specific link types

### P4.2 Graph Traversal (`specs/graph-traversal.md`)
- [ ] `qipu link tree <id>` - traversal tree from note
- [ ] `--max-depth` (default: 3)
- [ ] Direction, type filters (same as link list)
- [ ] `--max-nodes`, `--max-edges`, `--max-children` limits
- [ ] Deterministic BFS with spanning tree
- [ ] Cycle-safe traversal (visited set, "(seen)" markers)
- [ ] Truncation reporting when limits hit
- [ ] `qipu link path <from> <to>` - find path between notes

### P4.3 Traversal Output Formats (`specs/graph-traversal.md`)
- [ ] Human-readable tree output
- [ ] JSON output: `{root, direction, max_depth, truncated, nodes[], edges[], spanning_tree[]}`
- [ ] Records output (see Phase 5)

## Phase 5: Output Formats

### P5.1 Records Output (`specs/records-output.md`)
- [ ] Header line (H): bundle metadata with `qipu=1 records=1` format version
- [ ] Note metadata line (N): id, type, title, tags, path
- [ ] Summary line (S): extracted summary
- [ ] Edge line (E): from, type, to, source
- [ ] Body lines (B): raw markdown
- [ ] Summary extraction: frontmatter `summary` > `## Summary` > first paragraph

### P5.2 Budgeting (`specs/records-output.md`, `specs/llm-context.md`)
- [ ] `--max-chars` exact budget
- [ ] Truncation handling (set `truncated=true`, no partial records)
- [ ] Deterministic truncation (same selection = same output)

## Phase 6: LLM Integration

### P6.1 Prime Command (`specs/llm-context.md`)
- [ ] `qipu prime` - session-start primer
- [ ] Deterministic, bounded output (~1-2k tokens)
- [ ] Contents: qipu explanation, command reference, store location, top MOCs, recent notes
- [ ] `--format records` support for prime output

### P6.2 Context Command (`specs/llm-context.md`)
- [ ] `qipu context` - build context bundle
- [ ] Selection: `--note`, `--tag`, `--moc`, `--query`
- [ ] `--max-chars` budget
- [ ] Markdown output (default): note headers + content with `---` separators
- [ ] JSON output: `{generated_at, store, notes[]}`
- [ ] Records output
- [ ] Safety banner (optional): warning about untrusted content

### P6.3 Setup Command (`specs/cli-interface.md`, `specs/llm-context.md`)
- [ ] `qipu setup --list` - list available integrations
- [ ] `qipu setup <tool>` - install integration
- [ ] `qipu setup --print` - print integration instructions
- [ ] `qipu setup <tool> --check` - verify installation
- [ ] `qipu setup <tool> --remove` - remove integration
- [ ] AGENTS.md integration recipe

## Phase 7: Export

### P7.1 Export Modes (`specs/export.md`)
- [ ] `qipu export` - export notes to document
- [ ] Selection: `--note`, `--tag`, `--moc`, `--query`
- [ ] Bundle mode: concatenated markdown with metadata headers
- [ ] Outline mode: MOC-driven ordering
- [ ] Bibliography mode: extract sources to markdown bibliography

### P7.2 Export Options (`specs/export.md`)
- [ ] Deterministic ordering (MOC order or created+id)
- [ ] Link handling: preserve wiki links (default), rewrite options
- [ ] Attachment handling: no attachments (default), copy option

## Phase 8: Compaction

### P8.1 Compaction Model (`specs/compaction.md`)
- [ ] Digest note type (note that summarizes other notes)
- [ ] Compaction edge storage in frontmatter
- [ ] `canon(id)` function: follow compaction chains to topmost digest (cycle-safe via visited set)
- [ ] Invariant enforcement: one compactor per note, acyclic, no self-compaction, all IDs resolve
- [ ] Deterministic error behavior for invariant violations (never "pick arbitrarily")

### P8.2 Compaction Commands (`specs/compaction.md`)
- [ ] `qipu compact apply <digest-id> --note <id>...` - register compaction
- [ ] `qipu compact apply` input ergonomics: `--from-stdin`, `--notes-file`
- [ ] `qipu compact show <digest-id>` - show direct compaction set
- [ ] `qipu compact status <id>` - show compaction relationships
- [ ] `qipu compact report <digest-id>` - compaction quality metrics
- [ ] `qipu compact suggest` - suggest compaction candidates
- [ ] `qipu compact guide` - print compaction guidance

### P8.3 Compaction Integration (`specs/compaction.md`)
- [ ] `--no-resolve-compaction` flag for raw view
- [ ] `--with-compaction-ids` flag for showing compacted IDs
- [ ] `--compaction-depth` flag for depth-limited expansion
- [ ] `--compaction-max-nodes` optional bounding flag
- [ ] `--expand-compaction` flag for including compacted bodies
- [ ] Compaction percent metric: `100 * (1 - digest_size / expanded_size)`
- [ ] Size estimation based on summary extraction (chars)
- [ ] `compacts=<N>` annotation in outputs (human/json/records)
- [ ] Truncation indication when compaction limits hit
- [ ] Deterministic ordering for compaction expansion (sorted by note id)
- [ ] Contracted graph for resolved view traversals

### P8.4 Search/Traversal with Compaction (`specs/compaction.md`)
- [ ] Search matches compacted notes but surfaces canonical digest
- [ ] `via=<id>` annotation for indirect matches
- [ ] Traversal operates on contracted graph by default

## Phase 9: Validation and Maintenance

### P9.1 Doctor Command (`specs/cli-interface.md`, `specs/storage-format.md`, `specs/compaction.md`)
- [ ] `qipu doctor` - validate store invariants
- [ ] Check for duplicate IDs
- [ ] Check for broken links
- [ ] Check for invalid frontmatter
- [ ] Check for compaction invariant violations (cycles, multi-compactor conflicts)
- [ ] `qipu doctor --fix` - auto-repair where possible

### P9.2 Sync Command (`specs/cli-interface.md`)
- [ ] `qipu sync` - ensure indexes are current
- [ ] Run validations (optional)
- [ ] Branch workflow support (optional, for protected-branch mode)

## Phase 10: Testing and Quality

### P10.1 Test Infrastructure (`specs/cli-tool.md`)
- [ ] Integration test framework (temp directories/stores)
- [ ] Golden test framework for deterministic outputs
- [ ] Performance benchmarks against budgets (--help <100ms, list 1k <200ms, search 10k <1s)

### P10.2 Golden Tests
- [ ] `qipu --help` output
- [ ] `qipu --version` output
- [ ] `qipu prime` output
- [ ] `qipu context` output (all formats)
- [ ] `qipu link tree` output (all formats)
- [ ] `qipu link list` output (all formats)
- [ ] `qipu link path` output (all formats)

---

## Open Questions (from specs)

These are design decisions noted in specs that may need resolution during implementation:

- Default store location: `.qipu/` (current default) vs `qipu/`
- Note ID scheme: hash-based `qp-xxxx` vs ULID vs timestamp
- Protected-branch workflow details
- `qipu setup` recipes for specific tools
- Global (cross-repo) store option
- Note type taxonomy: enforce vs allow arbitrary
- Minimal typed link set
- Duplicate/merge detection
- `mocs/` as separate directory vs inside `notes/` with type flag
- Flat vs date-partitioned note paths
- JSON vs SQLite indexes (or both)
- Interactive pickers (fzf-style) as optional UX
- `qipu capture` default type
- `qipu sync` scope (index-only vs git operations)
- Pandoc integration for PDF export
