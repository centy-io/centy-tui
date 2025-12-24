# Add mouse support for improved TUI navigation

## Overview

centy-tui currently enables mouse capture (`EnableMouseCapture` in main.rs:37) but does not handle any mouse events. The event loop only matches `Event::Key`, ignoring `Event::Mouse` entirely (main.rs:90).

## Current State

- `crossterm::event::EnableMouseCapture` is enabled on startup
- `crossterm::event::DisableMouseCapture` is called on cleanup
- Only keyboard events are processed: `if let Event::Key(key) = event::read()?`
- No `Event::Mouse` handling exists in the codebase

## Proposed Mouse Features

### 1. Mouse Wheel Scrolling
- **Scroll lists** (Projects, Issues, PRs, Docs views) with mouse wheel
- **Scroll content** in detail views (IssueDetail, PrDetail, DocDetail) and Config view
- Should respect scroll acceleration for smooth UX

### 2. Click-to-Select in Lists
- Click on a project/issue/PR/doc in list views to select it
- Double-click or Enter-equivalent to navigate into the item
- Visual feedback on hover (optional, terminal support varies)

### 3. Sidebar Navigation
- Click on sidebar items (Projects, Issues, PRs, Docs, Config, Daemon)
- Should work the same as pressing number keys 1-6

### 4. Form Interaction
- Click on form fields to focus them (IssueCreate, IssueEdit, PrCreate, DocCreate views)
- Should update `active_form_field` in state

### 5. Button/Action Clicks
- Save button clicks in forms
- Cancel/back button clicks

## Technical Implementation

### Event Loop Changes (main.rs)
```rust
if event::poll(poll_duration)? {
    match event::read()? {
        Event::Key(key) => {
            // existing key handling
            app.handle_key(key).await?;
        }
        Event::Mouse(mouse) => {
            app.handle_mouse(mouse).await?;
        }
        _ => {}
    }
}
```

### App Changes (app.rs)
Add `handle_mouse` method that dispatches based on current view:
- `MouseEventKind::ScrollUp` / `ScrollDown` for wheel
- `MouseEventKind::Down(MouseButton::Left)` for clicks
- Track mouse position for hit-testing against UI elements

### State Changes (state/mod.rs)
May need to track:
- Currently hovered element (for visual feedback)
- Clickable regions computed during render

## Acceptance Criteria

- [ ] Mouse wheel scrolls lists and content views
- [ ] Clicking an item in a list selects it
- [ ] Double-click navigates into detail view
- [ ] Sidebar items are clickable
- [ ] Form fields can be focused via click
- [ ] All mouse interactions feel responsive and natural
- [ ] Mouse support works in common terminals (iTerm2, Terminal.app, Windows Terminal, kitty, Alacritty)

## References

- crossterm mouse events: https://docs.rs/crossterm/latest/crossterm/event/enum.MouseEventKind.html
- ratatui stateful widgets: https://docs.rs/ratatui/latest/ratatui/widgets/trait.StatefulWidget.html
