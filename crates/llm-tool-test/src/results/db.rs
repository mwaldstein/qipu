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
