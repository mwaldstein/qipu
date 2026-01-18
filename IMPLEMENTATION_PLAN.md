# Qipu Implementation Plan

## Status (Last Audited: 2026-01-18)
- Test baseline: `cargo test` passes (2026-01-18).
- Trust hierarchy: this plan is derived from code + tests; specs/docs are treated as hypotheses.
- All P1 correctness bugs completed (2026-01-18).

## P1: SQLite Migration & Ripgrep Removal (PRIORITY)

Per `specs/operational-database.md`, SQLite replaces both JSON cache and ripgrep. Ripgrep must be removed.

### Phase 1: Add SQLite Foundation
- [x] Add `rusqlite` dependency with bundled SQLite to `Cargo.toml`
- [x] Create database schema in `src/lib/db/schema.rs` (notes, notes_fts, tags, edges, unresolved, index_meta)
- [x] Implement `Database` struct with open/create/rebuild in `src/lib/db/mod.rs`
- [ ] Implement FTS5 with porter tokenizer and BM25 ranking (title 2.0x, tags 1.5x, body 1.0x)
- [x] Add database path at `.qipu/qipu.db`

### Phase 2: Inline Updates
- [ ] Update `Store` to hold `Database` instance
- [ ] Modify `create_note` to write file + insert into DB atomically
- [ ] Modify `update_note` (edit) to update file + re-index in DB
- [ ] Modify `delete_note` to remove file + remove from DB
- [ ] Modify `link add/remove` to update file + update edges table

### Phase 3: Migrate Queries to SQLite
- [ ] Migrate `search` command to use FTS5 (replace `search_with_ripgrep` and `search_embedded`)
- [ ] Migrate `list` command filters to use SQLite metadata queries
- [ ] Migrate backlinks lookup to use edges table
- [ ] Migrate graph traversal (`link tree/path`) to use recursive CTE
- [ ] Migrate `doctor` checks to use SQLite validation queries
- [ ] Migrate `context` note selection to use SQLite

### Phase 4: Remove Legacy Components
- [ ] Delete ripgrep integration:
  - [ ] Remove `RipgrepMatch`, `RipgrepBeginData`, `RipgrepEndData`, `RipgrepMatchData`, `RipgrepText` structs from `src/lib/index/search.rs`
  - [ ] Remove `is_ripgrep_available()` function
  - [ ] Remove `search_with_ripgrep()` function
  - [ ] Remove ripgrep fallback logic from `search()` function
- [ ] Delete JSON cache code:
  - [ ] Remove `.cache/` directory creation and all JSON index file code
  - [ ] Remove `Index` struct JSON serialization
  - [ ] Delete `src/lib/index/builder.rs` JSON cache building
- [ ] Update `index --rebuild` to only rebuild SQLite
- [ ] Add migration: detect `.cache/`, rebuild DB, delete `.cache/`
- [ ] Update tests that reference ripgrep (e.g., `test_search_title_only_match_included_with_ripgrep_results`)

### Phase 5: Startup Validation
- [ ] On startup: check if `qipu.db` exists, trigger full rebuild if missing
- [ ] Quick consistency check: compare note count in DB vs filesystem, sample mtimes
- [ ] Incremental repair when external changes detected
- [ ] Handle schema version mismatch with auto-rebuild

Refs: spec `specs/operational-database.md`, current ripgrep code `src/lib/index/search.rs:13-48,80-301`

## P1-LEGACY: Correctness Bugs (COMPLETED)

### `specs/export.md`
- [x] `--with-attachments` copies files but does not rewrite note markdown links to point at the copied `./attachments/` location
  - Fixed: added `rewrite_attachment_links()` to transform `../attachments/` to `./attachments/` in output content
- [x] `--mode bibliography --format json` does not produce a bibliography-shaped JSON output
  - Fixed: JSON export now emits `sources` array with extracted bibliography entries instead of `notes` array

### `specs/compaction.md`
- [x] JSON outputs that include `compacted_ids` do not indicate truncation when `--compaction-max-nodes` is hit
  - Fixed: JSON now includes `compacted_ids_truncated: true` when truncation occurs (`src/commands/list.rs:97-103`)
- [x] `--expand-compaction` drops truncation reporting entirely (expanded set can be silently truncated)
  - Fixed: JSON now includes `compacted_ids_truncated: true` and `compacted_notes_truncated: true` when truncation occurs (`src/commands/context/output.rs:128-186`)
- [x] `compact guide` claims `report/suggest` are "coming soon" even though both exist
  - Fixed: removed "(coming soon)" from guide output

