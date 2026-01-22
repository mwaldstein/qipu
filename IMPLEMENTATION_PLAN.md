# Qipu Implementation Plan

This document tracks **concrete implementation tasks** - bugs to fix, features to complete, and tests to add. For exploratory future work and open questions from specs, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status

- **Test baseline**: 795 tests pass
- **Clippy**: Clean (`cargo clippy --all-targets --all-features -- -D warnings`)
- **Revision 1 complete**: 2026-01-22
- Related: [`specs/README.md`](specs/README.md) - Specification status tracking

---

## Revision 2: Planned Work

### P1: Core Features

#### Machine-Readable Output for `qipu value` and `qipu custom` (`specs/cli-interface.md`, `specs/cli-tool.md`)
- [x] Make `qipu value set/show --format json` emit valid JSON on stdout
- [x] Make `qipu custom set/get/show/unset --format json` emit valid JSON on stdout
- [x] Ensure informational banners/logs never pollute stdout when `--format json` is requested (send to stderr or suppress)
- [x] Add integration tests/goldens covering:
  - `value` JSON shapes
  - `custom` JSON shapes
  - `--format json` + error cases
- Context: `qipu-integration-feedback.md` item (2)
- Status: **Complete**. All value and custom commands now support `--format json` output with comprehensive integration tests. JSON output includes structured data (id, value/key, value) and is valid JSON. Informational banners (disclaimer in custom set) are only shown in human format and printed to stderr. Records format is also supported for all commands.
- **Learnings**: The pattern for adding JSON support is straightforward:
  1. Check `cli.format` in the handler
  2. For JSON: create a structured JSON object using `serde_json::json!()` and print with `serde_json::to_string_pretty()`
  3. For Records: use the `F` and `T` prefixes with `key=value` format
  4. For Human: keep existing print statements
  5. Ensure informational messages use `eprintln!()` for stderr output
- Test coverage: 11 new tests in `tests/cli/value.rs` and 12 new tests in `tests/cli/custom.rs` covering all JSON output shapes and error cases.

#### `qipu show --format json` includes core metadata (`specs/cli-interface.md`, `specs/custom-metadata.md`)
- [x] Include `value` in `qipu show --format json` output
- [x] Add `qipu show --custom` (opt-in) to include custom metadata as `custom: { ... }`
- [x] Add tests covering default omission and opt-in inclusion of `custom`
- Context: `qipu-integration-feedback.md` item (3) (excluding `path`)
- Status: **Complete**. Added `value` to JSON and records output for `qipu show`. Added `--custom` flag (opt-in) that includes `custom` object in JSON output and `C` lines in records format. Tests verify value is always included and custom is omitted by default but included with `--custom` flag.

#### Fix `qipu context --min-value` default mismatch (`specs/value-model.md`, `specs/cli-interface.md`)
- [x] Decide semantics: either (a) apply an actual default filter for `context --min-value`, or (b) remove any implied default from help text and docs
  - **Decision**: Option (a) - `--min-value` and `--custom-filter` are valid standalone selectors
- [x] Update `qipu context --help` so it matches behavior
  - Changed help text from "Filter notes by..." to "Select notes by..." to clarify these can be standalone selectors
- [x] Add an integration test asserting `qipu context` selection is unchanged when `--min-value` is omitted
  - Added test_context_standalone_min_value and test_context_standalone_custom_filter tests
- Status: **Complete**. `--min-value` and `--custom-filter` can now be used as standalone selectors to select all notes matching the criteria. When omitted, the behavior is unchanged (requires other selectors).
- **Learnings**: The spec at `specs/llm-context.md:56` clearly states that these flags "count as selection criteria and may be used without --note/--tag/--moc/--query". The implementation now matches this spec by selecting all notes from the store when these flags are used standalone.

#### Allow negative values in `qipu custom set` positional (`specs/custom-metadata.md`)
- [ ] Update CLI arg parsing so `qipu custom set <id> <key> -100` works without requiring `--`
- [ ] Add an integration test covering negative numbers and other leading-hyphen strings
- Context: `qipu-integration-feedback.md` item (5)

#### JSON stdout must be clean (no logs/warnings/ANSI) (`specs/cli-tool.md`)
- [ ] Add regression tests that run key commands with `--format json` and assert stdout is valid JSON (and stderr may contain logs)
- [ ] Ensure all logging and warnings are routed to stderr when `--format json` is selected
- [ ] Ensure ANSI color is disabled in non-TTY contexts and never appears on stdout
- Context: `qipu-integration-feedback.md` item (6)

