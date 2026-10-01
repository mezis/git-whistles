//! Column widths and path prefix-elision for the worktree picker table.

use unicode_width::UnicodeWidthStr;

use super::state::PickerRow;

const ELLIPSIS: &str = "…";
const COLUMN_GAP: usize = 2;
const REPO_HEADER: &str = "repo";
const BRANCH_HEADER: &str = "branch";
const PATH_HEADER: &str = "path";

/// Display widths of the three picker columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableLayout {
    /// Width of the repo column.
    pub repo_width: usize,
    /// Width of the branch column.
    pub branch_width: usize,
    /// Width of the path column (remainder of the inner list width).
    pub path_width: usize,
}

impl TableLayout {
    /// Size columns from `rows` so they fill `inner_width` with two-space gaps.
    ///
    /// Repo and branch use their natural widths (and the header labels) until that
    /// would leave no path column; then branch shrinks first, then repo. The path
    /// column always gets whatever is left.
    pub fn from_rows(rows: &[&PickerRow], inner_width: usize) -> Self {
        let natural_repo = rows
            .iter()
            .map(|row| UnicodeWidthStr::width(row.repo_name()))
            .chain(std::iter::once(UnicodeWidthStr::width(REPO_HEADER)))
            .max()
            .unwrap_or(0);
        let natural_branch = rows
            .iter()
            .map(|row| UnicodeWidthStr::width(row.branch_label()))
            .chain(std::iter::once(UnicodeWidthStr::width(BRANCH_HEADER)))
            .max()
            .unwrap_or(0);
        allocate_widths(inner_width, natural_repo, natural_branch)
    }

    /// Header labels padded to column widths (branch/repo suffix-elided if needed).
    pub fn header_cells(&self) -> [String; 3] {
        [
            pad_end(
                &elide_suffix(REPO_HEADER, self.repo_width).text,
                self.repo_width,
            ),
            pad_end(
                &elide_suffix(BRANCH_HEADER, self.branch_width).text,
                self.branch_width,
            ),
            pad_end(
                &elide_suffix(PATH_HEADER, self.path_width).text,
                self.path_width,
            ),
        ]
    }

    /// Padded cells for one worktree. Path is prefix-elided to `path_width`.
    pub fn row_cells(&self, row: &PickerRow) -> TableRowCells {
        let repo = elide_suffix(row.repo_name(), self.repo_width);
        let branch = elide_suffix(row.branch_label(), self.branch_width);
        let path = elide_prefix(&row.path.display().to_string(), self.path_width);
        TableRowCells {
            repo: pad_end(&repo.text, self.repo_width),
            branch: pad_end(&branch.text, self.branch_width),
            path: pad_end(&path.text, self.path_width),
            repo_fit: repo,
            branch_fit: branch,
            path_fit: path,
        }
    }
}

/// One drawn table row: padded cell text plus elision metadata for highlights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRowCells {
    /// Repo column, padded to [`TableLayout::repo_width`].
    pub repo: String,
    /// Branch column, padded to [`TableLayout::branch_width`].
    pub branch: String,
    /// Path column, prefix-elided and padded to [`TableLayout::path_width`].
    pub path: String,
    /// Elision applied to the repo field (for filter highlight mapping).
    pub repo_fit: FittedText,
    /// Elision applied to the branch field.
    pub branch_fit: FittedText,
    /// Elision applied to the path field.
    pub path_fit: FittedText,
}

/// How `text` relates to the original field after fitting to a column width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FittedText {
    /// Visible cell contents before trailing padding.
    pub text: String,
    /// Which side of the original string was dropped, if any.
    pub origin: FitOrigin,
}

/// Which side of the original string was dropped to fit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FitOrigin {
    Full,
    /// Prefix replaced by `…`; `dropped_chars` original characters are hidden.
    PrefixElided {
        dropped_chars: usize,
    },
    /// Suffix replaced by `…`; `kept_chars` original characters remain.
    SuffixElided {
        kept_chars: usize,
    },
}

impl FittedText {
    /// Map haystack match indices (relative to the original field) onto `text`.
    pub fn remap_matches(&self, field_matches: &[usize]) -> Vec<usize> {
        match self.origin {
            FitOrigin::Full => field_matches
                .iter()
                .copied()
                .filter(|&index| index < self.text.chars().count())
                .collect(),
            FitOrigin::PrefixElided { dropped_chars } => {
                let ellipsis_chars = ELLIPSIS.chars().count();
                field_matches
                    .iter()
                    .filter_map(|&index| {
                        if index < dropped_chars {
                            None
                        } else {
                            Some(ellipsis_chars + (index - dropped_chars))
                        }
                    })
                    .collect()
            }
            FitOrigin::SuffixElided { kept_chars } => field_matches
                .iter()
                .copied()
                .filter(|&index| index < kept_chars)
                .collect(),
        }
    }
}

fn allocate_widths(inner_width: usize, natural_repo: usize, natural_branch: usize) -> TableLayout {
    let gaps = COLUMN_GAP.saturating_mul(2);
    let available = inner_width.saturating_sub(gaps);
    let min_path = usize::from(available > 0);
    let labels_budget = available.saturating_sub(min_path);
    let repo_width = natural_repo.min(labels_budget);
    let branch_width = natural_branch.min(labels_budget.saturating_sub(repo_width));
    let path_width = available.saturating_sub(repo_width + branch_width);
    TableLayout {
        repo_width,
        branch_width,
        path_width,
    }
}

