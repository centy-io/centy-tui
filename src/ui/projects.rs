//! Projects list view

use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Draw the projects list
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let projects = &app.state.projects;

    if projects.is_empty() {
        let message = Paragraph::new("No projects found.\nPress 'n' to add a project.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(" Projects ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(message, area);
        return;
    }

    // Sort projects: favorites first
    let mut sorted_projects: Vec<_> = projects.iter().enumerate().collect();
    sorted_projects.sort_by(|(_, a), (_, b)| b.is_favorite.cmp(&a.is_favorite));

    let items: Vec<ListItem> = sorted_projects
        .iter()
        .map(|(idx, project)| {
            let is_selected = *idx == app.state.selected_index;

            // Build the display line
            let favorite_indicator = if project.is_favorite { "★ " } else { "  " };
            let name = project.display_name();

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let favorite_style = if project.is_favorite {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▸" } else { " " };

            // First line: name
            let line1 = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(favorite_indicator, favorite_style),
                Span::styled(name, style),
            ]);

            // Second line: stats
            let stats = format!(
                "    {} issues, {} docs, {} PRs",
                project.issue_count, project.doc_count, project.pr_count
            );
            let mut stats_spans = vec![Span::styled(stats, Style::default().fg(Color::DarkGray))];

            if !project.initialized {
                stats_spans.push(Span::styled(
                    " (not initialized)",
                    Style::default().fg(Color::Yellow),
                ));
            }

            let line2 = Line::from(stats_spans);

            ListItem::new(vec![line1, line2])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Projects ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);

    // Draw confirmation dialog if active
    if let Some(action) = &app.state.confirm_action {
        if action == "untrack" {
            draw_confirm_dialog(frame, area, app);
        }
    }
}

/// Draw untrack confirmation dialog
fn draw_confirm_dialog(frame: &mut Frame, area: Rect, app: &App) {
    let project = app.state.projects.get(app.state.selected_index);
    let project_name = project
        .map(|p| p.display_name())
        .unwrap_or("Unknown");

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
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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
            Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" to confirm, "),
            Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
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
