use crate::priority::Priority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a found TODO item in the source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    /// The tag that was matched (e.g., "TODO", "FIXME")
    pub tag: String,

    /// The message following the tag
    pub message: String,

    /// Line number where the tag was found (1-indexed)
    pub line: usize,

    /// Column number where the tag starts (1-indexed)
    pub column: usize,

    /// The full line content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_content: Option<String>,

    /// Optional author/assignee if specified (e.g., TODO(john): ...)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Priority level inferred from tag type
    pub priority: Priority,
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

/// A file containing TODO items (for JSON serialization)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileResult {
    /// Relative path to the file
    pub path: String,
    /// TODO items found in this file
    pub items: Vec<TodoItem>,
}

/// Summary statistics from a scan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Result of scanning a directory for TODO items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// List of files containing TODO items (for JSON output)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileResult>>,

    /// Map of file paths to their TODO items (for internal use)
    #[serde(skip)]
    pub files_map: HashMap<PathBuf, Vec<TodoItem>>,

    /// Summary statistics
    pub summary: Summary,

    /// Root directory that was scanned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
}

impl ScanResult {
    /// Create a new empty scan result
    pub fn new(root: PathBuf) -> Self {
        Self {
            files: None,
            files_map: HashMap::new(),
            summary: Summary {
                total_count: 0,
                files_with_todos: 0,
                files_scanned: 0,
                tag_counts: HashMap::new(),
            },
            root: Some(root),
        }
    }

    /// Create a scan result from JSON-style data
    pub fn from_json(files: Vec<FileResult>, summary: Summary) -> Self {
        Self {
            files: Some(files),
            files_map: HashMap::new(),
            summary,
            root: None,
        }
    }

    /// Check if the scan found any TODO items
    pub fn is_empty(&self) -> bool {
        if let Some(files) = &self.files {
            files.is_empty()
        } else {
            self.files_map.is_empty()
        }
    }

    /// Add TODO items for a file
    pub fn add_file(&mut self, path: PathBuf, items: Vec<TodoItem>) {
        self.summary.files_scanned += 1;

        if !items.is_empty() {
            self.summary.files_with_todos += 1;
            self.summary.total_count += items.len();

            for item in &items {
                *self.summary.tag_counts.entry(item.tag.clone()).or_insert(0) += 1;
            }

            self.files_map.insert(path, items);
        }
    }

    /// Get all TODO items as a flat list
    pub fn all_items(&self) -> Vec<(PathBuf, TodoItem)> {
        let mut items = Vec::new();
        for (path, file_items) in &self.files_map {
            for item in file_items {
                items.push((path.clone(), item.clone()));
            }
        }
        items
    }

    /// Get files sorted by path
    pub fn sorted_files(&self) -> Vec<(&PathBuf, &Vec<TodoItem>)> {
        let mut files: Vec<_> = self.files_map.iter().collect();
        files.sort_by(|a, b| a.0.cmp(b.0));
        files
    }

    /// Filter items by tag
    pub fn filter_by_tag(&self, tag: &str) -> ScanResult {
        let root = self.root.clone().unwrap_or_else(|| PathBuf::from("."));
        let mut result = ScanResult::new(root);
        result.summary.files_scanned = self.summary.files_scanned;

        for (path, items) in &self.files_map {
            let filtered: Vec<TodoItem> = items
                .iter()
                .filter(|item| item.tag.eq_ignore_ascii_case(tag))
                .cloned()
                .collect();

            if !filtered.is_empty() {
                result.add_file(path.clone(), filtered);
            }
        }

        result
    }

    /// Convert to JSON-friendly format with FileResult list
    pub fn to_json_format(&self) -> Self {
        let mut files: Vec<FileResult> = self
            .files_map
            .iter()
            .map(|(path, items)| FileResult {
                path: path.display().to_string(),
                items: items.clone(),
            })
            .collect();

        // Sort files by path for consistent output
        files.sort_by(|a, b| a.path.cmp(&b.path));

        Self {
            files: Some(files),
            files_map: HashMap::new(),
            summary: self.summary.clone(),
            root: None,
        }
    }

