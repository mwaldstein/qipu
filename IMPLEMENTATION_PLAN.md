# Qipu Implementation Plan

## Status
- **Last audited:** 2026-01-20
- Test baseline: `cargo test` passes (483 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes

---

## P1: Correctness Bugs

### value-model.md

- [ ] `link tree` and `link path` use BFS instead of Dijkstra (weighted traversal)
  - Spec (line 95-103): "By default, traversal commands use **weighted traversal**"
  - Current: `src/commands/link/tree.rs:61` calls `bfs_traverse()` (unweighted)
  - Current: `src/commands/link/path.rs:75` calls `bfs_find_path()` (unweighted)
  - Should call: `dijkstra_traverse()` / equivalent weighted path function
  - Impact: Value-based edge costs are not applied in default mode

### semantic-graph.md

- [ ] Context budget doesn't prefer typed links over `related`
  - Spec (line 82-83): "When generating `qipu context`, strongly prefer typed links (especially `part-of` and `supports`) over generic `related`"
  - Location: `src/commands/context/budget.rs`
  - Gap: Budget handling doesn't differentiate link types

---

## P2: Missing Test Coverage

### workspaces.md

- [ ] Test `workspace merge --dry-run` shows conflict report
  - Implementation: `src/commands/workspace/merge.rs:110-131`
  - Gap: No CLI test verifies dry-run output format

- [ ] Test `workspace merge --strategy overwrite`
  - Implementation: `src/commands/workspace/merge.rs:74-79`
  - Gap: Strategy not explicitly tested (only merge-links tested)

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
  - Implementation: `src/commands/export/plan.rs:31-40`
  - Gap: No CLI test in `tests/cli/export.rs`

- [ ] Test `--query` selection for export
  - Implementation: `src/commands/export/plan.rs:52-62`
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

- [ ] Test attachment roundtrip (dump/load with actual files)
  - Gap: No test with real attachment files

### compaction.md

- [ ] Test `qipu compact guide` command
  - Implementation: `src/commands/compact/guide.rs:1-81`
  - Gap: No dedicated test in `tests/cli/compact/commands.rs`

### value-model.md

- [ ] Test `qipu value set` validation (score > 100 rejection)
  - Implementation: `src/commands/dispatch/mod.rs:320-343`
  - Gap: No dedicated validation test

- [ ] Test `qipu value show` output format ("(default)" annotation)
  - Implementation: `src/commands/dispatch/mod.rs:346-365`
  - Gap: No dedicated format test

- [ ] Test doctor validation for notes with invalid value
  - Implementation: `src/commands/doctor/content.rs:130-149`
  - Gap: No CLI test in `tests/cli/doctor.rs`

### structured-logging.md

- [ ] Test `--log-level trace` output
  - Gap: Only debug and warn levels tested in `tests/cli/logging.rs`

- [ ] Test default quiet behavior (no flags = no log output)
  - Gap: No explicit test

---

## P3: Unimplemented but Ready

### value-model.md

- [ ] Add `--ignore-value` / `--unweighted` CLI flag for `link tree` and `link path`
  - Spec: `specs/value-model.md:100-104`
  - Infrastructure exists: `src/lib/graph/types.rs:238-239` (`ignore_value` in TreeOptions)
  - Gap: No CLI flag exposed in `src/cli/link.rs`

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

- [ ] Add "Last updated" column to `workspace list` output
  - Spec: line 55
  - Gap: Column not displayed in `src/commands/workspace/list.rs:73-85`

### distribution.md

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
