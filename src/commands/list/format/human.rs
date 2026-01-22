//! Human-readable output formatting for list command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    _store: &Store,
    notes: &[crate::lib::note::Note],
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &crate::lib::note::Note>,
) {
    if notes.is_empty() {
        if !cli.quiet {
            println!("No notes found");
        }
        return;
    }

    for note in notes {
        let type_indicator = match note.note_type() {
            NoteType::Fleeting => "F",
            NoteType::Literature => "L",
            NoteType::Permanent => "P",
            NoteType::Moc => "M",
        };

        let annotations = build_compaction_annotations(note, compaction_ctx, note_map);

        println!(
            "{} [{}] {}{}",
            note.id(),
            type_indicator,
            note.title(),
            annotations
        );

        output_compaction_ids(cli, note, compaction_ctx);
    }
}

/// Build compaction annotations for a note
fn build_compaction_annotations(
    note: &crate::lib::note::Note,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &crate::lib::note::Note>,
) -> String {
    let mut annotations = String::new();
    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);

    if compacts_count > 0 {
        annotations.push_str(&format!(" compacts={}", compacts_count));

        if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
            annotations.push_str(&format!(" compaction={:.0}%", pct));
        }
    }

    annotations
}

/// Output compacted IDs if requested
fn output_compaction_ids(
    cli: &Cli,
    note: &crate::lib::note::Note,
    compaction_ctx: &CompactionContext,
) {
    if !cli.with_compaction_ids {
        return;
    }

    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count == 0 {
        return;
    }

    let depth = cli.compaction_depth.unwrap_or(1);
    if let Some((ids, truncated)) =
        compaction_ctx.get_compacted_ids(&note.frontmatter.id, depth, cli.compaction_max_nodes)
    {
        let ids_str = ids.join(", ");
        let suffix = if truncated {
            let max = cli.compaction_max_nodes.unwrap_or(ids.len());
            format!(" (truncated, showing {} of {})", max, compacts_count)
        } else {
            String::new()
        };
        println!("  Compacted: {}{}", ids_str, suffix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_compaction_annotations_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store =
            Store::init(temp_dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let annotations = build_compaction_annotations(&note, &compaction_ctx, &note_map);
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_build_compaction_annotations_with_compacts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store =
            Store::init(temp_dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store
            .create_note_with_content(
                "Original Note",
                None,
                &["original".to_string()],
                "# Summary\nContent from original note.",
                None,
            )
            .unwrap();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1.id().to_string()];
        store.save_note(&mut digest).unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let digest_note = all_notes
            .iter()
            .find(|n| n.frontmatter.compacts.iter().any(|id| id == note1.id()))
            .unwrap();
        let annotations = build_compaction_annotations(digest_note, &compaction_ctx, &note_map);

        assert!(annotations.contains("compacts=1"));
        assert!(annotations.contains("compaction="));
    }
}
