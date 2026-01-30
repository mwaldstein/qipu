//! Tests for store-local PRIME.md override feature
//!
//! Issue: qipu-rzv - Prime: support store-local override file (.qipu/PRIME.md)

use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_prime_with_use_prime_md_flag_reads_file() {
    let dir = setup_test_dir();
    let custom_content = "# Custom Store Primer\n\nThis is a custom primer for this project.";

    fs::write(dir.path().join(".qipu/PRIME.md"), custom_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["prime", "--use-prime-md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Custom Store Primer"))
        .stdout(predicate::str::contains(
            "This is a custom primer for this project.",
        ));
}

#[test]
fn test_prime_with_use_prime_md_flag_falls_back_when_no_file() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["prime", "--use-prime-md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Qipu Knowledge Store Primer"))
        .stdout(predicate::str::contains("About Qipu"));
}

#[test]
fn test_prime_without_flag_ignores_prime_md() {
    let dir = setup_test_dir();
    let custom_content = "# Custom Store Primer\n\nThis should NOT appear.";

    fs::write(dir.path().join(".qipu/PRIME.md"), custom_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Qipu Knowledge Store Primer"))
        .stdout(predicate::str::contains("About Qipu"))
        .stdout(predicate::str::contains("Custom Store Primer").not());
}

#[test]
fn test_prime_md_with_multiline_content() {
    let dir = setup_test_dir();
    let custom_content = r#"# Project Knowledge Primer

## Quick Commands
- `qipu list` - List all notes
- `qipu create "Title"` - Create a new note

## Important Notes
- Always use permanent notes for core concepts
- Link related ideas together
"#;

    fs::write(dir.path().join(".qipu/PRIME.md"), custom_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["prime", "--use-prime-md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project Knowledge Primer"))
        .stdout(predicate::str::contains("Quick Commands"))
        .stdout(predicate::str::contains("Important Notes"))
        .stdout(predicate::str::contains("qipu list"))
        .stdout(predicate::str::contains("permanent notes"));
}

#[test]
fn test_prime_md_empty_file() {
    let dir = setup_test_dir();

    fs::write(dir.path().join(".qipu/PRIME.md"), "").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["prime", "--use-prime-md"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_prime_md_with_special_characters() {
    let dir = setup_test_dir();
    let custom_content = r#"# Custom Primer

Special chars: <>&"quotes" and 'apostrophes'
Code: `fn main() { println!("Hello"); }`
Path: /home/user/.qipu/"#;

    fs::write(dir.path().join(".qipu/PRIME.md"), custom_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["prime", "--use-prime-md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Custom Primer"))
        .stdout(predicate::str::contains("Special chars"))
        .stdout(predicate::str::contains("fn main()"))
        .stdout(predicate::str::contains("/home/user/.qipu/"));
}
