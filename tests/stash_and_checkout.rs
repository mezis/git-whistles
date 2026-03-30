//! Integration tests for git stash-and-checkout.

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
fn stash_and_checkout_requires_repo() {
    let out = std::process::Command::new(bin())
        .args(["stash-and-checkout", "other"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn stash_and_checkout_switches_branch() {
    let dir = std::env::temp_dir().join("gw_stash_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    common::run_git_ok(&dir, &["checkout", "-b", "other"]).unwrap();
    common::run_git_ok(&dir, &["checkout", "-"]).unwrap(); // back to default branch
    std::fs::write(dir.join("file.txt"), "modified").unwrap();
    let out = std::process::Command::new(bin())
        .args(["stash-and-checkout", "other"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).trim() == "other");
    let _ = fs::remove_dir_all(&dir);
}
