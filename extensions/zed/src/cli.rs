use crate::types::ScanResult;
use zed_extension_api::{self as zed, Worktree};

/// Error type for CLI operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CliError {
    /// The todo-tree CLI binary was not found in PATH
    NotFound,
    /// Failed to execute the CLI command
    ExecutionFailed(String),
    /// Failed to parse the CLI output
    ParseFailed(String),
    /// No worktree available
    NoWorktree,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::NotFound => {
                write!(
                    f,
                    "todo-tree CLI not found. Please install it with: cargo install todo-tree"
                )
            }
            CliError::ExecutionFailed(msg) => write!(f, "Failed to run todo-tree: {}", msg),
            CliError::ParseFailed(msg) => write!(f, "Failed to parse todo-tree output: {}", msg),
            CliError::NoWorktree => write!(f, "No worktree available"),
        }
    }
}

impl From<CliError> for String {
    fn from(error: CliError) -> Self {
        error.to_string()
    }
}

/// Build the command arguments for a scan operation.
pub fn build_scan_args(
    cli_path: &str,
    root_path: &str,
    filter_tags: &[String],
) -> (String, Vec<String>) {
    let mut args = vec![
        "scan".to_string(),
        "--json".to_string(),
        root_path.to_string(),
    ];

    if !filter_tags.is_empty() {
        args.push("--tags".to_string());
        args.push(filter_tags.join(","));
    }

    (cli_path.to_string(), args)
}

/// Parse the JSON output from the CLI.
pub fn parse_output(json_output: &str) -> Result<ScanResult, CliError> {
    zed::serde_json::from_str(json_output).map_err(|e| CliError::ParseFailed(e.to_string()))
}

