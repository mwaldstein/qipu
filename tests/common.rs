use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::fs;
use std::path::Path;
use std::process::Output;

pub fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[allow(dead_code)]
pub fn extract_id(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn extract_id_from_bytes(stdout: &[u8]) -> String {
    let output = String::from_utf8_lossy(stdout);
    output
        .lines()
        .find(|line| line.starts_with("qp-"))
        .map(|line| line.trim().to_string())
        .expect("Failed to extract ID from output")
}

#[allow(dead_code)]
pub fn create_test_store_with_notes(
    store_dir: &Path,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    qipu()
        .arg("--store")
        .arg(store_dir)
        .arg("init")
        .assert()
        .success();

    for i in 0..count {
        let title = format!("Note {}", i);
        let content = if i % 5 == 0 {
            format!("This is a test note about programming and algorithms. Note number {} contains relevant content.", i)
        } else {
            format!("This is test note number {} with some content.", i)
        };

        let note_content = format!(
            "---\nid: qp-test{}\ntitle: {}\ntype: permanent\n---\n\n{}",
            i, title, content
        );

        let note_path = store_dir
            .join("notes")
            .join(format!("qp-test{}-note-{}.md", i, i));
        fs::create_dir_all(note_path.parent().unwrap())?;
        fs::write(note_path, note_content)?;
    }

    qipu()
        .arg("--store")
        .arg(store_dir)
        .arg("index")
        .assert()
        .success();

    Ok(())
}
