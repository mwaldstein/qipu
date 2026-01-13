# Graph Retrieval and Traversal

Status: Draft  
Last updated: 2026-01-12

## Motivation
Qipu’s knowledge store is a graph. For both humans and LLM tools, it must be easy to “walk the tree” (i.e., traverse the graph from one or more starting notes) to gather the set of related notes needed to achieve a goal.

The primary consumer for traversal is an LLM workflow:
- the model needs deterministic, bounded expansions
- the model benefits from both a human-readable view (debuggability) and a structured JSON view (tool integration)

## Principles (beads-aligned)
- **Graph-first**: traversal operates on an explicit note graph.
- **Deterministic**: same inputs yield the same traversal ordering and output.
- **Bounded**: traversal must have depth and/or size budgets.
- **Cycle-safe**: graphs may contain cycles; traversal must not loop.
- **Composable**: traversal results should be usable as inputs to other commands (`qipu context`, `qipu export`).

## What is the graph?
A note graph is defined by an **effective edge set** computed from two sources:

1. **Typed links (explicit edges)**
   - Stored in note frontmatter (`links[]`).
   - Semantically meaningful (e.g., `supports`, `derived-from`).

2. **Inline links (implicit edges)**
   - Extracted from note markdown bodies (wiki links + markdown links).
   - Primarily for human authoring.

### Inline link handling
Because inline links are common in markdown authoring and the primary consumer is an LLM, qipu must be able to traverse inline links.

Two acceptable implementations:
- **Index-time extraction (preferred)**: `qipu index` extracts inline links into the derived graph cache.
- **Materialization (optional)**: qipu may optionally rewrite/materialize extracted inline links into `links[]` (opt-in), reducing work at traversal time.

Inline links that lack explicit semantics are treated as:
- `type = related`
- `source = inline`

This preserves the distinction between an intentionally typed relationship and a “plain markdown” link.

## Traversal semantics
Traversal is a projection of a graph into a readable “tree view”. Because the underlying structure is a graph:
- nodes may be reachable via multiple paths
- cycles may exist

Traversal must therefore define:
- directionality
- edge inclusion rules
- de-duplication rules
- cycle handling
- output ordering

### Start set
The minimal traversal start set is a single note ID/path:
- `qipu link tree <id-or-path>`

(Extending to multiple start nodes is allowed as a future feature.)

### Direction
Traversal direction determines which edges are followed.

Supported values:
- `out`: follow links from a note to its neighbors
- `in`: follow backlinks (notes that link to the current note)
- `both`: follow both directions

Default: `both`.

### Edge inclusion
Traversal includes, by default:
- typed links
- inline links (as `related`, `source=inline`)

The user should be able to restrict traversal via flags:
- typed-only
- inline-only
- include/exclude by typed link `type`

### Depth and size limits
Traversal must be bounded. At minimum:
- `--max-hops <n>` (recommended default: 3)

Optional additional controls:
- `--max-nodes <n>` overall cap on visited nodes
- `--max-edges <n>` overall cap on edges emitted
- `--max-fanout <n>` cap neighbors per expanded node (prevents blow-ups)

When limits are hit, output must explicitly report truncation.

### Ordering (determinism)
Traversal must be deterministic.

Recommended ordering rules:
- Neighbors are sorted by:
  1. edge `type` (stable lexical order)
  2. target note `id`

Traversal algorithm recommendation:
- Use a deterministic BFS to compute first-discovery predecessors (a spanning tree)
- Render the spanning tree as the primary tree view

### De-duplication and cycles
Traversal maintains a visited set by note `id`.

- A note is expanded at most once.
- If an edge points to an already-visited node, the output should:
  - include the node as a reference (e.g., “(seen)”)
  - not recursively expand it again

This produces a stable “tree walk” output even on cyclic graphs.

## CLI surface
Traversal lives under `qipu link`, mirroring beads’ `bd dep tree` ergonomics.

### `qipu link tree <id-or-path>`
Show a traversal tree rooted at the given note.

Flags (proposed):
- `--direction <out|in|both>` (default: `both`)
- `--max-hops <n>` (default: `3`)
- `--type <t>` (repeatable) / `--types <csv>` (include only these typed link types)
- `--exclude-type <t>` / `--exclude-types <csv>`
- `--typed-only` (exclude inline links)
- `--inline-only` (exclude typed links)
- `--max-nodes <n>` (optional)

Output:
- Default: human-readable tree (optimized for scanning)
- With `--format json`: structured traversal result
- With `--format records`: line-oriented record output (see `specs/records-output.md`)

### `qipu link path <from> <to>` (recommended)
Find a path between two notes.

Use cases:
- Explain why two notes are related
- Find the shortest chain of evidence/support

Flags (proposed):
- `--direction <out|in|both>` (default: `both`)
- `--typed-only` / `--inline-only`
- `--type/--exclude-type` filters
- `--max-hops <n>`

Output:
- Default: a simple path listing
- With `--format json`: list of nodes and edges in the chosen path
- With `--format records`: line-oriented record output (see `specs/records-output.md`)

## JSON output (shape)
`qipu link tree --format json` should return a single JSON object.

Proposed minimal shape:
```json
{
  "root": "qp-a1b2",
  "direction": "both",
  "max_hops": 3,
  "truncated": false,
  "nodes": [
    {"id": "qp-a1b2", "title": "…", "type": "…", "tags": ["…"], "path": "…"}
  ],
  "edges": [
    {"from": "qp-a1b2", "to": "qp-f14c3", "type": "related", "source": "inline"},
    {"from": "qp-a1b2", "to": "qp-3e7a", "type": "supports", "source": "typed"}
  ],
  "spanning_tree": [
    {"from": "qp-a1b2", "to": "qp-f14c3", "hop": 1}
  ]
}
```

Notes:
- `edges[]` represents the effective edge set encountered during traversal.
- `spanning_tree[]` encodes the deterministic “tree view” projection (via first-discovery predecessor edges).

## Integration with `qipu context`
Traversal results should compose cleanly into context bundles:
- users/tools can traverse with `qipu link tree --format json`, select a subset of `nodes[]`, and then call `qipu context --note …`

Future-friendly extension (optional):
- add `qipu context --walk <id> --max-hops <n> ...` to perform traversal-and-bundle in one command.

## Open questions
- Default limits: should the default `--max-hops` be 2 or 3? Should there be a default `--max-nodes`?
- Should qipu ever materialize inline links into `links[]` automatically, or only as an explicit opt-in?
- Do we want additional first-class traversal queries beyond `tree` and `path` (e.g., `neighbors`, `subgraph`, `cycles`)?
