//! Reusable UI widget helpers

use ratatui::{
    layout::Rect,
    widgets::{List, ListState},
    Frame,
};

/// Render a scrollable list that automatically keeps the selected item visible.
///
/// This is the preferred way to render lists in the app. It wraps `render_stateful_widget`
/// with a `ListState`, ensuring the list scrolls to keep the selected item in view.
///
/// # Example
/// ```ignore
/// let list = List::new(items).block(block);
/// render_scrollable_list(frame, area, list, app.state.selected_index);
/// ```
pub fn render_scrollable_list(frame: &mut Frame, area: Rect, list: List, selected_index: usize) {
    let mut list_state = ListState::default().with_selected(Some(selected_index));
    frame.render_stateful_widget(list, area, &mut list_state);
}