    /// Get files from either format
    pub fn get_files(&self) -> Vec<FileResult> {
        if let Some(files) = &self.files {
            files.clone()
        } else {
            self.to_json_format().files.unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_item(tag: &str, message: &str, line: usize) -> TodoItem {
        TodoItem {
            tag: tag.to_string(),
            message: message.to_string(),
            line,
            column: 1,
            line_content: Some(format!("// {}: {}", tag, message)),
            author: None,
            priority: Priority::from_tag(tag),
        }
    }

    #[test]
    fn test_todo_item_format_author() {
        let mut item = create_test_item("TODO", "Test", 1);
        assert_eq!(item.format_author(), "");

        item.author = Some("alice".to_string());
        assert_eq!(item.format_author(), "(alice)");
    }

    #[test]
    fn test_scan_result_new() {
        let root = PathBuf::from("/test");
        let result = ScanResult::new(root.clone());

        assert_eq!(result.root, Some(root));
        assert_eq!(result.summary.total_count, 0);
        assert_eq!(result.summary.files_scanned, 0);
        assert_eq!(result.summary.files_with_todos, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_result_add_file() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        let items = vec![create_test_item("TODO", "Test", 1)];

        result.add_file(PathBuf::from("test.rs"), items);

        assert_eq!(result.summary.total_count, 1);
        assert_eq!(result.summary.files_scanned, 1);
        assert_eq!(result.summary.files_with_todos, 1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_scan_result_add_empty_file() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(PathBuf::from("empty.rs"), vec![]);

        assert_eq!(result.summary.files_scanned, 1);
        assert_eq!(result.summary.files_with_todos, 0);
        assert_eq!(result.summary.total_count, 0);
    }

    #[test]
    fn test_scan_result_tag_counts() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        let items = vec![
            create_test_item("TODO", "First", 1),
            create_test_item("TODO", "Second", 2),
            create_test_item("FIXME", "Third", 3),
        ];

        result.add_file(PathBuf::from("test.rs"), items);

        assert_eq!(result.summary.tag_counts.get("TODO"), Some(&2));
        assert_eq!(result.summary.tag_counts.get("FIXME"), Some(&1));
    }

    #[test]
    fn test_scan_result_filter_by_tag() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        let items = vec![
            create_test_item("TODO", "First", 1),
            create_test_item("FIXME", "Second", 2),
            create_test_item("TODO", "Third", 3),
        ];

        result.add_file(PathBuf::from("test.rs"), items);

        let filtered = result.filter_by_tag("TODO");
        assert_eq!(filtered.summary.total_count, 2);
        assert_eq!(filtered.summary.tag_counts.get("TODO"), Some(&2));
    }

    #[test]
    fn test_scan_result_all_items() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![create_test_item("TODO", "A", 1)],
        );
        result.add_file(
            PathBuf::from("b.rs"),
            vec![create_test_item("TODO", "B", 1)],
        );

        let all = result.all_items();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_scan_result_sorted_files() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("z.rs"),
            vec![create_test_item("TODO", "Z", 1)],
        );
        result.add_file(
            PathBuf::from("a.rs"),
            vec![create_test_item("TODO", "A", 1)],
        );

        let sorted = result.sorted_files();
        assert_eq!(sorted[0].0, &PathBuf::from("a.rs"));
        assert_eq!(sorted[1].0, &PathBuf::from("z.rs"));
    }

    #[test]
    fn test_summary_avg_items_per_file() {
        let summary = Summary {
            total_count: 10,
            files_with_todos: 2,
            files_scanned: 5,
            tag_counts: HashMap::new(),
        };

        assert!((summary.avg_items_per_file() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_summary_avg_items_per_file_empty() {
        let summary = Summary {
            total_count: 0,
            files_with_todos: 0,
            files_scanned: 5,
            tag_counts: HashMap::new(),
        };

        assert_eq!(summary.avg_items_per_file(), 0.0);
    }

    #[test]
    fn test_summary_tag_percentage() {
        let summary = Summary {
            total_count: 10,
            files_with_todos: 2,
            files_scanned: 5,
            tag_counts: HashMap::new(),
        };

        assert!((summary.tag_percentage(3) - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_summary_tag_percentage_empty() {
        let summary = Summary {
            total_count: 0,
            files_with_todos: 0,
            files_scanned: 5,
            tag_counts: HashMap::new(),
        };

        assert_eq!(summary.tag_percentage(0), 0.0);
    }

    #[test]
    fn test_scan_result_to_json_format() {
        let mut result = ScanResult::new(PathBuf::from("/test"));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![create_test_item("TODO", "A", 1)],
        );
        result.add_file(
            PathBuf::from("b.rs"),
            vec![create_test_item("TODO", "B", 1)],
        );

        let json_result = result.to_json_format();
        assert!(json_result.files.is_some());
        assert_eq!(json_result.files.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_scan_result_from_json() {
        let files = vec![FileResult {
            path: "test.rs".to_string(),
            items: vec![create_test_item("TODO", "Test", 1)],
        }];

        let summary = Summary {
            total_count: 1,
            files_with_todos: 1,
            files_scanned: 1,
            tag_counts: HashMap::new(),
        };

        let result = ScanResult::from_json(files, summary);
        assert!(result.files.is_some());
        assert_eq!(result.summary.total_count, 1);
    }

    #[test]
    fn test_todo_item_serialization() {
        let item = create_test_item("TODO", "Test", 1);
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: TodoItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }
}
