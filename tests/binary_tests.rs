use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn test_binary_runs() {
    let mut cmd = cargo_bin_cmd!("qipu");
    cmd.arg("--version").assert().success();
}

#[test]
fn test_binary_help() {
    let mut cmd = cargo_bin_cmd!("qipu");
    cmd.arg("--help").assert().success();
}

#[test]
fn test_binary_init() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let mut cmd = cargo_bin_cmd!("qipu");
    cmd.current_dir(dir.path()).arg("init").assert().success();
}

#[test]
fn test_binary_capture() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();

    let mut init_cmd = cargo_bin_cmd!("qipu");
    init_cmd
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let mut capture_cmd = cargo_bin_cmd!("qipu");
    capture_cmd
        .current_dir(dir.path())
        .args(["capture", "--type", "fleeting"])
        .write_stdin("test note")
        .assert()
        .success();
}

#[test]
fn test_binary_list() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();

    let mut init_cmd = cargo_bin_cmd!("qipu");
    init_cmd
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let mut capture_cmd = cargo_bin_cmd!("qipu");
    capture_cmd
        .current_dir(dir.path())
        .args(["capture", "--type", "fleeting"])
        .write_stdin("test note")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("qipu");
    list_cmd
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success();
}
