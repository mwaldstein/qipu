# Qipu Implementation Plan

This document tracks **concrete implementation tasks** - bugs to fix, features to complete, and tests to add. For exploratory future work and open questions from specs, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status
- Test baseline: 789 tests pass (all passing)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` has pre-existing warnings
- Audit Date: 2026-01-22
- Related: [`specs/README.md`](specs/README.md) - Specification status tracking

---

## P1: Correctness Bugs

### CLI Interface (`specs/cli-interface.md`)
- [x] `qipu create` does not print the created note path by default (spec says ID and path).
  - `src/commands/create.rs:88-95`

### CLI Tool (`specs/cli-tool.md`)
- [x] Emit JSON error envelopes for parse failures when `--format=json` is used (the `--format=json` form is currently missed).
  - `src/main.rs:82-93`

### Operational Database (`specs/operational-database.md`)
- [x] Treat the database as the source of truth (remove filesystem fallbacks).
  - `src/lib/store/query.rs:14-52`
  - `src/lib/store/query.rs:66-101`
  - Learnings: Added `list_note_ids()` method to Database module; replaced filesystem scanning in `existing_ids()` with database query
 - [x] Trigger incremental repair when startup validation fails.
   - `src/lib/db/mod.rs:84-85`
   - `src/lib/db/repair.rs:6-141`
   - Learnings: Changed validation to check result; call incremental_repair when false; removed #[allow(dead_code)] attribute from incremental_repair
- [x] Auto-rebuild the database when schema mismatches are detected.
   - `src/lib/db/schema.rs:19-21,74-91,94-127`
   - `src/lib/db/mod.rs:64-69`
   - `src/lib/db/tests.rs:713-752,938-987`
   - Learnings: Added SchemaCreateResult enum to signal when rebuild is needed; create_schema now drops all tables and returns NeedsRebuild on schema mismatch; Database::open checks this flag and calls rebuild(); updated tests to verify auto-rebuild behavior

### Indexing/Search (`specs/indexing-search.md`)
- [x] `qipu index --rebuild` is a no-op (always rebuilds); wire incremental indexing.
  - `src/commands/index.rs:14-19`
  - `src/lib/db/repair.rs:6-141`

### Graph Traversal (`specs/graph-traversal.md`)
- [x] `link path` defaults to weighted Dijkstra instead of shortest-hop traversal.
  - `src/cli/link.rs:154-156`
  - Learnings: Changed default_value for ignore_value flag from false to true in link path CLI definition; this makes link path use unweighted BFS (shortest-hop traversal) by default
- [x] `link tree/path` flags do not support CSV-style `--types/--exclude-types` (only repeatable `--type`/`--exclude-type`).
  - `src/cli/link.rs:73-79`
  - `src/cli/link.rs:130-136`
  - Learnings: Added `alias` and `value_delimiter` attributes to both `r#type` and `exclude_type` fields in Tree and Path commands; this allows both `--type/--exclude-type` (repeatable) and `--types/--exclude-types` (CSV) forms as specified in the spec

### Knowledge Model (`specs/knowledge-model.md`)
- [x] Context traversal does not preserve MOC ordering as a "reading path" (unordered outbound edges).
  - `src/commands/context/select.rs:21-38`
  - Learnings: Added `position INTEGER NOT NULL DEFAULT 0` column to edges schema; updated INSERT statements to include position; updated `get_outbound_edges` query to ORDER BY position; changed queue from vec to VecDeque with pop_front for FIFO behavior; bumped schema version to 3

