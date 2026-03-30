//! git staging: sync a feature branch with main, then merge it into staging and push.
//!
//! Sequence: stash-and-checkout branch → ff-all-branches → merge main → push →
//! stash -u → checkout staging → fetch → reset --hard origin/staging →
//! merge branch → push → stash-and-checkout back to branch.

use clap::Args;
use crate::git;
use crate::cmd::{ff_all_branches, stash_and_checkout};

#[derive(Args)]
#[command(about = "Sync branch with main, merge into staging, push, then return to branch.")]
pub struct StagingArgs {
    /// Branch to stage (default: current branch)
    #[arg(index = 1)]
    pub branch: Option<String>,
}

pub fn run(args: StagingArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("not a git repository".into());
    }

    let branch = args
        .branch
        .unwrap_or_else(|| git::current_branch().unwrap_or_else(|_| "HEAD".to_string()));

    // Primary branch local name (main or master) for merging into the feature branch.
    let primary = git::origin_primary_branch()
        .or_else(|_| Ok::<_, String>("origin/master".to_string()))
        .unwrap();
    let main_local = primary
        .strip_prefix("origin/")
        .unwrap_or(primary.as_str())
        .to_string();

    eprintln!("Staging branch {}", branch);

    // 1. Switch to the branch (stash, checkout, pop WIP if any)
    stash_and_checkout::run(stash_and_checkout::StashAndCheckoutArgs {
        branch: branch.clone(),
    })?;

    // 2. Fast-forward all tracking branches
    ff_all_branches::run(ff_all_branches::FfAllBranchesArgs {
        fetch: false,
        dry_run: false,
        remote: "origin".to_string(),
        verbose: false,
        quiet: false,
    })?;

    // 3. Merge main into the branch and push
    git::run_git_ok(&["merge", "--no-edit", &main_local])?;
    git::run_git_ok(&["push"])?;

    // 4. Stash (including untracked), checkout staging
    git::run_git_ok(&["stash", "push", "--include-untracked"])?;
    git::run_git_ok(&["checkout", "staging"])?;

    // 5. Update staging from origin
    git::run_git_ok(&["fetch"])?;
    git::run_git_ok(&["reset", "--hard", "origin/staging"])?;

    // 6. Merge branch into staging and push
    git::run_git_ok(&["merge", "--no-edit", &branch])?;
    git::run_git_ok(&["push"])?;

    // 7. Return to the feature branch (stash-and-checkout pops WIP if any)
    stash_and_checkout::run(stash_and_checkout::StashAndCheckoutArgs { branch })?;

    Ok(())
}
