// SPDX-License-Identifier: MIT

//! Commit history.

use std::path::Path;

use super::{FileStatus, Result, run, run_raw};

/// A commit in the history list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub sha: String,
    pub short_sha: String,
    pub author: String,
    pub email: String,
    /// Unix timestamp (author date).
    pub time: i64,
    pub summary: String,
    pub body: String,
    pub parents: usize,
    pub refs: Vec<String>,
}

/// A file touched by a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitFile {
    pub path: String,
    pub orig_path: Option<String>,
    pub status: FileStatus,
}

const FIELD: char = '\u{1f}';
const RECORD: char = '\u{1e}';

/// Loads up to `limit` commits reachable from HEAD, skipping `skip`.
pub async fn log(repo: &Path, skip: usize, limit: usize) -> Result<Vec<Commit>> {
    if !super::has_head(repo).await {
        return Ok(Vec::new());
    }
    let out = run(
        repo,
        [
            "log".to_string(),
            "--date-order".into(),
            format!("--skip={skip}"),
            format!("--max-count={limit}"),
            "--format=%H%x1f%h%x1f%an%x1f%ae%x1f%at%x1f%P%x1f%D%x1f%s%x1f%b%x1e".into(),
            "HEAD".into(),
            "--".into(),
        ],
    )
    .await?;
    Ok(parse_log(&out))
}

/// Number of local commits not yet on the upstream (unpushed).
pub async fn unpushed_shas(repo: &Path) -> Vec<String> {
    run(repo, ["rev-list", "@{upstream}..HEAD"])
        .await
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// Parses the custom `git log` format used by [`log`].
#[must_use]
pub fn parse_log(out: &str) -> Vec<Commit> {
    out.split(RECORD)
        .map(|r| r.trim_start_matches('\n'))
        .filter(|r| !r.trim().is_empty())
        .filter_map(|record| {
            let f: Vec<&str> = record.splitn(9, FIELD).collect();
            if f.len() < 9 {
                return None;
            }
            Some(Commit {
                sha: f[0].to_string(),
                short_sha: f[1].to_string(),
                author: f[2].to_string(),
                email: f[3].to_string(),
                time: f[4].parse().unwrap_or(0),
                parents: f[5].split_whitespace().count(),
                refs: f[6]
                    .split(", ")
                    .map(|r| r.trim_start_matches("HEAD -> ").to_string())
                    .filter(|r| !r.is_empty() && r != "HEAD")
                    .collect(),
                summary: f[7].to_string(),
                body: f[8].trim().to_string(),
            })
        })
        .collect()
}

/// Files changed by a commit (compared to its first parent).
pub async fn commit_files(repo: &Path, sha: &str) -> Result<Vec<CommitFile>> {
    let raw = run_raw(
        repo,
        [
            "show",
            "--format=",
            "--name-status",
            "-M",
            "--first-parent",
            "-z",
            sha,
        ],
        None,
    )
    .await?;
    Ok(parse_name_status(&raw))
}

/// Parses `--name-status -z` output.
#[must_use]
pub fn parse_name_status(raw: &[u8]) -> Vec<CommitFile> {
    let mut parts = raw
        .split(|b| *b == 0)
        .map(|p| String::from_utf8_lossy(p).trim_start_matches('\n').to_string());
    let mut files = Vec::new();
    while let Some(code) = parts.next() {
        if code.is_empty() {
            continue;
        }
        let kind = code.as_bytes()[0];
        let (status, two_paths) = match kind {
            b'A' => (FileStatus::Added, false),
            b'D' => (FileStatus::Deleted, false),
            b'R' => (FileStatus::Renamed, true),
            b'C' => (FileStatus::Copied, true),
            b'T' => (FileStatus::TypeChanged, false),
            b'U' => (FileStatus::Conflicted, false),
            _ => (FileStatus::Modified, false),
        };
        if two_paths {
            let (Some(orig), Some(path)) = (parts.next(), parts.next()) else {
                break;
            };
            files.push(CommitFile {
                path,
                orig_path: Some(orig),
                status,
            });
        } else if let Some(path) = parts.next() {
            files.push(CommitFile {
                path,
                orig_path: None,
                status,
            });
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_log_records() {
        let out = "abc123\u{1f}abc\u{1f}Ada\u{1f}ada@example.com\u{1f}1700000000\u{1f}p1 p2\u{1f}HEAD -> main, origin/main\u{1f}Merge things\u{1f}Body line\n\u{1e}\n\
def456\u{1f}def\u{1f}Bob\u{1f}bob@example.com\u{1f}1690000000\u{1f}\u{1f}\u{1f}Initial commit\u{1f}\u{1e}\n";
        let commits = parse_log(out);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].summary, "Merge things");
        assert_eq!(commits[0].parents, 2);
        assert_eq!(commits[0].refs, vec!["main", "origin/main"]);
        assert_eq!(commits[0].body, "Body line");
        assert_eq!(commits[1].parents, 0);
        assert!(commits[1].refs.is_empty());
    }

    #[test]
    fn parses_name_status() {
        let raw = b"M\0src/a.rs\0R100\0old.rs\0new.rs\0A\0added.txt\0";
        let files = parse_name_status(raw);
        assert_eq!(files.len(), 3);
        assert_eq!(files[1].status, FileStatus::Renamed);
        assert_eq!(files[1].orig_path.as_deref(), Some("old.rs"));
        assert_eq!(files[1].path, "new.rs");
        assert_eq!(files[2].status, FileStatus::Added);
    }
}
