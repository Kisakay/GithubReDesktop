// SPDX-License-Identifier: MIT

//! Integration with external applications (editor, terminal, file manager)
//! and small formatting helpers.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::fl;

/// Editors probed, in order, when none is configured.
const EDITORS: &[&str] = &[
    "code",
    "codium",
    "zed",
    "zeditor",
    "cosmic-edit",
    "subl",
    "gnome-text-editor",
    "kate",
    "gedit",
    "idea",
    "nvim-qt",
    "neovide",
];

/// Terminals probed, in order, when none is configured.
const TERMINALS: &[&str] = &[
    "cosmic-term",
    "ghostty",
    "kitty",
    "alacritty",
    "wezterm",
    "foot",
    "konsole",
    "gnome-terminal",
    "kgx",
    "ptyxis",
    "xfce4-terminal",
    "tilix",
    "xterm",
];

fn first_available(candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .find(|c| which::which(c).is_ok())
        .map(|c| (*c).to_string())
}

/// Editors found on `PATH`.
#[must_use]
pub fn available_editors() -> Vec<String> {
    EDITORS
        .iter()
        .filter(|c| which::which(c).is_ok())
        .map(|c| (*c).to_string())
        .collect()
}

/// Terminals found on `PATH`.
#[must_use]
pub fn available_terminals() -> Vec<String> {
    TERMINALS
        .iter()
        .filter(|c| which::which(c).is_ok())
        .map(|c| (*c).to_string())
        .collect()
}

fn spawn(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
        .map_err(|e| format!("{program}: {e}"))
}

/// Splits a user-provided command into program + arguments.
fn split_command(command: &str) -> Option<(String, Vec<String>)> {
    let mut parts = command.split_whitespace().map(str::to_string);
    let program = parts.next()?;
    Some((program, parts.collect()))
}

/// Opens the repository in the configured (or detected) editor.
pub fn open_in_editor(configured: &str, repo: &Path) -> Result<(), String> {
    let command = Some(configured.trim().to_string())
        .filter(|c| !c.is_empty())
        .or_else(|| first_available(EDITORS))
        .ok_or_else(|| fl!("no-editor-found"))?;
    let (program, mut args) = split_command(&command).ok_or_else(|| fl!("no-editor-found"))?;
    args.push(repo.to_string_lossy().into_owned());
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    spawn(&program, &args, repo)
}

/// Opens a terminal in the repository directory.
pub fn open_in_terminal(configured: &str, repo: &Path) -> Result<(), String> {
    let command = Some(configured.trim().to_string())
        .filter(|c| !c.is_empty())
        .or_else(|| first_available(TERMINALS))
        .ok_or_else(|| fl!("no-terminal-found"))?;
    let (program, args) = split_command(&command).ok_or_else(|| fl!("no-terminal-found"))?;
    let mut args: Vec<String> = args;
    // Terminals that ignore the process cwd need an explicit flag.
    let dir = repo.to_string_lossy().into_owned();
    match program.as_str() {
        "gnome-terminal" | "kgx" | "ptyxis" | "tilix" | "xfce4-terminal" if args.is_empty() => {
            args.push(format!("--working-directory={dir}"));
        }
        "konsole" if args.is_empty() => args.extend(["--workdir".into(), dir]),
        "wezterm" if args.is_empty() => args.extend(["start".into(), "--cwd".into(), dir]),
        "kitty" if args.is_empty() => args.extend(["--directory".into(), dir]),
        "alacritty" if args.is_empty() => args.extend(["--working-directory".into(), dir]),
        "foot" if args.is_empty() => args.push(format!("--working-directory={dir}")),
        "ghostty" if args.is_empty() => args.push(format!("--working-directory={dir}")),
        _ => {}
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    spawn(&program, &args, repo)
}

/// Opens a path with the default file manager / application.
pub fn open_path(path: &Path) -> Result<(), String> {
    open::that_detached(path).map_err(|e| e.to_string())
}

/// Opens a URL in the default browser.
pub fn open_url(url: &str) -> Result<(), String> {
    open::that_detached(url).map_err(|e| e.to_string())
}

/// Human-friendly relative time ("3 hours ago").
#[must_use]
pub fn relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let secs = (now - timestamp).max(0);
    let (n, unit) = match secs {
        0..60 => return fl!("time-just-now"),
        60..3_600 => (secs / 60, "minute"),
        3_600..86_400 => (secs / 3_600, "hour"),
        86_400..2_592_000 => (secs / 86_400, "day"),
        2_592_000..31_536_000 => (secs / 2_592_000, "month"),
        _ => (secs / 31_536_000, "year"),
    };
    match unit {
        "minute" => fl!("time-minutes", count = n),
        "hour" => fl!("time-hours", count = n),
        "day" => fl!("time-days", count = n),
        "month" => fl!("time-months", count = n),
        _ => fl!("time-years", count = n),
    }
}

/// Formats a unix timestamp as a local date/time.
#[must_use]
pub fn format_date(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

/// Splits `dir/sub/file.rs` into (`file.rs`, `dir/sub/`).
#[must_use]
pub fn split_path(path: &str) -> (&str, &str) {
    match path.trim_end_matches('/').rfind('/') {
        Some(i) => (&path[i + 1..], &path[..=i]),
        None => (path, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_paths() {
        assert_eq!(split_path("src/git/mod.rs"), ("mod.rs", "src/git/"));
        assert_eq!(split_path("README.md"), ("README.md", ""));
    }

    #[test]
    fn splits_commands() {
        let (p, a) = split_command("code --new-window").unwrap();
        assert_eq!(p, "code");
        assert_eq!(a, vec!["--new-window"]);
        assert!(split_command("   ").is_none());
    }
}
