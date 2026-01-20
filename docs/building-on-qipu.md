# Building on Top of Qipu

This guide is for developers building applications, agents, and tools that use qipu as a knowledge graph foundation.

Status: Draft  
Last updated: 2026-01-20

## Overview

Qipu is designed as a **knowledge graph tool for LLM agents**. It provides:
- A structured, traversable graph of notes with semantic link types
- Clean, deterministic context bundles for LLM prompts
- Custom metadata support for application-specific extensions
- CLI-first interface suitable for agent tool integration

This document covers how to integrate qipu into your application, particularly focusing on LLM-based systems.

## Core Integration Pattern

### 1. Session Initialization

Start each LLM session with qipu's primer:

```bash
qipu prime
```

This outputs a compact (~1-2k tokens) introduction to qipu that teaches the LLM:
- What qipu is and how it works
- Available commands and their purposes
- The current store location
- Key MOCs and recently updated notes

**Recommended:** Inject `qipu prime` output into your system prompt or initial context automatically.

### 2. Dynamic Context Injection

During task execution, provide the LLM with relevant knowledge:

```bash
# By topic/tag
qipu context --tag authentication --max-chars 8000

# By MOC (curated collection)
qipu context --moc qp-oauth-research --max-chars 12000

# By search query
qipu context --query "token validation" --max-chars 6000

# Multiple selectors (combined)
qipu context --moc qp-api-design --tag security --min-value 70 --max-chars 10000
```

**Key points:**
- Always use `--max-chars` to control token budget precisely
- `--format json` for programmatic consumption
- `--format records` for line-oriented parsing
- Default format (markdown) is human-friendly for direct LLM injection

### 3. Knowledge Capture

Allow your application to capture new knowledge:

```bash
# Capture from stdin
echo "OAuth PKCE flow prevents authorization code interception" | \
  qipu capture --title "PKCE Security Benefit" --type permanent --tag oauth

# With provenance tracking
qipu capture --type literature \
  --source "https://datatracker.ietf.org/doc/html/rfc7636" \
  --author "agent-research-bot" \
  --tag oauth --tag security
```

### 4. Graph Navigation

Traverse the knowledge graph programmatically:

```bash
# Find related notes via links
qipu link tree qp-oauth-research --max-hops 2 --format json

# Find connection path between concepts
qipu link path qp-auth-basics qp-token-validation --format json

# List all outbound links
qipu link list qp-api-design --direction outbound --format json
```

## Custom Metadata for Application Extensions

Custom metadata allows you to layer application-specific data onto qipu's knowledge graph without forking the core schema.

### Design Philosophy

**Custom metadata is for applications, not for LLMs.**

- Qipu's core fields (`value`, `verified`, `tags`, `links`) are designed for LLM-driven knowledge management
- Custom fields are for your application's internal tracking, workflow states, and integrations
- By default, custom fields are **hidden from LLM context** to keep prompts clean

### Setting Custom Properties

Use the `qipu custom` subcommand (hidden from standard help):

```bash
# Set application-specific metadata
qipu custom set qp-a1b2 workflow_state review
qipu custom set qp-a1b2 priority 1
qipu custom set qp-a1b2 assignee "alice@example.com"

# Complex values use JSON/YAML
qipu custom set qp-a1b2 metadata '{"version": 2, "imported_from": "notion"}'
```

#### Type Detection

Values are automatically typed using YAML parsing:

```bash
qipu custom set qp-123 count 42          # → int: 42
qipu custom set qp-123 score 3.14        # → float: 3.14
qipu custom set qp-123 active true       # → bool: true
qipu custom set qp-123 status pending    # → string: "pending"
qipu custom set qp-123 tags '[1, 2, 3]'  # → array: [1, 2, 3]

# Force string type when ambiguous
qipu custom set qp-123 code '"001"'      # → string: "001" (not int 1)
```

See `specs/custom-metadata.md` for complete details.

### Filtering by Custom Properties

