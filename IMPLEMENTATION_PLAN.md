# Qipu Implementation Plan

This document tracks **concrete implementation tasks** - bugs to fix, features to complete, and tests to add. For exploratory future work and open questions from specs, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status
- Test baseline: 634 tests pass (228 unit + 252 integration + 6 golden + 7 pack + 6 perf + 1 workspace_from_note + 3 workspace_merge + 130 llm-tool-test)
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
 - [x] Guard rails (LLM_TOOL_TEST_ENABLED) are missing before running scenarios.
   - `crates/llm-tool-test/src/commands.rs:28-48`
   - Learnings: Added check at start of handle_run_command to require LLM_TOOL_TEST_ENABLED environment variable; returns clear error message if not set; prevents accidental execution of LLM test scenarios
 - [x] `--dry-run` errors even for single-scenario runs.
   - `crates/llm-tool-test/src/run.rs:185-188`
   - Learnings: Replaced bail!("Dry run not supported in matrix mode") with returning a mock ResultRecord; dry-run now creates a dummy record with zero metrics and "Dry run" outcome; allows previewing what would run without execution

### Database Schema - Missing Fields (originally 22 test failures, all fixed)

**Root Cause**: Several frontmatter fields are stored in note files but NOT in the database schema. When notes are retrieved via `list_notes_full()` or `get_note()`, these fields are hardcoded to empty/null values instead of being read from the database.

 - [x] **Compacts field not stored in database** (11 tests - all fixed)
   - `src/lib/db/schema.rs:6,29-41` - Added `compacts TEXT DEFAULT '[]'` column, bumped schema version to 4
   - `src/lib/db/notes/create.rs:18-36,80-113` - Serialize compacts to JSON in both `insert_note` and `insert_note_internal`
   - `src/lib/db/notes/read.rs:222-331,348-476` - Deserialize compacts from JSON in both `get_note` and `list_notes_full`
   - `src/lib/db/tests.rs:911` - Updated schema version test expectation from 3 to 4
   - `src/lib/db/mod.rs:159-166` - Added Drop impl to checkpoint WAL on close for test reliability
   - `src/lib/db/repair.rs:60` - Changed `>` to `>=` to catch files modified at exact sync time
   - `tests/cli/compact/annotations.rs:55-59` - Added index call after manual file edits
   - `tests/cli/compact/commands.rs:200-206` - Added index call after manual file edits
   - **Status**: Complete. All 15 compaction tests pass.
   - **Learnings**: Tests that manually edit note files must call `qipu index` to sync changes to database. WAL checkpoint on Drop ensures rapid open/close cycles in tests see consistent data. Repair mtime comparison must use `>=` to catch edge case of same-second modifications.

 - [x] **Provenance fields not stored in database** (4 tests - all fixed)
   - `src/lib/db/schema.rs:6,28-46` - Added columns: `author TEXT`, `verified INTEGER`, `source TEXT`, `sources TEXT DEFAULT '[]'`, `generated_by TEXT`, `prompt_hash TEXT`. Bumped schema version to 5
   - `src/lib/db/notes/create.rs:18-47,101-129` - Serialize sources to JSON, convert verified to integer in both `insert_note` and `insert_note_internal`
   - `src/lib/db/notes/read.rs:222-343,360-490` - Deserialize provenance fields from database in both `get_note` and `list_notes_full`
   - `src/lib/db/tests.rs:911` - Updated schema version test expectation from 4 to 5
   - **Status**: Complete. All 4 provenance tests now pass (test_context_prioritizes_verified, test_create_with_provenance, test_context_json_with_provenance, test_context_records_with_body_and_sources).
   - **Learnings**: Schema version bump from 4 to 5 triggers full database rebuild. The verified boolean is stored as INTEGER (0/1) in SQLite. Sources is serialized as JSON array. All provenance fields are properly nullable.

### Edge Insertion - Duplicate Edge Handling (4 tests)

