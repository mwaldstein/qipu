//! Index infrastructure for qipu
//!
//! Per spec (specs/indexing-search.md):
//! - Derived indexes: metadata, tags, backlinks, graph
//! - Link extraction from wiki links, markdown links, and typed frontmatter links

pub mod builder;
pub mod links;
pub mod types;

pub use builder::IndexBuilder;
pub use types::{Edge, Index, LinkSource, SearchResult};

#[cfg(test)]
mod tests {
    use super::types::INDEX_VERSION;
    use super::*;
    use crate::lib::note::{Note, NoteFrontmatter};

    use std::collections::HashSet;
    use std::path::PathBuf;

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

        let edges = links::extract_links(
            &note,
            &valid_ids,
            &mut unresolved,
            None,
            &std::collections::HashMap::new(),
        );

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
                link_type: LinkType::from(LinkType::DERIVED_FROM),
                id: "qp-b2".to_string(),
            },
            TypedLink {
                link_type: LinkType::from(LinkType::SUPPORTS),
                id: "qp-c3".to_string(),
            },
        ];

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2", "qp-c3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut unresolved = HashSet::new();

        let edges = links::extract_links(
            &note,
            &valid_ids,
            &mut unresolved,
            None,
            &std::collections::HashMap::new(),
        );

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

        let edges = links::extract_links(
            &note,
            &valid_ids,
            &mut unresolved,
            None,
            &std::collections::HashMap::new(),
        );

        assert_eq!(edges.len(), 0);
        assert!(unresolved.contains("qp-missing"));
    }

    #[test]
    fn test_index_new() {
        let index = Index::new();
        assert_eq!(index.version, INDEX_VERSION);
        assert!(index.metadata.is_empty());
        assert!(index.tags.is_empty());
        assert!(index.edges.is_empty());
    }

    #[test]
    fn test_extract_markdown_relative_path_links() {
        use std::collections::HashMap;

        // Create a note with a relative path markdown link
        let mut note = make_note("qp-a1", "Test", "See [Other Note](../mocs/qp-b2-other.md)");
        note.frontmatter.links = vec![];
        // Set the note's path
        let source_path = PathBuf::from("/tmp/qipu/.qipu/notes/qp-a1-test.md");

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2"].iter().map(|s| s.to_string()).collect();
        let mut unresolved = HashSet::new();

        // Build path_to_id mapping
        let mut path_to_id = HashMap::new();
        path_to_id.insert(
            PathBuf::from("/tmp/qipu/.qipu/mocs/qp-b2-other.md"),
            "qp-b2".to_string(),
        );

        let edges = links::extract_links(
            &note,
            &valid_ids,
            &mut unresolved,
            Some(&source_path),
            &path_to_id,
        );

        // Should find the link via relative path resolution
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].to, "qp-b2");
        assert_eq!(edges[0].source, LinkSource::Inline);
        assert_eq!(edges[0].link_type, "related");
    }

    #[test]
    fn test_markdown_links_with_qipu_id() {
        use std::collections::HashMap;

        // Test that markdown links containing qp- IDs still work
        let mut note = make_note(
            "qp-a1",
            "Test",
            "See [Other](./qp-b2-slug.md) and [ref](qp-c3)",
        );
        note.frontmatter.links = vec![];

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2", "qp-c3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut unresolved = HashSet::new();

        let edges = links::extract_links(&note, &valid_ids, &mut unresolved, None, &HashMap::new());

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|e| e.to == "qp-b2"));
        assert!(edges.iter().any(|e| e.to == "qp-c3"));
    }

    #[test]
    fn test_markdown_links_skip_external_urls() {
        use std::collections::HashMap;

        // Test that external URLs are ignored
        let mut note = make_note(
            "qp-a1",
            "Test",
            "See [Google](https://google.com) and [anchor](#section)",
        );
        note.frontmatter.links = vec![];

        let valid_ids: HashSet<_> = ["qp-a1"].iter().map(|s| s.to_string()).collect();
        let mut unresolved = HashSet::new();

        let edges = links::extract_links(&note, &valid_ids, &mut unresolved, None, &HashMap::new());

        // Should not extract any edges from external URLs or anchors
        assert_eq!(edges.len(), 0);
    }
}
