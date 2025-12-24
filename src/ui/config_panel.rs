//! Configuration panel view

use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw the config panel
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let project_name = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|p| p.split('/').next_back())
        .unwrap_or("Project");

    let mut content = vec![
        Line::from(Span::styled(
            "Project Configuration",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if let Some(config) = &app.state.config {
        content.push(Line::from(vec![
            Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&config.version),
        ]));
        content.push(Line::from(vec![
            Span::styled("Priority Levels: ", Style::default().fg(Color::DarkGray)),
            Span::raw(config.priority_levels.to_string()),
        ]));
        content.push(Line::from(vec![
            Span::styled("Default State: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&config.default_state),
        ]));
        content.push(Line::from(""));

        content.push(Line::from(Span::styled(
            "Allowed States",
            Style::default().fg(Color::Cyan),
        )));
        for state in &config.allowed_states {
            content.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Cyan)),
                Span::raw(state),
            ]));
        }
    } else {
        content.push(Line::from(Span::styled(
            "No configuration loaded.",
            Style::default().fg(Color::DarkGray),
        )));
        content.push(Line::from(Span::styled(
            "Select a project to view its configuration.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(format!(" Config - {} ", project_name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.state.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}
