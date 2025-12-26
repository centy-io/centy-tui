//! People list and detail views

use super::render_scrollable_list;
use crate::app::App;
use crate::state::{PeopleListFocus, PersonDetailFocus, User};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the people list
pub fn draw_list(frame: &mut Frame, area: Rect, app: &App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let list_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw the people list content
    draw_people_list_content(frame, list_area, app);

    // Draw the action panel
    draw_people_list_action_panel(frame, action_area, app);
}

/// Draw the people list content (left side)
fn draw_people_list_content(frame: &mut Frame, area: Rect, app: &App) {
    let sorted_people = app.state.sorted_people();
    let project_name = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|p| p.split('/').next_back())
        .unwrap_or("Project");

    // Border color based on focus
    let border_color = match app.state.people_list_focus {
        PeopleListFocus::List => Color::Cyan,
        PeopleListFocus::ActionPanel => Color::DarkGray,
    };

    // Header with sort info
    let sort_label = format!(
        "Sort: {} {}",
        app.state.people_sort_field.label(),
        app.state.people_sort_direction.symbol()
    );

    if sorted_people.is_empty() {
        let content = Paragraph::new(
            "No team members found.\nRun 'centy user sync' to import from git history.",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .title(format!(" People - {} ", project_name))
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
    ]));
    frame.render_widget(header, chunks[0]);

    // Draw list
    let items: Vec<ListItem> = sorted_people
        .iter()
        .enumerate()
        .map(|(idx, person)| {
            let is_selected = idx == app.state.selected_index;

            let prefix = if is_selected { "▸" } else { " " };

            // Role badge
            let role_span = if let Some(role) = &person.role {
                Span::styled(format!("[{}] ", role), Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            };

            // Stats summary
            let stats = format!(
                " ({} commits, {} issues)",
                person.commit_count, person.issues_assigned
            );

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                role_span,
                Span::styled(&person.name, Style::default().fg(Color::Cyan)),
                Span::styled(stats, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " People - {} ({}) ",
                project_name,
                sorted_people.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    render_scrollable_list(frame, chunks[1], list, app.state.selected_index);
}

/// Draw the people list daemon actions (right side)
fn draw_people_list_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.state.people_list_focus, PeopleListFocus::ActionPanel);
    super::render_daemon_actions(frame, area, app, is_focused);
}

/// Draw person detail view
pub fn draw_detail(frame: &mut Frame, area: Rect, app: &App) {
    let person = app
        .state
        .selected_person_id
        .as_ref()
        .and_then(|id| app.state.sorted_people().into_iter().find(|u| &u.id == id));

    let Some(person) = person else {
        let message = Paragraph::new("Person not found")
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(" Person ")
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
    draw_person_content(frame, content_area, app, person);

    // Draw daemon actions panel
    draw_person_action_panel(frame, action_area, app);
}

/// Draw the person content (left side)
fn draw_person_content(frame: &mut Frame, area: Rect, app: &App, person: &User) {
    let title = format!(" {} ", person.name);

    // Border color based on focus
    let border_color = match app.state.person_detail_focus {
        PersonDetailFocus::Content => Color::Cyan,
        PersonDetailFocus::ActionPanel => Color::DarkGray,
    };

    let mut content = vec![
        // Profile header
        Line::from(Span::styled(
            "Profile",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        // Name
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&person.name),
        ]),
        // Email
        Line::from(vec![
            Span::styled("Email: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&person.email),
        ]),
    ];

    // Role if available
    if let Some(role) = &person.role {
        content.push(Line::from(vec![
            Span::styled("Role: ", Style::default().fg(Color::DarkGray)),
            Span::styled(role, Style::default().fg(Color::Yellow)),
        ]));
    }

    // Git usernames
    if !person.git_usernames.is_empty() {
        content.push(Line::from(vec![
            Span::styled("Git aliases: ", Style::default().fg(Color::DarkGray)),
            Span::raw(person.git_usernames.join(", ")),
        ]));
    }

    // UUID
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("UUID: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&person.id, Style::default().fg(Color::DarkGray)),
    ]));

    // Separator
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "─".repeat(40),
        Style::default().fg(Color::DarkGray),
    )));
    content.push(Line::from(""));

    // Contribution Stats Section
    content.push(Line::from(Span::styled(
        "Contribution Stats",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("Commits: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            person.commit_count.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]));
    content.push(Line::from(vec![
        Span::styled("Issues Assigned: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            person.issues_assigned.to_string(),
            Style::default().fg(Color::Blue),
        ),
    ]));
    content.push(Line::from(vec![
        Span::styled("Issues Created: ", Style::default().fg(Color::DarkGray)),
        Span::raw(person.issues_created.to_string()),
    ]));
    content.push(Line::from(vec![
        Span::styled("PRs Created: ", Style::default().fg(Color::DarkGray)),
        Span::raw(person.prs_created.to_string()),
    ]));

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

/// Draw the person action panel (right side)
fn draw_person_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(
        app.state.person_detail_focus,
        PersonDetailFocus::ActionPanel
    );
    super::render_daemon_actions(frame, area, app, is_focused);
}
