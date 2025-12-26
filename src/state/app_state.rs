//! Application state definitions

use super::forms::{
    DocCreateForm, FormState, IssueCreateForm, IssueEditForm, PrCreateForm, PrEditForm,
};
use super::SelectionState;
use crate::daemon::TempWorkspace;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Current view in the application
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum View {
    /// Splash screen with logo animation
    Splash,
    #[default]
    Projects,
    Organization,
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
    DocEdit,
    Config,
}

impl View {
    /// Returns true if this is a form view (create/edit) where sidebar should be hidden
    pub fn is_form_view(&self) -> bool {
        matches!(
            self,
            View::IssueCreate
                | View::IssueEdit
                | View::PrCreate
                | View::PrEdit
                | View::DocCreate
                | View::DocEdit
        )
    }
}

/// View parameters for navigation
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ViewParams {
    pub issue_id: Option<String>,
    pub pr_id: Option<String>,
    pub doc_slug: Option<String>,
    pub organization_slug: Option<String>,
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

/// Focus state for issues list view (list vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IssuesListFocus {
    #[default]
    List,
    ActionPanel,
}

impl IssuesListFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::List => Self::ActionPanel,
            Self::ActionPanel => Self::List,
        };
    }
}

/// Focus state for PRs list view (list vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrsListFocus {
    #[default]
    List,
    ActionPanel,
}

impl PrsListFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::List => Self::ActionPanel,
            Self::ActionPanel => Self::List,
        };
    }
}

/// Focus state for PR detail view (content vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrDetailFocus {
    #[default]
    Content,
    ActionPanel,
}

impl PrDetailFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Content => Self::ActionPanel,
            Self::ActionPanel => Self::Content,
        };
    }
}

/// Focus state for Docs list view (list vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DocsListFocus {
    #[default]
    List,
    ActionPanel,
}

impl DocsListFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::List => Self::ActionPanel,
            Self::ActionPanel => Self::List,
        };
    }
}

/// Focus state for Doc detail view (content vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DocDetailFocus {
    #[default]
    Content,
    ActionPanel,
}

impl DocDetailFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Content => Self::ActionPanel,
            Self::ActionPanel => Self::Content,
        };
    }
}

/// Focus state for Organization view (projects list vs action panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizationFocus {
    #[default]
    ProjectsList,
    ActionPanel,
}

impl OrganizationFocus {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::ProjectsList => Self::ActionPanel,
            Self::ActionPanel => Self::ProjectsList,
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
    pub organization_slug: Option<String>,
    pub organization_name: Option<String>,
}

impl Project {
    pub fn display_name(&self) -> &str {
        self.user_title
            .as_deref()
            .or(self.project_title.as_deref())
            .unwrap_or(&self.name)
    }
}

/// Organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project_count: u32,
}

/// User/team member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub git_usernames: Vec<String>,
}

/// A section in the grouped projects view
#[derive(Debug, Clone)]
pub struct ProjectSection<'a> {
    pub header: String,
    pub is_favorites: bool,
    pub projects: Vec<&'a Project>,
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

/// Entity type for action requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Issue,
    Pr,
    Doc,
}

/// Action category for grouping in UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActionCategory {
    #[default]
    Unspecified,
    Crud,     // Create, Delete, Duplicate, Move
    Mode,     // Plan, Implement (LLM actions)
    Status,   // Status/state changes
    External, // Open in VSCode, external tools
    Research, // Research/deep dive actions
}

impl ActionCategory {
    pub fn from_proto(value: i32) -> Self {
        match value {
            1 => Self::Crud,
            2 => Self::Mode,
            3 => Self::Status,
            4 => Self::External,
            5 => Self::Research,
            _ => Self::Unspecified,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Unspecified => "Other",
            Self::Crud => "Actions",
            Self::Mode => "Mode",
            Self::Status => "Status",
            Self::External => "External",
            Self::Research => "Research",
        }
    }
}

/// An action that can be performed on an entity
#[derive(Debug, Clone)]
pub struct EntityAction {
    pub id: String,
    pub label: String,
    pub category: ActionCategory,
    pub enabled: bool,
    pub disabled_reason: String,
    pub destructive: bool,
    pub keyboard_shortcut: String,
}

/// Response containing available actions
#[derive(Debug, Clone, Default)]
pub struct EntityActionsResponse {
    pub actions: Vec<EntityAction>,
}

impl EntityActionsResponse {
    /// Get actions grouped by category
    pub fn grouped_actions(&self) -> Vec<(ActionCategory, Vec<&EntityAction>)> {
        let mut groups: Vec<(ActionCategory, Vec<&EntityAction>)> = Vec::new();

        // Collect unique categories in order
        let categories = [
            ActionCategory::Crud,
            ActionCategory::Mode,
            ActionCategory::Research,
            ActionCategory::Status,
            ActionCategory::External,
            ActionCategory::Unspecified,
        ];

        for category in categories {
            let actions: Vec<_> = self
                .actions
                .iter()
                .filter(|a| a.category == category)
                .collect();
            if !actions.is_empty() {
                groups.push((category, actions));
            }
        }

        groups
    }
}

