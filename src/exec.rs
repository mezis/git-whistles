//! External command execution with optional bash-style tracing (`set -x`) and verbose streaming.

use std::io;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

static ECHO_COMMANDS: AtomicBool = AtomicBool::new(false);
static STREAM_OUTPUT: AtomicBool = AtomicBool::new(false);

/// Enable or disable printing each external command before it runs (stderr, `+ prog arg ...`).
pub fn set_echo_commands(on: bool) {
    ECHO_COMMANDS.store(on, Ordering::Relaxed);
}

pub fn echo_commands_enabled() -> bool {
    ECHO_COMMANDS.load(Ordering::Relaxed)
}

/// When true, side-effect subprocesses inherit stdout/stderr; stdout-capture runs inherit stderr only.
pub fn set_stream_output(on: bool) {
    STREAM_OUTPUT.store(on, Ordering::Relaxed);
}

pub fn stream_output_enabled() -> bool {
    STREAM_OUTPUT.load(Ordering::Relaxed)
}

/// Quote a single argument for human-readable shell-like tracing.
fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '-' | '.' | '/' | ':' | '@' | '+' | '=' | ',')
    }) {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

/// Log `program` and `args` when echo mode is on.
pub fn log_command(program: &str, args: &[&str]) {
    if !echo_commands_enabled() {
        return;
    }
    let mut line = String::from("+ ");
    line.push_str(program);
    for a in args {
        line.push(' ');
        line.push_str(&shell_quote(a));
    }
    eprintln!("{}", line);
}

/// Full capture of stdout and stderr (e.g. porcelain, binary-safe reads).
pub fn git_output_captured(args: &[&str]) -> io::Result<Output> {
    log_command("git", args);
    Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

/// Capture stdout; stderr is inherited when streaming so diagnostics appear live.
pub fn git_output_stdout(args: &[&str]) -> io::Result<Output> {
    log_command("git", args);
    let mut cmd = Command::new("git");
    cmd.args(args).stdout(Stdio::piped());
    if stream_output_enabled() {
        cmd.stderr(Stdio::inherit());
    } else {
        cmd.stderr(Stdio::piped());
    }
    cmd.output()
}

/// Run git for side effects only (success / failure). Inherits stdio when streaming.
pub fn git_side_effect(args: &[&str]) -> Result<(), String> {
    log_command("git", args);
    if stream_output_enabled() {
        let status = Command::new("git")
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| e.to_string())?;
        git_status_to_result_streaming(args, status)
    } else {
        let out = Command::new("git")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
        }
    }
}

fn git_status_to_result_streaming(args: &[&str], status: ExitStatus) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {} failed (exit status {})",
            args.join(" "),
            status.code().unwrap_or(-1)
        ))
    }
}

/// Run `git` with inherited stdout and stderr (e.g. `git log` where output is the product).
pub fn git_inherit_all(args: &[&str]) -> Result<(), String> {
    log_command("git", args);
    let status = Command::new("git")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| e.to_string())?;
    git_status_to_result_streaming(args, status)
}

/// Capture program stdout; stderr inherited when streaming.
pub fn command_output(program: &str, args: &[&str]) -> io::Result<Output> {
    log_command(program, args);
    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped());
    if stream_output_enabled() {
        cmd.stderr(Stdio::inherit());
    } else {
        cmd.stderr(Stdio::piped());
    }
    cmd.output()
}

/// Run a command for side effects only (writes to `-o` paths, etc.).
pub fn command_side_effect(program: &str, args: &[&str]) -> Result<(), String> {
    log_command(program, args);
    if stream_output_enabled() {
        let status = Command::new(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "{} failed (exit status {})",
                program,
                status.code().unwrap_or(-1)
            ))
        }
    } else {
        let out = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            Err(format!(
                "{} failed: {}",
                program,
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }
}

pub fn command_spawn(
    program: &str,
    args: &[&str],
    stdin: Stdio,
    stdout: Stdio,
) -> io::Result<Child> {
    log_command(program, args);
    Command::new(program)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
}
