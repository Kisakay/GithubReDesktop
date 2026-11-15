// SPDX-License-Identifier: MIT

//! Working tree status (`git status --porcelain=v2`).

use std::path::Path;

use super::{Result, run, run_raw};

/// The kind of change applied to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Conflicted,
    TypeChanged,
}

impl FileStatus {
    /// Single-letter badge displayed next to the file.
    #[must_use]
    pub fn letter(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Copied => "C",
            Self::Untracked => "U",
            Self::Conflicted => "!",
            Self::TypeChanged => "T",
        }
    }

    fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            b'M' => Self::Modified,
            b'A' => Self::Added,
            b'D' => Self::Deleted,
            b'R' => Self::Renamed,
            b'C' => Self::Copied,
            b'T' => Self::TypeChanged,
            b'U' => Self::Conflicted,
            _ => return None,
        })
    }
}

/// A changed file in the working tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub orig_path: Option<String>,
    pub status: FileStatus,
}

/// Snapshot of a repository's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoStatus {
    /// Current branch, `None` when HEAD is detached.
    pub branch: Option<String>,
    /// Abbreviated HEAD commit, `None` on an unborn branch.
    pub head: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<FileChange>,
    pub merging: bool,
    pub stash_count: usize,
}

impl RepoStatus {
    #[must_use]
    pub fn has_conflicts(&self) -> bool {
        self.files.iter().any(|f| f.status == FileStatus::Conflicted)
    }
}

/// Reads the status of the repository at `repo`.
pub async fn status(repo: &Path) -> Result<RepoStatus> {
    let raw = run_raw(
        repo,
        [
            "status",
            "--porcelain=v2",
            "--branch",
            "-z",
            "--untracked-files=all",
        ],
        None,
    )
    .await?;
    let mut status = parse(&raw);

    if let Ok(path) = run(repo, ["rev-parse", "--git-path", "MERGE_HEAD"]).await {
        status.merging = repo.join(path.trim()).exists();
    }
    status.stash_count = super::stash_count(repo).await;
    Ok(status)
}

/// Parses `git status --porcelain=v2 --branch -z` output.
#[must_use]
pub fn parse(raw: &[u8]) -> RepoStatus {
    let mut status = RepoStatus::default();
    let mut records = raw.split(|b| *b == 0).filter(|r| !r.is_empty());

    while let Some(record) = records.next() {
        let line = String::from_utf8_lossy(record);
        if let Some(header) = line.strip_prefix("# ") {
            parse_header(&mut status, header);
            continue;
        }
        let mut kind = line.splitn(2, ' ');
        match kind.next() {
            Some("1") => {
                // 1 XY sub mH mI mW hH hI path
                let fields: Vec<&str> = line.splitn(9, ' ').collect();
                if let (Some(xy), Some(path)) = (fields.get(1), fields.get(8)) {
                    status.files.push(FileChange {
                        path: (*path).to_string(),
                        orig_path: None,
                        status: status_from_xy(xy.as_bytes()),
                    });
                }
            }
            Some("2") => {
                // 2 XY sub mH mI mW hH hI Xscore path \0 origPath
                let fields: Vec<&str> = line.splitn(10, ' ').collect();
                let orig = records.next().map(|o| String::from_utf8_lossy(o).into_owned());
                if let (Some(xy), Some(path)) = (fields.get(1), fields.get(9)) {
                    let xy = xy.as_bytes();
                    let st = if xy.contains(&b'C') {
                        FileStatus::Copied
                    } else {
                        FileStatus::Renamed
                    };
                    status.files.push(FileChange {
                        path: (*path).to_string(),
                        orig_path: orig,
                        status: st,
                    });
                }
            }
            Some("u") => {
                // u XY sub m1 m2 m3 mW h1 h2 h3 path
                let fields: Vec<&str> = line.splitn(11, ' ').collect();
                if let Some(path) = fields.get(10) {
                    status.files.push(FileChange {
                        path: (*path).to_string(),
                        orig_path: None,
                        status: FileStatus::Conflicted,
                    });
                }
            }
            Some("?") => {
                if let Some(path) = kind.next() {
                    status.files.push(FileChange {
                        path: path.to_string(),
                        orig_path: None,
                        status: FileStatus::Untracked,
                    });
                }
            }
            _ => {}
        }
    }

    status.files.sort_by(|a, b| a.path.cmp(&b.path));
    status
}

fn parse_header(status: &mut RepoStatus, header: &str) {
    let Some((key, value)) = header.split_once(' ') else {
        return;
    };
    match key {
        "branch.oid" if value != "(initial)" => {
            status.head = Some(value.chars().take(7).collect());
        }
        "branch.head" if value != "(detached)" => status.branch = Some(value.to_string()),
        "branch.upstream" => status.upstream = Some(value.to_string()),
        "branch.ab" => {
            for part in value.split_whitespace() {
                if let Some(n) = part.strip_prefix('+') {
                    status.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix('-') {
                    status.behind = n.parse().unwrap_or(0);
                }
            }
        }
        _ => {}
    }
}

/// Picks the most relevant status from the index (X) and worktree (Y) codes.
fn status_from_xy(xy: &[u8]) -> FileStatus {
    let x = xy.first().copied().unwrap_or(b'.');
    let y = xy.get(1).copied().unwrap_or(b'.');
    if x == b'A' {
        // Added to the index; deleting it afterwards means nothing to show,
        // but keep it visible as added.
        return FileStatus::Added;
    }
    if x == b'D' || y == b'D' {
        return FileStatus::Deleted;
    }
    FileStatus::from_code(y)
        .or_else(|| FileStatus::from_code(x))
        .unwrap_or(FileStatus::Modified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_porcelain_v2() {
        let raw = b"# branch.oid 1234567890abcdef\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +2 -1\0\
1 .M N... 100644 100644 100644 aaaa bbbb src/main.rs\0\
1 A. N... 000000 100644 100644 0000 cccc new file.txt\0\
2 R. N... 100644 100644 100644 dddd eeee R100 renamed.rs\0old.rs\0\
u UU N... 100644 100644 100644 100644 a b c conflict.rs\0\
? untracked.md\0";
        let st = parse(raw);
        assert_eq!(st.branch.as_deref(), Some("main"));
        assert_eq!(st.head.as_deref(), Some("1234567"));
        assert_eq!(st.upstream.as_deref(), Some("origin/main"));
        assert_eq!((st.ahead, st.behind), (2, 1));
        assert_eq!(st.files.len(), 5);
        let find = |p: &str| st.files.iter().find(|f| f.path == p).unwrap();
        assert_eq!(find("src/main.rs").status, FileStatus::Modified);
        assert_eq!(find("new file.txt").status, FileStatus::Added);
        assert_eq!(find("renamed.rs").status, FileStatus::Renamed);
        assert_eq!(find("renamed.rs").orig_path.as_deref(), Some("old.rs"));
        assert_eq!(find("conflict.rs").status, FileStatus::Conflicted);
        assert_eq!(find("untracked.md").status, FileStatus::Untracked);
        assert!(st.has_conflicts());
    }

    #[test]
    fn parses_initial_and_detached() {
        let st = parse(b"# branch.oid (initial)\0# branch.head main\0");
        assert_eq!(st.head, None);
        assert_eq!(st.branch.as_deref(), Some("main"));
        let st = parse(b"# branch.oid abcdef1234\0# branch.head (detached)\0");
        assert_eq!(st.branch, None);
    }
}
