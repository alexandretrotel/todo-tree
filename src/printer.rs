use crate::parser::{Priority, TodoItem};
use crate::scanner::ScanResult;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Output format for printing results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Tree view grouped by file
    Tree,
    /// Flat list of all items
    Flat,
    /// JSON output
    Json,
}

/// Options for printing
#[derive(Debug, Clone)]
pub struct PrintOptions {
    /// Output format
    pub format: OutputFormat,

    /// Whether to use colors
    pub colored: bool,

    /// Whether to show line numbers
    pub show_line_numbers: bool,

    /// Whether to show full file paths
    pub full_paths: bool,

    /// Whether to show clickable links (OSC 8)
    pub clickable_links: bool,

    /// Base path for relative path display
    pub base_path: Option<PathBuf>,

    /// Whether to show the summary
    pub show_summary: bool,

    /// Group by tag instead of file
    pub group_by_tag: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Tree,
            colored: true,
            show_line_numbers: true,
            full_paths: false,
            clickable_links: true,
            base_path: None,
            show_summary: true,
            group_by_tag: false,
        }
    }
}

/// Printer for displaying scan results
pub struct Printer {
    options: PrintOptions,
}

impl Printer {
    /// Create a new printer with the given options
    pub fn new(options: PrintOptions) -> Self {
        // Disable colors if requested or if not a terminal
        if !options.colored {
            colored::control::set_override(false);
        }

        Self { options }
    }

