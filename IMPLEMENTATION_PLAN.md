# Qipu Implementation Plan

This document tracks completed implementation work. For exploratory future work and open questions from specs, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status

- **Test baseline**: 791 tests pass (excludes performance tests)
- **Revision 3 complete**: 2026-01-23
- **Last audited**: 2026-01-23
- Related: [`specs/README.md`](specs/README.md) - Specification status tracking

---

## Spec Audit (2026-01-23)

### Completed (Verified)

- [x] `custom-metadata.md`: Custom frontmatter + DB persistence + `qipu custom` + `context --custom-filter` + opt-in output (`src/lib/note/frontmatter.rs:52-57`, `src/lib/db/schema.rs:30-49`, `src/commands/custom.rs:19-280`, `tests/cli/custom.rs:4-653`, `tests/cli/context/basic.rs:903-1414`)

### P1: Correctness Bugs (Spec Mismatch)

- [x] `cli-tool.md`: Store discovery stops at project root markers, not filesystem root (`src/lib/store/paths.rs:29-38`, `src/lib/store/paths.rs:98-120`)
  - Fixed logic: removed `passed_project_root` flag and changed to stop immediately when project marker detected
  - Added 3 unit tests for project root marker stopping behavior (`.git`, `Cargo.toml`, and missing store case)

- [x] `cli-tool.md`: `--format json --help/--version` likely treated as error envelope instead of exit 0 (`src/main.rs:32-41`)
  - Fixed: Added checks for `DisplayHelp` and `DisplayVersion` error kinds to let clap handle them normally (exit 0)
  - Added 4 integration tests: `test_format_json_help_exits_zero`, `test_format_json_equals_help_exits_zero`, `test_format_json_version_exits_zero`, `test_format_json_equals_version_exits_zero`
- [x] `cli-tool.md` / `structured-logging.md`: Logs appear on stdout (breaks machine output expectations) (`tests/cli/logging.rs:19-25`, `src/lib/logging.rs:33-40`)
  - Fixed: Added `.with_writer(std::io::stderr)` to write logs to stderr instead of stdout
  - Updated all logging tests to check stderr instead of stdout
  - Updated `test_verbose_flag` and `test_workspace_delete_with_unmerged_changes` to check stderr

- [x] `cli-interface.md`: Search JSON omits spec-minimum note fields (`path/created/updated`) (`src/commands/search/format/json.rs:20-29`)
  - Added `created` and `updated` fields to `SearchResult` struct
  - Updated SQL query to select `created` and `updated` columns from notes table
  - Updated JSON output to include `path`, `created`, and `updated` fields
  - Updated `test_search_json_format` to verify `path` field is present
- [x] `cli-interface.md`: Inbox JSON omits `path` (`src/commands/dispatch/notes.rs:160-177`)
  - Added `path` field to inbox JSON output
  - Updated JSON formatting to include path from note's `Option<PathBuf>` field
  - Added test `test_inbox_json_format_includes_path` to verify spec compliance
- [x] `cli-interface.md` / `operational-database.md`: `qipu edit` and `qipu update` commands (atomic update + re-index) (`src/commands/edit.rs`, `src/commands/update.rs`, `src/commands/dispatch/notes.rs:67-130`, `src/commands/dispatch/mod.rs:271-288`)
- [x] `cli-interface.md`: `context` missing-selection returns exit 1 (not usage exit 2) (`src/commands/context/mod.rs:443-446`, `src/lib/error.rs:95-101`)
  - Fixed: Changed `QipuError::Other` to `QipuError::UsageError` for missing selection criteria (exit code 2)
  - Updated test to expect exit code 2 instead of exit code 1
- [x] `knowledge-model.md`: DB reads coerce unknown `type` to `fleeting` instead of rejecting (`src/lib/db/notes/read.rs:248-249`, `src/lib/db/search.rs:206-207`)
  - Fixed: Removed `.unwrap_or(NoteType::Fleeting)` from 5 locations and replaced with proper error propagation
  - Added `convert_qipu_error_to_sqlite` helper functions in `read.rs` and `search.rs` to convert `QipuError` to `rusqlite::Error`
  - Added test `test_unknown_note_type_rejected` to verify rejection of invalid note types

 - [x] `indexing-search.md`: DB edge insertion passes empty `path_to_id`, so `(...).md` relative links can be missed in backlinks/traversal (`src/lib/db/edges.rs:8-35`)
   - Fixed: Added `build_path_to_id_map()` function to query all note paths and IDs from database
   - Updated both `insert_edges()` and `insert_edges_internal()` to use the populated `path_to_id` HashMap
   - This allows relative markdown links like `[text](../other/note.md)` to be properly resolved to note IDs
