//! Trait abstraction for daemon client to enable mocking in tests

use crate::state::{
    Config, DaemonInfo, Doc, EntityActionsResponse, EntityType, Issue, Project, PullRequest,
};
use anyhow::Result;
use async_trait::async_trait;

use super::client::{proto, OpenInTerminalResult, OpenInVscodeResult};

/// Trait for daemon client operations, enabling mocking in tests
#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DaemonClientTrait: Send + Sync {
    /// Check if the daemon is reachable
    async fn check_connection(&self) -> bool;

    /// List all tracked projects
    async fn list_projects(&mut self) -> Result<Vec<Project>>;

    /// List issues for a project
    async fn list_issues(&mut self, project_path: &str) -> Result<Vec<Issue>>;

    /// List PRs for a project
    async fn list_prs(&mut self, project_path: &str) -> Result<Vec<PullRequest>>;

    /// List docs for a project
    async fn list_docs(&mut self, project_path: &str) -> Result<Vec<Doc>>;

    /// Get project configuration
    async fn get_config(&mut self, project_path: &str) -> Result<Config>;

    /// Get daemon information
    async fn get_daemon_info(&mut self) -> Result<DaemonInfo>;

    /// Set project favorite status
    async fn set_project_favorite(&mut self, project_path: &str, is_favorite: bool) -> Result<()>;

    /// Set project archived status
    async fn set_project_archived(&mut self, project_path: &str, is_archived: bool) -> Result<()>;

    /// Untrack a project
    async fn untrack_project(&mut self, project_path: &str) -> Result<()>;

    /// Create a new issue
    async fn create_issue(
        &mut self,
        project_path: &str,
        title: &str,
        description: &str,
        priority: u32,
        draft: bool,
    ) -> Result<String>;

    /// Update an existing issue
    async fn update_issue(
        &mut self,
        project_path: &str,
        issue_id: &str,
        title: &str,
        description: &str,
        priority: u32,
        status: &str,
    ) -> Result<()>;

    /// Delete an issue
    async fn delete_issue(&mut self, project_path: &str, issue_id: &str) -> Result<()>;

    /// Create a new PR
    async fn create_pr(
        &mut self,
        project_path: &str,
        title: &str,
        description: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String>;

    /// Update an existing PR
    #[allow(clippy::too_many_arguments)]
    async fn update_pr(
        &mut self,
        project_path: &str,
        pr_id: &str,
        title: &str,
        description: &str,
        source_branch: &str,
        target_branch: &str,
        status: &str,
    ) -> Result<()>;

    /// Create a new doc
    /// Returns (slug, sync_results) where sync_results contains org sync status
    async fn create_doc(
        &mut self,
        project_path: &str,
        title: &str,
        content: &str,
        slug: Option<String>,
        is_org_doc: bool,
    ) -> Result<(String, Vec<proto::OrgDocSyncResult>)>;

    /// Open a project in a temporary VS Code workspace
    async fn open_in_temp_vscode(
        &mut self,
        project_path: &str,
        issue_id: &str,
        action: i32,
        agent_name: &str,
        ttl_hours: u32,
    ) -> Result<OpenInVscodeResult>;

    /// Open an agent in a terminal for working on an issue
    async fn open_agent_in_terminal(
        &mut self,
        project_path: &str,
        issue_id: &str,
        agent_name: &str,
        workspace_mode: i32,
        ttl_hours: u32,
    ) -> Result<OpenInTerminalResult>;

    /// Restart the daemon
    async fn restart(&mut self) -> Result<()>;

    /// Shutdown the daemon
    async fn shutdown(&mut self) -> Result<()>;

    /// Get available actions for an entity
    async fn get_entity_actions(
        &mut self,
        project_path: &str,
        entity_type: EntityType,
        entity_id: Option<String>,
    ) -> Result<EntityActionsResponse>;
}