    /// Print scan results to stdout
    pub fn print(&self, result: &ScanResult) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.print_to(&mut handle, result)
    }

    /// Print scan results to a writer
    pub fn print_to<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        match self.options.format {
            OutputFormat::Tree => self.print_tree(writer, result),
            OutputFormat::Flat => self.print_flat(writer, result),
            OutputFormat::Json => self.print_json(writer, result),
        }
    }

    /// Print results in tree format
    fn print_tree<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        if result.files.is_empty() {
            writeln!(writer, "{}", "No TODO items found.".dimmed())?;
            return Ok(());
        }

        if self.options.group_by_tag {
            self.print_tree_by_tag(writer, result)?;
        } else {
            self.print_tree_by_file(writer, result)?;
        }

        if self.options.show_summary {
            writeln!(writer)?;
            self.print_summary(writer, result)?;
        }

        Ok(())
    }

    /// Print tree grouped by file
    fn print_tree_by_file<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        let sorted_files = result.sorted_files();
        let total_files = sorted_files.len();

        for (idx, (path, items)) in sorted_files.iter().enumerate() {
            let is_last_file = idx == total_files - 1;

            // Print file header
            self.print_file_header(writer, path, items.len(), is_last_file)?;

            // Print items
            let total_items = items.len();
            for (item_idx, item) in items.iter().enumerate() {
                let is_last_item = item_idx == total_items - 1;
                self.print_tree_item(writer, item, is_last_file, is_last_item, path)?;
            }
        }

        Ok(())
    }

    /// Print tree grouped by tag
    fn print_tree_by_tag<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        // Group items by tag
        let mut by_tag: HashMap<String, Vec<(PathBuf, TodoItem)>> = HashMap::new();

        for (path, items) in &result.files {
            for item in items {
                by_tag
                    .entry(item.tag.clone())
                    .or_default()
                    .push((path.clone(), item.clone()));
            }
        }

        let mut tags: Vec<_> = by_tag.keys().collect();
        tags.sort();

        let total_tags = tags.len();

        for (idx, tag) in tags.iter().enumerate() {
            let is_last_tag = idx == total_tags - 1;
            let items = by_tag.get(*tag).unwrap();

            // Print tag header
            let prefix = if is_last_tag {
                "└──"
            } else {
                "├──"
            };
            let colored_tag = self.colorize_tag(tag);
            writeln!(writer, "{} {} ({})", prefix, colored_tag, items.len())?;

            // Print items under this tag
            let total_items = items.len();
            for (item_idx, (path, item)) in items.iter().enumerate() {
                let is_last_item = item_idx == total_items - 1;
                let tree_prefix = if is_last_tag { "    " } else { "│   " };
                let item_prefix = if is_last_item {
                    "└──"
                } else {
                    "├──"
                };

                let display_path = self.format_path(path);
                let link = self.make_clickable_link(path, item.line);

                writeln!(
                    writer,
                    "{}{} {}:{} - {}",
                    tree_prefix,
                    item_prefix,
                    link.unwrap_or_else(|| display_path.to_string()),
                    item.line.to_string().cyan(),
                    item.message.dimmed()
                )?;
            }
        }

        Ok(())
    }

    /// Print file header for tree view
    fn print_file_header<W: Write>(
        &self,
        writer: &mut W,
        path: &Path,
        item_count: usize,
        is_last: bool,
    ) -> io::Result<()> {
        let prefix = if is_last { "└──" } else { "├──" };
        let display_path = self.format_path(path);
        let link = self.make_clickable_link(path, 1);

        let path_str = link.unwrap_or_else(|| {
            if self.options.colored {
                display_path.bold().to_string()
            } else {
                display_path.to_string()
            }
        });

        let count_str = format!("({})", item_count);
        let count_display = if self.options.colored {
            count_str.dimmed().to_string()
        } else {
            count_str
        };

        writeln!(writer, "{} {} {}", prefix, path_str, count_display)?;

        Ok(())
    }

    /// Print a single TODO item in tree format
    fn print_tree_item<W: Write>(
        &self,
        writer: &mut W,
        item: &TodoItem,
        is_last_file: bool,
        is_last_item: bool,
        path: &Path,
    ) -> io::Result<()> {
        let tree_prefix = if is_last_file { "    " } else { "│   " };
        let item_prefix = if is_last_item {
            "└──"
        } else {
            "├──"
        };

        let tag = self.colorize_tag(&item.tag);
        let line_num = if self.options.colored {
            format!("L{}", item.line).cyan().to_string()
        } else {
            format!("L{}", item.line)
        };

        let message = item.message.clone();

        // Add clickable link to line number if supported
        let line_display = if self.options.clickable_links {
            self.make_line_link(path, item.line)
                .unwrap_or_else(|| line_num.clone())
        } else {
            line_num
        };

        // Format with optional author
        let author_str = item
            .author
            .as_ref()
            .map(|a| format!("({})", a))
            .unwrap_or_default();

        if author_str.is_empty() {
            writeln!(
                writer,
                "{}{} [{}] {}: {}",
                tree_prefix, item_prefix, line_display, tag, message
            )?;
        } else {
            let author_display = if self.options.colored {
                author_str.yellow().to_string()
            } else {
                author_str
            };
            writeln!(
                writer,
                "{}{} [{}] {} {}: {}",
                tree_prefix, item_prefix, line_display, tag, author_display, message
            )?;
        }

        Ok(())
    }

    /// Print results in flat format
    fn print_flat<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        if result.files.is_empty() {
            writeln!(writer, "{}", "No TODO items found.".dimmed())?;
            return Ok(());
        }

        let mut all_items = result.all_items();
        all_items.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.line.cmp(&b.1.line)));

        for (path, item) in all_items {
            self.print_flat_item(writer, &path, &item)?;
        }

        if self.options.show_summary {
            writeln!(writer)?;
            self.print_summary(writer, result)?;
        }

        Ok(())
    }

    /// Print a single item in flat format
    fn print_flat_item<W: Write>(
        &self,
        writer: &mut W,
        path: &Path,
        item: &TodoItem,
    ) -> io::Result<()> {
        let display_path = self.format_path(path);
        let link = self.make_clickable_link(path, item.line);

        let path_str = link.unwrap_or_else(|| {
            if self.options.colored {
                display_path.bold().to_string()
            } else {
                display_path.to_string()
            }
        });

        let line_col = format!(":{}:{}", item.line, item.column);
        let line_col_display = if self.options.colored {
            line_col.cyan().to_string()
        } else {
            line_col
        };

        let tag = self.colorize_tag(&item.tag);

        writeln!(
            writer,
            "{}{} [{}] {}",
            path_str, line_col_display, tag, item.message
        )?;

        Ok(())
    }

    /// Print results in JSON format
    fn print_json<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        let json_output = JsonOutput::from_scan_result(result, &self.options);
        let json_str = serde_json::to_string_pretty(&json_output).map_err(io::Error::other)?;

        writeln!(writer, "{}", json_str)?;

        Ok(())
    }

    /// Print summary statistics
    fn print_summary<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        let summary_line = format!(
            "Found {} TODO items in {} files ({} files scanned)",
            result.total_count, result.files_with_todos, result.files_scanned
        );

        if self.options.colored {
            writeln!(writer, "{}", summary_line.bold())?;
        } else {
            writeln!(writer, "{}", summary_line)?;
        }

        // Print tag breakdown
        if !result.tag_counts.is_empty() {
            let mut tags: Vec<_> = result.tag_counts.iter().collect();
            tags.sort_by(|a, b| b.1.cmp(a.1));

            let breakdown: Vec<String> = tags
                .iter()
                .map(|(tag, count)| {
                    if self.options.colored {
                        format!("{}: {}", self.colorize_tag(tag), count)
                    } else {
                        format!("{}: {}", tag, count)
                    }
                })
                .collect();

            writeln!(writer, "  {}", breakdown.join(", "))?;
        }

        Ok(())
    }

    /// Format a path for display
    fn format_path(&self, path: &Path) -> String {
        if self.options.full_paths {
            path.display().to_string()
        } else if let Some(base) = &self.options.base_path {
            path.strip_prefix(base)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| path.display().to_string())
        } else {
            path.display().to_string()
        }
    }

    /// Create a clickable hyperlink using OSC 8 escape sequence
    fn make_clickable_link(&self, path: &Path, line: usize) -> Option<String> {
        if !self.options.clickable_links {
            return None;
        }

        // Check if terminal supports hyperlinks
        if !supports_hyperlinks() {
            return None;
        }

        let display_path = self.format_path(path);
        let abs_path = path.canonicalize().ok()?;
        let file_url = format!("file://{}:{}", abs_path.display(), line);

        // OSC 8 hyperlink format: \x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\
        let link = format!(
            "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            file_url,
            if self.options.colored {
                display_path.bold().to_string()
            } else {
                display_path
            }
        );

        Some(link)
    }

    /// Create a clickable link for a line number
    fn make_line_link(&self, path: &Path, line: usize) -> Option<String> {
        if !self.options.clickable_links || !supports_hyperlinks() {
            return None;
        }

        let abs_path = path.canonicalize().ok()?;
        let file_url = format!("file://{}:{}", abs_path.display(), line);
        let display = format!("L{}", line);

        let link = format!(
            "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            file_url,
            if self.options.colored {
                display.cyan().to_string()
            } else {
                display
            }
        );

        Some(link)
    }

    /// Colorize a tag based on its priority
    fn colorize_tag(&self, tag: &str) -> String {
        if !self.options.colored {
            return tag.to_string();
        }

        let color = Priority::from_tag(tag).to_color();
        tag.color(color).bold().to_string()
    }
}

