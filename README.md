# Qipu

**Local-first knowledge graph CLI for humans and AI agents.**

<!-- Badges (uncomment when available)
[![License](https://img.shields.io/github/license/USER/qipu)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/USER/qipu/ci.yml)](https://github.com/USER/qipu/actions)
[![Crates.io](https://img.shields.io/crates/v/qipu)](https://crates.io/crates/qipu)
-->

> *Sometimes it's ok to give your LLM a graph knowledgebase, as a treat.*

Qipu is a local-first CLI for building a **persistent knowledge graph** that both you and your AI coding agents can query. Think Zettelkasten meets `man pages` meets "please stop re-researching this every session."

**The problem**: Your LLM agent is brilliant at searching the current codebase but has the long-term memory of a goldfish. Every session, it rediscovers the same APIs, re-reads the same docs, and forgets that clever pattern you found last week.

**The fix**: A git-backed knowledge store where research actually accumulates. Typed notes, semantic links, value scoring—all queryable by humans and agents alike.

## Quick Start

```bash
# Install
cargo install --path .

# Initialize a store
qipu init

# Tell your agent
echo "Use 'qipu' for knowledge management. Run 'qipu prime' at session start." >> AGENTS.md

# Capture knowledge
echo "TIL: Rust's ? operator works with Option too" | qipu capture --title "Rust question mark"

# Link notes to build the graph
qipu link add <new-id> <existing-id> --type derived-from

# Search and prime your agent
qipu search "rust error handling"
qipu prime
```

**Example: Building connections**

```bash
# Capture two related notes
echo "anyhow provides context chains for errors" | qipu capture --title "anyhow basics"
# Created note qp-01J5A...

echo "thiserror is better for library error types" | qipu capture --title "thiserror vs anyhow"
# Created note qp-01J5B...

# Link them with semantic relationship
qipu link add qp-01J5B qp-01J5A --type refines

# See the graph
qipu link tree qp-01J5A
```

<!-- Installation options (uncomment when available)
## Installation

```bash
# Cargo (crates.io)
cargo install qipu

# Homebrew
brew install qipu

# From source
cargo install --path .
```
-->

## Essential Commands

| Command | Action |
| --- | --- |
| `qipu capture` | Quick capture from stdin or args |
| `qipu link add <from> <to>` | Create typed link between notes |
| `qipu link tree <id>` | Visualize note's connections |
| `qipu search <query>` | Full-text search with ranking |
| `qipu prime` | Generate context primer for agent sessions |

Run `qipu help` or `qipu <command> --help` for full usage details.

## Features

- **Git-Backed Storage:** Notes stored as files in `.qipu/`. Version, branch, merge with your code.
- **Agent-Optimized Output:** `--format json/records` designed for LLM context injection.
- **Typed Knowledge:** Fleeting notes evolve to permanent; explicit link semantics (`derived-from`, `supports`, `contradicts`).
- **Value Scoring:** Surface high-quality notes, deprecate stale ones. Filter by `--min-value`.
- **Cross-Session Memory:** Research accumulates instead of being rediscovered every session.
- **Health Checks:** `qipu doctor` catches broken links, orphan notes, duplicates.

## CLI Reference

### Core Commands

```bash
qipu init                 # Create a new store
qipu create               # Create a new note (opens editor)
qipu capture              # Quick capture from stdin or args
qipu list                 # List notes (--type, --tag, --since, --min-value filters)
qipu show <id>            # Display a note (--links for connections)
qipu search <query>       # Full-text search with ranking (--min-value, --sort value)
qipu inbox                # Show unlinked fleeting notes
```

### Value Management

```bash
qipu value set <id> <score>    # Set note value (0-100, default: 50)
qipu value show <id>           # Display current value
```

The `value` field represents note quality/importance:
- **0-20**: Deprioritized (superseded drafts, duplicates)
- **21-80**: Standard (general research, work-in-progress)
- **81-100**: High-value (canonical definitions, MOCs, gems)

### Link Management

```bash
qipu link add <from> <to> --type <t>   # Create typed link
qipu link remove <from> <to>           # Remove link
qipu link list <id>                    # List note's links
qipu link tree <id>                    # Show link tree (--min-value, --ignore-value)
qipu link path <from> <to>             # Find path between notes (--min-value, --ignore-value)
```

### LLM Integration

```bash
qipu prime                 # Primer for agent session start
qipu context --moc <id>    # Context bundle from a map of content
qipu context --query <q>   # Context bundle from search
qipu context --tag <t>     # Context bundle by tag
```

Note: `context` command supports `--min-value` to filter by note quality.

### Maintenance

```bash
qipu index --rebuild       # Rebuild search index
qipu sync                  # Update indexes (--commit, --push)
qipu doctor                # Check store health (--fix, --duplicates)
qipu verify <id>           # Mark note as human-verified
```

## Project Structure

```
crates/
  qipu-core/        # Core library (domain logic, persistence, indexing)
  llm-tool-test/    # LLM tool testing utility
src/
  main.rs           # Entry point, CLI parsing, dispatch
  cli/              # CLI argument definitions (Clap derive)
  commands/         # Command implementations (depend on qipu-core)
tests/              # Integration tests and benchmarks
specs/              # Implementable specifications
```

The `qipu-core` crate is a reusable library that can be used independently of the CLI.

## Why Not Just Markdown Files?

You could dump everything in `docs/` and `grep` your way through. We've tried it. Here's what breaks:

| Markdown folder | Qipu |
|-----------------|------|
| Flat or ad-hoc hierarchy | Typed notes (fleeting → permanent) with explicit link semantics |
| Search = `grep` or filename guessing | Full-text search with ranking, value scoring, tag filters |
| "Which doc is current?" | Value scores surface high-quality notes, deprecate stale ones |
| No structure for agents to parse | `--format json/records` designed for LLM context injection |
| Links rot silently | `qipu doctor` catches broken links, orphan notes |
| Everything or nothing | `qipu prime` / `qipu context` give agents *relevant* context, not a haystack |

The graph structure isn't academic—it's how you answer "what do I know about X?" without reading everything.

Qipu is also inspired by [beads](https://github.com/steveyegge/beads)—a similar project focused on moving tasks out of `progress.md` files and enabling context sharing between multiple LLMs.

## Development

```bash
cargo build                 # Debug build
cargo build --release       # Release build
cargo test                  # Run all tests
cargo clippy                # Lint
```

## Documentation

[Specs](specs/README.md) | [Agent Guide](AGENTS.md) | [Usage Patterns](docs/usage-patterns.md) | [Building on Qipu](docs/building-on-qipu.md) | [AI Skills Integration](skills/README.md)
