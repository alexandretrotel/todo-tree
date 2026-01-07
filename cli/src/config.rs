use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use todo_tree_core::tags;

/// Get default tags to search for if none are specified
pub fn default_tags() -> Vec<String> {
    tags::default_tag_names()
}

/// CLI options to merge with configuration
#[derive(Debug, Clone, Default)]
pub struct CliOptions {
    pub tags: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub json: bool,
    pub flat: bool,
    pub no_color: bool,
    pub case_sensitive: Option<bool>,
    pub ignore_case: bool,
    pub no_require_colon: bool,
}

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

    /// Case sensitive matching (default: true for uppercase-only matching)
    pub case_sensitive: bool,

    /// Whether to require a colon after tags (default: true)
    pub require_colon: bool,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            tags: default_tags(),
            include: Vec::new(),
            exclude: Vec::new(),
            json: false,
            flat: false,
            no_color: false,
            custom_pattern: None,
            case_sensitive: true,
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

        // CLI flags always override if set to true
        if cli.json {
            self.json = true;
        }
        if cli.flat {
            self.flat = true;
        }
        if cli.no_color {
            self.no_color = true;
        }

        // Handle case sensitivity - explicit flag takes precedence
        if let Some(case_sensitive) = cli.case_sensitive {
            self.case_sensitive = case_sensitive;
        }

        // If ignore_case flag is set, make case-insensitive
        if cli.ignore_case {
            self.case_sensitive = false;
        }

        // Handle colon requirement
        if cli.no_require_colon {
            self.require_colon = false;
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

        config.merge_with_cli(CliOptions {
            tags: Some(vec!["CUSTOM".to_string()]),
            include: Some(vec!["*.rs".to_string()]),
            exclude: Some(vec!["target/**".to_string()]),
            json: true,
            flat: false,
            no_color: true,
            case_sensitive: None,
            ignore_case: false,
            no_require_colon: false,
        });

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

    #[test]
    fn test_save_yaml_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let mut config = Config::new();
        config.tags = vec!["YAML_TAG".to_string()];
        config.flat = true;

        config.save(&config_path).unwrap();

        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.tags, vec!["YAML_TAG"]);
        assert!(loaded.flat);
    }

    #[test]
    fn test_save_yml_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yml");

        let mut config = Config::new();
        config.tags = vec!["YML_TAG".to_string()];

        config.save(&config_path).unwrap();

        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.tags, vec!["YML_TAG"]);
    }

    #[test]
    fn test_load_from_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        // Create config in parent directory
        let config_content = r#"{"tags": ["PARENT_TAG"]}"#;
        std::fs::write(temp_dir.path().join(".todorc.json"), config_content).unwrap();

        // Load from subdirectory
        let config = Config::load(&sub_dir).unwrap();
        assert!(config.is_some());
        assert_eq!(config.unwrap().tags, vec!["PARENT_TAG"]);
    }

    #[test]
    fn test_load_no_config_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::load(temp_dir.path()).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_load_todorc_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"{"tags": ["PLAIN_TODORC"]}"#;
        std::fs::write(temp_dir.path().join(".todorc"), config_content).unwrap();

        let config = Config::load(temp_dir.path()).unwrap();
        assert!(config.is_some());
        assert_eq!(config.unwrap().tags, vec!["PLAIN_TODORC"]);
    }

    #[test]
    fn test_load_yaml_as_fallback_for_todorc() {
        let temp_dir = TempDir::new().unwrap();
        // Write YAML content to .todorc file (no extension)
        let config_content = "tags:\n  - YAML_IN_TODORC\n";
        std::fs::write(temp_dir.path().join(".todorc"), config_content).unwrap();

        let config = Config::load(temp_dir.path()).unwrap();
        assert!(config.is_some());
        assert_eq!(config.unwrap().tags, vec!["YAML_IN_TODORC"]);
    }

    #[test]
    fn test_merge_with_cli_empty_options() {
        let mut config = Config::new();
        let original_tags = config.tags.clone();

        // Empty options should not change anything
        config.merge_with_cli(CliOptions {
            tags: Some(vec![]),
            include: Some(vec![]),
            exclude: Some(vec![]),
            json: false,
            flat: false,
            no_color: false,
            case_sensitive: None,
            ignore_case: false,
            no_require_colon: false,
        });

        assert_eq!(config.tags, original_tags);
        assert!(!config.json);
        assert!(!config.flat);
        assert!(!config.no_color);
    }

    #[test]
    fn test_merge_with_cli_none_options() {
        let mut config = Config::new();
        let original_tags = config.tags.clone();

        config.merge_with_cli(CliOptions::default());

        assert_eq!(config.tags, original_tags);
    }

    #[test]
    fn test_merge_extends_exclude() {
        let mut config = Config::new();
        config.exclude = vec!["existing/**".to_string()];

        config.merge_with_cli(CliOptions {
            tags: None,
            include: None,
            exclude: Some(vec!["new/**".to_string()]),
            json: false,
            flat: false,
            no_color: false,
            case_sensitive: None,
            ignore_case: false,
            no_require_colon: false,
        });

        assert!(config.exclude.contains(&"existing/**".to_string()));
        assert!(config.exclude.contains(&"new/**".to_string()));
    }

    #[test]
    fn test_config_with_all_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".todorc.json");

        let config_content = r#"{
            "tags": ["CUSTOM"],
            "include": ["*.rs"],
            "exclude": ["target/**"],
            "json": true,
            "flat": true,
            "no_color": true,
            "custom_pattern": "PATTERN",
            "case_sensitive": true,
            "require_colon": false
        }"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.tags, vec!["CUSTOM"]);
        assert_eq!(config.include, vec!["*.rs"]);
        assert_eq!(config.exclude, vec!["target/**"]);
        assert!(config.json);
        assert!(config.flat);
        assert!(config.no_color);
        assert_eq!(config.custom_pattern, Some("PATTERN".to_string()));
        assert!(config.case_sensitive);
        assert!(!config.require_colon);
    }

    #[test]
    fn test_load_stops_at_root() {
        // Test that loading from root doesn't panic
        let root = std::path::Path::new("/");
        let config = Config::load(root);
        // Should not panic, may return None or Some
        assert!(config.is_ok());
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".todorc.json");

        // Use content that's invalid for both JSON and YAML
        std::fs::write(&config_path, "{{{{{{").unwrap();

        let result = Config::load_from_file(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_file_nonexistent() {
        let result = Config::load_from_file(std::path::Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
    }
}
