# Qipu Agent Operations Guide

## Build & Run

Build/run instructions are intentionally not specified yet.

Qipu’s specs require a single, self-contained native `qipu` executable (see `specs/cli-tool.md`).

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

- `qipu init` - Create a new store
- `qipu create` - Create a new note
- `qipu list` - List notes
- `qipu show <id>` - Display a note

## Testing

Test harness is not specified yet. Specs emphasize determinism and integration tests (see `specs/cli-tool.md`).
