# Double-click/double-press on project card to open it

## Summary
Add double-click (mouse) and double-press (keyboard) support to open a project directly from the grid view, reducing the current two-step interaction to a single action.

## Current Behavior
1. User clicks a project card → card is **selected** (highlighted)
2. User presses Enter → project **opens** (navigates to Issues view)

This requires two distinct actions to open a project.

## Proposed Behavior
- **Double-click** on a project card: Select AND open the project in one action
- **Double-press Enter** on a project card: Same behavior (rapid Enter presses)

Single-click should still only select/highlight as it does today.

## Implementation Notes

### Mouse Double-Click Detection
In `centy-tui/src/app.rs` (around line 787-850), the mouse handler currently only handles `MouseEventKind::Down(Left)` for selection. Options:

1. **Track click timestamps**: Store last click time and position, detect if second click is within ~300-500ms on same card
2. **Crossterm's double-click**: Check if crossterm provides a DoubleClick variant (may require enabling extended mouse mode)

### Keyboard Double-Press Detection
In `centy-tui/src/app.rs` (around line 148-253), the key handler processes single Enter presses. Options:

1. **Track last Enter timestamp**: If Enter is pressed twice within ~300ms, treat as "open" action
2. **Alternative**: Could keep single Enter as open behavior (current) and this becomes mouse-only enhancement

### Relevant Code Locations
- Mouse handler: `centy-tui/src/app.rs:787-850`
- Keyboard handler: `centy-tui/src/app.rs:148-253` 
- Grid layout: `centy-tui/src/ui/projects.rs:19-72`
- Project opening logic: `centy-tui/src/app.rs:169-181`

### State Changes Needed
Add to `AppState`:
```rust
last_click_time: Option<Instant>,
last_click_index: Option<usize>,
```

## Acceptance Criteria
- [ ] Double-clicking a project card opens it directly
- [ ] Single-click still only selects the card
- [ ] Double-click timing threshold is reasonable (~300-500ms)
- [ ] Works correctly when clicking between different cards (no false positive)
- [ ] Consider keyboard double-press as optional enhancement
