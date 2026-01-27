use crate::lib::graph::types::Direction;
use crate::lib::note::TypedLink;
use crate::lib::store::Store;
use tempfile::tempdir;

#[test]
fn test_traverse_outbound() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

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
        link_type: crate::lib::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.get_note(note2_id).unwrap();
    note2.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("supports"),
        id: note3_id.to_string(),
    });
    store.save_note(&mut note2).unwrap();

    let mut note3 = store.get_note(note3_id).unwrap();
    note3.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note4_id.to_string(),
    });
    store.save_note(&mut note3).unwrap();

    let db = store.db();
    let reachable = db.traverse(note1_id, Direction::Out, 3, None).unwrap();

    assert_eq!(reachable.len(), 4);
    assert!(reachable.iter().any(|id| id == note1_id));
    assert!(reachable.iter().any(|id| id == note2_id));
    assert!(reachable.iter().any(|id| id == note3_id));
    assert!(reachable.iter().any(|id| id == note4_id));
}

#[test]
fn test_traverse_inbound() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note3 = store.get_note(note3_id).unwrap();
    note3.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("supports"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note3).unwrap();

    let db = store.db();
    let reachable = db.traverse(note2_id, Direction::In, 3, None).unwrap();

    assert_eq!(reachable.len(), 3);
    assert!(reachable.iter().any(|id| id == note1_id));
    assert!(reachable.iter().any(|id| id == note2_id));
    assert!(reachable.iter().any(|id| id == note3_id));
}

#[test]
fn test_traverse_both_directions() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note3 = store.get_note(note3_id).unwrap();
    note3.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note3).unwrap();

    let db = store.db();
    let reachable = db.traverse(note2_id, Direction::Both, 3, None).unwrap();

    assert_eq!(reachable.len(), 3);
    assert!(reachable.iter().any(|id| id == note1_id));
    assert!(reachable.iter().any(|id| id == note2_id));
    assert!(reachable.iter().any(|id| id == note3_id));
}

#[test]
fn test_traverse_max_hops() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.get_note(note2_id).unwrap();
    note2.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note3_id.to_string(),
    });
    store.save_note(&mut note2).unwrap();

    let db = store.db();
    let reachable = db.traverse(note1_id, Direction::Out, 1, None).unwrap();

    assert_eq!(reachable.len(), 2);
    assert!(reachable.iter().any(|id| id == note1_id));
    assert!(reachable.iter().any(|id| id == note2_id));
    assert!(!reachable.iter().any(|id| id == note3_id));
}

#[test]
fn test_traverse_max_nodes() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let note1_id = note1.id();
    let note2_id = note2.id();
    let note3_id = note3.id();

    let mut note1 = store.get_note(note1_id).unwrap();
    note1.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note2_id.to_string(),
    });
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.get_note(note2_id).unwrap();
    note2.frontmatter.links.push(TypedLink {
        link_type: crate::lib::note::LinkType::from("related"),
        id: note3_id.to_string(),
    });
    store.save_note(&mut note2).unwrap();

    let db = store.db();
    let reachable = db.traverse(note1_id, Direction::Out, 3, Some(2)).unwrap();

    assert_eq!(reachable.len(), 2);
    assert!(reachable.iter().any(|id| id == note1_id));
    assert!(reachable.iter().any(|id| id == note2_id));
}
