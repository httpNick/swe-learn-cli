pub mod home;
pub mod module_view;
pub mod topic_view;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => home::draw(frame, app),
        Screen::Module => module_view::draw(frame, app),
        Screen::Topic => topic_view::draw(frame, app),
    }
}
