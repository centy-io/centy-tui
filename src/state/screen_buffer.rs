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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_with_dimensions() {
        let buffer = ScreenBuffer::new(80, 24);
        assert_eq!(buffer.width, 80);
        assert_eq!(buffer.height, 24);
        assert!(buffer.cells.is_empty());
    }

    #[test]
    fn test_default_is_empty() {
        let buffer = ScreenBuffer::default();
        assert_eq!(buffer.width, 0);
        assert_eq!(buffer.height, 0);
        assert!(buffer.cells.is_empty());
    }

    #[test]
    fn test_clear_empties_cells() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(5, 5, 'A');
        buffer.set(10, 10, 'B');
        assert_eq!(buffer.cells.len(), 2);

        buffer.clear();
        assert!(buffer.cells.is_empty());
    }

    #[test]
    fn test_resize_updates_dimensions() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.resize(120, 40);
        assert_eq!(buffer.width, 120);
        assert_eq!(buffer.height, 40);
    }

    #[test]
    fn test_set_within_bounds() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(5, 10, 'X');
        assert_eq!(buffer.get(5, 10), Some('X'));
    }

    #[test]
    fn test_set_outside_bounds_ignored() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(100, 10, 'X'); // col out of bounds
        buffer.set(5, 30, 'Y'); // row out of bounds
        assert!(buffer.cells.is_empty());
    }

    #[test]
    fn test_set_at_boundary() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(79, 23, 'X'); // max valid position
        assert_eq!(buffer.get(79, 23), Some('X'));
    }

    #[test]
    fn test_get_returns_none_for_empty_cell() {
        let buffer = ScreenBuffer::new(80, 24);
        assert_eq!(buffer.get(5, 5), None);
    }

    #[test]
    fn test_get_returns_character() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(5, 5, 'Z');
        assert_eq!(buffer.get(5, 5), Some('Z'));
    }

    #[test]
    fn test_extract_text_single_line() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(0, 0, 'H');
        buffer.set(1, 0, 'e');
        buffer.set(2, 0, 'l');
        buffer.set(3, 0, 'l');
        buffer.set(4, 0, 'o');

        let text = buffer.extract_text(ScreenPos::new(0, 0), ScreenPos::new(4, 0));
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_extract_text_multi_line() {
        let mut buffer = ScreenBuffer::new(80, 24);
        // First line: "Hi"
        buffer.set(0, 0, 'H');
        buffer.set(1, 0, 'i');
        // Second line: "There"
        buffer.set(0, 1, 'T');
        buffer.set(1, 1, 'h');
        buffer.set(2, 1, 'e');
        buffer.set(3, 1, 'r');
        buffer.set(4, 1, 'e');

        let text = buffer.extract_text(ScreenPos::new(0, 0), ScreenPos::new(4, 1));
        assert_eq!(text, "Hi\nThere");
    }

    #[test]
    fn test_extract_text_trims_trailing_whitespace() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(0, 0, 'A');
        buffer.set(1, 0, 'B');
        // Columns 2-10 are empty (will be spaces)

        let text = buffer.extract_text(ScreenPos::new(0, 0), ScreenPos::new(10, 0));
        assert_eq!(text, "AB"); // Trailing spaces trimmed
    }

    #[test]
    fn test_extract_text_gaps_become_spaces() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(0, 0, 'A');
        // Column 1 is empty (gap)
        buffer.set(2, 0, 'B');

        let text = buffer.extract_text(ScreenPos::new(0, 0), ScreenPos::new(2, 0));
        assert_eq!(text, "A B"); // Gap becomes space
    }

    #[test]
    fn test_extract_text_partial_line_start() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(0, 0, 'A');
        buffer.set(1, 0, 'B');
        buffer.set(2, 0, 'C');
        buffer.set(3, 0, 'D');

        // Extract only "CD" (starting from column 2)
        let text = buffer.extract_text(ScreenPos::new(2, 0), ScreenPos::new(3, 0));
        assert_eq!(text, "CD");
    }

    #[test]
    fn test_extract_text_reversed_range_normalizes() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(0, 0, 'X');
        buffer.set(1, 0, 'Y');

        // End before start - should still work
        let text = buffer.extract_text(ScreenPos::new(1, 0), ScreenPos::new(0, 0));
        assert_eq!(text, "XY");
    }

    #[test]
    fn test_extract_text_empty_buffer() {
        let buffer = ScreenBuffer::new(80, 24);
        let text = buffer.extract_text(ScreenPos::new(0, 0), ScreenPos::new(5, 0));
        assert_eq!(text, ""); // All spaces trimmed
    }

    #[test]
    fn test_extract_text_single_character() {
        let mut buffer = ScreenBuffer::new(80, 24);
        buffer.set(5, 5, 'Z');

        let text = buffer.extract_text(ScreenPos::new(5, 5), ScreenPos::new(5, 5));
        assert_eq!(text, "Z");
    }

    #[test]
    fn test_extract_text_multiline_with_varying_lengths() {
        let mut buffer = ScreenBuffer::new(80, 24);
        // Line 0: "Short"
        for (i, ch) in "Short".chars().enumerate() {
            buffer.set(i as u16, 0, ch);
        }
        // Line 1: "Much longer line"
        for (i, ch) in "Much longer line".chars().enumerate() {
            buffer.set(i as u16, 1, ch);
        }
        // Line 2: "End"
        for (i, ch) in "End".chars().enumerate() {
            buffer.set(i as u16, 2, ch);
        }

        let text = buffer.extract_text(ScreenPos::new(0, 0), ScreenPos::new(2, 2));
        assert_eq!(text, "Short\nMuch longer line\nEnd");
    }
}