/// Identifies which button is currently pressed for animation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PressedButton {
    /// Sidebar button at given index (0-4: Projects, Issues, PRs, Docs, Config)
    Sidebar(usize),
    /// Action panel button at given index
    ActionPanel(usize),
}

/// Tracks button press animation state
#[derive(Debug, Clone)]
pub struct ButtonPressState {
    /// Which button is pressed
    pub button: PressedButton,
    /// When the button was pressed
    pub pressed_at: Instant,
}

impl ButtonPressState {
    /// Animation duration in milliseconds
    pub const DURATION_MS: u64 = 120;

    pub fn new(button: PressedButton) -> Self {
        Self {
            button,
            pressed_at: Instant::now(),
        }
    }

    /// Check if the animation has expired
    pub fn is_expired(&self) -> bool {
        self.pressed_at.elapsed() > Duration::from_millis(Self::DURATION_MS)
    }
}

/// Options for handling existing worktree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorktreeDialogOption {
    #[default]
    OpenExisting,
    DeleteAndRecreate,
}

impl WorktreeDialogOption {
    pub fn toggle(&self) -> Self {
        match self {
            Self::OpenExisting => Self::DeleteAndRecreate,
            Self::DeleteAndRecreate => Self::OpenExisting,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::OpenExisting => "Open existing workspace",
            Self::DeleteAndRecreate => "Delete and recreate",
        }
    }
}

/// Pending worktree action when a workspace already exists
#[derive(Debug, Clone)]
pub struct PendingWorktreeAction {
    pub project_path: String,
    pub issue_id: String,
    pub action: i32,
    pub existing_workspace: TempWorkspace,
    pub selected_option: WorktreeDialogOption,
}

/// Entity type for move operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveEntityType {
    Issue,
    Doc,
}

/// Pending move action state (project picker + confirmation)
#[derive(Debug, Clone)]
pub struct PendingMoveAction {
    pub entity_type: MoveEntityType,
    pub source_project_path: String,
    pub entity_id: String,
    pub entity_display: String,
    pub selected_project_index: usize,
    pub search_filter: String,
    pub show_confirmation: bool,
    pub target_project_path: Option<String>,
}

/// Pending start work action state (status change confirmation)
#[derive(Debug, Clone)]
pub struct PendingStartWorkAction {
    pub action_id: String,
    pub action_label: String,
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
    /// Scroll offset for sidebars (nav sidebar and action panel share this)
    pub sidebar_scroll_offset: usize,
    pub daemon_connected: bool,
    pub confirm_action: Option<String>,
    /// Pending worktree action when a workspace already exists
    pub pending_worktree_action: Option<PendingWorktreeAction>,
    /// Pending move action when moving an issue or doc
    pub pending_move_action: Option<PendingMoveAction>,
    /// Pending start work action when confirming status change to "in progress"
    pub pending_start_work_action: Option<PendingStartWorkAction>,
    /// Queue of error messages to display one at a time
    pub error_queue: VecDeque<String>,

    /// Cached context bar segment positions for mouse click detection
    /// Each entry is (start_col, end_col, target_view)
    pub context_bar_segments: Vec<(u16, u16, View)>,

    // Issue detail action panel state
    pub issue_detail_focus: IssueDetailFocus,
    pub action_panel_llm_action: LlmAction,

    // Issues list action panel state
    pub issues_list_focus: IssuesListFocus,

    // PRs action panel state
    pub prs_list_focus: PrsListFocus,
    pub pr_detail_focus: PrDetailFocus,

    // Docs action panel state
    pub docs_list_focus: DocsListFocus,
    pub doc_detail_focus: DocDetailFocus,

    // Organization view state
    pub organization_focus: OrganizationFocus,
    pub current_organization: Option<Organization>,
    pub organization_projects: Vec<Project>,
    pub project_users: HashMap<String, Vec<User>>,
    pub selected_project_in_org: usize,

    // Dynamic actions state (from GetEntityActions)
    pub current_actions: EntityActionsResponse,
    pub actions_loading: bool,
    pub actions_error: Option<String>,
    pub action_panel_selected_index: usize,

    // Double-click detection for project grid
    pub last_click_time: Option<Instant>,
    pub last_click_index: Option<usize>,

    // Form state (DDD) - scaffolding for future form refactoring
    #[allow(dead_code)]
    pub form: FormState,

    // Legacy form state (kept for compatibility during migration)
    pub active_form_field: usize,
    pub form_title: String,
    pub form_description: String,
    pub form_priority: u32,
    pub form_status: String,
    pub form_slug: String,
    pub form_is_org_doc: bool,
    pub form_source_branch: String,
    pub form_target_branch: String,
    /// Selected button index for create forms (0=Cancel, 1=Draft, 2=Create&New, 3=Create)
    pub form_selected_button: usize,

    // Text selection state
    pub selection: SelectionState,

