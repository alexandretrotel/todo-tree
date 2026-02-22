use crate::{cli, config::Config};
use anyhow::Result;
use std::path::Path;
use todo_tree_core::ScanResult;

pub mod init;
pub mod list;
pub mod scan;
pub mod stats;
pub mod tags;

pub(crate) fn load_config(path: &Path, config_path: Option<&Path>) -> Result<Config> {
    if let Some(config_path) = config_path {
        return Config::load_from_file(config_path);
    }

    match Config::load(path)? {
        Some(config) => Ok(config),
        None => Ok(Config::new()),
    }
}

pub(crate) fn save_config(config: &Config) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_files = [
        current_dir.join(".todorc"),
        current_dir.join(".todorc.json"),
        current_dir.join(".todorc.yaml"),
        current_dir.join(".todorc.yml"),
    ];

    for path in &config_files {
        if path.exists() {
            return config.save(path);
        }
    }

    let path = current_dir.join(".todorc.json");
    config.save(&path)
}

pub(crate) fn sort_results(result: &mut ScanResult, sort: cli::SortOrder) {
    match sort {
        cli::SortOrder::File => {}
        cli::SortOrder::Line => {
            for items in result.files_map.values_mut() {
                items.sort_by_key(|item| item.line);
            }
        }
        cli::SortOrder::Priority => {
            for items in result.files_map.values_mut() {
                items.sort_by_key(|item| std::cmp::Reverse(item.priority));
            }
        }
    }
}
