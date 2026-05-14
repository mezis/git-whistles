//! git staging: sync a feature branch with main, then merge it into staging and push.
//!
//! Sequence: stash-and-checkout branch → ff-all-branches → merge main → push →
//! stash -u → checkout staging → fetch → reset --hard origin/staging →
//! merge branch → push → stash-and-checkout back to branch.
//!
//! If the branch is checked out in another linked worktree, sync happens there
//! (fetch, merge main, push) instead of the first three steps; the command then
//! continues in the current worktree on `staging` and returns to the starting branch.

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

    let other_wt = git::other_worktree_path_for_branch(&branch)?;
    let starting_branch = if other_wt.is_some() {
        Some(git::current_branch()?)
    } else {
        None
    };

    if let Some(ref wt) = other_wt {
        eprintln!(
            "Branch {} is checked out in another worktree; syncing at {}",
            branch,
            wt.display()
        );
        git::run_git_ok(&["fetch"])?;
        git::run_git_ok_in(wt, &["fetch"])?;
        git::run_git_ok_in(wt, &["merge", "--no-edit", &main_local])?;
        git::run_git_ok_in(wt, &["push"])?;
    } else {
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
    }

    // 4. Stash (including untracked), checkout staging
    git::run_git_ok(&["stash", "push", "--include-untracked"])?;
    git::run_git_ok(&["checkout", "staging"])?;

    // 5. Update staging from origin
    git::run_git_ok(&["fetch"])?;
    git::run_git_ok(&["reset", "--hard", "origin/staging"])?;

    // 6. Merge branch into staging and push
    git::run_git_ok(&["merge", "--no-edit", &branch])?;
    git::run_git_ok(&["push"])?;

    // 7. Return to the feature branch or the branch we started on
    if let Some(sb) = starting_branch {
        stash_and_checkout::stash_checkout_pop_wip(&sb)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    } else {
        stash_and_checkout::run(stash_and_checkout::StashAndCheckoutArgs { branch })?;
    }

    Ok(())
}
