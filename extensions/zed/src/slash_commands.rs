use crate::cli::{CliError, CliRunner};
use crate::formatter::OutputFormatter;
use crate::types::{DEFAULT_TAGS, ScanResult};
use zed_extension_api::{
    self as zed, SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

/// Handler for slash commands.
///
/// Provides methods for executing each slash command and
/// generating argument completions.
pub struct SlashCommandHandler;

impl SlashCommandHandler {
    /// Execute a slash command by name.
    ///
    /// Routes to the appropriate handler based on command name.
    pub fn execute(
        command: &SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "todos" => Self::run_todos(args, worktree),
            "todos-stats" => Self::run_todos_stats(worktree),
            cmd => Err(format!("Unknown command: {cmd}")),
        }
    }

    /// Get argument completions for a slash command.
    ///
    /// Returns appropriate completions based on the command.
    pub fn complete_arguments(
        command: &SlashCommand,
        _args: &[String],
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "todos" => Ok(Self::get_tag_completions()),
            "todos-stats" => Ok(vec![]),
            _ => Ok(vec![]),
        }
    }

    /// Execute the /todos command.
    ///
    /// Lists all TODO items in the project, optionally filtered by tags.
    fn run_todos(
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let worktree = worktree.ok_or_else(|| CliError::NoWorktree.to_string())?;

        let result = CliRunner::scan(worktree, &args)?;

        Ok(build_todos_output(&result, &args))
    }

    /// Execute the /todos-stats command.
    ///
    /// Shows statistics about TODO items in the project.
    fn run_todos_stats(worktree: Option<&Worktree>) -> Result<SlashCommandOutput, String> {
        let worktree = worktree.ok_or_else(|| CliError::NoWorktree.to_string())?;

        let result = CliRunner::scan(worktree, &[])?;

        Ok(build_stats_output(&result))
    }

    /// Get tag completions for the /todos command.
    ///
    /// Returns completions for all default tags with descriptions.
    fn get_tag_completions() -> Vec<SlashCommandArgumentCompletion> {
        build_tag_completions()
    }
}

/// Build the output for the /todos command.
pub fn build_todos_output(result: &ScanResult, filter_tags: &[String]) -> SlashCommandOutput {
    let text = OutputFormatter::format_todos(result);
    let label = OutputFormatter::todos_label(result, filter_tags);

    SlashCommandOutput {
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label,
        }],
        text,
    }
}

/// Build the output for the /todos-stats command.
pub fn build_stats_output(result: &ScanResult) -> SlashCommandOutput {
    let text = OutputFormatter::format_stats(result);
    let label = OutputFormatter::stats_label(result);

    SlashCommandOutput {
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label,
        }],
        text,
    }
}

/// Build tag completions for the /todos command.
pub fn build_tag_completions() -> Vec<SlashCommandArgumentCompletion> {
    DEFAULT_TAGS
        .iter()
        .map(|tag| zed::SlashCommandArgumentCompletion {
            label: format!("{} - {}", tag.name, tag.description),
            new_text: tag.name.to_string(),
            run_command: true,
        })
        .collect()
}

