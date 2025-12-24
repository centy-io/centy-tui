//! Pull Request list and detail views

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the PRs list
pub fn draw_list(frame: &mut Frame, area: Rect, app: &App) {
    let sorted_prs = app.state.sorted_prs();
    let project_name = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|p| p.split('/').last())
        .unwrap_or("Project");

    let sort_label = format!(
        "Sort: {} {}",
        app.state.pr_sort_field.label(),
        app.state.pr_sort_direction.symbol()
    );

    let hidden_count = app
        .state
        .prs
        .iter()
        .filter(|p| p.metadata.status == "merged" || p.metadata.status == "closed")
        .count();

    let filter_label = if hidden_count > 0 {
        if app.state.show_merged_prs {
            format!("({} merged/closed)", hidden_count)
        } else {
            format!("(hiding {} merged/closed)", hidden_count)
        }
    } else {
        String::new()
    };

    if sorted_prs.is_empty() {
        let message = if !app.state.show_merged_prs && hidden_count > 0 {
            "No open PRs. Press 'a' to show all PRs.\nPress 'n' to create a new PR."
        } else {
            "No PRs found.\nPress 'n' to create a new PR."
        };
        let content = Paragraph::new(message)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(format!(" Pull Requests - {} ", project_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(content, area);
        return;
    }

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
    let items: Vec<ListItem> = sorted_prs
        .iter()
        .enumerate()
        .map(|(idx, pr)| {
            let is_selected = idx == app.state.selected_index;
            let prefix = if is_selected { "▸" } else { " " };
            let number = format!("#{}", pr.display_number);

            let priority_color = match pr.metadata.priority {
                1 => Color::Red,
                2 => Color::Yellow,
                _ => Color::Green,
            };

            let status_color = match pr.metadata.status.as_str() {
                "draft" => Color::DarkGray,
                "open" => Color::Blue,
                "merged" => Color::Magenta,
                "closed" => Color::Red,
                _ => Color::DarkGray,
            };

            let priority_label = match pr.metadata.priority {
                1 => "high",
                2 => "med",
                _ => "low",
            };

            let branches = format!(
                "({} → {})",
                pr.metadata.source_branch, pr.metadata.target_branch
            );

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(number, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(format!("[{}]", priority_label), Style::default().fg(priority_color)),
                Span::raw(" "),
                Span::styled(format!("[{}]", pr.metadata.status), Style::default().fg(status_color)),
                Span::raw(" "),
                Span::styled(&pr.title, style),
                Span::styled(format!(" {}", branches), Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Pull Requests - {} ", project_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, chunks[1]);
}

/// Draw PR detail view
pub fn draw_detail(frame: &mut Frame, area: Rect, app: &App) {
    let pr = app
        .state
        .selected_pr_id
        .as_ref()
        .and_then(|id| app.state.prs.iter().find(|p| &p.id == id));

    let Some(pr) = pr else {
        let message = Paragraph::new("PR not found")
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(" Pull Request ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(message, area);
        return;
    };

    let title = format!(" PR #{} {} ", pr.display_number, pr.title);

    let status_color = match pr.metadata.status.as_str() {
        "draft" => Color::DarkGray,
        "open" => Color::Blue,
        "merged" => Color::Magenta,
        "closed" => Color::Red,
        _ => Color::DarkGray,
    };

    let priority_color = match pr.metadata.priority {
        1 => Color::Red,
        2 => Color::Yellow,
        _ => Color::Green,
    };

    let mut content = vec![
        // Status and Priority
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(&pr.metadata.status, Style::default().fg(status_color)),
            Span::raw(" | Priority: "),
            Span::styled(
                match pr.metadata.priority {
                    1 => "High",
                    2 => "Medium",
                    _ => "Low",
                },
                Style::default().fg(priority_color),
            ),
        ]),
        Line::from(""),
        // UUID
        Line::from(vec![
            Span::styled("UUID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&pr.id, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        // Branches
        Line::from(Span::styled(
            "Branches",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(&pr.metadata.source_branch, Style::default().fg(Color::Cyan)),
            Span::styled(" → ", Style::default().fg(Color::DarkGray)),
            Span::styled(&pr.metadata.target_branch, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
    ];

    // Linked Issues
    if !pr.metadata.linked_issues.is_empty() {
        content.push(Line::from(Span::styled(
            "Linked Issues",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for issue in &pr.metadata.linked_issues {
            content.push(Line::from(Span::styled(
                format!("  #{}", issue),
                Style::default().fg(Color::Yellow),
            )));
        }
        content.push(Line::from(""));
    }

    // Reviewers
    if !pr.metadata.reviewers.is_empty() {
        content.push(Line::from(Span::styled(
            "Reviewers",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for reviewer in &pr.metadata.reviewers {
            content.push(Line::from(format!("  {}", reviewer)));
        }
        content.push(Line::from(""));
    }

    // Timestamps
    content.push(Line::from(Span::styled(
        "Timeline",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    content.push(Line::from(vec![
        Span::styled("Created: ", Style::default().fg(Color::DarkGray)),
        Span::raw(pr.metadata.created_at.format("%Y-%m-%d %H:%M").to_string()),
    ]));
    content.push(Line::from(vec![
        Span::styled("Updated: ", Style::default().fg(Color::DarkGray)),
        Span::raw(pr.metadata.updated_at.format("%Y-%m-%d %H:%M").to_string()),
    ]));
    if let Some(merged_at) = pr.metadata.merged_at {
        content.push(Line::from(vec![
            Span::styled("Merged: ", Style::default().fg(Color::Magenta)),
            Span::raw(merged_at.format("%Y-%m-%d %H:%M").to_string()),
        ]));
    }
    if let Some(closed_at) = pr.metadata.closed_at {
        content.push(Line::from(vec![
            Span::styled("Closed: ", Style::default().fg(Color::Red)),
            Span::raw(closed_at.format("%Y-%m-%d %H:%M").to_string()),
        ]));
    }

    // Separator and Description
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "─".repeat(40),
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "Description",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    if pr.description.is_empty() {
        content.push(Line::from(Span::styled(
            "No description provided.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for line in pr.description.lines() {
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
