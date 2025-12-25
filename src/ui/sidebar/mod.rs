//! Sidebar module for navigation

mod draw;

pub use draw::draw_sidebar;

/// Sidebar navigation items
pub const SIDEBAR_ITEMS: &[&str] = &["Projects", "Issues", "PRs", "Docs", "Config"];
