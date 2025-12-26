//! Init project form rendering

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

/// Draw init project form with action sidebar
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
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
    draw_form(frame, form_area, app);

    // Draw action panel on the right
    draw_action_panel(frame, action_area, app);
}

/// Draw the form fields
fn draw_form(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Project path
            Constraint::Length(4), // Help text
            Constraint::Min(0),    // Remaining space
        ])
        .margin(1)
        .split(area);

    // Form is focused when on field 0 (project path)
    let form_focused = app.state.active_form_field == 0;
    let border_color = if form_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Init Project ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    frame.render_widget(block, area);

    draw_field_with_value(
        frame,
        chunks[0],
        "Project Path",
        &app.state.form_project_path,
        app.state.active_form_field == 0,
        false,
    );

    // Help text
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "Enter the path to the project directory.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Works for both new and existing projects.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(help, chunks[1]);
}

/// Draw the action panel sidebar
fn draw_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    // Action panel is focused when on field 1
    let is_focused = app.state.active_form_field == 1;
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
            Constraint::Length(BUTTON_HEIGHT), // Initialize (primary)
            Constraint::Length(BUTTON_HEIGHT), // Cancel
            Constraint::Min(0),                // remaining space
        ])
        .split(inner_area);

    // Render buttons
    render_action_button(
        frame,
        button_chunks[0],
        "Initialize",
        is_focused && selected_button == 0,
        true,
        Some(Color::Green),
        false,
    );

    render_action_button(
        frame,
        button_chunks[1],
        "Cancel",
        is_focused && selected_button == 1,
        true,
        Some(Color::Gray),
        false,
    );
}
