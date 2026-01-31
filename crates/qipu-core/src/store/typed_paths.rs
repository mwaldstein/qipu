//! Typed path builders for Store paths
//!
//! This module provides type-safe path construction for store directories,
//! making it explicit what kind of path you're working with at compile time.
//!
//! # Example
//! ```
//! # use qipu_core::store::{Store, typed_paths::{NotePath, StorePathBuilder}, InitOptions};
//! # use std::path::PathBuf;
//! # let dir = tempfile::tempdir().unwrap();
//! let store = Store::init(dir.path(), InitOptions::default())?;
//! let note_path: NotePath = store.path_for_note("my-note.md");
//! let file_path: PathBuf = note_path.into();
//! # Ok::<(), qipu_core::error::QipuError>(())
//! ```

use std::path::{Path, PathBuf};

/// A path within the notes directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotePath(PathBuf);

/// A path within the mocs directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MocPath(PathBuf);

/// A path within the attachments directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentPath(PathBuf);

/// A path within the templates directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePath(PathBuf);

/// A path within the workspaces directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePath(PathBuf);

/// A path to the config file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPath(PathBuf);

/// A path to the database file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbPath(PathBuf);

/// A path to the store root
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRoot(PathBuf);

macro_rules! impl_typed_path {
    ($type:ident) => {
        impl $type {
            /// Create from a PathBuf (internal use)
            fn new(path: PathBuf) -> Self {
                Self(path)
            }

            /// Get the underlying Path reference
            pub fn as_path(&self) -> &Path {
                &self.0
            }

            /// Get the underlying PathBuf
            pub fn to_path_buf(&self) -> PathBuf {
                self.0.clone()
            }

            /// Check if the path exists
            pub fn exists(&self) -> bool {
                self.0.exists()
            }

            /// Join with another path component
            pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
                self.0.join(path)
            }

            /// Get parent directory
            pub fn parent(&self) -> Option<&Path> {
                self.0.parent()
            }

            /// Get the file name
            pub fn file_name(&self) -> Option<&std::ffi::OsStr> {
                self.0.file_name()
            }
        }

        impl From<$type> for PathBuf {
            fn from(typed: $type) -> PathBuf {
                typed.0
            }
        }

        impl AsRef<Path> for $type {
            fn as_ref(&self) -> &Path {
                &self.0
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.display().fmt(f)
            }
        }
    };
}

impl_typed_path!(NotePath);
impl_typed_path!(MocPath);
impl_typed_path!(AttachmentPath);
impl_typed_path!(TemplatePath);
impl_typed_path!(WorkspacePath);
impl_typed_path!(ConfigPath);
impl_typed_path!(DbPath);
impl_typed_path!(StoreRoot);

/// Builder trait for constructing typed paths from a store root
pub trait StorePathBuilder {
    /// Build a typed path to a specific note file
    fn path_for_note<P: AsRef<Path>>(&self, filename: P) -> NotePath;

    /// Build a typed path to a specific moc file
    fn path_for_moc<P: AsRef<Path>>(&self, filename: P) -> MocPath;

    /// Build a typed path to a specific attachment file
    fn path_for_attachment<P: AsRef<Path>>(&self, filename: P) -> AttachmentPath;

    /// Build a typed path to a specific template file
    fn path_for_template<P: AsRef<Path>>(&self, filename: P) -> TemplatePath;

    /// Build a typed path to a specific workspace
    fn path_for_workspace<P: AsRef<Path>>(&self, name: P) -> WorkspacePath;

    /// Get the typed config file path
    fn path_to_config(&self) -> ConfigPath;

    /// Get the typed database file path
    fn path_to_db(&self) -> DbPath;

    /// Get the typed store root path
    fn path_to_root(&self) -> StoreRoot;

    /// Get the typed notes directory path (for iteration, etc.)
    fn path_to_notes_dir(&self) -> NotePath;

    /// Get the typed mocs directory path (for iteration, etc.)
    fn path_to_mocs_dir(&self) -> MocPath;

    /// Get the typed attachments directory path (for iteration, etc.)
    fn path_to_attachments_dir(&self) -> AttachmentPath;

    /// Get the typed templates directory path (for iteration, etc.)
    fn path_to_templates_dir(&self) -> TemplatePath;

    /// Get the typed workspaces directory path (for iteration, etc.)
    fn path_to_workspaces_dir(&self) -> WorkspacePath;
}

