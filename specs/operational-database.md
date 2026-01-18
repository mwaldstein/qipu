# Operational Database (SQLite)

## Scope
This spec defines how qipu uses SQLite as the sole operational database for all index and query operations, while maintaining Markdown files with YAML frontmatter as the durable, git-tracked source of truth.

## Design Principles

1. **Markdown is the source of truth** - All note content and metadata lives in Markdown files with YAML frontmatter. These are git-tracked and human-readable.

2. **SQLite is the only index** - The database replaces both the legacy JSON cache (`.cache/*.json`) and ripgrep-based search. There is no fallback mode.

3. **Fully transparent** - Users have no control over database operations. All mutations (create, update, delete) automatically update both the source files AND the database in a single operation.

4. **Always consistent** - The database is kept in sync with source files. On startup, qipu validates consistency and repairs if needed.

## Database Location

```
.qipu/
  qipu.db              # SQLite operational database (gitignored)
```

Removed (no longer used):
- `.cache/` directory and all JSON index files
- ripgrep integration

The database is:
- Created automatically on first qipu operation
- Updated incrementally on every note mutation
- Validated on startup; rebuilt automatically if corrupt or missing
- Rebuilt via `qipu index --rebuild` (force full rebuild)

## Schema

### Core Tables

```sql
-- Note metadata (mirrors frontmatter)
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    type TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    created TEXT,           -- ISO 8601
    updated TEXT,           -- ISO 8601
    body TEXT,              -- Full note body for FTS
    mtime INTEGER           -- File modification time for incremental sync
);

-- Full-text search index
CREATE VIRTUAL TABLE notes_fts USING fts5(
    id,
    title,
    body,
    tags,
    content=notes,
    content_rowid=rowid
);

-- Tags (normalized)
CREATE TABLE tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);
CREATE INDEX idx_tags_tag ON tags(tag);

-- Links/edges (graph structure)
CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT,          -- NULL for inline links, otherwise typed
    inline INTEGER DEFAULT 0, -- 1 if discovered from body, 0 if from frontmatter
    PRIMARY KEY (source_id, target_id, link_type)
);
CREATE INDEX idx_edges_target ON edges(target_id);
CREATE INDEX idx_edges_type ON edges(link_type);

-- Unresolved links (for doctor)
CREATE TABLE unresolved (
    source_id TEXT NOT NULL,
    target_ref TEXT NOT NULL, -- The unresolved reference
    PRIMARY KEY (source_id, target_ref)
);

-- Index metadata
CREATE TABLE index_meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

### FTS5 Configuration

Use FTS5 with porter tokenizer for English stemming:

```sql
CREATE VIRTUAL TABLE notes_fts USING fts5(
    title,
    body,
    tags,
    tokenize='porter unicode61'
);
```

Ranking uses BM25 with field weights:
- Title: 2.0x boost
- Tags: 1.5x boost  
- Body: 1.0x (baseline)

## Operations

### Search

```sql
-- Full-text search with ranking
SELECT n.id, n.title, n.path, n.type,
       bm25(notes_fts, 2.0, 1.0, 1.5) AS rank
FROM notes_fts
JOIN notes n ON notes_fts.rowid = n.rowid
WHERE notes_fts MATCH ?
ORDER BY rank
LIMIT ?;
```

### Backlinks

```sql
-- Find all notes that link to a given note
SELECT source_id, link_type, inline
FROM edges
WHERE target_id = ?;
```

### Graph Traversal

```sql
-- BFS traversal via recursive CTE
WITH RECURSIVE reachable(id, depth) AS (
    SELECT ?, 0
    UNION
    SELECT e.target_id, r.depth + 1
    FROM reachable r
    JOIN edges e ON e.source_id = r.id
    WHERE r.depth < ?
)
SELECT DISTINCT id FROM reachable;
```

### Tag Queries

```sql
-- Notes with specific tag
SELECT n.* FROM notes n
JOIN tags t ON t.note_id = n.id
WHERE t.tag = ?;

-- Tag frequency
SELECT tag, COUNT(*) as count
FROM tags
GROUP BY tag
ORDER BY count DESC;
```

## Sync Strategy

### Inline Updates (Primary Path)

All note mutations update both source files and database atomically:

```rust
// Pseudocode for note creation
fn create_note(store: &Store, note: Note) -> Result<Note> {
    // 1. Write markdown file
    let path = write_note_file(&note)?;
    
    // 2. Update database in same operation
    store.db.insert_note(&note, &path)?;
    store.db.update_fts(&note)?;
    store.db.insert_tags(&note)?;
    store.db.insert_edges(&note)?;
    
    Ok(note)
}
```

This applies to:
- `qipu create` / `qipu capture` - insert note + index
- `qipu edit` - update note + re-index
- `qipu delete` - remove file + remove from index
- `qipu link add/remove` - update file + update edges table

### Startup Validation

On every qipu invocation:
1. Check if `qipu.db` exists
2. If missing, trigger full rebuild
3. If exists, quick consistency check:
   - Compare note count in DB vs filesystem
   - Sample a few file mtimes
4. If inconsistent, trigger incremental repair

### Incremental Repair

When external changes are detected (e.g., git pull, manual edits):

```sql
-- Find files changed since last sync
SELECT path, mtime FROM notes;
-- Compare against filesystem mtimes
-- Re-parse and update changed entries
```

### Full Rebuild

`qipu index --rebuild`:
1. Delete existing `qipu.db`
2. Create fresh database with schema
3. Scan all note files
4. Populate all tables
5. Build FTS index

## Migration from Legacy

### Removed Components

The following are removed entirely:

1. **JSON cache** (`.cache/*.json`)
   - Delete: `metadata.json`, `tags.json`, `edges.json`, `files.json`, etc.
   - Remove all code that reads/writes these files
   - Remove `.cache/` directory creation

2. **Ripgrep integration**
   - Delete: `search_with_ripgrep()` function
   - Remove ripgrep JSON parsing structs
   - Remove ripgrep availability checks

3. **Fallback modes**
   - No `--no-db` flag
   - No JSON-based search fallback
   - SQLite is required for qipu to function

### Migration Path

On first run after upgrade:
1. Detect if `.cache/` exists with old index files
2. Build new SQLite database from source files (not from cache)
3. Delete `.cache/` directory
4. Log: "Migrated to SQLite database"

## CLI Integration

### All Commands Use Database

Every command uses the database transparently:
- `qipu search` - FTS5 queries
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

SQLite should be 10-100x faster than JSON-based operations for:
- Full-text search
- Complex filtering
- Graph queries

## Error Handling

### Database Corruption

If database operations fail:
1. Log error with details
2. Attempt to delete and rebuild automatically
3. If rebuild fails, exit with error (no silent fallback)
4. User can manually delete `qipu.db` and retry

### Schema Mismatch

If schema version doesn't match current code:
1. Attempt migration if migration path exists
2. Otherwise, delete and rebuild automatically
3. Log: "Database schema updated"

### Concurrent Access

SQLite handles concurrent reads well. For writes:
- Use WAL mode for better concurrency
- Single-writer model (qipu operations are fast)
- No explicit locking needed for typical usage

## Open Questions

- Should FTS include attachment content (PDFs, etc.)?
- Should we track query statistics for optimization?
- Should `qipu doctor` report database size/stats?
