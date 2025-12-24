//! Configuration handling for the TUI

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User configuration for the TUI
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TuiConfig {
    /// Issue sort field
    pub issue_sort_field: Option<String>,
    /// Issue sort direction
    pub issue_sort_direction: Option<String>,
    /// PR sort field
    pub pr_sort_field: Option<String>,
    /// PR sort direction
    pub pr_sort_direction: Option<String>,
    /// Show closed issues by default
    pub show_closed_issues: Option<bool>,
    /// Show merged PRs by default
    pub show_merged_prs: Option<bool>,
    /// Daemon address
    pub daemon_address: Option<String>,
}

#[allow(dead_code)]
impl TuiConfig {
    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("io", "centy", "centy-tui")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if let Some(path) = path {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                let config: TuiConfig = serde_json::from_str(&content)?;
                return Ok(config);
            }
        }

        Ok(Self::default())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(&path, content)?;
        }
        Ok(())
    }
}
