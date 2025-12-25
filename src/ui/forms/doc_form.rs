//! Doc form rendering (create)

use super::field_renderer::draw_field_with_value;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw doc create form
pub fn draw_create(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Content
            Constraint::Length(3), // Slug
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Doc ")
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

    draw_field_with_value(
        frame,
        chunks[2],
        "Slug (optional)",
        &app.state.form_slug,
        app.state.active_form_field == 2,
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
    frame.render_widget(help, chunks[3]);
}
