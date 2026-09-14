//! Persistent list of remembered main clones.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// A canonical path to a Git main clone (not a linked worktree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainClone {
    path: PathBuf,
}

impl MainClone {
    /// Canonicalize `path` into a remembered clone.
    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        let canonical = path
            .canonicalize()
            .map_err(|err| format!("canonicalize {}: {}", path.display(), err))?;
        Ok(Self { path: canonical })
    }

    /// Absolute clone path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Ordered remembered clones (most recently used first).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnownClones {
    clones: Vec<MainClone>,
}

impl KnownClones {
    /// Parse the registry file. Missing file → empty. Skip blank lines and missing paths.
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self { clones: Vec::new() });
        }
        let text =
            fs::read_to_string(path).map_err(|err| format!("read {}: {}", path.display(), err))?;
        let mut clones = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let candidate = PathBuf::from(trimmed);
            if !candidate.exists() {
                continue;
            }
            let Ok(canonical) = candidate.canonicalize() else {
                continue;
            };
            if !seen.insert(canonical.clone()) {
                continue;
            }
            clones.push(MainClone { path: canonical });
        }
        Ok(Self { clones })
    }

    /// Write remembered clones (one canonical path per line). Creates parent dirs.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create {}: {}", parent.display(), err))?;
        }
        let mut body = String::new();
        for clone in &self.clones {
            body.push_str(&clone.path.to_string_lossy());
            body.push('\n');
        }
        fs::write(path, body).map_err(|err| format!("write {}: {}", path.display(), err))?;
        Ok(())
    }

    /// Move `clone` to the front (MRU). Drops any earlier copy of the same path.
    pub fn upsert_front(&mut self, clone: MainClone) {
        self.clones.retain(|existing| existing.path != clone.path);
        self.clones.insert(0, clone);
    }

    /// Remembered clones in MRU order.
    pub fn clones(&self) -> &[MainClone] {
        &self.clones
    }
}

/// Production registry file: `GIT_WHISTLES_DATA_HOME`, else XDG, else `~/.local/share/git-whistles`.
pub fn registry_path() -> Result<PathBuf, String> {
    if let Ok(data_home) = env::var("GIT_WHISTLES_DATA_HOME") {
        if !data_home.is_empty() {
            return Ok(PathBuf::from(data_home).join("known-clones"));
        }
    }
    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return Ok(PathBuf::from(xdg).join("git-whistles").join("known-clones"));
        }
    }
    let home = env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("git-whistles")
        .join("known-clones"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{}.txt", std::process::id(), unique))
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
    fn load_missing_file_is_empty() {
        let path = temp_file("gw_reg_missing");
        let loaded = KnownClones::load(&path).unwrap();
        assert!(loaded.clones().is_empty());
    }

    #[test]
    fn load_skips_blank_lines_and_missing_paths() {
        let existing = temp_dir("gw_reg_exists");
        let path = temp_file("gw_reg_skip");
        let body = format!(
            "\n{}\n\n/this/does/not/exist-{}\n",
            existing.display(),
            unique_nanos()
        );
        fs::write(&path, body).unwrap();
        let loaded = KnownClones::load(&path).unwrap();
        assert_eq!(loaded.clones().len(), 1);
        assert_eq!(
            loaded.clones()[0].path(),
            existing.canonicalize().unwrap().as_path()
        );
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&existing);
    }

    fn unique_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    #[test]
    fn upsert_front_is_mru_and_round_trips() {
        let first = temp_dir("gw_reg_a");
        let second = temp_dir("gw_reg_b");
        let mut known = KnownClones::default();
        known.upsert_front(MainClone::from_path(first.clone()).unwrap());
        known.upsert_front(MainClone::from_path(second.clone()).unwrap());
        known.upsert_front(MainClone::from_path(first.clone()).unwrap());
        assert_eq!(known.clones().len(), 2);
        assert_eq!(
            known.clones()[0].path(),
            first.canonicalize().unwrap().as_path()
        );
        assert_eq!(
            known.clones()[1].path(),
            second.canonicalize().unwrap().as_path()
        );
        let path = temp_file("gw_reg_save");
        known.save(&path).unwrap();
        let reloaded = KnownClones::load(&path).unwrap();
        assert_eq!(known, reloaded);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }

    #[test]
    fn registry_path_prefers_git_whistles_data_home() {
        let previous = env::var("GIT_WHISTLES_DATA_HOME").ok();
        env::set_var("GIT_WHISTLES_DATA_HOME", "/tmp/gw-data-test");
        let path = registry_path().unwrap();
        match previous {
            Some(value) => env::set_var("GIT_WHISTLES_DATA_HOME", value),
            None => env::remove_var("GIT_WHISTLES_DATA_HOME"),
        }
        assert_eq!(path, PathBuf::from("/tmp/gw-data-test/known-clones"));
    }
}
