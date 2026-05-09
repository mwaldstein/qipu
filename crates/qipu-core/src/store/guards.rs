//! Guarded path construction for store-owned files.

use std::path::{Component, Path, PathBuf};

use crate::error::{QipuError, Result};
use crate::id::{filename, NoteId};
use crate::note::NoteType;

use super::paths::{MOCS_DIR, NOTES_DIR};
use super::Store;

fn note_type_dir(note_type: &NoteType) -> &'static str {
    if note_type.is_moc() {
        MOCS_DIR
    } else {
        NOTES_DIR
    }
}

pub fn note_target_dir(store: &Store, note_type: &NoteType) -> PathBuf {
    store.root().join(note_type_dir(note_type))
}

pub fn generated_note_path(
    store: &Store,
    note_type: &NoteType,
    id: &NoteId,
    title: &str,
) -> PathBuf {
    note_target_dir(store, note_type).join(filename(id, title))
}

pub fn resolve_imported_note_path(
    store: &Store,
    note_type: &NoteType,
    id: &NoteId,
    title: &str,
    pack_path: Option<&str>,
    source_store_path: Option<&str>,
) -> Result<PathBuf> {
    let Some(pack_path) = pack_path.filter(|p| !p.trim().is_empty()) else {
        return Ok(generated_note_path(store, note_type, id, title));
    };

    let path = Path::new(pack_path);
    let relative = if path.is_absolute() {
        let source_root = source_store_path
            .filter(|p| !p.trim().is_empty())
            .map(Path::new)
            .filter(|p| p.is_absolute())
            .ok_or_else(|| {
                unsafe_note_path(pack_path, "absolute note paths require a source store path")
            })?;

        path.strip_prefix(source_root)
            .map_err(|_| unsafe_note_path(pack_path, "absolute path is outside the source store"))?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };

    let relative = normalize_relative(&relative, pack_path)?;
    let expected_dir = note_type_dir(note_type);

    let path = if relative.components().count() == 1 {
        note_target_dir(store, note_type).join(relative)
    } else {
        let first = relative
            .components()
            .next()
            .and_then(|c| match c {
                Component::Normal(value) => value.to_str(),
                _ => None,
            })
            .ok_or_else(|| unsafe_note_path(pack_path, "missing note path directory"))?;

        if first != expected_dir {
            return Err(unsafe_note_path(
                pack_path,
                format!("must be under {expected_dir}/"),
            ));
        }

        store.root().join(relative)
    };

    ensure_under(&path, &note_target_dir(store, note_type), pack_path)?;
    Ok(path)
}

fn normalize_relative(path: &Path, raw: &str) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(unsafe_note_path(raw, "path traversal is not allowed"));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(unsafe_note_path(raw, "empty note path"));
    }

    Ok(normalized)
}

fn ensure_under(path: &Path, parent: &Path, raw: &str) -> Result<()> {
    if path.starts_with(parent) {
        Ok(())
    } else {
        Err(unsafe_note_path(
            raw,
            format!("resolved path must stay under {}", parent.display()),
        ))
    }
}

fn unsafe_note_path(path: impl std::fmt::Display, reason: impl std::fmt::Display) -> QipuError {
    QipuError::invalid_value("unsafe note path", format!("{path} ({reason})"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InitOptions;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_imported_note_path_rejects_traversal() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();
        let id = NoteId::try_new("qp-test").unwrap();
        let note_type = NoteType::from(NoteType::FLEETING);

        assert!(resolve_imported_note_path(
            &store,
            &note_type,
            &id,
            "Test",
            Some("../outside.md"),
            None,
        )
        .is_err());
    }

    #[test]
    fn test_resolve_imported_note_path_rebases_source_absolute_path() {
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let store = Store::init(target.path(), InitOptions::default()).unwrap();
        let id = NoteId::try_new("qp-test").unwrap();
        let note_type = NoteType::from(NoteType::FLEETING);
        let source_path = source.path().join("notes/qp-test-test.md");

        let path = resolve_imported_note_path(
            &store,
            &note_type,
            &id,
            "Test",
            Some(&source_path.to_string_lossy()),
            Some(&source.path().to_string_lossy()),
        )
        .unwrap();

        assert_eq!(path, store.root().join("notes/qp-test-test.md"));
    }
}
