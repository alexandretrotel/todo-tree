pub mod cli;
pub mod config;
pub mod parser;
pub mod printer;
pub mod scanner;

pub use todo_tree_core::{Priority, ScanResult, Summary, TodoItem};

use anyhow::Result;
use cli::{Cli, Commands, ConfigFormat, ScanArgs, SortOrder};
use config::Config;
use parser::{TodoParser, priority_to_color};
use printer::{OutputFormat, PrintOptions, Printer};
use scanner::{ScanOptions, Scanner};
use std::path::PathBuf;

/// Main entry point for the CLI application
pub fn run() -> Result<()> {
    let cli = Cli::parse_args();

    // Handle no-color globally
    if cli.global.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    // Execute the command
    match cli.get_command() {
        Commands::Scan(args) => cmd_scan(args, &cli.global),
        Commands::List(args) => cmd_list(args, &cli.global),
        Commands::Tags(args) => cmd_tags(args, &cli.global),
        Commands::Init(args) => cmd_init(args),
        Commands::Stats(args) => cmd_stats(args, &cli.global),
    }
}

/// Execute the scan command
fn cmd_scan(args: ScanArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    // Load configuration
    let mut config = load_config(&path, global.config.as_deref())?;

    // Merge CLI options
    config.merge_with_cli(
        args.tags.clone(),
        args.include.clone(),
        args.exclude.clone(),
        args.json,
        args.flat,
        global.no_color,
    );

    // Create parser
    let parser = TodoParser::new(&config.tags, args.case_sensitive);

    // Create scan options
    let scan_options = ScanOptions {
        include: config.include.clone(),
        exclude: config.exclude.clone(),
        max_depth: args.depth,
        follow_links: args.follow_links,
        hidden: args.hidden,
        threads: 0, // Auto
        respect_gitignore: true,
    };

    // Create scanner and scan
    let scanner = Scanner::new(parser, scan_options);
    let mut result = scanner.scan(&path)?;

    // Sort results if needed
    sort_results(&mut result, args.sort);

    // Print results
    let print_options = PrintOptions {
        format: if args.json {
            OutputFormat::Json
        } else if args.flat {
            OutputFormat::Flat
        } else {
            OutputFormat::Tree
        },
        colored: !global.no_color,
        show_line_numbers: true,
        full_paths: false,
        clickable_links: !global.no_color,
        base_path: Some(path),
        show_summary: !args.json,
        group_by_tag: args.group_by_tag,
    };

    let printer = Printer::new(print_options);
    printer.print(&result)?;

    Ok(())
}

/// Execute the list command
fn cmd_list(args: cli::ListArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    // Load configuration
    let mut config = load_config(&path, global.config.as_deref())?;

    // Merge CLI options
    config.merge_with_cli(
        args.tags.clone(),
        args.include.clone(),
        args.exclude.clone(),
        args.json,
        true, // flat format for list
        global.no_color,
    );

    // Create parser
    let parser = TodoParser::new(&config.tags, args.case_sensitive);

    // Create scan options
    let scan_options = ScanOptions {
        include: config.include.clone(),
        exclude: config.exclude.clone(),
        ..Default::default()
    };

    // Create scanner and scan
    let scanner = Scanner::new(parser, scan_options);
    let result = scanner.scan(&path)?;

    // Filter by tag if specified
    let result = if let Some(filter_tag) = &args.filter {
        result.filter_by_tag(filter_tag)
    } else {
        result
    };

    // Print results
    let print_options = PrintOptions {
        format: if args.json {
            OutputFormat::Json
        } else {
            OutputFormat::Flat
        },
        colored: !global.no_color,
        show_line_numbers: true,
        full_paths: false,
        clickable_links: !global.no_color,
        base_path: Some(path),
        show_summary: !args.json,
        group_by_tag: false,
    };

    let printer = Printer::new(print_options);
    printer.print(&result)?;

    Ok(())
}

