//! git changes: show commits on current branch not in origin primary (like "git log --oneline HEAD ^origin/main").

use crate::exec;
use crate::git;
use clap::Args;

#[derive(Args)]
#[command(about = "Show commits on current branch not in origin primary (main/master).")]
pub struct ChangesArgs {}

pub fn run(_args: ChangesArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("not a git repository".into());
    }
    let primary = git::origin_primary_branch()?;
    if exec::stream_output_enabled() {
        exec::git_inherit_all(&["log", "--oneline", "HEAD", &format!("^{}", primary)])?;
    } else {
        let out = git::run_git_stdout(&["log", "--oneline", "HEAD", &format!("^{}", primary)])?;
        println!("{}", out);
    }
    Ok(())
}