### `specs/pack.md`
- [x] `load --strategy merge-links` does not match spec semantics (content preservation + links union)
  - Fixed: Now returns loaded IDs from `load_notes` and uses that set in `load_links` to ensure pack links are added to merged notes
  - Refs: empty links `src/commands/load/mod.rs:198`, note body set from pack `src/commands/load/mod.rs:211-213`, merge branch `src/commands/load/mod.rs:249-276`
 - [x] Dump `--typed-only` / `--inline-only` filtering is inverted
   - Fixed: Corrected filter conditions to skip non-inline links when `--inline-only` is set, and skip inline links when `--typed-only` is set
   - Refs: `src/commands/dump/mod.rs:236-241`
 - [x] Dump traversal expansion ignores type/source filters (`--type`, `--typed-only`, `--inline-only`)
    - Fixed: Added filter checks in `perform_simple_traversal` to respect type_include, typed_only, and inline_only options when deciding which edges to follow during traversal
    - Refs: traversal filtering `src/commands/dump/mod.rs:289-301`
 - [x] `load --strategy skip` can still mutate existing notes via `load_links()` (uses pack IDs, not "actually loaded" set)
   - Fixed: With skip strategy, don't process any links at all. This ensures that skipped notes are never mutated - even loaded notes cannot add links to skipped notes, preventing unintended modifications.
   - Refs: skip strategy skip link processing `src/commands/load/mod.rs:88-99`
 - [x] Pack format depends on `--format` (spec claims `--format` should not alter pack contents)
   - Fixed: Removed format-based encoding selection; dump now always uses records format per spec
   - Refs: encoding removed `src/commands/dump/mod.rs:52-62`

### `specs/workspaces.md`
- [x] `workspace merge --dry-run` does not produce a conflict report and prints a success-like message
  - Fixed: dry_run now produces detailed report showing notes to add, conflicts, and actions based on strategy; success message only shown when not dry_run
  - Refs: dry_run report `src/commands/workspace/merge.rs:39-80`
  - [x] `merge-links` strategy also unions tags (spec describes link-only merge)
     - Fixed: Removed tag unioning logic, now only unions links as specified
     - Refs: tag union `src/commands/workspace/merge.rs:52-57`, link union `src/commands/workspace/merge.rs:58-63`
  - [x] `workspace new --empty` flag is accepted but ignored
     - Fixed: Changed `_empty` to `empty` and added check to skip all copy operations when `empty` is true
     - Refs: empty flag check `src/commands/workspace/new.rs:51-74`
 - [x] `workspace merge --strategy overwrite` can leave duplicate note files for the same note ID (old file not removed)
      - Fixed: Now removes existing note file before overwriting when using overwrite strategy
      - Refs: file removal before overwrite `src/commands/workspace/merge.rs:54-58`
 - [x] Unknown merge strategies silently fall back to `skip` (typos and unimplemented `rename` are not rejected)
   - Fixed: Added early validation that rejects unknown strategies with UsageError (exit code 2) and lists valid options
   - Refs: validation `src/commands/workspace/merge.rs:16-21`, match updated `src/commands/workspace/merge.rs:53-58`
- [x] Workspace metadata schema differs from spec (`[workspace]` table vs top-level `WorkspaceMetadata`)
   - Fixed: Added `WorkspaceMetadataFile` wrapper to serialize metadata under `[workspace]` table per spec
   - Refs: wrapper struct `src/lib/store/workspace.rs:14-15`, load/save updated `src/lib/store/workspace.rs:18-36`

### `specs/structured-logging.md`
- [x] Logging is initialized, but most operational output still uses `eprintln!` + legacy `--verbose` gates (minimal/empty tracing output)
  - Fixed: Replaced legacy `VERBOSE` atomic and `verbose_enabled()` with proper `tracing::debug!`, `tracing::warn!` instrumentation
  - Updated all timing statements in dispatch.rs, search method selection in index/search.rs, regex warnings in index/links.rs, load operation info in commands/load/mod.rs, and parse warnings in store/query.rs
  - Fixed test expectations: changed from stderr to stdout for verbose output checks
  - Refs: tracing init `src/lib/logging.rs:14-51`, timing `tracing::debug!` `src/commands/dispatch.rs:22`, search method `tracing::debug!` `src/lib/index/search.rs:425,430`, regex warnings `tracing::warn!` `src/lib/index/links.rs:37,64`

