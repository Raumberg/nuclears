use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nuclear_monitor::app::{App, AppState};
use nuclear_monitor::ui;
use std::{io, time::{Duration, Instant}};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let app = App::new();
    
    // Run the app
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> Result<()> {
    // Increase frame rate to ~30 FPS for smoother animation
    let tick_rate = Duration::from_millis(33); // 33ms ≈ 30 FPS
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => app.handle_key(key),
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }
}
