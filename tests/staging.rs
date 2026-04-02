//! Integration tests for git staging.

mod common;

use std::fs;

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