/// Keep the end of `text`; replace a dropped prefix with `…`.
fn elide_prefix(text: &str, max_width: usize) -> FittedText {
    let full_width = UnicodeWidthStr::width(text);
    if full_width <= max_width {
        return FittedText {
            text: text.to_string(),
            origin: FitOrigin::Full,
        };
    }
    let ellipsis_width = UnicodeWidthStr::width(ELLIPSIS);
    if max_width < ellipsis_width {
        return FittedText {
            text: String::new(),
            origin: FitOrigin::PrefixElided {
                dropped_chars: text.chars().count(),
            },
        };
    }
    let keep_width = max_width - ellipsis_width;
    let suffix = longest_suffix_fitting(text, keep_width);
    let dropped_chars = text[..text.len() - suffix.len()].chars().count();
    FittedText {
        text: format!("{ELLIPSIS}{suffix}"),
        origin: FitOrigin::PrefixElided { dropped_chars },
    }
}

/// Keep the start of `text`; replace a dropped suffix with `…`.
fn elide_suffix(text: &str, max_width: usize) -> FittedText {
    let full_width = UnicodeWidthStr::width(text);
    if full_width <= max_width {
        return FittedText {
            text: text.to_string(),
            origin: FitOrigin::Full,
        };
    }
    let ellipsis_width = UnicodeWidthStr::width(ELLIPSIS);
    if max_width < ellipsis_width {
        return FittedText {
            text: String::new(),
            origin: FitOrigin::SuffixElided { kept_chars: 0 },
        };
    }
    let keep_width = max_width - ellipsis_width;
    let prefix = longest_prefix_fitting(text, keep_width);
    let kept_chars = prefix.chars().count();
    FittedText {
        text: format!("{prefix}{ELLIPSIS}"),
        origin: FitOrigin::SuffixElided { kept_chars },
    }
}

fn longest_suffix_fitting(text: &str, keep_width: usize) -> &str {
    for (byte_index, _) in text.char_indices() {
        let suffix = &text[byte_index..];
        if UnicodeWidthStr::width(suffix) <= keep_width {
            return suffix;
        }
    }
    ""
}

fn longest_prefix_fitting(text: &str, keep_width: usize) -> &str {
    let mut end = 0;
    for (byte_index, character) in text.char_indices() {
        let candidate_end = byte_index + character.len_utf8();
        if UnicodeWidthStr::width(&text[..candidate_end]) > keep_width {
            break;
        }
        end = candidate_end;
    }
    &text[..end]
}

fn pad_end(text: &str, width: usize) -> String {
    let used = UnicodeWidthStr::width(text);
    if used >= width {
        return text.to_string();
    }
    let mut padded = text.to_string();
    padded.push_str(&" ".repeat(width - used));
    padded
}

/// Two spaces between padded columns.
pub fn column_gap() -> &'static str {
    "  "
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_row(clone: &str, path: &str, branch: &str) -> PickerRow {
        PickerRow::new(
            PathBuf::from(clone),
            PathBuf::from(path),
            Some(branch.to_string()),
            false,
            true,
        )
    }

    #[test]
    fn elide_prefix_keeps_suffix() {
        let fitted = elide_prefix("/home/mezis/src/git-whistles", 14);
        assert!(fitted.text.starts_with(ELLIPSIS));
        assert!(fitted.text.ends_with("git-whistles"));
        assert_eq!(UnicodeWidthStr::width(fitted.text.as_str()), 14);
        match fitted.origin {
            FitOrigin::PrefixElided { dropped_chars } => assert!(dropped_chars > 0),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn elide_prefix_noop_when_short() {
        let fitted = elide_prefix("/tmp/a", 20);
        assert_eq!(fitted.text, "/tmp/a");
        assert_eq!(fitted.origin, FitOrigin::Full);
    }

    #[test]
    fn layout_gives_path_the_remainder() {
        let rows = [
            sample_row("/repos/app", "/very/long/path/to/worktree", "main"),
            sample_row("/repos/other", "/wt/b", "feature-x"),
        ];
        let refs: Vec<&PickerRow> = rows.iter().collect();
        let layout = TableLayout::from_rows(&refs, 40);
        assert_eq!(layout.repo_width, 5); // "other"
        assert_eq!(layout.branch_width, 9); // "feature-x"
        assert_eq!(layout.path_width, 40 - 5 - 9 - 4);
        let cells = layout.row_cells(&rows[0]);
        assert_eq!(
            UnicodeWidthStr::width(cells.path.as_str()),
            layout.path_width
        );
        assert!(cells.path_fit.text.starts_with(ELLIPSIS));
        assert!(cells.path_fit.text.contains("worktree"));
    }

    #[test]
    fn layout_shrinks_branch_before_repo_when_narrow() {
        let rows = [sample_row("/repos/longreponame", "/p", "very-long-branch")];
        let refs: Vec<&PickerRow> = rows.iter().collect();
        let layout = TableLayout::from_rows(&refs, 20);
        assert_eq!(
            layout.repo_width + layout.branch_width + layout.path_width + 4,
            20
        );
        assert_eq!(layout.repo_width, UnicodeWidthStr::width("longreponame"));
        assert!(layout.branch_width < UnicodeWidthStr::width("very-long-branch"));
        assert_eq!(layout.path_width, 1);
    }

    #[test]
    fn remap_skips_hidden_prefix_chars() {
        let fitted = elide_prefix("abcdefghij", 5);
        assert_eq!(fitted.text, "…ghij");
        assert_eq!(fitted.remap_matches(&[0, 6, 9]), vec![1, 4]);
    }
}