**Root Cause**: When a note is created with inline wiki-links `[[id]]`, edges are inserted. If a test then tries to add a typed link to the same target with `link add`, it fails with UNIQUE constraint violation because inline links create edges with `link_type='related'`.

 - [x] **Prevent inline links from being stored as typed links in frontmatter**
   - Failing tests (now fixed): `test_link_path_inline_only`, `test_link_path_typed_only`, `test_link_tree_inline_only`, `test_link_tree_typed_only`
   - `src/lib/index/links.rs:144-148` - Fixed deduplication to remove duplicates by (to, link_type) only, ensuring typed links take precedence
   - `src/lib/db/notes/read.rs:327-343,505-522` - Fixed `get_note` and `list_notes_full` to only include typed links (inline=0) in frontmatter
   - **Root Cause**: When loading notes from the database, both `get_note` and `list_notes_full` were adding ALL edges (including inline links with inline=1) to the frontmatter as typed links. This caused inline links to be persisted as typed links when the note was saved, creating duplicates.
   - **Fix**: Modified both functions to check the `inline` column and only add edges with `inline=0` to the frontmatter. Inline links remain in the note body and are extracted dynamically during indexing. Also updated deduplication logic to prefer typed links over inline links when both exist to the same target with the same type.
   - **Learnings**: The database correctly distinguishes inline and typed links via the `inline` column. The bug was in the note retrieval logic that wasn't respecting this distinction. The PRIMARY KEY constraint on (source_id, target_id, link_type) is correct and doesn't need UPSERT - we just needed to prevent creating duplicate edges by not adding inline links to frontmatter.

### Miscellaneous Test Failures (2 tests)

 - [x] **Doctor broken link detection test expects wrong exit code**
   - `src/lib/db/mod.rs:89-93` - Removed auto-repair on consistency check failure
   - `src/commands/merge/mod.rs:127-129` - Fixed merge command to use Store::delete_note() instead of just removing file
   - `src/lib/store/lifecycle.rs:225` - Removed #[allow(dead_code)] from delete_note method
   - **Root Cause**: Database::open was running incremental_repair when consistency check failed, removing missing files from database before doctor could detect them. This was masking the issue.
   - **Fix**: Removed auto-repair from Database::open. Consistency check now only logs warnings. Doctor command detects missing files and reports them with exit code 3. Fixed merge command to properly delete notes from both filesystem and database.
   - **Learnings**: Auto-repair should only happen when explicitly requested (via index command or doctor --fix). Commands that delete files (like merge) must also delete from database to maintain consistency.



---

## P2: Missing Test Coverage & Gaps

### CLI Tool (`specs/cli-tool.md`)
- [x] Add performance budget coverage for search at 10k notes (spec target) instead of 2k baseline.
  - `tests/performance_tests.rs:188-240`
  - Learnings: Changed note_count from 2000 to 10000 and renamed test from test_search_performance_2k_notes to test_search_performance_10k_notes; actual performance is 186ms for 10k notes, well within spec target of <1s

### Operational Database (`specs/operational-database.md`)
- [x] Search ranking boosts don't align with spec weights.
   - `src/lib/db/search.rs:68-105`
   - Learnings: Updated BM25 column weights to match spec (Title 2.0x, Body 1.0x, Tags 1.5x). Added separate queries for title-only matches (+2.0 boost) and tag-only matches (+3.0 boost) to ensure they rank above body matches. Simplified query from three separate queries with constant boosts to single query with BM25 column weights plus two additional queries for title/tag boosts.
- [x] Tag frequency statistics are missing.
   - `src/lib/db/schema.rs:19-72`
   - Learnings: Added `get_tag_frequencies()` method to Database module that returns tags ordered by count DESC, then tag ASC; added CLI `qipu tags list` command with support for human, json, and records output formats; added delegation through Store::get_tag_frequencies() to Database::get_tag_frequencies()

### Indexing/Search (`specs/indexing-search.md`)
- [x] Ignore qp-style links outside the store (currently treated as resolved).
    - `src/lib/index/links.rs:80-94`
    - Learnings: Added `continue` statement after adding to unresolved set for typed links, wiki links, and markdown links. This ensures that links to non-existent notes are ignored (not added to edges) while still being tracked in unresolved for doctor reporting. Updated `test_unresolved_links` test to reflect new behavior (edges.len() should be 0, not 1).
 - [x] Related-notes feature (shared tags / 2-hop) is missing.
   - `src/lib/similarity/mod.rs:95-163`
   - `src/commands/context/mod.rs:230-268`
   - Learnings: Added `find_by_shared_tags()` method that uses Jaccard similarity (intersection/union) to find notes sharing tags. Added `find_by_2hop_neighborhood()` method that finds notes within 2 hops in the link graph, scoring by number of 2-hop paths. Updated context command to use all three relatedness methods (TF-IDF, shared tags, 2-hop) when `--related` flag is specified. Added tests for both new methods.

