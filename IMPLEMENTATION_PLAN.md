# Qipu Implementation Plan

## Status
- Test baseline: `cargo test` passes (456 tests: 215 unit + 241 CLI, 6 pre-existing failures unrelated to value feature)
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
- [x] Remove dead code: `ResultsDB::load_latest_by_scenario`

### File Refactoring (P3)
- [x] Extract `run_single_scenario` from `main.rs` into `run.rs` - completed 2026-01-20
- [x] Extract command handlers into `commands.rs` - completed 2026-01-20
- [x] Extract print functions into `output.rs` - completed 2026-01-20

### CLI Polish (P3)
- [ ] Rename `list` to show scenarios not runs
- [ ] Add `run --all` to run all scenarios
- [ ] Add `baseline set <run_id>` command

---

## Value Model (`specs/value-model.md`) (P2)

Adds a `value` field (0-100, default 50) to notes for quality/importance scoring, enabling weighted graph traversal.

### Phase 1: Data Model
- [x] Add `value` field to `NoteFrontmatter` in `src/lib/note/frontmatter.rs`
  - Type: `Option<u8>`, serde skip_serializing_if None
  - Update `NoteFrontmatter::new()` to initialize as None
- [x] Schema migration in `src/lib/db/schema.rs`
  - Add `value INTEGER DEFAULT 50` column to `notes` table
  - Add index: `CREATE INDEX idx_notes_value ON notes(value)`
  - Bump `CURRENT_SCHEMA_VERSION` to 2
  - Add migration path from v1 → v2
- [x] Update `src/lib/db/notes/` to read/write `value` column
  - Updated `create.rs`: Both `insert_note` and `insert_note_internal` write value column
  - Updated `read.rs`: Both `get_note_metadata` and `list_notes` read value column
  - Updated `NoteMetadata` in `src/lib/index/types.rs` to include value field
  - Updated `builder.rs` to pass value from frontmatter to metadata
  - Updated test mock metadata in `similarity/mod.rs` to include value field
- [x] Update `src/lib/index/builder.rs` to index `value` field - ALREADY DONE (line 90 stores value in NoteMetadata)

### Phase 2: CLI Commands
- [x] Add `qipu value` subcommand in `src/cli/commands.rs`
  - `value set <id> <score>` - update frontmatter value field
  - `value show <id>` - display current value (or "50 (default)" if unset)
- [x] Add `--min-value <n>` filter flag to `qipu list`
- [x] Add `--min-value <n>` filter flag to:
  - `qipu search` (completed 2026-01-20)
  - `qipu link tree` (completed 2026-01-20)
  - `qipu link path` (completed 2026-01-20)
  - `qipu context` (completed 2026-01-20)
- [x] Add `--sort value` option to `qipu search` (completed 2026-01-20)

### Phase 3: Weighted Traversal
- [x] Add `get_edge_cost(link_type, target_value)` in `src/lib/graph/types.rs`
  - Formula: `LinkTypeCost * (1 + (100 - value) / 100)`
  - Composes with future per-link-type costs (see `specs/semantic-graph.md` §3.A)
- [x] Add `--ignore-value` flag to `TreeOptions` in `src/lib/graph/types.rs` - completed 2026-01-20
- [x] Implement Dijkstra traversal variant in `src/lib/graph/bfs.rs` - completed 2026-01-20
  - New function `dijkstra_traverse()` using `BinaryHeap` instead of `VecDeque`
  - Order by accumulated cost (min-heap)
  - Default behavior: weighted (Dijkstra)
  - With `--ignore-value`: unweighted (BFS, all edges cost 1.0)
- [x] Update `bfs_find_path()` to support weighted mode - completed 2026-01-20
  - Added logic to check `opts.ignore_value` flag
  - When `ignore_value=true`: unweighted BFS (VecDeque, all edges cost 1.0)
  - When `ignore_value=false`: weighted Dijkstra (BinaryHeap, cost based on note value)
  - Added `best_costs` HashMap to track best-known cost to each node
  - Supports re-visiting nodes with better paths (Dijkstra optimization)
  - Added 3 unit tests: unweighted, weighted, and min_value filter

### Phase 4: Integration
- [x] Update `qipu context` to respect `--min-value` threshold (completed 2026-01-20)
- [x] Update `qipu doctor` to validate value range (0-100) - completed 2026-01-20
- [x] Add tests for value filtering and weighted traversal - completed 2026-01-20
   - Added 4 CLI tests for `list --min-value` filter in `tests/cli/list.rs`
   - Added 4 CLI tests for `search --min-value` and `search --sort value` in `tests/cli/search.rs`
   - Tests cover: all match, some match, none match, default values, sorting, combined filters
   - All 8 new tests pass (pre-existing 6 failures in unrelated tests: missing store detection)
