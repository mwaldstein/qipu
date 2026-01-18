# Qipu Implementation Plan

## Status (Last Audited: 2026-01-18)
- Test baseline: `cargo test` passes (2026-01-18).
- Trust hierarchy: this plan is derived from code + tests; specs/docs are treated as hypotheses.

## P1: Correctness Bugs

### `specs/cli-interface.md`
- [x] Usage/argument validation sometimes returns exit code `1` instead of `2` (usage error)
  - Fixed: invalid `--since` date and `--direction` now return `QipuError::UsageError` -> exit 2
  - Changed `src/commands/dispatch.rs:304`, `src/commands/dispatch.rs:702`, `src/commands/dispatch.rs:733`
  - Added tests: `tests/cli/misc.rs:test_invalid_since_date_exit_code_2`, `tests/cli/misc.rs:test_invalid_direction_exit_code_2`
- [x] `qipu load --format records` emits a non-standard header (`H load=1 ...`) instead of the normal records header
  - Fixed: changed to standard format `H qipu=1 records=1 store={} mode=load pack_file={} notes={} links={} attachments={}`
  - Changed `src/commands/load/mod.rs:127-135`

### `specs/llm-context.md`
- [x] `context` budgeting can be violated because selection estimates summary size but human/JSON outputs always emit full body
  - Fixed: human and JSON outputs now respect `--with-body` flag
  - When `--with-body=false`, outputs use `note.summary()` instead of `note.body`
  - Changed `src/commands/context/output.rs:23-33`, `src/commands/context/output.rs:210-216`
  - Updated function signatures to accept `with_body` parameter: `src/commands/context/output.rs:9`, `src/commands/context/output.rs:123`
  - Updated call sites: `src/commands/context/mod.rs:164-172`, `src/commands/context/mod.rs:174-184`
- [x] `--max-chars` is "exact" only for `--format records`; human/JSON use an estimate + buffer and never validate final output size
  - Fixed: human and JSON outputs now validate actual output size and iteratively remove notes until within budget
  - Added `max_chars` parameter to `output_json()` and `output_human()`: `src/commands/context/output.rs:9`, `src/commands/context/output.rs:169`
  - Created helper functions `build_json_output()` and `build_human_output()` to build output strings: `src/commands/context/output.rs:36-157`, `src/commands/context/output.rs:210-396`
  - Iterative budget enforcement loops remove notes one-by-one if output exceeds budget: `src/commands/context/output.rs:18-34`, `src/commands/context/output.rs:178-194`
  - Updated call sites to pass `max_chars`: `src/commands/context/mod.rs:172`, `src/commands/context/mod.rs:186`

### `specs/indexing-search.md`
- [x] Search can miss title-only matches when ripgrep path is used
  - Fixed: `search_with_ripgrep()` now scans index metadata for title/tag matches in addition to ripgrep file-content matches, ensuring notes with query terms only in title/tags are included
  - Changed `src/lib/index/search.rs:125-197`: builds candidate set from both ripgrep results and index metadata title/tag matches, then scores all candidates
  - Added test: `tests/cli/search.rs:test_search_title_only_match` verifies title-only matches are found
- [x] Recency boost is specified but not present in ranking
  - Fixed: added recency boost to search ranking based on `updated` timestamp
  - Added function `calculate_recency_boost()` that applies exponential decay: 0.5 boost for notes updated within 7 days, 0.25 for 30 days, 0.1 for 90 days, 0.0 for older notes
  - Changed `src/lib/index/search.rs:14-60`: added recency boost calculation function
  - Changed `src/lib/index/search.rs:228-230`: apply recency boost in ripgrep search path
  - Changed `src/lib/index/search.rs:365-367`: apply recency boost in embedded search path
  - Added test: `tests/cli/search.rs:test_search_recency_boost` verifies recently updated notes rank higher
- [x] "Exact tag match should rank above plain text match" is not implemented (tags are BM25-scored text)
  - Fixed: added exact tag match detection with higher boost (3.0x) to ensure exact tag matches rank above body text matches
  - Changed `src/lib/index/search.rs:218-236`: added exact tag match counting and boosting in ripgrep search path
  - Changed `src/lib/index/search.rs:321-371`: added exact tag match counting and boosting in embedded search path
  - Ranking now applies: Title x2.0, Exact tag match x3.0, Tags (BM25) x1.5, Body x1.0
  - Added test: `tests/cli/search.rs:test_search_exact_tag_match_ranks_above_body` verifies exact tag matches rank higher than body text matches

