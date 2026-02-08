# Custom Note Metadata

## Motivation

Qipu is designed as a knowledge graph tool for LLM agents. As applications build on top of qipu, they need domain-specific metadata beyond qipu's core schema—user agreement signals, workflow states, integration identifiers, and more.

Currently, qipu's `NoteFrontmatter` uses fixed fields with `serde`; unknown YAML keys are silently ignored. This forces downstream tools into awkward workarounds:

1. **Separate metadata layer**: Maintain a parallel database keyed by note ID (fragile joins, split data).
2. **Abuse existing fields**: Encode domain data in tags or note body (breaks semantics).
3. **Fork qipu**: Add hard-coded fields per use case (unmaintainable).

None of these serve qipu's goal of being a strong foundation for LLM tooling.

### Design Principle: Open for Extension, Closed for Core Semantics

Qipu already follows this pattern for **link types**: a standard ontology with user extensibility via config. Custom metadata applies the same principle to **note attributes**.

**Core fields** (`id`, `title`, `type`, `value`, `verified`, etc.) retain strict semantics—they're what qipu traversal, context, and doctor commands understand. **Custom fields** are preserved, indexed, and queryable, but qipu doesn't interpret their meaning.

## Schema Design

### Core vs. Custom Fields

```yaml
---
# === Core fields (qipu-defined semantics) ===
id: qp-a1b2
title: "Raft Consensus Overview"
type: permanent
value: 85
verified: true
tags: [distributed-systems, consensus]
links:
  - type: supports
    id: qp-f14c

# === Custom fields (application-defined) ===
custom:
  alignment: disagree      # Blibio: user's agreement with content
  workflow_state: review   # Internal: editorial pipeline stage
  blibio_id: "sub-7x9k"    # Blibio: submission tracking ID
---
```

### Why a `custom` Namespace?

1. **Collision prevention**: Future qipu versions can add fields without breaking user data.
2. **Clear contract**: Tools know exactly which fields are theirs vs. qipu's.
3. **LLM clarity**: Context output can distinguish "qipu metadata" from "application metadata."
4. **Flat access**: Within `custom`, keys are arbitrary strings with arbitrary values.

### Alternative Considered: Flat Extension

```yaml
---
id: qp-a1b2
alignment: disagree  # Risk: future qipu version adds 'alignment' field
---
```

Rejected because:
- No namespace isolation
- Unknown fields indistinguishable from typos
- Harder to extract "all custom metadata" for downstream tools

## Technical Implementation

### Frontmatter Extension

```rust
// src/lib/note/frontmatter.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFrontmatter {
    // ... existing core fields ...

    /// Custom metadata for downstream applications
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_yaml::Value>,
}
```

The `serde_yaml::Value` type preserves arbitrary YAML: strings, numbers, booleans, arrays, nested objects.

### SQLite Index Extension

Custom fields are stored as JSON in a dedicated column for filtering:

```sql
-- Schema addition (requires version bump)
ALTER TABLE notes ADD COLUMN custom_json TEXT DEFAULT '{}';

-- Index for JSON extraction (SQLite 3.38+)
CREATE INDEX idx_notes_custom ON notes(json_extract(custom_json, '$'));
```

Individual custom fields are queryable via SQLite's JSON functions:

```sql
-- Find all notes where custom.alignment = 'disagree'
SELECT id FROM notes 
WHERE json_extract(custom_json, '$.alignment') = 'disagree';
```

### Indexing Behavior

When indexing a note:
1. Parse `custom` block from frontmatter.
2. Serialize to JSON.
3. Store in `custom_json` column.

Custom fields are **preserved** through round-trips but qipu does not validate their structure.

## CLI Integration

### Setting Custom Fields

```bash
# Set a single custom field
qipu custom set <id> <key> <value>

# Examples
qipu custom set qp-a1b2 alignment disagree
qipu custom set qp-a1b2 priority 1
qipu custom set qp-a1b2 tags '["imported", "v2"]'  # JSON for complex values
```

#### Type Detection

Since qipu doesn't enforce schemas for custom metadata, the CLI automatically detects value types using YAML/JSON parsing:

**Automatic type detection:**
```bash
# Numbers (integers and floats)
qipu custom set qp-a1b2 priority 1           # → int: 1
qipu custom set qp-a1b2 score 3.14           # → float: 3.14
qipu custom set qp-a1b2 count -5             # → int: -5

# Booleans
qipu custom set qp-a1b2 active true          # → bool: true
qipu custom set qp-a1b2 reviewed false       # → bool: false

# Strings (unquoted for simple strings)
qipu custom set qp-a1b2 alignment disagree   # → string: "disagree"
qipu custom set qp-a1b2 status in-progress   # → string: "in-progress"

# Null
qipu custom set qp-a1b2 reviewer null        # → null
```

