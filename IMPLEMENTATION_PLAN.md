# Qipu Implementation Plan

## Status
- Test baseline: `cargo test` passes (439 tests)
- Clippy baseline: `cargo clippy --all-targets --all-features -- -D warnings` passes
- Audit Date: 2026-01-20

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

---

## P2: Missing Test Coverage

### Workspaces
- [ ] Add tests for `--dry-run` and `--empty` flags
  - `src/commands/workspace/new.rs`: `empty` logic
  - `src/commands/workspace/merge.rs`: `dry_run` logic
  - `tests/workspace_merge_test.rs`: existing tests do not cover these

### LLM Tool Test Harness
- [ ] Add scenario schema validation tests
  - `crates/llm-tool-test/src/scenario.rs`: `Scenario` struct validation
  - Ensure all required fields (id, tags, docs) are present or optional as per spec

### Compaction
- [ ] Truncation indicators in CLI
  - `src/commands/compact/show.rs`: recursive display lacks bounds check and truncation indicators.
  - Spec: Requires bounded expansion and indicators.

---

## P3: Unimplemented Optional / Cleanup

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
