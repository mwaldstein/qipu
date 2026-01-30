use crate::graph::bfs_traverse;
use crate::graph::types::{Direction, HopCost, TreeOptions};
use crate::index::IndexBuilder;
use crate::note::TypedLink;
use crate::store::Store;
use tempfile::tempdir;

#[test]
fn test_traverse_outbound() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();
    let note4 = store.create_note("Note 4", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();
    let note4_id = note4.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.get_note(note2_id).unwrap();
    note2.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("supports"),
        id: note3_id.to_string(),
    });
    store.save_note(&mut note2).unwrap();

    let mut note3 = store.get_note(note3_id).unwrap();
    note3.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note4_id.to_string(),
    });
    store.save_note(&mut note3).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(3),
        ignore_value: true,
        ..Default::default()
    };

    let result = bfs_traverse(&index, &store, note1_id, &opts, None, None).unwrap();

    assert_eq!(result.notes.len(), 4);
    assert!(result.notes.iter().any(|n| n.id == note1_id));
    assert!(result.notes.iter().any(|n| n.id == note2_id));
    assert!(result.notes.iter().any(|n| n.id == note3_id));
    assert!(result.notes.iter().any(|n| n.id == note4_id));
}

#[test]
fn test_traverse_inbound() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note3 = store.get_note(note3_id).unwrap();
    note3.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("supports"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note3).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::In,
        max_hops: HopCost::from(3),
        ignore_value: true,
        ..Default::default()
    };

    let result = bfs_traverse(&index, &store, note2_id, &opts, None, None).unwrap();

    assert_eq!(result.notes.len(), 3);
    assert!(result.notes.iter().any(|n| n.id == note1_id));
    assert!(result.notes.iter().any(|n| n.id == note2_id));
    assert!(result.notes.iter().any(|n| n.id == note3_id));
}

#[test]
fn test_traverse_both_directions() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note3 = store.get_note(note3_id).unwrap();
    note3.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note3).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Both,
        max_hops: HopCost::from(3),
        ignore_value: true,
        ..Default::default()
    };

    let result = bfs_traverse(&index, &store, note2_id, &opts, None, None).unwrap();

    assert_eq!(result.notes.len(), 3);
    assert!(result.notes.iter().any(|n| n.id == note1_id));
    assert!(result.notes.iter().any(|n| n.id == note2_id));
    assert!(result.notes.iter().any(|n| n.id == note3_id));
}

#[test]
fn test_traverse_max_hops() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.get_note(note2_id).unwrap();
    note2.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note3_id.to_string(),
    });
    store.save_note(&mut note2).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(1),
        ignore_value: true,
        ..Default::default()
    };

    let result = bfs_traverse(&index, &store, note1_id, &opts, None, None).unwrap();

    assert_eq!(result.notes.len(), 2);
    assert!(result.notes.iter().any(|n| n.id == note1_id));
    assert!(result.notes.iter().any(|n| n.id == note2_id));
    assert!(!result.notes.iter().any(|n| n.id == note3_id));
}

#[test]
fn test_traverse_max_nodes() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.get_note(note2_id).unwrap();
    note2.frontmatter.links.push(TypedLink {
        link_type: crate::note::LinkType::from("related"),
        id: note3_id.to_string(),
    });
    store.save_note(&mut note2).unwrap();

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(3),
        max_nodes: Some(2),
        ignore_value: true,
        ..Default::default()
    };

    let result = bfs_traverse(&index, &store, note1_id, &opts, None, None).unwrap();

    assert_eq!(result.notes.len(), 2);
    assert!(result.notes.iter().any(|n| n.id == note1_id));
    assert!(result.notes.iter().any(|n| n.id == note2_id));
}
