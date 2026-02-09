//! Tests for workspace attachment handling
//!
//! Verifies that attachments are properly copied when:
//! - Creating workspaces from notes with attachments
//! - Merging workspaces back to primary

use std::fs;
use tempfile::tempdir;

use crate::support::{extract_id, qipu};

fn setup_attachment(store_path: &std::path::Path, filename: &str, content: &[u8]) {
    let attachments_dir = store_path.join("attachments");
    fs::create_dir_all(&attachments_dir).unwrap();
    fs::write(attachments_dir.join(filename), content).unwrap();
}

fn attachment_exists(store_path: &std::path::Path, filename: &str) -> bool {
    store_path.join("attachments").join(filename).exists()
}

#[test]
fn test_workspace_new_copy_primary_preserves_attachments() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    // Create a note with an attachment reference
    let output = qipu()
        .arg("create")
        .arg("Note with attachment")
        .current_dir(root)
        .output()
        .unwrap();
    let note_id = extract_id(&output);

    // Add attachment reference to the note
    let note_body = "See attachment: ![test](../attachments/test.txt)";
    fs::write(
        root.join(".qipu/notes")
            .join(format!("{}-note-with-attachment.md", note_id)),
        format!(
            "---\nid: {}\ntitle: Note with attachment\n---\n\n{}",
            note_id, note_body
        ),
    )
    .unwrap();

    // Create the attachment
    setup_attachment(&root.join(".qipu"), "test.txt", b"attachment content");

    // Create workspace with copy-primary
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_attachments")
        .arg("--copy-primary")
        .current_dir(root)
        .assert()
        .success();

    // Verify attachment was copied to workspace
    assert!(
        attachment_exists(&root.join(".qipu/workspaces/ws_attachments"), "test.txt"),
        "Attachment should be copied to workspace"
    );
}

#[test]
fn test_workspace_merge_copies_attachments() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    // Create a note in primary store
    let output = qipu()
        .arg("create")
        .arg("Primary Note")
        .current_dir(root)
        .output()
        .unwrap();
    let _primary_id = extract_id(&output);

    // Create empty workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_merge_attach")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_merge_attach");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // Create a note in workspace with attachment reference
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note")
        .current_dir(root)
        .output()
        .unwrap();
    let ws_note_id = extract_id(&output);

    // Add attachment to workspace
    setup_attachment(
        &ws_store_path,
        "workspace_file.txt",
        b"workspace attachment",
    );

    // Update note file directly to include attachment reference
    let note_path = ws_store_path
        .join("notes")
        .join(format!("{}-workspace-note.md", ws_note_id));
    let note_content = format!(
        "---\nid: {}\ntitle: Workspace Note\n---\n\nSee: ![file](../attachments/workspace_file.txt)",
        ws_note_id
    );
    fs::write(&note_path, note_content).unwrap();

    // Merge workspace back to primary
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_merge_attach")
        .arg(".")
        .current_dir(root)
        .assert()
        .success();

    // Verify attachment was copied to primary
    assert!(
        attachment_exists(&root.join(".qipu"), "workspace_file.txt"),
        "Attachment should be copied to primary store during merge"
    );
}

#[test]
fn test_workspace_merge_overwrite_preserves_attachments() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    // Create a note in primary
    let output = qipu()
        .arg("create")
        .arg("Shared Note")
        .current_dir(root)
        .output()
        .unwrap();
    let shared_id = extract_id(&output);

    // Create workspace with same ID
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_overwrite")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_overwrite");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // Create conflicting note with attachment
    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Version")
        .arg("--id")
        .arg(&shared_id)
        .current_dir(root)
        .assert()
        .success();

    setup_attachment(&ws_store_path, "overwrite_file.txt", b"overwrite content");

    // Update note file directly to include attachment reference
    let note_path = ws_store_path
        .join("notes")
        .join(format!("{}-workspace-version.md", shared_id));
    let note_content = format!(
        "---\nid: {}\ntitle: Workspace Version\n---\n\nSee: ![file](../attachments/overwrite_file.txt)",
        shared_id
    );
    fs::write(&note_path, note_content).unwrap();

    // Merge with overwrite strategy
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_overwrite")
        .arg(".")
        .arg("--strategy")
        .arg("overwrite")
        .current_dir(root)
        .assert()
        .success();

    // Verify attachment was copied
    assert!(
        attachment_exists(&root.join(".qipu"), "overwrite_file.txt"),
        "Attachment should be copied during overwrite merge"
    );
}

#[test]
fn test_workspace_new_from_tag_copies_attachments() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    // Create a note with tag and attachment
    let output = qipu()
        .args(["create", "Tagged Note", "--tag", "test-tag"])
        .current_dir(root)
        .output()
        .unwrap();
    let _note_id = extract_id(&output);

    // Add attachment reference
    let note_body = "![test](../attachments/tagged.txt)";
    fs::write(
        root.join(".qipu/notes").join("qp-*.md"),
        format!(
            "---\nid: {}\ntitle: Tagged Note\ntags:\n  - test-tag\n---\n\n{}",
            _note_id, note_body
        ),
    )
    .unwrap();

    setup_attachment(&root.join(".qipu"), "tagged.txt", b"tagged content");

    // Create workspace from tag
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_tag")
        .arg("--from-tag")
        .arg("test-tag")
        .current_dir(root)
        .assert()
        .success();

    // Attachment should be copied
    assert!(
        attachment_exists(&root.join(".qipu/workspaces/ws_tag"), "tagged.txt"),
        "Attachment should be copied when creating workspace from tag"
    );
}

#[test]
fn test_workspace_merge_rename_copies_attachments() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    // Create note in primary
    let output = qipu()
        .arg("create")
        .arg("Primary Note")
        .current_dir(root)
        .output()
        .unwrap();
    let shared_id = extract_id(&output);

    // Create workspace with conflicting note
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_rename")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_rename");
    let ws_store_str = ws_store_path.to_str().unwrap();

    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note")
        .arg("--id")
        .arg(&shared_id)
        .current_dir(root)
        .assert()
        .success();

    setup_attachment(&ws_store_path, "rename_file.txt", b"rename content");

    // Update note file directly to include attachment reference
    let note_path = ws_store_path
        .join("notes")
        .join(format!("{}-workspace-note.md", shared_id));
    let note_content = format!(
        "---\nid: {}\ntitle: Workspace Note\n---\n\nSee: ![file](../attachments/rename_file.txt)",
        shared_id
    );
    fs::write(&note_path, note_content).unwrap();

    // Merge with rename strategy
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_rename")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // Attachment should be copied even with rename
    assert!(
        attachment_exists(&root.join(".qipu"), "rename_file.txt"),
        "Attachment should be copied during rename merge"
    );
}
