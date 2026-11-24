//! Interactive column browser for the portfolio.
//!
//! `muhan` with no arguments opens the browser when attached to a terminal.
//! The subcommands remain available for scripts and piped output.

pub mod app;
pub mod tree;
pub mod view;

use std::io::{self, IsTerminal};

use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::data::Portfolio;
use app::App;

/// TUI needs a real terminal on both ends; otherwise fall back to the CLI.
pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Restore the screen and raw mode even if the event loop exits with an error.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
}

pub fn run(portfolio: &Portfolio) -> io::Result<()> {
    if !is_interactive() {
        return Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "the browser needs an interactive terminal",
        ));
    }

    enable_raw_mode()?;
    let _guard = TerminalGuard;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(portfolio);
    while !app.quit {
        terminal.draw(|frame| view::draw(frame, &app))?;
        if app.quit {
            break;
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => app.handle_key(key),
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}
