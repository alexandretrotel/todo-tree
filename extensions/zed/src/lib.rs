mod cli;
mod formatter;
mod slash_commands;
mod types;

use slash_commands::SlashCommandHandler;
use zed_extension_api::{self as zed, SlashCommand, Worktree};

/// The main extension struct that holds state and implements the Extension trait.
///
/// This struct serves as the entry point for all extension functionality,
/// delegating to specialized handlers for each feature.
struct TodoTreeExtension;

impl zed::Extension for TodoTreeExtension {
    /// Called when the extension is first loaded.
    fn new() -> Self
    where
        Self: Sized,
    {
        TodoTreeExtension
    }

    /// Execute a slash command.
    ///
    /// Delegates to `SlashCommandHandler` for processing.
    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        SlashCommandHandler::execute(&command, args, worktree)
    }

    /// Provide argument completions for slash commands.
    ///
    /// Delegates to `SlashCommandHandler` for completions.
    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        SlashCommandHandler::complete_arguments(&command, &args)
    }
}

// Register the extension with Zed
zed::register_extension!(TodoTreeExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{CliError, build_scan_args, parse_output, process_command_output};
    use crate::formatter::OutputFormatter;
    use crate::slash_commands::{build_stats_output, build_tag_completions, build_todos_output};
    use crate::types::{DEFAULT_TAGS, FileResult, Priority, ScanResult, Summary, TodoItem};
    use std::collections::HashMap;

    #[test]
    fn test_extension_struct_is_send_sync() {
        // Verify TodoTreeExtension can be used across threads
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TodoTreeExtension>();
    }

    #[test]
    fn test_types_integration_scan_result_to_formatter() {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("TODO".to_string(), 1);

        let result = ScanResult {
            files: vec![FileResult {
                path: "test.rs".to_string(),
                items: vec![TodoItem {
                    tag: "TODO".to_string(),
                    message: "Test integration".to_string(),
                    line: 1,
                    column: 1,
                    priority: "Medium".to_string(),
                    author: None,
                }],
            }],
            summary: Summary {
                total_count: 1,
                files_with_todos: 1,
                files_scanned: 1,
                tag_counts,
            },
        };

        let formatted = OutputFormatter::format_todos(&result);
        assert!(formatted.contains("test.rs"));
        assert!(formatted.contains("Test integration"));
    }

    #[test]
    fn test_types_integration_priority_in_default_tags() {
        // Verify all default tags have valid priorities
        for tag in DEFAULT_TAGS {
            match tag.priority {
                Priority::Critical | Priority::High | Priority::Medium | Priority::Low => {}
            }
        }
    }

    #[test]
    fn test_cli_output_to_formatter() {
        let json = r#"{
            "files": [
                {
                    "path": "src/main.rs",
                    "items": [
                        {"tag": "TODO", "message": "Implement feature", "line": 10, "column": 5, "priority": "Medium"},
                        {"tag": "FIXME", "message": "Fix bug", "line": 20, "column": 3, "priority": "Critical"}
                    ]
                }
            ],
            "summary": {
                "total_count": 2,
                "files_with_todos": 1,
                "files_scanned": 5,
                "tag_counts": {"TODO": 1, "FIXME": 1}
            }
        }"#;

        let result = parse_output(json).unwrap();
        let formatted = OutputFormatter::format_todos(&result);

        assert!(formatted.contains("src/main.rs"));
        assert!(formatted.contains("Implement feature"));
        assert!(formatted.contains("Fix bug"));
        assert!(formatted.contains("TODO"));
        assert!(formatted.contains("FIXME"));
    }

    #[test]
    fn test_cli_error_to_string_conversion() {
        let error = CliError::NotFound;
        let error_string: String = error.into();
        assert!(!error_string.is_empty());
    }

    #[test]
    fn test_slash_commands_build_output_integration() {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("BUG".to_string(), 3);

        let result = ScanResult {
            files: vec![FileResult {
                path: "buggy.rs".to_string(),
                items: vec![
                    TodoItem {
                        tag: "BUG".to_string(),
                        message: "Critical bug".to_string(),
                        line: 1,
                        column: 1,
                        priority: "Critical".to_string(),
                        author: Some("developer".to_string()),
                    },
                    TodoItem {
                        tag: "BUG".to_string(),
                        message: "Another bug".to_string(),
                        line: 10,
                        column: 1,
                        priority: "Critical".to_string(),
                        author: None,
                    },
                    TodoItem {
                        tag: "BUG".to_string(),
                        message: "Third bug".to_string(),
                        line: 20,
                        column: 1,
                        priority: "Critical".to_string(),
                        author: None,
                    },
                ],
            }],
            summary: Summary {
                total_count: 3,
                files_with_todos: 1,
                files_scanned: 10,
                tag_counts,
            },
        };

        let todos_output = build_todos_output(&result, &["BUG".to_string()]);
        assert!(todos_output.text.contains("Critical bug"));
        assert!(todos_output.text.contains("(developer)"));
        assert!(todos_output.sections[0].label.contains("BUG"));

        let stats_output = build_stats_output(&result);
        assert!(stats_output.text.contains("3"));
    }

    #[test]
    fn test_tag_completions_match_default_tags() {
        let completions = build_tag_completions();

        assert_eq!(completions.len(), DEFAULT_TAGS.len());

        for completion in &completions {
            let found = DEFAULT_TAGS.iter().any(|t| t.name == completion.new_text);
            assert!(
                found,
                "Completion '{}' not found in DEFAULT_TAGS",
                completion.new_text
            );
        }
    }

    #[test]
    fn test_full_pipeline_empty_project() {
        let json = r#"{
            "files": [],
            "summary": {
                "total_count": 0,
                "files_with_todos": 0,
                "files_scanned": 100,
                "tag_counts": {}
            }
        }"#;

        let result = parse_output(json).unwrap();
        let output = build_todos_output(&result, &[]);

        assert!(output.text.contains("No TODO items found"));
        assert!(output.sections[0].label.contains("0 items"));
    }

    #[test]
    fn test_full_pipeline_large_project() {
        let mut files = Vec::new();
        let mut tag_counts = HashMap::new();

        for i in 0..10 {
            let mut items = Vec::new();
            for j in 0..5 {
                items.push(TodoItem {
                    tag: "TODO".to_string(),
                    message: format!("Item {} in file {}", j, i),
                    line: j + 1,
                    column: 1,
                    priority: "Medium".to_string(),
                    author: None,
                });
            }
            files.push(FileResult {
                path: format!("src/file{}.rs", i),
                items,
            });
        }

        tag_counts.insert("TODO".to_string(), 50);

        let result = ScanResult {
            files,
            summary: Summary {
                total_count: 50,
                files_with_todos: 10,
                files_scanned: 100,
                tag_counts,
            },
        };

        let output = build_todos_output(&result, &[]);
        assert!(output.text.contains("50 items"));
        assert!(output.text.contains("10 files"));

        let stats = build_stats_output(&result);
        assert!(stats.text.contains("50"));
        assert!(stats.text.contains("5.00")); // avg items per file
    }

    #[test]
    fn test_build_scan_args_to_process_output() {
        let (cmd, args) = build_scan_args("/usr/bin/tt", "/project", &["TODO".to_string()]);

        assert_eq!(cmd, "/usr/bin/tt");
        assert!(args.contains(&"scan".to_string()));
        assert!(args.contains(&"--json".to_string()));
        assert!(args.contains(&"--tags".to_string()));
        assert!(args.contains(&"TODO".to_string()));

        // Simulate successful command output
        let json = r#"{"files": [], "summary": {"total_count": 0, "files_with_todos": 0, "files_scanned": 0, "tag_counts": {}}}"#;
        let result = process_command_output(Some(0), json.as_bytes(), b"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling_invalid_json_to_output() {
        let result = process_command_output(Some(0), b"invalid json", b"");
        assert!(result.is_err());

        if let Err(e) = result {
            let error_string: String = e.into();
            assert!(error_string.contains("parse"));
        }
    }

    #[test]
    fn test_error_handling_command_failure() {
        let result = process_command_output(Some(1), b"", b"Command failed: not found");
        assert!(result.is_err());

        if let Err(e) = result {
            let error_string: String = e.into();
            assert!(error_string.contains("Command failed"));
        }
    }

    #[test]
    fn test_tag_definition_structure() {
        for tag in DEFAULT_TAGS {
            assert!(!tag.name.is_empty());
            assert!(!tag.description.is_empty());
            // Priority is always valid since it's an enum
        }
    }

    #[test]
    fn test_priority_ordering_is_consistent() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
        assert!(Priority::Critical > Priority::Low);
    }

    #[test]
    fn test_scan_result_is_empty_consistency() {
        let empty = ScanResult {
            files: vec![],
            summary: Summary {
                total_count: 0,
                files_with_todos: 0,
                files_scanned: 0,
                tag_counts: HashMap::new(),
            },
        };
        assert!(empty.is_empty());

        let non_empty = ScanResult {
            files: vec![FileResult {
                path: "test.rs".to_string(),
                items: vec![],
            }],
            summary: Summary {
                total_count: 0,
                files_with_todos: 1,
                files_scanned: 1,
                tag_counts: HashMap::new(),
            },
        };
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_output_section_range_always_valid() {
        let test_cases = vec![
            (
                ScanResult {
                    files: vec![],
                    summary: Summary {
                        total_count: 0,
                        files_with_todos: 0,
                        files_scanned: 0,
                        tag_counts: HashMap::new(),
                    },
                },
                vec![],
            ),
            (
                ScanResult {
                    files: vec![FileResult {
                        path: "a".to_string(),
                        items: vec![TodoItem {
                            tag: "X".to_string(),
                            message: "Y".to_string(),
                            line: 1,
                            column: 1,
                            priority: "Low".to_string(),
                            author: None,
                        }],
                    }],
                    summary: Summary {
                        total_count: 1,
                        files_with_todos: 1,
                        files_scanned: 1,
                        tag_counts: HashMap::new(),
                    },
                },
                vec![],
            ),
        ];

        for (result, filter) in test_cases {
            let output = build_todos_output(&result, &filter);
            let range_end = output.sections[0].range.clone().end as usize;
            assert_eq!(
                range_end,
                output.text.len(),
                "Section range doesn't cover full text"
            );

            let stats = build_stats_output(&result);
            let stats_range_end = stats.sections[0].range.clone().end as usize;
            assert_eq!(
                stats_range_end,
                stats.text.len(),
                "Stats section range doesn't cover full text"
            );
        }
    }
}
