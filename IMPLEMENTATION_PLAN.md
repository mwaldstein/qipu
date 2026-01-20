# Qipu Implementation Plan

This document tracks **concrete implementation tasks** - bugs to fix, features to complete, and tests to add. For exploratory future work and open questions from specs, see [`FUTURE_PLAN.md`](FUTURE_PLAN.md).

## Status
- Test baseline: `cargo test` passes
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes
- Audit Date: 2026-01-20
- Related: [`specs/README.md`](specs/README.md) - Specification status tracking

---

## P1: Correctness Bugs

### Operational Database (`specs/operational-database.md`)
- [ ] Trigger incremental repair when startup validation fails.
  - `src/lib/db/mod.rs:83-85` (validation result ignored)
  - `src/lib/db/repair.rs:6-141` (repair exists but unused)
- [ ] Treat the database as the source of truth (remove filesystem fallbacks).
  - `src/lib/store/query.rs:13-52`
  - `src/lib/store/query.rs:66-101`
- [ ] Respect `qipu index --rebuild` (avoid full rebuild by default).
  - `src/commands/index.rs:14-19`
- [ ] Align search ranking weights with spec (multiplicative vs additive boosts).
  - `src/lib/db/search.rs:20-105`
- [ ] Add tag frequency statistics (schema + query).
  - `src/lib/db/schema.rs:19-72` (no stats table)
- [ ] Make file + DB updates transactional.
  - `src/lib/store/lifecycle.rs:52-70`
  - `src/lib/store/lifecycle.rs:165-171`

### Storage Format (`specs/storage-format.md`)
- [ ] Add wiki-link canonicalization rewrite option (currently only extracted).
  - `src/lib/index/links.rs:35-137`
- [ ] Persist store root in config (config has no store path field).
  - `src/lib/config.rs:14-115`
- [ ] Enforce flat notes directory (prevent subdirectories during indexing).
  - `src/lib/store/lifecycle.rs:87-205`

### Indexing/Search (`specs/indexing-search.md`)
- [ ] Wire incremental indexing path (currently always rebuilds).
  - `src/commands/index.rs:14-19`
  - `src/lib/db/repair.rs:6-141`

### Graph Traversal (`specs/graph-traversal.md`)
- [ ] Add CSV-style `--types`/`--exclude-types` flags (only repeated flags exist).
  - `src/cli/link.rs:73-79`
  - `src/cli/link.rs:130-136`
- [ ] Implement `qipu context --walk` integration.
  - `src/cli/commands.rs:226-279` (no `--walk`)
  - `src/commands/context/mod.rs:31-336`

### Records Output (`specs/records-output.md`)
- [ ] Add `--max-chars` budgeting for `qipu prime` records output.
  - `src/commands/prime.rs:184-196`
  - `src/cli/commands.rs:167-168`

### LLM Context (`specs/llm-context.md`)
- [ ] Add per-note truncation markers when budgets are applied.
  - `src/commands/context/budget.rs:55-81`
  - `src/commands/context/human.rs:86-167`
- [ ] Add token/char budget targeting for `qipu prime` output.
  - `src/commands/prime.rs:15-70`

### Workspaces (`specs/workspaces.md`)
- [ ] Include "last updated" in `workspace list` output.
  - `src/commands/workspace/list.rs:70-100`
- [ ] Add `rename` merge strategy.
  - `src/commands/workspace/merge.rs:20-91`
- [ ] Implement graph-slice copying for `--from-note`/`--from-tag`/`--from-query`.
  - `src/commands/workspace/new.rs:70-88`
- [ ] Align workspace metadata path/layout with spec.
  - `src/lib/store/paths.rs:19-26`
  - `src/commands/workspace/new.rs:55-62`
- [ ] Run post-merge `qipu doctor` integrity check.
  - `src/commands/workspace/merge.rs:141-149`
- [ ] Populate `parent_id` in `workspace.toml`.
  - `src/commands/workspace/new.rs:56-61`