```bash
# Filter notes by custom field
qipu list --custom workflow_state=review

# Combine with core filters
qipu list --tag api --custom priority=1 --min-value 70

# Context with custom filters
qipu context --moc qp-sprint-23 --custom assignee="alice@example.com"
```

### Including Custom Metadata in Context

By default, custom fields are **excluded** from `qipu context` output. To include them:

```bash
qipu context --note qp-a1b2 --custom
```

This opt-in design prevents LLMs from seeing (and potentially hallucinating about) your application's internal metadata.

## Teaching LLMs About Custom Properties

If your application needs the LLM to understand and work with custom properties, you can extend the session primer.

### Example: Custom Workflow Integration

**Your system prompt addition:**

```
This project uses qipu with custom workflow metadata. When viewing notes,
you may see custom fields like:

- workflow_state: Current stage (draft, review, approved, archived)
- priority: Importance level (1=highest, 5=lowest)
- assignee: Team member responsible for this note

To filter by workflow state:
  qipu list --custom workflow_state=review

To update a note's workflow state:
  qipu custom set <note-id> workflow_state approved

Custom fields are application-specific and do not affect qipu's core
knowledge graph features (links, traversal, value scoring).
```

### Example: Alignment Tracking (Blibio Use Case)

```
This system tracks your agreement/disagreement with sources using
custom metadata:

- alignment: Your stance on the content (agree, disagree, neutral, unsure)

When capturing research that you disagree with:
  qipu capture --type literature --tag <topic> < article.md
  qipu custom set <new-note-id> alignment disagree

To find sources you agree with:
  qipu context --tag <topic> --custom alignment=agree

To find sources you disagree with (for counter-argument research):
  qipu context --tag <topic> --custom alignment=disagree

This allows you to maintain high-quality sources in your knowledge graph
even when you disagree with their conclusions.
```

### Prompting Best Practices

1. **Be explicit about the schema:** Define what each custom field means and what values are valid
2. **Show examples:** Demonstrate both setting and querying custom fields
3. **Explain the boundary:** Make it clear that custom fields don't affect core qipu semantics
4. **Provide defaults:** If fields should have default values, specify them
5. **Mention opt-in visibility:** Remind the LLM that custom fields require `--custom` flag in context output

## Use Cases

### 1. Research Curation System (Blibio)

Track user alignment with sources:

```bash
# Capture a source you disagree with
qipu capture --type literature --source "https://example.com/article" \
  --tag economics < article.md

# Mark alignment
qipu custom set qp-xyz123 alignment disagree
qipu custom set qp-xyz123 blibio_submission "sub-7x9k"

# Build context of sources you agree with
qipu context --tag economics --custom alignment=agree --max-chars 15000

# Find counter-arguments (sources you disagree with)
qipu context --tag economics --custom alignment=disagree --max-chars 8000
```

### 2. Team Knowledge Base

Track review status and ownership:

```bash
# Create a note needing review
qipu create "API Rate Limiting Strategy" --type permanent --tag api
qipu custom set qp-abc456 workflow_state draft
qipu custom set qp-abc456 author "bob@example.com"
qipu custom set qp-abc456 reviewers '["alice@example.com", "charlie@example.com"]'

# Find notes needing review
qipu list --custom workflow_state=review --format json

# Approve a note
qipu custom set qp-abc456 workflow_state approved
qipu verify qp-abc456 --status true  # Mark as verified in qipu's core schema
```

### 3. LLM-Generated Content Tracking

Track generation metadata beyond qipu's core provenance fields:

```bash
# Capture LLM-generated content
qipu capture --type permanent --generated_by "claude-3.5-sonnet" \
  --author "research-agent" < generated-summary.md

# Add custom tracking
qipu custom set qp-def789 generation_cost_usd 0.042
qipu custom set qp-def789 input_token_count 8234
qipu custom set qp-def789 temperature 0.7
qipu custom set qp-def789 prompt_version "v2.3"

# Query by generation parameters
qipu list --custom prompt_version="v2.3" --tag research
```

### 4. Multi-Stage Document Pipeline

Track documents through production stages:

