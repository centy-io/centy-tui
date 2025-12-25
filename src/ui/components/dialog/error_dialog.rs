//! Error dialog component

use super::base::{render_dialog, DialogConfig, TEXT_STYLE};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    Frame,
};

/// Render an error dialog overlay centered on the screen
pub fn render_error_dialog(frame: &mut Frame, error_message: &str) {
    let hint = vec![
        Span::styled("Press ", TEXT_STYLE),
        Span::styled(
            "Enter",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", TEXT_STYLE),
        Span::styled(
            "Esc",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to dismiss", TEXT_STYLE),
    ];

    render_dialog(
        frame,
        DialogConfig {
            title: "Error",
            title_color: Color::Red,
            border_color: Color::Red,
            message: error_message,
            hint: Some(hint),
            max_width: 60,
        },
    );
}