### Pack (`specs/pack.md`)
- [ ] Preserve all links among included notes regardless of traversal filters.
  - `src/commands/dump/mod.rs:215-255`
- [ ] Preserve attachment paths on load (currently flattens to name).
  - `src/commands/load/mod.rs:70-90`

### Semantic Graph (`specs/semantic-graph.md`)
- [ ] CLI help should list the full standard type set (not a subset).
  - `src/cli/link.rs:17-56`

### Structured Logging (`specs/structured-logging.md`)
- [ ] Respect `--log-level` without requiring `--verbose`.
  - `src/commands/search.rs:36-54`
- [ ] Validate `--log-level` against allowed values.
  - `src/cli/mod.rs:52-54`
- [ ] Add tracing spans for note CRUD paths.
  - `src/lib/db/notes/create.rs:1-75`
  - `src/lib/db/notes/read.rs:1-140`

### Provenance (`specs/provenance.md`)
- [ ] Default `verified` for LLM-generated notes should be false instead of unset.
  - `src/commands/create.rs:49-61`
  - `src/commands/capture.rs:72-84`

### Compaction (`specs/compaction.md`)
- [ ] Enforce `--compaction-max-nodes` in `compact show` and surface truncation.
  - `src/commands/compact/show.rs:46-105`
  - `src/lib/compaction/expansion.rs:101-134`

---

## P2: Missing Test Coverage & Gaps

### CLI Tool (`specs/cli-tool.md`)
- [ ] Add tests for discovery when only `qipu/` exists.
  - `src/lib/store/paths.rs:28-41`
  - `tests/cli/misc.rs:99-114`
- [ ] Expand determinism coverage beyond help/list/prime.
  - `tests/golden_tests.rs:115-217`

### CLI Interface (`specs/cli-interface.md`)
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

### Value Model (`specs/value-model.md`)
- [ ] Add tests for `qipu value set/show` output + validation.
  - `src/cli/value.rs:5-18`
  - `src/commands/dispatch/mod.rs:307-361`
- [ ] Add tests for `search --sort value`.
  - `src/commands/search.rs:38-45`
- [ ] Add tests for `list --min-value` and `context --min-value`.
  - `src/commands/list.rs:59-63`
  - `src/commands/context/mod.rs:233-238`
- [ ] Add tests for `--ignore-value` traversal ordering.
  - `src/commands/link/tree.rs:61-73`
  - `src/commands/link/path.rs:75-88`
- [ ] Add tests for schema migration adding `value`.
  - `src/lib/db/schema.rs:95-103`

### Export (`specs/export.md`)
- [ ] Add tests for `--tag` and `--query` selection ordering.
  - `src/commands/export/plan.rs:10-62`
  - `tests/cli/export.rs:6-341`
- [ ] Add tests for `--mode bibliography`.
  - `src/commands/export/emit/bibliography.rs:4-40`
  - `tests/cli/export.rs:6-341`
- [ ] Add tests for `--link-mode markdown` and default `preserve`.
  - `src/commands/export/mod.rs:47-69`
  - `tests/cli/export.rs:6-341`

### Graph Traversal (`specs/graph-traversal.md`)
- [ ] Add tests for semantic inversion in `link tree/path`.
  - `src/commands/link/tree.rs:57-60`
  - `tests/cli/link/add_remove.rs:55-73`
- [ ] Add tests for `max_nodes`, `max_edges`, and `max_fanout` truncation.
  - `src/cli/link.rs:89-99`
  - `tests/cli/link/tree.rs:366-435`

### Records Output (`specs/records-output.md`)
- [ ] Add `max-chars` truncation tests for link tree/path records output.
  - `src/commands/link/tree.rs:276-396`
  - `src/commands/link/records.rs:205-314`
  - `tests/cli/link/tree.rs:148-178`
  - `tests/cli/link/path.rs:202-245`

### Workspaces (`specs/workspaces.md`)
- [ ] Add tests for `workspace merge --dry-run`.
  - `src/commands/workspace/merge.rs:20-147`
  - `tests/workspace_merge_test.rs:9-205`

