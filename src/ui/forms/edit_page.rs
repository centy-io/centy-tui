//! Unified edit page layout component
//!
//! Layout:
//! +------------------------------------------+
//! | Local |  Center Content        | Daemon  |
//! |Actions|  (Title, Description)  | Actions |
//! |       |                        |         |
//! |       |                        |         |
//! |       +------------------------+         |
//! |       |  Bottom Config Section |         |
//! +------------------------------------------+

use super::field_renderer::draw_field_with_value;
use crate::app::App;
use crate::ui::render_daemon_actions;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Entity type for the edit page
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Issue and Pr variants will be used when refactoring existing edit forms
pub enum EditEntityType {
    Issue,
    Pr,
    Doc,
}

/// Configuration for the edit page
pub struct EditPageConfig {
    pub entity_type: EditEntityType,
    pub title: String,
}

/// Draw the unified edit page layout
pub fn draw_edit_page(frame: &mut Frame, area: Rect, app: &App, config: &EditPageConfig) {
    // Main horizontal split: Left sidebar (placeholder) | Main content | Right actions
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(0),  // Left sidebar (placeholder for future)
            Constraint::Min(40),    // Main content area
            Constraint::Length(22), // Right action panel
        ])
        .split(area);

    let center_area = h_chunks[1];
    let action_area = h_chunks[2];

    // Vertical split for center: Main editable fields | Bottom config
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Main content (title, description)
            Constraint::Length(5), // Bottom config section
        ])
        .split(center_area);

    let main_content_area = v_chunks[0];
    let bottom_config_area = v_chunks[1];

    // Draw sections based on entity type
    match config.entity_type {
        EditEntityType::Issue => {
            draw_issue_edit_content(frame, main_content_area, app, &config.title);
            draw_issue_config_section(frame, bottom_config_area, app);
        }
        EditEntityType::Pr => {
            draw_pr_edit_content(frame, main_content_area, app, &config.title);
            draw_pr_config_section(frame, bottom_config_area, app);
        }
        EditEntityType::Doc => {
            draw_doc_edit_content(frame, main_content_area, app, &config.title);
            draw_doc_config_section(frame, bottom_config_area, app);
        }
    }

    // Draw daemon actions panel (same dynamic panel as detail views)
    let is_action_focused = is_action_panel_focused(app, &config.entity_type);
    render_daemon_actions(frame, action_area, app, is_action_focused);
}

/// Check if action panel should be focused based on current form field
fn is_action_panel_focused(app: &App, entity_type: &EditEntityType) -> bool {
    // For now, action panel is not focusable in edit views
    // This could be extended in the future
    let _ = entity_type;
    let _ = app;
    false
}

/// Draw issue edit main content (title, description)
fn draw_issue_edit_content(frame: &mut Frame, area: Rect, app: &App, title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Description
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field_with_value(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[1],
        "Description",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );
}

/// Draw issue config section (priority, status)
fn draw_issue_config_section(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Priority
            Constraint::Percentage(50), // Status
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Config ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);

    let priority_display = match app.state.form_priority {
        0 => "Default".to_string(),
        p => format!(
            "{} ({})",
            p,
            match p {
                1 => "high",
                2 => "medium",
                _ => "low",
            }
        ),
    };
    draw_field_with_value(
        frame,
        chunks[0],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[1],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 3,
        false,
    );
}

/// Draw PR edit main content (title, description, branches)
fn draw_pr_edit_content(frame: &mut Frame, area: Rect, app: &App, title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(4),    // Description
            Constraint::Length(3), // Source branch
            Constraint::Length(3), // Target branch
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field_with_value(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[1],
        "Description",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    draw_field_with_value(
        frame,
        chunks[2],
        "Source Branch",
        &app.state.form_source_branch,
        app.state.active_form_field == 2,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[3],
        "Target Branch",
        &app.state.form_target_branch,
        app.state.active_form_field == 3,
        false,
    );
}

/// Draw PR config section (priority, status)
fn draw_pr_config_section(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Priority
            Constraint::Percentage(50), // Status
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Config ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);

    let priority_display = format!("{}", app.state.form_priority);
    draw_field_with_value(
        frame,
        chunks[0],
        "Priority",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[1],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 5,
        false,
    );
}

/// Draw doc edit main content (title, content)
fn draw_doc_edit_content(frame: &mut Frame, area: Rect, app: &App, title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Content
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field_with_value(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[1],
        "Content (Markdown)",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );
}

/// Draw doc config section (slug)
fn draw_doc_config_section(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Slug
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Config ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);

    draw_field_with_value(
        frame,
        chunks[0],
        "Slug",
        &app.state.form_slug,
        app.state.active_form_field == 2,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled(
            crate::platform::SAVE_SHORTCUT,
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[1]);
}
