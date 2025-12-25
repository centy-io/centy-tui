//! Move entity dialog component - project picker with search and confirmation

use crate::state::{MoveEntityType, PendingMoveAction, Project};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the move dialog (project picker or confirmation)
pub fn render_move_dialog(
    frame: &mut Frame,
    action: &PendingMoveAction,
    available_projects: &[&Project],
) {
    if action.show_confirmation {
        render_confirmation(frame, action);
    } else {
        render_project_picker(frame, action, available_projects);
    }
}

fn render_project_picker(frame: &mut Frame, action: &PendingMoveAction, projects: &[&Project]) {
    // Dialog dimensions
    let dialog_width = 60u16;
    let dialog_height = 20u16;

    // Center dialog
    let area = frame.area();
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    frame.render_widget(Clear, dialog_area);

    // Build content: title, search box, project list, hints
    let entity_label = match action.entity_type {
        MoveEntityType::Issue => "Issue",
        MoveEntityType::Doc => "Doc",
    };

    let mut content = vec![
        Line::from(Span::styled(
            format!("Move {} to Project", entity_label),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Moving: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&action.entity_display, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        // Search field
        Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if action.search_filter.is_empty() {
                    "type to filter...".to_string()
                } else {
                    action.search_filter.clone()
                },
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
    ];

    // Filter projects by search
    let filtered: Vec<_> = projects
        .iter()
        .filter(|p| {
            action.search_filter.is_empty()
                || p.display_name()
                    .to_lowercase()
                    .contains(&action.search_filter.to_lowercase())
        })
        .collect();

    if filtered.is_empty() {
        content.push(Line::from(Span::styled(
            "  No matching projects",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Show up to 8 projects with scrolling
        let visible_count = 8.min(filtered.len());
        let start_idx = if action.selected_project_index >= visible_count {
            action.selected_project_index - visible_count + 1
        } else {
            0
        };

        for (idx, project) in filtered
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(visible_count)
        {
            let is_selected = idx == action.selected_project_index;
            let prefix = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            content.push(Line::from(Span::styled(
                format!("{}{}", prefix, project.display_name()),
                style,
            )));
        }

        // Show scroll indicator if needed
        if filtered.len() > visible_count {
            content.push(Line::from(Span::styled(
                format!("  ({} more...)", filtered.len() - visible_count),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Hints
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::styled(" select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" confirm  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
    ]));

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::new().bg(Color::Black).fg(Color::White));

    frame.render_widget(dialog, dialog_area);
}

fn render_confirmation(frame: &mut Frame, action: &PendingMoveAction) {
    let area = frame.area();
    let dialog_width = 55u16;
    let dialog_height = 10u16;

    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    frame.render_widget(Clear, dialog_area);

    let target_path = action.target_project_path.as_deref().unwrap_or("Unknown");
    // Extract just the folder name from the path
    let target_name = target_path.rsplit('/').next().unwrap_or(target_path);

    let entity_label = match action.entity_type {
        MoveEntityType::Issue => "issue",
        MoveEntityType::Doc => "doc",
    };

    let content = vec![
        Line::from(Span::styled(
            "Confirm Move",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("Move {} ", entity_label),
                Style::default().fg(Color::White),
            ),
            Span::styled(&action.entity_display, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("to ", Style::default().fg(Color::White)),
            Span::styled(target_name, Style::default().fg(Color::Green)),
            Span::styled("?", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::styled(" confirm  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::new().bg(Color::Black));

    frame.render_widget(dialog, dialog_area);
}