### Storage Format (`specs/storage-format.md`)
- [x] Configurable store root is not supported in config.
  - `src/lib/config.rs:14-117`
  - `src/lib/store/paths.rs:46-115`
  - Learnings: Added `store_path` field to StoreConfig (optional, relative or absolute path); modified discover_store() to check config for custom path; relative paths are resolved relative to project root (directory containing .qipu/); added tests for both relative and absolute store paths
- [x] Optional wiki-link rewrite/canonicalization is not implemented.
  - `src/lib/index/links.rs:155-200`
  - `src/cli/commands.rs:114-119`
  - `src/commands/index.rs:13-68`
  - `src/lib/config.rs:44-45`
  - Learnings: Added `rewrite_wiki_links` function to convert `[[id]]` and `[[id|label]]` to `[label](id.md)`. Added `--rewrite-wiki-links` flag to index command. Added `rewrite_wiki_links` config field to StoreConfig (default false). Added tests for simple links, labeled links, multiple links, and integration test. Feature is opt-in via CLI flag.
- [x] No cross-branch ID collision avoidance.
  - `src/lib/store/lifecycle.rs:177-215`
  - `src/lib/git.rs:228-342`
  - Learnings: Added `get_ids_from_all_branches()` function to git module that scans all git branches (local and remote) for note files and extracts IDs from filenames. Updated `Store::existing_ids()` to query both current database and all git branches when generating new IDs. Added `find_repo_root()` helper to locate git repository root. This provides additional collision protection for multi-branch/multi-agent workflows beyond the cryptographic collision resistance already provided by timestamp+random hash generation.

### Export (`specs/export.md`)
- [x] Outline export appends outbound edges beyond MOC body ordering.
  - `src/commands/export/emit/outline.rs:54`
  - Learnings: Removed `.chain()` call that was appending database outbound edges after MOC body ordering; outline export now strictly follows MOC body link order as per spec
- [x] Query export caps results at 200 notes.
  - `src/commands/export/plan.rs:54`
  - Learnings: Increased query result limit from 200 to 10,000 notes for export command; export is a deliberate bulk operation that should not arbitrarily cap results; 10,000 is 50x the previous limit and handles most realistic use cases while providing safety bound
- [x] Outline export falls back to bundle when `--moc` is missing.
  - `src/commands/export/emit/outline.rs:23-28`
  - `tests/cli/export.rs:344-377`
  - Learnings: Fallback logic was already implemented; added test coverage to verify outline mode falls back to bundle output when --moc flag is not provided

### Semantic Graph (`specs/semantic-graph.md`)
- [x] Context selection doesn't prefer typed links over `related` when constrained.
   - `src/commands/context/mod.rs:337-368`
   - `tests/cli/context/budget.rs:244-391`
   - **Status**: Already implemented. Context selection sorts notes by: (1) verified status, (2) link type priority (`part-of` and `supports` get highest priority 0, other typed links get priority 1, `related` gets priority 2), (3) created date, (4) ID. This ensures typed links are preferred over `related` when budget constraints are applied.
   - **Learnings**: Added comprehensive test that verifies typed links appear before `related` links in output and are prioritized when budget constraints force selection. The sorting happens before budget application in `apply_budget()`.
- [x] Doctor does not validate semantic misuse of standard link types.
   - `src/lib/db/validate.rs:121-147` - Added `get_all_typed_edges()` method to Database
   - `src/commands/doctor/database.rs:86-220` - Implemented `check_semantic_link_types()` validation function
   - `src/commands/doctor/mod.rs:62` - Added call to semantic link validation in doctor execute
   - `src/commands/doctor/mod.rs:420-549` - Added 5 comprehensive tests for semantic link validation
   - **Validations Implemented**:
     - Conflicting relationships: warns when a note both `supports` and `contradicts` the same target
     - Self-referential identity links: warns when a note has `same-as` or `alias-of` pointing to itself
     - Mixed identity types: warns when a note has both `same-as` and `alias-of` to the same target
   - **Learnings**: Semantic validation runs at database level using typed edges (inline=0). All validations produce warnings (not errors) since these are semantic issues that don't break functionality. The validation integrates seamlessly with existing doctor checks.

