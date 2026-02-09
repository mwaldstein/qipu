//! SQLite database schema for qipu

use rusqlite::{Connection, Result};
use std::sync::atomic::{AtomicI32, Ordering};

pub const CURRENT_SCHEMA_VERSION: i32 = 9;

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
-- Indexing checkpoints for batched indexing with resume support
CREATE TABLE IF NOT EXISTS indexing_checkpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_number INTEGER NOT NULL,
    last_note_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    completed_at TEXT
);

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
    conn.execute("DROP TABLE IF EXISTS indexing_checkpoints", [])?;
    Ok(())
}

fn create_fresh_schema(conn: &Connection, target_version: i32) -> Result<SchemaCreateResult> {
    conn.execute_batch(SCHEMA_SQL)?;
    conn.execute(
        "INSERT INTO index_meta (key, value) VALUES ('schema_version', ?1)",
        [&target_version.to_string()],
    )?;
    Ok(SchemaCreateResult::Ok)
}

fn recreate_schema(
    conn: &Connection,
    target_version: i32,
    from_version: i32,
) -> Result<SchemaCreateResult> {
    drop_all_tables(conn)?;
    conn.execute_batch(SCHEMA_SQL)?;
    conn.execute(
        "INSERT INTO index_meta (key, value) VALUES ('schema_version', ?1)",
        [&target_version.to_string()],
    )?;
    tracing::info!(
        "Database schema updated from version {} to {}",
        from_version,
        target_version
    );
    Ok(SchemaCreateResult::NeedsRebuild)
}

fn update_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "UPDATE index_meta SET value = ?1 WHERE key = 'schema_version'",
        [&version.to_string()],
    )?;
    Ok(())
}

fn migrate_v1_to_v2(conn: &Connection) -> Result<()> {
    conn.execute("ALTER TABLE notes ADD COLUMN value INTEGER DEFAULT 50", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_value ON notes(value)",
        [],
    )?;
    Ok(())
}

fn migrate_v6_to_v7(conn: &Connection, target_version: i32) -> Result<()> {
    conn.execute(
        "ALTER TABLE notes ADD COLUMN index_level INTEGER DEFAULT 2",
        [],
    )?;
    update_schema_version(conn, target_version)?;
    tracing::info!(
        "Database schema updated from version 6 to {}",
        target_version
    );
    Ok(())
}

fn migrate_v7_to_v8(conn: &Connection, target_version: i32) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_custom ON notes(json_extract(custom_json, '$'))",
        [],
    )?;
    update_schema_version(conn, target_version)?;
    tracing::info!(
        "Database schema updated from version 7 to {}",
        target_version
    );
    Ok(())
}

fn migrate_v8_to_v9(conn: &Connection, target_version: i32) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS indexing_checkpoints (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            batch_number INTEGER NOT NULL,
            last_note_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            completed_at TEXT
        )",
        [],
    )?;
    update_schema_version(conn, target_version)?;
    tracing::info!(
        "Database schema updated from version 8 to {} (added indexing_checkpoints)",
        target_version
    );
    Ok(())
}

fn apply_migration(conn: &Connection, current: i32, target: i32) -> Result<SchemaCreateResult> {
    match (current, target) {
        (1, 2) => {
            migrate_v1_to_v2(conn)?;
            update_schema_version(conn, target)?;
            Ok(SchemaCreateResult::Ok)
        }
        (6, 7) => {
            migrate_v6_to_v7(conn, target)?;
            Ok(SchemaCreateResult::Ok)
        }
        (7, 8) => {
            migrate_v7_to_v8(conn, target)?;
            Ok(SchemaCreateResult::Ok)
        }
        (8, 9) => {
            migrate_v8_to_v9(conn, target)?;
            Ok(SchemaCreateResult::Ok)
        }
        _ => recreate_schema(conn, target, current),
    }
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
        None => create_fresh_schema(conn, target_version)?,
        Some(v) if v < target_version => apply_migration(conn, v, target_version)?,
        Some(v) if v == target_version => SchemaCreateResult::Ok,
        Some(v) => recreate_schema(conn, target_version, v)?,
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
