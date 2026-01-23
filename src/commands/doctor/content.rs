use super::types::{CheckContext, DoctorCheck, DoctorResult, Issue, Severity};
use crate::lib::compaction::CompactionContext;
use crate::lib::index::Index;
use crate::lib::note::Note;
use crate::lib::store::paths::ATTACHMENTS_DIR;
use crate::lib::store::Store;
use regex::Regex;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn check_tag_aliases(store: &Store, notes: &[Note], result: &mut DoctorResult) {
    let config = store.config();

    if config.tag_aliases.is_empty() {
        return;
    }

    let mut all_tags: BTreeSet<String> = BTreeSet::new();
    for note in notes {
        for tag in &note.frontmatter.tags {
            all_tags.insert(tag.clone());
        }
    }

    for (alias, canonical) in &config.tag_aliases {
        if !all_tags.contains(canonical) {
            result.add_issue(Issue {
                severity: Severity::Warning,
                category: "orphaned-tag-alias".to_string(),
                message: format!(
                    "Tag alias '{}' points to canonical tag '{}' which does not exist in any note",
                    alias, canonical
                ),
                note_id: None,
                path: None,
                fixable: false,
            });
        }
    }
}

pub fn scan_notes(store: &Store) -> (Vec<Note>, Vec<(String, String)>) {
    let mut notes = Vec::new();
    let mut errors = Vec::new();

    for dir in [store.notes_dir(), store.mocs_dir()] {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(&dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                match fs::read_to_string(path) {
                    Ok(content) => match Note::parse(&content, Some(path.to_path_buf())) {
                        Ok(note) => notes.push(note),
                        Err(e) => {
                            errors.push((path.display().to_string(), e.to_string()));
                        }
                    },
                    Err(e) => {
                        errors.push((path.display().to_string(), format!("Failed to read: {}", e)));
                    }
                }
            }
        }
    }

    (notes, errors)
}

pub fn check_required_fields(notes: &[Note], result: &mut DoctorResult) {
    for note in notes {
        let path = note
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if note.id().is_empty() {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "missing-field".to_string(),
                message: format!("Note at '{}' is missing required 'id' field", path),
                note_id: None,
                path: Some(path.clone()),
                fixable: false,
            });
        }

        if note.title().is_empty() {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "missing-field".to_string(),
                message: format!("Note '{}' is missing required 'title' field", note.id()),
                note_id: Some(note.id_string()),
                path: Some(path.clone()),
                fixable: false,
            });
        }
    }
}

pub fn check_compaction_invariants(notes: &[Note], result: &mut DoctorResult) {
    let compaction_ctx = match CompactionContext::build(notes) {
        Ok(ctx) => ctx,
        Err(e) => {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "compaction-invariant".to_string(),
                message: e.to_string(),
                note_id: None,
                path: None,
                fixable: false,
            });
            return;
        }
    };

    let errors = compaction_ctx.validate(notes);
    for error in errors {
        result.add_issue(Issue {
            severity: Severity::Error,
            category: "compaction-invariant".to_string(),
            message: error,
            note_id: None,
            path: None,
            fixable: false,
        });
    }
}

pub fn check_near_duplicates(index: &Index, threshold: f64, result: &mut DoctorResult) {
    use crate::lib::similarity::find_all_duplicates;
    let duplicates = find_all_duplicates(index, threshold);

    for (id1, id2, score) in duplicates {
        result.add_issue(Issue {
            severity: Severity::Warning,
            category: "near-duplicate".to_string(),
            message: format!(
                "Notes '{}' and '{}' are similar ({:.2}%)",
                id1,
                id2,
                score * 100.0
            ),
            note_id: Some(id1),
            path: None,
            fixable: false,
        });
    }
}

pub fn check_value_range(notes: &[Note], result: &mut DoctorResult) {
    for note in notes {
        if let Some(value) = note.frontmatter.value {
            if value > 100 {
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "invalid-value".to_string(),
                    message: format!(
                        "Note '{}' has invalid value: {} (must be 0-100)",
                        note.id(),
                        value
                    ),
                    note_id: Some(note.id_string()),
                    path: note.path_display(),
                    fixable: false,
                });
            }
        }
    }
}

