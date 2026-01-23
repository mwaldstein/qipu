# Building on Top of Qipu

This guide is for developers building applications, agents, and tools that use qipu as a knowledge graph foundation.

Up to date as of qipu version `0.1.100`.

## Overview

Qipu is designed as a **knowledge graph tool for LLM agents**. It provides:
- A structured, traversable graph of notes with semantic link types
- Clean, deterministic context bundles for LLM prompts
- Custom metadata support for application-specific extensions
- CLI-first interface suitable for agent tool integration

This document covers how to integrate qipu into your application, particularly focusing on LLM-based systems.

Integration contract: Treat the qipu store (files + database) as an internal implementation detail. Do not read or write qipu markdown files or the SQLite database directly. Use the `qipu` CLI as the only supported interface; storage layout, file formats, and database schema may change.

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

Note: `qipu prime` is internally size-bounded, but does not currently accept `--max-chars` / `--max-tokens` flags.

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
- Default format (`--format human`) is human-friendly markdown for direct LLM injection

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

### 4. Editing Knowledge

Allow your application to modify existing knowledge:

```bash
# Programmatic/LLM update (non-interactive)
# Updates metadata and optionally replaces body from stdin atomically
qipu update qp-oauth-research --tag reviewed --value 90
echo "New body content" | qipu update qp-oauth-research --title "Revised Title"

# Open an existing note in system editor (interactive)
# Updates both the file and search index atomically
qipu edit qp-oauth-research

# Use a specific editor
qipu edit qp-oauth-research --editor "code --wait"
```

**For programmatic integration and LLMs, prefer `qipu update` over `qipu edit`**. The `update` command is non-interactive, atomic, and suitable for script-based workflows.

### 5. Graph Navigation

Traverse the knowledge graph programmatically:

```bash
# Find related notes via links
qipu link tree qp-oauth-research --max-hops 2 --format json

# Find connection path between concepts
qipu link path qp-auth-basics qp-token-validation --format json

# List all outbound links
qipu link list qp-api-design --direction out --format json
```

## Custom Metadata for Application Extensions

Custom metadata allows you to layer application-specific data onto qipu's knowledge graph without forking the core schema.

### Design Philosophy

**Custom metadata is for applications, not for LLMs.**

- Qipu's core fields (`value`, `verified`, `tags`, `links`) are designed for LLM-driven knowledge management
- Custom fields are for your application's internal tracking, workflow states, and integrations
- By default, custom fields are **hidden from LLM context** to keep prompts clean

**Custom is intentionally non-discoverable.**

- `qipu --help` does not list `qipu custom` on purpose.
- Your wrapper/application should already know which custom keys it uses; do not rely on discovery.
- If an LLM should use custom metadata, teach it explicitly via your system prompt (examples below).

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
qipu context --moc qp-sprint-23 --custom-filter assignee="alice@example.com"
```

### Including Custom Metadata in Output

By default, custom fields are **excluded** from `qipu context` and `qipu show` output. To include them:

```bash
# Include custom in context bundles
qipu context --note qp-a1b2 --custom

# Include custom when viewing a single note
qipu show qp-a1b2 --custom --format json
```

**JSON Output Structure:**

```json
{
  "id": "qp-a1b2",
  "title": "API Rate Limiting Strategy",
  "value": 85,
  "custom": {
    "workflow_state": "review",
    "priority": 1,
    "assignee": "alice@example.com"
  },
  "content": "..."
}
```

This opt-in design prevents LLMs from seeing (and potentially hallucinating about) your application's internal metadata.

**Note:** `qipu show --format json` always includes the `value` field (when set), but custom metadata requires the explicit `--custom` flag.

## Teaching LLMs About Custom Properties

If your application needs the LLM to understand and work with custom properties, extend your system prompt with the schema and the exact commands to use.

Principle: the wrapper owns the schema. The LLM should not invent new custom keys.

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
  qipu context --tag <topic> --custom-filter alignment=agree

To find sources you disagree with (for counter-argument research):
  qipu context --tag <topic> --custom-filter alignment=disagree

This allows you to maintain high-quality sources in your knowledge graph
even when you disagree with their conclusions.
```

### Prompting Best Practices

1. **Be explicit about the schema:** Define what each custom field means and what values are valid
2. **Show examples:** Demonstrate both setting and querying custom fields
3. **Explain the boundary:** Make it clear that custom fields don't affect core qipu semantics
4. **Provide defaults:** If fields should have default values, specify them
5. **Mention opt-in visibility:** Remind the LLM that custom fields require `--custom` flag in context output
6. **No discovery:** Tell the LLM not to use `qipu --help` to discover custom keys/commands; your prompt is the source of truth

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
qipu context --tag economics --custom-filter alignment=agree --max-chars 15000

