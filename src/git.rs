//! Helpers for running git commands and querying repo state.

use std::process::Output;

use crate::exec;

/// Run a git command; returns (stdout, stderr, success).
pub fn run_git(args: &[&str]) -> std::io::Result<Output> {
    exec::git_output(args)
}

/// Run git, return stdout as String. Errors on non-zero exit or I/O error.
pub fn run_git_stdout(args: &[&str]) -> Result<String, String> {
    let out = run_git(args).map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        Ok(stdout.trim().to_string())
    } else {
        Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
    }
}

/// Run git, return success. Stderr is preserved for error message.
pub fn run_git_ok(args: &[&str]) -> Result<(), String> {
    let out = run_git(args).map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
    }
}

/// Current branch name (refs/heads/ stripped), or Err if detached / not a repo.
pub fn current_branch() -> Result<String, String> {
    let ref_name = run_git_stdout(&["symbolic-ref", "HEAD"])?;
    Ok(ref_name
        .strip_prefix("refs/heads/")
        .unwrap_or(&ref_name)
        .to_string())
}

/// Detect primary branch for origin: origin/HEAD target, else origin/main, else origin/master.
pub fn origin_primary_branch() -> Result<String, String> {
    // Try origin/HEAD symbolic-ref first
    if let Ok(ref_name) = run_git_stdout(&["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        if let Some(short) = ref_name.strip_prefix("refs/remotes/") {
            return Ok(short.to_string());
        }
    }
    // Fallback: which of origin/main or origin/master exists?
    if run_git(&["rev-parse", "origin/main"]).ok().map(|o| o.status.success()) == Some(true) {
        return Ok("origin/main".to_string());
    }
    if run_git(&["rev-parse", "origin/master"]).ok().map(|o| o.status.success()) == Some(true) {
        return Ok("origin/master".to_string());
    }
    Err("could not determine primary branch (no origin/HEAD, origin/main, or origin/master)".to_string())
}

/// Check if we're in a git repo.
pub fn in_repo() -> bool {
    run_git(&["rev-parse", "HEAD"]).map(|o| o.status.success()).unwrap_or(false)
}
