//! UI module for rendering the TUI

mod config_panel;
mod docs;
mod forms;
mod issues;
mod layout;
mod projects;
mod prs;
mod splash;
mod widgets;

pub use widgets::render_scrollable_list;

use crate::app::App;
use crate::state::{ScreenPos, View};
use ratatui::style::Modifier;
use ratatui::Frame;

/// Main draw function
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Update screen buffer dimensions
    app.screen_buffer.resize(area.width, area.height);

    // Splash screen takes full screen (no sidebar or status bar)
    if let View::Splash = &app.state.current_view {
        if let Some(ref splash_state) = app.splash_state {
            splash::draw(frame, area, splash_state);
        }
        return;
    }

    // Determine if we should show sidebar (only when a project is selected)
    let has_project = app.state.selected_project_path.is_some();

    let main_area = if has_project {
        // Draw the main layout with sidebar
        let (sidebar_area, main_area) = layout::create_layout(area);
        layout::draw_sidebar(frame, sidebar_area, app);
        main_area
    } else {
        // Full-width layout without sidebar
        layout::create_layout_no_sidebar(area)
    };

    // Draw main content based on current view
    match &app.state.current_view {
        View::Splash => {} // Handled above
        View::Projects => projects::draw(frame, main_area, app),
        View::Issues => issues::draw_list(frame, main_area, app),
        View::IssueDetail => issues::draw_detail(frame, main_area, app),
        View::IssueCreate => forms::draw_issue_create(frame, main_area, app),
        View::IssueEdit => forms::draw_issue_edit(frame, main_area, app),
        View::Prs => prs::draw_list(frame, main_area, app),
        View::PrDetail => prs::draw_detail(frame, main_area, app),
        View::PrCreate => forms::draw_pr_create(frame, main_area, app),
        View::PrEdit => forms::draw_pr_edit(frame, main_area, app),
        View::Docs => docs::draw_list(frame, main_area, app),
        View::DocDetail => docs::draw_detail(frame, main_area, app),
        View::DocCreate => forms::draw_doc_create(frame, main_area, app),
        View::Config => config_panel::draw(frame, main_area, app),
    }

    // Draw status bar
    layout::draw_status_bar(frame, app);

    // Apply text selection highlighting
    if app.state.selection.has_selection() || app.state.selection.is_selecting {
        // Populate screen buffer with current frame content
        populate_screen_buffer(frame, &mut app.screen_buffer);
        // Apply selection highlighting
        apply_selection_highlight(frame, &app.state.selection, area);
    }
}

/// Populate the screen buffer from the current frame
fn populate_screen_buffer(frame: &mut Frame, screen_buffer: &mut crate::state::ScreenBuffer) {
    screen_buffer.clear();
    let area = frame.area();

    for row in area.y..area.y + area.height {
        for col in area.x..area.x + area.width {
            if let Some(cell) = frame.buffer_mut().cell((col, row)) {
                let symbol = cell.symbol();
                if let Some(ch) = symbol.chars().next() {
                    if ch != ' ' {
                        screen_buffer.set(col, row, ch);
                    }
                }
            }
        }
    }
}

/// Apply selection highlighting to the frame
fn apply_selection_highlight(
    frame: &mut Frame,
    selection: &crate::state::SelectionState,
    area: ratatui::layout::Rect,
) {
    if let Some((_start, _end)) = selection.get_range() {
        for row in area.y..area.y + area.height {
            for col in area.x..area.x + area.width {
                let pos = ScreenPos::new(col, row);
                if selection.contains(pos) {
                    if let Some(cell) = frame.buffer_mut().cell_mut((col, row)) {
                        // Apply reverse video for selection
                        let current_style = cell.style();
                        cell.set_style(current_style.add_modifier(Modifier::REVERSED));
                    }
                }
            }
        }

        // Show keyboard cursor if in keyboard mode
        if selection.keyboard_mode {
            if let Some(cursor_pos) = selection.keyboard_cursor {
                if cursor_pos.col < area.width && cursor_pos.row < area.height {
                    if let Some(cell) =
                        frame.buffer_mut().cell_mut((cursor_pos.col, cursor_pos.row))
                    {
                        let current_style = cell.style();
                        cell.set_style(current_style.add_modifier(Modifier::SLOW_BLINK));
                    }
                }
            }
        }
    }
}