### `specs/llm-user-validation.md`
- [x] `llm-tool-test` CLI default tool value is inconsistent with runtime support
  - Fixed: Changed default from "qipu" to "opencode" to match runtime support
  - Refs: CLI default `crates/llm-tool-test/src/cli.rs:23`; runtime match `crates/llm-tool-test/src/main.rs:59-63`
- [x] Rubric YAML fixtures don't match the deserialization shape expected by the judge
  - Fixed: Converted YAML fixtures from mapping structure to array structure with `id` field
  - Refs: `crates/llm-tool-test/fixtures/qipu/rubrics/capture_v1.yaml`, `crates/llm-tool-test/fixtures/qipu/rubrics/link_v1.yaml`
  - [x] Regression detection message/condition appears reversed
    - Fixed: Corrected condition from `!baseline.gates_passed && current.gates_passed` to `baseline.gates_passed && !current.gates_passed` to properly detect gate regressions (gates that previously passed now failing)
    - Refs: fixed condition `crates/llm-tool-test/src/results.rs:228`

## P2: Missing Test Coverage

### `specs/cli-tool.md`
- [x] Add tests for `--root` affecting discovery start dir and relative `--store` resolution
  - Added: `test_root_flag_affects_discovery_start_dir` and `test_relative_store_resolved_against_root` in `tests/cli/misc.rs:144-175`
  - Tests verify: discovery starts from `--root`, relative `--store` resolved against `--root`

### `specs/graph-traversal.md`
- [x] Add tests for `link tree/path` include/exclude type filters and `--typed-only/--inline-only`
  - Added: 6 tests for type/exclude filters and typed-only/inline-only
  - Refs: tree tests `tests/cli/link/tree.rs:5-230`, path tests `tests/cli/link/path.rs:203-490`
- [x] Add tests for `direction=in` and `direction=both` on `link tree` and `link path`
  - Added: 4 tests for direction=in and direction=both
  - Refs: tree direction tests `tests/cli/link/tree.rs:681-766`, path direction tests `tests/cli/link/path.rs:491-577`

### `specs/indexing-search.md`
- [x] Add tests asserting ranking rules (title boost > body; tag boost behavior)
  - Added: `test_search_title_match_ranks_above_body_match` and `test_search_exact_tag_match_ranks_above_body` already existed
  - Added: `test_search_title_only_match_included_with_ripgrep_results` to ensure title-only matches are found when ripgrep returns other results
  - Refs: boosts `src/lib/index/search.rs:176-178`
- [x] Add test that would fail if title-only matches are missed when ripgrep returns results
  - Added: `test_search_title_only_match_included_with_ripgrep_results` in `tests/cli/search.rs`
  - Verifies that title-only matches are included even when ripgrep finds body matches (so fallback to embedded search is not triggered)
  - Refs: ripgrep path `src/lib/index/search.rs:53-110`

### `specs/similarity-ranking.md`
- [x] Add CLI/integration test for `qipu doctor --duplicates` with threshold behavior
   - Added: `test_doctor_duplicates_threshold` in `tests/cli/doctor.rs:246-306`
   - Tests verify: `--duplicates` flag works, `--threshold` affects output, different thresholds (0.5, 0.99, default 0.85) detect duplicates appropriately
   - Refs: CLI flags `src/cli/commands.rs:173-186`, doctor path `src/commands/doctor/checks.rs:261-280`

### `specs/provenance.md`
- [ ] Add CLI test for `--prompt-hash` via `create` or `capture` (not just pack roundtrip)
  - Flags exist: `src/cli/args.rs:22-40`; test coverage currently relies on pack tests.

### `specs/export.md`
- [x] Add test that verifies MOC-driven bundle export respects MOC ordering (currently likely fails)
  - Fixed: Added `test_export_bundle_preserves_moc_order` in `tests/cli/export.rs:204-248`
  - Verified: MOC ordering is correctly preserved in bundle mode (sorting is skipped when `moc_id` is set)
  - Refs: global sort `src/commands/export/mod.rs:101-105`, test `tests/cli/export.rs:204-248`
- [ ] Add test validating anchor rewriting produces a target anchor that exists in output
  - Refs: rewrite `src/commands/export/emit/links.rs:56-96`
- [ ] Add test validating `--with-attachments` produces rewritten attachment links that resolve in the export folder
  - Refs: copy logic `src/commands/export/mod.rs:161-242`

### `specs/compaction.md`
- [ ] Add tests for `compact apply`, `compact show`, `compact status` (CLI-level)
  - Implementations exist but are not directly exercised: `src/commands/compact/apply.rs`, `src/commands/compact/show.rs`, `src/commands/compact/status.rs`

