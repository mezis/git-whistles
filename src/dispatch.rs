//! Resolve subcommand from argv[0] (symlink name) or from first argument.

use std::env;
use std::path::Path;

/// Subcommands that can be invoked by symlink name (e.g. `git-chop`, or bare `chop` if linked manually).
pub const SUBCOMMANDS: &[&str] = &[
    "chop",
    "ff-all-branches",
    "list-branches",
    "stash-and-checkout",
    "staging",
    "merge-po",
    "changes",
    "shim",
    "unshim",
];

/// Returns (subcommand, rest_args). If invoked as git-whistles, subcommand is None and
/// rest_args is full args (so parser will use first arg as subcommand).
pub fn resolve_subcommand_and_args() -> (Option<&'static str>, Vec<String>) {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return (None, args.get(1..).unwrap_or(&[]).to_vec());
    }

    let argv0 = args[0].clone();
    let name = Path::new(&argv0)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Invoked as "git-whistles" or "whistles" -> no inference, use first arg as subcommand
    if name == "git-whistles" || name == "whistles" {
        return (None, args[1..].to_vec());
    }

    // Invoked as "git-<subcommand>" (e.g. git-chop)
    if let Some(rest) = name.strip_prefix("git-") {
        if let Some(&cmd) = SUBCOMMANDS.iter().find(|&&c| c == rest) {
            return (Some(cmd), args[1..].to_vec());
        }
    }

    // Invoked as short name (e.g. chop, merge-po)
    if let Some(&cmd) = SUBCOMMANDS.iter().find(|&&c| c == name) {
        return (Some(cmd), args[1..].to_vec());
    }

    (None, args[1..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subcommand_list_is_non_empty() {
        assert!(!SUBCOMMANDS.is_empty());
    }
}
