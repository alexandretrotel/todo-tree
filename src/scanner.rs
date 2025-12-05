use crate::parser::{TodoItem, TodoParser};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result of scanning a directory for TODO items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Map of file paths to their TODO items
    pub files: HashMap<PathBuf, Vec<TodoItem>>,

    /// Total number of TODO items found
    pub total_count: usize,

    /// Number of files scanned
    pub files_scanned: usize,

    /// Number of files with TODO items
    pub files_with_todos: usize,

    /// Count by tag type
    pub tag_counts: HashMap<String, usize>,

    /// Root directory that was scanned
    pub root: PathBuf,
}

impl ScanResult {
    /// Create a new empty scan result
    pub fn new(root: PathBuf) -> Self {
        Self {
            files: HashMap::new(),
            total_count: 0,
            files_scanned: 0,
            files_with_todos: 0,
            tag_counts: HashMap::new(),
            root,
        }
    }

    /// Add TODO items for a file
    pub fn add_file(&mut self, path: PathBuf, items: Vec<TodoItem>) {
        self.files_scanned += 1;

        if !items.is_empty() {
            self.files_with_todos += 1;
            self.total_count += items.len();

            for item in &items {
                *self.tag_counts.entry(item.tag.clone()).or_insert(0) += 1;
            }

            self.files.insert(path, items);
        }
    }

    /// Get all TODO items as a flat list
    pub fn all_items(&self) -> Vec<(PathBuf, TodoItem)> {
        let mut items = Vec::new();
        for (path, file_items) in &self.files {
            for item in file_items {
                items.push((path.clone(), item.clone()));
            }
        }
        items
    }

    /// Get files sorted by path
    pub fn sorted_files(&self) -> Vec<(&PathBuf, &Vec<TodoItem>)> {
        let mut files: Vec<_> = self.files.iter().collect();
        files.sort_by(|a, b| a.0.cmp(b.0));
        files
    }

    /// Filter items by tag
    pub fn filter_by_tag(&self, tag: &str) -> ScanResult {
        let mut result = ScanResult::new(self.root.clone());
        result.files_scanned = self.files_scanned;

        for (path, items) in &self.files {
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
}

/// Options for scanning
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// File patterns to include (glob patterns)
    pub include: Vec<String>,

    /// File patterns to exclude (glob patterns)
    pub exclude: Vec<String>,

    /// Maximum depth to scan (0 = unlimited)
    pub max_depth: usize,

    /// Follow symbolic links
    pub follow_links: bool,

    /// Include hidden files
    pub hidden: bool,

    /// Number of threads to use (0 = auto)
    pub threads: usize,

    /// Respect .gitignore files
    pub respect_gitignore: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            max_depth: 0,
            follow_links: false,
            hidden: false,
            threads: 0,
            respect_gitignore: true,
        }
    }
}

/// Scanner for finding TODO items in a directory
pub struct Scanner {
    parser: TodoParser,
    options: ScanOptions,
}

impl Scanner {
    /// Create a new scanner with the given parser and options
    pub fn new(parser: TodoParser, options: ScanOptions) -> Self {
        Self { parser, options }
    }

