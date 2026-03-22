use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

const HEADING_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

const CODE_STYLE: Style = Style::new().fg(Color::Yellow);

const BOLD_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

const CODE_INLINE_STYLE: Style = Style::new().fg(Color::Green);

/// Render a markdown string into a ratatui [`Text`].
///
/// Handles the subset of markdown used in SWE Learn content files:
/// - ATX headings (`#`, `##`, `###`) — rendered bold cyan, `#` markers stripped
/// - Fenced code blocks (` ``` `) — rendered in yellow, fence lines stripped
/// - Bullet lists (`- ` and `  - `) — `-` replaced with `•`
/// - Inline bold (`**text**`) — rendered bold, markers stripped
/// - Inline code (`` `code` ``) — rendered green, backticks stripped
/// - Everything else — passed through unstyled
pub fn render(markdown: &str) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_code_block = false;

    for raw_line in markdown.lines() {
        // Code fence toggle — strip the fence line itself
        if raw_line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(raw_line.to_owned(), CODE_STYLE)));
            continue;
        }

        // ATX headings
        if let Some(text) = raw_line.strip_prefix("### ") {
            lines.push(Line::styled(text.to_owned(), HEADING_STYLE));
            continue;
        }
        if let Some(text) = raw_line.strip_prefix("## ") {
            lines.push(Line::styled(text.to_owned(), HEADING_STYLE));
            continue;
        }
        if let Some(text) = raw_line.strip_prefix("# ") {
            lines.push(Line::styled(
                text.to_owned(),
                HEADING_STYLE.add_modifier(Modifier::UNDERLINED),
            ));
            continue;
        }

        // Bullet lists
        let text = if let Some(rest) = raw_line.strip_prefix("  - ") {
            format!("  • {rest}")
        } else if let Some(rest) = raw_line.strip_prefix("- ") {
            format!("• {rest}")
        } else {
            raw_line.to_owned()
        };

        lines.push(parse_inline(&text));
    }

    Text::from(lines)
}

/// Parse inline markdown (`**bold**` and `` `code` ``) within a single line
/// into a [`Line`] of styled [`Span`]s.
fn parse_inline(text: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut rest = text;

    while !rest.is_empty() {
        // Find the nearest inline marker
        let bold_pos = rest.find("**");
        let code_pos = rest.find('`');

        // Determine which marker comes first
        let next = match (bold_pos, code_pos) {
            (None, None) => {
                spans.push(Span::raw(rest.to_owned()));
                break;
            }
            (Some(b), None) => ('b', b),
            (None, Some(c)) => ('c', c),
            (Some(b), Some(c)) => {
                if c < b {
                    ('c', c)
                } else {
                    ('b', b)
                }
            }
        };

        match next {
            ('c', pos) => {
                if pos > 0 {
                    spans.push(Span::raw(rest[..pos].to_owned()));
                }
                rest = &rest[pos + 1..];
                if let Some(end) = rest.find('`') {
                    spans.push(Span::styled(rest[..end].to_owned(), CODE_INLINE_STYLE));
                    rest = &rest[end + 1..];
                } else {
                    spans.push(Span::raw(format!("`{rest}")));
                    break;
                }
            }
            _ => {
                // bold
                let pos = next.1;
                if pos > 0 {
                    spans.push(Span::raw(rest[..pos].to_owned()));
                }
                rest = &rest[pos + 2..];
                if let Some(end) = rest.find("**") {
                    spans.push(Span::styled(rest[..end].to_owned(), BOLD_STYLE));
                    rest = &rest[end + 2..];
                } else {
                    spans.push(Span::raw(format!("**{rest}")));
                    break;
                }
            }
        }
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(text: &str) -> Vec<String> {
        render(text)
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect()
    }

    #[test]
    fn headings_strip_markers() {
        let out = render("## Problem Statement");
        assert_eq!(out.lines[0].spans[0].content, "Problem Statement");
        assert_eq!(out.lines[0].style, HEADING_STYLE);
    }

    #[test]
    fn code_fences_stripped() {
        let out = render("```\nlet x = 1;\n```");
        // Only the code line should appear, not the fences
        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].spans[0].content, "let x = 1;");
    }

    #[test]
    fn bullets_replaced() {
        assert_eq!(spans("- item"), vec!["• item"]);
        assert_eq!(spans("  - nested"), vec!["  • nested"]);
    }

    #[test]
    fn inline_bold() {
        let out = render("Use **bold** here");
        let contents: Vec<_> = out.lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(contents, vec!["Use ", "bold", " here"]);
        assert_eq!(out.lines[0].spans[1].style, BOLD_STYLE);
    }

    #[test]
    fn inline_code() {
        let out = render("Use `cargo build` here");
        let contents: Vec<_> = out.lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(contents, vec!["Use ", "cargo build", " here"]);
        assert_eq!(out.lines[0].spans[1].style, CODE_INLINE_STYLE);
    }
}