- [x] `value-model.md`: `link path` defaults to `--ignore-value` (unweighted) despite spec “weighted by default” (`src/cli/link.rs:154-155`)
  - Fixed: Changed `default_value = "true"` to `default_value = "false"` for both Tree and Path subcommands
  - Updated help text to indicate "weighted by default"
   - All 7 ignore_value tests verify correct behavior
 - [x] `graph-traversal.md`: `link tree` human view expands from `result.links` (not `spanning_tree`) and can expand nodes multiple times (`src/commands/link/tree.rs:171-293`)
  - Fixed: Changed from using `result.links` to `result.spanning_tree` for building children map
  - Updated function signature to use `&SpanningTreeEntry` instead of `&TreeLink`
  - Updated test `test_link_tree_cycle_shows_seen` to verify nodes appear exactly once (no duplicates)
  - Added test `test_link_tree_spanning_tree_not_all_links` to verify spanning tree behavior
  - Updated golden test `link_tree.txt` to reflect correct behavior (nodes appear once)
  - Note: Tree indentation issue (first-level children appear at same level as root) is a pre-existing bug in original code, not introduced by this fix

 - [x] `records-output.md`: Link records headers use store-root path (not CWD-relative) (`src/commands/link/records.rs:176-186`)
   - Added `path_relative_to_cwd` function to `src/lib/records.rs` (moved from context module)
   - Updated `build_header_base` to use CWD-relative store path
   - Updated `output_path_records` to use CWD-relative store path
   - Added unit tests for `path_relative_to_cwd`
   - Verified existing link records format tests pass
 - [x] `compaction.md`: Link JSON outputs omit compaction annotations/truncation indicators (`src/commands/link/json.rs:7-86`, `src/commands/link/tree.rs:120-153`)
   - Implementation already complete: JSON includes `compacts`, `compaction_pct`, `compacted_ids`, and `compacted_ids_truncated`
   - Fixed test assertions to match actual JSON format (spaces after colons, numeric types)
- [x] `pack.md`: Pack dump/load is lossy (value not serialized; custom dropped; merge-links semantics restricted) (`src/commands/dump/serialize.rs:107-148`, `src/commands/load/mod.rs:95-104`, `src/commands/load/mod.rs:245-246`)
   - Added `value` field to records format serialization
   - Added `custom` field (base64-encoded JSON) to records format serialization
   - Added `custom` field to `PackNote` model (both dump and load)
   - Added `serde_yaml_to_json` and `serde_json_to_yaml` conversion helpers
   - Updated deserializer to parse `custom` field from pack records
   - Added `test_pack_preserves_value_and_custom_metadata` integration test
- [x] `distribution.md`: Release workflow is disabled + installers hardcode repo slug inconsistent with Cargo metadata (`.github/workflows/release.yml:3-13`, `scripts/install.sh:13-16`, `Cargo.toml:4-12`)
   - Fixed: Updated Cargo.toml repository URL from `anomalyco/qipu` to `mwaldstein/qipu` to match install.sh
   - Note: Release workflow intentionally disabled pending GitHub Actions enablement (external dependency)

### P2: Missing Test Coverage

- [x] `semantic-graph.md`: Add direct CLI tests asserting semantic inversion + type filtering + `source=virtual` behavior (`src/lib/graph/algos/bfs.rs:146-164`, `src/commands/link/list.rs:119-136`)
  - Added 5 tests to `tests/cli/link/list.rs`:
    - `test_link_list_semantic_inversion_default`: Verifies inbound link shows as outbound virtual with inverted type
    - `test_link_list_semantic_inversion_disabled`: Verifies --no-semantic-inversion shows raw inbound link
    - `test_link_list_semantic_inversion_type_filter`: Verifies filtering by inverted type works with semantic inversion
    - `test_link_list_semantic_inversion_type_filter_disabled`: Verifies filtering by inverted type fails without semantic inversion
    - `test_link_list_semantic_inversion_type_filter_original`: Verifies filtering by original type behavior with/without semantic inversion
