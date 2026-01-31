# Qipu Agent Operations Guide (Test Fixture)

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