/// Execute the tags command
fn cmd_tags(args: cli::TagsArgs, global: &cli::GlobalOptions) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut config = load_config(&current_dir, global.config.as_deref())?;

    // Handle tag modifications
    if let Some(new_tag) = &args.add {
        if !config.tags.iter().any(|t| t.eq_ignore_ascii_case(new_tag)) {
            config.tags.push(new_tag.to_uppercase());
            save_config(&config)?;
            println!("Added tag: {}", new_tag.to_uppercase());
        } else {
            println!("Tag already exists: {}", new_tag);
        }
        return Ok(());
    }

    if let Some(remove_tag) = &args.remove {
        let original_len = config.tags.len();
        config.tags.retain(|t| !t.eq_ignore_ascii_case(remove_tag));
        if config.tags.len() < original_len {
            save_config(&config)?;
            println!("Removed tag: {}", remove_tag);
        } else {
            println!("Tag not found: {}", remove_tag);
        }
        return Ok(());
    }

    if args.reset {
        config.tags = config::default_tags();
        save_config(&config)?;
        println!("Tags reset to defaults");
        return Ok(());
    }

    // Display current tags
    if args.json {
        let json = serde_json::json!({
            "tags": config.tags,
            "default_tags": config::default_tags(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        use colored::Colorize;
        println!("{}", "Configured tags:".bold());
        for tag in &config.tags {
            if global.no_color {
                println!("  - {}", tag);
            } else {
                let color = priority_to_color(Priority::from_tag(tag));
                println!("  - {}", tag.color(color));
            }
        }
    }

    Ok(())
}

/// Execute the init command
fn cmd_init(args: cli::InitArgs) -> Result<()> {
    let filename = match args.format {
        ConfigFormat::Json => ".todorc.json",
        ConfigFormat::Yaml => ".todorc.yaml",
    };

    let path = PathBuf::from(filename);

    if path.exists() && !args.force {
        anyhow::bail!(
            "Config file {} already exists. Use --force to overwrite.",
            filename
        );
    }

    let config = Config::new();
    config.save(&path)?;

    println!("Created configuration file: {}", filename);
    println!("\nYou can customize the following settings:");
    println!("  - tags: List of tags to search for");
    println!("  - include: File patterns to include");
    println!("  - exclude: File patterns to exclude");
    println!("  - json: Default to JSON output");
    println!("  - flat: Default to flat output");

    Ok(())
}

/// Execute the stats command
fn cmd_stats(args: cli::StatsArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    // Load configuration
    let config = load_config(&path, global.config.as_deref())?;

    // Get tags from CLI or config
    let tags = args.tags.clone().unwrap_or(config.tags.clone());

    // Create parser and scanner
    let parser = TodoParser::new(&tags, false);
    let scanner = Scanner::new(parser, ScanOptions::default());
    let result = scanner.scan(&path)?;

    if args.json {
        let stats = serde_json::json!({
            "total_items": result.summary.total_count,
            "files_with_todos": result.summary.files_with_todos,
            "files_scanned": result.summary.files_scanned,
            "tag_counts": result.summary.tag_counts,
            "items_per_file": if result.summary.files_with_todos > 0 {
                result.summary.total_count as f64 / result.summary.files_with_todos as f64
            } else {
                0.0
            },
        });
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        use colored::Colorize;

        println!("{}", "TODO Statistics".bold().underline());
        println!();
        println!("  Total items:        {}", result.summary.total_count);
        println!("  Files with TODOs:   {}", result.summary.files_with_todos);
        println!("  Files scanned:      {}", result.summary.files_scanned);

        if result.summary.files_with_todos > 0 {
            let avg = result.summary.total_count as f64 / result.summary.files_with_todos as f64;
            println!("  Avg items per file: {:.2}", avg);
        }

        println!();
        println!("{}", "By Tag:".bold());

        let mut tags: Vec<_> = result.summary.tag_counts.iter().collect();
        tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in tags {
            let percentage = if result.summary.total_count > 0 {
                (*count as f64 / result.summary.total_count as f64) * 100.0
            } else {
                0.0
            };

            let bar_width = 20;
            let filled = ((percentage / 100.0) * bar_width as f64) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

            if global.no_color {
                println!("  {:<8} {:>4} ({:>5.1}%) {}", tag, count, percentage, bar);
            } else {
                let color = priority_to_color(Priority::from_tag(tag));
                println!(
                    "  {:<8} {:>4} ({:>5.1}%) {}",
                    tag.color(color),
                    count,
                    percentage,
                    bar.dimmed()
                );
            }
        }
    }

    Ok(())
}

/// Load configuration from file or use defaults
fn load_config(path: &std::path::Path, config_path: Option<&std::path::Path>) -> Result<Config> {
    if let Some(config_path) = config_path {
        return Config::load_from_file(config_path);
    }

    match Config::load(path)? {
        Some(config) => Ok(config),
        None => Ok(Config::new()),
    }
}

/// Save configuration to the default config file
fn save_config(config: &Config) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Try to find existing config file
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

    // Create new config file
    let path = current_dir.join(".todorc.json");
    config.save(&path)
}

/// Sort scan results based on the specified order
fn sort_results(result: &mut ScanResult, sort: SortOrder) {
    match sort {
        SortOrder::File => {
            // Already sorted by file path
        }

        SortOrder::Line => {
            // Sort items within each file by line number
            for items in result.files_map.values_mut() {
                items.sort_by_key(|item| item.line);
            }
        }
        SortOrder::Priority => {
            // Sort items within each file by priority
            for items in result.files_map.values_mut() {
                items.sort_by_key(|item| std::cmp::Reverse(item.priority));
            }
        }
    }
}

use anyhow::Context;

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files with TODOs
        fs::write(
            temp_dir.path().join("main.rs"),
            r#"
fn main() {
    // TODO: Implement main logic
    println!("Hello, world!");
    // FIXME: This is broken
}
"#,
        )
        .unwrap();

        fs::write(
            temp_dir.path().join("lib.rs"),
            r#"
// NOTE: This is a library module
pub fn hello() {
    // TODO(alice): Add documentation
    // BUG: Memory leak here
}
"#,
        )
        .unwrap();

        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("src/utils.rs"),
            r#"
// HACK: Temporary workaround
fn temp_fix() {}
"#,
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_scan_finds_todos() {
        let temp_dir = create_test_project();

        let tags: Vec<String> = config::DEFAULT_TAGS.iter().map(|s| s.to_string()).collect();
        let parser = TodoParser::new(&tags, false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let result = scanner.scan(temp_dir.path()).unwrap();

        assert!(result.summary.total_count >= 5);
        assert!(result.summary.files_with_todos >= 2);
    }

    #[test]
    fn test_config_loading() {
        let temp_dir = TempDir::new().unwrap();

        // Create a config file
        let config_content = r#"{
            "tags": ["CUSTOM", "TEST"],
            "include": ["*.rs"],
            "exclude": ["target/**"]
        }"#;

        fs::write(temp_dir.path().join(".todorc.json"), config_content).unwrap();

        let config = load_config(temp_dir.path(), None).unwrap();

        assert_eq!(config.tags, vec!["CUSTOM", "TEST"]);
        assert_eq!(config.include, vec!["*.rs"]);
    }

    #[test]
    fn test_sort_by_priority() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
// NOTE: Low priority
// TODO: Medium priority
// BUG: Critical priority
// HACK: High priority
"#,
        )
        .unwrap();

        let tags: Vec<String> = config::DEFAULT_TAGS.iter().map(|s| s.to_string()).collect();
        let parser = TodoParser::new(&tags, false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let mut result = scanner.scan(temp_dir.path()).unwrap();
        sort_results(&mut result, SortOrder::Priority);

        // Check that items are sorted by priority within files
        for items in result.files_map.values() {
            for window in items.windows(2) {
                assert!(window[0].priority >= window[1].priority);
            }
        }
    }

    #[test]
    fn test_sort_by_file() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.rs"), "// TODO: Test").unwrap();

        let tags: Vec<String> = config::DEFAULT_TAGS.iter().map(|s| s.to_string()).collect();
        let parser = TodoParser::new(&tags, false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let mut result = scanner.scan(temp_dir.path()).unwrap();
        // Sort by file should not panic
        sort_results(&mut result, SortOrder::File);

        assert!(result.summary.total_count >= 1);
    }

    #[test]
    fn test_sort_by_line() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
