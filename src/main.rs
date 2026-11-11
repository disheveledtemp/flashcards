
mod app;
mod parser;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    env,
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .context("Usage: flashcards <file.md>")?;

    let cards = parser::load_flashcards(&path)?;

    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("flashcards.md")
        .to_string();

    let app = App::new(cards, filename);

    run(app)
}

fn run(mut app: App) -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    // Restore terminal even if the application encounters an error.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    while app.running {
        terminal.draw(|frame| {
            ui::draw(frame, app);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => {
                        app.running = false;
                    }

                    KeyCode::Enter => {
                        app.toggle_answer();
                    }

                    KeyCode::Char('n') | KeyCode::Right => {
                        app.next();
                    }

                    KeyCode::Char('p') | KeyCode::Left => {
                        app.previous();
                    }

                    KeyCode::Char('r') => {
                        app.random();
                    }

                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.revealed {
                            app.scroll = app.scroll.saturating_sub(1);
                        }
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.revealed {
                            app.scroll = app.scroll.saturating_add(1);
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}