### Pack (`specs/pack.md`)
- [x] `load --strategy skip` drops all links, even for newly loaded notes.
  - Learnings: Added `new_ids` return value to track newly loaded notes (notes that didn't exist before); changed skip strategy to only load links between newly loaded notes (using `new_ids` instead of `loaded_ids`)
  - Fixes P1 correctness bug where skip strategy was dropping ALL links, even for newly loaded notes. Now only loads links where both source AND target notes are newly loaded, preventing any mutation of skipped existing notes.
- [x] `load --strategy merge-links` doesn't insert edges into database.
  - `src/commands/load/mod.rs:132-158,77-85,306-312,362` - Updated `write_note_preserving_updated` to accept `existing_ids` and call `insert_edges` after `insert_note`; updated all callers to pass the IDs needed for edge resolution
  - **Root Cause**: The `insert_note` function only updates the notes, FTS, and tags tables - it does not insert edges. When notes were loaded/modified during pack load, edges were written to the note file frontmatter but not to the database edges table.
  - **Learnings**: Database consistency requires calling both `insert_note` AND `insert_edges`. The `Store` methods (`create_note`, `update_note`, `save_note`) do this correctly, but the pack load command had its own write function that was missing the edge insertion.
### Records Output (`specs/records-output.md`)
- [x] Link records omit `path=` in `N` records for tree/path/list outputs.
   - `src/commands/link/records.rs:65-71`
   - `src/commands/link/tree.rs:293-299`
   - Learnings: Added `path=` field to N records in both `append_note_metadata_lines` (for link tree/list) and `output_path_records` (for link path) functions; both NoteMetadata and TreeNote structs already had path fields available

### Value Model (`specs/value-model.md`)
- [x] `--min-value` accepts values outside 0-100 without validation.
  - `src/cli/parse.rs:14-19`
  - `src/cli/commands.rs:51,136,278`
  - `src/cli/link.rs:105,150`
  - Learnings: Added `parse_min_value` function to validate range 0-100; applied value_parser to all 5 min_value CLI arguments (list, search, context, link tree, link path)
- [x] Dijkstra traversal uses a max-heap ordering, which can invert expected "shortest" paths.
  - `src/lib/graph/bfs.rs:35-43`
  - Learnings: Removed `.reverse()` from HeapEntry::cmp implementation; the Reverse<HeapEntry> wrapper already provides min-heap semantics for BinaryHeap; the reverse was causing max-heap behavior instead of min-heap

### Structured Logging (`specs/structured-logging.md`)
- [x] Debug logs are gated by `--verbose` even when `--log-level debug` is set.
  - `src/commands/dispatch/notes.rs:23-26`
  - `src/commands/search.rs:36-45`
  - Learnings: Removed `if cli.verbose` guards from debug!() calls in dispatch and search handlers; tracing's log level filter now controls whether debug logs appear, allowing `--log-level debug` to work independently of `--verbose`
  - Note: Pre-existing flaky tests discovered using `.trim()` instead of `extract_id()` in `tests/cli/inbox.rs`, `tests/cli/capture.rs`, `tests/cli/provenance.rs`, and `tests/cli/workspace.rs` - not related to this fix

### LLM User Validation (`specs/llm-user-validation.md`)
- [x] Tool adapter trait diverges from spec (`execute_task`/`ToolStatus` missing).
  - `crates/llm-tool-test/src/adapter/mod.rs:12-118` - Added ToolStatus, TaskContext, ExecutionResult, TokenUsage, CostEstimate, and AdapterError types
  - Updated ToolAdapter trait with `name()`, `is_available()`, `execute_task()`, and `estimate_cost()` methods per spec
  - All four adapters (mock, amp, opencode, claude_code) implement the new trait methods
  - Legacy `check_availability()` and `run()` methods preserved for backward compatibility during migration
  - **Learnings**: The new trait interface provides better separation of concerns with explicit types for status, context, and results. The execute_task method now properly handles transcript directory for artifact generation. Cost estimation is now a separate concern with its own type.
- [x] Missing `report` subcommand and `clean --older-than` support.
  - `crates/llm-tool-test/src/cli.rs:93-99` - Added `Report` subcommand and `--older-than` optional flag to `Clean` command
  - `crates/llm-tool-test/src/main.rs:148-152` - Added command dispatch for `Report` and updated `Clean` to pass parameters
  - `crates/llm-tool-test/src/commands.rs:277-421` - Implemented `handle_report_command()` which generates summary statistics grouped by scenario and tool, plus recent runs. Implemented `parse_duration()` helper for parsing duration strings like "30d", "7d", "1h". Updated `handle_clean_command()` to accept `older_than` parameter and clean transcripts based on modification time
  - **Learnings**: The report command provides a comprehensive overview of test runs including pass rates, costs, and performance metrics aggregated by scenario and tool. The clean command now supports optional time-based filtering using standard duration formats (d/h/m) for better maintenance of large test result sets.
- [x] Add PTY fallback and richer event logging (tool_call/tool_result).
  - `crates/llm-tool-test/src/session.rs:22-108` - Refactored `run_command` to try PTY first, fall back to piped stdout/stderr if PTY unavailable
  - `crates/llm-tool-test/src/session.rs:109-199` - Added `run_command_piped` method for fallback execution using std::process::Command with piped stdout/stderr
  - `crates/llm-tool-test/src/transcript.rs:1-6,24-88` - Added structured event logging methods: `log_tool_call`, `log_tool_result`, `log_spawn`, `log_output`, `log_complete`, and `timestamp` helper
  - `crates/llm-tool-test/src/adapter/mock.rs:11-78` - Added `run_with_events` method to MockAdapter that emits tool_call/tool_result events during command execution
  - **Learnings**: PTY fallback ensures the test harness works in environments without PTY support (e.g., some CI systems, containers). Structured event logging provides machine-readable execution traces with explicit tool_call/tool_result events as specified in the LLM User Validation spec (lines 307-308). Events include timestamps, command strings, outputs, and exit codes for detailed analysis.

### Workspaces (`specs/workspaces.md`)
- [x] `rename` merge strategy is not supported.
  - `src/commands/workspace/merge.rs:20-24`
  - Learnings: Added rename strategy support with ID suffix generation (e.g., qp-a1b2 -> qp-a1b2-1); implemented ID mapping to rewrite links in all incoming notes; both copy_note and copy_note_with_rename now handle link rewriting based on id_mappings HashMap
- [x] `--from-*` workspace creation is shallow; graph-slice copy is missing.
  - `src/commands/workspace/new.rs:67-101,136-180`
  - Learnings: Refactored `copy_graph_slice` to accept multiple root IDs instead of a single ID; both `--from-tag` and `--from-query` now collect matching notes and perform BFS graph traversal (3-hop limit) instead of shallow copying; this matches the spec requirement that all `--from-*` options should initialize with a graph slice
- [x] Post-merge integrity validation is missing.
  - `src/commands/workspace/merge.rs:167-190`
  - Learnings: Added post-merge validation by calling `doctor::execute()` after merge completes; validation runs all standard checks (broken links, duplicate IDs, semantic link misuse, etc.); results are reported to user with error/warning counts; doctor returns error if critical issues remain unfixed
- [x] `workspace list` omits last-updated metadata.
  - `src/commands/workspace/list.rs:12-21,49,72,96,111,126`
  - Learnings: Feature was already fully implemented; `get_last_updated()` queries database for max mtime; all three output formats (human, json, records) correctly display last_updated field
- [x] `parent_id` is never populated.
  - `src/commands/workspace/new.rs:58-66`
  - Learnings: Set parent_id to "(primary)" for all newly created workspaces since they are always created from the primary store; workspace identity uses names not IDs

### Similarity Ranking (`specs/similarity-ranking.md`)
- [x] Search ranking boosts are hardcoded (do not match spec weights).
  - `src/lib/db/search.rs:18-106`
  - Learnings: The BM25 field weights (title: 2.0, body: 1.0, tags: 1.5) were already correct. The query-specific boosts (+2.0 for title, +3.0 for tags) are necessary to ensure proper ranking as required by indexing-search spec ("Title matches rank above body matches" and "Exact tag matches rank above plain text"). The +3.0 boost for tags is higher than the tag field weight (1.5) because tags need to rank above body matches even when there are multiple stemmed occurrences in the body text.

### CLI Tool Tests (`specs/cli-tool.md`)
- [x] Add tests for visible-store discovery and `--format=json` parse errors.
  - `tests/cli/misc.rs:117-212` - Added `test_visible_store_discovery()` to verify discovery of non-hidden `qipu/` directory
  - `tests/cli/misc.rs:214-280` - Added `test_hidden_store_preferred_over_visible()` to verify `.qipu/` is preferred over `qipu/` when both exist
  - `tests/cli/misc.rs:286-308` - Added `test_missing_required_arg_json_format()` and `test_invalid_value_json_format()` for JSON error envelope tests
  - Learnings: Visible store discovery (`qipu/` vs `.qipu/`) is checked in order with hidden preferred; JSON error envelopes are correctly emitted for all parse error types (usage errors, missing args, invalid values)
- [x] Expand golden determinism coverage beyond help/list/prime.
  - `tests/golden_tests.rs:201-395` - Added 9 new golden tests for deterministic command outputs
  - Added `test_golden_context_with_note()` and `test_golden_context_with_moc()` for context command
  - Added `test_golden_search_basic()` for search command
  - Added `test_golden_inbox()` for inbox command
  - Added `test_golden_show_note()` and `test_golden_show_note_with_links()` for show command
  - Added `test_golden_link_list()`, `test_golden_link_tree()`, and `test_golden_link_path()` for link traversal commands
  - Learnings: Golden tests now cover the key deterministic commands mentioned in the spec (context, search, inbox, show, link list/tree/path); all outputs are normalized to use `<STORE_PATH>` placeholder for path-independence

### CLI Interface Tests (`specs/cli-interface.md`)
- [x] Add tests for `create` alias `new`, `--open`, and `--id`.
   - `tests/cli/create.rs:172-266` - Added test_new_alias, test_create_with_custom_id, test_create_with_open_flag
   - Learnings: The `new` alias works correctly; custom ID via `--id` flag creates notes with specified IDs; `--open` flag accepts EDITOR env var to launch editor (tested with `true` command to avoid blocking in CI)
- [x] Add tests for `list --tag`, `list --since`, and `list --format records`.
  - `tests/cli/list.rs:313-571` - Added test_list_filter_by_tag, test_list_filter_by_tag_no_matches, test_list_filter_by_since, test_list_filter_by_since_no_matches, test_list_records_format, test_list_records_format_empty, test_list_records_format_multiple_notes
  - Learnings: Tags are alphabetically sorted in output; --since accepts RFC3339 timestamps; records format includes header with note count and mode=list
- [x] Add tests for `search --exclude-mocs`, `--min-value`, and `--sort`.
  - `tests/cli/search.rs:619-937` - Added `test_search_with_min_value_filter`, `test_search_sort_by_value`, `test_search_sort_by_value_with_defaults`, `test_search_min_value_and_sort_combined` for min-value and sort testing
  - `tests/cli/search.rs:939-1214` - Added `test_search_exclude_mocs`, `test_search_exclude_mocs_no_results`, `test_search_exclude_mocs_with_filters`, `test_search_exclude_mocs_with_min_value` for exclude-mocs testing
  - Learnings: All three flags (`--exclude-mocs`, `--min-value`, `--sort`) work correctly; MOC filtering happens after sorting; tests verify JSON output format uses `"type"` field not `"note_type"`
- [x] Add tests for `compact apply --from-stdin` and `--notes-file`.
  - `tests/cli/compact/commands.rs:830-1248` - Added test_compact_apply_from_stdin, test_compact_apply_notes_file, test_compact_apply_mixed_sources, test_compact_apply_no_sources_error
  - Learnings: Both flags work correctly; --from-stdin reads note IDs from stdin (one per line); --notes-file reads from a file path; multiple sources (--note flags, --from-stdin, --notes-file) can be combined and are deduplicated; empty lines and whitespace are trimmed; all three output formats (human, json, records) are tested; error handling verified when no sources provided

### Value Model Tests (`specs/value-model.md`)
- [x] Add tests for `qipu value set/show` output + validation.
  - `tests/cli/value.rs:1-402` - Added comprehensive tests for value set/show commands
  - `src/lib/db/notes/create.rs:37,128` - Fixed database insert to preserve None values instead of defaulting to 50
  - Learnings: Value field was being defaulted to 50 during database insert (`.or(Some(50))`), preventing distinction between "unset" (None) and "explicitly set to 50" (Some(50)). This broke the `value show` command's "(default)" label. Fixed by removing the default conversion and relying on SQLite to store NULL for None values. Tests cover: basic set/show, min/max boundaries (0-100), validation (>100 rejected), file path operations, default value display with "(default)" label, frontmatter persistence, and error handling for nonexistent notes.
- [x] Add tests for `search --sort value`.
  - `tests/cli/search.rs:689-937` - Added comprehensive tests for search --sort value
  - Learnings: Tests cover basic value sorting (descending order), default value handling (None treated as 50), and combined --min-value + --sort value filters. Implementation at `src/commands/search.rs:138-147` sorts by value descending with stable ID-based tiebreaker. All tests pass.
- [x] Add tests for `list --min-value` and `context --min-value`.
  - `tests/cli/list.rs:112-310` - Added 4 comprehensive tests for list --min-value (all match, some match, none match, defaults)
  - `tests/cli/context/basic.rs:647-784` - Added comprehensive test for context --min-value with multiple value thresholds
  - Learnings: Tests were already fully implemented; verified all tests pass with cargo test
- [x] Add tests for `--ignore-value` traversal ordering.
  - `tests/cli/link/ignore_value.rs:1-502` - Added comprehensive tests for --ignore-value flag
  - Added 5 tests covering weighted vs unweighted traversal for both link tree and link path commands
  - Tests verify: default weighted behavior, --ignore-value unweighted behavior, hop count differences, and edge cost calculations
  - Learnings: The --ignore-value flag switches from Dijkstra (weighted, default) to BFS (unweighted) traversal. The implementation at `src/lib/graph/bfs.rs:299-310` correctly applies value-based costs when ignore_value=false and uniform costs when ignore_value=true
- [x] Add CLI coverage for `--unweighted`/`--weighted` aliases (spec names vs current flags).
  - `src/cli/link.rs:110,156` - Added `alias = "unweighted"` attribute to `ignore_value` field in both Tree and Path commands
  - `tests/cli/link/ignore_value.rs:600-800` - Added `test_unweighted_alias_tree()` and `test_unweighted_alias_path()` to verify alias functionality
  - Learnings: The `--unweighted` alias now works as specified in the Value Model spec; both flags (`--ignore-value` and `--unweighted`) have identical behavior for unweighted BFS traversal

### Export Tests (`specs/export.md`)
- [x] Add tests for `--tag`/`--query` selection ordering.
  - `tests/cli/export.rs:374-624` - Added 5 comprehensive tests for deterministic ordering
  - Tests verify tag/query exports sort by (created_at, id) and MOC exports preserve MOC order
  - Learnings: Implementation at `src/commands/export/mod.rs:110-111` correctly sorts by (created_at, id) for non-MOC exports; MOC exports skip sorting to preserve link order as specified
- [x] Add tests for `--mode bibliography`.
  - `tests/cli/export.rs:639-895` - Added 7 comprehensive tests for bibliography mode
  - Tests cover: basic bibliography export, empty sources case, multiple notes with sources, deterministic ordering (sorted by URL), source format variations (with/without title/accessed date), tag selection, and "bib" alias
  - Learnings: Bibliography mode extracts sources from selected notes and produces markdown output with sources sorted alphabetically by URL; implementation at `src/commands/export/emit/bibliography.rs:4-41` correctly handles all source field combinations and references back to originating notes
- [x] Add tests for `--link-mode markdown` and `--link-mode preserve`.
  - `tests/cli/export.rs:957-1280` - Added 6 comprehensive tests for link-mode variations
  - Tests cover: preserve mode (keeps wiki links unchanged), markdown mode with basic links, markdown mode with labels, markdown mode with multiple cross-referenced notes, preserve mode with MOC, and markdown mode with MOC
  - Learnings: The `preserve` mode keeps wiki links as-is (`[[qp-xxx]]`); the `markdown` mode converts wiki links to markdown file links using absolute paths (`[label](path/to/qp-xxx.md)`); labels are preserved during conversion; both modes work correctly with MOC-driven exports

### Graph Traversal Tests (`specs/graph-traversal.md`)
- [x] Add tests for semantic inversion in `link tree`/`link path`.
  - `tests/cli/link/tree.rs:1138-1377` - Added 3 comprehensive tests for link tree semantic inversion
  - `tests/cli/link/path.rs:53-355` - Added 4 comprehensive tests for link path semantic inversion
  - Learnings: Semantic inversion affects both traversal and presentation; when enabled (default), virtual inverted edges are created and followed during traversal; when disabled with `--no-semantic-inversion`, raw backlinks are used with original types retained; type filtering applies to inverted types when semantic inversion is enabled, and to original types when disabled; JSON output correctly handles both modes; all 7 new tests pass
- [x] Add tests for `max_nodes`, `max_edges`, and `max_fanout` truncation.
  - `tests/cli/link/tree.rs:1457-1697` - Added 3 comprehensive tests for link tree truncation limits
  - Learnings: All three truncation limits (`--max-nodes`, `--max-edges`, `--max-fanout`) correctly stop traversal when their respective limits are reached and report truncation in JSON output with appropriate truncation_reason field; max-fanout test uses `--direction out` to avoid counting inverted edges in bidirectional traversal; all tests verify both truncation flag and actual node/edge counts match expected limits

### Records Output Tests (`specs/records-output.md`)
- [x] Add `max-chars` truncation tests for link tree/path records output.
  - `tests/cli/link/tree.rs:1699-1909` - Added 3 comprehensive tests for link tree max-chars truncation
  - `tests/cli/link/path.rs:1198-1418` - Added 3 comprehensive tests for link path max-chars truncation
  - Learnings: Tests cover no truncation (large budget), truncation (small budget), and header-only output (minimal budget ~120-140 chars); verified budget enforcement and truncated flag in header; all tests verify exact character count limits

### Workspaces Tests (`specs/workspaces.md`)
- [x] Add tests for `workspace merge --dry-run`, `skip`, and `overwrite` strategies.
   - `tests/workspace_merge_test.rs:218-502`
   - Added `test_workspace_merge_dry_run()` to verify dry-run shows changes without making them
   - Added `test_workspace_merge_skip_strategy()` to verify skip keeps target notes unchanged
   - Added `test_workspace_merge_overwrite_strategy()` to verify overwrite replaces target notes with workspace versions
   - Learnings: Tests verify that dry-run mode reports conflicts and additions without modifying stores; skip strategy preserves target notes while adding new notes; overwrite strategy replaces conflicting notes with incoming versions

### Semantic Graph Tests (`specs/semantic-graph.md`)
- [x] Add tests for additional standard types and custom inverses.
  - `tests/cli/link/add_remove.rs:359-710`
  - Added 8 new tests covering all standard link types: `part-of`/`has-part`, `follows`/`precedes`, `contradicts`/`contradicted-by`, `answers`/`answered-by`, `refines`/`refined-by`, `same-as` (symmetric), `alias-of`/`has-alias`, and unknown type fallback (`inverse-<type>`)
  - Each test verifies both forward and inverse link display, semantic inversion with `(virtual)` marker, and proper inverse type transformation
  - Learnings: All standard types from semantic-graph spec are now comprehensively tested; unknown types correctly fall back to `inverse-<type>` pattern; symmetric types (same-as, related) show same type in both directions

### Similarity Ranking Tests (`specs/similarity-ranking.md`)
- [x] Add tests for default similarity thresholds and field weighting.
  - `src/lib/similarity/mod.rs:545-1075` - Added 6 comprehensive unit tests
  - Tests cover: field weighting (title 2.0 vs tags 1.5 vs body 1.0), combined field weights, default threshold 0.3 for related notes, default threshold 0.85 for duplicates
  - Learnings: Field weights (title=2.0, tags=1.5, body=1.0) are correctly applied during indexing; cosine similarity with single shared term produces 1.0 similarity regardless of TF weight (this is mathematically correct); tests must use multiple terms with different field distributions to verify weight effects; threshold tests verify 0.3 for context expansion and 0.85 for duplicate detection as specified
- [x] Add end-to-end tests for stop-word filtering.
  - `tests/cli/doctor.rs:393-623` - Added 6 comprehensive end-to-end tests
  - Tests cover: stop words don't affect duplicate detection, notes differing only by stop words are detected as duplicates, content word differences prevent false positives, specific stop words from spec are filtered (a, an, the, and, or, is, with, in, for, at, by, etc.), stop words filtered from both title and body, field weighting works correctly with stop words
  - Learnings: Stop-word filtering is implemented in `src/lib/text/mod.rs:32-40` and used by the similarity engine for duplicate detection (via `qipu doctor --duplicates`), not for FTS5 full-text search; FTS5 uses SQLite's Porter stemmer tokenizer which does not filter stop words; end-to-end tests verify stop-word filtering works correctly in the context of duplicate detection using the `doctor` command

### Pack Tests (`specs/pack.md`)
- [x] Add tests for `--tag`, `--moc`, `--query`, and "no selectors" dump.
  - `tests/cli/dump.rs:793-1320` - Added 5 comprehensive tests for dump selection modes
  - Added `test_dump_by_tag()` - Verifies `--tag` selector includes only notes with matching tag
  - Added `test_dump_by_moc()` - Verifies `--moc` selector includes notes linked from MOC
  - Added `test_dump_by_query()` - Verifies `--query` selector includes notes matching search query
  - Added `test_dump_no_selectors_full_store()` - Verifies default behavior dumps entire store with all notes and links preserved
  - Added `test_dump_tag_with_traversal()` - Verifies combining `--tag` selector with `--max-hops` traversal expands to include linked notes
  - Learnings: All four selection modes work correctly as specified; no selectors dumps full store; selectors can be combined with traversal options for graph expansion
- [x] Add tests for attachments round-trip and `--no-attachments`.
  - `tests/pack_tests.rs:857-1270` - Added 3 comprehensive tests for attachment handling
  - Added `test_pack_attachments_roundtrip()` - Verifies attachments are included by default, packed into records format, and restored correctly on load (tests text, JSON, and binary PNG files)
  - Added `test_pack_no_attachments_flag()` - Verifies `--no-attachments` flag excludes attachments from pack and they are not restored on load
  - Added `test_pack_attachments_multiple_notes()` - Verifies multiple notes can reference attachments, including shared attachments referenced by multiple notes
  - Learnings: Attachments are discovered by parsing markdown link syntax (`![alt](path)`) in note body; note body must be reindexed after manual file updates for database to reflect changes; attachment data is base64-encoded in pack records format
- [x] Add tests for pack version/store version compatibility errors.
  - `tests/pack_tests.rs:1266-1411` - Added 3 comprehensive tests for version compatibility
  - Added `test_pack_unsupported_version_error()` - Verifies loading pack with version 2.0 fails with "unsupported pack version" error
  - Added `test_pack_store_version_too_high()` - Verifies loading pack with store_version=999 fails with "please upgrade qipu" error
  - Added `test_pack_store_version_backward_compatible()` - Verifies loading pack with store_version=0 succeeds (backward compatible)
  - Learnings: Version validation at `src/commands/load/mod.rs:59-72` correctly rejects unsupported pack versions (only 1.0 supported) and store versions higher than STORE_FORMAT_VERSION (currently 1); lower store versions are accepted for backward compatibility per spec requirement

### Provenance Tests (`specs/provenance.md`)
- [x] Add tests for default `verified=false` behavior on LLM-origin notes.
  - `tests/cli/provenance.rs:127-189` - Added `test_llm_generated_defaults_verified_false()` and `test_llm_generated_can_override_verified()`
  - Learnings: Tests verify that notes created with `--generated-by` automatically get `verified=false` unless explicitly overridden with `--verified=true`; implementation at `src/commands/create.rs:61-68` correctly sets verified field based on presence of generated_by flag

---

## P3: Unimplemented Optional / Future

### Custom Metadata (`specs/custom-metadata.md`)
- [x] Add custom frontmatter field and DB storage (foundation).
  - `src/lib/note/frontmatter.rs:3,56-57,79` - Added `custom: HashMap<String, serde_yaml::Value>` field
  - `src/lib/db/schema.rs:6,47` - Bumped schema to v6; added `custom_json TEXT DEFAULT '{}'` column
  - `src/lib/db/notes/create.rs:24,29,46` - Serialize custom HashMap to JSON during insert
  - `src/lib/db/notes/read.rs:226,245,271,352,404,472,533` - Deserialize custom JSON when loading notes
  - `src/commands/load/mod.rs:245` - Initialize custom field in pack load
  - `src/lib/db/tests.rs:911` - Updated schema version test
  - Learnings: Schema version bump triggers auto-rebuild; custom fields round-trip through JSON storage; empty HashMap serializes to `{}`; all existing tests pass with new field
- [x] Implement CLI commands with type detection:
    - `src/cli/custom.rs` - Created CustomCommands enum with Set/Get/Show/Unset subcommands
    - `src/cli/commands.rs:169-176` - Added Custom subcommand with `#[command(hide = true)]` per spec
    - `src/commands/custom.rs` - Implemented all four commands with YAML type detection via `serde_yaml::from_str()`
    - `src/commands/dispatch/mod.rs:122,435-525` - Added dispatch handler for custom commands
    - `src/cli/commands.rs:56,302,304` - Added `--custom` filter to List and `--custom-filter`/`--custom` flags to Context commands
    - `src/commands/list.rs:27,47-65` - Added custom metadata filtering to list command
    - `src/commands/context/types.rs:18-19` - Added custom_filter and include_custom fields to ContextOptions
    - `src/commands/context/mod.rs:336-359` - Added custom metadata filtering to context selection
    - `src/commands/doctor/content.rs:151-172` - Added check_custom_metadata() to validate custom fields
    - `src/commands/doctor/mod.rs:71` - Integrated custom metadata check into doctor command
    - Learnings: Type detection using `serde_yaml::from_str()` provides intuitive CLI experience (numbers, booleans, strings, arrays, objects); custom filter uses simple key=value format with type-aware comparison; doctor validation warns on >10KB custom blocks; all 642 tests pass

### Distribution (`specs/distribution.md`)
- [x] Add release automation and install scripts (GitHub releases + installers).
  - `.github/workflows/release.yml:1-156` - Created release workflow with multi-platform builds (x86_64/aarch64 for Linux/macOS, x86_64 for Windows)
  - `scripts/install.sh:1-168` - Created Unix installer with platform detection, checksum verification, and PATH setup guidance
  - `scripts/install.ps1:1-201` - Created Windows PowerShell installer with platform detection, checksum verification, and PATH configuration
  - Learnings: Release workflow uses cross-compilation via `cross` tool for aarch64-linux target; both installers fetch latest release from GitHub API, verify checksums, and provide PATH setup guidance; Windows installer uses Expand-Archive for zip extraction; Unix installer uses tar; checksums are verified before installation to ensure binary integrity
- [x] Add checksum generation + installer verification.
  - `.github/workflows/release.yml:99-116` - Checksums generated for all platform binaries (SHA256 for Unix tar.gz, SHA256 for Windows zip)
  - `scripts/install.sh:86-99` - Unix installer verifies SHA256 checksums before installation
  - `scripts/install.ps1:80-96` - Windows installer verifies SHA256 checksums before installation
  - Learnings: Checksum verification is integrated into both installers; Unix uses shasum, Windows uses Get-FileHash; installation fails if checksums don't match; both checksums and binaries are uploaded as release assets

### Export (`specs/export.md`)
- [x] Add optional BibTeX/CSL JSON outputs.
  - `src/commands/export/emit/bibliography.rs:1-198` - Added BibFormat enum with Markdown, BibTeX, and CslJson variants
  - `src/cli/commands.rs:352-354` - Added --bib-format CLI flag with markdown default
  - `src/commands/dispatch/io.rs:17-56` - Updated handler to accept and parse bib_format parameter
  - `src/commands/export/mod.rs:82,145` - Added bib_format to ExportOptions and passed to export_bibliography
  - `tests/cli/export.rs:1294-1527` - Added 6 comprehensive tests for BibTeX and CSL JSON formats
  - Learnings: BibTeX format uses @misc entries with citation keys generated from URL domains; CSL JSON format uses webpage type with date-parts array for accessed dates; both formats support empty source lists with appropriate empty outputs; deterministic ordering by URL maintained across all formats
- [x] Add transitive export traversal (depth-limited).
  - `src/cli/commands.rs:357-359` - Added --max-hops CLI flag with default value 0 (no expansion)
  - `src/commands/export/mod.rs:83` - Added max_hops field to ExportOptions struct
  - `src/commands/dispatch/io.rs:30,52` - Updated handle_export to accept and pass max_hops parameter
  - `src/commands/dispatch/mod.rs:202,214` - Updated dispatch handler to extract and pass max_hops from CLI
  - `src/commands/export/plan.rs:4,68-103,212-268` - Added transitive traversal logic using BFS with HopCost tracking; imports Direction, HopCost, and TreeOptions; performs bidirectional traversal from initially selected notes
  - `tests/cli/export.rs:1534-1854` - Added 6 comprehensive tests covering: no traversal (max-hops=0), one-hop expansion, two-hop expansion, tag selection with traversal, bidirectional traversal, and JSON format output
  - Learnings: Transitive traversal expands note selection by following links up to max-hops distance; traversal is bidirectional by default (Direction::Both); uses simple BFS queue with accumulated cost tracking; integrates cleanly with existing selection modes (--note, --tag, --moc, --query); all 795 tests pass
- [ ] Add pandoc/PDF integration (future).
  - `src/commands/export/mod.rs:13-261`

### Similarity Ranking (`specs/similarity-ranking.md`)
- [ ] Add clustering/"see also" features for MOC generation.
  - `src/lib/similarity/mod.rs:27-135`

### Semantic Graph (`specs/semantic-graph.md`)
- [ ] Support per-link-type hop costs (currently hardcoded to 1.0).
  - `src/lib/graph/types.rs:48-53`

### LLM User Validation (`specs/llm-user-validation.md`)
- [ ] Include prime output hash in cache key.
  - `crates/llm-tool-test/src/results.rs:233-272`
- [ ] Fix MinLinks gate no-op in mock adapter.
  - `crates/llm-tool-test/src/adapter/mock.rs:60-61`
- [ ] Avoid error-swallowing defaults in evaluation metrics.
  - `crates/llm-tool-test/src/evaluation.rs:72-114`
  - `crates/llm-tool-test/src/evaluation.rs:410-458`

### Knowledge Model (`specs/knowledge-model.md`)
- [ ] Add quality bar / rationale validation and duplicate detection (future).
  - `src/commands/doctor/content.rs:109-127`

---

## P4: Spec Ambiguity

### LLM Context (`specs/llm-context.md`)
- [ ] Clarify whether store paths should be relative or absolute in outputs.
  - `src/commands/context/human.rs:86-88`
  - `src/commands/context/json.rs:87-88`
  - `src/commands/context/records.rs:203-207`
  - `src/commands/prime.rs:72-80`

### Indexing/Search (`specs/indexing-search.md`)
- [ ] Confirm whether backlink index must be stored or can be derived.
  - `src/lib/index/types.rs:161-169`

### Workspaces (`specs/workspaces.md`)
- [ ] Decide expected gitignore behavior for `--temp` workspaces.
  - `src/commands/workspace/new.rs:33-101`

### Telemetry (`specs/telemetry.md`)
- [ ] Spec is DRAFT and explicitly prohibits implementation; confirm when to revisit.
  - `specs/telemetry.md:1-5`

### Performance Tests (`tests/performance_tests.rs`)
- [ ] Review and validate performance test thresholds.
  - Current state: Tests use spec-compliant 1s budget for 10k notes, but actual performance is ~500-600ms
  - Questions to resolve:
    - Should we have a tighter "regression detection" threshold separate from spec compliance?
    - What are realistic baseline numbers across different hardware?
    - Should performance tests run in CI, or be marked `#[ignore]` by default?
  - `tests/performance_tests.rs:188-240` - 10k note search test
  - `tests/performance_tests.rs:50-120` - list/index performance tests

### Test Suite Optimization
- [ ] Review and rationalize test suite for faster feedback loops.
  - As the test suite grows, execution time increases; review for opportunities to optimize.
  - Questions to resolve:
    - Which tests are redundant or overlapping?
    - Can slow integration tests be split into fast unit tests?
    - Should tests be organized into tiers (fast/slow) for different CI stages?
    - Are there tests that can be parallelized more effectively?
    - Can expensive setup/teardown be shared across test groups?
  - `tests/` - Integration test suite
  - `src/lib/db/tests.rs` - Database unit tests
  - `crates/llm-tool-test/` - LLM validation tests

---

## Notes

- Audit Date: 2026-01-20
- Recent completions include workspaces `--empty` flag, structured logging verification, file size refactoring, and MOC ordering preservation
- Documentation additions: `docs/building-on-qipu.md` and type detection spec in `specs/custom-metadata.md`
- Test fixes: Added `extract_id` helper to test support to handle two-line create output (ID + path); updated ID extraction in test files; added index calls to test cases that manually create notes
