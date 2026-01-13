# Qipu Implementation Plan

Last updated: 2026-01-12 (revised)

This document tracks implementation progress against the specs in `specs/`.

---

## Current Status

**No source code exists yet.** The `src/` directory has not been created.

The spec `cli-tool.md` mandates Rust + Cargo. All items below are unimplemented.

---

## Priority Order

Work items are listed in dependency order. Complete each phase before starting the next.

---

## Phase 0: Project Bootstrap (BLOCKING - START HERE)

- [ ] Initialize Cargo project (`cargo init --name qipu`)
- [ ] Create `src/` directory structure
- [ ] Add core dependencies (clap, serde, toml, etc.)
- [ ] Set up integration test harness (temp directory stores)
- [ ] Set up golden test infrastructure
- [ ] Configure CI (build + test across Linux/macOS/Windows)

---

## Phase 1: Foundation

### 1.1 CLI Runtime Skeleton
- [ ] Implement `qipu --help` (clap-based, stable output)
- [ ] Implement `qipu --version` (single line, exit 0)
- [ ] Global flags: `--root`, `--store`, `--json`, `--token`, `--quiet`, `--verbose`
- [ ] Exit code handling (0=success, 1=failure, 2=usage, 3=data error)
- [ ] `--json` and `--token` mutual exclusivity enforcement
- [ ] `--verbose` timing output for major phases (parse args, discover store, load indexes, execute)
- [ ] Timing output deterministic in shape (keys/labels stable)

### 1.2 Store Discovery
- [ ] Walk up from cwd to find `.qipu/` directory
- [ ] Support `--store <path>` explicit override
- [ ] Support `--root <path>` base directory
- [ ] Missing-store detection (exit code 3)

### 1.3 Config System
- [ ] Parse `.qipu/config.toml`
- [ ] Define sensible defaults (format version, note type, id scheme)
- [ ] Config validation

### 1.4 Note Model (`src/lib/note.rs`)
- [ ] YAML frontmatter parsing (id, title, type, created, updated, tags, sources, links)
- [ ] Markdown body extraction
- [ ] Deterministic serialization (stable key order, stable formatting)
- [ ] Required field validation (id, title)

### 1.5 ID Generation (`src/lib/id.rs`)
- [ ] Hash-based ID scheme: `qp-<hash>` with adaptive length
- [ ] Slug generation from title (lowercase, hyphenated)
- [ ] Filename format: `<id>-<slug>.md`
- [ ] Collision detection support

### 1.6 Error Handling (`src/lib/error.rs`)
- [ ] Structured error types
- [ ] Human-friendly error messages (default)
- [ ] JSON error output when `--json` is set

---

## Phase 2: Core Commands

### 2.1 `qipu init`
- [ ] Create `.qipu/` directory structure (notes/, mocs/, attachments/, templates/)
- [ ] Create default `config.toml`
- [ ] Idempotent behavior (safe to run multiple times)
- [ ] `--stealth` flag (add to .gitignore)
- [ ] `--visible` flag (use `qipu/` instead of `.qipu/`)
- [ ] `--branch` flag (protected-branch workflow config)

### 2.2 `qipu create` / `qipu new`
- [ ] Create note with generated ID and slug
- [ ] `--type <fleeting|literature|permanent|moc>` flag
- [ ] `--tag <tag>` flag (repeatable)
- [ ] `--open` flag (open in $EDITOR)
- [ ] Template support
- [ ] Print ID/path on success
- [ ] `--json` output

### 2.3 `qipu capture`
- [ ] Read note content from stdin
- [ ] `--title` flag
- [ ] `--type` and `--tag` flags
- [ ] Non-interactive workflow support
- [ ] `--json` output

### 2.4 `qipu list`
- [ ] List all notes with metadata
- [ ] `--tag <tag>` filter
- [ ] `--type <type>` filter
- [ ] `--since <date>` filter
- [ ] Deterministic ordering (created_at, id)
- [ ] `--json` output (JSON lines or array)

### 2.5 `qipu show <id-or-path>`
- [ ] Resolve ID or path to note
- [ ] Print note content
- [ ] `--json` output (full note metadata + content)
- [ ] `--links` flag (show links from/to note)

---

## Phase 3: Indexing & Navigation

### 3.1 Index Infrastructure (`src/lib/index.rs`)
- [ ] Metadata index: id -> {title, type, tags, path, created, updated}
- [ ] Tag index: tag -> [ids...]
- [ ] Backlink index: id -> [ids that link to it]
- [ ] Graph adjacency list (inline + typed links)
- [ ] Cache location: `.qipu/.cache/*.json`

### 3.2 Link Extraction
- [ ] Parse wiki links: `[[<id>]]`, `[[<id>|label]]`
- [ ] Parse markdown links to qipu notes
- [ ] Extract typed links from frontmatter
- [ ] Handle unresolved links gracefully

