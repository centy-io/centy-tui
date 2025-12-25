//! Projects grid view

use crate::app::App;
use crate::state::Project;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Grid layout configuration
const MIN_CARD_WIDTH: u16 = 18;
const CARD_HEIGHT: u16 = 4; // 1 (top border) + 2 (content) + 1 (bottom border)
const CARD_SPACING_H: u16 = 1;
const SECTION_HEADER_HEIGHT: u16 = 2;

/// Helper for grid layout calculations
struct GridLayout {
    columns: usize,
    card_width: u16,
}

impl GridLayout {
    /// Create a new grid layout based on available width
    fn new(area_width: u16) -> Self {
        // Account for outer block borders (2 chars total)
        let usable_width = area_width.saturating_sub(2);

        // Calculate how many columns fit
        // Each card needs: MIN_CARD_WIDTH + spacing (except last card)
        let columns = if usable_width >= MIN_CARD_WIDTH {
            ((usable_width + CARD_SPACING_H) / (MIN_CARD_WIDTH + CARD_SPACING_H)) as usize
        } else {
            1
        };
        let columns = columns.max(1);

        // Distribute remaining space evenly across cards
        let total_spacing = (columns.saturating_sub(1) as u16) * CARD_SPACING_H;
        let card_width = if columns > 0 {
            (usable_width.saturating_sub(total_spacing)) / columns as u16
        } else {
            usable_width
        };

        Self {
            columns,
            card_width: card_width.max(MIN_CARD_WIDTH),
        }
    }
}

/// Draw the projects grid with organization grouping
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let projects = &app.state.projects;

    // Draw outer block first
    let block = Block::default()
        .title(" Projects ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    if projects.is_empty() {
        // Draw empty state centered in area
        let message = Paragraph::new("No projects found.\nPress 'n' to add a project.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        frame.render_widget(message, inner);
        return;
    }

    // Get grouped projects
    let sections = app.state.grouped_projects();
    let grid = GridLayout::new(area.width);

    // Inner area (inside outer block borders)
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Apply scroll offset
    let scroll_offset = app.state.scroll_offset as u16;
    let visible_height = inner.height;

    let mut y_offset: u16 = 0;
    let mut project_index: usize = 0; // Global index into selectable projects

    for section in &sections {
        // Calculate header position with scroll
        let header_y = y_offset.saturating_sub(scroll_offset);

        // Draw section header if visible
        if y_offset + SECTION_HEADER_HEIGHT > scroll_offset
            && y_offset < scroll_offset + visible_height
        {
            let visible_header_y = if y_offset >= scroll_offset {
                inner.y + header_y
            } else {
                inner.y
            };

            let header_area = Rect {
                x: inner.x,
                y: visible_header_y,
                width: inner.width,
                height: SECTION_HEADER_HEIGHT.min(visible_height.saturating_sub(header_y)),
            };

            if header_area.height > 0 {
                draw_section_header(frame, header_area, &section.header, section.is_favorites);
            }
        }
        y_offset += SECTION_HEADER_HEIGHT;

        // Calculate rows needed for this section's projects
        let projects_in_section = section.projects.len();
        let rows_in_section = projects_in_section.div_ceil(grid.columns);

        // Draw project cards in grid
        for (idx_in_section, project) in section.projects.iter().enumerate() {
            let row = idx_in_section / grid.columns;
            let col = idx_in_section % grid.columns;

            let card_y = y_offset + (row as u16) * CARD_HEIGHT;

            // Check if card is in visible range (accounting for scroll)
            if card_y + CARD_HEIGHT > scroll_offset && card_y < scroll_offset + visible_height {
                let visible_card_y = inner.y + card_y.saturating_sub(scroll_offset);

                let card_area = Rect {
                    x: inner.x + (col as u16) * (grid.card_width + CARD_SPACING_H),
                    y: visible_card_y,
                    width: grid.card_width,
                    height: CARD_HEIGHT,
                };

                // Only draw if fully within visible area
                if card_area.y + card_area.height <= area.y + area.height {
                    let is_selected = project_index == app.state.selected_index;
                    draw_project_card(frame, card_area, project, is_selected);
                }
            }
            project_index += 1;
        }

        y_offset += (rows_in_section as u16) * CARD_HEIGHT;
    }

    // Draw scroll indicator if content exceeds visible area
    let total_content_height = y_offset;
    if total_content_height > visible_height {
        draw_scroll_indicator(
            frame,
            inner,
            scroll_offset,
            total_content_height,
            visible_height,
        );
    }

    // Draw confirmation dialog if active
    if let Some(action) = &app.state.confirm_action {
        if action == "untrack" {
            draw_confirm_dialog(frame, area, app);
        }
    }
}

