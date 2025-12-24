# Remove 'q' key as global exit shortcut

## Summary

Remove the 'q' key as a global exit shortcut from the TUI. Currently, pressing 'q' anywhere (except in splash screen and terminal view) exits the application.

## Current Behavior

In `src/main.rs` (lines 104-114), the event loop handles 'q' as a global quit:

```rust
let in_terminal = matches!(app.state.current_view, View::Terminal);
if !in_splash
    && !in_terminal
    && (key.code == KeyCode::Char('q')
        || (key.code == KeyCode::Char('c')
            && key.modifiers.contains(KeyModifiers::CONTROL)))
{
    return Ok(());
}
```

## Problem

The 'q' key exit shortcut is problematic because:
1. It conflicts with potential search/filter functionality where users might type 'q'
2. It's inconsistent with modern TUI conventions (Ctrl+C is standard)
3. It can cause accidental exits when users expect to type in fields

## Proposed Solution

1. Remove `key.code == KeyCode::Char('q')` from the global exit condition
2. Keep `Ctrl+C` as the only global exit shortcut
3. Update the status bar hints in `src/ui/layout.rs` if 'q:quit' is displayed anywhere

## Files to Modify

- `centy-tui/src/main.rs` - Remove 'q' from global exit handler
- `centy-tui/src/ui/layout.rs` - Update any keybinding hints if needed