    // Button press animation state
    pub button_press: Option<ButtonPressState>,
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
    #[allow(dead_code)]
    pub fn move_selection_up_grid(&mut self, columns: usize) {
        if columns == 0 {
            return;
        }
        if self.selected_index >= columns {
            self.selected_index -= columns;
        }
    }

    /// Move selection down in grid (by one row)
    #[allow(dead_code)]
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

    /// Move selection down in grouped grid (by one row)
    /// Handles crossing section boundaries
    pub fn move_selection_down_grouped_grid(&mut self, columns: usize) {
        let sections = self.grouped_projects();
        let total = self.selectable_projects().len();

        if columns == 0 || total == 0 || sections.is_empty() {
            return;
        }

        // Find which section and position within section
        let (section_idx, pos_in_section) = self.find_section_position(self.selected_index);

        if section_idx >= sections.len() {
            return;
        }

        let section = &sections[section_idx];
        let section_size = section.projects.len();
        let current_row_in_section = pos_in_section / columns;
        let current_col = pos_in_section % columns;
        let rows_in_section = section_size.div_ceil(columns);

        if current_row_in_section + 1 < rows_in_section {
            // Move within same section
            let new_pos = (current_row_in_section + 1) * columns + current_col;
            if new_pos < section_size {
                let section_start = self.section_start_index(section_idx);
                self.selected_index = section_start + new_pos;
            } else {
                // Partial last row - go to last item in section
                let section_start = self.section_start_index(section_idx);
                self.selected_index = section_start + section_size - 1;
            }
        } else if section_idx + 1 < sections.len() {
            // Move to next section
            let next_section = &sections[section_idx + 1];
            let next_section_start = self.section_start_index(section_idx + 1);
            // Try to maintain column position
            let target_col = current_col.min(next_section.projects.len().saturating_sub(1));
            self.selected_index = next_section_start + target_col;
        }
    }

    /// Move selection up in grouped grid (by one row)
    /// Handles crossing section boundaries
    pub fn move_selection_up_grouped_grid(&mut self, columns: usize) {
        let sections = self.grouped_projects();

        if columns == 0 || sections.is_empty() {
            return;
        }

        let (section_idx, pos_in_section) = self.find_section_position(self.selected_index);

        if section_idx >= sections.len() {
            return;
        }

        let current_row_in_section = pos_in_section / columns;
        let current_col = pos_in_section % columns;

        if current_row_in_section > 0 {
            // Move within same section
            let new_pos = (current_row_in_section - 1) * columns + current_col;
            let section_start = self.section_start_index(section_idx);
            self.selected_index = section_start + new_pos;
        } else if section_idx > 0 {
            // Move to previous section's last row
            let prev_section = &sections[section_idx - 1];
            let prev_section_start = self.section_start_index(section_idx - 1);
            let prev_section_size = prev_section.projects.len();
            let prev_rows = prev_section_size.div_ceil(columns);
            let last_row_start = (prev_rows - 1) * columns;

            // Try to maintain column position
            let target_pos = last_row_start + current_col;
            if target_pos < prev_section_size {
                self.selected_index = prev_section_start + target_pos;
            } else {
                self.selected_index = prev_section_start + prev_section_size - 1;
            }
        }
    }

    /// Helper: Find section index and position within section for a global index
    fn find_section_position(&self, global_index: usize) -> (usize, usize) {
        let sections = self.grouped_projects();
        let mut offset = 0;

        for (i, section) in sections.iter().enumerate() {
            let section_size = section.projects.len();
            if global_index < offset + section_size {
                return (i, global_index - offset);
            }
            offset += section_size;
        }

        (sections.len(), 0) // Past end
    }

    /// Helper: Get starting index for a section
    fn section_start_index(&self, section_idx: usize) -> usize {
        let sections = self.grouped_projects();
        sections
            .iter()
            .take(section_idx)
            .map(|s| s.projects.len())
            .sum()
    }

    /// Reset selection
    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Calculate the Y position of a project card given its index and grid columns
    /// Returns (y_start, y_end) in content coordinates (before scroll offset)
    pub fn project_y_position(&self, project_index: usize, columns: usize) -> (usize, usize) {
        const SECTION_HEADER_HEIGHT: usize = 2;
        const CARD_HEIGHT: usize = 4;

        let sections = self.grouped_projects();
        let mut y_offset: usize = 0;
        let mut current_index: usize = 0;

        for section in &sections {
            y_offset += SECTION_HEADER_HEIGHT;

            let section_size = section.projects.len();

            if project_index < current_index + section_size {
                // Project is in this section
                let pos_in_section = project_index - current_index;
                let row = pos_in_section / columns;
                let card_y = y_offset + row * CARD_HEIGHT;
                return (card_y, card_y + CARD_HEIGHT);
            }

            let rows_in_section = section_size.div_ceil(columns);
            y_offset += rows_in_section * CARD_HEIGHT;
            current_index += section_size;
        }

        (y_offset, y_offset + CARD_HEIGHT)
    }

