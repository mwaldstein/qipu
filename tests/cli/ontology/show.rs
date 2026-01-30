use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_ontology_show_default_mode() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: default"))
        .stdout(predicate::str::contains("fleeting"))
        .stdout(predicate::str::contains("literature"))
        .stdout(predicate::str::contains("permanent"))
        .stdout(predicate::str::contains("moc"))
        .stdout(predicate::str::contains("related"))
        .stdout(predicate::str::contains("supports"));
}

#[test]
fn test_ontology_show_extended_mode() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.custom-note]
description = "A custom note type"
usage = "Use for custom notes"

[ontology.link_types.custom-link]
description = "A custom link type"
inverse = "inverse-custom-link"
usage = "Use for custom links"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: extended"))
        .stdout(predicate::str::contains("custom-note"))
        .stdout(predicate::str::contains("A custom note type"))
        .stdout(predicate::str::contains("custom-link"))
        .stdout(predicate::str::contains("A custom link type"))
        .stdout(predicate::str::contains("inverse-custom-link"));
}

#[test]
fn test_ontology_show_replacement_mode() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "custom-type"

[ontology]
mode = "replacement"

[ontology.note_types.custom-type]
description = "Only custom type"

[ontology.link_types.custom-link]
description = "Only custom link"
inverse = "inverse-link"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: replacement"))
        .stdout(predicate::str::contains("custom-type"))
        .stdout(predicate::str::contains("custom-link"));
}

#[test]
fn test_ontology_show_json_format() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.custom-note]
description = "A custom note type"

[ontology.link_types.custom-link]
description = "A custom link type"
inverse = "inverse-link"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["ontology", "show", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""mode": "extended""#))
        .stdout(predicate::str::contains(r#""name": "custom-note""#))
        .stdout(predicate::str::contains(
            r#""description": "A custom note type""#,
        ))
        .stdout(predicate::str::contains(r#""name": "custom-link""#))
        .stdout(predicate::str::contains(r#""inverse": "inverse-link""#));
}

#[test]
fn test_ontology_show_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["ontology", "show", "--format", "records"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=ontology.show"))
        .stdout(predicate::str::contains("O mode=default"))
        .stdout(predicate::str::contains("N note_type=\"fleeting\""))
        .stdout(predicate::str::contains("L link_type=\"related\""));
}
