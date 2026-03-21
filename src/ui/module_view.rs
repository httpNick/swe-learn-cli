use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{app::App, content::topics_for_module, modules::all_modules};

pub fn draw(frame: &mut Frame, app: &App) {
    let [content_area, footer_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let module = &all_modules()[app.active_module];
    let topics = topics_for_module(app.active_module);

    if topics.is_empty() {
        let placeholder = Paragraph::new(format!(
            "\n  No topics yet for \"{}\".\n\n  Content coming soon.",
            module.name
        ))
        .block(
            Block::default()
                .title(format!(" {} ", module.name))
                .borders(Borders::ALL),
        );
        frame.render_widget(placeholder, content_area);
    } else {
        let items: Vec<ListItem> = topics
            .iter()
            .map(|t| ListItem::new(format!("  {}", t.title)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} ", module.name))
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        let mut list_state = ListState::default().with_selected(Some(app.selected));
        frame.render_stateful_widget(list, content_area, &mut list_state);
    }

    let footer = Paragraph::new(
        "  [↑↓/jk] Navigate   [Enter] Open   [Esc/b] Back   [q] Quit",
    );
    frame.render_widget(footer, footer_area);
}
