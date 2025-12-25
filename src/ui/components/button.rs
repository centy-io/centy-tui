//! Button component for TUI

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Button height in rows (top border + content + bottom border)
pub const BUTTON_HEIGHT: u16 = 3;

/// Render a generic button with border
pub fn render_button(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    is_selected: bool,
    is_enabled: bool,
) {
    let border_style = if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let text_style = if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if !is_enabled {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    let paragraph = Paragraph::new(format!(" {content} ")).style(text_style);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    frame.render_widget(paragraph.block(block), area);
}

/// Render a sidebar button with key and label
pub fn render_sidebar_button(
    frame: &mut Frame,
    area: Rect,
    key: &str,
    label: &str,
    is_selected: bool,
    is_enabled: bool,
) {
    let content = format!("{key} {label}");
    render_button(frame, area, &content, is_selected, is_enabled);
}
