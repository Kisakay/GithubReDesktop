// SPDX-License-Identifier: MIT

//! Git backend.
//!
//! Like the original GitHub Desktop, every operation shells out to the `git`
//! executable. This keeps behaviour identical to the command line (hooks,
//! credential helpers, GPG signing, `.gitconfig`, …) and avoids linking a
//! second git implementation.

pub mod branch;
pub mod diff;
pub mod log;
pub mod status;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

pub use branch::Branch;
pub use diff::{Diff, DiffLineKind};
pub use log::{Commit, CommitFile};
pub use status::{FileChange, FileStatus, RepoStatus};

/// Environment variable telling our binary to behave as a `GIT_ASKPASS` helper.
pub const ASKPASS_ENV: &str = "GITHUB_REDESKTOP_ASKPASS";
/// Environment variable carrying the GitHub token to the askpass helper.
pub const TOKEN_ENV: &str = "GITHUB_REDESKTOP_TOKEN";

/// An error produced by a git invocation.
#[derive(Debug, Clone)]
pub struct GitError(pub String);

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GitError {}

pub type Result<T> = std::result::Result<T, GitError>;

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::NotFound {
            GitError("git executable not found. Please install git.".into())
        } else {
            GitError(err.to_string())
        }
    }
}

/// Builds a git command for the given working directory with a sane,
/// non-interactive environment.
fn command<I, S>(cwd: &Path, args: I, token: Option<&str>) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("git");
    cmd.current_dir(cwd)
        .args(["-c", "core.quotepath=false", "-c", "color.ui=false"])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    if let Some(token) = token.filter(|t| !t.is_empty())
        && let Ok(exe) = std::env::current_exe()
    {
        cmd.env("GIT_ASKPASS", exe)
            .env("SSH_ASKPASS_REQUIRE", "never")
            .env(ASKPASS_ENV, "1")
            .env(TOKEN_ENV, token);
    }

    cmd
}

/// Runs git and returns its stdout as bytes.
pub async fn run_raw<I, S>(cwd: &Path, args: I, token: Option<&str>) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = command(cwd, args, token).output().await?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = clean_error(if stderr.trim().is_empty() {
            &stdout
        } else {
            &stderr
        });
        Err(GitError(msg))
    }
}

/// Runs git and returns its stdout as a (lossy) UTF-8 string.
pub async fn run<I, S>(cwd: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_raw(cwd, args, None)
        .await
        .map(|out| String::from_utf8_lossy(&out).into_owned())
}

/// Runs a network-enabled git command, providing the GitHub token if any.
pub async fn run_net<I, S>(cwd: &Path, args: I, token: Option<&str>) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_raw(cwd, args, token)
        .await
        .map(|out| String::from_utf8_lossy(&out).into_owned())
}

/// Removes noisy prefixes from git error output.
fn clean_error(raw: &str) -> String {
    let lines: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("hint:"))
        .collect();
    let msg = lines.join("\n");
    let msg = msg
        .strip_prefix("fatal: ")
        .or_else(|| msg.strip_prefix("error: "))
        .unwrap_or(&msg)
        .to_string();
    if msg.is_empty() {
        "git exited with an error".into()
    } else {
        msg
    }
}

/// Behaviour when our executable is invoked by git as `GIT_ASKPASS`.
/// Returns `true` if the process acted as askpass and should exit.
pub fn handle_askpass() -> bool {
    if std::env::var_os(ASKPASS_ENV).is_none() {
        return false;
    }
    let prompt = std::env::args().nth(1).unwrap_or_default().to_lowercase();
    if prompt.starts_with("username") {
        println!("x-access-token");
    } else if prompt.starts_with("password") {
        println!("{}", std::env::var(TOKEN_ENV).unwrap_or_default());
    } else {
        // Unknown prompt (e.g. SSH host key confirmation): answer nothing.
        println!();
    }
    true
}