    /// Ensure the selected project is visible by adjusting scroll offset
    pub fn ensure_selected_visible(&mut self, columns: usize, visible_height: usize) {
        let (card_y_start, card_y_end) = self.project_y_position(self.selected_index, columns);

        // If card is above visible area, scroll up
        if card_y_start < self.scroll_offset {
            self.scroll_offset = card_y_start;
        }
        // If card is below visible area, scroll down
        else if card_y_end > self.scroll_offset + visible_height {
            self.scroll_offset = card_y_end.saturating_sub(visible_height);
        }
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
    #[allow(dead_code)]
    pub fn sorted_projects(&self) -> Vec<&Project> {
        let mut projects: Vec<_> = self.projects.iter().collect();
        projects.sort_by(|a, b| b.is_favorite.cmp(&a.is_favorite));
        projects
    }

    /// Get projects grouped by organization with favorites section first
    /// Returns sections in order:
    /// 1. Favorites (all favorited projects, regardless of org)
    /// 2. Organization groups (sorted alphabetically by org name)
    /// 3. Ungrouped (projects without organization)
    pub fn grouped_projects(&self) -> Vec<ProjectSection<'_>> {
        use std::collections::HashMap;

        let mut sections = Vec::new();

        // 1. Collect all favorites first
        let favorites: Vec<&Project> = self.projects.iter().filter(|p| p.is_favorite).collect();

        if !favorites.is_empty() {
            sections.push(ProjectSection {
                header: "Favorites".to_string(),
                is_favorites: true,
                projects: favorites,
            });
        }

        // 2. Group remaining non-favorite projects by organization
        let mut org_groups: HashMap<String, Vec<&Project>> = HashMap::new();
        let mut ungrouped: Vec<&Project> = Vec::new();

        for project in self.projects.iter().filter(|p| !p.is_favorite) {
            if let Some(org_slug) = &project.organization_slug {
                org_groups
                    .entry(org_slug.clone())
                    .or_default()
                    .push(project);
            } else {
                ungrouped.push(project);
            }
        }

        // Sort organization groups alphabetically and add them
        let mut org_keys: Vec<_> = org_groups.keys().cloned().collect();
        org_keys.sort();

        for org_slug in org_keys {
            if let Some(projects) = org_groups.get(&org_slug) {
                // Get display name from first project in group
                let org_name = projects
                    .first()
                    .and_then(|p| p.organization_name.clone())
                    .unwrap_or_else(|| org_slug.clone());

                sections.push(ProjectSection {
                    header: org_name,
                    is_favorites: false,
                    projects: projects.clone(),
                });
            }
        }

        // 3. Add ungrouped section at the end
        if !ungrouped.is_empty() {
            sections.push(ProjectSection {
                header: "Ungrouped".to_string(),
                is_favorites: false,
                projects: ungrouped,
            });
        }

        sections
    }

