//! Application state definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Current view in the application
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum View {
    /// Splash screen with logo animation
    Splash,
    #[default]
    Projects,
    Issues,
    IssueDetail,
    IssueCreate,
    IssueEdit,
    Prs,
    PrDetail,
    PrCreate,
    PrEdit,
    Docs,
    DocDetail,
    DocCreate,
    Config,
}

/// View parameters for navigation
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ViewParams {
    pub issue_id: Option<String>,
    pub pr_id: Option<String>,
    pub doc_slug: Option<String>,
}

/// Sort field for issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IssueSortField {
    #[default]
    Priority,
    DisplayNumber,
    CreatedAt,
    UpdatedAt,
    Status,
}

impl IssueSortField {
    pub fn next(&self) -> Self {
        match self {
            Self::Priority => Self::DisplayNumber,
            Self::DisplayNumber => Self::CreatedAt,
            Self::CreatedAt => Self::UpdatedAt,
            Self::UpdatedAt => Self::Status,
            Self::Status => Self::Priority,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Priority => "Priority",
            Self::DisplayNumber => "Number",
            Self::CreatedAt => "Created",
            Self::UpdatedAt => "Updated",
            Self::Status => "Status",
        }
    }
}

/// Sort field for PRs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrSortField {
    #[default]
    Priority,
    DisplayNumber,
    CreatedAt,
    UpdatedAt,
    Status,
}

impl PrSortField {
    pub fn next(&self) -> Self {
        match self {
            Self::Priority => Self::DisplayNumber,
            Self::DisplayNumber => Self::CreatedAt,
            Self::CreatedAt => Self::UpdatedAt,
            Self::UpdatedAt => Self::Status,
            Self::Status => Self::Priority,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Priority => "Priority",
            Self::DisplayNumber => "Number",
            Self::CreatedAt => "Created",
            Self::UpdatedAt => "Updated",
            Self::Status => "Status",
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Asc => "↑",
            Self::Desc => "↓",
        }
    }
}

/// Focus state for issue detail view (content vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IssueDetailFocus {
    #[default]
    Content,
    ActionPanel,
}

impl IssueDetailFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Content => Self::ActionPanel,
            Self::ActionPanel => Self::Content,
        };
    }
}

/// LLM action type for agent operations (mirrors proto LlmAction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LlmAction {
    #[default]
    Plan,
    Implement,
}

#[allow(dead_code)]
impl LlmAction {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Plan => Self::Implement,
            Self::Implement => Self::Plan,
        };
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Plan => "Plan",
            Self::Implement => "Implement",
        }
    }

    pub fn as_proto_value(self) -> i32 {
        match self {
            Self::Plan => 1,
            Self::Implement => 2,
        }
    }
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub project_title: Option<String>,
    pub user_title: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub initialized: bool,
    pub issue_count: u32,
    pub doc_count: u32,
    pub pr_count: u32,
}

impl Project {
    pub fn display_name(&self) -> &str {
        self.user_title
            .as_deref()
            .or(self.project_title.as_deref())
            .unwrap_or(&self.name)
    }
}

/// Issue metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueMetadata {
    pub status: String,
    pub priority: u32,
    pub priority_label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub custom_fields: HashMap<String, String>,
}

/// Issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub display_number: u32,
    pub title: String,
    pub description: String,
    pub metadata: IssueMetadata,
}

#[allow(dead_code)]
impl Issue {
    pub fn priority_color(&self) -> &'static str {
        match self.metadata.priority {
            1 => "red",
            2 => "yellow",
            _ => "green",
        }
    }

    pub fn priority_label(&self) -> &str {
        self.metadata
            .priority_label
            .as_deref()
            .unwrap_or(match self.metadata.priority {
                1 => "high",
                2 => "med",
                _ => "low",
            })
    }
}

/// PR metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrMetadata {
    pub status: String,
    pub priority: u32,
    pub priority_label: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub linked_issues: Vec<String>,
    pub reviewers: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub custom_fields: HashMap<String, String>,
}

/// Pull Request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: String,
    pub display_number: u32,
    pub title: String,
    pub description: String,
    pub metadata: PrMetadata,
}

#[allow(dead_code)]
impl PullRequest {
    pub fn status_color(&self) -> &'static str {
        match self.metadata.status.as_str() {
            "draft" => "gray",
            "open" => "blue",
            "merged" => "magenta",
            "closed" => "red",
            _ => "gray",
        }
    }
}

/// Doc information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub priority_levels: u32,
    pub allowed_states: Vec<String>,
    pub default_state: String,
    pub version: String,
}

/// Daemon information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub version: String,
    pub uptime_seconds: u64,
    pub project_count: u32,
}

/// Main application state
#[derive(Default)]
pub struct AppState {
    // Navigation
    pub current_view: View,
    pub view_params: ViewParams,
    pub view_history: Vec<(View, ViewParams)>,

    // Project
    pub projects: Vec<Project>,
    pub selected_project_path: Option<String>,

    // Data
    pub issues: Vec<Issue>,
    pub prs: Vec<PullRequest>,
    pub docs: Vec<Doc>,
    pub config: Option<Config>,
    #[allow(dead_code)]
    pub daemon_info: Option<DaemonInfo>,