/// Returns the top-level directory of the repository containing `path`.
pub async fn toplevel(path: &Path) -> Result<PathBuf> {
    let out = run(path, ["rev-parse", "--show-toplevel"]).await?;
    Ok(PathBuf::from(out.trim()))
}

/// Initialises a new repository, optionally with a README and an initial commit.
pub async fn init(path: &Path, name: &str, readme: bool, gitignore: Option<&str>) -> Result<()> {
    tokio::fs::create_dir_all(path).await?;
    run(path, ["init", "-b", "main"]).await?;
    let mut files = Vec::new();
    if readme {
        tokio::fs::write(path.join("README.md"), format!("# {name}\n")).await?;
        files.push("README.md");
    }
    if let Some(ignore) = gitignore.filter(|g| !g.trim().is_empty()) {
        tokio::fs::write(path.join(".gitignore"), format!("{}\n", ignore.trim())).await?;
        files.push(".gitignore");
    }
    if !files.is_empty() {
        let mut args = vec!["add", "--"];
        args.extend(files);
        run(path, args).await?;
        run(path, ["commit", "-m", "Initial commit"]).await?;
    }
    Ok(())
}

/// Clones `url` into `dest`.
pub async fn clone(url: &str, dest: &Path, token: Option<&str>) -> Result<()> {
    let parent = dest.parent().unwrap_or(Path::new("/"));
    tokio::fs::create_dir_all(parent).await?;
    run_net(
        parent,
        [
            OsStr::new("clone"),
            OsStr::new("--"),
            OsStr::new(url),
            dest.as_os_str(),
        ],
        token,
    )
    .await?;
    Ok(())
}

/// Stages the given paths and commits them.
pub async fn commit(
    repo: &Path,
    paths: &[String],
    summary: &str,
    description: &str,
    amend: bool,
) -> Result<()> {
    if !paths.is_empty() {
        let mut add = vec!["add", "-A", "--"];
        add.extend(paths.iter().map(String::as_str));
        run(repo, add).await?;
    }
    let mut args = vec!["commit", "-m", summary];
    if !description.trim().is_empty() {
        args.extend(["-m", description]);
    }
    if amend {
        args.push("--amend");
    }
    if !paths.is_empty() {
        args.push("--");
        args.extend(paths.iter().map(String::as_str));
    }
    run(repo, args).await?;
    Ok(())
}

/// Discards the working tree changes for the given files.
pub async fn discard(repo: &Path, files: &[FileChange]) -> Result<()> {
    let mut tracked = Vec::new();
    for file in files {
        match file.status {
            FileStatus::Untracked => {
                let full = repo.join(&file.path);
                if trash::delete(&full).is_err() {
                    if full.is_dir() {
                        tokio::fs::remove_dir_all(&full).await?;
                    } else {
                        tokio::fs::remove_file(&full).await?;
                    }
                }
            }
            FileStatus::Added => {
                run(repo, ["rm", "--cached", "-q", "--", &file.path]).await?;
                let _ = trash::delete(repo.join(&file.path));
            }
            _ => {
                tracked.push(file.path.clone());
                if let Some(orig) = &file.orig_path {
                    tracked.push(orig.clone());
                }
            }
        }
    }
    if !tracked.is_empty() {
        if has_head(repo).await {
            // Restores index and worktree from HEAD; paths absent from HEAD
            // (the new side of a rename) are removed from the index.
            for path in &tracked {
                if run(repo, ["cat-file", "-e", &format!("HEAD:{path}")])
                    .await
                    .is_ok()
                {
                    run(repo, ["checkout", "HEAD", "--", path]).await?;
                } else {
                    let _ = run(repo, ["rm", "--cached", "-q", "--", path]).await;
                    let _ = trash::delete(repo.join(path));
                }
            }
        } else {
            let mut args = vec!["rm", "--cached", "-rq", "--"];
            args.extend(tracked.iter().map(String::as_str));
            run(repo, args).await?;
        }
    }
    Ok(())
}

