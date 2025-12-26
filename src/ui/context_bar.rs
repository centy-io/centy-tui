//! Context bar (breadcrumb navigation) component

use crate::app::App;
use crate::state::View;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw the context bar (breadcrumb navigation) at the top of the screen
pub fn draw_context_bar(frame: &mut Frame, area: Rect, app: &mut App) {
    let separator = Span::styled(" / ", Style::default().fg(Color::DarkGray));

    // Build breadcrumbs based on current view
    let breadcrumbs = build_breadcrumbs(app);

    // Calculate total content width for centering
    let mut total_width: usize = 0;
    for (i, (label, _)) in breadcrumbs.iter().enumerate() {
        if i > 0 {
            total_width += 3; // " / " separator
        }
        let display_label = truncate_label(label, 40);
        total_width += display_label.len();
    }

    // Calculate padding for centered content (accounting for block border)
    let inner_width = area.width.saturating_sub(2) as usize; // -2 for borders
    let left_padding = inner_width.saturating_sub(total_width) / 2;

    // Build spans with manual left padding for centering
    let mut spans = Vec::new();

    // Add left padding as spaces
    if left_padding > 0 {
        spans.push(Span::raw(" ".repeat(left_padding)));
    }

    // Starting column is: area.x + 1 (left border) + padding
    let mut current_col: u16 = area.x + 1 + left_padding as u16;
    let mut segments = Vec::new();

    for (i, (label, target_view)) in breadcrumbs.iter().enumerate() {
        // Add separator between segments (after first)
        if i > 0 {
            spans.push(separator.clone());
            current_col += 3; // " / " is 3 chars
        }

        let start_col = current_col;

        // Determine style: last segment is current (bold), others are clickable (cyan)
        let is_last = i == breadcrumbs.len() - 1;
        let style = if is_last {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let display_label = truncate_label(label, 40);
        let label_len = display_label.len() as u16;
        spans.push(Span::styled(display_label, style));
        current_col += label_len;

        // Store segment bounds for click detection (only clickable segments)
        if !is_last {
            segments.push((start_col, current_col, target_view.clone()));
        }
    }

    // Store segments for mouse handling
    app.state.context_bar_segments = segments;

    // Create block with borders
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    // Render paragraph inside block (no Alignment::Center - we handle centering manually)
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);
}

/// Truncate a label if it exceeds max length
fn truncate_label(label: &str, max_len: usize) -> String {
    if label.len() > max_len {
        format!("{}...", &label[..max_len - 3])
    } else {
        label.to_string()
    }
}

