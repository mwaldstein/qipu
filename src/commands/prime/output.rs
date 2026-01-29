//! Output formatting for prime command

use qipu_core::config::OntologyConfig;
use qipu_core::note::NoteType;
use qipu_core::ontology::Ontology;
use qipu_core::records::escape_quotes;

pub fn format_mode(mode: qipu_core::config::OntologyMode) -> &'static str {
    match mode {
        qipu_core::config::OntologyMode::Default => "default",
        qipu_core::config::OntologyMode::Extended => "extended",
        qipu_core::config::OntologyMode::Replacement => "replacement",
    }
}

pub fn primer_description(is_empty: bool) -> &'static str {
    if is_empty {
        "Welcome to qipu! Your knowledge store is empty. Start by capturing your first insights with `qipu capture` or `qipu create`. Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content (MOCs)."
    } else {
        "Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content (MOCs)."
    }
}

pub fn getting_started_section(is_empty: bool) -> String {
    if is_empty {
        "## Getting Started\n\nYour knowledge store is empty. Start by capturing your first insights:\n\n  qipu capture --title \"...\"     Quick capture from stdin\n  qipu create <title>             Create a new note\n  qipu create <title> --type moc   Create a Map of Content\n\n".to_string()
    } else {
        String::new()
    }
}

pub fn build_base_json(store_path: &str, is_empty: bool) -> serde_json::Value {
    serde_json::json!({
        "store": store_path,
        "primer": {
            "description": primer_description(is_empty),
            "commands": [
                {"name": "qipu list", "description": "List notes"},
                {"name": "qipu search <query>", "description": "Search notes by title and body"},
                {"name": "qipu show <id>", "description": "Display a note"},
                {"name": "qipu create <title>", "description": "Create a new note"},
                {"name": "qipu capture", "description": "Create note from stdin"},
                {"name": "qipu link tree <id>", "description": "Show traversal tree from a note"},
                {"name": "qipu link path <from> <to>", "description": "Find path between notes"},
                {"name": "qipu context", "description": "Build context bundle for LLM"},
            ],
            "session_protocol": {
                "why": "Knowledge not committed is knowledge lost. The graph only grows if you save your work.",
                "steps": [
                    {"number": 1, "action": "Capture any new insights", "command": "qipu capture --title \"...\""},
                    {"number": 2, "action": "Link new notes to existing knowledge", "command": "qipu link add <new> <existing> --type <type>"},
                    {"number": 3, "action": "Commit changes", "command": "git add .qipu && git commit -m \"knowledge: ...\""}
                ]
            },
        },
        "mocs": [],
        "recent_notes": [],
    })
}

pub fn build_base_human(store_path: &str, is_empty: bool) -> String {
    let getting_started = getting_started_section(is_empty);
    format!(
        "# Qipu Knowledge Store Primer\n\nStore: {}\n\n## About Qipu\n\nQipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\n\n{}## Quick Reference\n\n  qipu list              List notes\n  qipu search <query>    Search notes by title and body\n  qipu show <id>         Display a note\n  qipu create <title>    Create a new note\n  qipu capture           Create note from stdin\n  qipu link tree <id>    Show traversal tree from a note\n  qipu link path A B     Find path between notes\n  qipu context           Build context bundle for LLM\n\n## Session Protocol\n\n**Before ending session:**\n1. Capture any new insights: `qipu capture --title \"...\"`\n2. Link new notes to existing knowledge: `qipu link add <new> <existing> --type <type>`\n3. Commit changes: `git add .qipu && git commit -m \"knowledge: ...\"`\n\n**Why this matters:** Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n\n",
        store_path, getting_started
    )
}

pub fn build_base_records(store_path: &str, is_empty: bool) -> String {
    let store_status = if is_empty { "empty" } else { "populated" };
    format!(
        "H qipu=1 records=1 store={} mode=prime mocs=0 recent=0 status={} truncated=false\nD Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\nC list \"List notes\"\nC search \"Search notes by title and body\"\nC show \"Display a note\"\nC create \"Create a new note\"\nC capture \"Create note from stdin\"\nC link.tree \"Show traversal tree from a note\"\nC link.path \"Find path between notes\"\nC context \"Build context bundle for LLM\"\nS 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"\nS 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"\nS 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"\nW Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n",
        store_path, store_status
    )
}