### Records Output (`specs/records-output.md`)
- [x] Link tree/path records rely on traversal order (no explicit ordering).
   - `src/lib/graph/bfs.rs:798-806,928-936` - Added explicit neighbor sorting in both unweighted BFS and weighted Dijkstra path finding
   - Learnings: While tree traversal already had explicit neighbor sorting (bfs.rs:208-213), path finding was missing this. Added `neighbors.sort_by()` after edge collection and before processing to ensure deterministic path selection when multiple equal-cost paths exist. The sorting order matches tree traversal: by (link_type, neighbor_id).

### LLM Context (`specs/llm-context.md`)
- [x] No per-note truncation marker when budgets are applied.
  - `src/commands/context/budget.rs:7-92` - Modified to return excluded notes
  - `src/commands/context/human.rs:10-223` - Added "Excluded Notes" section
  - `src/commands/context/json.rs:10-228` - Added "excluded_notes" array
  - `src/commands/context/records.rs:10-290` - Added "D excluded" markers
  - `tests/cli/context/budget.rs:412-502` - Added comprehensive test
  - **Learnings**: When budget constraints prevent notes from being included, all three output formats now show which notes were excluded. Human format shows an "Excluded Notes" section, JSON format includes an "excluded_notes" array with id/title, and records format adds "D excluded" markers. Records format budget calculation was updated to account for excluded note marker size to ensure exact budget compliance.
- [x] Default output uses summaries unless `--with-body` (spec implies full body).
  - `src/cli/commands.rs:267-276` - Added `--summary-only` flag; hid `--with-body` (now deprecated but preserved for compatibility)
  - `src/commands/dispatch/mod.rs:145-176` - Changed default to use full body; `use_full_body = !summary_only || *with_body`
  - **Learnings**: Spec says "Preserve original note markdown as-is" which means full body by default. Changed default behavior to include full body content, added `--summary-only` flag to opt into the old behavior (summary extraction). The `--with-body` flag is kept as a hidden option for backward compatibility but is now redundant since full body is the default.
- [x] Bundle output omits empty metadata headers (`Path`, `Tags`, `Sources`) when values are absent.
  - `src/commands/context/human.rs:116-118,121-123,159-168`
  - **Status**: Already implemented correctly. Path is only shown when Some (line 116-118), Tags only when not empty (line 121-123), and Sources only when not empty (line 159-168).
  - **Learnings**: The implementation already follows best practices for clean output by omitting empty metadata fields. Type is always shown since every note has a type.

### Compaction (`specs/compaction.md`)
- [x] Link outputs omit `compacts=`/`compaction=` annotations for digest nodes.
  - `src/commands/link/human.rs:1-110` - Added compacts=/compaction= annotations to link list and link path human output
  - `src/commands/link/tree.rs:133-374` - Added compacts=/compaction= annotations to link tree human and records output
  - `src/commands/link/records.rs:1-370` - Added compacts=/compaction= annotations to link list and link path records output
  - `src/commands/link/list.rs:148-175` - Build note_map for compaction percentage calculation
  - `src/commands/link/path.rs:71-103` - Build note_map for compaction percentage calculation
  - **Learnings**: Compaction annotations (compacts=N and compaction=P%) are now shown for digest nodes in all link command outputs (list, tree, path) across all formats (human, records). The compaction percentage requires building a note_map (HashMap<&str, &Note>) to look up note data for size estimation. The annotations appear directly on the N record line for records format, and as a separate line after the node for human format.
- [x] JSON outputs ignore compaction truncation flags.
  - `src/commands/search.rs:212-232` - Added `compacted_ids_truncated` flag to JSON output when truncation occurs
  - `src/commands/show.rs:97-113` - Added `compacted_ids_truncated` flag to JSON output when truncation occurs
  - **Learnings**: Both search and show JSON outputs now properly respect the `truncated` flag from `get_compacted_ids()` and include a `compacted_ids_truncated: true` field in JSON when compaction IDs are truncated by `--compaction-max-nodes`. This matches the behavior already implemented in context JSON output.
- [x] `compact show` ignores `--compaction-max-nodes` and truncation.
  - `src/commands/compact/show.rs:46-50,110-123,133-150,153-165,90-98` - Changed to use `get_compacted_ids()` with `compaction_max_nodes` parameter; added truncation indicators to all three output formats
  - `tests/cli/compact/commands.rs:578-725` - Added comprehensive test for `--compaction-max-nodes` truncation behavior
  - **Root Cause**: The `compact show` command was directly accessing `ctx.get_compacted_notes()` which returns a raw Vec<String> without truncation support.
  - **Fix**: Changed to use `ctx.get_compacted_ids()` which accepts the `max_nodes` parameter and returns a tuple with (ids, truncated). Added truncation indicators to all formats: human shows "(truncated: showing X of Y notes)", JSON adds `compacted_ids_truncated: true`, and records adds `D compacted_truncated max=X total=Y`. Also applied limit to nested tree when depth > 1.
  - **Learnings**: The `get_compacted_ids()` method is the correct API for getting compacted IDs with truncation support. All output formats should indicate when truncation occurs to maintain transparency per spec requirements. Test count increased to 633 tests (252 integration tests, +1 from baseline).