    /// Scan a directory for TODO items
    pub fn scan(&self, root: &Path) -> Result<ScanResult> {
        let root = root
            .canonicalize()
            .with_context(|| format!("Failed to resolve path: {}", root.display()))?;

        let mut result = ScanResult::new(root.clone());

        // Build the walker
        let mut builder = WalkBuilder::new(&root);

        // Configure the walker
        builder
            .hidden(!self.options.hidden)
            .follow_links(self.options.follow_links)
            .git_ignore(self.options.respect_gitignore)
            .git_global(self.options.respect_gitignore)
            .git_exclude(self.options.respect_gitignore);

        // Set max depth if specified
        if self.options.max_depth > 0 {
            builder.max_depth(Some(self.options.max_depth));
        }

        // Set number of threads
        if self.options.threads > 0 {
            builder.threads(self.options.threads);
        }

        // Add include/exclude patterns as overrides
        if !self.options.include.is_empty() || !self.options.exclude.is_empty() {
            let mut override_builder = OverrideBuilder::new(&root);

            // Add include patterns (must be prefixed with !)
            for pattern in &self.options.include {
                // Include patterns are added as-is
                override_builder
                    .add(pattern)
                    .with_context(|| format!("Invalid include pattern: {}", pattern))?;
            }

            // Add exclude patterns (prefixed with !)
            for pattern in &self.options.exclude {
                let exclude_pattern = format!("!{}", pattern);
                override_builder
                    .add(&exclude_pattern)
                    .with_context(|| format!("Invalid exclude pattern: {}", pattern))?;
            }

            let overrides = override_builder.build()?;
            builder.overrides(overrides);
        }

        // Walk the directory
        for entry in builder.build() {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip directories
                    if path.is_dir() {
                        continue;
                    }

                    // Skip non-text files (binary detection)
                    if let Some(file_type) = entry.file_type()
                        && !file_type.is_file()
                    {
                        continue;
                    }

                    // Parse the file
                    match self.parse_file(path) {
                        Ok(items) => {
                            result.add_file(path.to_path_buf(), items);
                        }
                        Err(_) => {
                            // Skip files that can't be read (binary files, permission errors, etc.)
                            result.files_scanned += 1;
                        }
                    }
                }
                Err(_) => {
                    // Skip entries that can't be accessed
                    continue;
                }
            }
        }

        Ok(result)
    }

    /// Parse a single file for TODO items
    fn parse_file(&self, path: &Path) -> Result<Vec<TodoItem>> {
        self.parser
            .parse_file(path)
            .with_context(|| format!("Failed to parse file: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        path
    }

    fn default_tags() -> Vec<String> {
        vec![
            "TODO".to_string(),
            "FIXME".to_string(),
            "BUG".to_string(),
            "NOTE".to_string(),
        ]
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();

        assert_eq!(result.total_count, 0);
        assert_eq!(result.files_with_todos, 0);
    }

    #[test]
    fn test_scan_with_todos() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(
            temp_dir.path(),
            "test.rs",
            r#"
// TODO: First todo
fn main() {
    // FIXME: Fix this
}
"#,
        );

        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();

        assert_eq!(result.total_count, 2);
        assert_eq!(result.files_with_todos, 1);
        assert_eq!(result.tag_counts.get("TODO"), Some(&1));
        assert_eq!(result.tag_counts.get("FIXME"), Some(&1));
    }

    #[test]
    fn test_scan_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "file1.rs", "// TODO: In file 1");
        create_test_file(temp_dir.path(), "file2.rs", "// TODO: In file 2");
        create_test_file(temp_dir.path(), "file3.rs", "// No todos here");

        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();

        assert_eq!(result.total_count, 2);
        assert_eq!(result.files_with_todos, 2);
    }

    #[test]
    fn test_scan_nested_directories() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "src/main.rs", "// TODO: Main todo");
        create_test_file(temp_dir.path(), "src/lib/mod.rs", "// FIXME: Lib todo");
        create_test_file(temp_dir.path(), "tests/test.rs", "// NOTE: Test note");

        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.files_with_todos, 3);
    }

    #[test]
    fn test_scan_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize a git repository so .gitignore is respected
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Create .gitignore
        create_test_file(temp_dir.path(), ".gitignore", "ignored/\n");

        // Create files
        create_test_file(temp_dir.path(), "included.rs", "// TODO: Should be found");
        create_test_file(
            temp_dir.path(),
            "ignored/hidden.rs",
            "// TODO: Should be ignored",
        );

        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();

        // Should only find the TODO in included.rs
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_scan_result_filter_by_tag() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(
            temp_dir.path(),
            "test.rs",
            r#"
// TODO: First
// FIXME: Second
// TODO: Third
// NOTE: Fourth
"#,
        );

        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();
        let filtered = result.filter_by_tag("TODO");

        assert_eq!(filtered.total_count, 2);
        assert_eq!(filtered.tag_counts.get("TODO"), Some(&2));
    }

    #[test]
    fn test_scan_result_all_items() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "a.rs", "// TODO: A");
        create_test_file(temp_dir.path(), "b.rs", "// TODO: B");

        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();
        let all_items = result.all_items();

        assert_eq!(all_items.len(), 2);
    }

    #[test]
    fn test_scan_max_depth() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "level1.rs", "// TODO: Level 1");
        create_test_file(temp_dir.path(), "sub/level2.rs", "// TODO: Level 2");
        create_test_file(temp_dir.path(), "sub/deep/level3.rs", "// TODO: Level 3");

        let parser = TodoParser::new(&default_tags(), false);
        let options = ScanOptions {
            max_depth: 2,
            ..Default::default()
        };
        let scanner = Scanner::new(parser, options);

        let result = scanner.scan(temp_dir.path()).unwrap();

        // Should find level1 and level2, but not level3
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_scan_hidden_files() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "visible.rs", "// TODO: Visible");
        create_test_file(temp_dir.path(), ".hidden.rs", "// TODO: Hidden");

        let parser = TodoParser::new(&default_tags(), false);

        // Without hidden option
        let scanner = Scanner::new(parser.clone(), ScanOptions::default());
        let result = scanner.scan(temp_dir.path()).unwrap();
        assert_eq!(result.total_count, 1);

        // With hidden option
        let options = ScanOptions {
            hidden: true,
            ..Default::default()
        };
        let parser = TodoParser::new(&default_tags(), false);
        let scanner = Scanner::new(parser, options);
        let result = scanner.scan(temp_dir.path()).unwrap();
        assert_eq!(result.total_count, 2);
    }
}
