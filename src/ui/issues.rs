//! Issues list and detail views

use crate::app::App;
use crate::state::{IssueDetailFocus, IssuesListFocus};
use super::render_scrollable_list;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the issues list
pub fn draw_list(frame: &mut Frame, area: Rect, app: &App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let list_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw the issues list content
    draw_issues_list_content(frame, list_area, app);

    // Draw the action panel
    draw_issues_list_action_panel(frame, action_area, app);
}

/// Draw the issues list content (left side)
fn draw_issues_list_content(frame: &mut Frame, area: Rect, app: &App) {
    let sorted_issues = app.state.sorted_issues();
    let project_name = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|p| p.split('/').next_back())
        .unwrap_or("Project");

    // Border color based on focus
    let border_color = match app.state.issues_list_focus {
        IssuesListFocus::List => Color::Cyan,
        IssuesListFocus::ActionPanel => Color::DarkGray,
    };

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
                    .border_style(Style::default().fg(border_color)),
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
            .border_style(Style::default().fg(border_color)),
    );

    render_scrollable_list(frame, chunks[1], list, app.state.selected_index);
}

/// Draw the issues list action panel (right side)
fn draw_issues_list_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    // Border color based on focus
    let border_color = match app.state.issues_list_focus {
        IssuesListFocus::ActionPanel => Color::Cyan,
        IssuesListFocus::List => Color::DarkGray,
    };

    let is_focused = matches!(app.state.issues_list_focus, IssuesListFocus::ActionPanel);
    let selected_index = app.state.issues_list_action_index;
    let has_selection = !app.state.sorted_issues().is_empty();

    let mut content = vec![Line::from("")];

    // Item 0: Create Issue
    let create_prefix = if is_focused && selected_index == 0 {
        Span::styled(" ▸ ", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("   ")
    };
    content.push(Line::from(vec![
        create_prefix,
        Span::styled(
            "Create Issue",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    content.push(Line::from(""));

    // Item 1: Move Issue
    let move_prefix = if is_focused && selected_index == 1 {
        Span::styled(" ▸ ", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("   ")
    };
    let move_style = if has_selection {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    content.push(Line::from(vec![
        move_prefix,
        Span::styled("Move Issue", move_style),
    ]));

    // Item 2: Delete Issue
    let delete_prefix = if is_focused && selected_index == 2 {
        Span::styled(" ▸ ", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("   ")
    };
    let delete_style = if has_selection {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    content.push(Line::from(vec![
        delete_prefix,
        Span::styled("Delete Issue", delete_style),
    ]));

    // Help text
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "─".repeat(18),
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
        " Tab to switch",
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

    // Split area into content (left) and action panel (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let content_area = chunks[0];
    let action_area = chunks[1];

    // Draw content
    draw_issue_content(frame, content_area, app, issue);

    // Draw action panel
    draw_action_panel(frame, action_area, app);
}

/// Draw the issue content (left side)
fn draw_issue_content(frame: &mut Frame, area: Rect, app: &App, issue: &crate::state::Issue) {
    let title = format!(" #{} {} ", issue.display_number, issue.title);
    let priority_color = match issue.metadata.priority {
        1 => Color::Red,
        2 => Color::Yellow,
        _ => Color::Green,
    };

    // Border color based on focus
    let border_color = match app.state.issue_detail_focus {
        IssueDetailFocus::Content => Color::Cyan,
        IssueDetailFocus::ActionPanel => Color::DarkGray,
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
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.state.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

/// Draw the action panel (right side)
fn draw_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    // Border color based on focus
    let border_color = match app.state.issue_detail_focus {
        IssueDetailFocus::ActionPanel => Color::Cyan,
        IssueDetailFocus::Content => Color::DarkGray,
    };

    let is_focused = matches!(app.state.issue_detail_focus, IssueDetailFocus::ActionPanel);
    let selected_index = app.state.action_panel_index;
    let llm_action = &app.state.action_panel_llm_action;

    // Get current issue status
    let current_status = app
        .state
        .selected_issue_id
        .as_ref()
        .and_then(|id| app.state.issues.iter().find(|i| &i.id == id))
        .map(|i| i.metadata.status.as_str())
        .unwrap_or("");

    // Get allowed states from config
    let allowed_states: Vec<&str> = app
        .state
        .config
        .as_ref()
        .map(|c| c.allowed_states.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    // Build action panel content
    let mut content = vec![Line::from("")];

    // Item 0: Open in VSCode
    let vscode_prefix = if is_focused && selected_index == 0 {
        Span::styled(" ▸ ", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("   ")
    };
    content.push(Line::from(vec![
        vscode_prefix,
        Span::styled(
            "Open in VSCode",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "─".repeat(18),
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(""));

    // Mode section
    content.push(Line::from(Span::styled(
        "Mode:",
        Style::default().fg(Color::DarkGray),
    )));

    // Item 1: Plan
    let plan_prefix = if is_focused && selected_index == 1 {
        " ▸ "
    } else {
        "   "
    };
    let plan_style = if matches!(llm_action, crate::state::LlmAction::Plan) {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    content.push(Line::from(vec![
        Span::styled(plan_prefix, Style::default().fg(Color::Cyan)),
        Span::styled("Plan", plan_style),
    ]));

    // Item 2: Implement
    let impl_prefix = if is_focused && selected_index == 2 {
        " ▸ "
    } else {
        "   "
    };
    let impl_style = if matches!(llm_action, crate::state::LlmAction::Implement) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    content.push(Line::from(vec![
        Span::styled(impl_prefix, Style::default().fg(Color::Cyan)),
        Span::styled("Impl", impl_style),
    ]));

    // Status section (only if we have allowed states)
    if !allowed_states.is_empty() {
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "─".repeat(18),
            Style::default().fg(Color::DarkGray),
        )));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "Status:",
            Style::default().fg(Color::DarkGray),
        )));

        for (i, state) in allowed_states.iter().enumerate() {
            let state_index = 3 + i; // Offset by VSCode, Plan, Impl
            let prefix = if is_focused && selected_index == state_index {
                " ▸ "
            } else {
                "   "
            };
            let is_current = *state == current_status;
            let suffix = if is_current { " ◀" } else { "" };

            let state_style = if is_current {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            content.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(*state, state_style),
                Span::styled(suffix, Style::default().fg(Color::Cyan)),
            ]));
        }
    }

    // Help text
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "─".repeat(18),
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

    let panel = Paragraph::new(content).block(
        Block::default()
            .title(" Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(panel, area);
}
