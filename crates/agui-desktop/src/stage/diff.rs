//! Diff Computation and Parsing
//!
//! Provides utilities for computing and parsing unified diffs.

use super::types::{DiffHunk, DiffLine, DiffLineType};

/// Compute a unified diff between two strings
pub fn compute_unified_diff(original: &str, modified: &str) -> String {
    // Simple line-by-line diff implementation
    // For production, consider using a proper diff library like `similar`
    let original_lines: Vec<&str> = original.lines().collect();
    let modified_lines: Vec<&str> = modified.lines().collect();

    let mut result = String::new();

    // Use a simple LCS-based diff algorithm
    let lcs = compute_lcs(&original_lines, &modified_lines);
    let hunks = generate_hunks(&original_lines, &modified_lines, &lcs);

    for hunk in hunks {
        result.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start + 1,
            hunk.old_count,
            hunk.new_start + 1,
            hunk.new_count
        ));

        for line in &hunk.lines {
            let prefix = match line.line_type {
                DiffLineType::Context => " ",
                DiffLineType::Addition => "+",
                DiffLineType::Deletion => "-",
                DiffLineType::Header => "@@",
            };
            result.push_str(&format!("{}{}\n", prefix, line.content));
        }
    }

    result
}

/// Parse a unified diff string into hunks
pub fn parse_hunks(diff: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line = 0usize;
    let mut new_line = 0usize;

    for line in diff.lines() {
        if line.starts_with("@@") {
            // Save previous hunk if any
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }

            // Parse hunk header: @@ -start,count +start,count @@
            if let Some((old_start, old_count, new_start, new_count)) = parse_hunk_header(line) {
                old_line = old_start;
                new_line = new_start;
                current_hunk = Some(DiffHunk {
                    old_start,
                    old_count,
                    new_start,
                    new_count,
                    lines: Vec::new(),
                });
            }
        } else if let Some(ref mut hunk) = current_hunk {
            let (line_type, content) = if let Some(content) = line.strip_prefix('+') {
                (DiffLineType::Addition, content)
            } else if let Some(content) = line.strip_prefix('-') {
                (DiffLineType::Deletion, content)
            } else if let Some(content) = line.strip_prefix(' ') {
                (DiffLineType::Context, content)
            } else {
                (DiffLineType::Context, line)
            };

            let (old_ln, new_ln) = match line_type {
                DiffLineType::Addition => {
                    let ln = new_line;
                    new_line += 1;
                    (None, Some(ln))
                }
                DiffLineType::Deletion => {
                    let ln = old_line;
                    old_line += 1;
                    (Some(ln), None)
                }
                DiffLineType::Context => {
                    let (o, n) = (old_line, new_line);
                    old_line += 1;
                    new_line += 1;
                    (Some(o), Some(n))
                }
                DiffLineType::Header => (None, None),
            };

            hunk.lines.push(DiffLine {
                content: content.to_string(),
                line_type,
                old_line: old_ln,
                new_line: new_ln,
            });
        }
    }

    // Don't forget the last hunk
    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    hunks
}

/// Parse a hunk header line
fn parse_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    // Format: @@ -old_start,old_count +new_start,new_count @@
    let line = line.trim_start_matches('@').trim();
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let old_part = parts[0].trim_start_matches('-');
    let new_part = parts[1].trim_start_matches('+');

    let (old_start, old_count) = parse_range(old_part)?;
    let (new_start, new_count) = parse_range(new_part)?;

    Some((old_start, old_count, new_start, new_count))
}

/// Parse a range like "start,count" or just "start"
fn parse_range(s: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = s.split(',').collect();
    let start = parts.first()?.parse::<usize>().ok()?.saturating_sub(1);
    let count = parts
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    Some((start, count))
}

