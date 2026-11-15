// SPDX-License-Identifier: MIT

//! End-to-end tests running the real `git` binary in a temporary repository.

use std::path::PathBuf;

use super::*;

struct TempRepo(PathBuf);

impl TempRepo {
    async fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "grd-test-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        init(&dir, name, true, Some("target/")).await.expect("init");
        run(&dir, ["config", "user.name", "Test"]).await.unwrap();
        run(&dir, ["config", "user.email", "test@example.com"])
            .await
            .unwrap();
        run(&dir, ["config", "commit.gpgsign", "false"]).await.unwrap();
        Self(dir)
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn full_workflow() {
    // `init` commits with the global identity; make sure one exists for CI.
    if config_get(None, "user.email").await.is_none() {
        // SAFETY: single-threaded test setup before any git process is spawned.
        unsafe {
            std::env::set_var("GIT_AUTHOR_NAME", "Test");
            std::env::set_var("GIT_AUTHOR_EMAIL", "test@example.com");
            std::env::set_var("GIT_COMMITTER_NAME", "Test");
            std::env::set_var("GIT_COMMITTER_EMAIL", "test@example.com");
        }
    }
    let repo = TempRepo::new("workflow").await;
    let path = &repo.0;

    // Initial commit created by init (README + .gitignore).
    let commits = log::log(path, 0, 10).await.unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].summary, "Initial commit");

    // Modify, add and verify status + diff.
    tokio::fs::write(path.join("README.md"), "# workflow\n\nHello\n")
        .await
        .unwrap();
    tokio::fs::write(path.join("new.txt"), "new file\n")
        .await
        .unwrap();
    let st = status::status(path).await.unwrap();
    assert_eq!(st.branch.as_deref(), Some("main"));
    assert_eq!(st.files.len(), 2);
    let readme = st.files.iter().find(|f| f.path == "README.md").unwrap();
    assert_eq!(readme.status, FileStatus::Modified);
    let d = diff::working_diff(path, readme).await.unwrap();
    assert_eq!(d.additions, 2);
    let new = st.files.iter().find(|f| f.path == "new.txt").unwrap();
    assert_eq!(new.status, FileStatus::Untracked);
    let d = diff::working_diff(path, new).await.unwrap();
    assert_eq!(d.additions, 1);

    // Commit only README; new.txt must stay uncommitted.
    commit(path, &["README.md".into()], "Update readme", "Body text", false)
        .await
        .unwrap();
    let st = status::status(path).await.unwrap();
    assert_eq!(st.files.len(), 1);
    assert_eq!(st.files[0].path, "new.txt");
    let commits = log::log(path, 0, 10).await.unwrap();
    assert_eq!(commits[0].summary, "Update readme");
    assert_eq!(commits[0].body, "Body text");
    let files = log::commit_files(path, &commits[0].sha).await.unwrap();
    assert_eq!(files.len(), 1);
    let d = diff::commit_diff(path, &commits[0].sha, "README.md", None)
        .await
        .unwrap();
    assert_eq!(d.additions, 2);

    // Branches: create, list, switch back, merge, delete.
    branch::create(path, "feature", None).await.unwrap();
    commit(path, &["new.txt".into()], "Add new file", "", false)
        .await
        .unwrap();
    let branches = branch::list(path).await.unwrap();
    assert!(branches.iter().any(|b| b.name == "feature" && b.current));
    let main = branches.iter().find(|b| b.name == "main").unwrap().clone();
    branch::checkout(path, &main).await.unwrap();
    assert!(!path.join("new.txt").exists());
    branch::merge(path, "feature").await.unwrap();
    assert!(path.join("new.txt").exists());
    branch::delete(path, "feature").await.unwrap();

    // Stash round-trip.
    tokio::fs::write(path.join("new.txt"), "changed\n").await.unwrap();
    stash(path, "test stash").await.unwrap();
    assert!(status::status(path).await.unwrap().files.is_empty());
    assert_eq!(stash_count(path).await, 1);
    stash_pop(path).await.unwrap();
    assert_eq!(status::status(path).await.unwrap().files.len(), 1);

    // Discard.
    let st = status::status(path).await.unwrap();
    discard(path, &st.files).await.unwrap();
    assert!(status::status(path).await.unwrap().files.is_empty());

    // Undo last commit keeps the changes staged.
    undo_last_commit(path).await.unwrap();
    let st = status::status(path).await.unwrap();
    assert_eq!(st.files.len(), 1);
    assert_eq!(st.files[0].status, FileStatus::Added);

    // Revert.
    commit(path, &["new.txt".into()], "Re-add", "", false)
        .await
        .unwrap();
    let head = log::log(path, 0, 1).await.unwrap().remove(0);
    revert(path, &head.sha).await.unwrap();
    assert!(!path.join("new.txt").exists());

    // Ignore.
    ignore(path, "*.log").await.unwrap();
    let gitignore = tokio::fs::read_to_string(path.join(".gitignore")).await.unwrap();
    assert!(gitignore.contains("*.log"));
}

#[tokio::test]
async fn toplevel_errors_outside_repo() {
    let dir = std::env::temp_dir();
    // The system temp dir is not a repository (in CI and dev machines).
    if toplevel(&dir).await.is_ok() {
        return;
    }
    assert!(toplevel(&dir).await.is_err());
}
