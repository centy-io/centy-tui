//! gRPC client for communicating with the Centy daemon
//!
//! This module provides a client for communicating with the Centy daemon
//! via gRPC using the generated proto types.

use crate::state::{
    ActionCategory, Config, DaemonInfo, Doc, EntityAction, EntityActionsResponse, EntityType,
    Issue, IssueMetadata, Organization, PrMetadata, Project, PullRequest, User,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Include the generated proto types
pub mod proto {
    tonic::include_proto!("centy");
}

use proto::centy_daemon_client::CentyDaemonClient;

/// Default daemon address
const DEFAULT_ADDRESS: &str = "http://127.0.0.1:50051";

/// Client for communicating with the Centy daemon
pub struct DaemonClient {
    /// The gRPC client
    client: Option<CentyDaemonClient<tonic::transport::Channel>>,
    /// The daemon address
    address: String,
}

/// Result from opening a project in a temporary VS Code workspace
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OpenInVscodeResult {
    pub workspace_path: String,
    pub issue_id: String,
    pub display_number: u32,
    pub expires_at: String,
    pub vscode_opened: bool,
}

/// Result from opening an agent in a terminal
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OpenInTerminalResult {
    pub working_directory: String,
    pub issue_id: String,
    pub display_number: u32,
    pub agent_command: String,
    pub terminal_opened: bool,
    pub expires_at: String,
}

/// A temporary workspace entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TempWorkspace {
    pub workspace_path: String,
    pub source_project_path: String,
    pub issue_id: String,
    pub issue_display_number: u32,
    pub issue_title: String,
    pub agent_name: String,
    pub action: i32,
    pub created_at: String,
    pub expires_at: String,
}

impl DaemonClient {
    /// Create a new daemon client
    pub async fn new() -> Result<Self> {
        let address =
            std::env::var("CENTY_DAEMON_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_string());

        // Try to connect to the daemon
        let client = CentyDaemonClient::connect(address.clone()).await.ok();

        Ok(Self { client, address })
    }

    /// Ensure connection is established
    async fn ensure_connected(
        &mut self,
    ) -> Result<&mut CentyDaemonClient<tonic::transport::Channel>> {
        if self.client.is_none() {
            self.client = Some(
                CentyDaemonClient::connect(self.address.clone())
                    .await
                    .map_err(|e| anyhow!("Failed to connect to daemon: {}", e))?,
            );
        }
        self.client
            .as_mut()
            .ok_or_else(|| anyhow!("Client not connected"))
    }

    /// Check if the daemon is reachable
    pub async fn check_connection(&self) -> bool {
        self.client.is_some()
    }

    /// List all tracked projects
    pub async fn list_projects(&mut self) -> Result<Vec<Project>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListProjectsRequest {
            include_stale: false,
            include_uninitialized: false,
            include_archived: false,
            organization_slug: String::new(),
            ungrouped_only: false,
            include_temp: false,
        });

        let response = client
            .list_projects(request)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        let projects = response
            .into_inner()
            .projects
            .into_iter()
            .map(|p| Project {
                path: p.path,
                name: p.name,
                project_title: if p.project_title.is_empty() {
                    None
                } else {
                    Some(p.project_title)
                },
                user_title: if p.user_title.is_empty() {
                    None
                } else {
                    Some(p.user_title)
                },
                is_favorite: p.is_favorite,
                is_archived: p.is_archived,
                initialized: p.initialized,
                issue_count: p.issue_count,
                doc_count: p.doc_count,
                pr_count: 0, // PR count not in proto, will need to be added or fetched separately
                organization_slug: if p.organization_slug.is_empty() {
                    None
                } else {
                    Some(p.organization_slug)
                },
                organization_name: if p.organization_name.is_empty() {
                    None
                } else {
                    Some(p.organization_name)
                },
            })
            .collect();