    /// Get flat list of selectable projects (for index-based navigation)
    /// This maintains the grouped order but returns only projects
    pub fn selectable_projects(&self) -> Vec<&Project> {
        self.grouped_projects()
            .into_iter()
            .flat_map(|section| section.projects)
            .collect()
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
            View::IssueCreate => 4, // title, description, priority, buttons
            View::IssueEdit => 4,   // title, description, priority, status
            View::PrCreate => 5,    // title, description, source, target, priority
            View::PrEdit => 6,      // title, description, source, target, priority, status
            View::DocCreate => 3,   // title, content, slug
            View::DocEdit => 3,     // title, content, slug
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
            View::DocCreate | View::DocEdit => match self.active_form_field {
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
            View::DocCreate | View::DocEdit => match self.active_form_field {
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
        self.form_is_org_doc = false;
        self.form_source_branch.clear();
        self.form_target_branch.clear();
        self.form_selected_button = 0; // Default to "Create" button (first in sidebar)
    }

    /// Load issue data into form for editing
    pub fn load_issue_to_form(&mut self, issue: &Issue) {
        self.form_title = issue.title.clone();
        self.form_description = issue.description.clone();
        self.form_priority = issue.metadata.priority;
        self.form_status = issue.metadata.status.clone();
    }

    /// Load PR data into form for editing
    pub fn load_pr_to_form(&mut self, pr: &PullRequest) {
        self.form_title = pr.title.clone();
        self.form_description = pr.description.clone();
        self.form_priority = pr.metadata.priority;
        self.form_status = pr.metadata.status.clone();
        self.form_source_branch = pr.metadata.source_branch.clone();
        self.form_target_branch = pr.metadata.target_branch.clone();
    }

    /// Load doc data into form for editing
    pub fn load_doc_to_form(&mut self, doc: &Doc) {
        self.form_title = doc.title.clone();
        self.form_description = doc.content.clone();
        self.form_slug = doc.slug.clone();
    }

    /// Toggle the org-wide doc checkbox
    pub fn toggle_org_doc(&mut self) {
        self.form_is_org_doc = !self.form_is_org_doc;
    }

    // =========== Form State Transitions (DDD) ===========
    // Scaffolding for future form refactoring - not yet integrated

    #[allow(dead_code)]
    pub fn start_issue_create(&mut self) {
        self.form = FormState::IssueCreate(IssueCreateForm::new());
    }

    #[allow(dead_code)]
    pub fn start_issue_edit(&mut self, issue: &Issue) {
        self.form = FormState::IssueEdit(IssueEditForm::from_issue(issue));
    }

    #[allow(dead_code)]
    pub fn start_pr_create(&mut self) {
        self.form = FormState::PrCreate(PrCreateForm::new());
    }

    #[allow(dead_code)]
    pub fn start_pr_edit(&mut self, pr: &PullRequest) {
        self.form = FormState::PrEdit(PrEditForm::from_pr(pr));
    }

    #[allow(dead_code)]
    pub fn start_doc_create(&mut self) {
        self.form = FormState::DocCreate(DocCreateForm::new());
    }

    #[allow(dead_code)]
    pub fn reset_form(&mut self) {
        self.form = FormState::None;
    }

    // =========== Action Panel Navigation ===========

    /// Get currently selected action
    pub fn selected_action(&self) -> Option<&EntityAction> {
        self.current_actions
            .actions
            .get(self.action_panel_selected_index)
    }

    /// Navigate action panel up
    pub fn action_panel_up(&mut self) {
        if self.action_panel_selected_index > 0 {
            self.action_panel_selected_index -= 1;
        }
    }

    /// Navigate action panel down
    pub fn action_panel_down(&mut self) {
        let total = self.current_actions.actions.len();
        if total > 0 && self.action_panel_selected_index < total - 1 {
            self.action_panel_selected_index += 1;
        }
    }

    /// Check if action panel is focused for current view
    pub fn is_action_panel_focused(&self) -> bool {
        match self.current_view {
            View::Issues => matches!(self.issues_list_focus, IssuesListFocus::ActionPanel),
            View::IssueDetail => matches!(self.issue_detail_focus, IssueDetailFocus::ActionPanel),
            View::Prs => matches!(self.prs_list_focus, PrsListFocus::ActionPanel),
            View::PrDetail => matches!(self.pr_detail_focus, PrDetailFocus::ActionPanel),
            View::Docs => matches!(self.docs_list_focus, DocsListFocus::ActionPanel),
            View::DocDetail => matches!(self.doc_detail_focus, DocDetailFocus::ActionPanel),
            View::Organization => matches!(self.organization_focus, OrganizationFocus::ActionPanel),
            _ => false,
        }
    }

    // =========== Error Queue Management ===========

    /// Add an error to the queue
    pub fn push_error(&mut self, message: String) {
        self.error_queue.push_back(message);
    }

    /// Get the current error (front of queue) without removing it
    pub fn current_error(&self) -> Option<&String> {
        self.error_queue.front()
    }

    /// Dismiss the current error (removes from front of queue)
    pub fn dismiss_error(&mut self) {
        self.error_queue.pop_front();
    }

    /// Check if there are any errors to display
    pub fn has_errors(&self) -> bool {
        !self.error_queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod view_tests {
        use super::*;

        #[test]
        fn test_default_is_projects() {
            let view = View::default();
            assert_eq!(view, View::Projects);
        }

        #[test]
        fn test_is_form_view_true_for_create_edit() {
            assert!(View::IssueCreate.is_form_view());
            assert!(View::IssueEdit.is_form_view());
            assert!(View::PrCreate.is_form_view());
            assert!(View::PrEdit.is_form_view());
            assert!(View::DocCreate.is_form_view());
            assert!(View::DocEdit.is_form_view());
        }

        #[test]
        fn test_is_form_view_false_for_other_views() {
            assert!(!View::Projects.is_form_view());
            assert!(!View::Issues.is_form_view());
            assert!(!View::IssueDetail.is_form_view());
            assert!(!View::Prs.is_form_view());
            assert!(!View::PrDetail.is_form_view());
            assert!(!View::Docs.is_form_view());
            assert!(!View::DocDetail.is_form_view());
            assert!(!View::Config.is_form_view());
            assert!(!View::Splash.is_form_view());
        }
    }

    mod issue_sort_field_tests {
        use super::*;

        #[test]
        fn test_default_is_priority() {
            assert_eq!(IssueSortField::default(), IssueSortField::Priority);
        }

        #[test]
        fn test_next_cycles_through_all() {
            let mut field = IssueSortField::Priority;
            field = field.next();
            assert_eq!(field, IssueSortField::DisplayNumber);
            field = field.next();
            assert_eq!(field, IssueSortField::CreatedAt);
            field = field.next();
            assert_eq!(field, IssueSortField::UpdatedAt);
            field = field.next();
            assert_eq!(field, IssueSortField::Status);
            field = field.next();
            assert_eq!(field, IssueSortField::Priority);
        }

        #[test]
        fn test_label_returns_correct_strings() {
            assert_eq!(IssueSortField::Priority.label(), "Priority");
            assert_eq!(IssueSortField::DisplayNumber.label(), "Number");
            assert_eq!(IssueSortField::CreatedAt.label(), "Created");
            assert_eq!(IssueSortField::UpdatedAt.label(), "Updated");
            assert_eq!(IssueSortField::Status.label(), "Status");
        }
    }

    mod pr_sort_field_tests {
        use super::*;

        #[test]
        fn test_default_is_priority() {
            assert_eq!(PrSortField::default(), PrSortField::Priority);
        }

        #[test]
        fn test_next_cycles_through_all() {
            let mut field = PrSortField::Priority;
            field = field.next();
            assert_eq!(field, PrSortField::DisplayNumber);
            field = field.next();
            assert_eq!(field, PrSortField::CreatedAt);
            field = field.next();
            assert_eq!(field, PrSortField::UpdatedAt);
            field = field.next();
            assert_eq!(field, PrSortField::Status);
            field = field.next();
            assert_eq!(field, PrSortField::Priority);
        }

        #[test]
        fn test_label_returns_correct_strings() {
            assert_eq!(PrSortField::Priority.label(), "Priority");
            assert_eq!(PrSortField::DisplayNumber.label(), "Number");
        }
    }

    mod sort_direction_tests {
        use super::*;

        #[test]
        fn test_default_is_asc() {
            assert_eq!(SortDirection::default(), SortDirection::Asc);
        }

        #[test]
        fn test_toggle_asc_to_desc() {
            let dir = SortDirection::Asc;
            assert_eq!(dir.toggle(), SortDirection::Desc);
        }

        #[test]
        fn test_toggle_desc_to_asc() {
            let dir = SortDirection::Desc;
            assert_eq!(dir.toggle(), SortDirection::Asc);
        }

        #[test]
        fn test_symbol_returns_arrows() {
            assert_eq!(SortDirection::Asc.symbol(), "↑");
            assert_eq!(SortDirection::Desc.symbol(), "↓");
        }
    }

    mod focus_toggle_tests {
        use super::*;

        #[test]
        fn test_issue_detail_focus_toggle() {
            let mut focus = IssueDetailFocus::Content;
            focus.toggle();
            assert_eq!(focus, IssueDetailFocus::ActionPanel);
            focus.toggle();
            assert_eq!(focus, IssueDetailFocus::Content);
        }

        #[test]
        fn test_issues_list_focus_toggle() {
            let mut focus = IssuesListFocus::List;
            focus.toggle();
            assert_eq!(focus, IssuesListFocus::ActionPanel);
            focus.toggle();
            assert_eq!(focus, IssuesListFocus::List);
        }

        #[test]
        fn test_prs_list_focus_toggle() {
            let mut focus = PrsListFocus::List;
            focus.toggle();
            assert_eq!(focus, PrsListFocus::ActionPanel);
        }

        #[test]
        fn test_pr_detail_focus_toggle() {
            let mut focus = PrDetailFocus::Content;
            focus.toggle();
            assert_eq!(focus, PrDetailFocus::ActionPanel);
        }

        #[test]
        fn test_docs_list_focus_toggle() {
            let mut focus = DocsListFocus::List;
            focus.toggle();
            assert_eq!(focus, DocsListFocus::ActionPanel);
        }

        #[test]
        fn test_doc_detail_focus_toggle() {
            let mut focus = DocDetailFocus::Content;
            focus.toggle();
            assert_eq!(focus, DocDetailFocus::ActionPanel);
        }
    }

    mod llm_action_tests {
        use super::*;

        #[test]
        fn test_default_is_plan() {
            assert_eq!(LlmAction::default(), LlmAction::Plan);
        }

        #[test]
        fn test_toggle() {
            let mut action = LlmAction::Plan;
            action.toggle();
            assert_eq!(action, LlmAction::Implement);
            action.toggle();
            assert_eq!(action, LlmAction::Plan);
        }

        #[test]
        fn test_label() {
            assert_eq!(LlmAction::Plan.label(), "Plan");
            assert_eq!(LlmAction::Implement.label(), "Implement");
        }

        #[test]
        fn test_as_proto_value() {
            assert_eq!(LlmAction::Plan.as_proto_value(), 1);
            assert_eq!(LlmAction::Implement.as_proto_value(), 2);
        }
    }

    mod action_category_tests {
        use super::*;

        #[test]
        fn test_from_proto() {
            assert_eq!(ActionCategory::from_proto(1), ActionCategory::Crud);
            assert_eq!(ActionCategory::from_proto(2), ActionCategory::Mode);
            assert_eq!(ActionCategory::from_proto(3), ActionCategory::Status);
            assert_eq!(ActionCategory::from_proto(4), ActionCategory::External);
            assert_eq!(ActionCategory::from_proto(0), ActionCategory::Unspecified);
            assert_eq!(ActionCategory::from_proto(99), ActionCategory::Unspecified);
        }

        #[test]
        fn test_label() {
            assert_eq!(ActionCategory::Unspecified.label(), "Other");
            assert_eq!(ActionCategory::Crud.label(), "Actions");
            assert_eq!(ActionCategory::Mode.label(), "Mode");
            assert_eq!(ActionCategory::Status.label(), "Status");
            assert_eq!(ActionCategory::External.label(), "External");
        }
    }

    mod project_tests {
        use super::*;

        #[test]
        fn test_display_name_prefers_user_title() {
            let project = Project {
                path: "/test".to_string(),
                name: "project".to_string(),
                project_title: Some("Project Title".to_string()),
                user_title: Some("My Custom Name".to_string()),
                is_favorite: false,
                is_archived: false,
                initialized: true,
                issue_count: 0,
                doc_count: 0,
                pr_count: 0,
                organization_slug: None,
                organization_name: None,
            };
            assert_eq!(project.display_name(), "My Custom Name");
        }

        #[test]
        fn test_display_name_falls_back_to_project_title() {
            let project = Project {
                path: "/test".to_string(),
                name: "project".to_string(),
                project_title: Some("Project Title".to_string()),
                user_title: None,
                is_favorite: false,
                is_archived: false,
                initialized: true,
                issue_count: 0,
                doc_count: 0,
                pr_count: 0,
                organization_slug: None,
                organization_name: None,
            };
            assert_eq!(project.display_name(), "Project Title");
        }

        #[test]
        fn test_display_name_falls_back_to_name() {
            let project = Project {
                path: "/test".to_string(),
                name: "project-name".to_string(),
                project_title: None,
                user_title: None,
                is_favorite: false,
                is_archived: false,
                initialized: true,
                issue_count: 0,
                doc_count: 0,
                pr_count: 0,
                organization_slug: None,
                organization_name: None,
            };
            assert_eq!(project.display_name(), "project-name");
        }
    }

    mod issue_tests {
        use super::*;

        fn create_test_issue(priority: u32, status: &str) -> Issue {
            Issue {
                id: "test-id".to_string(),
                display_number: 1,
                title: "Test".to_string(),
                description: "Desc".to_string(),
                metadata: IssueMetadata {
                    status: status.to_string(),
                    priority,
                    priority_label: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    custom_fields: HashMap::new(),
                },
            }
        }

        #[test]
        fn test_priority_color() {
            assert_eq!(create_test_issue(1, "open").priority_color(), "red");
            assert_eq!(create_test_issue(2, "open").priority_color(), "yellow");
            assert_eq!(create_test_issue(3, "open").priority_color(), "green");
        }

        #[test]
        fn test_priority_label_default() {
            assert_eq!(create_test_issue(1, "open").priority_label(), "high");
            assert_eq!(create_test_issue(2, "open").priority_label(), "med");
            assert_eq!(create_test_issue(3, "open").priority_label(), "low");
        }
    }

    mod app_state_tests {
        use super::*;

        fn create_default_state() -> AppState {
            AppState::default()
        }

        #[test]
        fn test_default_view_is_projects() {
            let state = create_default_state();
            // View::default() is Projects (see View enum with #[default])
            assert_eq!(state.current_view, View::Projects);
        }

        #[test]
        fn test_move_selection_down() {
            let mut state = create_default_state();
            state.selected_index = 0;
            state.move_selection_down(5);
            assert_eq!(state.selected_index, 1);
        }

        #[test]
        fn test_move_selection_down_at_max() {
            let mut state = create_default_state();
            state.selected_index = 4;
            state.move_selection_down(5);
            assert_eq!(state.selected_index, 4); // Stays at max
        }

        #[test]
        fn test_move_selection_up() {
            let mut state = create_default_state();
            state.selected_index = 3;
            state.move_selection_up();
            assert_eq!(state.selected_index, 2);
        }

        #[test]
        fn test_move_selection_up_at_zero() {
            let mut state = create_default_state();
            state.selected_index = 0;
            state.move_selection_up();
            assert_eq!(state.selected_index, 0);
        }

        #[test]
        fn test_scroll_down() {
            let mut state = create_default_state();
            state.scroll_offset = 0;
            state.scroll_down();
            assert_eq!(state.scroll_offset, 1);
        }

        #[test]
        fn test_scroll_up() {
            let mut state = create_default_state();
            state.scroll_offset = 5;
            state.scroll_up();
            assert_eq!(state.scroll_offset, 4);
        }

        #[test]
        fn test_scroll_up_at_zero() {
            let mut state = create_default_state();
            state.scroll_offset = 0;
            state.scroll_up();
            assert_eq!(state.scroll_offset, 0);
        }

        #[test]
        fn test_scroll_down_page() {
            let mut state = create_default_state();
            state.scroll_offset = 0;
            state.scroll_down_page();
            assert_eq!(state.scroll_offset, 10);
        }

        #[test]
        fn test_scroll_up_page() {
            let mut state = create_default_state();
            state.scroll_offset = 15;
            state.scroll_up_page();
            assert_eq!(state.scroll_offset, 5);
        }

        #[test]
        fn test_scroll_up_page_clamps_at_zero() {
            let mut state = create_default_state();
            state.scroll_offset = 3;
            state.scroll_up_page();
            assert_eq!(state.scroll_offset, 0);
        }

        #[test]
        fn test_form_field_count_for_views() {
            let mut state = create_default_state();

            state.current_view = View::IssueCreate;
            assert_eq!(state.form_field_count(), 4);

            state.current_view = View::IssueEdit;
            assert_eq!(state.form_field_count(), 4);

            state.current_view = View::PrCreate;
            assert_eq!(state.form_field_count(), 5);

            state.current_view = View::PrEdit;
            assert_eq!(state.form_field_count(), 6);

            state.current_view = View::DocCreate;
            assert_eq!(state.form_field_count(), 3);

            state.current_view = View::Projects;
            assert_eq!(state.form_field_count(), 1);
        }

        #[test]
        fn test_next_form_field_cycles() {
            let mut state = create_default_state();
            state.current_view = View::IssueCreate;
            state.active_form_field = 3;
            state.next_form_field();
            assert_eq!(state.active_form_field, 0);
        }

        #[test]
        fn test_prev_form_field_cycles() {
            let mut state = create_default_state();
            state.current_view = View::IssueCreate;
            state.active_form_field = 0;
            state.prev_form_field();
            assert_eq!(state.active_form_field, 3);
        }

        #[test]
        fn test_form_input_char_appends_to_title() {
            let mut state = create_default_state();
            state.current_view = View::IssueCreate;
            state.active_form_field = 0;
            state.form_input_char('a', false);
            state.form_input_char('b', false);
            assert_eq!(state.form_title, "ab");
        }

        #[test]
        fn test_form_input_char_with_shift() {
            let mut state = create_default_state();
            state.current_view = View::IssueCreate;
            state.active_form_field = 0;
            state.form_input_char('a', true);
            assert_eq!(state.form_title, "A");
        }

        #[test]
        fn test_form_backspace() {
            let mut state = create_default_state();
            state.current_view = View::IssueCreate;
            state.active_form_field = 0;
            state.form_title = "abc".to_string();
            state.form_backspace();
            assert_eq!(state.form_title, "ab");
        }

        #[test]
        fn test_clear_form() {
            let mut state = create_default_state();
            state.form_title = "Title".to_string();
            state.form_description = "Desc".to_string();
            state.form_priority = 2;
            state.active_form_field = 2;

            state.clear_form();

            assert_eq!(state.form_title, "");
            assert_eq!(state.form_description, "");
            assert_eq!(state.form_priority, 0);
            assert_eq!(state.active_form_field, 0);
        }

        #[test]
        fn test_action_panel_up() {
            let mut state = create_default_state();
            state.action_panel_selected_index = 3;
            state.action_panel_up();
            assert_eq!(state.action_panel_selected_index, 2);
        }

        #[test]
        fn test_action_panel_up_at_zero() {
            let mut state = create_default_state();
            state.action_panel_selected_index = 0;
            state.action_panel_up();
            assert_eq!(state.action_panel_selected_index, 0);
        }

        #[test]
        fn test_action_panel_down() {
            let mut state = create_default_state();
            state.current_actions.actions = vec![
                EntityAction {
                    id: "1".to_string(),
                    label: "One".to_string(),
                    category: ActionCategory::Crud,
                    enabled: true,
                    disabled_reason: String::new(),
                    destructive: false,
                    keyboard_shortcut: String::new(),
                },
                EntityAction {
                    id: "2".to_string(),
                    label: "Two".to_string(),
                    category: ActionCategory::Crud,
                    enabled: true,
                    disabled_reason: String::new(),
                    destructive: false,
                    keyboard_shortcut: String::new(),
                },
            ];
            state.action_panel_selected_index = 0;
            state.action_panel_down();
            assert_eq!(state.action_panel_selected_index, 1);
        }

        #[test]
        fn test_action_panel_down_at_max() {
            let mut state = create_default_state();
            state.current_actions.actions = vec![EntityAction {
                id: "1".to_string(),
                label: "One".to_string(),
                category: ActionCategory::Crud,
                enabled: true,
                disabled_reason: String::new(),
                destructive: false,
                keyboard_shortcut: String::new(),
            }];
            state.action_panel_selected_index = 0;
            state.action_panel_down();
            assert_eq!(state.action_panel_selected_index, 0);
        }

        #[test]
        fn test_error_queue_push() {
            let mut state = create_default_state();
            assert!(!state.has_errors());
            state.push_error("Error 1".to_string());
            assert!(state.has_errors());
        }

        #[test]
        fn test_error_queue_current() {
            let mut state = create_default_state();
            state.push_error("Error 1".to_string());
            state.push_error("Error 2".to_string());
            assert_eq!(state.current_error(), Some(&"Error 1".to_string()));
        }

        #[test]
        fn test_error_queue_dismiss() {
            let mut state = create_default_state();
            state.push_error("Error 1".to_string());
            state.push_error("Error 2".to_string());
            state.dismiss_error();
            assert_eq!(state.current_error(), Some(&"Error 2".to_string()));
        }

        #[test]
        fn test_is_action_panel_focused() {
            let mut state = create_default_state();

            state.current_view = View::Issues;
            state.issues_list_focus = IssuesListFocus::List;
            assert!(!state.is_action_panel_focused());

            state.issues_list_focus = IssuesListFocus::ActionPanel;
            assert!(state.is_action_panel_focused());
        }
    }
}
