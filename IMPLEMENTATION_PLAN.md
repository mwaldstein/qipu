# Qipu Implementation Plan

## Status (Last Audited: 2026-01-18)
- Test baseline: `cargo test` passes (2026-01-18).
- Trust hierarchy: this plan is derived from code + tests; specs/docs are treated as hypotheses.
- All P1 correctness bugs completed (2026-01-18).

## Technology Choices

### Database: SQLite with rusqlite
- **Crate**: `rusqlite` with `bundled` feature (embeds SQLite)
- **Mode**: WAL (Write-Ahead Logging) for better concurrency
- **FTS**: FTS5 with porter tokenizer for English stemming

### Logging: tracing ecosystem ✅ IMPLEMENTED
- **Crates**: `tracing`, `tracing-subscriber` with `env-filter` and `json` features
- **Output**: Compact format by default, JSON via `--log-json`
- **CLI flags**: `--verbose`, `--log-level`, `--log-json`
- **Env var**: `QIPU_LOG` override
- **Init**: `src/lib/logging.rs`

Current instrumentation:
- `src/main.rs` - parse timing
- `src/commands/dispatch.rs` - command timing
- `src/commands/load/mod.rs` - load operations
- `src/commands/search.rs` - search method selection
- `src/lib/index/search.rs` - search method selection
- `src/lib/index/links.rs` - regex warnings
- `src/lib/db/mod.rs` - parse failures
- `src/lib/store/query.rs` - parse failures

**Remaining instrumentation (low priority):**
- Add spans to `Store::open()`, `Database::rebuild()`
- Add timing spans to graph traversal operations
- Add structured context to error chains

## P1: SQLite Migration & Ripgrep Removal (PRIORITY)

Per `specs/operational-database.md`, SQLite replaces both JSON cache and ripgrep. Ripgrep must be removed.

### Phase 1: Add SQLite Foundation ✅ COMPLETE
- [x] Add `rusqlite` dependency with bundled SQLite to `Cargo.toml`
- [x] Create database schema in `src/lib/db/schema.rs`
- [x] Implement `Database` struct with open/create/rebuild in `src/lib/db/mod.rs`
- [x] Implement FTS5 with porter tokenizer and BM25 ranking
- [x] Add database path at `.qipu/qipu.db`

#### Schema (Implemented)
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    type TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    created TEXT,
    updated TEXT,
    body TEXT,
    mtime INTEGER
);

CREATE VIRTUAL TABLE notes_fts USING fts5(
    title, body, tags,
    content=notes, content_rowid=rowid,
    tokenize='porter unicode61'
);

CREATE TABLE tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);

CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT,
    inline INTEGER DEFAULT 0,
    PRIMARY KEY (source_id, target_id, link_type)
);

CREATE TABLE unresolved (
    source_id TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    PRIMARY KEY (source_id, target_ref)
);

CREATE TABLE index_meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

#### Key SQL Patterns
```sql
-- FTS search with BM25 ranking (title 2.0x, body 1.0x, tags 1.5x)
SELECT n.id, n.title, bm25(notes_fts, 2.0, 1.0, 1.5) AS rank
FROM notes_fts JOIN notes n ON notes_fts.rowid = n.rowid
WHERE notes_fts MATCH ? ORDER BY rank LIMIT ?;

-- Backlinks
SELECT source_id, link_type, inline FROM edges WHERE target_id = ?;

-- Graph traversal (recursive CTE)
WITH RECURSIVE reachable(id, depth) AS (
    SELECT ?, 0
    UNION
    SELECT e.target_id, r.depth + 1
    FROM reachable r JOIN edges e ON e.source_id = r.id
    WHERE r.depth < ?
) SELECT DISTINCT id FROM reachable;
```

### Phase 2: Inline Updates (CURRENT)
- [x] Update `Store` to hold `Database` instance
- [x] Modify `create_note` to write file + insert into DB atomically
- [x] Modify `update_note` (edit) to update file + re-index in DB

**Remaining Phase 2 tasks:**

#### 2.1: Implement `Database::delete_note()` ✅ COMPLETE
File: `src/lib/db/mod.rs:337-363`

Implemented method to remove note from all tables:
- Deletes from edges (both source and target references)
- Deletes from unresolved
- Deletes from tags
- Deletes from notes (FTS handled via external content table)
- Returns `NoteNotFound` error if note doesn't exist

#### 2.2: Add `Store::delete_note()` method ✅ COMPLETE
File: `src/lib/store/lifecycle.rs:212-225`

