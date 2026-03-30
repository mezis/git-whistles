//! git-whistles shim / unshim: add or remove `git-<subcommand>` symlinks to the main binary.

use clap::Args;
use std::env;
use std::fs;
use std::path::Path;

use crate::dispatch::SUBCOMMANDS;

const DEFAULT_DIR: &str = "/usr/local/bin";

#[derive(Args)]
#[command(about = "Add git-<subcommand> symlinks (not shim/unshim) to the main binary in a directory on PATH.")]
pub struct ShimArgs {
    /// Directory to install symlinks into
    #[arg(short, long, default_value = DEFAULT_DIR)]
    pub dir: String,
}

#[derive(Args)]
#[command(about = "Remove symlinks previously created by shim.")]
pub struct UnshimArgs {
    /// Directory to remove symlinks from
    #[arg(short, long, default_value = DEFAULT_DIR)]
    pub dir: String,
}

/// Path to the current executable (the main binary).
fn self_exe() -> Result<std::path::PathBuf, String> {
    env::current_exe().map_err(|e| e.to_string())
}

pub fn run_shim(args: ShimArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target = self_exe()?;
    let dir = Path::new(&args.dir);
    if !dir.exists() {
        return Err(format!("directory does not exist: {}", args.dir).into());
    }
    for name in shim_names() {
        let link = dir.join(name);
        if link.exists() {
            // Don't overwrite unless it's already our symlink
            if fs::read_link(&link).ok().as_ref() != Some(&target) {
                eprintln!("Skipping {} (exists, not our symlink)", link.display());
                continue;
            }
        }
        #[cfg(unix)]
        {
            let _ = fs::remove_file(&link);
            std::os::unix::fs::symlink(&target, &link).map_err(|e| format!("symlink {}: {}", link.display(), e))?;
            eprintln!("Linked {} -> {}", link.display(), target.display());
        }
        #[cfg(not(unix))]
        {
            return Err("shim only supported on Unix (symlinks)".into());
        }
    }
    Ok(())
}

pub fn run_unshim(args: UnshimArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target = self_exe()?;
    let dir = Path::new(&args.dir);
    for name in shim_names() {
        let link = dir.join(name);
        if let Ok(dest) = fs::read_link(&link) {
            if dest == target {
                fs::remove_file(&link).map_err(|e| format!("remove {}: {}", link.display(), e))?;
                eprintln!("Removed {}", link.display());
            }
        }
    }
    Ok(())
}

/// Subcommands that get a `git-...` shim. `shim` / `unshim` are only run via `git-whistles`.
const NOT_SHIMMED: &[&str] = &["shim", "unshim"];

/// Symlink names: `git-<subcommand>` only (no bare short names; no git-shim / git-unshim).
fn shim_names() -> Vec<String> {
    SUBCOMMANDS
        .iter()
        .copied()
        .filter(|cmd| !NOT_SHIMMED.contains(cmd))
        .map(|cmd| format!("git-{}", cmd))
        .collect()
}