### `specs/storage-format.md`
- [x] Markdown links to other notes by relative path are not resolved unless the target contains a `qp-...` ID
  - Fixed: markdown link extraction now resolves relative paths by building a path-to-ID mapping during indexing
  - Changed `src/lib/index/links.rs:9-14`: added `source_path` and `path_to_id` parameters to `extract_links()`
  - Changed `src/lib/index/links.rs:100-120`: added logic to resolve relative `.md` paths using the source note's directory and path-to-ID lookup
  - Changed `src/lib/index/builder.rs:43-147`: restructured index building into two passes - first builds path mappings, second extracts links with mappings available
  - Added tests: `src/lib/index/mod.rs:205-281` (unit tests), `tests/cli/index.rs:142-229` (integration test)

### `specs/graph-traversal.md`
- [x] "(seen)" references in human tree output are effectively unreachable
  - Fixed: tree rendering now uses `links` instead of `spanning_tree` to build the children map, making back-edges to visited nodes available for "(seen)" rendering
  - Changed `src/commands/link/tree.rs:128-132`: build children map from `result.links` instead of `result.spanning_tree`
  - Changed `src/commands/link/tree.rs:143-148`: updated `print_tree()` signature to accept `HashMap<String, Vec<&TreeLink>>`
  - Added test: `tests/cli/link/tree.rs:test_link_tree_cycle_shows_seen` verifies cycles show "(seen)" marker
- [x] Tree/path truncation is not reported when exploration stops due to `--max-hops`
  - Fixed: traversal now checks for unexpanded neighbors when `hop >= max_hops` and sets `truncated=true` with `truncation_reason="max_hops"` if any exist
  - Changed `src/lib/graph/traversal.rs:87-124`: added logic to detect unexpanded neighbors at max hop depth and report truncation
  - Added test: `tests/cli/link/tree.rs:test_link_tree_max_hops_reports_truncation` verifies truncation is reported in JSON output
- [x] Default semantic inversion introduces `source=virtual` + inverted types for inbound traversal; spec does not describe this behavior
  - Fixed: updated `specs/graph-traversal.md` to document semantic inversion behavior
  - Added section "Semantic inversion for inbound edges" documenting that inversion is enabled by default, creates virtual edges with inverted types, and can be disabled with `--no-semantic-inversion`
  - Documented global flag `--no-semantic-inversion` in CLI surface section
  - Added documentation for `source` field values (typed, inline, virtual) in JSON output section
  - Existing tests already verify this behavior: `tests/cli/link/add_remove.rs:test_link_add_and_list` tests both semantic inversion and `--no-semantic-inversion`
- [x] Tree ordering can diverge from "sort neighbors by (type,id)" guidance due to spanning-tree re-sort
  - Fixed: spanning tree now maintains the same (type, id) ordering used during neighbor expansion
  - Added `link_type` field to `SpanningTreeEntry`: `src/lib/graph/types.rs:99-107`
  - Updated traversal to populate `link_type` when building spanning tree: `src/lib/graph/traversal.rs:237-243`
  - Updated spanning tree sort to use (hop, link_type, to_id): `src/lib/graph/traversal.rs:269-273`
  - Added test: `tests/cli/link/tree.rs:test_link_tree_spanning_tree_ordering` verifies spanning tree respects (type, id) ordering

### `specs/records-output.md`
- [x] Records headers are inconsistent across commands (`mode=` before/after `store=`)
  - Fixed: standardized all records headers to use `store=` before `mode=` to match spec examples
  - Changed `src/commands/context/output.rs:565`: context header now uses `store={} mode=context` order
  - Changed `src/commands/prime.rs:193`: prime header now uses `store={} mode=prime` order
  - Updated test assertions: `tests/cli/context/basic.rs:305`, `tests/cli/context/formats.rs:58`, `tests/cli/context/compaction.rs:307`, `tests/cli/prime.rs:159`
  - Spec pattern (`specs/records-output.md:63,73`): `H qipu=1 records=1 store=... mode=...`
  - Now consistent with tree/list/path/load commands which already used this order