/// JSON output structure
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    /// All TODO items grouped by file
    pub files: Vec<JsonFileEntry>,

    /// Summary statistics
    pub summary: JsonSummary,
}

/// JSON entry for a single file
#[derive(Debug, Serialize)]
pub struct JsonFileEntry {
    /// File path
    pub path: String,

    /// TODO items in this file
    pub items: Vec<JsonTodoItem>,
}

/// JSON representation of a TODO item
#[derive(Debug, Serialize)]
pub struct JsonTodoItem {
    /// The tag (TODO, FIXME, etc.)
    pub tag: String,

    /// The message content
    pub message: String,

    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Optional author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Priority level
    pub priority: String,
}

/// JSON summary statistics
#[derive(Debug, Serialize)]
pub struct JsonSummary {
    /// Total number of TODO items
    pub total_count: usize,

    /// Number of files with TODOs
    pub files_with_todos: usize,

    /// Total files scanned
    pub files_scanned: usize,

    /// Count by tag
    pub tag_counts: HashMap<String, usize>,
}

impl JsonOutput {
    /// Create JSON output from scan result
    pub fn from_scan_result(result: &ScanResult, options: &PrintOptions) -> Self {
        let mut files: Vec<JsonFileEntry> = result
            .sorted_files()
            .iter()
            .map(|(path, items)| {
                let display_path = if options.full_paths {
                    path.display().to_string()
                } else if let Some(base) = &options.base_path {
                    path.strip_prefix(base)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| path.display().to_string())
                } else {
                    path.display().to_string()
                };

                JsonFileEntry {
                    path: display_path,
                    items: items
                        .iter()
                        .map(|item| JsonTodoItem {
                            tag: item.tag.clone(),
                            message: item.message.clone(),
                            line: item.line,
                            column: item.column,
                            author: item.author.clone(),
                            priority: format!("{:?}", item.priority),
                        })
                        .collect(),
                }
            })
            .collect();

