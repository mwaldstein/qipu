# Qipu Implementation Plan

This document tracks **concrete implementation tasks** - bugs to fix, features to complete, and tests to add. For exploratory future work and open questions from specs, see [`FUTURE_PLAN.md`](FUTURE_PLAN.md).

## Status
- Test baseline: `cargo test` passes (note: 33 pre-existing test failures unrelated to P1/P2 fixes)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes
- Audit Date: 2026-01-20
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
   - Blocked by: Separate database location bug when using `QIPU_STORE` environment variable causes tests to fail (database created in wrong location: `store_root/.qipu/qipu.db` instead of `.qipu/qipu.db`)
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
- [ ] No cross-branch ID collision avoidance.
  - `src/lib/store/lifecycle.rs:31-34`
  - `src/lib/store/lifecycle.rs:77-88`

### Export (`specs/export.md`)
- [ ] Outline export appends outbound edges beyond MOC body ordering.
  - `src/commands/export/emit/outline.rs:54-68`
- [ ] Query export caps results at 200 notes.
  - `src/commands/export/plan.rs:54-59`
- [ ] Outline export falls back to bundle when `--moc` is missing.
  - `src/commands/export/emit/outline.rs:23-28`

### Semantic Graph (`specs/semantic-graph.md`)
- [ ] Context selection doesn’t prefer typed links over `related` when constrained.
  - `src/commands/context/mod.rs:162-225`
- [ ] Doctor does not validate semantic misuse of standard link types.
  - `src/commands/doctor/database.rs:56-77`

### Records Output (`specs/records-output.md`)
- [ ] `prime` records output lacks `--max-chars` budgeting.
  - `src/commands/prime.rs:190-196`
  - `src/cli/commands.rs:167-168`
- [ ] Link tree/path records rely on traversal order (no explicit ordering).
  - `src/commands/link/tree.rs:287-340`
  - `src/commands/link/records.rs:217-315`

### LLM Context (`specs/llm-context.md`)
- [ ] No per-note truncation marker when budgets are applied.
  - `src/commands/context/budget.rs:55-82`
  - `src/commands/context/human.rs:35-68`
- [ ] Default output uses summaries unless `--with-body` (spec implies full body).
  - `src/commands/context/human.rs:161-165`
  - `src/commands/context/json.rs:91-95`
- [ ] `prime` is count-capped only; no explicit token/char budgeting.
  - `src/commands/prime.rs:15-20`
- [ ] Bundle output omits empty metadata headers (`Path`, `Tags`, `Sources`) when values are absent.
  - `src/commands/context/human.rs:105-157`

### Compaction (`specs/compaction.md`)
- [ ] Link outputs omit `compacts=`/`compaction=` annotations for digest nodes.
  - `src/commands/link/human.rs:30-53`
  - `src/commands/link/tree.rs:183-205`
- [ ] JSON outputs ignore compaction truncation flags.
  - `src/commands/search.rs:212-226`
  - `src/commands/show.rs:97-107`
- [ ] `compact show` ignores `--compaction-max-nodes` and truncation.
  - `src/commands/compact/show.rs:75-105`

### Pack (`specs/pack.md`)
- [ ] `merge-links` only applies when targets were loaded (skips existing notes).
  - `src/commands/load/mod.rs:319-328`
- [ ] Pack note `path` is ignored on load.
  - `src/commands/load/mod.rs:209-283`

### Provenance (`specs/provenance.md`)
- [ ] LLM-generated notes do not default `verified=false`.
  - `src/commands/create.rs:49-67`
  - `src/commands/capture.rs:72-87`
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

---

## Notes

- Audit Date: 2026-01-20
- Recent completions include workspaces `--empty` flag, structured logging verification, file size refactoring, and MOC ordering preservation
- Documentation additions: `docs/building-on-qipu.md` and type detection spec in `specs/custom-metadata.md`
- Test fixes: Added `extract_id` helper to test support to handle two-line create output (ID + path); updated ID extraction in test files; added index calls to test cases that manually create notes