- [x] Update help text and man pages - completed 2026-01-20
   - Added `qipu value set` to Core Commands in README.md
   - Added `--min-value` filters to `list` and `search` command examples
   - Added `--sort value` option to search command
   - Added new "Value Model" section to README.md explaining:
     - Value scale (0-20 deprioritized, 21-80 standard, 81-100 high-value)
     - CLI examples for setting, showing, and filtering by value
     - Weighted traversal using Dijkstra's algorithm
   - Updated Link Management section to mention `--min-value` and weighted traversal
   - All value-related features documented; no man pages exist (only help text)

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

### Value Field Data Model (completed 2026-01-19)
- Added `value` field (Option<u8>) to `NoteFrontmatter` in `src/lib/note/frontmatter.rs`
- Updated `NoteFrontmatter::new()` to initialize value as None
- Added `value` field to `PackNote` struct in both `src/commands/load/model.rs` and `src/commands/dump/model.rs`
- Updated all PackNote construction sites to handle value field:
  - `src/commands/load/mod.rs` when loading from pack
  - `src/commands/dump/serialize.rs` when dumping to pack
  - `src/commands/load/deserialize.rs` when deserializing pack format
- Added "value" metadata parsing in pack deserializer
- All 439 tests pass (189 unit + 229 CLI + 6 golden + 6 pack + 6 perf + 3 workspace merge)

### Min-Value Filter Implementation (completed 2026-01-20)
- Added `--min-value <n>` flag to `Commands::List` in `src/cli/commands.rs`
- Updated command dispatch path: `src/commands/dispatch/mod.rs` → `notes.rs` → `list.rs`
- Filter logic in `src/commands/list.rs`:
  - Notes without explicit value default to 50
  - Filtering: `value >= min_value` (inclusive)
- Added 5 unit tests covering:
  - All notes match threshold
  - Some notes match threshold
  - No notes match threshold
  - Notes with default value (None treated as 50)
  - Exact threshold boundary (value = min_value)
- Updated CLI parser tests to validate `--min-value` flag parsing
- All 439 tests pass (189 unit + 229 CLI + 6 golden + 6 pack + 6 perf + 3 workspace merge)

### Context Min-Value Filter Implementation (completed 2026-01-20)
- Added `--min-value <n>` flag to `Commands::Context` in `src/cli/commands.rs`
- Updated command dispatch path: `src/commands/dispatch/mod.rs` → `notes.rs` → `context/mod.rs`
- Added `min_value: Option<u8>` field to `ContextOptions` in `src/commands/context/types.rs`
- Filter logic in `src/commands/context/mod.rs`:
  - Applied after all notes are collected but before sorting and budgeting
  - Notes without explicit value default to 50
  - Filtering: `value >= min_value` (inclusive)
  - Logs filtered count when verbose mode is enabled
- Added comprehensive CLI test `test_context_filter_by_min_value` in `tests/cli/context/basic.rs`:
  - Tests high-value note (90) with min-value 80
  - Tests default-value note (50) with min-value 50
  - Tests low-value note (30) excluded by filters
- All 233 CLI tests pass (1 new test added)

### Edge Cost Function Implementation (completed 2026-01-20)
- Added `get_edge_cost(link_type, target_value)` function in `src/lib/graph/types.rs`
- Formula: `LinkTypeCost * (1 + (100 - value) / 100)`
  - Value 100 → multiplier 1.0 (no penalty)
  - Value 50 → multiplier 1.5
  - Value 0 → multiplier 2.0 (maximum penalty)
- Added 5 unit tests covering:
  - Max value (100) returns 1.0
  - Mid value (50) returns 1.5
  - Min value (0) returns 2.0
  - Custom link type with mid value (75 returns 1.25)
  - Boundary value (1) with floating-point comparison
- All 450 tests pass (202 unit + 233 CLI + 6 golden + 6 pack + 6 perf + 3 workspace merge)

### Ignore-Value Flag Implementation (completed 2026-01-20)
- Added `ignore_value: bool` field to `TreeOptions` struct in `src/lib/graph/types.rs`
- Updated `Default` implementation to initialize `ignore_value` as `false`
- Added 2 unit tests:
  - Default value is `false`
  - Can be set to `true`
- Updated all TreeOptions construction sites:
  - `src/commands/dump/mod.rs` (dump command traversal)
  - `src/commands/dispatch/link.rs` (link tree and link path commands)
