//! Database operation performance benchmarks
//!
//! These tests verify that qipu meets the performance budgets specified in specs/operational-database.md.
//!
//! Performance targets (2000 notes):
//! - Search: <50ms
//! - List with filters: <20ms
//! - Backlink lookup: <10ms
//! - Graph traversal (3 hops): <100ms
//!
//! NOTE: All benchmarks are marked #[ignore] and require --release flag to run.
//! Debug builds are significantly slower than release builds and will fail benchmarks.

use crate::config::SearchConfig;
use crate::note::{NoteType, TypedLink};
use crate::store::InitOptions;
use crate::store::Store;
use std::time::Instant;
use tempfile::tempdir;

/// Helper to create a test store with specified number of notes
pub fn create_test_store_with_notes(count: usize) -> Store {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    for i in 0..count {
        let title = format!("Test Note {}", i);
        store.create_note(&title, None, &[], None).unwrap();
    }

    store
}

/// Helper to create a test store with linked notes for traversal tests
pub fn create_test_store_with_links(note_count: usize, links_per_note: usize) -> Store {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note_ids = Vec::new();

    for i in 0..note_count {
        let title = format!("Test Note {}", i);
        let note = store.create_note(&title, None, &[], None).unwrap();
        note_ids.push(note.id().to_string());
    }

    for (i, note_id) in note_ids.iter().enumerate() {
        let mut note = store.get_note(note_id).unwrap();

        for j in 0..links_per_note {
            let target_index = (i + j + 1) % note_ids.len();
            note.frontmatter.links.push(TypedLink {
                link_type: crate::note::LinkType::from("related"),
                id: note_ids[target_index].clone(),
            });
        }

        store.save_note(&mut note).unwrap();
    }

    store
}

/// Helper to extract note ID from title
pub fn find_note_by_title(store: &Store, title_pattern: &str) -> Option<String> {
    store
        .db()
        .list_notes(None, None, None)
        .unwrap()
        .iter()
        .find(|n| n.title.contains(title_pattern))
        .map(|n| n.id.to_string())
}

#[cfg(test)]
mod backlinks;
#[cfg(test)]
mod list;
#[cfg(test)]
mod search;
#[cfg(test)]
mod traversal;
