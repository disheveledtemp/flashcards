
mod app;
mod browser;
mod parser;
mod ui;

use anyhow::Result;
use app::App;
use browser::Browser;
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
    path::{Path, PathBuf},
};

enum Screen {
    Pick(Browser),
    Study(App),
}

fn main() -> Result<()> {
    let screen = match env::args().nth(1) {
        Some(path) => {
            let path = PathBuf::from(path);
            Screen::Study(load_deck(&path)?)
        }
        None => Screen::Pick(Browser::new(env::current_dir()?)?),
    };

    run(screen)
}

fn load_deck(path: &Path) -> Result<App> {
    let cards = parser::load_flashcards(path)?;

    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("flashcards.md")
        .to_string();

    Ok(App::new(cards, filename))
}

fn run(mut screen: Screen) -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut screen);

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
    screen: &mut Screen,
) -> Result<()> {
    loop {
        terminal.draw(|frame| match screen {
            Screen::Pick(browser) => ui::draw_browser(frame, browser),
            Screen::Study(app) => ui::draw(frame, app),
        })?;

        let key = match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => key,
            // Redraw on resize and ignore other events.
            _ => continue,
        };

        match screen {
            Screen::Pick(browser) => match normalized(key.code) {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),

                KeyCode::Up | KeyCode::Char('k') => browser.move_up(),

                KeyCode::Down | KeyCode::Char('j') => browser.move_down(),

                KeyCode::Enter | KeyCode::Right => match browser.open() {
                    Ok(Some(path)) => match load_deck(&path) {
                        Ok(app) => *screen = Screen::Study(app),
                        Err(error) => browser.set_message(format!("{error:#}")),
                    },
                    Ok(None) => {}
                    Err(error) => browser.set_message(format!("{error:#}")),
                },

                KeyCode::Left | KeyCode::Backspace => {
                    if let Err(error) = browser.go_parent() {
                        browser.set_message(format!("{error:#}"));
                    }
                }

                _ => {}
            },

            Screen::Study(app) => {
                if app.show_help {
                    match normalized(key.code) {
                        KeyCode::Char('h') | KeyCode::Esc | KeyCode::Enter => {
                            app.show_help = false;
                        }

                        KeyCode::Char('q') => return Ok(()),

                        _ => {}
                    }

                    continue;
                }

                match normalized(key.code) {
                    KeyCode::Char('q') => return Ok(()),

                    KeyCode::Char('h') => app.show_help = true,

                    KeyCode::Enter => app.toggle_answer(),

                    KeyCode::Char('n') | KeyCode::Right => app.next(),

                    KeyCode::Char('p') | KeyCode::Left => app.previous(),

                    KeyCode::Char('r') => app.random(),

                    KeyCode::Up | KeyCode::Char('k') if app.revealed => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }

                    KeyCode::Down | KeyCode::Char('j') if app.revealed => {
                        app.scroll = app.scroll.saturating_add(1);
                    }

                    _ => {}
                }
            }
        }
    }
}

// Caps Lock must not turn every shortcut into a no-op.
fn normalized(code: KeyCode) -> KeyCode {
    match code {
        KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_case_insensitive() {
        assert_eq!(normalized(KeyCode::Char('Q')), KeyCode::Char('q'));
        assert_eq!(normalized(KeyCode::Char('h')), KeyCode::Char('h'));
        assert_eq!(normalized(KeyCode::Enter), KeyCode::Enter);
    }
}
