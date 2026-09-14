//! Detect and run Compose / `bin/teardown` before `git worktree remove`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::exec;

/// Which destroy-time cleanup commands apply to a worktree.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TeardownPlan {
    /// A Compose file is present in the worktree.
    pub compose: bool,
    /// Path to `bin/teardown` when that file exists.
    pub teardown: Option<PathBuf>,
}

impl TeardownPlan {
    /// Whether any cleanup command will run.
    pub fn is_empty(&self) -> bool {
        !self.compose && self.teardown.is_none()
    }
}

/// Compose filenames Git-whistles treats as a Compose project (same set as mezis-teardown).
const COMPOSE_FILES: &[&str] = &[
    "compose.yaml",
    "compose.yml",
    "docker-compose.yml",
    "docker-compose.yaml",
];

/// Inspect the worktree directory for Compose files and `bin/teardown`.
pub fn detect(worktree: &Path) -> TeardownPlan {
    let compose = COMPOSE_FILES
        .iter()
        .any(|name| worktree.join(name).is_file());
    let script = worktree.join("bin").join("teardown");
    let teardown = if script.is_file() { Some(script) } else { None };
    TeardownPlan { compose, teardown }
}

/// Runs a program in a directory (used so tests can record calls without Docker).
pub trait CommandRunner {
    /// Run `program` with `args` using `cwd` as the working directory.
    fn run_in(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<(), String>;
}

/// Child stdout and stderr both go to our stderr so eval never captures them.
pub struct StderrStreamingRunner;

impl CommandRunner for StderrStreamingRunner {
    fn run_in(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<(), String> {
        exec::log_command(program, args);
        let stdout_to_stderr = stderr_as_stdio()?;
        let status = Command::new(program)
            .current_dir(cwd)
            .args(args)
            .stdout(stdout_to_stderr)
            .stderr(Stdio::inherit())
            .status()
            .map_err(|err| format!("{program} failed to start: {err}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "{program} failed (exit status {})",
                status.code().unwrap_or(-1)
            ))
        }
    }
}

/// Run Compose (if any) then `bin/teardown` (if any). Fail-fast; caller must not git-remove on error.
pub fn run_plan<Runner: CommandRunner>(
    worktree: &Path,
    plan: &TeardownPlan,
    runner: &Runner,
) -> Result<(), String> {
    if plan.compose {
        runner.run_in(worktree, "docker", &["compose", "down"])?;
    }
    if let Some(script) = &plan.teardown {
        let program = script
            .to_str()
            .ok_or_else(|| "teardown path is not valid UTF-8".to_string())?;
        if is_executable(script) {
            runner.run_in(worktree, program, &[])?;
        } else {
            runner.run_in(worktree, "bash", &[program])?;
        }
    }
    Ok(())
}

fn stderr_as_stdio() -> Result<Stdio, String> {
    use std::io;
    use std::os::fd::AsFd;
    io::stderr()
        .as_fd()
        .try_clone_to_owned()
        .map(Stdio::from)
        .map_err(|err| format!("clone stderr: {err}"))
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        fs_mode_executable(path)
            .map(|mode| mode & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[cfg(unix)]
fn fs_mode_executable(path: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .ok()
        .map(|meta| meta.permissions().mode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct RecordingRunner {
        calls: Mutex<Vec<(PathBuf, String, Vec<String>)>>,
        fail_program: Option<String>,
    }

    impl CommandRunner for RecordingRunner {
        fn run_in(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<(), String> {
            self.calls.lock().unwrap().push((
                cwd.to_path_buf(),
                program.to_string(),
                args.iter().map(|arg| (*arg).to_string()).collect(),
            ));
            if self.fail_program.as_deref() == Some(program) {
                return Err("injected failure".to_string());
            }
            Ok(())
        }
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), unique));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detect_compose_only() {
        let dir = temp_dir("gw_td_compose");
        fs::write(dir.join("compose.yaml"), "services: {}\n").unwrap();
        let plan = detect(&dir);
        assert!(plan.compose);
        assert!(plan.teardown.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_teardown_only() {
        let dir = temp_dir("gw_td_script");
        fs::create_dir_all(dir.join("bin")).unwrap();
        fs::write(dir.join("bin").join("teardown"), "#!/bin/sh\n").unwrap();
        let plan = detect(&dir);
        assert!(!plan.compose);
        assert!(plan.teardown.is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_both_and_neither() {
        let empty = temp_dir("gw_td_none");
        assert!(detect(&empty).is_empty());
        let both = temp_dir("gw_td_both");
        fs::write(both.join("docker-compose.yml"), "").unwrap();
        fs::create_dir_all(both.join("bin")).unwrap();
        fs::write(both.join("bin").join("teardown"), "").unwrap();
        let plan = detect(&both);
        assert!(plan.compose);
        assert!(plan.teardown.is_some());
        let _ = fs::remove_dir_all(&empty);
        let _ = fs::remove_dir_all(&both);
    }

    #[test]
    fn run_plan_compose_then_script_and_stops_before_later_steps_on_failure() {
        let dir = temp_dir("gw_td_run");
        fs::create_dir_all(dir.join("bin")).unwrap();
        let script = dir.join("bin").join("teardown");
        fs::write(&script, "").unwrap();
        let plan = TeardownPlan {
            compose: true,
            teardown: Some(script.clone()),
        };
        let runner = RecordingRunner {
            calls: Mutex::new(Vec::new()),
            fail_program: Some("docker".to_string()),
        };
        assert!(run_plan(&dir, &plan, &runner).is_err());
        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "docker");
        let _ = fs::remove_dir_all(&dir);
    }
}
