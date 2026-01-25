# Qipu Agent Operations Guide

Qipu is a Rust CLI for Zettelkasten-inspired knowledge management. This guide helps coding agents work effectively in this codebase.

## Build & Test Commands

```bash
# Build
cargo build                     # Debug build
cargo build --release           # Release build (optimized)

# Lint & Format
cargo fmt --all -- --check      # Check formatting
cargo fmt --all                 # Auto-format
cargo clippy --all-targets --all-features -- -D warnings  # Lint (treat warnings as errors)

# Run all tests
cargo test                      # All tests (812 total)
cargo test --verbose            # With verbose output

# Run single test by name
cargo test test_capture_basic                    # Exact match
cargo test capture                               # Partial match (runs all matching)
cargo test --test cli_tests capture              # Tests in specific test file

# Run test module
cargo test --test cli_tests cli::search          # Module path

# Run benchmarks (release mode required)
cargo test --test bench_tests --release -- --ignored
```

**Note:** This is a binary crate—do not use `cargo test --lib` (no library target exists).

## Project Structure

```
src/
  main.rs           # Entry point, CLI parsing, dispatch
  cli/              # CLI argument definitions (Clap derive)
  commands/         # Command implementations
  lib/              # Shared library (db/, store/, note/, graph/, error.rs)
tests/
  cli/              # Integration tests by command
  cli/support.rs    # Test helpers (qipu() command builder)
  bench/            # Performance benchmarks
specs/              # Implementable specifications
```

## Code Style Guidelines

### Imports
- Group imports: std, external crates, internal modules
- Use `crate::` prefix for internal imports (not `super::`)
- Import types directly, not modules: `use crate::lib::note::NoteType;`

### Formatting
- Run `cargo fmt` before committing
- Max line length: implied by rustfmt defaults
- Use trailing commas in multi-line constructs

### Types & Naming
- Types: PascalCase (`NoteType`, `LinkType`)
- Functions/methods: snake_case (`extract_id_from_bytes`)
- Constants: SCREAMING_SNAKE_CASE (`VALID_TYPES`)
- Modules: snake_case
- Enums: PascalCase variants (`NoteType::Fleeting`)

### Error Handling
- Use `thiserror` for error enums (`#[derive(Error)]`)
- Use `anyhow` sparingly (prefer typed errors)
- Exit codes: 0=Success, 1=Generic failure, 2=Usage error, 3=Data/store error
- Use `?` operator for propagation; return `Result<T>` from fallible functions

### Documentation
- Module-level doc comments with `//!`, function/type docs with `///`
- Reference specs in comments when implementing spec requirements

## CI Enforcement

- **File Size Limit**: 500 lines max per file. Exceptions in `.github/workflows/ci.yml:67-78`.
- **Function Complexity**: 100 lines max per function. Checked by `scripts/check_function_complexity.py`.
- **All Warnings as Errors**: `cargo clippy -- -D warnings` is enforced.

## Test Patterns

### Integration Test Structure
```rust
use crate::cli::support::{extract_id_from_bytes, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_feature_behavior() {
    let dir = tempdir().unwrap();
    
    // Always init store first
    qipu().current_dir(dir.path()).arg("init").assert().success();
    
    // Run command under test
    let output = qipu()
        .current_dir(dir.path())
        .args(["command", "--flag", "value"])
        .write_stdin("input")
        .assert()
        .success();
    
    // Assert on output
    output.stdout(predicate::str::contains("expected"));
}
```

### Test Helpers
- `qipu()` - Returns Command builder for CLI testing
- `extract_id(&output)` - Extract note ID from command output

## Key Conventions

### Output Formats
All commands support `--format`: `human` (default), `json`, `records`

### Note Types
`fleeting`, `literature`, `permanent`, `moc`

### Link Types (Standard Ontology)
`related`, `derived-from`, `supports`, `contradicts`, `part-of`, `answers`, `refines`, `same-as`, `alias-of`, `follows`

### ID Format
Notes use ULID-based IDs prefixed with `qp-`: `qp-01HYX3...`

## Implementation Workflow

1. Use `bd ready` to find unblocked work (P1 bugs, P2 tech debt)
2. Read relevant spec in `specs/*.md` before implementing
3. Search codebase before assuming something is unimplemented
4. Run `cargo test` after changes - all 812 tests must pass
5. Use `bd close <id>` when completing tasks, then `bd sync`

## Commit Style

```
<type>: <description>
```

Types: `fix`, `feat`, `test`, `docs`, `refactor`

## Key Files

| File | Purpose |
|------|---------|
| `specs/README.md` | Spec index with implementation status |
| `src/lib/error.rs` | Error types and exit codes |
| `tests/cli/support.rs` | Test helper functions |

## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`