/// Build breadcrumb segments based on current view and state
fn build_breadcrumbs(app: &App) -> Vec<(String, View)> {
    let mut breadcrumbs = Vec::new();

    // Get project info if selected
    let project_info = app
        .state
        .selected_project_path
        .as_ref()
        .and_then(|path| app.state.projects.iter().find(|p| &p.path == path));

    match &app.state.current_view {
        View::Splash => {
            // No breadcrumbs for splash
        }
        View::Projects => {
            breadcrumbs.push(("Projects".to_string(), View::Projects));
        }
        View::Organization => {
            breadcrumbs.push(("Projects".to_string(), View::Projects));
            if let Some(org) = &app.state.current_organization {
                breadcrumbs.push((org.name.clone(), View::Organization));
            }
        }
        View::Issues => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Issues".to_string(), View::Issues));
        }
        View::IssueDetail => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Issues".to_string(), View::Issues));

            // Add issue title
            if let Some(issue) = app
                .state
                .selected_issue_id
                .as_ref()
                .and_then(|id| app.state.issues.iter().find(|i| &i.id == id))
            {
                let label = format!("#{} {}", issue.display_number, issue.title);
                breadcrumbs.push((label, View::IssueDetail));
            }
        }
        View::IssueCreate => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Issues".to_string(), View::Issues));
            breadcrumbs.push(("New Issue".to_string(), View::IssueCreate));
        }
        View::IssueEdit => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Issues".to_string(), View::Issues));

            // Add issue title
            if let Some(issue) = app
                .state
                .selected_issue_id
                .as_ref()
                .and_then(|id| app.state.issues.iter().find(|i| &i.id == id))
            {
                let label = format!("#{} {}", issue.display_number, issue.title);
                breadcrumbs.push((label, View::IssueDetail));
            }
            breadcrumbs.push(("Edit".to_string(), View::IssueEdit));
        }
        View::Prs => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("PRs".to_string(), View::Prs));
        }
        View::PrDetail => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("PRs".to_string(), View::Prs));

            // Add PR title
            if let Some(pr) = app
                .state
                .selected_pr_id
                .as_ref()
                .and_then(|id| app.state.prs.iter().find(|p| &p.id == id))
            {
                let label = format!("#{} {}", pr.display_number, pr.title);
                breadcrumbs.push((label, View::PrDetail));
            }
        }
        View::PrCreate => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("PRs".to_string(), View::Prs));
            breadcrumbs.push(("New PR".to_string(), View::PrCreate));
        }
        View::PrEdit => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("PRs".to_string(), View::Prs));

            // Add PR title
            if let Some(pr) = app
                .state
                .selected_pr_id
                .as_ref()
                .and_then(|id| app.state.prs.iter().find(|p| &p.id == id))
            {
                let label = format!("#{} {}", pr.display_number, pr.title);
                breadcrumbs.push((label, View::PrDetail));
            }
            breadcrumbs.push(("Edit".to_string(), View::PrEdit));
        }
        View::Docs => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Docs".to_string(), View::Docs));
        }
        View::DocDetail => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Docs".to_string(), View::Docs));

            // Add doc title
            if let Some(doc) = app
                .state
                .selected_doc_slug
                .as_ref()
                .and_then(|slug| app.state.docs.iter().find(|d| &d.slug == slug))
            {
                breadcrumbs.push((doc.title.clone(), View::DocDetail));
            }
        }
        View::DocCreate => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Docs".to_string(), View::Docs));
            breadcrumbs.push(("New Doc".to_string(), View::DocCreate));
        }
        View::DocEdit => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Docs".to_string(), View::Docs));

            // Add doc title
            if let Some(doc) = app
                .state
                .selected_doc_slug
                .as_ref()
                .and_then(|slug| app.state.docs.iter().find(|d| &d.slug == slug))
            {
                breadcrumbs.push((doc.title.clone(), View::DocDetail));
            }
            breadcrumbs.push(("Edit".to_string(), View::DocEdit));
        }
        View::People => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("People".to_string(), View::People));
        }
        View::PersonDetail => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("People".to_string(), View::People));

            // Add person name
            if let Some(person) = app.state.selected_person_id.as_ref().and_then(|id| {
                app.state
                    .sorted_people()
                    .into_iter()
                    .find(|u| &u.id == id)
                    .cloned()
            }) {
                breadcrumbs.push((person.name, View::PersonDetail));
            }
        }
        View::Config => {
            if let Some(project) = project_info {
                add_org_project_breadcrumbs(&mut breadcrumbs, project);
            }
            breadcrumbs.push(("Config".to_string(), View::Config));
        }
    }

    breadcrumbs
}

/// Add organization and project breadcrumbs
fn add_org_project_breadcrumbs(
    breadcrumbs: &mut Vec<(String, View)>,
    project: &crate::state::Project,
) {
    // Add organization if available - clicking navigates to Organization view
    if let Some(org_name) = &project.organization_name {
        breadcrumbs.push((org_name.clone(), View::Organization));
    }

    // Add project name - clicking returns to Projects view
    let project_name = project
        .project_title
        .clone()
        .unwrap_or_else(|| project.name.clone());
    breadcrumbs.push((project_name, View::Projects));
}
