use colored::Color;
use regex::{Regex, RegexBuilder};
use std::path::Path;
use todo_tree_core::{Priority, TodoItem};

/// Get the color associated with a priority level
pub fn priority_to_color(priority: Priority) -> Color {
    match priority {
        Priority::Critical => Color::Red,
        Priority::High => Color::Yellow,
        Priority::Medium => Color::Cyan,
        Priority::Low => Color::Green,
    }
}

/// Default regex pattern for matching TODO-style tags in comments.
///
/// This pattern is inspired by the VSCode Todo Tree extension and matches tags
/// that appear after common comment markers.
///
/// Pattern breakdown:
/// - `(//|#|<!--|;|/\*|\*|--)`  - Comment markers for most languages
/// - `\s*`                       - Optional whitespace after comment marker
/// - `($TAGS)`                   - The tag to match (placeholder, replaced at runtime)
/// - `(?:\(([^)]+)\))?`          - Optional author in parentheses
/// - `[:\s]`                     - Colon or whitespace after tag
/// - `(.*)`                      - The message
///
/// Supported comment syntaxes:
///   //    - C, C++, Java, JavaScript, TypeScript, Rust, Go, Swift, Kotlin
///   #     - Python, Ruby, Shell, YAML, TOML
///   /*    - C-style block comments
///   *     - Block comment continuation lines
///   <!--  - HTML, XML, Markdown comments
///   --    - SQL, Lua, Haskell, Ada
///   ;     - Lisp, Clojure, Assembly, INI files
///   %     - LaTeX, Erlang, MATLAB, Prolog
///   """   - Python docstrings
///   '''   - Python docstrings
///   REM   - Batch files
///   ::    - Batch files
pub const DEFAULT_REGEX: &str = r#"(//|#|<!--|;|/\*|\*|--|%|"""|'''|REM\s|::)\s*($TAGS)(?:\(([^)]+)\))?[:\s]+(.*)"#;

/// Parser for detecting TODO-style tags in source code
#[derive(Debug, Clone)]
pub struct TodoParser {
    /// Compiled regex pattern for matching tags (None if no tags to search for)
    pattern: Option<Regex>,

    /// Tags being searched for
    tags: Vec<String>,

    /// Whether matching is case-sensitive
    case_sensitive: bool,

    /// The regex pattern string (for ripgrep integration)
    pattern_string: Option<String>,
}

impl TodoParser {
    /// Create a new parser with the given tags
    pub fn new(tags: &[String], case_sensitive: bool) -> Self {
        Self::with_regex(tags, case_sensitive, None)
    }

    /// Create a new parser with a custom regex pattern
    ///
    /// The pattern should contain `$TAGS` as a placeholder which will be replaced
    /// with the alternation of escaped tags (e.g., `TODO|FIXME|BUG`).
    ///
    /// If no custom pattern is provided, the default pattern is used.
    pub fn with_regex(tags: &[String], case_sensitive: bool, custom_regex: Option<&str>) -> Self {
        let (pattern, pattern_string) = Self::build_pattern(tags, case_sensitive, custom_regex);
        Self {
            pattern,
            tags: tags.to_vec(),
            case_sensitive,
            pattern_string,
        }
    }

    /// Build the regex pattern for matching tags
    ///
    /// Returns both the compiled regex and the pattern string (for ripgrep integration).
    fn build_pattern(
        tags: &[String],
        case_sensitive: bool,
        custom_regex: Option<&str>,
    ) -> (Option<Regex>, Option<String>) {
        if tags.is_empty() {
            return (None, None);
        }

        // Escape special regex characters in tags
        let escaped_tags: Vec<String> = tags.iter().map(|t| regex::escape(t)).collect();
        let tags_alternation = escaped_tags.join("|");

        // Use custom regex or default
        let base_pattern = custom_regex.unwrap_or(DEFAULT_REGEX);

        // Replace $TAGS placeholder with the actual tags alternation
        let pattern_string = base_pattern.replace("$TAGS", &tags_alternation);

        let regex = RegexBuilder::new(&pattern_string)
            .case_insensitive(!case_sensitive)
            .multi_line(true)
            .build()
            .expect("Failed to build regex pattern");

        (Some(regex), Some(pattern_string))
    }

