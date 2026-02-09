use crate::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Store discovery tests
// ============================================================================

#[test]
fn test_store_discovery_walks_up() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("sub/dir/deep");
    std::fs::create_dir_all(&subdir).unwrap();

    // Init at top level
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // List from deep subdir should find store
    qipu().current_dir(&subdir).arg("list").assert().success();
}

#[test]
fn test_visible_store_discovery() {
    let dir = tempdir().unwrap();

    // Manually create a visible "qipu/" store structure
    let store_path = dir.path().join("qipu");
    std::fs::create_dir_all(&store_path).unwrap();
    std::fs::create_dir_all(store_path.join("notes")).unwrap();
    std::fs::create_dir_all(store_path.join("mocs")).unwrap();
    std::fs::create_dir_all(store_path.join("attachments")).unwrap();
    std::fs::create_dir_all(store_path.join("templates")).unwrap();

    // Create minimal config file
    std::fs::write(store_path.join("config.toml"), "# Qipu configuration\n").unwrap();

    // Should discover the visible "qipu/" store
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_hidden_store_preferred_over_visible() {
    let dir = tempdir().unwrap();

    // Create both hidden and visible stores
    let hidden_path = dir.path().join(".qipu");
    let visible_path = dir.path().join("qipu");

    std::fs::create_dir_all(&hidden_path).unwrap();
    std::fs::create_dir_all(hidden_path.join("notes")).unwrap();
    std::fs::create_dir_all(hidden_path.join("mocs")).unwrap();
    std::fs::create_dir_all(hidden_path.join("attachments")).unwrap();
    std::fs::create_dir_all(hidden_path.join("templates")).unwrap();
    std::fs::write(hidden_path.join("config.toml"), "# Hidden config\n").unwrap();

    std::fs::create_dir_all(&visible_path).unwrap();
    std::fs::create_dir_all(visible_path.join("notes")).unwrap();
    std::fs::create_dir_all(visible_path.join("mocs")).unwrap();
    std::fs::create_dir_all(visible_path.join("attachments")).unwrap();
    std::fs::create_dir_all(visible_path.join("templates")).unwrap();
    std::fs::write(visible_path.join("config.toml"), "# Visible config\n").unwrap();

    // The hidden .qipu/ should be preferred over qipu/
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success();

    // Create a note in hidden store to verify it's being used
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test in hidden store"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify the note was created in the hidden store (not visible)
    let hidden_notes: Vec<_> = std::fs::read_dir(hidden_path.join("notes"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let visible_notes: Vec<_> = std::fs::read_dir(visible_path.join("notes"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(hidden_notes.len(), 1);
    assert_eq!(visible_notes.len(), 0);
}

#[test]
fn test_explicit_store_path() {
    let dir = tempdir().unwrap();
    let store_dir = dir.path().join("custom-store");

    // Init at custom location
    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "init"])
        .assert()
        .success();

    // Verify structure was created under the explicit store path
    assert!(store_dir.join("config.toml").exists());
    assert!(store_dir.join("notes").exists());
    assert!(store_dir.join("mocs").exists());
    assert!(store_dir.join("attachments").exists());
    assert!(store_dir.join("templates").exists());

    // Should be able to use with --store
    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "list"])
        .assert()
        .success();
}

#[test]
fn test_root_flag_affects_discovery_start_dir() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    let subdir = dir.path().join("somewhere/else");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&subdir).unwrap();

    // Init at root location
    qipu().current_dir(&root).arg("init").assert().success();

    // From a different directory, --root should allow discovery
    qipu()
        .current_dir(&subdir)
        .args(["--root", root.to_str().unwrap(), "list"])
        .assert()
        .success();

    // Without --root, discovery from subdir should fail
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(&subdir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_discovery_stops_at_project_boundary_with_parent_store() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/project");
    let project_subdir = project_dir.join("subdir");
    std::fs::create_dir_all(&project_subdir).unwrap();

    // Create parent store
    let parent_store = parent_dir.join(".qipu");
    std::fs::create_dir_all(&parent_store).unwrap();
    std::fs::create_dir_all(parent_store.join("notes")).unwrap();
    std::fs::create_dir_all(parent_store.join("mocs")).unwrap();
    std::fs::create_dir_all(parent_store.join("attachments")).unwrap();
    std::fs::create_dir_all(parent_store.join("templates")).unwrap();
    std::fs::write(parent_store.join("config.toml"), "# Parent store config\n").unwrap();

    // Create project marker (.git) in project directory
    std::fs::create_dir_all(project_dir.join(".git")).unwrap();

    // From project_subdir, should NOT discover parent store
    // Discovery should stop at .git boundary
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(&project_subdir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_discovery_stops_at_cargo_toml_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/rust_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Create parent store
    let parent_store = parent_dir.join(".qipu");
    std::fs::create_dir_all(&parent_store).unwrap();
    std::fs::create_dir_all(parent_store.join("notes")).unwrap();
    std::fs::create_dir_all(parent_store.join("mocs")).unwrap();
    std::fs::create_dir_all(parent_store.join("attachments")).unwrap();
    std::fs::create_dir_all(parent_store.join("templates")).unwrap();
    std::fs::write(parent_store.join("config.toml"), "# Parent store config\n").unwrap();

    // Create Cargo.toml as project marker
    std::fs::write(
        project_dir.join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )
    .unwrap();

    // From project directory, should NOT discover parent store
    // Discovery should stop at Cargo.toml boundary
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(&project_dir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

fn setup_parent_store(parent_dir: &std::path::Path) {
    let parent_store = parent_dir.join(".qipu");
    std::fs::create_dir_all(&parent_store).unwrap();
    std::fs::create_dir_all(parent_store.join("notes")).unwrap();
    std::fs::create_dir_all(parent_store.join("mocs")).unwrap();
    std::fs::create_dir_all(parent_store.join("attachments")).unwrap();
    std::fs::create_dir_all(parent_store.join("templates")).unwrap();
    std::fs::write(parent_store.join("config.toml"), "# Parent store config\n").unwrap();
}

fn assert_discovery_stops_at_boundary(project_dir: &std::path::Path, temp_dir: &std::path::Path) {
    let nonexistent_store = temp_dir.join("nonexistent-store");
    qipu()
        .current_dir(project_dir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_discovery_stops_at_hg_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/hg_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    setup_parent_store(&parent_dir);

    // Create Mercurial marker
    std::fs::create_dir_all(project_dir.join(".hg")).unwrap();

    assert_discovery_stops_at_boundary(&project_dir, dir.path());
}

#[test]
fn test_discovery_stops_at_svn_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/svn_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    setup_parent_store(&parent_dir);

    // Create Subversion marker
    std::fs::create_dir_all(project_dir.join(".svn")).unwrap();

    assert_discovery_stops_at_boundary(&project_dir, dir.path());
}

#[test]
fn test_discovery_stops_at_package_json_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/node_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    setup_parent_store(&parent_dir);

    // Create package.json as project marker
    std::fs::write(project_dir.join("package.json"), r#"{"name": "test"}"#).unwrap();

    assert_discovery_stops_at_boundary(&project_dir, dir.path());
}

#[test]
fn test_discovery_stops_at_go_mod_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/go_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    setup_parent_store(&parent_dir);

    // Create go.mod as project marker
    std::fs::write(project_dir.join("go.mod"), "module test\n").unwrap();

    assert_discovery_stops_at_boundary(&project_dir, dir.path());
}

#[test]
fn test_discovery_stops_at_pyproject_toml_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/python_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    setup_parent_store(&parent_dir);

    // Create pyproject.toml as project marker
    std::fs::write(
        project_dir.join("pyproject.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    assert_discovery_stops_at_boundary(&project_dir, dir.path());
}

#[test]
fn test_relative_store_resolved_against_root() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    let subdir = dir.path().join("somewhere/else");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&subdir).unwrap();

    // Create a store in a subdirectory of root
    let store_path = root.join("mystore");

    // Init using relative path from root
    qipu()
        .current_dir(&subdir)
        .args([
            "--root",
            root.to_str().unwrap(),
            "--store",
            "mystore",
            "init",
        ])
        .assert()
        .success();

    // Verify store was created at root/mystore, not subdir/mystore
    assert!(store_path.join("config.toml").exists());
    assert!(!subdir.join("mystore").exists());

    // Should be able to use with relative --store and --root
    qipu()
        .current_dir(&subdir)
        .args([
            "--root",
            root.to_str().unwrap(),
            "--store",
            "mystore",
            "list",
        ])
        .assert()
        .success();
}

#[test]
fn test_store_flag_plain_directory_is_invalid() {
    let dir = tempdir().unwrap();
    let store_dir = dir.path().join("not-a-store");
    std::fs::create_dir_all(&store_dir).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "list"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("invalid store"));
}
