//! git stash-and-checkout: stash (including untracked), checkout target, then pop matching WIP stash.

use clap::Args;
use crate::git;

#[derive(Args)]
#[command(about = "Stash (including untracked), checkout target branch, then pop matching WIP stash if any.")]
pub struct StashAndCheckoutArgs {
    /// Target branch to checkout
    pub branch: String,
}

pub fn run(args: StashAndCheckoutArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("not a git repository".into());
    }
    git::run_git_ok(&["stash", "push", "--include-untracked"]).map_err(|e| e.to_string())?;
    git::run_git_ok(&["checkout", &args.branch]).map_err(|e| e.to_string())?;

    let list = git::run_git_stdout(&["stash", "list"])?;
    let target_prefix = format!("WIP on {}:", args.branch);
    let stash_ref = list
        .lines()
        .find(|l| l.contains(&target_prefix))
        .and_then(|l| l.split(':').next().map(str::trim));

    if let Some(stash_ref) = stash_ref {
        eprintln!("Popping {}", stash_ref);
        git::run_git_ok(&["stash", "pop", stash_ref]).map_err(|e| e.to_string())?;
    }
    Ok(())
}
