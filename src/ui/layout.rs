//! Layout components (header, status bar)

use crate::app::App;
use crate::state::View;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Create the main layout with sidebar
/// Returns (context_bar_area, sidebar_area, main_content_area)
pub fn create_layout(area: Rect) -> (Rect, Rect, Rect) {
    // First split: context bar at top, rest below
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Context bar (with borders)
            Constraint::Min(0),    // Rest of content
        ])
        .split(area);

    let context_bar_area = vertical_chunks[0];
    let content_area = vertical_chunks[1];

    // Horizontal split for sidebar and main
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar
            Constraint::Min(0),     // Main content
        ])
        .split(content_area);

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

    (context_bar_area, sidebar_chunks[0], main_chunks[0])
}

/// Create full-width layout without sidebar (for when no project is selected)
/// Returns (context_bar_area, main_content_area)
pub fn create_layout_no_sidebar(area: Rect) -> (Rect, Rect) {
    // First split: context bar at top, rest below
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Context bar (with borders)
            Constraint::Min(0),    // Rest of content
        ])
        .split(area);

    let context_bar_area = vertical_chunks[0];
    let content_area = vertical_chunks[1];

    // Reserve bottom line for status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(content_area);

    (context_bar_area, chunks[0])
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
        View::Organization => "j/k:nav  Tab:panel  Enter:open  p:projects  Esc:back".to_string(),
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
        View::DocDetail => "e:edit  j/k:scroll  d/u:page  Esc:back".to_string(),
        View::DocCreate | View::DocEdit => format!("Tab:next  {}  Esc:cancel", SAVE_HINT),
        View::Config => "j/k:scroll  Esc:back".to_string(),
    }
}
