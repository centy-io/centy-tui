//! Documentation list and detail views

use super::render_scrollable_list;
use crate::app::App;
use crate::state::{DocDetailFocus, DocsListFocus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the docs list
pub fn draw_list(frame: &mut Frame, area: Rect, app: &App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let list_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw the docs list content
    draw_docs_list_content(frame, list_area, app);

    // Draw the action panel
    let is_focused = matches!(app.state.docs_list_focus, DocsListFocus::ActionPanel);
    super::render_action_panel(frame, action_area, app, is_focused);
}

/// Draw the docs list content (left side)
fn draw_docs_list_content(frame: &mut Frame, area: Rect, app: &App) {
    // Border color based on focus
    let border_color = match app.state.docs_list_focus {
        DocsListFocus::List => Color::Cyan,
        DocsListFocus::ActionPanel => Color::DarkGray,
    };

    let docs = &app.state.docs;
    let project_name = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|p| p.split('/').next_back())
        .unwrap_or("Project");

    if docs.is_empty() {
        let content = Paragraph::new("No docs found.\nPress 'n' to create a new doc.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(format!(" Docs - {} ", project_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
        frame.render_widget(content, area);
        return;
    }

    let items: Vec<ListItem> = docs
        .iter()
        .enumerate()
        .map(|(idx, doc)| {
            let is_selected = idx == app.state.selected_index;
            let prefix = if is_selected { "▸" } else { " " };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&doc.slug, Style::default().fg(Color::Cyan)),
                Span::raw(" - "),
                Span::styled(&doc.title, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Docs - {} ", project_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    render_scrollable_list(frame, area, list, app.state.selected_index);
}

/// Draw doc detail view
pub fn draw_detail(frame: &mut Frame, area: Rect, app: &App) {
    // Split area into content (left) and action panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(22)])
        .split(area);

    let content_area = h_chunks[0];
    let action_area = h_chunks[1];

    // Draw content
    draw_doc_detail_content(frame, content_area, app);

    // Draw action panel
    let is_focused = matches!(app.state.doc_detail_focus, DocDetailFocus::ActionPanel);
    super::render_action_panel(frame, action_area, app, is_focused);
}

/// Draw doc detail content (left side)
fn draw_doc_detail_content(frame: &mut Frame, area: Rect, app: &App) {
    // Border color based on focus
    let border_color = match app.state.doc_detail_focus {
        DocDetailFocus::Content => Color::Cyan,
        DocDetailFocus::ActionPanel => Color::DarkGray,
    };

    let doc = app
        .state
        .selected_doc_slug
        .as_ref()
        .and_then(|slug| app.state.docs.iter().find(|d| &d.slug == slug));

    let Some(doc) = doc else {
        let message = Paragraph::new("Doc not found")
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(" Doc ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
        frame.render_widget(message, area);
        return;
    };

    let title = format!(" {} - {} ", doc.slug, doc.title);

    let mut content = vec![
        // Metadata
        Line::from(vec![
            Span::styled("Slug: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&doc.slug, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::DarkGray)),
            Span::raw(doc.created_at.format("%Y-%m-%d %H:%M").to_string()),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::DarkGray)),
            Span::raw(doc.updated_at.format("%Y-%m-%d %H:%M").to_string()),
        ]),
        Line::from(""),
        // Separator
        Line::from(Span::styled(
            "─".repeat(40),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        // Content header
        Line::from(Span::styled(
            "Content",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Add content lines
    for line in doc.content.lines() {
        // Basic markdown rendering
        let styled_line = if let Some(stripped) = line.strip_prefix("# ") {
            Line::from(Span::styled(
                stripped,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        } else if let Some(stripped) = line.strip_prefix("## ") {
            Line::from(Span::styled(stripped, Style::default().fg(Color::Cyan)))
        } else if let Some(stripped) = line.strip_prefix("### ") {
            Line::from(Span::styled(stripped, Style::default().fg(Color::Blue)))
        } else if let Some(stripped) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            Line::from(vec![
                Span::styled("• ", Style::default().fg(Color::Cyan)),
                Span::raw(stripped),
            ])
        } else if line.starts_with("```") {
            Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)))
        } else {
            Line::from(line.to_string())
        };
        content.push(styled_line);
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
