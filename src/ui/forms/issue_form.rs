//! Issue form rendering (create and edit)

use super::field_renderer::draw_field_with_value;
use crate::app::App;
use crate::ui::components::{render_action_button, BUTTON_HEIGHT};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw issue create form with action sidebar
pub fn draw_create(frame: &mut Frame, area: Rect, app: &App) {
    // Split into form (left) and action panel (right)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),    // Form area
            Constraint::Length(20), // Action panel
        ])
        .split(area);

    let form_area = main_chunks[0];
    let action_area = main_chunks[1];

    // Draw form on the left
    draw_create_form(frame, form_area, app);

    // Draw action panel on the right
    draw_create_action_panel(frame, action_area, app);
}

/// Draw the form fields
fn draw_create_form(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Description
            Constraint::Length(3), // Priority
        ])
        .margin(1)
        .split(area);

    // Form is focused when not on action panel (field 0-2)
    let form_focused = app.state.active_form_field < 3;
    let border_color = if form_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Create Issue ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
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
        chunks[2],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );
}

/// Draw the action panel sidebar
fn draw_create_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    // Action panel is focused when on field 3
    let is_focused = app.state.active_form_field == 3;
    let selected_button = app.state.form_selected_button;

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Actions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Layout for buttons vertically
    let button_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(BUTTON_HEIGHT), // Create (primary)
            Constraint::Length(BUTTON_HEIGHT), // Create & New
            Constraint::Length(BUTTON_HEIGHT), // Save as Draft
            Constraint::Length(BUTTON_HEIGHT), // Cancel
            Constraint::Min(0),                // remaining space
        ])
        .split(inner_area);

    // Render buttons (primary action first)
    render_action_button(
        frame,
        button_chunks[0],
        "Create",
        is_focused && selected_button == 0,
        true,
        Some(Color::Green),
        false,
    );

    render_action_button(
        frame,
        button_chunks[1],
        "Create & New",
        is_focused && selected_button == 1,
        true,
        Some(Color::Blue),
        false,
    );

    render_action_button(
        frame,
        button_chunks[2],
        "Save as Draft",
        is_focused && selected_button == 2,
        true,
        Some(Color::Yellow),
        false,
    );

    render_action_button(
        frame,
        button_chunks[3],
        "Cancel",
        is_focused && selected_button == 3,
        true,
        Some(Color::Gray),
        false,
    );
}

/// Draw issue edit form
pub fn draw_edit(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Description
            Constraint::Length(3), // Priority
            Constraint::Length(3), // Status
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let issue_number = app
        .state
        .selected_issue_id
        .as_ref()
        .and_then(|id| app.state.issues.iter().find(|i| &i.id == id))
        .map(|i| format!("#{}", i.display_number))
        .unwrap_or_default();

    let block = Block::default()
        .title(format!(" Edit Issue {} ", issue_number))
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
        chunks[2],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[3],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 3,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next field  "),
        Span::styled(
            crate::platform::SAVE_SHORTCUT,
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[4]);
}
