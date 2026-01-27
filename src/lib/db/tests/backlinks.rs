use crate::lib::index::types::LinkSource;
use crate::lib::note::TypedLink;
use crate::lib::store::Store;
use tempfile::tempdir;

#[test]
fn test_get_backlinks() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let note1 = store.create_note("Source Note", None, &[], None).unwrap();
    let note2 = store.create_note("Target Note", None, &[], None).unwrap();
    let note3 = store
        .create_note("Another Source", None, &[], None)
        .unwrap();

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
    let backlinks = db.get_backlinks(note2_id).unwrap();

    assert_eq!(backlinks.len(), 2);

    let backlink1 = backlinks
        .iter()
        .find(|e| e.from == note1_id)
        .expect("Expected backlink from note1");
    assert_eq!(backlink1.to, note2_id);
    assert_eq!(backlink1.link_type.as_str(), "related");
    assert_eq!(backlink1.source, LinkSource::Typed);

    let backlink2 = backlinks
        .iter()
        .find(|e| e.from == note3_id)
        .expect("Expected backlink from note3");
    assert_eq!(backlink2.to, note2_id);
    assert_eq!(backlink2.link_type.as_str(), "related");
    assert_eq!(backlink2.source, LinkSource::Typed);
}
