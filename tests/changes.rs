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
fn changes_streams_git_log_with_dash_v_via_symlink() {
    let dir = setup_changes_repo("gw_changes_symlink_stream");
    common::run_git_ok(&dir, &["commit", "--allow-empty", "-m", "ahead"]).unwrap();
    let link = common::symlink_to_bin(&dir, "git-changes").unwrap();

    let out = std::process::Command::new(&link)
        .args(["-v"])
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ahead"),
        "expected streamed git log to include commit message, got: {}",
        stdout
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

#[test]
fn changes_forwards_extra_args_to_git_log() {
    let dir = setup_changes_repo("gw_changes_extra_args");
    common::run_git_ok(&dir, &["commit", "--allow-empty", "-m", "ahead"]).unwrap();

    let out = std::process::Command::new(common::git_whistles_bin())
        .args(["changes", "-x", "--stat"])
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
        stderr.contains("+ git log --oneline HEAD '^origin/master' --stat"),
        "expected extra git log args in echoed command, got: {}",
        stderr
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn changes_forwards_pathspec_after_double_dash() {
    let dir = setup_changes_repo("gw_changes_pathspec");
    fs::write(dir.join("file.txt"), "changed").unwrap();
    common::run_git_ok(&dir, &["add", "file.txt"]).unwrap();
    common::run_git_ok(&dir, &["commit", "-m", "touch file"]).unwrap();
    common::run_git_ok(&dir, &["commit", "--allow-empty", "-m", "empty ahead"]).unwrap();

    let with_path = std::process::Command::new(common::git_whistles_bin())
        .args(["changes", "--", "file.txt"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        with_path.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&with_path.stderr)
    );
    let stdout = String::from_utf8_lossy(&with_path.stdout);
    assert!(
        stdout.contains("touch file"),
        "expected path-filtered log to include file commit, got: {}",
        stdout
    );
    assert!(
        !stdout.contains("empty ahead"),
        "expected path-filtered log to omit empty commit, got: {}",
        stdout
    );
    let _ = fs::remove_dir_all(&dir);
}
