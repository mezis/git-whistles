//! Integration tests for git changes.

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
fn changes_requires_repo() {
    let out = std::process::Command::new(bin())
        .args(["changes"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn changes_in_repo_with_origin_master() {
    let dir = std::env::temp_dir().join("gw_changes_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    common::run_git_ok(&dir, &["remote", "add", "origin", "https://example.com/repo.git"]).unwrap();
    common::run_git_ok(&dir, &["update-ref", "refs/remotes/origin/master", "HEAD"]).unwrap();
    let out = std::process::Command::new(bin())
        .args(["changes"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let _ = fs::remove_dir_all(&dir);
}
