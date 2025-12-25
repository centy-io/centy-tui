# Implement error queue system - replace status_message with queued error dialog

## Goal

Remove status_message from status bar and show errors in a queued error dialog.

## Requirements

* Remove only status_message from status bar (keep copy_message and disabled_reason)
* Queue errors so users dismiss them one at a time (FIFO)

## Implementation Steps

### 1. Update AppState (src/state/app_state.rs)

* Replace `error_dialog: Option<String>` with `error_queue: VecDeque<String>`
* Add helper methods: `push_error()`, `current_error()`, `dismiss_error()`, `has_errors()`

### 2. Update Error Dialog Dismissal (src/app.rs ~line 179)

* Change `self.state.error_dialog.is_some()` to `self.state.has_errors()`
* Change `self.state.error_dialog = None` to `self.state.dismiss_error()`

### 3. Add Helper Method to App (src/app.rs)

* Add `fn push_error(&mut self, message: impl Into<String>)`

### 4. Update UI Rendering (src/ui/mod.rs ~line 84)

* Change `&app.state.error_dialog` to `app.state.current_error()`

### 5. Remove Status Message from Status Bar (src/ui/layout.rs ~lines 166-170)

* Remove the yellow status_message rendering

### 6. Migrate status_message Usages (src/app.rs)

* Convert ~30 error messages to `push_error()`
* Remove success messages (UI shows changes visually)
* Keep transient feedback as `copy_message`

### 7. Update main.rs (~line 115)

* Convert copy failure to use error queue

### 8. Update Existing error_dialog Usage (src/app.rs ~line 765)

* Convert `self.state.error_dialog = Some(...)` to `self.state.push_error(...)`

## Files to Modify

* src/state/app_state.rs - Core state change
* src/app.rs - Migrate ~35 usages
* src/ui/layout.rs - Remove status_message rendering
* src/ui/mod.rs - Update error dialog rendering
* src/main.rs - Update copy failure handling

## Testing

1. Build: cargo build
1. Verify errors appear in dialog, not status bar
1. Verify multiple errors queue up
1. Verify Enter/Esc dismisses current error and shows next
1. Verify copy_message and disabled_reason still work in status bar
