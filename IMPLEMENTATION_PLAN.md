# Qipu Implementation Plan

## Status (Last Audited: 2026-01-19)
- Test baseline: `cargo test` passes (351/351 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes

---

## Remaining Work

### Quality Review (2026-01-19)

#### File Size Refactoring (P2)
Large files that should be split for maintainability:
- [ ] `src/commands/context/output.rs` (668 lines) - Split into: `json.rs`, `human.rs`, `records.rs`
- [ ] `src/lib/graph/traversal.rs` (470 lines) - Extract BFS/Dijkstra into separate modules
- [ ] `src/commands/link/list.rs` (454 lines) - Extract output formatters
- [ ] `src/commands/link/path.rs` (450 lines) - Extract output formatters
- [ ] `src/lib/db/notes.rs` (432 lines) - Consider splitting CRUD operations
- [ ] `src/commands/doctor/checks.rs` (402 lines) - Group checks by category

#### Structured Logging Gaps (P2)
Commands missing tracing instrumentation (39 files):
- [ ] `src/commands/capture.rs` - Add timing span for capture operation
- [ ] `src/commands/create.rs` - Add timing span for note creation
- [ ] `src/commands/search.rs` - Add timing span for search execution
- [ ] `src/commands/compact/*.rs` (7 files) - Add timing for compaction ops
- [ ] `src/commands/context/{budget,output,select,types}.rs` - Add timing spans
- [ ] `src/commands/workspace/{list,merge,new}.rs` - Add timing spans
- [ ] Lower priority: doctor, dump, export, link, load submodules

#### Test Coverage Gaps (P2)
Command files with no unit tests (integration tests may exist):
- [ ] Add unit tests to `src/commands/search.rs` (350 lines, high-value)
- [ ] Add unit tests to `src/commands/show.rs` (366 lines)
- [ ] Add unit tests to `src/commands/setup.rs` (378 lines)
- [ ] Add unit tests to `src/commands/list.rs` (231 lines)
- [ ] Add CLI test file for workspace commands (`tests/cli/workspace.rs`)
  - Existing `tests/workspace_merge_test.rs` covers merge only
  - Missing: `new`, `list`, `delete` command tests

#### eprintln! Remaining (P3)
4 remaining `eprintln!` calls in main.rs are appropriate for fatal error output:
- Lines 48, 59, 72, 74 - Pre-logging initialization errors and JSON error output
- **Status: ACCEPTABLE** - These run before tracing is configured

### Low Priority (P3)

#### Verbose Timing Keys
- [x] Add timing spans for `load_indexes` and `execute_command` phases
  - Added to all dispatch handlers (execute_command)
  - Added to commands that build indexes (load_indexes): dump, export, link list/tree/path, show, context, inbox
  - Files: `src/commands/dispatch/mod.rs`, `src/commands/dispatch/*.rs`, `src/commands/*/*.rs`
  - Implementation: debug logs with elapsed time, similar to discover_store pattern

#### eprintln! Cleanup
- [x] Replace 16 remaining `eprintln!` calls with tracing
  - Callsites in: main.rs, export/mod.rs, compact/apply.rs, workspace/delete.rs, dump/mod.rs, export/emit/outline.rs
  - Replaced with tracing::info! for verbose warnings, tracing::warn! for errors
  - Updated test expectation for workspace/delete warnings (now in stdout via tracing)

#### Startup Validation
- [x] Call `validate_consistency()` during DB open
  - Method exists at `src/lib/db/validate.rs:104-166` but marked `#[allow(dead_code)]`
  - File: `src/lib/db/mod.rs:69-83`
  - Implementation: Removed `#[allow(dead_code)]` attribute, added validation call after rebuild check
  - Validation runs when database has notes, logs warnings on inconsistencies

#### LLM Tool Test Harness
- [ ] Fix tool default (should be "amp", currently "opencode")
  - File: `crates/llm-tool-test/src/cli.rs:23`
- [ ] Add missing scenario schema fields (id, tags, docs.prime, setup, tool_matrix)
- [ ] Add more test scenarios

#### Workspace Tests
- [ ] Add `--dry-run` conflict report test
- [ ] Add `--empty` flag test

---

## Technology Reference

### Database
- **SQLite** with `rusqlite` (bundled), WAL mode, FTS5 with porter tokenizer
- Schema: notes, notes_fts, tags, edges, unresolved, index_meta tables
- Location: `.qipu/qipu.db`

### Logging
- **tracing** ecosystem with env-filter and json features
- Flags: `--verbose`, `--log-level`, `--log-json`
- Env: `QIPU_LOG` override

---

## Completed (Reference)

Core features all implemented and tested:
- SQLite FTS5 migration (ripgrep removed)
- Search ranking with BM25, recency boost, field weighting
- Graph traversal with semantic inversion, weighted costs
- Pack dump/load with all conflict strategies
- Export with MOC ordering, anchor rewriting, attachments
- Context command with budget, transitive, backlinks, related
- Compaction commands and global flags
- Provenance fields and verification
- Similarity with Porter stemming and stop words
