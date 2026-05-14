//! Helpers for running git commands and querying repo state.

use std::path::{Path, PathBuf};
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

/// Run git with `-C` so commands execute in another working tree of the same repo.
pub fn run_git_ok_in(repo: &Path, args: &[&str]) -> Result<(), String> {
    let repo_str = repo
        .to_str()
        .ok_or_else(|| "worktree path is not valid UTF-8".to_string())?;
    let mut full: Vec<&str> = vec!["-C", repo_str];
    full.extend_from_slice(args);
    run_git_ok(&full)
}

/// Top-level directory of the current worktree (canonicalized).
pub fn worktree_root() -> Result<PathBuf, String> {
    let raw = run_git_stdout(&["rev-parse", "--show-toplevel"])?;
    PathBuf::from(raw)
        .canonicalize()
        .map_err(|e| e.to_string())
}

/// If `branch` is checked out in a linked worktree other than the current one, return that path.
pub fn other_worktree_path_for_branch(branch: &str) -> Result<Option<PathBuf>, String> {
    let want = format!("refs/heads/{branch}");
    let here = worktree_root()?;
    let porcelain = run_git_stdout(&["worktree", "list", "--porcelain"])?;
    let mut blocks: Vec<(PathBuf, Option<String>)> = Vec::new();
    let mut cur_path: Option<PathBuf> = None;
    let mut cur_branch: Option<String> = None;
    for line in porcelain.lines() {
        if line.is_empty() {
            if let Some(p) = cur_path.take() {
                blocks.push((p, cur_branch.take()));
            }
            continue;
        }
        if let Some(p) = line.strip_prefix("worktree ") {
            if let Some(old) = cur_path.take() {
                blocks.push((old, cur_branch.take()));
            }
            cur_path = Some(PathBuf::from(p));
        } else if let Some(b) = line.strip_prefix("branch ") {
            cur_branch = Some(b.trim().to_string());
        }
    }
    if let Some(p) = cur_path.take() {
        blocks.push((p, cur_branch.take()));
    }
    for (path, br) in blocks {
        if br.as_deref() != Some(want.as_str()) {
            continue;
        }
        let path = path
            .canonicalize()
            .map_err(|e| format!("worktree path {}: {}", path.display(), e))?;
        if path != here {
            return Ok(Some(path));
        }
    }
    Ok(None)
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
