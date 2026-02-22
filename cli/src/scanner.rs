use crate::parser::TodoParser;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::path::Path;
use todo_tree_core::{ScanResult, TodoItem};

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub max_depth: usize,
    pub follow_links: bool,
    pub hidden: bool,
    pub threads: usize,
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

pub struct Scanner {
    parser: TodoParser,
    options: ScanOptions,
}

impl Scanner {
    pub fn new(parser: TodoParser, options: ScanOptions) -> Self {
        Self { parser, options }
    }

    pub fn scan(&self, root: &Path) -> Result<ScanResult> {
        let root = root
            .canonicalize()
            .with_context(|| format!("Failed to resolve path: {}", root.display()))?;

        let mut result = ScanResult::new(root.clone());
        let mut builder = WalkBuilder::new(&root);

        builder
            .hidden(!self.options.hidden)
            .follow_links(self.options.follow_links)
            .git_ignore(self.options.respect_gitignore)
            .git_global(self.options.respect_gitignore)
            .git_exclude(self.options.respect_gitignore);

        if self.options.max_depth > 0 {
            builder.max_depth(Some(self.options.max_depth));
        }

        if self.options.threads > 0 {
            builder.threads(self.options.threads);
        }

        if !self.options.include.is_empty() || !self.options.exclude.is_empty() {
            let mut override_builder = OverrideBuilder::new(&root);
            for pattern in &self.options.include {
                override_builder
                    .add(pattern)
                    .with_context(|| format!("Invalid include pattern: {}", pattern))?;
            }

            for pattern in &self.options.exclude {
                let exclude_pattern = format!("!{}", pattern);
                override_builder
                    .add(&exclude_pattern)
                    .with_context(|| format!("Invalid exclude pattern: {}", pattern))?;
            }

            let overrides = override_builder.build()?;
            builder.overrides(overrides);
        }

        for entry in builder.build() {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    if path.is_dir() {
                        continue;
                    }

                    if let Some(file_type) = entry.file_type()
                        && !file_type.is_file()
                    {
                        continue;
                    }

                    match self.parse_file(path) {
                        Ok(items) => {
                            result.add_file(path.to_path_buf(), items);
                        }
                        Err(_) => {
                            result.summary.files_scanned += 1;
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }

        Ok(result)
    }

    fn parse_file(&self, path: &Path) -> Result<Vec<TodoItem>> {
        self.parser
            .parse_file(path)
            .with_context(|| format!("Failed to parse file: {}", path.display()))
    }
}
