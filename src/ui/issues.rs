//! Issues list and detail views

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the issues list
pub fn draw_list(frame: &mut Frame, area: Rect, app: &App) {
    let sorted_issues = app.state.sorted_issues();
    let project_name = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|p| p.split('/').next_back())
        .unwrap_or("Project");

    // Header with sort info
    let sort_label = format!(
        "Sort: {} {}",
        app.state.issue_sort_field.label(),
        app.state.issue_sort_direction.symbol()
    );

    let closed_count = app
        .state
        .issues
        .iter()
        .filter(|i| i.metadata.status == "closed")
        .count();
    let filter_label = if closed_count > 0 {
        if app.state.show_closed_issues {
            format!("({} closed)", closed_count)
        } else {
            format!("(hiding {} closed)", closed_count)
        }
    } else {
        String::new()
    };

    if sorted_issues.is_empty() {
        let message = if !app.state.show_closed_issues && closed_count > 0 {
            "No open issues. Press 'a' to show all issues.\nPress 'n' to create a new issue."
        } else {
            "No issues found.\nPress 'n' to create a new issue."
        };
        let content = Paragraph::new(message)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(format!(" Issues - {} ", project_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(content, area);
        return;
    }

    // Split area for header and list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    // Draw header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(sort_label, Style::default().fg(Color::Cyan)),
        Span::styled(" [s]cycle [S]dir", Style::default().fg(Color::DarkGray)),
        Span::raw(" | "),
        Span::styled(filter_label, Style::default().fg(Color::DarkGray)),
        Span::styled(" [a]toggle", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(header, chunks[0]);

    // Draw list
    let items: Vec<ListItem> = sorted_issues
        .iter()
        .enumerate()
        .map(|(idx, issue)| {
            let is_selected = idx == app.state.selected_index;

            let prefix = if is_selected { "▸" } else { " " };
            let number = format!("#{}", issue.display_number);
            let priority_label = format!("[{}]", issue.priority_label());
            let status_label = format!("[{}]", issue.metadata.status);

            let priority_color = match issue.metadata.priority {
                1 => Color::Red,
                2 => Color::Yellow,
                _ => Color::Green,
            };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(number, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(priority_label, Style::default().fg(priority_color)),
                Span::raw(" "),
                Span::styled(status_label, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(&issue.title, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Issues - {} ", project_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, chunks[1]);
}

/// Draw issue detail view
pub fn draw_detail(frame: &mut Frame, area: Rect, app: &App) {
    let issue = app
        .state
        .selected_issue_id
        .as_ref()
        .and_then(|id| app.state.issues.iter().find(|i| &i.id == id));

    let Some(issue) = issue else {
        let message = Paragraph::new("Issue not found")
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(" Issue ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(message, area);
        return;
    };

    let title = format!(" #{} {} ", issue.display_number, issue.title);
    let priority_color = match issue.metadata.priority {
        1 => Color::Red,
        2 => Color::Yellow,
        _ => Color::Green,
    };

    let mut content = vec![
        // Status and Priority
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(&issue.metadata.status, Style::default().fg(Color::Cyan)),
            Span::raw(" | Priority: "),
            Span::styled(issue.priority_label(), Style::default().fg(priority_color)),
        ]),
        Line::from(""),
        // UUID
        Line::from(vec![
            Span::styled("UUID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&issue.id, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        // Timestamps
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                issue
                    .metadata
                    .created_at
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                issue
                    .metadata
                    .updated_at
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
            ),
        ]),
        Line::from(""),
        // Separator
        Line::from(Span::styled(
            "─".repeat(40),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        // Description header
        Line::from(Span::styled(
            "Description",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    // Add description lines
    if issue.description.is_empty() {
        content.push(Line::from(Span::styled(
            "No description provided.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for line in issue.description.lines() {
            content.push(Line::from(line.to_string()));
        }
    }

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.state.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}
