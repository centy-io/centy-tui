//! Application state and core logic

use crate::daemon::DaemonClient;
use crate::state::{
    AppState, ButtonPressState, DocDetailFocus, DocsListFocus, EntityType, IssueDetailFocus,
    IssuesListFocus, LlmAction, LogoStyle, PrDetailFocus, PressedButton, PrsListFocus,
    ScreenBuffer, ScreenPos, SplashState, View, ViewParams,
};
use crate::ui::BUTTON_HEIGHT;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};

/// Main application struct
pub struct App {
    /// Current application state
    pub state: AppState,
    /// Daemon client for gRPC communication
    pub daemon: DaemonClient,
    /// Whether the app should quit
    quit: bool,
    /// Status message to display
    pub status_message: Option<String>,
    /// Copy feedback message
    pub copy_message: Option<String>,
    /// Splash screen animation state
    pub splash_state: Option<SplashState>,
    /// Terminal size for grid calculations (height, width)
    pub terminal_size: Option<(u16, u16)>,
    /// Screen buffer for text selection
    pub screen_buffer: ScreenBuffer,
    /// Timestamp of last Ctrl+C press for double-tap quit
    pub last_ctrl_c: Option<Instant>,
}

impl App {
    /// Create a new App instance
    #[allow(clippy::field_reassign_with_default)]
    pub async fn new() -> Result<Self> {
        let mut daemon = DaemonClient::new().await?;
        let mut state = AppState::default();

        // Start with splash screen
        state.current_view = View::Splash;

        // Check daemon connection
        state.daemon_connected = daemon.check_connection().await;

        // Load projects if connected
        if state.daemon_connected {
            if let Ok(projects) = daemon.list_projects().await {
                state.projects = projects;
            }
        }

        Ok(Self {
            state,
            daemon,
            quit: false,
            status_message: None,
            copy_message: None,
            splash_state: Some(SplashState::new(LogoStyle::default())),
            terminal_size: None,
            screen_buffer: ScreenBuffer::default(),
            last_ctrl_c: None,
        })
    }

    /// Update splash animation state
    /// Returns true if animation is complete and we should transition
    pub fn update_splash(&mut self, terminal_height: u16) -> bool {
        if let Some(ref mut splash) = self.splash_state {
            splash.update(terminal_height);
            if splash.is_complete() {
                self.splash_state = None;
                self.state.current_view = View::Projects;
                return true;
            }
        }
        false
    }

    /// Update button press animation state.
    /// Clears the animation if it has expired.
    pub fn update_button_press(&mut self) {
        if let Some(ref press) = self.state.button_press {
            if press.is_expired() {
                self.state.button_press = None;
            }
        }
    }

    /// Check if in splash screen
    pub fn in_splash(&self) -> bool {
        matches!(self.state.current_view, View::Splash)
    }

    /// Check if app should quit
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Get the sidebar width (0 if no project selected, 20 otherwise)
    pub fn sidebar_width(&self) -> u16 {
        if self.state.selected_project_path.is_some() {
            20
        } else {
            0
        }
    }

    /// Calculate number of columns for project grid based on terminal width
    pub fn calculate_project_grid_columns(&self) -> usize {
        // Use stored terminal size or default
        // terminal_size is (height, width)
        let width = self.terminal_size.map(|(_, w)| w).unwrap_or(80);

        // Subtract sidebar width (dynamic) and outer borders (2)
        let sidebar_width = self.sidebar_width();
        let usable_width = width.saturating_sub(sidebar_width + 2);

        const MIN_CARD_WIDTH: u16 = 18;
        const CARD_SPACING_H: u16 = 1;

        let columns = if usable_width >= MIN_CARD_WIDTH {
            ((usable_width + CARD_SPACING_H) / (MIN_CARD_WIDTH + CARD_SPACING_H)) as usize
        } else {
            1
        };
        columns.max(1)
    }

    /// Calculate visible height for project grid (inner content area)
    pub fn calculate_project_grid_visible_height(&self) -> usize {
        // terminal_size is (height, width)
        let height = self.terminal_size.map(|(h, _)| h).unwrap_or(24);
        // Subtract outer borders (2) for the projects block
        height.saturating_sub(2) as usize
    }

    /// Calculate which action was clicked based on mouse row position
    ///
    /// The action panel layout is:
    /// - Row 0: Outer border
    /// - For each category:
    ///   - 1 row: Category label
    ///   - N rows: Action buttons (BUTTON_HEIGHT each)
    /// - Remaining: Help text
    fn calculate_action_index_from_click(&self, mouse_row: u16) -> Option<usize> {
        if mouse_row < 1 {
            return None; // Click on border
        }

        let row_in_panel = mouse_row - 1; // Account for outer border
        let grouped = self.state.current_actions.grouped_actions();

        let mut current_row: u16 = 0;
        let mut action_idx = 0;

        for (_category, actions) in &grouped {
            // Category label takes 1 row
            current_row += 1;

            // Each action button takes BUTTON_HEIGHT rows
            for _ in actions {
                if row_in_panel >= current_row && row_in_panel < current_row + BUTTON_HEIGHT {
                    return Some(action_idx);
                }
                current_row += BUTTON_HEIGHT;
                action_idx += 1;
            }
        }

        None
    }

