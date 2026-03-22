use include_dir::{include_dir, Dir};
use std::sync::LazyLock;

static CLOUD_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/cloud");
static SYSTEM_DESIGN_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/system-design");
static DATABASES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/databases");
static NETWORKING_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/networking");
static ALGORITHMS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/algorithms");
static DEVOPS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/devops");

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
        let s = self.content;
        if !s.starts_with("---") {
            return s;
        }
        let after_open = &s[3..];
        if let Some(close) = after_open.find("\n---\n") {
            &after_open[close + 5..]
        } else {
            s
        }
    }
}

/// Extracts the `title:` value from a frontmatter block.
/// Returns the filename-derived title if no `title:` line is found.
fn parse_title(content: &'static str) -> &'static str {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("title:") {
            return rest.trim().trim_matches('"');
        }
    }
    "Untitled"
}

/// Builds a sorted list of Topics from all `.md` files in an embedded directory.
fn build_topics(dir: &'static Dir<'static>) -> Vec<Topic> {
    let mut files: Vec<_> = dir.files().collect();
    files.sort_by_key(|f| f.path());
    files
        .into_iter()
        .filter(|f| f.path().extension().is_some_and(|e| e == "md"))
        .filter_map(|f| f.contents_utf8())
        .map(|content| Topic {
            title: parse_title(content),
            content,
        })
        .collect()
}

static CLOUD_TOPICS: LazyLock<Vec<Topic>> = LazyLock::new(|| build_topics(&CLOUD_DIR));
static SYSTEM_DESIGN_TOPICS: LazyLock<Vec<Topic>> =
    LazyLock::new(|| build_topics(&SYSTEM_DESIGN_DIR));
static DATABASES_TOPICS: LazyLock<Vec<Topic>> = LazyLock::new(|| build_topics(&DATABASES_DIR));
static NETWORKING_TOPICS: LazyLock<Vec<Topic>> = LazyLock::new(|| build_topics(&NETWORKING_DIR));
static ALGORITHMS_TOPICS: LazyLock<Vec<Topic>> = LazyLock::new(|| build_topics(&ALGORITHMS_DIR));
static DEVOPS_TOPICS: LazyLock<Vec<Topic>> = LazyLock::new(|| build_topics(&DEVOPS_DIR));

/// Returns the topics available for a given module index.
///
/// Modules without content yet return an empty slice — the UI handles this
/// gracefully by showing a "coming soon" message.
pub fn topics_for_module(module_index: usize) -> &'static [Topic] {
    match module_index {
        0 => &CLOUD_TOPICS,
        1 => &SYSTEM_DESIGN_TOPICS,
        2 => &DATABASES_TOPICS,
        3 => &NETWORKING_TOPICS,
        4 => &ALGORITHMS_TOPICS,
        5 => &DEVOPS_TOPICS,
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

    #[test]
    fn system_design_topics_loaded() {
        let topics = topics_for_module(1);
        assert_eq!(topics.len(), 10);
    }

    #[test]
    fn all_system_design_titles_non_empty() {
        for topic in topics_for_module(1) {
            assert!(!topic.title.is_empty(), "topic has empty title");
        }
    }
}
