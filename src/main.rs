//! Centy TUI - Terminal User Interface for Centy project management
//!
//! A Ratatui-based TUI for managing projects, issues, PRs, and docs
//! through the Centy daemon.

mod app;
mod config;
mod daemon;
mod platform;
mod state;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "centy_tui=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new().await?;
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle any errors
    if let Err(err) = result {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Get terminal size for animation and grid calculations
        let term_size = terminal.size()?;
        let terminal_height = term_size.height;
        let terminal_width = term_size.width;

        // Update terminal size for grid calculations
        app.terminal_size = Some((terminal_height, terminal_width));

        // Update splash animation if active
        let in_splash = app.in_splash();
        if in_splash {
            app.update_splash(terminal_height);
        }

        // Draw the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Clear selection on resize (positions are no longer valid)
        // This is handled in the Resize event below

        // Use faster polling during animation (16ms = ~60fps), normal polling (100ms) otherwise
        let poll_duration = if in_splash {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(100)
        };

        // Handle crossterm events
        if event::poll(poll_duration)? {
            match event::read()? {
                Event::Key(key) => {
                    // Platform-specific copy shortcut: Cmd+C on macOS, Ctrl+C on Linux/Windows
                    let is_copy_shortcut = key.code == KeyCode::Char('c')
                        && key.modifiers.contains(platform::COPY_MODIFIER);

                    // Ctrl+C is always the quit shortcut (double-tap) on all platforms
                    let is_ctrl_c = key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL);

                    if is_copy_shortcut && app.state.selection.has_selection() {
                        // Copy selection
                        if let Err(e) = app.copy_selection() {
                            app.status_message = Some(format!("Copy failed: {}", e));
                        }
                        app.last_ctrl_c = None;
                        continue;
                    }

                    if is_ctrl_c && !in_splash {
                        // Double-tap Ctrl+C to quit
                        let now = std::time::Instant::now();
                        if let Some(last) = app.last_ctrl_c {
                            if now.duration_since(last).as_millis() < 500 {
                                return Ok(());
                            }
                        }
                        app.last_ctrl_c = Some(now);
                        app.status_message = Some("Press ^C again to quit".to_string());
                        continue;
                    }

                    // Handle key event
                    app.handle_key(key).await?;
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse).await?;
                }
                Event::Resize(width, height) => {
                    // Update terminal size for grid calculations and pane recalculation
                    app.terminal_size = Some((height, width));
                    // Clear selection on resize (positions are no longer valid)
                    app.state.selection.clear();
                }
                _ => {}
            }
        }

        // Check if app wants to quit
        if app.should_quit() {
            return Ok(());
        }
    }
}
