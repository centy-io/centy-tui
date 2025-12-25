//! UI area positioning for mouse event handling
//!
//! This module provides a centralized way to calculate vertical offsets
//! for different UI areas, accounting for the context bar at the top of the screen.

/// Height of the context bar (with borders: top + content + bottom)
pub const CONTEXT_BAR_HEIGHT: u16 = 3;

/// UI area for calculating mouse Y offset
///
/// Different UI areas start at different vertical positions due to the context bar,
/// borders, and headers. This enum provides a centralized way to get the correct
/// starting Y position for each area.
///
/// # Layout
///
/// ```text
/// Row 0-2:  Context Bar (3 rows with borders)
/// Row 3+:   Main content area
///           - Sidebar on left (20 cols)
///           - Main content on right
/// Bottom:   Status bar (1 row)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiArea {
    /// Context bar itself (rows 0-2)
    ContextBar,
    /// Sidebar navigation (starts after context bar)
    Sidebar,
    /// Action panel in list/detail views (starts after context bar + border)
    ActionPanel,
    /// List items in Issues/PRs/Docs views (starts after context bar + header area)
    ListContent,
    /// Project grid cards (starts after context bar + outer border + inner start)
    GridContent,
    /// Form fields (starts after context bar + border)
    FormContent,
}

impl UiArea {
    /// Get the starting Y row for this UI area (absolute screen position)
    #[inline]
    pub const fn start_y(self) -> u16 {
        match self {
            UiArea::ContextBar => 0,
            UiArea::Sidebar => CONTEXT_BAR_HEIGHT,
            UiArea::ActionPanel => CONTEXT_BAR_HEIGHT + 1, // +1 for outer border
            UiArea::ListContent => CONTEXT_BAR_HEIGHT + 3, // +3 for header area
            UiArea::GridContent => CONTEXT_BAR_HEIGHT + 2, // +2 for outer border + inner start
            UiArea::FormContent => CONTEXT_BAR_HEIGHT + 1, // +1 for border
        }
    }

    /// Convert absolute mouse row to row relative to this UI area
    #[inline]
    pub fn relative_row(self, mouse_row: u16) -> u16 {
        mouse_row.saturating_sub(self.start_y())
    }

    /// Check if a mouse row is within this UI area (at or after start)
    #[inline]
    pub fn contains_row(self, mouse_row: u16) -> bool {
        mouse_row >= self.start_y()
    }

    /// Get the height adjustment for areas that need to account for context bar
    /// Used for calculating available height (e.g., sidebar height)
    #[inline]
    pub const fn height_adjustment() -> u16 {
        CONTEXT_BAR_HEIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_y_values() {
        assert_eq!(UiArea::ContextBar.start_y(), 0);
        assert_eq!(UiArea::Sidebar.start_y(), 3);
        assert_eq!(UiArea::ActionPanel.start_y(), 4);
        assert_eq!(UiArea::ListContent.start_y(), 6);
        assert_eq!(UiArea::GridContent.start_y(), 5);
        assert_eq!(UiArea::FormContent.start_y(), 4);
    }

    #[test]
    fn test_relative_row() {
        assert_eq!(UiArea::Sidebar.relative_row(3), 0);
        assert_eq!(UiArea::Sidebar.relative_row(5), 2);
        assert_eq!(UiArea::ListContent.relative_row(6), 0);
        assert_eq!(UiArea::ListContent.relative_row(10), 4);
    }

    #[test]
    fn test_relative_row_saturates() {
        // Should not underflow
        assert_eq!(UiArea::Sidebar.relative_row(0), 0);
        assert_eq!(UiArea::ListContent.relative_row(3), 0);
    }

    #[test]
    fn test_contains_row() {
        assert!(UiArea::ContextBar.contains_row(0));
        assert!(UiArea::ContextBar.contains_row(2));

        assert!(!UiArea::Sidebar.contains_row(2));
        assert!(UiArea::Sidebar.contains_row(3));
        assert!(UiArea::Sidebar.contains_row(10));

        assert!(!UiArea::ListContent.contains_row(5));
        assert!(UiArea::ListContent.contains_row(6));
    }

    #[test]
    fn test_height_adjustment() {
        assert_eq!(UiArea::height_adjustment(), CONTEXT_BAR_HEIGHT);
        assert_eq!(UiArea::height_adjustment(), 3);
    }
}
