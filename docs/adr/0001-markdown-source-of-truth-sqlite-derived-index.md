# ADR 0001: Markdown Source Of Truth With SQLite Derived Index

## Status

Accepted

## Context

Qipu stores durable knowledge as Markdown files with YAML frontmatter. It also
needs fast search, backlink, graph traversal, and filtering operations.

Two alternatives keep recurring:

- Treat Markdown as the only runtime data source and scan/parse files on demand.
- Treat SQLite as authoritative and make Markdown an export format.

Both create poor tradeoffs. File scanning makes graph/search commands slow and
hard to keep deterministic at scale. Making SQLite authoritative weakens qipu's
git-friendly, human-readable storage model.

## Decision

Markdown files are the source of truth. SQLite is a derived operational index.

All note mutations update both the Markdown file and the SQLite index through
qipu. The database is transparent to users, gitignored by default, rebuildable,
validated on startup, and repaired or rebuilt when inconsistent.

SQLite is the only operational index. Qipu does not maintain a second fallback
index path for normal operation.

## Consequences

- Commands can rely on SQLite for search, backlinks, graph reads, and metadata
  queries.
- Storage and schema changes must preserve Markdown readability and rebuild
  behavior.
- Bugs in write-through consistency belong in Store/Database persistence code,
  not in individual commands.
- Direct reads or writes of `.qipu/qipu.db` are not a supported integration
  seam. Integrations should use the qipu CLI.

## References

- `specs/storage-format.md`
- `specs/operational-database.md`
- `docs/building-on-qipu.md`
