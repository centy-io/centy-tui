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

/// Draw issue create form
pub fn draw_create(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),             // Title
            Constraint::Min(6),                // Description
            Constraint::Length(3),             // Priority
            Constraint::Length(BUTTON_HEIGHT), // Action buttons
            Constraint::Length(1),             // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Issue ")
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

    // Get selected button from form state
    let buttons_row_active = app.state.active_form_field == 3;
    let selected_button = app.state.form_selected_button;

    // Render action buttons
    let button_area = chunks[3];
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Cancel
            Constraint::Length(1),  // spacer
            Constraint::Length(16), // Save as Draft
            Constraint::Length(1),  // spacer
            Constraint::Length(16), // Create & New
            Constraint::Length(1),  // spacer
            Constraint::Length(12), // Create
            Constraint::Min(0),     // remaining space
        ])
        .split(button_area);

    render_action_button(
        frame,
        button_chunks[0],
        "Cancel",
        buttons_row_active && selected_button == 0,
        true,
        Some(Color::Gray),
        false,
    );

    render_action_button(
        frame,
        button_chunks[2],
        "Save as Draft",
        buttons_row_active && selected_button == 1,
        true,
        Some(Color::Yellow),
        false,
    );

    render_action_button(
        frame,
        button_chunks[4],
        "Create & New",
        buttons_row_active && selected_button == 2,
        true,
        Some(Color::Blue),
        false,
    );

    render_action_button(
        frame,
        button_chunks[6],
        "Create",
        buttons_row_active && selected_button == 3,
        true,
        Some(Color::Green),
        false,
    );

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled("←/→", Style::default().fg(Color::Cyan)),
        Span::raw(": select button  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(": confirm  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[4]);
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
