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
- a MOC (`--moc`)
- a query (`--query`)

## Export modes
### 1) Bundle export (concatenate)
- Produces a single markdown file containing selected notes.
- Includes metadata headers per note.

### 2) Outline export (MOC-first)
- Uses a MOC as the outline.
- Follows the MOC's ordering.

### 3) Bibliography export
- Extracts `sources` from notes.
- Produces a markdown bibliography section (future: BibTeX/CSL JSON).

## Deterministic ordering
Exports must be deterministic:
- For MOC-driven exports: follow MOC ordering.
- For tag/query-driven exports: sort by `(created_at, id)`.

## Link handling
Options:
- preserve wiki links
- rewrite wiki links to markdown links
- rewrite note links to section anchors in the exported bundle

Keep defaults conservative to avoid rewriting user content unexpectedly.

## Attachments
- Optionally copy attachments into an export folder.
- Provide a "no attachments" mode (default) for lightweight exports.

## Open questions
- Should export support `pandoc` integration (optional) for PDF generation?
- Should export allow including transitive links (depth-limited)?
