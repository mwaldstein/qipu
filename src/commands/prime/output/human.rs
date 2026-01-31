//! Human-readable output formatting for prime command

use qipu_core::config::OntologyConfig;
use qipu_core::note::NoteType;
use qipu_core::ontology::Ontology;

pub fn build_base_human(store_path: &str, is_empty: bool) -> String {
    let getting_started = getting_started_section(is_empty);
    format!(
        "# Qipu Knowledge Store Primer\n\nStore: {}\n\n## About Qipu\n\nQipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\n\n{}## Quick Reference\n\n  qipu list              List notes\n  qipu search <query>    Search notes by title and body\n  qipu show <id>         Display a note\n  qipu create <title>    Create a new note\n  qipu capture           Create note from stdin\n  qipu link tree <id>    Show traversal tree from a note\n  qipu link path A B     Find path between notes\n  qipu context           Build context bundle for LLM\n\n## Session Protocol\n\n**Before ending session:**\n1. Capture any new insights: `qipu capture --title \"...\"`\n2. Link new notes to existing knowledge: `qipu link add <new> <existing> --type <type>`\n3. Commit changes: `git add .qipu && git commit -m \"knowledge: ...\"`\n\n**Why this matters:** Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n\n",
        store_path, getting_started
    )
}

fn getting_started_section(is_empty: bool) -> String {
    if is_empty {
        "## Getting Started\n\nYour knowledge store is empty. Start by capturing your first insights:\n\n  qipu capture --title \"...\"     Quick capture from stdin\n  qipu create <title>             Create a new note\n  qipu create <title> --type moc   Create a Map of Content\n\n".to_string()
    } else {
        String::new()
    }
}

pub fn output_human(
    store_path: &str,
    ontology: &Ontology,
    config: &OntologyConfig,
    mocs: &[&qipu_core::note::Note],
    recent_notes: &[&qipu_core::note::Note],
    compact: bool,
    is_empty: bool,
) {
    let base = build_base_human(store_path, is_empty);
    print!("{}", base);

    println!("## Ontology");
    println!();
    println!("Mode: {}", config.mode);
    println!();

    print_note_types(ontology, config);
    println!();

    print_link_types(ontology, config);

    let has_mocs = !compact && !mocs.is_empty();
    if has_mocs {
        println!();
        print_mocs_section(mocs, compact);
    }

    let has_recent_notes = !recent_notes.is_empty();
    if has_recent_notes {
        println!();
        print_recent_notes_section(recent_notes);
    }

    println!();

    println!("Use `qipu context --note <id>` to fetch full note content.");
}

fn print_note_types(ontology: &Ontology, config: &OntologyConfig) {
    let note_types = ontology.note_types();
    println!("### Note Types");
    for nt in &note_types {
        let type_config = config.note_types.get(nt);
        if let Some(desc) = type_config.and_then(|c| c.description.as_deref()) {
            println!("  {} - {}", nt, desc);
        } else {
            println!("  {}", nt);
        }
        if let Some(usage) = type_config.and_then(|c| c.usage.as_deref()) {
            println!("    Usage: {}", usage);
        }
    }
}

fn print_link_types(ontology: &Ontology, config: &OntologyConfig) {
    let link_types = ontology.link_types();
    println!("### Link Types");
    for lt in &link_types {
        let inverse = ontology.get_inverse(lt);
        let type_config = config.link_types.get(lt);
        if let Some(desc) = type_config.and_then(|c| c.description.as_deref()) {
            println!("  {} -> {} ({})", lt, inverse, desc);
        } else {
            println!("  {} -> {}", lt, inverse);
        }
        if let Some(usage) = type_config.and_then(|c| c.usage.as_deref()) {
            println!("    Usage: {}", usage);
        }
    }
}

fn print_mocs_section(mocs: &[&qipu_core::note::Note], compact: bool) {
    if !compact && !mocs.is_empty() {
        println!("## Key Maps of Content");
        println!();
        for moc in mocs {
            let tags = if moc.frontmatter.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", moc.frontmatter.tags.join(", "))
            };
            println!("  {} - {}{}", moc.id(), moc.title(), tags);
        }
    }
}

fn print_recent_notes_section(recent_notes: &[&qipu_core::note::Note]) {
    if !recent_notes.is_empty() {
        println!("## Recently Updated Notes");
        println!();
        for note in recent_notes {
            let type_char = match note.note_type().as_str() {
                NoteType::FLEETING => 'F',
                NoteType::LITERATURE => 'L',
                NoteType::PERMANENT => 'P',
                NoteType::MOC => 'M',
                _ => 'F',
            };
            println!("  {} [{}] {}", note.id(), type_char, note.title());
        }
    }
}
