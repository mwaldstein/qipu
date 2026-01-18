# Qipu Implementation Plan

## Status (Last Audited: 2026-01-18)
- Test baseline: `cargo test` passes (2026-01-18).
- Trust hierarchy: this plan is derived from code + tests; specs/docs are treated as hypotheses.

## P1: Correctness Bugs

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
- [x] `compact guide` claims `report/suggest` are “coming soon” even though both exist
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
- [x] Add tests for `--root` affecting discovery start dir and relative `--store` resolution
  - Added: `test_root_flag_affects_discovery_start_dir` and `test_relative_store_resolved_against_root` in `tests/cli/misc.rs:144-175`
  - Tests verify: discovery starts from `--root`, relative `--store` resolved against `--root`

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