### Pack (`specs/pack.md`)
- [x] `merge-links` only applies when targets were loaded (skips existing notes).
   - `src/commands/load/mod.rs:77-97,317-345`
   - `tests/pack_tests.rs:564-735`
   - **Root Cause**: The `merge-links` strategy was merging links to ALL notes in `loaded_ids` (which included existing notes when using merge-links), not just to newly loaded notes.
   - **Fix**: Modified `load_links` function to accept separate `source_ids` and `target_ids` parameters. For `merge-links` strategy, pass `loaded_ids` as source (so existing notes can get new links) but `new_ids` as target (so only links TO newly loaded notes are added). For `skip` strategy, pass `new_ids` for both. For `overwrite`, pass `loaded_ids` for both.
   - **Learnings**: The distinction between "notes involved in link processing" (source) and "notes that links can point to" (target) is critical for the `merge-links` strategy. This allows enrichment of existing notes with new connections while avoiding links to notes that already existed in the target store.
- [x] Pack note `path` is ignored on load.
   - `src/commands/load/mod.rs:248-256,261,283,319-322`
   - `tests/pack_tests.rs:751-870`
   - **Root Cause**: The load command was not using the `path` field from `PackNote` when creating notes, always generating a new path based on note type and title slug.
   - **Fix**: Added logic to check if `pack_note.path` exists and use it when creating the note. Absolute paths are used as-is, relative paths are resolved against the store root. Falls back to slug-based path generation only if pack doesn't provide a path.
   - **Learnings**: Pack files already included the `path` field (line 36 in dump/serialize.rs), but it wasn't being consumed during load. This preserves the original file structure when round-tripping through pack/unpack. Added comprehensive test `test_pack_preserves_note_paths` to verify path preservation across dump/load cycle.

### Provenance (`specs/provenance.md`)
- [x] LLM-generated notes do not default `verified=false`.
  - `src/commands/create.rs:49-67`
  - `src/commands/capture.rs:72-87`
  - `tests/cli/provenance.rs:127-180`
  - `tests/cli/capture.rs:346-351`
  - **Learnings**: When `--generated-by` is provided, default `verified` to `false` unless explicitly overridden with `--verified` flag. This allows agents to track LLM-generated content with proper provenance while giving users the ability to pre-verify content if needed. Added comprehensive tests for both the default behavior and explicit override.
- [ ] Web capture defaults for `source`/`author` are not implemented.
  - `src/commands/capture.rs:72-87`

### Structured Logging (`specs/structured-logging.md`)
- [ ] `--log-level` accepts arbitrary strings (no validation).
  - `src/cli/mod.rs:52-54`
- [ ] Default log policy is `warn` (not silent-by-default).
  - `src/lib/logging.rs:10-13`

### LLM User Validation (`specs/llm-user-validation.md`)
- [ ] Scenario schema omits `docs`, `tags`, `run` limits, `cost`, and `cache` fields.
  - `crates/llm-tool-test/src/scenario.rs:4-86`
- [ ] Stage-1 evaluation lacks `qipu doctor` and transcript error checks.
  - `crates/llm-tool-test/src/evaluation.rs:75-148`
- [ ] `--max-usd` is parsed but unused (no budget enforcement).
  - `crates/llm-tool-test/src/commands.rs:23-36`
- [ ] Artifact set is incomplete (missing `run.json`, `report.md`, snapshots).
  - `crates/llm-tool-test/src/run.rs:89-114`
- [ ] Tool adapter trait diverges from spec (`execute_task`/`ToolStatus` missing).
  - `crates/llm-tool-test/src/adapter/mod.rs:9-22`
- [ ] Missing `report` subcommand and `clean --older-than` support.
  - `crates/llm-tool-test/src/cli.rs:11-115`

### Workspaces (`specs/workspaces.md`)
- [ ] `rename` merge strategy is not supported.
  - `src/commands/workspace/merge.rs:20-24`
