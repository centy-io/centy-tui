//! Daemon management panel view

use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw the daemon panel
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![
        Line::from(Span::styled(
            "Daemon Status",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Connection status
    let status_color = if app.state.daemon_connected {
        Color::Green
    } else {
        Color::Red
    };
    let status_text = if app.state.daemon_connected {
        "Connected"
    } else {
        "Disconnected"
    };

    content.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("● {}", status_text),
            Style::default().fg(status_color),
        ),
    ]));

    if let Some(info) = &app.state.daemon_info {
        content.push(Line::from(vec![
            Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&info.version),
        ]));

        let uptime = format_uptime(info.uptime_seconds);
        content.push(Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(Color::DarkGray)),
            Span::raw(uptime),
        ]));

        content.push(Line::from(vec![
            Span::styled("Tracked Projects: ", Style::default().fg(Color::DarkGray)),
            Span::raw(info.project_count.to_string()),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "─".repeat(40),
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(""));

    content.push(Line::from(Span::styled(
        "Actions",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw(" Restart daemon"),
    ]));
    content.push(Line::from(vec![
        Span::styled("[s]", Style::default().fg(Color::Red)),
        Span::raw(" Shutdown daemon"),
    ]));

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Daemon ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Format uptime in a human-readable way
fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}
