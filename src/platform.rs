//! Platform-specific configuration

use crossterm::event::KeyModifiers;

/// Platform-appropriate modifier for copy/save shortcuts
/// - macOS: SUPER (Cmd key)
/// - Linux/Windows: CONTROL (Ctrl key)
#[cfg(target_os = "macos")]
pub const COPY_MODIFIER: KeyModifiers = KeyModifiers::SUPER;

#[cfg(not(target_os = "macos"))]
pub const COPY_MODIFIER: KeyModifiers = KeyModifiers::CONTROL;

/// Save shortcut display for form help text
/// Ctrl+S works on all platforms (Cmd+W/Ctrl+W also work as fallback)
pub const SAVE_SHORTCUT: &str = "Ctrl+S";

/// Save as draft shortcut display
/// - macOS: "Cmd+D"
/// - Linux/Windows: "Ctrl+D"
#[cfg(target_os = "macos")]
pub const DRAFT_SHORTCUT: &str = "Cmd+D";

#[cfg(not(target_os = "macos"))]
pub const DRAFT_SHORTCUT: &str = "Ctrl+D";

/// Create and new shortcut display
/// - macOS: "Cmd+N"
/// - Linux/Windows: "Ctrl+N"
#[cfg(target_os = "macos")]
pub const CREATE_NEW_SHORTCUT: &str = "Cmd+N";

#[cfg(not(target_os = "macos"))]
pub const CREATE_NEW_SHORTCUT: &str = "Ctrl+N";
