//! Organization view showing projects and users

use crate::app::App;
use crate::state::OrganizationFocus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Draw the organization view
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let content_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw the organization content
    draw_org_content(frame, content_area, app);

    // Draw the action panel
    draw_action_panel(frame, action_area, app);
}

/// Draw the organization content (left side)
fn draw_org_content(frame: &mut Frame, area: Rect, app: &App) {
    let org = match &app.state.current_organization {
        Some(o) => o,
        None => {
            let msg = Paragraph::new("Organization not found")
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .title(" Organization ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
            frame.render_widget(msg, area);
            return;
        }
    };

    // Border color based on focus
    let border_color = match app.state.organization_focus {
        OrganizationFocus::ProjectsList => Color::Cyan,
        OrganizationFocus::ActionPanel => Color::DarkGray,
    };

    let title = format!(
        " {} ({} projects) ",
        org.name,
        app.state.organization_projects.len()
    );

    if app.state.organization_projects.is_empty() {
        let content = Paragraph::new("No projects in this organization.")
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

    // Build list items for each project with users
    let items: Vec<ListItem> = app
        .state
        .organization_projects
        .iter()
        .enumerate()
        .flat_map(|(idx, project)| {
            let mut project_items: Vec<ListItem> = Vec::new();
            let is_selected = idx == app.state.selected_project_in_org;

            // Project line
            let prefix = if is_selected { "▸ " } else { "  " };
            let project_style = if is_selected {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let project_line = Line::from(vec![
                Span::raw(prefix),
                Span::styled(project.display_name(), project_style),
                Span::styled(
                    format!(
                        " ({}i, {}d, {}p)",
                        project.issue_count, project.doc_count, project.pr_count
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            project_items.push(ListItem::new(project_line));

            // Users for this project
            if let Some(users) = app.state.project_users.get(&project.path) {
                for user in users {
                    let user_line = Line::from(vec![
                        Span::raw("    "),
                        Span::styled(&user.name, Style::default().fg(Color::Cyan)),
                        if !user.email.is_empty() {
                            Span::styled(
                                format!(" <{}>", user.email),
                                Style::default().fg(Color::DarkGray),
                            )
                        } else {
                            Span::raw("")
                        },
                    ]);
                    project_items.push(ListItem::new(user_line));
                }
            }

            // Add spacing between projects
            project_items.push(ListItem::new(Line::from("")));
            project_items
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(list, area);
}

/// Draw the action panel (right side)
fn draw_action_panel(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.state.organization_focus, OrganizationFocus::ActionPanel);
    super::render_daemon_actions(frame, area, app, is_focused);
}
