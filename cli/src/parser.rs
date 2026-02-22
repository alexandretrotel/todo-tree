use regex::{Regex, RegexBuilder};
use std::path::Path;
use todo_tree_core::{DEFAULT_REGEX, Priority, TodoItem};

#[derive(Debug, Clone)]
pub struct TodoParser {
    pattern: Option<Regex>,
    tags: Vec<String>,
    case_sensitive: bool,
}

impl TodoParser {
    pub fn new(tags: &[String], case_sensitive: bool) -> Self {
        Self::with_options(tags, case_sensitive, true, None)
    }

    pub fn with_options(
        tags: &[String],
        case_sensitive: bool,
        require_colon: bool,
        custom_regex: Option<&str>,
    ) -> Self {
        let pattern = Self::build_pattern(tags, case_sensitive, require_colon, custom_regex);
        Self {
            pattern,
            tags: tags.to_vec(),
            case_sensitive,
        }
    }

    fn build_pattern(
        tags: &[String],
        case_sensitive: bool,
        require_colon: bool,
        custom_regex: Option<&str>,
    ) -> Option<Regex> {
        if tags.is_empty() {
            return None;
        }

        let escaped_tags: Vec<String> = tags.iter().map(|t| regex::escape(t)).collect();
        let tags_alternation = escaped_tags.join("|");

        let mut base_pattern = custom_regex.unwrap_or(DEFAULT_REGEX).to_string();
        if custom_regex.is_none() && !require_colon {
            base_pattern = base_pattern.replace(":(.*)", "[:\\s]+(.*)");
        }

        let pattern_string = base_pattern.replace("$TAGS", &tags_alternation);
        let regex = RegexBuilder::new(&pattern_string)
            .case_insensitive(!case_sensitive)
            .multi_line(true)
            .build()
            .expect("Failed to build regex pattern");

        Some(regex)
    }

    pub fn parse_line(&self, line: &str, line_number: usize) -> Option<TodoItem> {
        let pattern = self.pattern.as_ref()?;
        if let Some(captures) = pattern.captures(line) {
            let tag_match = captures.get(2)?;
            let author = captures.get(3).map(|m| m.as_str().to_string());
            let message = captures
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            let tag = tag_match.as_str().to_string();
            let column = tag_match.start() + 1;

            let normalized_tag = if self.case_sensitive {
                tag
            } else {
                self.tags
                    .iter()
                    .find(|t| t.eq_ignore_ascii_case(&tag))
                    .cloned()
                    .unwrap_or(tag)
            };

            let priority = Priority::from_tag(&normalized_tag);

            return Some(TodoItem {
                tag: normalized_tag,
                message,
                line: line_number,
                column,
                line_content: Some(line.to_string()),
                author,
                priority,
            });
        }

        None
    }

    pub fn parse_content(&self, content: &str) -> Vec<TodoItem> {
        content
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| self.parse_line(line, idx + 1))
            .collect()
    }

    pub fn parse_file(&self, path: &Path) -> std::io::Result<Vec<TodoItem>> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.parse_content(&content))
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}