#### Negative values

`qipu custom set` must accept negative numeric values (and other leading-hyphen strings) in the `<value>` positional without requiring the `--` end-of-options delimiter.

Example:

```bash
qipu custom set qp-a1b2 alignment -100
```

**Complex types via YAML/JSON:**
```bash
# Arrays
qipu custom set qp-a1b2 tags '[1, 2, 3]'              # → array: [1, 2, 3]
qipu custom set qp-a1b2 labels '["imported", "v2"]'   # → array: ["imported", "v2"]

# Objects
qipu custom set qp-a1b2 metadata '{"version": 2, "source": "import"}'  # → object

# Mixed arrays
qipu custom set qp-a1b2 mixed '[1, "two", true]'      # → array: [1, "two", true]
```

**Forcing string type when ambiguous:**

When you need to store a value as a string that would otherwise be parsed as a different type, use YAML/JSON string quoting:

```bash
# Force "1" to be stored as string "1" instead of int 1
qipu custom set qp-a1b2 code '"001"'         # → string: "001"
qipu custom set qp-a1b2 flag '"true"'        # → string: "true"
qipu custom set qp-a1b2 value '"null"'       # → string: "null"

# The outer quotes are for the shell, inner quotes force YAML string parsing
```

**Implementation note:** Values are parsed using `serde_yaml::from_str()`. If parsing fails or produces a string, the value is stored as a string. This provides intuitive behavior for most use cases while allowing explicit type control when needed.

### Reading Custom Fields

```bash
# Show all custom fields for a note
qipu custom show <id>

# Output:
# qp-a1b2:
#   alignment: disagree
#   workflow_state: review

# Get a specific field
qipu custom get <id> <key>

# Output:
# disagree
```

### Filtering by Custom Fields

```bash
# List notes with a specific custom value
qipu list --custom alignment=disagree

# Combine with core filters
qipu list --tag consensus --custom workflow_state=review --min-value 50

# Context with custom filter
qipu context --custom-filter alignment=agree --max-chars 8000
```

#### Filter expressions (minimal)

Custom filtering is intentionally minimal:

- Equality: `key=value`
- Existence: `key` (present), `!key` (absent)
- Numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`
- Date comparisons: `key>YYYY-MM-DD`, `key>=YYYY-MM-DD`, `key<YYYY-MM-DD`, `key<=YYYY-MM-DD`

Date values are compared lexicographically (ISO-8601 format sorts correctly). Store dates as strings in `YYYY-MM-DD` format for filtering to work.

Multiple `--custom-filter` flags are combined with AND semantics.

#### Using custom filters as selection (context)

`qipu context` must allow `--custom-filter` to participate in selection.

Examples:

```bash
# Select notes only by custom metadata
qipu context --custom-filter alignment=disagree --max-chars 8000

# Combine custom filters with other selectors
qipu context --tag consensus --custom-filter workflow_state=review --max-chars 8000
```

### Removing Custom Fields

```bash
# Remove a custom field
qipu custom unset <id> <key>
```

## Context Output Integration

Custom metadata appears in context bundles, clearly separated from core metadata:

### Markdown Format

```markdown
## Note: Raft Consensus Overview (qp-a1b2)
Path: notes/qp-a1b2-raft-consensus.md
Type: permanent
Value: 85
Tags: distributed-systems, consensus
Custom:
  alignment: disagree
  workflow_state: review

---
[note content]
---
```

### JSON Format

```json
{
  "notes": [
    {
      "id": "qp-a1b2",
      "title": "Raft Consensus Overview",
      "type": "permanent",
      "value": 85,
      "custom": {
        "alignment": "disagree",
        "workflow_state": "review"
      },
      "content": "..."
    }
  ]
}
```

### Records Format

Custom fields are included in the header block, prefixed with `Custom.`:

```
=== qp-a1b2: Raft Consensus Overview ===
Type: permanent
Value: 85
Custom.alignment: disagree
Custom.workflow_state: review

