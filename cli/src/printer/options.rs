use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Tree,
    Flat,
    Json,
}

#[derive(Debug, Clone)]
pub struct PrintOptions {
    pub format: OutputFormat,
    pub colored: bool,
    pub show_line_numbers: bool,
    pub full_paths: bool,
    pub clickable_links: bool,
    pub base_path: Option<PathBuf>,
    pub show_summary: bool,
    pub group_by_tag: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Tree,
            colored: true,
            show_line_numbers: true,
            full_paths: false,
            clickable_links: true,
            base_path: None,
            show_summary: true,
            group_by_tag: false,
        }
    }
}
