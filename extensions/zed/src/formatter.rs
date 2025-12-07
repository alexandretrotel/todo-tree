use crate::types::{DEFAULT_TAGS, Priority, ScanResult};

/// Formats scan results for display in the Assistant.
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format TODO items for the /todos command output.
    ///
    /// Returns a markdown-formatted string with:
    /// - Header with summary counts
    /// - Files grouped with their TODO items
    /// - Summary by tag at the end
    pub fn format_todos(result: &ScanResult) -> String {
        if result.is_empty() {
            return "No TODO items found in this project.\n".to_string();
        }

        let mut output = String::new();

        // Header
        output.push_str(&Self::format_todos_header(result));

        // Files with their items
        for file in &result.get_files() {
            output.push_str(&Self::format_file_section(file));
        }

        // Tag summary
        output.push_str(&Self::format_tag_summary(result));

        output
    }

    /// Format statistics for the /todos-stats command output.
    ///
    /// Returns a markdown-formatted string with:
    /// - Summary table with key metrics
    /// - Breakdown by tag with percentages
    /// - Priority level reference
    pub fn format_stats(result: &ScanResult) -> String {
        let mut output = String::new();

        output.push_str("# TODO Statistics\n\n");
        output.push_str(&Self::format_stats_table(result));
        output.push_str(&Self::format_tag_breakdown(result));
        output.push_str(&Self::format_priority_reference());

        output
    }

    /// Create the label for the /todos command output section.
    pub fn todos_label(result: &ScanResult, filter_tags: &[String]) -> String {
        if filter_tags.is_empty() {
            format!(
                "TODOs ({} items in {} files)",
                result.summary.total_count, result.summary.files_with_todos
            )
        } else {
            format!(
                "TODOs [{}] ({} items)",
                filter_tags.join(", "),
                result.summary.total_count
            )
        }
    }

    /// Create the label for the /todos-stats command output section.
    pub fn stats_label(result: &ScanResult) -> String {
        format!("TODO Statistics ({} total)", result.summary.total_count)
    }

    fn format_todos_header(result: &ScanResult) -> String {
        format!(
            "# TODO Items\n\nFound {} items in {} files ({} files scanned)\n\n",
            result.summary.total_count,
            result.summary.files_with_todos,
            result.summary.files_scanned
        )
    }

    fn format_file_section(file: &crate::types::FileResult) -> String {
        let mut output = String::new();

        output.push_str(&format!("## {}\n\n", file.path));

        for item in &file.items {
            output.push_str(&format!(
                "- **[L{}]** `{}{}`: {}\n",
                item.line,
                item.tag,
                item.format_author(),
                item.message
            ));
        }
        output.push('\n');

        output
    }

    fn format_tag_summary(result: &ScanResult) -> String {
        let mut output = String::new();

        output.push_str("## Summary by Tag\n\n");
        for (tag, count) in &result.summary.tag_counts {
            output.push_str(&format!("- **{}**: {}\n", tag, count));
        }

        output
    }

    fn format_stats_table(result: &ScanResult) -> String {
        let mut output = String::new();

        output.push_str("| Metric | Value |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!(
            "| Total items | {} |\n",
            result.summary.total_count
        ));
        output.push_str(&format!(
            "| Files with TODOs | {} |\n",
            result.summary.files_with_todos
        ));
        output.push_str(&format!(
            "| Files scanned | {} |\n",
            result.summary.files_scanned
        ));

        if result.summary.files_with_todos > 0 {
            output.push_str(&format!(
                "| Avg items per file | {:.2} |\n",
                result.summary.avg_items_per_file()
            ));
        }

        output
    }

    fn format_tag_breakdown(result: &ScanResult) -> String {
        let mut output = String::new();

        output.push_str("\n## By Tag\n\n");
        output.push_str("| Tag | Count | Percentage |\n");
        output.push_str("|-----|-------|------------|\n");

        for (tag, count) in &result.summary.tag_counts {
            let percentage = result.summary.tag_percentage(*count);
            output.push_str(&format!("| {} | {} | {:.1}% |\n", tag, count, percentage));
        }

        output
    }

    fn format_priority_reference() -> String {
        let mut output = String::new();

        output.push_str("\n## By Priority\n\n");
        output.push_str("| Priority | Tags |\n");
        output.push_str("|----------|------|\n");

        // Group tags by priority
        let priorities = [
            Priority::Critical,
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ];

        for priority in priorities {
            let tags: Vec<&str> = DEFAULT_TAGS
                .iter()
                .filter(|t| t.priority == priority)
                .map(|t| t.name)
                .collect();

            if !tags.is_empty() {
                output.push_str(&format!(
                    "| {} {} | {} |\n",
                    priority.emoji(),
                    priority.display_name(),
                    tags.join(", ")
                ));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileResult;
    use std::collections::HashMap;
    use todo_tree_core::Summary;
    use todo_tree_core::TodoItem;

    fn create_test_scan_result() -> ScanResult {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("TODO".to_string(), 2);
        tag_counts.insert("FIXME".to_string(), 1);

        ScanResult {
            files: vec![
                FileResult {
                    path: "src/main.rs".to_string(),
                    items: vec![
                        TodoItem {
                            tag: "TODO".to_string(),
                            message: "Implement this".to_string(),
                            line: 10,
                            column: 5,
                            priority: "Medium".to_string(),
                            author: None,
                        },
                        TodoItem {
                            tag: "FIXME".to_string(),
                            message: "Fix this bug".to_string(),
                            line: 25,
                            column: 3,
                            priority: "Critical".to_string(),
                            author: Some("alice".to_string()),
                        },
                    ],
                },
                FileResult {
                    path: "src/lib.rs".to_string(),
                    items: vec![TodoItem {
                        tag: "TODO".to_string(),
                        message: "Add tests".to_string(),
                        line: 5,
                        column: 1,
                        priority: "Medium".to_string(),
                        author: None,
                    }],
                },
            ],
            summary: Summary {
                total_count: 3,
                files_with_todos: 2,
                files_scanned: 10,
                tag_counts,
            },
        }
    }

    fn create_empty_scan_result() -> ScanResult {
        ScanResult {
            files: vec![],
            summary: Summary {
                total_count: 0,
                files_with_todos: 0,
                files_scanned: 5,
                tag_counts: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_format_todos_empty_result() {
        let result = create_empty_scan_result();
        let output = OutputFormatter::format_todos(&result);
        assert_eq!(output, "No TODO items found in this project.\n");
    }

    #[test]
    fn test_format_todos_contains_header() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_todos(&result);
        assert!(output.contains("# TODO Items"));
        assert!(output.contains("Found 3 items in 2 files"));
    }

    #[test]
    fn test_format_todos_contains_file_sections() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_todos(&result);
        assert!(output.contains("## src/main.rs"));
        assert!(output.contains("## src/lib.rs"));
    }

    #[test]
    fn test_format_todos_contains_items() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_todos(&result);
        assert!(output.contains("**[L10]** `TODO`: Implement this"));
        assert!(output.contains("**[L25]** `FIXME(alice)`: Fix this bug"));
    }

    #[test]
    fn test_format_todos_contains_tag_summary() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_todos(&result);
        assert!(output.contains("## Summary by Tag"));
        assert!(output.contains("**TODO**: 2"));
        assert!(output.contains("**FIXME**: 1"));
    }

    #[test]
    fn test_format_stats_contains_header() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_stats(&result);
        assert!(output.contains("# TODO Statistics"));
    }

    #[test]
    fn test_format_stats_contains_metrics_table() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_stats(&result);
        assert!(output.contains("| Total items | 3 |"));
        assert!(output.contains("| Files with TODOs | 2 |"));
        assert!(output.contains("| Files scanned | 10 |"));
    }

    #[test]
    fn test_format_stats_contains_avg_items() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_stats(&result);
        assert!(output.contains("| Avg items per file | 1.50 |"));
    }

    #[test]
    fn test_format_stats_no_avg_when_empty() {
        let result = create_empty_scan_result();
        let output = OutputFormatter::format_stats(&result);
        assert!(!output.contains("Avg items per file"));
    }

    #[test]
    fn test_format_stats_contains_tag_breakdown() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_stats(&result);
        assert!(output.contains("## By Tag"));
        assert!(output.contains("| Tag | Count | Percentage |"));
    }

    #[test]
    fn test_format_stats_contains_priority_reference() {
        let result = create_test_scan_result();
        let output = OutputFormatter::format_stats(&result);
        assert!(output.contains("## By Priority"));
        assert!(output.contains("🔴 Critical"));
        assert!(output.contains("🟡 High"));
        assert!(output.contains("🔵 Medium"));
        assert!(output.contains("🟢 Low"));
    }

    #[test]
    fn test_todos_label_no_filter() {
        let result = create_test_scan_result();
        let label = OutputFormatter::todos_label(&result, &[]);
        assert_eq!(label, "TODOs (3 items in 2 files)");
    }

    #[test]
    fn test_todos_label_with_filter() {
        let result = create_test_scan_result();
        let label =
            OutputFormatter::todos_label(&result, &["BUG".to_string(), "FIXME".to_string()]);
        assert_eq!(label, "TODOs [BUG, FIXME] (3 items)");
    }

    #[test]
    fn test_stats_label() {
        let result = create_test_scan_result();
        let label = OutputFormatter::stats_label(&result);
        assert_eq!(label, "TODO Statistics (3 total)");
    }

    #[test]
    fn test_format_file_section() {
        let file = FileResult {
            path: "test.rs".to_string(),
            items: vec![TodoItem {
                tag: "TODO".to_string(),
                message: "Test".to_string(),
                line: 1,
                column: 1,
                priority: "Medium".to_string(),
                author: None,
            }],
        };

        let output = OutputFormatter::format_file_section(&file);
        assert!(output.starts_with("## test.rs\n"));
        assert!(output.contains("**[L1]** `TODO`: Test"));
    }

    #[test]
    fn test_format_priority_reference_groups_correctly() {
        let output = OutputFormatter::format_priority_reference();

        // Check that each priority line contains correct tags
        assert!(output.contains("FIXME") && output.contains("BUG") && output.contains("XXX"));
        assert!(output.contains("HACK") && output.contains("WARN"));
        assert!(output.contains("TODO") && output.contains("PERF"));
        assert!(output.contains("NOTE"));
    }
}
