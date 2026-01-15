use super::notes::default_template;
use super::paths::{
    ATTACHMENTS_DIR, CACHE_DIR, GITIGNORE_FILE, MOCS_DIR, NOTES_DIR, TEMPLATES_DIR,
};
use crate::lib::error::{QipuError, Result};
use crate::lib::note::NoteType;
use std::fs;
use std::path::Path;

pub(crate) fn validate_store_layout(store_root: &Path) -> Result<()> {
    let mut missing = Vec::new();

    for (dir_name, label) in [
        (NOTES_DIR, NOTES_DIR),
        (MOCS_DIR, MOCS_DIR),
        (ATTACHMENTS_DIR, ATTACHMENTS_DIR),
        (TEMPLATES_DIR, TEMPLATES_DIR),
    ] {
        let path = store_root.join(dir_name);
        if !path.is_dir() {
            missing.push(label.to_string());
        }
    }

    // Derived; safe to recreate.
    let cache_dir = store_root.join(CACHE_DIR);
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    if !missing.is_empty() {
        return Err(QipuError::InvalidStore {
            reason: format!(
                "missing required store dirs: {} (store_root={})",
                missing.join(", "),
                store_root.display()
            ),
        });
    }

    Ok(())
}

pub(crate) fn ensure_store_gitignore(store_root: &Path) -> Result<()> {
    let path = store_root.join(GITIGNORE_FILE);
    let required = ["qipu.db", ".cache/"];

    if !path.exists() {
        fs::write(&path, format!("{}\n{}\n", required[0], required[1]))?;
        return Ok(());
    }

    let mut content = fs::read_to_string(&path)?;
    let mut changed = false;

    for entry in required {
        if !content.lines().any(|l| l.trim() == entry) {
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            changed = true;
        }
    }

    if changed {
        fs::write(&path, content)?;
    }

    Ok(())
}

pub(crate) fn ensure_default_templates(templates_dir: &Path) -> Result<()> {
    fs::create_dir_all(templates_dir)?;

    let templates = [
        ("fleeting.md", default_template(NoteType::Fleeting)),
        ("literature.md", default_template(NoteType::Literature)),
        ("permanent.md", default_template(NoteType::Permanent)),
        ("moc.md", default_template(NoteType::Moc)),
    ];

    for (name, content) in templates {
        let path = templates_dir.join(name);
        if !path.exists() {
            fs::write(path, content)?;
        }
    }

    Ok(())
}
