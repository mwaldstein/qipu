# Pack (Dump/Load)

Status: Draft  
Last updated: 2026-01-12

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

## Open questions
- Do we require deterministic, byte-stable packs for identical inputs?
- What are the merge/conflict rules when loading into a non-empty store?
- Should packs carry explicit schema/version markers and integrity hashes?
