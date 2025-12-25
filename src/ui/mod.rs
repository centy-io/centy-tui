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

    // Determine if we should show sidebar (only when a project is selected and not in a form view)
    let has_project = app.state.selected_project_path.is_some();
    let show_sidebar = has_project && !app.state.current_view.is_form_view();

    let main_area = if show_sidebar {
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

/// Render a dynamic action panel with boxed buttons
///
/// This is a shared component used by Issues, PRs, and Docs views.
pub fn render_action_panel(frame: &mut Frame, area: Rect, app: &App, is_focused: bool) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let selected_idx = app.state.action_panel_selected_index;

    // Create outer block
    let block = Block::default()
        .title(" Actions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Show loading/error/empty states as simple text
    if app.state.actions_loading {
        let content = Paragraph::new(Span::styled(
            " Loading...",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(content, inner_area);
        return;
    }

    if let Some(error) = &app.state.actions_error {
        let content = Paragraph::new(Span::styled(
            format!(" Error: {}", error),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(content, inner_area);
        return;
    }

    if app.state.current_actions.actions.is_empty() {
        let content = Paragraph::new(Span::styled(
            " No actions",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(content, inner_area);
        return;
    }

    // Build dynamic constraints for buttons
    // Each category: 1 row label + N buttons (BUTTON_HEIGHT each)
    let mut constraints: Vec<Constraint> = Vec::new();
    let grouped = app.state.current_actions.grouped_actions();

    for (_category, actions) in &grouped {
        constraints.push(Constraint::Length(1)); // Category label
        for _ in actions {
            constraints.push(Constraint::Length(BUTTON_HEIGHT));
        }
    }
    constraints.push(Constraint::Min(0)); // Help text area

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let mut chunk_idx = 0;
    let mut action_idx = 0;

    for (category, actions) in &grouped {
        // Render category label
        let label = Paragraph::new(Span::styled(
            format!("{}:", category.label().to_uppercase()),
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(label, chunks[chunk_idx]);
        chunk_idx += 1;

        // Render action buttons
        for action in actions {
            let is_selected = is_focused && action_idx == selected_idx;

            // Get color based on category and state
            let label_color = if action.enabled {
                Some(get_category_color(action.category, action.destructive))
            } else {
                None // Will use disabled style
            };

            // Build label with optional shortcut hint
            let label = if !action.keyboard_shortcut.is_empty() {
                format!("{} [{}]", action.label, action.keyboard_shortcut)
            } else {
                action.label.clone()
            };

            components::render_action_button(
                frame,
                chunks[chunk_idx],
                &label,
                is_selected,
                action.enabled,
                label_color,
            );

            chunk_idx += 1;
            action_idx += 1;
        }
    }

    // Render help text in remaining area
    let help_text = vec![
        Line::from(Span::styled(
            " j/k navigate",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            " Enter select",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            " Tab switch",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let help = Paragraph::new(help_text);
    frame.render_widget(help, chunks[chunk_idx]);
}

/// Get color for an action category
fn get_category_color(category: ActionCategory, destructive: bool) -> Color {
    if destructive {
        return Color::Red;
    }
    match category {
        ActionCategory::Crud => Color::Green,
        ActionCategory::Mode => Color::Yellow,
        ActionCategory::Status => Color::Cyan,
        ActionCategory::External => Color::White,
        ActionCategory::Unspecified => Color::White,
    }
}
