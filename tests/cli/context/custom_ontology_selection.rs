use crate::support::{qipu, setup_test_dir};
use std::fs;

#[test]
fn test_context_collection_root_selector_accepts_replacement_ontology_root_type() {
    let dir = setup_test_dir();

    fs::write(
        dir.path().join(".qipu/config.toml"),
        r#"version = 1
default_note_type = "claim"

[ontology]
mode = "replacement"

[ontology.note_types.outline]
description = "A linked collection root"

[ontology.note_types.claim]
description = "A domain claim"

[ontology.link_types.related]
description = "Related domain item"
inverse = "related"
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join(".qipu/notes/project-index-index.md"),
        "---\nid: project-index\ntitle: Project Index\ntype: outline\nlinks:\n  - type: related\n    id: claim-one\n---\nIndex body",
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/claim-one-one.md"),
        "---\nid: claim-one\ntitle: Claim One\ntype: claim\n---\nClaim body",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "context",
            "--collection-root",
            "project-index",
            "--related",
            "0",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|note| note["id"].as_str().unwrap())
        .collect();

    assert!(ids.contains(&"project-index"));
    assert!(ids.contains(&"claim-one"));
}