        files.sort_by(|a, b| a.path.cmp(&b.path));

        let summary = JsonSummary {
            total_count: result.total_count,
            files_with_todos: result.files_with_todos,
            files_scanned: result.files_scanned,
            tag_counts: result.tag_counts.clone(),
        };

        Self { files, summary }
    }
}

/// Check if the terminal supports hyperlinks (OSC 8)
fn supports_hyperlinks() -> bool {
    // Check common environment variables that indicate hyperlink support
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let supported_terminals = [
            "iTerm.app",
            "WezTerm",
            "Hyper",
            "Tabby",
            "Alacritty",
            "vscode",
            "VSCodium",
        ];
        if supported_terminals.iter().any(|t| term_program.contains(t)) {
            return true;
        }
    }

    // Check for explicit hyperlink support
    if let Ok(colorterm) = std::env::var("COLORTERM")
        && (colorterm == "truecolor" || colorterm == "24bit")
    {
        // Modern terminals with truecolor often support hyperlinks
        return true;
    }

    // Check VTE version (GNOME Terminal and derivatives)
    if std::env::var("VTE_VERSION").is_ok() {
        return true;
    }

    // Check for Konsole
    if std::env::var("KONSOLE_VERSION").is_ok() {
        return true;
    }

    // Default to false for unknown terminals
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::TodoItem;

    fn create_test_result() -> ScanResult {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/src/main.rs"),
            vec![
                TodoItem {
                    tag: "TODO".to_string(),
                    message: "Implement feature".to_string(),
                    line: 10,
                    column: 5,
                    line_content: "// TODO: Implement feature".to_string(),
                    author: None,
                    priority: Priority::Medium,
                },
                TodoItem {
                    tag: "FIXME".to_string(),
                    message: "Fix this bug".to_string(),
                    line: 20,
                    column: 5,
                    line_content: "// FIXME: Fix this bug".to_string(),
                    author: Some("john".to_string()),
                    priority: Priority::Critical,
                },
            ],
        );
        result
    }

    #[test]
    fn test_print_tree() {
        let result = create_test_result();
        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("TODO"));
        assert!(output_str.contains("FIXME"));
        assert!(output_str.contains("Implement feature"));
        assert!(output_str.contains("Fix this bug"));
    }

    #[test]
    fn test_print_flat() {
        let result = create_test_result();
        let options = PrintOptions {
            format: OutputFormat::Flat,
            colored: false,
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains(":10:5"));
        assert!(output_str.contains(":20:5"));
    }

    #[test]
    fn test_print_json() {
        let result = create_test_result();
        let options = PrintOptions {
            format: OutputFormat::Json,
            colored: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        assert!(parsed.get("files").is_some());
        assert!(parsed.get("summary").is_some());
    }

    #[test]
    fn test_empty_result() {
        let result = ScanResult::new(PathBuf::from("/test"));
        let options = PrintOptions {
            colored: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("No TODO items found"));
    }

    #[test]
    fn test_colorize_tag() {
        let options = PrintOptions {
            colored: true,
            ..Default::default()
        };
        let printer = Printer::new(options);

        // Just verify it doesn't panic
        let _ = printer.colorize_tag("TODO");
        let _ = printer.colorize_tag("FIXME");
        let _ = printer.colorize_tag("BUG");
        let _ = printer.colorize_tag("NOTE");
    }

    #[test]
    fn test_format_path() {
        let options = PrintOptions {
            full_paths: false,
            base_path: Some(PathBuf::from("/test")),
            ..Default::default()
        };
        let printer = Printer::new(options);

        let path = PathBuf::from("/test/src/main.rs");
        let formatted = printer.format_path(&path);
        assert_eq!(formatted, "src/main.rs");
    }

    #[test]
    fn test_format_path_full() {
        let options = PrintOptions {
            full_paths: true,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let path = PathBuf::from("/test/src/main.rs");
        let formatted = printer.format_path(&path);
        assert_eq!(formatted, "/test/src/main.rs");
    }

    #[test]
    fn test_json_output_structure() {
        let result = create_test_result();
        let options = PrintOptions::default();
        let json_output = JsonOutput::from_scan_result(&result, &options);

        assert_eq!(json_output.summary.total_count, 2);
        assert_eq!(json_output.summary.files_with_todos, 1);
        assert_eq!(json_output.files.len(), 1);
        assert_eq!(json_output.files[0].items.len(), 2);
    }

    #[test]
    fn test_print_no_summary() {
        let result = create_test_result();
        let options = PrintOptions {
            colored: false,
            show_summary: false,
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(!output_str.contains("Found"));
        assert!(!output_str.contains("files scanned"));
    }

    #[test]
    fn test_group_by_tag() {
        let result = create_test_result();
        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            group_by_tag: true,
            show_summary: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        // Should have tag headers
        assert!(output_str.contains("FIXME (1)") || output_str.contains("TODO (1)"));
    }

    #[test]
    fn test_colorize_tag_disabled() {
        let options = PrintOptions {
            colored: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        // When color is disabled, should return plain text
        let result = printer.colorize_tag("TODO");
        assert_eq!(result, "TODO");
    }

    #[test]
    fn test_make_clickable_link_disabled() {
        let options = PrintOptions {
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let path = PathBuf::from("/test/file.rs");
        let result = printer.make_clickable_link(&path, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_make_line_link_disabled() {
        let options = PrintOptions {
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let path = PathBuf::from("/test/file.rs");
        let result = printer.make_line_link(&path, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_print_tree_with_author() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/src/main.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "With author".to_string(),
                line: 10,
                column: 5,
                line_content: "// TODO(alice): With author".to_string(),
                author: Some("alice".to_string()),
                priority: Priority::Medium,
            }],
        );

        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("(alice)"));
        assert!(output_str.contains("With author"));
    }

    #[test]
    fn test_print_flat_empty() {
        let result = ScanResult::new(PathBuf::from("/test"));
        let options = PrintOptions {
            format: OutputFormat::Flat,
            colored: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("No TODO items found"));
    }

    #[test]
    fn test_format_path_no_base() {
        let options = PrintOptions {
            full_paths: false,
            base_path: None,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let path = PathBuf::from("/test/src/main.rs");
        let formatted = printer.format_path(&path);
        assert_eq!(formatted, "/test/src/main.rs");
    }

    #[test]
    fn test_format_path_strip_prefix_fails() {
        let options = PrintOptions {
            full_paths: false,
            base_path: Some(PathBuf::from("/different/base")),
            ..Default::default()
        };
        let printer = Printer::new(options);

        let path = PathBuf::from("/test/src/main.rs");
        let formatted = printer.format_path(&path);
        // Should fall back to full path when strip_prefix fails
        assert_eq!(formatted, "/test/src/main.rs");
    }

    #[test]
    fn test_json_output_with_author() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/src/main.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "Test".to_string(),
                line: 10,
                column: 5,
                line_content: "// TODO(bob): Test".to_string(),
                author: Some("bob".to_string()),
                priority: Priority::Medium,
            }],
        );

        let options = PrintOptions::default();
        let json_output = JsonOutput::from_scan_result(&result, &options);

        assert_eq!(
            json_output.files[0].items[0].author,
            Some("bob".to_string())
        );
    }

    #[test]
    fn test_print_summary_with_multiple_tags() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/a.rs"),
            vec![
                TodoItem {
                    tag: "TODO".to_string(),
                    message: "First".to_string(),
                    line: 1,
                    column: 1,
                    line_content: "// TODO: First".to_string(),
                    author: None,
                    priority: Priority::Medium,
                },
                TodoItem {
                    tag: "FIXME".to_string(),
                    message: "Second".to_string(),
                    line: 2,
                    column: 1,
                    line_content: "// FIXME: Second".to_string(),
                    author: None,
                    priority: Priority::Critical,
                },
                TodoItem {
                    tag: "NOTE".to_string(),
                    message: "Third".to_string(),
                    line: 3,
                    column: 1,
                    line_content: "// NOTE: Third".to_string(),
                    author: None,
                    priority: Priority::Low,
                },
            ],
        );

        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            show_summary: true,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Found 3 TODO items"));
        assert!(output_str.contains("TODO:"));
        assert!(output_str.contains("FIXME:"));
        assert!(output_str.contains("NOTE:"));
    }

    #[test]
    fn test_print_tree_multiple_files() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/a.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "In A".to_string(),
                line: 1,
                column: 1,
                line_content: "// TODO: In A".to_string(),
                author: None,
                priority: Priority::Medium,
            }],
        );
        result.add_file(
            PathBuf::from("/test/b.rs"),
            vec![TodoItem {
                tag: "FIXME".to_string(),
                message: "In B".to_string(),
                line: 1,
                column: 1,
                line_content: "// FIXME: In B".to_string(),
                author: None,
                priority: Priority::Critical,
            }],
        );

        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            show_summary: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("In A"));
        assert!(output_str.contains("In B"));
        // Check tree structure characters
        assert!(output_str.contains("├──") || output_str.contains("└──"));
    }

    #[test]
    fn test_print_tree_colored() {
        let result = create_test_result();
        let options = PrintOptions {
            colored: true,
            clickable_links: false,
            show_summary: true,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        // Should not panic with colored output
        printer.print_to(&mut output, &result).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_print_flat_colored() {
        let result = create_test_result();
        let options = PrintOptions {
            format: OutputFormat::Flat,
            colored: true,
            clickable_links: false,
            show_summary: true,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_json_item_priority() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/file.rs"),
            vec![
                TodoItem {
                    tag: "BUG".to_string(),
                    message: "Critical".to_string(),
                    line: 1,
                    column: 1,
                    line_content: "// BUG: Critical".to_string(),
                    author: None,
                    priority: Priority::Critical,
                },
                TodoItem {
                    tag: "NOTE".to_string(),
                    message: "Low".to_string(),
                    line: 2,
                    column: 1,
                    line_content: "// NOTE: Low".to_string(),
                    author: None,
                    priority: Priority::Low,
                },
            ],
        );

        let options = PrintOptions::default();
        let json_output = JsonOutput::from_scan_result(&result, &options);

        // Check priority values in JSON output (priority is serialized as a string)
        let items = &json_output.files[0].items;
        assert!(items.iter().any(|i| i.priority == "Critical"));
        assert!(items.iter().any(|i| i.priority == "Low"));
    }

    #[test]
    fn test_group_by_tag_multiple_tags() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/file.rs"),
            vec![
                TodoItem {
                    tag: "TODO".to_string(),
                    message: "First TODO".to_string(),
                    line: 1,
                    column: 1,
                    line_content: "// TODO: First TODO".to_string(),
                    author: None,
                    priority: Priority::Medium,
                },
                TodoItem {
                    tag: "FIXME".to_string(),
                    message: "A FIXME".to_string(),
                    line: 2,
                    column: 1,
                    line_content: "// FIXME: A FIXME".to_string(),
                    author: None,
                    priority: Priority::Critical,
                },
                TodoItem {
                    tag: "TODO".to_string(),
                    message: "Second TODO".to_string(),
                    line: 3,
                    column: 1,
                    line_content: "// TODO: Second TODO".to_string(),
                    author: None,
                    priority: Priority::Medium,
                },
            ],
        );

        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            group_by_tag: true,
            show_summary: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        // Both tags should appear as headers
        assert!(output_str.contains("TODO (2)"));
        assert!(output_str.contains("FIXME (1)"));
    }

    #[test]
    fn test_print_to_stdout() {
        let result = create_test_result();
        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        // Test that print() works (writes to stdout)
        // We can't easily capture stdout, but we can verify it doesn't panic
        // The actual test is done via print_to with a buffer
        let mut buffer = Vec::new();
        let result_io = printer.print_to(&mut buffer, &result);
        assert!(result_io.is_ok());
    }

    #[test]
    fn test_output_format_enum() {
        // Test that OutputFormat variants work correctly
        let tree = OutputFormat::Tree;
        let flat = OutputFormat::Flat;
        let json = OutputFormat::Json;

        // Just verify they can be used in match expressions
        match tree {
            OutputFormat::Tree => {}
            _ => panic!("Expected Tree"),
        }
        match flat {
            OutputFormat::Flat => {}
            _ => panic!("Expected Flat"),
        }
        match json {
            OutputFormat::Json => {}
            _ => panic!("Expected Json"),
        }
    }

    #[test]
    fn test_print_options_default_values() {
        let options = PrintOptions::default();

        assert!(matches!(options.format, OutputFormat::Tree));
        assert!(options.colored);
        assert!(options.show_line_numbers);
        assert!(!options.full_paths);
        assert!(options.clickable_links);
        assert!(options.base_path.is_none());
        assert!(options.show_summary);
        assert!(!options.group_by_tag);
    }

    #[test]
    fn test_json_summary_fields() {
        let result = create_test_result();
        let options = PrintOptions::default();
        let json_output = JsonOutput::from_scan_result(&result, &options);

        assert_eq!(json_output.summary.total_count, result.total_count);
        assert_eq!(
            json_output.summary.files_with_todos,
            result.files_with_todos
        );
        assert_eq!(json_output.summary.files_scanned, result.files_scanned);
        assert!(!json_output.summary.tag_counts.is_empty());
    }

    #[test]
    fn test_json_file_entry_path() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/deep/nested/file.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "Test".to_string(),
                line: 1,
                column: 1,
                line_content: "// TODO: Test".to_string(),
                author: None,
                priority: Priority::Medium,
            }],
        );

        let options = PrintOptions {
            full_paths: false,
            base_path: Some(PathBuf::from("/test")),
            ..Default::default()
        };
        let json_output = JsonOutput::from_scan_result(&result, &options);

        // Path should be relative when base_path is set
        assert!(json_output.files[0].path.contains("nested"));
    }

    #[test]
    fn test_json_file_entry_full_path() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/file.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "Test".to_string(),
                line: 1,
                column: 1,
                line_content: "// TODO: Test".to_string(),
                author: None,
                priority: Priority::Medium,
            }],
        );

        let options = PrintOptions {
            full_paths: true,
            ..Default::default()
        };
        let json_output = JsonOutput::from_scan_result(&result, &options);

        // Path should be full when full_paths is true
        assert!(json_output.files[0].path.starts_with("/test"));
    }

    #[test]
    fn test_clickable_links_enabled_no_terminal_support() {
        // When clickable_links is enabled but terminal doesn't support them
        // (most test environments), the functions should return None
        let options = PrintOptions {
            clickable_links: true,
            colored: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        // Create a real file to test with
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "// TODO: test").unwrap();

        // These may return None if terminal doesn't support hyperlinks
        let link = printer.make_clickable_link(&file_path, 1);
        let line_link = printer.make_line_link(&file_path, 1);

        // In most test environments, these will be None
        // but we just verify they don't panic
        let _ = link;
        let _ = line_link;
    }

    #[test]
    fn test_tree_item_with_last_file_and_item() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/only_file.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "Only item".to_string(),
                line: 1,
                column: 1,
                line_content: "// TODO: Only item".to_string(),
                author: None,
                priority: Priority::Medium,
            }],
        );

        let options = PrintOptions {
            colored: false,
            clickable_links: false,
            show_summary: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        // Last file uses └── prefix
        assert!(output_str.contains("└──"));
    }

    #[test]
    fn test_print_tree_item_with_colored_author() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("/test/file.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "With colored author".to_string(),
                line: 1,
                column: 1,
                line_content: "// TODO(developer): With colored author".to_string(),
                author: Some("developer".to_string()),
                priority: Priority::Medium,
            }],
        );

        let options = PrintOptions {
            colored: true,
            clickable_links: false,
            show_summary: false,
            ..Default::default()
        };
        let printer = Printer::new(options);

        let mut output = Vec::new();
        printer.print_to(&mut output, &result).unwrap();

        // Should not panic and should produce output
        assert!(!output.is_empty());
    }

    #[test]
    fn test_supports_hyperlinks_function() {
        // Just test that the function runs without panicking
        // The actual result depends on the environment
        let result = supports_hyperlinks();
        // Result is a boolean
        let _ = result;
    }
}