### `specs/structured-logging.md`
- [ ] Add tests verifying `--log-level` / `--log-json` / `QIPU_LOG` behavior (currently only help text is covered)
  - Refs: init `src/lib/logging.rs:31-40`, flags `src/cli/mod.rs:50-57`, golden `tests/golden/help.txt:41-44`

### `specs/llm-context.md`
- [ ] Add test with large bodies in human/JSON to catch `--max-chars` / `--max-tokens` budget violations (summary-estimate vs full-body output)
  - Refs: estimate `src/commands/context/budget.rs:97-103`, output `src/commands/context/output.rs:208-213`
- [ ] Add tests for `context --transitive` (nested MOC traversal)
  - Refs: traversal `src/commands/context/select.rs:22-28`
- [ ] Add test for records safety banner line (`W ...`) under `--format records --safety-banner`
  - Refs: records banner `src/commands/context/output.rs:436-443`

### `specs/pack.md`
- [ ] Add tests for dump traversal filters (`--type`, `--typed-only`, `--inline-only`) and verify they affect reachability, not just included edges
  - Refs: traversal ignores options `src/commands/dump/mod.rs:81-112`



## P3: Unimplemented Optional / Future

### `specs/similarity-ranking.md`
- [ ] Optional stemming (Porter) is not implemented
  - Refs: no stemming code in `src/`
- [ ] "Related notes" similarity expansion (threshold > 0.3) is described but not implemented as a CLI/context feature
  - Similarity API exists but is unused by `context`.
  - Refs: similarity API `src/lib/similarity/mod.rs:49-75`, context selection `src/commands/context/mod.rs:72-109`

### `specs/llm-context.md`
- [ ] Backlinks-in-context is described as open/future; not implemented
  - Refs: context options have no backlinks flag `src/commands/context/types.rs:4-15`

### `specs/semantic-graph.md`
- [ ] Weighted traversal / per-edge hop costs are not implemented (if still desired)
  - Refs: traversal is unweighted BFS `src/lib/graph/traversal.rs:87-90`

## P4: Spec Ambiguity / Spec Drift (Needs Clarification Before Implementation)

### `specs/knowledge-model.md`
- [ ] Decide whether note "type" should remain a closed enum or allow arbitrary values (spec marks as open question)
  - Refs: strict enum `src/lib/note/types.rs:6-19`, parsing `src/lib/note/types.rs:27-42`

### `specs/semantic-graph.md`
- [ ] Align custom link-type config schema (spec uses `[graph.types.*]`; impl uses `[links.inverses]` + `[links.descriptions]`)
  - Refs: config `src/lib/config.rs:40-69`, spec mismatch noted in semantic-graph audit

### `specs/records-output.md`
- [ ] Reconcile record prefix set and terminators (spec suggests `H/N/S/E/B`; impl also emits `W/D/C/M` and `B-END`)
  - Refs: context records `src/commands/context/output.rs:436-443` (`W`), `src/commands/context/output.rs:344-354` (`D source`), `src/commands/context/output.rs:361-362` (`B-END`); prime records `src/commands/prime.rs:201-219` (`C/M`)

### `specs/graph-traversal.md` + `specs/semantic-graph.md`
- [ ] Clarify whether semantic inversion is part of traversal semantics (virtual edges) or only a presentation-layer feature
  - Refs: global flag `src/cli/mod.rs:82-85`, inversion `src/lib/index/types.rs:43-54`

### `specs/export.md`
- [ ] Clarify expected behavior for anchor rewriting (explicit anchors vs relying on Markdown renderer heading IDs)
  - Refs: rewrite targets `#note-<id>` `src/commands/export/emit/links.rs:16-18`

## Closed Design Decisions (Specs Updated)

### `specs/storage-format.md`
- [x] MOCs use separate `mocs/` directory (not inside `notes/` with type flag)
  - Provides clear filesystem separation, simpler glob patterns
  - Refs: `src/lib/store/mod.rs:140-141,211-213`
- [x] Note paths are flat (no date partitioning like `notes/2026/01/...`)
  - Keeps paths stable, simplifies resolution; SQLite handles large stores
  - Refs: `src/lib/store/mod.rs:207-208`

### `specs/graph-traversal.md`
- [x] Default `--max-hops` is 3 (not 2); no default `--max-nodes`
  - Surfaces 2-hop neighborhoods; users reduce with `--max-chars` for LLM context
  - Refs: `src/lib/graph/types.rs:64`