    /// Handle a key event
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle error dialog dismissal first (modal)
        if self.state.has_errors() {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                self.state.dismiss_error();
            }
            return Ok(());
        }

        // Clear any status messages on key press
        self.copy_message = None;

        // Handle keyboard text selection (Shift+arrows)
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                    self.handle_selection_key(key)?;
                    return Ok(());
                }
                _ => {}
            }
        }

        // Clear selection on Escape
        if key.code == KeyCode::Esc {
            self.state.selection.clear();
        }

        // Track view before handling key to detect navigation
        let view_before = self.state.current_view.clone();

        match self.state.current_view {
            View::Splash => self.handle_splash_key(key).await?,
            View::Projects => self.handle_projects_key(key).await?,
            View::Issues => self.handle_issues_key(key).await?,
            View::IssueDetail => self.handle_issue_detail_key(key).await?,
            View::IssueCreate => self.handle_issue_create_key(key).await?,
            View::IssueEdit => self.handle_issue_edit_key(key).await?,
            View::Prs => self.handle_prs_key(key).await?,
            View::PrDetail => self.handle_pr_detail_key(key).await?,
            View::PrCreate => self.handle_pr_create_key(key).await?,
            View::PrEdit => self.handle_pr_edit_key(key).await?,
            View::Docs => self.handle_docs_key(key).await?,
            View::DocDetail => self.handle_doc_detail_key(key).await?,
            View::DocCreate => self.handle_doc_create_key(key).await?,
            View::Config => self.handle_config_key(key).await?,
        }

        // Refresh actions if view changed to one that shows action panel
        if view_before != self.state.current_view {
            self.refresh_current_actions().await;
        }

        Ok(())
    }

    /// Navigate to a new view
    pub fn navigate(&mut self, view: View, params: ViewParams) {
        // Clear selection on view change
        self.state.selection.clear();
        // Save current view to history
        self.state.view_history.push((
            self.state.current_view.clone(),
            self.state.view_params.clone(),
        ));
        self.state.current_view = view;
        self.state.view_params = params;
    }

    /// Go back to previous view
    pub fn go_back(&mut self) {
        // Clear selection on view change
        self.state.selection.clear();
        // Skip form views in history to go back to the last non-form view
        while let Some((view, params)) = self.state.view_history.pop() {
            if view.is_form_view() {
                continue;
            }
            // Clear selected project when returning to Projects view
            if matches!(view, View::Projects) {
                self.state.selected_project_path = None;
            }
            self.state.current_view = view;
            self.state.view_params = params;
            return;
        }
    }

    /// Navigate to the detail view for a newly created item
    fn navigate_to_created_item(&mut self, view: View, params: ViewParams) {
        self.state.clear_form();
        self.navigate(view, params);
    }

    /// Handle keys in Projects view
    async fn handle_projects_key(&mut self, key: KeyEvent) -> Result<()> {
        // Calculate grid dimensions for navigation
        let columns = self.calculate_project_grid_columns();
        let visible_height = self.calculate_project_grid_visible_height();
        let total = self.state.selectable_projects().len();

        match key.code {
            // Vertical navigation (moves by row in grid, section-aware)
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down_grouped_grid(columns);
                self.state.ensure_selected_visible(columns, visible_height);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up_grouped_grid(columns);
                self.state.ensure_selected_visible(columns, visible_height);
            }
            // Horizontal navigation (moves within row in grid)
            KeyCode::Char('h') | KeyCode::Left => {
                self.state.move_selection_left(columns);
                self.state.ensure_selected_visible(columns, visible_height);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.state.move_selection_right(columns, total);
                self.state.ensure_selected_visible(columns, visible_height);
            }
            KeyCode::Enter => {
                let project_path = self
                    .state
                    .selectable_projects()
                    .get(self.state.selected_index)
                    .map(|p| p.path.clone());
                if let Some(path) = project_path {
                    self.state.selected_project_path = Some(path.clone());
                    if let Ok(issues) = self.daemon.list_issues(&path).await {
                        self.state.issues = issues;
                    }
                    if let Ok(config) = self.daemon.get_config(&path).await {
                        self.state.config = Some(config);
                    }
                    self.navigate(View::Issues, ViewParams::default());
                }
            }
            KeyCode::Char('f') => {
                let project_path = self
                    .state
                    .selectable_projects()
                    .get(self.state.selected_index)
                    .map(|p| p.path.clone());
                if let Some(path) = project_path {
                    if let Some(project) = self.state.projects.iter_mut().find(|p| p.path == path) {
                        let new_favorite = !project.is_favorite;
                        if self
                            .daemon
                            .set_project_favorite(&project.path, new_favorite)
                            .await
                            .is_ok()
                        {
                            project.is_favorite = new_favorite;
                            self.state.reset_selection();
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                let project_path = self
                    .state
                    .selectable_projects()
                    .get(self.state.selected_index)
                    .map(|p| p.path.clone());
                if let Some(path) = project_path {
                    if self.daemon.set_project_archived(&path, true).await.is_ok() {
                        if let Ok(projects) = self.daemon.list_projects().await {
                            self.state.projects = projects;
                            self.state.reset_selection();
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                self.state.confirm_action = Some("untrack".to_string());
            }
            KeyCode::Char('n') => {
                self.navigate(View::Projects, ViewParams::default());
            }
            KeyCode::Char('1') => self.state.sidebar_index = 0,
            KeyCode::Char('2') => {
                if self.state.selected_project_path.is_some() {
                    self.state.sidebar_index = 1;
                    self.navigate(View::Issues, ViewParams::default());
                }
            }
            KeyCode::Char('3') => {
                if self.state.selected_project_path.is_some() {
                    self.state.sidebar_index = 2;
                    self.navigate(View::Prs, ViewParams::default());
                }
            }
            KeyCode::Char('4') => {
                if self.state.selected_project_path.is_some() {
                    self.state.sidebar_index = 3;
                    self.navigate(View::Docs, ViewParams::default());
                }
            }
            KeyCode::Char('5') => {
                if self.state.selected_project_path.is_some() {
                    self.state.sidebar_index = 4;
                    self.navigate(View::Config, ViewParams::default());
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Issues view
    async fn handle_issues_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for dynamic action shortcut first (when focused on list)
        if matches!(self.state.issues_list_focus, IssuesListFocus::List) {
            if let Some(action_idx) = self.find_action_for_key(&key) {
                self.state.action_panel_selected_index = action_idx;
                self.execute_selected_dynamic_action().await?;
                return Ok(());
            }
        }

        match key.code {
            // Tab: Switch focus between list and action panel
            KeyCode::Tab => {
                self.state.issues_list_focus.toggle();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.issues_list_focus, IssuesListFocus::List) {
                    self.state
                        .move_selection_down(self.state.sorted_issues().len());
                } else {
                    // Navigate down in action panel
                    self.state.action_panel_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.issues_list_focus, IssuesListFocus::List) {
                    self.state.move_selection_up();
                } else {
                    // Navigate up in action panel
                    self.state.action_panel_up();
                }
            }
            KeyCode::Enter => {
                if matches!(self.state.issues_list_focus, IssuesListFocus::ActionPanel) {
                    self.execute_selected_dynamic_action().await?;
                } else {
                    // Open issue detail
                    let issue_id = self
                        .state
                        .sorted_issues()
                        .get(self.state.selected_index)
                        .map(|issue| issue.id.clone());
                    if let Some(id) = issue_id {
                        self.state.selected_issue_id = Some(id.clone());
                        self.navigate(
                            View::IssueDetail,
                            ViewParams {
                                issue_id: Some(id),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            KeyCode::Char('s') => {
                self.state.cycle_issue_sort_field();
            }
            KeyCode::Char('S') => {
                self.state.toggle_issue_sort_direction();
            }
            KeyCode::Char('a') => {
                self.state.show_closed_issues = !self.state.show_closed_issues;
                self.state.reset_selection();
            }
            KeyCode::Esc | KeyCode::Backspace => {
                // Reset focus state when leaving
                self.state.issues_list_focus = IssuesListFocus::List;
                self.state.action_panel_selected_index = 0;
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Delete the selected issue from the issues list
    async fn delete_selected_issue(&mut self) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.status_message = Some("No project selected".to_string());
                return Ok(());
            }
        };

        // Get the selected issue ID from the sorted list
        let issue_id = {
            let sorted = self.state.sorted_issues();
            sorted.get(self.state.selected_index).map(|i| i.id.clone())
        };

        let issue_id = match issue_id {
            Some(id) => id,
            None => {
                self.status_message = Some("No issue selected".to_string());
                return Ok(());
            }
        };

        // Delete the issue
        match self.daemon.delete_issue(&project_path, &issue_id).await {
            Ok(_) => {
                // Refresh issues list
                if let Ok(issues) = self.daemon.list_issues(&project_path).await {
                    self.state.issues = issues;
                    // Adjust selection if needed
                    let max = self.state.sorted_issues().len();
                    if self.state.selected_index >= max && max > 0 {
                        self.state.selected_index = max - 1;
                    }
                }
                self.status_message = Some("Issue deleted".to_string());
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to delete issue: {}", e));
            }
        }

        Ok(())
    }

    /// Handle keys in Issue Detail view
    async fn handle_issue_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for dynamic action shortcut first (when focused on content)
        if matches!(self.state.issue_detail_focus, IssueDetailFocus::Content) {
            if let Some(action_idx) = self.find_action_for_key(&key) {
                self.state.action_panel_selected_index = action_idx;
                self.execute_selected_dynamic_action().await?;
                return Ok(());
            }
        }

        match key.code {
            // Tab: Switch focus between content and action panel
            KeyCode::Tab => {
                self.state.issue_detail_focus.toggle();
            }
            // Edit issue
            KeyCode::Char('e') => {
                if let Some(issue_id) = &self.state.selected_issue_id {
                    self.navigate(
                        View::IssueEdit,
                        ViewParams {
                            issue_id: Some(issue_id.clone()),
                            ..Default::default()
                        },
                    );
                }
            }
            // Navigation (j/k/Up/Down)
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.issue_detail_focus, IssueDetailFocus::Content) {
                    self.state.scroll_down();
                } else {
                    // Navigate down in action panel
                    self.state.action_panel_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.issue_detail_focus, IssueDetailFocus::Content) {
                    self.state.scroll_up();
                } else {
                    // Navigate up in action panel
                    self.state.action_panel_up();
                }
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                if matches!(self.state.issue_detail_focus, IssueDetailFocus::Content) {
                    self.state.scroll_down_page();
                }
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                if matches!(self.state.issue_detail_focus, IssueDetailFocus::Content) {
                    self.state.scroll_up_page();
                }
            }
            // Execute action (Enter when action panel is focused)
            KeyCode::Enter => {
                if matches!(self.state.issue_detail_focus, IssueDetailFocus::ActionPanel) {
                    self.execute_selected_dynamic_action().await?;
                }
            }
            // Go back (also reset focus and action panel index)
            KeyCode::Esc | KeyCode::Backspace => {
                self.state.issue_detail_focus = IssueDetailFocus::Content;
                self.state.action_panel_selected_index = 0;
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Update the current issue's status
    async fn update_issue_status(&mut self, new_status: String) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.status_message = Some("No project selected".to_string());
                return Ok(());
            }
        };

        let issue_id = match &self.state.selected_issue_id {
            Some(id) => id.clone(),
            None => {
                self.status_message = Some("No issue selected".to_string());
                return Ok(());
            }
        };

        // Get current issue data
        let (title, description, priority) = {
            let issue = self.state.issues.iter().find(|i| i.id == issue_id);
            match issue {
                Some(i) => (i.title.clone(), i.description.clone(), i.metadata.priority),
                None => {
                    self.status_message = Some("Issue not found".to_string());
                    return Ok(());
                }
            }
        };

        // Update the issue
        match self
            .daemon
            .update_issue(
                &project_path,
                &issue_id,
                &title,
                &description,
                priority,
                &new_status,
            )
            .await
        {
            Ok(_) => {
                // Refresh issues list
                if let Ok(issues) = self.daemon.list_issues(&project_path).await {
                    self.state.issues = issues;
                }
                self.status_message = Some(format!("Status updated to '{}'", new_status));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to update status: {}", e));
            }
        }

        Ok(())
    }

    /// Update the current PR's status
    async fn update_pr_status(&mut self, new_status: String) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.status_message = Some("No project selected".to_string());
                return Ok(());
            }
        };

        let pr_id = match &self.state.selected_pr_id {
            Some(id) => id.clone(),
            None => {
                self.status_message = Some("No PR selected".to_string());
                return Ok(());
            }
        };

        // Get current PR data
        let (title, description, source_branch, target_branch) = {
            let pr = self.state.prs.iter().find(|p| p.id == pr_id);
            match pr {
                Some(p) => (
                    p.title.clone(),
                    p.description.clone(),
                    p.metadata.source_branch.clone(),
                    p.metadata.target_branch.clone(),
                ),
                None => {
                    self.status_message = Some("PR not found".to_string());
                    return Ok(());
                }
            }
        };

        // Update the PR
        match self
            .daemon
            .update_pr(
                &project_path,
                &pr_id,
                &title,
                &description,
                &source_branch,
                &target_branch,
                &new_status,
            )
            .await
        {
            Ok(()) => {
                // Refresh PRs list
                if let Ok(prs) = self.daemon.list_prs(&project_path).await {
                    self.state.prs = prs;
                }
                self.status_message = Some(format!("PR status updated to '{}'", new_status));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to update PR status: {}", e));
            }
        }

        Ok(())
    }

    /// Execute the "Open in VSCode" action
    async fn execute_open_in_vscode(&mut self) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.status_message = Some("No project selected".to_string());
                return Ok(());
            }
        };

        let issue_id = match &self.state.selected_issue_id {
            Some(id) => id.clone(),
            None => {
                self.status_message = Some("No issue selected".to_string());
                return Ok(());
            }
        };

        let action = self.state.action_panel_llm_action.as_proto_value();

        self.status_message = Some("Opening in VSCode...".to_string());

        match self
            .daemon
            .open_in_temp_vscode(&project_path, &issue_id, action, "", 0)
            .await
        {
            Ok(result) => {
                if result.vscode_opened {
                    self.status_message = Some(format!(
                        "Opened #{} in VSCode (expires: {})",
                        result.display_number,
                        result
                            .expires_at
                            .split('T')
                            .next()
                            .unwrap_or(&result.expires_at)
                    ));
                } else {
                    self.status_message =
                        Some(format!("Workspace created at {}", result.workspace_path));
                }
            }
            Err(e) => {
                let error_str = e.to_string();
                let error_lower = error_str.to_lowercase();
                let user_msg = if error_str.contains("detached HEAD") {
                    "Repository is in detached HEAD state.\nCheckout a branch first: git checkout <branch>".to_string()
                } else if error_lower.contains("worktree") {
                    "Failed to create git worktree.\nTry closing other VS Code windows for this project.".to_string()
                } else if error_lower.contains("not a git repository") {
                    "This project is not a git repository.\nInitialize with: git init".to_string()
                } else if error_lower.contains("not found") && error_lower.contains("vscode") {
                    "VS Code not found.\nInstall it and add 'code' to PATH.".to_string()
                } else if error_lower.contains("connection") {
                    "Cannot connect to centy daemon.\nIs it running? Try: centy daemon start"
                        .to_string()
                } else {
                    // Clean up nested error prefixes for unknown errors
                    error_str
                        .replace("Git error: ", "")
                        .replace("Worktree error: ", "")
                };
                self.state.error_dialog = Some(user_msg);
            }
        }

        Ok(())
    }

    /// Execute the "Open in Terminal" action
    async fn execute_open_in_terminal(&mut self) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.status_message = Some("No project selected".to_string());
                return Ok(());
            }
        };

        let issue_id = match &self.state.selected_issue_id {
            Some(id) => id.clone(),
            None => {
                self.status_message = Some("No issue selected".to_string());
                return Ok(());
            }
        };

        self.status_message = Some("Opening in terminal...".to_string());

        match self
            .daemon
            .open_agent_in_terminal(&project_path, &issue_id, "", 0, 0)
            .await
        {
            Ok(result) => {
                if result.terminal_opened {
                    self.status_message = Some(format!(
                        "Opened #{} in terminal with {}",
                        result.display_number, result.agent_command
                    ));
                } else {
                    self.status_message =
                        Some(format!("Agent ready at {}", result.working_directory));
                }
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to open: {}", e));
            }
        }

        Ok(())
    }

    // =========== Dynamic Actions ===========

    /// Fetch entity actions from daemon
    pub async fn fetch_entity_actions(&mut self, entity_type: EntityType, entity_id: Option<&str>) {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => return,
        };

        self.state.actions_loading = true;
        self.state.actions_error = None;

        match self
            .daemon
            .get_entity_actions(&project_path, entity_type, entity_id)
            .await
        {
            Ok(response) => {
                self.state.current_actions = response;
                self.state.actions_loading = false;
                self.state.action_panel_selected_index = 0;
            }
            Err(e) => {
                self.state.actions_error = Some(e.to_string());
                self.state.actions_loading = false;
                self.state.current_actions = Default::default();
            }
        }
    }

    /// Refresh actions for current view
    pub async fn refresh_current_actions(&mut self) {
        match self.state.current_view {
            View::Issues => {
                let entity_id = self
                    .state
                    .sorted_issues()
                    .get(self.state.selected_index)
                    .map(|i| i.id.clone());
                self.fetch_entity_actions(EntityType::Issue, entity_id.as_deref())
                    .await;
            }
            View::IssueDetail => {
                let entity_id = self.state.selected_issue_id.clone();
                self.fetch_entity_actions(EntityType::Issue, entity_id.as_deref())
                    .await;
            }
            View::Prs => {
                let entity_id = self
                    .state
                    .sorted_prs()
                    .get(self.state.selected_index)
                    .map(|p| p.id.clone());
                self.fetch_entity_actions(EntityType::Pr, entity_id.as_deref())
                    .await;
            }
            View::PrDetail => {
                let entity_id = self.state.selected_pr_id.clone();
                self.fetch_entity_actions(EntityType::Pr, entity_id.as_deref())
                    .await;
            }
            View::Docs => {
                let entity_id = self
                    .state
                    .docs
                    .get(self.state.selected_index)
                    .map(|d| d.slug.clone());
                self.fetch_entity_actions(EntityType::Doc, entity_id.as_deref())
                    .await;
            }
            View::DocDetail => {
                let entity_id = self.state.selected_doc_slug.clone();
                self.fetch_entity_actions(EntityType::Doc, entity_id.as_deref())
                    .await;
            }
            _ => {}
        }
    }

    /// Check if a key event matches an action's keyboard shortcut
    fn key_matches_shortcut(key: &KeyEvent, shortcut: &str) -> bool {
        if shortcut.is_empty() {
            return false;
        }

        // Parse shortcut format: "n", "d", "Ctrl+D", "Shift+N", "Enter"
        let parts: Vec<&str> = shortcut.split('+').collect();

        let (expected_modifiers, key_str) = if parts.len() == 2 {
            let mods = match parts[0].to_lowercase().as_str() {
                "ctrl" | "control" => KeyModifiers::CONTROL,
                "shift" => KeyModifiers::SHIFT,
                "alt" => KeyModifiers::ALT,
                _ => KeyModifiers::NONE,
            };
            (mods, parts[1])
        } else {
            (KeyModifiers::NONE, parts[0])
        };

        // Check modifiers
        if key.modifiers != expected_modifiers {
            return false;
        }

        // Check key
        match key_str.to_lowercase().as_str() {
            "enter" => matches!(key.code, KeyCode::Enter),
            "esc" | "escape" => matches!(key.code, KeyCode::Esc),
            "tab" => matches!(key.code, KeyCode::Tab),
            "backspace" => matches!(key.code, KeyCode::Backspace),
            "delete" => matches!(key.code, KeyCode::Delete),
            s if s.len() == 1 => {
                if let Some(c) = s.chars().next() {
                    matches!(key.code, KeyCode::Char(k) if k.to_ascii_lowercase() == c)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Find action matching a key event
    fn find_action_for_key(&self, key: &KeyEvent) -> Option<usize> {
        self.state
            .current_actions
            .actions
            .iter()
            .position(|a| Self::key_matches_shortcut(key, &a.keyboard_shortcut))
    }

    /// Execute the currently selected dynamic action
    pub async fn execute_selected_dynamic_action(&mut self) -> Result<()> {
        let action = match self.state.selected_action() {
            Some(a) => a.clone(),
            None => return Ok(()),
        };

        if !action.enabled {
            self.status_message = Some(action.disabled_reason.clone());
            return Ok(());
        }

        // Route to appropriate handler based on action ID
        // Action IDs from daemon are generic (e.g., "create", "delete")
        // We determine the entity type from the current view
        match action.id.as_str() {
            // Create action - contextual based on current view
            "create" => match self.state.current_view {
                View::Issues | View::IssueDetail => {
                    self.navigate(View::IssueCreate, ViewParams::default());
                }
                View::Prs | View::PrDetail => {
                    self.navigate(View::PrCreate, ViewParams::default());
                }
                View::Docs | View::DocDetail => {
                    self.navigate(View::DocCreate, ViewParams::default());
                }
                _ => {}
            },

            // Delete action - contextual based on current view
            "delete" => match self.state.current_view {
                View::Issues | View::IssueDetail => {
                    self.delete_selected_issue().await?;
                }
                View::Prs | View::PrDetail => {
                    self.status_message = Some("Delete PR: Not yet implemented".to_string());
                }
                View::Docs | View::DocDetail => {
                    self.status_message = Some("Delete doc: Not yet implemented".to_string());
                }
                _ => {}
            },

            // Duplicate action - contextual based on current view
            "duplicate" => match self.state.current_view {
                View::Issues | View::IssueDetail => {
                    self.status_message = Some("Duplicate issue: Not yet implemented".to_string());
                }
                View::Docs | View::DocDetail => {
                    self.status_message = Some("Duplicate doc: Not yet implemented".to_string());
                }
                _ => {}
            },

            // Move action - contextual based on current view
            "move" => match self.state.current_view {
                View::Issues | View::IssueDetail => {
                    self.status_message = Some("Move issue: Not yet implemented".to_string());
                }
                View::Docs | View::DocDetail => {
                    self.status_message = Some("Move doc: Not yet implemented".to_string());
                }
                _ => {}
            },

            // Mode actions (Issue-specific)
            "mode:plan" => {
                self.state.action_panel_llm_action = LlmAction::Plan;
                self.status_message = Some("Mode set to Plan".to_string());
            }
            "mode:implement" => {
                self.state.action_panel_llm_action = LlmAction::Implement;
                self.status_message = Some("Mode set to Implement".to_string());
            }

            // External actions
            "open_in_vscode" => {
                self.execute_open_in_vscode().await?;
            }
            "open_in_terminal" => {
                self.execute_open_in_terminal().await?;
            }

            // Status transitions (dynamic, e.g., "status:open", "status:closed")
            id if id.starts_with("status:") => {
                let new_status = id.strip_prefix("status:").unwrap_or("");
                match self.state.current_view {
                    View::Issues | View::IssueDetail => {
                        self.update_issue_status(new_status.to_string()).await?;
                    }
                    View::Prs | View::PrDetail => {
                        self.update_pr_status(new_status.to_string()).await?;
                    }
                    _ => {}
                }
            }

            _ => {
                self.status_message = Some(format!("Unknown action: {}", action.id));
            }
        }

        // Refresh actions after execution (state may have changed)
        self.refresh_current_actions().await;

        Ok(())
    }

    /// Handle keys in Issue Create view
    async fn handle_issue_create_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check if we're on the action panel (field 3)
        let on_action_panel = self.state.active_form_field == 3;

        match key.code {
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            // Up/Down for action panel navigation
            KeyCode::Up | KeyCode::Char('k') if on_action_panel => {
                if self.state.form_selected_button == 0 {
                    self.state.form_selected_button = 3;
                } else {
                    self.state.form_selected_button -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if on_action_panel => {
                self.state.form_selected_button = (self.state.form_selected_button + 1) % 4;
            }
            // Enter on action panel triggers selected button
            // Button order: 0=Create, 1=Create&New, 2=Draft, 3=Cancel
            KeyCode::Enter if on_action_panel => {
                match self.state.form_selected_button {
                    0 => {
                        // Create
                        self.create_issue_with_options(false, false).await;
                    }
                    1 => {
                        // Create & New
                        self.create_issue_with_options(false, true).await;
                    }
                    2 => {
                        // Save as Draft
                        self.create_issue_with_options(true, false).await;
                    }
                    3 => {
                        // Cancel
                        self.state.clear_form();
                        self.go_back();
                    }
                    _ => {}
                }
            }
            // Keyboard shortcuts (work from anywhere)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.create_issue_with_options(false, false).await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.create_issue_with_options(false, false).await;
            }
            KeyCode::Char('d') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.create_issue_with_options(true, false).await;
            }
            KeyCode::Char('n') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.create_issue_with_options(false, true).await;
            }
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            // Form field input (only when not on action panel)
            KeyCode::Char(c) if !on_action_panel => self
                .state
                .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Backspace if !on_action_panel => self.state.form_backspace(),
            KeyCode::Enter if !on_action_panel => {
                // Enter in description field adds newline
                if self.state.active_form_field == 1 {
                    self.state.form_description.push('\n');
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper function to create an issue with different options
    async fn create_issue_with_options(&mut self, draft: bool, create_new: bool) {
        if let Some(path) = self.state.selected_project_path.clone() {
            let result = self
                .daemon
                .create_issue(
                    &path,
                    &self.state.form_title,
                    &self.state.form_description,
                    self.state.form_priority,
                    draft,
                )
                .await;
            if let Ok(new_id) = result {
                if let Ok(issues) = self.daemon.list_issues(&path).await {
                    self.state.issues = issues;
                }

                if create_new {
                    // Clear form for next issue
                    self.state.clear_form();
                    self.status_message = Some("Issue created! Ready for next issue.".to_string());
                } else {
                    self.state.selected_issue_id = Some(new_id.clone());
                    let msg = if draft {
                        "Draft saved!"
                    } else {
                        "Issue created!"
                    };
                    self.status_message = Some(msg.to_string());
                    self.navigate_to_created_item(
                        View::IssueDetail,
                        ViewParams {
                            issue_id: Some(new_id),
                            ..Default::default()
                        },
                    );
                }
            } else {
                self.status_message = Some("Failed to create issue".to_string());
            }
        } else {
            self.status_message = Some("No project selected".to_string());
        }
    }

    /// Handle keys in Issue Edit view
    async fn handle_issue_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            // Save (Ctrl+S or Cmd+W / Ctrl+W)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_issue_edit().await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.save_issue_edit().await;
            }
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            KeyCode::Char(c) => self
                .state
                .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Backspace => self.state.form_backspace(),
            KeyCode::Enter => {
                if self.state.active_form_field == 1 {
                    self.state.form_description.push('\n');
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper to save issue edit
    async fn save_issue_edit(&mut self) {
        match (
            &self.state.selected_project_path,
            &self.state.selected_issue_id,
        ) {
            (Some(path), Some(issue_id)) => {
                let result = self
                    .daemon
                    .update_issue(
                        path,
                        issue_id,
                        &self.state.form_title,
                        &self.state.form_description,
                        self.state.form_priority,
                        &self.state.form_status,
                    )
                    .await;
                if result.is_ok() {
                    if let Ok(issues) = self.daemon.list_issues(path).await {
                        self.state.issues = issues;
                    }
                    self.state.clear_form();
                    self.go_back();
                } else {
                    self.status_message = Some("Failed to update issue".to_string());
                }
            }
            (None, _) => {
                self.status_message = Some("No project selected".to_string());
            }
            (_, None) => {
                self.status_message = Some("No issue selected".to_string());
            }
        }
    }

    /// Handle keys in PRs view
    async fn handle_prs_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for dynamic action shortcut first (when focused on list)
        if matches!(self.state.prs_list_focus, PrsListFocus::List) {
            if let Some(action_idx) = self.find_action_for_key(&key) {
                self.state.action_panel_selected_index = action_idx;
                self.execute_selected_dynamic_action().await?;
                return Ok(());
            }
        }

        match key.code {
            // Tab: Switch focus between list and action panel
            KeyCode::Tab => {
                self.state.prs_list_focus.toggle();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.prs_list_focus, PrsListFocus::List) {
                    self.state
                        .move_selection_down(self.state.sorted_prs().len());
                } else {
                    // Navigate down in action panel
                    self.state.action_panel_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.prs_list_focus, PrsListFocus::List) {
                    self.state.move_selection_up();
                } else {
                    // Navigate up in action panel
                    self.state.action_panel_up();
                }
            }
            KeyCode::Enter => {
                if matches!(self.state.prs_list_focus, PrsListFocus::ActionPanel) {
                    self.execute_selected_dynamic_action().await?;
                } else {
                    // Open PR detail
                    let pr_id = self
                        .state
                        .sorted_prs()
                        .get(self.state.selected_index)
                        .map(|pr| pr.id.clone());
                    if let Some(id) = pr_id {
                        self.state.selected_pr_id = Some(id.clone());
                        self.navigate(
                            View::PrDetail,
                            ViewParams {
                                pr_id: Some(id),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            KeyCode::Char('s') => self.state.cycle_pr_sort_field(),
            KeyCode::Char('S') => self.state.toggle_pr_sort_direction(),
            KeyCode::Char('a') => {
                self.state.show_merged_prs = !self.state.show_merged_prs;
                self.state.reset_selection();
            }
            KeyCode::Esc | KeyCode::Backspace => {
                // Reset focus state when leaving
                self.state.prs_list_focus = PrsListFocus::List;
                self.state.action_panel_selected_index = 0;
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in PR Detail view
    async fn handle_pr_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for dynamic action shortcut first (when focused on content)
        if matches!(self.state.pr_detail_focus, PrDetailFocus::Content) {
            if let Some(action_idx) = self.find_action_for_key(&key) {
                self.state.action_panel_selected_index = action_idx;
                self.execute_selected_dynamic_action().await?;
                return Ok(());
            }
        }

        match key.code {
            // Tab: Switch focus between content and action panel
            KeyCode::Tab => {
                self.state.pr_detail_focus.toggle();
            }
            // Edit PR
            KeyCode::Char('e') => {
                if let Some(pr_id) = &self.state.selected_pr_id {
                    self.navigate(
                        View::PrEdit,
                        ViewParams {
                            pr_id: Some(pr_id.clone()),
                            ..Default::default()
                        },
                    );
                }
            }
            // Navigation (j/k/Up/Down)
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.pr_detail_focus, PrDetailFocus::Content) {
                    self.state.scroll_down();
                } else {
                    // Navigate down in action panel
                    self.state.action_panel_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.pr_detail_focus, PrDetailFocus::Content) {
                    self.state.scroll_up();
                } else {
                    // Navigate up in action panel
                    self.state.action_panel_up();
                }
            }
            // Execute action (Enter when action panel is focused)
            KeyCode::Enter => {
                if matches!(self.state.pr_detail_focus, PrDetailFocus::ActionPanel) {
                    self.execute_selected_dynamic_action().await?;
                }
            }
            // Go back (also reset focus and action panel index)
            KeyCode::Esc | KeyCode::Backspace => {
                self.state.pr_detail_focus = PrDetailFocus::Content;
                self.state.action_panel_selected_index = 0;
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in PR Create view
    async fn handle_pr_create_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            // Save (Ctrl+S or Cmd+W / Ctrl+W)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_pr_create().await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.save_pr_create().await;
            }
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            KeyCode::Char(c) => self
                .state
                .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Backspace => self.state.form_backspace(),
            _ => {}
        }
        Ok(())
    }

    /// Helper to save PR create
    async fn save_pr_create(&mut self) {
        if let Some(path) = &self.state.selected_project_path {
            let result = self
                .daemon
                .create_pr(
                    path,
                    &self.state.form_title,
                    &self.state.form_description,
                    &self.state.form_source_branch,
                    &self.state.form_target_branch,
                )
                .await;
            if let Ok(new_id) = result {
                if let Ok(prs) = self.daemon.list_prs(path).await {
                    self.state.prs = prs;
                }
                self.state.selected_pr_id = Some(new_id.clone());
                self.navigate_to_created_item(
                    View::PrDetail,
                    ViewParams {
                        pr_id: Some(new_id),
                        ..Default::default()
                    },
                );
            } else {
                self.status_message = Some("Failed to create PR".to_string());
            }
        } else {
            self.status_message = Some("No project selected".to_string());
        }
    }

    /// Handle keys in PR Edit view
    async fn handle_pr_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            // Save (Ctrl+S or Cmd+W / Ctrl+W)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_pr_edit().await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.save_pr_edit().await;
            }
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            KeyCode::Char(c) => self
                .state
                .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Backspace => self.state.form_backspace(),
            _ => {}
        }
        Ok(())
    }

    /// Helper to save PR edit
    async fn save_pr_edit(&mut self) {
        match (
            &self.state.selected_project_path,
            &self.state.selected_pr_id,
        ) {
            (Some(path), Some(pr_id)) => {
                let result = self
                    .daemon
                    .update_pr(
                        path,
                        pr_id,
                        &self.state.form_title,
                        &self.state.form_description,
                        &self.state.form_source_branch,
                        &self.state.form_target_branch,
                        &self.state.form_status,
                    )
                    .await;
                if result.is_ok() {
                    if let Ok(prs) = self.daemon.list_prs(path).await {
                        self.state.prs = prs;
                    }
                    self.state.clear_form();
                    self.go_back();
                } else {
                    self.status_message = Some("Failed to update PR".to_string());
                }
            }
            (None, _) => {
                self.status_message = Some("No project selected".to_string());
            }
            (_, None) => {
                self.status_message = Some("No PR selected".to_string());
            }
        }
    }

    /// Handle keys in Docs view
    async fn handle_docs_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for dynamic action shortcut first (when focused on list)
        if matches!(self.state.docs_list_focus, DocsListFocus::List) {
            if let Some(action_idx) = self.find_action_for_key(&key) {
                self.state.action_panel_selected_index = action_idx;
                self.execute_selected_dynamic_action().await?;
                return Ok(());
            }
        }

        match key.code {
            // Tab: Switch focus between list and action panel
            KeyCode::Tab => {
                self.state.docs_list_focus.toggle();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.docs_list_focus, DocsListFocus::List) {
                    self.state.move_selection_down(self.state.docs.len());
                } else {
                    // Navigate down in action panel
                    self.state.action_panel_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.docs_list_focus, DocsListFocus::List) {
                    self.state.move_selection_up();
                } else {
                    // Navigate up in action panel
                    self.state.action_panel_up();
                }
            }
            KeyCode::Enter => {
                if matches!(self.state.docs_list_focus, DocsListFocus::ActionPanel) {
                    self.execute_selected_dynamic_action().await?;
                } else {
                    // Open doc detail
                    if let Some(doc) = self.state.docs.get(self.state.selected_index) {
                        self.state.selected_doc_slug = Some(doc.slug.clone());
                        self.navigate(
                            View::DocDetail,
                            ViewParams {
                                doc_slug: Some(doc.slug.clone()),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                // Reset focus state when leaving
                self.state.docs_list_focus = DocsListFocus::List;
                self.state.action_panel_selected_index = 0;
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Doc Detail view
    async fn handle_doc_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for dynamic action shortcut first (when focused on content)
        if matches!(self.state.doc_detail_focus, DocDetailFocus::Content) {
            if let Some(action_idx) = self.find_action_for_key(&key) {
                self.state.action_panel_selected_index = action_idx;
                self.execute_selected_dynamic_action().await?;
                return Ok(());
            }
        }

        match key.code {
            // Tab: Switch focus between content and action panel
            KeyCode::Tab => {
                self.state.doc_detail_focus.toggle();
            }
            // Navigation (j/k/Up/Down)
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.doc_detail_focus, DocDetailFocus::Content) {
                    self.state.scroll_down();
                } else {
                    // Navigate down in action panel
                    self.state.action_panel_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.doc_detail_focus, DocDetailFocus::Content) {
                    self.state.scroll_up();
                } else {
                    // Navigate up in action panel
                    self.state.action_panel_up();
                }
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                if matches!(self.state.doc_detail_focus, DocDetailFocus::Content) {
                    self.state.scroll_down_page();
                }
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                if matches!(self.state.doc_detail_focus, DocDetailFocus::Content) {
                    self.state.scroll_up_page();
                }
            }
            // Execute action (Enter when action panel is focused)
            KeyCode::Enter => {
                if matches!(self.state.doc_detail_focus, DocDetailFocus::ActionPanel) {
                    self.execute_selected_dynamic_action().await?;
                }
            }
            // Go back (also reset focus and action panel index)
            KeyCode::Esc | KeyCode::Backspace => {
                self.state.doc_detail_focus = DocDetailFocus::Content;
                self.state.action_panel_selected_index = 0;
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Doc Create view
    async fn handle_doc_create_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            // Save (Ctrl+S or Cmd+W / Ctrl+W)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_doc_create().await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.save_doc_create().await;
            }
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            KeyCode::Char(c) => self
                .state
                .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Backspace => self.state.form_backspace(),
            KeyCode::Enter => {
                if self.state.active_form_field == 1 {
                    self.state.form_description.push('\n');
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper to save doc create
    async fn save_doc_create(&mut self) {
        if let Some(path) = &self.state.selected_project_path {
            let slug = if self.state.form_slug.is_empty() {
                None
            } else {
                Some(self.state.form_slug.as_str())
            };
            let result = self
                .daemon
                .create_doc(
                    path,
                    &self.state.form_title,
                    &self.state.form_description,
                    slug,
                )
                .await;
            if let Ok(new_slug) = result {
                if let Ok(docs) = self.daemon.list_docs(path).await {
                    self.state.docs = docs;
                }
                self.state.selected_doc_slug = Some(new_slug.clone());
                self.navigate_to_created_item(
                    View::DocDetail,
                    ViewParams {
                        doc_slug: Some(new_slug),
                        ..Default::default()
                    },
                );
            } else {
                self.status_message = Some("Failed to create doc".to_string());
            }
        } else {
            self.status_message = Some("No project selected".to_string());
        }
    }

    /// Handle keys in Config view
    async fn handle_config_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.state.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.state.scroll_up(),
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Splash screen
    async fn handle_splash_key(&mut self, _key: KeyEvent) -> Result<()> {
        if let Some(ref mut splash) = self.splash_state {
            splash.skip();
        }
        Ok(())
    }

    /// Handle mouse events
    pub async fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        self.copy_message = None;

        // Handle text selection (drag and release)
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Start a new selection on mouse down
                // Check if Shift is held to extend selection
                if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                    if self.state.selection.anchor.is_some() {
                        self.state
                            .selection
                            .update(ScreenPos::new(mouse.column, mouse.row));
                    } else {
                        self.state
                            .selection
                            .start(ScreenPos::new(mouse.column, mouse.row));
                    }
                } else {
                    self.state
                        .selection
                        .start(ScreenPos::new(mouse.column, mouse.row));
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                // Update selection endpoint during drag
                if self.state.selection.is_selecting {
                    self.state
                        .selection
                        .update(ScreenPos::new(mouse.column, mouse.row));
                }
                // Don't process normal UI events during drag - just update selection
                return Ok(());
            }
            MouseEventKind::Up(MouseButton::Left) => {
                // Finish selection on mouse up
                if self.state.selection.is_selecting {
                    self.state.selection.finish();
                }
            }
            _ => {}
        }

        // Track view before handling mouse to detect navigation
        let view_before = self.state.current_view.clone();

        // Only check sidebar mouse if sidebar is visible (project selected and not in form view)
        let has_project = self.state.selected_project_path.is_some();
        if has_project
            && self.state.current_view != View::Splash
            && !self.state.current_view.is_form_view()
            && self.handle_sidebar_mouse(mouse).await?
        {
            // Refresh actions if view changed via sidebar
            if view_before != self.state.current_view {
                self.refresh_current_actions().await;
            }
            return Ok(());
        }
        match self.state.current_view {
            View::Splash => self.handle_splash_mouse(mouse).await?,
            View::Projects => self.handle_projects_grid_mouse(mouse).await?,
            View::Issues => {
                let len = self.state.sorted_issues().len();
                self.handle_list_mouse(mouse, len).await?
            }
            View::IssueDetail => self.handle_scroll_mouse(mouse).await?,
            View::IssueCreate | View::IssueEdit => self.handle_form_mouse(mouse).await?,
            View::Prs => {
                let len = self.state.sorted_prs().len();
                self.handle_list_mouse(mouse, len).await?
            }
            View::PrDetail => self.handle_scroll_mouse(mouse).await?,
            View::PrCreate | View::PrEdit => self.handle_form_mouse(mouse).await?,
            View::Docs => self.handle_list_mouse(mouse, self.state.docs.len()).await?,
            View::DocDetail => self.handle_scroll_mouse(mouse).await?,
            View::DocCreate => self.handle_form_mouse(mouse).await?,
            View::Config => self.handle_scroll_mouse(mouse).await?,
        }

        // Refresh actions if view changed to one that shows action panel
        if view_before != self.state.current_view {
            self.refresh_current_actions().await;
        }

        Ok(())
    }

    async fn handle_sidebar_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        const SIDEBAR_WIDTH: u16 = 20;
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if mouse.column < SIDEBAR_WIDTH {
                let item_index = (mouse.row / BUTTON_HEIGHT) as usize;
                let has_project = self.state.selected_project_path.is_some();
                match item_index {
                    0 => {
                        self.state.button_press =
                            Some(ButtonPressState::new(PressedButton::Sidebar(0)));
                        self.state.sidebar_index = 0;
                        self.navigate(View::Projects, ViewParams::default());
                        return Ok(true);
                    }
                    1 if has_project => {
                        self.state.button_press =
                            Some(ButtonPressState::new(PressedButton::Sidebar(1)));
                        self.state.sidebar_index = 1;
                        self.navigate(View::Issues, ViewParams::default());
                        return Ok(true);
                    }
                    2 if has_project => {
                        self.state.button_press =
                            Some(ButtonPressState::new(PressedButton::Sidebar(2)));
                        self.state.sidebar_index = 2;
                        self.navigate(View::Prs, ViewParams::default());
                        return Ok(true);
                    }
                    3 if has_project => {
                        self.state.button_press =
                            Some(ButtonPressState::new(PressedButton::Sidebar(3)));
                        self.state.sidebar_index = 3;
                        self.navigate(View::Docs, ViewParams::default());
                        return Ok(true);
                    }
                    4 if has_project => {
                        self.state.button_press =
                            Some(ButtonPressState::new(PressedButton::Sidebar(4)));
                        self.state.sidebar_index = 4;
                        self.navigate(View::Config, ViewParams::default());
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }

    /// Handle mouse events in list views (Issues, PRs, Docs)
    async fn handle_list_mouse(&mut self, mouse: MouseEvent, list_len: usize) -> Result<()> {
        let main_area_start_x = self.sidebar_width();
        const LIST_ITEMS_START_Y: u16 = 3;
        const ACTION_PANEL_WIDTH: u16 = 22;

        match mouse.kind {
            MouseEventKind::ScrollUp => self.state.move_selection_up(),
            MouseEventKind::ScrollDown => self.state.move_selection_down(list_len),
            MouseEventKind::Down(MouseButton::Left) => {
                let terminal_width = self.terminal_size.map(|(_, w)| w).unwrap_or(80);
                let action_panel_start_x = terminal_width.saturating_sub(ACTION_PANEL_WIDTH);

                // Check if click is in the action panel area
                if mouse.column >= action_panel_start_x {
                    // Handle action panel click
                    if let Some(action_idx) = self.calculate_action_index_from_click(mouse.row) {
                        let total_actions = self.state.current_actions.actions.len();
                        if action_idx < total_actions {
                            let is_enabled = self
                                .state
                                .current_actions
                                .actions
                                .get(action_idx)
                                .map(|a| a.enabled)
                                .unwrap_or(false);

                            if is_enabled {
                                // Trigger button press animation
                                self.state.button_press = Some(ButtonPressState::new(
                                    PressedButton::ActionPanel(action_idx),
                                ));
                                self.state.action_panel_selected_index = action_idx;
                                self.execute_selected_dynamic_action().await?;
                            }
                        }
                    }
                    return Ok(());
                }

                if mouse.column >= main_area_start_x && mouse.row >= LIST_ITEMS_START_Y {
                    let clicked_index = (mouse.row - LIST_ITEMS_START_Y) as usize;
                    if clicked_index < list_len {
                        // Check for double-click: same index clicked within 400ms
                        let is_double_click = self
                            .state
                            .last_click_index
                            .map(|last_idx| {
                                last_idx == clicked_index
                                    && self
                                        .state
                                        .last_click_time
                                        .map(|t| t.elapsed() < Duration::from_millis(400))
                                        .unwrap_or(false)
                            })
                            .unwrap_or(false);

                        if is_double_click {
                            // Double-click: open the item
                            match self.state.current_view {
                                View::Issues => {
                                    let issue_id = self
                                        .state
                                        .sorted_issues()
                                        .get(clicked_index)
                                        .map(|i| i.id.clone());
                                    if let Some(id) = issue_id {
                                        self.state.selected_issue_id = Some(id.clone());
                                        self.navigate(
                                            View::IssueDetail,
                                            ViewParams {
                                                issue_id: Some(id),
                                                ..Default::default()
                                            },
                                        );
                                    }
                                }
                                View::Prs => {
                                    let pr_id = self
                                        .state
                                        .sorted_prs()
                                        .get(clicked_index)
                                        .map(|p| p.id.clone());
                                    if let Some(id) = pr_id {
                                        self.state.selected_pr_id = Some(id.clone());
                                        self.navigate(
                                            View::PrDetail,
                                            ViewParams {
                                                pr_id: Some(id),
                                                ..Default::default()
                                            },
                                        );
                                    }
                                }
                                View::Docs => {
                                    let doc_slug =
                                        self.state.docs.get(clicked_index).map(|d| d.slug.clone());
                                    if let Some(slug) = doc_slug {
                                        self.state.selected_doc_slug = Some(slug.clone());
                                        self.navigate(
                                            View::DocDetail,
                                            ViewParams {
                                                doc_slug: Some(slug),
                                                ..Default::default()
                                            },
                                        );
                                    }
                                }
                                _ => {}
                            }
                            // Reset click tracking after opening
                            self.state.last_click_time = None;
                            self.state.last_click_index = None;
                        } else {
                            // Single click: select the item and update tracking
                            self.state.selected_index = clicked_index;
                            self.state.last_click_time = Some(Instant::now());
                            self.state.last_click_index = Some(clicked_index);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse events in Projects grid view
    async fn handle_projects_grid_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        let main_area_start_x = self.sidebar_width();
        const GRID_START_Y: u16 = 2; // After outer border (1) + inner content start
        const MIN_CARD_WIDTH: u16 = 18;
        const CARD_HEIGHT: u16 = 4;
        const CARD_SPACING_H: u16 = 1;
        const SECTION_HEADER_HEIGHT: u16 = 2;

        let columns = self.calculate_project_grid_columns();
        let visible_height = self.calculate_project_grid_visible_height();
        let total = self.state.selectable_projects().len();

        if total == 0 {
            return Ok(());
        }

        // Calculate card width based on available space
        let terminal_width = self.terminal_size.map(|(_, w)| w).unwrap_or(80);
        let usable_width = terminal_width.saturating_sub(main_area_start_x + 2);
        let total_spacing = (columns.saturating_sub(1) as u16) * CARD_SPACING_H;
        let card_width = if columns > 0 {
            (usable_width.saturating_sub(total_spacing)) / columns as u16
        } else {
            usable_width
        };
        let card_width = card_width.max(MIN_CARD_WIDTH);

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.state.move_selection_up_grouped_grid(columns);
                self.state.ensure_selected_visible(columns, visible_height);
            }
            MouseEventKind::ScrollDown => {
                self.state.move_selection_down_grouped_grid(columns);
                self.state.ensure_selected_visible(columns, visible_height);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if mouse.column >= main_area_start_x && mouse.row >= GRID_START_Y {
                    let rel_x = mouse.column - main_area_start_x - 1; // -1 for border
                                                                      // Account for scroll offset when calculating click position
                    let scroll_offset = self.state.scroll_offset as u16;
                    let rel_y = mouse.row - GRID_START_Y + scroll_offset;

                    // Calculate which card was clicked, accounting for section headers
                    let sections = self.state.grouped_projects();
                    let mut y_offset: u16 = 0;
                    let mut global_project_index: usize = 0;
                    let mut clicked_index: Option<usize> = None;

                    for section in &sections {
                        // Skip section header area
                        y_offset += SECTION_HEADER_HEIGHT;

                        let section_size = section.projects.len();
                        let rows_in_section = section_size.div_ceil(columns);
                        let section_height = (rows_in_section as u16) * CARD_HEIGHT;

                        // Check if click is in this section's project area
                        if rel_y >= y_offset && rel_y < y_offset + section_height {
                            let rel_y_in_section = rel_y - y_offset;
                            let row_in_section = (rel_y_in_section / CARD_HEIGHT) as usize;
                            let col = (rel_x / (card_width + CARD_SPACING_H)) as usize;

                            if col < columns {
                                let idx_in_section = row_in_section * columns + col;
                                if idx_in_section < section_size {
                                    clicked_index = Some(global_project_index + idx_in_section);
                                }
                            }
                            break;
                        }

                        y_offset += section_height;
                        global_project_index += section_size;
                    }

                    if let Some(clicked_idx) = clicked_index {
                        // Check for double-click: same index clicked within 400ms
                        let is_double_click = self
                            .state
                            .last_click_index
                            .map(|last_idx| {
                                last_idx == clicked_idx
                                    && self
                                        .state
                                        .last_click_time
                                        .map(|t| t.elapsed() < Duration::from_millis(400))
                                        .unwrap_or(false)
                            })
                            .unwrap_or(false);

                        if is_double_click {
                            // Double-click: open the project
                            let project_path = self
                                .state
                                .selectable_projects()
                                .get(clicked_idx)
                                .map(|p| p.path.clone());
                            if let Some(path) = project_path {
                                self.state.selected_project_path = Some(path.clone());
                                if let Ok(issues) = self.daemon.list_issues(&path).await {
                                    self.state.issues = issues;
                                }
                                if let Ok(config) = self.daemon.get_config(&path).await {
                                    self.state.config = Some(config);
                                }
                                self.navigate(View::Issues, ViewParams::default());
                            }
                            // Reset click tracking after opening
                            self.state.last_click_time = None;
                            self.state.last_click_index = None;
                        } else {
                            // Single click: select the card and update tracking
                            self.state.selected_index = clicked_idx;
                            self.state.last_click_time = Some(Instant::now());
                            self.state.last_click_index = Some(clicked_idx);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse events for scrollable content views (Detail views, Config)
    async fn handle_scroll_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        const ACTION_PANEL_WIDTH: u16 = 22;

        match mouse.kind {
            MouseEventKind::ScrollUp => self.state.scroll_up(),
            MouseEventKind::ScrollDown => self.state.scroll_down(),
            MouseEventKind::Down(MouseButton::Left) => {
                // Handle action panel clicks for detail views
                if matches!(
                    self.state.current_view,
                    View::IssueDetail | View::PrDetail | View::DocDetail
                ) {
                    let terminal_width = self.terminal_size.map(|(_, w)| w).unwrap_or(80);
                    let action_panel_start_x = terminal_width.saturating_sub(ACTION_PANEL_WIDTH);

                    if mouse.column >= action_panel_start_x {
                        if let Some(action_idx) = self.calculate_action_index_from_click(mouse.row)
                        {
                            let total_actions = self.state.current_actions.actions.len();
                            if action_idx < total_actions {
                                let is_enabled = self
                                    .state
                                    .current_actions
                                    .actions
                                    .get(action_idx)
                                    .map(|a| a.enabled)
                                    .unwrap_or(false);

                                if is_enabled {
                                    // Trigger button press animation
                                    self.state.button_press = Some(ButtonPressState::new(
                                        PressedButton::ActionPanel(action_idx),
                                    ));
                                    self.state.action_panel_selected_index = action_idx;
                                    self.execute_selected_dynamic_action().await?;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_form_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        let main_area_start_x = self.sidebar_width();
        const FORM_START_Y: u16 = 1;
        const FIELD_HEIGHT: u16 = 3;
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if mouse.column >= main_area_start_x && mouse.row >= FORM_START_Y {
                let field_index = ((mouse.row - FORM_START_Y) / FIELD_HEIGHT) as usize;
                let max_fields = self.state.form_field_count();
                if field_index < max_fields {
                    self.state.active_form_field = field_index;
                }
            }
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => self.state.scroll_up(),
            MouseEventKind::ScrollDown => self.state.scroll_down(),
            _ => {}
        }
        Ok(())
    }

    async fn handle_splash_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if let Some(ref mut splash) = self.splash_state {
                splash.skip();
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn get_current_issue(&self) -> Option<&crate::state::Issue> {
        self.state
            .selected_issue_id
            .as_ref()
            .and_then(|id| self.state.issues.iter().find(|i| &i.id == id))
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    /// Handle keyboard text selection (Shift+arrows)
    fn handle_selection_key(&mut self, key: KeyEvent) -> Result<()> {
        let selection = &mut self.state.selection;

        // Initialize keyboard cursor if not set
        if selection.keyboard_cursor.is_none() {
            // Start from center of screen
            let (height, width) = self.terminal_size.unwrap_or((24, 80));
            let cursor = ScreenPos::new(width / 2, height / 2);
            selection.keyboard_cursor = Some(cursor);
            selection.start(cursor);
            selection.keyboard_mode = true;
        }

        let current = selection.keyboard_cursor.unwrap();
        let (max_row, max_col) = self.terminal_size.unwrap_or((24, 80));

        let new_pos = match key.code {
            KeyCode::Left => ScreenPos::new(current.col.saturating_sub(1), current.row),
            KeyCode::Right => ScreenPos::new(
                (current.col + 1).min(max_col.saturating_sub(1)),
                current.row,
            ),
            KeyCode::Up => ScreenPos::new(current.col, current.row.saturating_sub(1)),
            KeyCode::Down => ScreenPos::new(
                current.col,
                (current.row + 1).min(max_row.saturating_sub(1)),
            ),
            _ => current,
        };

        selection.keyboard_cursor = Some(new_pos);
        selection.update(new_pos);
        selection.keyboard_mode = true;

        Ok(())
    }

    /// Copy current text selection to clipboard
    pub fn copy_selection(&mut self) -> Result<()> {
        if let Some((start, end)) = self.state.selection.get_range() {
            let text = self.screen_buffer.extract_text(start, end);
            if !text.is_empty() {
                self.copy_to_clipboard(&text)?;
                self.copy_message = Some(format!("Copied {} chars", text.len()));
            }
            self.state.selection.clear();
        }
        Ok(())
    }
}
