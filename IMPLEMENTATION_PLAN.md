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
- [x] Add unit tests to `src/commands/search.rs` (350 lines, high-value)
  - Added 13 unit tests covering: empty query, no results, type/tag filters, MOC exclusion, all output formats, compaction resolution, verbose/quiet modes
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

### LLM Tool Test Harness Deep Dive (2026-01-19)

#### Architecture Summary
- **Total**: 1,390 lines across 10 modules
- **Purpose**: Automated LLM tool validation with caching, regression detection, LLM-as-judge
- **Adapters**: amp (47 lines), opencode (25 lines) - both minimal stubs
- **Fixtures**: 1 fixture (`qipu/`), 2 scenarios, 2 rubrics

#### Evaluation Dimensions & Scoring Framework (P1)

The goal is not binary pass/fail but measuring *how well* the LLM uses qipu. Key dimensions:

##### 1. Efficiency Metrics (Transcript Analysis)
Extract from raw transcript:
- [x] **Command count**: Total qipu commands executed
- [x] **Error count**: Commands that returned non-zero exit codes
- [x] **Retry count**: Same command executed multiple times (indicates confusion)
- [x] **Help invocations**: Count of `--help` or `help` subcommands
- [x] **First-try success rate**: % of commands that succeeded on first attempt
- [x] **Iteration ratio**: (total_commands / unique_successful_commands)
  - 1.0 = perfect, higher = more fumbling

Implementation:
- [x] Add `TranscriptAnalyzer` struct to parse raw transcript
  - File: `crates/llm-tool-test/src/transcript.rs`
- [x] Extract command invocations via regex: `qipu <subcommand> ...`
- [x] Track exit codes per command (requires adapter changes to capture)
  - Modified `SessionRunner::run_command` to return `(String, i32)` for output + exit code
  - Updated `ToolAdapter::run` trait to return `(String, i32)`
  - Updated both adapters (opencode, amp) to propagate exit codes
  - Added `CommandEvent` struct with `command` and `exit_code` fields
  - Added `analyze_with_exit_codes()` method to `TranscriptAnalyzer` that parses transcript for exit codes
  - Exit codes extracted from transcript by looking for patterns like "exit code: 1" or "exit status: 0"
  - Falls back to heuristic error detection if exit codes not found
  - Updated `compute_efficiency_metrics()` to use `analyze_with_exit_codes()`
- [x] Add `EfficiencyMetrics` to `EvaluationMetrics`:
  ```rust
  pub struct EfficiencyMetrics {
      pub total_commands: usize,
      pub unique_commands: usize,
      pub error_count: usize,
      pub retry_count: usize,
      pub help_invocations: usize,
      pub first_try_success_rate: f64,
      pub iteration_ratio: f64,
  }
  ```
  - Added to `transcript.rs` as public struct
  - Integrated into `EvaluationMetrics` in `evaluation.rs`
  - Added to `EfficiencyMetricsRecord` in `results.rs`
  - Included in result records in `main.rs`
  - 6 unit tests added to `transcript.rs` (all passing)

##### 2. Quality Metrics (Store Analysis)
Automated analysis of the resulting store:
- [ ] **Note quality indicators**:
  - Average title length (too short = vague, too long = unfocused)
  - Body length distribution
  - Tag usage (notes with 0 tags, avg tags per note)
  - Type distribution (fleeting vs permanent vs literature)
- [ ] **Graph quality indicators**:
  - Link density (links per note)
  - Graph connectivity (orphan notes with no links)
  - Link type diversity (using multiple link types vs just one)
  - MOC coverage (% of notes reachable from a MOC)
- [ ] **Semantic quality** (requires LLM judge):
  - Relevance to task
  - Coherence of note structure
  - Appropriate granularity (not too broad, not too narrow)

Implementation:
- [ ] Add `StoreAnalyzer` to compute metrics from `qipu export --format json`
  - File: `crates/llm-tool-test/src/store_analysis.rs`
- [ ] Add `QualityMetrics` struct:
  ```rust
  pub struct QualityMetrics {
      pub avg_title_length: f64,
      pub avg_body_length: f64,
      pub avg_tags_per_note: f64,
      pub notes_without_tags: usize,
      pub links_per_note: f64,
      pub orphan_notes: usize,
      pub link_type_diversity: usize,
      pub type_distribution: HashMap<String, usize>,
  }
  ```

