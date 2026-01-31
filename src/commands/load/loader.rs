use std::collections::HashSet;

use base64::{engine::general_purpose, Engine as _};

use super::model::{PackAttachment, PackLink, PackNote};
use super::LoadStrategy;
use qipu_core::bail_invalid;
use qipu_core::error::{QipuError, Result};
use qipu_core::note::{Note, NoteFrontmatter, NoteType, Source};
use qipu_core::store::Store;

fn write_note_preserving_updated(
    store: &Store,
    note: &Note,
    existing_ids: &HashSet<String>,
) -> Result<()> {
    let path = note
        .path
        .as_ref()
        .ok_or_else(|| QipuError::invalid_value("note", "cannot save without path"))?;
    let new_content = note.to_markdown()?;

    let should_write = if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(existing) => existing != new_content,
            Err(_) => true,
        }
    } else {
        true
    };

    if should_write {
        std::fs::write(path, new_content)?;

        store.db().insert_note(note)?;
        store.db().insert_edges(note, existing_ids)?;
    }

    Ok(())
}

pub fn load_notes(
    store: &Store,
    pack_notes: &[PackNote],
    strategy: &LoadStrategy,
) -> Result<(usize, HashSet<String>, HashSet<String>)> {
    let mut loaded_count = 0;
    let mut loaded_ids: HashSet<String> = HashSet::new();
    let mut new_ids: HashSet<String> = HashSet::new();
    let existing_ids = store.existing_ids()?;

    for pack_note in pack_notes {
        let note_type = pack_note.note_type.parse::<NoteType>().map_err(|e| {
            QipuError::invalid_value(&format!("note type '{}'", pack_note.note_type), e)
        })?;

        let sources = pack_note
            .sources
            .iter()
            .map(|s| Source {
                url: s.url.clone(),
                title: s.title.clone(),
                accessed: s.accessed.as_ref().and_then(|s| s.parse().ok()),
            })
            .collect();

        let frontmatter = NoteFrontmatter {
            id: pack_note.id.clone(),
            title: pack_note.title.clone(),
            note_type: Some(note_type.clone()),
            tags: pack_note.tags.clone(),
            created: pack_note.created,
            updated: pack_note.updated,
            sources,
            links: Vec::new(),
            summary: pack_note.summary.clone(),
            compacts: pack_note.compacts.clone(),
            source: pack_note.source.clone(),
            author: pack_note.author.clone(),
            generated_by: pack_note.generated_by.clone(),
            prompt_hash: pack_note.prompt_hash.clone(),
            verified: pack_note.verified,
            value: pack_note.value,
            custom: pack_note
                .custom
                .iter()
                .map(|(k, v)| (k.clone(), serde_json_to_yaml(v)))
                .collect(),
        };

        let pack_path = pack_note.path.as_ref().map(|p| {
            let path = std::path::Path::new(p);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                store.root().join(path)
            }
        });

        let mut note = Note {
            frontmatter,
            body: pack_note.content.clone(),
            path: pack_path.clone(),
        };

        let target_dir = if note_type.is_moc() {
            store.mocs_dir()
        } else {
            store.notes_dir()
        };

        let should_load = if existing_ids.contains(note.id()) {
            if matches!(strategy, LoadStrategy::Overwrite) {
                if let Ok(existing_note) = store.get_note(note.id()) {
                    if let Some(existing_path) = existing_note.path {
                        note.path = Some(existing_path);
                    }
                }
            }

            if note.path.is_none() {
                let file_name = format!("{}-{}.md", note.id(), slug::slugify(note.title()));
                note.path = Some(target_dir.join(&file_name));
            }

            match strategy {
                LoadStrategy::Skip => {
                    tracing::debug!(
                        title = %note.title(),
                        id = %note.id(),
                        "Skipping conflicting note"
                    );
                    false
                }
                LoadStrategy::Overwrite => {
                    tracing::debug!(
                        title = %note.title(),
                        id = %note.id(),
                        "Overwriting existing note"
                    );
                    if let Some(path) = &note.path {
                        if path.exists() {
                            std::fs::remove_file(path).map_err(|e| {
                                QipuError::io_operation("delete existing file", path.display(), e)
                            })?;
                        }
                    }
                    true
                }
                LoadStrategy::MergeLinks => {
                    tracing::debug!(
                        title = %note.title(),
                        id = %note.id(),
                        "Keeping existing note content, will merge links from pack"
                    );
                    false
                }
            }
        } else {
            if note.path.is_none() {
                let file_name = format!("{}-{}.md", note.id(), slug::slugify(note.title()));
                note.path = Some(target_dir.join(&file_name));
            }
            true
        };

        if should_load {
            write_note_preserving_updated(store, &note, &existing_ids)?;
            loaded_count += 1;
            loaded_ids.insert(note.id().to_string());
            if !existing_ids.contains(note.id()) {
                new_ids.insert(note.id().to_string());
            }
        } else if matches!(strategy, LoadStrategy::MergeLinks) && existing_ids.contains(note.id()) {
            loaded_ids.insert(note.id().to_string());
        }
    }

    Ok((loaded_count, loaded_ids, new_ids))
}