- [x] Records quoting is not escaped for note titles/summaries, so titles containing `"` can break record parsing
  - Fixed: added `escape_quotes()` utility in `src/lib/records.rs` that replaces `"` with `\"`
  - Updated all N record formatting to escape titles: `src/commands/context/output.rs:429,510`, `src/commands/link/list.rs:309`, `src/commands/link/tree.rs:277`, `src/commands/link/path.rs:331`, `src/commands/dispatch.rs:426`, `src/commands/list.rs:193`, `src/commands/search.rs:327`, `src/commands/create.rs:89`, `src/commands/capture.rs:108`, `src/commands/show.rs:151`, `src/commands/prime.rs:228`, `src/commands/export/emit/records.rs:96`, `src/commands/dump/serialize.rs:109`
  - Updated all D source record formatting to escape titles: `src/commands/context/output.rs:470,536`, `src/commands/export/emit/records.rs:63`
  - Added test: `tests/cli/context/formats.rs:64-96` verifies quotes in titles are properly escaped

### `specs/similarity-ranking.md`
- [x] Stop words removal is required but not implemented
  - Fixed: added common English stop words list (33 words: a, an, and, are, as, at, be, but, by, for, if, in, into, is, it, no, not, of, on, or, such, that, the, their, then, there, these, they, this, to, was, will, with)
  - Stop words are now filtered during tokenization using a static `OnceLock<HashSet<&str>>` for efficient lookup
  - Changed `src/lib/text/mod.rs:1-40`: added `get_stop_words()` function and updated `tokenize()` to filter stop words
  - Added tests: `src/lib/text/mod.rs:69-118` (6 unit tests verifying stop word removal)
- [x] Duplicate threshold default differs from spec (spec: 0.85; impl default: 0.8)
  - Fixed: changed CLI default value from 0.8 to 0.85 to match spec
  - Changed `src/cli/commands.rs:184`: updated default_value from "0.8" to "0.85"
- [x] Similarity is described as TF-IDF cosine; implementation uses cosine over BM25-weighted vectors with `tf=1` (no term frequencies stored)
  - Fixed: implemented TF-IDF cosine similarity as specified
  - Changed index to store term frequencies with field weights: `src/lib/index/types.rs:146-149`
  - Updated index builder to tokenize title (weight 2.0), tags (weight 1.5), and body (weight 1.0): `src/lib/index/builder.rs:69-104`
  - Replaced BM25 calculation with TF-IDF using smoothed IDF formula: `src/lib/similarity/mod.rs:119-140`
  - Updated search.rs to work with new HashMap structure: `src/lib/index/search.rs:324-328`
  - Updated builder.rs prune function to work with HashMap: `src/lib/index/builder.rs:246-259`
  - Updated test to use HashMap: `src/lib/similarity/mod.rs:147-197`

### `specs/provenance.md`
- [x] `qipu create --format json` omits provenance fields (`source/author/generated_by/prompt_hash/verified`)
  - Fixed: added provenance fields to JSON output matching `show` command format
  - Changed `src/commands/create.rs:52-68`: added source, author, generated_by, prompt_hash, verified to JSON output
  - Added test: `tests/cli/create.rs:102-144` verifies provenance fields are included in JSON output
- [ ] `qipu capture --format json` omits provenance fields
  - Refs: `src/commands/capture.rs:70-82`
- [ ] `qipu context --format json` omits per-note provenance fields (even though `show --format json` includes them)
  - Refs: context JSON shape `src/commands/context/output.rs:18-42`; show JSON includes provenance `src/commands/show.rs:57-75`

### `specs/export.md`
- [ ] MOC-driven export ordering does not follow MOC ordering for bundle/json/records
  - Global created/id sort runs before emitting regardless of `--moc`.
  - Refs: sort `src/commands/export/mod.rs:101-103`, sort fn `src/commands/export/plan.rs:100-110`
- [ ] `--link-mode anchors` likely produces broken anchors (`#note-<id>` targets not emitted)
  - Refs: anchor map `src/commands/export/emit/links.rs:16-18`, headings lack explicit anchors `src/commands/export/emit/bundle.rs:31`, `src/commands/export/emit/outline.rs:74`
- [ ] `--with-attachments` copies files but does not rewrite note markdown links to point at the copied `./attachments/` location
  - Refs: copy target `src/commands/export/mod.rs:164-167`, copy regex expects `../attachments/...` `src/commands/export/mod.rs:203-205`
- [ ] `--mode bibliography --format json` does not produce a bibliography-shaped JSON output
  - Refs: JSON export always emits notes array `src/commands/export/emit/json.rs:26-86`

