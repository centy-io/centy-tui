//! Field rendering utilities for forms

use crate::state::FormField;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw a form field using FormField from the domain layer
pub fn draw_field(frame: &mut Frame, area: Rect, field: &FormField, is_active: bool) {
    let style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let display_value = field.display_value();
    let display_str = if display_value.is_empty() && !is_active {
        "(empty)".to_string()
    } else {
        display_value
    };

    let cursor = if is_active { "▌" } else { "" };

    let content = if field.is_multiline {
        let lines: Vec<Line> = display_str
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
        let mut lines = lines;
        if is_active {
            if let Some(last) = lines.last_mut() {
                last.spans
                    .push(Span::styled(cursor, Style::default().fg(Color::Cyan)));
            } else {
                lines.push(Line::from(Span::styled(
                    cursor,
                    Style::default().fg(Color::Cyan),
                )));
            }
        }
        Paragraph::new(lines)
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(&display_str, style),
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
        ]))
    };

    let block = Block::default()
        .title(format!(" {} ", field.label))
        .borders(Borders::ALL)
        .border_style(border_style);

    frame.render_widget(content.wrap(Wrap { trim: false }).block(block), area);
}

/// Draw a form field with custom display value (for legacy compatibility)
pub fn draw_field_with_value(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_active: bool,
    is_multiline: bool,
) {
    let style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let display_value = if value.is_empty() && !is_active {
        "(empty)"
    } else {
        value
    };

    let cursor = if is_active { "▌" } else { "" };

    let content = if is_multiline {
        let lines: Vec<Line> = display_value
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
        let mut lines = lines;
        if is_active {
            if let Some(last) = lines.last_mut() {
                last.spans
                    .push(Span::styled(cursor, Style::default().fg(Color::Cyan)));
            } else {
                lines.push(Line::from(Span::styled(
                    cursor,
                    Style::default().fg(Color::Cyan),
                )));
            }
        }
        Paragraph::new(lines)
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(display_value, style),
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
        ]))
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style);

    frame.render_widget(content.wrap(Wrap { trim: false }).block(block), area);
}