        Ok(projects)
    }

    /// List issues for a project
    pub async fn list_issues(&mut self, project_path: &str) -> Result<Vec<Issue>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListIssuesRequest {
            project_path: project_path.to_string(),
            status: String::new(),
            priority: 0,
            draft: None,
        });

        let response = client
            .list_issues(request)
            .await
            .map_err(|e| anyhow!("Failed to list issues: {}", e))?;

        let issues = response
            .into_inner()
            .issues
            .into_iter()
            .map(|i| {
                let metadata = i.metadata.unwrap_or_default();
                Issue {
                    id: i.id,
                    display_number: i.display_number,
                    title: i.title,
                    description: i.description,
                    metadata: IssueMetadata {
                        status: metadata.status,
                        priority: metadata.priority as u32,
                        priority_label: if metadata.priority_label.is_empty() {
                            None
                        } else {
                            Some(metadata.priority_label)
                        },
                        created_at: parse_timestamp(&metadata.created_at),
                        updated_at: parse_timestamp(&metadata.updated_at),
                        custom_fields: metadata.custom_fields,
                    },
                }
            })
            .collect();

        Ok(issues)
    }

    /// List PRs for a project
    pub async fn list_prs(&mut self, project_path: &str) -> Result<Vec<PullRequest>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListPrsRequest {
            project_path: project_path.to_string(),
            status: String::new(),
            source_branch: String::new(),
            target_branch: String::new(),
            priority: 0,
        });

        let response = client
            .list_prs(request)
            .await
            .map_err(|e| anyhow!("Failed to list PRs: {}", e))?;

        let prs = response
            .into_inner()
            .prs
            .into_iter()
            .map(|pr| {
                let metadata = pr.metadata.unwrap_or_default();
                PullRequest {
                    id: pr.id,
                    display_number: pr.display_number,
                    title: pr.title,
                    description: pr.description,
                    metadata: PrMetadata {
                        status: metadata.status,
                        priority: metadata.priority as u32,
                        priority_label: if metadata.priority_label.is_empty() {
                            None
                        } else {
                            Some(metadata.priority_label)
                        },
                        source_branch: metadata.source_branch,
                        target_branch: metadata.target_branch,
                        linked_issues: Vec::new(), // Links are now in separate system
                        reviewers: metadata.reviewers,
                        created_at: parse_timestamp(&metadata.created_at),
                        updated_at: parse_timestamp(&metadata.updated_at),
                        merged_at: if metadata.merged_at.is_empty() {
                            None
                        } else {
                            Some(parse_timestamp(&metadata.merged_at))
                        },
                        closed_at: if metadata.closed_at.is_empty() {
                            None
                        } else {
                            Some(parse_timestamp(&metadata.closed_at))
                        },
                        custom_fields: metadata.custom_fields,
                    },
                }
            })
            .collect();

        Ok(prs)
    }

    /// List docs for a project
    pub async fn list_docs(&mut self, project_path: &str) -> Result<Vec<Doc>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListDocsRequest {
            project_path: project_path.to_string(),
        });

        let response = client
            .list_docs(request)
            .await
            .map_err(|e| anyhow!("Failed to list docs: {}", e))?;

        let docs = response
            .into_inner()
            .docs
            .into_iter()
            .map(|d| {
                let metadata = d.metadata.unwrap_or_default();
                Doc {
                    slug: d.slug,
                    title: d.title,
                    content: d.content,
                    created_at: parse_timestamp(&metadata.created_at),
                    updated_at: parse_timestamp(&metadata.updated_at),
                }
            })
            .collect();

        Ok(docs)
    }

    /// Get project configuration
    #[allow(dead_code)]
    pub async fn get_config(&mut self, project_path: &str) -> Result<Config> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::GetConfigRequest {
            project_path: project_path.to_string(),
        });

        let response = client
            .get_config(request)
            .await
            .map_err(|e| anyhow!("Failed to get config: {}", e))?;

        let config = response.into_inner();
        Ok(Config {
            priority_levels: config.priority_levels as u32,
            allowed_states: config.allowed_states,
            default_state: config.default_state,
            version: config.version,
        })
    }

    /// Get daemon information
    #[allow(dead_code)]
    pub async fn get_daemon_info(&mut self) -> Result<DaemonInfo> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::GetDaemonInfoRequest {});

        let response = client
            .get_daemon_info(request)
            .await
            .map_err(|e| anyhow!("Failed to get daemon info: {}", e))?;

        let info = response.into_inner();
        Ok(DaemonInfo {
            version: info.version,
            uptime_seconds: 0, // Not provided by proto
            project_count: 0,  // Need to fetch separately
        })
    }

    /// Set project favorite status
    pub async fn set_project_favorite(
        &mut self,
        project_path: &str,
        is_favorite: bool,
    ) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::SetProjectFavoriteRequest {
            project_path: project_path.to_string(),
            is_favorite,
        });

        client
            .set_project_favorite(request)
            .await
            .map_err(|e| anyhow!("Failed to set project favorite: {}", e))?;

        Ok(())
    }

    /// Set project archived status
    pub async fn set_project_archived(
        &mut self,
        project_path: &str,
        is_archived: bool,
    ) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::SetProjectArchivedRequest {
            project_path: project_path.to_string(),
            is_archived,
        });

        client
            .set_project_archived(request)
            .await
            .map_err(|e| anyhow!("Failed to set project archived: {}", e))?;

        Ok(())
    }

    /// Untrack a project
    #[allow(dead_code)]
    pub async fn untrack_project(&mut self, project_path: &str) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::UntrackProjectRequest {
            project_path: project_path.to_string(),
        });

        client
            .untrack_project(request)
            .await
            .map_err(|e| anyhow!("Failed to untrack project: {}", e))?;

        Ok(())
    }

    /// Create a new issue
    pub async fn create_issue(
        &mut self,
        project_path: &str,
        title: &str,
        description: &str,
        priority: u32,
        draft: bool,
    ) -> Result<String> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::CreateIssueRequest {
            project_path: project_path.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            priority: priority as i32,
            status: String::new(),
            custom_fields: HashMap::new(),
            template: String::new(),
            draft,
        });

        let response = client
            .create_issue(request)
            .await
            .map_err(|e| anyhow!("Failed to create issue: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to create issue: {}", inner.error));
        }

        Ok(inner.id)
    }

    /// Update an existing issue
    pub async fn update_issue(
        &mut self,
        project_path: &str,
        issue_id: &str,
        title: &str,
        description: &str,
        priority: u32,
        status: &str,
    ) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::UpdateIssueRequest {
            project_path: project_path.to_string(),
            issue_id: issue_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: status.to_string(),
            priority: priority as i32,
            custom_fields: HashMap::new(),
            draft: None,
        });

        let response = client
            .update_issue(request)
            .await
            .map_err(|e| anyhow!("Failed to update issue: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to update issue: {}", inner.error));
        }

        Ok(())
    }

    /// Delete an issue
    pub async fn delete_issue(&mut self, project_path: &str, issue_id: &str) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::DeleteIssueRequest {
            project_path: project_path.to_string(),
            issue_id: issue_id.to_string(),
        });

        let response = client
            .delete_issue(request)
            .await
            .map_err(|e| anyhow!("Failed to delete issue: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to delete issue: {}", inner.error));
        }

        Ok(())
    }

    /// Move an issue to a different project
    pub async fn move_issue(
        &mut self,
        source_project_path: &str,
        issue_id: &str,
        target_project_path: &str,
    ) -> Result<(Issue, u32)> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::MoveIssueRequest {
            source_project_path: source_project_path.to_string(),
            issue_id: issue_id.to_string(),
            target_project_path: target_project_path.to_string(),
        });

        let response = client
            .move_issue(request)
            .await
            .map_err(|e| anyhow!("Failed to move issue: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("{}", inner.error));
        }

        let proto_issue = inner.issue.unwrap_or_default();
        let metadata = proto_issue.metadata.unwrap_or_default();
        let issue = Issue {
            id: proto_issue.id,
            display_number: proto_issue.display_number,
            title: proto_issue.title,
            description: proto_issue.description,
            metadata: IssueMetadata {
                status: metadata.status,
                priority: metadata.priority as u32,
                priority_label: if metadata.priority_label.is_empty() {
                    None
                } else {
                    Some(metadata.priority_label)
                },
                created_at: parse_timestamp(&metadata.created_at),
                updated_at: parse_timestamp(&metadata.updated_at),
                custom_fields: metadata.custom_fields,
            },
        };

        Ok((issue, inner.old_display_number))
    }

    /// Move a doc to a different project
    pub async fn move_doc(
        &mut self,
        source_project_path: &str,
        slug: &str,
        target_project_path: &str,
        new_slug: Option<&str>,
    ) -> Result<(Doc, String)> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::MoveDocRequest {
            source_project_path: source_project_path.to_string(),
            slug: slug.to_string(),
            target_project_path: target_project_path.to_string(),
            new_slug: new_slug.unwrap_or("").to_string(),
        });

        let response = client
            .move_doc(request)
            .await
            .map_err(|e| anyhow!("Failed to move doc: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("{}", inner.error));
        }

        let proto_doc = inner.doc.unwrap_or_default();
        let metadata = proto_doc.metadata.unwrap_or_default();
        let doc = Doc {
            slug: proto_doc.slug,
            title: proto_doc.title,
            content: proto_doc.content,
            created_at: parse_timestamp(&metadata.created_at),
            updated_at: parse_timestamp(&metadata.updated_at),
        };

        Ok((doc, inner.old_slug))
    }

    /// Create a new PR
    pub async fn create_pr(
        &mut self,
        project_path: &str,
        title: &str,
        description: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::CreatePrRequest {
            project_path: project_path.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            reviewers: Vec::new(),
            priority: 0,
            status: String::new(),
            custom_fields: HashMap::new(),
            template: String::new(),
        });

        let response = client
            .create_pr(request)
            .await
            .map_err(|e| anyhow!("Failed to create PR: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to create PR: {}", inner.error));
        }

        Ok(inner.id)
    }

    /// Update an existing PR
    #[allow(clippy::too_many_arguments)]
    pub async fn update_pr(
        &mut self,
        project_path: &str,
        pr_id: &str,
        title: &str,
        description: &str,
        source_branch: &str,
        target_branch: &str,
        status: &str,
    ) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::UpdatePrRequest {
            project_path: project_path.to_string(),
            pr_id: pr_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: status.to_string(),
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            reviewers: Vec::new(),
            priority: 0,
            custom_fields: HashMap::new(),
        });

        let response = client
            .update_pr(request)
            .await
            .map_err(|e| anyhow!("Failed to update PR: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to update PR: {}", inner.error));
        }

        Ok(())
    }

    /// Create a new doc
    /// Returns (slug, sync_results) where sync_results contains org sync status
    pub async fn create_doc(
        &mut self,
        project_path: &str,
        title: &str,
        content: &str,
        slug: Option<&str>,
        is_org_doc: bool,
    ) -> Result<(String, Vec<proto::OrgDocSyncResult>)> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::CreateDocRequest {
            project_path: project_path.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            slug: slug.unwrap_or("").to_string(),
            template: String::new(),
            is_org_doc,
        });

        let response = client
            .create_doc(request)
            .await
            .map_err(|e| anyhow!("Failed to create doc: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to create doc: {}", inner.error));
        }

        Ok((inner.slug, inner.sync_results))
    }

    /// Update an existing doc
    pub async fn update_doc(
        &mut self,
        project_path: &str,
        slug: &str,
        title: &str,
        content: &str,
        new_slug: Option<&str>,
    ) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::UpdateDocRequest {
            project_path: project_path.to_string(),
            slug: slug.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            new_slug: new_slug.unwrap_or("").to_string(),
        });

        let response = client
            .update_doc(request)
            .await
            .map_err(|e| anyhow!("Failed to update doc: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to update doc: {}", inner.error));
        }

        Ok(())
    }

    /// Open a project in a temporary VS Code workspace
    pub async fn open_in_temp_vscode(
        &mut self,
        project_path: &str,
        issue_id: &str,
        action: i32,
        agent_name: &str,
        ttl_hours: u32,
    ) -> Result<OpenInVscodeResult> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::OpenInTempVscodeRequest {
            project_path: project_path.to_string(),
            issue_id: issue_id.to_string(),
            action,
            agent_name: agent_name.to_string(),
            ttl_hours,
        });

        let response = client
            .open_in_temp_vscode(request)
            .await
            .map_err(|e| anyhow!("Connection error: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("{}", inner.error));
        }

        Ok(OpenInVscodeResult {
            workspace_path: inner.workspace_path,
            issue_id: inner.issue_id,
            display_number: inner.display_number,
            expires_at: inner.expires_at,
            vscode_opened: inner.vscode_opened,
        })
    }

    /// Open an agent in a terminal for working on an issue
    pub async fn open_agent_in_terminal(
        &mut self,
        project_path: &str,
        issue_id: &str,
        agent_name: &str,
        workspace_mode: i32,
        ttl_hours: u32,
    ) -> Result<OpenInTerminalResult> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::OpenAgentInTerminalRequest {
            project_path: project_path.to_string(),
            issue_id: issue_id.to_string(),
            agent_name: agent_name.to_string(),
            workspace_mode,
            ttl_hours,
        });

        let response = client
            .open_agent_in_terminal(request)
            .await
            .map_err(|e| anyhow!("Failed to open agent in terminal: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to open agent in terminal: {}", inner.error));
        }

        Ok(OpenInTerminalResult {
            working_directory: inner.working_directory,
            issue_id: inner.issue_id,
            display_number: inner.display_number,
            agent_command: inner.agent_command,
            terminal_opened: inner.terminal_opened,
            expires_at: inner.expires_at,
        })
    }

    /// List temporary workspaces, optionally filtered by project path
    pub async fn list_temp_workspaces(&mut self, project_path: &str) -> Result<Vec<TempWorkspace>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListTempWorkspacesRequest {
            include_expired: false,
            source_project_path: project_path.to_string(),
        });

        let response = client
            .list_temp_workspaces(request)
            .await
            .map_err(|e| anyhow!("Failed to list temp workspaces: {}", e))?;

        let workspaces = response
            .into_inner()
            .workspaces
            .into_iter()
            .map(|w| TempWorkspace {
                workspace_path: w.workspace_path,
                source_project_path: w.source_project_path,
                issue_id: w.issue_id,
                issue_display_number: w.issue_display_number,
                issue_title: w.issue_title,
                agent_name: w.agent_name,
                action: w.action,
                created_at: w.created_at,
                expires_at: w.expires_at,
            })
            .collect();

        Ok(workspaces)
    }

    /// Close a temporary workspace
    pub async fn close_temp_workspace(&mut self, workspace_path: &str, force: bool) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::CloseTempWorkspaceRequest {
            workspace_path: workspace_path.to_string(),
            force,
        });

        let response = client
            .close_temp_workspace(request)
            .await
            .map_err(|e| anyhow!("Failed to close temp workspace: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to close temp workspace: {}", inner.error));
        }

        Ok(())
    }

    /// Restart the daemon
    #[allow(dead_code)]
    pub async fn restart(&mut self) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::RestartRequest { delay_seconds: 0 });

        client
            .restart(request)
            .await
            .map_err(|e| anyhow!("Failed to restart daemon: {}", e))?;

        // Clear the client so we reconnect on next call
        self.client = None;

        Ok(())
    }

    /// Shutdown the daemon
    #[allow(dead_code)]
    pub async fn shutdown(&mut self) -> Result<()> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ShutdownRequest { delay_seconds: 0 });

        client
            .shutdown(request)
            .await
            .map_err(|e| anyhow!("Failed to shutdown daemon: {}", e))?;

        // Clear the client since daemon is shutting down
        self.client = None;

        Ok(())
    }

    /// Get available actions for an entity
    pub async fn get_entity_actions(
        &mut self,
        project_path: &str,
        entity_type: EntityType,
        entity_id: Option<&str>,
    ) -> Result<EntityActionsResponse> {
        let client = self.ensure_connected().await?;

        let proto_entity_type = match entity_type {
            EntityType::Issue => proto::EntityType::Issue as i32,
            EntityType::Pr => proto::EntityType::Pr as i32,
            EntityType::Doc => proto::EntityType::Doc as i32,
        };

        let request = tonic::Request::new(proto::GetEntityActionsRequest {
            project_path: project_path.to_string(),
            entity_type: proto_entity_type,
            entity_id: entity_id.unwrap_or("").to_string(),
        });

        let response = client
            .get_entity_actions(request)
            .await
            .map_err(|e| anyhow!("Failed to get entity actions: {}", e))?;

        let inner = response.into_inner();

        if !inner.success {
            return Err(anyhow!("Failed to get entity actions: {}", inner.error));
        }

        let actions = inner
            .actions
            .into_iter()
            .map(|a| EntityAction {
                id: a.id,
                label: a.label,
                category: ActionCategory::from_proto(a.category),
                enabled: a.enabled,
                disabled_reason: a.disabled_reason,
                destructive: a.destructive,
                keyboard_shortcut: a.keyboard_shortcut,
            })
            .collect();

        Ok(EntityActionsResponse { actions })
    }

    /// Get an organization by slug
    pub async fn get_organization(&mut self, slug: &str) -> Result<Option<Organization>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::GetOrganizationRequest {
            slug: slug.to_string(),
        });

        let response = client
            .get_organization(request)
            .await
            .map_err(|e| anyhow!("Failed to get organization: {}", e))?;

        let inner = response.into_inner();
        if !inner.found {
            return Ok(None);
        }

        let org = inner.organization.map(|o| Organization {
            slug: o.slug,
            name: o.name,
            description: o.description,
            created_at: parse_timestamp(&o.created_at),
            updated_at: parse_timestamp(&o.updated_at),
            project_count: o.project_count,
        });

        Ok(org)
    }

    /// List users for a project
    pub async fn list_users(&mut self, project_path: &str) -> Result<Vec<User>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListUsersRequest {
            project_path: project_path.to_string(),
            git_username: String::new(),
        });

        let response = client
            .list_users(request)
            .await
            .map_err(|e| anyhow!("Failed to list users: {}", e))?;

        let users = response
            .into_inner()
            .users
            .into_iter()
            .map(|u| User {
                id: u.id,
                name: u.name,
                email: u.email,
                git_usernames: u.git_usernames,
            })
            .collect();

        Ok(users)
    }

    /// List projects for a specific organization
    pub async fn list_projects_by_organization(&mut self, org_slug: &str) -> Result<Vec<Project>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::ListProjectsRequest {
            include_stale: false,
            include_uninitialized: false,
            include_archived: false,
            organization_slug: org_slug.to_string(),
            ungrouped_only: false,
            include_temp: false,
        });

        let response = client
            .list_projects(request)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        let projects = response
            .into_inner()
            .projects
            .into_iter()
            .map(|p| Project {
                path: p.path,
                name: p.name,
                project_title: if p.project_title.is_empty() {
                    None
                } else {
                    Some(p.project_title)
                },
                user_title: if p.user_title.is_empty() {
                    None
                } else {
                    Some(p.user_title)
                },
                is_favorite: p.is_favorite,
                is_archived: p.is_archived,
                initialized: p.initialized,
                issue_count: p.issue_count,
                doc_count: p.doc_count,
                pr_count: 0,
                organization_slug: if p.organization_slug.is_empty() {
                    None
                } else {
                    Some(p.organization_slug)
                },
                organization_name: if p.organization_name.is_empty() {
                    None
                } else {
                    Some(p.organization_name)
                },
            })
            .collect();

        Ok(projects)
    }

    // ============ Cross-Project Search Methods ============

    /// Search issues by UUID across all projects
    pub async fn search_issues_by_uuid(
        &mut self,
        uuid: &str,
    ) -> Result<Vec<(Issue, String, String)>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::GetIssuesByUuidRequest {
            uuid: uuid.to_string(),
        });

        let response = client
            .get_issues_by_uuid(request)
            .await
            .map_err(|e| anyhow!("Failed to search issues: {}", e))?;

        let inner = response.into_inner();

        let results = inner
            .issues
            .into_iter()
            .map(|iwp| {
                let proto_issue = iwp.issue.unwrap_or_default();
                let metadata = proto_issue.metadata.unwrap_or_default();
                let issue = Issue {
                    id: proto_issue.id,
                    display_number: proto_issue.display_number,
                    title: proto_issue.title,
                    description: proto_issue.description,
                    metadata: IssueMetadata {
                        status: metadata.status,
                        priority: metadata.priority as u32,
                        priority_label: if metadata.priority_label.is_empty() {
                            None
                        } else {
                            Some(metadata.priority_label)
                        },
                        created_at: parse_timestamp(&metadata.created_at),
                        updated_at: parse_timestamp(&metadata.updated_at),
                        custom_fields: metadata.custom_fields,
                    },
                };
                (issue, iwp.project_path, iwp.project_name)
            })
            .collect();

        Ok(results)
    }

    /// Search docs by slug across all projects
    pub async fn search_docs_by_slug(&mut self, slug: &str) -> Result<Vec<(Doc, String, String)>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::GetDocsBySlugRequest {
            slug: slug.to_string(),
        });

        let response = client
            .get_docs_by_slug(request)
            .await
            .map_err(|e| anyhow!("Failed to search docs: {}", e))?;

        let inner = response.into_inner();

        let results = inner
            .docs
            .into_iter()
            .map(|dwp| {
                let proto_doc = dwp.doc.unwrap_or_default();
                let metadata = proto_doc.metadata.unwrap_or_default();
                let doc = Doc {
                    slug: proto_doc.slug,
                    title: proto_doc.title,
                    content: proto_doc.content,
                    created_at: parse_timestamp(&metadata.created_at),
                    updated_at: parse_timestamp(&metadata.updated_at),
                };
                (doc, dwp.project_path, dwp.project_name)
            })
            .collect();

        Ok(results)
    }

    /// Search PRs by UUID across all projects
    pub async fn search_prs_by_uuid(
        &mut self,
        uuid: &str,
    ) -> Result<Vec<(PullRequest, String, String)>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::GetPrsByUuidRequest {
            uuid: uuid.to_string(),
        });

        let response = client
            .get_prs_by_uuid(request)
            .await
            .map_err(|e| anyhow!("Failed to search PRs: {}", e))?;

        let inner = response.into_inner();

        let results = inner
            .prs
            .into_iter()
            .map(|pwp| {
                let proto_pr = pwp.pr.unwrap_or_default();
                let metadata = proto_pr.metadata.unwrap_or_default();
                let pr = PullRequest {
                    id: proto_pr.id,
                    display_number: proto_pr.display_number,
                    title: proto_pr.title,
                    description: proto_pr.description,
                    metadata: PrMetadata {
                        status: metadata.status,
                        priority: metadata.priority as u32,
                        priority_label: if metadata.priority_label.is_empty() {
                            None
                        } else {
                            Some(metadata.priority_label)
                        },
                        source_branch: metadata.source_branch,
                        target_branch: metadata.target_branch,
                        linked_issues: Vec::new(),
                        reviewers: metadata.reviewers,
                        created_at: parse_timestamp(&metadata.created_at),
                        updated_at: parse_timestamp(&metadata.updated_at),
                        merged_at: if metadata.merged_at.is_empty() {
                            None
                        } else {
                            Some(parse_timestamp(&metadata.merged_at))
                        },
                        closed_at: if metadata.closed_at.is_empty() {
                            None
                        } else {
                            Some(parse_timestamp(&metadata.closed_at))
                        },
                        custom_fields: metadata.custom_fields,
                    },
                };
                (pr, pwp.project_path, pwp.project_name)
            })
            .collect();

        Ok(results)
    }

    /// Advanced search across all projects
    pub async fn advanced_search(&mut self, query: &str) -> Result<Vec<(Issue, String, String)>> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::AdvancedSearchRequest {
            query: query.to_string(),
            sort_by: "priority".to_string(),
            sort_descending: false,
            multi_project: true,
            project_path: String::new(),
        });

        let response = client
            .advanced_search(request)
            .await
            .map_err(|e| anyhow!("Failed to search: {}", e))?;

        let inner = response.into_inner();

        if !inner.success {
            return Err(anyhow!("{}", inner.error));
        }

        let results = inner
            .results
            .into_iter()
            .map(|sri| {
                let proto_issue = sri.issue.unwrap_or_default();
                let metadata = proto_issue.metadata.unwrap_or_default();
                let issue = Issue {
                    id: proto_issue.id,
                    display_number: proto_issue.display_number,
                    title: proto_issue.title,
                    description: proto_issue.description,
                    metadata: IssueMetadata {
                        status: metadata.status,
                        priority: metadata.priority as u32,
                        priority_label: if metadata.priority_label.is_empty() {
                            None
                        } else {
                            Some(metadata.priority_label)
                        },
                        created_at: parse_timestamp(&metadata.created_at),
                        updated_at: parse_timestamp(&metadata.updated_at),
                        custom_fields: metadata.custom_fields,
                    },
                };
                (issue, sri.project_path, sri.project_name)
            })
            .collect();

        Ok(results)
    }

    // ============ Organization Aggregation Methods ============

    /// List issues from all projects in an organization
    pub async fn list_issues_by_organization(
        &mut self,
        org_slug: &str,
    ) -> Result<Vec<(Issue, String, String)>> {
        // First get all projects in the organization
        let projects = self.list_projects_by_organization(org_slug).await?;

        let mut all_issues = Vec::new();

        // Fetch issues from each project
        for project in &projects {
            if let Ok(issues) = self.list_issues(&project.path).await {
                let project_name = project.display_name().to_string();
                for issue in issues {
                    all_issues.push((issue, project.path.clone(), project_name.clone()));
                }
            }
        }

        Ok(all_issues)
    }

    /// List PRs from all projects in an organization
    pub async fn list_prs_by_organization(
        &mut self,
        org_slug: &str,
    ) -> Result<Vec<(PullRequest, String, String)>> {
        let projects = self.list_projects_by_organization(org_slug).await?;

        let mut all_prs = Vec::new();

        for project in &projects {
            if let Ok(prs) = self.list_prs(&project.path).await {
                let project_name = project.display_name().to_string();
                for pr in prs {
                    all_prs.push((pr, project.path.clone(), project_name.clone()));
                }
            }
        }

        Ok(all_prs)
    }

    /// List docs from all projects in an organization
    pub async fn list_docs_by_organization(
        &mut self,
        org_slug: &str,
    ) -> Result<Vec<(Doc, String, String)>> {
        let projects = self.list_projects_by_organization(org_slug).await?;

        let mut all_docs = Vec::new();

        for project in &projects {
            if let Ok(docs) = self.list_docs(&project.path).await {
                let project_name = project.display_name().to_string();
                for doc in docs {
                    all_docs.push((doc, project.path.clone(), project_name.clone()));
                }
            }
        }

        Ok(all_docs)
    }
}

