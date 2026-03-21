use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{app::App, content::topics_for_module, modules::all_modules};

pub fn draw(frame: &mut Frame, app: &App) {
    let [content_area, footer_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let module = &all_modules()[app.active_module];
    let topic = &topics_for_module(app.active_module)[app.active_topic];

    let text = crate::markdown::render(topic.body());

    let content = Paragraph::new(text)
        .block(
            Block::default()
                .title(format!(" {} › {} ", module.name, topic.title))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));

    frame.render_widget(content, content_area);

    let footer = Paragraph::new(
        "  [↑↓/jk] Scroll   [Esc/b] Back   [q] Quit",
    );
    frame.render_widget(footer, footer_area);
}