```bash
# Research phase
qipu create "OAuth 2.1 Overview" --type permanent --tag auth
qipu custom set qp-ghi012 stage research
qipu custom set qp-ghi012 target_audience "backend-developers"

# Draft phase
qipu custom set qp-ghi012 stage draft
qipu custom set qp-ghi012 word_count 1500

# Review phase
qipu custom set qp-ghi012 stage review
qipu custom set qp-ghi012 review_deadline "2026-02-01"

# Find drafts ready for review
qipu list --custom stage=draft --tag auth

# Export published content
qipu context --custom stage=published --tag auth --format markdown
```

## Storage and Deployment Options

Qipu is designed to work with git-based storage by default, but provides several options for alternative deployment models.

### Store Location Options

#### Default: Repository-Local Store

By default, qipu creates a `.qipu/` directory in your repository:

```bash
qipu init
# Creates .qipu/ with notes/, mocs/, config.toml, etc.
```

This mirrors the beads pattern and is ideal for:
- Team collaboration (commit notes to git)
- Project-specific knowledge bases
- Documentation alongside code

#### Visible Store

Use `qipu/` instead of `.qipu/` for more prominent visibility:

```bash
qipu init --visible
# Creates qipu/ instead of .qipu/
```

Useful for:
- Documentation-first projects where notes are primary content
- Making the knowledge base more discoverable to team members

#### Stealth Mode (Git-Ignored)

Create a local-only store that's automatically gitignored:

```bash
qipu init --stealth
# Creates .qipu/ and adds it to .gitignore
```

Perfect for:
- Personal notes on shared projects
- Private research that shouldn't be committed
- Application-managed stores where end users don't interact with git

#### Custom Store Location

Specify an explicit store path:

```bash
# Global flag (any command)
qipu --store /path/to/custom/store list

# Or set working directory
qipu --root /path/to/project context --tag api
```

Applications can use this to:
- Manage multiple isolated stores
- Store notes in application-specific directories
- Keep qipu data separate from project files

### Protected Branch Workflow

For teams with protected main branches, qipu supports committing notes to a separate branch:

```bash
qipu init --branch qipu-metadata
```

This configures automatic branch management:

```bash
# Sync command will commit to the configured branch
qipu sync --commit --push
```

The `branch` setting is stored in `.qipu/config.toml`:

```toml
version = 1
branch = "qipu-metadata"
```

**Use cases:**
- Main branch requires PR reviews, but notes need frequent updates
- Separating knowledge commits from code commits
- Different access policies for code vs. documentation

### Alternative Storage Backends

For applications that need to bypass git entirely, you have several options:

#### 1. In-Memory / Ephemeral Stores

Create temporary stores for session-based knowledge:

```bash
# Create temp directory
TEMP_STORE=$(mktemp -d)

# Initialize store there
qipu --store "$TEMP_STORE" init

# Use for session
qipu --store "$TEMP_STORE" capture < input.md
qipu --store "$TEMP_STORE" context --tag session

# Clean up when done
rm -rf "$TEMP_STORE"
```

#### 2. Application-Managed File Storage

Your application can manage the file structure directly while using qipu as the interface:

```python
# Python example: Application-managed storage
import subprocess
import tempfile
from pathlib import Path

class ManagedQipuStore:
    def __init__(self, base_path):
        self.store_path = Path(base_path) / ".qipu"
        self.store_path.mkdir(parents=True, exist_ok=True)
        
        # Initialize qipu store
        subprocess.run(
            ["qipu", "--store", str(self.store_path), "init"],
            check=True
        )
    
    def sync_to_external_storage(self):
        """Sync .qipu/notes/ to your own storage system"""
        notes_dir = self.store_path / "notes"
        # Upload to S3, database, etc.
        pass
    
    def sync_from_external_storage(self):
        """Pull notes from external storage"""
        # Download from S3, database, etc.
        # Then rebuild index
        subprocess.run(
            ["qipu", "--store", str(self.store_path), "index", "--rebuild"],
            check=True
        )
```

#### 3. Database-Only Mode (Advanced)