/// Parse an ISO timestamp string to DateTime<Utc>
fn parse_timestamp(s: &str) -> DateTime<Utc> {
    if s.is_empty() {
        return Utc::now();
    }
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// Implement the trait for DaemonClient
use super::traits::DaemonClientTrait;
use async_trait::async_trait;

#[async_trait]
impl DaemonClientTrait for DaemonClient {
    async fn check_connection(&self) -> bool {
        DaemonClient::check_connection(self).await
    }

    async fn list_projects(&mut self) -> Result<Vec<Project>> {
        DaemonClient::list_projects(self).await
    }

    async fn list_issues(&mut self, project_path: &str) -> Result<Vec<Issue>> {
        DaemonClient::list_issues(self, project_path).await
    }

    async fn list_prs(&mut self, project_path: &str) -> Result<Vec<PullRequest>> {
        DaemonClient::list_prs(self, project_path).await
    }

    async fn list_docs(&mut self, project_path: &str) -> Result<Vec<Doc>> {
        DaemonClient::list_docs(self, project_path).await
    }

    async fn get_config(&mut self, project_path: &str) -> Result<Config> {
        DaemonClient::get_config(self, project_path).await
    }

    async fn get_daemon_info(&mut self) -> Result<DaemonInfo> {
        DaemonClient::get_daemon_info(self).await
    }

    async fn set_project_favorite(&mut self, project_path: &str, is_favorite: bool) -> Result<()> {
        DaemonClient::set_project_favorite(self, project_path, is_favorite).await
    }

    async fn set_project_archived(&mut self, project_path: &str, is_archived: bool) -> Result<()> {
        DaemonClient::set_project_archived(self, project_path, is_archived).await
    }

    async fn untrack_project(&mut self, project_path: &str) -> Result<()> {
        DaemonClient::untrack_project(self, project_path).await
    }

    async fn create_issue(
        &mut self,
        project_path: &str,
        title: &str,
        description: &str,
        priority: u32,
        draft: bool,
    ) -> Result<String> {
        DaemonClient::create_issue(self, project_path, title, description, priority, draft).await
    }

    async fn update_issue(
        &mut self,
        project_path: &str,
        issue_id: &str,
        title: &str,
        description: &str,
        priority: u32,
        status: &str,
    ) -> Result<()> {
        DaemonClient::update_issue(
            self,
            project_path,
            issue_id,
            title,
            description,
            priority,
            status,
        )
        .await
    }

    async fn delete_issue(&mut self, project_path: &str, issue_id: &str) -> Result<()> {
        DaemonClient::delete_issue(self, project_path, issue_id).await
    }

    async fn move_issue(
        &mut self,
        source_project_path: &str,
        issue_id: &str,
        target_project_path: &str,
    ) -> Result<(Issue, u32)> {
        DaemonClient::move_issue(self, source_project_path, issue_id, target_project_path).await
    }

    async fn move_doc(
        &mut self,
        source_project_path: &str,
        slug: &str,
        target_project_path: &str,
        new_slug: Option<String>,
    ) -> Result<(Doc, String)> {
        DaemonClient::move_doc(
            self,
            source_project_path,
            slug,
            target_project_path,
            new_slug.as_deref(),
        )
        .await
    }

    async fn create_pr(
        &mut self,
        project_path: &str,
        title: &str,
        description: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<String> {
        DaemonClient::create_pr(
            self,
            project_path,
            title,
            description,
            source_branch,
            target_branch,
        )
        .await
    }

    async fn update_pr(
        &mut self,
        project_path: &str,
        pr_id: &str,
        title: &str,
        description: &str,
        source_branch: &str,
        target_branch: &str,
        status: &str,
    ) -> Result<()> {
        DaemonClient::update_pr(
            self,
            project_path,
            pr_id,
            title,
            description,
            source_branch,
            target_branch,
            status,
        )
        .await
    }

    async fn create_doc(
        &mut self,
        project_path: &str,
        title: &str,
        content: &str,
        slug: Option<String>,
        is_org_doc: bool,
    ) -> Result<(String, Vec<proto::OrgDocSyncResult>)> {
        DaemonClient::create_doc(
            self,
            project_path,
            title,
            content,
            slug.as_deref(),
            is_org_doc,
        )
        .await
    }

    async fn open_in_temp_vscode(
        &mut self,
        project_path: &str,
        issue_id: &str,
        action: i32,
        agent_name: &str,
        ttl_hours: u32,
    ) -> Result<OpenInVscodeResult> {
        DaemonClient::open_in_temp_vscode(
            self,
            project_path,
            issue_id,
            action,
            agent_name,
            ttl_hours,
        )
        .await
    }

    async fn open_agent_in_terminal(
        &mut self,
        project_path: &str,
        issue_id: &str,
        agent_name: &str,
        workspace_mode: i32,
        ttl_hours: u32,
    ) -> Result<OpenInTerminalResult> {
        DaemonClient::open_agent_in_terminal(
            self,
            project_path,
            issue_id,
            agent_name,
            workspace_mode,
            ttl_hours,
        )
        .await
    }

    async fn list_temp_workspaces(&mut self, project_path: &str) -> Result<Vec<TempWorkspace>> {
        DaemonClient::list_temp_workspaces(self, project_path).await
    }

    async fn close_temp_workspace(&mut self, workspace_path: &str, force: bool) -> Result<()> {
        DaemonClient::close_temp_workspace(self, workspace_path, force).await
    }

    async fn restart(&mut self) -> Result<()> {
        DaemonClient::restart(self).await
    }

    async fn shutdown(&mut self) -> Result<()> {
        DaemonClient::shutdown(self).await
    }

    async fn get_entity_actions(
        &mut self,
        project_path: &str,
        entity_type: EntityType,
        entity_id: Option<String>,
    ) -> Result<EntityActionsResponse> {
        DaemonClient::get_entity_actions(self, project_path, entity_type, entity_id.as_deref())
            .await
    }
}
