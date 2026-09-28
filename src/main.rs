//! Rust Idle Game - Main Entry Point
//!
//! A base template for building idle games in Rust with a clean ECS architecture.

mod game;
mod systems;
mod components;
mod resources;
mod ui;
mod save;

use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use game::Game;
use ui::Ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Rust Idle Game");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run game
    let mut game = Game::new();
    let mut ui = Ui::new();

    let tick_rate = Duration::from_millis(100); // 10 ticks per second
    let mut last_tick = std::time::Instant::now();

    loop {
        // Handle events
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('s') => game.save()?,
                        KeyCode::Char('l') => game.load()?,
                        KeyCode::Tab => ui.next_panel(),
                        KeyCode::BackTab => ui.prev_panel(),
                        KeyCode::Up => ui.move_selection(-1),
                        KeyCode::Down => ui.move_selection(1),
                        KeyCode::Enter => ui.confirm_selection(&mut game),
                        _ => {}
                    }
                }
            }
        }

        // Game tick
        if last_tick.elapsed() >= tick_rate {
            game.tick(tick_rate.as_secs_f32());
            last_tick = std::time::Instant::now();
        }

        // Render
        terminal.draw(|f| ui.render(f, &game))?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    info!("Game exited cleanly");
    Ok(())
}