Implemented method that removes file and updates DB:
- Gets note using `get_note()`
- Deletes file from filesystem
- Calls `db.delete_note()` to remove from database
- Returns error if note has no path or file deletion fails

#### 2.3: Wire up `link add/remove` to update DB ✅ COMPLETE
Files: `src/commands/link/add.rs`, `src/commands/link/remove.rs`

Both already call `store.save_note()` which now updates the DB via `insert_note()` and `insert_edges()`. The current implementation already works because:
- `save_note()` calls `db.insert_note()` and `db.insert_edges()` (src/lib/store/lifecycle.rs:169-170)
- `insert_edges()` deletes all existing edges before inserting new ones

**Blocker resolved**: `INSERT OR REPLACE` doesn't remove edges when links are deleted from frontmatter. It only replaces edges with matching primary keys, but if a link is removed and no new link has the same (source_id, target_id, link_type), the edge stays in the database.

**Fix applied**: `Database::insert_edges()` now deletes all existing edges for a note before inserting new ones. This ensures edges are removed when links are deleted from frontmatter.

**Root cause**: WAL mode in SQLite delays writes to the main database file. Tests using separate connections were seeing stale data because the WAL changes weren't checkpointed to disk before the test queries.

**Solution**: Added `pragma wal_checkpoint(TRUNCATE)` after edge insertion/deletion in `Database::insert_edges()` to force WAL changes to be written to disk. This ensures test queries see the latest state.

**Learning**: When using WAL mode with multiple connections, explicitly checkpoint after writes if subsequent reads need to see the changes immediately. Alternatively, tests should use the same connection for all operations.

**Verified with test**: `test_link_add_remove_updates_database` in `tests/cli/link/add_remove.rs` confirms edges are correctly added and removed.

### Phase 3: Migrate Queries to SQLite

#### 3.1: Migrate `search` command to FTS5 ✅ COMPLETE
File: `src/commands/search.rs`, `src/commands/index.rs`

Changes made:
- Removed `Index::load()` and `IndexBuilder` usage
- Replaced `search(store, &index, ...)` with `store.db().search(...)`
- Updated `index --rebuild` command to call `store.db().rebuild(store.root())`
- Added `Database::get_note_metadata()` method for fetching note metadata
- Added `Database::insert_note()` method for inserting/updating notes in database
- Updated FTS5 schema to use manual insertion (removed `content=` option)
- Updated `insert_note()` and `insert_note_internal()` to insert into both `notes` and `notes_fts` tables
- Updated search query to JOIN on `rowid` columns

**Migration steps completed:**
1. ✅ Removed `Index::load()` and `IndexBuilder::new(store).build()` calls
2. ✅ Replaced `search(store, &index, query, type_filter, tag_filter)` with `store.db().search(...)`
3. ✅ Updated code that references `index.metadata` or `index.get_metadata()` to use `store.db().get_note_metadata()`

**Known issues:**
- BM25 ranking with FTS5 has some test failures related to ranking order
- Tests `test_search_exact_tag_match_ranks_above_body`, `test_search_recency_boost`, and `test_search_title_match_ranks_above_body_match` are failing due to ranking issues
- The basic search functionality works (notes are found), but ranking order may not match expectations

**Learning:**
- FTS5 `content=` option requires source table to have `INTEGER PRIMARY KEY` with implicit `rowid` column
- Since our `notes.id` is `TEXT PRIMARY KEY`, there's no implicit `rowid`, so `content=` won't work
- Must use manual insertion into FTS5 table (INSERT INTO notes_fts(rowid, title, body, tags))
- Join must be on explicit `rowid` column: `JOIN notes n ON notes_fts.rowid = n.rowid`

#### 3.2: Add `Database::list_notes()` for metadata queries
File: `src/lib/db/mod.rs`

Add method for `list` command filters:
```rust
pub fn list_notes(
    &self,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    since: Option<chrono::DateTime<Utc>>,
) -> Result<Vec<NoteMetadata>> {
    let mut sql = String::from("SELECT n.id, n.title, n.type, n.path, n.created, n.updated FROM notes n");
    // Build WHERE clauses based on filters
    // JOIN tags if tag_filter is set
    ...
}
```

#### 3.3: Add `Database::get_backlinks()` for backlink lookup
File: `src/lib/db/mod.rs`

```rust
pub fn get_backlinks(&self, note_id: &str) -> Result<Vec<Edge>> {
    let mut stmt = self.conn.prepare(
        "SELECT source_id, link_type, inline FROM edges WHERE target_id = ?1"
    )?;
    ...
}
```

Used by: `qipu show --links`, `qipu link list`

