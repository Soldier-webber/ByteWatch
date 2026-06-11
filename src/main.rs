/// ByteWatch - A modern terminal-based system monitor
/// 
/// This is the main entry point for the application.
/// It initializes the terminal, sets up the application state,
/// and manages the main event loop.

mod app;
mod system;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use std::time::Duration;
use ratatui::prelude::*;

/// Main function that sets up the terminal and runs the application
fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Run the application
    let app = App::new();
    let result = run_app(terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

/// Main application loop
/// 
/// This function handles:
/// - Event polling (keyboard input)
/// - System state updates
/// - UI rendering
/// - Application termination
fn run_app<B: Backend>(
    mut terminal: Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(1000); // Update every second
    let mut last_tick = std::time::Instant::now();

    loop {
        // Render the UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Calculate time until next tick
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Poll for events with timeout
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Char('r') => {
                        app.refresh_immediately();
                    }
                    _ => {}
                }
            }
        }

        // Update application state at tick rate
        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = std::time::Instant::now();
        }
    }
}
