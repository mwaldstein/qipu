use super::types::DoctorResult;
use qipu_core::error::Result;
use qipu_core::index::IndexBuilder;
use qipu_core::store::Store;
use std::fs;

/// Attempt to fix issues that are marked as fixable
#[tracing::instrument(skip(store, result), fields(store_root = %store.root().display()))]
pub fn attempt_fixes(store: &Store, result: &mut DoctorResult) -> Result<usize> {
    let mut fixed = 0;

    for issue in &result.issues {
        if !issue.fixable {
            continue;
        }

        match issue.category.as_str() {
            "missing-directory" => {
                if let Some(path) = &issue.path {
                    if fs::create_dir_all(path).is_ok() {
                        fixed += 1;
                    }
                }
            }
            "missing-config" => {
                // Recreate default config
                let config = qipu_core::config::StoreConfig::default();
                let config_path = store.root().join("config.toml");
                if config.save(&config_path).is_ok() {
                    fixed += 1;
                }
            }
            "broken-link" => {
                // For typed links (frontmatter), we can remove the broken link
                if let Some(note_id) = &issue.note_id {
                    if let Ok(mut note) = store.get_note(note_id) {
                        // Remove broken links from frontmatter
                        let valid_ids = store.existing_ids().unwrap_or_default();
                        let original_len = note.frontmatter.links.len();
                        note.frontmatter.links.retain(|l| valid_ids.contains(&l.id));

                        if note.frontmatter.links.len() < original_len
                            && store.save_note(&mut note).is_ok()
                        {
                            fixed += 1;
                        }
                    }
                }
            }
            "invalid-value" => {
                // Clamp value to 100
                if let Some(note_id) = &issue.note_id {
                    if let Ok(mut note) = store.get_note(note_id) {
                        if let Some(value) = note.frontmatter.value {
                            if value > 100 {
                                note.frontmatter.value = Some(100);
                                if store.save_note(&mut note).is_ok() {
                                    fixed += 1;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Other issue types don't have automated fixes
            }
        }
    }

    // Also rebuild indexes to ensure consistency
    let _index = IndexBuilder::new(store).build()?;

    Ok(fixed)
}
