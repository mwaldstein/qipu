# Qipu Implementation Plan

This document tracks **concrete implementation tasks** - bugs to fix, features to complete, and tests to add. For exploratory future work and open questions from specs, see [`FUTURE_PLAN.md`](FUTURE_PLAN.md).

## Status
- Test baseline: `cargo test` passes
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes
- Audit Date: 2026-01-20
- Related: [`specs/README.md`](specs/README.md) - Specification status tracking

---

## P1: Correctness Bugs

### Operational Database
- [ ] Startup repair trigger missing
  - `src/lib/db/mod.rs`: `Database::open` calls `validate_consistency` but ignores the result.
  - Spec: "On startup, qipu validates consistency and repairs if needed".
- [ ] FTS5 ranking scoring mismatch
  - `src/lib/db/search.rs`: Uses additive boosting (+5.0, +8.0).
  - Spec: Requires multiplicative boosting (2.0x, 1.5x).

### LLM Tool Test Harness
- [ ] Fix default tool mismatch in `llm-user-validation.md` (spec says "amp", code uses "opencode")
  - `crates/llm-tool-test/src/cli.rs:31`: `default_value = "opencode"`
  - `specs/llm-user-validation.md`: says default is "amp"

### Compaction
- [ ] Truncation indicators in CLI
  - `src/commands/compact/show.rs`: recursive display lacks bounds check and truncation indicators in header output.
  - Spec: Requires bounded expansion and indicators.

---

## P2: Missing Test Coverage & Gaps

### Workspaces
- [ ] Add tests for `--dry-run` flag
  - `src/commands/workspace/merge.rs`: `dry_run` logic exists but lacks tests.
  - `tests/workspace_merge_test.rs`: existing tests do not cover dry run.

### LLM Tool Test Harness
- [ ] Add scenario schema validation and missing fields
  - `crates/llm-tool-test/src/scenario.rs`: `Scenario` struct missing `tags`, `docs` fields.
  - Ensure all required fields are present as per spec.
- [ ] Fix fixture location mismatch
  - Spec says `tests/llm_scenarios/`, code uses `crates/llm-tool-test/fixtures/qipu/scenarios`.

---

## P3: Unimplemented Optional / Cleanup

### Custom Metadata
- [ ] Implement Custom Metadata spec
  - `specs/custom-metadata.md`: Entire spec unimplemented.
  - Needs `custom` field in `NoteFrontmatter`, CLI command `qipu custom`, and SQLite JSON index.

### Distribution
- [ ] Implement release automation and scripts
  - `specs/distribution.md`: Entire spec unimplemented.
  - Missing: `.github/workflows/release.yml`, `scripts/install.sh`, `scripts/install.ps1`.

### Structured Logging
- [ ] Remove or use `tracing-appender`
  - `Cargo.toml`: Dependency present but unused in code.
  - Spec: Does not explicitly mandate file logging, but dependency suggests intent.

---

## Completed (Verified 2026-01-20)

### Workspaces
- [x] `--empty` flag in `workspace new` verified and tested.
- [x] Merge strategies verified.

### Structured Logging
- [x] `src/commands/capture.rs` - Verified `tracing::debug!` usage
- [x] `src/commands/compact/*.rs` - Verified
- [x] `src/commands/context/*.rs` - Verified
- [x] `src/commands/workspace/*.rs` - Verified
- [x] `eprintln!` cleanup (reduced from 16 to 4 acceptable calls in `main.rs`)

### File Size Refactoring
- [x] `src/commands/context/output.rs` split -> `json.rs`, `human.rs`, `records.rs`
- [x] `src/lib/graph/traversal.rs` split -> `bfs.rs`
- [x] `src/commands/link/list.rs` extracted output formatters
- [x] `src/lib/db/notes.rs` split CRUD operations
- [x] `src/commands/doctor/checks.rs` split by category

### Value Model (`specs/value-model.md`)
- [x] Data Model: `value` in `NoteFrontmatter`, schema v2, index support
- [x] CLI: `qipu value` command, `--min-value` filters in list/search/context/link
- [x] Traversal: `get_edge_cost`, `dijkstra_traverse`, `--ignore-value`
- [x] Integration: `doctor` range check, weighted traversal tests

### Export (`specs/export.md`)
- [x] Strategies: `outline`, `bundle`, `bibliography`
- [x] Ordering: Deterministic (MOC-driven or sorted)
- [x] Format: Anchor rewriting and linking

### LLM Tool Test Harness
- [x] Scenarios: smoke, basic, search, context
- [x] Gates: NoteExists, LinkExists, TagExists, ContentContains, CommandSucceeds
- [x] Judge: Semantic quality, composite score
- [x] Human Review: `review` command, database storage
- [x] Infrastructure: `commands.rs` extraction

### Operational Database
- [x] Consistency check on startup: `db.validate_consistency()` called in `src/lib/db/mod.rs:84`
