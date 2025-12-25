//! Error dialog component

use super::base::{render_dialog, DialogConfig};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    Frame,
};

/// Render an error dialog overlay centered on the screen
pub fn render_error_dialog(frame: &mut Frame, error_message: &str) {
    let hint = vec![
        Span::raw("Press "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" or "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to dismiss"),
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