- [x] `workspaces.md`: Add tests for `workspace merge --strategy rename` (`src/commands/workspace/merge.rs:219-289`)
  - Added 5 tests to `tests/workspace_merge_test.rs`:
    - `test_workspace_merge_rename_strategy_basic`: Verifies conflicting notes are renamed and added
    - `test_workspace_merge_rename_strategy_multiple_conflicts_same_id`: Verifies suffix increments correctly for multiple conflicts
    - `test_workspace_merge_rename_strategy_dry_run`: Verifies dry-run shows correct output format
    - `test_workspace_merge_rename_strategy_no_conflicts`: Verifies rename strategy works with no conflicts
    - `test_workspace_merge_rename_strategy_preserves_unique_notes`: Verifies unique notes are added correctly
  - Note: Link rewriting tests revealed merge bug (broken links after merge) - tracked separately
 - [x] `custom-metadata.md`: Add tests for `qipu list --custom ...` and doctor custom checks (`tests/cli/list.rs:632-890`, `tests/cli/doctor.rs:713-809`)
- [x] `llm-user-validation.md`: Enforce per-run budget env var and per-scenario `cost.max_usd` preflight (`crates/llm-tool-test/src/commands.rs:51-71`)
   - Implemented: Read `LLM_TOOL_TEST_BUDGET_USD` env var as session budget
   - CLI `--max-usd` flag overrides env var
   - Env var parsed as float, validated non-negative
   - Added 3 unit tests for env var parsing (valid, invalid, negative)
   - All 158 tests pass

### P3: Unimplemented But Ready

- [x] `workspaces.md`: Git integration for temp workspaces (auto-add to `.gitignore`) (`src/commands/workspace/new.rs`)
  - Added gitignore entry for temp workspaces in `.qipu/workspaces/<name>/` format
  - Added 6 integration tests covering all scenarios (create, preserve, no-duplicate, non-temp)
  - All 448 CLI tests pass
 - [x] `storage-format.md`: Wiki-link canonicalization (opt-in `--rewrite-wiki-links` flag) (`src/lib/index/links.rs:153-199`)
   - Implementation already complete: `rewrite_wiki_links()` function converts `[[id]]` and `[[id|label]]` to markdown links
   - Added `--rewrite-wiki-links` flag to `qipu index` command (not `--canonicalize-links` as originally planned)
   - Added test `test_index_rewrites_wiki_links` to verify behavior
   - Note: Flag name differs from plan (`--rewrite-wiki-links` vs `--canonicalize-links`), but matches commit `a889f25` implementation
 - [x] `cli-interface.md`: `qipu capture` default type (default to `fleeting`) (`src/commands/capture.rs`, `specs/cli-interface.md:73-80`)
    - Implementation already present: line 63 defaults to `NoteType::Fleeting`
    - Test already present: `test_capture_default_type_fleeting` verifies behavior
    - Updated spec to document the `--type` flag with explicit default
    - Removed open question from spec
  - [x] `graph-traversal.md`: Context walk command (`qipu context --walk`) (`src/commands/context/walk.rs`)
    - Added `--walk <id>` flag to `qipu context` command
    - Added walk-specific options: `--walk-direction`, `--walk-max-hops`, `--walk-type`, `--walk-exclude-type`, `--walk-typed-only`, `--walk-inline-only`, `--walk-max-nodes`, `--walk-max-edges`, `--walk-max-fanout`, `--walk-min-value`, `--walk-ignore-value`
    - Implemented `walk_for_context()` function to perform graph traversal using existing link tree traversal logic
    - Updated context command to use walk results as note selection criteria
    - Added 5 integration tests: `test_context_walk_basic`, `test_context_walk_max_hops`, `test_context_walk_direction`, `test_context_walk_with_type_filter`, `test_context_walk_json_format`
    - All 47 context tests pass
  - [ ] `operational-database.md`: Database size/stats reporting (`qipu store stats`) (`src/commands/store/stats.rs`)

## Revision 2 (2026-01-23)

### P1: Core Features

| Feature | Summary |
|---------|---------|
| **Machine-readable output** | `qipu value` and `qipu custom` support `--format json` and `--format records` |
| **show --format json** | Includes `value`; opt-in `--custom` flag includes custom metadata |
| **context selectors** | `--min-value` and `--custom-filter` work as standalone selectors |
| **custom-filter parsing** | Supports `key=value`, `key`, `!key`, `key>n`, `key>=n`, `key<n`, `key<=n`; multiple filters AND together |
| **Negative values** | `qipu custom set <id> <key> -100` works without `--` |
| **Clean JSON stdout** | Logging to stderr, ANSI disabled with `.with_ansi(false)` |
| **Budget truncation** | Notes truncated with `…[truncated]` marker instead of dropped entirely |
| **Search breadcrumbs** | `via=<id>` annotation when search hits compacted notes |

