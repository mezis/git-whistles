//! Integration tests for git staging.

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
fn staging_requires_repo() {
    let out = std::process::Command::new(bin())
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
    let dir = std::env::temp_dir().join("gw_staging_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    common::init_repo(&dir).unwrap();
    common::run_git_ok(&dir, &["checkout", "-b", "feature"]).unwrap();

    let out = std::process::Command::new(bin())
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
