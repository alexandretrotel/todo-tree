use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use todo_tree_core::tags::default_tag_names;

#[derive(Debug, Clone, Default)]
pub struct CliOptions {
    pub tags: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub json: bool,
    pub flat: bool,
    pub no_color: bool,
    pub ignore_case: bool,
    pub no_require_colon: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub tags: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub json: bool,
    pub flat: bool,
    pub no_color: bool,
    pub custom_pattern: Option<String>,
    pub ignore_case: bool,
    pub require_colon: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            tags: default_tag_names(),
            include: Vec::new(),
            exclude: Vec::new(),
            json: false,
            flat: false,
            no_color: false,
            custom_pattern: None,
            ignore_case: false,
            require_colon: true,
        }
    }

    /// Load configuration from a .todorc file
    ///
    /// Searches for configuration files in the following order:
    /// 1. .todorc in the current directory
    /// 2. .todorc.json in the current directory
    /// 3. .todorc.yaml or .todorc.yml in the current directory
    /// 4. ~/.config/todo-tree/config.json (global config)
    pub fn load(start_path: &Path) -> Result<Option<Self>> {
        let local_configs = [
            start_path.join(".todorc"),
            start_path.join(".todorc.json"),
            start_path.join(".todorc.yaml"),
            start_path.join(".todorc.yml"),
        ];

        for config_path in &local_configs {
            if config_path.exists() {
                return Self::load_from_file(config_path).map(Some);
            }
        }

        if let Some(parent) = start_path.parent()
            && parent != start_path
            && let Ok(Some(config)) = Self::load(parent)
        {
            return Ok(Some(config));
        }

        if let Some(config_dir) = dirs::config_dir() {
            let global_configs = [
                config_dir.join("todo-tree").join("config.json"),
                config_dir.join("todo-tree").join("config.yaml"),
                config_dir.join("todo-tree").join("config.yml"),
            ];

            for config_path in &global_configs {
                if config_path.exists() {
                    return Self::load_from_file(config_path).map(Some);
                }
            }
        }

        Ok(None)
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let parse_result = if extension == "yaml" || extension == "yml" {
            serde_yaml::from_str(&content)
        } else {
            serde_json::from_str(&content).or_else(|_| serde_yaml::from_str(&content))
        };

        parse_result.with_context(|| format!("Failed to parse config: {}", path.display()))
    }

    pub fn merge_with_cli(&mut self, cli: CliOptions) {
        if let Some(tags) = cli.tags
            && !tags.is_empty()
        {
            self.tags = tags;
        }

        if let Some(include) = cli.include
            && !include.is_empty()
        {
            self.include = include;
        }

        if let Some(exclude) = cli.exclude
            && !exclude.is_empty()
        {
            self.exclude.extend(exclude);
        }

        if cli.json {
            self.json = true;
        }
        if cli.flat {
            self.flat = true;
        }
        if cli.no_color {
            self.no_color = true;
        }

        if cli.ignore_case {
            self.ignore_case = true;
        }

        if cli.no_require_colon {
            self.require_colon = false;
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let content = if extension == "yaml" || extension == "yml" {
            serde_yaml::to_string(self)?
        } else {
            serde_json::to_string_pretty(self)?
        };

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
}
