//! Git worktree porcelain types and listing.

use std::path::{Path, PathBuf};

use crate::git;

/// One checkout registered in `git worktree list --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    /// Absolute path as Git printed it (not necessarily canonicalized).
    pub path: PathBuf,
    /// Short branch name when attached; `None` when detached.
    pub branch: Option<String>,
    /// Whether Git reports the worktree as locked.
    pub locked: bool,
    /// Whether this entry is a bare repository.
    pub is_bare: bool,
}

/// List worktrees for the repository that contains `clone` (untrimmed porcelain).
pub fn list_worktrees(clone: &Path) -> Result<Vec<Worktree>, String> {
    let out = git::run_git_in(clone, &["worktree", "list", "--porcelain"])
        .map_err(|err| err.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git worktree list failed: {}", stderr.trim()));
    }
    // Porcelain uses blank lines as record separators; trimming would drop the last separator.
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(parse_worktree_porcelain(&stdout))
}

/// Main clone directory: parent of `.git` when linked, otherwise this checkout (or the bare repo).
#[allow(dead_code)] // used by worktree-list (later commits)
pub fn main_clone_root() -> Result<PathBuf, String> {
    main_clone_root_in(Path::new("."))
}

/// Same as [`main_clone_root`], resolved with `git -C repo`.
#[allow(dead_code)] // used by worktree-list (later commits)
pub fn main_clone_root_in(repo: &Path) -> Result<PathBuf, String> {
    let git_dir = git::run_git_stdout_in(repo, &["rev-parse", "--git-dir"])?;
    let common_dir = git::run_git_stdout_in(repo, &["rev-parse", "--git-common-dir"])?;
    let git_dir_path = resolve_git_path(repo, &git_dir)?;
    let common_path = resolve_git_path(repo, &common_dir)?;
    if git_dir_path == common_path {
        return git::worktree_root_in(repo);
    }
    if common_path.file_name().and_then(|name| name.to_str()) == Some(".git") {
        common_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "git common dir has no parent".to_string())
    } else {
        Ok(common_path)
    }
}

/// Turn a possibly-relative `rev-parse` path into a canonical absolute path.
fn resolve_git_path(repo: &Path, raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    let absolute = if path.is_absolute() {
        path
    } else {
        repo.join(path)
    };
    absolute.canonicalize().map_err(|err| err.to_string())
}

/// Parse `git worktree list --porcelain` stdout into domain records.
pub fn parse_worktree_porcelain(stdout: &str) -> Vec<Worktree> {
    let mut trees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut current_locked = false;
    let mut current_bare = false;

    for line in stdout.lines() {
        if line.is_empty() {
            if let Some(path) = current_path.take() {
                trees.push(Worktree {
                    path,
                    branch: current_branch.take(),
                    locked: current_locked,
                    is_bare: current_bare,
                });
                current_locked = false;
                current_bare = false;
            }
            continue;
        }
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(old_path) = current_path.take() {
                trees.push(Worktree {
                    path: old_path,
                    branch: current_branch.take(),
                    locked: current_locked,
                    is_bare: current_bare,
                });
                current_locked = false;
                current_bare = false;
            }
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            current_branch = Some(short_branch_name(branch_ref.trim()));
        } else if line == "detached" {
            current_branch = None;
        } else if line == "bare" {
            current_bare = true;
        } else if line == "locked" || line.starts_with("locked ") {
            current_locked = true;
        }
    }
    if let Some(path) = current_path.take() {
        trees.push(Worktree {
            path,
            branch: current_branch.take(),
            locked: current_locked,
            is_bare: current_bare,
        });
    }
    trees
}

fn short_branch_name(branch_ref: &str) -> String {
    branch_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(branch_ref)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture_attached_and_detached() -> String {
        "worktree /tmp/repo\nHEAD abcdef\nbranch refs/heads/main\n\nworktree /tmp/repo-feature\nHEAD 123456\ndetached\n".to_string()
    }

    #[test]
    fn parse_attached_and_detached_blocks() {
        let trees = parse_worktree_porcelain(&fixture_attached_and_detached());
        assert_eq!(trees.len(), 2);
        assert_eq!(trees[0].path, PathBuf::from("/tmp/repo"));
        assert_eq!(trees[0].branch.as_deref(), Some("main"));
        assert!(!trees[0].locked);
        assert!(!trees[0].is_bare);
        assert_eq!(trees[1].path, PathBuf::from("/tmp/repo-feature"));
        assert_eq!(trees[1].branch, None);
    }

    #[test]
    fn parse_locked_and_bare() {
        let porcelain = "worktree /tmp/locked-wt\nHEAD abc\nbranch refs/heads/topic\nlocked reason here\n\nworktree /tmp/bare.git\nHEAD def\nbare\n";
        let trees = parse_worktree_porcelain(porcelain);
        assert_eq!(trees.len(), 2);
        assert!(trees[0].locked);
        assert_eq!(trees[0].branch.as_deref(), Some("topic"));
        assert!(trees[1].is_bare);
        assert!(!trees[1].locked);
    }

    #[test]
    fn parse_last_block_without_trailing_blank_line() {
        let porcelain = "worktree /tmp/only\nHEAD abc\nbranch refs/heads/main\n";
        let trees = parse_worktree_porcelain(porcelain);
        assert_eq!(trees.len(), 1);
        assert_eq!(trees[0].branch.as_deref(), Some("main"));
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), unique))
    }

    fn run_git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn init_repo(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        run_git(dir, &["init"]);
        run_git(dir, &["config", "user.email", "test@test.com"]);
        run_git(dir, &["config", "user.name", "Test"]);
        run_git(dir, &["config", "commit.gpgsign", "false"]);
        fs::write(dir.join("file.txt"), "hello").unwrap();
        run_git(dir, &["add", "file.txt"]);
        run_git(dir, &["commit", "-m", "initial"]);
    }

    #[test]
    fn main_clone_root_is_primary_checkout() {
        let dir = temp_dir("gw_main_clone");
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        let root = main_clone_root_in(&dir).unwrap();
        assert_eq!(root, dir.canonicalize().unwrap());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn main_clone_root_from_linked_worktree() {
        let base = temp_dir("gw_linked_clone");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let repo = base.join("repo");
        let linked = base.join("feature_wt");
        init_repo(&repo);
        run_git(&repo, &["branch", "feature"]);
        run_git(
            &repo,
            &["worktree", "add", linked.to_str().unwrap(), "feature"],
        );
        let from_primary = main_clone_root_in(&repo).unwrap();
        let from_linked = main_clone_root_in(&linked).unwrap();
        let expected = repo.canonicalize().unwrap();
        assert_eq!(from_primary, expected);
        assert_eq!(from_linked, expected);
        let trees = list_worktrees(&repo).unwrap();
        assert!(trees.len() >= 2);
        let _ = fs::remove_dir_all(&base);
    }
}
