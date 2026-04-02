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
    resolve_subcommand_and_args_from(&args)
}

fn resolve_subcommand_and_args_from(args: &[String]) -> (Option<&'static str>, Vec<String>) {
    let rest_args = args.get(1..).unwrap_or(&[]).to_vec();
    let Some(argv0) = args.first() else {
        return (None, rest_args);
    };

    // Always inspect argv[0] so bare git-<subcommand> symlink invocations work.
    let name = Path::new(argv0)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Invoked as "git-whistles" or "whistles" -> no inference, use first arg as subcommand
    if name == "git-whistles" || name == "whistles" {
        return (None, rest_args);
    }

    // Invoked as "git-<subcommand>" (e.g. git-chop)
    if let Some(rest) = name.strip_prefix("git-") {
        if let Some(&cmd) = SUBCOMMANDS.iter().find(|&&c| c == rest) {
            return (Some(cmd), rest_args);
        }
    }

    // Invoked as short name (e.g. chop, merge-po)
    if let Some(&cmd) = SUBCOMMANDS.iter().find(|&&c| c == name) {
        return (Some(cmd), rest_args);
    }

    (None, rest_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subcommand_list_is_non_empty() {
        assert!(!SUBCOMMANDS.is_empty());
    }

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn git_whistles_uses_first_argument_as_subcommand() {
        let (subcommand, args) =
            resolve_subcommand_and_args_from(&strings(&["git-whistles", "changes"]));
        assert_eq!(subcommand, None);
        assert_eq!(args, vec!["changes"]);
    }

    #[test]
    fn git_style_symlink_without_extra_args_resolves_subcommand() {
        let (subcommand, args) = resolve_subcommand_and_args_from(&strings(&["git-changes"]));
        assert_eq!(subcommand, Some("changes"));
        assert!(args.is_empty());
    }

    #[test]
    fn git_style_symlink_preserves_remaining_arguments() {
        let (subcommand, args) = resolve_subcommand_and_args_from(&strings(&["git-changes", "-x"]));
        assert_eq!(subcommand, Some("changes"));
        assert_eq!(args, vec!["-x"]);
    }

    #[test]
    fn unknown_executable_name_falls_back_to_normal_parsing() {
        let (subcommand, args) =
            resolve_subcommand_and_args_from(&strings(&["mystery-binary", "changes"]));
        assert_eq!(subcommand, None);
        assert_eq!(args, vec!["changes"]);
    }
}
