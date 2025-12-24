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
        // Get terminal size for animation calculations
        let terminal_height = terminal.size()?.height;

        // Update splash animation if active
        let in_splash = app.in_splash();
        if in_splash {
            app.update_splash(terminal_height);
        }

        // Draw the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Use faster polling during animation (16ms = ~60fps)
        // Normal polling (100ms) otherwise
        let poll_duration = if in_splash {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(100)
        };

        // Handle events
        if event::poll(poll_duration)? {
            match event::read()? {
                Event::Key(key) => {
                    // Global quit: q or Ctrl+C (but not during splash)
                    if !in_splash
                        && (key.code == KeyCode::Char('q')
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL)))
                    {
                        return Ok(());
                    }

                    // Handle key event
                    app.handle_key(key).await?;
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse).await?;
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