##### 3. Cost/Speed Metrics
- [ ] **Wall-clock duration**: Already captured as `duration_secs`
- [ ] **Token usage**: Extract from tool output if available
- [ ] **API cost**: Calculate from token usage × model pricing
- [ ] **Commands per minute**: Throughput indicator

##### 4. LLM Judge Dimensions
Enhance rubrics to evaluate specific dimensions:
- [ ] **Task completion**: Did it achieve the stated goal?
- [ ] **Command fluency**: Correct syntax on first try?
- [ ] **Knowledge structure**: Is the graph well-organized?
- [ ] **Appropriate tooling**: Used the right commands for the job?
- [ ] **Idiomatic usage**: Follows qipu best practices (tags, types, links)?

Example enhanced rubric:
```yaml
criteria:
  - id: task_completion
    weight: 0.25
    description: "Task goal was fully achieved"
  - id: command_fluency
    weight: 0.20
    description: "Commands executed correctly without retries or help lookups"
  - id: knowledge_structure
    weight: 0.25
    description: "Notes are well-linked with appropriate types and tags"
  - id: idiomatic_usage
    weight: 0.15
    description: "Follows qipu conventions (MOCs, link types, verification)"
  - id: efficiency
    weight: 0.15
    description: "Minimal commands needed to complete task"
```

##### 5. Composite Scores
- [ ] Add weighted composite score combining automated + judge metrics
- [ ] Define score thresholds for grading:
  - **Excellent** (0.9+): First-try success, high quality output
  - **Good** (0.7-0.9): Minor retries, solid output
  - **Acceptable** (0.5-0.7): Multiple retries, basic output achieved
  - **Poor** (<0.5): Many errors, incomplete or low-quality output

##### 6. Human Review Integration (Asynchronous)
Human review happens out-of-band after runs complete—runs are never paused:
- [ ] Ensure all transcripts and artifacts are saved for later review
- [ ] Add `review` subcommand to score past runs:
  ```bash
  llm-tool-test review <RUN_ID> \
    --dimension clarity=0.8 \
    --dimension insight_value=0.7 \
    --notes "Good structure but missed key concept X"
  ```
- [ ] Store human scores in results record
- [ ] Add `HumanReviewMetrics` struct:
  ```rust
  pub struct HumanReviewMetrics {
      pub scores: HashMap<String, f64>,
      pub notes: Option<String>,
      pub reviewer: Option<String>,
      pub reviewed_at: Option<DateTime<Utc>>,
  }
  ```
- [ ] Add `list --pending-review` to find runs without human scores

##### Implementation Priority
1. Transcript analysis (efficiency metrics) - can derive from existing data
2. Store analysis (quality metrics) - uses existing qipu export
3. Enhanced rubrics - extends existing judge system
4. Human review integration - new workflow
5. Composite scoring - aggregation layer

#### Critical Gaps (P1)

##### Tool/Model Matrix Execution
- [x] Add `--model` parameter to CLI for tool model selection
  - File: `crates/llm-tool-test/src/cli.rs`
- [x] Extend `ToolAdapter` trait to accept model parameter
  ```rust
  fn run(&self, scenario: &Scenario, cwd: &Path, model: Option<&str>) -> Result<String>;
  ```
  - File: `crates/llm-tool-test/src/adapter/mod.rs`
- [x] Update adapters to pass model to underlying tools
  - opencode: `opencode run --model <model> ...`
  - amp: Determine if model selection is supported
  - File: `crates/llm-tool-test/src/adapter/*.rs`
- [x] Store model in results and include in cache key
  - Updated `CacheKey::compute()` to accept model parameter
  - ResultRecord now uses actual model value instead of "default"
- [x] Add `--tools` and `--models` list parameters for matrix runs
  ```bash
  llm-tool-test run --scenario capture_basic.yaml \
    --tools opencode,claude-code \
    --models claude-sonnet-4-20250514,gpt-4o
  ```
  - Added CLI parameters to Run command in `cli.rs`
  - Added `build_tool_matrix()` function to parse tools/models or use scenario-level tool_matrix
  - Added `run_single_scenario()` function to handle individual runs
  - Added `print_matrix_summary()` to display pass/fail grid
  - Updated main.rs to execute matrix when --tools and --models provided
  - All 355 tests passing
