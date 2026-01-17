use super::model::{PackAttachment, PackHeader, PackLink, PackNote, PackSource};
use crate::lib::error::Result;
use crate::lib::note::Note;
use crate::lib::store::Store;
use base64::{engine::general_purpose, Engine as _};

/// Serialize pack in readable format (for human/JSON output)
pub fn serialize_pack_readable(
    notes: &[Note],
    links: &[PackLink],
    attachments: &[PackAttachment],
    store: &Store,
) -> Result<String> {
    let header = PackHeader {
        version: "1.0".to_string(),
        created: chrono::Utc::now(),
        store_path: store.root().display().to_string(),
        notes_count: notes.len(),
        attachments_count: attachments.len(),
        links_count: links.len(),
    };

    let pack_notes: Vec<PackNote> = notes
        .iter()
        .map(|note| PackNote {
            id: note.id().to_string(),
            title: note.title().to_string(),
            note_type: note.note_type().to_string(),
            tags: note.frontmatter.tags.clone(),
            created: note.frontmatter.created,
            updated: note.frontmatter.updated,
            path: note.path.as_ref().map(|p| p.display().to_string()),
            content: note.body.clone(),
            sources: note
                .frontmatter
                .sources
                .iter()
                .map(|s| PackSource {
                    url: s.url.clone(),
                    title: s.title.clone(),
                    accessed: s.accessed.clone(),
                })
                .collect(),
            summary: note.frontmatter.summary.clone(),
            compacts: note.frontmatter.compacts.clone(),
            source: note.frontmatter.source.clone(),
            author: note.frontmatter.author.clone(),
            generated_by: note.frontmatter.generated_by.clone(),
            prompt_hash: note.frontmatter.prompt_hash.clone(),
            verified: note.frontmatter.verified,
        })
        .collect();

    let pack_data = serde_json::json!({
        "header": header,
        "notes": pack_notes,
        "links": links,
        "attachments": attachments.iter().map(|att| {
            let mut obj = serde_json::json!({
                "path": att.path,
                "name": att.name,
                "data": general_purpose::STANDARD.encode(&att.data),
            });
            if let Some(content_type) = &att.content_type {
                obj["content_type"] = serde_json::json!(content_type);
            }
            obj
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&pack_data)?)
}

/// Serialize pack in records format (compact, line-oriented)
pub fn serialize_pack_records(
    notes: &[Note],
    links: &[PackLink],
    attachments: &[PackAttachment],
    store: &Store,
) -> Result<String> {
    let mut output = String::new();

    // Header line
    output.push_str(&format!(
        "H pack=1 version=1.0 created={} store={} notes={} links={} attachments={}\n",
        chrono::Utc::now().to_rfc3339(),
        store.root().display(),
        notes.len(),
        links.len(),
        attachments.len()
    ));

    // Notes section
    for note in notes {
        let tags_csv = if note.frontmatter.tags.is_empty() {
            "-".to_string()
        } else {
            note.frontmatter.tags.join(",")
        };

        // Note metadata line
        output.push_str(&format!(
            "N {} {} \"{}\" tags={} created={} updated={}",
            note.id(),
            note.note_type(),
            note.title(),
            tags_csv,
            note.frontmatter
                .created
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "-".to_string()),
            note.frontmatter
                .updated
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "-".to_string())
        ));

        if let Some(summary) = &note.frontmatter.summary {
            output.push_str(&format!(" summary=\"{}\"", summary));
        }
        if !note.frontmatter.compacts.is_empty() {
            output.push_str(&format!(
                " compacts={}",
                note.frontmatter.compacts.join(",")
            ));
        }
        if let Some(source) = &note.frontmatter.source {
            output.push_str(&format!(" source=\"{}\"", source));
        }
        if let Some(author) = &note.frontmatter.author {
            output.push_str(&format!(" author=\"{}\"", author));
        }
        if let Some(generated_by) = &note.frontmatter.generated_by {
            output.push_str(&format!(" generated_by=\"{}\"", generated_by));
        }
        if let Some(prompt_hash) = &note.frontmatter.prompt_hash {
            output.push_str(&format!(" prompt_hash=\"{}\"", prompt_hash));
        }
        if let Some(verified) = note.frontmatter.verified {
            output.push_str(&format!(" verified={}", verified));
        }
        output.push_str("\n");

        // Note content line (base64 encoded for safe transport)
        if !note.body.is_empty() {
            let encoded = general_purpose::STANDARD.encode(note.body.as_bytes());
            output.push_str(&format!("C {}\n", encoded));
            output.push_str("C-END\n");
        }

        // Sources
        for source in &note.frontmatter.sources {
            let title = source.title.as_deref().unwrap_or("");
            let accessed = source.accessed.as_deref().unwrap_or("-");
            output.push_str(&format!(
                "S {} url={} title=\"{}\" accessed={}\n",
                note.id(),
                source.url,
                title,
                accessed
            ));
        }
    }

    // Links section
    for link in links {
        let link_type = link.link_type.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "L {} {} type={} inline={}\n",
            link.from, link.to, link_type, link.inline
        ));
    }

    // Attachments section
    for attachment in attachments {
        let content_type = attachment.content_type.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "A {} name={} content_type={}\n",
            attachment.path, attachment.name, content_type
        ));

        // Attachment data (base64 encoded)
        let encoded = general_purpose::STANDARD.encode(&attachment.data);
        output.push_str(&format!("D {}\n", encoded));
        output.push_str("D-END\n");
    }

    // End marker
    output.push_str("END\n");

    Ok(output)
}
