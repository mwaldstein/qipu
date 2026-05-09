# Knowledge Model (Zettelkasten-inspired)

## Overview
Qipu uses a Zettelkasten-inspired model: many small, linkable notes that form a graph. It is optimized for:

- Capturing research while it is fresh
- Distilling research into durable insights
- Navigating by links, tags, and linked collection roots rather than deep folder hierarchies

Qipu should also borrow from beads' core idea: **agent memory works best when it is a graph with stable identifiers and deterministic queries**, stored and shared via git.

## Core concepts
- **Note (zettel)**: an atomic unit of knowledge stored as a markdown file.
- **Linked collection root**: a note whose outbound links define a curated collection, often an ordered reading path.
- **MOC (Map of Content)**: the standard ontology's built-in linked collection root note type.
- **Tag**: lightweight topical grouping.
- **Inline link**: human-facing link in the markdown body.
- **Typed link (edge)**: machine-friendly relationship with semantics (in metadata), inspired by beads dependency types.
- **Backlink**: a derived relationship (notes that link to a note).

## Note types
Qipu should support a small set of note "types" (stored in metadata) to guide workflows:

- **Fleeting**: quick capture; low ceremony; meant to be refined later.
- **Literature**: notes derived from an external source (URL, book, paper).
- **Permanent**: distilled insight, phrased in the author's own words, meant to stand alone.
- **MOC**: the default ontology's map/index note type. Replacement ontologies may use names like `outline`, `index`, `collection`, or `project-map`.

Type is a hint for tooling and templates; it should not impose rigid structure.

## Atomicity and quality bar
- Prefer "one idea per note".
- Notes should be understandable without hidden context (include short background when needed).
- Links should include *why* they exist (use a short phrase around links, not a bare list).

## IDs, titles, and filenames
### Why IDs matter (beads parallel)
Beads uses collision-resistant IDs because multiple agents/branches create data concurrently. Qipu should adopt the same posture: **IDs must be stable and collision-resistant under parallel creation**.

### Recommended default
- **ID**: `qp-<hash>` with adaptive length (e.g., `qp-a1b2`, `qp-f14c3`, `qp-3e7a5b`)
  - length should grow as the store grows, to keep collision probability negligible
- **Filename**: `<id>-<slug(title)>.md` (slug is lowercase, hyphenated)

Example:
- `qp-a1b2-zettelkasten-note-types.md`

Notes:
- A ULID/timestamp-based scheme can be supported as an alternate mode, but the default should be safe for multi-agent, multi-branch creation.

## Tags
- Tags are short, stable labels: `zettelkasten`, `llm-tools`, `paper`, `rust`.
- Prefer a small number of tags per note; use linked collection roots for deeper structure.
- Tooling may support tag aliases, but the on-disk representation should remain simple.

## Typed links (graph semantics)
Inline links are great for humans, but agents benefit from explicit semantics.

Qipu should support **typed links** in note metadata. These are inspired by beads' "dependency types": not every relationship is the same.

Proposed minimal link types:
- `related` (soft relationship)
- `derived-from` (note created because of another note or source; beads analog: `discovered-from`)
- `supports` (evidence supports a claim)
- `contradicts` (evidence contradicts a claim)
- `part-of` (note is part of a larger outline or linked collection)

Typed links should:
- be optional (qipu remains usable with plain markdown)
- be derivable/inspectable via `qipu link` and `qipu show --links`
- enable deterministic navigation (e.g., "show all evidence that supports claim X")

## Linked collection roots
Linked collection roots are a primary navigation mechanism and are the closest analog to beads' hierarchical epics: they provide **curation and ordering**.

A linked collection root typically:
- Starts with a brief "what belongs here" description
- Groups links by subtopic
- Provides an ordered reading path

Linked collection roots can serve as:
- A project knowledge index
- A whitepaper outline
- A curated "context set" for LLM tools

## Knowledge lifecycle (capture -> distill)
Suggested flow:
1. **Capture** fleeting/literature notes during research
2. **Triage**: tag, link, and attach to relevant linked collection roots
3. **Distill**: create permanent notes for durable insights
4. **Connect**: add links between notes (inline + typed)
5. **Promote**: convert stabilized insights into specs/tickets when they become commitments

This lifecycle is qipu's version of beads' "ready queue": it makes the backlog of raw research visible, and it encourages distillation into durable, low-context artifacts.

## Design Decisions

**Note types are ontology-defined strings.** The standard ontology provides
`fleeting`, `literature`, `permanent`, and `moc`, and custom ontology modes can
extend or replace that set. Link types follow the same ontology mechanism.
Legacy CLI names such as `--moc` continue to work as linked collection root
selectors, but new usage should prefer the neutral `--collection-root` alias.
This remains true when replacement ontologies use domain-specific terms like
`outline` or `index`.

## Open questions
- Which typed link set is the minimal useful set?
- Should qipu support duplicate/near-duplicate detection and merge (beads analog: `bd duplicates`/`bd merge`)?
