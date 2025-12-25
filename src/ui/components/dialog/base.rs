//! Base dialog component

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Configuration for rendering a dialog
pub struct DialogConfig<'a> {
    /// Dialog title
    pub title: &'a str,
    /// Title color
    pub title_color: Color,
    /// Border color
    pub border_color: Color,
    /// Message content (can be multi-line with \n)
    pub message: &'a str,
    /// Hint text shown at the bottom (e.g., "Press Enter to dismiss")
    pub hint: Option<Vec<Span<'a>>>,
    /// Maximum width of the dialog
    pub max_width: u16,
}

impl<'a> Default for DialogConfig<'a> {
    fn default() -> Self {
        Self {
            title: "Dialog",
            title_color: Color::White,
            border_color: Color::White,
            message: "",
            hint: None,
            max_width: 60,
        }
    }
}

/// Render a centered dialog overlay
pub fn render_dialog(frame: &mut Frame, config: DialogConfig) {
    let area = frame.area();
    let padding = 4u16; // 2 chars padding on each side
    let max_line_width = (config.max_width - padding) as usize;

    // Word-wrap the message
    let wrapped_lines = wrap_text(config.message, max_line_width);
    let line_count = wrapped_lines.len();

    // Calculate dialog dimensions
    let content_width = wrapped_lines
        .iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(0)
        .max(config.title.len()) as u16;
    let dialog_width = (content_width + padding + 2).min(config.max_width); // +2 for borders

    // Height: title + blank + message lines + blank (if hint) + hint + borders
    let hint_lines = if config.hint.is_some() { 2 } else { 0 }; // blank + hint
    let dialog_height = (2 + line_count as u16 + hint_lines + 2).max(5); // +2 for borders

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
    let mut content = vec![
        Line::from(Span::styled(
            config.title,
            Style::default()
                .fg(config.title_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for line in wrapped_lines {
        content.push(Line::from(line));
    }

    if let Some(hint_spans) = config.hint {
        content.push(Line::from(""));
        content.push(Line::from(hint_spans));
    }

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.border_color))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(dialog, dialog_area);
}

/// Wrap text to fit within a maximum width
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.len() + word.len() + 1 > max_width && !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
