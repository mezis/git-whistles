//! Interactive worktree picker across remembered main clones.

pub mod filter;
pub mod registry;
pub mod state;
pub mod teardown;
pub mod ui;

use std::path::{Path, PathBuf};

use clap::Args;

use crate::git;
use crate::worktree;

use registry::{KnownClones, MainClone};

/// `git-whistles worktree-list` takes no subcommand flags (globals `-x`/`-v` still apply).
#[derive(Args)]
#[command(about = "Interactive list of worktrees across remembered clones.")]
pub struct WorktreeListArgs {}

/// Remember the current clone if any, list all remembered worktrees, run the picker.
pub fn run(_args: WorktreeListArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let known = remember_current_clone()?;
    if known.clones().is_empty() {
        return Err("not a git repository; no remembered clones".into());
    }
    let clones: Vec<PathBuf> = known
        .clones()
        .iter()
        .map(|clone| clone.path().to_path_buf())
        .collect();
    let listings = worktree::list_worktrees_for_clones(&clones);
    let cwd = std::env::current_dir().map_err(|err| err.to_string())?;
    let (rows, warnings) = state::rows_from_listings(listings, &cwd);
    for warning in warnings {
        eprintln!("{warning}");
    }
    if rows.is_empty() {
        return Err("no worktrees to list".into());
    }
    if let Some(path) = ui::run(rows, cwd)? {
        println!("{}", cd_eval_line(&path));
    }
    Ok(())
}

fn remember_current_clone() -> Result<KnownClones, String> {
    let path = registry::registry_path()?;
    if git::in_repo() {
        let clone = worktree::main_clone_root()?;
        let mut known = KnownClones::load(&path)?;
        known.upsert_front(MainClone::from_path(clone)?);
        known.save(&path)?;
        Ok(known)
    } else {
        KnownClones::load(&path)
    }
}

/// One eval-able line: `cd -- 'posix-quoted-path'`.
pub fn cd_eval_line(path: &Path) -> String {
    format!(
        "cd -- {}",
        crate::exec::posix_single_quote(&path.to_string_lossy())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_spaces_and_single_quotes() {
        assert_eq!(cd_eval_line(Path::new("/tmp/my wt")), "cd -- '/tmp/my wt'");
        assert_eq!(
            cd_eval_line(Path::new("/tmp/o'reilly")),
            "cd -- '/tmp/o'\\''reilly'"
        );
    }

    #[test]
    fn leading_dash_uses_cd_dash_dash() {
        assert_eq!(cd_eval_line(Path::new("/tmp/-odd")), "cd -- '/tmp/-odd'");
    }
}
