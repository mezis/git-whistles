//! Integration tests for git chop.

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
fn chop_requires_repo() {
    let out = std::process::Command::new(bin())
        .args(["chop", "some-branch"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn chop_deletes_local_branch() {
    let dir = std::env::temp_dir().join("gw_chop_test");
    let remote_dir = std::env::temp_dir().join("gw_chop_remote");
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&remote_dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    std::process::Command::new("git")
        .args(["init", "--bare", remote_dir.to_str().unwrap()])
        .output()
        .unwrap();
    common::run_git_ok(&dir, &["remote", "add", "origin", remote_dir.to_str().unwrap()]).unwrap();
    common::run_git_ok(&dir, &["push", "-u", "origin", "HEAD"]).unwrap();
    common::run_git_ok(&dir, &["checkout", "-b", "feature"]).unwrap();
    std::fs::write(dir.join("file.txt"), "feature change").unwrap();
    common::run_git_ok(&dir, &["add", "file.txt"]).unwrap();
    common::run_git_ok(&dir, &["commit", "-m", "feature"]).unwrap();
    common::run_git_ok(&dir, &["push", "origin", "feature"]).unwrap();
    common::run_git_ok(&dir, &["checkout", "-"]).unwrap();
    let out = std::process::Command::new(bin())
        .args(["chop", "feature"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let out = std::process::Command::new("git")
        .args(["branch"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&out.stdout);
    assert!(!branches.contains("feature"));
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&remote_dir);
}
