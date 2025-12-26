//! Start work confirmation dialog component

use crate::state::PendingStartWorkAction;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render a dialog asking if user wants to move issue to "in progress"
pub fn render_start_work_dialog(frame: &mut Frame, action: &PendingStartWorkAction) {
    let area = frame.area();

    // Dialog dimensions
    let dialog_width = 50u16;
    let dialog_height = 9u16;

    // Center the dialog
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Build content
    let content = vec![
        Line::from(Span::styled(
            "Start Work",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Action: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&action.action_label, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Move issue to \"in progress\"?",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("y/Enter", Style::default().fg(Color::Cyan)),
            Span::styled(" yes  ", Style::default().fg(Color::DarkGray)),
            Span::styled("n", Style::default().fg(Color::Cyan)),
            Span::styled(" no  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::new().bg(Color::Black).fg(Color::White));

    frame.render_widget(dialog, dialog_area);
}
