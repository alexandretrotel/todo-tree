use serde::Deserialize;
use std::collections::HashMap;

/// Result from scanning a project with todo-tree CLI.
///
/// This struct matches the JSON output format of `tt scan --json`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ScanResult {
    /// List of files containing TODO items
    pub files: Vec<FileResult>,
    /// Summary statistics
    pub summary: Summary,
}

impl ScanResult {
    /// Check if the scan found any TODO items
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// A file containing TODO items.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FileResult {
    /// Relative path to the file
    pub path: String,
    /// TODO items found in this file
    pub items: Vec<TodoItem>,
}

/// A single TODO item found in source code.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TodoItem {
    /// The tag type (TODO, FIXME, BUG, etc.)
    pub tag: String,
    /// The message content following the tag
    pub message: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Priority level as string (Critical, High, Medium, Low)
    pub priority: String,
    /// Optional author/assignee (e.g., from TODO(author): ...)
    pub author: Option<String>,
}

impl TodoItem {
    /// Format the author for display, returns "(author)" or empty string
    pub fn format_author(&self) -> String {
        self.author
            .as_ref()
            .map(|a| format!("({})", a))
            .unwrap_or_default()
    }
}

/// Summary statistics from a scan.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Summary {
    /// Total number of TODO items found
    pub total_count: usize,
    /// Number of files containing at least one TODO
    pub files_with_todos: usize,
    /// Total number of files scanned
    pub files_scanned: usize,
    /// Count of items per tag type
    pub tag_counts: HashMap<String, usize>,
}

impl Summary {
    /// Calculate average items per file (returns 0.0 if no files with todos)
    pub fn avg_items_per_file(&self) -> f64 {
        if self.files_with_todos > 0 {
            self.total_count as f64 / self.files_with_todos as f64
        } else {
            0.0
        }
    }

