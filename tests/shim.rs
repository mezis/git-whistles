//! Integration tests for git shim / unshim.

use std::fs;
use std::path::PathBuf;

fn bin() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("git-whistles")
}

#[test]
fn shim_creates_symlinks_in_dir() {
    let dir = std::env::temp_dir().join("gw_shim_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let out = std::process::Command::new(bin())
        .args(["shim", "--dir", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let git_chop = dir.join("git-chop");
    assert!(git_chop.exists(), "git-chop symlink should exist");
    assert!(fs::read_link(&git_chop).unwrap().ends_with("git-whistles"));
    assert!(!dir.join("chop").exists(), "bare subcommand name should not be shimmed");
    assert!(!dir.join("git-shim").exists(), "shim should not be shimmed");
    assert!(!dir.join("git-unshim").exists(), "unshim should not be shimmed");
    let out = std::process::Command::new(bin())
        .args(["unshim", "--dir", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!git_chop.exists());
    let _ = fs::remove_dir_all(&dir);
}