For applications that want to manage files separately, you can work primarily with the SQLite database:

```bash
# Initialize store
qipu init --stealth

# Your application manages note files
# but uses qipu for indexing and querying
qipu index --rebuild

# Query via qipu CLI
qipu list --format json

# Or query database directly
sqlite3 .qipu/qipu.db "SELECT id, title FROM notes WHERE ..."
```

**Important caveats:**
- The SQLite database is a **derived index**, not the source of truth
- Always maintain note files (markdown with frontmatter) as the primary data
- Use `qipu index --rebuild` to regenerate the database from files

### Configuration Management

Store configuration is managed via `.qipu/config.toml`:

```toml
# Store format version
version = 1

# Default note type for new notes
default_note_type = "fleeting"

# ID generation scheme: "hash" (default), "ulid", or "timestamp"
id_scheme = "hash"

# Optional: Editor override
editor = "vim"

# Optional: Protected branch workflow
branch = "qipu-metadata"

# Optional: Custom link types
[graph.types.custom_link]
inverse = "custom_inverse"
description = "Application-specific link type"
```

Applications can:
- Pre-generate config files with application-specific defaults
- Use different ID schemes (hash is collision-resistant for multi-agent workflows)
- Define custom link types for domain-specific semantics

### Sync and Automation

For automated workflows:

```bash
# Update indexes
qipu index --rebuild

# Validate and repair
qipu sync --validate --fix

# With git automation (if branch configured)
qipu sync --commit --push
```

Applications can:
- Run `qipu sync` after bulk operations
- Schedule periodic validation with `qipu doctor`
- Automate git commits for team stores

### Multi-Store Management

Applications managing multiple stores can use shell functions or wrappers:

```bash
# Bash example: Multi-store management
function qipu-project() {
    qipu --store ~/projects/$1/.qipu "${@:2}"
}

function qipu-global() {
    qipu --store ~/.qipu-global "$@"
}

# Usage
qipu-project my-app list --tag api
qipu-global context --tag reference
```

### Performance Considerations

For large stores or high-throughput applications:

1. **Database is local and fast:** SQLite operations are instant for typical workloads
2. **File count matters:** Flat storage in `notes/` works well up to ~10k notes
3. **Index rebuilds:** `qipu index --rebuild` scales linearly with note count
4. **Search is indexed:** Full-text search uses SQLite FTS5 for fast queries

**Recommendations:**
- Keep individual notes small (< 100KB)
- Use `--format json` for programmatic consumption (avoids terminal formatting overhead)
- Batch operations when possible (bulk import, then single index rebuild)
- Monitor `.qipu/qipu.db` size (should be < 10% of total note content)

## Advanced Integration

### Direct Database Access

For advanced queries, you can access qipu's SQLite database directly:

```bash
# Location: .qipu/qipu.db

# Query custom metadata with SQLite JSON functions
sqlite3 .qipu/qipu.db \
  "SELECT id, title FROM notes 
   WHERE json_extract(custom_json, '$.priority') = 1
   ORDER BY value DESC LIMIT 10"

# Find notes with specific custom field
sqlite3 .qipu/qipu.db \
  "SELECT id, title, json_extract(custom_json, '$.workflow_state') as state
   FROM notes
   WHERE json_extract(custom_json, '$.workflow_state') IS NOT NULL"
```

**Note:** Direct database access bypasses qipu's abstraction layer. Use for read-only queries and advanced filtering only.

### Programmatic Integration

For language-specific integrations, you can:

1. **Shell out to qipu CLI:** Use subprocess/exec to call qipu commands
2. **Parse JSON output:** Use `--format json` for structured data
3. **Parse records output:** Use `--format records` for streaming/line-oriented parsing

Example (Python pseudocode):

```python
import subprocess
import json

def get_context(tag, max_chars=10000):
    result = subprocess.run(
        ['qipu', 'context', '--tag', tag, '--max-chars', str(max_chars), '--format', 'json'],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

def set_custom_field(note_id, key, value):
    subprocess.run(
        ['qipu', 'custom', 'set', note_id, key, str(value)],
        check=True
    )
```

