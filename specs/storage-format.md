# Storage Format and Repository Layout

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
- Mirrors beads' `.beads/` pattern for "agent memory lives here".
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
  qipu.db            # derived local index (gitignored)
```

Notes:
- `notes/` contains all non-MOC notes.
- `mocs/` contains map-of-content notes.
- `attachments/` contains optional binaries (images, PDFs).
- `qipu.db` is derived and should be gitignored by default.

## "Stealth mode" (local-only store)
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
- `.qipu/qipu.db` accelerates search/backlinks/graph queries.
- All derived data must be rebuildable (`qipu index --rebuild`).

## Git integration
Recommended defaults (beads-inspired: portable data in git, local acceleration out of git):

- Commit:
  - `.qipu/notes/`
  - `.qipu/mocs/`
  - `.qipu/config.toml` (optional)
- Ignore:
  - `.qipu/qipu.db`

Provide `qipu doctor` checks for:
- duplicate IDs
- broken links
- invalid frontmatter

## Design Decisions

**MOCs use a separate directory.** MOCs live in `mocs/` rather than inside `notes/` with a type flag. This provides clear visual separation in the filesystem, simplifies glob patterns for tools, and matches the conceptual distinction between content notes and structural notes. The `type: moc` frontmatter field remains authoritative; the directory is a convention that aids discoverability.

**Note paths are flat.** Notes are stored directly in `notes/` without date partitioning. This keeps paths stable (no moves when dates change), simplifies ID-to-path resolution, and avoids empty directory hierarchies. For stores with thousands of notes, the SQLite index handles listing/filtering efficiently.

## Store discovery

When no explicit `--store` path is provided, qipu searches for an existing store by walking up the directory tree from the current working directory (or `--root` if specified).

### Discovery algorithm

1. Starting from the search root, check for `.qipu/` or `qipu/` directories
2. If found, use that store
3. If not found, move to the parent directory and repeat
4. Stop searching when reaching a **project boundary** or the filesystem root

### Project boundaries

To prevent accidental discovery of unrelated stores in parent directories, the search stops after passing a project root marker. Project markers include:

- `.git/` — Git repository root
- `.hg/` — Mercurial repository root
- `.svn/` — Subversion working copy
- `Cargo.toml` — Rust project
- `package.json` — Node.js project
- `go.mod` — Go module
- `pyproject.toml` — Python project

The search checks the current directory for a store first, then checks for project markers. If a project marker is found, qipu will not search above that directory.

### Rationale

- **Security**: Prevents malicious stores in shared parent directories from being used
- **Predictability**: Users won't accidentally use a store from an unrelated project
- **Test isolation**: Stray stores in `/tmp` or `/home` won't pollute test runs
- **Explicit over implicit**: If a store exists above a project root, use `--store` to reference it explicitly

### Examples

```
/home/user/
  .qipu/                    # Personal store (won't be found from project)
  projects/
    myapp/
      .git/                 # Project boundary - search stops here
      .qipu/                # Project store - will be found
      src/
        main.rs             # Search starts here
```

Running `qipu list` from `src/` will find `myapp/.qipu/` and stop. It will not traverse above `.git/` to find `/home/user/.qipu/`.

## Open questions
- Should attachments be per-note folders (`attachments/<id>/...`)?
