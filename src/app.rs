//! Application state and core logic

use crate::daemon::DaemonClient;
use crate::state::{AppState, LogoStyle, SplashState, View, ViewParams};
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
        let width = self
            .terminal_size
            .map(|(_, w)| w)
            .unwrap_or(80);

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

    /// Handle a key event
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Clear any status messages on key press
        self.copy_message = None;

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

        Ok(())
    }

    /// Navigate to a new view
    pub fn navigate(&mut self, view: View, params: ViewParams) {
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
        if let Some((view, params)) = self.state.view_history.pop() {
            self.state.current_view = view;
            self.state.view_params = params;
        }
    }

    /// Handle keys in Projects view
    async fn handle_projects_key(&mut self, key: KeyEvent) -> Result<()> {
        // Calculate grid dimensions for navigation
        let columns = self.calculate_project_grid_columns();
        let total = self.state.sorted_projects().len();

        match key.code {
            // Vertical navigation (moves by row in grid)
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down_grid(columns, total);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up_grid(columns);
            }
            // Horizontal navigation (moves within row in grid)
            KeyCode::Char('h') | KeyCode::Left => {
                self.state.move_selection_left(columns);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.state.move_selection_right(columns, total);
            }
            KeyCode::Enter => {
                let project_path = self
                    .state
                    .sorted_projects()
                    .get(self.state.selected_index)
                    .map(|p| p.path.clone());
                if let Some(path) = project_path {
                    self.state.selected_project_path = Some(path.clone());
                    if let Ok(issues) = self.daemon.list_issues(&path).await {
                        self.state.issues = issues;
                    }
                    self.navigate(View::Issues, ViewParams::default());
                }
            }
            KeyCode::Char('f') => {
                let project_path = self
                    .state
                    .sorted_projects()
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
                    .sorted_projects()
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
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state
                    .move_selection_down(self.state.sorted_issues().len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up();
            }
            KeyCode::Enter => {
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
            KeyCode::Char('n') => {
                self.navigate(View::IssueCreate, ViewParams::default());
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
            KeyCode::Char('y') => {
                let sorted = self.state.sorted_issues();
                if let Some(issue) = sorted.get(self.state.selected_index) {
                    self.copy_to_clipboard(&format!("#{} {}", issue.display_number, issue.title))?;
                    self.copy_message = Some("Copied title".to_string());
                }
            }
            KeyCode::Char('Y') => {
                let sorted = self.state.sorted_issues();
                if let Some(issue) = sorted.get(self.state.selected_index) {
                    self.copy_to_clipboard(&issue.id)?;
                    self.copy_message = Some("Copied UUID".to_string());
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Issue Detail view
    async fn handle_issue_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
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
            KeyCode::Char('j') | KeyCode::Down => self.state.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.state.scroll_up(),
            KeyCode::Char('d') | KeyCode::PageDown => self.state.scroll_down_page(),
            KeyCode::Char('u') | KeyCode::PageUp => self.state.scroll_up_page(),
            KeyCode::Char('y') => {
                if let Some(issue) = self.get_current_issue() {
                    self.copy_to_clipboard(&format!("#{} {}", issue.display_number, issue.title))?;
                    self.copy_message = Some("Copied title".to_string());
                }
            }
            KeyCode::Char('Y') => {
                if let Some(issue) = self.get_current_issue() {
                    self.copy_to_clipboard(&issue.id)?;
                    self.copy_message = Some("Copied UUID".to_string());
                }
            }
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Issue Create view
    async fn handle_issue_create_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(path) = &self.state.selected_project_path {
                    let result = self
                        .daemon
                        .create_issue(
                            path,
                            &self.state.form_title,
                            &self.state.form_description,
                            self.state.form_priority,
                        )
                        .await;
                    if result.is_ok() {
                        if let Ok(issues) = self.daemon.list_issues(path).await {
                            self.state.issues = issues;
                        }
                        self.state.clear_form();
                        self.go_back();
                    } else {
                        self.status_message = Some("Failed to create issue".to_string());
                    }
                }
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

    /// Handle keys in Issue Edit view
    async fn handle_issue_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Tab => self.state.next_form_field(),
            KeyCode::BackTab => self.state.prev_form_field(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let (Some(path), Some(issue_id)) = (
                    &self.state.selected_project_path,
                    &self.state.selected_issue_id,
                ) {
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

    /// Handle keys in PRs view
    async fn handle_prs_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self
                .state
                .move_selection_down(self.state.sorted_prs().len()),
            KeyCode::Char('k') | KeyCode::Up => self.state.move_selection_up(),
            KeyCode::Enter => {
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
            KeyCode::Char('n') => self.navigate(View::PrCreate, ViewParams::default()),
            KeyCode::Char('s') => self.state.cycle_pr_sort_field(),
            KeyCode::Char('S') => self.state.toggle_pr_sort_direction(),
            KeyCode::Char('a') => {
                self.state.show_merged_prs = !self.state.show_merged_prs;
                self.state.reset_selection();
            }
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in PR Detail view
    async fn handle_pr_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
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
            KeyCode::Char('j') | KeyCode::Down => self.state.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.state.scroll_up(),
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
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
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                    if result.is_ok() {
                        if let Ok(prs) = self.daemon.list_prs(path).await {
                            self.state.prs = prs;
                        }
                        self.state.clear_form();
                        self.go_back();
                    }
                }
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

    /// Handle keys in PR Edit view
    async fn handle_pr_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.clear_form();
                self.go_back();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let (Some(path), Some(pr_id)) = (
                    &self.state.selected_project_path,
                    &self.state.selected_pr_id,
                ) {
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
                    }
                }
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

    /// Handle keys in Docs view
    async fn handle_docs_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down(self.state.docs.len())
            }
            KeyCode::Char('k') | KeyCode::Up => self.state.move_selection_up(),
            KeyCode::Enter => {
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
            KeyCode::Char('n') => self.navigate(View::DocCreate, ViewParams::default()),
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Doc Detail view
    async fn handle_doc_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.state.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.state.scroll_up(),
            KeyCode::Char('d') | KeyCode::PageDown => self.state.scroll_down_page(),
            KeyCode::Char('u') | KeyCode::PageUp => self.state.scroll_up_page(),
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
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
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                    if result.is_ok() {
                        if let Ok(docs) = self.daemon.list_docs(path).await {
                            self.state.docs = docs;
                        }
                        self.state.clear_form();
                        self.go_back();
                    }
                }
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
        // Only check sidebar mouse if sidebar is visible (project selected)
        let has_project = self.state.selected_project_path.is_some();
        if has_project && self.state.current_view != View::Splash && self.handle_sidebar_mouse(mouse).await? {
            return Ok(());
        }
        match self.state.current_view {
            View::Splash => self.handle_splash_mouse(mouse).await?,
            View::Projects => {
                self.handle_projects_grid_mouse(mouse).await?
            }
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
        Ok(())
    }

    async fn handle_sidebar_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        const SIDEBAR_WIDTH: u16 = 20;
        const SIDEBAR_ITEMS_START_Y: u16 = 2;
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if mouse.column < SIDEBAR_WIDTH && mouse.row >= SIDEBAR_ITEMS_START_Y {
                let item_index = (mouse.row - SIDEBAR_ITEMS_START_Y) as usize;
                let has_project = self.state.selected_project_path.is_some();
                match item_index {
                    0 => {
                        self.state.sidebar_index = 0;
                        self.navigate(View::Projects, ViewParams::default());
                        return Ok(true);
                    }
                    1 if has_project => {
                        self.state.sidebar_index = 1;
                        self.navigate(View::Issues, ViewParams::default());
                        return Ok(true);
                    }
                    2 if has_project => {
                        self.state.sidebar_index = 2;
                        self.navigate(View::Prs, ViewParams::default());
                        return Ok(true);
                    }
                    3 if has_project => {
                        self.state.sidebar_index = 3;
                        self.navigate(View::Docs, ViewParams::default());
                        return Ok(true);
                    }
                    4 if has_project => {
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
        const LIST_ITEMS_START_Y: u16 = 1;
        match mouse.kind {
            MouseEventKind::ScrollUp => self.state.move_selection_up(),
            MouseEventKind::ScrollDown => self.state.move_selection_down(list_len),
            MouseEventKind::Down(MouseButton::Left) => {
                if mouse.column >= main_area_start_x && mouse.row >= LIST_ITEMS_START_Y {
                    let clicked_index = (mouse.row - LIST_ITEMS_START_Y) as usize;
                    if clicked_index < list_len {
                        self.state.selected_index = clicked_index;
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

        let columns = self.calculate_project_grid_columns();
        let total = self.state.sorted_projects().len();

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
                self.state.move_selection_up_grid(columns);
            }
            MouseEventKind::ScrollDown => {
                self.state.move_selection_down_grid(columns, total);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if mouse.column >= main_area_start_x && mouse.row >= GRID_START_Y {
                    let rel_x = mouse.column - main_area_start_x - 1; // -1 for border
                    let rel_y = mouse.row - GRID_START_Y;

                    // Calculate which card was clicked
                    let col = (rel_x / (card_width + CARD_SPACING_H)) as usize;
                    let row = (rel_y / CARD_HEIGHT) as usize;

                    if col < columns {
                        let clicked_index = row * columns + col;
                        if clicked_index < total {
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
                                // Double-click: open the project
                                let project_path = self
                                    .state
                                    .sorted_projects()
                                    .get(clicked_index)
                                    .map(|p| p.path.clone());
                                if let Some(path) = project_path {
                                    self.state.selected_project_path = Some(path.clone());
                                    if let Ok(issues) = self.daemon.list_issues(&path).await {
                                        self.state.issues = issues;
                                    }
                                    self.navigate(View::Issues, ViewParams::default());
                                }
                                // Reset click tracking after opening
                                self.state.last_click_time = None;
                                self.state.last_click_index = None;
                            } else {
                                // Single click: select the card and update tracking
                                self.state.selected_index = clicked_index;
                                self.state.last_click_time = Some(Instant::now());
                                self.state.last_click_index = Some(clicked_index);
                            }
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
        match mouse.kind {
            MouseEventKind::ScrollUp => self.state.scroll_up(),
            MouseEventKind::ScrollDown => self.state.scroll_down(),
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
}