# Find counter-arguments (sources you disagree with)
qipu context --tag economics --custom-filter alignment=disagree --max-chars 8000
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
qipu capture --type permanent --generated-by "claude-3.5-sonnet" \
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
qipu context --custom-filter stage=published --tag auth --format human
```

## Storage and Deployment Options

Qipu is designed to work with git-based storage by default. For integrations, prefer explicit store targeting and CLI-only access.

### Store Location Options

#### Default: Repository-Local Store

By default, qipu creates a `.qipu/` directory in your repository:

```bash
qipu init
# Creates a qipu store in the current project
```

This mirrors the beads pattern and is ideal for:
- Team collaboration (commit notes to git)
- Project-specific knowledge bases
- Documentation alongside code

#### Visible Store

Use `qipu/` instead of `.qipu/` for more prominent visibility:

```bash
qipu init --visible
# Uses qipu/ instead of .qipu/
```

Useful for:
- Documentation-first projects where notes are primary content
- Making the knowledge base more discoverable to team members

#### Stealth Mode (Git-Ignored)

Create a local-only store that's automatically gitignored:

```bash
qipu init --stealth
# Creates a store and adds it to .gitignore
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

 Note: Treat store configuration files as internal implementation details; configure branch workflows via `qipu init --branch ...` and `qipu sync --commit/--push`.

**Use cases:**
- Main branch requires PR reviews, but notes need frequent updates
- Separating knowledge commits from code commits
- Different access policies for code vs. documentation

### Alternative Storage Backends

For applications that need to bypass git entirely, you can still use qipu by pointing `--store` at an application-managed directory. The directory contents are not a stable API; only qipu CLI behavior is supported.

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

### Configuration

Configure stores via `qipu init` flags and qipu commands. Avoid depending on the on-disk config format.

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
    qipu --root ~/projects/$1 "${@:2}"
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

1. **Local and fast:** typical listing/search should feel instant
2. **Index rebuilds:** `qipu index --rebuild` scales linearly with note count
3. **Search is indexed:** results are fast after indexing

**Recommendations:**
- Keep individual notes small (< 100KB)
- Use `--format json` for programmatic consumption (avoids terminal formatting overhead)
- Batch operations when possible (bulk import, then single index rebuild)

## Advanced Integration

### Programmatic Integration

For language-specific integrations, you can:

1. **Shell out to qipu CLI:** Use subprocess/exec to call qipu commands
2. **Parse JSON output:** Use `--format json` for structured data
3. **Parse records output:** Use `--format records` for streaming/line-oriented parsing

Operational guidance for integrations:
- Assume stdout must be parseable (JSON/records). Treat stderr as the only channel for logs.
- Always pass an explicit log level in wrappers (e.g. `--log-level error`) rather than relying on implicit defaults.
- ANSI should be auto-disabled when qipu is not writing to a TTY; do not rely on colored output for integration.

Implementation note (qipu `0.1.100`): not every subcommand currently honors `--format json`. If you need reliable machine-readable output today, prefer `qipu context --format records` / `qipu context --format json` and treat other commands as human-oriented until they are made consistent.

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
- `3`: Data/store error (missing store, invalid note data, not found)

With `--format json`, errors are returned as JSON envelopes on stdout:

```json
{"error":{"code":3,"message":"note not found: qp-invalid","type":"note_not_found"}}
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
qipu capture --generated-by "gpt-4" \
  --author "my-research-agent" \
  --prompt-hash "sha256:abc123..." \
  --verified false
```

This keeps generation metadata visible to LLMs (unlike custom fields), helping them understand content origins.

### 5. Validation

Run `qipu doctor` regularly to validate graph integrity:

```bash
# After bulk operations
qipu sync --validate

# Check custom metadata structure
qipu doctor
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

❌ **Don't bypass qipu's abstractions for writes:** Use the CLI only (no direct writes to store files or databases)
```bash
# Good: Use qipu commands
qipu custom set qp-123 field value
```

❌ **Don't expose custom metadata to LLMs by default:** Use `--custom` flag selectively
```bash
# Bad: Always including custom fields
qipu context --tag api --custom

# Good: Include only when LLM needs it
qipu context --tag api  # No custom fields
qipu context --custom-filter workflow_state=review --tag api  # Filter, custom fields still hidden
qipu context --custom-filter workflow_state=review --tag api --custom  # Filter and show custom fields
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
