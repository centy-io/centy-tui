//! Layout components (header, sidebar, status bar)

use crate::app::App;
use crate::state::View;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Sidebar items
const SIDEBAR_ITEMS: &[(&str, &str)] = &[
    ("1", "Projects"),
    ("2", "Issues"),
    ("3", "PRs"),
    ("4", "Docs"),
    ("5", "Config"),
];

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

/// Draw the sidebar
pub fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let has_project = app.state.selected_project_path.is_some();

    let items: Vec<ListItem> = SIDEBAR_ITEMS
        .iter()
        .enumerate()
        .map(|(idx, (key, label))| {
            // Determine if this item should be highlighted
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

            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if !is_enabled {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▸ " } else { "  " };
            let content = format!("{prefix}[{key}] {label}");

            ListItem::new(Line::from(vec![Span::styled(content, style)]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Centy ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
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

    // Copy message
    if let Some(msg) = &app.copy_message {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(msg, Style::default().fg(Color::Green)));
    }

    // Status message
    if let Some(msg) = &app.status_message {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(msg, Style::default().fg(Color::Yellow)));
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

    // Quit hint on the right
    let quit_hint = " ^C:quit ";

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

/// Get keyboard hints for the current view
fn get_view_hints(view: &View) -> &'static str {
    match view {
        View::Splash => "Press any key to skip",
        View::Projects => "h/j/k/l:nav  Enter:select  f:fav  a:archive  x:untrack  n:new",
        View::Issues => "j/k:nav  Tab:panel  Enter:view/run  n:new  s/S:sort  a:all  y:copy",
        View::IssueDetail => "Tab:panel  e:edit  p/i:mode  Enter:run  j/k:scroll  Esc:back",
        View::IssueCreate | View::IssueEdit => "Tab:next  ^S:save  Esc:cancel",
        View::Prs => "j/k:nav  Enter:view  n:new  s/S:sort  a:all",
        View::PrDetail => "e:edit  j/k:scroll  Esc:back",
        View::PrCreate | View::PrEdit => "Tab:next  ^S:save  Esc:cancel",
        View::Docs => "j/k:nav  Enter:view  n:new  Esc:back",
        View::DocDetail => "j/k:scroll  d/u:page  Esc:back",
        View::DocCreate => "Tab:next  ^S:save  Esc:cancel",
        View::Config => "j/k:scroll  Esc:back",
    }
}
