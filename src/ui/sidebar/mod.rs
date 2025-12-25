//! Sidebar module for local actions

mod draw;

pub use draw::draw_sidebar;

use crate::state::View;

/// A local action that can be shown in the left sidebar
#[derive(Debug, Clone)]
pub struct LocalAction {
    pub id: &'static str,
    pub label: &'static str,
    pub keyboard_shortcut: &'static str,
}

impl LocalAction {
    pub const fn new(id: &'static str, label: &'static str, shortcut: &'static str) -> Self {
        Self {
            id,
            label,
            keyboard_shortcut: shortcut,
        }
    }
}

/// Get local actions for a given view
pub fn get_local_actions(view: &View) -> Vec<LocalAction> {
    match view {
        View::Issues => vec![LocalAction::new("new_issue", "New Issue", "n")],
        View::IssueDetail => vec![LocalAction::new("edit_issue", "Edit Issue", "e")],
        View::Prs => vec![LocalAction::new("new_pr", "New PR", "n")],
        View::PrDetail => vec![LocalAction::new("edit_pr", "Edit PR", "e")],
        View::Docs => vec![LocalAction::new("new_doc", "New Doc", "n")],
        View::DocDetail => vec![LocalAction::new("edit_doc", "Edit Doc", "e")],
        // Views with no local actions
        View::Splash | View::Projects | View::Config => vec![],
        // Form views - no sidebar
        View::IssueCreate
        | View::IssueEdit
        | View::PrCreate
        | View::PrEdit
        | View::DocCreate
        | View::DocEdit => vec![],
    }
}

/// Check if a view should show the left sidebar
pub fn should_show_sidebar(view: &View) -> bool {
    !matches!(
        view,
        View::Splash
            | View::Projects
            | View::IssueCreate
            | View::IssueEdit
            | View::PrCreate
            | View::PrEdit
            | View::DocCreate
            | View::DocEdit
    )
}