[content]
```

## Traversal and Value Model

Custom fields are **orthogonal to traversal** by default. They do not affect:
- Edge costs (that's `value`'s domain)
- Semantic inversion (that's link types' domain)
- Hop limits or budget allocation

### Future: Traversal Hooks

Applications that need custom fields to affect traversal can do so by:
1. Filtering post-traversal: `qipu link tree qp-a1b2 | filter-by-custom`
2. Pre-filtering the working set: `qipu context --custom-filter active=true`

If demand emerges, a future spec could add traversal cost modifiers based on custom fields, but this is out of scope for v1.

## Validation and Doctor

`qipu doctor` does not validate custom field values (they're application-defined), but it can check structural integrity:

```bash
qipu doctor --check custom

# Checks:
# - custom field is valid YAML mapping (not scalar/array at top level)
# - warns on very large custom blocks (>10KB) that might bloat index
```

## Migration Path

### Existing Notes

Notes without `custom` blocks continue to work unchanged. The field defaults to an empty map.

### Adoption Pattern

Applications building on qipu:
1. Define their custom field schema in their own documentation.
2. Use `qipu custom set` or direct frontmatter editing.
3. Query via `qipu list --custom` or SQLite for advanced use cases.

## Use Case: Blibio Alignment

The motivating use case from Blibio:

```yaml
---
id: qp-a1b2
title: "Article: Why X is Wrong"
type: literature
value: 75                    # Quality of the source
source: "https://example.com/article"
custom:
  alignment: disagree        # User disagrees with the content
  blibio_submission: "sub-7x9k"
---
```

Blibio can now:
- Filter: `qipu list --custom alignment=disagree`
- Build context: `qipu context --custom-filter alignment=agree` (select only agreed-with sources)
- Maintain the linkage: Blibio's own DB stores `sub-7x9k` details; qipu stores the cross-reference

This avoids the "separate metadata layer" problem while keeping qipu's core schema clean.

## Visibility and Discoverability

Custom metadata is an **integration feature**, not part of qipu's core LLM interface. LLM agents should use qipu's standard fields (`value`, `verified`, `tags`, `links`) for knowledge management. Custom fields are for applications that embed qipu as a storage layer.

### Hidden from Default Help

Custom commands are excluded from standard help output:

```bash
qipu --help          # Does NOT list 'custom' subcommand
qipu custom --help   # Works, but must be explicitly invoked
```

Implementation: The `custom` subcommand is registered with a `hidden = true` flag (clap's `#[command(hide = true)]`).

### Excluded from Prime Output

`qipu prime` provides LLM session primers. Custom metadata is **not mentioned** in prime output—LLMs don't need to know it exists.

### Opt-in for Context Output

By default, `qipu context` **excludes** custom fields from output to keep the LLM-facing context clean:

```bash
qipu context --note qp-a1b2           # Custom fields omitted
qipu context --note qp-a1b2 --custom  # Custom fields included (opt-in)
```

Applications that need custom fields in context use `--custom` explicitly. This prevents LLMs from seeing (and potentially hallucinating about) application-specific metadata.

### Opt-in for `show` JSON Output

When using `qipu show --format json`, custom metadata is omitted by default.

To include it, use:

```bash
qipu show <note-id> --format json --custom
```

This provides a CLI-only way for integrations to fetch a single note along with its application metadata, without relying on qipu's on-disk storage details.

### CLI Disclaimer

When invoked directly, the custom command displays a brief disclaimer:

```bash
$ qipu custom set qp-a1b2 alignment disagree

Note: Custom metadata is for applications building on qipu.
For standard note management, use 'qipu value', 'qipu tag', or 'qipu link'.

Set qp-a1b2 custom.alignment = disagree
```

The disclaimer can be suppressed with `--quiet` or `-q` for scripted use.

### Documentation Separation

Custom metadata is documented in a separate spec (this file) rather than in primary user-facing docs (README, CLI help). Applications building on qipu reference this spec; casual users never encounter it.

## Design Decisions

**Custom fields use a namespace.** The `custom:` block provides collision-free extensibility. Alternative (flat serde_flatten) was rejected for clarity and forward-compatibility.

**Custom fields are stored as JSON.** SQLite's JSON functions enable filtering without schema changes per application. This is more flexible than promoted columns.

**Custom fields don't affect traversal.** This maintains qipu's core promise: graph semantics are well-defined. Applications layer their own logic on top.

**No custom field schemas.** Qipu doesn't enforce types or allowed values. Applications define and validate their own contracts. This keeps qipu simple and universal.

## Open Questions

1. **Bulk operations**: Should `qipu custom set-all --tag foo key value` set a custom field on all matching notes?

2. **Export filtering**: Should `qipu export` support including/excluding custom fields? (e.g., `--strip-custom` for sharing without internal metadata)

3. **Reserved prefixes**: Should qipu reserve `qp_*` or `_*` for future internal use within the custom namespace?
