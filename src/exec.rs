//! External command execution with optional bash-style tracing (`set -x`).

use std::io;
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

static ECHO_COMMANDS: AtomicBool = AtomicBool::new(false);

/// Enable or disable printing each external command before it runs (stderr, `+ prog arg ...`).
pub fn set_echo_commands(on: bool) {
    ECHO_COMMANDS.store(on, Ordering::Relaxed);
}

pub fn echo_commands_enabled() -> bool {
    ECHO_COMMANDS.load(Ordering::Relaxed)
}

/// Quote a single argument for human-readable shell-like tracing.
fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '@' | '+' | '=' | ',')
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

pub fn git_output(args: &[&str]) -> io::Result<Output> {
    log_command("git", args);
    Command::new("git").args(args).output()
}

pub fn command_output(program: &str, args: &[&str]) -> io::Result<Output> {
    log_command(program, args);
    Command::new(program).args(args).output()
}

pub fn command_spawn(program: &str, args: &[&str], stdin: Stdio, stdout: Stdio) -> io::Result<Child> {
    log_command(program, args);
    Command::new(program)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
}
