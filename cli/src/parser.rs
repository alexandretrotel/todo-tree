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
            base_pattern = base_pattern.replace(":(.*)", r"(?:\s*$|(?:(?::|\s+)(.*)))");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tags() -> Vec<String> {
        vec!["TODO".to_string(), "FIXME".to_string(), "BUG".to_string()]
    }

    fn custom_parser(tags: &[String], case_sensitive: bool) -> TodoParser {
        // Capture layout must match parse_line():
        // 1 = prefix
        // 2 = tag
        // 3 = author
        // 4 = message
        TodoParser::with_options(
            tags,
            case_sensitive,
            true,
            Some(r"(^|\s)($TAGS)(?:\(([^)]+)\))?(?::(.*))?$"),
        )
    }

    #[test]
    fn new_uses_default_options() {
        let parser = TodoParser::new(&tags(), true);
        assert_eq!(parser.tags(), &tags());
        assert!(parser.pattern.is_some());
    }

    #[test]
    fn empty_tags_disable_parsing() {
        let parser = TodoParser::new(&[], true);

        assert!(parser.pattern.is_none());
        assert!(parser.parse_line("// TODO: message", 1).is_none());
        assert!(parser.parse_content("// TODO: message").is_empty());
    }

    #[test]
    fn parse_line_with_custom_regex_extracts_basic_fields() {
        let parser = custom_parser(&tags(), true);
        let item = parser
            .parse_line(" TODO: write more tests", 7)
            .expect("expected TODO item");

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, "write more tests");
        assert_eq!(item.author, None);
        assert_eq!(item.line, 7);
        assert_eq!(item.column, 2);
        assert_eq!(
            item.line_content.as_deref(),
            Some(" TODO: write more tests")
        );
        assert_eq!(item.priority, Priority::from_tag("TODO"));
    }

    #[test]
    fn parse_line_with_custom_regex_extracts_author() {
        let parser = custom_parser(&tags(), true);
        let item = parser
            .parse_line(" FIXME(alice): handle edge case", 3)
            .expect("expected FIXME item");

        assert_eq!(item.tag, "FIXME");
        assert_eq!(item.author.as_deref(), Some("alice"));
        assert_eq!(item.message, "handle edge case");
        assert_eq!(item.line, 3);
        assert_eq!(item.column, 2);
        assert_eq!(item.priority, Priority::from_tag("FIXME"));
    }

    #[test]
    fn parse_line_trims_message() {
        let parser = custom_parser(&tags(), true);
        let item = parser
            .parse_line(" TODO:   message with spaces   ", 1)
            .expect("expected TODO item");

        assert_eq!(item.message, "message with spaces");
    }

    #[test]
    fn case_sensitive_parser_rejects_wrong_case() {
        let parser = custom_parser(&tags(), true);

        assert!(parser.parse_line(" todo: lower-case tag", 1).is_none());
        assert!(parser.parse_line(" TODO: upper-case tag", 1).is_some());
    }

    #[test]
    fn case_insensitive_parser_accepts_and_normalizes_tag() {
        let parser = custom_parser(&tags(), false);
        let item = parser
            .parse_line(" todo: lower-case tag", 1)
            .expect("expected TODO item");

        // In case-insensitive mode, the tag should be normalized back
        // to the configured spelling from self.tags.
        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, "lower-case tag");
        assert_eq!(item.priority, Priority::from_tag("TODO"));
    }

    #[test]
    fn case_insensitive_parser_uses_first_matching_configured_tag_spelling() {
        let tags = vec!["ToDo".to_string(), "FixMe".to_string()];
        let parser = custom_parser(&tags, false);

        let item = parser
            .parse_line(" todo: mixed case normalization", 1)
            .expect("expected ToDo item");

        assert_eq!(item.tag, "ToDo");
        assert_eq!(item.priority, Priority::from_tag("ToDo"));
    }

    #[test]
    fn parse_content_collects_multiple_items_with_correct_line_numbers() {
        let parser = custom_parser(&tags(), false);
        let content = "\
first line
 TODO: first task
nothing here
 fixme(bob): second task
 BUG: third task";

        let items = parser.parse_content(content);

        assert_eq!(items.len(), 3);

        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[0].message, "first task");
        assert_eq!(items[0].line, 2);

        assert_eq!(items[1].tag, "FIXME");
        assert_eq!(items[1].author.as_deref(), Some("bob"));
        assert_eq!(items[1].message, "second task");
        assert_eq!(items[1].line, 4);

        assert_eq!(items[2].tag, "BUG");
        assert_eq!(items[2].message, "third task");
        assert_eq!(items[2].line, 5);
    }

    #[test]
    fn parse_file_reads_and_parses_content() {
        let parser = custom_parser(&tags(), false);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("todo_parser_test_{unique}.txt"));

        fs::write(
            &path,
            "\
ignore
 TODO: from file
 FIXME(jane): also from file",
        )
        .unwrap();

        let items = parser.parse_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(items.len(), 2);

        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[0].message, "from file");
        assert_eq!(items[0].line, 2);

        assert_eq!(items[1].tag, "FIXME");
        assert_eq!(items[1].author.as_deref(), Some("jane"));
        assert_eq!(items[1].message, "also from file");
        assert_eq!(items[1].line, 3);
    }

    #[test]
    fn require_colon_true_does_not_match_default_pattern_without_colon() {
        let parser = TodoParser::with_options(&tags(), false, true, None);

        assert!(parser.parse_line("// TODO missing colon", 1).is_none());
        assert!(parser.parse_line("// TODO: has colon", 1).is_some());
    }

    #[test]
    fn require_colon_false_matches_default_pattern_with_or_without_colon() {
        let parser = TodoParser::with_options(&tags(), false, false, None);

        let with_colon = parser.parse_line("// TODO: with colon", 1);
        let with_space = parser.parse_line("// TODO with space", 2);
        let bare_tag = parser.parse_line("// TODO", 3);

        assert!(with_colon.is_some(), "should match with colon");
        assert!(
            with_space.is_some(),
            "should match with space when colon is optional"
        );
        assert!(
            bare_tag.is_some(),
            "should match bare tag when colon is optional"
        );

        let with_space = with_space.unwrap();
        assert_eq!(with_space.tag, "TODO");
        assert_eq!(with_space.message, "with space");

        let bare_tag = bare_tag.unwrap();
        assert_eq!(bare_tag.tag, "TODO");
        assert_eq!(bare_tag.message, "");
    }

    #[test]
    fn require_colon_false_rejects_false_positives() {
        let parser = TodoParser::with_options(&tags(), false, false, None);

        assert!(
            parser.parse_line("// TODO.complete()", 4).is_none(),
            "tag followed by '.' must not match"
        );
        assert!(
            parser.parse_line("// todoList", 5).is_none(),
            "tag embedded in a word must not match"
        );
    }

    #[test]
    fn require_colon_false_documents_double_colon_behavior() {
        let parser = TodoParser::with_options(&tags(), false, false, None);

        let item = parser
            .parse_line("* TODO::module::fn", 6)
            .expect("double-colon form should match current default regex behavior");

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, ":module::fn");
    }

    #[test]
    fn custom_regex_can_support_non_default_syntax() {
        let tags = vec!["TODO".to_string(), "FIXME".to_string()];
        let parser = TodoParser::with_options(
            &tags,
            false,
            true,
            // Matches e.g. "[TODO]{alice}: message"
            // 1 = prefix
            // 2 = tag
            // 3 = author
            // 4 = message
            Some(r"(^|\s)\[($TAGS)\](?:\{([^}]+)\})?:(.*)$"),
        );

        let item = parser
            .parse_line("[todo]{alice}: custom format works", 10)
            .expect("expected custom format to match");

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.author.as_deref(), Some("alice"));
        assert_eq!(item.message, "custom format works");
        assert_eq!(item.line, 10);
        assert_eq!(item.priority, Priority::from_tag("TODO"));
    }

    #[test]
    fn default_regex_smoke_test_common_comment_styles() {
        let parser = TodoParser::with_options(&tags(), false, true, None);

        let slash = parser.parse_line("// TODO: implement feature", 1);
        let hash = parser.parse_line("# FIXME: fix the bug", 2);

        assert!(slash.is_some(), "default regex should match // TODO: ...");
        assert!(hash.is_some(), "default regex should match # FIXME: ...");

        let slash = slash.unwrap();
        assert_eq!(slash.tag, "TODO");
        assert_eq!(slash.message, "implement feature");

        let hash = hash.unwrap();
        assert_eq!(hash.tag, "FIXME");
        assert_eq!(hash.message, "fix the bug");
    }

    #[test]
    fn tags_accessor_returns_configured_tags() {
        let tags = vec!["TODO".to_string(), "FIXME".to_string()];
        let parser = TodoParser::new(&tags, true);

        assert_eq!(parser.tags(), &tags);
    }
}
