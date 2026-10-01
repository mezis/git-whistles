//! Shell installer: syntax, PATH, shims, and the wt alias.

use std::process::Command;

fn repo_root() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

#[test]
fn install_script_passes_bash_syntax_check() {
    let status = Command::new("bash")
        .arg("-n")
        .arg(format!("{}/install.sh", repo_root()))
        .status()
        .expect("bash");
    assert!(status.success());
}

#[test]
fn install_script_shell_setup_is_idempotent() {
    let status = Command::new("bash")
        .arg(format!("{}/tests/install_script.bash", repo_root()))
        .status()
        .expect("bash");
    assert!(status.success());
}
