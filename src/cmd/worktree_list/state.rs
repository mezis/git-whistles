//! Picker list state and key handling (no terminal IO).

use std::path::{Path, PathBuf};

use super::filter::subsequence_char_indices;
use super::teardown::TeardownPlan;
use crate::worktree::DirtySample;

/// One worktree row in the picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRow {
    /// Main clone this worktree belongs to.
    pub clone: PathBuf,
    /// Worktree path.
    pub path: PathBuf,
    /// Short branch name, or `None` if detached.
    pub branch: Option<String>,
    /// Git reports the worktree as locked.
    pub locked: bool,
    /// This checkout is the clone's primary worktree (not a linked one).
    pub is_main: bool,
    display_line: String,
}

impl PickerRow {
    /// Build a row and cache its display line for filtering and drawing.
    pub fn new(
        clone: PathBuf,
        path: PathBuf,
        branch: Option<String>,
        locked: bool,
        is_main: bool,
    ) -> Self {
        let repo = clone
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let branch_label = branch.as_deref().unwrap_or("detached");
        let display_line = format!("{} {} {}", repo, branch_label, path.display());
        Self {
            clone,
            path,
            branch,
            locked,
            is_main,
            display_line,
        }
    }

    /// `{clone basename} {branch|detached} {path}` used for display and filtering.
    pub fn display_line(&self) -> &str {
        &self.display_line
    }
}

/// Keys the picker understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Enter,
    Esc,
    CtrlC,
    CtrlD,
    Backspace,
    /// ASCII letter or digit. Matching case-folds; the caller does not lowercase.
    FilterChar(char),
    Other,
}

/// What the UI should do after a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Select(PathBuf),
    /// Gather dirty/teardown info and call [`PickerState::begin_confirm`].
    OpenDestroyConfirm,
    /// User confirmed destroy of this row.
    Destroy {
        clone: PathBuf,
        path: PathBuf,
        force: bool,
        plan: TeardownPlan,
    },
}

/// Overlay shown after Ctrl-d.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmState {
    pub dirty: DirtySample,
    /// `git status` failed; shown instead of a clean/dirty summary.
    pub status_error: Option<String>,
    pub locked: bool,
    pub plan: TeardownPlan,
    pub clone: PathBuf,
    pub path: PathBuf,
}

/// Filterable list + optional destroy confirmation.
pub struct PickerState {
    rows: Vec<PickerRow>,
    filter: String,
    /// Indices into `rows` that match the current filter.
    visible_indices: Vec<usize>,
    /// Subsequence match char indices, parallel to `visible_indices`.
    match_indices: Vec<Vec<usize>>,
    /// Index into the current visible list.
    selected: usize,
    confirm: Option<ConfirmState>,
    refuse_message: Option<String>,
    pub(crate) cwd: PathBuf,
}

impl PickerState {
    /// Build a picker. `cwd` is used to refuse destroying the current worktree.
    pub fn new(rows: Vec<PickerRow>, cwd: PathBuf) -> Self {
        let mut state = Self {
            rows,
            filter: String::new(),
            visible_indices: Vec::new(),
            match_indices: Vec::new(),
            selected: 0,
            confirm: None,
            refuse_message: None,
            cwd,
        };
        state.rebuild_visible();
        state
    }

    /// Keep other clones' rows; splice `new_rows` at this clone's first position.
    pub fn replace_clone_rows(&mut self, clone: &Path, new_rows: Vec<PickerRow>) {
        let mut replacement = Some(new_rows);
        let mut next = Vec::new();
        for row in std::mem::take(&mut self.rows) {
            if row.clone.as_path() == clone {
                if let Some(rows) = replacement.take() {
                    next.extend(rows);
                }
            } else {
                next.push(row);
            }
        }
        if let Some(rows) = replacement {
            next.extend(rows);
        }
        self.rows = next;
        self.confirm = None;
        self.rebuild_visible();
    }

    /// Current filter needle.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Selected index into [`Self::visible`].
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Destroy overlay, if any.
    pub fn confirm(&self) -> Option<&ConfirmState> {
        self.confirm.as_ref()
    }

    /// Brief refuse text (main checkout / cwd).
    pub fn refuse_message(&self) -> Option<&str> {
        self.refuse_message.as_deref()
    }

    /// Currently selected row among visible rows.
    pub fn selected_row(&self) -> Option<&PickerRow> {
        let row_index = *self.visible_indices.get(self.selected)?;
        self.rows.get(row_index)
    }

    /// Visible rows: those whose display line subsequence-matches the filter.
    pub fn visible(&self) -> Vec<&PickerRow> {
        self.visible_indices
            .iter()
            .map(|&index| &self.rows[index])
            .collect()
    }

