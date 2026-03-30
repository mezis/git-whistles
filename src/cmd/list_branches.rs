//! git list-branches: list branch status and age against an integration branch.

use clap::Args;
use crate::git;

#[derive(Args)]
#[command(about = "List local or remote branches and their distance to an integration branch.")]
pub struct ListBranchesArgs {
    #[arg(short, long)]
    pub local: bool,
    #[arg(short, long)]
    pub remote: bool,
    #[arg(short, long, default_value = "origin/master")]
    pub integration: String,
    #[arg(short = 'p', long)]
    pub porcelain: bool,
}

pub fn run(args: ListBranchesArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("Not in a git repository !".into());
    }
    let where_ = if args.remote {
        "remote"
    } else {
        "local"
    };
    let against = &args.integration;

    let branches = if args.remote {
        list_refs_remote()
    } else {
        list_refs_local()
    }?;

    if !args.porcelain {
        println!("Listing {} branches against {}", where_, against);
        println!("{:<70} {:>6} {:>6} {:>18} {}", "BRANCH NAME", "AHEAD", "BEHIND", "OLDEST UNPULLED", "AUTHOR");
    }

    for branch in branches {
        let (ahead, behind, behind_by, author) = branch_stats(&branch, against)?;
        if args.porcelain {
            println!("{},{},{},{},{}", branch, ahead, behind, behind_by, author);
        } else {
            let line = format!("{:<70} {:>6} {:>6} {:>18} {}", branch, ahead, behind, behind_by, author);
            let colored = color_by_duration(&line);
            println!("{}", colored);
        }
    }
    Ok(())
}

fn list_refs_local() -> Result<Vec<String>, String> {
    let out = git::run_git_stdout(&["show-ref", "--heads"])?;
    Ok(out
        .lines()
        .filter_map(|line| {
            show_ref_name(line)?
                .strip_prefix("refs/heads/")
                .map(String::from)
        })
        .collect())
}

fn list_refs_remote() -> Result<Vec<String>, String> {
    let out = git::run_git_stdout(&["show-ref"])?;
    Ok(out
        .lines()
        .filter_map(|line| {
            let ref_name = show_ref_name(line)?;
            if ref_name.starts_with("refs/remotes/origin/") && ref_name != "refs/remotes/origin/HEAD" {
                ref_name.strip_prefix("refs/remotes/").map(String::from)
            } else {
                None
            }
        })
        .collect())
}

fn show_ref_name(line: &str) -> Option<&str> {
    // `git show-ref` prints `<oid> <refname>`; the branch/ref name is the second field.
    line.split_whitespace().nth(1)
}

fn branch_stats(branch: &str, against: &str) -> Result<(usize, usize, String, String), String> {
    let ahead_out = git::run_git_stdout(&["rev-list", "--count", branch, &format!("^{}", against)])?;
    let ahead: usize = ahead_out.trim().parse().unwrap_or(0);
    let behind_out = git::run_git_stdout(&["rev-list", "--count", against, &format!("^{}", branch)])?;
    let behind: usize = behind_out.trim().parse().unwrap_or(0);

    let behind_by = if behind > 0 {
        let first = git::run_git_stdout(&["rev-list", "--reverse", against, &format!("^{}", branch)])?;
        let first_rev = first.lines().next().unwrap_or("").trim();
        if first_rev.is_empty() {
            String::new()
        } else {
            git::run_git_stdout(&["log", "-1", "--format=%ar", first_rev]).unwrap_or_default()
        }
    } else {
        String::new()
    };

    let latest = git::run_git_stdout(&["rev-list", "-n", "1", branch])?;
    let latest_rev = latest.lines().next().unwrap_or("").trim();
    let author = if latest_rev.is_empty() {
        String::new()
    } else {
        git::run_git_stdout(&["log", "-1", "--format=%an", latest_rev]).unwrap_or_default()
    };

    Ok((ahead, behind, behind_by, author))
}

/// Simple color by age string (matches shell script semantics).
fn color_by_duration(line: &str) -> String {
    let (code, reset) = if line.contains("+0") {
        (37, 0)
    } else if line.contains("minute") || line.contains("hour") {
        (32, 0)
    } else if line.contains("days") {
        (33, 0)
    } else if line.contains("week") {
        (33, 0)
    } else if line.contains("month") {
        (31, 0)
    } else if line.contains("year") {
        (31, 0)
    } else {
        (0, 0)
    };
    if code == 0 {
        line.to_string()
    } else {
        format!("\x1b[{}m{}\x1b[{}m", code, line, reset)
    }
}
