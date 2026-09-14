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
    let out = git::run_git_captured_in(clone, &["worktree", "list", "--porcelain"])?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git worktree list failed: {}", stderr.trim()));
    }
    // Porcelain uses blank lines as record separators; trimming would drop the last separator.
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(parse_worktree_porcelain(&stdout))
}

/// Main clone directory: parent of `.git` when linked, otherwise this checkout (or the bare repo).
pub fn main_clone_root() -> Result<PathBuf, String> {
    main_clone_root_in(Path::new("."))
}

/// Same as [`main_clone_root`], resolved with `git -C repo`.
pub fn main_clone_root_in(repo: &Path) -> Result<PathBuf, String> {
    let git_dir = git::run_git_stdout_in(repo, &["rev-parse", "--git-dir"])?;
    let common_dir = git::run_git_stdout_in(repo, &["rev-parse", "--git-common-dir"])?;
    let git_dir_path = resolve_git_path(repo, &git_dir)?;
    let common_path = resolve_git_path(repo, &common_dir)?;
    if git_dir_path == common_path {
        let bare = git::run_git_stdout_in(repo, &["rev-parse", "--is-bare-repository"])?;
        if bare == "true" {
            return Ok(common_path);
        }
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
            flush_worktree(
                &mut trees,
                &mut current_path,
                &mut current_branch,
                &mut current_locked,
                &mut current_bare,
            );
            continue;
        }
        if let Some(path) = line.strip_prefix("worktree ") {
            flush_worktree(
                &mut trees,
                &mut current_path,
                &mut current_branch,
                &mut current_locked,
                &mut current_bare,
            );
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
    flush_worktree(
        &mut trees,
        &mut current_path,
        &mut current_branch,
        &mut current_locked,
        &mut current_bare,
    );
    trees
}

fn flush_worktree(
    trees: &mut Vec<Worktree>,
    current_path: &mut Option<PathBuf>,
    current_branch: &mut Option<String>,
    current_locked: &mut bool,
    current_bare: &mut bool,
) {
    if let Some(path) = current_path.take() {
        trees.push(Worktree {
            path,
            branch: current_branch.take(),
            locked: *current_locked,
            is_bare: *current_bare,
        });
        *current_locked = false;
        *current_bare = false;
    }
}

fn short_branch_name(branch_ref: &str) -> String {
    branch_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(branch_ref)
        .to_string()
}

/// List every clone's worktrees concurrently, then return results in `clones` order.
///
/// One OS thread per clone so independent `git worktree list` calls overlap. Results are
/// joined in input order (not completion order) so picker grouping matches the registry.
pub fn list_worktrees_for_clones(
    clones: &[PathBuf],
) -> Vec<(PathBuf, Result<Vec<Worktree>, String>)> {
    std::thread::scope(|scope| {
        let mut joins = Vec::with_capacity(clones.len());
        for clone in clones {
            let clone = clone.clone();
            joins.push(scope.spawn(move || {
                let listed = list_worktrees(&clone);
                (clone, listed)
            }));
        }
        joins
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(pair) => pair,
                Err(_) => (
                    PathBuf::new(),
                    Err("worktree list thread panicked".to_string()),
                ),
            })
            .collect()
    })
}

/// Uncommitted and untracked paths from `git status --porcelain -uall`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtySample {
    /// First paths (up to the requested cap).
    pub paths: Vec<String>,
    /// Total dirty entries including those not listed in `paths`.
    pub total: usize,
}

impl DirtySample {
    /// Whether the worktree has any uncommitted or untracked paths.
    pub fn is_dirty(&self) -> bool {
        self.total > 0
    }
}

/// Sample dirty paths in `worktree`, keeping at most `limit` names.
pub fn status_sample(worktree: &Path, limit: usize) -> Result<DirtySample, String> {
    let out = git::run_git_captured_in(worktree, &["status", "--porcelain", "-uall"])?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git status failed: {}", stderr.trim()));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(parse_status_porcelain(&stdout, limit))
}

