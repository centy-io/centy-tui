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

    /// Convert linear index to (row, col)
    fn index_to_pos(&self, index: usize) -> (usize, usize) {
        let row = index / self.columns;
        let col = index % self.columns;
        (row, col)
    }

    /// Get card area for a given position within the inner area
    fn card_area(&self, inner: Rect, row: usize, col: usize) -> Rect {
        let x = inner.x + (col as u16) * (self.card_width + CARD_SPACING_H);
        let y = inner.y + (row as u16) * CARD_HEIGHT;

        Rect {
            x,
            y,
            width: self.card_width,
            height: CARD_HEIGHT,
        }
    }
}

/// Draw the projects grid
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

    // Get sorted projects (favorites first)
    let sorted_projects = app.state.sorted_projects();
    let grid = GridLayout::new(area.width);

    // Inner area (inside outer block borders)
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Render each card
    for (idx, project) in sorted_projects.iter().enumerate() {
        let (row, col) = grid.index_to_pos(idx);
        let card_area = grid.card_area(inner, row, col);

        // Skip if card is outside visible area
        if card_area.y + card_area.height > area.y + area.height {
            continue;
        }

        let is_selected = idx == app.state.selected_index;
        draw_project_card(frame, card_area, project, is_selected);
    }

    // Draw confirmation dialog if active
    if let Some(action) = &app.state.confirm_action {
        if action == "untrack" {
            draw_confirm_dialog(frame, area, app);
        }
    }
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
    let max_name_len = inner.width.saturating_sub(favorite_prefix.chars().count() as u16) as usize;
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
        .sorted_projects()
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
