//! UI module for rendering the TUI

mod config_panel;
mod docs;
mod forms;
mod issues;
mod layout;
mod projects;
mod prs;
mod splash;
mod widgets;

pub use widgets::render_scrollable_list;

use crate::app::App;
use crate::state::View;
use ratatui::Frame;

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Splash screen takes full screen (no sidebar or status bar)
    if let View::Splash = &app.state.current_view {
        if let Some(ref splash_state) = app.splash_state {
            splash::draw(frame, area, splash_state);
        }
        return;
    }

    // Determine if we should show sidebar (only when a project is selected)
    let has_project = app.state.selected_project_path.is_some();

    let main_area = if has_project {
        // Draw the main layout with sidebar
        let (sidebar_area, main_area) = layout::create_layout(area);
        layout::draw_sidebar(frame, sidebar_area, app);
        main_area
    } else {
        // Full-width layout without sidebar
        layout::create_layout_no_sidebar(area)
    };

    // Draw main content based on current view
    match &app.state.current_view {
        View::Splash => {} // Handled above
        View::Projects => projects::draw(frame, main_area, app),
        View::Issues => issues::draw_list(frame, main_area, app),
        View::IssueDetail => issues::draw_detail(frame, main_area, app),
        View::IssueCreate => forms::draw_issue_create(frame, main_area, app),
        View::IssueEdit => forms::draw_issue_edit(frame, main_area, app),
        View::Prs => prs::draw_list(frame, main_area, app),
        View::PrDetail => prs::draw_detail(frame, main_area, app),
        View::PrCreate => forms::draw_pr_create(frame, main_area, app),
        View::PrEdit => forms::draw_pr_edit(frame, main_area, app),
        View::Docs => docs::draw_list(frame, main_area, app),
        View::DocDetail => docs::draw_detail(frame, main_area, app),
        View::DocCreate => forms::draw_doc_create(frame, main_area, app),
        View::Config => config_panel::draw(frame, main_area, app),
    }

    // Draw status bar
    layout::draw_status_bar(frame, app);
}