- [ ] `--from-*` workspace creation is shallow; graph-slice copy is missing.
  - `src/commands/workspace/new.rs:70-89`
- [ ] Post-merge integrity validation is missing.
  - `src/commands/workspace/merge.rs:10-149`
- [ ] `workspace list` omits last-updated metadata.
  - `src/commands/workspace/list.rs:70-85`
- [ ] `parent_id` is never populated.
  - `src/commands/workspace/new.rs:55-61`

### Similarity Ranking (`specs/similarity-ranking.md`)
- [ ] Related-note expansion only runs with explicit `--related`.
  - `src/commands/context/mod.rs:162-165`
- [ ] Stemming is always enabled; there is no opt-out.
  - `src/lib/index/builder.rs:49-62`
- [ ] Search ranking boosts are hardcoded (do not match spec weights).
  - `src/lib/db/search.rs:81-102`

### CLI Tool Tests (`specs/cli-tool.md`)
- [ ] Add tests for visible-store discovery and `--format=json` parse errors.
  - `src/lib/store/paths.rs:28-41`
  - `src/main.rs:82-93`
  - `tests/cli/misc.rs:53-114`
- [ ] Expand golden determinism coverage beyond help/list/prime.
  - `tests/golden_tests.rs:94-217`

### CLI Interface Tests (`specs/cli-interface.md`)
- [ ] Add tests for `create` alias `new`, `--open`, and `--id`.
  - `src/cli/commands.rs:31-36`
  - `src/cli/args.rs:18-20`
  - `src/cli/args.rs:42-44`
- [ ] Add tests for `list --tag`, `list --since`, and `list --format records`.
  - `src/cli/commands.rs:39-49`
  - `tests/cli/list.rs:10-109`
- [ ] Add tests for `search --exclude-mocs`, `--min-value`, and `--sort`.
  - `src/cli/commands.rs:132-142`
  - `tests/cli/search.rs:10-616`
- [ ] Add tests for `compact apply --from-stdin` and `--notes-file`.
  - `src/cli/compact.rs:18-24`
  - `tests/cli/compact/commands.rs:9-659`

### Value Model Tests (`specs/value-model.md`)
- [ ] Add tests for `qipu value set/show` output + validation.
  - `src/cli/value.rs:5-18`
  - `src/commands/dispatch/mod.rs:319-365`
- [ ] Add tests for `search --sort value`.
  - `src/commands/search.rs:138-145`
- [ ] Add tests for `list --min-value` and `context --min-value`.
  - `src/commands/list.rs:59-63`
  - `src/commands/context/mod.rs:233-237`
- [ ] Add tests for `--ignore-value` traversal ordering.
  - `src/commands/link/tree.rs:61-78`
  - `src/commands/link/path.rs:71-95`
- [ ] Add CLI coverage for `--unweighted`/`--weighted` aliases (spec names vs current flags).
  - `src/cli/link.rs:105-111`

### Export Tests (`specs/export.md`)
- [ ] Add tests for `--tag`/`--query` selection ordering.
  - `src/commands/export/plan.rs:9-62`
  - `tests/cli/export.rs:7-341`
- [ ] Add tests for `--mode bibliography`.
  - `src/commands/export/emit/bibliography.rs:4-37`
  - `tests/cli/export.rs:7-341`
- [ ] Add tests for `--link-mode markdown` and `--link-mode preserve`.
  - `src/commands/export/mod.rs:47-69`
  - `tests/cli/export.rs:7-341`

### Graph Traversal Tests (`specs/graph-traversal.md`)
- [ ] Add tests for semantic inversion in `link tree`/`link path`.
  - `src/commands/link/tree.rs:57-63`
  - `src/commands/link/path.rs:71-76`
  - `tests/cli/link/add_remove.rs:55-73`
- [ ] Add tests for `max_nodes`, `max_edges`, and `max_fanout` truncation.
  - `src/cli/link.rs:89-99`
  - `tests/cli/link/tree.rs:366-435`

### Records Output Tests (`specs/records-output.md`)
- [ ] Add `max-chars` truncation tests for link tree/path records output.
  - `src/commands/link/tree.rs:276-396`
  - `src/commands/link/records.rs:205-315`
  - `tests/cli/link/tree.rs:149-178`
  - `tests/cli/link/path.rs:203-245`

