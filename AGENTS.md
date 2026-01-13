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
- `qipu link add <from> <to>` - Create a link between notes
- `qipu link remove <from> <to>` - Remove a link
- `qipu link list <id>` - List links for a note
- `qipu link tree <id>` - Show link tree
- `qipu link path <from> <to>` - Find path between notes

### Advanced
- `qipu context` - Show contextual notes
- `qipu prime` - Prime notes for AI context
- `qipu export` - Export store data
- `qipu index` - Manage search index
- `qipu sync` - Update indexes and optionally validate
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

- 117 integration tests in `tests/cli_tests.rs`
- 60 unit tests in `src/*/tests`
- All tests currently passing