// TODO: Line 2
fn main() {}
// TODO: Line 4
// TODO: Line 5
"#,
        )
        .unwrap();

        let tags: Vec<String> = config::DEFAULT_TAGS.iter().map(|s| s.to_string()).collect();
        let parser = TodoParser::new(&tags, false);
        let scanner = Scanner::new(parser, ScanOptions::default());

        let mut result = scanner.scan(temp_dir.path()).unwrap();
        sort_results(&mut result, SortOrder::Line);

        // Check that items are sorted by line number within files
        for items in result.files_map.values() {
            for window in items.windows(2) {
                assert!(window[0].line <= window[1].line);
            }
        }
    }

    #[test]
    fn test_load_config_with_explicit_path() {
        let temp_dir = TempDir::new().unwrap();

        let config_content = r#"{"tags": ["EXPLICIT"]}"#;
        let config_path = temp_dir.path().join("custom.json");
        fs::write(&config_path, config_content).unwrap();

        let config = load_config(temp_dir.path(), Some(&config_path)).unwrap();
        assert_eq!(config.tags, vec!["EXPLICIT"]);
    }

    #[test]
    fn test_load_config_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let config = load_config(temp_dir.path(), None).unwrap();
        // Should return default config
        assert!(!config.tags.is_empty());
        assert!(config.tags.contains(&"TODO".to_string()));
    }

    #[test]
    #[serial]
    fn test_save_config_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();
        let result = save_config(&config);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp_dir.path().join(".todorc.json").exists());
    }

    #[test]
    #[serial]
    fn test_save_config_updates_existing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Get absolute path before changing directories
        let temp_path = temp_dir.path().to_path_buf();

        // Create existing config file
        let existing_path = temp_path.join(".todorc.json");
        fs::write(&existing_path, r#"{"tags": ["OLD"]}"#).unwrap();

        // Change to temp directory
        std::env::set_current_dir(&temp_path).unwrap();

        let mut config = Config::new();
        config.tags = vec!["NEW".to_string()];
        let result = save_config(&config);

        // Restore original directory
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());

        // Verify the file was updated
        let loaded = Config::load_from_file(&existing_path).unwrap();
        assert_eq!(loaded.tags, vec!["NEW"]);
    }

    #[test]
    fn test_save_config_to_yaml_file() {
        // Test saving config directly to a YAML file (not via save_config)
        // This avoids directory change issues in parallel tests
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join(".todorc.yaml");

        let mut config = Config::new();
        config.tags = vec!["YAML_TEST".to_string()];
        config.save(&yaml_path).unwrap();

        // Verify the YAML file was created and can be loaded
        let loaded = Config::load_from_file(&yaml_path).unwrap();
        assert_eq!(loaded.tags, vec!["YAML_TEST"]);
    }

    #[test]
    fn test_create_test_project_structure() {
        let temp_dir = create_test_project();

        assert!(temp_dir.path().join("main.rs").exists());
        assert!(temp_dir.path().join("lib.rs").exists());
        assert!(temp_dir.path().join("src/utils.rs").exists());
    }

    #[test]
    fn test_cmd_scan_basic() {
        let temp_dir = create_test_project();

        let args = cli::ScanArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: false,
            flat: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            case_sensitive: false,
            sort: cli::SortOrder::File,
            group_by_tag: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_scan(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_scan_with_json_output() {
        let temp_dir = create_test_project();

        let args = cli::ScanArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: Some(vec!["TODO".to_string()]),
            include: Some(vec!["*.rs".to_string()]),
            exclude: None,
            json: true,
            flat: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            case_sensitive: true,
            sort: cli::SortOrder::Priority,
            group_by_tag: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_scan(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_scan_with_flat_output() {
        let temp_dir = create_test_project();

        let args = cli::ScanArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: Some(vec!["src/**".to_string()]),
            json: false,
            flat: true,
            depth: 1,
            follow_links: true,
            hidden: true,
            case_sensitive: false,
            sort: cli::SortOrder::Line,
            group_by_tag: false,
        };

        let global = cli::GlobalOptions {
            no_color: false,
            verbose: false,
            config: None,
        };

        let result = cmd_scan(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_scan_group_by_tag() {
        let temp_dir = create_test_project();

        let args = cli::ScanArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: false,
            flat: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            case_sensitive: false,
            sort: cli::SortOrder::File,
            group_by_tag: true,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_scan(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_scan_group_by_tag_with_color() {
        let temp_dir = create_test_project();

        let args = cli::ScanArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: false,
            flat: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            case_sensitive: false,
            sort: cli::SortOrder::File,
            group_by_tag: true,
        };

        let global = cli::GlobalOptions {
            no_color: false,
            verbose: false,
            config: None,
        };

        let result = cmd_scan(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_list_basic() {
        let temp_dir = create_test_project();

        let args = cli::ListArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: false,
            filter: None,
            case_sensitive: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_list(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_list_with_filter() {
        let temp_dir = create_test_project();

        let args = cli::ListArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: Some(vec!["TODO".to_string(), "FIXME".to_string()]),
            include: Some(vec!["*.rs".to_string()]),
            exclude: Some(vec!["src/**".to_string()]),
            json: false,
            filter: Some("TODO".to_string()),
            case_sensitive: true,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_list(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_list_with_json_output() {
        let temp_dir = create_test_project();

        let args = cli::ListArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: true,
            filter: None,
            case_sensitive: false,
        };

        let global = cli::GlobalOptions {
            no_color: false,
            verbose: false,
            config: None,
        };

        let result = cmd_list(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_display() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Create a config file
        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO", "FIXME"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: None,
            remove: None,
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_display_json() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO", "FIXME"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: true,
            add: None,
            remove: None,
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_display_with_color() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO", "FIXME", "BUG"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: None,
            remove: None,
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: false,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_add_new() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: Some("NEWTAG".to_string()),
            remove: None,
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_add_existing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: Some("todo".to_string()), // case-insensitive match
            remove: None,
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_remove_existing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO", "FIXME"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: None,
            remove: Some("TODO".to_string()),
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_remove_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: None,
            remove: Some("NONEXISTENT".to_string()),
            reset: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_tags_reset() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["CUSTOM"]}"#,
        )
        .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::TagsArgs {
            json: false,
            add: None,
            remove: None,
            reset: true,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_tags(args, &global);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_cmd_init_json() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::InitArgs {
            format: cli::ConfigFormat::Json,
            force: false,
        };

        let result = cmd_init(args);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp_dir.path().join(".todorc.json").exists());
    }

    #[test]
    #[serial]
    fn test_cmd_init_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::InitArgs {
            format: cli::ConfigFormat::Yaml,
            force: false,
        };

        let result = cmd_init(args);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp_dir.path().join(".todorc.yaml").exists());
    }

    #[test]
    #[serial]
    fn test_cmd_init_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Create existing config
        fs::write(temp_dir.path().join(".todorc.json"), "{}").unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::InitArgs {
            format: cli::ConfigFormat::Json,
            force: false,
        };

        let result = cmd_init(args);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_cmd_init_force_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Create existing config
        fs::write(temp_dir.path().join(".todorc.json"), r#"{"tags": ["OLD"]}"#).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = cli::InitArgs {
            format: cli::ConfigFormat::Json,
            force: true,
        };

        let result = cmd_init(args);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_basic() {
        let temp_dir = create_test_project();

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            json: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_with_json() {
        let temp_dir = create_test_project();

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: Some(vec!["TODO".to_string(), "FIXME".to_string()]),
            json: true,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_with_color() {
        let temp_dir = create_test_project();

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            json: false,
        };

        let global = cli::GlobalOptions {
            no_color: false,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_empty_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file with no TODOs
        fs::write(temp_dir.path().join("empty.rs"), "fn main() {}").unwrap();

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            json: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_empty_project_json() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file with no TODOs - tests the 0.0 edge case for items_per_file
        fs::write(temp_dir.path().join("empty.rs"), "fn main() {}").unwrap();

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            json: true,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_zero_percentage() {
        let temp_dir = TempDir::new().unwrap();

        // Create a completely empty directory (no files at all)
        // This tests the 0.0 percentage edge case

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: Some(vec!["NONEXISTENT".to_string()]),
            json: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_zero_percentage_with_color() {
        let temp_dir = TempDir::new().unwrap();

        // Create a completely empty directory (no files at all)
        // This tests the 0.0 percentage edge case with color output

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: Some(vec!["NONEXISTENT".to_string()]),
            json: false,
        };

        let global = cli::GlobalOptions {
            no_color: false,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_with_config() {
        let temp_dir = create_test_project();

        // Create config file
        fs::write(
            temp_dir.path().join(".todorc.json"),
            r#"{"tags": ["TODO", "FIXME", "NOTE"]}"#,
        )
        .unwrap();

        let args = cli::StatsArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            json: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: None,
        };

        let result = cmd_stats(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_scan_with_config_file() {
        let temp_dir = create_test_project();

        // Create config file
        let config_path = temp_dir.path().join("custom-config.json");
        fs::write(&config_path, r#"{"tags": ["TODO", "CUSTOM"]}"#).unwrap();

        let args = cli::ScanArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: false,
            flat: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            case_sensitive: false,
            sort: cli::SortOrder::File,
            group_by_tag: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: Some(config_path),
        };

        let result = cmd_scan(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_list_with_config_file() {
        let temp_dir = create_test_project();

        // Create config file
        let config_path = temp_dir.path().join("custom-config.json");
        fs::write(&config_path, r#"{"tags": ["TODO"]}"#).unwrap();

        let args = cli::ListArgs {
            path: Some(temp_dir.path().to_path_buf()),
            tags: None,
            include: None,
            exclude: None,
            json: false,
            filter: None,
            case_sensitive: false,
        };

        let global = cli::GlobalOptions {
            no_color: true,
            verbose: false,
            config: Some(config_path),
        };

        let result = cmd_list(args, &global);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_save_config_yaml_existing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Create existing YAML config file
        fs::write(temp_dir.path().join(".todorc.yaml"), "tags:\n  - OLD").unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut config = Config::new();
        config.tags = vec!["UPDATED".to_string()];
        let result = save_config(&config);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_save_config_yml_existing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Create existing YML config file
        fs::write(temp_dir.path().join(".todorc.yml"), "tags:\n  - OLD").unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut config = Config::new();
        config.tags = vec!["UPDATED".to_string()];
        let result = save_config(&config);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_save_config_todorc_existing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Create existing .todorc file (without extension)
        fs::write(temp_dir.path().join(".todorc"), r#"{"tags": ["OLD"]}"#).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut config = Config::new();
        config.tags = vec!["UPDATED".to_string()];
        let result = save_config(&config);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }
}