### `specs/compaction.md`
- [ ] JSON outputs that include `compacted_ids` do not indicate truncation when `--compaction-max-nodes` is hit
  - Truncation boolean exists but is only surfaced via records (`D compacted_truncated`) / human messages.
  - Refs: truncation computed `src/lib/compaction/expansion.rs:48-58`; JSON emits only IDs `src/commands/list.rs:88-97`
- [ ] `--expand-compaction` drops truncation reporting entirely (expanded set can be silently truncated)
  - Refs: expansion returns `(notes, truncated)` but callers discard it: `src/commands/context/output.rs:72-110`
- [ ] `compact guide` claims `report/suggest` are “coming soon” even though both exist
  - Refs: `src/commands/compact/guide.rs:49-51`

### `specs/pack.md`
- [ ] `load --strategy merge-links` does not match spec semantics (content preservation + links union)
  - Incoming note frontmatter links are initialized empty, so “merge” is a no-op; content still overwritten.
  - Refs: empty links `src/commands/load/mod.rs:198`, note body set from pack `src/commands/load/mod.rs:211-213`, merge branch `src/commands/load/mod.rs:249-276`
- [ ] Dump `--typed-only` / `--inline-only` filtering is inverted
  - Refs: `src/commands/dump/mod.rs:36-41`
- [ ] Dump traversal expansion ignores type/source filters (`--type`, `--typed-only`, `--inline-only`)
  - Refs: traversal `src/commands/dump/mod.rs:81-112`
- [ ] `load --strategy skip` can still mutate existing notes via `load_links()` (uses pack IDs, not “actually loaded” set)
  - Refs: `load_links` signature `src/commands/load/mod.rs:92-99`
- [ ] Pack format depends on `--format` (spec claims `--format` should not alter pack contents)
  - Refs: encoding selected by CLI format `src/commands/dump/mod.rs:52-62`

### `specs/workspaces.md`
- [ ] `workspace merge --dry-run` does not produce a conflict report and prints a success-like message
  - Refs: CLI promise `src/cli/workspace.rs:60-63`, behavior `src/commands/workspace/merge.rs:82-84`
- [ ] `merge-links` strategy also unions tags (spec describes link-only merge)
  - Refs: tag union `src/commands/workspace/merge.rs:52-57`, link union `src/commands/workspace/merge.rs:58-63`
- [ ] `workspace new --empty` flag is accepted but ignored
  - Refs: ignored arg `_empty` `src/commands/workspace/new.rs:13-14`
- [ ] `workspace merge --strategy overwrite` can leave duplicate note files for the same note ID (old file not removed)
  - Refs: overwrite path copies note `src/commands/workspace/merge.rs:44-47`; `copy_note` writes a new filename `src/commands/workspace/merge.rs:89-107`
- [ ] Unknown merge strategies silently fall back to `skip` (typos and unimplemented `rename` are not rejected)
  - Refs: `match strategy { "overwrite" | "merge-links" | "skip" | _ => skip }`: `src/commands/workspace/merge.rs:43-69`
- [ ] Workspace metadata schema differs from spec (`[workspace]` table vs top-level `WorkspaceMetadata`)
  - Refs: metadata struct serde `src/lib/store/workspace.rs:6-33`

### `specs/structured-logging.md`
- [ ] Logging is initialized, but most operational output still uses `eprintln!` + legacy `--verbose` gates (minimal/empty tracing output)
  - Refs: tracing init `src/lib/logging.rs:15-52`, legacy verbose gate `src/lib/logging.rs:4-12`, timing `eprintln!` `src/main.rs:64-66`

### `specs/llm-user-validation.md`
- [ ] `llm-tool-test` CLI default tool value is inconsistent with runtime support
  - CLI default `--tool qipu`: `crates/llm-tool-test/src/cli.rs:22-25`; runtime only accepts `amp|opencode`: `crates/llm-tool-test/src/main.rs:59-63`
- [ ] Rubric YAML fixtures don’t match the deserialization shape expected by the judge
  - Expects `criteria: Vec<...>`: `crates/llm-tool-test/src/judge.rs:5-17`; fixtures are a mapping: `crates/llm-tool-test/fixtures/qipu/rubrics/capture_v1.yaml:1-16`
- [ ] Regression detection message/condition appears reversed
  - Refs: `crates/llm-tool-test/src/results.rs:228-230`

