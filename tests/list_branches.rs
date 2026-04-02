//! Integration tests for git list-branches.

mod common;

use std::fs;
use std::path::PathBuf;

fn bin() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("git-whistles")
}

#[test]
fn list_branches_requires_repo() {
    let out = std::process::Command::new(bin())
        .args(["list-branches"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn list_branches_in_repo() {
    let dir = std::env::temp_dir().join("gw_list_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    common::run_git_ok(&dir, &["checkout", "-b", "feature"]).unwrap();
    // Use HEAD as integration branch so we don't need a remote
    let out = std::process::Command::new(bin())
        .args(["list-branches", "--integration", "HEAD"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("feature"), "expected feature branch in output: {stdout}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn list_branches_default_integration_matches_origin_primary() {
    let dir = common::temp_test_dir("gw_list_default_integration");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo_with_origin_master(&dir).unwrap();
    common::run_git_ok(&dir, &["checkout", "-b", "feature"]).unwrap();
    let out = std::process::Command::new(common::git_whistles_bin())
        .args(["list-branches"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("against origin/master"),
        "expected default integration in output: {stdout}"
    );
    assert!(stdout.contains("feature"), "expected feature branch in output: {stdout}");
    let _ = fs::remove_dir_all(&dir);
}