/// Compute longest common subsequence indices
fn compute_lcs<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<(usize, usize)> {
    let m = a.len();
    let n = b.len();

    // Build LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find LCS
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

/// Generate diff hunks from LCS
fn generate_hunks(
    original: &[&str],
    modified: &[&str],
    lcs: &[(usize, usize)],
) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_lines = Vec::new();

    let mut old_idx = 0usize;
    let mut new_idx = 0usize;
    let mut lcs_idx = 0usize;

    let mut hunk_old_start = 0usize;
    let mut hunk_new_start = 0usize;
    let mut in_hunk = false;

    let context_lines = 3; // Lines of context around changes

    while old_idx < original.len() || new_idx < modified.len() {
        // Check if current position is in LCS (unchanged)
        let is_match = lcs_idx < lcs.len()
            && lcs[lcs_idx].0 == old_idx
            && lcs[lcs_idx].1 == new_idx;

        if is_match {
            // Context line
            if in_hunk {
                current_lines.push(DiffLine {
                    content: original[old_idx].to_string(),
                    line_type: DiffLineType::Context,
                    old_line: Some(old_idx),
                    new_line: Some(new_idx),
                });
            }
            old_idx += 1;
            new_idx += 1;
            lcs_idx += 1;
        } else {
            // Start a new hunk if needed
            if !in_hunk {
                in_hunk = true;
                hunk_old_start = old_idx.saturating_sub(context_lines);
                hunk_new_start = new_idx.saturating_sub(context_lines);

                // Add leading context
                let context_start_old = old_idx.saturating_sub(context_lines);
                let context_start_new = new_idx.saturating_sub(context_lines);
                for (i, j) in (context_start_old..old_idx).zip(context_start_new..new_idx) {
                    if i < original.len() && j < modified.len() && original[i] == modified[j] {
                        current_lines.push(DiffLine {
                            content: original[i].to_string(),
                            line_type: DiffLineType::Context,
                            old_line: Some(i),
                            new_line: Some(j),
                        });
                    }
                }
            }

            // Check what's next in LCS
            let next_old = if lcs_idx < lcs.len() {
                lcs[lcs_idx].0
            } else {
                original.len()
            };
            let next_new = if lcs_idx < lcs.len() {
                lcs[lcs_idx].1
            } else {
                modified.len()
            };

            // Add deletions
            while old_idx < next_old {
                current_lines.push(DiffLine {
                    content: original[old_idx].to_string(),
                    line_type: DiffLineType::Deletion,
                    old_line: Some(old_idx),
                    new_line: None,
                });
                old_idx += 1;
            }

            // Add additions
            while new_idx < next_new {
                current_lines.push(DiffLine {
                    content: modified[new_idx].to_string(),
                    line_type: DiffLineType::Addition,
                    old_line: None,
                    new_line: Some(new_idx),
                });
                new_idx += 1;
            }
        }

        // Check if we should end the current hunk
        let should_end_hunk = in_hunk && {
            // Count trailing context
            let remaining_context = current_lines
                .iter()
                .rev()
                .take_while(|l| l.line_type == DiffLineType::Context)
                .count();
            remaining_context >= context_lines * 2
        };

        if should_end_hunk {
            // Trim trailing context
            while current_lines
                .last()
                .map(|l| l.line_type == DiffLineType::Context)
                .unwrap_or(false)
                && current_lines
                    .iter()
                    .rev()
                    .take_while(|l| l.line_type == DiffLineType::Context)
                    .count()
                    > context_lines
            {
                current_lines.pop();
            }

            // Calculate counts
            let old_count = current_lines
                .iter()
                .filter(|l| {
                    matches!(l.line_type, DiffLineType::Context | DiffLineType::Deletion)
                })
                .count();
            let new_count = current_lines
                .iter()
                .filter(|l| {
                    matches!(l.line_type, DiffLineType::Context | DiffLineType::Addition)
                })
                .count();

            if old_count > 0 || new_count > 0 {
                hunks.push(DiffHunk {
                    old_start: hunk_old_start,
                    old_count,
                    new_start: hunk_new_start,
                    new_count,
                    lines: std::mem::take(&mut current_lines),
                });
            }

            in_hunk = false;
        }
    }

    // Handle remaining hunk
    if in_hunk && !current_lines.is_empty() {
        // Only include lines that have changes or are near changes
        let has_changes = current_lines
            .iter()
            .any(|l| matches!(l.line_type, DiffLineType::Addition | DiffLineType::Deletion));

        if has_changes {
            let old_count = current_lines
                .iter()
                .filter(|l| {
                    matches!(l.line_type, DiffLineType::Context | DiffLineType::Deletion)
                })
                .count();
            let new_count = current_lines
                .iter()
                .filter(|l| {
                    matches!(l.line_type, DiffLineType::Context | DiffLineType::Addition)
                })
                .count();

            hunks.push(DiffHunk {
                old_start: hunk_old_start,
                old_count,
                new_start: hunk_new_start,
                new_count,
                lines: current_lines,
            });
        }
    }

    hunks
}

/// Get statistics about a diff
pub fn diff_stats(hunks: &[DiffHunk]) -> DiffStats {
    let mut additions = 0;
    let mut deletions = 0;

    for hunk in hunks {
        for line in &hunk.lines {
            match line.line_type {
                DiffLineType::Addition => additions += 1,
                DiffLineType::Deletion => deletions += 1,
                _ => {}
            }
        }
    }

    DiffStats {
        additions,
        deletions,
        hunks: hunks.len(),
    }
}

/// Diff statistics
#[derive(Debug, Clone, Copy)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub hunks: usize,
}

impl DiffStats {
    pub fn summary(&self) -> String {
        format!("+{} -{} in {} hunks", self.additions, self.deletions, self.hunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_lcs() {
        let a = vec!["a", "b", "c", "d"];
        let b = vec!["a", "x", "c", "d"];
        let lcs = compute_lcs(&a, &b);
        // LCS should be [(0, 0), (2, 2), (3, 3)] for "a", "c", "d"
        assert_eq!(lcs.len(), 3);
    }

    #[test]
    fn test_simple_diff() {
        let original = "line1\nline2\nline3";
        let modified = "line1\nmodified\nline3";

        let diff = compute_unified_diff(original, modified);
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
    }

    #[test]
    fn test_parse_hunks() {
        let diff = "@@ -1,3 +1,3 @@\n line1\n-line2\n+modified\n line3\n";
        let hunks = parse_hunks(diff);

        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 0);
        assert_eq!(hunks[0].new_start, 0);
    }

    #[test]
    fn test_parse_hunk_header() {
        let result = parse_hunk_header("@@ -1,5 +1,6 @@");
        assert_eq!(result, Some((0, 5, 0, 6)));
    }

    #[test]
    fn test_diff_stats() {
        let hunks = vec![DiffHunk {
            old_start: 0,
            old_count: 3,
            new_start: 0,
            new_count: 3,
            lines: vec![
                DiffLine {
                    content: "line1".to_string(),
                    line_type: DiffLineType::Context,
                    old_line: Some(0),
                    new_line: Some(0),
                },
                DiffLine {
                    content: "old".to_string(),
                    line_type: DiffLineType::Deletion,
                    old_line: Some(1),
                    new_line: None,
                },
                DiffLine {
                    content: "new".to_string(),
                    line_type: DiffLineType::Addition,
                    old_line: None,
                    new_line: Some(1),
                },
            ],
        }];

        let stats = diff_stats(&hunks);
        assert_eq!(stats.additions, 1);
        assert_eq!(stats.deletions, 1);
        assert_eq!(stats.hunks, 1);
    }
}
