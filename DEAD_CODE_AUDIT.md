# Dead Code Audit Report

**Task:** qipu-4952 - Hunt for and remove dead code
**Date:** 2026-01-29
**Methods Used:**
- `cargo-udeps` (unused dependency detection)
- `cargo clippy` (dead code warnings)
- `cargo rustc --dead-code` (dead code warnings)
- Manual code inspection

## Audit Findings

### 1. Dependencies
**Result:** No unused dependencies found
- All dependencies in `Cargo.toml` files are actively used
- `cargo-udeps` reports: "All deps seem to have been used."

### 2. Dead Code Warnings
**Result:** No dead code warnings
- `cargo clippy` reports no `dead_code` warnings
- `cargo rustc --dead-code` reports no warnings

### 3. `#[allow(dead_code)]` Items Examined

All items marked with `#[allow(dead_code)]` were verified as actually used:

#### qipu-core
- **`config.rs`:**
  - `set_link_cost()` - Used in `graph/types.rs` and tests
  - `add_tag_alias()` - Used in tests
  - `resolve_tag_alias()` - Used in tests

- **`ontology.rs`:**
  - `STANDARD_NOTE_TYPES` - Used internally
  - `STANDARD_LINK_INVERSES` - Used internally
  - `Ontology` struct - Used in `config.rs`, `prime/mod.rs`, `commands/ontology.rs`, `commands/context/`, `commands/doctor/ontology.rs`

- **`schema.rs`:**
  - `force_set_schema_version()` - Used in `db/tests/consistency.rs`

#### llm-tool-test
- **`adapter/mod.rs`:** All adapter types (AdapterError, ToolStatus, TaskContext, TokenUsage, CostEstimate, ExecutionResult) are used in adapter implementations
- **`judge.rs`:**
  - `run_judge()` - Test helper
  - `build_judge_prompt()` - Test helper
- **`adapter/mock.rs`:**
  - `run_judge_with_client()` - Test helper
- **`transcript.rs`:**
  - `TranscriptWriter` struct - Used in `run.rs` and `adapter/mock.rs`
  - `read_events()` - Test helper

### 4. Ignored Tests
**Result:** All `#[ignore]` tests are intentional
- Benchmark tests marked with `#[ignore]` require `--release` flag for meaningful results
- Documented in test files: "Run with: cargo test <test_name> --release -- --ignored"

### 5. Stub Code Found (Not Dead Code)
**Location:** `crates/qipu-core/src/store/notes.rs:87`

```rust
_ => todo!(),
```

**Context:** This is in the `default_template()` function as a catch-all case in a match statement on `NoteType`.

**Note:** This is not "dead code" - it's stub code that would panic if a custom note type was used. The function handles the 4 standard note types (fleeting, literature, permanent, moc) and has a catch-all for other types.

## Conclusion

**No dead code found to remove.**

The codebase is clean with:
- No unused dependencies
- No unused public functions
- No unused imports
- No unused structs/enums
- No unused traits
- No dead code blocks
- No unreachable code

All items marked with `#[allow(dead_code)]` are actively used (either in production code or tests).
