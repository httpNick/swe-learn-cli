use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

/// High-level actions produced by the event loop.
///
/// Each variant represents a user intention derived from one or more raw
/// terminal input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The user requested to quit the application.
    Quit,
    /// Navigate back to the previous screen.
    Back,
    /// Move the selection cursor up by one position.
    NavigateUp,
    /// Move the selection cursor down by one position.
    NavigateDown,
    /// Confirm / open the currently highlighted item.
    Select,
    /// The event did not map to any meaningful action.
    None,
}

/// Read the next terminal event and map it to an [`Action`].
///
/// This function blocks until an event is available. It only processes
/// key-press events; key-release and repeat events on Windows are ignored.
///
/// | Input          | Action           |
/// |----------------|------------------|
/// | `q`            | `Quit`           |
/// | `Esc` or `b`   | `Back`           |
/// | `↑` or `k`     | `NavigateUp`     |
/// | `↓` or `j`     | `NavigateDown`   |
/// | `Enter`        | `Select`         |
/// | anything else  | `None`           |
pub fn next_action() -> Result<Action> {
    let action = match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Esc | KeyCode::Char('b') => Action::Back,
            KeyCode::Up | KeyCode::Char('k') => Action::NavigateUp,
            KeyCode::Down | KeyCode::Char('j') => Action::NavigateDown,
            KeyCode::Enter => Action::Select,
            _ => Action::None,
        },
        _ => Action::None,
    };
    Ok(action)
}
