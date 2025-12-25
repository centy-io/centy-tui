//! Issue form rendering

use super::field_renderer::{draw_field, draw_help_text};
use crate::app::App;
use crate::state::forms::{Form, FormState, IssueCreateForm, IssueEditForm};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

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

    // Try to use new FormState if available, otherwise fall back to legacy
    if let FormState::IssueCreate(ref form) = app.state.form {
        draw_issue_create_form(frame, &chunks, form);
    } else {
        // Legacy fallback
        draw_issue_create_legacy(frame, &chunks, app);
    }

    draw_help_text(frame, chunks[3]);
}

/// Draw issue create form using new FormState
fn draw_issue_create_form(frame: &mut Frame, chunks: &[Rect], form: &IssueCreateForm) {
    if let Some(field) = form.get_field(0) {
        draw_field(frame, chunks[0], field, form.active_field() == 0);
    }
    if let Some(field) = form.get_field(1) {
        draw_field(frame, chunks[1], field, form.active_field() == 1);
    }
    if let Some(field) = form.get_field(2) {
        draw_field(frame, chunks[2], field, form.active_field() == 2);
    }
}

/// Legacy fallback for issue create form (during migration)
fn draw_issue_create_legacy(frame: &mut Frame, chunks: &[Rect], app: &App) {
    use super::field_renderer::draw_field_raw;

    draw_field_raw(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field_raw(
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
    draw_field_raw(
        frame,
        chunks[2],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );
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

    // Try to use new FormState if available, otherwise fall back to legacy
    if let FormState::IssueEdit(ref form) = app.state.form {
        draw_issue_edit_form(frame, &chunks, form);
    } else {
        // Legacy fallback
        draw_issue_edit_legacy(frame, &chunks, app);
    }

    draw_help_text(frame, chunks[4]);
}

/// Draw issue edit form using new FormState
fn draw_issue_edit_form(frame: &mut Frame, chunks: &[Rect], form: &IssueEditForm) {
    if let Some(field) = form.get_field(0) {
        draw_field(frame, chunks[0], field, form.active_field() == 0);
    }
    if let Some(field) = form.get_field(1) {
        draw_field(frame, chunks[1], field, form.active_field() == 1);
    }
    if let Some(field) = form.get_field(2) {
        draw_field(frame, chunks[2], field, form.active_field() == 2);
    }
    if let Some(field) = form.get_field(3) {
        draw_field(frame, chunks[3], field, form.active_field() == 3);
    }
}

/// Legacy fallback for issue edit form (during migration)
fn draw_issue_edit_legacy(frame: &mut Frame, chunks: &[Rect], app: &App) {
    use super::field_renderer::draw_field_raw;

    draw_field_raw(
        frame,
        chunks[0],
        "Title",
        &app.state.form_title,
        app.state.active_form_field == 0,
        false,
    );

    draw_field_raw(
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
    draw_field_raw(
        frame,
        chunks[2],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 2,
        false,
    );

    draw_field_raw(
        frame,
        chunks[3],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 3,
        false,
    );
}
