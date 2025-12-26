//! Global search view for cross-project searching

use crate::app::App;
use crate::state::{GlobalSearchFilter, GlobalSearchFocus, GlobalSearchResult};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Draw the global search view
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    // Split into: search input (top), filter bar, results list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Length(1), // Filter bar
            Constraint::Min(0),    // Results
        ])
        .split(area);

    draw_search_input(frame, chunks[0], app);
    draw_filter_bar(frame, chunks[1], app);
    draw_results(frame, chunks[2], app);
}

fn draw_search_input(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(
        app.state.global_search_focus,
        GlobalSearchFocus::SearchInput
    );
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let input_text = if app.state.global_search_query.is_empty() {
        Span::styled(
            "Type to search across all projects...",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::styled(
            app.state.global_search_query.as_str(),
            Style::default().fg(Color::White),
        )
    };

    let input = Paragraph::new(Line::from(input_text)).block(
        Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(input, area);

    // Show cursor if focused
    if is_focused {
        frame.set_cursor_position((
            area.x + 1 + app.state.global_search_query.len() as u16,
            area.y + 1,
        ));
    }
}

fn draw_filter_bar(frame: &mut Frame, area: Rect, app: &App) {
    let filters = [
        GlobalSearchFilter::All,
        GlobalSearchFilter::Issues,
        GlobalSearchFilter::Prs,
        GlobalSearchFilter::Docs,
    ];

    let spans: Vec<Span> = filters
        .iter()
        .map(|f| {
            let is_selected = *f == app.state.global_search_filter;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(format!(" [{}] ", f.label()), style)
        })
        .collect();

    let mut all_spans = vec![Span::styled(
        "Filter: ",
        Style::default().fg(Color::DarkGray),
    )];
    all_spans.extend(spans);
    all_spans.push(Span::styled(
        " (Ctrl+F to cycle)",
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(all_spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn draw_results(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.state.global_search_focus, GlobalSearchFocus::Results);
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    if app.state.global_search_loading {
        let loading = Paragraph::new("Searching...")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
        frame.render_widget(loading, area);
        return;
    }

    if app.state.global_search_results.is_empty() {
        let message = if app.state.global_search_query.is_empty() {
            "Enter a search term to find issues, PRs, and docs across all projects."
        } else {
            "No results found."
        };
        let empty = Paragraph::new(message)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .state
        .global_search_results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let is_selected = idx == app.state.global_search_selected;

            let prefix = if is_selected { "▸" } else { " " };

            let (type_label, type_color, title) = match result {
                GlobalSearchResult::Issue { issue, .. } => (
                    format!("#{}", issue.display_number),
                    Color::Yellow,
                    issue.title.clone(),
                ),
                GlobalSearchResult::Pr { pr, .. } => (
                    format!("PR#{}", pr.display_number),
                    Color::Magenta,
                    pr.title.clone(),
                ),
                GlobalSearchResult::Doc { doc, .. } => {
                    (doc.slug.clone(), Color::Green, doc.title.clone())
                }
            };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            // Truncate title if too long
            let max_title_len = 40;
            let truncated_title = if title.len() > max_title_len {
                format!("{}...", &title[..max_title_len - 3])
            } else {
                title
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("[{}]", type_label), Style::default().fg(type_color)),
                Span::styled(format!(" {} ", truncated_title), style),
                Span::styled(
                    format!("({})", result.project_name()),
                    Style::default().fg(Color::Blue),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let count = app.state.global_search_results.len();
    let list = List::new(items).block(
        Block::default()
            .title(format!(" Results ({}) ", count))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(list, area);
}
