#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::config::{StoreConfig, STORE_FORMAT_VERSION};
    use crate::error::QipuError;
    use crate::id::IdScheme;
    use crate::note::NoteType;
    use crate::store::{
        paths, InitOptions, Store, ATTACHMENTS_DIR, CONFIG_FILE, DEFAULT_STORE_DIR, MOCS_DIR,
        NOTES_DIR, TEMPLATES_DIR, VISIBLE_STORE_DIR,
    };

    #[test]
    fn test_init_store() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        assert!(store.root().exists());
        assert!(store.notes_dir().exists());
        assert!(store.mocs_dir().exists());
        assert!(store.root().join(CONFIG_FILE).exists());
    }

    #[test]
    fn test_init_visible() {
        let dir = tempdir().unwrap();
        let options = InitOptions {
            visible: true,
            ..Default::default()
        };
        let _store = Store::init(dir.path(), options).unwrap();

        assert!(dir.path().join(VISIBLE_STORE_DIR).exists());
    }

    #[test]
    fn test_discover_store() {
        let dir = tempdir().unwrap();
        Store::init(dir.path(), InitOptions::default()).unwrap();

        let discovered = Store::discover(dir.path()).unwrap();
        assert_eq!(discovered.root(), dir.path().join(DEFAULT_STORE_DIR));
    }

    #[test]
    fn test_create_note() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store.create_note("Test Note", None, &[], None).unwrap();
        assert!(note.id().starts_with("qp-"));
        assert_eq!(note.title(), "Test Note");
        assert!(note.path.is_some());
        assert!(note.path.as_ref().unwrap().exists());
    }

    #[test]
    fn test_list_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Note 1", None, &[], None).unwrap();
        store.create_note("Note 2", None, &[], None).unwrap();

        let notes = store.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_store_without_config() {
        let dir = tempdir().unwrap();
        let store_root = dir.path().join(DEFAULT_STORE_DIR);

        fs::create_dir_all(store_root.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(store_root.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(store_root.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(store_root.join(TEMPLATES_DIR)).unwrap();

        let store = Store::open(&store_root).unwrap();
        assert_eq!(store.config().version, STORE_FORMAT_VERSION);
        assert_eq!(
            store.config().default_note_type,
            NoteType::from(NoteType::FLEETING)
        );
        assert_eq!(store.config().id_scheme, IdScheme::Hash);

        let note = store.create_note("Test Note", None, &[], None).unwrap();
        assert!(note.id().starts_with("qp-"));

        let templates_dir = store.root().join(TEMPLATES_DIR);
        assert!(templates_dir.join("fleeting.md").exists());
        assert!(templates_dir.join("permanent.md").exists());
    }

    #[test]
    fn test_discover_with_custom_store_path() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let default_store = project_root.join(DEFAULT_STORE_DIR);
        let custom_store = project_root.join("custom_notes");

        fs::create_dir_all(&default_store).unwrap();
        fs::create_dir_all(custom_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(custom_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(custom_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(custom_store.join(TEMPLATES_DIR)).unwrap();

        let config_path = default_store.join(CONFIG_FILE);
        let config = StoreConfig {
            store_path: Some("custom_notes".to_string()),
            ..Default::default()
        };
        config.save(&config_path).unwrap();

        let loaded_config = StoreConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.store_path, Some("custom_notes".to_string()));

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, custom_store);
    }

    #[test]
    fn test_discover_without_custom_store_path() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let default_store = project_root.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(default_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(default_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(default_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(default_store.join(TEMPLATES_DIR)).unwrap();

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, default_store);
    }

    #[test]
    fn test_discovery_stops_at_project_root() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let project_store = project_root.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(project_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(project_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(TEMPLATES_DIR)).unwrap();

        let parent_dir = project_root.parent().unwrap();
        let parent_store = parent_dir.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(parent_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(TEMPLATES_DIR)).unwrap();

        fs::File::create(project_root.join(".git")).unwrap();

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, project_store);

        let result = paths::discover_store(parent_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), parent_store);
    }

    #[test]
    fn test_discovery_fails_at_project_root_without_store() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let parent_dir = project_root.parent().unwrap();
        let parent_store = parent_dir.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(parent_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(TEMPLATES_DIR)).unwrap();

        fs::File::create(project_root.join(".git")).unwrap();

        let result = paths::discover_store(project_root);
        assert!(result.is_err());
        assert!(matches!(result, Err(QipuError::StoreNotFound { .. })));
    }

    #[test]
    fn test_discovery_stops_at_cargo_toml() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let project_store = project_root.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(project_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(project_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(project_store.join(TEMPLATES_DIR)).unwrap();

        let parent_dir = project_root.parent().unwrap();
        let parent_store = parent_dir.join(DEFAULT_STORE_DIR);

        fs::create_dir_all(parent_store.join(NOTES_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(MOCS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(ATTACHMENTS_DIR)).unwrap();
        fs::create_dir_all(parent_store.join(TEMPLATES_DIR)).unwrap();

        fs::File::create(project_root.join("Cargo.toml")).unwrap();

        let discovered = paths::discover_store(project_root).unwrap();
        assert_eq!(discovered, project_store);

        let result = paths::discover_store(parent_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), parent_store);
    }
}
