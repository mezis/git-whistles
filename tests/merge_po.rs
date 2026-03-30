//! Integration test for merge-po: run with fixtures and assert expected msgstr values.
//! Gettext (msguniq, msgcat, msgmerge, msggrep) is mandatory for the test suite.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Minimal PO parser: extract msgid -> msgstr map (first msgstr per msgid).
fn load_po(data: &str) -> std::collections::HashMap<String, String> {
    let mut h = std::collections::HashMap::new();
    let mut state = 0u8; // 0=none, 1=id, 2=str
    let mut id = String::new();
    let mut str_ = String::new();
    for line in data.lines().chain(std::iter::once("")) {
        let line = line.trim_end();
        match state {
            0 => {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("msgid \"") {
                    id = rest.strip_suffix('"').unwrap_or(rest).replace("\\n", "\n").replace("\\\"", "\"");
                    state = 1;
                }
            }
            1 => {
                if let Some(rest) = line.strip_prefix("msgstr \"") {
                    str_ = rest.strip_suffix('"').unwrap_or(rest).replace("\\n", "\n").replace("\\\"", "\"");
                    state = 2;
                } else if line.starts_with('"') {
                    if let Some(inner) = line.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                        id.push_str(&inner.replace("\\n", "\n").replace("\\\"", "\""));
                    }
                }
            }
            2 => {
                if line.is_empty() || line.starts_with('#') {
                    if !id.is_empty() {
                        h.insert(std::mem::take(&mut id), std::mem::take(&mut str_));
                    }
                    state = if line.is_empty() { 0 } else { 2 };
                } else if line.starts_with('"') {
                    if let Some(inner) = line.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                        str_.push_str(&inner.replace("\\n", "\n").replace("\\\"", "\""));
                    }
                }
            }
            _ => {}
        }
    }
    h
}

fn merge_po_exe() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("git-whistles")
}

/// Asserts gettext tools are on PATH; call at start of merge-po tests.
fn require_gettext() {
    for cmd in ["msguniq", "msgcat", "msgmerge", "msggrep"] {
        let ok = Command::new(cmd).arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
        assert!(ok, "gettext is mandatory: {} not found or failed. Install gettext (e.g. apt-get install gettext, brew install gettext)", cmd);
    }
}

#[test]
fn merge_po_spec_fixtures() {
    require_gettext();
    let exe = merge_po_exe();
    assert!(exe.exists(), "binary not found at {:?}, run cargo build first", exe);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("merge_po");
    let base = fixtures.join("base.po");
    let local = fixtures.join("local.po");
    let remote = fixtures.join("remote.po");
    assert!(base.exists(), "tests/fixtures/merge_po fixtures not found");
    let out_dir = std::env::temp_dir().join("merge_po_test");
    let _ = fs::create_dir_all(&out_dir);
    let local_copy = out_dir.join("local.po");
    fs::copy(&local, &local_copy).unwrap();
    let status = Command::new(&exe)
        .args(["merge-po", base.to_str().unwrap(), local_copy.to_str().unwrap(), remote.to_str().unwrap()])
        .status()
        .expect("run merge-po");
    let output = fs::read_to_string(&local_copy).unwrap();
    let got = load_po(&output);
    let expected: std::collections::HashMap<_, _> = [
        ("This little piggie is unchanged", vec!["1"]),
        ("This little piggie is removed from remote, unchanged on local", vec!["2"]),
        ("This little piggie is removed from remote, changed on local", vec!["3.local"]),
        ("This little piggie is removed from local, unchanged on remote", vec!["4"]),
        ("This little piggie is removed from local, changed on remote", vec!["5.remote"]),
        ("This little piggie is changed on local, unchanged on remote", vec!["6.local", "CONFLICT6"]),
        // gettext msgcat may or may not emit conflict when 2-of-3 agree; accept 7.remote, 7, or conflict block
        ("This little piggie is changed on remote, unchanged on local", vec!["7.remote", "7", "CONFLICT"]),
        ("This little piggie is added on remote, not on local", vec!["9.remote"]),
        ("This little piggie is added on local, not on remote", vec!["10.local"]),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    for (msgid, allowed) in &expected {
        let g = got.get(msgid.as_str()).map(|s| s.as_str()).unwrap_or("");
        let ok = allowed.iter().any(|a| {
            if *a == "CONFLICT" {
                g.contains("7.remote") && g.contains("#-#-#")
            } else if *a == "CONFLICT6" {
                g.contains("6.local") && g.contains("#-#-#")
            } else {
                *a == g || (g.contains(a) && !g.contains("#-#-#"))
            }
        });
        assert!(ok, "msgid {:?}: got {:?}, expected one of {:?}", msgid, g, allowed);
    }
    // Conflict cases: value should contain both and marker
    let conflict1 = got.get("This little piggie is changed on remote and local").map(|s| s.as_str()).unwrap_or("");
    assert!(conflict1.contains("8.local") && conflict1.contains("8.remote") && conflict1.contains("#-#-#"), "conflict 8: {:?}", conflict1);
    let conflict2 = got.get("This little piggie is added on local and remote, with different values").map(|s| s.as_str()).unwrap_or("");
    assert!(conflict2.contains("11.local") && conflict2.contains("11.remote") && conflict2.contains("#-#-#"), "conflict 11: {:?}", conflict2);
    // Merge may exit 1 when conflicts remain; assertions above are the real check.
    let _ = status;
}
