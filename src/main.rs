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
use cockpit::PaneEvent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use state::View;
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
        // Get terminal size for animation calculations and pane sizing
        let term_size = terminal.size()?;
        let terminal_height = term_size.height;
        let terminal_width = term_size.width;

        // Update terminal size for pane creation
        app.terminal_size = Some((terminal_height, terminal_width));

        // Update splash animation if active
        let in_splash = app.in_splash();
        if in_splash {
            app.update_splash(terminal_height);
        }

        // Draw the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Use faster polling during animation or when terminal panes are active (16ms = ~60fps)
        // Normal polling (100ms) otherwise
        let has_panes = app
            .state
            .pane_manager
            .as_ref()
            .map(|m| !m.pane_ids().is_empty())
            .unwrap_or(false);
        let poll_duration = if in_splash || has_panes {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(100)
        };

        // Handle crossterm events
        if event::poll(poll_duration)? {
            match event::read()? {
                Event::Key(key) => {
                    // Global quit: Ctrl+C (but not during splash or terminal view)
                    let in_terminal = matches!(app.state.current_view, View::Terminal);
                    if !in_splash
                        && !in_terminal
                        && key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }

                    // Handle key event
                    app.handle_key(key).await?;
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse).await?;
                }
                Event::Resize(_width, _height) => {
                    // Terminal was resized - panes will be recalculated on next draw
                }
                _ => {}
            }
        }

        // Poll cockpit pane events
        if let Some(ref mut manager) = app.state.pane_manager {
            for event in manager.poll_events() {
                match event {
                    PaneEvent::Exited { pane_id, code } => {
                        tracing::info!("Pane {pane_id:?} exited with code {code}");
                        app.status_message = Some(format!("Terminal exited (code {code})"));
                    }
                    PaneEvent::Crashed {
                        pane_id,
                        signal,
                        error,
                    } => {
                        tracing::warn!(
                            "Pane {pane_id:?} crashed: signal={signal:?}, error={error:?}"
                        );
                        app.status_message = Some("Terminal crashed".to_string());
                    }
                    _ => {}
                }
            }
        }

        // Check if app wants to quit
        if app.should_quit() {
            return Ok(());
        }
    }
}
