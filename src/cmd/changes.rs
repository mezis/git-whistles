//! git changes: show commits on current branch not in origin primary (like "git log --oneline HEAD ^origin/main").

use crate::exec;
use crate::git;
use clap::Args;

#[derive(Args)]
#[command(about = "Show commits on current branch not in origin primary (main/master).")]
pub struct ChangesArgs {
    /// Extra arguments forwarded to the underlying `git log` invocation
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    git_log_args: Vec<String>,
}

pub fn run(args: ChangesArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("not a git repository".into());
    }
    let primary = git::origin_primary_branch()?;
    let exclude_primary = format!("^{}", primary);
    let mut git_args = vec![
        "log".to_string(),
        "--oneline".to_string(),
        "HEAD".to_string(),
        exclude_primary,
    ];
    git_args.extend(forwarded_git_log_args(args.git_log_args));
    let git_args: Vec<&str> = git_args.iter().map(String::as_str).collect();
    if exec::stream_output_enabled() {
        exec::git_inherit_all(&git_args)?;
    } else {
        let out = git::run_git_stdout(&git_args)?;
        println!("{}", out);
    }
    Ok(())
}

/// Clap consumes `--`, so reinsert it before the first pathspec (an extra arg that does not start with `-`).
fn forwarded_git_log_args(extra: Vec<String>) -> Vec<String> {
    let mut forwarded = Vec::with_capacity(extra.len() + 1);
    let mut inserted_pathspec_separator = false;
    for arg in extra {
        if !inserted_pathspec_separator && !arg.starts_with('-') {
            forwarded.push("--".to_string());
            inserted_pathspec_separator = true;
        }
        forwarded.push(arg);
    }
    forwarded
}
