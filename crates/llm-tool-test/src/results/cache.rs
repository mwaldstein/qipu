//! File-based caching for test results.
//!
//! Provides persistent caching of test results keyed by scenario,
/// prompt, and tool configuration to avoid redundant test runs.
use crate::results::types::{CacheKey, ResultRecord};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// File-based cache for test results.
///
/// Stores result records as JSON files in a cache directory,
/// keyed by cache keys computed from scenario and run parameters.
///
/// # Example
///
/// ```rust,no_run
/// use llm_tool_test::results::{Cache, CacheKey, ResultRecord};
/// use std::path::Path;
///
/// let cache = Cache::new(Path::new("./test-data"));
///
/// // Check for cached result
/// if let Some(record) = cache.get(&cache_key) {
///     println!("Found cached result: {}", record.id);
/// }
///
/// // Store a result
/// cache.put(&cache_key, &record).unwrap();
/// ```
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    /// Create a new cache in the specified base directory.
    ///
    /// The cache will be stored in a `cache` subdirectory.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for the cache
    ///
    /// # Returns
    ///
    /// A new `Cache` instance
    pub fn new(base_dir: &Path) -> Self {
        let cache_dir = base_dir.join("cache");
        std::fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    /// Retrieve a cached result by key.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to look up
    ///
    /// # Returns
    ///
    /// * `Some(ResultRecord)` - Cached result if found
    /// * `None` - If no cached result exists
    pub fn get(&self, key: &CacheKey) -> Option<ResultRecord> {
        let cache_file = self.cache_dir.join(key.as_string());
        if !cache_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cache_file).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Store a result in the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `record` - Result record to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` - On success
    /// * `Err` - IO or serialization error
    pub fn put(&self, key: &CacheKey, record: &ResultRecord) -> Result<()> {
        let cache_file = self.cache_dir.join(key.as_string());
        let content = serde_json::to_string_pretty(record)?;
        std::fs::write(&cache_file, content)?;
        Ok(())
    }

    /// Clear all cached results.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - On success
    /// * `Err` - IO error
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