#### 3.4: Add `Database::traverse()` for graph traversal
File: `src/lib/db/mod.rs`

```rust
pub fn traverse(
    &self,
    start_id: &str,
    direction: TraversalDirection,
    max_hops: u32,
    max_nodes: Option<usize>,
) -> Result<Vec<String>> {
    // Use recursive CTE per spec (specs/operational-database.md:137-148)
    let sql = match direction {
        TraversalDirection::Out => "WITH RECURSIVE reachable(id, depth) AS (...) SELECT DISTINCT id FROM reachable",
        TraversalDirection::In => "...",
        TraversalDirection::Both => "...",
    };
    ...
}
```

Used by: `qipu link tree`, `qipu link path`, `qipu context --moc`

#### 3.5: Migrate `doctor` checks to SQLite
File: `src/commands/doctor/checks.rs`

Replace file-scanning checks with DB queries:
- Orphaned notes: `SELECT id FROM notes WHERE id NOT IN (SELECT target_id FROM edges)`
- Broken links: Use `unresolved` table
- Duplicate IDs: `SELECT id, COUNT(*) FROM notes GROUP BY id HAVING COUNT(*) > 1`

#### 3.6: Migrate `context` note selection to SQLite
File: `src/commands/context/select.rs`

Replace `store.list_notes()` + in-memory filtering with DB queries.

### Phase 4: Remove Legacy Components

#### 4.1: Delete ripgrep integration
File: `src/lib/index/search.rs`

Delete entirely:
- Lines 13-48: `RipgrepMatch`, `RipgrepBeginData`, `RipgrepEndData`, `RipgrepMatchData`, `RipgrepText` structs
- Lines 80-87: `is_ripgrep_available()` function
- Lines 101-301: `search_with_ripgrep()` function
- Lines 303-429: `search_embedded()` function (replaced by DB search)
- Lines 431-450: `search()` function wrapper

After removal, `src/lib/index/search.rs` should only contain utility functions if any are still needed, or can be deleted entirely.

#### 4.2: Delete JSON cache code
Files to modify:
- `src/lib/index/mod.rs` - Remove `Index::load()`, `Index::save()`
- `src/lib/index/builder.rs` - Delete file entirely or gut JSON-building logic
- `src/lib/index/types.rs` - Keep `SearchResult`, `Edge`, `NoteMetadata`; remove JSON-specific fields

Delete:
- Any code creating `.cache/` directory
- `Index` struct JSON serialization (serde derives if only for caching)

#### 4.3: Update `index --rebuild` command
File: `src/commands/index.rs`

Change from:
```rust
IndexBuilder::new(store).build()?.save(&cache_dir)?;
```

To:
```rust
store.db().rebuild(store.root())?;
```

#### 4.4: Add migration from `.cache/`
File: `src/lib/store/mod.rs` or `src/lib/db/mod.rs`

On startup (in `Store::open` or `Database::open`):
```rust
let cache_dir = store_root.join(".cache");
if cache_dir.exists() {
    tracing::info!("Migrating from JSON cache to SQLite...");
    // DB rebuild happens automatically when qipu.db doesn't exist
    // After successful rebuild, delete .cache/
    std::fs::remove_dir_all(&cache_dir)?;
    tracing::info!("Migration complete, deleted .cache/");
}
```

#### 4.5: Update tests referencing ripgrep
File: `tests/cli/search.rs`

Test `test_search_title_only_match_included_with_ripgrep_results` will need renaming/updating since ripgrep no longer exists. The test validates title-only matches work - keep the test but update name/comments.

### Phase 5: Startup Validation

#### 5.1: Check if `qipu.db` exists on startup
File: `src/lib/db/mod.rs` in `Database::open()`

Already partially implemented - schema is created if missing. Need to add:
```rust
// Check if tables are empty (fresh DB vs existing)
let note_count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))?;
if note_count == 0 && store_has_notes(store_root) {
    // DB is empty but store has notes - trigger rebuild
    let db = Database { conn };
    db.rebuild(store_root)?;
}
```

#### 5.2: Quick consistency check
File: `src/lib/db/mod.rs`

Add method:
```rust
pub fn validate_consistency(&self, store_root: &Path) -> Result<bool> {
    // Count notes in DB
    let db_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))?;
    
    // Count files on filesystem
    let fs_count = count_note_files(store_root)?;
    
    if db_count != fs_count {
        return Ok(false);
    }
    
    // Sample a few mtimes
    let mut stmt = self.conn.prepare("SELECT path, mtime FROM notes ORDER BY RANDOM() LIMIT 5")?;
    // Compare against actual file mtimes
    ...
    
    Ok(true)
}
```

