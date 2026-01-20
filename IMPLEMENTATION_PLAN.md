# Qipu Implementation Plan

## Status
- Test baseline: `cargo test` passes (456 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes

---

## Completed Work

### Qipu Core
- File size refactoring (context/output, graph/traversal, link/list, link/path, db/notes, doctor/checks)
- Structured logging for all commands
- Unit tests for list command, CLI tests for workspace commands

### Value Model (`specs/value-model.md`)
- Data model: `value` field (0-100) in frontmatter, schema migration v1→v2
- CLI: `qipu value set/show`, `--min-value` filter on list/search/context/link tree/link path
- Weighted traversal: Dijkstra algorithm, `--ignore-value` flag, edge cost formula
- Doctor validation for value range (0-100)

### LLM Tool Test Harness (`crates/llm-tool-test`)
- Scenarios: tier 0/1 scenarios, setup step support
- Gate types: NoteExists, LinkExists, TagExists, ContentContains, CommandSucceeds
- LLM judge: semantic quality rubric, weighted composite score, score thresholds
- Human review: `review` command, `--pending-review` flag
- Test infrastructure: mock adapter, end-to-end tests
- CLI: `run --all`, `baseline set/clear/list`, file refactoring (commands.rs, output.rs, run.rs)

---

## Future Work

*(Add new items here as needed)*

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
