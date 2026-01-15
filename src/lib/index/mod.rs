//! Index infrastructure for qipu
//!
//! Per spec (specs/indexing-search.md):
//! - Derived indexes: metadata, tags, backlinks, graph
//! - Cache location: `.qipu/.cache/*.json`
//! - Incremental indexing with mtime/hash tracking
//! - Link extraction from wiki links, markdown links, and typed frontmatter links

pub mod builder;
pub mod cache;
pub mod links;
pub mod search;
pub mod types;

pub use builder::IndexBuilder;
pub use search::search;
pub use types::{Edge, Index, LinkSource, SearchResult};

#[cfg(test)]
mod tests {
    use super::types::{NoteMetadata, INDEX_VERSION};
    use super::*;
    use crate::lib::note::{Note, NoteFrontmatter};
    use crate::lib::store::{InitOptions, Store};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::tempdir;

    fn make_note(id: &str, title: &str, body: &str) -> Note {
        let fm = NoteFrontmatter::new(id.to_string(), title.to_string());
        Note::new(fm, body.to_string())
    }

    #[test]
    fn test_extract_wiki_links() {
        let mut note = make_note("qp-a1", "Test", "See [[qp-b2]] and [[qp-c3|some label]]");
        note.frontmatter.links = vec![];

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2", "qp-c3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut unresolved = HashSet::new();

        let edges = links::extract_links(&note, &valid_ids, &mut unresolved);

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|e| e.to == "qp-b2"));
        assert!(edges.iter().any(|e| e.to == "qp-c3"));
        assert!(edges.iter().all(|e| e.source == LinkSource::Inline));
        assert!(edges.iter().all(|e| e.link_type == "related"));
    }

    #[test]
    fn test_extract_typed_links() {
        use crate::lib::note::{LinkType, TypedLink};

        let mut note = make_note("qp-a1", "Test", "Body text");
        note.frontmatter.links = vec![
            TypedLink {
                link_type: LinkType::DerivedFrom,
                id: "qp-b2".to_string(),
            },
            TypedLink {
                link_type: LinkType::Supports,
                id: "qp-c3".to_string(),
            },
        ];

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2", "qp-c3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut unresolved = HashSet::new();

        let edges = links::extract_links(&note, &valid_ids, &mut unresolved);

        assert_eq!(edges.len(), 2);
        assert!(edges
            .iter()
            .any(|e| e.to == "qp-b2" && e.link_type == "derived-from"));
        assert!(edges
            .iter()
            .any(|e| e.to == "qp-c3" && e.link_type == "supports"));
        assert!(edges.iter().all(|e| e.source == LinkSource::Typed));
    }

    #[test]
    fn test_unresolved_links() {
        let note = make_note("qp-a1", "Test", "See [[qp-missing]]");

        let valid_ids: HashSet<_> = ["qp-a1"].iter().map(|s| s.to_string()).collect();
        let mut unresolved = HashSet::new();

        let edges = links::extract_links(&note, &valid_ids, &mut unresolved);

        assert_eq!(edges.len(), 1);
        assert!(unresolved.contains("qp-missing"));
    }

    #[test]
    fn test_incremental_index_updates_tags() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();
        let initial_tags = vec!["alpha".to_string()];

        let mut note = store
            .create_note("Tagged Note", None, &initial_tags)
            .unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();
        index.save(&store.root().join(".cache")).unwrap();

        sleep(Duration::from_secs(2));
        note.frontmatter.tags = vec!["beta".to_string()];
        store.save_note(&mut note).unwrap();

        let index = IndexBuilder::new(&store)
            .load_existing()
            .unwrap()
            .build()
            .unwrap();

        assert!(index.tags.get("alpha").map_or(true, |ids| ids.is_empty()));
        assert!(index
            .tags
            .get("beta")
            .map_or(false, |ids| ids.iter().any(|id| id == note.id())));
    }

    #[test]
    fn test_index_cache_roundtrip() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join(".cache");

        let mut index = Index::new();
        index.metadata.insert(
            "qp-a1".to_string(),
            NoteMetadata {
                id: "qp-a1".to_string(),
                title: "Cached Note".to_string(),
                note_type: crate::lib::note::NoteType::Fleeting,
                tags: vec!["alpha".to_string()],
                path: "notes/qp-a1.md".to_string(),
                created: None,
                updated: None,
            },
        );
        index
            .tags
            .insert("alpha".to_string(), vec!["qp-a1".to_string()]);
        index.edges.push(Edge {
            from: "qp-a1".to_string(),
            to: "qp-b2".to_string(),
            link_type: "related".to_string(),
            source: LinkSource::Inline,
        });
        index.unresolved.insert("qp-missing".to_string());
        index.files.insert(
            PathBuf::from("notes/qp-a1.md"),
            types::FileEntry {
                mtime: 123,
                note_id: "qp-a1".to_string(),
            },
        );
        index
            .id_to_path
            .insert("qp-a1".to_string(), PathBuf::from("notes/qp-a1.md"));

        index.save(&cache_dir).unwrap();

        let loaded = Index::load(&cache_dir).unwrap();
        let loaded_meta = loaded.metadata.get("qp-a1").unwrap();

        assert_eq!(loaded.version, INDEX_VERSION);
        assert_eq!(loaded.metadata.len(), 1);
        assert_eq!(loaded_meta.title, "Cached Note");
        assert_eq!(loaded_meta.tags, vec!["alpha".to_string()]);
        assert_eq!(
            loaded.tags.get("alpha").unwrap(),
            &vec!["qp-a1".to_string()]
        );
        assert_eq!(loaded.edges.len(), 1);
        assert!(loaded.unresolved.contains("qp-missing"));
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(
            loaded.id_to_path.get("qp-a1").unwrap(),
            &PathBuf::from("notes/qp-a1.md")
        );
    }

    #[test]
    fn test_index_new() {
        let index = Index::new();
        assert_eq!(index.version, INDEX_VERSION);
        assert!(index.metadata.is_empty());
        assert!(index.tags.is_empty());
        assert!(index.edges.is_empty());
    }
}
