use super::*;
use crate::lib::note::{LinkType, TypedLink};
use crate::lib::store::InitOptions;
use tempfile::tempdir;

#[test]
fn test_doctor_duplicate_ids() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    store.create_note("Note 1", None, &[], None).unwrap();
    store.create_note("Note 2", None, &[], None).unwrap();

    let mut result = DoctorResult::new();
    check_duplicate_ids(&store, &mut result);

    assert_eq!(result.error_count, 0);
}

#[test]
fn test_doctor_broken_links() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note = store.create_note("Test Note", None, &[], None).unwrap();
    note.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::RELATED),
        id: "qp-missing".to_string(),
    }];
    note.body = "See [[qp-also-missing]]".to_string();

    store.save_note(&mut note).unwrap();

    let mut result = DoctorResult::new();
    check_broken_links(&store, &mut result);

    assert!(result.error_count >= 1);
}

#[test]
fn test_doctor_semantic_link_conflicting_support_contradict() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let mut note2 = store.create_note("Note 2", None, &[], None).unwrap();

    note2.frontmatter.links = vec![
        TypedLink {
            link_type: LinkType::from(LinkType::SUPPORTS),
            id: note1.frontmatter.id.clone(),
        },
        TypedLink {
            link_type: LinkType::from(LinkType::CONTRADICTS),
            id: note1.frontmatter.id.clone(),
        },
    ];

    store.save_note(&mut note2).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert!(result.warning_count >= 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "semantic-link-misuse"
            && i.message.contains("both supports and contradicts")));
}

#[test]
fn test_doctor_semantic_link_self_referential_same_as() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note = store.create_note("Note 1", None, &[], None).unwrap();
    note.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::SAME_AS),
        id: note.frontmatter.id.clone(),
    }];

    store.save_note(&mut note).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert!(result.warning_count >= 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "semantic-link-misuse"
            && i.message.contains("self-referential 'same-as'")));
}

#[test]
fn test_doctor_semantic_link_self_referential_alias_of() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note = store.create_note("Note 1", None, &[], None).unwrap();
    note.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::ALIAS_OF),
        id: note.frontmatter.id.clone(),
    }];

    store.save_note(&mut note).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert!(result.warning_count >= 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "semantic-link-misuse"
            && i.message.contains("self-referential 'alias-of'")));
}

#[test]
fn test_doctor_semantic_link_mixed_identity_types() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let mut note2 = store.create_note("Note 2", None, &[], None).unwrap();

    note2.frontmatter.links = vec![
        TypedLink {
            link_type: LinkType::from(LinkType::SAME_AS),
            id: note1.frontmatter.id.clone(),
        },
        TypedLink {
            link_type: LinkType::from(LinkType::ALIAS_OF),
            id: note1.frontmatter.id.clone(),
        },
    ];

    store.save_note(&mut note2).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert!(result.warning_count >= 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "semantic-link-misuse"
            && i.message.contains("both 'same-as' and 'alias-of'")));
}

#[test]
fn test_doctor_semantic_link_valid_relationships() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let mut note3 = store.create_note("Note 3", None, &[], None).unwrap();

    note3.frontmatter.links = vec![
        TypedLink {
            link_type: LinkType::from(LinkType::SUPPORTS),
            id: note1.frontmatter.id.clone(),
        },
        TypedLink {
            link_type: LinkType::from(LinkType::PART_OF),
            id: note2.frontmatter.id.clone(),
        },
    ];

    store.save_note(&mut note3).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert_eq!(
        result
            .issues
            .iter()
            .filter(|i| i.category == "semantic-link-misuse")
            .count(),
        0
    );
}

#[test]
fn test_doctor_semantic_link_part_of_self_loop() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note = store.create_note("Note 1", None, &[], None).unwrap();
    note.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::PART_OF),
        id: note.frontmatter.id.clone(),
    }];

    store.save_note(&mut note).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert!(result.warning_count >= 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "semantic-link-misuse"
            && i.message.contains("self-referential 'part-of'")));
}

#[test]
fn test_doctor_semantic_link_follows_cycle() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let mut note1_mut = note1.clone();
    note1_mut.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::FOLLOWS),
        id: note2.frontmatter.id.clone(),
    }];
    store.save_note(&mut note1_mut).unwrap();

    let mut note2_mut = note2.clone();
    note2_mut.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::FOLLOWS),
        id: note3.frontmatter.id.clone(),
    }];
    store.save_note(&mut note2_mut).unwrap();

    let mut note3_mut = note3.clone();
    note3_mut.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::FOLLOWS),
        id: note1.frontmatter.id.clone(),
    }];
    store.save_note(&mut note3_mut).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert!(result.warning_count >= 1);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.category == "semantic-link-misuse"
                && i.message.contains("'follows' cycle"))
    );
}

#[test]
fn test_doctor_semantic_link_follows_no_cycle() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note1 = store.create_note("Note 1", None, &[], None).unwrap();
    let note2 = store.create_note("Note 2", None, &[], None).unwrap();
    let note3 = store.create_note("Note 3", None, &[], None).unwrap();

    let mut note1_mut = note1.clone();
    note1_mut.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::FOLLOWS),
        id: note2.frontmatter.id.clone(),
    }];
    store.save_note(&mut note1_mut).unwrap();

    let mut note2_mut = note2.clone();
    note2_mut.frontmatter.links = vec![TypedLink {
        link_type: LinkType::from(LinkType::FOLLOWS),
        id: note3.frontmatter.id.clone(),
    }];
    store.save_note(&mut note2_mut).unwrap();

    let mut result = DoctorResult::new();
    check_semantic_link_types(&store, &mut result);

    assert_eq!(
        result
            .issues
            .iter()
            .filter(|i| i.category == "semantic-link-misuse"
                && i.message.contains("'follows' cycle"))
            .count(),
        0
    );
}
