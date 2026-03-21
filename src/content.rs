/// A single study topic with a display title and embedded markdown content.
pub struct Topic {
    pub title: &'static str,
    /// Raw markdown source, including frontmatter. Use `body()` to get the
    /// displayable text with frontmatter stripped.
    content: &'static str,
}

impl Topic {
    /// Returns the content with the YAML frontmatter block removed.
    pub fn body(&self) -> &'static str {
        // Frontmatter is delimited by `---` on its own line at the top of the
        // file. Strip everything up to and including the closing `---\n`.
        let s = self.content;
        if !s.starts_with("---") {
            return s;
        }
        // Find the closing delimiter after the opening one.
        let after_open = &s[3..];
        if let Some(close) = after_open.find("\n---\n") {
            // Skip past `---\n` (close marker) and the trailing newline.
            &after_open[close + 5..]
        } else {
            s
        }
    }
}

/// Returns the topics available for a given module index.
///
/// Modules without content yet return an empty slice — the UI handles this
/// gracefully by showing a "coming soon" message.
pub fn topics_for_module(module_index: usize) -> &'static [Topic] {
    match module_index {
        // System Design Questions (module index 1)
        1 => &[
            Topic {
                title: "URL Shortener",
                content: include_str!("../content/system-design/url-shortener.md"),
            },
            Topic {
                title: "Rate Limiter",
                content: include_str!("../content/system-design/rate-limiter.md"),
            },
            Topic {
                title: "Social Media Feed (Twitter/X)",
                content: include_str!("../content/system-design/twitter-feed.md"),
            },
        ],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_strips_frontmatter() {
        let topic = Topic {
            title: "test",
            content: "---\ntitle: \"Test\"\n---\n## Hello\nworld",
        };
        assert_eq!(topic.body(), "## Hello\nworld");
    }

}