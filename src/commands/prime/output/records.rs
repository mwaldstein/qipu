//! Records-format output for prime command

use qipu_core::config::OntologyConfig;
use qipu_core::ontology::Ontology;
use qipu_core::records::escape_quotes;

pub fn build_base_records(store_path: &str, is_empty: bool) -> String {
    let store_status = if is_empty { "empty" } else { "populated" };
    format!(
        "H qipu=1 records=1 store={} mode=prime mocs=0 recent=0 status={} truncated=false\nD Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\nC list \"List notes\"\nC search \"Search notes by title and body\"\nC show \"Display a note\"\nC create \"Create a new note\"\nC capture \"Create note from stdin\"\nC link.tree \"Show traversal tree from a note\"\nC link.path \"Find path between notes\"\nC context \"Build context bundle for LLM\"\nS 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"\nS 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"\nS 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"\nW Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n",
        store_path, store_status
    )
}

pub fn output_records(
    store_path: &str,
    ontology: &Ontology,
    config: &OntologyConfig,
    mocs: &[&qipu_core::note::Note],
    recent_notes: &[&qipu_core::note::Note],
    is_empty: bool,
) {
    let mocs_count = mocs.len();
    let notes_count = recent_notes.len();
    let store_status = if is_empty { "empty" } else { "populated" };

    println!(
        "H qipu=1 records=1 store={} mode=prime mocs={} recent={} status={} truncated=false",
        store_path, mocs_count, notes_count, store_status
    );

    println!("O mode={}", config.mode);

    print_note_type_records(ontology, config);
    print_link_type_records(ontology, config);

    println!("D Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.");

    print_command_records();
    print_session_records();

    for moc in mocs {
        let tags_csv = moc.frontmatter.format_tags();
        println!("M {} \"{}\" tags={}", moc.id(), moc.title(), tags_csv);
    }

    for note in recent_notes {
        let tags_csv = note.frontmatter.format_tags();
        println!(
            "N {} {} \"{}\" tags={}",
            note.id(),
            note.note_type(),
            escape_quotes(note.title()),
            tags_csv
        );
    }
}

fn print_note_type_records(ontology: &Ontology, config: &OntologyConfig) {
    let note_types = ontology.note_types();
    for nt in &note_types {
        let type_config = config.note_types.get(nt);
        if let Some(desc) = type_config.and_then(|c| c.description.as_deref()) {
            println!("T note_type=\"{}\" description=\"{}\"", nt, desc);
        } else {
            println!("T note_type=\"{}\"", nt);
        }
        if let Some(usage) = type_config.and_then(|c| c.usage.as_deref()) {
            println!("U note_type=\"{}\" usage=\"{}\"", nt, usage);
        }
    }
}

fn print_link_type_records(ontology: &Ontology, config: &OntologyConfig) {
    let link_types = ontology.link_types();
    for lt in &link_types {
        let inverse = ontology.get_inverse(lt);
        let type_config = config.link_types.get(lt);
        if let Some(desc) = type_config.and_then(|c| c.description.as_deref()) {
            println!(
                "L link_type=\"{}\" inverse=\"{}\" description=\"{}\"",
                lt, inverse, desc
            );
        } else {
            println!("L link_type=\"{}\" inverse=\"{}\"", lt, inverse);
        }
        if let Some(usage) = type_config.and_then(|c| c.usage.as_deref()) {
            println!("U link_type=\"{}\" usage=\"{}\"", lt, usage);
        }
    }
}

fn print_command_records() {
    println!("C list \"List notes\"");
    println!("C search \"Search notes by title and body\"");
    println!("C show \"Display a note\"");
    println!("C create \"Create a new note\"");
    println!("C capture \"Create note from stdin\"");
    println!("C link.tree \"Show traversal tree from a note\"");
    println!("C link.path \"Find path between notes\"");
    println!("C context \"Build context bundle for LLM\"");
}

fn print_session_records() {
    println!("S 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"");
    println!("S 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"");
    println!("S 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"");
    println!(
        "W Knowledge not committed is knowledge lost. The graph only grows if you save your work."
    );
}
