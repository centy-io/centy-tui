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
///
/// This is the core button rendering function that handles all button animation.
/// All other button functions should delegate to this one.
pub fn render_button(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    is_selected: bool,
    is_enabled: bool,
    is_pressed: bool,
    label_color: Option<Color>,
) {
    let base_border_style = if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let base_text_style = if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if !is_enabled {
        Style::default().fg(Color::DarkGray)
    } else if let Some(color) = label_color {
        Style::default().fg(color)
    } else {
        Style::default()
    };

    // Apply REVERSED modifier if pressed for inverted color effect
    let border_style = if is_pressed {
        base_border_style.add_modifier(Modifier::REVERSED)
    } else {
        base_border_style
    };

    let text_style = if is_pressed {
        base_text_style.add_modifier(Modifier::REVERSED)
    } else {
        base_text_style
    };

    let paragraph = Paragraph::new(format!(" {content} ")).style(text_style);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    frame.render_widget(paragraph.block(block), area);
}

/// Render a sidebar button
pub fn render_sidebar_button(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    is_selected: bool,
    is_enabled: bool,
    is_pressed: bool,
) {
    render_button(frame, area, label, is_selected, is_enabled, is_pressed, None);
}

/// Render an action panel button with optional custom label color
pub fn render_action_button(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    is_selected: bool,
    is_enabled: bool,
    label_color: Option<Color>,
    is_pressed: bool,
) {
    render_button(
        frame,
        area,
        label,
        is_selected,
        is_enabled,
        is_pressed,
        label_color,
    );
}
