// SPDX-License-Identifier: MIT

//! Minimal GitHub REST API client and secure token storage.

use serde::{Deserialize, Serialize};

const API: &str = "https://api.github.com";
const KEYRING_SERVICE: &str = "io.github.kisakay.GithubReDesktop";
const KEYRING_USER: &str = "github-token";
/// Page where users can create a personal access token.
pub const NEW_TOKEN_URL: &str =
    "https://github.com/settings/tokens/new?scopes=repo,read:org,workflow&description=GitHub%20ReDesktop";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Repo {
    pub full_name: String,
    pub clone_url: String,
    pub html_url: String,
    pub private: bool,
    pub fork: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
struct NewRepo<'a> {
    name: &'a str,
    description: &'a str,
    private: bool,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("GithubReDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
}

async fn error_of(resp: reqwest::Response) -> String {
    let status = resp.status();
    match resp.json::<ApiError>().await {
        Ok(err) => format!("GitHub: {} ({status})", err.message),
        Err(_) => format!("GitHub returned {status}"),
    }
}

/// Returns the authenticated user.
pub async fn user(token: &str) -> Result<User, String> {
    let resp = client()
        .map_err(|e| e.to_string())?
        .get(format!("{API}/user"))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(error_of(resp).await);
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// Lists the repositories the user can access (up to 500).
pub async fn repos(token: &str) -> Result<Vec<Repo>, String> {
    let client = client().map_err(|e| e.to_string())?;
    let mut all = Vec::new();
    for page in 1..=5 {
        let resp = client
            .get(format!(
                "{API}/user/repos?per_page=100&sort=pushed&affiliation=owner,collaborator,organization_member&page={page}"
            ))
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(error_of(resp).await);
        }
        let batch: Vec<Repo> = resp.json().await.map_err(|e| e.to_string())?;
        let done = batch.len() < 100;
        all.extend(batch);
        if done {
            break;
        }
    }
    Ok(all)
}

/// Creates a repository on the user's account and returns it.
pub async fn create_repo(token: &str, name: &str, description: &str, private: bool) -> Result<Repo, String> {
    let resp = client()
        .map_err(|e| e.to_string())?
        .post(format!("{API}/user/repos"))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .json(&NewRepo {
            name,
            description,
            private,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(error_of(resp).await);
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// Downloads an avatar image.
pub async fn avatar(url: &str) -> Result<Vec<u8>, String> {
    let resp = client()
        .map_err(|e| e.to_string())?
        .get(format!("{url}{}s=96", if url.contains('?') { '&' } else { '?' }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())
}

/// Reads the token of the GitHub CLI (`gh auth token`), if installed.
pub async fn gh_cli_token() -> Option<String> {
    let out = tokio::process::Command::new("gh")
        .args(["auth", "token"])
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .ok()?;
    let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !token.is_empty()).then_some(token)
}

fn fallback_token_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("github-redesktop").join("token"))
}

/// Loads the stored token (Secret Service, then a private fallback file).
pub async fn load_token() -> Option<String> {
    let from_keyring = tokio::task::spawn_blocking(|| {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .and_then(|e| e.get_password())
            .ok()
    })
    .await
    .ok()
    .flatten();
    if from_keyring.is_some() {
        return from_keyring;
    }
    let path = fallback_token_path()?;
    tokio::fs::read_to_string(path)
        .await
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Stores the token securely, falling back to a `0600` file when no
/// Secret Service provider is available.
pub async fn store_token(token: String) -> Result<(), String> {
    let t = token.clone();
    let stored = tokio::task::spawn_blocking(move || {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).and_then(|e| e.set_password(&t))
    })
    .await
    .map_err(|e| e.to_string())?;
    if stored.is_ok() {
        return Ok(());
    }
    let path = fallback_token_path().ok_or("no config directory")?;
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    }
    write_private(&path, &token).map_err(|e| e.to_string())
}

#[cfg(unix)]
fn write_private(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())
}

#[cfg(not(unix))]
fn write_private(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

/// Removes the stored token from every backend.
pub async fn delete_token() {
    let _ = tokio::task::spawn_blocking(|| {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).and_then(|e| e.delete_credential())
    })
    .await;
    if let Some(path) = fallback_token_path() {
        let _ = tokio::fs::remove_file(path).await;
    }
}
