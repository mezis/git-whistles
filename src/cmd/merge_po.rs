//! Three-way merge driver for gettext PO files. Shells out to msguniq, msgcat, msgmerge, msggrep.

use clap::Args;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;

use crate::exec;

#[derive(Args)]
#[command(about = "Three-way merge driver for gettext PO files (uses msguniq, msgcat, msgmerge, msggrep).")]
pub struct MergePoArgs {
    /// Base (ancestor) PO file
    pub base: String,
    /// Our (current) PO file — overwritten with merge result
    pub local: String,
    /// Theirs (incoming) PO file
    pub remote: String,
}

pub fn run(args: MergePoArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base = Path::new(&args.base);
    let local = Path::new(&args.local);
    let remote = Path::new(&args.remote);

    let temp_dir = tempfile::tempdir()?;
    let t = temp_dir.path();
    let (base_t, local_t, remote_t) = (
        t.join("base.po"),
        t.join("local.po"),
        t.join("remote.po"),
    );
    let (header, local_changes, remote_changes, unchanged, conflicts, local_only, remote_only) = (
        t.join("header.po"),
        t.join("local-changes.po"),
        t.join("remote-changes.po"),
        t.join("unchanged.po"),
        t.join("conflicts.po"),
        t.join("local-only.po"),
        t.join("remote-only.po"),
    );
    let (merge1, merge2, merge3) = (
        t.join("merge1.po"),
        t.join("merge2.po"),
        t.join("merge3.po"),
    );

    // Optional: show file path (like original)
    if local.exists() {
        if let (Ok(hash_out), Ok(tree_out)) = (
            exec::git_output(&["hash-object", local.to_str().unwrap_or("")]),
            exec::git_output(&["ls-tree", "-r", "HEAD"]),
        ) {
            if hash_out.status.success() && tree_out.status.success() {
                let h = String::from_utf8_lossy(&hash_out.stdout).trim().to_string();
                let tree = String::from_utf8_lossy(&tree_out.stdout);
                for line in tree.lines() {
                    if line.contains(&h) && line.len() > 53 {
                        eprintln!("Using custom PO merge driver ({}; {:?})", line[53..].trim(), t);
                        break;
                    }
                }
            }
        }
    }

    // Header: first paragraph of local
    let local_content = fs::read_to_string(local).map_err(|e| e.to_string())?;
    let header_end = local_content.find("\n\n").unwrap_or(local_content.len());
    fs::write(&header, &local_content[..header_end]).map_err(|e| e.to_string())?;

    run_ok(
        "msguniq",
        &[
            "--force-po",
            "-o",
            base_t.to_str().unwrap(),
            "--unique",
            base.to_str().unwrap(),
        ],
    )?;
    run_ok(
        "msguniq",
        &[
            "--force-po",
            "-o",
            local_t.to_str().unwrap(),
            "--unique",
            local.to_str().unwrap(),
        ],
    )?;
    run_ok(
        "msguniq",
        &[
            "--force-po",
            "-o",
            remote_t.to_str().unwrap(),
            "--unique",
            remote.to_str().unwrap(),
        ],
    )?;

    extract_changes(&local_t, &base_t, &local_changes)?;
    extract_changes(&remote_t, &base_t, &remote_changes)?;

    // unchanged: msgcat base local remote | msggrep -v conflict
    pipe_through_msggrep_inverse(
        &[
            "msgcat",
            "-o",
            "-",
            "--force-po",
            base_t.to_str().unwrap(),
            local_t.to_str().unwrap(),
            remote_t.to_str().unwrap(),
        ],
        &unchanged,
    )?;

    // conflicts: msgcat remote-changes local-changes | msggrep conflict
    pipe_through_msggrep(
        &[
            "msgcat",
            "-o",
            "-",
            "--force-po",
            remote_changes.to_str().unwrap(),
            local_changes.to_str().unwrap(),
        ],
        &conflicts,
    )?;

    run_ok(
        "msgcat",
        &[
            "--force-po",
            "-o",
            local_only.to_str().unwrap(),
            "--unique",
            local_changes.to_str().unwrap(),
            conflicts.to_str().unwrap(),
        ],
    )?;
    run_ok(
        "msgcat",
        &[
            "--force-po",
            "-o",
            remote_only.to_str().unwrap(),
            "--unique",
            remote_changes.to_str().unwrap(),
            conflicts.to_str().unwrap(),
        ],
    )?;
    run_ok(
        "msgcat",
        &[
            "--force-po",
            "-o",
            merge1.to_str().unwrap(),
            unchanged.to_str().unwrap(),
            conflicts.to_str().unwrap(),
            local_only.to_str().unwrap(),
            remote_only.to_str().unwrap(),
        ],
    )?;

    // template from local + remote, then msgmerge merge1 template -> merge2
    let template_p = t.join("template.po");
    run_ok(
        "msgcat",
        &[
            "--force-po",
            "-o",
            template_p.to_str().unwrap(),
            local_t.to_str().unwrap(),
            remote_t.to_str().unwrap(),
        ],
    )?;
    run_ok(
        "msgmerge",
        &[
            "--force-po",
            "--quiet",
            "--no-fuzzy-matching",
            "-o",
            merge2.to_str().unwrap(),
            merge1.to_str().unwrap(),
            template_p.to_str().unwrap(),
        ],
    )?;

    run_ok(
        "msgcat",
        &[
            "-o",
            merge3.to_str().unwrap(),
            "--use-first",
            header.to_str().unwrap(),
            merge2.to_str().unwrap(),
        ],
    )?;

    let result = fs::read_to_string(&merge3).map_err(|e| e.to_string())?;
    // Write result first (like shell script), then exit with error if conflicts remain
    fs::write(local, &result).map_err(|e| e.to_string())?;
    if result.contains("#-#-#-#-#") {
        eprintln!("Conflict(s) detected");
        eprintln!("   between {:?} and {:?}", local_t, remote_t);
        return Err("merge left conflict markers in output".into());
    }

    Ok(())
}

