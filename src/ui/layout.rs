//! Layout components (header, sidebar, status bar)

use super::components::{render_sidebar_button, BUTTON_HEIGHT};
use crate::app::App;
use crate::state::{PressedButton, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Sidebar items
const SIDEBAR_ITEMS: &[&str] = &["Projects", "Issues", "PRs", "Docs", "Config"];

/// Create the main layout with sidebar
pub fn create_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar
            Constraint::Min(0),     // Main content
        ])
        .split(area);

    // Reserve bottom line for status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(chunks[1]);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Sidebar content
            Constraint::Length(1), // Status bar continuation
        ])
        .split(chunks[0]);

    (sidebar_chunks[0], main_chunks[0])
}

/// Create full-width layout without sidebar (for when no project is selected)
pub fn create_layout_no_sidebar(area: Rect) -> Rect {
    // Reserve bottom line for status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    chunks[0]
}

/// Draw the sidebar with boxed buttons
pub fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let has_project = app.state.selected_project_path.is_some();

    // Create vertical layout for button boxes (centered vertically)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                // Top padding (flex)
            Constraint::Length(BUTTON_HEIGHT), // Projects
            Constraint::Length(BUTTON_HEIGHT), // Issues
            Constraint::Length(BUTTON_HEIGHT), // PRs
            Constraint::Length(BUTTON_HEIGHT), // Docs
            Constraint::Length(BUTTON_HEIGHT), // Config
            Constraint::Min(0),                // Bottom padding (flex)
        ])
        .split(area);

    // Render each button
    for (idx, label) in SIDEBAR_ITEMS.iter().enumerate() {
        let is_selected = match idx {
            0 => matches!(app.state.current_view, View::Projects),
            1 => matches!(
                app.state.current_view,
                View::Issues | View::IssueDetail | View::IssueCreate | View::IssueEdit
            ),
            2 => matches!(
                app.state.current_view,
                View::Prs | View::PrDetail | View::PrCreate | View::PrEdit
            ),
            3 => matches!(
                app.state.current_view,
                View::Docs | View::DocDetail | View::DocCreate
            ),
            4 => matches!(app.state.current_view, View::Config),
            _ => false,
        };

        // Check if item requires project selection
        let requires_project = (1..=4).contains(&idx);
        let is_enabled = !requires_project || has_project;

        // Check if this sidebar button is pressed for animation
        let is_pressed = app
            .state
            .button_press
            .as_ref()
            .map(|bp| matches!(&bp.button, PressedButton::Sidebar(i) if *i == idx))
            .unwrap_or(false);

        render_sidebar_button(
            frame,
            chunks[idx + 1],
            label,
            is_selected,
            is_enabled,
            is_pressed,
        );
    }
}

/// Draw the status bar
pub fn draw_status_bar(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let status_area = Rect {
        x: 0,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    // Build status bar content
    let mut spans = vec![];

    // Connection status
    let conn_status = if app.state.daemon_connected {
        Span::styled(" ● ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ○ ", Style::default().fg(Color::Red))
    };
    spans.push(conn_status);

    // View-specific hints
    let hints = get_view_hints(&app.state.current_view);
    spans.push(Span::styled(hints, Style::default().fg(Color::DarkGray)));

    // Show disabled reason when action panel is focused and action is disabled
    if app.state.is_action_panel_focused() {
        if let Some(action) = app.state.selected_action() {
            if !action.enabled && !action.disabled_reason.is_empty() {
                spans.push(Span::raw(" | "));
                spans.push(Span::styled(
                    &action.disabled_reason,
                    Style::default().fg(Color::Red),
                ));
            }
        }
    }

    // Copy message
    if let Some(msg) = &app.copy_message {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(msg, Style::default().fg(Color::Green)));
    }

    // Project path
    if let Some(path) = &app.state.selected_project_path {
        let project_name = path.split('/').next_back().unwrap_or(path);
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            format!("📁 {project_name}"),
            Style::default().fg(Color::Blue),
        ));
    }

    // Quit hint on the right (double Ctrl+C to quit)
    let quit_hint = " ^C^C:quit ";

    let status = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status, status_area);

    // Render quit hint on the right
    let quit_area = Rect {
        x: area.width.saturating_sub(quit_hint.len() as u16),
        y: area.height.saturating_sub(1),
        width: quit_hint.len() as u16,
        height: 1,
    };
    let quit_widget =
        Paragraph::new(quit_hint).style(Style::default().bg(Color::DarkGray).fg(Color::Gray));
    frame.render_widget(quit_widget, quit_area);
}

/// Platform-specific save shortcut hint
#[cfg(target_os = "macos")]
const SAVE_HINT: &str = "Cmd+W:save";
#[cfg(not(target_os = "macos"))]
const SAVE_HINT: &str = "^W:save";

/// Get keyboard hints for the current view
fn get_view_hints(view: &View) -> String {
    match view {
        View::Splash => "Press any key to skip".to_string(),
        View::Projects => {
            "h/j/k/l:nav  Enter:select  f:fav  a:archive  x:untrack  n:new".to_string()
        }
        View::Issues => {
            "j/k:nav  Tab:panel  Enter:view/run  n:new  s/S:sort  a:all  y:copy".to_string()
        }
        View::IssueDetail => {
            "Tab:panel  e:edit  p/i:mode  Enter:run  j/k:scroll  Esc:back".to_string()
        }
        View::IssueCreate | View::IssueEdit => format!("Tab:next  {}  Esc:cancel", SAVE_HINT),
        View::Prs => "j/k:nav  Enter:view  n:new  s/S:sort  a:all".to_string(),
        View::PrDetail => "e:edit  j/k:scroll  Esc:back".to_string(),
        View::PrCreate | View::PrEdit => format!("Tab:next  {}  Esc:cancel", SAVE_HINT),
        View::Docs => "j/k:nav  Enter:view  n:new  Esc:back".to_string(),
        View::DocDetail => "j/k:scroll  d/u:page  Esc:back".to_string(),
        View::DocCreate => format!("Tab:next  {}  Esc:cancel", SAVE_HINT),
        View::Config => "j/k:scroll  Esc:back".to_string(),
    }
}
