//! Integration tests for git staging.

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn setup_staging_repo(prefix: &str) -> std::path::PathBuf {
    let dir = common::temp_test_dir(prefix);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    common::run_git_ok(&dir, &["checkout", "-b", "feature"]).unwrap();
    dir
}

#[test]
fn staging_requires_repo() {
    let out = std::process::Command::new(common::git_whistles_bin())
        .args(["staging"])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not a git repository"));
}

#[test]
fn staging_fails_in_minimal_repo() {
    // Repo with a feature branch but no remote/staging — staging runs and fails somewhere in the sequence
    let dir = setup_staging_repo("gw_staging_test");

    let out = std::process::Command::new(common::git_whistles_bin())
        .args(["staging"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Staging branch"),
        "expected staging command to have run, got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn staging_runs_via_git_style_symlink() {
    let dir = setup_staging_repo("gw_staging_symlink");
    let link = common::symlink_to_bin(&dir, "git-staging").unwrap();

    let out = std::process::Command::new(&link)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Staging branch feature"),
        "expected staging command to have run, got: {}",
        stderr
    );
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn staging_echoes_commands_via_git_style_symlink() {
    let dir = setup_staging_repo("gw_staging_symlink_echo");
    let link = common::symlink_to_bin(&dir, "git-staging").unwrap();

    let out = std::process::Command::new(&link)
        .args(["-x"])
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Staging branch feature"),
        "expected staging command to have run, got: {}",
        stderr
    );
    assert!(
        stderr.contains("+ git"),
        "expected echoed git command, got: {}",
        stderr
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Bare `origin`, normal repo on `main`, `staging`, `feature`, and a linked worktree on `feature`.
/// Returns `(primary repo, temp base)` for cleanup; repo is left on `main`.
fn setup_repo_with_origin_staging_and_feature_worktree(prefix: &str) -> (PathBuf, PathBuf) {
    let base = common::temp_test_dir(prefix);
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let base = base.canonicalize().unwrap();
    let bare = base.join("origin.git");
    let repo = base.join("repo");
    let feat_wt = base.join("feature_wt");

    fs::create_dir_all(&bare).unwrap();
    common::run_git_ok(&bare, &["init", "--bare"]).unwrap();
    fs::create_dir_all(&repo).unwrap();
    common::run_git_ok(&repo, &["init"]).unwrap();
    common::run_git_ok(&repo, &["config", "user.email", "test@test.com"]).unwrap();
    common::run_git_ok(&repo, &["config", "user.name", "Test"]).unwrap();
    common::run_git_ok(&repo, &["config", "commit.gpgsign", "false"]).unwrap();
    fs::write(repo.join("file.txt"), "hello\n").unwrap();
    common::run_git_ok(&repo, &["add", "file.txt"]).unwrap();
    common::run_git_ok(&repo, &["commit", "-m", "initial"]).unwrap();
    common::run_git_ok(&repo, &["branch", "-M", "main"]).unwrap();
    let bare_url = bare.canonicalize().unwrap();
    let bare_url = bare_url.to_str().unwrap();
    common::run_git_ok(&repo, &["remote", "add", "origin", bare_url]).unwrap();
    common::run_git_ok(&repo, &["push", "-u", "origin", "main"]).unwrap();

    common::run_git_ok(&repo, &["checkout", "-b", "staging"]).unwrap();
    fs::write(repo.join("staging.txt"), "staging\n").unwrap();
    common::run_git_ok(&repo, &["add", "staging.txt"]).unwrap();
    common::run_git_ok(&repo, &["commit", "-m", "staging commit"]).unwrap();
    common::run_git_ok(&repo, &["push", "-u", "origin", "staging"]).unwrap();

    common::run_git_ok(&repo, &["checkout", "main"]).unwrap();
    common::run_git_ok(&repo, &["checkout", "-b", "feature"]).unwrap();
    fs::write(repo.join("feature.txt"), "feature\n").unwrap();
    common::run_git_ok(&repo, &["add", "feature.txt"]).unwrap();
    common::run_git_ok(&repo, &["commit", "-m", "feature commit"]).unwrap();
    common::run_git_ok(&repo, &["push", "-u", "origin", "feature"]).unwrap();

    common::run_git_ok(&repo, &["checkout", "main"]).unwrap();
    fs::write(repo.join("mainline.txt"), "mainline\n").unwrap();
    common::run_git_ok(&repo, &["add", "mainline.txt"]).unwrap();
    common::run_git_ok(&repo, &["commit", "-m", "advance main"]).unwrap();
    common::run_git_ok(&repo, &["push"]).unwrap();

    let feat_str = feat_wt.to_str().unwrap();
    common::run_git_ok(&repo, &["worktree", "add", feat_str, "feature"]).unwrap();

    (repo, base)
}

#[test]
fn staging_succeeds_when_feature_branch_in_other_worktree() {
    let (repo, base) =
        setup_repo_with_origin_staging_and_feature_worktree("gw_staging_worktree_ok");

    let out = Command::new(common::git_whistles_bin())
        .args(["staging", "feature"])
        .current_dir(&repo)
        .output()
        .unwrap();

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        panic!("staging failed\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }

    let head = common::run_git(&repo, &["symbolic-ref", "--short", "HEAD"]).unwrap();
    let branch = String::from_utf8_lossy(&head.stdout);
    assert_eq!(
        branch.trim(),
        "main",
        "expected to return to main after staging from worktree path"
    );

    let merge = common::run_git(&repo, &["log", "-1", "--oneline", "staging"]).unwrap();
    let merge_msg = String::from_utf8_lossy(&merge.stdout);
    assert!(
        merge_msg.contains("Merge branch 'feature'"),
        "expected merge on staging, got: {merge_msg}"
    );

    let _ = fs::remove_dir_all(&base);
}
