//! Screen buffer for text selection and extraction

use super::ScreenPos;
use std::collections::HashMap;

/// Screen buffer that maps positions to characters
/// Rebuilt every frame during rendering
#[derive(Debug, Default)]
pub struct ScreenBuffer {
    /// Map of (col, row) -> character
    cells: HashMap<(u16, u16), char>,
    /// Terminal dimensions (for bounds checking)
    pub width: u16,
    pub height: u16,
}

impl ScreenBuffer {
    #[allow(dead_code)]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            cells: HashMap::new(),
            width,
            height,
        }
    }

    /// Clear the buffer (called at start of each frame)
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Update dimensions
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    /// Record a character at a position
    pub fn set(&mut self, col: u16, row: u16, ch: char) {
        if col < self.width && row < self.height {
            self.cells.insert((col, row), ch);
        }
    }

    /// Get character at a position
    pub fn get(&self, col: u16, row: u16) -> Option<char> {
        self.cells.get(&(col, row)).copied()
    }

    /// Extract text from a selection range
    pub fn extract_text(&self, start: ScreenPos, end: ScreenPos) -> String {
        let (start, end) = ScreenPos::normalize_range(start, end);

        let mut result = String::new();

        for row in start.row..=end.row {
            // Determine column range for this row
            let col_start = if row == start.row { start.col } else { 0 };
            let col_end = if row == end.row {
                end.col
            } else {
                self.width.saturating_sub(1)
            };

            let mut row_text = String::new();
            for col in col_start..=col_end {
                if let Some(ch) = self.get(col, row) {
                    row_text.push(ch);
                } else {
                    row_text.push(' ');
                }
            }

            // Trim trailing whitespace from each line
            let trimmed = row_text.trim_end();
            result.push_str(trimmed);

            // Add newline between rows (but not after last row)
            if row < end.row {
                result.push('\n');
            }
        }

        result
    }
}
