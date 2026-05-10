use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_workspace_merge_invalid_strategy_shows_guidance() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "merge", "scratch", ".", "--strategy", "merge"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "Use: qipu workspace merge scratch . --strategy merge-links",
        ))
        .stderr(predicate::str::contains(
            "Other strategies: skip, overwrite, rename",
        ));
}
