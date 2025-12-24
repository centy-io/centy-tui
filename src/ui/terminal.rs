//! Terminal pane rendering using cockpit

use crate::app::App;
use cockpit::CockpitWidget;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw the terminal view with cockpit panes
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(ref manager) = app.state.pane_manager {
        let pane_ids = manager.pane_ids();

        if pane_ids.is_empty() {
            // No panes - show empty state
            draw_empty_state(frame, area);
            return;
        }

        let areas = manager.calculate_areas(area);
        let areas_vec: Vec<_> = pane_ids.iter().map(|id| (*id, areas[id])).collect();

        let panes: Vec<_> = pane_ids
            .iter()
            .filter_map(|id| manager.get_pane(*id).map(|h| (*id, h)))
            .collect();

        let widget = CockpitWidget::new(&panes, &areas_vec, manager.focused());
        frame.render_widget(widget, area);
    } else {
        // No manager initialized - show empty state
        draw_empty_state(frame, area);
    }
}

/// Draw empty state when no terminal panes exist
fn draw_empty_state(frame: &mut Frame, area: Rect) {
    let text = "No terminal panes.\n\nPress 'n' to create a new terminal.";
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Terminal ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
