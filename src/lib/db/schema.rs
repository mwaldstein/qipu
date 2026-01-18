//! SQLite database schema for qipu

use rusqlite::{Connection, Result};

const SCHEMA_SQL: &str = r#"
-- Note metadata (mirrors frontmatter)
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    type TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    created TEXT,
    updated TEXT,
    body TEXT,
    mtime INTEGER
);

-- Full-text search index with FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    body,
    tags,
    content=notes,
    content_rowid=rowid,
    tokenize='porter unicode61'
);

-- Tags (normalized)
CREATE TABLE IF NOT EXISTS tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

-- Links/edges (graph structure)
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT,
    inline INTEGER DEFAULT 0,
    PRIMARY KEY (source_id, target_id, link_type)
);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(link_type);

-- Unresolved links (for doctor)
CREATE TABLE IF NOT EXISTS unresolved (
    source_id TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    PRIMARY KEY (source_id, target_ref)
);

-- Index metadata
CREATE TABLE IF NOT EXISTS index_meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
"#;

pub fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}
