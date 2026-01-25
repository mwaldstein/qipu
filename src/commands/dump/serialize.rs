use super::model::{PackAttachment, PackLink};
use crate::lib::config::STORE_FORMAT_VERSION;
use crate::lib::error::Result;
use crate::lib::note::Note;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;
use base64::{engine::general_purpose, Engine as _};

/// Convert serde_yaml::Value to serde_json::Value
fn serde_yaml_to_json(yaml: &serde_yaml::Value) -> serde_json::Value {
    match yaml {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                serde_json::Value::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(serde_yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    k.as_str()
                        .map(|key| (key.to_string(), serde_yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => serde_yaml_to_json(&tagged.value),
    }
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
        "H pack=1 version=1.0 store_version={} created={} store={} notes={} links={} attachments={}\n",
        STORE_FORMAT_VERSION,
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
            escape_quotes(note.title()),
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
        if let Some(value) = note.frontmatter.value {
            output.push_str(&format!(" value={}", value));
        }
        if !note.frontmatter.custom.is_empty() {
            let custom_json = serde_json::to_string(&note.frontmatter.custom).unwrap_or_default();
            let encoded_custom = general_purpose::STANDARD.encode(custom_json.as_bytes());
            output.push_str(&format!(" custom={}", encoded_custom));
        }
        output.push('\n');

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
