//! Scrollable sidebar component for vertical button lists

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

use super::button::{render_button, BUTTON_HEIGHT};

/// Height for header items (category labels)
pub const HEADER_HEIGHT: u16 = 1;

/// Configuration for the scrollable sidebar
pub struct ScrollableSidebarConfig {
    /// Whether to show scroll indicators when content is clipped
    pub show_scroll_indicators: bool,
    /// Whether to vertically center content when it fits
    pub center_content: bool,
}

impl Default for ScrollableSidebarConfig {
    fn default() -> Self {
        Self {
            show_scroll_indicators: true,
            center_content: true,
        }
    }
}

/// Type of sidebar item
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SidebarItemKind {
    /// A clickable button (height = BUTTON_HEIGHT)
    Button,
    /// A category header label (height = HEADER_HEIGHT)
    Header,
}

/// A single item in the scrollable sidebar
pub struct SidebarItem {
    pub label: String,
    pub kind: SidebarItemKind,
    pub is_selected: bool,
    pub is_enabled: bool,
    pub is_pressed: bool,
    pub label_color: Option<Color>,
}

impl SidebarItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: SidebarItemKind::Button,
            is_selected: false,
            is_enabled: true,
            is_pressed: false,
            label_color: None,
        }
    }

    /// Create a header item (non-interactive category label)
    pub fn header(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: SidebarItemKind::Header,
            is_selected: false,
            is_enabled: true,
            is_pressed: false,
            label_color: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }

    pub fn pressed(mut self, pressed: bool) -> Self {
        self.is_pressed = pressed;
        self
    }

    pub fn color(mut self, color: Option<Color>) -> Self {
        self.label_color = color;
        self
    }

    /// Get the height of this item
    pub fn height(&self) -> u16 {
        match self.kind {
            SidebarItemKind::Button => BUTTON_HEIGHT,
            SidebarItemKind::Header => HEADER_HEIGHT,
        }
    }
}

/// Render a scrollable sidebar with buttons and optional headers
pub fn render_scrollable_sidebar(
    frame: &mut Frame,
    area: Rect,
    items: &[SidebarItem],
    scroll_offset: usize,
    config: &ScrollableSidebarConfig,
) {
    let total_items = items.len();
    let available_height = area.height;

    if total_items == 0 {
        return;
    }

    // Calculate total content height (sum of all item heights)
    let total_content_height: u16 = items.iter().map(|item| item.height()).sum();

    // Calculate visible items and scroll state
    let (top_padding, first_visible, visible_count) =
        if total_content_height <= available_height && config.center_content {
            // Content fits - center it, no scrolling needed
            let padding = (available_height - total_content_height) / 2;
            (padding, 0, total_items)
        } else {
            // Content doesn't fit - enable scrolling
            let indicator_space = if config.show_scroll_indicators { 2 } else { 0 };
            let usable_height = available_height.saturating_sub(indicator_space);

            // Find first visible item and count how many fit
            let max_offset = calculate_max_scroll_offset(items, usable_height);
            let clamped_offset = scroll_offset.min(max_offset);

            // Count how many items fit from the offset
            let mut height_used: u16 = 0;
            let mut visible = 0;
            for item in items.iter().skip(clamped_offset) {
                if height_used + item.height() > usable_height {
                    break;
                }
                height_used += item.height();
                visible += 1;
            }

            (0, clamped_offset, visible.max(1)) // Always show at least 1 item
        };

    let can_scroll_up = first_visible > 0;
    let can_scroll_down = first_visible + visible_count < total_items;

    // Build layout constraints
    let mut constraints = Vec::new();

    if top_padding > 0 {
        constraints.push(Constraint::Length(top_padding));
    }

    if can_scroll_up && config.show_scroll_indicators {
        constraints.push(Constraint::Length(1)); // Up indicator
    }

    for i in 0..visible_count {
        let item = &items[first_visible + i];
        constraints.push(Constraint::Length(item.height()));
    }

    if can_scroll_down && config.show_scroll_indicators {
        constraints.push(Constraint::Length(1)); // Down indicator
    }

    constraints.push(Constraint::Min(0)); // Fill remaining space

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Render scroll indicators and items
    let mut chunk_idx = 0;

    if top_padding > 0 {
        chunk_idx += 1; // Skip padding chunk
    }

    if can_scroll_up && config.show_scroll_indicators {
        let indicator = Paragraph::new("^")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(indicator, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    for i in 0..visible_count {
        let item = &items[first_visible + i];
        match item.kind {
            SidebarItemKind::Button => {
                render_button(
                    frame,
                    chunks[chunk_idx],
                    &item.label,
                    item.is_selected,
                    item.is_enabled,
                    item.is_pressed,
                    item.label_color,
                );
            }
            SidebarItemKind::Header => {
                let header = Paragraph::new(Span::styled(
                    &item.label,
                    Style::default().fg(Color::DarkGray),
                ));
                frame.render_widget(header, chunks[chunk_idx]);
            }
        }
        chunk_idx += 1;
    }

    if can_scroll_down && config.show_scroll_indicators {
        let indicator = Paragraph::new("v")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(indicator, chunks[chunk_idx]);
    }
}

/// Calculate the maximum scroll offset for variable-height items
fn calculate_max_scroll_offset(items: &[SidebarItem], usable_height: u16) -> usize {
    if items.is_empty() {
        return 0;
    }

    // Find the first offset where remaining items fit in usable_height
    for offset in (0..items.len()).rev() {
        let mut height: u16 = 0;
        for item in items.iter().skip(offset) {
            height += item.height();
        }
        if height > usable_height {
            return (offset + 1).min(items.len().saturating_sub(1));
        }
    }
    0
}
