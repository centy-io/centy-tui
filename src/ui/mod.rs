//! UI module for rendering the TUI

mod config_panel;
mod docs;
mod forms;
mod issues;
mod layout;
mod projects;
mod prs;
mod splash;
mod terminal;

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

    // Draw the main layout with sidebar
    let (sidebar_area, main_area) = layout::create_layout(area);

    // Draw sidebar
    layout::draw_sidebar(frame, sidebar_area, app);

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
        View::Terminal => terminal::draw(frame, main_area, app),
    }

    // Draw status bar
    layout::draw_status_bar(frame, app);
}