pub fn load_links(
    store: &Store,
    pack_links: &[PackLink],
    source_ids: &HashSet<String>,
    target_ids: &HashSet<String>,
    existing_ids: &HashSet<String>,
) -> Result<usize> {
    let mut loaded_count = 0;

    let mut links_by_source: std::collections::HashMap<String, Vec<PackLink>> =
        std::collections::HashMap::new();
    for pack_link in pack_links {
        links_by_source
            .entry(pack_link.from.clone())
            .or_default()
            .push(pack_link.clone());
    }

    for (source_id, links) in links_by_source {
        if source_ids.contains(&source_id) {
            let mut source_note = store.get_note(&source_id)?;

            for pack_link in links {
                if target_ids.contains(&pack_link.to) {
                    let link_exists = source_note.frontmatter.links.iter().any(|l| {
                        l.id == pack_link.to.as_str()
                            && l.link_type == pack_link.link_type.as_deref().unwrap_or("")
                    });

                    if !link_exists {
                        if let Some(ref type_str) = pack_link.link_type {
                            let link_type = type_str.parse().map_err(|e| {
                                QipuError::invalid_value(&format!("link type '{}'", type_str), e)
                            })?;

                            source_note
                                .frontmatter
                                .links
                                .push(qipu_core::note::TypedLink {
                                    link_type,
                                    id: pack_link.to.clone(),
                                });
                            loaded_count += 1;
                        }
                    }
                }
            }

            write_note_preserving_updated(store, &source_note, existing_ids)?;
        }
    }

    Ok(loaded_count)
}

pub fn load_attachments(
    store: &Store,
    pack_attachments: &[PackAttachment],
    _loaded_notes: &[PackNote],
) -> Result<usize> {
    let mut loaded_count = 0;

    let attachments_dir = store.root().join("attachments");
    std::fs::create_dir_all(&attachments_dir)
        .map_err(|e| QipuError::io_operation("create", "attachments directory", e))?;

    for pack_attachment in pack_attachments {
        let data = general_purpose::STANDARD
            .decode(&pack_attachment.data)
            .map_err(|e| QipuError::FailedOperation {
                operation: "decode attachment data".to_string(),
                reason: e.to_string(),
            })?;

        let canonical_attachments_dir =
            std::fs::canonicalize(&attachments_dir).map_err(|e| QipuError::FailedOperation {
                operation: "resolve attachments directory".to_string(),
                reason: e.to_string(),
            })?;

        let file_name = std::path::Path::new(&pack_attachment.name)
            .file_name()
            .ok_or_else(|| {
                QipuError::invalid_value(
                    &format!("attachment name '{}'", pack_attachment.name),
                    "no filename component",
                )
            })?;
        let safe_attachment_path = canonical_attachments_dir.join(file_name);

        // Defense in depth: canonicalize the final path and verify it's within attachments dir
        let canonical_safe_path = std::fs::canonicalize(&safe_attachment_path).or_else(|_| {
            // Path doesn't exist yet, which is expected for new attachments
            // We'll verify by checking the parent chain
            Ok::<_, QipuError>(safe_attachment_path.clone())
        })?;

        if !canonical_safe_path.starts_with(&canonical_attachments_dir) {
            bail_invalid!(
                &format!("attachment name '{}'", pack_attachment.name),
                "would write outside attachments directory"
            );
        }

        std::fs::write(&safe_attachment_path, data)
            .map_err(|e| QipuError::io_operation("write attachment", &pack_attachment.name, e))?;

        loaded_count += 1;
    }

    Ok(loaded_count)
}

pub(crate) fn serde_json_to_yaml(json: &serde_json::Value) -> serde_yaml::Value {
    match json {
        serde_json::Value::Null => serde_yaml::Value::Null,
        serde_json::Value::Bool(b) => serde_yaml::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(i))
            } else if let Some(u) = n.as_u64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(u))
            } else if let Some(f) = n.as_f64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(f))
            } else {
                serde_yaml::Value::Null
            }
        }
        serde_json::Value::String(s) => serde_yaml::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            serde_yaml::Value::Sequence(arr.iter().map(serde_json_to_yaml).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: serde_yaml::Mapping = obj
                .iter()
                .map(|(k, v)| (serde_yaml::Value::String(k.clone()), serde_json_to_yaml(v)))
                .collect();
            serde_yaml::Value::Mapping(map)
        }
    }
}