### 3.3 `qipu index`
- [ ] Build/update all indexes
- [ ] Incremental indexing (track mtimes)
- [ ] `--rebuild` flag (drop and regenerate)

### 3.4 `qipu search <query>`
- [ ] Full-text search (title + body)
- [ ] `--tag`, `--type` filters
- [ ] `--include-mocs`, `--exclude-mocs` filters
- [ ] Result ranking (title > body, exact > partial)
- [ ] Recency boost for recently updated notes
- [ ] `--json` output

### 3.5 `qipu inbox`
- [ ] List unprocessed notes (fleeting, literature)
- [ ] Exclude notes linked into MOCs (optional)
- [ ] `--json` output

---

## Phase 4: Link Management & Graph Traversal

### 4.1 `qipu link add/remove/list`
- [ ] `qipu link add <from> <to> --type <type>`
- [ ] `qipu link remove <from> <to> --type <type>`
- [ ] `qipu link list <id> [--direction out|in|both] [--typed-only|--inline-only] [--type <t>]`
- [ ] Update frontmatter links array on add/remove
- [ ] `--json` output

### 4.2 `qipu link tree <id>`
- [ ] BFS traversal with deterministic ordering
- [ ] `--direction <out|in|both>` (default: both)
- [ ] `--max-depth <n>` (default: 3)
- [ ] `--typed-only`, `--inline-only`, `--type`, `--exclude-type` filters
- [ ] `--max-nodes` limit
- [ ] Cycle detection (mark as "(seen)")
- [ ] Truncation reporting
- [ ] Human-readable tree output
- [ ] `--json` output (nodes, edges, spanning_tree)
- [ ] `--token` output (per token-optimized-output spec)

### 4.3 `qipu link path <from> <to>`
- [ ] Find shortest path between notes
- [ ] Same filter flags as tree
- [ ] Human-readable path output
- [ ] `--json` output

---

## Phase 5: LLM Integration

