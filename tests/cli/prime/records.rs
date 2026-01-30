use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_prime_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("truncated=false"))
        .stdout(predicate::str::contains("D Qipu is"))
        .stdout(predicate::str::contains("C list"))
        .stdout(predicate::str::contains("S "))
        .stdout(predicate::str::contains("W Knowledge not committed"))
        .stdout(predicate::str::contains("M "))
        .stdout(predicate::str::contains("N "));
}

#[test]
fn test_prime_records_comprehensive_structure() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc", "--tag", "moc-tag"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "Tagged Note",
            "--type",
            "fleeting",
            "--tag",
            "research",
            "--tag",
            "important",
        ])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(lines[0].starts_with("H "), "First line should be header");
    assert!(lines[0].contains("qipu=1"), "Header should contain qipu=1");
    assert!(
        lines[0].contains("records=1"),
        "Header should contain records=1"
    );
    assert!(lines[0].contains("store="), "Header should contain store=");
    assert!(
        lines[0].contains("mode=prime"),
        "Header should contain mode=prime"
    );
    assert!(lines[0].contains("mocs=1"), "Header should contain mocs=1");
    assert!(
        lines[0].contains("recent=1"),
        "Header should contain recent=1"
    );
    assert!(
        lines[0].contains("truncated=false"),
        "Header should contain truncated=false"
    );

    let d_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("D "))
        .copied()
        .collect();
    assert!(!d_lines.is_empty(), "Should have at least one D line");

    let c_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("C "))
        .copied()
        .collect();
    assert_eq!(c_lines.len(), 8, "Should have 8 command lines");
    for c_line in &c_lines {
        assert!(c_line.contains(" "), "Command line should have space");
    }

    let s_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("S "))
        .copied()
        .collect();
    assert_eq!(s_lines.len(), 3, "Should have 3 session protocol lines");
    for s_line in &s_lines {
        assert!(
            s_line.contains(" "),
            "Session protocol line should have space"
        );
        assert!(
            s_line.contains("\""),
            "Session protocol line should have quoted action and command"
        );
    }

    let w_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("W "))
        .copied()
        .collect();
    assert_eq!(w_lines.len(), 1, "Should have 1 why line");
    assert!(
        w_lines[0].contains("Knowledge not committed"),
        "Why line should contain explanation"
    );

    let m_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("M "))
        .copied()
        .collect();
    assert_eq!(m_lines.len(), 1, "Should have 1 MOC line");
    let m_line = &m_lines[0];
    assert!(m_line.contains("qp-"), "MOC line should contain ID");
    assert!(m_line.contains("Test MOC"), "MOC line should contain title");
    assert!(
        m_line.contains("tags=moc-tag"),
        "MOC line should have correct tag"
    );

    let n_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with("N "))
        .copied()
        .collect();
    assert_eq!(n_lines.len(), 1, "Should have 1 note line");
    let n_line = &n_lines[0];
    assert!(n_line.contains("qp-"), "Note line should contain ID");
    assert!(n_line.contains("fleeting"), "Note line should contain type");
    assert!(
        n_line.contains("Tagged Note"),
        "Note line should contain title"
    );
    assert!(
        n_line.contains("tags=important,research"),
        "Note line should have correct tags (comma-separated, alphabetically sorted)"
    );
}

#[test]
fn test_prime_records_empty_tags() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    let m_line = lines.iter().find(|l| l.starts_with("M ")).unwrap();
    assert!(
        m_line.contains("tags=-"),
        "MOC line should have tags=- for empty tags"
    );

    let n_line = lines.iter().find(|l| l.starts_with("N ")).unwrap();
    assert!(
        n_line.contains("tags=-"),
        "Note line should have tags=- for empty tags"
    );
}

#[test]
fn test_prime_records_truncated_field() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=false"),
        "prime records output should contain truncated=false in header"
    );
}
