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
        View::Organization => vec![LocalAction::new("nav_projects", "All Projects", "p")],
        View::Issues => vec![
            LocalAction::new("new_issue", "New Issue", "n"),
            LocalAction::new("toggle_scope", "Toggle Scope", "o"),
            LocalAction::new("nav_prs", "Pull Requests", "3"),
            LocalAction::new("nav_docs", "Docs", "4"),
            LocalAction::new("nav_people", "People", "5"),
        ],
        View::IssueDetail => vec![LocalAction::new("edit_issue", "Edit Issue", "e")],
        View::Prs => vec![
            LocalAction::new("new_pr", "New PR", "n"),
            LocalAction::new("toggle_scope", "Toggle Scope", "o"),
            LocalAction::new("nav_issues", "Issues", "2"),
            LocalAction::new("nav_docs", "Docs", "4"),
            LocalAction::new("nav_people", "People", "5"),
        ],
        View::PrDetail => vec![LocalAction::new("edit_pr", "Edit PR", "e")],
        View::Docs => vec![
            LocalAction::new("new_doc", "New Doc", "n"),
            LocalAction::new("toggle_scope", "Toggle Scope", "o"),
            LocalAction::new("nav_issues", "Issues", "2"),
            LocalAction::new("nav_prs", "Pull Requests", "3"),
            LocalAction::new("nav_people", "People", "5"),
        ],
        View::DocDetail => vec![LocalAction::new("edit_doc", "Edit Doc", "e")],
        View::GlobalSearch => vec![LocalAction::new("cycle_filter", "Cycle Filter", "^F")],
        View::People => vec![
            LocalAction::new("nav_issues", "Issues", "2"),
            LocalAction::new("nav_prs", "Pull Requests", "3"),
            LocalAction::new("nav_docs", "Docs", "4"),
        ],
        View::PersonDetail => vec![],
        // Views with no local actions
        View::Splash | View::Projects | View::Config => vec![],
        // Form views - no sidebar
        View::IssueCreate
        | View::IssueEdit
        | View::PrCreate
        | View::PrEdit
        | View::DocCreate
        | View::DocEdit
        | View::InitProject => vec![],
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
            | View::InitProject
    )
}
