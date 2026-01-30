use crate::results::types::ResultRecord;
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub struct ResultsDB {
    results_path: PathBuf,
}

impl ResultsDB {
    pub fn new(base_dir: &Path) -> Self {
        let results_dir = base_dir.join("results");
        std::fs::create_dir_all(&results_dir).ok();
        Self {
            results_path: results_dir.join("results.jsonl"),
        }
    }

    pub fn append(&self, record: &ResultRecord) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.results_path)
            .context("Failed to open results.jsonl")?;

        let line = serde_json::to_string(record)?;
        writeln!(file, "{}", line).context("Failed to write to results.jsonl")?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<ResultRecord>> {
        if !self.results_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.results_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line from results.jsonl")?;
            let record: ResultRecord =
                serde_json::from_str(&line).context("Failed to parse result record")?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn load_by_id(&self, id: &str) -> Result<Option<ResultRecord>> {
        let records = self.load_all()?;
        Ok(records.into_iter().find(|r| r.id == id))
    }
}

#[cfg(test)]
mod tests {
    use crate::results::test_helpers::{create_test_record, TestDb};

    #[test]
    fn test_results_db_append_and_load_all() {
        let test_db = TestDb::new();

        let record1 = create_test_record("run-1");
        let record2 = create_test_record("run-2");

        test_db.db.append(&record1).unwrap();
        test_db.db.append(&record2).unwrap();

        let loaded = test_db.db.load_all().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "run-1");
        assert_eq!(loaded[1].id, "run-2");
    }

    #[test]
    fn test_results_db_load_empty() {
        let test_db = TestDb::new();

        let loaded = test_db.db.load_all().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_results_db_load_by_id() {
        let test_db = TestDb::new();

        let record1 = create_test_record("run-1");
        let record2 = create_test_record("run-2");

        test_db.db.append(&record1).unwrap();
        test_db.db.append(&record2).unwrap();

        let loaded = test_db.db.load_by_id("run-1").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "run-1");

        let not_found = test_db.db.load_by_id("run-3").unwrap();
        assert!(not_found.is_none());
    }
}
