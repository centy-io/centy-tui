//! Sidebar drawing functions

use super::SIDEBAR_ITEMS;
use crate::app::App;
use crate::state::{PressedButton, View};
use crate::ui::components::{
    render_vertical_button_group, ButtonGroupItem, VerticalButtonGroupConfig,
};
use ratatui::{layout::Rect, Frame};

/// Draw the sidebar with boxed buttons (scrollable when screen is small)
pub fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let has_project = app.state.selected_project_path.is_some();

    // Build sidebar items
    let items: Vec<ButtonGroupItem> = SIDEBAR_ITEMS
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let is_selected = match idx {
                0 => matches!(app.state.current_view, View::Projects),
                1 => matches!(
                    app.state.current_view,
                    View::Issues | View::IssueDetail | View::IssueCreate | View::IssueEdit
                ),
                2 => matches!(
                    app.state.current_view,
                    View::Prs | View::PrDetail | View::PrCreate | View::PrEdit
                ),
                3 => matches!(
                    app.state.current_view,
                    View::Docs | View::DocDetail | View::DocCreate
                ),
                4 => matches!(app.state.current_view, View::Config),
                _ => false,
            };

            // Check if item requires project selection
            let requires_project = (1..=4).contains(&idx);
            let is_enabled = !requires_project || has_project;

            // Check if this sidebar button is pressed for animation
            let is_pressed = app
                .state
                .button_press
                .as_ref()
                .map(|bp| matches!(&bp.button, PressedButton::Sidebar(i) if *i == idx))
                .unwrap_or(false);

            ButtonGroupItem::new(*label)
                .selected(is_selected)
                .enabled(is_enabled)
                .pressed(is_pressed)
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
