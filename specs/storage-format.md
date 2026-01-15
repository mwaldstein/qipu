# Storage Format and Repository Layout

Status: Draft  
Last updated: 2026-01-12

## Design constraints
- Files must be readable and editable without qipu.
- Git diffs should stay clean.
- Notes are the source of truth; caches/indexes are derived.
- Layout should be easy for LLM tools to ingest.
- Multi-agent/multi-branch workflows must not create ID collisions.

## Store root
Default (repo-local) store root (beads-aligned):
- `.qipu/`

Rationale:
- Mirrors beads’ `.beads/` pattern for “agent memory lives here”.
- Keeps repo root tidy while remaining git-trackable.

Config should allow alternate roots:
- `qipu/` (visible)
- Custom path via `--store <path>` or config

## Directory structure (proposed)
```
.qipu/
  config.toml
  notes/
  mocs/
  attachments/
  templates/
  qipu.db            # optional, derived local index (gitignored)
  .cache/            # derived; safe to delete
    index_meta.json
    metadata.json
    tags.json
    edges.json
    unresolved.json
    files.json
    id_to_path.json
```

Notes:
- `notes/` contains all non-MOC notes.
- `mocs/` contains map-of-content notes.
- `attachments/` contains optional binaries (images, PDFs).
- `.cache/` and `qipu.db` are derived and should be gitignored by default.

## “Stealth mode” (local-only store)
Beads supports `bd init --stealth` for private per-repo tracking. Qipu should support an analogous mode.

In stealth mode:
- `.qipu/` is created but added to `.gitignore` (or stored outside the repo)
- the user can keep private research notes without changing the shared repo

## Protected branch workflow (optional)
Beads supports committing metadata to a separate branch for protected `main`. Qipu should support an analogous workflow for teams that want it:

- Store qipu notes on a dedicated branch (e.g., `qipu-metadata`)
- Optionally automate via `qipu sync` (details in CLI spec)

This should be opt-in and must not be required for normal operation.

## Config
A repo-local config file is stored at `.qipu/config.toml`.

Minimum fields:
- store format version
- default note type
- id scheme (`hash` | `ulid` | `timestamp`)
- editor preference override

Config must have sensible defaults so `qipu init` is optional.

## Note file format
Notes are markdown with YAML frontmatter.

Example:
```markdown
---
id: qp-a1b2
title: Zettelkasten note types
type: permanent
created: 2026-01-12T13:07:00Z
updated: 2026-01-12T13:20:00Z
tags:
  - zettelkasten
  - qipu
sources:
  - url: https://example.com/article
    title: Example Article
    accessed: 2026-01-12
links:
  - type: derived-from
    id: qp-f14c
  - type: related
    id: qp-3e7a
---

## Summary
One-paragraph summary for fast scanning.

## Notes
…

## Links
- Related: [[qp-3e7a]]
```

Rules:
- Frontmatter keys should be stable and minimal.
- `title` and `id` are required.
- `updated` may be omitted if unknown.
- The `links` array is optional; inline links remain valid without it.

## Linking syntax
Qipu should accept both:

- **Wiki-style**: `[[<id>]]` and `[[<id>|label]]`
- **Markdown links**: `[label](relative/path/to/note.md)`

Canonicalization:
- `qipu index` may rewrite wiki-links into markdown links (optional; opt-in).
- Tools should be able to resolve wiki-links without rewriting.

## Attachments
- Stored under `.qipu/attachments/`.
- Prefer referencing by relative markdown links.
- Avoid embedding huge binaries by default; provide guidance.

## Derived files / caches
- `.qipu/qipu.db` (optional) accelerates search/backlinks/graph queries.
- `.qipu/.cache/` can store JSON indexes for interoperability.
- All derived data must be rebuildable (`qipu index --rebuild`).
- The absence of caches must not break core workflows.

## Git integration
Recommended defaults (beads-inspired: portable data in git, local acceleration out of git):

- Commit:
  - `.qipu/notes/`
  - `.qipu/mocs/`
  - `.qipu/config.toml` (optional)
- Ignore:
  - `.qipu/qipu.db`
  - `.qipu/.cache/`

Provide `qipu doctor` checks for:
- duplicate IDs
- broken links
- invalid frontmatter

## Open questions
- Should `mocs/` live inside `notes/` with a type flag instead?
- Should note paths be flat or date-partitioned (`notes/2026/01/...`)?
- Should attachments be per-note folders (`attachments/<id>/...`)?
