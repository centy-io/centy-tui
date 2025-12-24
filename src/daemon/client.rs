//! gRPC client for communicating with the Centy daemon
//!
//! This module provides a client for communicating with the Centy daemon
//! via gRPC using the generated proto types.

use crate::state::{Config, DaemonInfo, Doc, Issue, IssueMetadata, PrMetadata, Project, PullRequest};
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

impl DaemonClient {
    /// Create a new daemon client
    pub async fn new() -> Result<Self> {
        let address =
            std::env::var("CENTY_DAEMON_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_string());

        // Try to connect to the daemon
        let client = match CentyDaemonClient::connect(address.clone()).await {
            Ok(client) => Some(client),
            Err(_) => None,
        };

        Ok(Self { client, address })
    }

    /// Ensure connection is established
    async fn ensure_connected(&mut self) -> Result<&mut CentyDaemonClient<tonic::transport::Channel>> {
        if self.client.is_none() {
            self.client = Some(
                CentyDaemonClient::connect(self.address.clone())
                    .await
                    .map_err(|e| anyhow!("Failed to connect to daemon: {}", e))?,
            );
        }
        self.client.as_mut().ok_or_else(|| anyhow!("Client not connected"))
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
            draft: false,
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
    pub async fn create_doc(
        &mut self,
        project_path: &str,
        title: &str,
        content: &str,
        slug: Option<&str>,
    ) -> Result<String> {
        let client = self.ensure_connected().await?;

        let request = tonic::Request::new(proto::CreateDocRequest {
            project_path: project_path.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            slug: slug.unwrap_or("").to_string(),
            template: String::new(),
        });

        let response = client
            .create_doc(request)
            .await
            .map_err(|e| anyhow!("Failed to create doc: {}", e))?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to create doc: {}", inner.error));
        }

        Ok(inner.slug)
    }

    /// Restart the daemon
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
