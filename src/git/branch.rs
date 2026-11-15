// SPDX-License-Identifier: MIT

//! Branch management.

use std::path::Path;

use super::{Result, run};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    /// Short name (`main`, `origin/feature`).
    pub name: String,
    pub remote: bool,
    pub current: bool,
    pub upstream: Option<String>,
    /// Unix timestamp of the tip commit.
    pub time: i64,
}

/// Lists local and remote branches, most recently updated first.
pub async fn list(repo: &Path) -> Result<Vec<Branch>> {
    let out = run(
        repo,
        [
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname)%1f%(refname:short)%1f%(upstream:short)%1f%(committerdate:unix)%1f%(HEAD)",
            "refs/heads",
            "refs/remotes",
        ],
    )
    .await?;
    Ok(parse(&out))
}

#[must_use]
pub fn parse(out: &str) -> Vec<Branch> {
    out.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\u{1f}').collect();
            if f.len() < 5 {
                return None;
            }
            let remote = f[0].starts_with("refs/remotes/");
            // Skip symbolic `origin/HEAD` refs.
            if remote && f[0].ends_with("/HEAD") {
                return None;
            }
            Some(Branch {
                name: f[1].to_string(),
                remote,
                current: f[4].trim() == "*",
                upstream: Some(f[2].to_string()).filter(|s| !s.is_empty()),
                time: f[3].parse().unwrap_or(0),
            })
        })
        .collect()
}

/// Switches to an existing branch. Remote branches get a tracking local branch.
pub async fn checkout(repo: &Path, branch: &Branch) -> Result<()> {
    if branch.remote {
        let local = branch
            .name
            .split_once('/')
            .map_or(branch.name.as_str(), |(_, b)| b);
        if run(
            repo,
            ["rev-parse", "--verify", "-q", &format!("refs/heads/{local}")],
        )
        .await
        .is_ok()
        {
            run(repo, ["switch", local]).await?;
        } else {
            run(repo, ["switch", "--track", &branch.name]).await?;
        }
    } else {
        run(repo, ["switch", &branch.name]).await?;
    }
    Ok(())
}

/// Creates a new branch from `start` (HEAD when `None`) and switches to it.
pub async fn create(repo: &Path, name: &str, start: Option<&str>) -> Result<()> {
    let mut args = vec!["switch", "-c", name];
    if let Some(start) = start {
        args.push(start);
    }
    run(repo, args).await?;
    Ok(())
}

/// Deletes a local branch (force).
pub async fn delete(repo: &Path, name: &str) -> Result<()> {
    run(repo, ["branch", "-D", name]).await?;
    Ok(())
}

/// Renames a local branch.
pub async fn rename(repo: &Path, old: &str, new: &str) -> Result<()> {
    run(repo, ["branch", "-m", old, new]).await?;
    Ok(())
}

/// Merges `name` into the current branch.
pub async fn merge(repo: &Path, name: &str) -> Result<()> {
    run(repo, ["merge", "--no-edit", name]).await?;
    Ok(())
}

/// Sanitizes user input into a valid branch name (spaces → dashes, …).
#[must_use]
pub fn sanitize_name(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        let c = match c {
            ' ' | '~' | '^' | ':' | '?' | '*' | '[' | '\\' => '-',
            c if c.is_control() => continue,
            c => c,
        };
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }
    while out.contains("..") {
        out = out.replace("..", ".");
    }
    out.trim_matches(['-', '/', '.'])
        .trim_end_matches(".lock")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_refs() {
        let out = "refs/heads/main\u{1f}main\u{1f}origin/main\u{1f}1700000000\u{1f}*\n\
refs/remotes/origin/HEAD\u{1f}origin\u{1f}\u{1f}1700000000\u{1f} \n\
refs/remotes/origin/dev\u{1f}origin/dev\u{1f}\u{1f}1600000000\u{1f} \n";
        let branches = parse(out);
        assert_eq!(branches.len(), 2);
        assert!(branches[0].current);
        assert_eq!(branches[0].upstream.as_deref(), Some("origin/main"));
        assert!(branches[1].remote);
        assert_eq!(branches[1].name, "origin/dev");
    }

    #[test]
    fn sanitizes() {
        assert_eq!(sanitize_name("my new  feature"), "my-new-feature");
        assert_eq!(sanitize_name("fix: bug?"), "fix-bug");
        assert_eq!(sanitize_name("a..b"), "a.b");
        assert_eq!(sanitize_name("/feature/x/"), "feature/x");
    }
}
