//! Application state and core logic

use crate::daemon::DaemonClient;
use crate::state::{AppState, LogoStyle, SplashState, View, ViewParams};
use anyhow::Result;
use cockpit::{PaneManager, PaneSize, SpawnConfig};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    /// Terminal size for pane initialization
    pub terminal_size: Option<(u16, u16)>,
    /// Esc key counter for double-Esc detection
    esc_count: u8,
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
            esc_count: 0,
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
            View::Daemon => self.handle_daemon_key(key).await?,
            View::Terminal => self.handle_terminal_key(key).await?,
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
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down(self.state.projects.len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up();
            }
            KeyCode::Enter => {
                if let Some(project) = self.state.projects.get(self.state.selected_index) {
                    self.state.selected_project_path = Some(project.path.clone());
                    // Load issues for selected project
                    if let Some(path) = &self.state.selected_project_path {
                        if let Ok(issues) = self.daemon.list_issues(path).await {
                            self.state.issues = issues;
                        }
                    }
                    self.navigate(View::Issues, ViewParams::default());
                }
            }
            KeyCode::Char('f') => {
                // Toggle favorite
                if let Some(project) = self.state.projects.get_mut(self.state.selected_index) {
                    let new_favorite = !project.is_favorite;
                    if self
                        .daemon
                        .set_project_favorite(&project.path, new_favorite)
                        .await
                        .is_ok()
                    {
                        project.is_favorite = new_favorite;
                    }
                }
            }
            KeyCode::Char('a') => {
                // Archive project
                if let Some(project) = self.state.projects.get(self.state.selected_index) {
                    if self
                        .daemon
                        .set_project_archived(&project.path, true)
                        .await
                        .is_ok()
                    {
                        // Reload projects
                        if let Ok(projects) = self.daemon.list_projects().await {
                            self.state.projects = projects;
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                // Untrack project (with confirmation)
                self.state.confirm_action = Some("untrack".to_string());
            }
            KeyCode::Char('n') => {
                self.navigate(View::Projects, ViewParams::default());
                // TODO: Navigate to project create
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
            KeyCode::Char('6') => {
                self.state.sidebar_index = 5;
                self.navigate(View::Daemon, ViewParams::default());
            }
            KeyCode::Char('7') => {
                self.state.sidebar_index = 6;
                self.navigate(View::Terminal, ViewParams::default());
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Issues view
    async fn handle_issues_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down(self.state.issues.len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up();
            }
            KeyCode::Enter => {
                if let Some(issue) = self.state.issues.get(self.state.selected_index) {
                    self.state.selected_issue_id = Some(issue.id.clone());
                    self.navigate(
                        View::IssueDetail,
                        ViewParams {
                            issue_id: Some(issue.id.clone()),
                            ..Default::default()
                        },
                    );
                }
            }
            KeyCode::Char('n') => {
                self.navigate(View::IssueCreate, ViewParams::default());
            }
            KeyCode::Char('s') => {
                // Cycle sort field
                self.state.cycle_issue_sort_field();
            }
            KeyCode::Char('S') => {
                // Toggle sort direction
                self.state.toggle_issue_sort_direction();
            }
            KeyCode::Char('a') => {
                // Toggle show all (including closed)
                self.state.show_closed_issues = !self.state.show_closed_issues;
            }
            KeyCode::Char('y') => {
                // Copy issue title
                if let Some(issue) = self.state.issues.get(self.state.selected_index) {
                    self.copy_to_clipboard(&format!("#{} {}", issue.display_number, issue.title))?;
                    self.copy_message = Some("Copied title".to_string());
                }
            }
            KeyCode::Char('Y') => {
                // Copy issue UUID
                if let Some(issue) = self.state.issues.get(self.state.selected_index) {
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.scroll_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.scroll_up();
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                self.state.scroll_down_page();
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                self.state.scroll_up_page();
            }
            KeyCode::Char('y') => {
                // Copy title
                if let Some(issue) = self.get_current_issue() {
                    self.copy_to_clipboard(&format!("#{} {}", issue.display_number, issue.title))?;
                    self.copy_message = Some("Copied title".to_string());
                }
            }
            KeyCode::Char('Y') => {
                // Copy UUID
                if let Some(issue) = self.get_current_issue() {
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

    /// Handle keys in Issue Create view
    async fn handle_issue_create_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle form input
        match key.code {
            KeyCode::Tab => {
                self.state.next_form_field();
            }
            KeyCode::BackTab => {
                self.state.prev_form_field();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save issue
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
                        // Reload issues and go back
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
            KeyCode::Char(c) => {
                self.state
                    .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Backspace => {
                self.state.form_backspace();
            }
            KeyCode::Enter => {
                if self.state.active_form_field == 1 {
                    // Add newline to description
                    self.state.form_description.push('\n');
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Issue Edit view
    async fn handle_issue_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        // Similar to create, but updates existing issue
        match key.code {
            KeyCode::Tab => {
                self.state.next_form_field();
            }
            KeyCode::BackTab => {
                self.state.prev_form_field();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save changes
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
                        // Reload issues and go back
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
            KeyCode::Char(c) => {
                self.state
                    .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Backspace => {
                self.state.form_backspace();
            }
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down(self.state.prs.len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up();
            }
            KeyCode::Enter => {
                if let Some(pr) = self.state.prs.get(self.state.selected_index) {
                    self.state.selected_pr_id = Some(pr.id.clone());
                    self.navigate(
                        View::PrDetail,
                        ViewParams {
                            pr_id: Some(pr.id.clone()),
                            ..Default::default()
                        },
                    );
                }
            }
            KeyCode::Char('n') => {
                self.navigate(View::PrCreate, ViewParams::default());
            }
            KeyCode::Char('s') => {
                self.state.cycle_pr_sort_field();
            }
            KeyCode::Char('S') => {
                self.state.toggle_pr_sort_direction();
            }
            KeyCode::Char('a') => {
                self.state.show_merged_prs = !self.state.show_merged_prs;
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back();
            }
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.scroll_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.scroll_up();
            }
            KeyCode::Esc | KeyCode::Backspace => {
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
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save PR
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
            KeyCode::Char(c) => {
                self.state
                    .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT));
            }
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
                // Save PR
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
            KeyCode::Char(c) => {
                self.state
                    .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Backspace => self.state.form_backspace(),
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Docs view
    async fn handle_docs_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_selection_down(self.state.docs.len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_selection_up();
            }
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
            KeyCode::Char('n') => {
                self.navigate(View::DocCreate, ViewParams::default());
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Doc Detail view
    async fn handle_doc_detail_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.scroll_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.scroll_up();
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                self.state.scroll_down_page();
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                self.state.scroll_up_page();
            }
            KeyCode::Esc | KeyCode::Backspace => {
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
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save doc
                if let Some(path) = &self.state.selected_project_path {
                    let result = self
                        .daemon
                        .create_doc(
                            path,
                            &self.state.form_title,
                            &self.state.form_description,
                            if self.state.form_slug.is_empty() {
                                None
                            } else {
                                Some(&self.state.form_slug)
                            },
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
            KeyCode::Char(c) => {
                self.state
                    .form_input_char(c, key.modifiers.contains(KeyModifiers::SHIFT));
            }
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.scroll_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.scroll_up();
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Daemon view
    async fn handle_daemon_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('r') => {
                // Restart daemon
                if self.daemon.restart().await.is_ok() {
                    self.status_message = Some("Daemon restart initiated".to_string());
                }
            }
            KeyCode::Char('s') => {
                // Shutdown daemon
                if self.daemon.shutdown().await.is_ok() {
                    self.status_message = Some("Daemon shutdown initiated".to_string());
                    self.state.daemon_connected = false;
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in Splash screen - any key skips the animation
    async fn handle_splash_key(&mut self, _key: KeyEvent) -> Result<()> {
        if let Some(ref mut splash) = self.splash_state {
            splash.skip();
        }
        Ok(())
    }

    /// Handle keys in Terminal view
    async fn handle_terminal_key(&mut self, key: KeyEvent) -> Result<()> {
        // Reset esc count on non-Esc keys
        if key.code != KeyCode::Esc {
            self.esc_count = 0;
        }

        match key.code {
            KeyCode::Esc => {
                self.esc_count += 1;
                if self.esc_count >= 2 {
                    // Double-Esc: exit terminal view
                    self.esc_count = 0;
                    self.go_back();
                }
            }
            KeyCode::Char('n') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Create new terminal pane
                self.create_terminal_pane().await?;
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Focus next pane
                if let Some(ref mut manager) = self.state.pane_manager {
                    manager.focus_next();
                }
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Focus previous pane
                if let Some(ref mut manager) = self.state.pane_manager {
                    manager.focus_prev();
                }
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Close current pane
                if let Some(ref mut manager) = self.state.pane_manager {
                    if let Some(focused) = manager.focused() {
                        manager.close_pane(focused);
                    }
                }
            }
            _ => {
                // Forward all other keys to the focused pane
                if let Some(ref mut manager) = self.state.pane_manager {
                    manager.route_key(key).await?;
                }
            }
        }
        Ok(())
    }

    /// Create a new terminal pane
    async fn create_terminal_pane(&mut self) -> Result<()> {
        // Get terminal size or use default
        let (height, width) = self.terminal_size.unwrap_or((24, 80));
        let pane_size = PaneSize::new(height, width);

        // Initialize pane manager if needed
        if self.state.pane_manager.is_none() {
            self.state.pane_manager = Some(PaneManager::new());
        }

        if let Some(ref mut manager) = self.state.pane_manager {
            // Spawn a new pane with default shell
            let config = SpawnConfig::new(pane_size);
            match manager.spawn(config) {
                Ok(_pane) => {
                    self.status_message = Some("Terminal pane created".to_string());
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to create pane: {e}"));
                }
            }
        }

        Ok(())
    }

    /// Get the currently selected issue
    fn get_current_issue(&self) -> Option<&crate::state::Issue> {
        self.state
            .selected_issue_id
            .as_ref()
            .and_then(|id| self.state.issues.iter().find(|i| &i.id == id))
    }

    /// Copy text to clipboard
    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }
}
