//! Shared helpers for integration tests.

use std::path::Path;
use std::process::Command;

pub fn run_git(dir: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git").current_dir(dir).args(args).output()
}

pub fn run_git_ok(dir: &Path, args: &[&str]) -> Result<(), String> {
    let out = run_git(dir, args).map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

pub fn init_repo(dir: &Path) -> Result<(), String> {
    run_git_ok(dir, &["init"])?;
    run_git_ok(dir, &["config", "user.email", "test@test.com"])?;
    run_git_ok(dir, &["config", "user.name", "Test"])?;
    std::fs::write(dir.join("file.txt"), "hello").map_err(|e| e.to_string())?;
    run_git_ok(dir, &["add", "file.txt"])?;
    run_git_ok(dir, &["commit", "-m", "initial"])?;
    Ok(())
}

#[allow(dead_code)]
pub fn git_whistles_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("git-whistles")
}
