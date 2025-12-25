//! Form components for creating and editing issues, PRs, and docs

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw a form field
fn draw_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_active: bool,
    is_multiline: bool,
) {
    let style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let display_value = if value.is_empty() && !is_active {
        "(empty)"
    } else {
        value
    };

    let cursor = if is_active { "▌" } else { "" };

    let content = if is_multiline {
        let lines: Vec<Line> = display_value
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
        let mut lines = lines;
        if is_active {
            if let Some(last) = lines.last_mut() {
                last.spans
                    .push(Span::styled(cursor, Style::default().fg(Color::Cyan)));
            } else {
                lines.push(Line::from(Span::styled(
                    cursor,
                    Style::default().fg(Color::Cyan),
                )));
            }
        }
        Paragraph::new(lines)
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(display_value, style),
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
        ]))
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style);

    frame.render_widget(content.wrap(Wrap { trim: false }).block(block), area);
}

/// Draw issue create form
pub fn draw_issue_create(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Description
            Constraint::Length(3), // Priority
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Issue ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field(
        frame,
        chunks[1],
        "Description",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    let priority_display = match app.state.form_priority {
        0 => "Default".to_string(),
        p => format!(
            "{} ({})",
            p,
            match p {
                1 => "high",
                2 => "medium",
                _ => "low",
            }
        ),
    };
    draw_field(
        frame,
        chunks[2],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next field  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);
}

/// Draw issue edit form
pub fn draw_issue_edit(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Description
            Constraint::Length(3), // Priority
            Constraint::Length(3), // Status
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let issue_number = app
        .state
        .selected_issue_id
        .as_ref()
        .and_then(|id| app.state.issues.iter().find(|i| &i.id == id))
        .map(|i| format!("#{}", i.display_number))
        .unwrap_or_default();

    let block = Block::default()
        .title(format!(" Edit Issue {} ", issue_number))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field(
        frame,
        chunks[1],
        "Description",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    let priority_display = match app.state.form_priority {
        0 => "Default".to_string(),
        p => format!(
            "{} ({})",
            p,
            match p {
                1 => "high",
                2 => "medium",
                _ => "low",
            }
        ),
    };
    draw_field(
        frame,
        chunks[2],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );

    draw_field(
        frame,
        chunks[3],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 3,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next field  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[4]);
}

/// Draw PR create form
pub fn draw_pr_create(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(4),    // Description
            Constraint::Length(3), // Source branch
            Constraint::Length(3), // Target branch
            Constraint::Length(3), // Priority
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Pull Request ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field(
        frame,
        chunks[1],
        "Description",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    draw_field(
        frame,
        chunks[2],
        "Source Branch",
        &app.state.form_source_branch,
        app.state.active_form_field == 2,
        false,
    );

    draw_field(
        frame,
        chunks[3],
        "Target Branch",
        if app.state.form_target_branch.is_empty() {
            "main"
        } else {
            &app.state.form_target_branch
        },
        app.state.active_form_field == 3,
        false,
    );

    let priority_display = match app.state.form_priority {
        0 => "Default".to_string(),
        p => format!("{}", p),
    };
    draw_field(
        frame,
        chunks[4],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[5]);
}

/// Draw PR edit form
pub fn draw_pr_edit(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(3),    // Description
            Constraint::Length(3), // Source branch
            Constraint::Length(3), // Target branch
            Constraint::Length(3), // Priority
            Constraint::Length(3), // Status
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let pr_number = app
        .state
        .selected_pr_id
        .as_ref()
        .and_then(|id| app.state.prs.iter().find(|p| &p.id == id))
        .map(|p| format!("#{}", p.display_number))
        .unwrap_or_default();

    let block = Block::default()
        .title(format!(" Edit PR {} ", pr_number))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field(
        frame,
        chunks[1],
        "Description",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    draw_field(
        frame,
        chunks[2],
        "Source Branch",
        &app.state.form_source_branch,
        app.state.active_form_field == 2,
        false,
    );

    draw_field(
        frame,
        chunks[3],
        "Target Branch",
        &app.state.form_target_branch,
        app.state.active_form_field == 3,
        false,
    );

    let priority_display = format!("{}", app.state.form_priority);
    draw_field(
        frame,
        chunks[4],
        "Priority",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );

    draw_field(
        frame,
        chunks[5],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 5,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[6]);
}

/// Draw doc create form
pub fn draw_doc_create(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Content
            Constraint::Length(3), // Slug
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    let block = Block::default()
        .title(" Create Doc ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    draw_field(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field(
        frame,
        chunks[1],
        "Content (Markdown)",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    draw_field(
        frame,
        chunks[2],
        "Slug (optional)",
        &app.state.form_slug,
        app.state.active_form_field == 2,
        false,
    );

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next field  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
        Span::raw(": save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);
}
