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
