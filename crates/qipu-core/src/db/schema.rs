//! SQLite database schema for qipu

use rusqlite::{Connection, Result};
use std::sync::atomic::{AtomicI32, Ordering};

pub const CURRENT_SCHEMA_VERSION: i32 = 8;

static GLOBAL_SCHEMA_VERSION: AtomicI32 = AtomicI32::new(CURRENT_SCHEMA_VERSION);

pub fn get_schema_version() -> i32 {
    GLOBAL_SCHEMA_VERSION.load(Ordering::SeqCst)
}

/// Result of schema creation - indicates whether database needs rebuild
#[derive(Debug, PartialEq, Eq)]
pub enum SchemaCreateResult {
    /// Schema created/updated successfully, no rebuild needed
    Ok,
    /// Schema was recreated from scratch, rebuild needed
    NeedsRebuild,
}

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
    mtime INTEGER,
    value INTEGER DEFAULT 50,
    compacts TEXT DEFAULT '[]',
    author TEXT,
    verified INTEGER,
    source TEXT,
    sources TEXT DEFAULT '[]',
    generated_by TEXT,
    prompt_hash TEXT,
    custom_json TEXT DEFAULT '{}',
    index_level INTEGER DEFAULT 2
);
CREATE INDEX IF NOT EXISTS idx_notes_value ON notes(value);
CREATE INDEX IF NOT EXISTS idx_notes_custom ON notes(json_extract(custom_json, '$'));

 -- Full-text search index with FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    body,
    tags,
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
    position INTEGER NOT NULL DEFAULT 0,
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

fn drop_all_tables(conn: &Connection) -> Result<()> {
    conn.execute("DROP TABLE IF EXISTS notes", [])?;
    conn.execute("DROP TABLE IF EXISTS notes_fts", [])?;
    conn.execute("DROP TABLE IF EXISTS tags", [])?;
    conn.execute("DROP TABLE IF EXISTS edges", [])?;
    conn.execute("DROP TABLE IF EXISTS unresolved", [])?;
    conn.execute("DROP TABLE IF EXISTS index_meta", [])?;
    Ok(())
}

pub fn create_schema(conn: &Connection) -> Result<SchemaCreateResult> {
    let current_version: Option<i32> = conn
        .query_row(
            "SELECT value FROM index_meta WHERE key = 'schema_version'",
            [],
            |r| r.get::<_, String>(0).map(|s| s.parse().unwrap_or(0)),
        )
        .ok();

    let target_version = get_schema_version();

    let result = match current_version {
        None => {
            conn.execute_batch(SCHEMA_SQL)?;
            conn.execute(
                "INSERT INTO index_meta (key, value) VALUES ('schema_version', ?1)",
                [&target_version.to_string()],
            )?;
            SchemaCreateResult::Ok
        }
        Some(v) if v < target_version => {
            if v == 1 && target_version == 2 {
                conn.execute("ALTER TABLE notes ADD COLUMN value INTEGER DEFAULT 50", [])?;
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_notes_value ON notes(value)",
                    [],
                )?;
                conn.execute(
                    "UPDATE index_meta SET value = ?1 WHERE key = 'schema_version'",
                    [&target_version.to_string()],
                )?;
                SchemaCreateResult::Ok
            } else if v == 6 && target_version == 7 {
                conn.execute(
                    "ALTER TABLE notes ADD COLUMN index_level INTEGER DEFAULT 2",
                    [],
                )?;
                conn.execute(
                    "UPDATE index_meta SET value = ?1 WHERE key = 'schema_version'",
                    [&target_version.to_string()],
                )?;
                tracing::info!(
                    "Database schema updated from version {} to {}",
                    v,
                    target_version
                );
                SchemaCreateResult::Ok
            } else if v == 7 && target_version == 8 {
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_notes_custom ON notes(json_extract(custom_json, '$'))",
                    [],
                )?;
                conn.execute(
                    "UPDATE index_meta SET value = ?1 WHERE key = 'schema_version'",
                    [&target_version.to_string()],
                )?;
                tracing::info!(
                    "Database schema updated from version {} to {}",
                    v,
                    target_version
                );
                SchemaCreateResult::Ok
            } else {
                drop_all_tables(conn)?;
                conn.execute_batch(SCHEMA_SQL)?;
                conn.execute(
                    "INSERT INTO index_meta (key, value) VALUES ('schema_version', ?1)",
                    [&target_version.to_string()],
                )?;
                tracing::info!(
                    "Database schema updated from version {} to {}",
                    v,
                    target_version
                );
                SchemaCreateResult::NeedsRebuild
            }
        }
        Some(v) if v == target_version => SchemaCreateResult::Ok,
        Some(v) => {
            drop_all_tables(conn)?;
            conn.execute_batch(SCHEMA_SQL)?;
            conn.execute(
                "INSERT INTO index_meta (key, value) VALUES ('schema_version', ?1)",
                [&target_version.to_string()],
            )?;
            tracing::info!(
                "Database schema updated from version {} to {}",
                v,
                target_version
            );
            SchemaCreateResult::NeedsRebuild
        }
    };

    Ok(result)
}

#[cfg(test)]
pub fn force_set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO index_meta (key, value) VALUES ('schema_version', ?1)",
        [&version.to_string()],
    )?;
    Ok(())
}