/// Process the command output and return parsed results.
pub fn process_command_output(
    status: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<ScanResult, CliError> {
    // Check for successful execution
    if status.is_none_or(|s| s != 0) {
        let stderr_str = String::from_utf8_lossy(stderr);
        return Err(CliError::ExecutionFailed(stderr_str.to_string()));
    }

    let stdout_str = String::from_utf8_lossy(stdout);
    parse_output(&stdout_str)
}

/// Runner for the todo-tree CLI.
///
/// Encapsulates all interaction with the external CLI binary,
/// including finding the binary, building commands, and parsing output.
pub struct CliRunner;

impl CliRunner {
    /// Find the path to the todo-tree CLI binary.
    ///
    /// Searches for either `tt` or `todo-tree` in the system PATH.
    pub fn find_cli_path(worktree: &Worktree) -> Result<String, CliError> {
        if let Some(path) = worktree.which("tt") {
            return Ok(path);
        }
        if let Some(path) = worktree.which("todo-tree") {
            return Ok(path);
        }
        Err(CliError::NotFound)
    }

    /// Run a scan command and return the parsed results.
    pub fn scan(worktree: &Worktree, filter_tags: &[String]) -> Result<ScanResult, CliError> {
        let cli_path = Self::find_cli_path(worktree)?;
        let root_path = worktree.root_path();

        let (cmd_path, args) = build_scan_args(&cli_path, &root_path, filter_tags);

        let mut cmd = zed::process::Command::new(&cmd_path);
        for arg in &args {
            cmd = cmd.arg(arg);
        }

        let output = cmd
            .output()
            .map_err(|e| CliError::ExecutionFailed(e.to_string()))?;

        process_command_output(output.status, &output.stdout, &output.stderr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_error_display_not_found() {
        let error = CliError::NotFound;
        let display = error.to_string();
        assert!(display.contains("todo-tree CLI not found"));
        assert!(display.contains("cargo install todo-tree"));
    }

    #[test]
    fn test_cli_error_display_execution_failed() {
        let error = CliError::ExecutionFailed("command failed".to_string());
        let display = error.to_string();
        assert!(display.contains("Failed to run todo-tree"));
        assert!(display.contains("command failed"));
    }

    #[test]
    fn test_cli_error_display_execution_failed_empty_message() {
        let error = CliError::ExecutionFailed(String::new());
        let display = error.to_string();
        assert_eq!(display, "Failed to run todo-tree: ");
    }

    #[test]
    fn test_cli_error_display_parse_failed() {
        let error = CliError::ParseFailed("invalid json".to_string());
        let display = error.to_string();
        assert!(display.contains("Failed to parse todo-tree output"));
        assert!(display.contains("invalid json"));
    }

    #[test]
    fn test_cli_error_display_parse_failed_empty_message() {
        let error = CliError::ParseFailed(String::new());
        let display = error.to_string();
        assert_eq!(display, "Failed to parse todo-tree output: ");
    }

    #[test]
    fn test_cli_error_display_no_worktree() {
        let error = CliError::NoWorktree;
        let display = error.to_string();
        assert_eq!(display, "No worktree available");
    }

    #[test]
    fn test_cli_error_into_string_not_found() {
        let error = CliError::NotFound;
        let s: String = error.into();
        assert!(s.contains("todo-tree CLI not found"));
    }

    #[test]
    fn test_cli_error_into_string_execution_failed() {
        let error = CliError::ExecutionFailed("test error".to_string());
        let s: String = error.into();
        assert!(s.contains("test error"));
    }

    #[test]
    fn test_cli_error_into_string_parse_failed() {
        let error = CliError::ParseFailed("parse error".to_string());
        let s: String = error.into();
        assert!(s.contains("parse error"));
    }

    #[test]
    fn test_cli_error_into_string_no_worktree() {
        let error = CliError::NoWorktree;
        let s: String = error.into();
        assert_eq!(s, "No worktree available");
    }

    #[test]
    fn test_cli_error_equality_not_found() {
        assert_eq!(CliError::NotFound, CliError::NotFound);
    }

    #[test]
    fn test_cli_error_equality_execution_failed() {
        assert_eq!(
            CliError::ExecutionFailed("test".to_string()),
            CliError::ExecutionFailed("test".to_string())
        );
        assert_ne!(
            CliError::ExecutionFailed("test1".to_string()),
            CliError::ExecutionFailed("test2".to_string())
        );
    }

    #[test]
    fn test_cli_error_equality_parse_failed() {
        assert_eq!(
            CliError::ParseFailed("test".to_string()),
            CliError::ParseFailed("test".to_string())
        );
    }

    #[test]
    fn test_cli_error_inequality_different_variants() {
        assert_ne!(CliError::NotFound, CliError::NoWorktree);
        assert_ne!(
            CliError::ExecutionFailed("test".to_string()),
            CliError::ParseFailed("test".to_string())
        );
    }

    #[test]
    fn test_build_scan_args_no_filter() {
        let (cmd, args) = build_scan_args("/usr/bin/tt", "/project", &[]);
        assert_eq!(cmd, "/usr/bin/tt");
        assert_eq!(args, vec!["scan", "--json", "/project"]);
    }

    #[test]
    fn test_build_scan_args_single_filter() {
        let (cmd, args) = build_scan_args("/usr/bin/tt", "/project", &["TODO".to_string()]);
        assert_eq!(cmd, "/usr/bin/tt");
        assert_eq!(args, vec!["scan", "--json", "/project", "--tags", "TODO"]);
    }

    #[test]
    fn test_build_scan_args_multiple_filters() {
        let (cmd, args) = build_scan_args(
            "/usr/bin/tt",
            "/project",
            &["TODO".to_string(), "FIXME".to_string(), "BUG".to_string()],
        );
        assert_eq!(cmd, "/usr/bin/tt");
        assert_eq!(
            args,
            vec!["scan", "--json", "/project", "--tags", "TODO,FIXME,BUG"]
        );
    }

    #[test]
    fn test_build_scan_args_different_paths() {
        let (cmd, args) = build_scan_args("todo-tree", "/home/user/code", &[]);
        assert_eq!(cmd, "todo-tree");
        assert_eq!(args, vec!["scan", "--json", "/home/user/code"]);
    }

    #[test]
    fn test_build_scan_args_path_with_spaces() {
        let (cmd, args) = build_scan_args("/usr/bin/tt", "/path/with spaces/project", &[]);
        assert_eq!(cmd, "/usr/bin/tt");
        assert_eq!(args, vec!["scan", "--json", "/path/with spaces/project"]);
    }

    #[test]
    fn test_parse_output_valid_empty_result() {
        let json = r#"{
            "files": [],
            "summary": {
                "total_count": 0,
                "files_with_todos": 0,
                "files_scanned": 0,
                "tag_counts": {}
            }
        }"#;

        let result = parse_output(json);
        assert!(result.is_ok());
        let scan_result = result.unwrap();
        assert!(scan_result.files.is_empty());
        assert_eq!(scan_result.summary.total_count, 0);
    }

    #[test]
    fn test_parse_output_with_single_file() {
        let json = r#"{
            "files": [
                {
                    "path": "test.rs",
                    "items": [
                        {
                            "tag": "TODO",
                            "message": "Test message",
                            "line": 10,
                            "column": 5,
                            "priority": "Medium"
                        }
                    ]
                }
            ],
            "summary": {
                "total_count": 1,
                "files_with_todos": 1,
                "files_scanned": 5,
                "tag_counts": {"TODO": 1}
            }
        }"#;

        let result = parse_output(json).unwrap();
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].path, "test.rs");
        assert_eq!(result.files[0].items.len(), 1);
        assert_eq!(result.files[0].items[0].tag, "TODO");
        assert_eq!(result.files[0].items[0].message, "Test message");
        assert_eq!(result.files[0].items[0].line, 10);
    }

    #[test]
    fn test_parse_output_with_multiple_files() {
        let json = r#"{
            "files": [
                {
                    "path": "src/main.rs",
                    "items": [
                        {"tag": "TODO", "message": "First", "line": 1, "column": 1, "priority": "Medium"}
                    ]
                },
                {
                    "path": "src/lib.rs",
                    "items": [
                        {"tag": "FIXME", "message": "Second", "line": 5, "column": 3, "priority": "Critical"}
                    ]
                }
            ],
            "summary": {
                "total_count": 2,
                "files_with_todos": 2,
                "files_scanned": 10,
                "tag_counts": {"TODO": 1, "FIXME": 1}
            }
        }"#;

        let result = parse_output(json).unwrap();
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.summary.total_count, 2);
    }

    #[test]
    fn test_parse_output_with_author() {
        let json = r#"{
            "files": [
                {
                    "path": "test.rs",
                    "items": [
                        {
                            "tag": "TODO",
                            "message": "Fix this",
                            "line": 1,
                            "column": 1,
                            "priority": "Medium",
                            "author": "alice"
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

        let result = parse_output(json).unwrap();
        assert_eq!(result.files[0].items[0].author, Some("alice".to_string()));
    }

    #[test]
    fn test_parse_output_invalid_json() {
        let json = "not valid json";
        let result = parse_output(json);
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_parse_output_empty_string() {
        let result = parse_output("");
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_parse_output_missing_files_field() {
        let json = r#"{"summary": {"total_count": 0, "files_with_todos": 0, "files_scanned": 0, "tag_counts": {}}}"#;
        let result = parse_output(json);
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_parse_output_missing_summary_field() {
        let json = r#"{"files": []}"#;
        let result = parse_output(json);
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_parse_output_null_value() {
        let json = "null";
        let result = parse_output(json);
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_parse_output_array_instead_of_object() {
        let json = "[]";
        let result = parse_output(json);
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_process_command_output_success() {
        let json = r#"{
            "files": [],
            "summary": {
                "total_count": 0,
                "files_with_todos": 0,
                "files_scanned": 0,
                "tag_counts": {}
            }
        }"#;

        let result = process_command_output(Some(0), json.as_bytes(), b"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_command_output_non_zero_exit() {
        let result = process_command_output(Some(1), b"", b"Error occurred");
        assert!(matches!(result, Err(CliError::ExecutionFailed(_))));
        if let Err(CliError::ExecutionFailed(msg)) = result {
            assert_eq!(msg, "Error occurred");
        }
    }

    #[test]
    fn test_process_command_output_none_status() {
        let result = process_command_output(None, b"", b"Process killed");
        assert!(matches!(result, Err(CliError::ExecutionFailed(_))));
        if let Err(CliError::ExecutionFailed(msg)) = result {
            assert_eq!(msg, "Process killed");
        }
    }

    #[test]
    fn test_process_command_output_empty_stderr_on_failure() {
        let result = process_command_output(Some(1), b"", b"");
        assert!(matches!(result, Err(CliError::ExecutionFailed(_))));
        if let Err(CliError::ExecutionFailed(msg)) = result {
            assert_eq!(msg, "");
        }
    }

    #[test]
    fn test_process_command_output_invalid_json_with_success_status() {
        let result = process_command_output(Some(0), b"not json", b"");
        assert!(matches!(result, Err(CliError::ParseFailed(_))));
    }

    #[test]
    fn test_process_command_output_with_valid_data() {
        let json = r#"{
            "files": [
                {
                    "path": "test.rs",
                    "items": [
                        {"tag": "TODO", "message": "Test", "line": 1, "column": 1, "priority": "Medium"}
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

        let result = process_command_output(Some(0), json.as_bytes(), b"").unwrap();
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.summary.total_count, 1);
    }

    #[test]
    fn test_process_command_output_utf8_stderr() {
        let result = process_command_output(Some(1), b"", "Error: fichier non trouvé".as_bytes());
        if let Err(CliError::ExecutionFailed(msg)) = result {
            assert!(msg.contains("fichier"));
        } else {
            panic!("Expected ExecutionFailed error");
        }
    }

    #[test]
    fn test_process_command_output_ignores_stderr_on_success() {
        let json = r#"{
            "files": [],
            "summary": {
                "total_count": 0,
                "files_with_todos": 0,
                "files_scanned": 0,
                "tag_counts": {}
            }
        }"#;

        // Even with stderr content, if status is 0, it should succeed
        let result = process_command_output(Some(0), json.as_bytes(), b"Some warning");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_error_clone() {
        let error = CliError::ExecutionFailed("test".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_cli_error_debug_not_found() {
        let error = CliError::NotFound;
        let debug = format!("{:?}", error);
        assert_eq!(debug, "NotFound");
    }

    #[test]
    fn test_cli_error_debug_execution_failed() {
        let error = CliError::ExecutionFailed("msg".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("ExecutionFailed"));
        assert!(debug.contains("msg"));
    }
}