    /// Get the regex pattern string for ripgrep integration
    pub fn pattern_string(&self) -> Option<&str> {
        self.pattern_string.as_deref()
    }

    /// Parse a single line for TODO items
    pub fn parse_line(&self, line: &str, line_number: usize) -> Option<TodoItem> {
        let pattern = self.pattern.as_ref()?;

        // Try to match the pattern
        if let Some(captures) = pattern.captures(line) {
            // Default pattern capture groups:
            // Group 1: Comment marker (e.g., //, #, /*, etc.)
            // Group 2: Tag (e.g., TODO, FIXME, BUG)
            // Group 3: Author (optional, in parentheses)
            // Group 4: Message
            let tag_match = captures.get(2)?;
            let author = captures.get(3).map(|m| m.as_str().to_string());
            let message = captures
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            let tag = tag_match.as_str().to_string();

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
                line_content: Some(line.to_string()),
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

    #[test]
    fn test_priority_to_color() {
        // Test all priority levels have a color
        assert_eq!(priority_to_color(Priority::Critical), Color::Red);
        assert_eq!(priority_to_color(Priority::High), Color::Yellow);
        assert_eq!(priority_to_color(Priority::Medium), Color::Cyan);
        assert_eq!(priority_to_color(Priority::Low), Color::Green);
    }

    #[test]
    fn test_priority_from_unknown_tag() {
        // Unknown tags should default to Medium priority
        assert_eq!(Priority::from_tag("UNKNOWN"), Priority::Medium);
        assert_eq!(Priority::from_tag("CUSTOM"), Priority::Medium);
        assert_eq!(Priority::from_tag("RANDOM"), Priority::Medium);
    }

    #[test]
    fn test_priority_from_tag_case_variations() {
        // Test case variations
        assert_eq!(Priority::from_tag("bug"), Priority::Critical);
        assert_eq!(Priority::from_tag("Bug"), Priority::Critical);
        assert_eq!(Priority::from_tag("hack"), Priority::High);
        assert_eq!(Priority::from_tag("Hack"), Priority::High);
        assert_eq!(Priority::from_tag("warn"), Priority::High);
        assert_eq!(Priority::from_tag("WARNING"), Priority::High);
        assert_eq!(Priority::from_tag("perf"), Priority::Low);
        assert_eq!(Priority::from_tag("info"), Priority::Low);
        assert_eq!(Priority::from_tag("IDEA"), Priority::Low);
    }

    #[test]
    fn test_parse_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        std::fs::write(
            &file_path,
            r#"
// TODO: First item
fn main() {
    // FIXME: Second item
}
"#,
        )
        .unwrap();

        let parser = TodoParser::new(&default_tags(), false);
        let items = parser.parse_file(&file_path).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[1].tag, "FIXME");
    }

