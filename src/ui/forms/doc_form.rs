//! Doc form rendering (create)

use super::field_renderer::draw_field_with_value;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Check if the selected project belongs to an organization
fn has_organization(app: &App) -> bool {
    app.state
        .selected_project_path
        .as_ref()
        .and_then(|path| app.state.projects.iter().find(|p| &p.path == path))
        .and_then(|p| p.organization_slug.as_ref())
        .is_some()
}

/// Get the organization name for display
fn get_org_name(app: &App) -> Option<String> {
    app.state
        .selected_project_path
        .as_ref()
        .and_then(|path| app.state.projects.iter().find(|p| &p.path == path))
        .and_then(|p| {
            p.organization_name
                .clone()
                .or_else(|| p.organization_slug.clone())
        })
}

/// Draw a checkbox field
fn draw_checkbox_field(frame: &mut Frame, area: Rect, label: &str, checked: bool, focused: bool) {
    let checkbox = if checked { "[x]" } else { "[ ]" };
    let display = format!("{} {}", checkbox, label);

    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(display).style(style).block(block);
    frame.render_widget(paragraph, area);
}

/// Draw doc create form
pub fn draw_create(frame: &mut Frame, area: Rect, app: &App) {
    let has_org = has_organization(app);

    // Build constraints based on org availability
    let constraints: Vec<Constraint> = if has_org {
        vec![
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Content
            Constraint::Length(3), // Slug
            Constraint::Length(3), // Org-wide checkbox
            Constraint::Length(2), // Help text
        ]
    } else {
        vec![
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Content
            Constraint::Length(3), // Slug
            Constraint::Length(2), // Help text
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Doc ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field_with_value(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field_with_value(
        frame,
        chunks[1],
        "Content (Markdown)",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    draw_field_with_value(
        frame,
        chunks[2],
        "Slug (optional)",
        &app.state.form_slug,
        app.state.active_form_field == 2,
        false,
    );

    // Render checkbox if org exists, otherwise render help text in chunks[3]
    let help_chunk_idx = if has_org {
        // Draw org-wide checkbox
        let org_name = get_org_name(app).unwrap_or_else(|| "organization".to_string());
        let label = format!("Organization-wide (sync to all {} projects)", org_name);
        draw_checkbox_field(
            frame,
            chunks[3],
            &label,
            app.state.form_is_org_doc,
            app.state.active_form_field == 3,
        );
        4 // Help text is at index 4
    } else {
        3 // Help text is at index 3
    };

    // Build help text based on current field
    let help_spans = if has_org && app.state.active_form_field == 3 {
        // On checkbox field - show toggle hint
        vec![
            Span::styled("Space", Style::default().fg(Color::Cyan)),
            Span::raw(": toggle  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": next field  "),
            Span::styled(
                crate::platform::SAVE_SHORTCUT,
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(": save  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": cancel"),
        ]
    } else {
        // Standard help text
        vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": next field  "),
            Span::styled(
                crate::platform::SAVE_SHORTCUT,
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(": save  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": cancel"),
        ]
    };

    let help = Paragraph::new(Line::from(help_spans)).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[help_chunk_idx]);
}

/// Get the number of form fields based on org availability
pub fn get_field_count(app: &App) -> usize {
    if has_organization(app) {
        4 // Title, Content, Slug, Org checkbox
    } else {
        3 // Title, Content, Slug
    }
}
