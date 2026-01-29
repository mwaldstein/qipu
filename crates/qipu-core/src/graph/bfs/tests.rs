use crate::graph::bfs::*;
use crate::graph::types::{HopCost, TreeOptions};
use crate::index::IndexBuilder;
use crate::store::Store;
use tempfile::tempdir;

/// Test that bfs_find_path works with ignore_value=true (unweighted)
#[test]
fn test_bfs_find_path_unweighted() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    from_note.frontmatter.value = Some(100);
    store.save_note(&mut from_note).unwrap();

    let mut mid_note = store
        .create_note("Mid Note", None, &["mid".to_string()], None)
        .unwrap();
    mid_note.frontmatter.value = Some(50);
    store.save_note(&mut mid_note).unwrap();

    let mut to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();
    to_note.frontmatter.value = Some(0);
    store.save_note(&mut to_note).unwrap();

    // Create links: from -> mid -> to
    from_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: mid_note.id().to_string(),
        });
    store.save_note(&mut from_note).unwrap();

    mid_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: to_note.id().to_string(),
        });
    store.save_note(&mut mid_note).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    assert!(result.found);
    assert_eq!(result.path_length, 2);
    assert_eq!(result.notes.len(), 3);
}

/// Test that bfs_find_path works with ignore_value=false (weighted)
#[test]
fn test_bfs_find_path_weighted() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    from_note.frontmatter.value = Some(100);
    store.save_note(&mut from_note).unwrap();

    let mut mid_note = store
        .create_note("Mid Note", None, &["mid".to_string()], None)
        .unwrap();
    mid_note.frontmatter.value = Some(50);
    store.save_note(&mut mid_note).unwrap();

    let mut to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();
    to_note.frontmatter.value = Some(0);
    store.save_note(&mut to_note).unwrap();

    // Create links: from -> mid -> to
    from_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: mid_note.id().to_string(),
        });
    store.save_note(&mut from_note).unwrap();

    mid_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: to_note.id().to_string(),
        });
    store.save_note(&mut mid_note).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: false,
        max_hops: HopCost::from(10),
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    assert!(result.found);
    assert_eq!(result.path_length, 2);
    assert_eq!(result.notes.len(), 3);
}

/// Test that bfs_find_path respects min_value filter
#[test]
fn test_bfs_find_path_min_value_filter() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    from_note.frontmatter.value = Some(100);
    store.save_note(&mut from_note).unwrap();

    let mut low_mid = store
        .create_note("Low Mid Note", None, &["lowmid".to_string()], None)
        .unwrap();
    low_mid.frontmatter.value = Some(30);
    store.save_note(&mut low_mid).unwrap();

    let mut high_mid = store
        .create_note("High Mid Note", None, &["highmid".to_string()], None)
        .unwrap();
    high_mid.frontmatter.value = Some(80);
    store.save_note(&mut high_mid).unwrap();

    let mut to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();
    to_note.frontmatter.value = Some(100);
    store.save_note(&mut to_note).unwrap();

    // Create links: from -> low_mid -> to and from -> high_mid -> to
    from_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: low_mid.id().to_string(),
        });
    store.save_note(&mut from_note).unwrap();

    low_mid.frontmatter.links.push(crate::note::TypedLink {
        link_type: crate::note::LinkType::from("supports"),
        id: to_note.id().to_string(),
    });
    store.save_note(&mut low_mid).unwrap();

    from_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: high_mid.id().to_string(),
        });
    store.save_note(&mut from_note).unwrap();

    high_mid
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: to_note.id().to_string(),
        });
    store.save_note(&mut high_mid).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        min_value: Some(50),
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    // Should find path through high_mid (passes filter), not low_mid (excluded)
    assert!(result.found);
    assert_eq!(result.path_length, 2);
    assert_eq!(result.notes.len(), 3);
}

/// Test that bfs_find_path returns not found when target unreachable
#[test]
fn test_bfs_find_path_not_found() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    let to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();

    // No links created - notes are disconnected

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    assert!(!result.found);
    assert_eq!(result.path_length, 0);
    assert_eq!(result.notes.len(), 0);
}

/// Test that bfs_find_path handles from/to notes that fail min_value filter
#[test]
fn test_bfs_find_path_from_fails_filter() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    from_note.frontmatter.value = Some(10);
    store.save_note(&mut from_note).unwrap();

    let mut to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();
    to_note.frontmatter.value = Some(90);
    store.save_note(&mut to_note).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        min_value: Some(50),
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    assert!(!result.found); // From note fails filter
}

/// Test that bfs_find_path handles to notes that fail min_value filter
#[test]
fn test_bfs_find_path_to_fails_filter() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    from_note.frontmatter.value = Some(90);
    store.save_note(&mut from_note).unwrap();

    let mut to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();
    to_note.frontmatter.value = Some(10);
    store.save_note(&mut to_note).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        min_value: Some(50),
        max_hops: HopCost::from(5),
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    assert!(!result.found); // To note fails filter
}

/// Test that bfs_find_path respects max_hops limit
#[test]
fn test_bfs_find_path_max_hops() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut from_note = store
        .create_note("From Note", None, &["from".to_string()], None)
        .unwrap();
    store.save_note(&mut from_note).unwrap();

    let mut mid1_note = store
        .create_note("Mid1 Note", None, &["mid1".to_string()], None)
        .unwrap();
    store.save_note(&mut mid1_note).unwrap();

    let mut mid2_note = store
        .create_note("Mid2 Note", None, &["mid2".to_string()], None)
        .unwrap();
    store.save_note(&mut mid2_note).unwrap();

    let mut to_note = store
        .create_note("To Note", None, &["to".to_string()], None)
        .unwrap();
    store.save_note(&mut to_note).unwrap();

    // Create links: from -> mid1 -> mid2 -> to (3 hops)
    from_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: mid1_note.id().to_string(),
        });
    store.save_note(&mut from_note).unwrap();

    mid1_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: mid2_note.id().to_string(),
        });
    store.save_note(&mut mid1_note).unwrap();

    mid2_note
        .frontmatter
        .links
        .push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: to_note.id().to_string(),
        });
    store.save_note(&mut mid2_note).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        ignore_value: true,
        max_hops: HopCost::from(2), // Limit to 2 hops, path needs 3
        ..Default::default()
    };

    let result = bfs_find_path(
        &index,
        &store,
        from_note.id(),
        to_note.id(),
        &opts,
        None,
        None,
    )
    .unwrap();

    assert!(!result.found); // Should not find path within max_hops limit
}
