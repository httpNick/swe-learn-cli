use std::io::{self, stdout};

use color_eyre::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod content;
mod events;
mod markdown;
mod modules;
mod ui;

use app::App;

fn main() -> Result<()> {
    // Install color-eyre's panic and error hooks before doing anything else so
    // that panics and errors are presented with full context.
    color_eyre::install()?;

    run()
}

/// Set up the terminal, drive the application loop, then restore the terminal.
///
/// Terminal restoration happens whether `App::run` returns `Ok` or `Err`,
/// because the cleanup code runs before the error propagates.
fn run() -> Result<()> {
    // Enter alternate screen and raw mode so the TUI has exclusive control.
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = App::new().run(&mut terminal);

    // Always restore the terminal, even if the app loop returned an error.
    // We do this explicitly rather than via a Drop guard so the error from
    // restoration (if any) can be reported separately.
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}