/// Appends a pattern to the repository's `.gitignore`.
pub async fn ignore(repo: &Path, pattern: &str) -> Result<()> {
    let path = repo.join(".gitignore");
    let mut content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(pattern);
    content.push('\n');
    tokio::fs::write(&path, content).await?;
    Ok(())
}

/// Whether the repository has at least one commit.
pub async fn has_head(repo: &Path) -> bool {
    run(repo, ["rev-parse", "--verify", "-q", "HEAD"]).await.is_ok()
}

/// Fetches all remotes.
pub async fn fetch(repo: &Path, token: Option<&str>) -> Result<()> {
    run_net(repo, ["fetch", "--all", "--prune"], token).await?;
    Ok(())
}

/// Pulls the current branch.
pub async fn pull(repo: &Path, token: Option<&str>) -> Result<()> {
    run_net(repo, ["pull", "--no-edit"], token).await?;
    Ok(())
}

/// Pushes the current branch, publishing it to the default remote if needed.
pub async fn push(repo: &Path, has_upstream: bool, force: bool, token: Option<&str>) -> Result<()> {
    let mut args: Vec<String> = vec!["push".into()];
    if force {
        args.push("--force-with-lease".into());
    }
    if !has_upstream {
        let remote = default_remote(repo)
            .await
            .ok_or_else(|| GitError("This repository has no remote to publish to.".into()))?;
        args.extend(["--set-upstream".into(), remote, "HEAD".into()]);
    }
    run_net(repo, args, token).await?;
    Ok(())
}

/// Returns the first remote (preferring `origin`).
pub async fn default_remote(repo: &Path) -> Option<String> {
    let out = run(repo, ["remote"]).await.ok()?;
    let remotes: Vec<&str> = out.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if remotes.contains(&"origin") {
        Some("origin".into())
    } else {
        remotes.first().map(|s| (*s).to_string())
    }
}

