//! Integration tests for git changes.

mod common;

use std::fs;

fn setup_changes_repo(prefix: &str) -> std::path::PathBuf {
    let dir = common::temp_test_dir(prefix);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo_with_origin_master(&dir).unwrap();
    dir
}

#[test]
fn running_without_subcommand_prints_help() {
    let out = std::process::Command::new(common::git_whistles_bin())
        .output()
        .unwrap();
    assert!(!out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("Helpers for classic Git workflows"),
        "stdout: {}",
        stdout
    );
    assert!(stderr.contains("missing subcommand"), "stderr: {}", stderr);
}

#[test]
fn changes_requires_repo() {
    let out = std::process::Command::new(common::git_whistles_bin())
        .args(["changes"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn changes_in_repo_with_origin_master() {
    let dir = setup_changes_repo("gw_changes_test");
    let out = std::process::Command::new(common::git_whistles_bin())
        .args(["changes"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn changes_works_via_git_style_symlink() {
    let dir = setup_changes_repo("gw_changes_symlink");
    let link = common::symlink_to_bin(&dir, "git-changes").unwrap();

    let out = std::process::Command::new(&link)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn changes_echoes_commands_via_git_style_symlink() {
    let dir = setup_changes_repo("gw_changes_symlink_echo");
    let link = common::symlink_to_bin(&dir, "git-changes").unwrap();

    let out = std::process::Command::new(&link)
        .args(["-x"])
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("+ git log --oneline HEAD '^origin/master'"),
        "expected echoed git command, got: {}",
        stderr
    );
    let _ = fs::remove_dir_all(&dir);
}