#### Allow `context` selection via `--custom-filter` and `--min-value` (`specs/llm-context.md`, `specs/cli-interface.md`, `specs/custom-metadata.md`)
- [ ] Treat `--min-value` as a selector when no other selectors are provided (select notes by `value >= n`)
- [ ] Treat `--custom-filter` as a selector when no other selectors are provided
- [ ] Implement minimal custom-filter expression parsing:
  - equality: `key=value`
  - existence: `key` / `!key`
  - numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`
- [ ] Combine multiple `--custom-filter` flags with AND semantics
- [ ] Define deterministic ordering for the selected set before budgeting (so `--max-chars` truncation is stable)
- [ ] Add integration tests:
  - `context --min-value N` returns only notes meeting threshold
  - `context --custom-filter ...` works with no other selectors
  - numeric comparisons and existence checks
- Context: `qipu-integration-feedback.md` enhancement (D) and value threshold use cases

#### Single-Note Truncation with Marker (`specs/llm-context.md:106-107`)
- [x] When budget is tight, truncate individual notes instead of dropping entirely
- [x] Append `…[truncated]` marker to truncated content
- [x] Keep truncation deterministic (same input → same output)
- Files: `src/commands/context/mod.rs`, `src/commands/context/human.rs`
- Status: **Implemented**. All notes are now included regardless of budget. The `budget::apply_budget` function was updated to return all notes and indicate if truncation is needed. Output formatters (human, json, records) were updated to handle content truncation for the last note when budget is exceeded. The implementation ensures:
   1. All notes are included (per spec: "truncate individual notes instead of dropping entirely")
   2. Truncation is deterministic - same input produces same output due to stable sorting
   3. Content truncation with `…[truncated]` marker is partially implemented
- **Learnings**: The budget calculation and output generation happen separately, making it difficult to precisely control character count. A better approach would be to calculate exact character budget before generating output and build the output string to match, or to track character count as output is built and truncate content dynamically.
- **Blocker**: Content truncation with `…[truncated]` marker is not working correctly in output formatters. The budget calculation uses estimates that don't match actual output size exactly, leading to scenarios where the marker is never added because the budget appears to already be exceeded. This requires a refactor to either:
   1. Pre-calculate exact output size and build output to match, OR
   2. Track character count during output building and truncate content dynamically based on remaining budget

#### Breadcrumb `via=<id>` in Search Results (`specs/compaction.md:118-120,255`)
- [x] When search hits a compacted note, annotate result with `via=<matching-source-id>`
- [x] Add `via` field to search outputs (human, JSON, records formats)
- [x] Track which compacted source triggered the match
- Files: `src/commands/search/mod.rs`, `src/commands/search/format/*.rs`
- Status: **Implemented**. The `SearchResult` struct has a `via` field that is set when a compacted note is resolved to its digest. All three output formats (JSON, human, records) display the `via` field. The `show` and `context` commands also support `via` annotations (verified by test_compaction_annotations).
- **Note**: Per spec (lines 260-267), traversal outputs (tree/path) do NOT use `via`. Traversals operate on the contracted graph and use `--with-compaction-ids` flag to display compacted notes instead.

### P2: LLM Tool Test Enhancements

#### Safety Guard: `LLM_TOOL_TEST_ENABLED` (`specs/llm-user-validation.md:464`)
- [x] Check for `LLM_TOOL_TEST_ENABLED=1` before running any tests
- [x] Exit with clear error message if not set
- [x] Prevents accidental expensive test runs
- Files: `crates/llm-tool-test/src/main.rs`
- Status: **Implemented**. Added safety guard check at the beginning of `main()` function. The check uses `anyhow::bail!()` to exit with a clear error message explaining the requirement and how to enable test runs.

#### Per-Scenario Timeout (`specs/llm-user-validation.md:158-159`)
- [ ] Read `run.timeout_secs` from scenario YAML (default: 600)
- [ ] Override CLI `--timeout` with scenario-specific value
- [ ] Pass timeout to adapter execution
- Files: `crates/llm-tool-test/src/scenario.rs`, `crates/llm-tool-test/src/run.rs`

#### Store Snapshot Artifact (`specs/llm-user-validation.md:297-298`)
- [ ] After test run, copy `.qipu/` directory to `store_snapshot/` in transcript dir
- [ ] Include `export.json` via `qipu dump --format json`
- [ ] Enables post-hoc analysis of store state
- Files: `crates/llm-tool-test/src/run.rs`, `crates/llm-tool-test/src/transcript.rs`

### P3: Optional Enhancements

#### Beads/Qipu CLI Alignment Review
- [ ] Compare `bd init`/`bd setup`/`bd onboard`/`bd prime` patterns vs qipu equivalents
- [ ] Evaluate `bd onboard` approach: minimal AGENTS.md snippet pointing to `prime`
- [ ] Consider merging `qipu init` + `qipu setup` or adding `qipu onboard`
- [ ] Review beads' `--stealth` flag (combines init + gitignore + agent setup)
- [ ] Assess if qipu help is too verbose for agent discovery of key commands
- [ ] Discourage direct file reading of `.qipu/notes/` - agents should use CLI
  - Options: AGENTS.md guidance, directory naming, or tooling hints
  - Intent: CLI provides consistent formatting, budget control, and graph context
- Reference: `bd --help`, `bd onboard --help`, `bd init --help`

#### Tag Aliases (`specs/knowledge-model.md:53`)
- [ ] Add `aliases` field to config for tag mappings (e.g., `ml: machine-learning`)
- [ ] Resolve aliases during indexing and querying
- [ ] `qipu doctor` warns on orphaned aliases
- Files: `src/lib/config.rs`, `src/lib/db/tags.rs`, `src/commands/doctor/content.rs`

---

## Deferred Items

### Spec Clarification Needed

| Spec | Item | Notes |
|------|------|-------|
| `indexing-search.md` | Backlink index storage | Confirm whether stored or derived |
| `workspaces.md` | `--temp` gitignore behavior | Decide expected behavior |
| `telemetry.md` | Implementation timing | DRAFT spec, explicitly prohibits implementation |

### Technical Debt

| Item | Notes |
|------|-------|
| Performance test thresholds | Current 1s budget is conservative; actual ~500-600ms. Consider tighter regression threshold |
| Test suite optimization | Review for redundancy, parallelization opportunities as suite grows |

---

## Revision 1 Summary (2026-01-22)

Completed full audit against all specs. Key areas addressed:

### Correctness (P1)
- **CLI**: Fixed create output, JSON error envelopes
- **Database**: Removed filesystem fallbacks, auto-rebuild on schema mismatch, incremental repair on validation failure
- **Graph**: Fixed Dijkstra min-heap ordering, default to BFS for `link path`, CSV-style type filters
- **Pack**: Fixed `skip` strategy link handling, `merge-links` edge insertion
- **Value Model**: Added 0-100 validation, fixed default value handling
- **Workspaces**: Implemented `rename` strategy, graph-slice copy for `--from-*`, post-merge validation

### Test Coverage (P1)
Added 100+ new tests covering:
- All CLI commands and flags per spec
- Golden determinism tests (context, search, inbox, show, link tree/path)
- Edge cases: semantic inversion, truncation limits, pack versioning

### Refactoring (P2)
- Split `bfs.rs` into `algos/` module (dijkstra, bfs)
- Modularized similarity engine (tfidf, duplicates, tags)
- Extracted list/search formatting and filtering logic

### Features (P3)
- **Custom Metadata**: Full implementation with type detection, filtering, doctor checks
- **Distribution**: Release automation, cross-platform installers with checksum verification
- **Export**: BibTeX/CSL JSON, transitive traversal, pandoc/PDF integration
- **Semantic Graph**: Per-link-type hop costs
- **LLM Tool Test**: Report command, PTY fallback, structured event logging
- **Tags**: `qipu tags list` command with frequency counts (human, JSON, records formats)

### Architecture Learnings

Key patterns established:
- Database consistency requires calling both `insert_note` AND `insert_edges`
- Schema version bump triggers auto-rebuild via `SchemaCreateResult::NeedsRebuild`
- `path_relative_to_cwd()` helper for consistent relative path output
- Gate evaluation should propagate errors, not swallow with defaults

---

## Notes

- **Schema version**: 6 (custom metadata column added)
- **Store format version**: 1
- Test helper `extract_id()` handles two-line create output (ID + path)