/// Create a helper function to build a SlashCommand for testing.
#[cfg(test)]
fn make_test_command(name: &str) -> SlashCommand {
    SlashCommand {
        name: name.to_string(),
        description: format!("Test {}", name),
        tooltip_text: format!("Test tooltip for {}", name),
        requires_argument: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileResult;
    use std::collections::HashMap;
    use todo_tree_core::{Priority, Summary, TodoItem};

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

    fn create_populated_scan_result() -> ScanResult {
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
                            priority: Priority::Medium,
                            author: None,
                        },
                        TodoItem {
                            tag: "FIXME".to_string(),
                            message: "Fix this bug".to_string(),
                            line: 25,
                            column: 3,
                            line_content: None,
                            priority: Priority::Critical,
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
                        line_content: None,
                        priority: Priority::Medium,
                        author: None,
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

    #[test]
    fn test_build_tag_completions_returns_all_tags() {
        let completions = build_tag_completions();
        assert_eq!(completions.len(), DEFAULT_TAGS.len());
    }

    #[test]
    fn test_build_tag_completions_format() {
        let completions = build_tag_completions();

        let todo_completion = completions.iter().find(|c| c.new_text == "TODO");
        assert!(todo_completion.is_some());

        let todo = todo_completion.unwrap();
        assert!(todo.label.contains("TODO"));
        assert!(todo.label.contains(" - "));
        assert!(todo.label.contains("General TODO items"));
        assert!(todo.run_command);
    }

    #[test]
    fn test_build_tag_completions_all_run_command() {
        let completions = build_tag_completions();
        assert!(completions.iter().all(|c| c.run_command));
    }

    #[test]
    fn test_build_tag_completions_all_have_labels() {
        let completions = build_tag_completions();
        assert!(completions.iter().all(|c| !c.label.is_empty()));
    }

    #[test]
    fn test_build_tag_completions_all_have_new_text() {
        let completions = build_tag_completions();
        assert!(completions.iter().all(|c| !c.new_text.is_empty()));
    }

    #[test]
    fn test_build_tag_completions_contains_critical_tags() {
        let completions = build_tag_completions();
        let critical_tags = ["BUG", "FIXME", "XXX"];
        for tag in critical_tags {
            assert!(
                completions.iter().any(|c| c.new_text == tag),
                "Missing critical tag: {}",
                tag
            );
        }
    }

    #[test]
    fn test_build_tag_completions_contains_all_default_tags() {
        let completions = build_tag_completions();
        for tag_def in DEFAULT_TAGS {
            assert!(
                completions.iter().any(|c| c.new_text == tag_def.name),
                "Missing tag: {}",
                tag_def.name
            );
        }
    }

    #[test]
    fn test_build_todos_output_empty_result() {
        let result = create_empty_scan_result();
        let output = build_todos_output(&result, &[]);

        assert!(output.text.contains("No TODO items found"));
        assert_eq!(output.sections.len(), 1);
        assert!(output.sections[0].label.contains("0 items"));
    }

    #[test]
    fn test_build_todos_output_with_items() {
        let result = create_populated_scan_result();
        let output = build_todos_output(&result, &[]);

        assert!(output.text.contains("# TODO Items"));
        assert!(output.text.contains("src/main.rs"));
        assert!(output.text.contains("Implement this"));
        assert_eq!(output.sections.len(), 1);
    }

    #[test]
    fn test_build_todos_output_label_no_filter() {
        let result = create_populated_scan_result();
        let output = build_todos_output(&result, &[]);

        assert!(output.sections[0].label.contains("3 items"));
        assert!(output.sections[0].label.contains("2 files"));
    }

    #[test]
    fn test_build_todos_output_label_with_filter() {
        let result = create_populated_scan_result();
        let output = build_todos_output(&result, &["BUG".to_string(), "FIXME".to_string()]);

        assert!(output.sections[0].label.contains("BUG"));
        assert!(output.sections[0].label.contains("FIXME"));
    }

    #[test]
    fn test_build_todos_output_section_range_covers_text() {
        let result = create_populated_scan_result();
        let output = build_todos_output(&result, &[]);

        let range_end = output.sections[0].range.end as usize;
        assert_eq!(range_end, output.text.len());
    }

    #[test]
    fn test_build_todos_output_with_single_filter() {
        let result = create_populated_scan_result();
        let output = build_todos_output(&result, &["TODO".to_string()]);

        assert!(output.sections[0].label.contains("TODO"));
    }

    #[test]
    fn test_build_stats_output_empty_result() {
        let result = create_empty_scan_result();
        let output = build_stats_output(&result);

        assert!(output.text.contains("# TODO Statistics"));
        assert!(output.text.contains("Total items | 0"));
        assert_eq!(output.sections.len(), 1);
    }

    #[test]
    fn test_build_stats_output_with_items() {
        let result = create_populated_scan_result();
        let output = build_stats_output(&result);

        assert!(output.text.contains("# TODO Statistics"));
        assert!(output.text.contains("Total items | 3"));
        assert!(output.text.contains("Files with TODOs | 2"));
    }

    #[test]
    fn test_build_stats_output_label() {
        let result = create_populated_scan_result();
        let output = build_stats_output(&result);

        assert!(output.sections[0].label.contains("3 total"));
    }

    #[test]
    fn test_build_stats_output_section_range_covers_text() {
        let result = create_populated_scan_result();
        let output = build_stats_output(&result);

        let range_end = output.sections[0].range.end as usize;
        assert_eq!(range_end, output.text.len());
    }

    #[test]
    fn test_build_stats_output_contains_priority_info() {
        let result = create_populated_scan_result();
        let output = build_stats_output(&result);

        assert!(output.text.contains("By Priority"));
        assert!(output.text.contains("Critical"));
    }

    #[test]
    fn test_get_tag_completions_returns_all_tags() {
        let completions = SlashCommandHandler::get_tag_completions();
        assert_eq!(completions.len(), DEFAULT_TAGS.len());
    }

    #[test]
    fn test_get_tag_completions_format() {
        let completions = SlashCommandHandler::get_tag_completions();

        let todo_completion = completions.iter().find(|c| c.new_text == "TODO");
        assert!(todo_completion.is_some());

        let todo = todo_completion.unwrap();
        assert!(todo.label.contains("TODO"));
        assert!(todo.label.contains(" - "));
        assert!(todo.run_command);
    }

    #[test]
    fn test_get_tag_completions_all_run_command() {
        let completions = SlashCommandHandler::get_tag_completions();
        assert!(completions.iter().all(|c| c.run_command));
    }

    #[test]
    fn test_complete_arguments_todos() {
        let command = make_test_command("todos");

        let result = SlashCommandHandler::complete_arguments(&command, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), DEFAULT_TAGS.len());
    }

    #[test]
    fn test_complete_arguments_todos_with_existing_args() {
        let command = make_test_command("todos");

        let result = SlashCommandHandler::complete_arguments(&command, &["BUG".to_string()]);
        assert!(result.is_ok());
        // Should still return all completions (args are ignored currently)
        assert_eq!(result.unwrap().len(), DEFAULT_TAGS.len());
    }

    #[test]
    fn test_complete_arguments_todos_stats() {
        let command = make_test_command("todos-stats");

        let result = SlashCommandHandler::complete_arguments(&command, &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_complete_arguments_unknown_command() {
        let command = make_test_command("unknown");

        let result = SlashCommandHandler::complete_arguments(&command, &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_complete_arguments_empty_command_name() {
        let command = make_test_command("");

        let result = SlashCommandHandler::complete_arguments(&command, &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_run_todos_no_worktree() {
        let result = SlashCommandHandler::run_todos(vec![], None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("No worktree"));
    }

    #[test]
    fn test_run_todos_no_worktree_with_args() {
        let result = SlashCommandHandler::run_todos(vec!["TODO".to_string()], None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("No worktree"));
    }

    #[test]
    fn test_run_todos_stats_no_worktree() {
        let result = SlashCommandHandler::run_todos_stats(None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("No worktree"));
    }

    #[test]
    fn test_execute_todos_no_worktree() {
        let command = make_test_command("todos");
        let result = SlashCommandHandler::execute(&command, vec![], None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No worktree"));
    }

    #[test]
    fn test_execute_todos_stats_no_worktree() {
        let command = make_test_command("todos-stats");
        let result = SlashCommandHandler::execute(&command, vec![], None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No worktree"));
    }

    #[test]
    fn test_execute_unknown_command() {
        let command = make_test_command("unknown-command");
        let result = SlashCommandHandler::execute(&command, vec![], None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown command"));
        assert!(err.contains("unknown-command"));
    }

    #[test]
    fn test_execute_empty_command_name() {
        let command = make_test_command("");
        let result = SlashCommandHandler::execute(&command, vec![], None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }

    #[test]
    fn test_execute_case_sensitive_command() {
        let command = make_test_command("TODOS");
        let result = SlashCommandHandler::execute(&command, vec![], None);
        // Command names are case-sensitive
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }

    #[test]
    fn test_make_test_command_creates_correct_struct() {
        let command = make_test_command("test-cmd");
        assert_eq!(command.name, "test-cmd");
        assert!(command.description.contains("test-cmd"));
        assert!(command.tooltip_text.contains("test-cmd"));
        assert!(!command.requires_argument);
    }
}
