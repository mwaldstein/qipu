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

#### Doctor Warnings for Semantic Misuse (`specs/semantic-graph.md:109`)
- [x] Warn when standard link types point to non-existent notes (already covered by `check_broken_links`)
- [x] Warn on suspicious patterns (e.g., `part-of` self-loop, `follows` cycle)
- [x] Add `check_semantic_link_misuse()` to doctor content checks (function exists as `check_semantic_link_types` in database.rs)
- Files: `src/commands/doctor/database.rs`, `src/commands/doctor/mod.rs`

#### Single-Note Truncation with Marker (`specs/llm-context.md:106-107`)
- [ ] When budget is tight, truncate individual notes instead of dropping entirely
- [ ] Append `…[truncated]` marker to truncated content
- [ ] Keep truncation deterministic (same input → same output)
- Files: `src/commands/context/mod.rs`, `src/commands/context/human.rs`

#### Breadcrumb `via=<id>` in Traversal Outputs (`specs/compaction.md:118-120,255`)
- [ ] When search/traversal hits a compacted note, annotate result with `via=<matching-source-id>`
- [ ] Add `via` field to link tree/path outputs (human, JSON, records formats)
- [ ] Track which compacted source triggered the match
- Files: `src/commands/link/tree.rs`, `src/commands/link/path.rs`, `src/commands/search/mod.rs`

### P2: LLM Tool Test Enhancements

#### Safety Guard: `LLM_TOOL_TEST_ENABLED` (`specs/llm-user-validation.md:464`)
- [ ] Check for `LLM_TOOL_TEST_ENABLED=1` before running any tests
- [ ] Exit with clear error message if not set
- [ ] Prevents accidental expensive test runs
- Files: `crates/llm-tool-test/src/main.rs`

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