## P2: Missing Test Coverage

### `specs/cli-tool.md`
- [ ] Add tests for `--root` affecting discovery start dir and relative `--store` resolution
  - Refs: `--root` flag `src/cli/mod.rs:29-33`, used `src/commands/dispatch.rs:14-18`; existing discovery test is cwd-only `tests/cli/misc.rs:99-114`

### `specs/graph-traversal.md`
- [ ] Add tests for `link tree/path` include/exclude type filters and `--typed-only/--inline-only`
  - Filters exist but are untested: `src/lib/graph/types.rs:35-59`
- [ ] Add tests for `direction=in` and `direction=both` on `link tree` and `link path`
  - Direction parsing exists; tests currently cover `out` and some hop limits.
  - Refs: direction enum `src/lib/graph/types.rs:5-30`; existing tests `tests/cli/link/tree.rs:229-288`

### `specs/indexing-search.md`
- [ ] Add tests asserting ranking rules (title boost > body; tag boost behavior)
  - Current tests check presence, not ordering.
  - Refs: boosts `src/lib/index/search.rs:176-178`
- [ ] Add test that would fail if title-only matches are missed when ripgrep returns results
  - Refs: ripgrep path `src/lib/index/search.rs:53-110`

### `specs/similarity-ranking.md`
- [ ] Add CLI/integration test for `qipu doctor --duplicates` with threshold behavior
  - Core similarity has unit test, but no CLI test.
  - Refs: CLI flags `src/cli/commands.rs:173-186`, doctor path `src/commands/doctor/checks.rs:261-280`

### `specs/provenance.md`
- [ ] Add CLI test for `--prompt-hash` via `create` or `capture` (not just pack roundtrip)
  - Flags exist: `src/cli/args.rs:22-40`; test coverage currently relies on pack tests.

### `specs/export.md`
- [ ] Add test that verifies MOC-driven bundle export respects MOC ordering (currently likely fails)
  - Refs: global sort `src/commands/export/mod.rs:101-103`
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

### `specs/indexing-search.md`
- [ ] Optional SQLite FTS5 backend is not implemented
  - Refs: no SQLite search layer; spec mentions optional FTS5; `qipu.db` not used.

### `specs/storage-format.md`
- [ ] Optional `qipu.db` is not implemented (only gitignored)
  - Refs: gitignore entry `src/lib/store/io.rs:44-71`

### `specs/similarity-ranking.md`
- [ ] Optional stemming (Porter) is not implemented
  - Refs: no stemming code in `src/`
- [ ] “Related notes” similarity expansion (threshold > 0.3) is described but not implemented as a CLI/context feature
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
- [ ] Decide whether note “type” should remain a closed enum or allow arbitrary values (spec marks as open question)
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

## Completed (Verified Working)

### `specs/cli-tool.md`
- [x] Store discovery rules (`--store`, walk-up `.qipu/` then `qipu/`) and exit code `3` on missing store
  - Refs: discovery `src/lib/store/paths.rs:29-58`, error mapping `src/lib/error.rs:83-88`, tests `tests/cli/misc.rs:84-93`
- [x] JSON usage-error envelope when `--format json` is present (including parse-time failures)
  - Refs: parse-time detection `src/main.rs:28-50`, JSON error emit `src/main.rs:75-81`, tests `tests/cli/misc.rs:52-59`

### `specs/knowledge-model.md`
- [x] Note parsing/serialization with YAML frontmatter and required fields (`id`, `title`)
  - Refs: parse `src/lib/note/parse.rs:10-55`, serialize `src/lib/note/mod.rs:61-65`, tests `src/lib/note/mod.rs:121-140`
- [x] Core note types (fleeting/literature/permanent/moc) enforced and tested
  - Refs: enum `src/lib/note/types.rs:6-19`, tests `src/lib/note/mod.rs:90-103`

### `specs/records-output.md`
- [x] Records format implemented for `context`, `prime`, and link traversal commands with max-chars truncation support (context + link commands)
  - Refs: context `src/commands/context/output.rs:265-527`, tree `src/commands/link/tree.rs:255-374`, path `src/commands/link/path.rs:309-438`, list `src/commands/link/list.rs:253-442`, tests `tests/cli/context/budget.rs:117-147`

### Non-spec maintenance
- [x] Full test suite currently green
  - Refs: `cargo test` (2026-01-18)