/// Returns the URL of the default remote.
pub async fn remote_url(repo: &Path) -> Option<String> {
    let remote = default_remote(repo).await?;
    run(repo, ["remote", "get-url", &remote])
        .await
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Adds or updates the `origin` remote.
pub async fn set_origin(repo: &Path, url: &str) -> Result<()> {
    if run(repo, ["remote", "get-url", "origin"]).await.is_ok() {
        run(repo, ["remote", "set-url", "origin", url]).await?;
    } else {
        run(repo, ["remote", "add", "origin", url]).await?;
    }
    Ok(())
}

/// Stashes all changes (including untracked files).
pub async fn stash(repo: &Path, message: &str) -> Result<()> {
    run(repo, ["stash", "push", "--include-untracked", "-m", message]).await?;
    Ok(())
}

/// Restores the latest stash entry.
pub async fn stash_pop(repo: &Path) -> Result<()> {
    run(repo, ["stash", "pop"]).await?;
    Ok(())
}

/// Number of stash entries.
pub async fn stash_count(repo: &Path) -> usize {
    run(repo, ["stash", "list"])
        .await
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// Undoes the last commit, keeping its changes staged.
pub async fn undo_last_commit(repo: &Path) -> Result<()> {
    if run(repo, ["rev-parse", "--verify", "-q", "HEAD~1"]).await.is_ok() {
        run(repo, ["reset", "--soft", "HEAD~1"]).await?;
    } else {
        // Root commit: drop the branch ref, keep the index.
        run(repo, ["update-ref", "-d", "HEAD"]).await?;
    }
    Ok(())
}

/// Reverts a commit.
pub async fn revert(repo: &Path, sha: &str) -> Result<()> {
    run(repo, ["revert", "--no-edit", sha]).await?;
    Ok(())
}

/// Cherry-picks a commit onto the current branch.
pub async fn cherry_pick(repo: &Path, sha: &str) -> Result<()> {
    run(repo, ["cherry-pick", sha]).await?;
    Ok(())
}

/// Aborts an in-progress merge.
pub async fn merge_abort(repo: &Path) -> Result<()> {
    run(repo, ["merge", "--abort"]).await?;
    Ok(())
}

/// Reads a git config value (global when `repo` is `None`).
pub async fn config_get(repo: Option<&Path>, key: &str) -> Option<String> {
    let cwd = repo.map_or_else(
        || dirs::home_dir().unwrap_or_else(|| "/".into()),
        Path::to_path_buf,
    );
    run(&cwd, ["config", "--get", key])
        .await
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Writes a global git config value.
pub async fn config_set_global(key: &str, value: &str) -> Result<()> {
    let cwd = dirs::home_dir().unwrap_or_else(|| "/".into());
    run(&cwd, ["config", "--global", key, value]).await?;
    Ok(())
}

/// Turns a remote URL into a browsable `https://` URL (GitHub & co).
#[must_use]
pub fn web_url(remote: &str) -> Option<String> {
    let remote = remote.trim();
    let (host, path) = if let Some(rest) = remote.strip_prefix("ssh://") {
        let rest = rest.split_once('@').map_or(rest, |(_, r)| r);
        rest.split_once('/')?
    } else if let Some(rest) = remote
        .strip_prefix("https://")
        .or_else(|| remote.strip_prefix("http://"))
    {
        let rest = rest.split_once('@').map_or(rest, |(_, r)| r);
        rest.split_once('/')?
    } else if let Some((_, rest)) = remote.split_once('@') {
        rest.split_once(':')?
    } else {
        return None;
    };
    let host = host.split_once(':').map_or(host, |(h, _)| h);
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    Some(format!("https://{host}/{path}"))
}

/// Infers a directory name from a clone URL.
#[must_use]
pub fn repo_name_from_url(url: &str) -> String {
    url.trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit(['/', ':'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("repository")
        .to_string()
}

/// Expands `owner/repo` shorthands into a GitHub clone URL.
#[must_use]
pub fn normalize_clone_url(input: &str) -> String {
    let input = input.trim();
    let is_shorthand = !input.contains("://")
        && !input.contains('@')
        && input.matches('/').count() == 1
        && !input.starts_with('/')
        && !input.starts_with('.')
        && !input.starts_with('~');
    if is_shorthand {
        format!("https://github.com/{input}.git")
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_urls() {
        assert_eq!(
            web_url("git@github.com:Kisakay/GithubReDesktop.git").as_deref(),
            Some("https://github.com/Kisakay/GithubReDesktop")
        );
        assert_eq!(
            web_url("https://github.com/desktop/desktop.git").as_deref(),
            Some("https://github.com/desktop/desktop")
        );
        assert_eq!(
            web_url("https://user:tok@github.com/a/b").as_deref(),
            Some("https://github.com/a/b")
        );
        assert_eq!(
            web_url("ssh://git@gitlab.com:22/group/proj.git").as_deref(),
            Some("https://gitlab.com/group/proj")
        );
        assert_eq!(web_url("/local/path"), None);
    }

    #[test]
    fn names_from_urls() {
        assert_eq!(
            repo_name_from_url("https://github.com/desktop/desktop.git"),
            "desktop"
        );
        assert_eq!(repo_name_from_url("git@github.com:a/b-c.git"), "b-c");
        assert_eq!(repo_name_from_url("https://github.com/a/b/"), "b");
    }

    #[test]
    fn shorthand_urls() {
        assert_eq!(
            normalize_clone_url("desktop/desktop"),
            "https://github.com/desktop/desktop.git"
        );
        assert_eq!(
            normalize_clone_url("git@github.com:a/b.git"),
            "git@github.com:a/b.git"
        );
        assert_eq!(normalize_clone_url("./a/b"), "./a/b");
    }

    #[test]
    fn error_cleanup() {
        assert_eq!(
            clean_error("fatal: not a git repository\nhint: foo"),
            "not a git repository"
        );
        assert_eq!(clean_error(""), "git exited with an error");
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod workflow_tests;