### Error Handling

Qipu uses standard exit codes:
- `0`: Success
- `1`: General error
- `2`: Invalid arguments

With `--format json`, errors are wrapped in JSON envelopes:

```json
{
  "error": "Note not found: qp-invalid",
  "code": "not_found"
}
```

Parse stderr for human-readable error messages when not using JSON format.

## Best Practices

### 1. Separate Application State from Knowledge

- **Use qipu for:** Durable knowledge, research findings, documentation, decisions
- **Use custom metadata for:** Workflow states, application IDs, internal tracking
- **Use your own database for:** Ephemeral state, user sessions, runtime data

### 2. Design Custom Fields Carefully

- Document your custom field schema in your own docs
- Use consistent naming conventions
- Prefer flat structures over deep nesting
- Keep custom field values small (<10KB total per note)

### 3. LLM Context Budget Management

- Always set `--max-chars` to prevent context overflow
- Use MOCs to curate high-value content for specific tasks
- Leverage `--min-value` to filter for quality
- Use `--custom` flag selectively (only when LLM needs that metadata)

### 4. Provenance Tracking

Use qipu's built-in provenance fields for LLM-generated content:

```bash
qipu capture --generated_by "gpt-4" \
  --author "my-research-agent" \
  --prompt_hash "sha256:abc123..." \
  --verified false
```

This keeps generation metadata visible to LLMs (unlike custom fields), helping them understand content origins.

### 5. Validation

Run `qipu doctor` regularly to validate graph integrity:

```bash
# After bulk operations
qipu sync --validate

# Check custom metadata structure
qipu doctor --check custom
```

## Migration and Versioning

### Schema Evolution

Custom metadata is forward-compatible:
- New qipu versions will preserve unknown custom fields
- Your application owns the custom field schema
- Update your application's validation when changing custom fields

### Backward Compatibility

If your application's custom field schema changes:
- Add migration logic in your application
- Use `qipu list --format json` to enumerate all notes
- Update custom fields with `qipu custom set`
- Consider versioning your schema with a custom field like `schema_version`

## Anti-Patterns

❌ **Don't use custom metadata for core knowledge:** Use qipu's built-in fields instead
```bash
# Bad: Using custom field for quality
qipu custom set qp-123 quality 85

# Good: Use qipu's value field
qipu value set qp-123 85
```

❌ **Don't bypass qipu's abstractions for writes:** Use the CLI, not direct DB manipulation
```bash
# Bad: Direct database writes
sqlite3 .qipu/qipu.db "UPDATE notes SET ..."

# Good: Use qipu commands
qipu custom set qp-123 field value
```

❌ **Don't expose custom metadata to LLMs by default:** Use `--custom` flag selectively
```bash
# Bad: Always including custom fields
qipu context --tag api --custom

# Good: Include only when LLM needs it
qipu context --tag api  # No custom fields
qipu context --custom workflow_state=review --tag api  # Filter, but don't show in output
qipu context --tag api --custom  # Explicitly show custom fields when needed
```

❌ **Don't store large data in custom fields:** Link to external storage instead
```bash
# Bad: Store entire JSON blob
qipu custom set qp-123 analysis "$(cat 100kb-report.json)"

# Good: Store reference
qipu custom set qp-123 analysis_file "reports/2026-01-20-analysis.json"
```

## Further Reading

- `specs/custom-metadata.md` - Complete custom metadata specification
- `specs/llm-context.md` - LLM integration patterns and context bundle formats
- `specs/knowledge-model.md` - Qipu's knowledge graph semantics
- `docs/usage-patterns.md` - Common workflow patterns

## Support

Qipu is designed to be a foundation for building. If you find gaps in the extension mechanisms:

1. Check if qipu's core fields can address your need (they often can)
2. Consider whether custom metadata is the right fit
3. Open an issue to discuss new extension points

The goal is to keep qipu's core simple and universal while providing clean extension mechanisms for diverse use cases.
