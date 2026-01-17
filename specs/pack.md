# Pack (Dump/Load)

## Goals
- Provide a single-file, raw exchange format for sharing knowledge outside a project.
- Support dumping the full store or a specific slice of the graph.
- Allow round-trip loading back into a qipu store without lossy transformation.
- Keep the dump/load workflow aligned with existing note selection and traversal semantics.

## Non-goals
- Human-friendly publishing or narrative export (see `specs/export.md`).
- Redaction, filtering, or content rewriting during dump or load.

## Terminology
- **Pack**: a single-file artifact produced by `qipu dump` and consumed by `qipu load`.
- **Slice**: a subset of notes determined by selectors and/or graph traversal.

## Commands
### `qipu dump`
Creates a pack file from the store.

Selection options:
- **Note selectors** (same "language" as note-fetching commands such as `qipu context`):
  - `--note <id>` (repeatable)
  - `--tag <tag>`
  - `--moc <id>`
  - `--query <text>`
- **Graph traversal** (same options as `qipu link` traversal):
  - `--direction <out|in|both>`
  - `--max-hops <n>`
  - `--type <t>`
  - `--typed-only`
  - `--inline-only`

Defaults:
- If no selectors are provided, dump the full store.
- Attachments are included by default; `--no-attachments` disables them.

Traversal semantics:
- Traversal-based selection must be closed over the traversal rules: if a note is
  reached within `--max-hops`, it is included; no additional filtering removes
  intermediate notes discovered by traversal.
- Max hop limits are always respected.
- Link relationships among included notes are preserved.
- For selector-driven sets (e.g., `--tag`, `--query`) connectivity is not required.

### `qipu load <pack>`
Loads a pack file into a store.

Requirements:
- Restores notes, links, and attachments present in the pack.
- Does not transform or redact content.

## Pack contents (conceptual)
The pack is a single file that contains:
- notes (as defined in `specs/knowledge-model.md` and `specs/storage-format.md`)
- links between included notes
- attachments (optional; included by default)
- minimal metadata needed to reconstruct the slice

The concrete on-disk encoding is intentionally unspecified in this spec.

## Output formats
`qipu dump` produces a pack file, not a `--format` output stream. `--format` flags
do not alter the pack contents.

## Conflict Resolution

When loading a pack into a non-empty store, incoming notes may have IDs that already exist in the target store. The conflict resolution strategies are shared with workspace merging (see `specs/workspaces.md`).

### Strategies

`qipu load` accepts `--strategy <strategy>`:

1. **`skip`** (default):
   - If ID exists in target, keep target. Ignore incoming.
   - Good for "fill in missing gaps" without touching established notes.

2. **`overwrite`**:
   - If ID exists, replace target with incoming.
   - Good for "pack is the authority" workflows.

3. **`merge-links`**:
   - Keep target *content* (title/body).
   - Union the `links` arrays (add new typed links from incoming).
   - Good for "enrichment" packs that add connections.

### Graph Integrity

After loading, the target store should remain valid. Run `qipu doctor` to ensure no broken links were introduced.

## Pack Metadata

Pack files carry metadata in a header section, following the pattern established by workspace metadata and store configuration.

### Header Fields

| Field | Type | Description |
| --- | --- | --- |
| `version` | string | Pack format version (e.g., `"1.0"`) |
| `store_version` | u32 | Store format version from source store (for compatibility checking) |
| `created` | datetime | When the pack was created |
| `notes_count` | int | Number of notes in the pack |
| `links_count` | int | Number of links in the pack |
| `attachments_count` | int | Number of attachments in the pack |

### Compatibility

On `qipu load`, the pack's `store_version` should be checked against the target store's version:
- If pack version > target store version: warn or error (pack may use features not supported)
- If pack version < target store version: load normally (backward compatible)

## Non-goals

**Byte-stable determinism**: Packs are an internal exchange format. The contract is that `dump` followed by `load` produces correct store state per the conflict resolution strategy. The intermediate pack format is not guaranteed to be byte-identical across runs and should not be used for diffing, caching, or external consumption.