### P2: Code Quality & Refactoring

| Target | Result |
|--------|--------|
| `bfs_find_path` (400→59 lines) | Extracted BFS, Dijkstra, neighbor collection, path reconstruction helpers |
| `pricing.rs` module | Extracted model pricing from results.rs (-72 lines) |
| Gate evaluation | Created `GateEvaluator` trait; 10 validator functions (<17 lines each) |
| Output formatting | Created `src/commands/format/mod.rs` with shared helpers |
| Doctor command | `DoctorCheck` trait; mod.rs reduced to 116 lines (orchestration only) |
| Similarity engine | 82-line main module; field weights in `src/lib/index/weights.rs` |
| Test file splits | `export.rs` → 8 modules, `tree.rs` → 9 modules, `pack_tests.rs` → 4 modules |

### P2: LLM Tool Test Enhancements

| Feature | Summary |
|---------|---------|
| **Pre-existing test fixes** | Fixed 4 failing tests (MockAdapter CLI syntax, shlex parsing) |
| **Safety guard** | Requires `LLM_TOOL_TEST_ENABLED=1` to run |
| **Per-scenario timeout** | `run.timeout_secs` in YAML overrides CLI `--timeout` |
| **Store snapshot** | Copies `.qipu/` and `export.json` to transcript dir |

### P3: Optional Enhancements

| Feature | Summary |
|---------|---------|
| **qipu onboard** | Outputs minimal AGENTS.md snippet pointing to `qipu prime` |
| **Tag aliases** | `tag_aliases` config with bidirectional resolution; doctor warns on orphaned aliases |

---

## Revision 1 (2026-01-22)

### Correctness (P1)
- **CLI**: Fixed create output, JSON error envelopes
- **Database**: Removed filesystem fallbacks, auto-rebuild on schema mismatch
- **Graph**: Fixed Dijkstra min-heap ordering, default BFS for `link path`, CSV-style type filters
- **Pack**: Fixed `skip` strategy link handling, `merge-links` edge insertion
- **Value Model**: Added 0-100 validation, fixed default value handling
- **Workspaces**: Implemented `rename` strategy, graph-slice copy, post-merge validation

### Test Coverage (P1)
- Added 100+ new tests covering all CLI commands/flags per spec
- Golden determinism tests (context, search, inbox, show, link tree/path)

### Refactoring (P2)
- Split `bfs.rs` into `algos/` module
- Modularized similarity engine (tfidf, duplicates, tags)

### Features (P3)
- **Custom Metadata**: Full implementation with type detection, filtering, doctor checks
- **Distribution**: Release automation, cross-platform installers
- **Export**: BibTeX/CSL JSON, transitive traversal, pandoc/PDF integration
- **Semantic Graph**: Per-link-type hop costs
- **LLM Tool Test**: Report command, PTY fallback, structured event logging
- **Tags**: `qipu tags list` command with frequency counts

---

## Deferred Items

### Spec Clarification Needed

| Spec | Item |
|------|------|
| `indexing-search.md` | Backlink index storage - confirm stored vs derived |
| `workspaces.md` | `--temp` gitignore behavior |
| `telemetry.md` | DRAFT spec, explicitly prohibits implementation |

### Technical Debt

| Item | Priority |
|------|----------|
| File size monitoring | High - add CI check for files >500 lines |
| Function complexity monitoring | High - flag functions >100 lines |
| Refactor large files and functions | High |
| - `src/lib/graph/bfs.rs` (842 lines): Split `dijkstra_search` (116 lines) and `bfs_search` (88 lines) functions | |
| - `src/commands/doctor/content.rs` (829 lines): Consider modularizing check functions | |
| - `src/commands/setup.rs` (780 lines): Mostly constants and tests; consider extracting constants to separate file | |
| - `src/commands/doctor/database.rs` (722 lines): Check for large functions | |
| - `src/lib/similarity/mod.rs` (635 lines): Check for large functions | |
| - `src/commands/context/mod.rs` (627 lines): Check for large functions | |
| - `src/lib/db/notes/read.rs` (609 lines): Check for large functions | |
| - `src/commands/show.rs` (570 lines): Check for large functions | |
| Model pricing externalization | Medium - move to config file |
| Output format consolidation | Medium - shared OutputFormatter trait |

---

## Notes

- **Schema version**: 6 (custom metadata column)
- **Store format version**: 1
