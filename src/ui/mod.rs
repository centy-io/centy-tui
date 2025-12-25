//! UI module for rendering the TUI

pub mod components;
mod config_panel;
mod docs;
mod forms;
mod issues;
mod layout;
mod projects;
mod prs;
mod splash;
mod widgets;

pub use components::BUTTON_HEIGHT;
pub use widgets::render_scrollable_list;

use crate::app::App;
use crate::state::{ActionCategory, ScreenPos, View};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
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

/// Get style for an action category
fn get_category_style(category: ActionCategory, destructive: bool) -> Style {
    if destructive {
        return Style::default().fg(Color::Red);
    }
    match category {
        ActionCategory::Crud => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ActionCategory::Mode => Style::default().fg(Color::Yellow),
        ActionCategory::Status => Style::default().fg(Color::Cyan),
        ActionCategory::External => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ActionCategory::Unspecified => Style::default().fg(Color::White),
    }
}

/// Render a dynamic action panel
///
/// This is a shared component used by Issues, PRs, and Docs views.
pub fn render_action_panel(frame: &mut Frame, area: Rect, app: &App, is_focused: bool) {
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let selected_idx = app.state.action_panel_selected_index;

    let mut content: Vec<Line> = vec![Line::from("")];

    // Show loading state
    if app.state.actions_loading {
        content.push(Line::from(Span::styled(
            "  Loading...",
            Style::default().fg(Color::DarkGray),
        )));
    } else if let Some(error) = &app.state.actions_error {
        content.push(Line::from(Span::styled(
            format!("  Error: {}", error),
            Style::default().fg(Color::Red),
        )));
    } else if app.state.current_actions.actions.is_empty() {
        content.push(Line::from(Span::styled(
            "  No actions",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Render actions grouped by category
        let mut action_idx = 0;
        for (category, actions) in app.state.current_actions.grouped_actions() {
            // Category header
            content.push(Line::from(Span::styled(
                format!(" {}:", category.label().to_uppercase()),
                Style::default().fg(Color::DarkGray),
            )));

            for action in actions {
                let is_selected = is_focused && action_idx == selected_idx;

                let prefix = if is_selected {
                    Span::styled(" > ", Style::default().fg(Color::Cyan))
                } else {
                    Span::raw("   ")
                };

                // Action style based on enabled state and category
                let action_style = if !action.enabled {
                    Style::default().fg(Color::DarkGray) // Grayed out
                } else {
                    get_category_style(action.category, action.destructive)
                };

                // Show shortcut hint if available
                let shortcut_hint = if !action.keyboard_shortcut.is_empty() {
                    format!(" [{}]", action.keyboard_shortcut)
                } else {
                    String::new()
                };

                content.push(Line::from(vec![
                    prefix,
                    Span::styled(&action.label, action_style),
                    Span::styled(shortcut_hint, Style::default().fg(Color::DarkGray)),
                ]));

                action_idx += 1;
            }

            content.push(Line::from("")); // Space between categories
        }
    }

    // Help text
    content.push(Line::from(Span::styled(
        " ".to_string() + &"-".repeat(16),
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(Span::styled(
        " j/k navigate",
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(Span::styled(
        " Enter to select",
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(Span::styled(
        " Tab switch panel",
        Style::default().fg(Color::DarkGray),
    )));

    let panel = Paragraph::new(content).block(
        Block::default()
            .title(" Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(panel, area);
}
