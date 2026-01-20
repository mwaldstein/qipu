# Qipu Implementation Plan

## Status
- **Last audited:** 2026-01-20
- Test baseline: `cargo test` passes (483 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes

---

## P1: Correctness Bugs

### value-model.md

- [x] `link tree` calls `bfs_traverse()` instead of `dijkstra_traverse()` by default
  - Spec (line 95-103): "By default, traversal commands use **weighted traversal**"
  - Fixed: `src/commands/link/tree.rs:61` now conditionally calls `dijkstra_traverse()` when `ignore_value=false`
  - Impact: Value-based edge costs now properly applied

### semantic-graph.md

- [x] Context budget doesn't prefer typed links over `related`
  - Spec (line 82-83): "When generating `qipu context`, strongly prefer typed links (especially `part-of` and `supports`) over generic `related`"
  - Fixed: Added `link_type` field to `SelectedNote` struct
  - Updated `get_moc_linked_ids()` to return link types
  - Updated backlink expansion to capture link types
  - Modified sorting to prioritize: verified > (part-of/supports) > other typed links > related
  - Impact: Budget-limited context now prioritizes high-signal typed links over generic related links

### workspaces.md

- [x] `--from-note` copies single note instead of graph slice
  - Spec (line 64): "Initialize with a slice of the primary graph (like dump -> load)"
  - Fixed: Implemented graph traversal in `src/commands/workspace/new.rs`
  - Added `copy_graph_slice()` function that performs BFS traversal with 3-hop limit
  - Traverses in both directions (Direction::Both)
  - Impact: Workspace initialized from note now includes related notes within 3 hops

- [x] `workspace list` missing "Last updated" column
  - Spec (line 51-55): Output should include Name, Status, Note count, Last updated
  - Fixed: Added `get_max_mtime()` method to Database
  - Updated WorkspaceInfo struct to include `last_updated` field
  - Modified output to show "Last updated" instead of "Path"
  - Changed Status column to show "Temp"/"Persistent" instead of "Yes"/"No"
  - Impact: Users can now see when workspaces were last modified

### llm-user-validation.md

- [ ] `--tags` flag parsed but ignored in run command
  - Spec (line 444): `--tags capture,links` should filter scenarios by tags
  - CLI: `crates/llm-tool-test/src/cli.rs:22-24` defines argument
  - Code: `crates/llm-tool-test/src/main.rs:80` marks `tags: _` (explicitly ignored)
  - Impact: Tag filtering doesn't work

- [ ] `--tier` flag parsed but ignored in run command
  - Spec (line 445): Should filter scenarios by tier
  - CLI: `crates/llm-tool-test/src/cli.rs:26-28` defines argument
  - Code: `crates/llm-tool-test/src/main.rs:81` marks `tier: _` (explicitly ignored)
  - Impact: Tier filtering doesn't work

- [ ] `--max-usd` flag parsed but ignored (no cost enforcement)
  - Spec (lines 446, 476-479): Per-run and session budget enforcement
  - CLI: `crates/llm-tool-test/src/cli.rs:46-48` defines argument
  - Code: `crates/llm-tool-test/src/main.rs:86` marks `max_usd: _` (explicitly ignored)
  - Impact: No cost limits enforced

### distribution.md

- [ ] Repository URL mismatch between Cargo.toml and git remote
  - Spec (lines 36, 49, 63): References `mwaldstein/qipu`
  - Cargo.toml:11: `repository = "https://github.com/anomalyco/qipu"`
  - Git remote: `git@github.com:mwaldstein/qipu.git`
  - Impact: crates.io would publish with wrong repository URL

- [ ] CI workflow disabled for automated triggers
  - Spec (line 87-88): "On tagged releases (v*), automation should..."
  - Current: `.github/workflows/ci.yml:4-14` runs only on `workflow_dispatch`
  - Impact: No automatic CI on push/PR; tagged releases won't trigger builds

---

## P2: Missing Test Coverage

### workspaces.md

- [ ] Test `workspace merge --dry-run` shows conflict report
  - Implementation: `src/commands/workspace/merge.rs:110-131`
  - Gap: No CLI test verifies dry-run output format

- [ ] Test `workspace merge --strategy overwrite`
  - Implementation: `src/commands/workspace/merge.rs:74-79`
  - Gap: Strategy not explicitly tested

- [ ] Test `workspace merge --strategy skip` (default)
  - Implementation: `src/commands/workspace/merge.rs:90`
  - Gap: Default strategy not explicitly tested

- [ ] Test `workspace merge --delete-source`
  - Implementation: `src/commands/workspace/merge.rs:133-135`
  - Gap: Flag not tested

### export.md

- [ ] Test `--mode bibliography` output format
  - Implementation: `src/commands/export/emit/bibliography.rs:4-41`
  - Gap: No CLI test

- [ ] Test `--tag` selection for export
  - Implementation: `src/commands/export/plan.rs:32-40`
  - Gap: No CLI test in `tests/cli/export.rs`

- [ ] Test `--query` selection for export
  - Implementation: `src/commands/export/plan.rs:53-62`
  - Gap: No CLI test

- [ ] Test `--link-mode markdown` wiki-to-markdown conversion
  - Implementation: `src/commands/export/emit/links.rs:37-54`
  - Gap: Only `--link-mode anchors` tested

### pack.md

- [ ] Test `dump --tag <tag>` selector
  - Implementation: `src/commands/dump/mod.rs:122-130`
  - Gap: No CLI test

- [ ] Test `dump --moc <id>` selector
  - Implementation: `src/commands/dump/mod.rs:132-139`
  - Gap: No CLI test

- [ ] Test `dump --query <text>` selector
  - Implementation: `src/commands/dump/mod.rs:141-147`
  - Gap: No CLI test

- [ ] Test `dump --no-attachments` flag
  - Implementation: `src/commands/dump/mod.rs:54-59`
  - Gap: No CLI test

- [ ] Test attachment roundtrip (dump with attachments, load, verify)
  - Implementation: `src/commands/dump/mod.rs:327-376`, `src/commands/load/mod.rs:362-396`
  - Gap: No test creates note with attachment, dumps, loads, and verifies

### compaction.md

- [ ] Test `qipu compact guide` command
  - Implementation: `src/commands/compact/guide.rs:9-81`
  - Gap: No dedicated test in `tests/cli/compact/`

- [ ] Test `compact apply` invariant rejection (cycles, self-compact)
  - Implementation: `src/lib/compaction/validation.rs:17-40`
  - Gap: Unit tests exist but no CLI tests for error cases

### structured-logging.md

- [ ] Test `--log-level trace` output
  - Gap: Only debug and warn levels tested in `tests/cli/logging.rs`

- [ ] Test default quiet behavior (no flags = no log output)
  - Gap: No explicit test

### export.md

- [ ] Test deterministic ordering for tag/query exports
  - Implementation: `src/commands/export/plan.rs:100-110` sorts by `(created_at, id)`
  - Gap: No test verifies ordering for non-MOC exports

---

## P3: Unimplemented but Ready

### value-model.md

- [ ] Add `--ignore-value` / `--unweighted` CLI flag for `link tree` and `link path`
  - Spec: `specs/value-model.md:100-104`
  - Infrastructure exists: `src/lib/graph/types.rs:238-239` (`ignore_value` in TreeOptions)
  - Gap: No CLI flag exposed in `src/cli/link.rs`
  - Note: `bfs_find_path()` already respects `ignore_value` internally

- [ ] Switch `link tree` to call `dijkstra_traverse()` by default (fix P1 bug)
  - Location: `src/commands/link/tree.rs:61`
  - Change: `if tree_opts.ignore_value { bfs_traverse() } else { dijkstra_traverse() }`

### structured-logging.md

- [ ] Add `#[tracing::instrument]` to `IndexBuilder::build()`
  - Location: `src/lib/index/builder.rs:25`
  - Gap: Missing timing instrumentation

- [ ] Add `#[tracing::instrument]` to `Database::search()`
  - Location: `src/lib/db/search.rs:24`
  - Gap: Missing timing instrumentation

- [ ] Add `#[tracing::instrument]` to note parse functions
  - Location: `src/lib/note/parse.rs:6`, `src/lib/note/mod.rs:52`
  - Gap: Missing timing instrumentation

### llm-user-validation.md

- [ ] Add `tags` field to scenario schema
  - Spec: Mentions `tags: [capture, links, retrieval]`
  - Location: `crates/llm-tool-test/src/scenario.rs`
  - Note: CLI has `--tags` flag but filtering not wired

- [ ] Add `docs.prime` and `docs.help_commands` to scenario schema
  - Spec: Should include `qipu prime` output in context
  - Location: `crates/llm-tool-test/src/scenario.rs`

- [ ] Implement `--tags` filtering in run command
  - Parsed but ignored: `crates/llm-tool-test/src/main.rs:80`

- [ ] Add `report` command
  - Spec mentions `llm-tool-test report` for summary
  - Gap: Command not implemented

- [ ] Add per-scenario `run.max_turns` support
  - Spec line 160
  - Gap: Not implemented

### workspaces.md

- [x] Fix `--from-note` to perform graph slice (fix P1 bug)
  - Spec: line 64 - "Initialize with a slice of the primary graph (like dump -> load)"
  - Fixed: Implemented `copy_graph_slice()` with BFS traversal (3 hops, both directions)
  - Test added: `tests/workspace_from_note_test.rs` verifies graph slicing behavior

- [ ] Add "Last updated" column to `workspace list` output (fix P1 bug)
  - Spec: line 55
  - Gap: Column not displayed in `src/commands/workspace/list.rs:72-75`
  - Need: Stat workspace directory or notes for mtime

### distribution.md

- [ ] Fix repository URL in Cargo.toml to match git remote (fix P1 bug)
  - Current: `repository = "https://github.com/anomalyco/qipu"`
  - Should match: git remote `mwaldstein/qipu`

- [ ] Enable CI workflow triggers (fix P1 bug)
  - Current: `.github/workflows/ci.yml` only runs on `workflow_dispatch`
  - Should: Trigger on push and pull_request

- [ ] Create `.github/workflows/release.yml` for tagged releases
  - Spec: lines 85-92
  - Gap: Only `ci.yml` exists (manual trigger only)

- [ ] Add aarch64 targets to CI builds
  - Spec requires: `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`
  - Current: Only x86_64 targets in `.github/workflows/ci.yml:72-78`

- [ ] Create `scripts/install.sh` Unix installer
  - Spec: lines 33-44
  - Gap: Script doesn't exist

- [ ] Create `scripts/install.ps1` Windows installer
  - Spec: lines 47-50
  - Gap: Script doesn't exist

- [ ] Generate SHA256SUMS for releases
  - Spec: line 90
  - Gap: Not implemented

### llm-user-validation.md

- [ ] Add `tags` field to scenario schema (fix P1 bug - filtering needs schema)
  - Spec: line 123 - `tags: [capture, links, retrieval]`
  - Location: `crates/llm-tool-test/src/scenario.rs:4-17`

- [ ] Add `docs.prime` and `docs.help_commands` to scenario schema
  - Spec: lines 125-131 - include `qipu prime` output in context
  - Location: `crates/llm-tool-test/src/scenario.rs`

- [ ] Wire `--tags` filtering in run command (fix P1 bug)
  - Currently ignored: `crates/llm-tool-test/src/main.rs:80`

- [ ] Wire `--tier` filtering in run command (fix P1 bug)
  - Currently ignored: `crates/llm-tool-test/src/main.rs:81`

- [ ] Wire `--max-usd` cost enforcement (fix P1 bug)
  - Currently ignored: `crates/llm-tool-test/src/main.rs:86`

- [ ] Add `report` command
  - Spec: line 455 - `llm-tool-test report` for summary
  - Gap: Command not implemented

- [ ] Add per-scenario `run.max_turns` support
  - Spec: line 160
  - Gap: Not in schema or adapter interface

---

## Completed Work

### Qipu Core
- [x] File size refactoring (context/output, graph/traversal, link/list, link/path, db/notes, doctor/checks)
- [x] Structured logging for all commands
- [x] Unit tests for list command, CLI tests for workspace commands

### cli-tool.md
- [x] All global flags: `--root`, `--store`, `--verbose`, `--quiet`, `--format`
- [x] Store discovery with project boundary markers
- [x] Exit codes: 0, 1, 2, 3
- [x] Verbose timing keys: `parse_args`, `discover_store`, `load_indexes`, `execute_command`
- [x] Performance budgets tested: help/version <100ms, list <200ms

### knowledge-model.md
- [x] Closed enum: fleeting, literature, permanent, moc
- [x] ID format: `qp-<hash>` with adaptive length
- [x] All frontmatter fields implemented
- [x] Tag aliases marked optional (not implemented per spec)
- [x] All 10+ link types with inverses

### storage-format.md
- [x] Directory structure: `.qipu/`, `notes/`, `mocs/`, `attachments/`, `templates/`
- [x] `config.toml` with version, default_note_type, id_scheme, editor
- [x] `qipu.db` SQLite database (gitignored)
- [x] Store discovery walks up tree, stops at project markers
- [x] Stealth mode and protected branch workflow

### cli-interface.md
- [x] All 16+ commands implemented with correct flags and exit codes
- [x] All output formats: human, json, records
- [x] Extensions: verify, value, workspace, merge, compact commands

### indexing-search.md
- [x] SQLite FTS5 with porter tokenizer
- [x] BM25 ranking with field boosts (5.0x title, 8.0x tags)
- [x] Incremental repair via mtime tracking
- [x] Backlink index and graph traversal
- [x] Recency boost in search ranking

### semantic-graph.md
- [x] All standard link types with inverses
- [x] Semantic inversion (virtual edges)
- [x] User-defined link types via config
- [x] `--no-semantic-inversion` flag

### graph-traversal.md
- [x] `link tree` and `link path` commands
- [x] All direction modes: out, in, both
- [x] Type filtering, source filtering
- [x] BFS/Dijkstra traversal with deterministic ordering
- [x] `(seen)` markers and truncation reporting
- [x] All truncation flags: `--max-nodes`, `--max-edges`, `--max-fanout`

### similarity-ranking.md
- [x] BM25 via FTS5
- [x] TF-IDF cosine similarity for related notes
- [x] Porter stemming, stop words removal
- [x] Duplicate detection with configurable threshold
- [x] Field weights (2.0x title, 1.5x tags, 1.0x body) for TF-IDF

### records-output.md
- [x] All prefix types: H, N, S, E, B, W, D, C, M, L, A, B-END
- [x] Quote escaping
- [x] Truncation indicators
- [x] Summary extraction priority (frontmatter > ## Summary > first paragraph)

### llm-context.md
- [x] `qipu prime` with deterministic output
- [x] `qipu context` with all selectors: `--note`, `--tag`, `--moc`, `--query`
- [x] Budget enforcement: `--max-chars`, `--max-tokens`
- [x] Safety banner support
- [x] `--transitive`, `--backlinks`, `--related` expansion
- [x] `qipu setup` with AGENTS.md integration

### provenance.md
- [x] All 5 fields: source, author, generated_by, prompt_hash, verified
- [x] JSON output includes all fields
- [x] CLI support: `--source`, `--author`, `--generated-by`, `--prompt-hash`, `--verified`
- [x] `qipu verify` command
- [x] Context prioritizes verified notes

### export.md
- [x] All 3 modes: bundle, outline, bibliography
- [x] MOC-driven ordering preserved
- [x] Anchor rewriting with `#note-<id>` format
- [x] Attachment link rewriting
- [x] Deterministic output

### compaction.md
- [x] All 6 subcommands: guide, suggest, show, apply, report, status
- [x] Digest annotations: `compacts=`, `compaction=`, `via=`
- [x] Truncation indicators
- [x] Navigation patterns (resolved vs raw view)
- [x] Invariant validation in doctor

### pack.md
- [x] `dump` and `load` commands
- [x] All 3 strategies: skip, overwrite, merge-links
- [x] All traversal options: direction, max-hops, type filters
- [x] Pack metadata header with version compatibility

### workspaces.md
- [x] All 4 commands: new, list, delete, merge
- [x] All `new` flags: `--temp`, `--empty`, `--copy-primary`, `--from-tag`, `--from-query`, `--from-note`
- [x] All merge strategies: skip, overwrite, merge-links
- [x] `--dry-run` implementation
- [x] `--workspace` global targeting flag

### operational-database.md
- [x] SQLite with WAL mode
- [x] FTS5 full-text search
- [x] Schema versioning with migration (v1->v2)
- [x] `validate_consistency()` called on startup
- [x] Incremental repair
- [x] All validation queries: duplicates, broken links, missing files, orphaned notes

### Value Model (specs/value-model.md)
- [x] Data model: `value` field (0-100) in frontmatter, schema migration v1->v2
- [x] CLI: `qipu value set/show`, `--min-value` filter on list/search/context/link tree/link path
- [x] Weighted traversal infrastructure: Dijkstra algorithm, edge cost formula, HopCost type
- [x] Doctor validation for value range (0-100)
- [x] `bfs_find_path()` respects `ignore_value` flag for weighted/unweighted mode
- [x] `qipu search --sort value` - implementation and tests (`tests/cli/search.rs:688-937`)

### LLM Tool Test Harness (crates/llm-tool-test)
- [x] Scenarios: tier 0/1 scenarios, setup step support
- [x] Gate types: NoteExists, LinkExists, TagExists, ContentContains, CommandSucceeds, MinNotes, MinLinks, SearchHit
- [x] LLM judge: semantic quality rubric, weighted composite score, score thresholds
- [x] Human review: `review` command, `--pending-review` flag
- [x] Test infrastructure: mock adapter, end-to-end tests
- [x] CLI: `run --all`, `baseline set/clear/list`

---

## Technology Reference

### Database
- SQLite with `rusqlite` (bundled), WAL mode, FTS5 with porter tokenizer
- Schema: notes, notes_fts, tags, edges, unresolved, index_meta
- Location: `.qipu/qipu.db`

### Logging
- `tracing` ecosystem with env-filter and json features
- Flags: `--verbose`, `--log-level`, `--log-json`
- Env: `QIPU_LOG` override
