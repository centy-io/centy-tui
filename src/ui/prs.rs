//! Pull Request list and detail views

use super::render_scrollable_list;
use crate::app::App;
use crate::state::{ListScope, PrDetailFocus, PrsListFocus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the PRs list
pub fn draw_list(frame: &mut Frame, area: Rect, app: &mut App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let list_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw the PRs list content
    draw_prs_list_content(frame, list_area, app);

    // Draw the daemon actions panel
    let is_focused = matches!(app.state.prs_list_focus, PrsListFocus::ActionPanel);
    super::render_daemon_actions(frame, action_area, app, is_focused);
}

/// Draw the PRs list content (left side)
fn draw_prs_list_content(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_org_scope = matches!(app.state.prs_list_scope, ListScope::Organization);

    let title = if is_org_scope {
        " Pull Requests - Org ".to_string()
    } else {
        let project_name = app
            .state
            .selected_project_path
            .as_ref()
            .and_then(|p| p.split('/').next_back())
            .unwrap_or("Project");
        format!(" Pull Requests - {} ", project_name)
    };

    // Border color based on focus
    let border_color = match app.state.prs_list_focus {
        PrsListFocus::List => Color::Cyan,
        PrsListFocus::ActionPanel => Color::DarkGray,
    };

    let sort_label = format!(
        "Sort: {} {}",
        app.state.pr_sort_field.label(),
        app.state.pr_sort_direction.symbol()
    );

    let scope_label = if is_org_scope {
        "[Org]".to_string()
    } else {
        String::new()
    };

    let item_count = if is_org_scope {
        app.state.sorted_org_prs().len()
    } else {
        app.state.sorted_prs().len()
    };

    let hidden_count = if is_org_scope {
        app.state
            .org_prs
            .iter()
            .filter(|p| p.pr.metadata.status == "merged" || p.pr.metadata.status == "closed")
            .count()
    } else {
        app.state
            .prs
            .iter()
            .filter(|p| p.metadata.status == "merged" || p.metadata.status == "closed")
            .count()
    };

    let filter_label = if hidden_count > 0 {
        if app.state.show_merged_prs {
            format!("({} merged/closed)", hidden_count)
        } else {
            format!("(hiding {} merged/closed)", hidden_count)
        }
    } else {
        String::new()
    };

    if item_count == 0 {
        let message = if !app.state.show_merged_prs && hidden_count > 0 {
            "No open PRs. Press 'a' to show all PRs.\nPress 'n' to create a new PR."
        } else {
            "No PRs found.\nPress 'n' to create a new PR."
        };
        let content = Paragraph::new(message)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
        frame.render_widget(content, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    // Draw header with scope indicator
    let mut header_spans = vec![
        Span::styled(sort_label, Style::default().fg(Color::Cyan)),
        Span::styled(" [s]cycle [S]dir", Style::default().fg(Color::DarkGray)),
    ];
    if !scope_label.is_empty() {
        header_spans.push(Span::raw(" | "));
        header_spans.push(Span::styled(
            scope_label,
            Style::default().fg(Color::Yellow),
        ));
    }
    header_spans.push(Span::styled(
        " [o]scope",
        Style::default().fg(Color::DarkGray),
    ));
    if !filter_label.is_empty() {
        header_spans.push(Span::raw(" | "));
        header_spans.push(Span::styled(
            filter_label,
            Style::default().fg(Color::DarkGray),
        ));
    }
    header_spans.push(Span::styled(
        " [a]toggle",
        Style::default().fg(Color::DarkGray),
    ));

    let header = Paragraph::new(Line::from(header_spans));
    frame.render_widget(header, chunks[0]);

    // Draw list - handle both scopes
    let items: Vec<ListItem> = if is_org_scope {
        app.state
            .sorted_org_prs()
            .iter()
            .enumerate()
            .map(|(idx, org_pr)| {
                let pr = &org_pr.pr;
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

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(number, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", priority_label),
                        Style::default().fg(priority_color),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", pr.metadata.status),
                        Style::default().fg(status_color),
                    ),
                    Span::raw(" "),
                    Span::styled(&pr.title, style),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", org_pr.project_name),
                        Style::default().fg(Color::Blue),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    } else {
        app.state
            .sorted_prs()
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
                    Span::styled(
                        format!("[{}]", priority_label),
                        Style::default().fg(priority_color),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", pr.metadata.status),
                        Style::default().fg(status_color),
                    ),
                    Span::raw(" "),
                    Span::styled(&pr.title, style),
                    Span::styled(
                        format!(" {}", branches),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    app.state.list_scroll_offset =
        render_scrollable_list(frame, chunks[1], list, app.state.selected_index);
}

/// Draw PR detail view
pub fn draw_detail(frame: &mut Frame, area: Rect, app: &App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let content_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw content
    draw_pr_detail_content(frame, content_area, app);

    // Draw daemon actions panel
    let is_focused = matches!(app.state.pr_detail_focus, PrDetailFocus::ActionPanel);
    super::render_daemon_actions(frame, action_area, app, is_focused);
}

/// Draw PR detail content (left side)
fn draw_pr_detail_content(frame: &mut Frame, area: Rect, app: &App) {
    // Border color based on focus
    let border_color = match app.state.pr_detail_focus {
        PrDetailFocus::Content => Color::Cyan,
        PrDetailFocus::ActionPanel => Color::DarkGray,
    };

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
                    .border_style(Style::default().fg(border_color)),
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
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(&pr.metadata.source_branch, Style::default().fg(Color::Cyan)),
            Span::styled(" → ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &pr.metadata.target_branch,
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
    ];

    // Linked Issues
    if !pr.metadata.linked_issues.is_empty() {
        content.push(Line::from(Span::styled(
            "Linked Issues",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
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
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        for reviewer in &pr.metadata.reviewers {
            content.push(Line::from(format!("  {}", reviewer)));
        }
        content.push(Line::from(""));
    }

    // Timestamps
    content.push(Line::from(Span::styled(
        "Timeline",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
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
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
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
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.state.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}