pub fn output_json(
    store_path: &str,
    ontology: &Ontology,
    config: &OntologyConfig,
    selected_mocs: &[&qipu_core::note::Note],
    selected_recent: &[&qipu_core::note::Note],
    is_empty: bool,
) -> Result<(), qipu_core::error::QipuError> {
    let note_types = ontology.note_types();
    let link_types = ontology.link_types();

    let note_type_objs: Vec<_> = note_types
        .iter()
        .map(|nt| {
            let type_config = config.note_types.get(nt);
            serde_json::json!({
                "name": nt,
                "description": type_config.and_then(|c| c.description.clone()),
                "usage": type_config.and_then(|c| c.usage.clone()),
            })
        })
        .collect();

    let link_type_objs: Vec<_> = link_types
        .iter()
        .map(|lt| {
            let inverse = ontology.get_inverse(lt);
            let type_config = config.link_types.get(lt);
            serde_json::json!({
                "name": lt,
                "inverse": inverse,
                "description": type_config.and_then(|c| c.description.clone()),
                "usage": type_config.and_then(|c| c.usage.clone()),
            })
        })
        .collect();

    let mut output = build_base_json(store_path, is_empty);

    output["ontology"] = serde_json::json!({
        "mode": format_mode(config.mode),
        "note_types": note_type_objs,
        "link_types": link_type_objs,
    });

    output["mocs"] = serde_json::to_value(
        selected_mocs
            .iter()
            .map(|n: &&qipu_core::note::Note| {
                serde_json::json!({
                    "id": n.id(),
                    "title": n.title(),
                    "tags": n.frontmatter.tags,
                })
            })
            .collect::<Vec<_>>(),
    )?;

    output["recent_notes"] = serde_json::to_value(
        selected_recent
            .iter()
            .map(|n: &&qipu_core::note::Note| {
                serde_json::json!({
                    "id": n.id(),
                    "title": n.title(),
                    "type": n.note_type().to_string(),
                    "tags": n.frontmatter.tags,
                })
            })
            .collect::<Vec<_>>(),
    )?;

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
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
    println!("Mode: {}", format_mode(config.mode));
    println!();

    let note_types = ontology.note_types();
    let link_types = ontology.link_types();

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
    println!();

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
    println!();

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
        println!();
    }

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
        println!();
    }

    println!("Use `qipu context --note <id>` to fetch full note content.");
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

    println!("O mode={}", format_mode(config.mode));

    let note_types = ontology.note_types();
    let link_types = ontology.link_types();

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

    println!("D Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.");

    println!("C list \"List notes\"");
    println!("C search \"Search notes by title and body\"");
    println!("C show \"Display a note\"");
    println!("C create \"Create a new note\"");
    println!("C capture \"Create note from stdin\"");
    println!("C link.tree \"Show traversal tree from a note\"");
    println!("C link.path \"Find path between notes\"");
    println!("C context \"Build context bundle for LLM\"");

    println!("S 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"");
    println!("S 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"");
    println!("S 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"");
    println!(
        "W Knowledge not committed is knowledge lost. The graph only grows if you save your work."
    );

    for moc in mocs {
        let tags_csv = moc.frontmatter.tags.join(",");
        println!(
            "M {} \"{}\" tags={}",
            moc.id(),
            moc.title(),
            if tags_csv.is_empty() { "-" } else { &tags_csv }
        );
    }

    for note in recent_notes {
        let tags_csv = note.frontmatter.tags.join(",");
        println!(
            "N {} {} \"{}\" tags={}",
            note.id(),
            note.note_type(),
            escape_quotes(note.title()),
            if tags_csv.is_empty() { "-" } else { &tags_csv }
        );
    }
}
