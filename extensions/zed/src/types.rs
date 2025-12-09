pub use todo_tree_core::tags::DEFAULT_TAGS;
pub use todo_tree_core::{FileResult, Priority, ScanResult};

#[cfg(test)]
mod tests {
    use todo_tree_core::{Summary, TodoItem};

    use super::*;
    use std::collections::HashMap;

    fn create_test_scan_result() -> ScanResult {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("TODO".to_string(), 2);
        tag_counts.insert("FIXME".to_string(), 1);

        ScanResult::from_json(
            vec![
                FileResult {
                    path: "src/main.rs".to_string(),
                    items: vec![
                        TodoItem {
                            tag: "TODO".to_string(),
                            message: "Implement this".to_string(),
                            line: 10,
                            column: 5,
                            line_content: None,
                            author: None,
                            priority: Priority::Medium,
                        },
                        TodoItem {
                            tag: "FIXME".to_string(),
                            message: "Fix this bug".to_string(),
                            line: 25,
                            column: 3,
                            line_content: None,
                            author: Some("alice".to_string()),
                            priority: Priority::Critical,
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
                        line_content: None,
                        author: None,
                        priority: Priority::Medium,
                    }],
                },
            ],
            Summary {
                total_count: 3,
                files_with_todos: 2,
                files_scanned: 10,
                tag_counts,
            },
        )
    }

    fn create_empty_scan_result() -> ScanResult {
        ScanResult::from_json(
            vec![],
            Summary {
                total_count: 0,
                files_with_todos: 0,
                files_scanned: 5,
                tag_counts: HashMap::new(),
            },
        )
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
        let files = result.get_files();
        assert_eq!(files[0].items.len(), 2);
        assert_eq!(files[1].items.len(), 1);
    }

    #[test]
    fn test_todo_item_format_author_with_author() {
        let item = TodoItem {
            tag: "TODO".to_string(),
            message: "Test".to_string(),
            line: 1,
            column: 1,
            line_content: None,
            author: Some("alice".to_string()),
            priority: Priority::Medium,
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
            line_content: None,
            author: None,
            priority: Priority::Medium,
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
        assert_eq!(DEFAULT_TAGS.len(), 17);
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
        let files = result.get_files();
        assert_eq!(files.len(), 1);
        assert_eq!(result.summary.total_count, 1);
        assert_eq!(files[0].items[0].tag, "TODO");
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
        let files = result.get_files();
        assert_eq!(files[0].items[0].author, Some("bob".to_string()));
    }
}
