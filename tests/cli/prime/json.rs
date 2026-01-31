use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_prime_json_format() {
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
        .args(["--format", "json", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"store\""))
        .stdout(predicate::str::contains("\"primer\""))
        .stdout(predicate::str::contains("\"mocs\""))
        .stdout(predicate::str::contains("\"recent_notes\""))
        .stdout(predicate::str::contains("\"commands\""))
        .stdout(predicate::str::contains("\"session_protocol\""));
}

#[test]
fn test_prime_json_comprehensive_structure() {
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
        .args(["--format", "json", "prime"])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(json["store"].is_string(), "store should be a string");
    assert!(json["primer"].is_object(), "primer should be an object");
    assert!(json["mocs"].is_array(), "mocs should be an array");
    assert!(
        json["recent_notes"].is_array(),
        "recent_notes should be an array"
    );

    assert!(
        json["primer"]["description"].is_string(),
        "primer.description should be a string"
    );
    assert!(
        json["primer"]["commands"].is_array(),
        "primer.commands should be an array"
    );
    assert!(
        json["primer"]["session_protocol"].is_object(),
        "primer.session_protocol should be an object"
    );

    let session_protocol = &json["primer"]["session_protocol"];
    assert!(
        session_protocol["why"].is_string(),
        "session_protocol.why should be a string"
    );
    assert!(
        session_protocol["steps"].is_array(),
        "session_protocol.steps should be an array"
    );
    assert_eq!(
        session_protocol["steps"].as_array().unwrap().len(),
        3,
        "session_protocol.steps should have 3 steps"
    );
    let steps = session_protocol["steps"].as_array().unwrap();
    for step in steps {
        assert!(step["number"].is_u64(), "step.number should be a number");
        assert!(step["action"].is_string(), "step.action should be a string");
        assert!(
            step["command"].is_string(),
            "step.command should be a string"
        );
    }

    let commands = json["primer"]["commands"].as_array().unwrap();
    assert!(!commands.is_empty(), "commands should not be empty");
    for cmd in commands {
        assert!(cmd["name"].is_string(), "command name should be a string");
        assert!(
            cmd["description"].is_string(),
            "command description should be a string"
        );
    }

    let mocs = json["mocs"].as_array().unwrap();
    assert_eq!(mocs.len(), 1, "should have 1 MOC");
    let moc = &mocs[0];
    assert!(moc["id"].is_string(), "MOC id should be a string");
    assert!(moc["title"].is_string(), "MOC title should be a string");
    assert!(moc["tags"].is_array(), "MOC tags should be an array");
    let moc_tags = moc["tags"].as_array().unwrap();
    assert_eq!(moc_tags.len(), 1, "MOC should have 1 tag");
    assert_eq!(moc_tags[0], "moc-tag", "MOC tag should be correct");

    let recent_notes = json["recent_notes"].as_array().unwrap();
    assert_eq!(recent_notes.len(), 1, "should have 1 recent note");
    let note = &recent_notes[0];
    assert!(note["id"].is_string(), "note id should be a string");
    assert!(note["title"].is_string(), "note title should be a string");
    assert!(note["type"].is_string(), "note type should be a string");
    assert!(note["tags"].is_array(), "note tags should be an array");
    let note_tags = note["tags"].as_array().unwrap();
    assert_eq!(note_tags.len(), 2, "note should have 2 tags");
    let tags: Vec<&str> = note_tags.iter().map(|t| t.as_str().unwrap()).collect();
    assert!(
        tags.contains(&"research"),
        "note should have 'research' tag"
    );
    assert!(
        tags.contains(&"important"),
        "note should have 'important' tag"
    );
}
