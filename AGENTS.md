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

## Key Commands

### Core Operations
- `qipu init` - Create a new store
- `qipu create` - Create a new note
- `qipu capture` - Quick capture with optional stdin
- `qipu list` - List notes
- `qipu show <id>` - Display a note
- `qipu search <query>` - Search notes
- `qipu inbox` - Show inbox items

### Link Management
- `qipu link add <from> <to> --type <t>` - Create a link between notes
- `qipu link remove <from> <to> --type <t>` - Remove a link
- `qipu link list <id>` - List links for a note
- `qipu link tree <id>` - Show link tree
- `qipu link path <from> <to>` - Find path between notes

### Advanced
- `qipu context` - Show contextual notes
- `qipu prime` - Prime notes for AI context
- `qipu verify <id>` - Toggle verification status of a note
- `qipu export` - Export store data; supports --with-attachments to copy media
- `qipu index` - Manage search index
- `qipu sync` - Update indexes and optionally validate; supports --commit/--push for git automation
- `qipu doctor` - Check store health

### Output Formats

All commands support multiple output formats:
```bash
--format human    # Human-readable (default)
--format json     # JSON output
--format records  # Record-based format
```

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
