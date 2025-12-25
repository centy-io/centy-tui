//! Sidebar drawing functions for local actions

use super::get_local_actions;
use crate::app::App;
use crate::state::PressedButton;
use crate::ui::components::{
    render_vertical_button_group, ButtonGroupItem, VerticalButtonGroupConfig,
};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw the local actions sidebar
pub fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let actions = get_local_actions(&app.state.current_view);

    if actions.is_empty() {
        // Draw empty sidebar with a subtle placeholder
        draw_empty_sidebar(frame, area);
        return;
    }

    // Build sidebar items from local actions
    let items: Vec<ButtonGroupItem> = actions
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            // Check if this sidebar button is pressed for animation
            let is_pressed = app
                .state
                .button_press
                .as_ref()
                .map(|bp| matches!(&bp.button, PressedButton::Sidebar(i) if *i == idx))
                .unwrap_or(false);

            // Build label with keyboard shortcut hint
            let label = if !action.keyboard_shortcut.is_empty() {
                format!("{} [{}]", action.label, action.keyboard_shortcut)
            } else {
                action.label.to_string()
            };

            ButtonGroupItem::new(label)
                .selected(false) // Local actions aren't "selected" like nav
                .enabled(true)
                .pressed(is_pressed)
                .color(Some(Color::Green)) // Edit actions in green
        })
        .collect();

    render_vertical_button_group(
        frame,
        area,
        &items,
        app.state.sidebar_scroll_offset,
        &VerticalButtonGroupConfig::default(),
    );
}

/// Draw an empty sidebar with placeholder
fn draw_empty_sidebar(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let content = Paragraph::new(" No actions")
        .style(Style::default().fg(Color::DarkGray))
        .block(block);

    frame.render_widget(content, area);
}
