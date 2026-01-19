# Qipu Implementation Plan

## Status (Last Audited: 2026-01-19)
- Test baseline: `cargo test` passes (351/351 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes

---

## Remaining Work

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
- [ ] Call `validate_consistency()` during DB open
  - Method exists at `src/lib/db/validate.rs:104-166` but marked `#[allow(dead_code)]`
  - File: `src/lib/db/mod.rs:69-83`

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
