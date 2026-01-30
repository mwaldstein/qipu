use crate::results::types::{CacheKey, ResultRecord};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(base_dir: &Path) -> Self {
        let cache_dir = base_dir.join("cache");
        std::fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    pub fn get(&self, key: &CacheKey) -> Option<ResultRecord> {
        let cache_file = self.cache_dir.join(key.as_string());
        if !cache_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cache_file).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn put(&self, key: &CacheKey, record: &ResultRecord) -> Result<()> {
        let cache_file = self.cache_dir.join(key.as_string());
        let content = serde_json::to_string_pretty(record)?;
        std::fs::write(&cache_file, content)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let path = entry?.path();
            if path.is_file() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}
