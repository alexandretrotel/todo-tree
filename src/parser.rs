use colored::Color;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a found TODO item in the source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    /// The tag that was matched (e.g., "TODO", "FIXME")
    pub tag: String,

    /// The message following the tag
    pub message: String,

    /// Line number where the tag was found (1-indexed)
    pub line: usize,

    /// Column number where the tag starts (1-indexed)
    pub column: usize,

    /// The full line content
    pub line_content: String,

    /// Optional author/assignee if specified (e.g., TODO(john): ...)
    pub author: Option<String>,

    /// Priority level inferred from tag type
    pub priority: Priority,
}

/// Priority levels for different tag types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Infer priority from tag name
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_uppercase().as_str() {
            "BUG" | "FIXME" | "XXX" => Priority::Critical,
            "HACK" | "WARN" | "WARNING" => Priority::High,
            "TODO" | "PERF" => Priority::Medium,
            "NOTE" | "INFO" | "IDEA" => Priority::Low,
            _ => Priority::Medium,
        }
    }

    /// Get the color associated with this priority level
    pub fn to_color(self) -> Color {
        match self {
            Priority::Critical => Color::Red,
            Priority::High => Color::Yellow,
            Priority::Medium => Color::Cyan,
            Priority::Low => Color::Green,
        }
    }
}

/// Parser for detecting TODO-style tags in source code
#[derive(Debug, Clone)]
pub struct TodoParser {
    /// Compiled regex pattern for matching tags (None if no tags to search for)
    pattern: Option<Regex>,

    /// Tags being searched for
    tags: Vec<String>,

    /// Whether matching is case-sensitive
    case_sensitive: bool,
}

impl TodoParser {
    /// Create a new parser with the given tags
    pub fn new(tags: &[String], case_sensitive: bool) -> Self {
        let pattern = Self::build_pattern(tags, case_sensitive);
        Self {
            pattern,
            tags: tags.to_vec(),
            case_sensitive,
        }
    }

    /// Build the regex pattern for matching tags
    fn build_pattern(tags: &[String], case_sensitive: bool) -> Option<Regex> {
        if tags.is_empty() {
            return None;
        }

        // Escape special regex characters in tags
        let escaped_tags: Vec<String> = tags.iter().map(|t| regex::escape(t)).collect();

        // Build pattern that matches:
        // - Optional comment prefix (// # /* <!-- -- ; etc.)
        // - Tag
        // - Optional author in parentheses
        // - Optional colon
        // - Message
        let pattern = format!(
            r"(?:^|[^a-zA-Z0-9_])({tags})(?:\(([^)]+)\))?[:\s]+(.*)$",
            tags = escaped_tags.join("|")
        );

        Some(
            RegexBuilder::new(&pattern)
                .case_insensitive(!case_sensitive)
                .multi_line(true)
                .build()
                .expect("Failed to build regex pattern"),
        )
    }

    /// Parse a single line for TODO items
    pub fn parse_line(&self, line: &str, line_number: usize) -> Option<TodoItem> {
        let pattern = self.pattern.as_ref()?;

        // Try to match the pattern
        if let Some(captures) = pattern.captures(line) {
            let tag_match = captures.get(1)?;
            let tag = tag_match.as_str().to_string();

            let author = captures.get(2).map(|m| m.as_str().to_string());

            let message = captures
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            // Calculate column (1-indexed)
            let column = tag_match.start() + 1;

            // Normalize the tag case for consistency
            let normalized_tag = if self.case_sensitive {
                tag
            } else {
                // Find the matching tag from our list (preserving original case)
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
                line_content: line.to_string(),
                author,
                priority,
            });
        }

        None
    }

    /// Parse content (multiple lines) for TODO items
    pub fn parse_content(&self, content: &str) -> Vec<TodoItem> {
        content
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| self.parse_line(line, idx + 1))
            .collect()
    }

    /// Parse a file for TODO items
    pub fn parse_file(&self, path: &Path) -> std::io::Result<Vec<TodoItem>> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.parse_content(&content))
    }

    /// Get the tags being searched for
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_tags() -> Vec<String> {
        vec![
            "TODO".to_string(),
            "FIXME".to_string(),
            "BUG".to_string(),
            "NOTE".to_string(),
            "HACK".to_string(),
        ]
    }

    #[test]
    fn test_parse_simple_todo() {
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_line("// TODO: Fix this later", 1);

        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, "Fix this later");
        assert_eq!(item.line, 1);
    }

    #[test]
    fn test_parse_todo_with_author() {
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_line("// TODO(john): Implement this", 5);

        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.tag, "TODO");
        assert_eq!(item.author, Some("john".to_string()));
        assert_eq!(item.message, "Implement this");
    }

    #[test]
    fn test_parse_hash_comment() {
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_line("# FIXME: This is broken", 1);

        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.tag, "FIXME");
        assert_eq!(item.message, "This is broken");
    }

    #[test]
    fn test_parse_case_insensitive() {
        let parser = TodoParser::new(&default_tags(), false);

        let result1 = parser.parse_line("// todo: lowercase", 1);
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().tag, "TODO");

        let result2 = parser.parse_line("// Todo: mixed case", 1);
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().tag, "TODO");
    }

    #[test]
    fn test_parse_case_sensitive() {
        let parser = TodoParser::new(&default_tags(), true);

        let result1 = parser.parse_line("// TODO: uppercase", 1);
        assert!(result1.is_some());

        let result2 = parser.parse_line("// todo: lowercase", 1);
        assert!(result2.is_none());
    }

    #[test]
    fn test_parse_multiple_lines() {
        let parser = TodoParser::new(&default_tags(), false);
        let content = r#"
// Regular comment
// TODO: First item
fn main() {}
// FIXME: Second item
// NOTE: Third item
"#;
        let items = parser.parse_content(content);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[1].tag, "FIXME");
        assert_eq!(items[2].tag, "NOTE");
    }

    #[test]
    fn test_priority_from_tag() {
        assert_eq!(Priority::from_tag("BUG"), Priority::Critical);
        assert_eq!(Priority::from_tag("FIXME"), Priority::Critical);
        assert_eq!(Priority::from_tag("HACK"), Priority::High);
        assert_eq!(Priority::from_tag("TODO"), Priority::Medium);
        assert_eq!(Priority::from_tag("NOTE"), Priority::Low);
    }

    #[test]
    fn test_todo_without_colon() {
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_line("// TODO fix this", 1);

        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, "fix this");
    }

    #[test]
    fn test_empty_tags() {
        let parser = TodoParser::new(&[], false);
        let result = parser.parse_line("// TODO: something", 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_special_characters_in_message() {
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_line("// TODO: Handle special chars: @#$%^&*()", 1);

        assert!(result.is_some());
        let item = result.unwrap();
        assert!(item.message.contains("@#$%^&*()"));
    }
}
