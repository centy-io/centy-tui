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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TuiConfig::default();
        assert!(config.issue_sort_field.is_none());
        assert!(config.issue_sort_direction.is_none());
        assert!(config.pr_sort_field.is_none());
        assert!(config.pr_sort_direction.is_none());
        assert!(config.show_closed_issues.is_none());
        assert!(config.show_merged_prs.is_none());
        assert!(config.daemon_address.is_none());
    }

    #[test]
    fn test_serialization() {
        let config = TuiConfig {
            issue_sort_field: Some("priority".to_string()),
            issue_sort_direction: Some("asc".to_string()),
            pr_sort_field: Some("created".to_string()),
            pr_sort_direction: Some("desc".to_string()),
            show_closed_issues: Some(true),
            show_merged_prs: Some(false),
            daemon_address: Some("http://localhost:50051".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: TuiConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.issue_sort_field, Some("priority".to_string()));
        assert_eq!(parsed.issue_sort_direction, Some("asc".to_string()));
        assert_eq!(parsed.pr_sort_field, Some("created".to_string()));
        assert_eq!(parsed.pr_sort_direction, Some("desc".to_string()));
        assert_eq!(parsed.show_closed_issues, Some(true));
        assert_eq!(parsed.show_merged_prs, Some(false));
        assert_eq!(
            parsed.daemon_address,
            Some("http://localhost:50051".to_string())
        );
    }

    #[test]
    fn test_partial_serialization() {
        let config = TuiConfig {
            issue_sort_field: Some("priority".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: TuiConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.issue_sort_field, Some("priority".to_string()));
        assert!(parsed.issue_sort_direction.is_none());
    }

    #[test]
    fn test_deserialize_from_empty_json() {
        let json = "{}";
        let parsed: TuiConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.issue_sort_field.is_none());
    }

    #[test]
    fn test_deserialize_with_extra_fields() {
        // Should ignore unknown fields
        let json = r#"{"issue_sort_field": "priority", "unknown_field": "value"}"#;
        let parsed: TuiConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.issue_sort_field, Some("priority".to_string()));
    }

    #[test]
    fn test_config_path_returns_option() {
        // Just test that the function doesn't panic
        let _path = TuiConfig::config_path();
    }

    #[test]
    fn test_load_returns_default_when_no_file() {
        // Load should return default config when file doesn't exist
        // This test may pass or fail depending on whether config file exists
        let result = TuiConfig::load();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_clone() {
        let config = TuiConfig {
            issue_sort_field: Some("priority".to_string()),
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(config.issue_sort_field, cloned.issue_sort_field);
    }

    #[test]
    fn test_config_debug() {
        let config = TuiConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TuiConfig"));
    }
}
