//! CLI parsing and dispatch to subcommands.

use clap::{CommandFactory, Parser, Subcommand};

use crate::cmd;
use crate::exec;

#[derive(Parser)]
#[command(name = "git-whistles")]
#[command(about = "Helpers for classic Git workflows", long_about = None)]
struct CliApp {
    /// Echo external commands to stderr before running (like `set -x`)
    #[arg(short = 'x', long = "echo-commands", global = true)]
    echo_commands: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Delete local and remote branch(es); if current branch is chopped, checkout primary (main/master) first
    Chop(cmd::chop::ChopArgs),
    /// Fast-forward all local tracking branches to their remote counterpart where possible
    #[command(name = "ff-all-branches")]
    FfAllBranches(cmd::ff_all_branches::FfAllBranchesArgs),
    /// List local or remote branches and their distance to an integration branch
    #[command(name = "list-branches")]
    ListBranches(cmd::list_branches::ListBranchesArgs),
    /// Stash (including untracked), checkout target branch, then pop matching WIP stash if any
    #[command(name = "stash-and-checkout")]
    StashAndCheckout(cmd::stash_and_checkout::StashAndCheckoutArgs),
    /// Sync branch with main, merge into staging, push, then return to branch
    Staging(cmd::staging::StagingArgs),
    /// Three-way merge driver for gettext PO files (uses msguniq, msgcat, msgmerge, msggrep)
    #[command(name = "merge-po")]
    MergePo(cmd::merge_po::MergePoArgs),
    /// Show commits on current branch not in origin primary (main/master)
    Changes(cmd::changes::ChangesArgs),
    /// Add symlinks (e.g. git-chop, chop) to the main binary in a directory on PATH
    Shim(cmd::shim::ShimArgs),
    /// Remove symlinks previously created by shim
    Unshim(cmd::shim::UnshimArgs),
}

pub struct Cli;

impl Cli {
    /// Run with optional pre-resolved subcommand name and remaining args.
    /// If subcommand is None, args should include the subcommand as first element.
    pub fn run(
        subcommand: Option<&str>,
        args: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let argv = build_argv(subcommand, args);
        let cli = CliApp::try_parse_from(argv)?;
        exec::set_echo_commands(cli.echo_commands);
        match cli.command {
            Some(Commands::Chop(a)) => cmd::chop::run(a),
            Some(Commands::FfAllBranches(a)) => cmd::ff_all_branches::run(a),
            Some(Commands::ListBranches(a)) => cmd::list_branches::run(a),
            Some(Commands::StashAndCheckout(a)) => cmd::stash_and_checkout::run(a),
            Some(Commands::Staging(a)) => cmd::staging::run(a),
            Some(Commands::MergePo(a)) => cmd::merge_po::run(a),
            Some(Commands::Changes(a)) => cmd::changes::run(a),
            Some(Commands::Shim(a)) => cmd::shim::run_shim(a),
            Some(Commands::Unshim(a)) => cmd::shim::run_unshim(a),
            None => {
                // Surface a real help message when argv resolution leaves us without a subcommand.
                let mut command = CliApp::command();
                command.print_help()?;
                eprintln!();
                Err("missing subcommand".into())
            }
        }
    }
}

/// Build argv for clap: ["git-whistles", subcommand?, ...args]
fn build_argv(subcommand: Option<&str>, args: Vec<String>) -> Vec<String> {
    let mut argv = vec!["git-whistles".to_string()];
    if let Some(cmd) = subcommand {
        argv.push(cmd.to_string());
    }
    argv.extend(args);
    argv
}
