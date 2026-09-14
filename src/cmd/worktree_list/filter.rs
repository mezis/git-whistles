//! Case-insensitive subsequence matching for the worktree filter.

/// Character indices in `haystack` that complete `needle` in order, if any.
///
/// An empty needle matches (no highlights). Comparison is ASCII case-insensitive.
pub fn subsequence_char_indices(haystack: &str, needle: &str) -> Option<Vec<usize>> {
    if needle.is_empty() {
        return Some(Vec::new());
    }
    let needle_chars: Vec<char> = needle
        .chars()
        .map(|needle_char| needle_char.to_ascii_lowercase())
        .collect();
    let mut needle_at = 0;
    let mut matched = Vec::new();
    for (char_index, hay_char) in haystack.chars().enumerate() {
        if hay_char.to_ascii_lowercase() == needle_chars[needle_at] {
            matched.push(char_index);
            needle_at += 1;
            if needle_at == needle_chars.len() {
                return Some(matched);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_letters_in_order() {
        let indices = subsequence_char_indices("worktree", "wt").unwrap();
        assert_eq!(indices, vec![0, 4]);
    }

    #[test]
    fn rejects_reversed_order() {
        assert_eq!(subsequence_char_indices("worktree", "tw"), None);
    }

    #[test]
    fn is_case_insensitive() {
        let indices = subsequence_char_indices("Repo/Feature", "rf").unwrap();
        assert_eq!(indices, vec![0, 5]);
    }

    #[test]
    fn empty_needle_matches_without_highlights() {
        assert_eq!(subsequence_char_indices("abc", ""), Some(Vec::new()));
    }
}