/// Parse porcelain status lines into a capped sample.
pub fn parse_status_porcelain(stdout: &str, limit: usize) -> DirtySample {
    let mut paths = Vec::new();
    let mut total = 0;
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        total += 1;
        if paths.len() < limit {
            if let Some(path) = porcelain_status_path(line) {
                paths.push(path.to_string());
            }
        }
    }
    DirtySample { paths, total }
}

fn porcelain_status_path(line: &str) -> Option<&str> {
    if line.len() < 4 {
        return None;
    }
    Some(line[3..].trim())
}

/// Remove a linked worktree. `force` is `--force`; locked trees need `--force` twice.
pub fn remove_worktree(
    clone: &Path,
    worktree: &Path,
    force: bool,
    locked: bool,
) -> Result<(), String> {
    let worktree_str = worktree
        .to_str()
        .ok_or_else(|| "worktree path is not valid UTF-8".to_string())?;
    let args: Vec<&str> = if locked {
        vec!["worktree", "remove", "--force", "--force", worktree_str]
    } else if force {
        vec!["worktree", "remove", "--force", worktree_str]
    } else {
        vec!["worktree", "remove", worktree_str]
    };
    git::run_git_ok_captured_in(clone, &args)
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
    fn main_clone_root_of_bare_repo_is_the_common_dir() {
        let dir = temp_dir("gw_bare_clone");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        run_git(&dir, &["init", "--bare"]);
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

    #[test]
    fn parse_status_caps_at_five_and_counts_total() {
        let porcelain = " M a.rs\n M b.rs\n?? c.rs\n?? d.rs\n?? e.rs\n?? f.rs\n";
        let sample = parse_status_porcelain(porcelain, 5);
        assert_eq!(sample.total, 6);
        assert_eq!(sample.paths.len(), 5);
        assert_eq!(sample.paths[0], "a.rs");
        assert!(sample.is_dirty());
    }

    #[test]
    fn parse_status_empty_is_clean() {
        let sample = parse_status_porcelain("", 5);
        assert!(!sample.is_dirty());
        assert!(sample.paths.is_empty());
    }

    #[test]
    fn concurrent_list_preserves_input_order() {
        let first_base = temp_dir("gw_conc_a");
        let second_base = temp_dir("gw_conc_b");
        let _ = fs::remove_dir_all(&first_base);
        let _ = fs::remove_dir_all(&second_base);
        init_repo(&first_base);
        init_repo(&second_base);
        let clones = vec![
            first_base.canonicalize().unwrap(),
            second_base.canonicalize().unwrap(),
        ];
        let listed = list_worktrees_for_clones(&clones);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].0, clones[0]);
        assert_eq!(listed[1].0, clones[1]);
        assert!(listed[0].1.is_ok());
        assert!(listed[1].1.is_ok());
        let _ = fs::remove_dir_all(&first_base);
        let _ = fs::remove_dir_all(&second_base);
    }

    #[test]
    fn remove_worktree_force_deletes_dirty_linked_tree() {
        let base = temp_dir("gw_remove_wt");
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
        fs::write(linked.join("dirty.txt"), "nope").unwrap();
        remove_worktree(&repo, &linked, true, false).unwrap();
        assert!(!linked.exists());
        let remaining = list_worktrees(&repo).unwrap();
        assert_eq!(remaining.len(), 1);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn remove_worktree_double_force_deletes_locked_linked_tree() {
        let base = temp_dir("gw_remove_locked");
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
        run_git(&repo, &["worktree", "lock", linked.to_str().unwrap()]);
        remove_worktree(&repo, &linked, true, true).unwrap();
        assert!(!linked.exists());
        let remaining = list_worktrees(&repo).unwrap();
        assert_eq!(remaining.len(), 1);
        let _ = fs::remove_dir_all(&base);
    }
}
