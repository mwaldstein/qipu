use super::*;
use crate::index::IndexBuilder;
use crate::store::Store;
use tempfile::tempdir;

/// Test HeapEntry comparison ordering
#[test]
fn test_heap_entry_ordering() {
    let entry1 = HeapEntry {
        node_id: "A".to_string(),
        accumulated_cost: HopCost::from(1),
    };
    let entry2 = HeapEntry {
        node_id: "B".to_string(),
        accumulated_cost: HopCost::from(2),
    };
    let entry3 = HeapEntry {
        node_id: "C".to_string(),
        accumulated_cost: HopCost::from(1),
    };

    // Lower cost should compare as less (normal ordering)
    assert_eq!(entry1.cmp(&entry2), std::cmp::Ordering::Less);
    assert_eq!(entry2.cmp(&entry1), std::cmp::Ordering::Greater);

    // Equal costs with different node_ids
    assert_eq!(entry1.cmp(&entry3), std::cmp::Ordering::Equal);

    // PartialEq should work
    assert_eq!(entry1, entry1);
    assert_ne!(entry1, entry2);
}

/// Test that dijkstra_traverse works with ignore_value=true (unweighted)
#[test]
fn test_dijkstra_traverse_unweighted() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut root = store
        .create_note("Root Note", None, &["root".to_string()], None)
        .unwrap();
    root.frontmatter.value = Some(100);
    store.save_note(&mut root).unwrap();

    let mut mid = store
        .create_note("Mid Note", None, &["mid".to_string()], None)
        .unwrap();
    mid.frontmatter.value = Some(50);
    store.save_note(&mut mid).unwrap();

    let mut leaf = store
        .create_note("Leaf Note", None, &["leaf".to_string()], None)
        .unwrap();
    leaf.frontmatter.value = Some(0);
    store.save_note(&mut leaf).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = dijkstra_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

    assert_eq!(result.root, root.id());
    assert!(!result.truncated);
    assert_eq!(result.notes.len(), 1); // Only root (no links yet)
}

/// Test that dijkstra_traverse with ignore_value=false (weighted)
#[test]
fn test_dijkstra_traverse_weighted() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut root = store
        .create_note("Root Note", None, &["root".to_string()], None)
        .unwrap();
    root.frontmatter.value = Some(100);
    store.save_note(&mut root).unwrap();

    let mut mid = store
        .create_note("Mid Note", None, &["mid".to_string()], None)
        .unwrap();
    mid.frontmatter.value = Some(50);
    store.save_note(&mut mid).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: false,
        max_hops: HopCost::from(10),
        ..Default::default()
    };

    let result = dijkstra_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

    assert_eq!(result.root, root.id());
    assert!(!result.truncated);
    assert_eq!(result.notes.len(), 1);
}

/// Test that dijkstra_traverse respects min_value filter
#[test]
fn test_dijkstra_traverse_min_value_filter() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut root = store
        .create_note("Root Note", None, &["root".to_string()], None)
        .unwrap();
    root.frontmatter.value = Some(100);
    store.save_note(&mut root).unwrap();

    let mut low = store
        .create_note("Low Value Note", None, &["low".to_string()], None)
        .unwrap();
    low.frontmatter.value = Some(30);
    store.save_note(&mut low).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        min_value: Some(50),
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = dijkstra_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

    assert_eq!(result.root, root.id());
    assert!(!result.truncated);
    assert_eq!(result.notes.len(), 1); // Only root (low-value note excluded)
}