pub fn check_custom_metadata(notes: &[Note], result: &mut DoctorResult) {
    for note in notes {
        if note.frontmatter.custom.is_empty() {
            continue;
        }

        // Check if custom field size is reasonable (warn if >10KB)
        if let Ok(json_str) = serde_json::to_string(&note.frontmatter.custom) {
            if json_str.len() > 10 * 1024 {
                result.add_issue(Issue {
                    severity: Severity::Warning,
                    category: "large-custom-metadata".to_string(),
                    message: format!(
                        "Note '{}' has large custom metadata block ({} bytes, >10KB)",
                        note.id(),
                        json_str.len()
                    ),
                    note_id: Some(note.id_string()),
                    path: note.path_display(),
                    fixable: false,
                });
            }
        }
    }
}

pub fn check_attachments(store: &Store, notes: &[Note], result: &mut DoctorResult) {
    let attachments_dir = store.root().join(ATTACHMENTS_DIR);
    let mut referenced_attachments = HashSet::new();
    let attachment_re = Regex::new(r"\[[^\]]*\]\(([^)]*attachments/[^)]+)\)")
        .expect("Invalid attachment regex pattern");

    for note in notes {
        let from_id = note.id_string();
        let note_path = note
            .path
            .as_ref()
            .map(|p| p.parent().unwrap_or(Path::new("")))
            .unwrap_or(Path::new(""));

        for cap in attachment_re.captures_iter(&note.body) {
            let rel_path_str = &cap[1];
            let full_path = note_path.join(rel_path_str);

            if let Ok(canonical_path) = fs::canonicalize(&full_path) {
                if let Ok(canonical_attachments_dir) = fs::canonicalize(&attachments_dir) {
                    if canonical_path.starts_with(&canonical_attachments_dir) {
                        referenced_attachments.insert(canonical_path.clone());

                        if !canonical_path.exists() {
                            result.add_issue(Issue {
                                severity: Severity::Error,
                                category: "broken-attachment".to_string(),
                                message: format!(
                                    "Note '{}' references missing attachment: {}",
                                    from_id, rel_path_str
                                ),
                                note_id: Some(from_id.clone()),
                                path: note.path_display(),
                                fixable: false,
                            });
                        }
                    }
                }
            } else {
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "broken-attachment".to_string(),
                    message: format!(
                        "Note '{}' references missing or invalid attachment: {}",
                        from_id, rel_path_str
                    ),
                    note_id: Some(from_id.clone()),
                    path: note.path_display(),
                    fixable: false,
                });
            }
        }
    }

    if attachments_dir.exists() {
        for entry in WalkDir::new(&attachments_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(canonical_path) = fs::canonicalize(path) {
                    if !referenced_attachments.contains(&canonical_path) {
                        result.add_issue(Issue {
                            severity: Severity::Warning,
                            category: "orphaned-attachment".to_string(),
                            message: format!(
                                "Orphaned attachment found: {}",
                                path.strip_prefix(&attachments_dir)
                                    .unwrap_or(path)
                                    .display()
                            ),
                            note_id: None,
                            path: Some(path.display().to_string()),
                            fixable: true,
                        });
                    }
                }
            }
        }
    }
}

pub fn check_bare_link_lists(notes: &[Note], result: &mut DoctorResult) {
    let wiki_link_re = Regex::new(r"\[\[([^\]]+)\]\]").expect("Invalid wiki link regex pattern");

    for note in notes {
        let lines: Vec<&str> = note.body.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let line_without_marker = trimmed[2..].trim();
                let links: Vec<&str> = wiki_link_re
                    .find_iter(line_without_marker)
                    .map(|m| m.as_str())
                    .collect();

                if !links.is_empty() {
                    let non_link_content = wiki_link_re.replace_all(line_without_marker, "");
                    let non_link_content = non_link_content.trim();

                    if non_link_content.is_empty() || non_link_content.len() < 20 {
                        result.add_issue(Issue {
                            severity: Severity::Warning,
                            category: "bare-link-list".to_string(),
                            message: format!(
                                "Note '{}' contains bare link list at line {}: links should include descriptive text explaining why they are relevant",
                                note.id(),
                                line_idx + 1
                            ),
                            note_id: Some(note.id_string()),
                            path: note.path_display(),
                            fixable: false,
                        });
                        break;
                    }
                }
            }
        }
    }
}