#### 5.3: Incremental repair
File: `src/lib/db/mod.rs`

```rust
pub fn incremental_repair(&self, store_root: &Path) -> Result<()> {
    // Find files changed since last sync
    // Re-parse and update changed entries
    // Remove entries for deleted files
    ...
}
```

#### 5.4: Handle schema version mismatch
File: `src/lib/db/schema.rs`

Add version tracking:
```rust
const SCHEMA_VERSION: i32 = 1;

pub fn create_schema(conn: &Connection) -> Result<()> {
    // Check existing version
    let current_version: Option<i32> = conn.query_row(
        "SELECT value FROM index_meta WHERE key = 'schema_version'",
        [], |r| r.get(0)
    ).ok();
    
    match current_version {
        None => { /* Fresh install - create tables */ }
        Some(v) if v < SCHEMA_VERSION => { /* Migration needed */ }
        Some(v) if v == SCHEMA_VERSION => { /* Up to date */ }
        Some(v) => { /* Future version - error or rebuild */ }
    }
}
```

---

## P2: Missing Test Coverage

### Completed
- [x] `--root` flag tests (specs/cli-tool.md)
- [x] `link tree/path` type filters and direction tests (specs/graph-traversal.md)
- [x] Search ranking tests (specs/indexing-search.md)
- [x] `doctor --duplicates` threshold test (specs/similarity-ranking.md)
- [x] MOC-driven export order test (specs/export.md)

### Remaining

#### `specs/provenance.md`
- [ ] Add CLI test for `--prompt-hash` via `create` or `capture`
  - Test: Create note with `--prompt-hash`, verify it appears in frontmatter
  - File: `tests/cli/capture.rs`

#### `specs/export.md`
- [ ] Add test validating anchor rewriting produces existing target anchor
  - Test: Export bundle with internal links, verify `#note-<id>` anchors exist
  - File: `tests/cli/export.rs`
- [ ] Add test validating `--with-attachments` link rewriting
  - Test: Export with attachments, verify `./attachments/` links resolve
  - File: `tests/cli/export.rs`

#### `specs/compaction.md`
- [ ] Add CLI tests for `compact apply`, `compact show`, `compact status`
  - File: `tests/cli/compact.rs`

#### `specs/structured-logging.md`
- [ ] Add tests for `--log-level`, `--log-json`, `QIPU_LOG` behavior
  - Test: Verify `--log-json` produces JSON output, `--log-level debug` shows debug messages
  - Test: Verify `QIPU_LOG=trace` overrides CLI flags
  - File: `tests/cli/logging.rs` (new)

#### `specs/llm-context.md`
- [ ] Add test for `--max-chars` / `--max-tokens` budget enforcement
  - File: `tests/cli/context.rs`
- [ ] Add test for `--transitive` nested MOC traversal
  - File: `tests/cli/context.rs`
- [ ] Add test for records safety banner (`W ...` line)
  - File: `tests/cli/context.rs`

#### `specs/pack.md`
- [ ] Add tests for dump traversal filters affecting reachability
  - File: `tests/cli/dump.rs`

---

## P3: Unimplemented Optional / Future

### `specs/similarity-ranking.md`
- [ ] Optional stemming (Porter) - no stemming code exists
- [ ] "Related notes" similarity expansion in `context` command

### `specs/llm-context.md`
- [ ] Backlinks-in-context (described as future)

### `specs/semantic-graph.md`
- [ ] Weighted traversal / per-edge hop costs

---

## P4: Spec Ambiguity / Spec Drift

### `specs/knowledge-model.md`
- [ ] Decide: note "type" closed enum vs arbitrary values
  - Current: strict enum (src/lib/note/types.rs:6-19)

### `specs/semantic-graph.md`
- [ ] Align link-type config schema
  - Spec: `[graph.types.*]`
  - Impl: `[links.inverses]` + `[links.descriptions]`

### `specs/records-output.md`
- [ ] Reconcile record prefix set (`H/N/S/E/B` vs `W/D/C/M`, `B-END`)

### `specs/graph-traversal.md` + `specs/semantic-graph.md`
- [ ] Clarify: semantic inversion in traversal vs presentation-only

### `specs/export.md`
- [ ] Clarify anchor rewriting behavior (explicit vs heading IDs)

---

## Closed Design Decisions (Specs Updated)

### `specs/storage-format.md`
- [x] MOCs use separate `mocs/` directory
- [x] Note paths are flat (no date partitioning)

### `specs/graph-traversal.md`
- [x] Default `--max-hops` is 3; no default `--max-nodes`
