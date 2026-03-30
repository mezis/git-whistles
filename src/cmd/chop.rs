//! git chop: delete local and remote branch(es). If current branch is being chopped, checkout primary (main/master) first.

use clap::Args;
use crate::git;

#[derive(Args)]
#[command(about = "Delete local and remote branch(es); if current branch is chopped, checkout primary (main/master) first.")]
pub struct ChopArgs {
    /// Branch names to delete (local and origin)
    #[arg(required = true)]
    branches: Vec<String>,
}

pub fn run(args: ChopArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("not a git repository".into());
    }
    let current = git::current_branch().ok().unwrap_or_default();
    let primary = git::origin_primary_branch()
        .or_else(|_| Ok::<_, String>("master".to_string()))
        .unwrap();
    let primary_local = primary.strip_prefix("origin/").unwrap_or(&primary).to_string();

    for branch in &args.branches {
        eprintln!("Closing feature branch {}", branch);
        if branch == &current {
            git::run_git_ok(&["checkout", &primary_local])
                .or_else(|_| git::run_git_ok(&["checkout", "main"]))
                .map_err(|e| format!("checkout primary branch: {}", e))?;
        }
        git::run_git_ok(&["branch", "-D", branch]).map_err(|e| e.to_string())?;
        git::run_git_ok(&["push", "origin", &format!(":{}", branch)]).map_err(|e| e.to_string())?;
    }
    Ok(())
}
