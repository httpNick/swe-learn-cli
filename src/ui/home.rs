use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::{app::App, modules::all_modules};

/// Render the home screen: module selection list + keybinding footer.
pub fn draw(frame: &mut Frame, app: &App) {
    // Split the frame vertically: main content area + fixed-height footer.
    let [content_area, footer_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

    // Build the list items from the module registry.
    let items: Vec<ListItem> = all_modules()
        .iter()
        .map(|m| ListItem::new(m.name))
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" SWE Learn ").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // Drive the list widget with a stateful `ListState` derived from `app`.
    let mut list_state = ListState::default().with_selected(Some(app.selected));

    frame.render_stateful_widget(list, content_area, &mut list_state);

    // Keybinding hint footer (no border — sits flush at the bottom).
    let footer = Paragraph::new(Line::from("  [↑↓/jk] Navigate   [Enter] Open   [q] Quit"));

    frame.render_widget(footer, footer_area);
}