### Workspaces Tests (`specs/workspaces.md`)
- [ ] Add tests for `workspace merge --dry-run`, `skip`, and `overwrite` strategies.
  - `src/commands/workspace/merge.rs:20-149`
  - `tests/workspace_merge_test.rs:55-205`

### Semantic Graph Tests (`specs/semantic-graph.md`)
- [ ] Add tests for additional standard types and custom inverses.
  - `src/lib/note/types.rs:92-169`
  - `src/lib/config.rs:65-79`
  - `tests/cli/link/add_remove.rs:31-239`

### Similarity Ranking Tests (`specs/similarity-ranking.md`)
- [ ] Add tests for default similarity thresholds and field weighting.
  - `src/lib/similarity/mod.rs:27-135`
  - `tests/cli/doctor.rs:305-389`
- [ ] Add end-to-end tests for stop-word filtering.
  - `src/lib/text/mod.rs:8-54`
  - `src/lib/similarity/mod.rs:27-135`

### Pack Tests (`specs/pack.md`)
- [ ] Add tests for `--tag`, `--moc`, `--query`, and "no selectors" dump.
  - `src/commands/dump/mod.rs:117-158`
  - `tests/cli/dump.rs:5-789`
- [ ] Add tests for attachments round-trip and `--no-attachments`.
  - `src/commands/dump/mod.rs:344-392`
  - `src/commands/load/mod.rs:362-395`
  - `tests/cli/pack.rs:8-211`
- [ ] Add tests for pack version/store version compatibility errors.
  - `src/commands/load/mod.rs:58-72`
  - `tests/cli/pack.rs:8-211`

### Provenance Tests (`specs/provenance.md`)
- [ ] Add tests for default `verified=false` behavior on LLM-origin notes.
  - `src/commands/create.rs:49-67`
  - `tests/cli/provenance.rs:6-123`

---

## P3: Unimplemented Optional / Future

### Custom Metadata (`specs/custom-metadata.md`)
- [ ] Implement custom frontmatter, DB storage, and CLI/filter/output support.
  - Add `custom: HashMap<String, serde_yaml::Value>` field to NoteFrontmatter (`src/lib/note/frontmatter.rs:7-54`)
  - Add `custom_json TEXT DEFAULT '{}'` column to notes table (`src/lib/db/schema.rs:19-60`)
  - Implement CLI commands with type detection:
    - `qipu custom set <id> <key> <value>` - Parse value using `serde_yaml::from_str()` for automatic type detection
    - `qipu custom get <id> <key>` - Display single field value
    - `qipu custom show <id>` - Display all custom fields for a note
    - `qipu custom unset <id> <key>` - Remove a custom field
    - Mark commands with `#[command(hide = true)]` per spec
  - Add filtering support:
    - `qipu list --custom key=value` - Filter by custom field value
    - `qipu context --custom key=value` - Context selection with custom filters
  - Add context output support:
    - `qipu context --custom` flag to include custom fields in output (opt-in)
    - Format output for markdown, JSON, and records formats
  - Add doctor validation (`src/commands/doctor/mod.rs:170-323`):
    - Validate custom block is a valid YAML mapping
    - Warn on very large custom blocks (>10KB)

### Distribution (`specs/distribution.md`)
- [ ] Add release automation and install scripts (GitHub releases + installers).
  - `.github/workflows/ci.yml:1-120`
- [ ] Add checksum generation + installer verification.
  - `tests/golden/version.txt:1`

### Export (`specs/export.md`)
- [ ] Add optional BibTeX/CSL JSON outputs.
  - `src/commands/export/emit/bibliography.rs:4-37`
- [ ] Add transitive export traversal (depth-limited).
  - `src/commands/export/plan.rs:112-209`
- [ ] Add pandoc/PDF integration (future).
  - `src/commands/export/mod.rs:13-261`

### Similarity Ranking (`specs/similarity-ranking.md`)
- [ ] Add clustering/"see also" features for MOC generation.
  - `src/lib/similarity/mod.rs:27-135`

### Semantic Graph (`specs/semantic-graph.md`)
- [ ] Support per-link-type hop costs (currently hardcoded to 1.0).
  - `src/lib/graph/types.rs:48-53`

### LLM User Validation (`specs/llm-user-validation.md`)
- [ ] Add PTY fallback and richer event logging (tool_call/tool_result).
  - `crates/llm-tool-test/src/session.rs:22-108`
  - `crates/llm-tool-test/src/run.rs:91-99`
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
