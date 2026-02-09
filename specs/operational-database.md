# Operational Database

## Scope
This spec defines the operational database requirements for qipu's index and query operations. The database is a derived cache that accelerates operations; Markdown files with YAML frontmatter remain the source of truth.

## Design Principles

1. **Markdown is the source of truth** - All note content and metadata lives in Markdown files with YAML frontmatter. These are git-tracked and human-readable.

2. **Database is the only index** - A single database replaces all other indexing mechanisms. There is no fallback mode.

3. **Fully transparent** - Users have no control over database operations. All mutations (create, update, delete) automatically update both the source files AND the database atomically.

4. **Always consistent** - The database is kept in sync with source files. On startup, qipu validates consistency and repairs if needed.

## Database Location

```
.qipu/
  qipu.db              # Operational database (gitignored)
```

The database is:
- Created automatically on first qipu operation
- Updated incrementally on every note mutation
- Validated on startup; rebuilt automatically if corrupt or missing
- Rebuilt via `qipu index --rebuild` (force full rebuild)

## Data Model

### Notes Table
Stores note metadata (mirrors frontmatter):
- `id` - Note identifier (primary key)
- `title` - Note title
- `type` - Note type (fleeting, literature, permanent, moc)
- `path` - File path (unique)
- `created` - Creation timestamp (ISO 8601)
- `updated` - Last update timestamp (ISO 8601)
- `body` - Full note body for search
- `mtime` - File modification time for incremental sync

### Full-Text Search Index
Enables fast text search across notes with field weighting:
- Title: 2.0x boost
- Tags: 1.5x boost  
- Body: 1.0x (baseline)

### Tags Table
Normalized tag storage:
- `note_id` - Reference to note
- `tag` - Tag value

### Edges Table
Graph structure (links between notes):
- `source_id` - Source note
- `target_id` - Target note
- `link_type` - Type of link (NULL for inline links)
- `inline` - Whether discovered from body vs frontmatter

### Unresolved Links Table
Tracks broken references for `doctor`:
- `source_id` - Note containing the broken link
- `target_ref` - The unresolved reference

### Metadata Table
Key-value store for index metadata (schema version, etc.)

## Operations

### Search
Full-text search with BM25 ranking. Returns results ordered by relevance with field weighting applied.

### Backlinks
Query all notes that link to a given note. Returns source notes with link type and inline flag.

### Graph Traversal
Multi-hop traversal from a starting note. Supports:
- Direction: outbound, inbound, or both
- Depth limit (max hops)
- Node limit

### Tag Queries
- Find all notes with a specific tag
- Get tag frequency statistics

## Sync Strategy

### Inline Updates (Primary Path)
All note mutations update both source files and database atomically:
- `qipu create` / `qipu capture` - insert note + index
- `qipu edit` - update note + re-index
- `qipu delete` - remove file + remove from index
- `qipu link add/remove` - update file + update edges

### Startup Validation
On every qipu invocation:
1. Check if database exists
2. If missing, trigger full rebuild
3. If exists, quick consistency check:
   - Compare note count in DB vs filesystem
   - Sample a few file mtimes
4. If inconsistent, trigger incremental repair

### Incremental Repair
When external changes are detected (e.g., git pull, manual edits):
- Find files changed since last sync
- Re-parse and update changed entries
- Remove entries for deleted files

### Full Rebuild
`qipu index --rebuild`:
1. Delete existing database
2. Create fresh database with schema
3. Scan all note files
4. Populate all tables
5. Build search index

## CLI Integration

### All Commands Use Database
Every command uses the database transparently:
- `qipu search` - Full-text queries
- `qipu list` - Metadata queries  
- `qipu link list/tree/path` - Graph queries
- `qipu context` - Combined queries
- `qipu doctor` - Validation queries
- `qipu create/capture/edit` - Write file + update DB
- `qipu link add/remove` - Write file + update edges

### Database Management Commands

```bash
# Rebuild database from source files (force)
qipu index --rebuild

# Check database health (part of doctor)
qipu doctor
```

No user-facing flags for database control. The database is an implementation detail.

## Performance Expectations

Target performance (2000 notes):
- Search: <50ms
- List with filters: <20ms
- Backlink lookup: <10ms
- Graph traversal (3 hops): <100ms

## Error Handling

### Database Corruption
If database operations fail:
1. Log error with details
2. Attempt to delete and rebuild automatically
3. If rebuild fails, exit with error (no silent fallback)
4. User can manually delete database and retry

### Schema Mismatch
If schema version doesn't match current code:
1. Attempt migration if migration path exists
2. Otherwise, delete and rebuild automatically
3. Log: "Database schema updated"

### Concurrent Access
The database should handle concurrent reads well. For writes:
- Use appropriate journaling mode for concurrency
- Single-writer model (qipu operations are fast)
- No explicit locking needed for typical usage

## Open Questions

- ~~Should search include attachment content (PDFs, etc.)?~~ **Resolved**: No - attachment content is not indexed. Search covers note title, body, and tags only. Rationale: avoid scope creep, reduce attack surface from binary parsing, maintain performance. Users can describe attachments in note body for searchability.
- Should we track query statistics for optimization?
- Should `qipu doctor` report database size/stats?
