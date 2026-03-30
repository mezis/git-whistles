//! Integration tests for git ff-all-branches.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

fn bin() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("git-whistles")
}

fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let out = common::run_git(dir, args).unwrap();
    assert!(
        out.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn setup_fetch_scenario(name: &str) -> (PathBuf, PathBuf, String) {
    let root = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let remote = root.join("remote.git");
    let seed = root.join("seed");
    let clone = root.join("clone");

    common::run_git_ok(&root, &["init", "--bare", "remote.git"]).unwrap();

    fs::create_dir_all(&seed).unwrap();
    common::init_repo(&seed).unwrap();
    let branch = git_stdout(&seed, &["branch", "--show-current"]);
    common::run_git_ok(&seed, &["remote", "add", "origin", remote.to_str().unwrap()]).unwrap();
    common::run_git_ok(&seed, &["push", "-u", "origin", branch.as_str()]).unwrap();

    common::run_git_ok(&root, &["clone", remote.to_str().unwrap(), clone.to_str().unwrap()]).unwrap();

    fs::write(seed.join("file.txt"), "hello again").unwrap();
    common::run_git_ok(&seed, &["commit", "-am", "remote update"]).unwrap();
    common::run_git_ok(&seed, &["push", "origin", branch.as_str()]).unwrap();

    let remote_head = git_stdout(&seed, &["rev-parse", "HEAD"]);
    (root, clone, remote_head)
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

#[test]
fn ff_all_branches_fetches_by_default() {
    let (root, clone, remote_head) = setup_fetch_scenario("gw_ff_fetch_default");
    let local_head_before = git_stdout(&clone, &["rev-parse", "HEAD"]);
    assert_ne!(local_head_before, remote_head);

    let out = std::process::Command::new(bin())
        .args(["ff-all-branches"])
        .current_dir(&clone)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let local_head_after = git_stdout(&clone, &["rev-parse", "HEAD"]);
    assert_eq!(local_head_after, remote_head);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn ff_all_branches_can_skip_fetch() {
    let (root, clone, remote_head) = setup_fetch_scenario("gw_ff_no_fetch");
    let local_head_before = git_stdout(&clone, &["rev-parse", "HEAD"]);
    assert_ne!(local_head_before, remote_head);

    let out = std::process::Command::new(bin())
        .args(["ff-all-branches", "--no-fetch"])
        .current_dir(&clone)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let local_head_after = git_stdout(&clone, &["rev-parse", "HEAD"]);
    assert_eq!(local_head_after, local_head_before);
    assert_ne!(local_head_after, remote_head);

    let _ = fs::remove_dir_all(&root);
}
