//! Application state and core logic

use crate::daemon::DaemonClient;
use crate::state::{
    AppState, ButtonPressState, DocDetailFocus, DocsListFocus, EntityType, IssueDetailFocus,
    IssuesListFocus, ListScope, LlmAction, LogoStyle, MoveEntityType, OrganizationFocus,
    PendingMoveAction, PendingWorktreeAction, PrDetailFocus, PressedButton, Project, PrsListFocus,
    ScreenBuffer, ScreenPos, SplashState, UiArea, View, ViewParams, WorktreeDialogOption,
};
use crate::ui::forms::get_doc_field_count;
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

    /// Push an error message to the error queue for display
    pub fn push_error(&mut self, message: impl Into<String>) {
        self.state.push_error(message.into());
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
        if !UiArea::ActionPanel.contains_row(mouse_row) {
            return None; // Click on border or context bar
        }

        let row_in_panel = UiArea::ActionPanel.relative_row(mouse_row);
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

    /// Calculate which sidebar item index was clicked based on mouse row position.
    ///
    /// This accounts for:
    /// - Top padding when content is vertically centered
    /// - Scroll indicators when content is scrollable
    /// - Scroll offset to map visible position to correct item index
    fn calculate_sidebar_item_from_click(&self, mouse_row: u16) -> Option<usize> {
        use crate::ui::sidebar::get_local_actions;

        let actions = get_local_actions(&self.state.current_view);
        let item_count = actions.len();

        if item_count == 0 {
            return None;
        }

        // Adjust mouse row for context bar offset
        let adjusted_row = UiArea::Sidebar.relative_row(mouse_row);

        // Get sidebar height from terminal size (height - 1 for status bar - context bar height)
        let sidebar_height = self
            .terminal_size
            .map(|(h, _)| h.saturating_sub(1 + UiArea::height_adjustment()))
            .unwrap_or(24);

        // Calculate content height: N items × BUTTON_HEIGHT rows each
        let content_height = (item_count as u16) * BUTTON_HEIGHT;

        // Calculate top padding (centering logic from vertical_button_group.rs:124-128)
        let (top_padding, first_visible) = if content_height <= sidebar_height {
            // Content fits - calculate centering padding
            let padding = (sidebar_height - content_height) / 2;
            (padding, 0)
        } else {
            // Content doesn't fit - account for scroll indicators
            // Check if we have an up indicator (only when scrolled down)
            let up_indicator = if self.state.sidebar_scroll_offset > 0 {
                1u16
            } else {
                0
            };

            (up_indicator, self.state.sidebar_scroll_offset)
        };

        // Calculate click position relative to the first button
        if adjusted_row < top_padding {
            return None; // Clicked in padding or indicator area
        }

        let row_in_buttons = adjusted_row - top_padding;
        let clicked_visible_index = (row_in_buttons / BUTTON_HEIGHT) as usize;
        let item_index = first_visible + clicked_visible_index;

        if item_index < item_count {
            Some(item_index)
        } else {
            None
        }
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

        // Handle worktree dialog (modal)
        if self.state.pending_worktree_action.is_some() {
            self.handle_worktree_dialog_key(key).await?;
            return Ok(());
        }

        // Handle move dialog (modal)
        if self.state.pending_move_action.is_some() {
            self.handle_move_dialog_key(key).await?;
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

        // Global search shortcut (/) - only from non-form views
        if key.code == KeyCode::Char('/')
            && !self.state.current_view.is_form_view()
            && !matches!(self.state.current_view, View::GlobalSearch | View::Splash)
        {
            self.state.global_search_focus = crate::state::GlobalSearchFocus::SearchInput;
            self.navigate(View::GlobalSearch, ViewParams::default());
            return Ok(());
        }

        // Track view before handling key to detect navigation
        let view_before = self.state.current_view.clone();

        match self.state.current_view {
            View::Splash => self.handle_splash_key(key).await?,
            View::Projects => self.handle_projects_key(key).await?,
            View::Organization => self.handle_organization_key(key).await?,
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
            View::DocEdit => self.handle_doc_edit_key(key).await?,
            View::Config => self.handle_config_key(key).await?,
            View::GlobalSearch => self.handle_global_search_key(key).await?,
            View::InitProject => self.handle_init_project_key(key).await?,
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
                    if let Ok(docs) = self.daemon.list_docs(&path).await {
                        self.state.docs = docs;
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
            KeyCode::Char('i') => {
                // Open Init Project form
                // If an uninitialized project is selected, pre-fill the path
                let project_path = self
                    .state
                    .selectable_projects()
                    .get(self.state.selected_index)
                    .filter(|p| !p.initialized)
                    .map(|p| p.path.clone());

                self.state.clear_form();
                if let Some(path) = project_path {
                    self.state.form_project_path = path;
                }
                self.navigate(View::InitProject, ViewParams::default());
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

    /// Handle keys in Organization view
    async fn handle_organization_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Toggle focus between list and action panel
            KeyCode::Tab => {
                self.state.organization_focus.toggle();
            }
            // Navigate up
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(
                    self.state.organization_focus,
                    OrganizationFocus::ProjectsList
                ) {
                    if self.state.selected_project_in_org > 0 {
                        self.state.selected_project_in_org -= 1;
                    }
                } else {
                    self.state.action_panel_up();
                }
            }
            // Navigate down
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(
                    self.state.organization_focus,
                    OrganizationFocus::ProjectsList
                ) {
                    let max = self.state.organization_projects.len();
                    if max > 0 && self.state.selected_project_in_org < max - 1 {
                        self.state.selected_project_in_org += 1;
                    }
                } else {
                    self.state.action_panel_down();
                }
            }
            // Enter: open selected project or execute action
            KeyCode::Enter => {
                if matches!(
                    self.state.organization_focus,
                    OrganizationFocus::ActionPanel
                ) {
                    self.execute_selected_dynamic_action().await?;
                } else {
                    // Open selected project - navigate to Issues view
                    if let Some(project) = self
                        .state
                        .organization_projects
                        .get(self.state.selected_project_in_org)
                        .cloned()
                    {
                        self.state.selected_project_path = Some(project.path.clone());
                        if let Ok(issues) = self.daemon.list_issues(&project.path).await {
                            self.state.issues = issues;
                        }
                        if let Ok(docs) = self.daemon.list_docs(&project.path).await {
                            self.state.docs = docs;
                        }
                        if let Ok(prs) = self.daemon.list_prs(&project.path).await {
                            self.state.prs = prs;
                        }
                        self.navigate(View::Issues, ViewParams::default());
                    }
                }
            }
            // Go back to projects
            KeyCode::Esc | KeyCode::Backspace => {
                self.state.organization_focus = OrganizationFocus::ProjectsList;
                self.go_back();
            }
            // 'p' - Navigate to All Projects
            KeyCode::Char('p') => {
                self.navigate(View::Projects, ViewParams::default());
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse events in Organization view
    async fn handle_organization_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.state.scroll_down();
            }
            MouseEventKind::ScrollUp => {
                self.state.scroll_up();
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Simple click handling - could be expanded to select projects
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
            KeyCode::Char('o') => {
                // Toggle org scope
                self.state.issues_list_scope = self.state.issues_list_scope.toggle();
                self.state.reset_selection();
                // Load org issues if switching to org scope
                if matches!(
                    self.state.issues_list_scope,
                    crate::state::ListScope::Organization
                ) {
                    self.load_org_issues().await?;
                }
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
                self.push_error("No project selected");
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
                self.push_error("No issue selected");
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
                // Navigate to issues list after successful deletion
                self.navigate(View::Issues, ViewParams::default());
            }
            Err(e) => {
                self.push_error(format!("Failed to delete issue: {}", e));
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
                if let Some(issue_id) = self.state.selected_issue_id.clone() {
                    // Load issue data into form before navigating
                    if let Some(issue) =
                        self.state.issues.iter().find(|i| i.id == issue_id).cloned()
                    {
                        self.state.load_issue_to_form(&issue);
                    }
                    self.navigate(
                        View::IssueEdit,
                        ViewParams {
                            issue_id: Some(issue_id),
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
                self.push_error("No project selected");
                return Ok(());
            }
        };

        let issue_id = match &self.state.selected_issue_id {
            Some(id) => id.clone(),
            None => {
                self.push_error("No issue selected");
                return Ok(());
            }
        };

        // Get current issue data
        let (title, description, priority) = {
            let issue = self.state.issues.iter().find(|i| i.id == issue_id);
            match issue {
                Some(i) => (i.title.clone(), i.description.clone(), i.metadata.priority),
                None => {
                    self.push_error("Issue not found");
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
            }
            Err(e) => {
                self.push_error(format!("Failed to update status: {}", e));
            }
        }

        Ok(())
    }

    /// Update the current PR's status
    async fn update_pr_status(&mut self, new_status: String) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.push_error("No project selected");
                return Ok(());
            }
        };

        let pr_id = match &self.state.selected_pr_id {
            Some(id) => id.clone(),
            None => {
                self.push_error("No PR selected");
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
                    self.push_error("PR not found");
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
            }
            Err(e) => {
                self.push_error(format!("Failed to update PR status: {}", e));
            }
        }

        Ok(())
    }

    /// Execute the "Open in VSCode" action
    async fn execute_open_in_vscode(&mut self) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.push_error("No project selected");
                return Ok(());
            }
        };

        let issue_id = match &self.state.selected_issue_id {
            Some(id) => id.clone(),
            None => {
                self.push_error("No issue selected");
                return Ok(());
            }
        };

        let action = self.state.action_panel_llm_action.as_proto_value();

        self.copy_message = Some("Opening in VSCode...".to_string());

        match self
            .daemon
            .open_in_temp_vscode(&project_path, &issue_id, action, "", 0)
            .await
        {
            Ok(result) => {
                if result.vscode_opened {
                    self.copy_message = Some(format!(
                        "Opened #{} in VSCode (expires: {})",
                        result.display_number,
                        result
                            .expires_at
                            .split('T')
                            .next()
                            .unwrap_or(&result.expires_at)
                    ));
                } else {
                    self.copy_message =
                        Some(format!("Workspace created at {}", result.workspace_path));
                }
            }
            Err(e) => {
                let error_str = e.to_string();
                let error_lower = error_str.to_lowercase();

                // Check if this is a worktree/folder exists error
                if error_lower.contains("worktree") || error_lower.contains("already exists") {
                    // Try to find existing workspace for this issue
                    if let Ok(workspaces) = self.daemon.list_temp_workspaces(&project_path).await {
                        // Find workspace matching this issue
                        if let Some(existing) = workspaces.into_iter().find(|w| {
                            w.issue_id == issue_id || w.issue_display_number.to_string() == issue_id
                        }) {
                            // Show dialog to let user choose action
                            self.state.pending_worktree_action = Some(PendingWorktreeAction {
                                project_path: project_path.clone(),
                                issue_id: issue_id.clone(),
                                action,
                                existing_workspace: existing,
                                selected_option: WorktreeDialogOption::OpenExisting,
                            });
                            self.copy_message = None;
                            return Ok(());
                        }
                    }
                    // Fallback to error message if we couldn't find existing workspace
                    self.push_error("Failed to create git worktree.\nTry closing other VS Code windows for this project.");
                } else if error_str.contains("detached HEAD") {
                    self.push_error("Repository is in detached HEAD state.\nCheckout a branch first: git checkout <branch>");
                } else if error_lower.contains("not a git repository") {
                    self.push_error(
                        "This project is not a git repository.\nInitialize with: git init",
                    );
                } else if error_lower.contains("not found") && error_lower.contains("vscode") {
                    self.push_error("VS Code not found.\nInstall it and add 'code' to PATH.");
                } else if error_lower.contains("connection") {
                    self.push_error(
                        "Cannot connect to centy daemon.\nIs it running? Try: centy daemon start",
                    );
                } else {
                    // Clean up nested error prefixes for unknown errors
                    let user_msg = error_str
                        .replace("Git error: ", "")
                        .replace("Worktree error: ", "");
                    self.push_error(user_msg);
                }
            }
        }

        Ok(())
    }

    /// Execute the "Open in Terminal" action
    async fn execute_open_in_terminal(&mut self) -> Result<()> {
        let project_path = match &self.state.selected_project_path {
            Some(path) => path.clone(),
            None => {
                self.push_error("No project selected");
                return Ok(());
            }
        };

        let issue_id = match &self.state.selected_issue_id {
            Some(id) => id.clone(),
            None => {
                self.push_error("No issue selected");
                return Ok(());
            }
        };

        self.copy_message = Some("Opening in terminal...".to_string());

        match self
            .daemon
            .open_agent_in_terminal(&project_path, &issue_id, "", 0, 0)
            .await
        {
            Ok(result) => {
                if result.terminal_opened {
                    self.copy_message = Some(format!(
                        "Opened #{} in terminal with {}",
                        result.display_number, result.agent_command
                    ));
                } else {
                    self.copy_message =
                        Some(format!("Agent ready at {}", result.working_directory));
                }
            }
            Err(e) => {
                self.push_error(format!("Failed to open: {}", e));
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
            self.push_error(action.disabled_reason.clone());
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
                    self.push_error("Delete PR: Not yet implemented");
                }
                View::Docs | View::DocDetail => {
                    self.push_error("Delete doc: Not yet implemented");
                }
                _ => {}
            },

            // Duplicate action - contextual based on current view
            "duplicate" => match self.state.current_view {
                View::Issues | View::IssueDetail => {
                    self.push_error("Duplicate issue: Not yet implemented");
                }
                View::Docs | View::DocDetail => {
                    self.push_error("Duplicate doc: Not yet implemented");
                }
                _ => {}
            },

            // Move action - contextual based on current view
            "move" => match self.state.current_view {
                View::Issues | View::IssueDetail => {
                    self.start_move_issue();
                }
                View::Docs | View::DocDetail => {
                    self.start_move_doc();
                }
                _ => {}
            },

            // Mode actions (Issue-specific)
            "mode:plan" => {
                self.state.action_panel_llm_action = LlmAction::Plan;
            }
            "mode:implement" => {
                self.state.action_panel_llm_action = LlmAction::Implement;
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
                self.push_error(format!("Unknown action: {}", action.id));
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
                    self.copy_message = Some("Issue created! Ready for next issue.".to_string());
                } else {
                    self.state.selected_issue_id = Some(new_id.clone());
                    let msg = if draft {
                        "Draft saved!"
                    } else {
                        "Issue created!"
                    };
                    self.copy_message = Some(msg.to_string());
                    self.navigate_to_created_item(
                        View::IssueDetail,
                        ViewParams {
                            issue_id: Some(new_id),
                            ..Default::default()
                        },
                    );
                }
            } else {
                self.push_error("Failed to create issue");
            }
        } else {
            self.push_error("No project selected");
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
                    self.push_error("Failed to update issue");
                }
            }
            (None, _) => {
                self.push_error("No project selected");
            }
            (_, None) => {
                self.push_error("No issue selected");
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
                    let count = if matches!(self.state.prs_list_scope, ListScope::Organization) {
                        self.state.sorted_org_prs().len()
                    } else {
                        self.state.sorted_prs().len()
                    };
                    self.state.move_selection_down(count);
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
                } else if matches!(self.state.prs_list_scope, ListScope::Organization) {
                    // Navigate to PR from org scope (need to switch project context)
                    // Extract data first to avoid borrow issues
                    let org_pr_data = self
                        .state
                        .sorted_org_prs()
                        .get(self.state.selected_index)
                        .map(|op| (op.project_path.clone(), op.pr.id.clone()));

                    if let Some((project_path, pr_id)) = org_pr_data {
                        // Switch to the PR's project and navigate to detail
                        self.state.selected_project_path = Some(project_path);
                        self.state.prs_list_scope = ListScope::Project;
                        self.state.selected_pr_id = Some(pr_id.clone());
                        // Load project PRs in background
                        if let Err(e) = self.load_project_prs().await {
                            eprintln!("Error loading project PRs: {e}");
                        }
                        self.navigate(
                            View::PrDetail,
                            ViewParams {
                                pr_id: Some(pr_id),
                                ..Default::default()
                            },
                        );
                    }
                } else {
                    // Open PR detail (project scope)
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
            KeyCode::Char('o') => {
                // Toggle between project and org scope
                self.state.prs_list_scope = match self.state.prs_list_scope {
                    ListScope::Project => {
                        // Load org PRs when switching to org scope
                        self.load_org_prs().await?;
                        ListScope::Organization
                    }
                    ListScope::Organization => ListScope::Project,
                };
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
                if let Some(pr_id) = self.state.selected_pr_id.clone() {
                    // Load PR data into form before navigating
                    if let Some(pr) = self.state.prs.iter().find(|p| p.id == pr_id).cloned() {
                        self.state.load_pr_to_form(&pr);
                    }
                    self.navigate(
                        View::PrEdit,
                        ViewParams {
                            pr_id: Some(pr_id),
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
                self.push_error("Failed to create PR");
            }
        } else {
            self.push_error("No project selected");
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
                    self.push_error("Failed to update PR");
                }
            }
            (None, _) => {
                self.push_error("No project selected");
            }
            (_, None) => {
                self.push_error("No PR selected");
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
                    let count = if matches!(self.state.docs_list_scope, ListScope::Organization) {
                        self.state.sorted_org_docs().len()
                    } else {
                        self.state.docs.len()
                    };
                    self.state.move_selection_down(count);
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
                } else if matches!(self.state.docs_list_scope, ListScope::Organization) {
                    // Navigate to doc from org scope (need to switch project context)
                    // Extract data first to avoid borrow issues
                    let org_doc_data = self
                        .state
                        .sorted_org_docs()
                        .get(self.state.selected_index)
                        .map(|od| (od.project_path.clone(), od.doc.slug.clone()));

                    if let Some((project_path, doc_slug)) = org_doc_data {
                        // Switch to the doc's project and navigate to detail
                        self.state.selected_project_path = Some(project_path);
                        self.state.docs_list_scope = ListScope::Project;
                        self.state.selected_doc_slug = Some(doc_slug.clone());
                        // Load project docs in background
                        if let Err(e) = self.load_project_docs().await {
                            eprintln!("Error loading project docs: {e}");
                        }
                        self.navigate(
                            View::DocDetail,
                            ViewParams {
                                doc_slug: Some(doc_slug),
                                ..Default::default()
                            },
                        );
                    }
                } else {
                    // Open doc detail (project scope)
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
            KeyCode::Char('o') => {
                // Toggle between project and org scope
                self.state.docs_list_scope = match self.state.docs_list_scope {
                    ListScope::Project => {
                        // Load org docs when switching to org scope
                        self.load_org_docs().await?;
                        ListScope::Organization
                    }
                    ListScope::Organization => ListScope::Project,
                };
                self.state.reset_selection();
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
            // Edit doc
            KeyCode::Char('e') => {
                if let Some(slug) = self.state.selected_doc_slug.clone() {
                    // Load doc data into form before navigating
                    if let Some(doc) = self.state.docs.iter().find(|d| d.slug == slug).cloned() {
                        self.state.load_doc_to_form(&doc);
                    }
                    self.navigate(
                        View::DocEdit,
                        ViewParams {
                            doc_slug: Some(slug),
                            ..Default::default()
                        },
                    );
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
        let field_count = get_doc_field_count(self);
        let org_checkbox_field = if field_count == 4 { Some(3) } else { None };

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
            KeyCode::Tab => {
                // Wrap around based on actual field count
                self.state.active_form_field = (self.state.active_form_field + 1) % field_count;
            }
            KeyCode::BackTab => {
                // Wrap around based on actual field count
                if self.state.active_form_field == 0 {
                    self.state.active_form_field = field_count - 1;
                } else {
                    self.state.active_form_field -= 1;
                }
            }
            // Toggle checkbox with Space or Enter when on org checkbox field
            KeyCode::Char(' ') | KeyCode::Enter
                if org_checkbox_field == Some(self.state.active_form_field) =>
            {
                self.state.toggle_org_doc();
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

    /// Helper to save doc create
    async fn save_doc_create(&mut self) {
        let Some(path) = self.state.selected_project_path.clone() else {
            self.push_error("No project selected");
            return;
        };

        let slug = if self.state.form_slug.is_empty() {
            None
        } else {
            Some(self.state.form_slug.clone())
        };
        let result = self
            .daemon
            .create_doc(
                &path,
                &self.state.form_title,
                &self.state.form_description,
                slug.as_deref(),
                self.state.form_is_org_doc,
            )
            .await;
        match result {
            Ok((new_slug, sync_results)) => {
                // Show sync results if this was an org doc
                if !sync_results.is_empty() {
                    let success_count = sync_results.iter().filter(|r| r.success).count();
                    let total = sync_results.len();
                    if success_count < total {
                        // Some failed - show warning
                        let failed: Vec<_> = sync_results
                            .iter()
                            .filter(|r| !r.success)
                            .map(|r| r.error.clone())
                            .collect();
                        self.push_error(format!(
                            "Doc created, but sync failed for {} project(s): {}",
                            total - success_count,
                            failed.join(", ")
                        ));
                    }
                }

                if let Ok(docs) = self.daemon.list_docs(&path).await {
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
            }
            Err(e) => {
                self.push_error(format!("Failed to create doc: {}", e));
            }
        }
    }

    /// Handle keys in Doc Edit view
    async fn handle_doc_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            // Save (Ctrl+S or Cmd+W / Ctrl+W)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_doc_edit().await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.save_doc_edit().await;
            }
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            KeyCode::Char(c) => self
                .state
                .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Backspace => self.state.form_backspace(),
            KeyCode::Enter => {
                // Enter in content field adds newline
                if self.state.active_form_field == 1 {
                    self.state.form_description.push('\n');
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper to save doc edit
    async fn save_doc_edit(&mut self) {
        let Some(path) = self.state.selected_project_path.clone() else {
            self.push_error("No project selected");
            return;
        };
        let Some(slug) = self.state.selected_doc_slug.clone() else {
            self.push_error("No doc selected");
            return;
        };

        // Check if slug changed
        let new_slug = if !self.state.form_slug.is_empty() && self.state.form_slug != slug {
            Some(self.state.form_slug.as_str())
        } else {
            None
        };

        let result = self
            .daemon
            .update_doc(
                &path,
                &slug,
                &self.state.form_title,
                &self.state.form_description,
                new_slug,
            )
            .await;

        match result {
            Ok(()) => {
                if let Ok(docs) = self.daemon.list_docs(&path).await {
                    self.state.docs = docs;
                }
                // Update selected slug if it changed
                if let Some(ns) = new_slug {
                    self.state.selected_doc_slug = Some(ns.to_string());
                }
                self.state.clear_form();
                self.go_back();
            }
            Err(e) => {
                self.push_error(format!("Failed to update doc: {}", e));
            }
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

    /// Handle keys for the global search view
    async fn handle_global_search_key(&mut self, key: KeyEvent) -> Result<()> {
        use crate::state::GlobalSearchFocus;

        match key.code {
            // Escape: go back
            KeyCode::Esc => {
                self.state.global_search_query.clear();
                self.state.global_search_results.clear();
                self.state.global_search_focus = GlobalSearchFocus::SearchInput;
                self.go_back();
            }
            // Tab: toggle focus between input and results
            KeyCode::Tab => {
                self.state.global_search_focus.toggle();
            }
            // Ctrl+F: cycle filter type
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.global_search_filter = self.state.global_search_filter.next();
                // Re-execute search if we have a query
                if !self.state.global_search_query.is_empty() {
                    self.execute_global_search().await?;
                }
            }
            // Navigation in results
            KeyCode::Down | KeyCode::Char('j') => {
                if matches!(self.state.global_search_focus, GlobalSearchFocus::Results) {
                    let max = self.state.global_search_results.len();
                    if max > 0 && self.state.global_search_selected < max - 1 {
                        self.state.global_search_selected += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if matches!(self.state.global_search_focus, GlobalSearchFocus::Results)
                    && self.state.global_search_selected > 0
                {
                    self.state.global_search_selected -= 1;
                }
            }
            // Enter: execute search or navigate to result
            KeyCode::Enter => {
                if matches!(
                    self.state.global_search_focus,
                    GlobalSearchFocus::SearchInput
                ) {
                    // Execute search
                    self.execute_global_search().await?;
                    if !self.state.global_search_results.is_empty() {
                        self.state.global_search_focus = GlobalSearchFocus::Results;
                    }
                } else {
                    // Navigate to selected result
                    self.navigate_to_search_result().await?;
                }
            }
            // Character input in search box
            KeyCode::Char(c)
                if matches!(
                    self.state.global_search_focus,
                    GlobalSearchFocus::SearchInput
                ) =>
            {
                self.state.global_search_query.push(c);
            }
            KeyCode::Backspace
                if matches!(
                    self.state.global_search_focus,
                    GlobalSearchFocus::SearchInput
                ) =>
            {
                self.state.global_search_query.pop();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Init Project view
    async fn handle_init_project_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check if we're on the action panel (field 1)
        let on_action_panel = self.state.active_form_field == 1;

        match key.code {
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            // Up/Down for action panel navigation
            KeyCode::Up | KeyCode::Char('k') if on_action_panel => {
                if self.state.form_selected_button == 0 {
                    self.state.form_selected_button = 1;
                } else {
                    self.state.form_selected_button -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if on_action_panel => {
                self.state.form_selected_button = (self.state.form_selected_button + 1) % 2;
            }
            // Enter triggers the selected button (or Initialize if on path field)
            KeyCode::Enter => {
                if on_action_panel {
                    match self.state.form_selected_button {
                        0 => {
                            // Initialize
                            self.init_project().await;
                        }
                        1 => {
                            // Cancel
                            self.state.clear_form();
                            self.go_back();
                        }
                        _ => {}
                    }
                } else {
                    // Enter from path field also triggers init
                    self.init_project().await;
                }
            }
            // Keyboard shortcuts (work from anywhere)
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.init_project().await;
            }
            KeyCode::Char('w') if key.modifiers.contains(crate::platform::COPY_MODIFIER) => {
                self.init_project().await;
            }
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            // Form field input (only when not on action panel)
            KeyCode::Char(c) if !on_action_panel => {
                self.state.form_project_path.push(c);
            }
            KeyCode::Backspace if !on_action_panel => {
                self.state.form_project_path.pop();
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute global search based on current query and filter
    async fn execute_global_search(&mut self) -> Result<()> {
        use crate::state::{GlobalSearchFilter, GlobalSearchResult};

        let query = self.state.global_search_query.clone();
        if query.is_empty() {
            self.state.global_search_results.clear();
            return Ok(());
        }

        self.state.global_search_loading = true;
        self.state.global_search_results.clear();

        let mut results = Vec::new();

        // Search issues if filter includes them
        if matches!(
            self.state.global_search_filter,
            GlobalSearchFilter::All | GlobalSearchFilter::Issues
        ) {
            // Try advanced search first
            if let Ok(issues) = self.daemon.advanced_search(&query).await {
                for (issue, path, name) in issues {
                    results.push(GlobalSearchResult::Issue {
                        issue,
                        project_path: path,
                        project_name: name,
                    });
                }
            } else {
                // Fallback: try UUID search if query looks like UUID
                if let Ok(issues) = self.daemon.search_issues_by_uuid(&query).await {
                    for (issue, path, name) in issues {
                        results.push(GlobalSearchResult::Issue {
                            issue,
                            project_path: path,
                            project_name: name,
                        });
                    }
                }
            }
        }

        // Search PRs if filter includes them
        if matches!(
            self.state.global_search_filter,
            GlobalSearchFilter::All | GlobalSearchFilter::Prs
        ) {
            if let Ok(prs) = self.daemon.search_prs_by_uuid(&query).await {
                for (pr, path, name) in prs {
                    results.push(GlobalSearchResult::Pr {
                        pr,
                        project_path: path,
                        project_name: name,
                    });
                }
            }
        }

        // Search docs if filter includes them
        if matches!(
            self.state.global_search_filter,
            GlobalSearchFilter::All | GlobalSearchFilter::Docs
        ) {
            if let Ok(docs) = self.daemon.search_docs_by_slug(&query).await {
                for (doc, path, name) in docs {
                    results.push(GlobalSearchResult::Doc {
                        doc,
                        project_path: path,
                        project_name: name,
                    });
                }
            }
        }

        self.state.global_search_results = results;
        self.state.global_search_selected = 0;
        self.state.global_search_loading = false;

        Ok(())
    }

    /// Navigate to selected search result, switching project context as needed
    async fn navigate_to_search_result(&mut self) -> Result<()> {
        use crate::state::GlobalSearchResult;

        let result = match self
            .state
            .global_search_results
            .get(self.state.global_search_selected)
        {
            Some(r) => r.clone(),
            None => return Ok(()),
        };

        // Switch project context
        let project_path = result.project_path().to_string();
        self.state.selected_project_path = Some(project_path.clone());

        // Load project data
        if let Ok(issues) = self.daemon.list_issues(&project_path).await {
            self.state.issues = issues;
        }
        if let Ok(prs) = self.daemon.list_prs(&project_path).await {
            self.state.prs = prs;
        }
        if let Ok(docs) = self.daemon.list_docs(&project_path).await {
            self.state.docs = docs;
        }

        // Clear search state
        self.state.global_search_query.clear();
        self.state.global_search_results.clear();
        self.state.global_search_focus = crate::state::GlobalSearchFocus::SearchInput;

        // Navigate to the appropriate detail view
        match result {
            GlobalSearchResult::Issue { issue, .. } => {
                self.state.selected_issue_id = Some(issue.id.clone());
                self.navigate(
                    View::IssueDetail,
                    ViewParams {
                        issue_id: Some(issue.id),
                        ..Default::default()
                    },
                );
            }
            GlobalSearchResult::Pr { pr, .. } => {
                self.state.selected_pr_id = Some(pr.id.clone());
                self.navigate(
                    View::PrDetail,
                    ViewParams {
                        pr_id: Some(pr.id),
                        ..Default::default()
                    },
                );
            }
            GlobalSearchResult::Doc { doc, .. } => {
                self.state.selected_doc_slug = Some(doc.slug.clone());
                self.navigate(
                    View::DocDetail,
                    ViewParams {
                        doc_slug: Some(doc.slug),
                        ..Default::default()
                    },
                );
            }
        }

        Ok(())
    }

    /// Load org-level issues for org scope view
    async fn load_org_issues(&mut self) -> Result<()> {
        use crate::state::OrgIssue;

        if let Some(org_slug) = self.state.current_organization_slug().map(String::from) {
            match self.daemon.list_issues_by_organization(&org_slug).await {
                Ok(issues) => {
                    self.state.org_issues = issues
                        .into_iter()
                        .map(|(issue, path, name)| OrgIssue {
                            issue,
                            project_path: path,
                            project_name: name,
                        })
                        .collect();
                }
                Err(e) => {
                    self.push_error(format!("Failed to load org issues: {}", e));
                }
            }
        } else {
            self.push_error("Current project is not part of an organization");
            self.state.issues_list_scope = crate::state::ListScope::Project;
        }
        Ok(())
    }

    /// Load org-level PRs for org scope view
    async fn load_org_prs(&mut self) -> Result<()> {
        use crate::state::OrgPr;

        if let Some(org_slug) = self.state.current_organization_slug().map(String::from) {
            match self.daemon.list_prs_by_organization(&org_slug).await {
                Ok(prs) => {
                    self.state.org_prs = prs
                        .into_iter()
                        .map(|(pr, path, name)| OrgPr {
                            pr,
                            project_path: path,
                            project_name: name,
                        })
                        .collect();
                }
                Err(e) => {
                    self.push_error(format!("Failed to load org PRs: {}", e));
                }
            }
        } else {
            self.push_error("Current project is not part of an organization");
            self.state.prs_list_scope = crate::state::ListScope::Project;
        }
        Ok(())
    }

    /// Load org-level docs for org scope view
    async fn load_org_docs(&mut self) -> Result<()> {
        use crate::state::OrgDoc;

        if let Some(org_slug) = self.state.current_organization_slug().map(String::from) {
            match self.daemon.list_docs_by_organization(&org_slug).await {
                Ok(docs) => {
                    self.state.org_docs = docs
                        .into_iter()
                        .map(|(doc, path, name)| OrgDoc {
                            doc,
                            project_path: path,
                            project_name: name,
                        })
                        .collect();
                }
                Err(e) => {
                    self.push_error(format!("Failed to load org docs: {}", e));
                }
            }
        } else {
            self.push_error("Current project is not part of an organization");
            self.state.docs_list_scope = crate::state::ListScope::Project;
        }
        Ok(())
    }

    /// Load PRs for the current project
    async fn load_project_prs(&mut self) -> Result<()> {
        if let Some(path) = &self.state.selected_project_path {
            if let Ok(prs) = self.daemon.list_prs(path).await {
                self.state.prs = prs;
            }
        }
        Ok(())
    }

    /// Load Docs for the current project
    async fn load_project_docs(&mut self) -> Result<()> {
        if let Some(path) = &self.state.selected_project_path {
            if let Ok(docs) = self.daemon.list_docs(path).await {
                self.state.docs = docs;
            }
        }
        Ok(())
    }

    /// Initialize a project
    async fn init_project(&mut self) {
        let project_path = self.state.form_project_path.trim().to_string();

        if project_path.is_empty() {
            self.push_error("Project path is required");
            return;
        }

        // Expand ~ to home directory if present
        let expanded_path = if let Some(stripped) = project_path.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                format!("{}/{}", home, stripped)
            } else {
                project_path.clone()
            }
        } else {
            project_path.clone()
        };

        match self.daemon.init_project(&expanded_path, true).await {
            Ok(()) => {
                // Refresh projects list
                if let Ok(projects) = self.daemon.list_projects().await {
                    self.state.projects = projects;
                }
                self.copy_message = Some("Project initialized!".to_string());
                self.state.clear_form();
                self.go_back();
            }
            Err(e) => {
                self.push_error(format!("Failed to initialize project: {}", e));
            }
        }
    }

    /// Handle keys for the worktree dialog
    async fn handle_worktree_dialog_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Cancel - dismiss dialog
            KeyCode::Esc => {
                self.state.pending_worktree_action = None;
            }
            // Navigate between options
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') => {
                if let Some(ref mut action) = self.state.pending_worktree_action {
                    action.selected_option = action.selected_option.toggle();
                }
            }
            // Confirm selection
            KeyCode::Enter => {
                if let Some(action) = self.state.pending_worktree_action.take() {
                    match action.selected_option {
                        WorktreeDialogOption::OpenExisting => {
                            // Open VS Code at the existing workspace path
                            self.open_vscode_at_path(&action.existing_workspace.workspace_path)
                                .await;
                        }
                        WorktreeDialogOption::DeleteAndRecreate => {
                            // Delete the existing workspace and retry
                            self.copy_message = Some("Deleting workspace...".to_string());
                            if self
                                .daemon
                                .close_temp_workspace(
                                    &action.existing_workspace.workspace_path,
                                    true,
                                )
                                .await
                                .is_ok()
                            {
                                // Retry opening
                                self.copy_message = Some("Recreating workspace...".to_string());
                                match self
                                    .daemon
                                    .open_in_temp_vscode(
                                        &action.project_path,
                                        &action.issue_id,
                                        action.action,
                                        "",
                                        0,
                                    )
                                    .await
                                {
                                    Ok(result) => {
                                        if result.vscode_opened {
                                            self.copy_message = Some(format!(
                                                "Opened #{} in VSCode (expires: {})",
                                                result.display_number,
                                                result
                                                    .expires_at
                                                    .split('T')
                                                    .next()
                                                    .unwrap_or(&result.expires_at)
                                            ));
                                        } else {
                                            self.copy_message = Some(format!(
                                                "Workspace created at {}",
                                                result.workspace_path
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        self.push_error(format!(
                                            "Failed to recreate workspace: {}",
                                            e
                                        ));
                                    }
                                }
                            } else {
                                self.push_error(
                                    "Failed to delete existing workspace.\nPlease close VS Code and try again.",
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys for the move dialog
    async fn handle_move_dialog_key(&mut self, key: KeyEvent) -> Result<()> {
        let is_confirmation = self
            .state
            .pending_move_action
            .as_ref()
            .map(|a| a.show_confirmation)
            .unwrap_or(false);

        if is_confirmation {
            // Confirmation dialog: Enter to confirm, Esc to go back
            match key.code {
                KeyCode::Esc => {
                    // Go back to picker
                    if let Some(ref mut action) = self.state.pending_move_action {
                        action.show_confirmation = false;
                        action.target_project_path = None;
                    }
                }
                KeyCode::Enter => {
                    // Execute the move
                    self.execute_move().await?;
                }
                _ => {}
            }
        } else {
            // Project picker dialog
            match key.code {
                KeyCode::Esc => {
                    self.state.pending_move_action = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(ref mut action) = self.state.pending_move_action {
                        if action.selected_project_index > 0 {
                            action.selected_project_index -= 1;
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let filtered_count = self.get_filtered_move_targets().len();
                    if let Some(ref mut action) = self.state.pending_move_action {
                        if action.selected_project_index < filtered_count.saturating_sub(1) {
                            action.selected_project_index += 1;
                        }
                    }
                }
                KeyCode::Char(c) => {
                    // Add to search filter
                    if let Some(ref mut action) = self.state.pending_move_action {
                        action.search_filter.push(c);
                        action.selected_project_index = 0; // Reset selection on filter change
                    }
                }
                KeyCode::Backspace => {
                    if let Some(ref mut action) = self.state.pending_move_action {
                        action.search_filter.pop();
                        action.selected_project_index = 0;
                    }
                }
                KeyCode::Enter => {
                    // Select target and show confirmation
                    // Get the target path before mutating
                    let target_path = {
                        let targets = self.get_filtered_move_targets();
                        let idx = self
                            .state
                            .pending_move_action
                            .as_ref()
                            .map(|a| a.selected_project_index)
                            .unwrap_or(0);
                        targets.get(idx).map(|p| p.path.clone())
                    };
                    if let Some(path) = target_path {
                        if let Some(ref mut action) = self.state.pending_move_action {
                            action.target_project_path = Some(path);
                            action.show_confirmation = true;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Get filtered list of valid move targets (excludes current project)
    fn get_filtered_move_targets(&self) -> Vec<&Project> {
        let current_path = self
            .state
            .pending_move_action
            .as_ref()
            .map(|a| &a.source_project_path);

        let search_filter = self
            .state
            .pending_move_action
            .as_ref()
            .map(|a| a.search_filter.to_lowercase())
            .unwrap_or_default();

        self.state
            .projects
            .iter()
            .filter(|p| {
                p.initialized
                    && Some(&p.path) != current_path
                    && (search_filter.is_empty()
                        || p.display_name().to_lowercase().contains(&search_filter))
            })
            .collect()
    }

    /// Execute the pending move operation
    async fn execute_move(&mut self) -> Result<()> {
        let action = match self.state.pending_move_action.take() {
            Some(a) => a,
            None => return Ok(()),
        };

        let target_path = match action.target_project_path {
            Some(p) => p,
            None => {
                self.push_error("No target project selected");
                return Ok(());
            }
        };

        match action.entity_type {
            MoveEntityType::Issue => {
                match self
                    .daemon
                    .move_issue(&action.source_project_path, &action.entity_id, &target_path)
                    .await
                {
                    Ok((_moved_issue, old_num)) => {
                        // Refresh issues list
                        if let Ok(issues) =
                            self.daemon.list_issues(&action.source_project_path).await
                        {
                            self.state.issues = issues;
                            // Adjust selection if needed
                            let max = self.state.sorted_issues().len();
                            if self.state.selected_index >= max && max > 0 {
                                self.state.selected_index = max - 1;
                            }
                        }
                        let target_name = target_path.rsplit('/').next().unwrap_or(&target_path);
                        self.copy_message =
                            Some(format!("Moved issue #{} to {}", old_num, target_name));
                        // Navigate back to issues list
                        self.navigate(View::Issues, ViewParams::default());
                    }
                    Err(e) => {
                        self.push_error(format!("Failed to move issue: {}", e));
                    }
                }
            }
            MoveEntityType::Doc => {
                match self
                    .daemon
                    .move_doc(
                        &action.source_project_path,
                        &action.entity_id,
                        &target_path,
                        None,
                    )
                    .await
                {
                    Ok((_moved_doc, old_slug)) => {
                        // Refresh docs list
                        if let Ok(docs) = self.daemon.list_docs(&action.source_project_path).await {
                            self.state.docs = docs;
                            // Adjust selection if needed
                            let max = self.state.docs.len();
                            if self.state.selected_index >= max && max > 0 {
                                self.state.selected_index = max - 1;
                            }
                        }
                        let target_name = target_path.rsplit('/').next().unwrap_or(&target_path);
                        self.copy_message =
                            Some(format!("Moved doc '{}' to {}", old_slug, target_name));
                        // Navigate back to docs list
                        self.navigate(View::Docs, ViewParams::default());
                    }
                    Err(e) => {
                        self.push_error(format!("Failed to move doc: {}", e));
                    }
                }
            }
        }

        Ok(())
    }

    /// Start the move flow for an issue
    fn start_move_issue(&mut self) {
        let project_path = match &self.state.selected_project_path {
            Some(p) => p.clone(),
            None => {
                self.push_error("No project selected");
                return;
            }
        };

        // Check if there are other projects to move to
        let target_count = self
            .state
            .projects
            .iter()
            .filter(|p| p.initialized && p.path != project_path)
            .count();

        if target_count == 0 {
            self.push_error("No other projects to move to");
            return;
        }

        // Get issue from current selection context
        let (issue_id, display) = if let Some(id) = &self.state.selected_issue_id {
            let issue = self.state.issues.iter().find(|i| &i.id == id);
            match issue {
                Some(i) => (
                    i.id.clone(),
                    format!("#{} {}", i.display_number, truncate_str(&i.title, 30)),
                ),
                None => {
                    self.push_error("Issue not found");
                    return;
                }
            }
        } else {
            // From list view
            let sorted = self.state.sorted_issues();
            match sorted.get(self.state.selected_index) {
                Some(i) => (
                    i.id.clone(),
                    format!("#{} {}", i.display_number, truncate_str(&i.title, 30)),
                ),
                None => {
                    self.push_error("No issue selected");
                    return;
                }
            }
        };

        self.state.pending_move_action = Some(PendingMoveAction {
            entity_type: MoveEntityType::Issue,
            source_project_path: project_path,
            entity_id: issue_id,
            entity_display: display,
            selected_project_index: 0,
            search_filter: String::new(),
            show_confirmation: false,
            target_project_path: None,
        });
    }

    /// Start the move flow for a doc
    fn start_move_doc(&mut self) {
        let project_path = match &self.state.selected_project_path {
            Some(p) => p.clone(),
            None => {
                self.push_error("No project selected");
                return;
            }
        };

        // Check if there are other projects to move to
        let target_count = self
            .state
            .projects
            .iter()
            .filter(|p| p.initialized && p.path != project_path)
            .count();

        if target_count == 0 {
            self.push_error("No other projects to move to");
            return;
        }

        let (slug, display) = if let Some(s) = &self.state.selected_doc_slug {
            let doc = self.state.docs.iter().find(|d| &d.slug == s);
            match doc {
                Some(d) => (
                    d.slug.clone(),
                    format!("{} ({})", truncate_str(&d.title, 25), d.slug),
                ),
                None => {
                    self.push_error("Doc not found");
                    return;
                }
            }
        } else {
            match self.state.docs.get(self.state.selected_index) {
                Some(d) => (
                    d.slug.clone(),
                    format!("{} ({})", truncate_str(&d.title, 25), d.slug),
                ),
                None => {
                    self.push_error("No doc selected");
                    return;
                }
            }
        };

        self.state.pending_move_action = Some(PendingMoveAction {
            entity_type: MoveEntityType::Doc,
            source_project_path: project_path,
            entity_id: slug,
            entity_display: display,
            selected_project_index: 0,
            search_filter: String::new(),
            show_confirmation: false,
            target_project_path: None,
        });
    }

    /// Open VS Code at a specific path
    async fn open_vscode_at_path(&mut self, workspace_path: &str) {
        // Use the 'code' command to open VS Code at the workspace path
        // This will focus an existing window if one is open for this path
        match std::process::Command::new("code")
            .arg(workspace_path)
            .spawn()
        {
            Ok(_) => {
                self.copy_message = Some(format!("Opening existing workspace: {}", workspace_path));
            }
            Err(e) => {
                self.push_error(format!("Failed to open VS Code: {}", e));
            }
        }
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

        // Check context bar mouse (rows 0-2 for the bordered context bar)
        if mouse.row < UiArea::Sidebar.start_y()
            && self.state.current_view != View::Splash
            && self.handle_context_bar_mouse(mouse).await?
        {
            if view_before != self.state.current_view {
                self.refresh_current_actions().await;
            }
            return Ok(());
        }

        // Only check sidebar mouse if sidebar is visible (local actions sidebar)
        if crate::ui::sidebar::should_show_sidebar(&self.state.current_view)
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
            View::Organization => self.handle_organization_mouse(mouse).await?,
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
            View::DocCreate | View::DocEdit => self.handle_form_mouse(mouse).await?,
            View::Config => self.handle_scroll_mouse(mouse).await?,
            View::GlobalSearch => {
                let len = self.state.global_search_results.len();
                self.handle_list_mouse(mouse, len).await?
            }
            View::InitProject => self.handle_form_mouse(mouse).await?,
        }

        // Refresh actions if view changed to one that shows action panel
        if view_before != self.state.current_view {
            self.refresh_current_actions().await;
        }

        Ok(())
    }

    async fn handle_sidebar_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        use crate::ui::sidebar::get_local_actions;

        const SIDEBAR_WIDTH: u16 = 20;

        // Only handle events within the sidebar area
        if mouse.column >= SIDEBAR_WIDTH {
            return Ok(false);
        }

        let actions = get_local_actions(&self.state.current_view);
        if actions.is_empty() {
            return Ok(false);
        }

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.state.sidebar_scroll_offset =
                    self.state.sidebar_scroll_offset.saturating_sub(1);
                return Ok(true);
            }
            MouseEventKind::ScrollDown => {
                let max_offset = actions.len().saturating_sub(1);
                if self.state.sidebar_scroll_offset < max_offset {
                    self.state.sidebar_scroll_offset += 1;
                }
                return Ok(true);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let Some(item_index) = self.calculate_sidebar_item_from_click(mouse.row) else {
                    return Ok(false);
                };

                if item_index < actions.len() {
                    let action = &actions[item_index];

                    // Trigger button press animation
                    self.state.button_press =
                        Some(ButtonPressState::new(PressedButton::Sidebar(item_index)));

                    // Execute the local action
                    self.execute_local_action(action.id).await?;
                    return Ok(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Execute a local action by its ID
    async fn execute_local_action(&mut self, action_id: &str) -> Result<()> {
        match action_id {
            "new_issue" => {
                self.state.clear_form();
                self.navigate(View::IssueCreate, ViewParams::default());
            }
            "edit_issue" => {
                if let Some(issue_id) = &self.state.selected_issue_id.clone() {
                    if let Some(issue) = self
                        .state
                        .issues
                        .iter()
                        .find(|i| &i.id == issue_id)
                        .cloned()
                    {
                        self.state.load_issue_to_form(&issue);
                        self.navigate(View::IssueEdit, ViewParams::default());
                    }
                }
            }
            "new_pr" => {
                self.state.clear_form();
                self.navigate(View::PrCreate, ViewParams::default());
            }
            "edit_pr" => {
                if let Some(pr_id) = &self.state.selected_pr_id.clone() {
                    if let Some(pr) = self.state.prs.iter().find(|p| &p.id == pr_id).cloned() {
                        self.state.load_pr_to_form(&pr);
                        self.navigate(View::PrEdit, ViewParams::default());
                    }
                }
            }
            "new_doc" => {
                self.state.clear_form();
                self.navigate(View::DocCreate, ViewParams::default());
            }
            "edit_doc" => {
                if let Some(doc_slug) = &self.state.selected_doc_slug.clone() {
                    if let Some(doc) = self
                        .state
                        .docs
                        .iter()
                        .find(|d| &d.slug == doc_slug)
                        .cloned()
                    {
                        self.state.load_doc_to_form(&doc);
                        self.navigate(View::DocEdit, ViewParams::default());
                    }
                }
            }
            // Navigation actions between list pages
            "nav_issues" => {
                self.navigate(View::Issues, ViewParams::default());
            }
            "nav_prs" => {
                self.navigate(View::Prs, ViewParams::default());
            }
            "nav_docs" => {
                self.navigate(View::Docs, ViewParams::default());
            }
            "nav_projects" => {
                self.navigate(View::Projects, ViewParams::default());
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse events in the context bar (breadcrumb navigation)
    async fn handle_context_bar_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        // Only handle left clicks on the content row (row 1, between the borders)
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return Ok(false);
        }

        // Content is on row 1 (row 0 is top border, row 2 is bottom border)
        // Context bar spans rows 0 to CONTEXT_BAR_HEIGHT-1
        const CONTENT_ROW: u16 = 1; // Middle row of the 3-row context bar
        if mouse.row != UiArea::ContextBar.start_y() + CONTENT_ROW {
            return Ok(false);
        }

        // Find which segment was clicked
        for (start_col, end_col, target_view) in &self.state.context_bar_segments {
            if mouse.column >= *start_col && mouse.column < *end_col {
                // Special handling for Organization view - needs async data loading
                if matches!(target_view, View::Organization) {
                    // Get org slug from current project
                    if let Some(path) = &self.state.selected_project_path {
                        if let Some(project) = self.state.projects.iter().find(|p| &p.path == path)
                        {
                            if let Some(org_slug) = &project.organization_slug {
                                self.load_organization_data(org_slug.clone()).await;
                            }
                        }
                    }
                } else {
                    self.navigate_to_breadcrumb_view(target_view.clone());
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Load organization data and navigate to Organization view
    async fn load_organization_data(&mut self, org_slug: String) {
        // Fetch organization details
        match self.daemon.get_organization(&org_slug).await {
            Ok(Some(org)) => {
                self.state.current_organization = Some(org);
            }
            Ok(None) => {
                self.push_error("Organization not found");
                return;
            }
            Err(e) => {
                self.push_error(format!("Failed to load organization: {}", e));
                return;
            }
        }

        // Fetch projects for this organization
        match self.daemon.list_projects_by_organization(&org_slug).await {
            Ok(projects) => {
                self.state.organization_projects = projects.clone();

                // Fetch users for each project
                self.state.project_users.clear();
                for project in &projects {
                    if let Ok(users) = self.daemon.list_users(&project.path).await {
                        self.state.project_users.insert(project.path.clone(), users);
                    }
                }
            }
            Err(e) => {
                self.push_error(format!("Failed to load projects: {}", e));
                return;
            }
        }

        // Reset selection and navigate
        self.state.selected_project_in_org = 0;
        self.state.organization_focus = OrganizationFocus::ProjectsList;
        self.state.scroll_offset = 0;
        self.navigate(
            View::Organization,
            ViewParams {
                organization_slug: Some(org_slug),
                ..Default::default()
            },
        );
    }

    /// Navigate to a view from breadcrumb click
    fn navigate_to_breadcrumb_view(&mut self, target: View) {
        match target {
            View::Projects => {
                // Clicking on org or project in breadcrumb goes back to projects
                self.state.selected_project_path = None;
                self.state.sidebar_index = 0;
                self.navigate(View::Projects, ViewParams::default());
            }
            View::Issues => {
                self.state.sidebar_index = 1;
                self.navigate(View::Issues, ViewParams::default());
            }
            View::IssueDetail => {
                // Keep current selection, just navigate to detail view
                if let Some(id) = self.state.selected_issue_id.clone() {
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
                self.state.sidebar_index = 2;
                self.navigate(View::Prs, ViewParams::default());
            }
            View::PrDetail => {
                if let Some(id) = self.state.selected_pr_id.clone() {
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
                self.state.sidebar_index = 3;
                self.navigate(View::Docs, ViewParams::default());
            }
            View::DocDetail => {
                if let Some(slug) = self.state.selected_doc_slug.clone() {
                    self.navigate(
                        View::DocDetail,
                        ViewParams {
                            doc_slug: Some(slug),
                            ..Default::default()
                        },
                    );
                }
            }
            View::Config => {
                self.state.sidebar_index = 4;
                self.navigate(View::Config, ViewParams::default());
            }
            _ => {}
        }
    }

    /// Handle mouse events in list views (Issues, PRs, Docs)
    async fn handle_list_mouse(&mut self, mouse: MouseEvent, list_len: usize) -> Result<()> {
        let main_area_start_x = self.sidebar_width();
        const ACTION_PANEL_WIDTH: u16 = 22;

        let terminal_width = self.terminal_size.map(|(_, w)| w).unwrap_or(80);
        let action_panel_start_x = terminal_width.saturating_sub(ACTION_PANEL_WIDTH);
        let is_in_action_panel = mouse.column >= action_panel_start_x;

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if is_in_action_panel {
                    // Scroll action panel up
                    self.state.sidebar_scroll_offset =
                        self.state.sidebar_scroll_offset.saturating_sub(1);
                } else {
                    self.state.move_selection_up();
                    // Update list scroll offset to keep in sync with selected_index
                    if self.state.selected_index < self.state.list_scroll_offset {
                        self.state.list_scroll_offset = self.state.selected_index;
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if is_in_action_panel {
                    // Scroll action panel down
                    let max_offset = self.state.current_actions.actions.len().saturating_sub(1);
                    if self.state.sidebar_scroll_offset < max_offset {
                        self.state.sidebar_scroll_offset += 1;
                    }
                } else {
                    self.state.move_selection_down(list_len);
                    // Update list scroll offset to keep in sync with selected_index
                    let terminal_height = self.terminal_size.map(|(h, _)| h).unwrap_or(24) as usize;
                    let list_start_row = match self.state.current_view {
                        View::Issues | View::Prs => 6,
                        View::Docs => 4,
                        _ => UiArea::ListContent.start_y() as usize,
                    };
                    let visible_height = terminal_height.saturating_sub(list_start_row + 2);
                    if visible_height > 0
                        && self.state.selected_index
                            >= self.state.list_scroll_offset + visible_height
                    {
                        self.state.list_scroll_offset =
                            self.state.selected_index - visible_height + 1;
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
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

                if mouse.column >= main_area_start_x && UiArea::ListContent.contains_row(mouse.row)
                {
                    // Calculate list start row based on view
                    // Issues/PRs have a 2-row header, Docs doesn't
                    // Layout: context bar (3) + header (2 for Issues/PRs, 0 for Docs) + top border (1)
                    let list_start_row = match self.state.current_view {
                        View::Issues | View::Prs => 6, // context bar (3) + header (2) + border (1)
                        View::Docs => 4,               // context bar (3) + border (1), no header
                        _ => UiArea::ListContent.start_y() as usize,
                    };

                    // Only process if click is in the list area
                    if (mouse.row as usize) < list_start_row {
                        return Ok(());
                    }

                    // Calculate visible height for list items
                    let terminal_height = self.terminal_size.map(|(h, _)| h).unwrap_or(24) as usize;
                    let visible_height = terminal_height.saturating_sub(list_start_row + 2); // -2 for status bar + bottom border

                    // Calculate scroll offset: ratatui keeps selected item visible
                    let scroll_offset = if visible_height > 0 {
                        let min_offset =
                            self.state.selected_index.saturating_sub(visible_height - 1);
                        let max_offset = self.state.selected_index;
                        self.state.list_scroll_offset.clamp(min_offset, max_offset)
                    } else {
                        0
                    };

                    let relative_row = (mouse.row as usize).saturating_sub(list_start_row);
                    let clicked_index = relative_row + scroll_offset;
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
                if mouse.column >= main_area_start_x && UiArea::GridContent.contains_row(mouse.row)
                {
                    let rel_x = mouse.column - main_area_start_x - 1; // -1 for border
                                                                      // Account for scroll offset when calculating click position
                    let scroll_offset = self.state.scroll_offset as u16;
                    let rel_y = UiArea::GridContent.relative_row(mouse.row) + scroll_offset;

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
                                if let Ok(docs) = self.daemon.list_docs(&path).await {
                                    self.state.docs = docs;
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
        const FIELD_HEIGHT: u16 = 3;
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if mouse.column >= main_area_start_x && UiArea::FormContent.contains_row(mouse.row) {
                let field_index =
                    (UiArea::FormContent.relative_row(mouse.row) / FIELD_HEIGHT) as usize;
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

/// Truncate a string to a maximum length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ActionCategory, EntityAction, EntityActionsResponse};

    /// Test-only struct that mirrors App but doesn't require DaemonClient
    /// Only use this for testing synchronous methods that don't touch daemon
    #[allow(dead_code)]
    struct TestApp {
        state: AppState,
        quit: bool,
        status_message: Option<String>,
        copy_message: Option<String>,
        splash_state: Option<SplashState>,
        terminal_size: Option<(u16, u16)>,
        screen_buffer: ScreenBuffer,
        last_ctrl_c: Option<Instant>,
    }

    impl TestApp {
        fn new() -> Self {
            Self {
                state: AppState::default(),
                quit: false,
                status_message: None,
                copy_message: None,
                splash_state: None,
                terminal_size: Some((24, 80)),
                screen_buffer: ScreenBuffer::default(),
                last_ctrl_c: None,
            }
        }

        fn should_quit(&self) -> bool {
            self.quit
        }

        fn in_splash(&self) -> bool {
            matches!(self.state.current_view, View::Splash)
        }

        fn sidebar_width(&self) -> u16 {
            if self.state.selected_project_path.is_some() {
                20
            } else {
                0
            }
        }

        fn calculate_project_grid_columns(&self) -> usize {
            let width = self.terminal_size.map(|(_, w)| w).unwrap_or(80);
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

        fn calculate_project_grid_visible_height(&self) -> usize {
            let height = self.terminal_size.map(|(h, _)| h).unwrap_or(24);
            height.saturating_sub(2) as usize
        }

        fn navigate(&mut self, view: View, params: ViewParams) {
            self.state.selection.clear();
            self.state.view_history.push((
                self.state.current_view.clone(),
                self.state.view_params.clone(),
            ));
            self.state.current_view = view;
            self.state.view_params = params;
        }

        fn go_back(&mut self) {
            self.state.selection.clear();
            while let Some((view, params)) = self.state.view_history.pop() {
                if view.is_form_view() {
                    continue;
                }
                if matches!(view, View::Projects) {
                    self.state.selected_project_path = None;
                }
                self.state.current_view = view;
                self.state.view_params = params;
                return;
            }
        }

        fn calculate_action_index_from_click(&self, mouse_row: u16) -> Option<usize> {
            if mouse_row < 1 {
                return None;
            }
            let row_in_panel = mouse_row - 1;
            let grouped = self.state.current_actions.grouped_actions();
            let mut current_row: u16 = 0;
            let mut action_idx = 0;

            for (_category, actions) in &grouped {
                current_row += 1;
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

        fn calculate_sidebar_item_from_click(&self, mouse_row: u16) -> Option<usize> {
            const SIDEBAR_ITEM_COUNT: usize = 5;

            // Adjust mouse row for context bar offset
            let adjusted_row = UiArea::Sidebar.relative_row(mouse_row);

            let sidebar_height = self
                .terminal_size
                .map(|(h, _)| h.saturating_sub(1 + UiArea::height_adjustment()))
                .unwrap_or(24);

            let content_height = (SIDEBAR_ITEM_COUNT as u16) * BUTTON_HEIGHT;

            let (top_padding, first_visible) = if content_height <= sidebar_height {
                let padding = (sidebar_height - content_height) / 2;
                (padding, 0)
            } else {
                let up_indicator = if self.state.sidebar_scroll_offset > 0 {
                    1u16
                } else {
                    0
                };
                (up_indicator, self.state.sidebar_scroll_offset)
            };

            if adjusted_row < top_padding {
                return None;
            }

            let row_in_buttons = adjusted_row - top_padding;
            let clicked_visible_index = (row_in_buttons / BUTTON_HEIGHT) as usize;
            let item_index = first_visible + clicked_visible_index;

            if item_index < SIDEBAR_ITEM_COUNT {
                Some(item_index)
            } else {
                None
            }
        }

        fn update_splash(&mut self, terminal_height: u16) -> bool {
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

        fn update_button_press(&mut self) {
            if let Some(ref press) = self.state.button_press {
                if press.is_expired() {
                    self.state.button_press = None;
                }
            }
        }

        fn find_action_for_key(&self, key: &KeyEvent) -> Option<usize> {
            self.state
                .current_actions
                .actions
                .iter()
                .position(|a| App::key_matches_shortcut(key, &a.keyboard_shortcut))
        }
    }

    mod app_basic_tests {
        use super::*;

        #[test]
        fn test_should_quit_initially_false() {
            let app = TestApp::new();
            assert!(!app.should_quit());
        }

        #[test]
        fn test_in_splash_false_for_default_view() {
            let app = TestApp::new();
            // Default view is Projects
            assert!(!app.in_splash());
        }

        #[test]
        fn test_in_splash_true_when_splash_view() {
            let mut app = TestApp::new();
            app.state.current_view = View::Splash;
            assert!(app.in_splash());
        }

        #[test]
        fn test_sidebar_width_zero_without_project() {
            let app = TestApp::new();
            assert_eq!(app.sidebar_width(), 0);
        }

        #[test]
        fn test_sidebar_width_20_with_project() {
            let mut app = TestApp::new();
            app.state.selected_project_path = Some("/path/to/project".to_string());
            assert_eq!(app.sidebar_width(), 20);
        }
    }

    mod grid_calculation_tests {
        use super::*;

        #[test]
        fn test_calculate_project_grid_columns_default() {
            let app = TestApp::new();
            // 80 width - 0 sidebar - 2 borders = 78 usable
            // 78 / 19 (18 + 1 spacing) = 4 columns
            let cols = app.calculate_project_grid_columns();
            assert!(cols >= 1);
        }

        #[test]
        fn test_calculate_project_grid_columns_with_sidebar() {
            let mut app = TestApp::new();
            app.state.selected_project_path = Some("/path".to_string());
            // 80 width - 20 sidebar - 2 borders = 58 usable
            let cols = app.calculate_project_grid_columns();
            assert!(cols >= 1);
        }

        #[test]
        fn test_calculate_project_grid_columns_small_terminal() {
            let mut app = TestApp::new();
            app.terminal_size = Some((24, 30)); // Small width
            let cols = app.calculate_project_grid_columns();
            assert_eq!(cols, 1); // Should be at least 1
        }

        #[test]
        fn test_calculate_project_grid_columns_large_terminal() {
            let mut app = TestApp::new();
            app.terminal_size = Some((50, 200)); // Large width
            let cols = app.calculate_project_grid_columns();
            assert!(cols > 5); // Should have many columns
        }

        #[test]
        fn test_calculate_project_grid_visible_height() {
            let app = TestApp::new();
            let height = app.calculate_project_grid_visible_height();
            // 24 - 2 borders = 22
            assert_eq!(height, 22);
        }

        #[test]
        fn test_calculate_project_grid_visible_height_small_terminal() {
            let mut app = TestApp::new();
            app.terminal_size = Some((10, 80));
            let height = app.calculate_project_grid_visible_height();
            assert_eq!(height, 8); // 10 - 2
        }

        #[test]
        fn test_calculate_project_grid_columns_no_terminal_size() {
            let mut app = TestApp::new();
            app.terminal_size = None;
            // Should use default 80
            let cols = app.calculate_project_grid_columns();
            assert!(cols >= 1);
        }
    }

    mod navigation_tests {
        use super::*;

        #[test]
        fn test_navigate_changes_view() {
            let mut app = TestApp::new();
            app.state.current_view = View::Projects;
            app.navigate(View::Issues, ViewParams::default());
            assert_eq!(app.state.current_view, View::Issues);
        }

        #[test]
        fn test_navigate_saves_history() {
            let mut app = TestApp::new();
            app.state.current_view = View::Projects;
            app.navigate(View::Issues, ViewParams::default());
            assert_eq!(app.state.view_history.len(), 1);
            assert_eq!(app.state.view_history[0].0, View::Projects);
        }

        #[test]
        fn test_navigate_clears_selection() {
            let mut app = TestApp::new();
            app.state.selection.start(ScreenPos::new(5, 5));
            app.state.selection.update(ScreenPos::new(10, 10));
            app.navigate(View::Issues, ViewParams::default());
            assert!(!app.state.selection.has_selection());
        }

        #[test]
        fn test_go_back_restores_previous_view() {
            let mut app = TestApp::new();
            app.state.current_view = View::Projects;
            app.navigate(View::Issues, ViewParams::default());
            app.go_back();
            assert_eq!(app.state.current_view, View::Projects);
        }

        #[test]
        fn test_go_back_skips_form_views() {
            let mut app = TestApp::new();
            app.state.current_view = View::Projects;
            app.navigate(View::Issues, ViewParams::default());
            app.navigate(View::IssueCreate, ViewParams::default());
            app.navigate(View::IssueEdit, ViewParams::default());
            app.go_back();
            // Should skip IssueEdit and IssueCreate, go back to Issues
            assert_eq!(app.state.current_view, View::Issues);
        }

        #[test]
        fn test_go_back_clears_project_on_return_to_projects() {
            let mut app = TestApp::new();
            app.state.selected_project_path = Some("/path".to_string());
            app.state.current_view = View::Projects;
            app.navigate(View::Issues, ViewParams::default());
            app.go_back();
            assert!(app.state.selected_project_path.is_none());
        }

        #[test]
        fn test_go_back_empty_history_does_nothing() {
            let mut app = TestApp::new();
            app.state.current_view = View::Issues;
            app.go_back();
            // View unchanged when history is empty
            assert_eq!(app.state.current_view, View::Issues);
        }
    }

    mod key_matches_shortcut_tests {
        use super::*;

        #[test]
        fn test_empty_shortcut_returns_false() {
            let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
            assert!(!App::key_matches_shortcut(&key, ""));
        }

        #[test]
        fn test_single_char_matches() {
            let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "n"));
        }

        #[test]
        fn test_single_char_case_insensitive() {
            let key = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "n"));
        }

        #[test]
        fn test_ctrl_modifier() {
            let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
            assert!(App::key_matches_shortcut(&key, "Ctrl+D"));
        }

        #[test]
        fn test_shift_modifier() {
            let key = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT);
            assert!(App::key_matches_shortcut(&key, "Shift+N"));
        }

        #[test]
        fn test_enter_key() {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "Enter"));
        }

        #[test]
        fn test_escape_key() {
            let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "Escape"));
        }

        #[test]
        fn test_tab_key() {
            let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "Tab"));
        }

        #[test]
        fn test_backspace_key() {
            let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "Backspace"));
        }

        #[test]
        fn test_delete_key() {
            let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
            assert!(App::key_matches_shortcut(&key, "Delete"));
        }

        #[test]
        fn test_wrong_modifier_returns_false() {
            let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
            assert!(!App::key_matches_shortcut(&key, "Ctrl+D"));
        }

        #[test]
        fn test_wrong_key_returns_false() {
            let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
            assert!(!App::key_matches_shortcut(&key, "n"));
        }
    }

    mod action_index_calculation_tests {
        use super::*;

        fn create_app_with_actions(actions: Vec<EntityAction>) -> TestApp {
            let mut app = TestApp::new();
            app.state.current_actions = EntityActionsResponse { actions };
            app
        }

        #[test]
        fn test_click_on_border_returns_none() {
            let app = create_app_with_actions(vec![EntityAction {
                id: "test".to_string(),
                label: "Test".to_string(),
                category: ActionCategory::Crud,
                enabled: true,
                disabled_reason: String::new(),
                destructive: false,
                keyboard_shortcut: String::new(),
            }]);
            assert!(app.calculate_action_index_from_click(0).is_none());
        }

        #[test]
        fn test_click_on_first_action() {
            let app = create_app_with_actions(vec![EntityAction {
                id: "test".to_string(),
                label: "Test".to_string(),
                category: ActionCategory::Crud,
                enabled: true,
                disabled_reason: String::new(),
                destructive: false,
                keyboard_shortcut: String::new(),
            }]);
            // Row 1 is category header, rows 2-4 are first action button (BUTTON_HEIGHT = 3)
            let result = app.calculate_action_index_from_click(2);
            assert_eq!(result, Some(0));
        }

        #[test]
        fn test_no_actions_returns_none() {
            let app = create_app_with_actions(vec![]);
            assert!(app.calculate_action_index_from_click(5).is_none());
        }
    }

    mod sidebar_click_calculation_tests {
        use super::*;

        fn create_app_with_terminal_size(height: u16, width: u16) -> TestApp {
            let mut app = TestApp::new();
            app.terminal_size = Some((height, width));
            app
        }

        #[test]
        fn test_click_with_centered_content() {
            // Terminal height 50, sidebar height 46 (50 - 1 - 3 for status bar and context bar)
            // Content height: 5 items × 3 rows = 15
            // Top padding: (46 - 15) / 2 = 15
            // Mouse rows are offset by CONTEXT_BAR_HEIGHT (3)
            let app = create_app_with_terminal_size(50, 80);

            // Click on context bar area (rows 0-2) should return None
            assert!(app.calculate_sidebar_item_from_click(0).is_none());
            assert!(app.calculate_sidebar_item_from_click(2).is_none());

            // Click on padding area should return None (rows 3-17)
            assert!(app.calculate_sidebar_item_from_click(5).is_none());
            assert!(app.calculate_sidebar_item_from_click(17).is_none());

            // Click on first item (rows 18-20: adjusted 15-17)
            assert_eq!(app.calculate_sidebar_item_from_click(18), Some(0));
            assert_eq!(app.calculate_sidebar_item_from_click(19), Some(0));
            assert_eq!(app.calculate_sidebar_item_from_click(20), Some(0));

            // Click on second item (rows 21-23)
            assert_eq!(app.calculate_sidebar_item_from_click(21), Some(1));
            assert_eq!(app.calculate_sidebar_item_from_click(23), Some(1));

            // Click on third item (rows 24-26)
            assert_eq!(app.calculate_sidebar_item_from_click(24), Some(2));

            // Click on fourth item (rows 27-29)
            assert_eq!(app.calculate_sidebar_item_from_click(27), Some(3));

            // Click on fifth item (rows 30-32)
            assert_eq!(app.calculate_sidebar_item_from_click(30), Some(4));
            assert_eq!(app.calculate_sidebar_item_from_click(32), Some(4));

            // Click past the last item should return None
            assert!(app.calculate_sidebar_item_from_click(33).is_none());
        }

        #[test]
        fn test_click_on_small_terminal_no_scroll() {
            // Terminal height 20, sidebar height 16 (20 - 1 - 3)
            // Content height: 5 items × 3 rows = 15
            // This fits! Top padding: (16 - 15) / 2 = 0
            // Mouse rows offset by 3 (context bar)
            let app = create_app_with_terminal_size(20, 80);

            // First item at rows 3-5 (adjusted 0-2)
            assert_eq!(app.calculate_sidebar_item_from_click(3), Some(0));
            assert_eq!(app.calculate_sidebar_item_from_click(5), Some(0));

            // Second item at rows 6-8 (adjusted 3-5)
            assert_eq!(app.calculate_sidebar_item_from_click(6), Some(1));
        }

        #[test]
        fn test_click_on_small_terminal_with_scroll() {
            // Terminal height 13, sidebar height 9 (13 - 1 - 3)
            // Content height: 5 items × 3 rows = 15 (doesn't fit)
            // No scroll offset, so first item is visible
            let mut app = create_app_with_terminal_size(13, 80);
            app.state.sidebar_scroll_offset = 0;

            // First item at rows 3-5 (adjusted 0-2, no up indicator when not scrolled)
            assert_eq!(app.calculate_sidebar_item_from_click(3), Some(0));
            assert_eq!(app.calculate_sidebar_item_from_click(5), Some(0));

            // Second item at rows 6-8 (adjusted 3-5)
            assert_eq!(app.calculate_sidebar_item_from_click(6), Some(1));
        }

        #[test]
        fn test_click_on_scrolled_sidebar() {
            // Terminal height 13, sidebar height 9 (13 - 1 - 3)
            // Content height: 5 items × 3 rows = 15 (doesn't fit)
            // With scroll offset = 1, first visible is item 1
            let mut app = create_app_with_terminal_size(13, 80);
            app.state.sidebar_scroll_offset = 1;

            // Row 3 (adjusted 0) is the up indicator "^" - clicking should return None
            assert!(app.calculate_sidebar_item_from_click(3).is_none());

            // First visible item (item 1) at rows 4-6 (adjusted 1-3)
            assert_eq!(app.calculate_sidebar_item_from_click(4), Some(1));
            assert_eq!(app.calculate_sidebar_item_from_click(6), Some(1));

            // Second visible item (item 2) at rows 7-9 (adjusted 4-6)
            assert_eq!(app.calculate_sidebar_item_from_click(7), Some(2));
        }

        #[test]
        fn test_click_past_last_item_returns_none() {
            let app = create_app_with_terminal_size(50, 80);

            // With 50 row height, items end around row 33
            // Any click past the last item should return None
            assert!(app.calculate_sidebar_item_from_click(50).is_none());
            assert!(app.calculate_sidebar_item_from_click(100).is_none());
        }
    }

    mod splash_tests {
        use super::*;

        #[test]
        fn test_update_splash_no_splash_state() {
            let mut app = TestApp::new();
            app.splash_state = None;
            let result = app.update_splash(24);
            assert!(!result);
        }

        #[test]
        fn test_update_splash_transitions_when_complete() {
            let mut app = TestApp::new();
            // Create a splash state that has already completed (by skipping)
            let mut splash = SplashState::new(LogoStyle::default());
            splash.skip(); // This sets phase to Complete
            app.splash_state = Some(splash);
            app.state.current_view = View::Splash;

            // Check directly that splash is in Complete phase
            assert!(app.splash_state.as_ref().unwrap().is_complete());

            // Now when we check is_complete, it should be true and we transition
            if let Some(ref splash) = app.splash_state {
                if splash.is_complete() {
                    app.splash_state = None;
                    app.state.current_view = View::Projects;
                }
            }

            assert_eq!(app.state.current_view, View::Projects);
            assert!(app.splash_state.is_none());
        }

        #[test]
        fn test_update_splash_not_complete_yet() {
            let mut app = TestApp::new();
            app.splash_state = Some(SplashState::new(LogoStyle::default()));
            app.state.current_view = View::Splash;

            // Don't skip - animation just started
            let result = app.update_splash(24);
            assert!(!result);
            assert_eq!(app.state.current_view, View::Splash);
        }
    }

    mod button_press_tests {
        use super::*;

        #[test]
        fn test_update_button_press_clears_expired() {
            let mut app = TestApp::new();
            // Create an expired button press
            let mut press = ButtonPressState::new(PressedButton::Sidebar(0));
            // Manually set pressed_at to a past time
            press.pressed_at = Instant::now() - Duration::from_millis(200);
            app.state.button_press = Some(press);

            app.update_button_press();
            assert!(app.state.button_press.is_none());
        }

        #[test]
        fn test_update_button_press_keeps_active() {
            let mut app = TestApp::new();
            // Create a fresh button press
            app.state.button_press = Some(ButtonPressState::new(PressedButton::Sidebar(0)));

            app.update_button_press();
            assert!(app.state.button_press.is_some());
        }

        #[test]
        fn test_update_button_press_no_press() {
            let mut app = TestApp::new();
            app.state.button_press = None;
            app.update_button_press();
            assert!(app.state.button_press.is_none());
        }
    }

    mod find_action_tests {
        use super::*;

        #[test]
        fn test_find_action_for_key_with_match() {
            let mut app = TestApp::new();
            app.state.current_actions = EntityActionsResponse {
                actions: vec![
                    EntityAction {
                        id: "create".to_string(),
                        label: "Create".to_string(),
                        category: ActionCategory::Crud,
                        enabled: true,
                        disabled_reason: String::new(),
                        destructive: false,
                        keyboard_shortcut: "n".to_string(),
                    },
                    EntityAction {
                        id: "delete".to_string(),
                        label: "Delete".to_string(),
                        category: ActionCategory::Crud,
                        enabled: true,
                        disabled_reason: String::new(),
                        destructive: true,
                        keyboard_shortcut: "d".to_string(),
                    },
                ],
            };

            let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
            let result = app.find_action_for_key(&key);
            assert_eq!(result, Some(1));
        }

        #[test]
        fn test_find_action_for_key_no_match() {
            let mut app = TestApp::new();
            app.state.current_actions = EntityActionsResponse {
                actions: vec![EntityAction {
                    id: "create".to_string(),
                    label: "Create".to_string(),
                    category: ActionCategory::Crud,
                    enabled: true,
                    disabled_reason: String::new(),
                    destructive: false,
                    keyboard_shortcut: "n".to_string(),
                }],
            };

            let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
            let result = app.find_action_for_key(&key);
            assert!(result.is_none());
        }

        #[test]
        fn test_find_action_for_key_empty_actions() {
            let app = TestApp::new();
            let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
            let result = app.find_action_for_key(&key);
            assert!(result.is_none());
        }
    }
}