    /// Cached subsequence highlights for visible row `visible_index`.
    pub fn match_indices(&self, visible_index: usize) -> &[usize] {
        self.match_indices
            .get(visible_index)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn rebuild_visible(&mut self) {
        self.visible_indices.clear();
        self.match_indices.clear();
        for (index, row) in self.rows.iter().enumerate() {
            if let Some(matched) = subsequence_char_indices(row.display_line(), &self.filter) {
                self.visible_indices.push(index);
                self.match_indices.push(matched);
            }
        }
        self.clamp_selected();
    }

    /// Apply a key. Domain-only; the UI fetches git status when this returns [`Action::OpenDestroyConfirm`].
    pub fn handle_key(&mut self, key: Key) -> Action {
        if self.confirm.is_some() {
            return self.handle_confirm_key(key);
        }
        self.refuse_message = None;
        match key {
            Key::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            Key::Down => {
                let last = self.visible_indices.len().saturating_sub(1);
                if self.selected < last {
                    self.selected += 1;
                }
                Action::None
            }
            Key::Enter => {
                if let Some(row) = self.selected_row() {
                    Action::Select(row.path.clone())
                } else {
                    Action::None
                }
            }
            Key::Esc | Key::CtrlC => Action::Quit,
            Key::CtrlD => self.ctrl_d(),
            Key::Backspace => {
                self.filter.pop();
                self.selected = 0;
                self.rebuild_visible();
                Action::None
            }
            Key::FilterChar(filter_char) => {
                if filter_char.is_ascii_alphanumeric() {
                    self.filter.push(filter_char);
                    self.selected = 0;
                    self.rebuild_visible();
                }
                Action::None
            }
            Key::Other => Action::None,
        }
    }

    fn handle_confirm_key(&mut self, key: Key) -> Action {
        match key {
            Key::Esc | Key::CtrlC => {
                self.confirm = None;
                Action::None
            }
            Key::Enter => {
                let Some(confirm) = self.confirm.take() else {
                    return Action::None;
                };
                let force = confirm.dirty.is_dirty() || confirm.locked;
                Action::Destroy {
                    clone: confirm.clone,
                    path: confirm.path,
                    force,
                    plan: confirm.plan,
                }
            }
            _ => Action::None,
        }
    }

    fn ctrl_d(&mut self) -> Action {
        let Some(row) = self.selected_row() else {
            return Action::None;
        };
        if row.is_main {
            self.refuse_message = Some("cannot destroy the main checkout".to_string());
            return Action::None;
        }
        if cwd_is_inside(&self.cwd, &row.path) {
            self.refuse_message = Some("cannot destroy the worktree the shell is in".to_string());
            return Action::None;
        }
        Action::OpenDestroyConfirm
    }

    /// Fill the confirm overlay after the UI loaded dirty status and teardown detection.
    pub fn begin_confirm(
        &mut self,
        dirty: DirtySample,
        plan: TeardownPlan,
        status_error: Option<String>,
    ) {
        let Some(row) = self.selected_row() else {
            return;
        };
        self.confirm = Some(ConfirmState {
            dirty,
            status_error,
            locked: row.locked,
            plan,
            clone: row.clone.clone(),
            path: row.path.clone(),
        });
    }

    fn clamp_selected(&mut self) {
        let visible_len = self.visible_indices.len();
        if visible_len == 0 {
            self.selected = 0;
        } else if self.selected >= visible_len {
            self.selected = visible_len - 1;
        }
    }
}

/// `cwd` is the worktree itself or a directory inside it.
pub fn cwd_is_inside(cwd: &Path, worktree: &Path) -> bool {
    let cwd_canonical = cwd.canonicalize().ok();
    cwd_is_inside_with_canonical_cwd(cwd, cwd_canonical.as_deref(), worktree)
}

fn cwd_is_inside_with_canonical_cwd(
    cwd: &Path,
    cwd_canonical: Option<&Path>,
    worktree: &Path,
) -> bool {
    if let (Some(cwd), Ok(root)) = (cwd_canonical, worktree.canonicalize()) {
        return cwd == root.as_path() || cwd.starts_with(&root);
    }
    cwd == worktree || cwd.starts_with(worktree)
}

/// Build picker rows from concurrent list results; skip bare; float `cwd` to the front.
pub fn rows_from_listings(
    listings: Vec<(PathBuf, Result<Vec<crate::worktree::Worktree>, String>)>,
    cwd: &Path,
) -> (Vec<PickerRow>, Vec<String>) {
    let cwd_canonical = cwd.canonicalize().ok();
    let mut warnings = Vec::new();
    let mut rows = Vec::new();
    for (clone, result) in listings {
        match result {
            Ok(trees) => {
                let clone_canonical = clone.canonicalize().ok();
                for tree in trees {
                    if tree.is_bare {
                        continue;
                    }
                    let is_main = same_path_with_canonical_right(
                        &tree.path,
                        &clone,
                        clone_canonical.as_deref(),
                    );
                    rows.push(PickerRow::new(
                        clone.clone(),
                        tree.path,
                        tree.branch,
                        tree.locked,
                        is_main,
                    ));
                }
            }
            Err(message) => {
                warnings.push(format!("{}: {message}", clone.display()));
            }
        }
    }
    if let Some(index) = rows
        .iter()
        .position(|row| cwd_is_inside_with_canonical_cwd(cwd, cwd_canonical.as_deref(), &row.path))
    {
        let current = rows.remove(index);
        rows.insert(0, current);
    }
    (rows, warnings)
}

fn same_path_with_canonical_right(
    left: &Path,
    right: &Path,
    right_canonical: Option<&Path>,
) -> bool {
    match (left.canonicalize().ok(), right_canonical) {
        (Some(left), Some(right)) => left.as_path() == right,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(clone: &str, path: &str, branch: &str, is_main: bool) -> PickerRow {
        PickerRow::new(
            PathBuf::from(clone),
            PathBuf::from(path),
            Some(branch.to_string()),
            false,
            is_main,
        )
    }

    fn picker() -> PickerState {
        PickerState::new(
            vec![
                row("/repos/app", "/wt/a", "main", true),
                row("/repos/app", "/wt/b", "feature", false),
            ],
            PathBuf::from("/elsewhere"),
        )
    }

    #[test]
    fn arrows_clamp() {
        let mut state = picker();
        state.handle_key(Key::Up);
        assert_eq!(state.selected(), 0);
        state.handle_key(Key::Down);
        assert_eq!(state.selected(), 1);
        state.handle_key(Key::Down);
        assert_eq!(state.selected(), 1);
    }

    #[test]
    fn filter_selects_first_match() {
        let mut state = picker();
        state.handle_key(Key::Down);
        state.handle_key(Key::FilterChar('f'));
        assert_eq!(state.selected(), 0);
        assert_eq!(state.visible().len(), 1);
        assert_eq!(state.visible()[0].branch.as_deref(), Some("feature"));
    }

    #[test]
    fn enter_selects_path() {
        let mut state = picker();
        match state.handle_key(Key::Enter) {
            Action::Select(path) => assert_eq!(path, PathBuf::from("/wt/a")),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn esc_quits() {
        let mut state = picker();
        assert!(matches!(state.handle_key(Key::Esc), Action::Quit));
    }

    #[test]
    fn ctrl_d_refuses_main() {
        let mut state = picker();
        assert!(matches!(state.handle_key(Key::CtrlD), Action::None));
        assert!(state.refuse_message().unwrap().contains("main checkout"));
    }

    #[test]
    fn ctrl_d_refuses_cwd() {
        let mut state = PickerState::new(
            vec![row("/repos/app", "/wt/b", "feature", false)],
            PathBuf::from("/wt/b"),
        );
        assert!(matches!(state.handle_key(Key::CtrlD), Action::None));
        assert!(state.refuse_message().unwrap().contains("shell is in"));
    }

    #[test]
    fn confirm_enter_emits_destroy_esc_cancels() {
        let mut state = picker();
        state.handle_key(Key::Down);
        assert!(matches!(
            state.handle_key(Key::CtrlD),
            Action::OpenDestroyConfirm
        ));
        state.begin_confirm(
            DirtySample {
                paths: vec!["x".into()],
                total: 1,
            },
            TeardownPlan::default(),
            None,
        );
        assert!(state.confirm().unwrap().status_error.is_none());
        state.handle_key(Key::Esc);
        assert!(state.confirm().is_none());
        state.handle_key(Key::CtrlD);
        state.begin_confirm(
            DirtySample {
                paths: vec![],
                total: 0,
            },
            TeardownPlan::default(),
            Some("git status failed: boom".to_string()),
        );
        assert_eq!(
            state.confirm().unwrap().status_error.as_deref(),
            Some("git status failed: boom")
        );
        match state.handle_key(Key::Enter) {
            Action::Destroy { path, force, .. } => {
                assert_eq!(path, PathBuf::from("/wt/b"));
                assert!(!force);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn replace_clone_rows_keeps_other_clones() {
        let mut state = PickerState::new(
            vec![
                row("/repos/app", "/wt/a", "main", true),
                row("/repos/app", "/wt/b", "feature", false),
                row("/repos/other", "/wt/c", "main", true),
            ],
            PathBuf::from("/elsewhere"),
        );
        state.replace_clone_rows(
            Path::new("/repos/app"),
            vec![row("/repos/app", "/wt/a", "main", true)],
        );
        let paths: Vec<_> = state
            .visible()
            .iter()
            .map(|row| row.path.as_path())
            .collect();
        assert_eq!(paths, vec![Path::new("/wt/a"), Path::new("/wt/c")]);
    }
}
