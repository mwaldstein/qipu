use super::types::{DoctorResult, Issue, Severity};
use qipu_core::store::paths::ATTACHMENTS_DIR;
use qipu_core::store::Store;

pub fn check_store_structure(store: &Store, result: &mut DoctorResult) {
    if !store.notes_dir().exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-directory".to_string(),
            message: "Notes directory does not exist".to_string(),
            note_id: None,
            path: Some(store.notes_dir().display().to_string()),
            fixable: true,
        });
    }

    if !store.mocs_dir().exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-directory".to_string(),
            message: "MOCs directory does not exist".to_string(),
            note_id: None,
            path: Some(store.mocs_dir().display().to_string()),
            fixable: true,
        });
    }

    let attachments_dir = store.root().join(ATTACHMENTS_DIR);
    if !attachments_dir.exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-directory".to_string(),
            message: "Attachments directory does not exist".to_string(),
            note_id: None,
            path: Some(attachments_dir.display().to_string()),
            fixable: true,
        });
    }

    let config_path = store.root().join("config.toml");
    if !config_path.exists() {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "missing-config".to_string(),
            message: "Config file does not exist".to_string(),
            note_id: None,
            path: Some(config_path.display().to_string()),
            fixable: true,
        });
    }
}
