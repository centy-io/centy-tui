//! Text selection state for TUI

/// Represents a screen position (column, row)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScreenPos {
    pub col: u16,
    pub row: u16,
}

impl ScreenPos {
    pub fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }

    /// Order positions for selection (top-left to bottom-right)
    pub fn normalize_range(start: Self, end: Self) -> (Self, Self) {
        if start.row < end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        }
    }
}

/// Current selection state
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Start position of selection (anchor point)
    pub anchor: Option<ScreenPos>,
    /// Current end position (cursor during selection)
    pub cursor: Option<ScreenPos>,
    /// Whether selection is actively being made (mouse dragging)
    pub is_selecting: bool,
    /// Whether keyboard selection mode is active
    pub keyboard_mode: bool,
    /// Keyboard cursor position (for Shift+arrow navigation)
    pub keyboard_cursor: Option<ScreenPos>,
}

impl SelectionState {
    /// Get normalized selection range (start always before end)
    pub fn get_range(&self) -> Option<(ScreenPos, ScreenPos)> {
        match (self.anchor, self.cursor) {
            (Some(anchor), Some(cursor)) => Some(ScreenPos::normalize_range(anchor, cursor)),
            _ => None,
        }
    }

    /// Check if a position is within the selection
    pub fn contains(&self, pos: ScreenPos) -> bool {
        if let Some((start, end)) = self.get_range() {
            if pos.row < start.row || pos.row > end.row {
                return false;
            }
            if pos.row == start.row && pos.col < start.col {
                return false;
            }
            if pos.row == end.row && pos.col > end.col {
                return false;
            }
            true
        } else {
            false
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.anchor = None;
        self.cursor = None;
        self.is_selecting = false;
        self.keyboard_mode = false;
        self.keyboard_cursor = None;
    }

    /// Start a new selection at the given position
    pub fn start(&mut self, pos: ScreenPos) {
        self.anchor = Some(pos);
        self.cursor = Some(pos);
        self.is_selecting = true;
    }

    /// Update the selection endpoint
    pub fn update(&mut self, pos: ScreenPos) {
        self.cursor = Some(pos);
    }

    /// Finish the selection (e.g., mouse up)
    pub fn finish(&mut self) {
        self.is_selecting = false;
    }

    /// Check if there is an active selection
    pub fn has_selection(&self) -> bool {
        if let Some((start, end)) = self.get_range() {
            // Only consider it a selection if start != end
            start.row != end.row || start.col != end.col
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod screen_pos {
        use super::*;

        #[test]
        fn test_new_creates_correct_position() {
            let pos = ScreenPos::new(5, 10);
            assert_eq!(pos.col, 5);
            assert_eq!(pos.row, 10);
        }

        #[test]
        fn test_default_is_zero() {
            let pos = ScreenPos::default();
            assert_eq!(pos.col, 0);
            assert_eq!(pos.row, 0);
        }

        #[test]
        fn test_normalize_range_already_ordered() {
            let start = ScreenPos::new(5, 10);
            let end = ScreenPos::new(15, 20);
            let (s, e) = ScreenPos::normalize_range(start, end);
            assert_eq!(s, start);
            assert_eq!(e, end);
        }

        #[test]
        fn test_normalize_range_reversed_rows() {
            let start = ScreenPos::new(5, 20);
            let end = ScreenPos::new(15, 10);
            let (s, e) = ScreenPos::normalize_range(start, end);
            assert_eq!(s, end);
            assert_eq!(e, start);
        }

        #[test]
        fn test_normalize_range_same_row_ordered() {
            let start = ScreenPos::new(5, 10);
            let end = ScreenPos::new(15, 10);
            let (s, e) = ScreenPos::normalize_range(start, end);
            assert_eq!(s, start);
            assert_eq!(e, end);
        }

        #[test]
        fn test_normalize_range_same_row_reversed() {
            let start = ScreenPos::new(15, 10);
            let end = ScreenPos::new(5, 10);
            let (s, e) = ScreenPos::normalize_range(start, end);
            assert_eq!(s, end);
            assert_eq!(e, start);
        }

        #[test]
        fn test_normalize_range_same_position() {
            let pos = ScreenPos::new(5, 10);
            let (s, e) = ScreenPos::normalize_range(pos, pos);
            assert_eq!(s, pos);
            assert_eq!(e, pos);
        }
    }

    mod selection_state {
        use super::*;

        #[test]
        fn test_default_is_empty() {
            let state = SelectionState::default();
            assert!(state.anchor.is_none());
            assert!(state.cursor.is_none());
            assert!(!state.is_selecting);
            assert!(!state.keyboard_mode);
            assert!(state.keyboard_cursor.is_none());
        }

        #[test]
        fn test_start_sets_anchor_and_cursor() {
            let mut state = SelectionState::default();
            let pos = ScreenPos::new(5, 10);
            state.start(pos);

            assert_eq!(state.anchor, Some(pos));
            assert_eq!(state.cursor, Some(pos));
            assert!(state.is_selecting);
        }

        #[test]
        fn test_update_changes_cursor_only() {
            let mut state = SelectionState::default();
            let start = ScreenPos::new(5, 10);
            let end = ScreenPos::new(15, 20);

            state.start(start);
            state.update(end);

            assert_eq!(state.anchor, Some(start));
            assert_eq!(state.cursor, Some(end));
        }

        #[test]
        fn test_finish_stops_selecting() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 10));
            assert!(state.is_selecting);

            state.finish();
            assert!(!state.is_selecting);
        }

        #[test]
        fn test_clear_resets_all_fields() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 10));
            state.update(ScreenPos::new(15, 20));
            state.keyboard_mode = true;
            state.keyboard_cursor = Some(ScreenPos::new(10, 15));

            state.clear();

            assert!(state.anchor.is_none());
            assert!(state.cursor.is_none());
            assert!(!state.is_selecting);
            assert!(!state.keyboard_mode);
            assert!(state.keyboard_cursor.is_none());
        }

        #[test]
        fn test_get_range_with_valid_selection() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 10));
            state.update(ScreenPos::new(15, 20));

            let range = state.get_range();
            assert!(range.is_some());
            let (start, end) = range.unwrap();
            assert_eq!(start, ScreenPos::new(5, 10));
            assert_eq!(end, ScreenPos::new(15, 20));
        }

        #[test]
        fn test_get_range_normalizes_reversed_selection() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(15, 20));
            state.update(ScreenPos::new(5, 10));

            let range = state.get_range();
            assert!(range.is_some());
            let (start, end) = range.unwrap();
            assert_eq!(start, ScreenPos::new(5, 10));
            assert_eq!(end, ScreenPos::new(15, 20));
        }

        #[test]
        fn test_get_range_returns_none_without_anchor() {
            let state = SelectionState::default();
            assert!(state.get_range().is_none());
        }

        #[test]
        fn test_get_range_returns_none_with_only_anchor() {
            let state = SelectionState {
                anchor: Some(ScreenPos::new(5, 10)),
                ..Default::default()
            };
            assert!(state.get_range().is_none());
        }

        #[test]
        fn test_contains_inside_selection() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            assert!(state.contains(ScreenPos::new(10, 10)));
        }

        #[test]
        fn test_contains_on_start_boundary() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            assert!(state.contains(ScreenPos::new(5, 5)));
        }

        #[test]
        fn test_contains_on_end_boundary() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            assert!(state.contains(ScreenPos::new(15, 15)));
        }

        #[test]
        fn test_contains_on_start_row() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            // Should contain positions on start row after start col
            assert!(state.contains(ScreenPos::new(10, 5)));
            // Should not contain positions on start row before start col
            assert!(!state.contains(ScreenPos::new(3, 5)));
        }

        #[test]
        fn test_contains_on_end_row() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            // Should contain positions on end row before end col
            assert!(state.contains(ScreenPos::new(10, 15)));
            // Should not contain positions on end row after end col
            assert!(!state.contains(ScreenPos::new(20, 15)));
        }

        #[test]
        fn test_contains_outside_above() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            assert!(!state.contains(ScreenPos::new(10, 3)));
        }

        #[test]
        fn test_contains_outside_below() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(15, 15));

            assert!(!state.contains(ScreenPos::new(10, 20)));
        }

        #[test]
        fn test_contains_false_without_selection() {
            let state = SelectionState::default();
            assert!(!state.contains(ScreenPos::new(5, 5)));
        }

        #[test]
        fn test_has_selection_true_when_different_positions() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(10, 5));

            assert!(state.has_selection());
        }

        #[test]
        fn test_has_selection_false_when_same_position() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            // Don't update - cursor and anchor are the same

            assert!(!state.has_selection());
        }

        #[test]
        fn test_has_selection_false_without_selection() {
            let state = SelectionState::default();
            assert!(!state.has_selection());
        }

        #[test]
        fn test_has_selection_true_with_different_rows() {
            let mut state = SelectionState::default();
            state.start(ScreenPos::new(5, 5));
            state.update(ScreenPos::new(5, 10));

            assert!(state.has_selection());
        }
    }
}
