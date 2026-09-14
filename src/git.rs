//! Helpers for running git commands and querying repo state.

use std::path::{Path, PathBuf};
use std::process::Output;

use crate::exec;

/// Run a git command with full stdout/stderr capture (porcelain, binary-safe parsing).
pub fn run_git(args: &[&str]) -> std::io::Result<Output> {
    exec::git_output_captured(args)
}

/// Run git, return stdout as String. Errors on non-zero exit or I/O error.
pub fn run_git_stdout(args: &[&str]) -> Result<String, String> {
    let out = exec::git_output_stdout(args).map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        Ok(stdout.trim().to_string())
    } else {
        Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
    }
}

/// Run git, return success. When not streaming, stderr is preserved for error messages.
pub fn run_git_ok(args: &[&str]) -> Result<(), String> {
    exec::git_side_effect(args)
}

/// UTF-8 path for `git -C`, or an error if the path is not valid Unicode.
fn repo_as_str(repo: &Path) -> Result<&str, String> {
    repo.to_str()
        .ok_or_else(|| "worktree path is not valid UTF-8".to_string())
}

/// Prepend `git -C <repo>` to `args`.
fn git_args_in<'a>(repo: &'a Path, args: &'a [&'a str]) -> Result<Vec<&'a str>, String> {
    let repo_str = repo_as_str(repo)?;
    let mut full: Vec<&str> = vec!["-C", repo_str];
    full.extend_from_slice(args);
    Ok(full)
}

/// Run git with `-C` so commands execute in another working tree of the same repo.
pub fn run_git_ok_in(repo: &Path, args: &[&str]) -> Result<(), String> {
    let full = git_args_in(repo, args)?;
    run_git_ok(&full)
}

/// Fully captured git in another working tree (porcelain: do not trim stdout).
pub fn run_git_in(repo: &Path, args: &[&str]) -> std::io::Result<Output> {
    match git_args_in(repo, args) {
        Ok(full) => run_git(&full),
        Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, err)),
    }
}

/// Same as [`run_git_in`], mapping I/O errors to a string.
pub fn run_git_captured_in(repo: &Path, args: &[&str]) -> Result<Output, String> {
    run_git_in(repo, args).map_err(|err| err.to_string())
}

/// Run git in another working tree; return trimmed stdout (not for porcelain).
pub fn run_git_stdout_in(repo: &Path, args: &[&str]) -> Result<String, String> {
    let full = git_args_in(repo, args)?;
    run_git_stdout(&full)
}

/// Captured git side-effect in another working tree (never inherits stdout).
pub fn run_git_ok_captured_in(repo: &Path, args: &[&str]) -> Result<(), String> {
    let out = run_git_captured_in(repo, args)?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
    }
}

/// Top-level directory of the current worktree (canonicalized).
pub fn worktree_root() -> Result<PathBuf, String> {
    worktree_root_in(Path::new("."))
}

/// Top-level directory of the worktree at `repo` (canonicalized).
pub fn worktree_root_in(repo: &Path) -> Result<PathBuf, String> {
    let raw = run_git_stdout_in(repo, &["rev-parse", "--show-toplevel"])?;
    PathBuf::from(raw).canonicalize().map_err(|e| e.to_string())
}

/// If `branch` is checked out in a linked worktree other than the current one, return that path.
pub fn other_worktree_path_for_branch(branch: &str) -> Result<Option<PathBuf>, String> {
    let here = worktree_root()?;
    let trees = crate::worktree::list_worktrees(&here)?;
    for tree in trees {
        if tree.branch.as_deref() != Some(branch) {
            continue;
        }
        let path = tree
            .path
            .canonicalize()
            .map_err(|e| format!("worktree path {}: {}", tree.path.display(), e))?;
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
    if run_git_ok(&["rev-parse", "origin/main"]).is_ok() {
        return Ok("origin/main".to_string());
    }
    if run_git_ok(&["rev-parse", "origin/master"]).is_ok() {
        return Ok("origin/master".to_string());
    }
    Err(
        "could not determine primary branch (no origin/HEAD, origin/main, or origin/master)"
            .to_string(),
    )
}

/// Check if we're in a git repo.
pub fn in_repo() -> bool {
    run_git_ok(&["rev-parse", "HEAD"]).is_ok()
}
