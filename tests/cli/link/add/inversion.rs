//! Tests for link command
use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_custom_link_inversion() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.recommends]
inverse = "recommended-by"
description = "This note recommends another note"

[graph.types."recommended-by"]
inverse = "recommends"
description = "This note is recommended by another note"
"#;
    std::fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    eprintln!("Created notes: {} -> {}", id1, id2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "recommends"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("recommended-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_part_of() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Chapter 1"])
        .output()
        .unwrap();
    let chapter = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Book"])
        .output()
        .unwrap();
    let book = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &chapter, &book, "--type", "part-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &chapter])
        .assert()
        .success()
        .stdout(predicate::str::contains(&book))
        .stdout(predicate::str::contains("part-of"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &book])
        .assert()
        .success()
        .stdout(predicate::str::contains(&chapter))
        .stdout(predicate::str::contains("has-part"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_follows() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Step 2"])
        .output()
        .unwrap();
    let step2 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Step 1"])
        .output()
        .unwrap();
    let step1 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &step2, &step1, "--type", "follows"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &step2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&step1))
        .stdout(predicate::str::contains("follows"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &step1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&step2))
        .stdout(predicate::str::contains("precedes"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_contradicts() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Claim A"])
        .output()
        .unwrap();
    let claim_a = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Claim B"])
        .output()
        .unwrap();
    let claim_b = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &claim_a, &claim_b, "--type", "contradicts"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &claim_a])
        .assert()
        .success()
        .stdout(predicate::str::contains(&claim_b))
        .stdout(predicate::str::contains("contradicts"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &claim_b])
        .assert()
        .success()
        .stdout(predicate::str::contains(&claim_a))
        .stdout(predicate::str::contains("contradicted-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_answers() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Answer Note"])
        .output()
        .unwrap();
    let answer = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Question Note"])
        .output()
        .unwrap();
    let question = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &answer, &question, "--type", "answers"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &answer])
        .assert()
        .success()
        .stdout(predicate::str::contains(&question))
        .stdout(predicate::str::contains("answers"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &question])
        .assert()
        .success()
        .stdout(predicate::str::contains(&answer))
        .stdout(predicate::str::contains("answered-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_refines() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Version 2"])
        .output()
        .unwrap();
    let v2 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Version 1"])
        .output()
        .unwrap();
    let v1 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &v2, &v1, "--type", "refines"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &v2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&v1))
        .stdout(predicate::str::contains("refines"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &v1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&v2))
        .stdout(predicate::str::contains("refined-by"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_same_as() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Concept A"])
        .output()
        .unwrap();
    let concept_a = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Concept B"])
        .output()
        .unwrap();
    let concept_b = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &concept_a, &concept_b, "--type", "same-as"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &concept_a])
        .assert()
        .success()
        .stdout(predicate::str::contains(&concept_b))
        .stdout(predicate::str::contains("same-as"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &concept_b])
        .assert()
        .success()
        .stdout(predicate::str::contains(&concept_a))
        .stdout(predicate::str::contains("same-as"))
        .stdout(predicate::str::contains("(virtual)"));
}

#[test]
fn test_standard_type_alias_of() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Alternative Name"])
        .output()
        .unwrap();
    let alias = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Canonical Name"])
        .output()
        .unwrap();
    let canonical = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &alias, &canonical, "--type", "alias-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &alias])
        .assert()
        .success()
        .stdout(predicate::str::contains(&canonical))
        .stdout(predicate::str::contains("alias-of"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &canonical])
        .assert()
        .success()
        .stdout(predicate::str::contains(&alias))
        .stdout(predicate::str::contains("has-alias"))
        .stdout(predicate::str::contains("(virtual)"));
}
