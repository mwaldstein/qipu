use super::*;
use qipu_core::note::NoteFrontmatter;
use qipu_core::store::InitOptions;
use tempfile::tempdir;

fn test_store() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();
    (dir, store)
}

fn test_note(store: &Store, title: &str, body: &str) -> Note {
    let mut note = store.create_note(title, None, &[], None).unwrap();
    note.body = body.to_string();
    store.save_note(&mut note).unwrap();
    note
}

#[test]
fn test_doctor_bare_link_lists() {
    let (_dir, store) = test_store();
    test_note(&store, "Note 1", "- [[qp-2]]\n- [[qp-3]]\n");
    let notes = scan_notes(&store).0;
    let mut result = DoctorResult::new();
    check_bare_link_lists(&notes, &mut result);
    assert!(result.warning_count >= 1);
    assert!(result.issues.iter().any(|i| i.category == "bare-link-list"));
}

#[test]
fn test_doctor_bare_link_lists_with_context() {
    let (_dir, store) = test_store();
    test_note(
        &store,
        "Note 1",
        "- See [[qp-2]] for more details on this topic\n- [[qp-3]] explains the counter-argument\n",
    );
    let notes = scan_notes(&store).0;
    let mut result = DoctorResult::new();
    check_bare_link_lists(&notes, &mut result);
    assert_eq!(
        result
            .issues
            .iter()
            .filter(|i| i.category == "bare-link-list")
            .count(),
        0
    );
}

#[test]
fn test_doctor_note_complexity_too_long() {
    let (_dir, store) = test_store();
    let long = "word ".repeat(1600);
    test_note(
        &store,
        "Note 1",
        &format!("{}\n\nThis note is very long.", long),
    );
    let notes = scan_notes(&store).0;
    let mut result = DoctorResult::new();
    check_note_complexity(&notes, &mut result);
    assert!(result.warning_count >= 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "note-complexity"));
}

#[test]
fn test_doctor_note_complexity_normal() {
    let (_dir, store) = test_store();
    test_note(
        &store,
        "Note 1",
        "This is a normal note with reasonable length.",
    );
    let notes = scan_notes(&store).0;
    let mut result = DoctorResult::new();
    check_note_complexity(&notes, &mut result);
    assert_eq!(
        result
            .issues
            .iter()
            .filter(|i| i.category == "note-complexity")
            .count(),
        0
    );
}

#[test]
fn test_doctor_compaction_cycle() {
    let mut n1 = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
    n1.compacts = vec!["qp-2".to_string()];
    let mut n2 = NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string());
    n2.compacts = vec!["qp-1".to_string()];
    let notes = vec![Note::new(n1, String::new()), Note::new(n2, String::new())];
    let mut result = DoctorResult::new();
    check_compaction_invariants(&notes, &mut result);
    assert!(result.error_count > 0);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "compaction-invariant" && i.message.contains("cycle")));
}

#[test]
fn test_doctor_compaction_self_compaction() {
    let mut n = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
    n.compacts = vec!["qp-1".to_string()];
    let notes = vec![Note::new(n, String::new())];
    let mut result = DoctorResult::new();
    check_compaction_invariants(&notes, &mut result);
    assert!(result.error_count > 0);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "compaction-invariant" && i.message.contains("compacts itself")));
}

#[test]
fn test_doctor_compaction_multiple_compactors() {
    let mut d1 = NoteFrontmatter::new("qp-d1".to_string(), "Digest 1".to_string());
    d1.compacts = vec!["qp-1".to_string()];
    let mut d2 = NoteFrontmatter::new("qp-d2".to_string(), "Digest 2".to_string());
    d2.compacts = vec!["qp-1".to_string()];
    let notes = vec![
        Note::new(
            NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
            String::new(),
        ),
        Note::new(d1, String::new()),
        Note::new(d2, String::new()),
    ];
    let mut result = DoctorResult::new();
    check_compaction_invariants(&notes, &mut result);
    assert!(result.error_count > 0);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.category == "compaction-invariant"
                && i.message.contains("multiple compactors"))
    );
}

#[test]
fn test_doctor_compaction_valid() {
    let mut d = NoteFrontmatter::new("qp-digest".to_string(), "Digest".to_string());
    d.compacts = vec!["qp-1".to_string(), "qp-2".to_string()];
    let notes = vec![
        Note::new(
            NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
            String::new(),
        ),
        Note::new(
            NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string()),
            String::new(),
        ),
        Note::new(d, String::new()),
    ];
    let mut result = DoctorResult::new();
    check_compaction_invariants(&notes, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_doctor_value_range_invalid() {
    let mut n = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
    n.value = Some(150);
    let mut result = DoctorResult::new();
    check_value_range(&[Note::new(n, String::new())], &mut result);
    assert_eq!(result.error_count, 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "invalid-value" && i.message.contains("150")));
}

#[test]
fn test_doctor_value_range_valid() {
    let mut n1 = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
    n1.value = Some(100);
    let mut n2 = NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string());
    n2.value = Some(0);
    let mut n3 = NoteFrontmatter::new("qp-3".to_string(), "Note 3".to_string());
    n3.value = Some(50);
    let mut result = DoctorResult::new();
    check_value_range(
        &[
            Note::new(n1, String::new()),
            Note::new(n2, String::new()),
            Note::new(n3, String::new()),
        ],
        &mut result,
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_doctor_value_range_none() {
    let n = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
    let mut result = DoctorResult::new();
    check_value_range(&[Note::new(n, String::new())], &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_doctor_value_range_boundary() {
    let mut n1 = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
    n1.value = Some(100);
    let mut n2 = NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string());
    n2.value = Some(101);
    let mut result = DoctorResult::new();
    check_value_range(
        &[Note::new(n1, String::new()), Note::new(n2, String::new())],
        &mut result,
    );
    assert_eq!(result.error_count, 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "invalid-value" && i.message.contains("101")));
}

#[test]
fn test_doctor_attachments() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();
    let att_dir = store.root().join(ATTACHMENTS_DIR);
    fs::write(att_dir.join("valid.png"), "dummy data").unwrap();

    let mut n1 = Note::new(
        NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
        "![Valid](../attachments/valid.png)".to_string(),
    );
    n1.path = Some(store.notes_dir().join("qp-1.md"));

    let mut n2 = Note::new(
        NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string()),
        "![Broken](../attachments/missing.jpg)".to_string(),
    );
    n2.path = Some(store.notes_dir().join("qp-2.md"));

    fs::write(att_dir.join("orphaned.txt"), "nobody loves me").unwrap();

    let mut result = DoctorResult::new();
    check_attachments(&store, &[n1, n2], &mut result);

    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "broken-attachment" && i.message.contains("missing.jpg")));
    assert!(result
        .issues
        .iter()
        .any(|i| i.category == "orphaned-attachment" && i.message.contains("orphaned.txt")));
}
