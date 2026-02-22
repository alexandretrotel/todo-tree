use super::options::PrintOptions;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Write};
use todo_tree_core::ScanResult;

#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub files: Vec<JsonFileEntry>,
    pub summary: JsonSummary,
}

#[derive(Debug, Serialize)]
pub struct JsonFileEntry {
    pub path: String,
    pub items: Vec<JsonTodoItem>,
}

#[derive(Debug, Serialize)]
pub struct JsonTodoItem {
    pub tag: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub priority: String,
}

#[derive(Debug, Serialize)]
pub struct JsonSummary {
    pub total_count: usize,
    pub files_with_todos: usize,
    pub files_scanned: usize,
    pub tag_counts: HashMap<String, usize>,
}

pub fn print_json<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let json_result = JsonOutput::from_scan_result(result, options);
    let json_str = serde_json::to_string_pretty(&json_result).map_err(io::Error::other)?;
    writeln!(writer, "{}", json_str)?;
    Ok(())
}

impl JsonOutput {
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
            total_count: result.summary.total_count,
            files_with_todos: result.summary.files_with_todos,
            files_scanned: result.summary.files_scanned,
            tag_counts: result.summary.tag_counts.clone(),
        };

        Self { files, summary }
    }
}