/// Draw a scroll indicator on the right side
fn draw_scroll_indicator(
    frame: &mut Frame,
    inner: Rect,
    scroll_offset: u16,
    total_height: u16,
    visible_height: u16,
) {
    if total_height == 0 || visible_height == 0 {
        return;
    }

    // Calculate scrollbar position and size
    let scrollbar_height =
        ((visible_height as u32 * visible_height as u32) / total_height as u32).max(1) as u16;
    let scrollbar_height = scrollbar_height.min(visible_height);

    let scroll_range = total_height.saturating_sub(visible_height);
    let scrollbar_pos = if scroll_range > 0 {
        ((scroll_offset as u32 * (visible_height - scrollbar_height) as u32) / scroll_range as u32)
            as u16
    } else {
        0
    };

    // Draw scrollbar track and thumb
    for y in 0..visible_height {
        let char = if y >= scrollbar_pos && y < scrollbar_pos + scrollbar_height {
            "█"
        } else {
            "░"
        };
        let indicator = Paragraph::new(char).style(Style::default().fg(Color::DarkGray));
        let indicator_area = Rect {
            x: inner.x + inner.width - 1,
            y: inner.y + y,
            width: 1,
            height: 1,
        };
        frame.render_widget(indicator, indicator_area);
    }
}

/// Draw a section header
fn draw_section_header(frame: &mut Frame, area: Rect, title: &str, is_favorites: bool) {
    let style = if is_favorites {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };

    let prefix = if is_favorites { "★ " } else { "" };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Yellow)),
        Span::styled(title, style),
    ]));

    frame.render_widget(header, area);
}

/// Draw a single project card
fn draw_project_card(frame: &mut Frame, area: Rect, project: &Project, is_selected: bool) {
    // Card styling based on selection
    let border_style = if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let bg_style = if is_selected {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    // Draw card border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(bg_style);
    frame.render_widget(block, area);

    // Inner area for content (inside borders)
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Line 1: Favorite star + Project name (truncated if needed)
    let name = project.display_name();
    let favorite_prefix = if project.is_favorite { "★ " } else { "" };
    let max_name_len = inner
        .width
        .saturating_sub(favorite_prefix.chars().count() as u16) as usize;
    let truncated_name = if name.len() > max_name_len && max_name_len > 3 {
        format!("{}...", &name[..max_name_len.saturating_sub(3)])
    } else if name.len() > max_name_len {
        name[..max_name_len].to_string()
    } else {
        name.to_string()
    };

    let line1 = Line::from(vec![
        Span::styled(favorite_prefix, Style::default().fg(Color::Yellow)),
        Span::styled(
            truncated_name,
            if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ),
    ]);

    // Line 2: Compact stats with icons
    let mut stats_spans = vec![Span::styled(
        format!(
            "{}i {}d {}p",
            project.issue_count, project.doc_count, project.pr_count
        ),
        Style::default().fg(Color::DarkGray),
    )];

    // Add warning indicator if not initialized
    if !project.initialized {
        stats_spans.push(Span::styled(" !", Style::default().fg(Color::Yellow)));
    }

    let line2 = Line::from(stats_spans);

    // Render content lines
    let content = vec![line1, line2];
    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}

/// Draw untrack confirmation dialog
fn draw_confirm_dialog(frame: &mut Frame, area: Rect, app: &App) {
    let project = app
        .state
        .selectable_projects()
        .get(app.state.selected_index)
        .copied();
    let project_name = project.map(|p| p.display_name()).unwrap_or("Unknown");

    let dialog_width = 50;
    let dialog_height = 7;
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    let content = vec![
        Line::from(Span::styled(
            "Remove project from tracking?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("\"{}\"", project_name)),
        Line::from(Span::styled(
            "(This only removes it from the list, not from disk)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled(
                "y",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to confirm, "),
            Span::styled(
                "n",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to cancel"),
        ]),
    ];

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(dialog, dialog_area);
}
