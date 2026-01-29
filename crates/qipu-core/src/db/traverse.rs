use crate::error::{QipuError, Result};
use crate::graph::types::Direction;
use rusqlite::params;

impl super::Database {
    /// Perform graph traversal using recursive CTE
    #[allow(dead_code)]
    #[tracing::instrument(skip(self), fields(start_id = %start_id, direction = ?direction, max_hops, max_nodes))]
    pub fn traverse(
        &self,
        start_id: &str,
        direction: Direction,
        max_hops: u32,
        max_nodes: Option<usize>,
    ) -> Result<Vec<String>> {
        let sql = match direction {
            Direction::Out => {
                "WITH RECURSIVE reachable(id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT e.target_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.source_id = r.id
                    WHERE r.depth < ?2
                ) SELECT DISTINCT id FROM reachable"
            }
            Direction::In => {
                "WITH RECURSIVE reachable(id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT e.source_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.target_id = r.id
                    WHERE r.depth < ?2
                ) SELECT DISTINCT id FROM reachable"
            }
            Direction::Both => {
                "WITH RECURSIVE reachable(id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT e.target_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.source_id = r.id
                    WHERE r.depth < ?2
                    UNION
                    SELECT e.source_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.target_id = r.id
                    WHERE r.depth < ?2
                ) SELECT DISTINCT id FROM reachable"
            }
        };

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| QipuError::Other(format!("failed to prepare traversal query: {}", e)))?;

        let mut rows = stmt
            .query(params![start_id, max_hops])
            .map_err(|e| QipuError::Other(format!("failed to execute traversal query: {}", e)))?;

        let mut reachable = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read traversal result: {}", e)))?
        {
            let id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get note id: {}", e)))?;
            reachable.push(id);
        }

        if let Some(max) = max_nodes {
            if reachable.len() > max {
                reachable.truncate(max);
            }
        }

        Ok(reachable)
    }
}
