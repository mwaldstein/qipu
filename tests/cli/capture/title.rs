use crate::support::{qipu, setup_test_dir};

#[test]
fn test_capture_auto_title_from_content() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("Auto generated title from this line\nMore content")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Auto generated title from this line"));
}

#[test]
fn test_capture_auto_title_from_heading() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("# My Heading\n\nSome content below")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("My Heading"));
}

#[test]
fn test_capture_auto_title_empty_content() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Untitled capture"));
}