### 5.1 Token-Optimized Output (`src/lib/token.rs`)
- [ ] Header line format (H record) with format version (`token=1`)
- [ ] Note metadata line format (N record)
- [ ] Summary line format (S record)
- [ ] Edge line format (E record)
- [ ] Body lines format (B record)
- [ ] Summary extraction (frontmatter > ## Summary section > first paragraph)
- [ ] `--with-body` flag (include body lines, default summaries-only)
- [ ] `--with-edges` flag (include edge records)

### 5.2 `qipu prime`
- [ ] Emit bounded primer (~1-2k tokens)
- [ ] Include: qipu explanation, command reference, store location
- [ ] Include: top MOCs, recently updated notes
- [ ] Deterministic, stable output
- [ ] `--json` output
- [ ] `--token` output

### 5.3 `qipu context`
- [ ] Bundle selection: `--note`, `--tag`, `--moc`, `--query`
- [ ] MOC modes: `--moc-mode direct` (default) vs `--moc-mode transitive`
- [ ] `--walk <id>` shortcut for graph-based context (traversal from ID)
- [ ] Budgeting: `--max-chars`, `--max-tokens`
- [ ] Markdown output format (per llm-context spec)
- [ ] `--json` output
- [ ] `--token` output (summaries-first, optional `--with-body`)
- [ ] Truncation handling (complete notes, explicit markers)
- [ ] `--safety-banner` flag (prepend warning about untrusted content)

---

## Phase 6: Export

### 6.1 `qipu export`
- [ ] Bundle export mode (concatenate selected notes)
- [ ] Outline export mode (MOC-first ordering)
- [ ] Bibliography export mode (extract sources)
- [ ] Selection: `--note`, `--tag`, `--moc`, `--query`
- [ ] Deterministic ordering (MOC order or created_at,id)
- [ ] Link handling options (preserve, rewrite to markdown, rewrite to anchors)
- [ ] Attachment handling (`--no-attachments` default)

---

## Phase 7: Compaction

### 7.1 Digest Model
- [ ] Digest note type
- [ ] Compaction edges in frontmatter (digest -> sources)
- [ ] Invariants: one compactor per note, acyclic, no self-compaction

### 7.2 Canonicalization (`src/lib/compaction.rs`)
- [ ] `canon(id)` function (follow compaction chain)
- [ ] Contracted graph computation
- [ ] Cycle detection and error handling

### 7.3 Visibility & Metrics
- [ ] Hidden-by-default for compacted notes
- [ ] `--no-resolve-compaction` flag
- [ ] `compacts=<N>` annotation
- [ ] `compaction=<P%>` metric calculation
- [ ] `via=<id>` breadcrumb for search hits

### 7.4 Compaction Commands
- [ ] `qipu compact apply <digest-id> --note <id>...`
  - [ ] `--from-stdin` (read IDs from stdin)
  - [ ] `--notes-file <path>` (read IDs from file)
  - [ ] Invariant validation (no cycles, no multi-compactor conflicts)
  - [ ] Idempotent behavior
- [ ] `qipu compact show <digest-id>` (with `--compaction-depth`)
- [ ] `qipu compact status <id>`
- [ ] `qipu compact report <digest-id>` (mechanical checks)
  - [ ] `compacts_direct_count`
  - [ ] `compaction_pct`
  - [ ] Boundary edge ratio
  - [ ] Staleness indicator (sources updated after digest)
  - [ ] Conflicts/cycles detection
- [ ] `qipu compact suggest` (candidate detection)
  - [ ] Community/clump detection algorithm
  - [ ] Ranking by size, cohesion, boundary edges
  - [ ] `--json` output with candidate list and suggested commands
- [ ] `qipu compact guide` (LLM guidance prompt)
  - [ ] Steps: suggest → review → author → apply → validate
  - [ ] Prompt template for digest authoring

### 7.5 Compaction Integration
- [ ] `--no-resolve-compaction` flag (show raw view, disable canonicalization)
- [ ] `--with-compaction-ids`, `--compaction-depth` flags
- [ ] `--compaction-max-nodes <n>` bound for expansion
- [ ] `--expand-compaction` for context/traversal
- [ ] Search canonicalization (return digest, annotate `via=<id>`)
- [ ] Traversal on contracted graph
- [ ] Output annotations: `compacts=<N>`, `compaction=<P%>`

---

## Phase 8: Maintenance & Validation

### 8.1 `qipu doctor`
- [ ] Check for duplicate IDs
- [ ] Check for broken links
- [ ] Check for invalid frontmatter
- [ ] Check for compaction invariant violations:
  - [ ] Cycle detection in compaction chains
  - [ ] Multi-compactor conflicts
  - [ ] Self-compaction
  - [ ] Compaction staleness (sources updated after digest)
- [ ] Report unresolved links
- [ ] `--json` output

### 8.2 `qipu doctor --fix`
- [ ] Attempt automatic repairs where safe
- [ ] Report unfixable issues

### 8.3 `qipu sync`
- [ ] Run `qipu index`
- [ ] Run `qipu doctor`
- [ ] Optional git commit/push (if branch mode configured)

---

## Phase 9: Setup & Integration

### 9.1 `qipu setup`
- [ ] `qipu setup --list` (available tools)
- [ ] `qipu setup <tool>` (install integration)
- [ ] `qipu setup --print` (print without installing)
- [ ] `qipu setup <tool> --check` (verify installation)
- [ ] `qipu setup <tool> --remove` (uninstall)
- [ ] AGENTS.md integration recipe

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

## Future Considerations (from open questions)

These items are noted in specs as open questions but may warrant implementation:

- [ ] `qipu promote` command (upgrade fleeting → permanent notes)
- [ ] Duplicate/near-duplicate detection for notes
- [ ] Graph: `neighbors`, `subgraph`, `cycles` traversal queries
- [ ] Link materialization (opt-in inline → typed link conversion)
- [ ] `--tokenizer` option for model-specific token estimation
- [ ] `--token-version` flag for format stability
- [ ] Bibliography export formats (BibTeX, CSL JSON)
- [ ] Global (cross-repo) store option

---

## Implementation Notes

### Dependency Graph (Phases)

```
Phase 0 (Bootstrap) ─────────────────────────────────┐
    │                                                │
    v                                                │
Phase 1 (Foundation)                                 │
    │                                                │
    v                                                │
Phase 2 (Core Commands)                              │
    │                                                │
    v                                                │
Phase 3 (Indexing) ──────────────────────────────────┤
    │                                                │
    ├────────> Phase 4 (Graph Traversal)             │
    │                     │                          │
    └─────────────────────┴────> Phase 5 (LLM)       │
                                     │               │
                                     v               │
                               Phase 6 (Export)      │
                                     │               │
                                     v               │
                               Phase 7 (Compaction)  │
                                     │               │
                                     v               │
                               Phase 8 (Maintenance) │
                                     │               │
                                     v               │
                               Phase 9 (Setup) ──────┘
```

### Testing Strategy

- Integration tests for CLI commands (temporary directory stores)
- Golden tests for deterministic outputs (`prime`, `context`, traversal)
- Golden tests for `--verbose` timing output shape (deterministic keys/labels)
- Property-based tests (ID collision resistance, parsing round-trips)
- Performance benchmarks (meet budget targets from cli-tool.md)

### Recommended Crate Dependencies

- `clap` - CLI argument parsing
- `serde`, `serde_yaml`, `serde_json` - serialization
- `toml` - config parsing
- `chrono` - timestamps
- `uuid` or custom hash - ID generation
- `regex` - link extraction
- `walkdir` - directory traversal
- `anyhow` or `thiserror` - error handling

### Performance Targets (from cli-tool.md)

- `qipu --help` / `--version`: < 100ms
- `qipu list` (~1k notes): < 200ms
- `qipu search` (~10k notes): < 1s (with indexes)
