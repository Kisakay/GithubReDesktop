// SPDX-License-Identifier: MIT

//! Unified diff loading and parsing.

use std::path::Path;

use super::{FileChange, FileStatus, Result, run_raw};

/// Hash of git's empty tree, used to diff against an unborn branch.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
/// Maximum number of diff lines kept for display.
pub const MAX_LINES: usize = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
    HunkHeader,
    /// "\ No newline at end of file"
    Marker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_no: Option<u32>,
    pub new_no: Option<u32>,
    pub text: String,
}

/// A parsed diff of a single file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    pub lines: Vec<DiffLine>,
    pub binary: bool,
    pub truncated: bool,
    pub additions: usize,
    pub deletions: usize,
}

/// Diff of a working tree change against HEAD.
pub async fn working_diff(repo: &Path, file: &FileChange) -> Result<Diff> {
    let raw = if file.status == FileStatus::Untracked {
        // `--no-index` exits with 1 when files differ: read stdout regardless.
        let out = tokio::process::Command::new("git")
            .current_dir(repo)
            .args([
                "-c",
                "core.quotepath=false",
                "diff",
                "--no-color",
                "--no-ext-diff",
                "--no-index",
                "--",
            ])
            .arg("/dev/null")
            .arg(&file.path)
            .env("LC_ALL", "C")
            .stdin(std::process::Stdio::null())
            .output()
            .await?;
        out.stdout
    } else {
        let base = if super::has_head(repo).await {
            "HEAD"
        } else {
            EMPTY_TREE
        };
        let mut args = vec![
            "diff".to_string(),
            "--no-color".into(),
            "--no-ext-diff".into(),
            "-M".into(),
            base.into(),
            "--".into(),
            file.path.clone(),
        ];
        if let Some(orig) = &file.orig_path {
            args.push(orig.clone());
        }
        run_raw(repo, args, None).await?
    };
    Ok(parse(&String::from_utf8_lossy(&raw)))
}

/// Diff of a file inside a commit.
pub async fn commit_diff(repo: &Path, sha: &str, path: &str, orig: Option<&str>) -> Result<Diff> {
    let mut args = vec![
        "show".to_string(),
        "--format=".into(),
        "--no-color".into(),
        "--no-ext-diff".into(),
        "-M".into(),
        "--first-parent".into(),
        sha.into(),
        "--".into(),
        path.into(),
    ];
    if let Some(orig) = orig {
        args.push(orig.into());
    }
    let raw = run_raw(repo, args, None).await?;
    Ok(parse(&String::from_utf8_lossy(&raw)))
}

/// Parses unified diff text.
#[must_use]
pub fn parse(raw: &str) -> Diff {
    let mut diff = Diff::default();
    let mut old_no = 0u32;
    let mut new_no = 0u32;
    let mut in_hunk = false;

    for line in raw.lines() {
        if !in_hunk {
            if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
                diff.binary = true;
            }
            if !line.starts_with("@@") {
                continue;
            }
        }

        if line.starts_with("@@") {
            in_hunk = true;
            if let Some((o, n)) = parse_hunk_header(line) {
                old_no = o;
                new_no = n;
            }
            push(&mut diff, DiffLineKind::HunkHeader, None, None, line);
            continue;
        }

        if let Some(rest) = line.strip_prefix('+') {
            diff.additions += 1;
            push(&mut diff, DiffLineKind::Added, None, Some(new_no), rest);
            new_no += 1;
        } else if let Some(rest) = line.strip_prefix('-') {
            diff.deletions += 1;
            push(&mut diff, DiffLineKind::Removed, Some(old_no), None, rest);
            old_no += 1;
        } else if let Some(rest) = line.strip_prefix(' ') {
            push(&mut diff, DiffLineKind::Context, Some(old_no), Some(new_no), rest);
            old_no += 1;
            new_no += 1;
        } else if line.starts_with('\\') {
            push(&mut diff, DiffLineKind::Marker, None, None, line);
        } else if line.starts_with("diff --git") {
            // Next file (only happens for renames listing both paths).
            in_hunk = false;
        } else if line.is_empty() {
            push(&mut diff, DiffLineKind::Context, Some(old_no), Some(new_no), "");
            old_no += 1;
            new_no += 1;
        }
    }
    diff
}

fn push(diff: &mut Diff, kind: DiffLineKind, old_no: Option<u32>, new_no: Option<u32>, text: &str) {
    if diff.lines.len() >= MAX_LINES {
        diff.truncated = true;
        return;
    }
    diff.lines.push(DiffLine {
        kind,
        old_no,
        new_no,
        text: text.replace('\t', "    "),
    });
}

/// Parses `@@ -a,b +c,d @@` into the starting line numbers.
fn parse_hunk_header(line: &str) -> Option<(u32, u32)> {
    let inner = line.strip_prefix("@@ ")?;
    let mut parts = inner.split_whitespace();
    let old = parts.next()?.strip_prefix('-')?;
    let new = parts.next()?.strip_prefix('+')?;
    let start = |s: &str| s.split(',').next().and_then(|n| n.parse().ok());
    Some((start(old)?, start(new)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "diff --git a/a.txt b/a.txt
index 1111111..2222222 100644
--- a/a.txt
+++ b/a.txt
@@ -1,3 +1,4 @@ fn main()
 one
-two
+deux
+trois
 three
\\ No newline at end of file
";

    #[test]
    fn parses_hunks() {
        let diff = parse(SAMPLE);
        assert_eq!(diff.additions, 2);
        assert_eq!(diff.deletions, 1);
        assert!(!diff.binary);
        let kinds: Vec<_> = diff.lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![
                DiffLineKind::HunkHeader,
                DiffLineKind::Context,
                DiffLineKind::Removed,
                DiffLineKind::Added,
                DiffLineKind::Added,
                DiffLineKind::Context,
                DiffLineKind::Marker,
            ]
        );
        assert_eq!(diff.lines[2].old_no, Some(2));
        assert_eq!(diff.lines[4].new_no, Some(3));
        assert_eq!(diff.lines[5].old_no, Some(3));
        assert_eq!(diff.lines[5].new_no, Some(4));
    }

    #[test]
    fn detects_binary() {
        let diff = parse("diff --git a/x.png b/x.png\nBinary files a/x.png and b/x.png differ\n");
        assert!(diff.binary);
    }

    #[test]
    fn hunk_header() {
        assert_eq!(parse_hunk_header("@@ -10,7 +12,8 @@"), Some((10, 12)));
        assert_eq!(parse_hunk_header("@@ -0,0 +1 @@"), Some((0, 1)));
    }
}