fn run_ok(program: &str, args: &[&str]) -> Result<(), String> {
    let out = exec::command_output(program, args).map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} failed: {}",
            program,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// extract_changes A B -> out: msgcat A B | msggrep conflict | msgmerge -o - A - | strip #~
fn extract_changes(a: &Path, base: &Path, out: &Path) -> Result<(), String> {
    let cat = exec::command_output(
        "msgcat",
        &[
            "-o",
            "-",
            "--force-po",
            a.to_str().unwrap(),
            base.to_str().unwrap(),
        ],
    )
    .map_err(|e| e.to_string())?;
    if !cat.status.success() {
        return Err(String::from_utf8_lossy(&cat.stderr).to_string());
    }
    let mut grep = exec::command_spawn(
        "msggrep",
        &["--msgstr", "-F", "-e", "#-#-#-#-#", "-"],
        Stdio::piped(),
        Stdio::piped(),
    )
    .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = grep.stdin.take() {
        stdin.write_all(&cat.stdout).map_err(|e| e.to_string())?;
    }
    let grep_out = grep.wait_with_output().map_err(|e| e.to_string())?;
    if !grep_out.status.success() {
        return Err("msggrep failed".to_string());
    }
    let merge_in = String::from_utf8_lossy(&grep_out.stdout);
    let stripped: String = merge_in
        .lines()
        .filter(|l| !l.starts_with("#~"))
        .collect::<Vec<_>>()
        .join("\n");
    let merge_in_file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    fs::write(merge_in_file.path(), &stripped).map_err(|e| e.to_string())?;
    let merge_out = exec::command_output(
        "msgmerge",
        &[
            "--force-po",
            "--quiet",
            "--no-fuzzy-matching",
            "-o",
            out.to_str().unwrap(),
            a.to_str().unwrap(),
            merge_in_file.path().to_str().unwrap(),
        ],
    )
    .map_err(|e| e.to_string())?;
    if !merge_out.status.success() {
        return Err(String::from_utf8_lossy(&merge_out.stderr).to_string());
    }
    Ok(())
}

fn pipe_through_msggrep_inverse(args: &[&str], out: &Path) -> Result<(), String> {
    let mut a = args.to_vec();
    let idx = a.iter().position(|&x| x == "-o").unwrap_or(0);
    a[idx + 1] = "-";
    let cat = exec::command_output(a[0], &a[1..]).map_err(|e| e.to_string())?;
    if !cat.status.success() {
        return Err(String::from_utf8_lossy(&cat.stderr).to_string());
    }
    let mut grep = exec::command_spawn(
        "msggrep",
        &["--msgstr", "-F", "-e", "#-#-#-#-#", "-v", "-"],
        Stdio::piped(),
        Stdio::piped(),
    )
    .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = grep.stdin.take() {
        stdin.write_all(&cat.stdout).map_err(|e| e.to_string())?;
    }
    let grep_out = grep.wait_with_output().map_err(|e| e.to_string())?;
    fs::write(out, grep_out.stdout).map_err(|e| e.to_string())?;
    Ok(())
}

fn pipe_through_msggrep(msgcat_args: &[&str], out: &Path) -> Result<(), String> {
    let cat = exec::command_output(msgcat_args[0], &msgcat_args[1..]).map_err(|e| e.to_string())?;
    if !cat.status.success() {
        return Err(String::from_utf8_lossy(&cat.stderr).to_string());
    }
    let mut grep = exec::command_spawn(
        "msggrep",
        &["--msgstr", "-F", "-e", "#-#-#-#-#", "-"],
        Stdio::piped(),
        Stdio::piped(),
    )
    .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = grep.stdin.take() {
        stdin.write_all(&cat.stdout).map_err(|e| e.to_string())?;
    }
    let grep_out = grep.wait_with_output().map_err(|e| e.to_string())?;
    fs::write(out, grep_out.stdout).map_err(|e| e.to_string())?;
    Ok(())
}
