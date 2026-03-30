//! Integration tests for git ff-all-branches.

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
fn ff_all_branches_requires_repo() {
    let out = std::process::Command::new(bin())
        .args(["ff-all-branches"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn ff_all_branches_dry_run_in_repo() {
    let dir = std::env::temp_dir().join("gw_ff_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    let out = std::process::Command::new(bin())
        .args(["ff-all-branches", "--dry-run"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let _ = fs::remove_dir_all(&dir);
}
