use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct TranscriptWriter {
    pub base_dir: PathBuf,
}

impl TranscriptWriter {
    pub fn new(base_dir: PathBuf) -> anyhow::Result<Self> {
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)?;
        }
        Ok(Self { base_dir })
    }

    pub fn write_raw(&self, content: &str) -> anyhow::Result<()> {
        fs::write(self.base_dir.join("transcript.raw.txt"), content)?;
        Ok(())
    }

    pub fn append_event(&self, event: &serde_json::Value) -> anyhow::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.base_dir.join("events.jsonl"))?;
        writeln!(file, "{}", serde_json::to_string(event)?)?;
        Ok(())
    }
}
