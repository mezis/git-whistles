//! git ff-all-branches: fast-forward all local tracking branches to their remote counterparts.

use clap::{ArgAction, Args};
use std::collections::HashMap;
use crate::git;

#[derive(Args)]
#[command(about = "Fast-forward all local tracking branches to their remote counterpart where possible.")]
pub struct FfAllBranchesArgs {
    #[arg(long = "no-fetch", default_value_t = true, action = ArgAction::SetFalse)]
    pub fetch: bool,
    #[arg(short = 'p', long)]
    pub dry_run: bool,
    #[arg(short, long, default_value = "origin")]
    pub remote: String,
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub quiet: bool,
}

fn short_sha(sha: &str) -> &str {
    if sha.len() >= 8 {
        &sha[..8]
    } else {
        sha
    }
}

pub fn run(args: FfAllBranchesArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !git::in_repo() {
        return Err("not a git repository".into());
    }
    if args.fetch {
        git::run_git_ok(&["fetch"])?;
    }

    let current = git::current_branch()?;
    let (local, remote_refs) = load_refs(&args.remote)?;

    for (branch_name, old_head) in &local {
        let Some(new_head) = remote_refs.get(branch_name) else {
            continue;
        };
        if old_head.as_str() == new_head.as_str() {
            continue;
        }

        if branch_name == &current {
            let status = git::run_git(&["status", "--porcelain"]).map_err(|e| e.to_string())?;
            let stdout = String::from_utf8_lossy(&status.stdout);
            if !stdout.trim().is_empty() {
                eprintln!("not merging current branch as it has local changes");
                continue;
            }
            if !args.dry_run {
                let flag = if args.quiet { "-q" } else { "" };
                let mut a: Vec<&str> = vec!["merge", "--ff-only"];
                if !flag.is_empty() {
                    a.push(flag);
                }
                a.push(new_head.as_str());
                git::run_git_ok(&a).map_err(|e| e.to_string())?;
            }
        } else {
            let merge_base = git::run_git_stdout(&["merge-base", old_head.as_str(), new_head.as_str()])?;
            if merge_base != old_head.as_str() {
                eprintln!("cannot fast-forward {}", branch_name);
                continue;
            }
            if !args.dry_run {
                let ref_name = format!("refs/heads/{}", branch_name);
                git::run_git_ok(&["update-ref", &ref_name, new_head.as_str(), old_head.as_str()])
                    .map_err(|e| e.to_string())?;
            }
        }
        if args.verbose {
            eprintln!("{}: {} -> {}", branch_name, short_sha(old_head), short_sha(new_head.as_str()));
        }
    }
    Ok(())
}

fn load_refs(remote: &str) -> Result<(HashMap<String, String>, HashMap<String, String>), String> {
    let out = git::run_git_stdout(&["show-ref"])?;
    let mut local = HashMap::new();
    let mut remote_refs = HashMap::new();
    let prefix_heads = "refs/heads/";
    let prefix_remotes = format!("refs/remotes/{}", remote);
    for line in out.lines() {
        let line = line.trim();
        let mut it = line.splitn(2, ' ');
        let sha = it.next().ok_or("invalid show-ref output")?;
        let ref_name = it.next().ok_or("invalid show-ref output")?;
        if ref_name.starts_with(prefix_heads) {
            local.insert(ref_name[prefix_heads.len()..].to_string(), sha.to_string());
        } else if ref_name.starts_with(&prefix_remotes) {
            let branch = ref_name[prefix_remotes.len()..].trim_start_matches('/');
            if branch != "HEAD" {
                remote_refs.insert(branch.to_string(), sha.to_string());
            }
        }
    }
    Ok((local, remote_refs))
}