    // Selection
    pub selected_index: usize,
    pub selected_issue_id: Option<String>,
    pub selected_pr_id: Option<String>,
    pub selected_doc_slug: Option<String>,
    pub sidebar_index: usize,

    // Sorting
    pub issue_sort_field: IssueSortField,
    pub issue_sort_direction: SortDirection,
    pub pr_sort_field: PrSortField,
    pub pr_sort_direction: SortDirection,

    // Filters
    pub show_closed_issues: bool,
    pub show_merged_prs: bool,

    // UI state
    pub scroll_offset: usize,
    pub daemon_connected: bool,
    pub confirm_action: Option<String>,

    // Issue detail action panel state
    pub issue_detail_focus: IssueDetailFocus,
    pub action_panel_llm_action: LlmAction,

    // Double-click detection for project grid
    pub last_click_time: Option<Instant>,
    pub last_click_index: Option<usize>,

    // Form state
    pub active_form_field: usize,
    pub form_title: String,
    pub form_description: String,
    pub form_priority: u32,
    pub form_status: String,
    pub form_slug: String,
    pub form_source_branch: String,
    pub form_target_branch: String,
}

impl AppState {
    /// Move selection down
    pub fn move_selection_down(&mut self, max: usize) {
        if max > 0 && self.selected_index < max - 1 {
            self.selected_index += 1;
        }
    }