pub fn check_note_complexity(notes: &[Note], result: &mut DoctorResult) {
    const MAX_WORDS: usize = 1500;
    const MAX_PARAGRAPHS: usize = 50;

    for note in notes {
        let word_count = note.body.split_whitespace().count();
        let paragraph_count = note
            .body
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count();

        if word_count > MAX_WORDS || paragraph_count > MAX_PARAGRAPHS {
            let reason = if word_count > MAX_WORDS {
                format!("{} words", word_count)
            } else {
                format!("{} paragraphs", paragraph_count)
            };

            result.add_issue(Issue {
                severity: Severity::Warning,
                category: "note-complexity".to_string(),
                message: format!(
                    "Note '{}' may be too long ({}) - consider splitting into multiple notes (prefer 'one idea per note')",
                    note.id(),
                    reason
                ),
                note_id: Some(note.id_string()),
                path: note.path_display(),
                fixable: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::NoteFrontmatter;
    use crate::lib::store::InitOptions;
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
        test_note(&store, "Note 1", "- See [[qp-2]] for more details on this topic\n- [[qp-3]] explains the counter-argument\n");
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
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.category == "compaction-invariant"
                    && i.message.contains("compacts itself"))
        );
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
        assert!(result
            .issues
            .iter()
            .any(|i| i.category == "compaction-invariant"
                && i.message.contains("multiple compactors")));
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
}

macro_rules! impl_doctor_check {
    ($struct_name:ident, $name:expr, $description:expr, $check_fn:ident, notes) => {
        pub struct $struct_name;

        impl DoctorCheck for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn description(&self) -> &str {
                $description
            }

            fn run(&self, ctx: &CheckContext<'_>, result: &mut DoctorResult) {
                let Some(notes) = ctx.notes else { return };
                $check_fn(notes, result);
            }
        }
    };
    ($struct_name:ident, $name:expr, $description:expr, $check_fn:ident, index_threshold) => {
        pub struct $struct_name;

        impl DoctorCheck for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn description(&self) -> &str {
                $description
            }

            fn run(&self, ctx: &CheckContext<'_>, result: &mut DoctorResult) {
                let Some(index) = ctx.index else { return };
                let Some(threshold) = ctx.threshold else {
                    return;
                };
                $check_fn(index, threshold, result);
            }
        }
    };
}

impl_doctor_check!(
    CheckRequiredFields,
    "required-fields",
    "Validates that all notes have required frontmatter fields (id, title)",
    check_required_fields,
    notes
);

impl_doctor_check!(
    CheckValueRange,
    "value-range",
    "Validates that note values are within the valid range (0-100)",
    check_value_range,
    notes
);

impl_doctor_check!(
    CheckCustomMetadata,
    "custom-metadata",
    "Validates custom metadata size and structure",
    check_custom_metadata,
    notes
);

impl_doctor_check!(
    CheckCompactionInvariants,
    "compaction-invariants",
    "Validates compaction graph invariants (no cycles, no self-compaction, no multiple compactors)",
    check_compaction_invariants,
    notes
);

impl_doctor_check!(
    CheckBareLinkLists,
    "bare-link-lists",
    "Warns about bare link lists without descriptive context",
    check_bare_link_lists,
    notes
);

impl_doctor_check!(
    CheckNoteComplexity,
    "note-complexity",
    "Warns about overly complex or long notes",
    check_note_complexity,
    notes
);

impl_doctor_check!(
    CheckNearDuplicates,
    "near-duplicates",
    "Finds near-duplicate notes based on content similarity",
    check_near_duplicates,
    index_threshold
);
