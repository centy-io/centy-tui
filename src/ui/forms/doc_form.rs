//! Document form rendering

use super::field_renderer::{draw_field, draw_help_text};
use crate::app::App;
use crate::state::forms::{DocCreateForm, Form, FormState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

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

    // Try to use new FormState if available, otherwise fall back to legacy
    if let FormState::DocCreate(ref form) = app.state.form {
        draw_doc_create_form(frame, &chunks, form);
    } else {
        // Legacy fallback
        draw_doc_create_legacy(frame, &chunks, app);
    }

    draw_help_text(frame, chunks[3]);
}

/// Draw doc create form using new FormState
fn draw_doc_create_form(frame: &mut Frame, chunks: &[Rect], form: &DocCreateForm) {
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

/// Legacy fallback for doc create form (during migration)
fn draw_doc_create_legacy(frame: &mut Frame, chunks: &[Rect], app: &App) {
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
        "Content (Markdown)",
        &app.state.form_description,
        app.state.active_form_field == 1,
        true,
    );

    draw_field_raw(
        frame,
        chunks[2],
        "Slug (optional)",
        &app.state.form_slug,
        app.state.active_form_field == 2,
        false,
    );
}
