//! Database validation methods

use crate::lib::error::{QipuError, Result};
use std::path::Path;

impl super::Database {
    /// Find duplicate note IDs in the database
    pub fn get_duplicate_ids(&self) -> Result<Vec<(String, Vec<String>)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, GROUP_CONCAT(path, ', ') as paths
                 FROM notes
                 GROUP BY id
                 HAVING COUNT(*) > 1",
            )
            .map_err(|e| QipuError::Other(format!("failed to prepare duplicate query: {}", e)))?;

        let duplicates = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let paths_str: String = row.get(1)?;
                let paths: Vec<String> = paths_str.split(", ").map(|s| s.to_string()).collect();
                Ok((id, paths))
            })
            .map_err(|e| QipuError::Other(format!("failed to query duplicates: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read duplicate rows: {}", e)))?;

        Ok(duplicates)
    }

    /// Get all broken links from the unresolved table
    pub fn get_broken_links(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, target_ref FROM unresolved")
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare broken links query: {}", e))
            })?;

        let broken_links = stmt
            .query_map([], |row| {
                let source_id: String = row.get(0)?;
                let target_ref: String = row.get(1)?;
                Ok((source_id, target_ref))
            })
            .map_err(|e| QipuError::Other(format!("failed to query broken links: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read broken link rows: {}", e)))?;

        Ok(broken_links)
    }

    /// Find notes that are in database but missing from filesystem
    pub fn get_missing_files(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path FROM notes")
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare missing files query: {}", e))
            })?;

        let missing = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                Ok((id, path))
            })
            .map_err(|e| QipuError::Other(format!("failed to query notes: {}", e)))?
            .filter_map(|r| r.ok())
            .filter(|(_, path)| !Path::new(path).exists())
            .collect();

        Ok(missing)
    }

    /// Find orphaned notes (notes with no incoming links)
    pub fn get_orphaned_notes(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id FROM notes
                 WHERE id NOT IN (SELECT DISTINCT target_id FROM edges)",
            )
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare orphaned notes query: {}", e))
            })?;

        let orphaned = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| QipuError::Other(format!("failed to query orphaned notes: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read orphaned note rows: {}", e)))?;

        Ok(orphaned)
    }

    /// Get all typed edges (non-inline links) for semantic validation
    pub fn get_all_typed_edges(&self) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, target_id, link_type FROM edges WHERE inline = 0")
            .map_err(|e| QipuError::Other(format!("failed to prepare edges query: {}", e)))?;

        let edges = stmt
            .query_map([], |row| {
                let source_id: String = row.get(0)?;
                let target_id: String = row.get(1)?;
                let link_type: String = row.get(2)?;
                Ok((source_id, target_id, link_type))
            })
            .map_err(|e| QipuError::Other(format!("failed to query edges: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read edge rows: {}", e)))?;

        Ok(edges)
    }

    /// Quick consistency check between database and filesystem
    ///
    /// Returns true if database is consistent with filesystem, false otherwise
    pub fn validate_consistency(&self, store_root: &Path) -> Result<bool> {
        let db_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .map_err(|e| QipuError::Other(format!("failed to count notes in DB: {}", e)))?;

        let fs_count = Self::count_note_files(store_root)?;

        if db_count != fs_count as i64 {
            tracing::warn!(
                "Consistency check failed: DB has {} notes, filesystem has {}",
                db_count,
                fs_count
            );
            return Ok(false);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT path, mtime FROM notes ORDER BY RANDOM() LIMIT 5")
            .map_err(|e| QipuError::Other(format!("failed to prepare mtime query: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| QipuError::Other(format!("failed to query mtime samples: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read mtime sample: {}", e)))?
        {
            let path_str: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;
            let db_mtime: i64 = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get mtime: {}", e)))?;

            let path = Path::new(&path_str);
            if !path.exists() {
                tracing::warn!("Consistency check failed: file {} missing", path_str);
                return Ok(false);
            }

            let fs_mtime = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if db_mtime != fs_mtime {
                tracing::warn!(
                    "Consistency check failed: file {} mtime mismatch (DB: {}, FS: {})",
                    path_str,
                    db_mtime,
                    fs_mtime
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
