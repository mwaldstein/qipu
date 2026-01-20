# Qipu Implementation Plan

## Status
- Test baseline: `cargo test` passes (189 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes

---

## Qipu Core (P2)

### File Size Refactoring
Large files to split for maintainability:
- [x] `src/commands/context/output.rs` (668 lines) → `json.rs`, `human.rs`, `records.rs`
- [x] `src/lib/graph/traversal.rs` (470 lines) → Extract BFS module (bfs.rs)
- [x] `src/commands/link/list.rs` (454 lines) → Extract output formatters
- [x] `src/commands/link/path.rs` (450 lines) → Extract output formatters
- [x] `src/lib/db/notes.rs` (432 lines) → Split CRUD operations
- [ ] `src/commands/doctor/checks.rs` (402 lines) → Group by category

### Structured Logging Gaps
Commands missing tracing instrumentation:
- [ ] `src/commands/capture.rs`, `create.rs`, `search.rs`
- [ ] `src/commands/compact/*.rs` (7 files)
- [ ] `src/commands/context/{budget,output,select,types}.rs`
- [ ] `src/commands/workspace/{list,merge,new}.rs`

### Test Coverage Gaps
- [x] Add unit tests to `src/commands/list.rs` (231 lines)
- [x] Add CLI tests for workspace commands (`new`, `list`, `delete`)

---

## LLM Tool Test Harness (`crates/llm-tool-test`)

### Scenarios (P1)
- [x] Create tier 0 (smoke) scenario: single `qipu create` command
- [x] Create tier 1 (quick) scenarios: basic capture, simple linking
- [x] Add scenarios: `search_basic`, `context_retrieval`, `compaction_workflow`
- [x] Add `setup` step support (pre-populate store with seed notes)

### Gate Types (P1)
Current: `MinNotes`, `MinLinks`, `SearchHit`, `NoteExists`, `LinkExists`, `TagExists`, `ContentContains`, `CommandSucceeds`
- [x] Add `NoteExists { id }` - verify specific note created
- [x] Add `LinkExists { from, to, link_type }` - verify specific link
- [x] Add `TagExists { tag }` - verify tag usage
- [x] Add `ContentContains { id, substring }` - verify note content
- [x] Add `CommandSucceeds { command }` - arbitrary qipu command

### LLM Judge Enhancements (P2)
- [x] Add semantic quality evaluation (relevance, coherence, granularity)
- [ ] Add weighted composite score combining automated + judge metrics
- [ ] Define score thresholds: Excellent (0.9+), Good (0.7-0.9), Acceptable (0.5-0.7), Poor (<0.5)

### Human Review Integration (P2)
- [ ] Add `review <RUN_ID> --dimension key=value --notes "..."` subcommand
- [ ] Add `list --pending-review` to find unreviewed runs
- [ ] Store human scores in results record

### Test Infrastructure (P2)
- [x] Add tests for command parsing from transcript
- [x] Add tests for `build_tool_matrix()` edge cases
- [ ] Add mock adapter for offline testing
- [ ] Add end-to-end test with mock adapter
- [ ] Remove dead code: `ResultsDB::load_latest_by_scenario`

### File Refactoring (P3)
- [ ] Extract `run_single_scenario` from `main.rs` into `run.rs`
- [ ] Extract command handlers into `commands.rs`
- [ ] Extract print functions into `output.rs`

### CLI Polish (P3)
- [ ] Rename `list` to show scenarios not runs
- [ ] Add `run --all` to run all scenarios
- [ ] Add `baseline set <run_id>` command

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

## Learnings

### Semantic Quality Evaluation (completed 2026-01-19)
- The existing judge system already supports flexible rubrics via YAML configuration
- Created `crates/llm-tool-test/fixtures/qipu/rubrics/semantic_quality.yaml` with three criteria:
  - Relevance (0.35): Notes directly address the task prompt
  - Coherence (0.35): Notes are logically connected with consistent terminology
  - Granularity (0.30): Notes are appropriately scoped, neither too broad nor too fragmented
- Rubric can be used by scenarios by adding `evaluation.judge` configuration
- Added tests to verify rubric loading works correctly and validates weight sums
