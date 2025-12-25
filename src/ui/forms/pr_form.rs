//! PR form rendering (create and edit)

use super::field_renderer::draw_field_with_value;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw PR create form
pub fn draw_create(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(4),    // Description
            Constraint::Length(3), // Source branch
            Constraint::Length(3), // Target branch
            Constraint::Length(3), // Priority
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Pull Request ")
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
        if app.state.form_target_branch.is_empty() {
            "main"
        } else {
            &app.state.form_target_branch
        },
        app.state.active_form_field == 3,
        false,
    );

    let priority_display = match app.state.form_priority {
        0 => "Default".to_string(),
        p => format!("{}", p),
    };
    draw_field_with_value(
        frame,
        chunks[4],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[5]);
}

/// Draw PR edit form
pub fn draw_edit(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(3),    // Description
            Constraint::Length(3), // Source branch
            Constraint::Length(3), // Target branch
            Constraint::Length(3), // Priority
            Constraint::Length(3), // Status
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let pr_number = app
        .state
        .selected_pr_id
        .as_ref()
        .and_then(|id| app.state.prs.iter().find(|p| &p.id == id))
        .map(|p| format!("#{}", p.display_number))
        .unwrap_or_default();

    let block = Block::default()
        .title(format!(" Edit PR {} ", pr_number))
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

    let priority_display = format!("{}", app.state.form_priority);
    draw_field_with_value(
        frame,
        chunks[4],
        "Priority",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[5],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 5,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[6]);
}
