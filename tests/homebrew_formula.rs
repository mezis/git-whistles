//! Homebrew formula writer used by the release workflow.

use std::process::Command;

fn repo_root() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

#[test]
fn formula_writer_records_release_checksums() {
    let status = Command::new("bash")
        .arg(format!("{}/tests/homebrew_formula.bash", repo_root()))
        .status()
        .expect("bash");
    assert!(status.success());
}