### Semantic Graph (`specs/semantic-graph.md`)
- [ ] Add tests for additional standard types and custom inverses.
  - `src/lib/note/types.rs:92-169`
  - `src/lib/config.rs:65-78`
  - `tests/cli/link/add_remove.rs:46-73`

### Similarity Ranking (`specs/similarity-ranking.md`)
- [ ] Add tests for default similarity thresholds and field weighting.
  - `src/lib/similarity/mod.rs:27-138`
  - `tests/cli/doctor.rs:305-389`
- [ ] Add end-to-end tests for stop-word filtering.
  - `src/lib/text/mod.rs:8-54`
  - `src/lib/similarity/mod.rs:27-138`

### Pack (`specs/pack.md`)
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

### Provenance (`specs/provenance.md`)
- [ ] Add tests for default `verified=false` behavior on LLM-origin notes.
  - `src/commands/create.rs:49-61`
  - `tests/cli/provenance.rs:6-124`

---

## P3: Unimplemented Optional / Future

### Custom Metadata (`specs/custom-metadata.md`)
- [ ] Implement custom frontmatter fields, DB storage, and CLI access.
  - `src/lib/note/frontmatter.rs:5-54`
  - `src/lib/db/schema.rs:19-33`
  - `src/cli/commands.rs:226-279`

### Distribution (`specs/distribution.md`)
- [ ] Add release automation and install scripts (GitHub releases + installers).
  - `src/commands/setup.rs:1-138` (integration setup only)

### LLM User Validation (`specs/llm-user-validation.md`)
- [ ] Implement the llm-tool-test harness in code + tests.
  - `specs/llm-user-validation.md:17-520`
  - `tests/transcripts/opencode/` (transcript artifacts only)

### Export (`specs/export.md`)
- [ ] Add optional BibTeX/CSL JSON outputs.
  - `src/commands/export/emit/bibliography.rs:4-40`
- [ ] Add transitive export traversal (depth-limited).
  - `src/commands/export/plan.rs:112-209`

### Similarity Ranking (`specs/similarity-ranking.md`)
- [ ] Add clustering/"see also" features for MOC generation.
  - `src/lib/similarity/mod.rs:27-138`

### Knowledge Model (`specs/knowledge-model.md`)
- [ ] Implement tag alias support (optional in spec).
  - `src/lib/note/frontmatter.rs:21-23`

---

## P4: Spec Ambiguity

### LLM Context (`specs/llm-context.md`)
- [ ] Clarify whether store paths should be relative or absolute in outputs.
  - `src/commands/context/human.rs:86-88`
  - `src/commands/context/json.rs:87-88`
  - `src/commands/context/records.rs:203-207`
  - `src/commands/prime.rs:72-80`

### Indexing/Search (`specs/indexing-search.md`)
- [ ] Confirm whether "backlink index" must be stored or can be derived.
  - `src/lib/index/types.rs:161-169`

### Workspaces (`specs/workspaces.md`)
- [ ] Decide expected gitignore behavior for `--temp` workspaces.
  - `src/commands/workspace/new.rs:33-101`

---

## Completed (Verified 2026-01-20)

### Workspaces
- [x] `--empty` flag in `workspace new` verified and tested.
- [x] Merge strategies verified.

### Structured Logging
- [x] `src/commands/capture.rs` - Verified `tracing::debug!` usage.
- [x] `src/commands/compact/*.rs` - Verified.
- [x] `src/commands/context/*.rs` - Verified.
- [x] `src/commands/workspace/*.rs` - Verified.
- [x] `eprintln!` cleanup (reduced from 16 to 4 acceptable calls in `main.rs`).

### File Size Refactoring
- [x] `src/commands/context/output.rs` split -> `json.rs`, `human.rs`, `records.rs`.
- [x] `src/lib/graph/traversal.rs` split -> `bfs.rs`.
- [x] `src/commands/link/list.rs` extracted output formatters.
- [x] `src/lib/db/notes.rs` split CRUD operations.
- [x] `src/commands/doctor/checks.rs` split by category.
