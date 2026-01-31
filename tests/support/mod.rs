use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::fs;
use std::path::Path;
use std::process::Output;
use tempfile::TempDir;

/// Get a Command for qipu
pub fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

/// Extract note ID from create command output (first line)
/// Create outputs: <id>\n<path>\n, so we take the first line
pub fn extract_id(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Extract note ID from stdout bytes (first line only)
/// Use when you have raw stdout bytes from a command
#[allow(dead_code)]
pub fn extract_id_from_bytes(stdout: &[u8]) -> String {
    let output = String::from_utf8_lossy(stdout);
    output
        .lines()
        .find(|line| line.starts_with("qp-"))
        .map(|line| line.trim().to_string())
        .expect("Failed to extract ID from output")
}

/// Setup a test store and return the directory only
/// Use when you need full control over command construction
#[allow(dead_code)]
pub fn setup_test_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();
    dir
}

/// Create a note and return its ID
#[allow(dead_code)]
pub fn create_note(dir: &TempDir, title: &str) -> String {
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title])
        .output()
        .unwrap();
    extract_id(&output)
}

/// Create a note with tags and return its ID
#[allow(dead_code)]
pub fn create_note_with_tags(dir: &TempDir, title: &str, tags: &[&str]) -> String {
    let mut args = vec!["create", title];
    for tag in tags {
        args.push("--tag");
        args.push(tag);
    }
    let output = qipu().current_dir(dir.path()).args(&args).output().unwrap();
    extract_id(&output)
}

/// Run qipu command and return stdout as String
#[allow(dead_code)]
pub fn run_and_get_stdout(dir: &TempDir, args: &[&str]) -> String {
    let output = qipu().current_dir(dir.path()).args(args).output().unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Run qipu command and assert success
#[allow(dead_code)]
pub fn run_assert_success(dir: &TempDir, args: &[&str]) {
    qipu().current_dir(dir.path()).args(args).assert().success();
}

/// Add text content to a note file by ID
#[allow(dead_code)]
pub fn append_to_note(dir: &TempDir, note_id: &str, content: &str) {
    let notes_dir = dir.path().join(".qipu").join("notes");
    for entry in std::fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(note_id) {
            let mut file_content = std::fs::read_to_string(entry.path()).unwrap();
            file_content.push_str(content);
            std::fs::write(entry.path(), file_content).unwrap();
            return;
        }
    }
    panic!("Note with ID {} not found", note_id);
}

/// Create a link between two notes
#[allow(dead_code)]
pub fn create_link(dir: &TempDir, from_id: &str, to_id: &str, link_type: &str) {
    run_assert_success(dir, &["link", "add", from_id, to_id, "--type", link_type]);
}

/// Apply compaction to combine notes into a digest
#[allow(dead_code)]
pub fn apply_compaction(dir: &TempDir, digest_id: &str, note_ids: &[&str]) {
    let mut args = vec!["compact", "apply", digest_id];
    for id in note_ids {
        args.push("--note");
        args.push(id);
    }
    run_assert_success(dir, &args);
}

/// Rebuild index to sync database with file changes
#[allow(dead_code)]
pub fn rebuild_index(dir: &TempDir) {
    run_assert_success(dir, &["index", "--rebuild"]);
}

/// Setup a test store using QIPU_STORE env var and return the directory
/// Use when tests need to use env var instead of current_dir
#[allow(dead_code)]
pub fn setup_test_store() -> TempDir {
    let dir = TempDir::new().unwrap();
    qipu()
        .arg("init")
        .env("QIPU_STORE", dir.path())
        .assert()
        .success();
    dir
}

/// Create a test store with specified number of notes for performance testing
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

/// Initialize a store at given path using current_dir approach
#[allow(dead_code)]
pub fn init_store_at_path(path: &Path) {
    qipu().current_dir(path).arg("init").assert().success();
}

/// Create a note at given path and return its ID
#[allow(dead_code)]
pub fn create_note_at_path(path: &Path, title: &str) -> String {
    let output = qipu()
        .current_dir(path)
        .args(["create", title])
        .output()
        .unwrap();
    extract_id(&output)
}

/// Create a link between two notes at given path
#[allow(dead_code)]
pub fn create_link_at_path(path: &Path, from_id: &str, to_id: &str, link_type: &str) {
    qipu()
        .current_dir(path)
        .args(["link", "add", from_id, to_id, "--type", link_type])
        .assert()
        .success();
}

/// Two-store test setup for pack/unpack operations
pub struct TwoStoreSetup {
    pub dir1: TempDir,
    pub dir2: TempDir,
    pub pack_file: std::path::PathBuf,
}

impl TwoStoreSetup {
    #[allow(dead_code)]
    pub fn new(pack_name: &str) -> Self {
        let dir1 = TempDir::new().unwrap();
        let dir2 = TempDir::new().unwrap();
        let pack_file = dir1.path().join(pack_name);

        qipu()
            .arg("init")
            .env("QIPU_STORE", dir1.path())
            .assert()
            .success();

        qipu()
            .arg("init")
            .env("QIPU_STORE", dir2.path())
            .assert()
            .success();

        Self {
            dir1,
            dir2,
            pack_file,
        }
    }

    #[allow(dead_code)]
    pub fn store1_path(&self) -> &Path {
        self.dir1.path()
    }

    #[allow(dead_code)]
    pub fn store2_path(&self) -> &Path {
        self.dir2.path()
    }

    #[allow(dead_code)]
    pub fn pack_file(&self) -> &Path {
        &self.pack_file
    }
}

/// Create a note using QIPU_STORE env var and return its ID
#[allow(dead_code)]
pub fn create_note_with_env(store_path: &Path, title: &str) -> String {
    let output = qipu()
        .arg("create")
        .arg(title)
        .env("QIPU_STORE", store_path)
        .output()
        .unwrap();
    extract_id(&output)
}

/// Create a link using QIPU_STORE env var
#[allow(dead_code)]
pub fn create_link_with_env(store_path: &Path, from_id: &str, to_id: &str, link_type: &str) {
    qipu()
        .args(["link", "add", from_id, to_id, "--type", link_type])
        .env("QIPU_STORE", store_path)
        .assert()
        .success();
}
