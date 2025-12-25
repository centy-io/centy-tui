//! Worktree exists dialog component

use crate::state::{PendingWorktreeAction, WorktreeDialogOption};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render a dialog for handling existing worktree
pub fn render_worktree_dialog(frame: &mut Frame, action: &PendingWorktreeAction) {
    let area = frame.area();

    // Dialog dimensions
    let dialog_width = 55u16;
    let dialog_height = 12u16;

    // Center the dialog
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Format workspace path for display (truncate if too long)
    let max_path_len = (dialog_width - 6) as usize;
    let workspace_path = if action.existing_workspace.workspace_path.len() > max_path_len {
        format!(
            "...{}",
            &action.existing_workspace.workspace_path
                [action.existing_workspace.workspace_path.len() - max_path_len + 3..]
        )
    } else {
        action.existing_workspace.workspace_path.clone()
    };

    // Build content
    let mut content = vec![
        Line::from(Span::styled(
            "Workspace Already Exists",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Issue: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("#{} ", action.existing_workspace.issue_display_number),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                truncate_string(&action.existing_workspace.issue_title, 35),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(workspace_path, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
    ];

    // Add options with selection highlighting
    let options = [
        WorktreeDialogOption::OpenExisting,
        WorktreeDialogOption::DeleteAndRecreate,
    ];

    for option in options {
        let is_selected = action.selected_option == option;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        content.push(Line::from(Span::styled(
            format!("{}{}", prefix, option.label()),
            style,
        )));
    }

    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::styled(" select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" confirm  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
    ]));

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::new().bg(Color::Black).fg(Color::White));

    frame.render_widget(dialog, dialog_area);
}

/// Truncate a string to a maximum length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
