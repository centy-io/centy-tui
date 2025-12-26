//! Confirmation dialog component for destructive actions

use crate::state::PendingDeleteAction;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render a confirmation dialog for delete action
pub fn render_confirm_dialog(frame: &mut Frame, action: &PendingDeleteAction) {
    let area = frame.area();

    // Dialog dimensions
    let dialog_width = 50u16;
    let dialog_height = 10u16;

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

    // Format display text (truncate if too long)
    let max_display_len = (dialog_width - 6) as usize;
    let display_text = truncate_string(&action.entity_display, max_display_len);

    // Build content
    let mut content = vec![
        Line::from(Span::styled(
            "Confirm Delete",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Are you sure you want to delete ",
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![
            Span::styled(
                format!("#{} ", action.entity_number),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(display_text, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![Span::styled("?", Style::default().fg(Color::White))]),
        Line::from(""),
    ];

    // Add options with selection highlighting
    let options = [false, true]; // Cancel, Delete
    let labels = ["Cancel", "Delete"];
    let colors = [Color::White, Color::Red];

    for (i, (&is_delete, &label)) in options.iter().zip(labels.iter()).enumerate() {
        let is_selected = action.selected_option == is_delete;
        let prefix = if is_selected { "▸ " } else { "  " };
        let base_color = colors[i];
        let style = if is_selected {
            Style::default().fg(base_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        content.push(Line::from(Span::styled(
            format!("{}{}", prefix, label),
            style,
        )));
    }

    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::styled(" select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" confirm  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
    ]));

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::new().bg(Color::Black).fg(Color::White));

    frame.render_widget(dialog, dialog_area);
}

/// Truncate a string to a maximum length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