    /// Calculate percentage for a given tag count
    pub fn tag_percentage(&self, count: usize) -> f64 {
        if self.total_count > 0 {
            (count as f64 / self.total_count as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Tag definition with metadata for completions and display.
#[derive(Debug, Clone, PartialEq)]
pub struct TagDefinition {
    /// Tag name (e.g., "TODO")
    pub name: &'static str,
    /// Description for UI display
    pub description: &'static str,
    /// Priority level
    pub priority: Priority,
}

/// Priority levels for TODO tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Get emoji representation for the priority
    pub fn emoji(&self) -> &'static str {
        match self {
            Priority::Critical => "🔴",
            Priority::High => "🟡",
            Priority::Medium => "🔵",
            Priority::Low => "🟢",
        }
    }

    /// Get display name for the priority
    pub fn display_name(&self) -> &'static str {
        match self {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

/// Default tag definitions used by todo-tree.
pub const DEFAULT_TAGS: &[TagDefinition] = &[
    TagDefinition {
        name: "TODO",
        description: "General TODO items",
        priority: Priority::Medium,
    },
    TagDefinition {
        name: "FIXME",
        description: "Items that need fixing",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "BUG",
        description: "Known bugs",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "NOTE",
        description: "Notes and documentation",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "HACK",
        description: "Hacky solutions",
        priority: Priority::High,
    },
    TagDefinition {
        name: "XXX",
        description: "Critical items requiring attention",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "WARN",
        description: "Warnings",
        priority: Priority::High,
    },
    TagDefinition {
        name: "PERF",
        description: "Performance issues",
        priority: Priority::Medium,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_scan_result_is_empty() {
        let result = create_test_scan_result();
        assert!(!result.is_empty());

        let empty_result = create_empty_scan_result();
        assert!(empty_result.is_empty());
    }

    #[test]
    fn test_scan_result_summary_total_count() {
        let result = create_test_scan_result();
        assert_eq!(result.summary.total_count, 3);
    }

    #[test]
    fn test_scan_result_summary_files_with_todos() {
        let result = create_test_scan_result();
        assert_eq!(result.summary.files_with_todos, 2);
    }

    #[test]
    fn test_file_result_items_len() {
        let result = create_test_scan_result();
        assert_eq!(result.files[0].items.len(), 2);
        assert_eq!(result.files[1].items.len(), 1);
    }

    #[test]
    fn test_todo_item_format_author_with_author() {
        let item = TodoItem {
            tag: "TODO".to_string(),
            message: "Test".to_string(),
            line: 1,
            column: 1,
            priority: "Medium".to_string(),
            author: Some("alice".to_string()),
        };
        assert_eq!(item.format_author(), "(alice)");
    }

    #[test]
    fn test_todo_item_format_author_without_author() {
        let item = TodoItem {
            tag: "TODO".to_string(),
            message: "Test".to_string(),
            line: 1,
            column: 1,
            priority: "Medium".to_string(),
            author: None,
        };
        assert_eq!(item.format_author(), "");
    }

    #[test]
    fn test_summary_avg_items_per_file() {
        let result = create_test_scan_result();
        assert!((result.summary.avg_items_per_file() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_summary_avg_items_per_file_empty() {
        let result = create_empty_scan_result();
        assert_eq!(result.summary.avg_items_per_file(), 0.0);
    }

    #[test]
    fn test_summary_tag_percentage() {
        let result = create_test_scan_result();
        // TODO has 2 out of 3, so ~66.67%
        let todo_percentage = result.summary.tag_percentage(2);
        assert!((todo_percentage - 66.666).abs() < 0.01);
    }

    #[test]
    fn test_summary_tag_percentage_empty() {
        let result = create_empty_scan_result();
        assert_eq!(result.summary.tag_percentage(0), 0.0);
    }

    #[test]
    fn test_priority_emoji() {
        assert_eq!(Priority::Critical.emoji(), "🔴");
        assert_eq!(Priority::High.emoji(), "🟡");
        assert_eq!(Priority::Medium.emoji(), "🔵");
        assert_eq!(Priority::Low.emoji(), "🟢");
    }

    #[test]
    fn test_priority_display_name() {
        assert_eq!(Priority::Critical.display_name(), "Critical");
        assert_eq!(Priority::High.display_name(), "High");
        assert_eq!(Priority::Medium.display_name(), "Medium");
        assert_eq!(Priority::Low.display_name(), "Low");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_default_tags_count() {
        assert_eq!(DEFAULT_TAGS.len(), 8);
    }

    #[test]
    fn test_default_tags_contains_todo() {
        assert!(DEFAULT_TAGS.iter().any(|t| t.name == "TODO"));
    }

    #[test]
    fn test_default_tags_priorities() {
        let critical_tags: Vec<_> = DEFAULT_TAGS
            .iter()
            .filter(|t| t.priority == Priority::Critical)
            .collect();
        assert_eq!(critical_tags.len(), 3); // FIXME, BUG, XXX
    }

    #[test]
    fn test_scan_result_deserialization() {
        let json = r#"{
            "files": [
                {
                    "path": "test.rs",
                    "items": [
                        {
                            "tag": "TODO",
                            "message": "Test message",
                            "line": 1,
                            "column": 5,
                            "priority": "Medium"
                        }
                    ]
                }
            ],
            "summary": {
                "total_count": 1,
                "files_with_todos": 1,
                "files_scanned": 1,
                "tag_counts": {"TODO": 1}
            }
        }"#;

        let result: ScanResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.summary.total_count, 1);
        assert_eq!(result.files[0].items[0].tag, "TODO");
    }

    #[test]
    fn test_scan_result_deserialization_with_author() {
        let json = r#"{
            "files": [
                {
                    "path": "test.rs",
                    "items": [
                        {
                            "tag": "TODO",
                            "message": "Test message",
                            "line": 1,
                            "column": 5,
                            "priority": "Medium",
                            "author": "bob"
                        }
                    ]
                }
            ],
            "summary": {
                "total_count": 1,
                "files_with_todos": 1,
                "files_scanned": 1,
                "tag_counts": {"TODO": 1}
            }
        }"#;

        let result: ScanResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.files[0].items[0].author, Some("bob".to_string()));
    }
}
