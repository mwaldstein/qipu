use super::types::{Index, INDEX_VERSION};
use crate::lib::error::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub(crate) const INDEX_META_FILE: &str = "index_meta.json";
pub(crate) const INDEX_METADATA_FILE: &str = "metadata.json";
pub(crate) const INDEX_TAGS_FILE: &str = "tags.json";
pub(crate) const INDEX_EDGES_FILE: &str = "edges.json";
pub(crate) const INDEX_UNRESOLVED_FILE: &str = "unresolved.json";
pub(crate) const INDEX_FILES_FILE: &str = "files.json";
pub(crate) const INDEX_ID_TO_PATH_FILE: &str = "id_to_path.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMeta {
    version: u32,
}

impl Index {
    /// Load index from cache directory
    pub fn load(cache_dir: &Path) -> Result<Self> {
        let meta_path = cache_dir.join(INDEX_META_FILE);
        if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)?;
            let meta: IndexMeta = serde_json::from_str(&content)?;
            if meta.version != INDEX_VERSION {
                return Ok(Self::new());
            }

            let metadata = load_cache_file(&cache_dir.join(INDEX_METADATA_FILE))?;
            let tags = load_cache_file(&cache_dir.join(INDEX_TAGS_FILE))?;
            let edges = load_cache_file(&cache_dir.join(INDEX_EDGES_FILE))?;
            let unresolved = load_cache_file(&cache_dir.join(INDEX_UNRESOLVED_FILE))?;
            let files = load_cache_file(&cache_dir.join(INDEX_FILES_FILE))?;
            let id_to_path = load_cache_file(&cache_dir.join(INDEX_ID_TO_PATH_FILE))?;

            return Ok(Index {
                version: meta.version,
                metadata,
                tags,
                edges,
                unresolved,
                files,
                id_to_path,
            });
        }

        let index_path = cache_dir.join("index.json");
        if !index_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&index_path)?;
        let index: Index = serde_json::from_str(&content)?;

        if index.version != INDEX_VERSION {
            return Ok(Self::new());
        }

        Ok(index)
    }

    /// Save index to cache directory
    ///
    /// Per specs/cli-tool.md: "Avoid writing derived caches unless command explicitly calls for it"
    /// This function only writes if the index content has actually changed.
    pub fn save(&self, cache_dir: &Path) -> Result<()> {
        fs::create_dir_all(cache_dir)?;

        let meta = IndexMeta {
            version: INDEX_VERSION,
        };

        write_cache_file(&cache_dir.join(INDEX_META_FILE), &meta)?;
        write_cache_file(&cache_dir.join(INDEX_METADATA_FILE), &self.metadata)?;
        write_cache_file(&cache_dir.join(INDEX_TAGS_FILE), &self.tags)?;
        write_cache_file(&cache_dir.join(INDEX_EDGES_FILE), &self.edges)?;
        write_cache_file(&cache_dir.join(INDEX_UNRESOLVED_FILE), &self.unresolved)?;
        write_cache_file(&cache_dir.join(INDEX_FILES_FILE), &self.files)?;
        write_cache_file(&cache_dir.join(INDEX_ID_TO_PATH_FILE), &self.id_to_path)?;

        let legacy_path = cache_dir.join("index.json");
        if legacy_path.exists() {
            fs::remove_file(legacy_path)?;
        }

        Ok(())
    }
}

/// Get file modification time as unix timestamp
pub(crate) fn get_mtime(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()
        .map_err(|e| crate::lib::error::QipuError::Other(format!("failed to get mtime: {}", e)))?;
    Ok(mtime
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}

pub(crate) fn load_cache_file<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(path)?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

pub(crate) fn write_cache_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let new_content = serde_json::to_string_pretty(value)?;
    write_if_changed(path, &new_content)
}

pub(crate) fn write_if_changed(path: &Path, new_content: &str) -> Result<()> {
    let should_write = if path.exists() {
        match fs::read_to_string(path) {
            Ok(existing) => existing != new_content,
            Err(_) => true,
        }
    } else {
        true
    };

    if should_write {
        fs::write(path, new_content)?;
    }

    Ok(())
}