- [x] Add scenario-level `tool_matrix` field for declarative matrix
  ```yaml
  tool_matrix:
    - tool: opencode
      models: [claude-sonnet-4-20250514, claude-3-5-haiku-20241022, gpt-4o]
    - tool: claude-code
      models: [opus, sonnet]
    - tool: amp
      models: [default]
  ```
  - File: `crates/llm-tool-test/src/scenario.rs`
  - Added `ToolConfig` struct with `tool` and `models` fields
  - Added `tool_matrix` optional field to `Scenario` struct with serde default
  - Added 3 unit tests to verify YAML parsing with/without tool_matrix
  - All existing tests pass
- [x] Add claude-code adapter
   - File: `crates/llm-tool-test/src/adapter/claude_code.rs`
   - Created adapter module with `ClaudeCodeAdapter` struct
   - Implemented `ToolAdapter` trait with `check_availability` and `run` methods
   - Uses `claude run --model <model> --prompt-file prompt.txt` pattern
   - Registered in `main.rs` match statement and exported in `adapter/mod.rs`
   - Updated CLI help text to include claude-code
   - All 375 tests passing (130 + 213 + 11 + 6 + 6 + 6 + 3)
- [x] Add matrix summary report (pass/fail grid by tool×model)
   - Already implemented in `print_matrix_summary()` function at main.rs:213-260
   - Displays tool names as rows, model names as columns
   - Shows Pass/Fail/Error outcome for each tool×model combination
   - Called automatically after matrix runs

##### Amp Adapter Implementation
- [ ] `adapter/amp.rs` uses hypothetical CLI: `amp run --context AGENTS.md --prompt-file prompt.txt`
  - Verify actual Amp CLI syntax and update adapter
  - Current implementation is untested speculation
- [ ] Add timeout handling for long-running LLM sessions
- [ ] Add cost tracking (currently hardcoded to 0.0 in `main.rs:116`)

##### Scenario Tiers
- [ ] Add `tier` field to scenario schema (0=smoke, 1=quick, 2=standard, 3=comprehensive)
  - File: `crates/llm-tool-test/src/scenario.rs`
- [ ] Add `--tier` CLI flag to filter scenarios by tier
  - Tier N runs all scenarios with tier <= N
  - File: `crates/llm-tool-test/src/cli.rs`
- [ ] Create tier 0 (smoke) scenario: single `qipu create` command
- [ ] Create tier 1 (quick) scenarios: basic capture, simple linking

##### Scenario Coverage
- [ ] Only 2 scenarios exist (`capture_basic`, `link_navigation`)
  - Add: `search_basic`, `context_retrieval`, `compaction_workflow`
  - Add: `multi_note_linking`, `inbox_processing`, `export_workflow`
- [ ] Scenarios lack `setup` step (e.g., pre-populate store with seed notes)
- [ ] No negative test scenarios (expected failures, error handling)

##### Gate Types
- [ ] Only 3 gate types: `MinNotes`, `MinLinks`, `SearchHit`
  - Add: `NoteExists { id: String }` - verify specific note was created
  - Add: `LinkExists { from, to, link_type }` - verify specific link
  - Add: `TagExists { tag: String }` - verify tag usage
  - Add: `ContentContains { id, substring }` - verify note content
  - Add: `CommandSucceeds { command: Vec<String> }` - arbitrary qipu command

#### Important Gaps (P2)

##### Test Infrastructure
- [ ] Only 2 unit tests in entire crate (judge prompt, gate evaluation)
- [ ] No integration tests for adapters
- [ ] No mock adapter for offline testing
- [ ] Dead code: `ResultsDB::load_latest_by_scenario` (line 94)

##### Fixture Improvements
- [ ] `qipu/AGENTS.md` is minimal - should match real AGENTS.md patterns
- [ ] No pre-populated store fixture (for testing against existing data)
- [ ] Rubrics only exist for capture_v1 and link_v1

##### Session Runner
- [ ] `session.rs` uses PTY but no timeout mechanism
- [ ] No streaming output for long sessions
- [ ] No way to interact mid-session (for multi-turn scenarios)

##### Results & Reporting
- [ ] No HTML/Markdown report generation
- [ ] No CI integration (GitHub Actions workflow)
- [ ] No aggregate statistics across runs

#### Minor Gaps (P3)

##### Code Quality
- [ ] No tracing instrumentation anywhere in crate
- [ ] `main.rs` (259 lines) should be split - extract command handlers
- [ ] `results.rs` (262 lines) - split cache and DB logic
- [ ] `evaluation.rs` (357 lines) - split gate evaluation from judge logic

##### CLI Polish
- [ ] `list` command shows runs, not scenarios - misleading
- [ ] No `run --all` to run all scenarios
- [ ] No `baseline set <run_id>` to mark a run as baseline

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