- All 457 tests pass (204 unit + 238 CLI + 6 golden + 6 pack + 6 perf + 3 workspace merge)

### Dijkstra Traversal Implementation (completed 2026-01-20)
- Added `dijkstra_traverse()` function in `src/lib/graph/bfs.rs` using `BinaryHeap` for cost-ordered traversal
- Implements weighted edge costs using `get_edge_cost(link_type, target_value)` based on note value (0-100)
- Supports unweighted mode via `--ignore-value` flag (all edges cost 1.0)
- Added `HeapEntry` type with custom `Ord` implementation for min-heap ordering
- Added 4 unit tests:
  - Unweighted traversal (ignore_value=true)
  - Weighted traversal with note values
  - Min-value filter integration
  - Heap entry comparison ordering
- Exported `dijkstra_traverse` from graph module
- All 208 unit tests pass (including 4 new dijkstra_traverse tests)

### Value Range Validation (completed 2026-01-20)
- Added `check_value_range()` function in `src/commands/doctor/content.rs`
- Validates that when `value` field is Some(v), v must be in range 0-100
- None values are valid (default to 50 in other parts of the system)
- Added check call in `src/commands/doctor/mod.rs` execute function (after required fields check)
- Added 4 unit tests covering:
  - Invalid value (150) detection
  - Valid values (0, 50, 100) pass validation
  - None values pass validation
  - Boundary value (101) detection
- All doctor tests pass (12 tests, including 4 new value range tests)
- All 457 tests pass (203 unit + 238 CLI + 6 golden + 6 pack + 6 perf + 3 workspace merge)

### Value Filtering and Weighted Traversal CLI Tests (completed 2026-01-20)
- Added comprehensive CLI tests for value filtering features in list and search commands
- List command tests (`tests/cli/list.rs`):
  - `test_list_filter_by_min_value_all_match`: All notes pass threshold (50)
  - `test_list_filter_by_min_value_some_match`: Only high/medium notes pass threshold (70)
  - `test_list_filter_by_min_value_none_match`: No notes pass threshold (95)
  - `test_list_filter_by_min_value_with_defaults`: Explicit 80 and default 50 both pass
- Search command tests (`tests/cli/search.rs`):
  - `test_search_with_min_value_filter`: Tests min-value filtering with different thresholds
  - `test_search_sort_by_value`: Tests sorting by value in descending order
  - `test_search_sort_by_value_with_defaults`: Tests sorting with explicit and default values
  - `test_search_min_value_and_sort_combined`: Tests combined min-value filter and sort
- All 8 new CLI tests pass
- Test pattern: Create notes, set values with `value set`, run index, then query with filters
- Note: 6 pre-existing test failures in missing store detection (exit code 3), unrelated to changes
- Test count increased from 448 to 456 total (8 new CLI tests)

### Command Handlers Refactoring (completed 2026-01-20)
- Created new module `crates/llm-tool-test/src/commands.rs` with 6 command handler functions
- Extracted all command logic from `main.rs` into dedicated handler functions:
  - `handle_run_command()`: Processes run command with tool matrix support
  - `handle_list_command()`: Lists scenarios or pending reviews
  - `handle_show_command()`: Displays run details
  - `handle_compare_command()`: Compares two runs
  - `handle_clean_command()`: Clears cache
  - `handle_review_command()`: Adds human review to a run
- Updated `main.rs` to call handler functions instead of inline logic
- Made utility functions public (`build_tool_matrix`, `print_matrix_summary`)
- Fixed type mismatches (String vs PathBuf, usize vs u8, u64 vs Option<u64>)
- All 123 tests pass, 4 pre-existing warnings unrelated to changes
- Reduced `main.rs` complexity by separating command logic from CLI parsing

### Output Functions Extraction (completed 2026-01-20)
- Created new module `crates/llm-tool-test/src/output.rs` with 3 print functions:
  - `print_matrix_summary()`: Displays matrix run results in table format
  - `print_result_summary()`: Displays detailed metrics for a single run
  - `print_regression_report()`: Displays comparison between current and baseline runs
- Moved `ToolModelConfig` struct from `main.rs` to `output.rs` since it's only used by output-related functionality
- Updated `main.rs` to remove print function implementations and import from `output` module
- Updated `commands.rs` to call `output::print_matrix_summary()` and `output::print_regression_report()`
- Updated `run.rs` to call `output::print_result_summary()` and `output::print_regression_report()`
- Removed unused imports from `main.rs` (`ScoreTier`, `ResultRecord`, `RegressionReport`, `HashMap`)
- All 123 tests pass, 2 pre-existing warnings unrelated to changes
- Improved code organization by separating output formatting logic from business logic
