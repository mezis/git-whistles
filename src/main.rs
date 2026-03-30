//! git-whistles: helpers for classic Git workflows.
//!
//! Dispatches by subcommand (when run as `git-whistles <cmd>`) or by executable
//! name when invoked via symlink (e.g. `git-chop`, `chop`).

use std::process::ExitCode;

mod cli;
mod dispatch;
mod exec;
mod git;
mod cmd;

use cli::Cli;
use dispatch::resolve_subcommand_and_args;

fn main() -> ExitCode {
    let (subcommand, args) = resolve_subcommand_and_args();
    match Cli::run(subcommand, args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}
