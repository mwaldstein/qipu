# Export and Publishing

## Goals
Export should make qipu notes usable outside the qipu store, especially for:
- writing white papers and design docs
- sharing research sets with an LLM tool
- archiving a project's knowledge base

Beads has "compaction" (memory decay) to keep agent context small. In qipu, export is one of the primary ways to produce condensed artifacts (bundles/outlines/digests) without requiring an LLM.

## Export inputs
Exports can be driven by:
- explicit notes (`--note`)
- a tag (`--tag`)
- a linked collection root (`--moc`; legacy name from the standard `moc` type)
- a query (`--query`)

## Export modes
### 1) Bundle export (concatenate)
- Produces a single markdown file containing selected notes.
- Includes metadata headers per note.

### 2) Outline export (linked-root first)
- Uses the `--moc` root note as the outline.
- Follows the root note's linked-note ordering.
- The root note does not need literal `type: moc`; custom ontology users may
  use domain-specific outline/index/root note types.

### 3) Bibliography export
- Extracts `sources` from notes.
- Produces a markdown bibliography section (future: BibTeX/CSL JSON).

## Deterministic ordering
Exports must be deterministic:
- For `--moc` driven exports: follow linked-root ordering.
- For tag/query-driven exports: sort by `(created_at, id)`.

## Link handling
Options:
- preserve wiki links
- rewrite wiki links to markdown links
- rewrite note links to section anchors in the exported bundle

Keep defaults conservative to avoid rewriting user content unexpectedly.

### Anchor rewriting details
When using anchor mode, note links are rewritten to explicit section anchors:
- Each note gets an HTML anchor element: `<a id="note-<note-id>"></a>` (placed before the note header)
- Links to notes are rewritten to point to these anchors: `#note-<note-id>`
- Anchor format is explicit (`#note-<note-id>`), not derived from heading text
- This ensures deterministic linking regardless of note title changes

## Attachments
- Optionally copy attachments into an export folder.
- Provide a "no attachments" mode (default) for lightweight exports.

## Open questions
- Should export support `pandoc` integration (optional) for PDF generation?
- Should export allow including transitive links (depth-limited)?
