//! Splash screen rendering with ASCII art logo

use crate::state::SplashState;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Build the CENTY text with styling
fn build_centy_text() -> Vec<Line<'static>> {
    let style = Style::default().fg(Color::Cyan);
    vec![
        Line::from(Span::styled(
            "   ██████╗███████╗███╗   ██╗████████╗██╗   ██╗",
            style,
        )),
        Line::from(Span::styled(
            "  ██╔════╝██╔════╝████╗  ██║╚══██╔══╝╚██╗ ██╔╝",
            style,
        )),
        Line::from(Span::styled(
            "  ██║     █████╗  ██╔██╗ ██║   ██║    ╚████╔╝ ",
            style,
        )),
        Line::from(Span::styled(
            "  ██║     ██╔══╝  ██║╚██╗██║   ██║     ╚██╔╝  ",
            style,
        )),
        Line::from(Span::styled(
            "  ╚██████╗███████╗██║ ╚████║   ██║      ██║   ",
            style,
        )),
        Line::from(Span::styled(
            "   ╚═════╝╚══════╝╚═╝  ╚═══╝   ╚═╝      ╚═╝   ",
            style,
        )),
    ]
}

/// Draw the splash screen
pub fn draw(frame: &mut Frame, area: Rect, splash_state: &SplashState) {
    // Build CENTY text
    let lines: Vec<Line> = build_centy_text();

    let logo_height = lines.len() as u16;
    let logo_width = 46u16; // Approximate width of the CENTY text

    // Calculate center position with scroll offset (can go negative/above screen)
    let base_y = area.y as i32 + (area.height.saturating_sub(logo_height)) as i32 / 2;
    let y_pos = base_y - splash_state.scroll_offset as i32;
    let x = area.x + (area.width.saturating_sub(logo_width)) / 2;

    // Calculate how many lines are visible (text scrolls off top)
    let lines_off_top = if y_pos < 0 { (-y_pos) as usize } else { 0 };

    // If all lines have scrolled off, we're done
    if lines_off_top >= lines.len() {
        return;
    }

    // Get only the visible lines (skip ones that went off top)
    let visible_lines: Vec<Line> = lines.into_iter().skip(lines_off_top).collect();
    let visible_height = visible_lines.len() as u16;

    // Render position (starts at 0 if text is going off top)
    let render_y = if y_pos < 0 { area.y } else { y_pos as u16 };

    let logo_area = Rect {
        x,
        y: render_y,
        width: logo_width.min(area.width),
        height: visible_height.min(area.height),
    };

    let paragraph = Paragraph::new(visible_lines);
    frame.render_widget(paragraph, logo_area);

    // Draw "Press any key to skip" hint at bottom (only when not scrolling)
    if splash_state.scroll_offset < 1.0 {
        let hint = "Press any key to skip";
        let hint_x = area.x + (area.width.saturating_sub(hint.len() as u16)) / 2;
        let hint_y = area.y + area.height - 2;

        let hint_line = Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray)));
        let hint_area = Rect {
            x: hint_x,
            y: hint_y,
            width: hint.len() as u16,
            height: 1,
        };
        frame.render_widget(Paragraph::new(hint_line), hint_area);
    }
}
