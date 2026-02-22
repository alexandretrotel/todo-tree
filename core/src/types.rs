use crate::priority::Priority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    pub tag: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub priority: Priority,
}

impl TodoItem {
    pub fn format_author(&self) -> String {
        self.author
            .as_ref()
            .map(|a| format!("({})", a))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileResult {
    pub path: String,
    pub items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanSummary {
    pub total_count: usize,
    pub files_with_todos: usize,
    pub files_scanned: usize,
    pub tag_counts: HashMap<String, usize>,
}

impl ScanSummary {
    pub fn avg_items_per_file(&self) -> f64 {
        if self.files_with_todos > 0 {
            self.total_count as f64 / self.files_with_todos as f64
        } else {
            0.0
        }
    }

    pub fn tag_percentage(&self, count: usize) -> f64 {
        if self.total_count > 0 {
            (count as f64 / self.total_count as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileResult>>,
    #[serde(skip)]
    pub files_map: HashMap<PathBuf, Vec<TodoItem>>,
    pub summary: ScanSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
}

impl ScanResult {
    pub fn new(root: PathBuf) -> Self {
        Self {
            files: None,
            files_map: HashMap::new(),
            summary: ScanSummary {
                total_count: 0,
                files_with_todos: 0,
                files_scanned: 0,
                tag_counts: HashMap::new(),
            },
            root: Some(root),
        }
    }

    pub fn from_json(files: Vec<FileResult>, summary: ScanSummary) -> Self {
        Self {
            files: Some(files),
            files_map: HashMap::new(),
            summary,
            root: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        if let Some(files) = &self.files {
            files.is_empty()
        } else {
            self.files_map.is_empty()
        }
    }

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

    pub fn all_items(&self) -> Vec<(PathBuf, TodoItem)> {
        let mut items = Vec::new();
        for (path, file_items) in &self.files_map {
            for item in file_items {
                items.push((path.clone(), item.clone()));
            }
        }
        items
    }

    pub fn sorted_files(&self) -> Vec<(&PathBuf, &Vec<TodoItem>)> {
        let mut files: Vec<_> = self.files_map.iter().collect();
        files.sort_by(|a, b| a.0.cmp(b.0));
        files
    }

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

    pub fn to_json_format(&self) -> Self {
        let mut files: Vec<FileResult> = self
            .files_map
            .iter()
            .map(|(path, items)| FileResult {
                path: path.display().to_string(),
                items: items.clone(),
            })
            .collect();

        files.sort_by(|a, b| a.path.cmp(&b.path));

        Self {
            files: Some(files),
            files_map: HashMap::new(),
            summary: self.summary.clone(),
            root: None,
        }
    }

    pub fn get_files(&self) -> Vec<FileResult> {
        if let Some(files) = &self.files {
            files.clone()
        } else {
            self.to_json_format().files.unwrap_or_default()
        }
    }
}
