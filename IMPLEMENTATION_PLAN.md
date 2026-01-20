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
- [x] `src/commands/doctor/checks.rs` (402 lines) → Group by category

### Structured Logging Gaps
Commands missing tracing instrumentation:
- [x] `src/commands/capture.rs`
- [x] `src/commands/create.rs`
- [x] `src/commands/search.rs`
- [x] `src/commands/compact/*.rs` (7 files) - completed 2026-01-19
- [x] `src/commands/context/{budget,output,select,types}.rs` - completed 2026-01-19
- [x] `src/commands/workspace/{list,merge,new}.rs` - completed 2026-01-19

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
- [x] Add weighted composite score combining automated + judge metrics
- [x] Define score thresholds: Excellent (0.9+), Good (0.7-0.9), Acceptable (0.5-0.7), Poor (<0.5)

### Human Review Integration (P2)
- [x] Add `review <RUN_ID> --dimension key=value --notes "..."` subcommand
- [x] Add `list --pending-review` to find unreviewed runs
- [x] Store human scores in results record

### Test Infrastructure (P2)
- [x] Add tests for command parsing from transcript
- [x] Add tests for `build_tool_matrix()` edge cases
- [x] Add mock adapter for offline testing
- [x] Add end-to-end test with mock adapter
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

## Value Model (`specs/value-model.md`) (P2)

Adds a `value` field (0-100, default 50) to notes for quality/importance scoring, enabling weighted graph traversal.

### Phase 1: Data Model
- [ ] Add `value` field to `NoteFrontmatter` in `src/lib/note/frontmatter.rs`
  - Type: `Option<u8>`, serde skip_serializing_if None
  - Update `NoteFrontmatter::new()` to initialize as None
- [ ] Schema migration in `src/lib/db/schema.rs`
  - Add `value INTEGER DEFAULT 50` column to `notes` table
  - Add index: `CREATE INDEX idx_notes_value ON notes(value)`
  - Bump `CURRENT_SCHEMA_VERSION` to 2
  - Add migration path from v1 → v2
- [ ] Update `src/lib/db/notes.rs` to read/write `value` column
- [ ] Update `src/lib/index/builder.rs` to index `value` field

### Phase 2: CLI Commands
- [ ] Add `qipu value` subcommand in `src/cli/commands.rs`
  - `value set <id> <score>` - update frontmatter value field
  - `value show <id>` - display current value (or "50 (default)" if unset)
- [ ] Add `--min-value <n>` filter flag to:
  - `qipu list`
  - `qipu search`
  - `qipu link tree`
  - `qipu link path`
  - `qipu context`
- [ ] Add `--sort value` option to `qipu search`

### Phase 3: Weighted Traversal
- [ ] Add `get_edge_cost(link_type, target_value)` in `src/lib/graph/types.rs`
  - Formula: `LinkTypeCost * (1 + (100 - value) / 100)`
  - Composes with future per-link-type costs (see `specs/semantic-graph.md` §3.A)
- [ ] Add `--ignore-value` flag to `TreeOptions` in `src/lib/graph/types.rs`
- [ ] Implement Dijkstra traversal variant in `src/lib/graph/bfs.rs`
  - New function `dijkstra_traverse()` using `BinaryHeap` instead of `VecDeque`
  - Order by accumulated cost (min-heap)
  - Default behavior: weighted (Dijkstra)
  - With `--ignore-value`: unweighted (BFS, all edges cost 1.0)
- [ ] Update `bfs_find_path()` to support weighted mode

### Phase 4: Integration
- [ ] Update `qipu context` to respect `--min-value` threshold
- [ ] Update `qipu doctor` to validate value range (0-100)
- [ ] Add tests for value filtering and weighted traversal
- [ ] Update help text and man pages

### Dependencies
- Builds on existing `HopCost` infrastructure (`src/lib/graph/types.rs`)
- Complements compaction system (`specs/compaction.md`) - low-value notes are compaction candidates

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

### Human Review Integration (completed 2026-01-19)
- Added `HumanReviewRecord` struct with dimension scores (HashMap<String, f64>) and optional notes
- Implemented `ResultsDB::update_human_review()` with atomic file updates using temporary file pattern
- Implemented `ResultsDB::load_pending_review()` to filter runs without human reviews
- Added CLI `review` command with `--dimension key=value` parser and `--notes` flag
- Added `--pending-review` flag to `list` command
- Updated `show` command to display human review data with dimension scores and timestamp
- Added 14 tests covering all new functionality (DB methods, CLI parser, serialization)
- All 111 tests in llm-tool-test pass, 250+ tests in entire codebase

### Doctor Checks Refactoring (completed 2026-01-19)
- Split `src/commands/doctor/checks.rs` (403 lines) into three focused modules:
  - `structure.rs` (store directory structure checks)
  - `database.rs` (database consistency checks: duplicate IDs, missing files, broken/orphaned notes)
  - `content.rs` (note validation: scan, required fields, compaction invariants, duplicates, attachments)
- Used re-exports in `checks.rs` for backward compatibility with existing code
- All 229 tests pass after refactoring

### Semantic Quality Evaluation (completed 2026-01-19)
- The existing judge system already supports flexible rubrics via YAML configuration
- Created `crates/llm-tool-test/fixtures/qipu/rubrics/semantic_quality.yaml` with three criteria:
  - Relevance (0.35): Notes directly address the task prompt
  - Coherence (0.35): Notes are logically connected with consistent terminology
  - Granularity (0.30): Notes are appropriately scoped, neither too broad nor too fragmented
- Rubric can be used by scenarios by adding `evaluation.judge` configuration
- Added tests to verify rubric loading works correctly and validates weight sums

### Weighted Composite Score (completed 2026-01-19)
- Added `composite_score` field to `EvaluationMetrics` and `EvaluationMetricsRecord` structs
- Implemented `compute_composite_score()` function with weighted components:
  - Judge score (0.50): LLM-as-judge semantic evaluation
  - Gate pass rate (0.30): Automated functional correctness checks
  - First try success rate (0.10): Command efficiency
  - Quality score (0.10): Store health (tags, links, orphan penalty)
- Quality component calculation:
  - Tags score: avg_tags_per_note (clamped at 3.0) / 3.0
  - Links score: links_per_note (clamped at 2.0) / 2.0
  - Orphan penalty: (orphan_notes / total_notes) * 0.3
  - Quality = (tags_score + links_score) / 2.0 - orphan_penalty
- Composite score is clamped to [0.0, 1.0] range
- All 91 tests pass, including 4 new composite score tests

### Workspace Commands Tracing (completed 2026-01-19)
- Added structured logging to `src/commands/workspace/list.rs`, `merge.rs`, and `new.rs`
- Instrumentation follows the same pattern as other commands:
  - Import `tracing::debug` and `std::time::Instant`
  - Log key parameters at function entry when verbose
  - Log intermediate milestones (store discovery, workspace initialization, note copying)
  - Log completion with elapsed time
- All 438 tests pass (189 unit + 229 CLI + 6 golden + 6 pack + 6 perf + 3 workspace merge)