    #[test]
    fn test_parse_file_nonexistent() {
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_file(std::path::Path::new("/nonexistent/file.rs"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_tags_method() {
        let tags = default_tags();
        let parser = TodoParser::new(&tags, false);
        assert_eq!(parser.tags(), &tags);
    }

    #[test]
    fn test_parse_xxx_tag() {
        let tags = vec!["XXX".to_string()];
        let parser = TodoParser::new(&tags, false);
        let result = parser.parse_line("// XXX: Critical issue", 1);

        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.tag, "XXX");
        assert_eq!(item.priority, Priority::Low);
    }

    #[test]
    fn test_todo_item_equality() {
        let item1 = TodoItem {
            tag: "TODO".to_string(),
            message: "Test".to_string(),
            line: 1,
            column: 1,
            line_content: Some("// TODO: Test".to_string()),
            author: None,
            priority: Priority::Medium,
        };

        let item2 = TodoItem {
            tag: "TODO".to_string(),
            message: "Test".to_string(),
            line: 1,
            column: 1,
            line_content: Some("// TODO: Test".to_string()),
            author: None,
            priority: Priority::Medium,
        };

        assert_eq!(item1, item2);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_no_match_todo_in_accented_word() {
        // "método" (Spanish/Portuguese for "method") contains "todo" but should not match
        let parser = TodoParser::new(&default_tags(), false);
        let result = parser.parse_line("El método es importante", 1);
        assert!(result.is_none(), "Should not match 'todo' inside 'método'");
    }

    #[test]
    fn test_no_match_todos_spanish_portuguese() {
        // "todos" means "all" in Spanish/Portuguese and should not match
        let parser = TodoParser::new(&default_tags(), false);

        let result1 = parser.parse_line("Para todos los usuarios", 1);
        assert!(
            result1.is_none(),
            "Should not match 'todos' (Spanish for 'all')"
        );

        let result2 = parser.parse_line("Obrigado a todos vocês", 1);
        assert!(
            result2.is_none(),
            "Should not match 'todos' (Portuguese for 'all')"
        );
    }

    #[test]
    fn test_no_match_todo_suffix_in_unicode() {
        // Words ending in -todo with accented prefix should not match
        let parser = TodoParser::new(&default_tags(), false);

        let result = parser.parse_line("O método científico", 1);
        assert!(
            result.is_none(),
            "Should not match '-todo' suffix after accented char"
        );
    }

    #[test]
    fn test_match_real_todo_after_unicode() {
        // A real TODO after Unicode text should still match
        let parser = TodoParser::new(&default_tags(), false);

        let result = parser.parse_line("café // TODO: add milk", 1);
        assert!(
            result.is_some(),
            "Should match real TODO after Unicode text"
        );
        assert_eq!(result.unwrap().message, "add milk");
    }

    #[test]
    fn test_match_todo_with_unicode_in_message() {
        // TODO with Unicode characters in the message should work
        let parser = TodoParser::new(&default_tags(), false);

        let result = parser.parse_line("// TODO: añadir más café", 1);
        assert!(
            result.is_some(),
            "Should match TODO with Unicode in message"
        );
        assert_eq!(result.unwrap().message, "añadir más café");
    }

    #[test]
    fn test_no_match_cyrillic_boundary() {
        // Cyrillic characters should also be treated as word characters
        let parser = TodoParser::new(&default_tags(), false);

        // "методология" contains "todo" pattern but with Cyrillic prefix
        let result = parser.parse_line("использовать методологию", 1);
        assert!(
            result.is_none(),
            "Should not match TODO inside Cyrillic word"
        );
    }

    #[test]
    fn test_no_match_cjk_adjacent() {
        // CJK characters adjacent to TODO should prevent matching
        // (though CJK doesn't typically have this issue, good to test)
        let parser = TodoParser::new(&default_tags(), false);

        let result = parser.parse_line("完成TODO任务", 1);
        // This actually should NOT match because 成 and 任 are letters
        assert!(
            result.is_none(),
            "Should not match TODO between CJK characters"
        );
    }

    #[test]
    fn test_match_todo_after_cjk_with_comment() {
        // TODO in a comment after CJK text should match
        let parser = TodoParser::new(&default_tags(), false);

        // With comment-only detection, bare "TODO:" doesn't match - needs comment marker
        let result = parser.parse_line("中文 // TODO: task here", 1);
        assert!(
            result.is_some(),
            "Should match TODO in comment after CJK"
        );
        assert_eq!(result.unwrap().message, "task here");

        // Without comment marker, should NOT match
        let result2 = parser.parse_line("中文 TODO: task here", 1);
        assert!(
            result2.is_none(),
            "Should NOT match TODO without comment marker"
        );
    }

    #[test]
    fn test_typst_document_false_positive() {
        // Real-world case from the issue: typst document with "método"
        let parser = TodoParser::new(&default_tags(), false);

        let content = r#"
O método científico é fundamental.
Para todos os estudantes.
El método de investigación.
"#;
        let items = parser.parse_content(content);
        assert_eq!(
            items.len(),
            0,
            "Should not find any false positive TODOs in typst content"
        );
    }

    #[test]
    fn test_mixed_real_and_false_todos() {
        // Mix of real TODOs and false positives
        let parser = TodoParser::new(&default_tags(), false);

        let content = r#"
// TODO: This is a real todo
O método científico
# FIXME: Another real one
Para todos vocês
"#;
        let items = parser.parse_content(content);
        assert_eq!(
            items.len(),
            2,
            "Should only find real TODOs, not false positives"
        );
        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[1].tag, "FIXME");
    }

    fn tags_with_error() -> Vec<String> {
        vec![
            "TODO".to_string(),
            "FIXME".to_string(),
            "BUG".to_string(),
            "NOTE".to_string(),
            "HACK".to_string(),
            "ERROR".to_string(),
        ]
    }

    #[test]
    fn test_hash_comment_matches_like_vscode_extension() {
        // With ripgrep-style matching (like VSCode Todo Tree extension),
        // # is treated as a comment marker. This means markdown headings
        // like "# Error Handling" will match if ERROR is a tag.
        //
        // This is intentional - users should exclude *.md files if they
        // don't want to scan markdown headings. The VSCode extension
        // works the same way.
        let parser = TodoParser::new(&tags_with_error(), false);

        // "# Error Handling" matches because # is a comment marker
        // and "Error" is followed by whitespace
        let result = parser.parse_line("# ERROR: something", 1);
        assert!(result.is_some(), "Should match # ERROR: comment");
        assert_eq!(result.unwrap().tag, "ERROR");

        // Tags require a colon or whitespace separator after them
        let result2 = parser.parse_line("# ErrorHandling", 1);
        assert!(
            result2.is_none(),
            "Should not match without separator after tag"
        );
    }

    #[test]
    fn test_no_match_error_in_markdown_prose() {
        // Prose containing "error" should NOT match
        let parser = TodoParser::new(&tags_with_error(), false);

        let result = parser.parse_line("Use error classes for granular error handling", 1);
        assert!(result.is_none(), "Should not match 'error' in prose text");

        let result2 = parser.parse_line("The error message with status info", 1);
        assert!(
            result2.is_none(),
            "Should not match 'error' in prose describing error messages"
        );
    }

    #[test]
    fn test_no_match_error_in_code_block_content() {
        // Code examples showing error usage should NOT match
        let parser = TodoParser::new(&tags_with_error(), false);

        let result = parser.parse_line("throw new Error('Something went wrong')", 1);
        assert!(result.is_none(), "Should not match 'Error' in code example");

        let result2 = parser.parse_line("catch (error) {", 1);
        assert!(
            result2.is_none(),
            "Should not match 'error' in catch statement"
        );
    }

    #[test]
    fn test_match_real_error_comment() {
        // Real ERROR comments should still match
        let parser = TodoParser::new(&tags_with_error(), false);

        let result = parser.parse_line("// ERROR: This needs to be fixed", 1);
        assert!(result.is_some(), "Should match real ERROR comment");
        assert_eq!(result.unwrap().tag, "ERROR");

        let result2 = parser.parse_line("# ERROR: Handle this case", 1);
        assert!(result2.is_some(), "Should match ERROR with # comment");
        assert_eq!(result2.unwrap().tag, "ERROR");

        let result3 = parser.parse_line("/* ERROR: Critical issue */", 1);
        assert!(result3.is_some(), "Should match ERROR in block comment");
        assert_eq!(result3.unwrap().tag, "ERROR");
    }

    #[test]
    fn test_markdown_docs_with_ripgrep_style() {
        // With ripgrep-style matching, # is a comment marker, so markdown
        // headings with tags followed by separators will match.
        //
        // This matches VSCode Todo Tree extension behavior. Users should
        // exclude markdown files or use custom regex if this is undesired.
        let parser = TodoParser::new(&tags_with_error(), false);

        let content = r#"
# Error Handling

Use error classes for granular error handling.

## Error Classes

The following error classes are available:

- `FetchError`: Base error class
- `NetworkError`: Network-related errors
- `TimeoutError`: Request timeout errors

### Custom Error Types

You can create custom error types by extending the base class.

The error message with status info helps debugging.

```typescript
class CustomError extends FetchError {
  constructor(message: string) {
    super(message);
  }
}
```
"#;
        let items = parser.parse_content(content);
        // With ripgrep-style, "# Error Handling" and "## Error Classes" match
        // because # is a comment marker and ERROR tag is followed by space
        assert!(
            items.len() >= 2,
            "Markdown headings with ERROR followed by space will match with ripgrep-style"
        );
    }

    #[test]
    fn test_real_error_comments_in_markdown() {
        // Real ERROR comments embedded in markdown should still be detected
        let parser = TodoParser::new(&tags_with_error(), false);

        let content = r#"
# Error Handling

<!-- ERROR: This section needs review -->

Some documentation text.

<!-- TODO: Add more examples -->
"#;
        let items = parser.parse_content(content);
        assert_eq!(
            items.len(),
            2,
            "Should find ERROR and TODO in HTML comments"
        );
        assert!(items.iter().any(|i| i.tag == "ERROR"));
        assert!(items.iter().any(|i| i.tag == "TODO"));
    }

    fn tags_with_test() -> Vec<String> {
        vec![
            "TODO".to_string(),
            "FIXME".to_string(),
            "TEST".to_string(),
            "NOTE".to_string(),
        ]
    }

    #[test]
    fn test_no_match_json_script_keys() {
        // JSON script keys like "test: ci" should NOT match
        let parser = TodoParser::new(&tags_with_test(), false);

        // These are from package.json scripts section
        let result = parser.parse_line(r#"    "test: ci": "turbo run test","#, 1);
        assert!(
            result.is_none(),
            "Should not match 'test' in JSON key '\"test: ci\"'"
        );

        let result2 = parser.parse_line(r#"    "test:ci": "turbo run test","#, 1);
        assert!(
            result2.is_none(),
            "Should not match 'test' in JSON key '\"test:ci\"'"
        );

        let result3 = parser.parse_line(r#"    "test:coverage": "vitest --coverage","#, 1);
        assert!(
            result3.is_none(),
            "Should not match 'test' in JSON key '\"test:coverage\"'"
        );

        let result4 = parser.parse_line(r#"    "test:watch": "vitest --watch","#, 1);
        assert!(
            result4.is_none(),
            "Should not match 'test' in JSON key '\"test:watch\"'"
        );
    }

    #[test]
    fn test_no_match_json_various_patterns() {
        let parser = TodoParser::new(&tags_with_test(), false);

        // npm script naming conventions
        let cases = vec![
            r#""test:unit": "jest""#,
            r#""test:e2e": "cypress run""#,
            r#""test:lint": "eslint .""#,
            r#"  "note:important": "value","#,
            r#"{"test": "vitest"}"#,
        ];

        for case in cases {
            let result = parser.parse_line(case, 1);
            assert!(
                result.is_none(),
                "Should not match tag in JSON: {}",
                case
            );
        }
    }

    #[test]
    fn test_match_real_todo_in_json_comment() {
        // Real TODO comments in JS files with JSON-like content should match
        let parser = TodoParser::new(&tags_with_test(), false);

        let result = parser.parse_line("// TODO: update package.json scripts", 1);
        assert!(result.is_some(), "Should match real TODO comment");
        assert_eq!(result.unwrap().tag, "TODO");
    }

    #[test]
    fn test_package_json_comprehensive() {
        // Full package.json content test
        let parser = TodoParser::new(&tags_with_test(), false);

        let content = r#"
{
  "name": "my-project",
  "scripts": {
    "build": "turbo run build",
    "test": "vitest",
    "test:ci": "turbo run test",
    "test:coverage": "vitest --coverage",
    "test:ui": "vitest --ui",
    "test:watch": "vitest --watch",
    "note:deploy": "echo 'deploy script'"
  }
}
"#;
        let items = parser.parse_content(content);
        assert_eq!(
            items.len(),
            0,
            "Should not find any false positive TODOs in package.json"
        );
    }
}
