use std::io::Stdout;

use color_eyre::Result;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    content::topics_for_module,
    events::{next_action, Action},
    modules::all_modules,
    ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    /// Topic list for the active module.
    Module,
    /// Scrollable content view for the active topic.
    Topic,
}

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    /// Highlighted row index in whichever list is currently visible.
    pub selected: usize,
    /// Module the user navigated into.
    pub active_module: usize,
    /// Topic the user opened within the active module.
    pub active_topic: usize,
    /// Vertical scroll offset for the topic content view.
    pub scroll: u16,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            screen: Screen::Home,
            selected: 0,
            active_module: 0,
            active_topic: 0,
            scroll: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::draw(frame, self))?;
            let action = next_action()?;
            self.handle_action(action);
        }
        Ok(())
    }

    pub fn handle_action(&mut self, action: Action) {
        match self.screen {
            Screen::Home => self.handle_home(action),
            Screen::Module => self.handle_module(action),
            Screen::Topic => self.handle_topic(action),
        }
    }

    fn handle_home(&mut self, action: Action) {
        let module_count = all_modules().len();
        match action {
            Action::Quit | Action::Back => self.should_quit = true,
            Action::NavigateUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            Action::NavigateDown => {
                if self.selected + 1 < module_count {
                    self.selected += 1;
                }
            }
            Action::Select => {
                self.active_module = self.selected;
                self.selected = 0;
                self.screen = Screen::Module;
            }
            Action::None => {}
        }
    }

    fn handle_module(&mut self, action: Action) {
        let topic_count = topics_for_module(self.active_module).len();
        match action {
            Action::Quit => self.should_quit = true,
            Action::Back => {
                self.selected = self.active_module;
                self.screen = Screen::Home;
            }
            Action::NavigateUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            Action::NavigateDown => {
                if topic_count > 0 && self.selected + 1 < topic_count {
                    self.selected += 1;
                }
            }
            Action::Select => {
                if topic_count > 0 {
                    self.active_topic = self.selected;
                    self.scroll = 0;
                    self.screen = Screen::Topic;
                }
            }
            Action::None => {}
        }
    }

    fn handle_topic(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Back => {
                self.selected = self.active_topic;
                self.screen = Screen::Module;
            }
            Action::NavigateUp => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            Action::NavigateDown => {
                self.scroll = self.scroll.saturating_add(1);
            }
            Action::Select | Action::None => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
