# Qipu

Qipu is a local-first CLI for capturing and navigating research/knowledge so it stays available to humans and LLM coding agents.

## Quick Start

```bash
# Install (from source)
cargo install --path .

# Create a store in the current directory
qipu init

# Capture a quick note
echo "TIL: Rust's ? operator works with Option too" | qipu capture --title "Rust question mark"

# Create a note interactively
qipu create --type permanent --tag rust

# Search your notes
qipu search "rust error handling"

# Get context for an LLM session
qipu prime
```

## Why Qipu?

LLM coding agents are great at searching the current codebase, but often struggle to:

- Preserve external research (docs, blog posts, papers, issue threads)
- Keep intermediate insights discoverable across sessions
- Avoid repeating the same research on future work
- Distinguish "useful knowledge" from specs and tickets

Qipu provides a git-backed, local-first knowledge store optimized for both humans and agents.

## CLI Reference

### Core Commands

```bash
qipu init                 # Create a new store
qipu create               # Create a new note (opens editor)
qipu capture              # Quick capture from stdin or args
qipu list                 # List notes (--type, --tag, --since, --min-value filters)
qipu show <id>            # Display a note (--links for connections)
qipu search <query>       # Full-text search with ranking (--sort value)
qipu inbox                # Show unlinked fleeting notes
qipu value set <id> <n>   # Set note value (0-100, quality/importance score)
```

### Link Management

```bash
qipu link add <from> <to> --type <t>   # Create typed link
qipu link remove <from> <to>           # Remove link
qipu link list <id>                    # List note's links
qipu link tree <id>                    # Show link tree (--min-value, weighted)
qipu link path <from> <to>             # Find path between notes (--min-value)
```

### LLM Integration

```bash
qipu prime                 # Primer for agent session start
qipu context --moc <id>    # Context bundle from a map of content
qipu context --query <q>   # Context bundle from search
qipu context --tag <t>     # Context bundle by tag
```

### Workspaces

```bash
qipu workspace new <name>      # Create workspace for agent task
qipu workspace list            # List workspaces
qipu workspace merge <name>    # Merge changes back to primary
qipu workspace delete <name>   # Delete workspace
```

### Maintenance

```bash
qipu index --rebuild       # Rebuild search index
qipu sync                  # Update indexes (--commit, --push)
qipu doctor                # Check store health (--fix, --duplicates)
qipu verify <id>           # Mark note as human-verified
```

### Output Formats

All commands support `--format`:
- `human` - Human-readable (default)
- `json` - Structured JSON for programmatic use
- `records` - Line-oriented format for LLM context injection

## Value Model

Notes can have a `value` attribute (0-100) representing quality/importance:

- **0-20**: Deprioritized/junk (superseded drafts, noisy sources)
- **21-80**: Standard (general research, work-in-progress)
- **81-100**: High-value/gems (distilled insights, canonical definitions)

Value affects weighted traversal and filtering:

```bash
qipu value set qp-a1b2 90            # Mark note as high-value
qipu value show qp-a1b2              # Show current value
qipu list --min-value 80             # List only high-value notes
qipu search "topic" --sort value     # Rank results by value
qipu context --moc <id> --min-value 70  # High-value context only
qipu link tree <id> --min-value 50   # Traversal respects value
```

Notes without explicit value default to 50 (neutral). Weighted traversal uses Dijkstra's algorithm where edge cost scales with `(100 - value)`, prioritizing high-value notes.

## Store Structure

```
.qipu/
├── notes/          # Regular notes (fleeting, literature, permanent)
├── mocs/           # Maps of content
├── attachments/    # Referenced files
├── templates/      # Note templates
├── config.toml     # Store configuration
└── qipu.db         # SQLite index (gitignored, rebuildable)
```

## Development

```bash
cargo build                 # Debug build
cargo build --release       # Release build
cargo test                  # Run all tests
cargo clippy                # Lint
```

See `specs/` for detailed specifications and `AGENTS.md` for agent-specific guidance.
