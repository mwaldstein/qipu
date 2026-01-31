//! Output formatting for prime command

pub mod human;
pub mod records;

use qipu_core::config::OntologyConfig;
use qipu_core::ontology::Ontology;

pub use human::{build_base_human, output_human};
pub use records::{build_base_records, output_records};

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

/// MCP mode output (~50 tokens) - minimal primer for agent environments
/// Contains only essential reminders to keep context small
pub fn output_mcp_human(_store_path: &str) {
    println!("Qipu knowledge graph. Commands: capture, search, link.");
    println!("Session protocol: capture insights, link notes, git commit.");
}

/// MCP mode output in JSON format (~50 tokens equivalent)
pub fn output_mcp_json() {
    let output = serde_json::json!({
        "tool": "qipu",
        "type": "knowledge-graph",
        "commands": ["capture", "search", "link"],
        "protocol": "capture, link, commit"
    });
    println!("{}", serde_json::to_string(&output).unwrap());
}

/// MCP mode output in records format (~50 tokens equivalent)
pub fn output_mcp_records() {
    println!("H qipu=1 mode=mcp");
    println!("D Knowledge graph tool. Capture insights, search, link notes.");
    println!("C capture \"Create note from stdin\"");
    println!("C search \"Search notes\"");
    println!("C link \"Link notes\"");
    println!("W Commit changes: git add .qipu && git commit");
}
