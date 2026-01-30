use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_prime_shows_custom_ontology() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task item"
usage = "Track tasks"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
usage = "Note B depends on Note A"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("## Ontology"))
        .stdout(predicate::str::contains("Mode: extended"))
        .stdout(predicate::str::contains("task - A task item"))
        .stdout(predicate::str::contains("Usage: Track tasks"))
        .stdout(predicate::str::contains(
            "depends-on -> required-by (Dependency relationship)",
        ))
        .stdout(predicate::str::contains("Usage: Note B depends on Note A"));
}

#[test]
fn test_graph_types_backward_compatibility() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.custom-link]
description = "Custom link via graph.types"
inverse = "inverse-custom"
cost = 0.5
"#;
    fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "custom-link"])
        .assert()
        .success();
}

#[test]
fn test_pack_roundtrip_with_custom_ontology() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();
    let pack_file = dir1.path().join("test.pack.json");

    qipu()
        .current_dir(dir1.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir1.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task item"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
"#;
    fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir1.path())
        .args(["create", "Task A", "--type", "task"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir1.path())
        .args(["create", "Task B", "--type", "task"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir1.path())
        .args(["link", "add", &id1, &id2, "--type", "depends-on"])
        .assert()
        .success();

    qipu()
        .current_dir(dir1.path())
        .args([
            "dump",
            "--output",
            pack_file.to_string_lossy().as_ref(),
            "--format",
            "json",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir2.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir2.path())
        .args([
            "load",
            "--apply-config",
            pack_file.to_string_lossy().as_ref(),
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir2.path())
        .arg("show")
        .arg(&id1)
        .assert()
        .success()
        .stdout(predicate::str::contains("Task A"));

    qipu()
        .current_dir(dir2.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: extended"))
        .stdout(predicate::str::contains("task"))
        .stdout(predicate::str::contains("depends-on"));
}
