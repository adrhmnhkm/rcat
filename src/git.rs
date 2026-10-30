use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitLineStatus {
    Added,
    Modified,
    Removed,
}

pub struct GitDiffTracker {
    line_statuses: HashMap<usize, GitLineStatus>,
    all_added: bool,
}

impl GitDiffTracker {
    /// Mencoba membaca git diff untuk path file yang diberikan
    pub fn for_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        // 1. Cek apakah file untracked (file baru)
        if let Ok(status_output) = Command::new("git")
            .args(["status", "--porcelain", "--"])
            .arg(path)
            .output()
        {
            let status_str = String::from_utf8_lossy(&status_output.stdout);
            if status_str.trim_start().starts_with("??") {
                return Some(Self {
                    line_statuses: HashMap::new(),
                    all_added: true,
                });
            }
        }

        // 2. Jalankan git diff -U0 HEAD -- <path>
        if let Ok(diff_output) = Command::new("git")
            .args(["diff", "-U0", "HEAD", "--"])
            .arg(path)
            .output()
            && diff_output.status.success()
        {
            return Some(Self::parse_diff(&String::from_utf8_lossy(&diff_output.stdout)));
        }

        // 3. Fallback: coba git diff tanpa HEAD (berguna jika repo baru belum punya commit)
        if let Ok(fallback_output) = Command::new("git")
            .args(["diff", "-U0", "--"])
            .arg(path)
            .output()
            && fallback_output.status.success()
        {
            return Some(Self::parse_diff(&String::from_utf8_lossy(&fallback_output.stdout)));
        }

        None
    }

    /// Parsing teks unified diff
    pub fn parse_diff(diff_str: &str) -> Self {
        let mut map = HashMap::new();
        for line in diff_str.lines() {
            if line.starts_with("@@ ")
                && let Some((old_part, new_part)) = parse_hunk_header(line)
            {
                let (_old_start, old_count) = old_part;
                let (new_start, new_count) = new_part;

                if old_count == 0 {
                    // Penambahan baris baru
                    for i in 0..new_count {
                        map.insert(new_start + i, GitLineStatus::Added);
                    }
                } else if new_count == 0 {
                    // Penghapusan baris
                    map.insert(new_start, GitLineStatus::Removed);
                } else {
                    // Modifikasi baris
                    for i in 0..new_count {
                        map.insert(new_start + i, GitLineStatus::Modified);
                    }
                }
            }
        }
        Self {
            line_statuses: map,
            all_added: false,
        }
    }

    /// Mengambil status git untuk baris tertentu
    pub fn get_status(&self, line_num: usize) -> Option<GitLineStatus> {
        if self.all_added {
            return Some(GitLineStatus::Added);
        }
        self.line_statuses.get(&line_num).copied()
    }
}

/// Parsing header blok diff: @@ -old_start,old_count +new_start,new_count @@
fn parse_hunk_header(line: &str) -> Option<((usize, usize), (usize, usize))> {
    // Ambil bagian di antara @@ ... @@
    let inner = line.strip_prefix("@@ ")?.split_once(" @@")?.0;
    let mut parts = inner.split_whitespace();

    let old_str = parts.next()?.strip_prefix('-')?;
    let new_str = parts.next()?.strip_prefix('+')?;

    let old_range = parse_range(old_str)?;
    let new_range = parse_range(new_str)?;

    Some((old_range, new_range))
}

fn parse_range(s: &str) -> Option<(usize, usize)> {
    if let Some((start_str, count_str)) = s.split_once(',') {
        let start = start_str.parse::<usize>().ok()?;
        let count = count_str.parse::<usize>().ok()?;
        Some((start, count))
    } else {
        let start = s.parse::<usize>().ok()?;
        Some((start, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hunk() {
        let line = "@@ -8,4 +10,3 @@ some context";
        let (old, new) = parse_hunk_header(line).unwrap();
        assert_eq!(old, (8, 4));
        assert_eq!(new, (10, 3));

        let line2 = "@@ -5 +7 @@";
        let (old2, new2) = parse_hunk_header(line2).unwrap();
        assert_eq!(old2, (5, 1));
        assert_eq!(new2, (7, 1));
    }

    #[test]
    fn test_parse_diff_content() {
        let diff = "\
--- a/test.rs
+++ b/test.rs
@@ -1,0 +2,2 @@
+new line 1
+new line 2
@@ -10,3 +12,1 @@
-removed
+modified
";
        let tracker = GitDiffTracker::parse_diff(diff);
        assert_eq!(tracker.get_status(2), Some(GitLineStatus::Added));
        assert_eq!(tracker.get_status(3), Some(GitLineStatus::Added));
        assert_eq!(tracker.get_status(12), Some(GitLineStatus::Modified));
        assert_eq!(tracker.get_status(1), None);
    }
}

