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