use crate::store::paths::{
    ATTACHMENTS_DIR, CONFIG_FILE, MOCS_DIR, NOTES_DIR, TEMPLATES_DIR, WORKSPACES_DIR,
};

impl StorePathBuilder for crate::store::Store {
    fn path_for_note<P: AsRef<Path>>(&self, filename: P) -> NotePath {
        NotePath::new(self.root().join(NOTES_DIR).join(filename))
    }

    fn path_for_moc<P: AsRef<Path>>(&self, filename: P) -> MocPath {
        MocPath::new(self.root().join(MOCS_DIR).join(filename))
    }

    fn path_for_attachment<P: AsRef<Path>>(&self, filename: P) -> AttachmentPath {
        AttachmentPath::new(self.root().join(ATTACHMENTS_DIR).join(filename))
    }

    fn path_for_template<P: AsRef<Path>>(&self, filename: P) -> TemplatePath {
        TemplatePath::new(self.root().join(TEMPLATES_DIR).join(filename))
    }

    fn path_for_workspace<P: AsRef<Path>>(&self, name: P) -> WorkspacePath {
        WorkspacePath::new(self.root().join(WORKSPACES_DIR).join(name))
    }

    fn path_to_config(&self) -> ConfigPath {
        ConfigPath::new(self.root().join(CONFIG_FILE))
    }

    fn path_to_db(&self) -> DbPath {
        DbPath::new(self.root().join("qipu.db"))
    }

    fn path_to_root(&self) -> StoreRoot {
        StoreRoot::new(self.root().to_path_buf())
    }

    fn path_to_notes_dir(&self) -> NotePath {
        NotePath::new(self.root().join(NOTES_DIR))
    }

    fn path_to_mocs_dir(&self) -> MocPath {
        MocPath::new(self.root().join(MOCS_DIR))
    }

    fn path_to_attachments_dir(&self) -> AttachmentPath {
        AttachmentPath::new(self.root().join(ATTACHMENTS_DIR))
    }

    fn path_to_templates_dir(&self) -> TemplatePath {
        TemplatePath::new(self.root().join(TEMPLATES_DIR))
    }

    fn path_to_workspaces_dir(&self) -> WorkspacePath {
        WorkspacePath::new(self.root().join(WORKSPACES_DIR))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{InitOptions, Store};
    use tempfile::tempdir;

    #[test]
    fn test_typed_paths_basic() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note_path = store.path_for_note("test.md");
        assert!(note_path
            .as_path()
            .to_str()
            .unwrap()
            .contains("notes/test.md"));

        let moc_path = store.path_for_moc("overview.md");
        assert!(moc_path
            .as_path()
            .to_str()
            .unwrap()
            .contains("mocs/overview.md"));

        let attachment_path = store.path_for_attachment("image.png");
        assert!(attachment_path
            .as_path()
            .to_str()
            .unwrap()
            .contains("attachments/image.png"));

        let template_path = store.path_for_template("default.md");
        assert!(template_path
            .as_path()
            .to_str()
            .unwrap()
            .contains("templates/default.md"));

        let workspace_path = store.path_for_workspace("myworkspace");
        assert!(workspace_path
            .as_path()
            .to_str()
            .unwrap()
            .contains("workspaces/myworkspace"));
    }

    #[test]
    fn test_typed_path_conversions() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note_path = store.path_for_note("test.md");

        // Test Into<PathBuf>
        let path_buf: PathBuf = note_path.clone().into();
        assert!(path_buf.to_str().unwrap().contains("notes/test.md"));

        // Test AsRef<Path>
        let _path_ref: &Path = note_path.as_ref();

        let _ = note_path.to_path_buf();
    }

    #[test]
    fn test_directory_paths() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        assert!(store.path_to_notes_dir().exists());
        assert!(store.path_to_mocs_dir().exists());
        assert!(store.path_to_attachments_dir().exists());
        assert!(store.path_to_templates_dir().exists());

        // These should exist after init
        assert!(store.path_to_config().exists());
    }

    #[test]
    fn test_path_operations() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note_path = store.path_for_note("subdir/file.md");
        assert!(note_path
            .as_path()
            .to_str()
            .unwrap()
            .contains("subdir/file.md"));

        // Test join
        let joined = note_path.join("extra.txt");
        assert!(joined.to_str().unwrap().contains("extra.txt"));

        // Test parent
        assert!(note_path.parent().is_some());

        // Test file_name
        let file_name = note_path.file_name().unwrap();
        assert_eq!(file_name, "file.md");
    }
}
