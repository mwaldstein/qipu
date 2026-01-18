# Qipu Agent Operations Guide

## Build & Run

Qipu is a Rust project. Build and test with:

```bash
cargo build                 # Debug build
cargo build --release       # Release build
cargo test                  # Run all tests
```

Binary locations:
- Debug: `target/debug/qipu`
- Release: `target/release/qipu`

## CLI Usage

Once built/installed, these should work:

```bash
qipu --help
qipu --version
```

## Project Structure

- `specs/` - Application specifications (implementable, technology-agnostic)
- `docs/` - Usage patterns and guidance
- `prompts/` - Prompt templates used by agents/tools

## Code Structure (Prefer Small)

When making changes, prefer small functions and small files.

- Keep functions focused on a single job; extract helpers instead of growing one "do everything" function.
- Aim for functions you can understand without scrolling; if it gets long, split it.
- Keep modules/files cohesive and purpose-driven; if a file starts collecting unrelated concerns, split by responsibility.
- Prefer private helpers (`fn ...`) and small types over large, complicated control flow.
- Avoid over-fragmentation: split when it improves clarity, testability, or reuse—not just to hit a line count.

## CLI Reference

Keep `AGENTS.md` focused on agent/developer operations rather than end-user CLI documentation.

- For a user-facing command quick reference, see `README.md`.
- When writing scripts/tools that parse output, prefer `--format json`.

## Testing

Run the test suite:

```bash
cargo test
```

## Cleanup

Keep the workspace clean:

```bash
cargo clean                 # Remove target/ build artifacts (reclaim disk space)
```

**Do not create ad-hoc test directories** in the repo root (e.g., `*_test/`, `*_debug/`). These patterns are gitignored but clutter the workspace. Use:
- `tests/` for actual test code
- Temporary directories in `/tmp` for manual testing
- The test harness fixtures in `crates/llm-tool-test/fixtures/`

## LLM Tool Validation

Run the LLM validation harness:

```bash
cargo run -p llm-tool-test -- run --scenario crates/llm-tool-test/fixtures/qipu/scenarios/capture_basic.yaml
# Or with a specific tool:
cargo run -p llm-tool-test -- run --scenario crates/llm-tool-test/fixtures/qipu/scenarios/capture_basic.yaml --tool opencode
```
