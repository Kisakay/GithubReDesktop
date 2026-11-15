// SPDX-License-Identifier: MIT

//! Persistent application settings (stored with `cosmic-config`).

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    #[default]
    System,
    Dark,
    Light,
}

impl AppTheme {
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];

    pub fn theme(self) -> cosmic::Theme {
        match self {
            Self::System => cosmic::theme::system_preference(),
            Self::Dark => cosmic::Theme::dark(),
            Self::Light => cosmic::Theme::light(),
        }
    }

    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq, Eq)]
#[version = 1]
pub struct Config {
    /// Absolute paths of the repositories shown in the sidebar.
    pub repositories: Vec<String>,
    /// Repository opened on the previous run.
    pub last_repository: Option<String>,
    pub app_theme: AppTheme,
    /// Editor command (empty = auto-detect).
    pub editor: String,
    /// Terminal command (empty = auto-detect).
    pub terminal: String,
    /// Default parent directory for clones and new repositories.
    pub clone_dir: String,
    /// Periodically fetch the current repository in the background.
    pub auto_fetch: bool,
    /// Ask before discarding changes.
    pub confirm_discard: bool,
}

impl Default for Config {
    fn default() -> Self {
        let clone_dir = dirs::document_dir()
            .or_else(dirs::home_dir)
            .map(|d| d.join("GitHub").to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            repositories: Vec::new(),
            last_repository: None,
            app_theme: AppTheme::System,
            editor: String::new(),
            terminal: String::new(),
            clone_dir,
            auto_fetch: true,
            confirm_discard: true,
        }
    }
}

impl Config {
    /// Loads the configuration, falling back to defaults on errors.
    #[must_use]
    pub fn load(app_id: &str) -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new(app_id, Self::VERSION) {
            Ok(handler) => {
                let config = match Self::get_entry(&handler) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                };
                (Some(handler), config)
            }
            Err(err) => {
                eprintln!("failed to open config: {err}");
                (None, Self::default())
            }
        }
    }
}