    /// Move selection up
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection left in grid
    pub fn move_selection_left(&mut self, columns: usize) {
        if columns == 0 {
            return;
        }
        let col = self.selected_index % columns;
        if col > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection right in grid
    pub fn move_selection_right(&mut self, columns: usize, total: usize) {
        if columns == 0 || total == 0 {
            return;
        }
        let col = self.selected_index % columns;
        if col < columns - 1 && self.selected_index + 1 < total {
            self.selected_index += 1;
        }
    }

    /// Move selection up in grid (by one row)
    pub fn move_selection_up_grid(&mut self, columns: usize) {
        if columns == 0 {
            return;
        }
        if self.selected_index >= columns {
            self.selected_index -= columns;
        }
    }

    /// Move selection down in grid (by one row)
    pub fn move_selection_down_grid(&mut self, columns: usize, total: usize) {
        if columns == 0 || total == 0 {
            return;
        }
        let new_index = self.selected_index + columns;
        if new_index < total {
            self.selected_index = new_index;
        } else {
            // If moving down would go past end, try to move to last row same column
            let current_row = self.selected_index / columns;
            let last_row = (total - 1) / columns;
            if current_row < last_row {
                // Move to same column in last row, or last item if column doesn't exist
                let target_col = self.selected_index % columns;
                let last_row_start = last_row * columns;
                let potential_target = last_row_start + target_col;
                if potential_target < total {
                    self.selected_index = potential_target;
                } else {
                    self.selected_index = total - 1;
                }
            }
        }
    }

    /// Reset selection
    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down a page (10 lines)
    pub fn scroll_down_page(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(10);
    }

    /// Scroll up a page (10 lines)
    pub fn scroll_up_page(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    /// Cycle issue sort field
    pub fn cycle_issue_sort_field(&mut self) {
        self.issue_sort_field = self.issue_sort_field.next();
        self.reset_selection();
    }

    /// Toggle issue sort direction
    pub fn toggle_issue_sort_direction(&mut self) {
        self.issue_sort_direction = self.issue_sort_direction.toggle();
        self.reset_selection();
    }

    /// Cycle PR sort field
    pub fn cycle_pr_sort_field(&mut self) {
        self.pr_sort_field = self.pr_sort_field.next();
        self.reset_selection();
    }

    /// Toggle PR sort direction
    pub fn toggle_pr_sort_direction(&mut self) {
        self.pr_sort_direction = self.pr_sort_direction.toggle();
        self.reset_selection();
    }

    /// Get sorted projects (favorites first)
    pub fn sorted_projects(&self) -> Vec<&Project> {
        let mut projects: Vec<_> = self.projects.iter().collect();
        projects.sort_by(|a, b| b.is_favorite.cmp(&a.is_favorite));
        projects
    }

    /// Get sorted issues
    pub fn sorted_issues(&self) -> Vec<&Issue> {
        let mut issues: Vec<_> = self
            .issues
            .iter()
            .filter(|i| self.show_closed_issues || i.metadata.status != "closed")
            .collect();

        issues.sort_by(|a, b| {
            let cmp = match self.issue_sort_field {
                IssueSortField::Priority => a.metadata.priority.cmp(&b.metadata.priority),
                IssueSortField::DisplayNumber => a.display_number.cmp(&b.display_number),
                IssueSortField::CreatedAt => a.metadata.created_at.cmp(&b.metadata.created_at),
                IssueSortField::UpdatedAt => a.metadata.updated_at.cmp(&b.metadata.updated_at),
                IssueSortField::Status => a.metadata.status.cmp(&b.metadata.status),
            };

            match self.issue_sort_direction {
                SortDirection::Asc => cmp,
                SortDirection::Desc => cmp.reverse(),
            }
        });

        issues
    }

    /// Get sorted PRs
    pub fn sorted_prs(&self) -> Vec<&PullRequest> {
        let mut prs: Vec<_> = self
            .prs
            .iter()
            .filter(|p| {
                self.show_merged_prs
                    || (p.metadata.status != "merged" && p.metadata.status != "closed")
            })
            .collect();

        prs.sort_by(|a, b| {
            let cmp = match self.pr_sort_field {
                PrSortField::Priority => a.metadata.priority.cmp(&b.metadata.priority),
                PrSortField::DisplayNumber => a.display_number.cmp(&b.display_number),
                PrSortField::CreatedAt => a.metadata.created_at.cmp(&b.metadata.created_at),
                PrSortField::UpdatedAt => a.metadata.updated_at.cmp(&b.metadata.updated_at),
                PrSortField::Status => a.metadata.status.cmp(&b.metadata.status),
            };

            match self.pr_sort_direction {
                SortDirection::Asc => cmp,
                SortDirection::Desc => cmp.reverse(),
            }
        });

        prs
    }

    /// Move to next form field
    pub fn next_form_field(&mut self) {
        self.active_form_field = (self.active_form_field + 1) % self.form_field_count();
    }

    /// Move to previous form field
    pub fn prev_form_field(&mut self) {
        if self.active_form_field == 0 {
            self.active_form_field = self.form_field_count() - 1;
        } else {
            self.active_form_field -= 1;
        }
    }

    /// Get number of form fields for current view
    pub fn form_field_count(&self) -> usize {
        match self.current_view {
            View::IssueCreate => 3, // title, description, priority
            View::IssueEdit => 4,   // title, description, priority, status
            View::PrCreate => 5,    // title, description, source, target, priority
            View::PrEdit => 6,      // title, description, source, target, priority, status
            View::DocCreate => 3,   // title, content, slug
            _ => 1,
        }
    }

    /// Handle character input in form
    pub fn form_input_char(&mut self, c: char, shift: bool) {
        let ch = if shift { c.to_ascii_uppercase() } else { c };

        match self.current_view {
            View::IssueCreate | View::IssueEdit => match self.active_form_field {
                0 => self.form_title.push(ch),
                1 => self.form_description.push(ch),
                2 => {
                    if let Some(d) = c.to_digit(10) {
                        self.form_priority = d;
                    }
                }
                3 => self.form_status.push(ch),
                _ => {}
            },
            View::PrCreate | View::PrEdit => match self.active_form_field {
                0 => self.form_title.push(ch),
                1 => self.form_description.push(ch),
                2 => self.form_source_branch.push(ch),
                3 => self.form_target_branch.push(ch),
                4 => {
                    if let Some(d) = c.to_digit(10) {
                        self.form_priority = d;
                    }
                }
                5 => self.form_status.push(ch),
                _ => {}
            },
            View::DocCreate => match self.active_form_field {
                0 => self.form_title.push(ch),
                1 => self.form_description.push(ch),
                2 => self.form_slug.push(ch),
                _ => {}
            },
            _ => {}
        }
    }

    /// Handle backspace in form
    pub fn form_backspace(&mut self) {
        match self.current_view {
            View::IssueCreate | View::IssueEdit => match self.active_form_field {
                0 => {
                    self.form_title.pop();
                }
                1 => {
                    self.form_description.pop();
                }
                3 => {
                    self.form_status.pop();
                }
                _ => {}
            },
            View::PrCreate | View::PrEdit => match self.active_form_field {
                0 => {
                    self.form_title.pop();
                }
                1 => {
                    self.form_description.pop();
                }
                2 => {
                    self.form_source_branch.pop();
                }
                3 => {
                    self.form_target_branch.pop();
                }
                5 => {
                    self.form_status.pop();
                }
                _ => {}
            },
            View::DocCreate => match self.active_form_field {
                0 => {
                    self.form_title.pop();
                }
                1 => {
                    self.form_description.pop();
                }
                2 => {
                    self.form_slug.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Clear form state
    pub fn clear_form(&mut self) {
        self.active_form_field = 0;
        self.form_title.clear();
        self.form_description.clear();
        self.form_priority = 0;
        self.form_status.clear();
        self.form_slug.clear();
        self.form_source_branch.clear();
        self.form_target_branch.clear();
    }

    /// Load issue data into form for editing
    #[allow(dead_code)]
    pub fn load_issue_to_form(&mut self, issue: &Issue) {
        self.form_title = issue.title.clone();
        self.form_description = issue.description.clone();
        self.form_priority = issue.metadata.priority;
        self.form_status = issue.metadata.status.clone();
    }

    /// Load PR data into form for editing
    #[allow(dead_code)]
    pub fn load_pr_to_form(&mut self, pr: &PullRequest) {
        self.form_title = pr.title.clone();
        self.form_description = pr.description.clone();
        self.form_priority = pr.metadata.priority;
        self.form_status = pr.metadata.status.clone();
        self.form_source_branch = pr.metadata.source_branch.clone();
        self.form_target_branch = pr.metadata.target_branch.clone();
    }
}
