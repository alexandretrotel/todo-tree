use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default tags to search for if none are specified
pub const DEFAULT_TAGS: &[&str] = &[
    "TODO", "FIXME", "BUG", "HACK", "NOTE", "XXX", "WARN", "PERF",
];

/// Configuration for the todo-tree tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Tags to search for (e.g., TODO, FIXME, BUG)
    pub tags: Vec<String>,

    /// File patterns to include (glob patterns)
    pub include: Vec<String>,

    /// File patterns to exclude (glob patterns)
    pub exclude: Vec<String>,

    /// Whether to output in JSON format
    pub json: bool,

    /// Whether to output in flat format (no tree structure)
    pub flat: bool,

    /// Whether to disable colored output
    pub no_color: bool,

    /// Custom regex pattern for matching (advanced)
    pub custom_pattern: Option<String>,

    /// Case sensitive matching
    pub case_sensitive: bool,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            tags: DEFAULT_TAGS.iter().map(|s| s.to_string()).collect(),
            include: Vec::new(),
            exclude: Vec::new(),
            json: false,
            flat: false,
            no_color: false,
            custom_pattern: None,
            case_sensitive: false,
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
        // Try local config files first
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

        // Try parent directories
        if let Some(parent) = start_path.parent()
            && parent != start_path
            && let Ok(Some(config)) = Self::load(parent)
        {
            return Ok(Some(config));
        }

        // Try global config
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

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Determine format based on extension or filename
        let parse_result = if extension == "yaml" || extension == "yml" {
            // YAML files: parse as YAML only
            serde_yaml::from_str(&content)
        } else {
            // JSON, .todorc, or unknown: try JSON first, then YAML
            serde_json::from_str(&content).or_else(|_| serde_yaml::from_str(&content))
        };

        parse_result.with_context(|| format!("Failed to parse config: {}", path.display()))
    }

    /// Merge CLI options with the loaded configuration
    ///
    /// CLI options take precedence over config file options
    pub fn merge_with_cli(
        &mut self,
        tags: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        json: bool,
        flat: bool,
        no_color: bool,
    ) {
        if let Some(tags) = tags
            && !tags.is_empty()
        {
            self.tags = tags;
        }

        if let Some(include) = include
            && !include.is_empty()
        {
            self.include = include;
        }

        if let Some(exclude) = exclude
            && !exclude.is_empty()
        {
            self.exclude.extend(exclude);
        }

        // CLI flags always override if set to true
        if json {
            self.json = true;
        }
        if flat {
            self.flat = true;
        }
        if no_color {
            self.no_color = true;
        }
    }

    /// Save the current configuration to a file
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert!(!config.tags.is_empty());
        assert!(config.tags.contains(&"TODO".to_string()));
        assert!(config.tags.contains(&"FIXME".to_string()));
        assert!(!config.json);
        assert!(!config.flat);
        assert!(!config.no_color);
    }

    #[test]
    fn test_load_json_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".todorc.json");

        let config_content = r#"{
            "tags": ["TODO", "CUSTOM"],
            "include": ["*.rs"],
            "exclude": ["target/**"],
            "json": true
        }"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.tags, vec!["TODO", "CUSTOM"]);
        assert_eq!(config.include, vec!["*.rs"]);
        assert_eq!(config.exclude, vec!["target/**"]);
        assert!(config.json);
    }

    #[test]
    fn test_load_yaml_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".todorc.yaml");

        let config_content = r#"
tags:
  - TODO
  - YAML_TAG
include:
  - "*.py"
flat: true
"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.tags, vec!["TODO", "YAML_TAG"]);
        assert_eq!(config.include, vec!["*.py"]);
        assert!(config.flat);
    }

    #[test]
    fn test_merge_with_cli() {
        let mut config = Config::new();
        config.json = false;

        config.merge_with_cli(
            Some(vec!["CUSTOM".to_string()]),
            Some(vec!["*.rs".to_string()]),
            Some(vec!["target/**".to_string()]),
            true,
            false,
            true,
        );

        assert_eq!(config.tags, vec!["CUSTOM"]);
        assert_eq!(config.include, vec!["*.rs"]);
        assert!(config.exclude.contains(&"target/**".to_string()));
        assert!(config.json);
        assert!(config.no_color);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let mut config = Config::new();
        config.tags = vec!["SAVED".to_string()];
        config.json = true;

        config.save(&config_path).unwrap();

        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.tags, vec!["SAVED"]);
        assert!(loaded.json);
    }
}
