//! Content constants for setup command

pub const ONBOARD_SNIPPET: &str = r#"## Qipu Knowledge

This project uses **qipu** for knowledge management.
Run `qipu prime` for workflow context.

**Quick reference:**
- `qipu prime` - Get store overview
- `qipu create` - Create note
- `qipu capture` - Quick capture
- `qipu search` - Search notes
- `qipu context` - Build LLM context

For full workflow: `qipu prime`
"#;

pub const AGENTS_MD_CONTENT: &str = r#"# Qipu Agent Integration

Qipu is a Zettelkasten-inspired knowledge management system designed for agent workflows.

## Quick Start

Add this section to your agent tool's configuration or prompt:

```
## Qipu Knowledge Memory

You have access to qipu, a knowledge management CLI for capturing research notes and navigating knowledge via links, tags, and Maps of Content.

### Important: Always Use the CLI

**Never directly read files from `.qipu/notes/` or `.qipu/mocs/`.** Always use the qipu CLI commands:

- The CLI provides consistent formatting (human, json, records)
- Budget control with `--max-chars` ensures you stay within context limits
- Graph context is preserved (links, tags, relationships are resolved correctly)
- Compaction and other internal features work correctly via CLI queries

### Core Commands

- `qipu prime` - Get a session-start primer (store overview, key MOCs, recent notes)
- `qipu create <title>` - Create a new note
- `qipu capture` - Capture note from stdin
- `qipu list` - List notes (filter by --tag, --type, --since)
- `qipu show <id>` - Display a note
- `qipu search <query>` - Search notes by title and body
- `qipu inbox` - Show unprocessed notes (fleeting/literature)
- `qipu context` - Build context bundle for LLM (use --note, --tag, --moc, or --query to select)
- `qipu link list <id>` - List links for a note
- `qipu link tree <id>` - Show link tree (graph neighborhood)
- `qipu link path <from> <to>` - Find path between notes

### Output Formats

All commands support `--format <human|json|records>`:
- `human` - Human-readable (default)
- `json` - Machine-readable structured output
- `records` - Line-oriented format optimized for context injection

### Example Workflows

**Session Start:**
```bash
qipu prime --format records
```

**Capture Research:**
```bash
qipu create "Paper: XYZ" --type literature --tag paper
echo "Key findings..." | qipu capture --title "Insights from XYZ"
```

**Build Context for a Task:**
```bash
# Get overview first
qipu link tree <topic-note-id> --max-hops 2 --format records --max-chars 8000

# Then fetch full content for selected notes
qipu context --note <id1> --note <id2> --format records --with-body --max-chars 16000
```

**Explore Knowledge:**
```bash
qipu search "compaction" --format json
qipu link list <id> --direction both --format json
qipu inbox --exclude-linked
```

### Best Practices

1. **Progressive Disclosure**: Use `qipu link tree` with `--max-chars` to get summaries, then `qipu context --with-body` for details
2. **Deterministic Output**: All commands produce stable, deterministic output for reproducible workflows
3. **Budgeting**: Use `--max-chars` to fit within context limits
4. **Types**: Use note types (fleeting, literature, permanent, moc) to organize knowledge lifecycle
5. **Links**: Use typed links (derived-from, supports, contradicts, part-of) for explicit relationships
```

## Integration Examples

### OpenCode / Cline / Roo-Cline
Add to your project's `AGENTS.md` file (this file is automatically loaded by these tools).

### Cursor
Install cursor rules: `qipu setup cursor`

### Other Agent Tools
Refer to your tool's documentation for adding custom instructions or tool integrations.

## Store Location

Qipu stores are discovered by walking up from the current directory looking for `.qipu/` or `qipu/`.

To create a store: `qipu init`

For stealth mode (gitignored): `qipu init --stealth`

## More Information

Run `qipu --help` for complete command reference.
Visit the qipu repository for full documentation.
"#;

pub const CURSOR_RULES_CONTENT: &str = r#"---
description: Qipu Knowledge Management Integration
glob: "**/*"
---

# Qipu Knowledge Management

You have access to qipu, a Zettelkasten-inspired knowledge management CLI for capturing research notes and navigating knowledge via links, tags, and Maps of Content.

## Critical: Always Use the CLI

**Never directly read files from `.qipu/notes/` or `.qipu/mocs/`.** Always use the qipu CLI commands:

- The CLI provides consistent formatting (human, json, records)
- Budget control with `--max-chars` ensures you stay within context limits
- Graph context is preserved (links, tags, relationships are resolved correctly)
- Compaction and other internal features work correctly via CLI queries

## Core Commands

- `qipu prime` - Get a session-start primer (store overview, key MOCs, recent notes)
- `qipu create <title>` - Create a new note
- `qipu capture` - Capture note from stdin
- `qipu list` - List notes (filter by --tag, --type, --since)
- `qipu show <id>` - Display a note
- `qipu search <query>` - Search notes by title and body
- `qipu inbox` - Show unprocessed notes (fleeting/literature)
- `qipu context` - Build context bundle for LLM (use --note, --tag, --moc, or --query to select)
- `qipu link list <id>` - List links for a note
- `qipu link tree <id>` - Show link tree (graph neighborhood)
- `qipu link path <from> <to>` - Find path between notes

## Output Formats

All commands support `--format <human|json|records>`:
- `human` - Human-readable (default)
- `json` - Machine-readable structured output
- `records` - Line-oriented format optimized for context injection

## Example Workflows

**Session Start:**
```bash
qipu prime --format records
```

**Capture Research:**
```bash
qipu create "Paper: XYZ" --type literature --tag paper
echo "Key findings..." | qipu capture --title "Insights from XYZ"
```

**Build Context for a Task:**
```bash
# Get overview first
qipu link tree <topic-note-id> --max-hops 2 --format records --max-chars 8000

# Then fetch full content for selected notes
qipu context --note <id1> --note <id2> --format records --with-body --max-chars 16000
```

**Explore Knowledge:**
```bash
qipu search "compaction" --format json
qipu link list <id> --direction both --format json
qipu inbox --exclude-linked
```

## Best Practices

1. **Progressive Disclosure**: Use `qipu link tree` with `--max-chars` to get summaries, then `qipu context --with-body` for details
2. **Deterministic Output**: All commands produce stable, deterministic output for reproducible workflows
3. **Budgeting**: Use `--max-chars` to fit within context limits
4. **Types**: Use note types (fleeting, literature, permanent, moc) to organize knowledge lifecycle
5. **Links**: Use typed links (derived-from, supports, contradicts, part-of) for explicit relationships

## Store Location

Qipu stores are discovered by walking up from the current directory looking for `.qipu/` or `qipu/`.

To create a store: `qipu init`

For stealth mode (gitignored): `qipu init --stealth`
"#;
