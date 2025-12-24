//! Centy TUI - Terminal User Interface for Centy project management
//!
//! A Ratatui-based TUI for managing projects, issues, PRs, and docs
//! through the Centy daemon.

mod app;
mod config;
mod daemon;
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
                    // Global Ctrl+C: copy selection if exists, otherwise quit (but not during splash)
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if app.state.selection.has_selection() {
                            if let Err(e) = app.copy_selection() {
                                app.status_message = Some(format!("Copy failed: {}", e));
                            }
                        } else if !in_splash {
                            return Ok(());
                        }
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
