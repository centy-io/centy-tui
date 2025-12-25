//! Dialog components for TUI

mod base;
mod error_dialog;
mod worktree_dialog;

pub use error_dialog::render_error_dialog;
pub use worktree_dialog::render_worktree_dialog;
