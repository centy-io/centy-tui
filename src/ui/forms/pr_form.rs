//! Pull request form rendering

use super::field_renderer::{draw_field, draw_help_text};
use crate::app::App;
use crate::state::forms::{Form, FormState, PrCreateForm, PrEditForm};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

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

    // Try to use new FormState if available, otherwise fall back to legacy
    if let FormState::PrCreate(ref form) = app.state.form {
        draw_pr_create_form(frame, &chunks, form);
    } else {
        // Legacy fallback
        draw_pr_create_legacy(frame, &chunks, app);
    }

    draw_help_text(frame, chunks[5]);
}

/// Draw PR create form using new FormState
fn draw_pr_create_form(frame: &mut Frame, chunks: &[Rect], form: &PrCreateForm) {
    if let Some(field) = form.get_field(0) {
        draw_field(frame, chunks[0], field, form.active_field() == 0);
    }
    if let Some(field) = form.get_field(1) {
        draw_field(frame, chunks[1], field, form.active_field() == 1);
    }
    if let Some(field) = form.get_field(2) {
        draw_field(frame, chunks[2], field, form.active_field() == 2);
    }
    // Special handling for target branch - show "main" if empty
    if let Some(field) = form.get_field(3) {
        use super::field_renderer::draw_field_raw;
        let display_value = form.target_branch_or_default();
        draw_field_raw(
            frame,
            chunks[3],
            &field.label,
            display_value,
            form.active_field() == 3,
            false,
        );
    }
    if let Some(field) = form.get_field(4) {
        draw_field(frame, chunks[4], field, form.active_field() == 4);
    }
}

/// Legacy fallback for PR create form (during migration)
fn draw_pr_create_legacy(frame: &mut Frame, chunks: &[Rect], app: &App) {
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

    draw_field_raw(
        frame,
        chunks[2],
        "Source Branch",
        &app.state.form_source_branch,
        app.state.active_form_field == 2,
        false,
    );

    draw_field_raw(
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
    draw_field_raw(
        frame,
        chunks[4],
        "Priority (1-3)",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );
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

    // Try to use new FormState if available, otherwise fall back to legacy
    if let FormState::PrEdit(ref form) = app.state.form {
        draw_pr_edit_form(frame, &chunks, form);
    } else {
        // Legacy fallback
        draw_pr_edit_legacy(frame, &chunks, app);
    }

    draw_help_text(frame, chunks[6]);
}

/// Draw PR edit form using new FormState
fn draw_pr_edit_form(frame: &mut Frame, chunks: &[Rect], form: &PrEditForm) {
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
    if let Some(field) = form.get_field(4) {
        draw_field(frame, chunks[4], field, form.active_field() == 4);
    }
    if let Some(field) = form.get_field(5) {
        draw_field(frame, chunks[5], field, form.active_field() == 5);
    }
}

/// Legacy fallback for PR edit form (during migration)
fn draw_pr_edit_legacy(frame: &mut Frame, chunks: &[Rect], app: &App) {
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

    draw_field_raw(
        frame,
        chunks[2],
        "Source Branch",
        &app.state.form_source_branch,
        app.state.active_form_field == 2,
        false,
    );

    draw_field_raw(
        frame,
        chunks[3],
        "Target Branch",
        &app.state.form_target_branch,
        app.state.active_form_field == 3,
        false,
    );

    let priority_display = format!("{}", app.state.form_priority);
    draw_field_raw(
        frame,
        chunks[4],
        "Priority",
        &priority_display,
        app.state.active_form_field == 4,
        false,
    );

    draw_field_raw(
        frame,
        chunks[5],
        "Status",
        &app.state.form_status,
        app.state.active_form_field == 5,
        false,
    );
}
